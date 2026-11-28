// The 3D view. It draws what the core describes and handles orbit, pan and
// zoom. Z is up and the ground is the XY plane, as in the core.

import * as THREE from "three";
import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import type { PanelView, Scene } from "./api";
import { Grid } from "./grid";
import { onThemeChange, viewColors } from "./theme";

/** A direction to look from: degrees around the Z axis, and down from straight above. */
export interface ViewAngle {
  around: number;
  fromTop: number;
}

/** The three-quarter view the app opens in: from the front, a little to the right. */
export const HOME_VIEW: ViewAngle = { around: -55, fromTop: 58 };

const TURN_MS = 280;

export class Viewport {
  private renderer = new THREE.WebGLRenderer({ antialias: true });
  private scene = new THREE.Scene();
  private camera = new THREE.PerspectiveCamera(32, 1, 1, 10000);
  private controls: OrbitControls;
  private grid = new Grid();
  private sun = new THREE.DirectionalLight(0xffffff, 2.1);
  private shadowCatcher: THREE.Mesh<THREE.PlaneGeometry, THREE.ShadowMaterial>;
  private panels = new THREE.Group();
  private wood = new THREE.MeshLambertMaterial();
  private woodEdge = new THREE.LineBasicMaterial();
  private bounds = new THREE.Sphere(new THREE.Vector3(100, 60, 0), 120);
  private drawQueued = false;
  private turning = 0;
  private cameraListeners: ((camera: THREE.Camera) => void)[] = [];

  constructor(private host: HTMLElement) {
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    host.appendChild(this.renderer.domElement);

    this.camera.up.set(0, 0, 1);
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    // Mouse: left-drag orbits, right-drag or Shift-drag pans, the wheel zooms.
    // Touch: one finger orbits, two fingers pan and pinch to zoom.
    this.controls.zoomToCursor = true;
    this.controls.minDistance = 5;
    this.controls.maxDistance = 20000;
    this.controls.addEventListener("change", () => this.draw());

    this.scene.add(new THREE.HemisphereLight(0xffffff, 0xc4b094, 1.75));
    this.sun.castShadow = true;
    this.sun.shadow.mapSize.set(2048, 2048);
    this.sun.shadow.radius = 6;
    this.sun.shadow.bias = -0.0005;
    this.scene.add(this.sun, this.sun.target);

    this.shadowCatcher = new THREE.Mesh(
      new THREE.PlaneGeometry(1, 1),
      new THREE.ShadowMaterial({ depthWrite: false, polygonOffset: true, polygonOffsetFactor: 1, polygonOffsetUnits: 1 }),
    );
    this.shadowCatcher.receiveShadow = true;
    this.shadowCatcher.renderOrder = -1;

    this.scene.add(this.grid.mesh, this.shadowCatcher, this.panels);

    this.applyTheme();
    onThemeChange(() => this.applyTheme());
    new ResizeObserver(() => this.resize()).observe(host);
    this.resize();
    this.lookFrom(HOME_VIEW, false);
  }

  /** Replace what is drawn with a new description from the core. */
  show(scene: Scene, { frame = false } = {}) {
    for (const child of [...this.panels.children]) {
      this.panels.remove(child);
      child.traverse((part) => (part as Partial<THREE.Mesh>).geometry?.dispose());
    }
    for (const panel of scene.panels) {
      if (panel.outline.length >= 3) this.panels.add(this.buildPanel(panel));
    }

    const box = new THREE.Box3().setFromObject(this.panels);
    if (!box.isEmpty()) box.getBoundingSphere(this.bounds);
    this.aimSun();
    if (frame) this.lookFrom(HOME_VIEW, false);
    this.draw();
  }

  /** Turn the view to look from an angle, keeping the whole design in frame. */
  lookFrom(angle: ViewAngle, animate = true) {
    const fit = (this.bounds.radius / Math.sin(THREE.MathUtils.degToRad(this.camera.fov / 2))) * 1.25;
    const from = this.currentAngle();
    const start = { target: this.controls.target.clone(), distance: from.distance, around: from.around, fromTop: from.fromTop };
    // Go the short way round.
    const turn = THREE.MathUtils.euclideanModulo(angle.around - start.around + 180, 360) - 180;
    const began = performance.now();
    const id = ++this.turning;

    const step = (now: number) => {
      if (id !== this.turning) return;
      const t = animate ? Math.min((now - began) / TURN_MS, 1) : 1;
      const k = t * t * (3 - 2 * t);
      const around = THREE.MathUtils.degToRad(start.around + turn * k);
      // Straight up or down has no "which way is up", so stop a hair short.
      const fromTop = THREE.MathUtils.degToRad(
        THREE.MathUtils.clamp(THREE.MathUtils.lerp(start.fromTop, angle.fromTop, k), 0.05, 179.95),
      );
      const distance = THREE.MathUtils.lerp(start.distance, fit, k);
      this.controls.target.lerpVectors(start.target, this.bounds.center, k);
      this.camera.position.set(
        this.controls.target.x + distance * Math.sin(fromTop) * Math.cos(around),
        this.controls.target.y + distance * Math.sin(fromTop) * Math.sin(around),
        this.controls.target.z + distance * Math.cos(fromTop),
      );
      this.controls.update();
      this.draw();
      if (t < 1) requestAnimationFrame(step);
    };
    step(began);
  }

  /** Be told whenever the camera moves, and once straight away. */
  onCameraChange(listener: (camera: THREE.Camera) => void) {
    this.cameraListeners.push(listener);
    listener(this.camera);
  }

  private currentAngle() {
    const offset = this.camera.position.clone().sub(this.controls.target);
    const distance = offset.length() || 1;
    return {
      distance,
      around: THREE.MathUtils.radToDeg(Math.atan2(offset.y, offset.x)),
      fromTop: THREE.MathUtils.radToDeg(Math.acos(THREE.MathUtils.clamp(offset.z / distance, -1, 1))),
    };
  }

  /** A panel as the core describes it: its outline, extruded to its thickness. */
  private buildPanel(panel: PanelView): THREE.Object3D {
    const shape = new THREE.Shape(panel.outline.map(([x, y]) => new THREE.Vector2(x, y)));
    for (const hole of panel.holes) {
      shape.holes.push(new THREE.Path(hole.map(([x, y]) => new THREE.Vector2(x, y))));
    }
    const solid = new THREE.ExtrudeGeometry(shape, { depth: panel.thickness, bevelEnabled: false });

    const body = new THREE.Mesh(solid, this.wood);
    body.castShadow = true;
    const edges = new THREE.LineSegments(new THREE.EdgesGeometry(solid, 20), this.woodEdge);

    const group = new THREE.Group();
    group.name = panel.name;
    group.userData.panelId = panel.id;
    group.add(body, edges);
    // The outline's x, y and thickness direction, placed in space.
    group.matrix
      .makeBasis(
        new THREE.Vector3(...panel.x_axis),
        new THREE.Vector3(...panel.y_axis),
        new THREE.Vector3(...panel.normal),
      )
      .setPosition(...panel.origin);
    group.matrixAutoUpdate = false;
    group.updateMatrixWorld(true);
    return group;
  }

  /** Point the light at the design and size its shadow to cover it. */
  private aimSun() {
    const { center, radius } = this.bounds;
    const reach = radius * 1.5;
    this.sun.target.position.copy(center);
    this.sun.position.set(center.x - radius * 1.2, center.y - radius * 1.6, center.z + radius * 3.2);
    const frustum = this.sun.shadow.camera;
    frustum.left = frustum.bottom = -reach;
    frustum.right = frustum.top = reach;
    frustum.near = radius * 0.1;
    frustum.far = radius * 8;
    frustum.updateProjectionMatrix();
    this.shadowCatcher.position.set(center.x, center.y, 0);
    this.shadowCatcher.scale.set(reach * 4, reach * 4, 1);
  }

  private applyTheme() {
    const colors = viewColors();
    this.renderer.setClearColor(colors.workArea);
    this.grid.setColors(colors.gridMinor, colors.gridMajor);
    this.wood.color.copy(colors.wood);
    this.woodEdge.color.copy(colors.woodEdge);
    this.shadowCatcher.material.opacity = colors.groundShadow;
    this.draw();
  }

  private resize() {
    const { clientWidth: width, clientHeight: height } = this.host;
    if (!width || !height) return;
    this.renderer.setSize(width, height);
    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.draw();
  }

  /** Draw on the next frame. The view only redraws when something changed. */
  private draw() {
    if (this.drawQueued) return;
    this.drawQueued = true;
    requestAnimationFrame(() => {
      this.drawQueued = false;
      const distance = this.camera.position.distanceTo(this.controls.target);
      this.camera.near = Math.max(distance / 200, 0.1);
      this.camera.far = distance * 200 + this.bounds.radius * 4;
      this.camera.updateProjectionMatrix();
      this.grid.follow(this.controls.target, this.camera.far * 2);
      this.renderer.render(this.scene, this.camera);
      for (const listener of this.cameraListeners) listener(this.camera);
    });
  }
}

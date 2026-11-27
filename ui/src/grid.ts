// The ground grid: lines on the XY plane that run to the edge of the view.
// Every fourth line is heavier. Zooming out fades the finest lines away and the
// next size up takes over, so the grid never turns into a solid wash.

import * as THREE from "three";

/** Distance between the finest lines, in millimeters. */
const FINEST_SPACING = 10;

const vertexShader = /* glsl */ `
  varying vec2 vGround;
  void main() {
    vec4 world = modelMatrix * vec4(position, 1.0);
    vGround = world.xy;
    gl_Position = projectionMatrix * viewMatrix * world;
  }
`;

const fragmentShader = /* glsl */ `
  uniform vec3 uMinor;
  uniform vec3 uMajor;
  uniform float uSpacing;
  varying vec2 vGround;

  // How visible lines 'spacing' apart are: gone when they crowd together.
  vec2 presence(vec2 cellsPerPixel) {
    return 1.0 - smoothstep(0.06, 0.25, cellsPerPixel);
  }

  void main() {
    vec3 color = uMinor;
    float alpha = 0.0;
    float spacing = uSpacing;
    vec2 finer = vec2(0.0);           // presence of the level below this one
    for (int level = 0; level < 5; level++) {
      vec2 cell = vGround / spacing;
      vec2 cellsPerPixel = fwidth(cell);
      vec2 here = presence(cellsPerPixel);
      // A line is heavy while finer lines are still showing between its kind.
      vec2 heavy = finer;
      vec2 halfWidth = mix(vec2(0.4), vec2(0.7), heavy);
      vec2 pixelsFromLine = abs(fract(cell - 0.5) - 0.5) / max(cellsPerPixel, vec2(1e-6));
      vec2 line = (1.0 - smoothstep(halfWidth - 0.5, halfWidth + 0.5, pixelsFromLine)) * here;
      for (int axis = 0; axis < 2; axis++) {
        float a = axis == 0 ? line.x : line.y;
        float h = axis == 0 ? heavy.x : heavy.y;
        color = mix(color, mix(uMinor, uMajor, h), a);
        alpha = max(alpha, a);
      }
      finer = here;
      spacing *= 4.0;
    }
    if (alpha < 0.003) discard;
    gl_FragColor = vec4(color, alpha);
    #include <colorspace_fragment>
  }
`;

export class Grid {
  readonly mesh: THREE.Mesh;
  private material: THREE.ShaderMaterial;

  constructor() {
    this.material = new THREE.ShaderMaterial({
      vertexShader,
      fragmentShader,
      uniforms: {
        uMinor: { value: new THREE.Color() },
        uMajor: { value: new THREE.Color() },
        uSpacing: { value: FINEST_SPACING },
      },
      transparent: true,
      depthWrite: false,
      side: THREE.DoubleSide,
      // Sit a hair behind anything lying on the ground, so the two never flicker.
      polygonOffset: true,
      polygonOffsetFactor: 2,
      polygonOffsetUnits: 2,
    });
    this.mesh = new THREE.Mesh(new THREE.PlaneGeometry(1, 1), this.material);
    this.mesh.renderOrder = -2;
    this.mesh.frustumCulled = false;
  }

  setColors(minor: THREE.Color, major: THREE.Color) {
    this.material.uniforms.uMinor.value.copy(minor);
    this.material.uniforms.uMajor.value.copy(major);
  }

  /** Keep the grid under the camera and big enough to reach the horizon. */
  follow(center: THREE.Vector3, reach: number) {
    this.mesh.position.set(center.x, center.y, 0);
    this.mesh.scale.set(reach, reach, 1);
    this.mesh.updateMatrixWorld();
  }
}

// The view cube: a small cube that turns with the model to show which way it
// faces. Each face is a button that turns the view to that side.

import * as THREE from "three";
import { HOME_VIEW, type ViewAngle } from "./viewport";

interface Face {
  label: string;
  /** Which way the face points, and its right and up as you look at it. */
  normal: [number, number, number];
  right: [number, number, number];
  up: [number, number, number];
  view: ViewAngle;
}

const FACES: Face[] = [
  { label: "Top", normal: [0, 0, 1], right: [1, 0, 0], up: [0, 1, 0], view: { around: -90, fromTop: 0 } },
  { label: "Bottom", normal: [0, 0, -1], right: [1, 0, 0], up: [0, -1, 0], view: { around: -90, fromTop: 180 } },
  { label: "Front", normal: [0, -1, 0], right: [1, 0, 0], up: [0, 0, 1], view: { around: -90, fromTop: 90 } },
  { label: "Back", normal: [0, 1, 0], right: [-1, 0, 0], up: [0, 0, 1], view: { around: 90, fromTop: 90 } },
  { label: "Right", normal: [1, 0, 0], right: [0, 1, 0], up: [0, 0, 1], view: { around: 0, fromTop: 90 } },
  { label: "Left", normal: [-1, 0, 0], right: [0, -1, 0], up: [0, 0, 1], view: { around: 180, fromTop: 90 } },
];

const HALF = 28; // half the cube's side, in px; matches .view-cube-face

export class ViewCube {
  private faces: { face: Face; button: HTMLButtonElement }[] = [];

  constructor(host: HTMLElement, lookFrom: (view: ViewAngle) => void) {
    const root = document.createElement("div");
    root.className = "view-cube";
    const stage = document.createElement("div");
    stage.className = "view-cube-stage";
    root.appendChild(stage);

    for (const face of FACES) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "view-cube-face";
      button.textContent = face.label;
      button.setAttribute("aria-label", `Look at the ${face.label.toLowerCase()}`);
      button.addEventListener("click", () => lookFrom(face.view));
      stage.appendChild(button);
      this.faces.push({ face, button });
    }

    const home = document.createElement("button");
    home.type = "button";
    home.className = "view-cube-home";
    home.innerHTML =
      '<svg viewBox="0 0 16 16" width="16" height="16" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M2.5 7.5 8 2.5l5.5 5M4 6.5v6.5h8V6.5"/></svg>Home';
    home.setAttribute("aria-label", "Home view: show the whole design");
    home.addEventListener("click", () => lookFrom(HOME_VIEW));
    root.appendChild(home);

    host.appendChild(root);
  }

  /** Turn the cube to match the camera. */
  update(camera: THREE.Camera) {
    // World directions as the camera sees them: x right, y up, z toward the viewer.
    const view = new THREE.Matrix3().setFromMatrix4(camera.matrixWorldInverse);
    const seen = (v: [number, number, number]) => new THREE.Vector3(...v).applyMatrix3(view);

    for (const { face, button } of this.faces) {
      const n = seen(face.normal);
      // Faces turned away are hidden, so they can be neither seen nor pressed.
      const facing = n.z > 0.08;
      button.style.visibility = facing ? "visible" : "hidden";
      if (!facing) continue;
      const r = seen(face.right);
      const u = seen(face.up);
      // CSS y runs down the screen, so every y is flipped, and the button's own
      // downward y axis is the face's "up" reversed.
      const m = [
        r.x, -r.y, r.z, 0,
        -u.x, u.y, -u.z, 0,
        n.x, -n.y, n.z, 0,
        n.x * HALF, -n.y * HALF, n.z * HALF, 1,
      ];
      button.style.transform = `matrix3d(${m.map((v) => v.toFixed(5)).join(",")})`;
      button.style.zIndex = String(Math.round(n.z * 10));
    }
  }
}

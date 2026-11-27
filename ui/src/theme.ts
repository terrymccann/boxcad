// Colors for the 3D view, read from the same CSS tokens the rest of the
// interface uses, so light and dark stay in step.

import * as THREE from "three";

export interface ViewColors {
  workArea: THREE.Color;
  gridMinor: THREE.Color;
  gridMajor: THREE.Color;
  wood: THREE.Color;
  woodEdge: THREE.Color;
  groundShadow: number;
}

export function viewColors(): ViewColors {
  const style = getComputedStyle(document.documentElement);
  const token = (name: string) => style.getPropertyValue(name).trim();
  const color = (name: string) => new THREE.Color(token(name));
  return {
    workArea: color("--work-area"),
    gridMinor: color("--grid-minor"),
    gridMajor: color("--grid-major"),
    wood: color("--wood"),
    woodEdge: color("--wood-edge"),
    groundShadow: Number(token("--ground-shadow")),
  };
}

/** Call `listener` whenever the system switches between light and dark. */
export function onThemeChange(listener: () => void) {
  matchMedia("(prefers-color-scheme: dark)").addEventListener("change", listener);
}

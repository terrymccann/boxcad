// Everything the interface asks of the core goes through this file.

import { invoke } from "@tauri-apps/api/core";

export type Vec2 = [number, number];
export type Vec3 = [number, number, number];

/** One panel as the core describes it for drawing. Lengths are millimeters. */
export interface PanelView {
  id: number;
  name: string;
  material: string;
  thickness: number;
  /** A point on the face the thickness grows from. */
  origin: Vec3;
  x_axis: Vec3;
  y_axis: Vec3;
  /** The direction the thickness grows in. */
  normal: Vec3;
  /** Outside edge, counter-clockwise, in the panel's own x and y. */
  outline: Vec2[];
  holes: Vec2[][];
  /** Why the panel cannot be drawn, if it cannot. */
  problem: string | null;
}

export interface Scene {
  panels: PanelView[];
}

/** False when the interface is open in a plain browser, with no core behind it. */
export const inApp = "__TAURI_INTERNALS__" in window;

/** What the 3D view should draw for the open design. */
export function loadScene(): Promise<Scene> {
  return inApp ? invoke<Scene>("scene") : Promise.resolve(BROWSER_PREVIEW);
}

// Stands in for the core when the interface is opened in a browser to work on
// its looks. The app never uses it.
const BROWSER_PREVIEW: Scene = {
  panels: [
    {
      id: 7,
      name: "Panel",
      material: "Birch ply",
      thickness: 3,
      origin: [0, 0, 0],
      x_axis: [1, 0, 0],
      y_axis: [0, 1, 0],
      normal: [0, 0, 1],
      outline: [[0, 0], [200, 0], [200, 120], [0, 120]],
      holes: [],
      problem: null,
    },
  ],
};

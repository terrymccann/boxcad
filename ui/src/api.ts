// Everything the interface asks of the core goes through this file. Each
// request answers with a fresh snapshot of the open design.

import { invoke } from "@tauri-apps/api/core";
import { browserPreview } from "./preview";

export type Vec2 = [number, number];
export type Vec3 = [number, number, number];
export type Units = "mm" | "in";

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
  /** Overall width and depth of the outline. */
  size: Vec2;
  /** Why the panel cannot be drawn, if it cannot. */
  problem: string | null;
}

export interface Scene {
  panels: PanelView[];
}

export interface Material {
  id: number;
  name: string;
  thickness: number;
  kerf: number;
  sheet: { width: number; height: number };
  grain: "none" | "along_width" | "along_height";
}

/** Everything the interface shows for the open design. */
export interface Snapshot {
  name: string;
  modified: boolean;
  /** What Undo and Redo would act on, if anything. */
  undo: string | null;
  redo: string | null;
  display_units: Units;
  materials: Material[];
  scene: Scene;
}

export interface Api {
  snapshot(): Promise<Snapshot>;
  undo(): Promise<Snapshot>;
  redo(): Promise<Snapshot>;
  /** Thickness and kerf in millimeters. Rejects with the reason if refused. */
  setMaterialNumbers(id: number, thickness: number, kerf: number): Promise<Snapshot>;
  setDisplayUnits(units: Units): Promise<Snapshot>;
}

const shell: Api = {
  snapshot: () => invoke("snapshot"),
  undo: () => invoke("undo"),
  redo: () => invoke("redo"),
  setMaterialNumbers: (id, thickness, kerf) => invoke("set_material_numbers", { id, thickness, kerf }),
  setDisplayUnits: (units) => invoke("set_display_units", { units }),
};

/** False when the interface is open in a plain browser, with no core behind it. */
export const inApp = "__TAURI_INTERNALS__" in window;

export const api: Api = inApp ? shell : browserPreview();

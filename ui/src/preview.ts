// Stands in for the core when the interface is opened in a plain browser to
// work on its looks. The app never uses it, and it knows no geometry: it only
// hands back a fixed panel with whatever numbers were last set.

import type { Api, Snapshot } from "./api";

export function browserPreview(): Api {
  const first: Snapshot = {
    name: "Sample panel",
    modified: false,
    undo: null,
    redo: null,
    display_units: "mm",
    materials: [
      { id: 1, name: "Birch ply", thickness: 3, kerf: 0.15, sheet: { width: 600, height: 400 }, grain: "none" },
    ],
    scene: {
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
          size: [200, 120],
          problem: null,
        },
      ],
    },
  };

  let states = [structuredClone(first)];
  let labels: string[] = [];
  let at = 0;
  const now = (): Snapshot => ({
    ...structuredClone(states[at]),
    modified: at !== 0,
    undo: at > 0 ? labels[at - 1] : null,
    redo: at < labels.length ? labels[at] : null,
  });
  const change = (label: string, edit: (s: Snapshot) => void) => {
    const next = structuredClone(states[at]);
    edit(next);
    states = [...states.slice(0, at + 1), next];
    labels = [...labels.slice(0, at), label];
    at += 1;
  };

  return {
    snapshot: async () => now(),
    undo: async () => {
      at = Math.max(0, at - 1);
      return now();
    },
    redo: async () => {
      at = Math.min(labels.length, at + 1);
      return now();
    },
    setMaterialNumbers: async (_id, thickness, kerf) => {
      if (!(thickness > 0)) {
        throw `the thickness of material "Birch ply" must be greater than zero, but is ${thickness}`;
      }
      if (!(kerf >= 0)) throw `the kerf of material "Birch ply" must be zero or more, but is ${kerf}`;
      change("Change material", (s) => {
        s.materials[0].thickness = thickness;
        s.materials[0].kerf = kerf;
        s.scene.panels[0].thickness = thickness;
      });
      return now();
    },
    setDisplayUnits: async (units) => {
      change("Change units", (s) => (s.display_units = units));
      return now();
    },
  };
}

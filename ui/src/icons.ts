// Icons, drawn as strokes on a 24-unit grid so they take the color of their
// button. The shapes come from the approved mockups.

const PATHS = {
  // Tool groups
  create: "M12 3l8 4.5v9L12 21l-8-4.5v-9zM4 7.5 12 12l8-4.5M12 12v9",
  sketch: "M4 20l1-4L16 5l3 3L8 19zM14 7l3 3",
  modify: "M12 3v18M3 12h18M9 6l3-3 3 3M9 18l3 3 3-3M6 9l-3 3 3 3M18 9l3 3-3 3",
  joints: "M3 6h6v4h6V6h6v12h-6v-4H9v4H3z",
  decorate: "M12 3l4 4-4 4-4-4zM6 11l4 4-4 4-4-4zM18 11l4 4-4 4-4-4z",
  measure: "M3 9h18v6H3zM7 9v3M11 9v3M15 9v3M19 9v3",
  explode: "M5 14h14v5H5zM7 5h10v4H7zM9 11v1M15 11v1",
  // Create
  box: "M12 3l8 4.5v9L12 21l-8-4.5v-9zM4 7.5 12 12l8-4.5M12 12v9",
  polygonBox: "M12 3l8 5.8-3 9.4H7L4 8.8z",
  floor: "M3 15l9-5 9 5-9 5z",
  walls: "M4 8v9l8 4 8-4V8M4 8l8 4 8-4",
  divider: "M4 6h16v12H4zM12 6v12",
  panel: "M5 5h14v14H5z",
  lid: "M4 13h16v6H4zM6 6h12v3H6z",
  // Sketch
  line: "M5 19 19 5",
  arc: "M4 18a14 14 0 0 1 16-12",
  circle: "M4 12a8 8 0 1 0 16 0 8 8 0 1 0-16 0",
  rectangle: "M4 7h16v10H4z",
  polygon: "M12 4l7.6 5.5-2.9 9H7.3l-2.9-9z",
  slot: "M8 8h8a4 4 0 0 1 0 8H8a4 4 0 0 1 0-8z",
  // Modify
  pushPull: "M5 15h14v5H5zM12 12V3M8 7l4-4 4 4",
  move: "M12 3v18M3 12h18M9 6l3-3 3 3M9 18l3 3 3-3M6 9l-3 3 3 3M18 9l3 3-3 3",
  roundCorner: "M5 20v-8a7 7 0 0 1 7-7h8",
  profileEdge: "M4 15c4 0 4-8 8-8s4 8 8 8M4 20h16",
  // Decorate
  cutout: "M6 9h12a3 3 0 0 1 0 6H6a3 3 0 0 1 0-6z",
  pattern:
    "M5 5h3v3H5zM10.5 5h3v3h-3zM16 5h3v3h-3zM5 10.5h3v3H5zM10.5 10.5h3v3h-3zM16 10.5h3v3h-3zM5 16h3v3H5zM10.5 16h3v3h-3zM16 16h3v3h-3z",
  artwork: "M4 5h16v14H4zM4 16l5-5 4 4 3-3 4 4",
  text: "M6 6h12M12 6v13M9 19h6",
  // Action pill
  undo: "M8 5 3 10l5 5M3 10h11a5.5 5.5 0 0 1 0 11h-3",
  redo: "M16 5l5 5-5 5M21 10H10a5.5 5.5 0 0 0 0 11h3",
  parts: "M12 3 3 8l9 5 9-5zM3 13l9 5 9-5",
  parameters: "M4 7h8M16 7h4M4 17h3M11 17h9M12 7a2 2 0 1 0 4 0 2 2 0 1 0-4 0M7 17a2 2 0 1 0 4 0 2 2 0 1 0-4 0",
  flatParts: "M4 5h16v14H4zM7.5 8.5h5v4h-5zM15 9.5h2M15 12.5h2M7.5 15.5h9",
  search: "M5 11a6 6 0 1 0 12 0 6 6 0 1 0-12 0M20 20l-4.6-4.6",
  export: "M12 15V4M8 8l4-4 4 4M5 14v5h14v-5",
  close: "M6 6l12 12M18 6 6 18",
} as const;

export type IconName = keyof typeof PATHS;

/** An icon as SVG markup. It is decoration: the button it sits in carries the name. */
export function icon(name: IconName, size = 22): string {
  return `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="${PATHS[name]}"/></svg>`;
}

/** The Box CAD mark shown in the project pill. */
export const LOGO =
  '<svg width="24" height="24" viewBox="0 0 22 22" aria-hidden="true"><path d="M11 2 20 6.5v9L11 20l-9-4.5v-9z" fill="#d9b382" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/><path d="M2 6.5 11 11l9-4.5M11 11v9" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linejoin="round"/></svg>';

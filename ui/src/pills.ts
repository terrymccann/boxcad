// The pills: the project name at top left, the actions at top right, and the
// size readout at bottom center.

import type { Snapshot } from "./api";
import { icon, LOGO, type IconName } from "./icons";
import { formatSize } from "./units";

export interface PillActions {
  undo(): void;
  redo(): void;
  toggleMaterial(): void;
}

export class Pills {
  private name = document.createElement("span");
  private state = document.createElement("span");
  private undo: HTMLButtonElement;
  private redo: HTMLButtonElement;
  private readout = document.createElement("div");
  private material = document.createElement("button");
  private sizes = document.createElement("span");

  constructor(host: HTMLElement, actions: PillActions) {
    const project = document.createElement("div");
    project.className = "pill project-pill";
    this.name.className = "project-name";
    this.state.className = "project-state";
    project.insertAdjacentHTML("afterbegin", LOGO);
    project.append(this.name, this.state);

    const bar = document.createElement("div");
    bar.className = "pill action-pill";
    bar.setAttribute("role", "toolbar");
    bar.setAttribute("aria-label", "Actions");
    this.undo = action("Undo", "undo", actions.undo);
    this.redo = action("Redo", "redo", actions.redo);
    const exportButton = document.createElement("button");
    exportButton.type = "button";
    exportButton.className = "export";
    exportButton.innerHTML = `${icon("export", 18)}Export`;
    exportButton.disabled = true;
    bar.append(
      this.undo,
      this.redo,
      // These open cards that are not built yet.
      action("Parts", "parts"),
      action("Parameters", "parameters"),
      action("Flat parts", "flatParts"),
      action("Search commands", "search"),
      exportButton,
    );

    this.readout.className = "pill readout-pill";
    this.material.type = "button";
    this.material.className = "readout-material";
    this.material.setAttribute("aria-expanded", "false");
    this.material.addEventListener("click", actions.toggleMaterial);
    this.readout.append(this.sizes, this.material);

    host.append(project, bar, this.readout);
  }

  update(snapshot: Snapshot) {
    this.name.textContent = snapshot.name;
    this.state.textContent = snapshot.modified ? "Edited" : "No changes";

    this.undo.disabled = !snapshot.undo;
    this.undo.setAttribute("aria-label", snapshot.undo ? `Undo: ${snapshot.undo}` : "Undo");
    this.redo.disabled = !snapshot.redo;
    this.redo.setAttribute("aria-label", snapshot.redo ? `Redo: ${snapshot.redo}` : "Redo");

    const units = snapshot.display_units;
    this.sizes.replaceChildren(
      ...snapshot.scene.panels.map((p) => entry(p.name, formatSize([p.size[0], p.size[1], p.thickness], units))),
    );
    const material = snapshot.materials[0];
    this.material.hidden = !material;
    if (material) this.material.replaceChildren(...entry("Material", material.name).childNodes);
    this.readout.hidden = !material && !snapshot.scene.panels.length;
  }

  /** Show whether the material card is open. */
  setMaterialOpen(open: boolean) {
    this.material.setAttribute("aria-expanded", String(open));
  }
}

function action(label: string, name: IconName, press?: () => void): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "action";
  button.setAttribute("aria-label", label);
  button.innerHTML = icon(name);
  if (press) button.addEventListener("click", press);
  else button.disabled = true;
  return button;
}

/** A dim label followed by its value, as in "Material Birch ply". */
function entry(label: string, value: string): HTMLElement {
  const item = document.createElement("span");
  item.className = "readout-entry";
  const dim = document.createElement("span");
  dim.className = "readout-label";
  dim.textContent = label;
  const strong = document.createElement("span");
  strong.className = "readout-value";
  strong.textContent = value;
  item.append(dim, " ", strong);
  return item;
}

// The material card: the sheet's measured thickness and the laser's kerf, as
// exact numbers, and the units lengths are shown in. It opens from the readout
// pill and closes when it is no longer needed.

import type { Material, Snapshot, Units } from "./api";
import { float, grip, type Floating } from "./floating";
import { icon } from "./icons";
import { formatLength, parseLength } from "./units";

export interface MaterialActions {
  /** Thickness and kerf in millimeters. Rejects with the reason if refused. */
  setNumbers(id: number, thickness: number, kerf: number): Promise<void>;
  setUnits(units: Units): Promise<void>;
  closed(): void;
}

export class MaterialCard {
  private root = document.createElement("section");
  private name = document.createElement("span");
  private thickness = numberField("Thickness");
  private kerf = numberField("Kerf");
  private unitButtons = new Map<Units, HTMLButtonElement>();
  private problem = document.createElement("div");
  private floating: Floating;
  private material: Material | null = null;
  private units: Units = "mm";

  constructor(host: HTMLElement, private actions: MaterialActions) {
    this.root.className = "card floating surface";
    this.root.setAttribute("aria-label", "Material settings");
    this.root.hidden = true;

    const handle = grip();
    const header = document.createElement("div");
    header.className = "card-header";
    const title = document.createElement("h2");
    title.textContent = "Material";
    const close = document.createElement("button");
    close.type = "button";
    close.className = "card-close";
    close.setAttribute("aria-label", "Close material settings");
    close.innerHTML = icon("close", 18);
    close.addEventListener("click", () => this.setOpen(false));
    header.append(title, close);

    const nameRow = document.createElement("div");
    nameRow.className = "field";
    const nameLabel = document.createElement("span");
    nameLabel.className = "field-label";
    nameLabel.textContent = "Name";
    this.name.className = "field-static";
    nameRow.append(nameLabel, this.name);

    const unitsRow = document.createElement("div");
    unitsRow.className = "segmented";
    unitsRow.setAttribute("role", "group");
    unitsRow.setAttribute("aria-label", "Units");
    for (const [units, label] of [["mm", "Millimeters"], ["in", "Inches"]] as const) {
      const button = document.createElement("button");
      button.type = "button";
      button.textContent = label;
      button.addEventListener("click", () => void this.run(() => this.actions.setUnits(units)));
      this.unitButtons.set(units, button);
      unitsRow.appendChild(button);
    }

    this.problem.className = "note problem";
    this.problem.setAttribute("role", "alert");
    this.problem.hidden = true;

    for (const field of [this.thickness, this.kerf]) {
      field.input.addEventListener("change", () => this.commit());
      field.input.addEventListener("keydown", (e) => {
        if (e.key === "Enter") field.input.blur();
        if (e.key === "Escape") {
          this.fill();
          field.input.blur();
        }
      });
      field.input.addEventListener("focus", () => field.input.select());
    }

    this.root.append(handle, header, nameRow, this.thickness.row, this.kerf.row, unitsRow, this.problem);
    host.appendChild(this.root);
    // Its home is the right edge, below the view cube.
    this.floating = float(host, this.root, handle, "material-card", (area, self) => ({
      left: area.width - self.width - 12,
      top: 250,
    }));
  }

  get open(): boolean {
    return !this.root.hidden;
  }

  setOpen(open: boolean) {
    if (open === this.open) return;
    this.root.hidden = !open;
    if (open) {
      this.fill();
      this.floating.settle();
      this.thickness.input.focus();
    } else {
      this.actions.closed();
    }
  }

  update(snapshot: Snapshot) {
    this.material = snapshot.materials[0] ?? null;
    this.units = snapshot.display_units;
    if (!this.material) this.setOpen(false);
    this.fill();
  }

  /** Show the design's current values, dropping anything half-typed. */
  private fill() {
    const material = this.material;
    if (!material) return;
    this.name.textContent = material.name;
    this.thickness.input.value = formatLength(material.thickness, this.units, false);
    this.kerf.input.value = formatLength(material.kerf, this.units, false);
    this.thickness.unit.textContent = this.kerf.unit.textContent = this.units;
    for (const [units, button] of this.unitButtons) {
      button.setAttribute("aria-pressed", String(units === this.units));
    }
    this.fail(null);
  }

  /** Send what was typed to the core. */
  private commit() {
    const material = this.material;
    if (!material) return;
    const thickness = parseLength(this.thickness.input.value, this.units);
    const kerf = parseLength(this.kerf.input.value, this.units);
    if (thickness === null || kerf === null) {
      const which = thickness === null ? "Thickness" : "Kerf";
      this.fail(`${which} needs a number, such as 3, 2.8 or 1/8.`);
      return;
    }
    void this.run(() => this.actions.setNumbers(material.id, thickness, kerf));
  }

  private async run(change: () => Promise<void>) {
    try {
      await change();
    } catch (reason) {
      const text = String(reason);
      this.fail(text.charAt(0).toUpperCase() + text.slice(1) + ".");
    }
  }

  private fail(message: string | null) {
    this.problem.textContent = message ?? "";
    this.problem.hidden = !message;
  }
}

function numberField(label: string) {
  const row = document.createElement("label");
  row.className = "field";
  const name = document.createElement("span");
  name.className = "field-label";
  name.textContent = label;
  const box = document.createElement("span");
  box.className = "field-box";
  const input = document.createElement("input");
  input.type = "text";
  input.inputMode = "decimal";
  input.autocomplete = "off";
  input.spellcheck = false;
  const unit = document.createElement("span");
  unit.className = "field-unit";
  box.append(input, unit);
  row.append(name, box);
  return { row, input, unit };
}

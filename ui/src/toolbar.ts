// The tool bar: tool groups down the left edge. Pressing a group opens a
// fly-out of its tools beside it. A tool that is not built yet is shown dimmed.

import { float, grip } from "./floating";
import { icon, type IconName } from "./icons";

interface Tool {
  id: string;
  label: string;
  icon: IconName;
  /** Whether the tool works yet. */
  ready: boolean;
}

interface Group {
  id: string;
  label: string;
  icon: IconName;
  /** Tools in the fly-out. A group with none is a single tool in itself. */
  tools: Tool[];
}

const soon = (id: string, label: string, name: IconName): Tool => ({ id, label, icon: name, ready: false });

const GROUPS: Group[] = [
  {
    id: "create",
    label: "Create",
    icon: "create",
    tools: [
      soon("box", "Box", "box"),
      soon("polygon-box", "Polygon box", "polygonBox"),
      soon("floor", "Floor", "floor"),
      soon("walls", "Walls", "walls"),
      soon("divider", "Divider", "divider"),
      soon("panel", "Panel", "panel"),
      soon("lid", "Lid", "lid"),
    ],
  },
  {
    id: "sketch",
    label: "Sketch",
    icon: "sketch",
    tools: [
      soon("line", "Line", "line"),
      soon("arc", "Arc", "arc"),
      soon("circle", "Circle", "circle"),
      soon("rectangle", "Rectangle", "rectangle"),
      soon("polygon", "Polygon", "polygon"),
      soon("slot", "Slot", "slot"),
    ],
  },
  {
    id: "modify",
    label: "Modify",
    icon: "modify",
    tools: [
      soon("push-pull", "Push/pull", "pushPull"),
      soon("move", "Move", "move"),
      soon("round-corner", "Round corner", "roundCorner"),
      soon("profile-edge", "Profile edge", "profileEdge"),
    ],
  },
  { id: "joints", label: "Joints", icon: "joints", tools: [soon("joint", "Joint", "joints")] },
  {
    id: "decorate",
    label: "Decorate",
    icon: "decorate",
    tools: [
      soon("cutout", "Cutout", "cutout"),
      soon("pattern", "Pattern", "pattern"),
      soon("artwork", "Artwork", "artwork"),
      soon("text", "Text", "text"),
    ],
  },
  { id: "measure", label: "Measure", icon: "measure", tools: [] },
  { id: "explode", label: "Explode", icon: "explode", tools: [] },
];

export class Toolbar {
  private root = document.createElement("div");
  private flyout = document.createElement("div");
  private buttons = new Map<string, HTMLButtonElement>();
  private openGroup: string | null = null;

  constructor(host: HTMLElement) {
    this.root.className = "toolbar floating";
    const handle = grip();
    const nav = document.createElement("nav");
    nav.setAttribute("aria-label", "Tools");

    for (const group of GROUPS) {
      const button = toolButton(group.label, group.icon);
      if (group.tools.length) {
        button.setAttribute("aria-expanded", "false");
        button.addEventListener("click", () => this.toggle(group.id));
      } else {
        // A single tool with no fly-out, not built yet.
        button.disabled = true;
      }
      this.buttons.set(group.id, button);
      nav.appendChild(button);
    }

    this.flyout.className = "flyout surface";
    this.flyout.hidden = true;
    this.root.append(handle, nav, this.flyout);
    host.appendChild(this.root);
    float(host, this.root, handle, "toolbar", () => ({ left: 12, top: 66 }));
  }

  private toggle(id: string) {
    this.openGroup = this.openGroup === id ? null : id;
    for (const [groupId, button] of this.buttons) {
      if (button.hasAttribute("aria-expanded")) {
        button.setAttribute("aria-expanded", String(groupId === this.openGroup));
      }
    }
    const group = GROUPS.find((g) => g.id === this.openGroup);
    this.flyout.hidden = !group;
    if (!group) return;

    this.flyout.setAttribute("role", "group");
    this.flyout.setAttribute("aria-label", `${group.label} tools`);
    this.flyout.replaceChildren(
      ...group.tools.map((tool) => {
        const button = toolButton(tool.label, tool.icon);
        button.disabled = !tool.ready;
        return button;
      }),
    );
    // Line the fly-out up with its group's button, on whichever side has room.
    const button = this.buttons.get(group.id)!;
    this.flyout.style.top = `${button.offsetTop - 5}px`;
    this.flyout.classList.remove("to-left");
    const overflow = this.flyout.getBoundingClientRect().right > window.innerWidth - 6;
    this.flyout.classList.toggle("to-left", overflow);
  }
}

function toolButton(label: string, name: IconName): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "tool";
  button.innerHTML = `${icon(name)}<span>${label}</span>`;
  return button;
}

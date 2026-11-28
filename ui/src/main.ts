import "@fontsource-variable/inter";
import { api, type Snapshot } from "./api";
import { MaterialCard } from "./material-card";
import { Pills } from "./pills";
import { Toolbar } from "./toolbar";
import { ViewCube } from "./viewcube";
import { Viewport } from "./viewport";

const workspace = document.getElementById("workspace")!;
const viewport = new Viewport(document.getElementById("viewport")!);
const cube = new ViewCube(workspace, (view) => viewport.lookFrom(view));
viewport.onCameraChange((camera) => cube.update(camera));

new Toolbar(workspace);
const pills = new Pills(workspace, {
  undo: () => void change(api.undo()),
  redo: () => void change(api.redo()),
  toggleMaterial: () => {
    card.setOpen(!card.open);
    pills.setMaterialOpen(card.open);
  },
});
const card = new MaterialCard(workspace, {
  setNumbers: (id, thickness, kerf) => change(api.setMaterialNumbers(id, thickness, kerf)),
  setUnits: (units) => change(api.setDisplayUnits(units)),
  closed: () => pills.setMaterialOpen(false),
});

/** Redraw everything from a snapshot of the design. */
function show(snapshot: Snapshot, frame = false) {
  viewport.show(snapshot.scene, { frame });
  pills.update(snapshot);
  pills.setMaterialOpen(card.open);
  card.update(snapshot);
  const problems = snapshot.scene.panels.filter((p) => p.problem).map((p) => `${p.name}: ${p.problem}`);
  notify(problems.length ? problems.join(" ") : null);
}

/** Wait for the core to make a change, then show the result. A refusal is passed on. */
async function change(request: Promise<Snapshot>) {
  show(await request);
}

/** Tell the user about something wrong with the design. */
function notify(message: string | null) {
  const notice = document.getElementById("notice")!;
  notice.textContent = message ?? "";
  notice.hidden = !message;
}

// Undo and redo from the keyboard, except while typing, where they belong to the text.
window.addEventListener("keydown", (e) => {
  if (!(e.metaKey || e.ctrlKey) || e.key.toLowerCase() !== "z") return;
  if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
  e.preventDefault();
  void change(e.shiftKey ? api.redo() : api.undo());
});

api
  .snapshot()
  .then((snapshot) => show(snapshot, true))
  .catch((error) => notify(`Could not load the design: ${error}`));

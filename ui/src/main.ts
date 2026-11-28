import { loadScene } from "./api";
import { ViewCube } from "./viewcube";
import { Viewport } from "./viewport";

const workspace = document.getElementById("workspace")!;
const viewport = new Viewport(document.getElementById("viewport")!);
const cube = new ViewCube(workspace, (view) => viewport.lookFrom(view));
viewport.onCameraChange((camera) => cube.update(camera));

/** Tell the user about something wrong with the design. */
function notify(message: string | null) {
  const notice = document.getElementById("notice")!;
  notice.textContent = message ?? "";
  notice.hidden = !message;
}

async function start() {
  try {
    const scene = await loadScene();
    viewport.show(scene, { frame: true });
    const problems = scene.panels.filter((p) => p.problem).map((p) => `${p.name}: ${p.problem}`);
    notify(problems.length ? problems.join(" ") : null);
  } catch (error) {
    notify(`Could not load the design: ${error}`);
  }
}

void start();

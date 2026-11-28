// Floating bars and cards: pieces of interface that sit over the 3D view and
// can be dragged anywhere in the window by their grip, with a mouse or a finger.
// Each remembers where it was left.

const MARGIN = 6;

export interface Floating {
  /** Put the piece back inside the window, or at its home if it was never moved. */
  settle(): void;
}

/**
 * Make `piece` draggable by `handle` anywhere inside `bounds`. `home` gives its
 * starting corner from the window's size and its own, and `key` names it for
 * remembering its place.
 */
export function float(
  bounds: HTMLElement,
  piece: HTMLElement,
  handle: HTMLElement,
  key: string,
  home: (window: { width: number; height: number }, self: { width: number; height: number }) => { left: number; top: number },
): Floating {
  let moved = recall(key);

  const settle = () => {
    if (piece.hidden) return;
    const area = { width: bounds.clientWidth, height: bounds.clientHeight };
    const self = { width: piece.offsetWidth, height: piece.offsetHeight };
    const want = moved ?? home(area, self);
    piece.style.left = `${clamp(want.left, MARGIN, area.width - self.width - MARGIN)}px`;
    piece.style.top = `${clamp(want.top, MARGIN, area.height - self.height - MARGIN)}px`;
  };

  let grab: { x: number; y: number; left: number; top: number } | null = null;
  handle.addEventListener("pointerdown", (e) => {
    if (e.button !== 0) return;
    handle.setPointerCapture(e.pointerId);
    grab = { x: e.clientX, y: e.clientY, left: piece.offsetLeft, top: piece.offsetTop };
    piece.classList.add("dragging");
    e.preventDefault();
  });
  handle.addEventListener("pointermove", (e) => {
    if (!grab) return;
    moved = { left: grab.left + e.clientX - grab.x, top: grab.top + e.clientY - grab.y };
    settle();
  });
  const release = () => {
    if (!grab) return;
    grab = null;
    piece.classList.remove("dragging");
    // Remember where it actually ended up, not where the pointer pulled it to.
    moved = { left: piece.offsetLeft, top: piece.offsetTop };
    remember(key, moved);
  };
  handle.addEventListener("pointerup", release);
  handle.addEventListener("pointercancel", release);

  new ResizeObserver(settle).observe(bounds);
  settle();
  return { settle };
}

/** The strip at the top of a bar or card that it is dragged by. */
export function grip(): HTMLElement {
  const handle = document.createElement("div");
  handle.className = "grip";
  handle.setAttribute("aria-hidden", "true");
  handle.innerHTML = "<span></span>";
  return handle;
}

function clamp(value: number, low: number, high: number): number {
  return Math.max(low, Math.min(value, Math.max(low, high)));
}

// Positions are a convenience, so a browser that refuses storage just forgets them.
function recall(key: string): { left: number; top: number } | null {
  try {
    const saved = JSON.parse(localStorage.getItem(`boxcad.place.${key}`) ?? "null");
    return saved && Number.isFinite(saved.left) && Number.isFinite(saved.top) ? saved : null;
  } catch {
    return null;
  }
}

function remember(key: string, place: { left: number; top: number }) {
  try {
    localStorage.setItem(`boxcad.place.${key}`, JSON.stringify(place));
  } catch {
    // Not remembered; nothing else depends on it.
  }
}

# Box CAD

A desktop CAD program for designing laser-cut boxes from flat panels.

## Run it

You need Node, Rust and, on a Mac, the Xcode command-line tools.

```
npm install
npm run tauri dev
```

## Layout

| Folder | What it holds |
| --- | --- |
| `crates/boxcad-core` | The document model and all cut geometry. No UI dependency. |
| `crates/boxcad-cli` | A command-line tool over the core, for tests and sweeps. |
| `src-tauri` | The Tauri shell: the window, and commands that call the core. |
| `ui` | The interface, in TypeScript. |

## Checks

```
cargo test -p boxcad-core -p boxcad-cli
npm run build
```

## Command-line tool

```
cargo run -p boxcad-cli -- sample panel.boxcad    # write a one-panel project
cargo run -p boxcad-cli -- info panel.boxcad      # open it, check it, list its contents
```

A project is a `.boxcad` file: versioned JSON that describes the design.

## Version

`package.json` holds the version. Change it with `scripts/bump.sh 0.2.0`, which
also updates the Cargo workspace.

The rules for working in this repository are in `CLAUDE.md`.

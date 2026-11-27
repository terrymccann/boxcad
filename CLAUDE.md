# Box CAD

A desktop CAD program for designing laser-cut boxes from flat panels.
Stack: Tauri 2, a Rust core library with a command-line tool, and TypeScript
with Three.js for the interface.

## Where the plan lives

Read these before writing code. If they disagree, the spec wins.

1. Product spec (source of truth): https://claude.ai/artifact/KQDMQZeuCGymTdhZVwjqC7
2. Build handoff (decisions made, phase 0 steps, design values, how Terry works):
   https://claude.ai/artifact/KhAXHdJwHixDp1oSU8tFK9
3. Interface mockups: https://claude.ai/artifact/LQPdYriSSbH8e2puHJD9xP
   Use the four floating-layout artboards. Ignore the one named "Workspace";
   that layout was rejected.
4. `terrymccann/layercut`, read-only, as reference only.

## Layout

```
crates/boxcad-core/   geometry and document model, no UI dependency
crates/boxcad-cli/    command-line tool for tests and sweeps
src-tauri/            the Tauri shell
ui/                   TypeScript and Three.js interface
.github/workflows/    CI and, later, release and publish
```

## Rules

- **One branch per task.** Never commit to the default branch. Push the branch;
  Terry opens the pull request, merges it and pulls. Wait for him to say it is
  merged before starting work that depends on it.
- **Core first.** Geometry lives in `boxcad-core` and is reachable from the
  command-line tool. The interface only calls the core; it never computes cut
  geometry itself.
- **Tests with every geometry change.** Each joint, offset or export change
  comes with a core test. Sweep the command-line tool over a range of sizes and
  thicknesses before calling geometry done.
- **Never copy LayerCut code wholesale.** Read it, understand it, and write
  Box CAD's own version. Third-party libraries LayerCut uses may be used
  directly.
- **Units.** Millimeters internally, everywhere. Inches are a display setting.
- **Touch-ready.** No feature may depend on right-click or hover, and controls
  are at least 44 px.
- **No secrets in the session.** Claude never types, reads or stores passwords,
  signing keys or API tokens. Terry creates them and runs `gh secret set`
  himself.
- **Releases are Terry's to publish.** Builds may run automatically; making a
  release public is a manual step he takes.
- **Version in one place.** `package.json` holds the version; the Tauri config
  and Cargo files follow it through a bump script, `scripts/bump.sh`.

## Working with Terry

- Phase 0 is done one step per branch. After each step, stop and give him
  complete, copy-pasteable instructions for anything he must do, and say what
  he should see when it works. Terse instructions frustrate him.
- He reviews by looking. Show the interface early; send screenshots or mockups
  rather than descriptions.
- Put a clear recommendation with its trade-off in front of him, and record
  the decision in the spec.
- He prefers a choice over automatic behavior when an action changes his design.
- His machine is an Intel MacBook Pro. He runs `npm run tauri dev` to try a
  build.

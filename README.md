# Formiga Hill

Formiga Desktop is where the creatures live around you. Formiga Hill is somewhere you send them on
purpose, to spend time together.

Hill is a separate desktop app that receives a colony from
[Formiga Desktop](https://github.com/Von-Van/Formiga-Desktop) by train, draws each companion
exactly as Desktop does, and gives them a small storybook place to be in: free play, short
authored scenes, minigames, and later community-made content. Desktop stays the home of the colony;
Hill owns only the destination.

- [docs/DESIGN.md](docs/DESIGN.md): the design handoff this project started from.
- [docs/TRAVEL.md](docs/TRAVEL.md): how Hill keeps its side of Desktop's travel contract.

## Where it is

Phase 0.1, the travel prototype, is under way.

| | |
| --- | --- |
| Done | Desktop's travel contract, from Hill's side: acknowledgement, recall, receipt (`trip.rs`), and the colony made ready to draw (`cast.rs`) |
| Done | The station, painted in detail: the house, canopy, nameboard, garden, platform and line; the colony on the platform, drawn by `formiga-art` from the snapshot, accessories and all; names, and a tooltip with temperament, traits, habits, family and closest friend |
| Done | The train: it pulls in with everyone at a window, they hop down one by one, and it steams away; "Take the train home" runs it in reverse. Cuts instead of motion with reduced motion; a click or Space skips the arrival |
| Elsewhere | Desktop's side is built on Desktop's `work/hill-enablement` branch: the tray item, the train on the desktop, launching Hill, recall and recovery |
| Next | The Village Green: free play with petting, snacks and toys, behaviour read from each traveller's temperament, habits and bonds |
| Later | Packaging Hill so Desktop can find it installed ([TRAVEL.md](docs/TRAVEL.md#not-done-yet)) |
| Then | 0.2 character proof, 0.3 story runtime and packages ([DESIGN.md §12](docs/DESIGN.md#12-recommended-development-phases)) |

## Running it

Rust installs itself from `rust-toolchain.toml` (1.97.1, as Desktop pins). macOS 14+ and Windows
10/11, like Desktop.

```sh
cargo run -p formiga-hill                     # Desktop's sample colony
cargo run -p formiga-hill -- --formiga-travel <trip directory>
cargo run -p formiga-hill -- --from-save ~/path/to/colony.json
cargo run -p formiga-hill -- --render-station station.png
cargo run -p formiga-hill -- --render-station arriving.png --at 3.6
```

`--render-station` draws the scene without a window: settled, or `--at` that many seconds into
the arrival.

For a real trip, run Desktop with `FORMIGA_HILL_PATH` pointing at Hill's binary and choose "Go to
Formiga Hill…". `--from-save` reads a colony file, never writes to it, and projects it exactly
as Desktop does for a trip. It reads any save the pinned `formiga-core` understands.

## Layout

```text
crates/
  formiga-hill/     the app
    src/trip.rs       Hill's side of the trip: acknowledgement, recall, receipt
    src/cast.rs       the travellers, ready to draw, and how they get on
    src/paint.rs      painting tools for scenery: ramps, bevels, polygons, texture
    src/font.rs       5×7 lettering for signs painted into a scene
    src/station/      the station: scenery, the train, and the timeline of its comings and goings
docs/
```

The design's `formiga-hill-runtime` (cast resolution, scenes, packages, the Hill save) and
`formiga-hill-ui` crates will split out of `formiga-hill` when there is enough of each to split.

## Formiga Desktop's crates

`formiga-core`, `formiga-art` and `formiga-travel` all come from Formiga Desktop, from one source
so their types agree. Until Desktop publishes its Hill work, that source is the
`work/hill-enablement` branch of the local Desktop checkout, so Hill builds only on this machine
and CI cannot build it. Once Desktop releases, point all three at the GitHub repository at that
release's tag.

### Working against a Desktop checkout

To build Hill against unreleased changes in a local Desktop checkout, without touching the
committed manifest, create `.cargo/config.toml` (it is ignored by git):

```toml
[patch."https://github.com/Von-Van/Formiga-Desktop"]
formiga-core = { path = "../Formiga Desktop/crates/formiga-core" }
formiga-art = { path = "../Formiga Desktop/crates/formiga-art" }
```

Delete it again before cutting a release.

## Checks

The same gate CI runs on macOS and Windows:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

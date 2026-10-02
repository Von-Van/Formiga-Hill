# Formiga Hill

Formiga Desktop is where the creatures live around you. Formiga Hill is somewhere you send them on
purpose, to spend time together.

Hill is a separate desktop app that receives a colony from
[Formiga Desktop](https://github.com/Von-Van/Formiga-Desktop) by train, draws each companion
exactly as Desktop does, and gives them a small storybook place to be in: free play, short
authored scenes, games the colony plays while you watch, and later expeditions into the Woods and
a Hilltop the colony fills with what it finds there. Desktop stays the home of the colony; Hill
owns only the destination.

- [docs/DESIGN.md](docs/DESIGN.md): the design handoff (revised), including the Woods → Hilltop
  loop.
- [docs/TRAVEL.md](docs/TRAVEL.md): how Hill keeps its side of Desktop's travel contract.
- [docs/PACKAGES.md](docs/PACKAGES.md): writing a story for Hill, no code needed.

## The places

| Area | What happens there | State |
| --- | --- | --- |
| Station | The train in and out; the colony's display case of kept souvenirs | Built |
| Village Green | Free play, pats, snacks and toys; stories are staged here for now | Built |
| Fairground | Games the colony plays among themselves while you watch, starting with hide-and-seek | Built |
| Clubhouse | Authored scenes (DESIGN.md §2) | Later |
| Woods | Activities you play yourself, with a companion or two, for finds to take home (DESIGN.md §2, the Woods → Hilltop loop) | Planned |
| Hilltop | The colony's own place, empty at first, filled and arranged with what the Woods turns up | Planned |

## Where it is

| | |
| --- | --- |
| Done | Desktop's travel contract, from Hill's side: acknowledgement, recall, receipt (`trip.rs`), and the colony made ready to draw (`cast.rs`) |
| Done | The station, painted in detail: the house, canopy, nameboard, garden, platform and line; the colony on the platform, drawn by `formiga-art` from the snapshot, accessories and all; names, and a tooltip with temperament, traits, habits, family and closest friend |
| Done | The train: it pulls in with everyone at a window, they hop down one by one, and it steams away; "Take the train home" runs it in reverse. Cuts instead of motion with reduced motion; a click or Space skips the arrival |
| Elsewhere | Desktop's side is built on Desktop's `work/hill-enablement` branch: the tray item, the train on the desktop, launching Hill, recall and recovery |
| Done | The Village Green, painted to the station's standard: free play read from each traveller's temperament, habits, pace and bonds. Wandering, the blanket, naps in the oak's shade, visiting friends, playing with playmates, minis trailing parents, rivals keeping apart. A pat, a snack or a toy is answered in each one's own way, warming to the person over the visit |
| Done | Stories as declarative content packages ([PACKAGES.md](docs/PACKAGES.md)): a strict, sandboxed loader; cast selectors with fallbacks; a beat vocabulary of walking, reactions, poses, lines, choices, branches and flags; lines that vary by temperament; a director that stages it on the green with a speech bubble over whoever is talking |
| Done | *The First Picnic*, Hill's first story: three scenes, a choice and a branch, shipped as a package in exactly the community format |
| Done | Hill's own memories of each colony: stories finished, souvenirs kept, visits, each seeker's quickest game |
| Done | The station's display case, showing each souvenir the colony has kept |
| Done | The Village Green repainted from above, with no houses: lawn, gravel path, the old oak and its swing, the well, a flower bed, the toy chest and the picnic blanket, against the edge of the woods |
| Done | The Fairground at dusk, reached from the green: a big top, a carousel, a hoopla stall and things to hide behind, with the Hill's tree on the skyline. Free play there as on the green, and hide-and-seek to watch: you pick who is "it" (or let the colony decide), it counts with its eyes covered, everyone hides, and it goes looking. Where each hides, how each gives itself away, and how "it" searches all come from temperament; a parent never finds its little one until last. The first game seen through keeps a ticket in the display case |
| Later | A mods folder and a package list for community stories; packaging Hill so Desktop can find it installed ([TRAVEL.md](docs/TRAVEL.md#not-done-yet)) |
| Next | The Woods → Hilltop loop (DESIGN.md §2, §11): one deep Woods activity you play, with finds that differ by companion, and a Hilltop you arrange them on, seen from the other areas |

## Running it

Rust installs itself from `rust-toolchain.toml` (1.97.1, as Desktop pins). macOS 14+ and Windows
10/11, like Desktop.

```sh
cargo run -p formiga-hill                     # Desktop's sample colony
cargo run -p formiga-hill -- --formiga-travel <trip directory>
cargo run -p formiga-hill -- --from-save ~/path/to/colony.json
cargo run -p formiga-hill -- --render-station station.png
cargo run -p formiga-hill -- --render-station arriving.png --at 3.6
cargo run -p formiga-hill -- --render-green green.png --at 25
cargo run -p formiga-hill -- --render-fairground fairground.png --at 25
cargo run -p formiga-hill -- --render-hide-and-seek hiding.png --at 10
cargo run -p formiga-hill -- --render-reactions reactions.png
cargo run -p formiga-hill -- --render-story story.png --at 20
cargo run -p formiga-hill -- --check-package path/to/my-story.formiga-hill
cargo run -p formiga-hill -- --package path/to/my-story.formiga-hill
```

The render options draw without a window: the station settled, or `--at` seconds into the
arrival; the green `--at` seconds into free play; and a review sheet of every traveller answering
a pat, a snack and a toy.

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
    src/materials.rs  the palette: every material as a ramp of tones
    src/kit.rs        pieces areas are built from: walls, roofs, windows, stonework, bushes
    src/station/      the station: scenery, the train, and the timeline of its comings and goings
    src/green/        the Village Green: scenery and free play
    src/character.rs  who each traveller is, turned into what it does
    src/actor.rs      a traveller performing on a stage: steps, beats, cached frames, gaze
    src/cues.rs       hearts, notes and other signs over a creature's head
    src/sheet.rs      review sheets
    src/story/        content packages: loading, casting, the script, the director
    src/memories.rs   what Hill remembers of each colony
  content/            Hill's own story packages, in the community format
docs/
```

The design's `formiga-hill-runtime` (cast resolution, scenes, packages, the Hill save) and
`formiga-hill-ui` crates will split out of `formiga-hill` when there is enough of each to split.

## Formiga Desktop's crates

`formiga-core`, `formiga-art` and `formiga-travel` all come from Formiga Desktop, from one source
so their types agree. Until Desktop releases its Hill work, that source is the
`work/hill-enablement` branch on GitHub, pinned to a commit by `Cargo.lock`
(`cargo update -p formiga-travel -p formiga-core -p formiga-art` moves all three). Once Desktop
releases, point all three at that release's tag.

### Working against a Desktop checkout

To build Hill against unreleased changes in a local Desktop checkout, without touching the
committed manifest, create `.cargo/config.toml` (it is ignored by git):

```toml
[patch."https://github.com/Von-Van/Formiga-Desktop"]
formiga-core = { path = "../Formiga Desktop/crates/formiga-core" }
formiga-art = { path = "../Formiga Desktop/crates/formiga-art" }
formiga-travel = { path = "../Formiga Desktop/crates/formiga-travel" }
```

Patch all three or none, so their types agree, and point the paths at whichever Desktop checkout
or worktree has the changes. Delete it again before committing a change that depends on it.

## Checks

The same gate CI runs on macOS and Windows:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

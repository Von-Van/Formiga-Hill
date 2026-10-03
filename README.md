# Formiga Hill

Formiga Desktop is where the creatures live around you. Formiga Hill is somewhere you send them on
purpose, to spend time together.

Hill is a separate desktop app that receives a colony from
[Formiga Desktop](https://github.com/Von-Van/Formiga-Desktop) by train, draws each companion
exactly as Desktop does, and gives them a small storybook place to be in: free play, short
authored scenes, games the colony plays while you watch, outings into the Woods that you play
yourself, and a Hilltop the colony fills with what it finds there. Desktop stays the home of the colony; Hill
owns only the destination.

- [docs/DESIGN.md](docs/DESIGN.md): the design handoff (revised), including the Woods → Hilltop
  loop.
- [docs/TRAVEL.md](docs/TRAVEL.md): how Hill keeps its side of Desktop's travel contract.
- [docs/PACKAGES.md](docs/PACKAGES.md): writing a story for Hill, no code needed.

## The places

| Area | What happens there | State |
| --- | --- | --- |
| Station | The train in and out; the colony's display case of kept souvenirs | Built |
| Village Green | Free play, pats, snacks and toys | Built |
| Fairground | Games the colony plays among themselves while you watch, starting with hide-and-seek | Built |
| Clubhouse | Authored scenes, picked from the notice board and played out by the fire; free play between them | Built |
| Woods | Activities you play yourself, with a companion or two, for finds to take home (DESIGN.md §2, the Woods → Hilltop loop): rummaging in the glade, fishing at the pool | Being built |
| Hilltop | The colony's own place, empty at first, filled and arranged with what the Woods turns up | Being built |

## Where it is

| | |
| --- | --- |
| Done | Desktop's travel contract, from Hill's side: acknowledgement, recall, receipt (`trip.rs`), and the colony made ready to draw (`cast.rs`) |
| Done | The station, painted in detail: the house, canopy, nameboard, garden, platform and line; the colony on the platform, drawn by `formiga-art` from the snapshot, accessories and all; names, and a tooltip with temperament, traits, habits, family and closest friend |
| Done | The train: it pulls in with everyone at a window, they hop down one by one, and it steams away; "Take the train home" runs it in reverse. Cuts instead of motion with reduced motion; a click or Space skips the arrival |
| Elsewhere | Desktop's side is released in Formiga Desktop 0.66.4: the tray item, the train on the desktop, launching Hill, recall and recovery |
| Done | The Village Green, painted to the station's standard: free play read from each traveller's temperament, habits, pace and bonds. Wandering, the blanket, naps in the oak's shade, visiting friends, playing with playmates, minis trailing parents, rivals keeping apart. A pat, a snack or a toy is answered in each one's own way, warming to the person over the visit |
| Done | Stories as declarative content packages ([PACKAGES.md](docs/PACKAGES.md)): a strict, sandboxed loader; cast selectors with fallbacks; a beat vocabulary of walking, reactions, poses, lines, choices, branches and flags; lines that vary by temperament; a director that stages it with a speech bubble over whoever is talking |
| Done | *The First Picnic*, Hill's first story: three scenes, a choice and a branch, shipped as a package in exactly the community format |
| Done | The Clubhouse, where stories moved indoors: a panelled room with a crackling fire (held steady with reduced motion), a braided rug, armchairs, a bookshelf, a gingham-covered table and a toy box, and the Hill through the window with whatever stands on it. The notice board pins up a card for each story and stars the finished ones; click it to pick one. Between stories the colony warms by the fire, browses the books and gazes out of the window. A second story, *The Book with No Ending*, with a choice between making up an ending and hunting for the missing page |
| Done | *The Last Bun*, a small mystery: one bun left on the table is gone, and whoever likes a mystery takes the case, questioning the colony's own suspects (whoever looks food over, the troublemaker) or following the crumbs, to find it kept safe by the one fondest of them |
| Done | Hill's own memories of each colony: stories finished, souvenirs kept, visits, each seeker's quickest game |
| Done | The station's display case, showing each souvenir the colony has kept |
| Done | The Village Green repainted from above, with no houses: lawn, gravel path, the old oak and its swing, the well, a flower bed, the toy chest and the picnic blanket, against the edge of the woods |
| Done | The Fairground, reached from the green: a big top, a carousel, a hoopla stall and things to hide behind, with the Hill's tree on the skyline, its bulbs lit after dark. Free play there as on the green, and hide-and-seek to watch: you pick who is "it" (or let the colony decide), it counts with its eyes covered, everyone hides, and it goes looking. Where each hides, how each gives itself away, and how "it" searches all come from temperament; a parent never finds its little one until last. The first game seen through keeps a ticket in the display case |
| Done | A packages folder for community stories: drop a package in and its stories join the notice board. The board's package list shows where each came from, anything that would not load and why, and sets any community package aside without touching it. `--packages-folder` says where the folder is |
| Done | Packaging Hill so Desktop finds it installed: a universal macOS app with the bundle id and travel version Desktop looks for, and a per-user Windows installer writing its registry values (`scripts/package-macos.sh`, `scripts/package-windows.ps1`). A trip that arrives while Hill is already hosting a colony is refused as busy ([TRAVEL.md](docs/TRAVEL.md#being-found-and-being-busy)) |
| Done | Rummaging in the Woods: bring one or two companions into a glade of twelve spots (dig, reach in, scoop, shake). Spots that hold something give signs, fainter for rarer finds, the rarest only as the light goes; choose where to search, then catch the moment on a turning ring. Walking and each try cost light, and the glade darkens to dusk. Each companion's knack, from its temperament, widens the ring for one kind of spot; curious ones point out signs; a pair, and a close pair more so, catch more easily. Reduced motion turns the ring only while you hold it |
| Done | Twenty-eight finds in four rarities, leaning towards some companions (shiny things to the bold, old things to scholars, small things to little ones) without ruling anything out. Undiscovered finds turn up more often, and after two outings with nothing new one is certain, so a colony that only ever brings one companion still finds everything |
| Done | The Hilltop: any find stands on any of eighteen spots, moved or put back at will, and the colony visits what stands there, sitting on the stones, gazing through the telescope, napping by the berry bush. A journal of every find, who found it first, and hints of the rest |
| Done | What stands on the Hilltop changes the Woods (a lantern lends light, the telescope brings rare signs out sooner, each piece draws the eye to its kind of spot), and shows small on the station's hill and in silhouette on the Fairground's skyline |
| Done | Spots only some company opens: a badger sett for an explorer, a crevice only a little one fits, a boulder a close pair can heave over. Extra chances, never the only way to anything |
| Done | Fishing at the pool: eight kinds of fish, each keeping to its own part of the pool (trout under the falls, pike in the reeds, carp by the lilies). Cast where you think they are, but not on top of a wary one; tell a nibble from a bite, each kind nibbling its own number of times; reel in and ease off when it pulls, or the line snaps. The Old One only comes up at dusk. A patient angler gets a longer moment to strike, a playful one draws fish in, a strong one strains the line less, a dozing lazybones gets bolder bites, and a second companion lands fish with the net. Fish are let go and remembered: how many, the longest, and who caught the first. Now and then something snags and comes home for the Hilltop |
| Done | A secret in the Woods (DESIGN.md §2, the Cursor Sovereign): an interactive cutscene played completely straight, with a menu of absurd attacks each acted out by the companion it belongs to, three phases, and the whole colony arriving for the finale. It leaves a relic for the Hilltop. How to find it is deliberately not written here; see CLAUDE.md |
| Done | The Hill keeps your hours: its light follows your computer's clock, so a morning visit is morning everywhere and a late one is under the stars. Dawn comes up rose and evening goes down gold; after dark the station's lamp, ticket office and display case are lit, fireflies come out on the green and in the Woods, the Clubhouse dims only to lamp and firelight with the night in its window, and the Hilltop's lantern and fallen star glow, with the Fairground's bulbs far below. Nothing in play depends on the hour |
| Next | More Woods activities (bug catching, longer expeditions); finds that open new Woods possibilities |

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
cargo run -p formiga-hill -- --render-clubhouse clubhouse.png --at 20
cargo run -p formiga-hill -- --render-station night.png --hour 23
cargo run -p formiga-hill -- --render-fairground fairground.png --at 25
cargo run -p formiga-hill -- --render-hide-and-seek hiding.png --at 10
cargo run -p formiga-hill -- --render-woods woods.png --at 30
cargo run -p formiga-hill -- --render-hilltop hilltop.png
cargo run -p formiga-hill -- --render-finds finds.png
cargo run -p formiga-hill -- --render-fishing fishing.png --at 20
cargo run -p formiga-hill -- --render-fish fish.png
cargo run -p formiga-hill -- --render-sovereign secret.png --at 60
cargo run -p formiga-hill -- --render-reactions reactions.png
cargo run -p formiga-hill -- --render-story story.png --at 20
cargo run -p formiga-hill -- --check-package path/to/my-story.formiga-hill
cargo run -p formiga-hill -- --package path/to/my-story.formiga-hill
```

The render options draw without a window: the station settled, or `--at` seconds into the
arrival; the green or the Clubhouse `--at` seconds into free play; and a review sheet of every traveller answering
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
    src/clubhouse/    the Clubhouse: the room where stories are staged, and its fire
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
so their types agree: a release tag on GitHub, now `v0.66.4`. To move to a newer release, change
the tag on all three in the root `Cargo.toml` together.

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

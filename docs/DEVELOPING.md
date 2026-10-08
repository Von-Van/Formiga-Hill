# Developing Formiga Hill

This page is for anyone building Hill from source, checking a change, or trying to find their way
around the code. For what Hill is and why it works the way it does, the [README](../README.md) is
a better place to start.

## Getting set up

Hill is written in Rust, and the right toolchain installs itself from `rust-toolchain.toml`
(1.97.1, the same version Desktop pins). Like Desktop, it runs on macOS 14 or later and on Windows
10 or 11.

```sh
cargo run -p formiga-hill                                  # Desktop's sample colony
cargo run -p formiga-hill -- --from-save ~/path/to/colony.json
cargo run -p formiga-hill -- --formiga-travel <trip directory>
```

There are a few ways to give Hill a colony while developing, depending on how close to the real
thing a test needs to be:

- **The sample colony.** With no arguments, Hill hosts Desktop's own sample colony. This is
  usually enough for art and behaviour.
- **`--from-save`.** Reads a Desktop save, never writes to it, and projects it exactly as Desktop
  does for a trip. It reads any save the pinned `formiga-core` understands, which makes it useful
  for seeing how a particular colony plays out.
- **A real trip.** Run Desktop with `FORMIGA_HILL_PATH` pointing at Hill's binary, and the whole
  journey runs as it would for anyone else.

Story packages have their own options: `--check-package <folder>` checks one, `--package <folder>`
loads one for this run, and `--packages-folder` prints where the packages folder is.
[PACKAGES.md](PACKAGES.md) covers the format.

To build the downloads, `scripts/package-macos.sh` makes a universal `Formiga Hill.app` with the
bundle id and travel version Desktop looks for, and `scripts/package-windows.ps1` makes a per-user
installer that writes the registry values Desktop reads.

## Renders

Tests can say that something happened, but they cannot really say whether it looks right. So every
place and review sheet can also be drawn to a PNG without opening a window. After changing anything
visual, or anything about how the creatures behave, it is worth drawing the affected places and
looking at them cropped and enlarged.

```sh
cargo run -p formiga-hill -- --render-station station.png --at 3.6 --hour 21
```

| Option | Draws |
| --- | --- |
| `--render-station` | The station; `--at` seconds into the arrival, or settled; `--sample-hilltop` puts a sample Hilltop on its skyline |
| `--render-green`, `--render-clubhouse`, `--render-fairground` | The place, `--at` seconds into free play |
| `--render-hide-and-seek`, `--render-sack-race`, `--render-high-striker`, `--render-hoopla`, `--render-tug-of-war` | A game at the Fairground, `--at` seconds in |
| `--render-woods`, `--render-fishing`, `--render-bug-hunt` | A Woods outing, `--at` seconds in |
| `--render-meadow` | The meadow |
| `--render-hedgerow` | A foray along the hedgerow, `--at` seconds in |
| `--render-produce` | Everything the hedgerow grows, at every stage of ripeness |
| `--render-expedition` | An expedition, `--at` seconds in: the map, each stop on the way, the picnic |
| `--render-falls` | Wading at the Far Falls, `--at` seconds in |
| `--render-track` | A scavenge along the old track, `--at` seconds in |
| `--render-treasure` | A treasure hunt off the old track, `--at` seconds in |
| `--render-landmarks` | Every landmark a map can name, near and far, and the dig |
| `--render-hilltop` | The Hilltop, with a sample of finds on it |
| `--render-building` | The colony building on the Hilltop, `--at` seconds in |
| `--render-story` | A story, `--at` seconds in |
| `--render-sovereign` | The secret, `--at` seconds in |
| `--render-finds`, `--render-fish`, `--render-bugs` | Sheets of every find, fish and bug |
| `--render-growing`, `--render-plans` | Every growing thing at every stage, and every plan |
| `--render-reactions` | Every traveller answering a pat, a snack and a toy |
| `--render-costumes` | Every traveller in every costume piece |
| `--render-sounds <folder>` | Not a picture: every sound, and a minute of each piece of music by day and by night, as WAV files to listen to |

Any of the pictures takes `--hour <0-24>` to see it at that hour.

The window itself, with its buttons, cards and speech, is pictured with `--snap <png>`: it opens,
waits `--at` seconds (3 if not given), saves what it shows and closes, letting in nothing the keys
or pointer do. `--place` opens it somewhere other than the station, `--card` opens one of its
cards (at the place it belongs to, if none is named), and `--story` begins the first story in the
Clubhouse. If the picture can't be saved, it says why and fails. Give it a scratch
`FORMIGA_HILL_DATA_DIR` so a review never touches the colony's memories.

```sh
FORMIGA_HILL_DATA_DIR=/tmp/hill-review \
  cargo run -p formiga-hill -- --snap board.png --place clubhouse --card board --hour 12
```

## Layout

Most of the code is organised by place, with a few shared pieces underneath that every place uses.
The most important of those is `character.rs`, which turns what Desktop sends about a companion
into what it actually does. Nothing anywhere is written for one particular creature.

```text
crates/formiga-hill/src/
  main.rs            arguments, and opening the window
  render.rs          the `--render-*` review pictures, drawn without a window
  app.rs, app/       the window: one module per place's controls and cards, and `paper.rs`,
                     the paper buttons, cards, trays and speech they are all made of
  lettering.rs       the pixel lettering everything in the window is written in
  trip.rs            Hill's side of the trip: acknowledgement, recall, receipt
  hosting.rs         one colony at a time
  cast.rs            the travellers, ready to draw, and how they get on
  character.rs       who each traveller is, turned into what it does
  playground.rs      free play anywhere: each traveller choosing what to do next
  actor.rs           a traveller performing: steps, beats, cached frames, what it wears
  cues.rs            hearts, notes and other signs over a creature's head
  dice.rs            small seeded randomness, so a visit unfolds the same way from the same start
  station/           the station, the train, and its comings and goings
  green/             the Village Green
  clubhouse/         the Clubhouse and its fire
  fairground/        the Fairground and its games
  woods/             the glade, rummaging there, and the basket other outings draw with it
  fishing/           the pool, its fish, and angling
  meadow/            the meadow, its bugs, and catching them
  hedgerow/          the hedgerow, what ripens there, and foraging
  expedition/        a day out on the Woods' map: its ways, each stop, the picnic, the basket
  falls/             the Far Falls, and wading for what comes down them
  track/             the old track, its heaps and scavenging, and treasure maps
  hilltop/           the Hilltop, and the Hilltop seen from elsewhere
  clearing/          the secret
  finds/             everything the Woods turns up, and how each looks
  costume/           the dress-up box
  story/             story packages: loading, casting, the script, the director
  memories.rs        what Hill remembers of each colony
  photos.rs          the album
  daylight.rs        the hour's light, and what glows after dark
  audio.rs, audio/   the music and the sounds, all made in code: the synthesiser, every cue, the
                     pieces and their band, the mixer, and the one place a speaker is opened
  paint.rs, kit.rs, materials.rs, font.rs
                     painting tools, building pieces, the palette, lettering
  icon.rs            the app icon: the Hill, in pixels
  sheet.rs           review sheets: pictures for checking by eye what tests can only count
crates/formiga-hill/content/
                     Hill's own story packages, in the community format
packaging/, scripts/ the macOS app and the Windows installer
docs/
```

Everything is in one crate for now. It will probably split into a runtime crate and a window crate
once there is enough of each to justify it.

## Formiga Desktop's crates

`formiga-core`, `formiga-art` and `formiga-travel` all come from Formiga Desktop. They come from a
single source so that their types agree, and that source is a release tag on GitHub, currently
`v0.67.5` (travel version 4). Moving to a newer release means changing the tag on all three in the
root `Cargo.toml` together, which `scripts/set-version.sh <version> <tag>` does along with Hill's own
version and `Cargo.lock`. Because the train and the souvenirs are drawn by `formiga-art`, they
look the same in both apps.

To build Hill against unreleased changes in a local Desktop checkout, without touching the
committed manifest, create `.cargo/config.toml` (git ignores it):

```toml
[patch."https://github.com/Von-Van/Formiga-Desktop"]
formiga-core = { path = "../Formiga Desktop/crates/formiga-core" }
formiga-art = { path = "../Formiga Desktop/crates/formiga-art" }
formiga-travel = { path = "../Formiga Desktop/crates/formiga-travel" }
```

Patch all three or none, so their types still agree, and point the paths at whichever Desktop
checkout or worktree has the changes. Delete the file again before committing anything that
depends on it.

## Checks

CI runs the same three checks on macOS and Windows, and a change is ready when all three pass
locally:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

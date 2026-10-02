# Formiga Hill

Formiga Desktop is where the creatures live around you. Formiga Hill is somewhere you send them on
purpose, to spend time together.

Hill is a separate desktop app that receives a colony from
[Formiga Desktop](https://github.com/Von-Van/Formiga-Desktop) by train, draws each companion
exactly as Desktop does, and gives them a small storybook place to be in: free play, short
authored scenes, minigames, and later community-made content. Desktop stays the home of the colony;
Hill owns only the destination.

- [docs/DESIGN.md](docs/DESIGN.md): the design handoff this project started from.
- [docs/TRAVEL.md](docs/TRAVEL.md): the travel contract between the two apps, as built.

## Where it is

Phase 0.1, the travel prototype, is under way.

| | |
| --- | --- |
| Done | `formiga-travel`: snapshot and receipt v1, validation, bounded atomic I/O, Desktop-side exporter, frozen v1 fixture |
| Done | The station: the colony standing on the platform, drawn by `formiga-art` from the snapshot, accessories and all; names, and a tooltip with temperament, traits, habits, family and closest friend |
| Done | `--snapshot` trips write a validated receipt on the way home |
| Next | Desktop's side: "Go to Formiga Hill…", the train, launching Hill, reading the receipt, recovering an away colony |
| Next | The train arriving and leaving at the station |
| Then | 0.2 character proof, 0.3 story runtime and packages ([DESIGN.md §12](docs/DESIGN.md#12-recommended-development-phases)) |

## Running it

Rust installs itself from `rust-toolchain.toml` (1.97.1, as Desktop pins). macOS 14+ and Windows
10/11, like Desktop.

```sh
cargo run -p formiga-hill                     # the sample colony
cargo run -p formiga-hill -- --snapshot trip.snapshot.json
cargo run -p formiga-hill -- --from-save ~/path/to/colony.json
cargo run -p formiga-hill -- --render-station station.png
```

`--from-save` stands in for Desktop's exporter until Desktop has one. It reads a colony file and
never writes to it, but it can only read saves this build's `formiga-core` understands (save v24
and earlier, from Desktop 0.66.1). For anything newer, bump the Desktop tag, or point it at a copy.

## Layout

```text
crates/
  formiga-travel/   the Desktop ↔ Hill contract: shared, versioned, validated
  formiga-hill/     the app: the station scene and the window
docs/
```

The design's `formiga-hill-runtime` (cast resolution, scenes, packages, the Hill save) and
`formiga-hill-ui` crates will split out of `formiga-hill` when there is enough of each to split.

## Formiga Desktop's crates

`formiga-core` and `formiga-art` come from the public Desktop repository, pinned to a release tag
in the root `Cargo.toml`. To move to a newer Desktop, change both `tag`s together, then run the
whole check below; the frozen fixture test will say if the travel format broke.

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

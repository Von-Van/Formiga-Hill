# Formiga Hill

A separate desktop app that receives a Formiga Desktop colony "by train" and gives the same
creatures a storybook place to spend time together. Read [docs/DESIGN.md](docs/DESIGN.md) for the
product and [docs/TRAVEL.md](docs/TRAVEL.md) for the contract with Desktop before changing either
side of it.

## Relationship to Formiga Desktop

- Desktop lives at `../Formiga Desktop` (GitHub `Von-Van/Formiga-Desktop`, public). Hill is
  private. `formiga-core` and `formiga-art` come from Desktop as git dependencies pinned to a
  release tag in the root `Cargo.toml`; never copy their source into this repository.
- Desktop is authoritative for the colony. Hill never reads or writes Desktop's `colony.json` at
  runtime. The only exception is the `--from-save` development mode, which reads (never writes) a
  save to stand in for Desktop's exporter until Desktop has one.
- Do not edit the Desktop checkout from a Hill session. It is a separate project with its own
  work in progress; propose Desktop-side changes instead.
- `formiga-travel` must eventually move into the public Desktop workspace (or its own public
  repository), because Desktop cannot depend on a private crate. Keep it free of anything
  Hill-specific so that move stays mechanical.

## The travel contract

- Everything that crosses between the apps goes through `formiga-travel` and is validated on the
  way in. Nothing machine-specific crosses: no seeds, positions, monitors, paths, journal, or
  settings beyond presentation.
- Readers ignore unknown fields. Adding an optional field needs nothing else; a change an older
  reader would misread raises `minimum_reader_version` and `FORMAT_VERSION` and adds a new frozen
  fixture. Never edit or regenerate a committed fixture.
- The receipt carries requests only. Nothing Hill writes may change a companion's identity,
  temperament, habits, family, bonds, membership, or Desktop's settings.

## Design rules that apply to every change

- The same creatures, not lookalikes: draw travellers with `formiga-art` from the snapshot's
  appearance; never regenerate one.
- Soft play, not maintenance: no neglect penalties, streaks, hunger debt, or login pressure.
- Reduced motion gets an authored alternative (a held pose, a cut), not merely a slower animation.
- Community content will be declarative and sandboxed: no native code, no arbitrary file,
  network, or process access. Official content should use the same package format.

## Conventions

- Rust 1.97.1, edition 2024, matching Desktop. Match Desktop's code style: doc comments that say
  why, plain names, tests named as sentences (`a_receipt_cannot_settle_another_trip`).
- The gate, which CI runs on macOS and Windows:
  `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
- `cargo run -p formiga-hill -- --render-station station.png` draws the station without a
  window; look at it after changing anything visual.

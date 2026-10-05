# Formiga Hill

A separate desktop app that receives a Formiga Desktop colony "by train" and gives the same
creatures a storybook place to spend time together. Read [docs/DESIGN.md](docs/DESIGN.md) for the
product and [docs/TRAVEL.md](docs/TRAVEL.md) for the contract with Desktop before changing either
side of it.

## Relationship to Formiga Desktop

- Desktop lives at `../Formiga Desktop` (GitHub `Von-Van/Formiga-Desktop`, public). Hill is
  private. `formiga-core`, `formiga-art` and `formiga-travel` all come from Desktop, from one
  source in the root `Cargo.toml` so their types agree; never copy their source into this
  repository.
- The travel contract is Desktop's: its `formiga-travel` crate is the authority, and
  [docs/TRAVEL.md](docs/TRAVEL.md) is Hill's side of it. A change to the contract is asked of
  Desktop's session ("Formiga Hill integration spec"), not made here.
- Desktop is authoritative for the colony. Hill never reads or writes Desktop's `colony.json` at
  runtime. The only exception is the `--from-save` development mode, which reads (never writes) a
  save and projects it as Desktop would.
- Do not edit the Desktop checkout from a Hill session. It is a separate project with its own
  work in progress; propose Desktop-side changes instead.

## The travel contract

- Hill answers a trip once with an acknowledgement and once with a receipt, written only through
  `trip.rs`. It asks only for return effects the snapshot's capabilities offer, and never sends
  prose. A recall, or the trip's snapshot disappearing, ends the visit with nothing written.
- Nothing Hill writes may change a companion's identity, temperament, habits, family, bonds,
  membership, or Desktop's settings. Desktop brings the colony home unchanged whatever Hill does.

## Design rules that apply to every change

- Who plays decides where a game goes. Fairground games are automated: the colony plays and the
  person watches, choosing at most who plays what. Woods activities are played by the person,
  with a companion or two, for finds; finds fill the Hilltop, which the person arranges. The
  plan for the Woods → Hilltop loop is DESIGN.md §2; build it to that, and to the owner's go-ahead.
- The Hill's light follows the person's own clock (`daylight.rs`, the owner's call): a late
  visit finds it night everywhere. Paint every place by day; what glows of its own (lamps, lit
  windows, fires, fireflies, glowing finds) goes in that place's nightlights, and its sky is
  found by painting the sky on its own. Nothing in play depends on the hour: the Woods' and the
  pool's dusk stay each outing's own, so every find, fish and the secret are there at any time.
- Stories are staged in the Clubhouse, picked from its notice board; the green is for free play.
  A story's places are the Clubhouse's (`story::script::PLACES`, checked against `clubhouse.rs`).
- The Cursor Sovereign (`clearing/`) is the one secret, and stays first-party. In-game nothing
  points to it: on a rummage below a fifth of the light, a colony that has found ten or more
  finds and stands a sky-gazing piece (telescope, star stone, weathervane) on the Hilltop may
  see a small white pointer glinting between the trees; following it leads to the clearing.
  After the first victory (which leaves the relic) it comes back on every third outing only.
  It is a cutscene, never a combat system: no stats, nothing lost, every choice lands. Keep its
  trigger out of the README and any in-game hint.
- Watching earns keepsakes for the display case at most: the finds that build the Hilltop come
  only from the Woods.
- Every find and every kind of fish and bug stays findable by any single companion: a companion's leanings and knacks weight
  the odds and widen the ring, and the spots only some company opens are extra chances; none of
  it gates anything. Any Hilltop spot takes any piece, so every
  piece fits within `finds::art::PIECE_MAX`.
- The same creatures, not lookalikes: draw travellers with `formiga-art` from the snapshot's
  appearance; never regenerate one.
- Soft play, not maintenance: no neglect penalties, streaks, hunger debt, or login pressure.
- Sound is optional and never needed to play, and every cue and piece of music is a placeholder
  made in code (`audio/`) until real audio replaces it.
- Reduced motion gets an authored alternative (a held pose, a cut), not merely a slower animation.
- Areas are drawn with more fidelity and detail than Desktop's overlays: Formiga's pixel scale,
  but every material shaded with a ramp, light from the upper left, edges outlined in a darker
  shade of their own colour (never black, so the near-black-outlined creatures read first), and
  surfaces textured from `paint::noise`. Background, then props and texture: an area should look
  composed and lived-in, not flat. Keep everyone's standing spots clear of clutter.
- Community content is declarative and sandboxed ([docs/PACKAGES.md](docs/PACKAGES.md)): no
  native code, no file, network, or process access. Official stories live in `content/` as
  packages in exactly that format and load through the same code. A new beat, selector, place or
  condition goes in the script parser, the director and PACKAGES.md together.
- Text in a package is the author's: it stays in localisation files, is checked before it is
  shown, and never reaches Desktop. Rewards are only from Hill's own souvenir catalogue.

## Conventions

- Rust 1.97.1, edition 2024, matching Desktop. Match Desktop's code style: doc comments that say
  why, plain names, tests named as sentences (`a_receipt_cannot_settle_another_trip`).
- The gate, which CI runs on macOS and Windows:
  `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
- `--render-station`, `--render-green`, `--render-clubhouse`, `--render-fairground`, `--render-hide-and-seek`,
  `--render-sack-race`, `--render-high-striker`, `--render-hoopla`, `--render-tug-of-war`,
  `--render-woods`, `--render-fishing`, `--render-bug-hunt`, `--render-meadow`, `--render-hedgerow`,
  `--render-track`, `--render-treasure`, `--render-hilltop`, `--render-building`,
  `--render-expedition`, `--render-falls` (each with
  `--at <seconds>`), `--render-finds`, `--render-growing`, `--render-plans`, `--render-produce`, `--render-fish`,
  `--render-bugs`, `--render-landmarks`, `--render-sovereign`, `--render-reactions` and
  `--render-costumes` draw
  without a window (any with `--hour <0-24>` to see it at that hour); look at them, cropped and
  enlarged, after changing anything visual or any behaviour in `character.rs`.
  `--render-sounds <folder>` writes every cue and a minute of every piece of music as WAVs:
  check their levels after changing any sound.
- Behaviour is never written for a particular creature: it is read from the snapshot (axes,
  kind, pace, habits, bonds, family) in `character.rs`, so every colony plays out differently.

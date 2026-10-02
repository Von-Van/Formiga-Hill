# The travel contract

How the colony gets from Formiga Desktop to Formiga Hill and back. The types live in
`crates/formiga-travel`; this document covers the handoff and records the decisions the code has
made. The founding brief is [DESIGN.md §8](DESIGN.md#8-inter-app-travel-contract).

## Ownership

| Who | Owns | Never touches |
| --- | --- | --- |
| Desktop | The colony: identity, appearance, temperament, habits, family, bonds, history, settings | Hill's save |
| Snapshot | A read-only projection of the travellers for one trip | — |
| Hill | Everything at the Hill: areas, stories, scores, photos, souvenirs, package state | Desktop's save, ever |
| Receipt | Requests only: souvenirs and one journal line, which Desktop may ignore | — |

## The handoff

```text
Desktop                                              Hill
───────                                              ────
save the colony
draw a SessionId
export_snapshot(save) ─────► <dir>/<session>.snapshot.json
gather, train arrives, board
hide the colony
launch: formiga-hill --snapshot <dir>/<session>.snapshot.json
                                                     read_snapshot: bounded, versioned, validated
                                                     station arrival scene
                                                     … the visit …
                                                     "Take the train home" or the window closes
                     <dir>/<session>.receipt.json ◄── write_receipt (validated, atomic)
Hill exits
read_receipt(path, &snapshot), if there is one
train returns, colony reappears
```

`TravelFiles::new(dir, session_id)` names both files. Desktop picks `dir` (somewhere under its own
data directory is the obvious place) and is the only side that ever lists or cleans it.

**Desktop must not depend on the receipt.** When Hill exits without writing one, because it
crashed, was force-quit, or failed to write, the trip ends exactly like a trip that came home with
nothing: the colony comes back and nothing else changes. Desktop should also recover a colony that
is "away" when Desktop itself starts, so no failure can strand it off-screen.

Hill writes the receipt when the person takes the train home and, failing that, when the window
closes. `Completion::Unclean` is for a Hill that noticed it was going wrong and still managed to
say so; it has no other meaning yet.

## Snapshot v1

```jsonc
{
  "format_version": 1,
  "minimum_reader_version": 1,
  "session_id": "32 hex digits",
  "created_at_utc": "RFC 3339",
  "desktop_version": "0.67.0",
  "colony": { "public_id": "16 hex digits" },
  "travelers": [
    {
      "id": 5877483765951804486,
      "name": "Poppy",
      "role": { "kind": "adult" },          // or { "kind": "mini", "parent_id": … }
      "generation": 0,
      "born_at_utc": "RFC 3339",
      "appearance": { … },                  // formiga_core::AppearanceGenome, verbatim
      "temperament": { "kind": …, "axes": { … }, "tension": … },
      "traits": ["Lazy", "Competitive", "Independent"],
      "habits": ["WavesHello"],
      "accessory": { "item": { "Worn": "LeafHat" }, "ink": { "outline": [r,g,b,a], … } }
    }
  ],
  "bonds": [
    { "a": 1, "b": 2, "warmth": "high", "familiarity": "high", "playfulness": "high", "friction": "none" }
  ],
  "presentation": { "reduce_motion": false, "theme": "system", "text_scale_percent": 100 },
  "capabilities": ["appearance", "temperament", "habits", "accessories", "family", "bonds"]
}
```

[`crates/formiga-travel/tests/fixtures/snapshot-v1.json`](../crates/formiga-travel/tests/fixtures/snapshot-v1.json)
is a complete example, generated from the sample colony.

### What is deliberately left out

Seeds (the colony's and each creature's behaviour seed), positions, monitors, windows, the cursor,
habitat geometry, live plans, routines, the journal, settings beyond presentation, update state,
and file paths. `export::tests::nothing_machine_specific_crosses` checks the obvious ones.

### Decisions taken for v1

These were open in [DESIGN.md §14](DESIGN.md#14-open-design-decisions); each can change in a
later format version.

- **Who travels.** The whole colony, every time. The format already allows a partial train (a
  mini's parent may be absent), so choosing travellers later needs no format change.
- **Bonds.** Banded, not raw. Each of Desktop's four 0–255 scores becomes `none`, `low`, `medium`
  or `high` (edges at 16, 64 and 128, near the thresholds Desktop's own behaviour reads at), under
  Hill's names: affinity is `warmth`, avoidance is `friction`. Enough for casting and staging,
  and Desktop can retune its scales without Hill noticing.
- **Accessories.** Only what each companion is wearing. Its inks travel resolved, because Desktop
  chooses them against the colony seed and coats, which Hill does not receive.
- **Appearance.** `formiga_core::AppearanceGenome` crosses verbatim. Hill draws it with the same
  `formiga-art` Desktop uses, which is the whole point: the same individual, not a lookalike. The
  cost is that the snapshot's shape follows `formiga-core`'s serde shape for that type; the frozen
  fixture test catches any change to it.
- **Traits.** Carried as resolved by Desktop rather than recomputed by Hill, as variant names
  (`Trait` has no serde derive). A name this build does not know is dropped on reading.

## Versioning

- A reader ignores fields it does not know. A writer may add optional fields without raising
  anything.
- `minimum_reader_version` rises only for a change an older reader would misread. A reader refuses
  a file whose `minimum_reader_version` is above its own `FORMAT_VERSION`, with a message saying
  so, before it tries to parse the rest.
- `capabilities` says which parts of the contract a snapshot fills in. Hill content should ask for
  a capability, not a version, and step aside when it is missing.
- Every format version gets a fixture, frozen once committed
  (`FORMIGA_TRAVEL_NEW_FIXTURE=snapshot-vN.json cargo test -p formiga-travel --test compat`).
  `todays_export_still_has_everything_v1_wrote` fails if an export stops writing a field v1 wrote.

## Bounds

Snapshots are read only up to 512 KiB and receipts up to 64 KiB. At most six travellers, fifteen
bonds, three traits and two habits each, 64 capabilities. Names must pass Desktop's own name
rule. A receipt carries at most four souvenirs; package and item IDs are lowercase
`[a-z0-9._-]` identifiers, so none can name a path or a URL, and titles are single-line display
text of at most 48 characters. Files are written to a temporary name and renamed into place.

## Where this crate should live

`formiga-travel` starts here because Hill is where it is being built, but Desktop has to depend
on it too, and Desktop is public while this repository is private: a public build cannot fetch a
private git dependency. Before Desktop gains its exporter, move the crate into the Desktop
workspace (beside `formiga-core`, which it already depends on), or into a small public repository
of its own, and have Hill depend on it by tag like the other shared crates. Move it rather than
copy it.

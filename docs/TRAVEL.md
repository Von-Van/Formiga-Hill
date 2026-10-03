# The travel contract, from Hill's side

The contract between the two apps belongs to Formiga Desktop. It lives in Desktop's workspace as
the `formiga-travel` crate, and that crate's documentation is the authority: start with its
`lib.rs`, then `snapshot.rs` and `receipt.rs`. Golden fixtures for every document are in its
`tests/fixtures`. This page records how Hill keeps its side of it, and where to look in Hill's
code. The founding brief is [DESIGN.md §8](DESIGN.md#8-inter-app-travel-contract).

Hill takes `formiga-core`, `formiga-art` and `formiga-travel` from the same Desktop source, so
their types agree: a Desktop release tag, now `v0.66.4`; see the root `Cargo.toml`.

## Who owns what

| Who | Owns | Never touches |
| --- | --- | --- |
| Desktop | The colony, the travel format, the session directory and everything in `travel/` | Hill's own records |
| Hill | Everything at the Hill: areas, stories, scores, photos, package state | Desktop's colony, ever |

## A trip, from Hill's side

Desktop creates `<Desktop data>/travel/<session>/`, writes `snapshot.json` there, plays its own
train leaving the desktop, and starts Hill as the train pulls away:

```text
formiga-hill --formiga-travel <absolute session directory>
```

On macOS Desktop runs `open -W -n -a <Hill.app> --args …` and takes that process ending as Hill
ending, so Hill must quit when the visit is over. It does: the window closes and the process exits.

1. **Arrive** (`trip::arrive`). The session id is read from the directory's name. The snapshot
   is read bounded, decoded with its header checked first, checked to be for this session, and
   turned into a `Cast`. Every traveller has to be drawable, or the trip is refused.
2. **Acknowledge.** `ack.json` is written once: accepted, or refused with
   `UnsupportedVersion { reads }` or `Invalid`. A refused trip opens no window, and Hill exits.
3. **Visit.** Hill plays its own arrival at the station.
4. **Watch for a recall.** About once a second, Hill checks for `recall.json` appearing or
   `snapshot.json` disappearing. Either one means Desktop has taken the colony home: Hill closes
   and writes nothing.
5. **Come home.** "Take the train home" plays the departure. Then Hill writes `receipt.json` once
   with `ReturnReceipt::new(&seal, now, version, effects)` and exits. Closing the window writes
   the same receipt. The only effect written today is `ReturnEffect::Visit { arrived, left }`, and
   only when the snapshot's capabilities offer `visit_record`. Desktop writes its own journal line;
   Hill never sends prose.

Whatever goes wrong, whether Hill crashes, is force-quit, or writes nothing, Desktop brings the
colony home exactly as it left. Hill never relies on Desktop noticing anything else.

## What Hill reads from a traveller

- `to_creature()` gives Desktop's stand-in creature, and `accessory.to_art()` gives what it
  wears, already in Desktop's inks. These are what `formiga-art` draws, so each traveller is the
  same individual, not a lookalike.
- The character holds the temperament kind, nine axes, tension, the traits and phrase in
  Desktop's words, and the habits. `motion` holds the companion's own pace.
- Relationships come as banded `affinity`, `familiarity`, `playfulness` and `avoidance`. Hill
  calls affinity *warmth* and avoidance *friction* (`cast::Bond`).
- `presentation` holds reduce motion, theme and text size.

## Not done yet

- Souvenirs and keepsakes. Desktop will accept catalogue ids such as `accessory.leaf_hat` once
  it has a catalogue. None are offered yet.

## Being found, and being busy

- `scripts/package-macos.sh` builds `Formiga Hill.app` with the bundle id `com.formiga.hill` and
  an integer `FormigaTravelVersion` in its `Info.plist`. `scripts/package-windows.ps1` builds a
  per-user installer that writes the `HKCU\Software\Formiga\Hill` values `Path`, `Version` and
  `TravelVersion`. Both take the travel version from the binary (`--travel-version`), and a test
  holds the plist and the installer to `formiga_travel::discovery`. For development, Desktop
  still takes `FORMIGA_HILL_PATH`.
- While a Hill window is open it holds `hosting.lock` in Hill's data folder. A trip that arrives
  meanwhile is answered `Busy` and nothing else is read from it, so Desktop can try again later.
  The lock is the operating system's, so it goes with the window however that closes.

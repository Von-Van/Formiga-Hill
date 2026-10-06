# Changelog

This page records what changed in each version of Formiga Hill. Each version's section doubles as
its release notes: `.github/workflows/release.yml` publishes it alongside the downloads when the
version is tagged.

## 0.1.0 (2026-10-06)

This is the first release of Formiga Hill. Hill receives a colony from Formiga Desktop by train,
draws each companion exactly as Desktop does, and gives the same creatures a storybook place to
spend time together. Most of this release is about testing one idea: whether the personalities
Desktop already tracks are enough, on their own, to make a whole village of activities feel
different for every colony.

### The places

- **The station.** The train pulls in with everyone at a window. A display case keeps the
  colony's souvenirs, a notice board pins up whatever is worth telling, and the departures board
  lists every place, what is on there and who is keen to go.
- **The Village Green.** Free play shaped by each companion's temperament, habits, pace, family
  and friendships. You can pat, offer a snack, throw a toy or groom with the brush, and a dress-up
  box lends eight costume pieces for the visit.
- **The Clubhouse.** Stories by the fire, chosen from its notice board: *The First Picnic*, *The
  Book with No Ending* and *The Last Bun*. Anyone can write another as a sandboxed package
  ([PACKAGES.md](docs/PACKAGES.md)) and drop it in the packages folder.
- **The Fairground.** Five games the colony plays while you watch: hide-and-seek, the sack race,
  the high striker, hoopla and tug-of-war. Each keeps a souvenir the first time it is played
  through, along with its records.
- **The Woods.** Activities you play yourself, with a companion or two, for finds to take home:
  rummaging in the glade, fishing at the pool, bug catching in the meadow, foraging along the
  hedgerow, scavenging and treasure maps on the old track, and expeditions across a map of the
  Woods to the Far Falls. There is also a secret somewhere.
- **The Hilltop.** The colony's own place, empty at first. Finds can be arranged on its eighteen
  spots, planted things grow with every visit, and plans turn finds into grander things that come
  apart again whenever you like. The Hilltop shows on the skylines of the other places.

### Everywhere

- The Hill's light follows your computer's clock.
- A camera, and an album for each colony.
- Music and sounds, with separate volumes for each and a mute (M).
- Desktop's reduced motion, theme and text size carry over. With reduced motion, Hill uses held
  poses and cuts rather than slower animation.

### With Formiga Desktop

- Hill speaks travel version 3, from Formiga Desktop 0.66.6, and reads trips from Desktop 0.66.4
  onwards. Whatever happens at the Hill, Desktop brings the colony home exactly as it left.
- Kept souvenirs go home to Desktop's Journal. The Fairground's four (the rosette, the little
  brass bell, the teddy and the knot of rope) stay at the Hill until a Desktop release adopts
  them.
- Hill hosts one colony at a time. A trip that arrives while Hill is already open is refused, so
  Desktop can try again later.

### Downloads

- **macOS 14 or later:** a universal app, as a disk image or a zip.
- **Windows 10 or 11:** a per-user installer, or a portable zip.

Both put Hill where Formiga Desktop looks for it, and each comes with a SHA-256 checksum.

### Known limitations

A first release leaves a few things rough, and these are the ones worth knowing about:

- The downloads are not signed or notarised yet. On macOS, open the app the first time with
  Control-click → Open; on Windows, SmartScreen may ask before it runs.
- The music and sounds are placeholders, made in code.
- Hill's own text is in English only.
- Community packages can add stories, but not yet anything else.

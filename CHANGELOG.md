# Changelog

What changed in each version of Formiga Hill. A version's section here is the release's notes:
`.github/workflows/release.yml` publishes it with the downloads when the version is tagged.

## 0.1.0 (unreleased)

The first release. Formiga Hill receives a colony from Formiga Desktop by train, draws each
companion exactly as Desktop does, and gives the same creatures a storybook place to spend time
together.

### The places

- **The station.** The train pulls in with everyone at a window. The display case keeps the
  colony's souvenirs. The notice board pins up whatever is worth telling, and the departures
  board lists every place, what's on there and who's keen to go.
- **The Village Green.** Free play read from each companion's temperament, habits, pace, family
  and friendships. Pat, offer a snack, throw a toy or groom with the brush, and the dress-up box
  lends eight costume pieces for the visit.
- **The Clubhouse.** Stories by the fire, picked from its notice board: *The First Picnic*, *The
  Book with No Ending* and *The Last Bun*. Anyone can write one as a sandboxed package
  ([PACKAGES.md](docs/PACKAGES.md)) and drop it in the packages folder.
- **The Fairground.** Five games the colony plays while you watch: hide-and-seek, the sack race,
  the high striker, hoopla and tug-of-war. Each keeps a souvenir the first time, and its records.
- **The Woods.** Played by you, with a companion or two, for finds to take home: rummaging in the
  glade, fishing at the pool, bug catching in the meadow, foraging along the hedgerow, scavenging
  and treasure maps on the old track, and expeditions across a map of the Woods to the Far Falls.
  And somewhere, a secret.
- **The Hilltop.** The colony's own place, empty at first. Arrange finds on its eighteen spots,
  plant things that grow with every visit, and build grander things from plans, which come apart
  again whenever you like. It shows on the skylines of the other places.

### Everywhere

- The Hill's light follows your clock.
- A camera, and an album for each colony.
- Music and sounds, with volumes for the music and the sounds, and a mute (M).
- Desktop's reduced motion, theme and text size carry over. Reduced motion gets held poses and cuts
  rather than slowed-down animation.

### With Formiga Desktop

- Travel version 3, from Formiga Desktop 0.66.6. Hill reads trips from Desktop 0.66.4 onwards.
  Whatever happens at the Hill, Desktop brings the colony home exactly as it left.
- Kept souvenirs go home to Desktop's Journal. The Fairground's four (the rosette, the little brass
  bell, the teddy and the knot of rope) stay at the Hill until a Desktop release adopts them.
- One colony at a time: a trip that arrives while Hill is open is refused, so Desktop can try
  again later.

### Downloads

- **macOS 14 or later:** a universal app, as a disk image or a zip.
- **Windows 10 or 11:** a per-user installer, or a portable zip.

Both put Hill where Formiga Desktop looks for it, and each comes with a SHA-256 checksum.

### Known limitations

- The downloads aren't signed or notarised yet. On macOS, open the app the first time with
  Control-click → Open; on Windows, SmartScreen may ask before it runs.
- The music and sounds are placeholders, made in code.
- Hill's own text is in English only.
- Community packages can add stories, but not yet anything else.

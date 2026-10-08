# Changelog

This page records what changed in each version of Formiga Hill. Each version's section doubles as
its release notes: `.github/workflows/release.yml` publishes it alongside the downloads when the
version is tagged.

## 0.67.5 (2026-10-08)

Hill is built on Formiga Desktop 0.67.5 and still speaks travel version 4, so it takes the same
trips as before.

### On the scene

- Objects whose parts did not quite meet are drawn as one piece, about 25 joins in all across the
  Clubhouse, the Station, the Hilltop, the green, the Fairground, the Falls and the Woods map. The
  armchairs' seat band no longer runs across their arms, the bookshelf stands clear of the
  skirting, the painting over the mantel stays inside its frame, no bottle sits on top of a lantern
  on the lantern tree, and the burrow house's porch post no longer runs through its round window.
- The train at the station reads as one piece. The engine's side tank runs up to the smokebox and
  the smokebox comes down to the running plate, and the coaches' doors and ends are lined in cream
  above the waist and maroon below. The train is drawn by Formiga Desktop, so this comes with
  Desktop 0.67.5.

## 0.67.3 (2026-10-06)

Hill's version now matches Formiga Desktop's, so the apps released together carry one number. Hill
is built on Formiga Desktop 0.67.3 and still speaks travel version 4, so it takes the same trips
as before.

### On the scene

- Hill's controls sit on the scene itself, on paper and in Formiga's own pixel lettering, instead
  of in a grey bar underneath. Buttons line the bottom edge: what can be done here on the left, and
  Go to, sound, the album and the camera on the right.
- The notice boards, the journal, the album, the dress-up box and the sound levels open as cards
  over the scene's top-right corner, one at a time, instead of separate windows. Choices such as
  where to go, which game to play, or who is It open as menus from a paper button.
- In the Clubhouse, each line of a story appears in a speech box over whoever says it, with their
  name on a tab, and narration is a caption at the top.
- The lettering stays crisp at any window size and at Desktop's text scale, and the paper looks
  the same in light and dark.

### Movement

- Idle companions keep moving however long a visit lasts. Before, after about a minute and a half
  of standing about, every idle companion froze on one frame, which showed most at the station.
- The station is drawn smoothly, so its idles and chimney smoke no longer step unevenly.
- In free play nobody stands on top of anyone else. A companion headed for a spot that is taken
  stands beside it, and one that ends up over someone steps aside. In the meadow, the helper on a
  bug hunt waits beside the one with the net instead of in its way.

## 0.1.1 (2026-10-06)

### With Formiga Desktop

- Hill is now built on Formiga Desktop 0.67.1 and speaks travel version 4. Desktop 0.67.0 adopts
  the Fairground's four souvenirs (the rosette, the little brass bell, the teddy and the knot of
  rope), so all eleven go home to its Journal. Hill 0.1.0 already sends those four to Desktop
  0.67.0; with Desktop 0.66.6 they stay at the Hill.
- The display case draws every souvenir with Desktop's own pictures, so each looks the same in both
  apps. Hill's own copies of the four Fairground pictures are gone.
- If Desktop calls the colony home just before Hill's window closes, Hill no longer answers with a
  receipt as well. Desktop brings everyone home as they left, and souvenirs kept on that visit go
  home with the next one.

### Downloads

- On Windows, Desktop finds Hill only when it is put there by the installer. The portable zip runs
  on its own, but Desktop does not look for it.

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

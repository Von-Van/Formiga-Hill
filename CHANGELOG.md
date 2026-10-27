# Changelog

This page records what changed in each version of Formiga Hill. Each version's section doubles as
its release notes: `.github/workflows/release.yml` publishes it alongside the downloads when the
version is tagged.

## 0.68.1 (2026-10-26)

Hill is built on Formiga Desktop 0.68.1 and still speaks travel version 6, so it takes the same
trips as 0.68.0.

- Every place but the station redraws its picture only when something on it changes, so a quiet
  scene costs less to keep open. The pictures are the same, pixel for pixel.
- Inside, the six Woods outings share one way of starting, carrying on and ending, and the app and
  its map editor are now tested without a window.

## 0.68.0 (2026-10-26)

Hill is built on Formiga Desktop 0.68.0 and speaks travel version 6, so a creature reshaped in
Formiga Farm comes to the Hill as itself. An older Formiga still takes the colony on the same
trips as before.

- Farm creatures are drawn as themselves everywhere Hill draws the colony: a whale, a turtle or a
  giraffe, not the nearest ordinary companion.
- Opened on its own, from Applications or the Start menu, Hill shows a welcome card saying your
  colony comes by train from Formiga, and offers a look around with a sample colony.
- Each visit a companion wishes for one of the things it likes best, and the wish comes true when
  the colony does it. The display case at the station opens a card of every souvenir kept and what
  is still to find, and the expeditions' chapter keeps its latest 40 pages.
- Four new Clubhouse stories give the four souvenirs no built-in story gave.
- The story builder and the map editor have Undo and Redo, with Command-Z and Command-Shift-Z (Ctrl
  on Windows).
- In the Woods, Left and Right step through what a click would choose and Enter chooses it; K or
  the new ? button opens a card of every key the Hill answers to.
- The Sound card is now Sound and play: "More time to catch" slows the Woods' catching for anyone
  who wants gentler timing, and text can be set larger than Desktop's.
- A Woods basket is brought home and kept however the colony leaves the Woods, including when Hill
  closes or Formiga calls everyone home.
- The Hill's memories are never lost in silence: a book from a newer Hill is left untouched, an
  unreadable one goes back to the copy kept as the visit began, and the station's notice board
  says what happened.
- Removing a place in the map editor can no longer crash Hill, the stars no longer show through
  anything new on the skyline at night, and a shared package whose names differ only in capitals
  is refused.
- Busy places take a quarter to two thirds less work each frame, with the same pictures.

## 0.67.14 (2026-10-17)

Hill is built on Formiga Desktop 0.67.14 and still speaks travel version 5, so it takes the same
trips as before.

- A shared story, adventure or competition file holding two names that differ only in capitals,
  or a name used for both a file and a folder, is turned away with a plain reason before anything
  is written, so the version already installed keeps working.
- A memory book Hill can't read is set aside under a name of its own each time, so an earlier one
  set aside is never written over.

## 0.67.11 (2026-10-16)

Hill is built on Formiga Desktop 0.67.11 and still speaks travel version 5, so it takes the same
trips as before.

- Every download carries Hill's MIT licence: inside the macOS app, and as License.txt on the disk
  image, in the portable Windows ZIP and beside the installed program.

## 0.67.9 (2026-10-16)

Hill is built on Formiga Desktop 0.67.9 and still speaks travel version 5, so it takes the same
trips as before. This version mends a few things found after 0.67.8.

- The dress-up box on the Green shows its shaded right wall again.
- With **Take a costume off** chosen, pointing at someone in a tall hat no longer makes the preview
  flicker.
- An adventure now arrives at the place it starts, so that place's words, an ending there or a
  signpost there happen as they should.
- An adventure's description and its names are checked as plain words, so `{leader}` no longer
  shows word for word before anyone leads. A package that lists its kinds of content must list
  stories to load stories.
- In the Fairground, only a package holding exactly one competition and nothing else is changed in
  place. Any other package is copied first, so saving never drops what else it held.
- Hill now has its MIT licence in the repository, and its README shows the station and the Green.

## 0.67.8 (2026-10-15)

Hill is built on Formiga Desktop 0.67.8 and still speaks travel version 5, so it takes the same
trips as before. Stories, Woods adventures and Fairground competitions can now be made at the Hill
itself, tried with whoever is visiting, and passed on to someone else as one file.

### Writing stories in the Clubhouse

- **Write a story** in the Clubhouse opens the story builder, which makes a story from plain
  choices rather than a file: who is in it and how each role is cast (the most curious, a close
  friend of the lead, a little one if there is one), its scenes in order, and each scene's steps,
  such as narration, a line said with a different version for any temperament, someone going
  somewhere, a feeling or a pose, play, a pause, a choice for whoever is watching, or a souvenir.
  Anything still missing is said in plain words, where it is.
- **Try it** plays the story straight away with whoever is visiting and keeps nothing. **Put on
  the board** adds it to the notice board for every colony, and **Share…** saves it as one file
  to give to someone. A story being written is kept as a draft as it goes, so nothing is lost by
  closing the card.
- Words are typed on the cards themselves, in Hill's own lettering, and keys typed there don't set
  off the Hill's shortcuts.

### Woods adventures

- Under **Adventure** in the Woods' tray, the colony can set off on an adventure: a short outing
  laid out on a map of its own from the Woods' places, such as a hedgerow, a glade, a signpost at
  a fork, a pool or the falls. Up to three companions come along, choosing at each fork and
  stopping wherever there is something to do, until they reach an ending or head home. Some places
  stay hidden in the mist until somebody gets there, and some paths are shortcuts only some
  company takes, such as a little one or two close friends together; there is always another way
  round.
- Hill has three adventures of its own: *The Lantern Walk*, *Up to the Singing Falls* and *The
  Owl's Question*.
- Whatever is found on the way comes by the Woods' usual rules and goes home in the satchel as
  from any expedition. Hill remembers how often each adventure has been played and finished.
- **Make an adventure** opens the map editor, to put places down, join them with paths, name
  everything, and see what is still to do. An adventure can be tried at once with whoever is
  visiting, and nothing found on a try is kept; then it can be kept in the Woods or saved as one
  file to share. Adventures being made are kept as drafts.

### Fairground competitions

- The Fairground's new **Competitions** card runs a competition: several of the Fairground's games
  one after another, for points. The colony plays each game exactly as it always does, and you
  watch. A slip on the tray says which game is next and how the points stand, a line after each
  game says who scored, and the last competition's result stays on a slip beside the button.
  **Call it off** stops one part way.
- Who plays each game is said without naming anyone, so a competition works in any colony:
  everyone, the few keenest on that game by their own temperaments, or the few with the most
  points so far, for a final. A game too few can play is passed over.
- Hill has two competitions of its own: *The Fairground Five*, every game once with everyone
  playing, and *Heats and a Final*, two heats for the keenest four and then sack races between the
  three in front, the first to 10 points winning.
- **Make a new one** chooses the games in order, the rounds of each, who plays, the points for each
  place and how it is won (the most points, or the first to a number), with how it would go with
  the colony visiting: who would play each game, and any game too few could play. A competition
  can be changed, copied from another (Hill's own included), passed on as a file, or removed.

### Sharing

- Anything made at the Hill, whether a story, an adventure or a competition, is passed on as one
  plain text file ending `.formiga-hill-share`. Something shared can only be played at the Hill:
  it can't run anything, look at your files, or change your colony.
- A shared file is opened from the Clubhouse's Story packages card, from **Open a shared one…** in
  the Woods or on the Competitions card, or by dropping it on the window. Whichever way, the same
  **Shared with you** card shows whose it is and what it holds before anything is kept: how each
  story begins and which souvenirs it can give, the shape of each adventure's map, and each
  competition's games. A story can be tried first; then it is added or turned away. Adding a newer
  version of something already here takes the old one's place.
- The Story packages card counts the stories, Woods adventures and competitions in each package,
  and **Take out…** removes one that was added.
- For anyone writing by hand, the packages guide (`docs/PACKAGES.md` in this repository) now
  describes adventure and competition files beside stories, and `formiga-hill --share-package`
  writes a share file from a package folder.

## 0.67.7 (2026-10-15)

Hill is built on Formiga Desktop 0.67.7 and still speaks travel version 5, so it takes the same
trips as before.

### On the scene

- Whatever a companion arrives wearing from the desktop is drawn in what it is made of, with a
  few tones of shade, so an acorn cap is brown and a leaf hat green wherever it goes on the Hill.
  What companions wear from home is drawn by Formiga Desktop, so this comes with Desktop 0.67.7.

### In the window

- In the dress-up box on the Green, a costume picked out shows on whoever the pointer is over, so
  it can be seen on them before the click puts it on; with "Take a costume off" picked, they are
  shown without theirs. The box's pictures of each piece are half as large again.
- The journal opens on its contents: how much has been found, then a line for each chapter (each
  place in the Woods, the pool, the meadow, the hedgerow, expeditions, the old track, and the
  plans, relics and growing things once there are any) with how it stands. Choosing a line opens
  that chapter alone, with what has been found first and what is still to find gathered under it,
  each hint once with how many it stands for. Back, or Escape, returns to the contents where they
  were left, with the chapter just read marked; Escape there closes the journal.
- Hints, dates and what is not known yet are darker on the paper, so they read as easily as the
  rest: better than 6:1 and 5:1 where the faintest were 2.7:1.

## 0.67.6 (2026-10-12)

Hill is built on Formiga Desktop 0.67.6 and reads travel version 5, so a colony in Desktop's Echo
Fox Mode arrives as foxes. It takes the same trips as before.

### On the scene

- A colony that comes over while Desktop's Echo Fox Mode is on is a skulk of foxes here too: each
  companion Desktop draws as the Echo Fox is drawn as it at the station, on the green, in the
  Clubhouse, at the Fairground and in the Woods, and does everything it would do as itself. Its
  name, traits and friendships are its own, and it comes over as itself on the next trip after the
  mode is switched off.

### In the window

- The notice that appears over the trays at the bottom of the scene keeps its width. A long
  notice that followed a short one used to wrap to the short one's width, a column about five
  letters wide; it now runs up to its full width, and words on any slip over the scene wrap no
  narrower than twenty letters.

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

# Formiga Hill

Formiga Hill is a small desktop app for spending time with a Formiga colony somewhere other than
the desktop. In [Formiga Desktop](https://github.com/Von-Van/Formiga-Desktop), the creatures live
around the edges of the screen while you work. Hill is somewhere you send them on purpose. The
colony arrives by train, and Hill gives the same creatures a storybook village to spend time in
together: a green for free play, a clubhouse for stories, a fairground, the Woods, and a hilltop
that the colony slowly makes its own.

Hill is a separate app rather than a new part of Desktop, and that decision shapes most of what
follows. Desktop stays the home of the colony and the only place its records are kept. Hill
borrows the colony for a visit, draws each companion exactly as Desktop does, and sends everyone
home unchanged. Whatever happens at the Hill, even a crash, the colony goes back exactly as it
came.

## The idea

Desktop is good at quiet company, but a desktop overlay has some natural limits. The creatures
share the screen with everything else, so there is not much room for them to do things together,
and anything they do has to stay out of the way. Hill began as a way to explore what the same
colony might do with a place of its own. The question was less about adding features and more
about whether the personalities Desktop already tracks could carry a whole set of activities
without anything being written for one particular creature.

So far that seems to hold up. Everything a companion does at the Hill is read from what Desktop
sends: its temperament, habits, pace, family and friendships. A curious one peeks under things
first, an impulsive one jumps the start of a race, and a parent helps its little one ring the bell.
Two colonies will probably never play out quite alike, and a story or a game only has to be
written once rather than once for every creature.

A few other ideas guide the design:

- **Soft play.** There is nothing to keep up and nothing lost by staying away. There are no streaks,
  no hunger, and no reminders of how long it has been. Things that grow on the Hilltop simply wait.
- **Who plays decides where a game goes.** Some activities are watched and some are played. At the
  Fairground the colony plays and you watch. In the Woods you play, with a companion or two, and
  what you find there builds the Hilltop. Keeping the two apart lets watching stay relaxing and
  lets playing actually matter.
- **The same creatures, not lookalikes.** Hill draws each traveller from the appearance Desktop
  sends, using Desktop's own art code. It never makes up a new one.
- **Open to other people's stories.** Anyone can write a story for the Clubhouse as a small folder
  of text files, with no code involved ([docs/PACKAGES.md](docs/PACKAGES.md)).

## A visit

A visit starts at the station and moves between places from the departures board. What follows is
a short tour; the detail is in the app itself.

**The station.** The train pulls in with everyone at a window, and they hop down onto the platform
one by one. Hovering over a companion shows who it is: temperament, traits, habits, family and
closest friend. The notice board pins up whatever is worth telling, such as what each companion
would like to do, stories not yet told, and finds still waiting for a place. A display case in the
station house holds the souvenirs the colony has kept. "Take the train home" ends the visit, and
every kept souvenir goes home with the colony to the Journal in Formiga Desktop (0.66.6 and later,
or 0.67.0 and later for the Fairground's four newest).

**The Village Green.** A lawn with an old oak and its swing, a well, a picnic blanket and a toy
chest. The colony wanders, naps in the shade, visits friends and keeps apart from rivals, while
the little ones trail their parents. You can pat, offer a snack, throw a toy or groom with the
brush, and each companion answers in its own way and warms to you over the visit. A dress-up box
lends hats, daisy chains and scarves for the visit only; Desktop's own accessories are never
touched.

**The Clubhouse.** Stories are played out by the fire, each chosen from the Clubhouse's notice
board. Hill comes with three: *The First Picnic*, *The Book with No Ending*, and *The Last Bun*, a
small mystery in which whoever likes a mystery takes the case. Casting is the interesting part. A
story describes the kind of companion each role needs (the most curious, a close friend of the
lead, a little one if there is one), and Hill fills the roles from whoever happens to be visiting.

**The Fairground.** A big top, a carousel, a hoopla stall and a high striker, with five games the
colony plays among themselves while you watch: hide-and-seek, the sack race, the high striker,
hoopla and tug-of-war. You choose the game and, at most, who plays. How each game goes comes from
the players rather than from chance alone. In the sack race, for example, a lazybones may sit down
for a breather, and a show-off may wave to the crowd just short of the line and be overtaken. In
tug-of-war the colony sorts itself into sides, friends together and rivals apart, and two who don't
get on pull out of step. Each game keeps a souvenir the first time it is seen through, and its
records go up on the station's boards.

**The Woods.** This is where you play, with a companion or two, for finds to take home. Who comes
along changes how an outing goes and what turns up, but it never rules anything out: every find
can be found by any single companion.

- **Rummaging in the glade.** Read the signs at twelve spots, then catch the moment on a turning
  ring before the light goes.
- **Fishing at the pool.** Eight kinds of fish, each with its own haunt and its own way of biting.
- **Bug catching in the meadow.** Eleven bugs. Creep up, keep still, and swing when one leaves
  itself open.
- **Foraging along the hedgerow.** Everything ripens and goes over during the outing, read by its
  look, and only so much fits in the basket.
- **Scavenging along the old track.** Lift things off each heap in the right order, since a heap
  shifts as it is moved and anything fragile it jolts can crack.
- **Treasure maps.** Torn maps turn up now and then, each sketching a route in landmarks and turns
  that ends at a buried chest.
- **Expeditions.** A whole day out on the Woods' map with up to three companions. Every path
  spends some of the day's light, and the stops along the way offer a short go at the other
  activities, a heap or two on the old track, or a picnic. Past the signpost, the Far Falls bring
  down things found nowhere else.

There is also a secret somewhere in the Woods, played completely straight. How to find it is left
out of this page on purpose.

**The Hilltop.** The colony's own place, which starts out empty. Any find can stand on any of its
eighteen spots, and the colony goes to visit whatever stands there. Seeds and cuttings grow a stage
with every visit, and the colony tends them, each in its own way. Finds also bring plans to mind
for grander things, twelve in all, from a tea party to the great telescope, and the colony gathers
round to help build them. A plan can always be taken apart again, which gives every find back. The
journal records every find and who found it first, and the Hilltop itself shows on the skyline
from the other places. What stands there also changes the Woods a little: a lantern lends light,
and a telescope brings rare signs out sooner.

**Everywhere.**

- The Hill's light follows your computer's clock, so a late visit finds it under the stars, with
  lamps, lit windows and fireflies. Nothing in play depends on the hour.
- Each place has its own soft music, quieter once the lamps are lit, and most things that happen
  have a sound. All of it is placeholder audio made in code for now. Sound is never needed to play.
- The camera (C) frames part of any scene and keeps the photo in the colony's album.
- Desktop's reduced motion, theme and text size carry over. With reduced motion, Hill uses held
  poses and cuts rather than simply slowing the animation down.

### Controls

| Where | Keys |
| --- | --- |
| The station | Click, Space, Enter or Esc to skip the arrival |
| Free play (the green, the Clubhouse, the Fairground) | 1 to 4: pat, snack, toy, brush |
| A story | Space or Enter to go on; 1 to 4 to choose |
| The Fairground | Esc calls a game off |
| The Woods | Space to try, to strike, or to swing; hold the pointer, ↑ or W to creep up on a bug |
| The Hilltop | Esc puts down whatever you are holding |
| Anywhere | C for the camera, and Esc to put it away; M mutes the sound, or brings it back |

## Getting there

1. Download Hill from the repository's Releases page. On macOS 14 or later there is a universal
   app, as a disk image or a zip. On Windows 10 or 11 there is a per-user installer, or a portable
   zip. Both put Hill where Formiga Desktop looks for it. (To build it yourself, see
   [docs/DEVELOPING.md](docs/DEVELOPING.md).)
2. In Formiga Desktop (0.66.4 or later), choose "Go to Formiga Hill…" from the tray.
3. Desktop's train carries the colony off the desktop, and Hill opens at the station.

The downloads are not signed yet. On macOS, open the app the first time with Control-click → Open;
on Windows, SmartScreen may ask before it runs.

Hill hosts one colony at a time. A trip that arrives while a Hill window is already open is
politely refused, so Desktop can try again later. If Hill closes unexpectedly, or is force-quit,
Desktop brings the colony home exactly as it left.

### What Hill keeps

Hill keeps a few records of its own, and only its own. It never reads or writes Desktop's files,
and it knows each colony only by a one-way id that Desktop sends with the trip, so it never learns
more about a colony than the trip itself told it. The records live in Hill's data folder:

- macOS: `~/Library/Application Support/com.Formiga.Formiga-Hill`
- Windows: `%APPDATA%\Formiga\Formiga Hill\data`
- or wherever `FORMIGA_HILL_DATA_DIR` says.

| File | What it holds |
| --- | --- |
| `memories.json` | Each colony's finished stories, kept souvenirs, finds, journal, Hilltop and records |
| `photos/` | Each colony's album |
| `packages/` | Community story packages |
| `packages.json` | Which of them are set aside |
| `settings.json` | Music and sound levels, and whether they are muted, for the Hill as a whole |
| `hosting.lock` | Held while a window is open |

## Where things stand

Version 0.1.0 is the first release, and a fair amount is still open. The music and sounds are
placeholders. Hill's own text is in English only. Community packages can add stories, but not yet
anything else, such as new games or places, and it is not yet clear what the right shape for those
would be. The Fairground's four souvenirs also stay at the Hill for now, until a Desktop release
learns to show them in its Journal. The [changelog](CHANGELOG.md) keeps track of what changes from
one version to the next.

## Further reading

- [docs/PACKAGES.md](docs/PACKAGES.md): writing a story for Hill, no code needed.
- [docs/DEVELOPING.md](docs/DEVELOPING.md): building, testing and finding your way around the code.

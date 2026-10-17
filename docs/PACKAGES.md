# Writing for Formiga Hill

A story is a short scene, or a few, that the colony plays out in the Clubhouse at the Hill, a
snug room with a fire where the person picks a story from the notice board. Stories are written as
plain TOML files in a folder. There is no code to write and nothing to compile, and a package can
do nothing except tell Hill who should do what, where, and what they say. That limit is
deliberate: it keeps community stories safe to share and lets Hill check every one before it is
played.

The main difference from writing an ordinary script is that a story never knows who will be in
it. Each colony is different, so instead of naming characters, a story describes the kind of
companion each role needs, such as the most curious one or a close friend of the lead, and Hill
casts the roles from whoever happens to be visiting. The same story can therefore play out quite
differently from one colony to the next, and part of the fun is writing with that in mind.

Hill's own stories, *The First Picnic*, *The Last Bun* and *The Book with No Ending*, are written
exactly this way, and each is a complete working example in
[`crates/formiga-hill/content`](../crates/formiga-hill/content). *The Last Bun* is probably the
best place to start, since it shows a mystery whose suspects are cast from the colony's own
habits, temperaments and friendships.

A package can carry Woods adventures and Fairground competitions too, in the same folder and
format, and much of what follows holds for them as it does for stories;
[A Woods adventure](#a-woods-adventure) and [A Fairground competition](#a-fairground-competition)
say what is different.

## Writing a story in Hill

The Clubhouse's **Write a story** button opens the story builder, which writes a package in exactly
this format without anyone having to see it. A story is built from plain choices: who is in it and
how each role is cast, its scenes in order, and each scene's steps, such as someone going to the
rug, a line said (with a different version for any temperament), a feeling shown, or a choice for
whoever is watching. **Try it** plays the story straight away with whoever is visiting and keeps
nothing; **Put on the board** adds it to the packages folder, for every colony; **Share…** saves it
as one file to pass on. Stories being written are kept as drafts in Hill's data folder, under
`drafts/stories/`, as they are written.

The builder says less than a story file can, on purpose: each role is cast by one choice (with
`any` behind it unless it is left out when nobody fits), scenes follow one another unless one says
where to go next, and a choice is the only way a story forks. Anything it writes can be opened and
carried on by hand.

## Sharing a story, or anything else made at the Hill

A story, an adventure, a competition, or anything else a package carries, is passed on as one
plain text file ending `.formiga-hill-share`: the package's files, each written out as text in a
TOML table, under `formiga_hill_share = 1`.

```toml
formiga_hill_share = 1

[files]
"manifest.toml" = "package_id = \"org.example.lost-kite\"\n…"
"content/lost-kite.toml" = "…"
"localization/en.toml" = "…"
```

Hill reads at most 2 MiB of one, holds its files to exactly the limits of a package folder below,
and loads the package by the same loader before anything is written. There is one way in,
whatever the file holds: the Clubhouse's **Story packages…** card (**Open a shared story…**), the
Woods' tray under **Adventure** and the Fairground's **Competitions** card (each **Open a shared
one…**), or dropping the file on the window, all open the same **Shared with you** card. It shows
whose it is, what it holds, how each story begins and which souvenirs it can give, each adventure's
map, and each competition's games, with ways to try a story first, add it, or turn it away.
Adding it puts the package in the packages folder as `<package_id>.formiga-hill`, in place of any
earlier version with the same id. Ids beginning `com.formiga.` are kept for Hill's own packages and
are refused. **Take out…** on the Story packages card removes a package from the folder again, and
a package holding only competitions can be removed from the Competitions card too.

`formiga-hill --share-package my-story.formiga-hill my-story.formiga-hill-share` writes a share
file from a package folder, and `--check-package` checks a share file as it does a folder.

## Trying a story

The quickest way to work on a story is to check it, play it, and look at a moment of it, all
from the command line:

```sh
formiga-hill --check-package my-story.formiga-hill        # does it load? what is wrong?
formiga-hill --package my-story.formiga-hill               # play it, from the notice board
formiga-hill --package my-story.formiga-hill --render-story moment.png --at 20
```

`--check-package` names the file, scene and beat of anything wrong, in words like
`scene "spread", beat 2: "the bandstand" is not a place in the clubhouse`.

## Installing a story

Put the package's folder (the one ending `.formiga-hill`) in Hill's packages folder, and its
stories are on the notice board next time the colony visits. `formiga-hill --packages-folder` says
where the folder is and how each package in it fares. On macOS it is
`~/Library/Application Support/com.Formiga.Formiga-Hill/packages`.

The notice board's **Story packages…** button lists every package Hill found, with anything that
would not load and why. Untick a package to set it aside: its stories come off the board until it
is ticked again, and the package itself is never touched. Hill reads at most 64 packages from the
folder, and only folders: links are refused, as they are inside a package.

## The folder

```text
my-story.formiga-hill/
  manifest.toml
  content/
    my-story.toml          # one file per story
  localization/
    en.toml                # every word the story says
  README.md                # optional; Hill ignores it
```

Only `.toml`, `.md` and `.txt` files are allowed. File names use letters, digits, `.`, `-` and
`_`, and no two names in a package may differ only in capitals, which most computers keep as one
file. Links aren't allowed, folders nest at most three deep, a file is at most 128 KiB, and a whole
package at most 1 MiB.

## `manifest.toml`

```toml
package_id = "org.example.lost-kite"   # yours: lowercase, with at least one dot
title = "The Lost Kite"
author = "Your name"
version = "1.0.0"
hill_api = 2                           # the content API you wrote for
content_types = ["story"]
entry_points = ["content/lost-kite.toml"]
default_locale = "en"

[requirements]
min_cast = 2           # the fewest travellers it can be played with
areas = ["clubhouse"]  # where it is staged
```

Hill refuses a package that:
- asks for `permissions`;
- needs a `capabilities` entry Hill doesn't have;
- is written for a newer `hill_api`;
- has an entry point outside `content/`.

Hill ignores fields it doesn't know, so a package written for a later Hill can still load.

### Written for content API 1

Content API 2 moved stories indoors, to the Clubhouse. A package written for API 1, when stories
were staged on the green, still loads as it was written, with `area = "green"` and the green's
places, and is played in the Clubhouse. Each place on the green is read as its counterpart there:

| On the green | In the Clubhouse |
| --- | --- |
| `blanket` | `rug` |
| `well` | `hearth` |
| `oak` | `bookshelf` |
| `swing` | `armchair` |
| `chest`, `centre`, `left`, `right`, `front`, `back` | the same |

`--check-package` says how such a story is read. To use the Clubhouse's own places, write for
`hill_api = 2` and `area = "clubhouse"`; a package speaks one API's places or the other's, never
both.

## A story file

A story file has three parts: the story itself, the roles it needs, and its scenes, each of which
is a list of small steps called beats. A short example looks like this:

```toml
[story]
id = "lost-kite"
title = "title"          # a line in the localisation file
area = "clubhouse"
start = "found"          # the first scene

[roles.finder]
order = 1
select = ["most:curiosity", "any"]

[roles.helper]
order = 2
select = ["friend_of:finder", "any"]

[roles.worrier]
order = 3
select = ["least:boldness"]
optional = true

[[scenes]]
id = "found"

[[scenes.beats]]
walk = "finder"
to = "bookshelf"

[[scenes.beats]]
say = "finder"
line = "found.look"
```

### Roles and casting

A story never assumes what a colony is like: it doesn't know how many travellers came, or what
temperaments, minis or friendships they have. Each role lists ways of choosing someone, tried in
order, and the first that finds someone not already cast wins. Roles are cast in `order`.

| Selector | Chooses |
| --- | --- |
| `any` | whoever is left first |
| `random` | someone left, the same one each time for the same colony |
| `most:<scale>`, `least:<scale>` | the highest or lowest on a temperament scale: `social`, `energy`, `boldness`, `playfulness`, `curiosity`, `feistiness`, `impulsiveness`, `suspicion`, `affection` |
| `kind:<temperament>` | `sweetheart`, `troublemaker`, `grump`, `explorer`, `wallflower`, `showoff`, `scholar`, `oddball`, `lazybones`, `guardian` |
| `trait:<trait>` | a traveller whose profile shows that trait, such as `trait:brave` or `trait:night_owl` (see below) |
| `habit:<habit>` | `looks_food_over`, `stretches_before_naps`, `circles_before_naps`, `waves_hello`, `play_bows` |
| `friend_of:<role>` | that role's closest friend |
| `playmate_of:<role>` | someone that role plays with |
| `rival_of:<role>` | someone that role doesn't get on with |
| `parent_of:<role>`, `mini_of:<role>` | family |
| `mini`, `adult` | any little one, or any grown one |

Traits are named by Desktop's identifiers, which stay the same however Desktop words or
translates them. The way a profile spells one (`trait:Brave`, `trait:Night owl`) is still
accepted, so older stories keep working. The traits are:

`outgoing`, `chatty`, `clingy`, `loner`, `energetic`, `excitable`, `restless`, `mellow`,
`sleepy`, `lazy`, `brave`, `confident`, `reckless`, `cautious`, `shy`, `nervous`, `cowardly`,
`playful`, `mischievous`, `silly`, `serious`, `stoic`, `grumpy`, `feisty`, `stubborn`, `bossy`,
`irritable`, `competitive`, `gentle`, `sweet`, `easygoing`, `impulsive`, `impatient`,
`distractible`, `spontaneous`, `patient`, `steady`, `particular`, `picky`, `suspicious`, `wary`,
`watchful`, `trusting`, `friendly`, `affectionate`, `loyal`, `jealous`, `protective`,
`nurturing`, `independent`, `aloof`, `curious`, `nosy`, `obsessive`, `observant`, `unbothered`,
`dramatic`, `vain`, `attention_seeking`, `food_motivated`, `messy`, `eccentric`,
`unpredictable`, `night_owl`, `early_riser`, `easily_embarrassed`, `secretly_affectionate`,
`deeply_loyal`

A role must either end its list with `any` or `random`, or be `optional = true`. An optional
role nobody fits is left empty, and every beat about it is quietly skipped. That includes any
line that names it.

### Beats

A scene is a list of beats, played in order. Each beat does one thing.

| Beat | Does |
| --- | --- |
| `walk = "<who>"`, `to = "<place>"` | walks there. Places: `rug` (before the fire), `hearth`, `armchair`, `bookshelf`, `window`, `board` (the notice board), `table`, `chest` (the toy box), `centre`, `left`, `right`, `front`, `back`, or `beside:<role>` |
| `face = "<who>"`, `to = "<role>"` | turns to face someone |
| `react = "<who>"`, `feeling = "<feeling>"` | `joy`, `surprise`, `worry`, `fond`, `proud`, `sleepy`, `grumpy`, `curious`, `bored`, `shy`, shown the way that creature would show it |
| `pose = "<who>"`, `as = "<pose>"` | `inspect`, `sit`, `crouch`, `stretch`, `wave`, `peek`, `strut`, `cheer`, `dance`, `huff`, `beg`, `watch`, `cover`, `balance` |
| `eat`, `nap`, `celebrate` `= "<who>"` | eats a snack; naps (with a nap habit, if it has one); celebrates in its own way |
| `play = "<role>"`, `with = "<role>"` | the two play together |
| `say = "<role>"`, `line = "<key>"` | the role says a line; the person reads on when ready |
| `narrate = "<key>"` (optional `about = "<role>"`) | a line with no speaker, varied by `about`'s temperament |
| `wait = <seconds>` | a pause, up to 30 seconds |
| `choose = [{ line = "<key>", goto = "<scene>" }, …]` | up to four choices for the person |
| `goto = "<scene>"` | carries on from another scene |
| `if = "<condition>"`, `goto = "<scene>"` | goes there if the condition holds |
| `set = "<flag>"` | remembers something for later in this playing |
| `souvenir = "<id>"` | gives one of Hill's souvenirs (below) |
| `end = true` | the end; a scene that runs out of beats also ends the story |

`<who>` is a role, or `all` for everyone cast.

Each beat starts once the beats before it have finished. Add `meanwhile = true` to start it at
the same time as the beat before, in the background. A later beat only waits for a background
beat if it involves the same traveller. Use this to have several people walk at once, or to show
a line while someone moves.

Add `when = "<condition>"` to any beat to play it only when the condition holds.

### Conditions

| Condition | Holds when |
| --- | --- |
| `flag:<name>` | the flag was `set` earlier in this playing |
| `present:<role>` | someone was cast in that role |
| `mini:<role>` | that role is played by a little one |
| `kind:<role>=<temperament>` | that role has that temperament |
| `trait:<role>=<trait>`, `habit:<role>=<habit>` | that role has that trait or habit |
| `high:<role>.<scale>`, `low:<role>.<scale>` | that role is high (at least 0.65) or low (at most 0.35) on a scale |
| `close:<role>,<role>` | the two are close friends |
| `rivals:<role>,<role>` | the two don't get on |

Put `!` in front of any condition to mean its opposite: `!flag:shared`.

## Localisation

Every word lives in `localization/<locale>.toml`, never in the story file. Keeping the words
apart makes a story easier to translate, and it also lets a line change depending on who says it.
A line is plain text,
or a table with a `text` and a version for any temperament that would say it differently. Tables
without `text` are namespaces: `[found]` then `look = "…"` is the line `found.look`.

```toml
title = "The Lost Kite"

[found]
look = "A kite, caught in the oak!"

[found.thanks]
text = "Thank you, {helper}."
grump = "I could have got it myself, {helper}."
sweetheart = "{helper}, you're the best."
```

`{role}` is replaced by the name of whoever plays that role. Hill checks that:
- every line a story uses exists;
- every `{name}` in it is one of that story's roles;
- no line is longer than 280 characters, breaks onto a second line, or has a stray brace;
- every temperament version is spelt as one of the ten temperaments.

## Souvenirs

A story can give only Hill's own souvenirs, by id, and can't invent new ones. Those a story can
give are `picnic_ribbon`, `pressed_daisy`, `well_penny`, `oak_acorn`, `swing_feather` and
`chest_marble`. Some others, such as the Fairground's ticket, are only ever won at Hill's own
games.

Hill remembers which stories each colony has finished and what it has kept, and nothing is ever
lost by staying away. When the colony goes home, every souvenir it has kept goes home with it to
Formiga Desktop's Journal, whichever story or game gave it: from Desktop 0.66.6, or from 0.67.0 for
the Fairground's four newest.

## A Woods adventure

A package can also carry an adventure: a short outing in the Woods, laid out on a map of its own
and walked the way an expedition is. The person picks it from the Woods' tray under
**Adventure**, chooses up to three companions, and makes their way from place to place, choosing
at each fork and stopping where there is something to do, until they reach an ending or head
home. Hill's own, *The Lantern Walk*, *Up to the Singing Falls* and *The Owl's Question*, are in
[`crates/formiga-hill/content`](../crates/formiga-hill/content) and written exactly this way.

There is no need to write one by hand. The map editor in the Woods (**Adventure**, then **Make
an adventure**) puts places down, joins them with paths and names everything, says what is still
missing, lets it be tried at once with whoever is visiting (nothing found on a try is kept), and
keeps it in the Woods or saves it as one file to share, as above. Adventures being made are kept as
drafts in Hill's data folder, under `drafts/adventures/`. What it writes is this format.

The manifest lists `"adventure"` in `content_types`, and the adventure's file in
`entry_points`. A package may carry stories and adventures together; each file says which it is
by its first table.

```toml
[adventure]
id = "lantern-walk"
title = "title"        # a line in the localisation file
about = "about"        # optional: shown when choosing it
start = "lane"         # where everyone comes in, up the lane from the Hill

[[places]]
id = "lane"
site = "edge"
at = [36, 186]         # where on the map, from [16, 48] to [368, 206]
arrive = "lane.arrive" # optional: said on getting here

[[places]]
id = "pool"
site = "pool"
at = [300, 180]
name = "pool.name"     # optional: otherwise "the pool"
hidden = true          # out of sight in the mist until somebody gets there
ending = "pool.ending" # getting here finishes the adventure; this is said

[[paths]]
between = ["lane", "pool"]
bends = [[150, 200]]   # optional, up to 8: where it bends on the way
length = 12            # optional: how much of the day it takes; otherwise as drawn

[[paths]]
between = ["lane", "pool"]
way = "little_one"     # a shortcut only some company takes
name = "ways.rabbit"   # what it is called
```

Each place is one of the Woods' own kinds, and what is done there is that place's own activity,
played as it always is:

| `site` | What it is | What is done there |
| --- | --- | --- |
| `edge` | A way in, with a stile | Nothing: it is where the lane comes in |
| `hedgerow` | A hedge with a gate | A patch of foraging |
| `meadow` | Open grass and flowers | Catching a bug |
| `glade` | A clearing under an oak | A few searches, rummaging |
| `log` | A fallen log | A picnic, a rest |
| `signpost` | An old signpost at a fork | Nothing, but someone who reads old signs may read it |
| `pool` | A pond | A cast or two |
| `old_track` | An old cart track and the woodcutter's hut | A heap or two to scavenge |
| `falls` | Falls coming down a crag | Wading in for what the water brings |

A path is open to anyone unless its `way` says otherwise: `explorer` (an explorer, or anyone very
curious), `little_one`, `close_pair` (two close friends together), `bold`, or `reader` (someone
who reads old signs, once they have read a signpost at one end of it, which it must start from).
A way only some company takes is a shortcut, never the only way: whoever comes, the adventure
has to be playable.

Hill refuses an adventure, saying why, if:
- any place can't be reached from the start along open paths, by anyone on their own;
- no ending is near enough to reach in a day's light, at the slowest pace;
- it has no ending, its start is hidden, or a path names a place that isn't there;
- it has more than 16 places or 32 paths, or places stand closer than 22 apart, or off the map.

An adventure's words follow the same rules as a story's (see Localisation), with one name to use:
`{leader}`, whoever leads the party. A line can have a version for each temperament, told in the
leader's. What it is about is shown before anyone leads, so it is plain and names nobody; its
title and place and path names are plain, short and name nobody too.

An adventure gives nothing of its own. Whatever is found, caught or turned up on the way comes by
the Woods' usual rules and goes home in the satchel as from any expedition; Hill remembers how
often each adventure has been played and finished, and that is all.

## A Fairground competition

A package can also carry competitions for the Fairground: a run of its own games, one after
another, for points. The colony plays each game exactly as it always does, and the person
watches. Hill's own, *The Fairground Five* and *Heats and a Final*, are in
[`crates/formiga-hill/content`](../crates/formiga-hill/content).

There is no need to write one by hand either. The Fairground's **Competitions** card makes them
(**Make a new one**: the games in order, who plays each, the points and how it is won, with how it
would go with the colony visiting), changes or copies them, and passes one on as a file to share,
as above. A competition made there is kept as a package in the packages folder, like one added
from a file. What it writes is this format.

The manifest lists `"competition"` in `content_types`, and each competition's own file in
`entry_points`. A package may carry competitions beside stories and adventures; each file says
which it is by its first table.

```toml
[competition]
id = "summer-games"
title = "title"          # a line in the localisation file: up to 40 characters, naming nobody
points = [3, 2, 1]       # for 1st, 2nd and 3rd, from 1 to 6 places, each worth up to 10
win = "most-points"      # or "first-to:10": the first to that many points, up to 99

[[events]]               # from 1 to 8, played in order
game = "sack-race"       # hide-and-seek, sack-race, high-striker, hoopla or tug-of-war
rounds = 2               # from 1 to 3; 1 if left out
players = "everyone"     # or "keenest:4", or "leaders:3"; everyone if left out
```

Who plays is said without naming anyone, so a competition means something in any colony:
everyone, the few keenest on that game by their own temperaments, or the few with the most points
so far, for a final. A game finishes in places: the order home in the sack race, the mark on the
striker, rings rung at hoopla (a ring or more to be placed), the best hidden at hide-and-seek, and
the winning side of the tug-of-war. Those level share a place, and the next place is as far down as
there were of them. A game too few can play is passed over. Unknown fields in a competition file
are refused, so nothing in one can say anything about a companion.

## What packages can't do

Packages can't run code, read or write files, reach the network, start programs, change anything
in Formiga Desktop, rewrite a companion's temperament, or make up history for it. These limits are
what make it reasonable to install a story, an adventure or a competition from someone you don't
know: the worst a package can do is tell a story badly, lay out a dull walk, or order the
Fairground's games dully. A package that fails to load is reported and left out, and it never stops
Hill from opening.

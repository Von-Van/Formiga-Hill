# Writing a story for Formiga Hill

A story is a short scene, or a few, that the colony plays out in the Clubhouse at the Hill: a
snug room with a fire, where the person picks a story from the notice board. You write it as plain
TOML files in a folder. There is no code to write, nothing to compile, and nothing your package
can do except tell Hill who should do what, where, and what they say.

Hill's own stories, *The First Picnic*, *The Last Bun* and *The Book with No Ending*, are written
exactly this way. Each is a complete working example in
[`crates/formiga-hill/content`](../crates/formiga-hill/content); *The Last Bun* shows a mystery
whose suspects are cast from the colony's own habits, temperaments and friendships.

## Trying a story

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
`_`. Links aren't allowed, folders nest at most three deep, a file is at most 128 KiB, and a whole
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
| `trait:<Trait>` | a traveller whose profile names that trait, such as `trait:Brave` |
| `habit:<habit>` | `looks_food_over`, `stretches_before_naps`, `circles_before_naps`, `waves_hello`, `play_bows` |
| `friend_of:<role>` | that role's closest friend |
| `playmate_of:<role>` | someone that role plays with |
| `rival_of:<role>` | someone that role doesn't get on with |
| `parent_of:<role>`, `mini_of:<role>` | family |
| `mini`, `adult` | any little one, or any grown one |

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
| `trait:<role>=<Trait>`, `habit:<role>=<habit>` | that role has that trait or habit |
| `high:<role>.<scale>`, `low:<role>.<scale>` | that role is high (at least 0.65) or low (at most 0.35) on a scale |
| `close:<role>,<role>` | the two are close friends |
| `rivals:<role>,<role>` | the two don't get on |

Put `!` in front of any condition to mean its opposite: `!flag:shared`.

## Localisation

Every word lives in `localization/<locale>.toml`, never in the story file. A line is plain text,
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
lost by staying away.

## What packages can't do

Packages can't run code, read or write files, reach the network, start programs, change anything
in Formiga Desktop, rewrite a companion's temperament, or make up history for it. A package that
fails to load is reported and left out. It never stops Hill from opening.

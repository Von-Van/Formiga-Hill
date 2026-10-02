# Formiga Hill: design handoff

Product, technical, and community-content design specification. Written against Formiga Desktop
0.66.1 (save schema v24), 2 October 2026. This is the founding brief for the project; where the
code has since made a decision this document left open, [TRAVEL.md](TRAVEL.md) and the README
record it.

> **Product premise.** Formiga Desktop is where the creatures live around the user. Formiga Hill
> is somewhere the user deliberately sends them to spend time together.

## 1. Purpose and scope

Formiga Hill is a separate desktop application that imports a user's existing Formiga colony and
uses those creatures as the cast of a more intentional, interactive experience. It should feel
like a destination in the same ecosystem rather than a second copy of Formiga Desktop.

Hill is deliberately broader than a single genre. Its persistent hub can support free-form
pet-sim interaction, authored visual-novel-like scenes, repeatable minigames, small exploration
activities, and later community-authored content. The unifying feature is not the activity type;
it is that the user's own Formiga creatures retain their identity and behavior across all of them.

### Goals

- Make the handoff from Desktop to Hill feel diegetic: the colony gathers, a small train arrives,
  the creatures board, the train leaves the desktop, and Hill opens at its station.
- Preserve creature identity: appearance, name, temperament, learned habits, family ties,
  meaningful relationship context, and selected cosmetic state should matter inside Hill.
- Support several activity styles under one coherent hub rather than forcing Hill to choose
  between pet sim, visual novel, or minigame collection.
- Be friendly to community content from the beginning through a safe, declarative package format
  and a stable content API.
- Avoid turning Hill into an obligation loop. No neglect punishment, streak pressure, hunger debt,
  or "you failed to log in" mechanics.
- Keep Formiga Desktop authoritative for the colony's canonical identity and long-lived desktop
  simulation.

### Non-goals for the first releases

- A full RPG, large open world, combat system, or crafting economy.
- Arbitrary third-party code execution as part of mods.
- Direct editing of Formiga Desktop's `colony.json` from Hill.
- A requirement that every Desktop system be simulated while the colony is visiting Hill.
- A large creator IDE before the runtime and package format prove themselves.

## 2. Core experience

### The train handoff

1. The user chooses "Go to Formiga Hill…" from Formiga Desktop.
2. Desktop saves the colony and creates a versioned travel snapshot.
3. Creatures finish or safely interrupt incompatible runtime actions, gather near the bottom of
   the active display, and react to the arrival.
4. A small Formiga-scale train enters. Boarding reactions can vary by temperament, habits,
   age/mini status, and relationships.
5. The creatures board; the train leaves the desktop. Desktop hides the colony while the Hill
   session is active.
6. Formiga Hill opens on, or transitions into, its station arrival scene using the same
   travel-session identifier.
7. When leaving Hill, the return trip mirrors the departure. Desktop receives a constrained return
   receipt and resumes the colony safely even if Hill closes unexpectedly.

> The train is not just launch chrome. It is the fiction that makes two executables feel like one
> world.

### Hill as a place

The first Hill should be compact and readable as a storybook diorama rather than a large map. The
station is the anchor, with a handful of destinations unlocked or expanded over time.

| Area | Primary mode | Examples |
| --- | --- | --- |
| Station | Arrival / return / visitors | Train handoff, notices, visiting creatures, activity board |
| Village Green | Pet-sim / free play | Petting, toys, snacks, grooming, dressing, photos |
| Clubhouse | Authored scenes | Conversations, mysteries, short stories, relationship scenes |
| Pond / Woods | Light exploration | Fishing, bug hunt, treasure search, environmental discoveries |
| Fairground | Minigames | Races, timing games, toy challenges, co-op/competitive activities |
| Hilltop | Quiet social space | Picnic, stargazing, photo moments, rare events |

## 3. Design pillars

| Pillar | Implication |
| --- | --- |
| The same creatures | Hill must consume stable creature identity rather than regenerate approximations. |
| Activities provide structure; Formiga provides character | Personality, habits, family and relationship context alter reactions, animation, role selection, and small outcomes. |
| Soft play, not maintenance | Interaction should be inviting but never punish absence. |
| A world worth revisiting | Areas accumulate visual detail, souvenirs, stories, photos, and activity history rather than relying on XP levels. |
| Moddable by design | Official content uses the same public package model, as far as practical, that community content uses. |
| Safe boundaries | Mods cannot access arbitrary files, processes, network endpoints, or canonical Desktop saves. |

## 4. Creature casting and behavioral adaptation

Hill should not treat imported creatures as skins over fixed actors. Content asks for roles and
the runtime casts appropriate companions using the travel snapshot.

### Role selectors

- Explicit companion selected by the player.
- Most/least aligned with a temperament axis: curious, bold, social, playful, affectionate,
  suspicious, etc.
- Companion matching a trait or temperament kind.
- Parent and mini pair.
- Closest available pair or a pair with high playfulness/avoidance context.
- Companion with a learned habit matching the activity.
- Random eligible companion using the story/activity package's deterministic session seed.

Selectors should always define fallbacks. Community content must not assume a six-creature
colony, a specific temperament, a mini, or a particular relationship.

### Behavior hooks

A content package should request intent and presentation categories rather than micromanage the
sprite frame-by-frame. Example: `react: surprise`, `move_to: prop.lantern`,
`social: celebrate_with partner`, `pose: inspect`. The Hill runtime maps these to the creature's
available Formiga animation vocabulary, temperament, reduced-motion setting, and body plan.

- Temperament chooses line variants, hesitation, eagerness, preferred activity roles, and reaction
  intensity.
- Learned habits may appear naturally when their cue occurs.
- Family and relationship context can alter staging, proximity, celebration, teasing, or
  reassurance.
- Accessories and visual identity carry through unless an activity explicitly uses a temporary
  costume.
- Content may never permanently rewrite core temperament or fabricate Desktop history.

## 5. Activity model

| Content type | Typical duration | Persistence | Examples |
| --- | --- | --- | --- |
| Free-play scene | Open-ended | Area state only | Petting, toys, snacks, posing, decorating |
| Activity | 2–10 min | Scores/unlocks optional | Race, fishing, hide-and-seek, cooking |
| Story scene | 1–5 min | Completion + souvenir optional | Conversation, discovery, comic incident |
| Story | 5–20 min | Chapter progress | Mystery, outing, visitor episode |
| Adventure | Multiple sessions | Hill-owned campaign state | Several locations and authored chapters |

## 6. Mod and community-content architecture

Hill should be moddable without allowing arbitrary native code. The recommended model is a
declarative content package with a manifest, data files, and sandboxed assets. Official content
should preferentially use the same format so the community-facing API stays exercised.

### Package shape

```text
my-story.formiga-hill/
  manifest.toml
  content/
    story.toml
    scenes/
      station.toml
      woods.toml
  assets/
    images/
    audio/
    fonts/        # optional only if licensing/embedding rules permit
  localization/
    en.toml
```

### Manifest requirements

| Field | Purpose |
| --- | --- |
| `package_id` | Reverse-domain or author-scoped stable ID |
| `title` / `author` / `version` | Human and package identity |
| `hill_api` | Supported Hill content API range |
| `content_types` | story, activity, area-extension, cosmetic-pack |
| `entry_points` | Scenes/activities exposed to the Hill UI |
| `requirements` | Minimum cast size, optional capabilities, required official area IDs |
| `permissions` | Must be empty/minimal in v1; no arbitrary filesystem/network/process access |
| `assets` | Declared files with size/type limits and optional hashes |
| `localization` | Supported locale tables |

### Declarative scene vocabulary

- **Cast:** declare roles, selectors, exclusions, and fallbacks.
- **Stage:** built-in Hill area or package-owned scene, spawn points, props, camera framing.
- **Beat:** move, face, gesture, expression, bubble, dialogue, wait, inspect, use prop, play
  animation intent.
- **Choice:** player choice with localized labels and branch targets.
- **Condition:** cast facts, Hill-owned story variables, completed content, time/season where
  exposed by runtime.
- **State:** package-scoped variables only, with typed bounded values.
- **Reward request:** cosmetic/souvenir descriptor submitted to Hill's reward validator, never
  direct mutation of Desktop data.
- **End:** completion, exit destination, optional return-to-hub transition.

A future scripting layer can be considered only after the declarative model proves too
restrictive. If added, it should be a sandboxed embedded language with deterministic
resource/time limits and no direct OS APIs.

### Content safety and validation

- Package parser imposes strict file-count, file-size, image-dimension, audio-duration and
  total-package limits.
- Paths are canonicalized and prevented from escaping the package root.
- No native libraries, shell commands, executables, arbitrary URLs, or unrestricted network
  requests.
- Unknown manifest fields may be tolerated for forward compatibility; unknown required
  capabilities must fail closed with a clear message.
- Packages can be disabled individually. A failed package never prevents Hill from opening.
- The loader reports package ID, version, offending file, and actionable validation errors.
- Optional future signing/trust UI may distinguish official, locally authored, and third-party
  packages without making unsigned community packages impossible.

## 7. Hill-owned state and progression

Hill should maintain its own save. Desktop supplies travelers; Hill owns the destination.

| State owner | Examples |
| --- | --- |
| Desktop canonical | Creature identity, appearance, temperament, learned habits, canonical family/relationship IDs and Desktop history |
| Travel snapshot | Read-only projection of canonical facts for one Hill session |
| Hill persistent | Area unlocks, story completion, minigame scores, Hill decorations, package-scoped state, photos/souvenirs |
| Return receipt | Validated requests for cosmetic souvenirs or a bounded "visited Hill" event; Desktop decides what to accept |

Progression should emphasize world accumulation rather than levels: areas become more lived-in,
new activities appear, photos and souvenirs collect, and stories remember completion. Avoid
generic XP/skill trees unless a later activity has a specific reason for them.

## 8. Inter-app travel contract

Hill consumes a dedicated travel format, not Desktop's save file and not the current creature
share code. The existing share code intentionally carries origin/design lineage and is excellent
for sharing a creature, but it does not represent a living colony member's name, temperament
state, learned habits, family, relationships, accessories, or history.

### Travel snapshot: proposed v1

- `format_version` and `minimum_reader_version`
- `session_id` (random 128-bit or UUID-style identifier)
- `created_at` and originating Desktop version
- colony display name/seed-derived public identifier if desired, but no unnecessary
  machine-specific data
- for each traveler: stable creature ID, display name, resolved appearance/design,
  body/size/role, temperament projection, traits/tension, learned habits, accessory/cosmetic
  state, parent ID where applicable
- relationship projection for traveler pairs: coarse or typed values needed for casting and
  reactions, not Desktop runtime plans
- selected collection/cosmetic entitlements where Hill is allowed to display them
- accessibility/user presentation preferences that should carry across: reduced motion, text
  scale/theme preference where appropriate
- capabilities bitset/list so Hill content can adapt to old/new Desktop exports

### Explicitly excluded

- Desktop coordinates, monitor IDs, cursor/window history, habitat geometry, live action plans,
  update state, diagnostic paths.
- Desktop's raw save-version internals and migration history.
- Anything a community package could use to locate arbitrary user files.
- Unbounded journal history unless a future story capability explicitly needs a privacy-reviewed
  subset.

### Return receipt: proposed v1

- `session_id` and Hill version
- clean/unclean completion status
- Hill session duration for diagnostics only if user-visible/privacy-appropriate
- bounded souvenir/reward descriptors
- optional high-level event: "returned from Formiga Hill" with activity/story package ID and
  display title
- no direct relationship, temperament, habit, family, creature-membership, or Desktop-settings
  mutation

## 9. Technical architecture

The Formiga Desktop repo already has two useful reusable layers: `formiga-core`, which contains
the serializable creature/simulation model with few platform dependencies, and `formiga-art`,
which renders creatures, shelters, objects, cards, stickers, trinkets and wonders above core. Hill
should reuse these concepts rather than duplicate creature generation or art.

| Layer | Responsibility |
| --- | --- |
| `formiga-core` (shared) | Stable creature/model types and behavior vocabulary suitable for both apps |
| `formiga-art` (shared) | Creature rendering and reusable visual assets; may need new Hill-facing render helpers |
| `formiga-travel` (new shared crate or small spec library) | Versioned snapshot/receipt DTOs, validation, compatibility tests |
| `formiga-hill-runtime` | Scene graph, cast resolution, activity/story state, package loader, sandbox, Hill save |
| `formiga-hill-ui` | Hub, areas, dialogue, activity selection, accessibility |
| Formiga Desktop host | Save/export snapshot, train departure/return, Hill discovery/launch, safe recovery |

Hill may live in a separate repository, but the shared crates should have a reproducible
dependency strategy: tagged git dependencies, a dedicated shared repository, or published crates.
Avoid copying source files between repos.

## 10. Visual and interaction direction

- Preserve Formiga's pixel-art scale and creature readability, but allow Hill scenes to be larger
  and more composed than transparent desktop overlays.
- Use a storybook/model-village composition: compact spaces, clear interactive props, readable
  entrances/exits, little wasted traversal.
- Keep direct manipulation simple: point/click, drag where appropriate, keyboard/controller
  equivalent focus actions if supported.
- Dialogue should be optional and concise; animation and reactions should carry much of the
  character.
- Reduced motion must have authored alternatives rather than merely slowing every animation.

## 11. MVP / vertical slice

| Capability | Vertical-slice requirement |
| --- | --- |
| Travel | Desktop train departure → Hill station → return train |
| Import | 3–4 real colony members through travel snapshot v1 |
| Hub | Station plus one interactive area |
| Pet sim | Pet, toy/snack, simple creature preference/reaction variation |
| Minigame | One repeatable activity with temperament-sensitive behavior or celebration |
| Story | One short 3-scene authored story with cast selectors and at least one branch |
| Modding | The story is loaded from the same package format documented for community use |
| Persistence | Hill save remembers completion and one cosmetic/souvenir |
| Return | One validated souvenir returns to Desktop; crash/early-close restores colony safely |

## 12. Recommended development phases

| Phase | Focus |
| --- | --- |
| 0.1 – Travel prototype | Snapshot contract, train handoff, Hill station, same-creature rendering |
| 0.2 – Character proof | Free play and one activity demonstrating temperament/habit/relationship differences |
| 0.3 – Story runtime | Declarative scenes, choices, cast selectors, package validation |
| 0.4 – Community package alpha | Mods folder, package browser, error reporting, sample SDK/docs |
| 0.5 – Persistent Hill | Several areas, unlocks, souvenirs, return receipts |
| Later | Creator tools, additional activities, richer stories/adventures, optional package signing/discovery |

## 13. Acceptance criteria for the concept

- A user immediately recognizes imported companions as the same individuals, not regenerated
  lookalikes.
- At least two activities produce visibly different behavior from different colonies without
  authoring bespoke branches for every creature.
- A community author can create a short scene/story without compiling Rust or executing code.
- A malformed or malicious content package cannot read arbitrary files, run programs, or corrupt
  the user's Desktop colony.
- Closing or crashing Hill never strands the Desktop colony off-screen.
- Desktop and Hill can evolve independently because the travel contract is versioned and narrower
  than either app's internal save.

## 14. Open design decisions

- Whether Hill initially takes the whole colony every time or later supports selecting travelers.
- How much relationship information should cross the boundary: raw four-score records, normalized
  bands, or a Hill-specific projection.
- Whether Hill can display Desktop keepsakes/accessories as entitlements or only equipped
  cosmetics in v1.
- Whether official Hill content ships embedded, as first-party packages, or a mix of both.
- Whether community assets may include audio in v1; image-only content is substantially simpler to
  validate and package.
- Whether the player controls one creature in any activity, or Hill remains primarily
  indirect/ensemble-focused.

## 15. Source grounding

This handoff was written against the Formiga Desktop repository,
[Von-Van/Formiga-Desktop](https://github.com/Von-Van/Formiga-Desktop). Relevant existing
boundaries include `formiga-core` (model, temperament, habits, relationships, persistence),
`formiga-art` (procedural creature and village rendering), and `formiga-desktop` (platform
integration, tray/menu, save ownership, overlays and notebook UI). The workspace reported version
0.66.1; `formiga-core` reported save version 24.

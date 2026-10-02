# Formiga Hill: design handoff (revised)

Product, technical, and community-content design specification, second revision. Written against
Formiga Desktop 0.66.1 (save schema v24), 2 October 2026. This revision adds the Woods → Hilltop
loop and the hidden Cursor Sovereign encounter to the founding brief. Where the code has since
made a decision this document left open, [TRAVEL.md](TRAVEL.md) and the README record it.

> **Product premise.** Formiga Desktop is where the creatures live around the user. Formiga Hill
> is somewhere the user deliberately sends them to spend time together.

> **From the owner, alongside this revision.** The Fairground and the Woods differ in who plays.
> The Fairground's minigames are automated: the colony plays them and the person watches. In the
> Woods the person actively takes part in the activities, and earns the rewards and finds that go
> on to fill the Hilltop.

## 1. Purpose and scope

Formiga Hill is a separate desktop application that imports a user's existing Formiga colony and uses those creatures as the cast of a more intentional, interactive experience. It should feel like a destination in the same ecosystem rather than a second copy of Formiga Desktop.

Hill is deliberately broader than a single genre. Its persistent hub can support free-form pet-sim interaction, authored visual-novel-like scenes, repeatable minigames, RPG-lite Woods expeditions, a persistent player-specific Hilltop, and later community-authored content. The unifying feature is not the activity type; it is that the user's own Formiga creatures retain their identity and behavior across all of them.

### Goals

- Make the handoff from Desktop to Hill feel diegetic: the colony gathers, a small train arrives, the creatures board, the train leaves the desktop, and Hill opens at its station.
- Preserve creature identity: appearance, name, temperament, learned habits, family ties, meaningful relationship context, and selected cosmetic state should matter inside Hill.
- Support several activity styles under one coherent hub rather than forcing Hill to choose between pet sim, visual novel, or minigame collection.
- Be friendly to community content from the beginning through a safe, declarative package format and a stable content API.
- Avoid turning Hill into an obligation loop. No neglect punishment, streak pressure, hunger debt, or 'you failed to log in' mechanics.
- Keep Formiga Desktop authoritative for the colony's canonical identity and long-lived desktop simulation.

### Non-goals for the first releases

- A full RPG, large open world, conventional combat progression, or grind-heavy crafting economy. Hill may contain isolated RPG-like systems and authored encounters, but they should serve the colony rather than redefine the whole product.
- Arbitrary third-party code execution as part of mods.
- Direct editing of Formiga Desktop's colony.json from Hill.
- A requirement that every Desktop system be simulated while the colony is visiting Hill.
- A large creator IDE before the runtime and package format prove themselves.

## 2. Core experience

### The train handoff

- The user chooses “Go to Formiga Hill…” from Formiga Desktop.
- Desktop saves the colony and creates a versioned travel snapshot.
- Creatures finish or safely interrupt incompatible runtime actions, gather near the bottom of the active display, and react to the arrival.
- A small Formiga-scale train enters. Boarding reactions can vary by temperament, habits, age/mini status, and relationships.
- The creatures board; the train leaves the desktop. Desktop hides the colony while the Hill session is active.
- Formiga Hill opens on, or transitions into, its station arrival scene using the same travel-session identifier.
- When leaving Hill, the return trip mirrors the departure. Desktop receives a constrained return receipt and resumes the colony safely even if Hill closes unexpectedly.

> The train is not just launch chrome. It is the fiction that makes two executables feel like one
> world.

### Hill as a place

The first Hill should be compact and readable as a storybook diorama rather than a large map. The station is the anchor, with a handful of destinations unlocked or expanded over time.

| Area | Primary mode | Examples |
| --- | --- | --- |
| Station | Arrival / return / visitors | Train handoff, notices, visiting creatures, activity board |
| Village Green | Pet-sim / free play | Petting, toys, snacks, grooming, dressing, photos |
| Clubhouse | Authored scenes | Conversations, mysteries, short stories, relationship scenes |
| Woods | RPG-lite expeditions / skill minigames | Fishing, foraging, bug catching, scavenging, treasure hunting, rare encounters |
| Fairground | Minigames | Races, timing games, toy challenges, co-op/competitive activities |
| Hilltop | Persistent non-linear colony place | Find-driven structures, gardens, oddities, social spaces, skyline/background growth |

### The Woods → Hilltop loop

The Woods and Hilltop should form the main RPG-lite progression loop, but not a linear progression track. The Hilltop begins essentially as an empty, persistent space belonging to the colony. The Woods generates the things that can fill it: objects, structures, plants, curiosities, landmarks, and other discoveries. The player decides what to place and where within the Hilltop's available space, gradually turning an initially sparse hill into a unique place assembled from that colony's actual experiences.

Core loop: take one or more Formiga creatures into the Woods → play a substantial activity or expedition → return with materials, notable finds, rare discoveries, objects, or new ideas → place, build, grow, or display those discoveries on the Hilltop → interact with many of the placed additions afterward → let the changed Hilltop alter the visible world and expand the pool of future activities, stories, and discoveries.

#### Hilltop: discovery-shaped, not level-shaped

- The Hilltop should begin close to empty: an open grassy space with only the minimum scenery needed to establish the location. It is not a predefined town waiting to be upgraded. Its contents are primarily created by the player's Woods discoveries. Finds can become directly placeable objects, inspire structures, grow into plants or gardens, or unlock unusual decorative and functional pieces. The player should have meaningful authorship over arrangement, so two colonies with similar finds can still produce different Hilltops.
- Common finds may provide broad building material or visual texture; notable finds can inspire a specific object or structure; exceptional finds can create major landmarks or rare variants. A brass lens might make an observatory possible, a hollow log might lead to a den or tree structure, an unusual seed might reshape a garden, and a strange relic might become the center of an otherwise ordinary gathering place.
- Placement and construction should be expressive without becoming a survival-game building editor. Discoveries resolve into authored Formiga-scale objects and structures with sensible footprints and placement rules; the player chooses where they belong within valid Hilltop space. Some pieces can have variants or combinations based on other finds. Formigas then animate around construction or placement so the colony appears to participate rather than the user simply editing a map.
- Placed Hilltop discoveries should remain part of play rather than becoming static trophies. Where appropriate, Formigas can sit on, climb, inspect, nap beside, play with, tend, activate, or gather around them; some objects can expose small interactions for the player as well. A telescope might be used for stargazing, a pond feature for fishing or watching water, a picnic setup for social scenes, or a strange relic for a later event. The Hilltop should also be visible from other major zones wherever composition allows. Station, Village Green, Fairground, Clubhouse exterior views, and Woods overlooks can show the Hilltop in the background, so its silhouette and density change as the player fills the space.
- No Hilltop feature decays because the user was away. Growth is additive. The Hilltop may become dense, strange, cozy, organized, chaotic, botanical, scavenged, observatory-like, or something else entirely depending on what was found and chosen.

#### Woods: deep activities with creature-specific replay value

The Woods should not be a shallow gathering screen. If it is the source of Hilltop possibilities, its activities need enough mechanical depth, reward variety, and creature-specific behavior to justify repeated visits over many sessions and with different Formiga creatures.

- Favor a small number of substantial activity families over many one-note minigames. Initial families can include fishing, foraging, bug catching, scavenging/treasure hunting, and longer expeditions that combine several mechanics.
- Each activity needs mechanical mastery: timing, route or spot selection, risk/reward choices, pattern recognition, equipment or approach choices where appropriate, and enough variation that the player can genuinely improve rather than merely wait for random drops.
- Each activity needs a broad discovery pool. Common rewards remain useful; uncommon rewards open unusual Hilltop possibilities; rare rewards provide distinctive variants, props, story hooks, or environmental changes; exceptional rewards can become defining landmarks.
- Replayability should be companion-specific. Temperament, traits, habits, family relationships, and pair dynamics can change small rules, reactions, opportunities, mistakes, and advantages. The question should become 'Have I taken this Formiga fishing yet?' rather than only 'Have I played fishing already?'
- Different Formigas can expose different content: a curious explorer may notice an alternate path, a patient scholar may make a precision activity easier, an impulsive creature may reveal risky shortcuts, a mini may reach a small hiding place, and a close pair may cooperate on something one creature cannot manage alone.
- Rewards should include more than materials. An outing can produce a physical find, a rare collectible, a new Hilltop possibility, a clue, a story trigger, an environmental idea, or a discovery that only becomes meaningful when combined with something found later.
- The Woods can broaden in response to the Hilltop without becoming a linear lock-and-key map. What the colony builds changes what becomes noticeable or relevant: a telescope can make distant or nighttime outings meaningful; a botanical space can surface plant-related routes; an unusual fishing display may prompt a new stream or pond event.

#### Hidden encounter: the Cursor Sovereign

Somewhere behind the ordinary Woods loop should be one deliberately absurd secret: a hidden location containing what looks like a fully committed, super-anime boss battle against a gigantic mouse cursor. In mechanical terms, however, it is primarily an interactive cutscene rather than a true combat system. The joke comes from giving an almost trivial interaction the presentation of a climactic JRPG battle.

- The encounter should be rare, optional, and discovered through strange combinations of finds, Hilltop state, creature history, or an obscure Woods event rather than a visible level requirement.
- Presentation should be played completely straight: dramatic cut-ins, exaggerated poses, an enormous boss title and health bar, phase transitions, screen shake, heroic friendship moments, and attack names wildly disproportionate to the tiny creatures involved. The health bar can function mostly as theatrical pacing rather than a simulation-heavy combat value.
- The boss is a giant mouse cursor - an intentionally uncanny inversion of the object Formiga creatures normally react to on the desktop. It can click, drag, box-select, sweep across the arena, create selection rectangles, or use other cursor-inspired attacks.
- The player's main interaction during the encounter is choosing from a small command-menu-style list of 'attacks.' These are not part of a deep damage, equipment, timing, or stat system. Each choice primarily selects a different extravagant Formiga animation, team-up, joke, or cinematic sequence and advances the encounter. The menu can use wonderfully over-serious attack names while the underlying outcome remains authored and forgiving.
- Creature personality remains intact under the anime presentation. The selected 'attack' can be reinterpreted through the acting creature: a showoff gets a ridiculous entrance pose, a cautious creature hesitates before committing, a lazy one may yawn during the monologue, a mini can perform something visually pathetic that inexplicably devastates the boss, and close companions can receive absurd combination animations.
- The reward should be memorable rather than power-creeping: a bizarre relic, cosmetic, Hilltop centerpiece, journal/story marker, or background landmark that permanently tells the player 'yes, that really happened.'
- Community content can later use the interactive-cutscene framework for other secret set pieces, but the giant cursor should remain a signature first-party surprise. This framework should not imply a reusable stat-combat layer or routine combat progression.

## 3. Design pillars

| Pillar | Implication |
| --- | --- |
| The same creatures | Hill must consume stable creature identity rather than regenerate approximations. |
| Activities provide structure; Formiga provides character | Personality, habits, family and relationship context alter reactions, animation, role selection, and small outcomes. |
| Soft play, not maintenance | Interaction should be inviting but never punish absence. |
| A world worth revisiting | The Woods continually generates discoveries, while the Hilltop accumulates a non-linear visual history shaped by those finds. Other zone backgrounds reflect that evolving Hilltop rather than relying on XP levels. |
| Moddable by design | Official content uses the same public package model, as far as practical, that community content uses. |
| Safe boundaries | Mods cannot access arbitrary files, processes, network endpoints, or canonical Desktop saves. |

## 4. Creature casting and behavioral adaptation

Hill should not treat imported creatures as skins over fixed actors. Content asks for roles and the runtime casts appropriate companions using the travel snapshot.

### Role selectors

- Explicit companion selected by the player.
- Most/least aligned with a temperament axis: curious, bold, social, playful, affectionate, suspicious, etc.
- Companion matching a trait or temperament kind.
- Parent and mini pair.
- Closest available pair or a pair with high playfulness/avoidance context.
- Companion with a learned habit matching the activity.
- Random eligible companion using the story/activity package's deterministic session seed.

Selectors should always define fallbacks. Community content must not assume a six-creature colony, a specific temperament, a mini, or a particular relationship.

### Behavior hooks

A content package should request intent and presentation categories rather than micromanage the sprite frame-by-frame. Example: `react: surprise`, `move_to: prop.lantern`, `social: celebrate_with partner`, `pose: inspect`. The Hill runtime maps these to the creature's available Formiga animation vocabulary, temperament, reduced-motion setting, and body plan.

- Temperament chooses line variants, hesitation, eagerness, preferred activity roles, and reaction intensity.
- Learned habits may appear naturally when their cue occurs.
- Family and relationship context can alter staging, proximity, celebration, teasing, or reassurance.
- Accessories and visual identity carry through unless an activity explicitly uses a temporary costume.
- Content may never permanently rewrite core temperament or fabricate Desktop history.

## 5. Activity model

| Content type | Typical duration | Persistence | Examples |
| --- | --- | --- | --- |
| Free-play scene | Open-ended | Area state only | Petting, toys, snacks, posing, decorating |
| Activity | 2-10 min | Scores/unlocks optional | Race, fishing, hide-and-seek, cooking |
| Story scene | 1-5 min | Completion + souvenir optional | Conversation, discovery, comic incident |
| Story | 5-20 min | Chapter progress | Mystery, outing, visitor episode |
| Adventure | Multiple sessions | Hill-owned campaign state | Several locations and authored chapters |
| Woods expedition | 5-15 min | Discoveries + Hilltop possibilities | Fishing, foraging, bug catching, scavenging, treasure routes, rare encounters |
| Secret set piece | 5-10 min | Rare completion + landmark | Interactive cinematic encounter: attack-menu choices trigger authored animations/cutscenes |

## 6. Mod and community-content architecture

Hill should be moddable without allowing arbitrary native code. The recommended model is a declarative content package with a manifest, data files, and sandboxed assets. Official content should preferentially use the same format so the community-facing API stays exercised.

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
| package_id | Reverse-domain or author-scoped stable ID |
| title / author / version | Human and package identity |
| hill_api | Supported Hill content API range |
| content_types | story, activity, woods-encounter, hilltop-addition, area-extension, cosmetic-pack |
| entry_points | Scenes/activities exposed to the Hill UI |
| requirements | Minimum cast size, optional capabilities, required official area IDs |
| permissions | Must be empty/minimal in v1; no arbitrary filesystem/network/process access |
| assets | Declared files with size/type limits and optional hashes |
| localization | Supported locale tables |

### Declarative scene vocabulary

- Cast: declare roles, selectors, exclusions, and fallbacks.
- Stage: built-in Hill area or package-owned scene, spawn points, props, camera framing.
- Beat: move, face, gesture, expression, bubble, dialogue, wait, inspect, use prop, play animation intent.
- Choice: player choice with localized labels and branch targets.
- Condition: cast facts, Hill-owned story variables, completed content, time/season where exposed by runtime.
- State: package-scoped variables only, with typed bounded values.
- Reward request: cosmetic/souvenir descriptor submitted to Hill's reward validator, never direct mutation of Desktop data.
- End: completion, exit destination, optional return-to-hub transition.
- Discovery: declare Woods reward pools, rarity bands, conditions, companion-sensitive modifiers, and what Hilltop possibilities a discovery can expose.
- Hilltop addition: declare an authored structure/prop/landmark, placement class, visual variants, required discoveries, and optional background silhouettes used by other zones.
- Encounter: declare hidden-location conditions, cinematic phases, command-menu choices, animation/cutscene mappings, boss presentation, and a bounded landmark or souvenir reward.

A future scripting layer can be considered only after the declarative model proves too restrictive. If added, it should be a sandboxed embedded language with deterministic resource/time limits and no direct OS APIs.

### Content safety and validation

- Package parser imposes strict file-count, file-size, image-dimension, audio-duration and total-package limits.
- Paths are canonicalized and prevented from escaping the package root.
- No native libraries, shell commands, executables, arbitrary URLs, or unrestricted network requests.
- Unknown manifest fields may be tolerated for forward compatibility; unknown required capabilities must fail closed with a clear message.
- Packages can be disabled individually. A failed package never prevents Hill from opening.
- The loader reports package ID, version, offending file, and actionable validation errors.
- Optional future signing/trust UI may distinguish official, locally authored, and third-party packages without making unsigned community packages impossible.

## 7. Hill-owned state and progression

Hill should maintain its own save. Desktop supplies travelers; Hill owns the destination.

| State owner | Examples |
| --- | --- |
| Desktop canonical | Creature identity, appearance, temperament, learned habits, canonical family/relationship IDs and Desktop history |
| Travel snapshot | Read-only projection of canonical facts for one Hill session |
| Hill persistent | Woods discoveries, per-creature activity records, Hilltop composition/landmarks, story completion, minigame scores, package-scoped state, photos/souvenirs |
| Return receipt | Validated requests for cosmetic souvenirs or a bounded 'visited Hill' event; Desktop decides what to accept |

Progression should emphasize world accumulation rather than levels. The Hilltop is fundamentally an initially empty persistent space that the player fills with things enabled or recovered through Woods play. There is no mandatory upgrade chain: structures, landmarks, gardens, oddities, social spaces, and small interactive objects appear because the colony found them or found what was needed to make them, and the player chooses how to arrange them. Those placed objects remain visible and, where appropriate, usable by the Formigas or player. The resulting Hilltop should visibly alter backgrounds in other zones. Woods activities, in turn, gain new contexts and encounter possibilities from that Hilltop state. Avoid generic XP/skill trees and material grinding; discovery, arrangement, interaction, mechanical mastery, companion-specific replayability and visible world change are the progression.

## 8. Inter-app travel contract

Hill consumes a dedicated travel format, not Desktop's save file and not the current creature share code. The existing share code intentionally carries origin/design lineage and is excellent for sharing a creature, but it does not represent a living colony member's name, temperament state, learned habits, family, relationships, accessories, or history.

### Travel snapshot - proposed v1

- format_version and minimum_reader_version
- session_id (random 128-bit or UUID-style identifier)
- created_at and originating Desktop version
- colony display name/seed-derived public identifier if desired, but no unnecessary machine-specific data
- for each traveler: stable creature ID, display name, resolved appearance/design, body/size/role, temperament projection, traits/tension, learned habits, accessory/cosmetic state, parent ID where applicable
- relationship projection for traveler pairs: coarse or typed values needed for casting and reactions, not Desktop runtime plans
- selected collection/cosmetic entitlements where Hill is allowed to display them
- accessibility/user presentation preferences that should carry across: reduced motion, text scale/theme preference where appropriate
- capabilities bitset/list so Hill content can adapt to old/new Desktop exports

### Explicitly excluded

- Desktop coordinates, monitor IDs, cursor/window history, habitat geometry, live action plans, update state, diagnostic paths.
- Desktop's raw save-version internals and migration history.
- Anything a community package could use to locate arbitrary user files.
- Unbounded journal history unless a future story capability explicitly needs a privacy-reviewed subset.

### Return receipt - proposed v1

- session_id and Hill version
- clean/unclean completion status
- Hill session duration for diagnostics only if user-visible/privacy-appropriate
- bounded souvenir/reward descriptors
- optional high-level event: 'returned from Formiga Hill' with activity/story package ID and display title
- no direct relationship, temperament, habit, family, creature-membership, or Desktop-settings mutation

## 9. Technical architecture

The current Formiga repo already has two useful reusable layers: `formiga-core`, which contains the serializable creature/simulation model with few platform dependencies, and `formiga-art`, which renders creatures, shelters, objects, cards, stickers, trinkets and wonders above core. Hill should reuse these concepts rather than duplicate creature generation or art.

| Layer | Responsibility |
| --- | --- |
| formiga-core (shared) | Stable creature/model types and behavior vocabulary suitable for both apps |
| formiga-art (shared) | Creature rendering and reusable visual assets; may need new Hill-facing render helpers |
| formiga-travel (new shared crate or small spec library) | Versioned snapshot/receipt DTOs, validation, compatibility tests |
| formiga-hill-runtime | Scene graph, cast resolution, activity/story state, package loader, sandbox, Hill save |
| formiga-hill-ui | Hub, areas, dialogue, activity selection, accessibility |
| Formiga Desktop host | Save/export snapshot, train departure/return, Hill discovery/launch, safe recovery |

Hill may live in a separate repository, but the shared crates should have a reproducible dependency strategy: tagged git dependencies, a dedicated shared repository, or published crates. Avoid copying source files between repos.

## 10. Visual and interaction direction

- Preserve Formiga's pixel-art scale and creature readability, but allow Hill scenes to be larger and more composed than transparent desktop overlays.
- Use a storybook/model-village composition: compact spaces, clear interactive props, readable entrances/exits, little wasted traversal.
- Keep direct manipulation simple: point/click, drag where appropriate, keyboard/controller-equivalent focus actions if supported.
- Dialogue should be optional and concise; animation and reactions should carry much of the character.
- Reduced motion must have authored alternatives rather than merely slowing every animation.
- Compose major zone backgrounds so the Hilltop can be seen when spatially plausible, using a generated/assembled skyline layer that reflects the save's current structures and landmarks.
- Woods activity presentation should support repeated play without looking like disconnected arcade screens: preserve environmental continuity, companion staging, and discovery context.
- The hidden giant-cursor boss may temporarily break the normal presentation rules with dramatic anime-style overlays, cut-ins, titles and effects; that contrast is intentional.

## 11. MVP / vertical slice

| Capability | Vertical-slice requirement |
| --- | --- |
| Travel | Desktop train departure -> Hill station -> return train |
| Import | 3-4 real colony members through travel snapshot v1 |
| Hub | Station, a visible early Hilltop, and one Woods route/activity area |
| Pet sim | Pet, toy/snack, simple creature preference/reaction variation |
| Woods loop | One replayable Woods activity with enough mechanical depth for repeated runs, multiple reward tiers, and creature-sensitive variation |
| Story | One short 3-scene authored story with cast selectors and at least one branch |
| Modding | The story is loaded from the same package format documented for community use |
| Persistence | Hill save remembers discoveries, placed Hilltop objects and their arrangement, and uses at least one rare find to add an interactable feature visible from another zone |
| Return | One validated souvenir returns to Desktop; crash/early-close restores colony safely |
| Secret encounter architecture | Runtime can support a hidden interactive cutscene with command-menu choices mapped to authored animations; the full giant-cursor encounter does not need to ship in the first vertical slice |

## 12. Recommended development phases

| Phase | Focus |
| --- | --- |
| 0.1 - Travel prototype | Snapshot contract, train handoff, Hill station, same-creature rendering |
| 0.2 - Character proof | Free play plus one deep Woods activity demonstrating temperament/habit/relationship differences across repeated runs |
| 0.3 - Woods/Hilltop loop | Discovery pools, non-linear Hilltop additions, cross-zone background composition, first feedback loop from Hilltop state back into Woods |
| 0.4 - Story + community runtime | Declarative scenes, cast selectors, Woods encounters/Hilltop additions in packages, mods folder, validation and sample docs |
| 0.5 - Persistent Hill + secrets | Several Woods activity families, richer Hilltop variety, souvenirs/return receipts, hidden-location framework and giant-cursor boss encounter |
| Later | Creator tools, additional Woods activities, deeper stories/adventures, more Hilltop content, optional package signing/discovery |

## 13. Acceptance criteria for the concept

- A user immediately recognizes imported companions as the same individuals, not regenerated lookalikes.
- At least two activities produce visibly different behavior from different colonies without authoring bespoke branches for every creature.
- A community author can create a short scene/story without compiling Rust or executing code.
- A malformed or malicious content package cannot read arbitrary files, run programs, or corrupt the user's Desktop colony.
- Closing or crashing Hill never strands the Desktop colony off-screen.
- Desktop and Hill can evolve independently because the travel contract is versioned and narrower than either app's internal save.
- Two saves with meaningfully different discovery histories can produce visibly different Hilltops, and the player can meaningfully arrange the objects/structures those discoveries make available rather than following a superior or required progression path.
- At least one Woods activity remains worthwhile across repeated runs because skill, reward variety, and companion-specific behavior all change the experience.
- At least one other zone visibly reflects the current Hilltop composition in its background.
- The giant-cursor boss feels like a surprising anime-style interactive cutscene: the player chooses absurd 'attacks' from a menu and sees different authored animations, without the encounter implying that ordinary Hill play requires a real combat system.

## 14. Open design decisions

- Whether Hill initially takes the whole colony every time or later supports selecting travelers.
- How much relationship information should cross the boundary: raw four-score records, normalized bands, or a Hill-specific projection.
- Whether Hill can display Desktop keepsakes/accessories as entitlements or only equipped cosmetics in v1.
- Whether official Hill content ships embedded, as first-party packages, or a mix of both.
- Whether community assets may include audio in v1; image-only content is substantially simpler to validate and package.
- Whether the player controls one creature in any activity, or Hill remains primarily indirect/ensemble-focused.
- How much direct control the player has over Hilltop placement versus choosing among authored placement sites and variants.
- Whether Woods rewards use a visible rarity language or remain primarily described through discovery context and presentation.
- How secret the giant-cursor trigger should remain in UI/documentation, and whether repeat victories change only presentation or also unlock additional cosmetic/landmark variants.

## 15. Source grounding

This handoff was written against the current Formiga Desktop repository:

Von-Van/Formiga-Desktop

Relevant existing boundaries include `formiga-core` (model, temperament, habits, relationships, persistence), `formiga-art` (procedural creature and village rendering), and `formiga-desktop` (platform integration, tray/menu, save ownership, overlays and notebook UI). The workspace currently reports version 0.66.1; `formiga-core` reports save version 24.

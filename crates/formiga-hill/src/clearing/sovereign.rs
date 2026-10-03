//! The Cursor Sovereign: a giant mouse pointer, met in a clearing that isn't on any map, and a
//! battle played completely straight. It is an interactive cutscene, not combat: the person picks
//! from a menu of absurd attacks, each one a little scene acted out by the companion it belongs
//! to, and the Sovereign answers with a cursor's worst: a click, a drag, a select-all. There are
//! no stats and nothing to lose. Every choice lands, the Sovereign falls in three phases, and at
//! the end the rest of the colony comes running.
//!
//! Who came along still decides how it plays: a show-off's entrance, a scholar's footnotes, a
//! lazybones asleep through the villain's speech, a little one's tiny squeak that is somehow the
//! most devastating thing the Sovereign has ever heard.

use super::boss::Move;
use super::stage::{Act, Speaker, Stage};
use crate::actor::Step;
use crate::cast::{Cast, Id};
use crate::character::{Beat, Character, Cue};
use crate::dice::Dice;
use crate::playground::Playground;
use formiga_art::ExpressionKind;
use formiga_core::{ActionKind, Gesture, TemperamentKind};

/// Where the two who came stand, and where the rest of the colony runs to at the end.
pub const MARKS: [(f32, f32); 2] = [(96.0, 188.0), (136.0, 194.0)];
const FRONT: (f32, f32) = (190.0, 184.0);
/// Below these, each phase gives way to the next; below the last, the finale.
const FLOORS: [f32; 3] = [0.67, 0.34, 0.12];

/// Whether, on a rummage late in the day, the glint may show: only for a colony that has found a
/// good many things and set something on the Hilltop for looking up at the sky with (or anything
/// made with one). Once seen off, it comes back only now and then.
pub fn may_beckon(colony: &crate::memories::ColonyMemories) -> bool {
    let found = colony
        .finds
        .keys()
        .filter(|id| !crate::finds::is_relic(id))
        .count();
    let gazing = colony
        .hilltop
        .values()
        .any(crate::hilltop::Standing::sky_gazing);
    let due = colony.sovereign_bested == 0 || colony.outings.is_multiple_of(3);
    found >= 10 && gazing && due
}

/// What happens at the end, for whoever is watching.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Bested,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// The Sovereign introduces itself.
    Intro,
    /// The person chooses what to do.
    Choosing,
    /// An attack, and the Sovereign's answer, playing out.
    Acting,
    /// The Sovereign changing to its next form.
    Shifting,
    Finale,
    Over,
}

/// One thing the person can choose: what it is called, and how it plays out.
#[derive(Clone, Debug)]
pub struct Choice {
    pub name: String,
    acts: Vec<Act>,
    damage: f32,
}

pub struct Sovereign {
    pub stage: Stage,
    party: Vec<(Id, Character, String)>,
    /// The rest of the colony, who come running at the end.
    rest: Vec<Id>,
    close_pair: bool,
    phase: usize,
    state: State,
    choices: Vec<Choice>,
    dice: Dice,
    events: Vec<Event>,
}

impl Sovereign {
    /// The Sovereign meets `party` in the clearing (already standing in `ground`), with the rest
    /// of `cast` ready to come running.
    pub fn new(
        ground: &mut Playground,
        cast: &Cast,
        party: &[Id],
        close_pair: bool,
        now: f32,
    ) -> Self {
        let members: Vec<(Id, Character, String)> = party
            .iter()
            .filter_map(|id| {
                let member = cast.member(*id)?;
                Some((*id, Character::of(member), member.name.clone()))
            })
            .collect();
        ground.reserve(party.to_vec());
        let rest = cast.ids().filter(|id| !party.contains(id)).collect();
        let mut sovereign = Self {
            stage: Stage::new(now),
            party: members,
            rest,
            close_pair,
            phase: 0,
            state: State::Intro,
            choices: Vec::new(),
            dice: Dice::new(party.iter().fold(0x507e_2e16_u64, |seed, id| seed ^ id)),
            events: Vec::new(),
        };
        let intro = sovereign.intro();
        sovereign.stage.play(intro);
        sovereign
    }

    pub fn state(&self) -> State {
        self.state
    }

    /// What the person can choose now, if anything.
    pub fn choices(&self) -> &[Choice] {
        if self.state == State::Choosing {
            &self.choices
        } else {
            &[]
        }
    }

    pub fn take_events(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    /// Who came along, for the close-ups.
    pub fn party(&self) -> Vec<Id> {
        self.party.iter().map(|(id, _, _)| *id).collect()
    }

    /// The person picks an attack.
    pub fn choose(&mut self, index: usize) {
        if self.state != State::Choosing {
            return;
        }
        let Some(choice) = self.choices.get(index).cloned() else {
            return;
        };
        let after = self.stage.health - choice.damage;
        self.stage.play(choice.acts);
        // The Sovereign answers, unless that blow ended this form.
        if after > FLOORS[self.phase] {
            let answer = self.answer();
            self.stage.play(answer);
        }
        self.state = State::Acting;
    }

    pub fn tick(&mut self, ground: &mut Playground, cast: &Cast, now: f32) {
        self.stage.tick(ground, cast, now);
        if !self.stage.idle() {
            return;
        }
        match self.state {
            State::Intro | State::Shifting => self.offer(),
            State::Acting => {
                if self.stage.health <= FLOORS[self.phase] + 0.001 {
                    if self.phase + 1 < FLOORS.len() {
                        let shift = self.shift();
                        self.stage.play(shift);
                        self.phase += 1;
                        self.state = State::Shifting;
                    } else {
                        let finale = self.finale();
                        self.stage.play(finale);
                        self.state = State::Finale;
                    }
                } else {
                    self.offer();
                }
            }
            State::Finale => {
                self.state = State::Over;
                self.events.push(Event::Bested);
            }
            State::Choosing | State::Over => {}
        }
    }

    fn offer(&mut self) {
        self.choices = self.menu();
        self.state = State::Choosing;
    }

    fn home(&self, id: Id) -> (f32, f32) {
        let index = self
            .party
            .iter()
            .position(|(member, _, _)| *member == id)
            .unwrap_or(0);
        MARKS[index.min(MARKS.len() - 1)]
    }

    fn intro(&mut self) -> Vec<Act> {
        let boss = |text: &str| Act::Say(Speaker::Boss, text.to_owned());
        let narrate = |text: String| Act::Say(Speaker::Narrator, text);
        let mut acts = vec![
            Act::Shake(0.8),
            Act::Wait(0.6),
            boss("SO. THE LITTLE ICONS HAVE WANDERED OFF THE DESKTOP."),
        ];
        // Each meets the Sovereign in its own way.
        let party = self.party.clone();
        for (index, (id, character, name)) in party.iter().enumerate() {
            let other = party
                .get(1 - index.min(1))
                .filter(|_| party.len() > 1)
                .map(|(_, _, name)| name.clone());
            let (beat, line) = match character.kind {
                _ if character.parent.is_some() => (
                    Beat::new(Gesture::Cheer, ExpressionKind::Determined, 1.2),
                    format!("{name} squeaks, as fiercely as it can."),
                ),
                TemperamentKind::Lazybones => (
                    Beat::new(Gesture::Yawn, ExpressionKind::Yawning, 2.5),
                    format!("{name} yawns enormously, all the way through the speech."),
                ),
                TemperamentKind::Showoff => (
                    with(
                        Beat::new(Gesture::Strut, ExpressionKind::Smug, 1.6),
                        Cue::Sparkle,
                    ),
                    format!("{name} strikes a pose. Nobody asked it to."),
                ),
                TemperamentKind::Wallflower => (
                    Beat::new(Gesture::Peek, ExpressionKind::Worried, 2.0),
                    match other {
                        Some(other) => format!("{name} hides behind {other}."),
                        None => format!("{name} hides behind nothing at all, very carefully."),
                    },
                ),
                TemperamentKind::Grump => (
                    with(
                        Beat::new(Gesture::Huff, ExpressionKind::Grumpy, 1.2),
                        Cue::Huff,
                    ),
                    // Said by the grump itself, not told.
                    "Hmph.".to_owned(),
                ),
                TemperamentKind::Scholar => (
                    Beat::new(Gesture::Watch, ExpressionKind::Focused, 2.0),
                    format!("{name} is quietly taking notes."),
                ),
                TemperamentKind::Sweetheart => (
                    with(
                        Beat::new(Gesture::Reach, ExpressionKind::Joy, 1.2),
                        Cue::Heart,
                    ),
                    format!("{name} waves hello to the Sovereign, politely."),
                ),
                TemperamentKind::Explorer => (
                    Beat::new(Gesture::Watch, ExpressionKind::Curious, 1.6),
                    format!("{name} is already looking for a weak point."),
                ),
                TemperamentKind::Oddball => (
                    spun(Beat::new(Gesture::Cheer, ExpressionKind::Joy, 1.2)),
                    format!("{name} appears to be humming."),
                ),
                TemperamentKind::Troublemaker => (
                    Beat::new(Gesture::Bop, ExpressionKind::Smug, 1.2),
                    format!("{name} looks like it has done this before."),
                ),
                TemperamentKind::Guardian => (
                    Beat::new(ActionKind::Idle, ExpressionKind::Determined, 1.6),
                    format!("{name} steps in front of everyone."),
                ),
            };
            acts.push(Act::Do(*id, vec![Step::Beat(beat), Step::Beat(standing())]));
            acts.push(
                if character.kind == TemperamentKind::Grump && character.parent.is_none() {
                    Act::Say(Speaker::Member(*id), line)
                } else {
                    narrate(line)
                },
            );
        }
        acts.extend([
            boss("I AM THE CURSOR SOVEREIGN. I HAVE CLICKED UPON KINGS."),
            boss("EVERY WINDOW OPENS AT MY COMMAND. EVERY FILE TREMBLES."),
            Act::Shake(0.5),
            boss("KNEEL. OR BE... DRAGGED."),
        ]);
        acts
    }

    /// What the person can choose: each companion's own attack, the two together if they are
    /// close, and a snack for everyone.
    fn menu(&mut self) -> Vec<Choice> {
        let mut menu: Vec<Choice> = self
            .party
            .clone()
            .iter()
            .map(|(id, character, name)| self.signature(*id, character, name))
            .collect();
        if self.close_pair && self.party.len() > 1 {
            menu.push(self.together());
        }
        menu.push(self.snack());
        menu
    }

    /// A companion's own attack, after its temperament.
    fn signature(&mut self, id: Id, character: &Character, name: &str) -> Choice {
        let narrate = |text: String| Act::Say(Speaker::Narrator, text);
        let boss = |text: &str| Act::Say(Speaker::Boss, text.to_owned());
        let home = self.home(id);
        let go = Step::Stride { to: FRONT };
        let back = Step::Walk { to: home };
        let face = Step::FaceX(300.0);
        let blow = |acts: &mut Vec<Act>, damage: f32| {
            acts.extend([Act::Boss(Move::Reel), Act::Shake(0.5), Act::Damage(damage)]);
        };
        let (title, damage, mut acts) = match character.kind {
            _ if character.parent.is_some() => {
                let mut acts = vec![
                    Act::Do(
                        id,
                        vec![
                            go,
                            face,
                            Step::Beat(Beat::new(Gesture::Cheer, ExpressionKind::Determined, 1.0)),
                        ],
                    ),
                    Act::Wait(0.6),
                    boss("HA! WHAT COULD SUCH A SMALL ONE POSSIBLY\u{2014}"),
                    Act::Do(
                        id,
                        vec![Step::Beat(with(
                            Beat::new(Gesture::Stomp, ExpressionKind::Determined, 0.8),
                            Cue::Exclaim,
                        ))],
                    ),
                    narrate(format!("{name} squeaks.")),
                ];
                acts.extend([Act::Boss(Move::Reel), Act::Shake(1.4), Act::Damage(0.34)]);
                acts.push(narrate(
                    "CRITICAL HIT. Nobody is more surprised than the Sovereign.".to_owned(),
                ));
                ("TINY DEFIANT SQUEAK".to_owned(), 0.34, acts)
            }
            TemperamentKind::Showoff => {
                let mut acts = vec![
                    Act::Do(
                        id,
                        vec![
                            go,
                            face,
                            Step::Beat(with(
                                Beat::new(Gesture::Strut, ExpressionKind::Smug, 1.6),
                                Cue::Sparkle,
                            )),
                        ],
                    ),
                    Act::Wait(1.2),
                ];
                blow(&mut acts, 0.2);
                acts.push(narrate(format!(
                    "{name} poses so magnificently that the Sovereign has to look away."
                )));
                ("RADIANT SUPERSTAR ENTRANCE".to_owned(), 0.2, acts)
            }
            TemperamentKind::Scholar => {
                let mut acts = vec![
                    Act::Do(
                        id,
                        vec![
                            go,
                            face,
                            Step::Beat(Beat::new(Gesture::Watch, ExpressionKind::Focused, 1.4)),
                        ],
                    ),
                    Act::Wait(1.0),
                ];
                blow(&mut acts, 0.18);
                acts.push(narrate(format!(
                    "{name} cites three sources. The Sovereign has no rebuttal."
                )));
                ("FOOTNOTE OF INFINITE RIGOUR".to_owned(), 0.18, acts)
            }
            TemperamentKind::Grump => {
                let mut acts = vec![
                    Act::Do(
                        id,
                        vec![
                            go,
                            face,
                            Step::Beat(with(
                                Beat::new(Gesture::Huff, ExpressionKind::Grumpy, 1.4),
                                Cue::Huff,
                            )),
                        ],
                    ),
                    Act::Wait(1.0),
                ];
                blow(&mut acts, 0.19);
                acts.push(narrate(format!(
                    "{name} glares. Somewhere, a Monday weeps."
                )));
                ("GLARE OF TEN THOUSAND MONDAYS".to_owned(), 0.19, acts)
            }
            TemperamentKind::Lazybones => {
                let mut acts = vec![
                    Act::Do(
                        id,
                        vec![
                            face,
                            Step::Beat(with(
                                Beat::new(ActionKind::Sleep, ExpressionKind::Sleepy, 2.4),
                                Cue::Sleep,
                            )),
                        ],
                    ),
                    Act::Wait(1.4),
                    boss("WHAT ARE YOU... DOING..."),
                    boss("...SO... DROWSY..."),
                ];
                blow(&mut acts, 0.2);
                acts.push(narrate(format!(
                    "{name} has fallen asleep, and the Sovereign nodded off watching. CRITICAL NAP."
                )));
                ("ETERNAL SLUMBER FIELD".to_owned(), 0.2, acts)
            }
            TemperamentKind::Sweetheart => {
                let mut acts = vec![
                    Act::Do(
                        id,
                        vec![
                            go,
                            face,
                            Step::Beat(with(
                                Beat::new(Gesture::Beg, ExpressionKind::Affectionate, 1.4),
                                Cue::Heart,
                            )),
                        ],
                    ),
                    Act::Wait(1.0),
                ];
                blow(&mut acts, 0.18);
                acts.push(narrate(format!(
                    "{name} hugs the Sovereign's pointy end. It has no idea what to do with that."
                )));
                ("HEARTFELT HUG ANNIHILATION".to_owned(), 0.18, acts)
            }
            TemperamentKind::Explorer => {
                let mut acts = vec![
                    Act::Do(
                        id,
                        vec![
                            Step::Stride { to: (230.0, 176.0) },
                            Step::Stride { to: (60.0, 196.0) },
                            go,
                            face,
                            Step::Beat(Beat::new(Gesture::Cheer, ExpressionKind::Joy, 0.8)),
                        ],
                    ),
                    Act::Wait(2.6),
                ];
                blow(&mut acts, 0.2);
                acts.push(narrate(format!(
                    "{name} runs a full lap of the clearing, looking for a way in. Found one."
                )));
                ("UNCHARTED TERRITORY DASH".to_owned(), 0.2, acts)
            }
            TemperamentKind::Wallflower => {
                let mut acts = vec![
                    Act::Do(
                        id,
                        vec![
                            Step::Beat(Beat::new(Gesture::Peek, ExpressionKind::Worried, 1.0)),
                            go,
                            face,
                            Step::Beat(Beat::new(Gesture::Peek, ExpressionKind::Worried, 0.8)),
                            Step::Beat(Beat::new(Gesture::Reach, ExpressionKind::Worried, 0.6)),
                        ],
                    ),
                    Act::Wait(2.0),
                    boss("WAS THAT... AN ATTACK?"),
                    Act::Wait(0.8),
                ];
                blow(&mut acts, 0.22);
                acts.push(narrate("It was.".to_owned()));
                ("HESITANT... ULTIMATE... PEEK".to_owned(), 0.22, acts)
            }
            TemperamentKind::Oddball => {
                let mut acts = vec![
                    Act::Do(
                        id,
                        vec![
                            go,
                            face,
                            Step::Beat(spun(Beat::new(Gesture::Cheer, ExpressionKind::Joy, 1.4))),
                        ],
                    ),
                    Act::Wait(1.0),
                ];
                blow(&mut acts, 0.19);
                acts.push(narrate(format!(
                    "{name} does something nobody understands, least of all the Sovereign."
                )));
                ("INCOMPREHENSIBLE GAMBIT OMEGA".to_owned(), 0.19, acts)
            }
            TemperamentKind::Troublemaker => {
                let mut acts = vec![
                    Act::Do(
                        id,
                        vec![
                            go,
                            face,
                            Step::Beat(Beat::new(Gesture::Bop, ExpressionKind::Smug, 1.2)),
                        ],
                    ),
                    Act::Wait(1.0),
                ];
                blow(&mut acts, 0.19);
                acts.push(narrate(format!(
                    "{name} empties a pocket at it: acorns, string, and one suspicious button."
                )));
                ("POCKET FULL OF CHAOS".to_owned(), 0.19, acts)
            }
            TemperamentKind::Guardian => {
                let mut acts = vec![
                    Act::Do(
                        id,
                        vec![
                            go,
                            face,
                            Step::Beat(Beat::new(
                                ActionKind::Idle,
                                ExpressionKind::Determined,
                                1.4,
                            )),
                        ],
                    ),
                    Act::Wait(1.0),
                ];
                blow(&mut acts, 0.18);
                acts.push(narrate(format!(
                    "{name} stands firm. The Sovereign's own click bounces back off it."
                )));
                ("UNBREAKABLE BULWARK".to_owned(), 0.18, acts)
            }
        };
        acts.insert(0, Act::Cutin(vec![id], title.clone()));
        acts.push(Act::Do(id, vec![back, face, Step::Beat(standing())]));
        Choice {
            name: title,
            acts,
            damage,
        }
    }

    /// Two close friends, together.
    fn together(&mut self) -> Choice {
        let ids: Vec<Id> = self.party.iter().map(|(id, _, _)| *id).collect();
        let names: Vec<String> = self.party.iter().map(|(_, _, name)| name.clone()).collect();
        let title = "BONDS BEYOND THE DESKTOP".to_owned();
        let mut acts = vec![Act::Cutin(ids.clone(), title.clone())];
        for (index, id) in ids.iter().enumerate() {
            let to = (
                FRONT.0 - 14.0 + index as f32 * 28.0,
                FRONT.1 + index as f32 * 4.0,
            );
            acts.push(Act::Do(
                *id,
                vec![
                    Step::Stride { to },
                    Step::FaceX(300.0),
                    Step::Beat(with(
                        Beat::new(Gesture::Cheer, ExpressionKind::Joy, 1.2),
                        Cue::Heart,
                    )),
                ],
            ));
        }
        acts.extend([
            Act::Wait(1.6),
            Act::Boss(Move::Reel),
            Act::Shake(0.9),
            Act::Damage(0.26),
            Act::Say(
                Speaker::Narrator,
                format!(
                    "{} and {}, together. It was never going to be close.",
                    names[0], names[1]
                ),
            ),
        ]);
        for id in &ids {
            acts.push(Act::Do(
                *id,
                vec![
                    Step::Walk { to: self.home(*id) },
                    Step::FaceX(300.0),
                    Step::Beat(standing()),
                ],
            ));
        }
        Choice {
            name: title,
            acts,
            damage: 0.26,
        }
    }

    /// The person holds out a snack: everyone feels braver, and the Sovereign, who cannot eat,
    /// seethes.
    fn snack(&mut self) -> Choice {
        let title = "HOLD OUT A SNACK".to_owned();
        let mut acts = Vec::new();
        for (id, _, _) in &self.party {
            acts.push(Act::Do(
                *id,
                vec![
                    Step::Beat(with(
                        Beat::new(ActionKind::Eat, ExpressionKind::Joy, 1.4),
                        Cue::Heart,
                    )),
                    Step::Beat(standing()),
                ],
            ));
        }
        acts.extend([
            Act::Wait(1.4),
            Act::Say(Speaker::Boss, "YOU DARE SNACK IN MY PRESENCE?".to_owned()),
            Act::Boss(Move::Reel),
            Act::Damage(0.12),
            Act::Say(
                Speaker::Narrator,
                "Everyone has a snack. Courage restored. The Sovereign, who cannot eat, seethes."
                    .to_owned(),
            ),
        ]);
        Choice {
            name: title,
            acts,
            damage: 0.12,
        }
    }

    /// The Sovereign's answer, after its current form.
    fn answer(&mut self) -> Vec<Act> {
        let pick = (self.dice.unit() * self.party.len() as f32) as usize % self.party.len().max(1);
        let Some((target, _, name)) = self.party.get(pick).cloned() else {
            return Vec::new();
        };
        let home = self.home(target);
        let boss = |text: &str| Act::Say(Speaker::Boss, text.to_owned());
        let narrate = |text: String| Act::Say(Speaker::Narrator, text);
        match self.phase {
            0 => vec![
                boss("CLICK."),
                Act::Boss(Move::Click { at: home }),
                Act::Do(
                    target,
                    vec![
                        Step::Beat(with(
                            Beat::new(Gesture::Gasp, ExpressionKind::Startled, 0.8),
                            Cue::Exclaim,
                        )),
                        Step::Beat(standing()),
                    ],
                ),
                Act::Shake(0.3),
                narrate(format!("{name} is clicked! It tickles, mostly.")),
            ],
            1 => {
                let to = (home.0 + 70.0, home.1 + 4.0);
                vec![
                    boss("I SHALL DRAG YOU TO THE RECYCLE BIN."),
                    Act::Do(
                        target,
                        vec![Step::Beat(spun(Beat::new(
                            Gesture::Cheer,
                            ExpressionKind::Startled,
                            2.6,
                        )))],
                    ),
                    Act::Boss(Move::Drag {
                        who: target,
                        from: home,
                        to,
                    }),
                    Act::Do(
                        target,
                        vec![
                            Step::Beat(Beat::new(Gesture::Huff, ExpressionKind::Grumpy, 0.8)),
                            Step::Walk { to: home },
                            Step::FaceX(300.0),
                            Step::Beat(standing()),
                        ],
                    ),
                    narrate(format!(
                        "{name} is dragged across the clearing and dropped. Undignified, but unharmed."
                    )),
                ]
            }
            _ => {
                let mut acts = vec![
                    boss("SELECT ALL."),
                    Act::Boss(Move::Select {
                        area: (60.0, 150.0, 172.0, 206.0),
                    }),
                ];
                for (id, _, _) in &self.party {
                    acts.push(Act::Do(
                        *id,
                        vec![
                            Step::Beat(Beat::new(Gesture::Crouch, ExpressionKind::Worried, 1.6)),
                            Step::Beat(standing()),
                        ],
                    ));
                }
                acts.extend([
                    boss("AND NOW... DELETE\u{2014}"),
                    narrate("But you can't delete friendship. Everyone pops straight back out of the selection.".to_owned()),
                ]);
                for (id, character, _) in &self.party {
                    acts.push(Act::Do(
                        *id,
                        vec![Step::Beat(character.celebrate(1.0)), Step::Beat(standing())],
                    ));
                }
                acts
            }
        }
    }

    /// The Sovereign gives up a form, and takes on the next.
    fn shift(&mut self) -> Vec<Act> {
        let boss = |text: &str| Act::Say(Speaker::Boss, text.to_owned());
        match self.phase {
            0 => vec![
                Act::Boss(Move::Reel),
                boss("YOU HAVE MERELY... HOVERED."),
                Act::Shake(0.6),
                boss("BEHOLD MY SECOND FORM: THE DRAG."),
            ],
            _ => vec![
                Act::Boss(Move::Reel),
                Act::Shake(0.8),
                boss("ENOUGH! I SHALL SELECT YOU ALL!"),
            ],
        }
    }

    /// The end: the rest of the colony comes running, and together they finish it.
    fn finale(&mut self) -> Vec<Act> {
        let narrate = |text: &str| Act::Say(Speaker::Narrator, text.to_owned());
        let boss = |text: &str| Act::Say(Speaker::Boss, text.to_owned());
        let party: Vec<Id> = self.party.iter().map(|(id, _, _)| *id).collect();
        // Spread round the two who came, clear of where they stand.
        const AROUND: [(f32, f32); 10] = [
            (44.0, 172.0),
            (176.0, 172.0),
            (60.0, 202.0),
            (184.0, 200.0),
            (136.0, 168.0),
            (96.0, 166.0),
            (216.0, 186.0),
            (30.0, 190.0),
            (150.0, 204.0),
            (220.0, 166.0),
        ];
        let arrivals: Vec<(Id, (f32, f32))> = self
            .rest
            .iter()
            .enumerate()
            .map(|(index, id)| {
                let (x, y) = AROUND[index % AROUND.len()];
                let further = (index / AROUND.len()) as f32 * 8.0;
                (*id, (x + further, y - further * 0.5))
            })
            .collect();
        let mut acts = vec![
            boss("YOU... ARE STILL... STANDING?"),
            narrate("Then, from the path, the sound of a great many small feet."),
            Act::Join(arrivals.clone()),
            Act::Wait(if self.rest.is_empty() { 0.2 } else { 3.0 }),
        ];
        let everyone: Vec<Id> = party
            .iter()
            .copied()
            .chain(self.rest.iter().copied())
            .collect();
        if !self.rest.is_empty() {
            acts.push(narrate("The whole colony has come."));
        }
        acts.push(Act::Cutin(
            everyone.iter().copied().take(4).collect(),
            "BONDS BEYOND THE DESKTOP: EVERYONE".to_owned(),
        ));
        for id in &everyone {
            acts.push(Act::Do(
                *id,
                vec![
                    Step::FaceX(300.0),
                    Step::Beat(Beat::new(Gesture::Cheer, ExpressionKind::Joy, 1.6)),
                    Step::Beat(standing()),
                ],
            ));
        }
        acts.extend([
            Act::Wait(1.2),
            Act::Shake(1.4),
            Act::Damage(1.0),
            Act::Boss(Move::Shatter),
            boss("IMPOSSIBLE... I WAS... THE POINTER... THAT POINTS..."),
            Act::Boss(Move::Flee),
            Act::Boss(Move::Gone),
            narrate("What is left of the Sovereign, a very ordinary little pointer, scurries off into the trees."),
            narrate("Where it stood lies a strange, heavy arrow, still warm. It comes home for the Hilltop."),
        ]);
        acts
    }
}

fn with(mut beat: Beat, cue: Cue) -> Beat {
    beat.cue = Some(cue);
    beat
}

fn spun(mut beat: Beat) -> Beat {
    beat.spin = true;
    beat
}

/// Standing ready, facing the Sovereign, for as long as it takes.
fn standing() -> Beat {
    Beat::new(ActionKind::Idle, ExpressionKind::Determined, 600.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clearing;

    fn sample() -> Cast {
        Cast::new(formiga_travel::sample::snapshot()).unwrap()
    }

    /// Plays the encounter through, always choosing the first thing offered and reading on at
    /// once. Returns how many choices it took.
    fn play_through(
        cast: &Cast,
        party: Vec<Id>,
        close: bool,
        pick: impl Fn(usize) -> usize,
    ) -> (usize, Vec<Event>, Sovereign, Playground) {
        let mut ground = clearing::open(cast, &party, 0.0);
        let mut sovereign = Sovereign::new(&mut ground, cast, &party, close, 0.0);
        let mut events = Vec::new();
        let mut chosen = 0;
        let mut now = 0.0;
        while now < 900.0 && sovereign.state() != State::Over {
            now += 1.0 / 30.0;
            ground.tick(cast, now);
            sovereign.tick(&mut ground, cast, now);
            sovereign.stage.read_on();
            if sovereign.state() == State::Choosing {
                let count = sovereign.choices().len();
                sovereign.choose(pick(chosen) % count);
                chosen += 1;
            }
            events.extend(sovereign.take_events());
        }
        (chosen, events, sovereign, ground)
    }

    #[test]
    fn whatever_is_chosen_the_sovereign_falls_in_a_handful_of_turns() {
        let cast = sample();
        for party in [
            vec![cast.members[0].id],
            vec![cast.members[1].id, cast.members[2].id],
        ] {
            for pick in [0, 1, 7] {
                let (turns, events, sovereign, _) =
                    play_through(&cast, party.clone(), false, |n| n * pick);
                assert_eq!(events, vec![Event::Bested], "with {party:?} picking {pick}");
                assert!((4..=14).contains(&turns), "{turns} turns");
                assert_eq!(sovereign.stage.health, 0.0);
            }
        }
    }

    #[test]
    fn everyone_has_their_own_attack_and_a_close_pair_has_one_together() {
        let cast = sample();
        let party = vec![cast.members[0].id, cast.members[1].id];
        let mut ground = clearing::open(&cast, &party, 0.0);
        let mut sovereign = Sovereign::new(&mut ground, &cast, &party, true, 0.0);
        let menu = sovereign.menu();
        assert_eq!(menu.len(), 4, "two attacks, one together, and a snack");
        assert!(
            menu.iter()
                .any(|choice| choice.name == "BONDS BEYOND THE DESKTOP")
        );
        let names: std::collections::BTreeSet<String> = cast
            .members
            .iter()
            .map(|member| {
                let character = Character::of(member);
                sovereign
                    .signature(member.id, &character, &member.name)
                    .name
            })
            .collect();
        assert!(
            names.len() >= 3,
            "the sample colony all attack alike: {names:?}"
        );
    }

    #[test]
    fn it_beckons_only_a_well_travelled_colony_that_watches_the_sky() {
        use crate::memories::{ColonyMemories, FindRecord};
        let mut colony = ColonyMemories::default();
        assert!(!may_beckon(&colony));
        for find in crate::finds::CATALOGUE.iter().take(10) {
            colony
                .finds
                .insert(find.id.to_owned(), FindRecord::default());
        }
        assert!(
            !may_beckon(&colony),
            "nothing on the Hilltop to watch the sky with"
        );
        colony.hilltop.insert(3, "brass_lens".into());
        assert!(may_beckon(&colony));
        colony.sovereign_bested = 1;
        colony.outings = 4;
        assert!(!may_beckon(&colony), "once seen off, only now and then");
        colony.outings = 6;
        assert!(may_beckon(&colony));
    }

    #[test]
    fn at_the_end_the_whole_colony_comes_running() {
        let cast = sample();
        let party = vec![cast.members[0].id];
        let (_, _, _, ground) = play_through(&cast, party, false, |_| 0);
        assert_eq!(ground.ids().len(), cast.members.len());
    }
}

//! The machinery of a hidden set piece: an interactive cutscene played on a stage. A script is a
//! list of acts (a line said, a close-up with a title, a creature told what to do, the boss making
//! a move, the screen shaking, a blow landing, others arriving), played one after another, each
//! waiting as long as it needs. There are no hit points to manage and nothing to lose: the "battle"
//! is pacing, and what the person chooses only picks which scene plays next.

use super::boss::{self, Move};
use crate::actor::Step;
use crate::cast::{Cast, Id};
use crate::playground::Playground;
use std::collections::VecDeque;

/// How long a line stays up if nobody reads on, and how long a close-up lasts.
pub const LINE_SECS: f32 = 6.0;
pub const CUTIN_SECS: f32 = 1.8;
/// How fast the health bar follows a blow, per second.
const BAR_SPEED: f32 = 0.6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Speaker {
    Boss,
    Member(Id),
    Narrator,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Act {
    /// A line, shown until read on.
    Say(Speaker, String),
    /// A close-up across the screen of these few, with a title.
    Cutin(Vec<Id>, String),
    /// Tells a creature what to do, and carries straight on.
    Do(Id, Vec<Step>),
    Wait(f32),
    /// The boss makes a move, and the stage waits for it.
    Boss(Move),
    Shake(f32),
    /// A blow lands: the health bar drops by this much.
    Damage(f32),
    /// Others arrive, each to its place.
    Join(Vec<(Id, (f32, f32))>),
}

pub struct Stage {
    queue: VecDeque<Act>,
    /// The act being waited on, and since when.
    current: Option<(Act, f32)>,
    pub line: Option<(Speaker, String)>,
    /// The close-up showing, and since when.
    pub cutin: Option<(Vec<Id>, String, f32)>,
    /// The boss's move, and since when.
    pub boss: (Move, f32),
    /// The boss's health from 1 to 0, and the bar as drawn, catching up.
    pub health: f32,
    pub bar: f32,
    pub shake_until: f32,
    read_on: bool,
    clock: f32,
}

impl Stage {
    pub fn new(now: f32) -> Self {
        Self {
            queue: VecDeque::new(),
            current: None,
            line: None,
            cutin: None,
            boss: (Move::Hover, now),
            health: 1.0,
            bar: 1.0,
            shake_until: 0.0,
            read_on: false,
            clock: now,
        }
    }

    pub fn play(&mut self, acts: impl IntoIterator<Item = Act>) {
        self.queue.extend(acts);
    }

    /// Whether every act has played out.
    pub fn idle(&self) -> bool {
        self.queue.is_empty() && self.current.is_none()
    }

    /// The person reads on past the line showing.
    pub fn read_on(&mut self) {
        if self.line.is_some() {
            self.read_on = true;
        }
    }

    pub fn tick(&mut self, ground: &mut Playground, cast: &Cast, now: f32) {
        let dt = (now - self.clock).clamp(0.0, 0.1);
        self.clock = now;
        loop {
            if let Some((act, since)) = &self.current {
                let age = now - since;
                let done = match act {
                    Act::Say(..) => self.read_on || age >= LINE_SECS,
                    Act::Cutin(..) => age >= CUTIN_SECS,
                    Act::Wait(secs) => age >= *secs,
                    Act::Boss(step) => age >= step.secs(),
                    _ => true,
                };
                if !done {
                    break;
                }
                match act {
                    Act::Say(..) => self.line = None,
                    Act::Cutin(..) => self.cutin = None,
                    Act::Boss(Move::Drag { who, to, .. }) => {
                        ground.teleport(*who, *to);
                        ground.set_lift(*who, 0.0);
                        self.boss = (Move::Hover, now);
                    }
                    Act::Boss(Move::Shatter | Move::Flee | Move::Gone) => {}
                    Act::Boss(_) => self.boss = (Move::Hover, now),
                    _ => {}
                }
                self.read_on = false;
                self.current = None;
            }
            let Some(act) = self.queue.pop_front() else {
                break;
            };
            match &act {
                Act::Say(speaker, text) => self.line = Some((*speaker, text.clone())),
                Act::Cutin(who, title) => self.cutin = Some((who.clone(), title.clone(), now)),
                Act::Do(id, steps) => ground.direct(*id, steps.clone(), now),
                Act::Boss(step) => self.boss = (*step, now),
                Act::Shake(secs) => self.shake_until = now + secs,
                Act::Damage(amount) => self.health = (self.health - amount).max(0.0),
                Act::Join(arrivals) => {
                    for (id, to) in arrivals {
                        ground.join(cast, *id, *to, now);
                    }
                }
                Act::Wait(_) => {}
            }
            self.current = Some((act, now));
        }
        // Whoever is being dragged hangs from the cursor's tip.
        if let (Move::Drag { who, .. }, since) = self.boss {
            let tip = boss::tip(self.boss.0, now - since);
            ground.teleport(who, (tip.0, tip.1 + 30.0));
        }
        if self.bar > self.health {
            self.bar = (self.bar - BAR_SPEED * dt).max(self.health);
        }
    }
}

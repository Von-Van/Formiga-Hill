//! The time of day at the Hill, which is the time of day where the person is: a morning visit
//! finds the Hill in the morning, a late one finds it under the stars. Nothing depends on it but
//! the look of things. Every game plays the same at any hour, and nobody is kept waiting for one.
//!
//! Scenes are painted by day. The light of the hour is laid over the finished picture, everyone
//! in it included, so the colony stands in the same light as everything around it. Then
//! whatever gives off light of its own (lit windows, lamps, a fire, the stars) is laid back over
//! what nobody is standing in front of.

use crate::paint::{noise, over, put, rgb, rgba};
use formiga_art::{Canvas, Rgba};
use time::{OffsetDateTime, UtcOffset};

/// The light at one hour of the day, the hours between taking a share of each side.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Key {
    hour: f32,
    /// What every colour is multiplied by outdoors.
    ambient: [f32; 3],
    /// How lit lamps, windows and fires look, against the daylight.
    lamps: f32,
    /// How many stars are out.
    stars: f32,
}

const fn key(hour: f32, ambient: u32, lamps: f32, stars: f32) -> Key {
    Key {
        hour,
        ambient: [
            ((ambient >> 16) & 0xff) as f32 / 255.0,
            ((ambient >> 8) & 0xff) as f32 / 255.0,
            (ambient & 0xff) as f32 / 255.0,
        ],
        lamps,
        stars,
    }
}

/// A day at the Hill, from midnight round to midnight. Night is a cool blue, never so dark that
/// the near-black outlines of the colony stop reading; dawn comes up rose, and the evening goes
/// down gold.
const DAY: [Key; 10] = [
    key(0.0, 0x6070a8, 1.0, 1.0),
    key(4.5, 0x6070a8, 1.0, 1.0),
    key(6.0, 0xc4a4b8, 0.6, 0.25),
    key(7.5, 0xfff2e2, 0.0, 0.0),
    key(10.0, 0xffffff, 0.0, 0.0),
    key(16.5, 0xffffff, 0.0, 0.0),
    key(18.5, 0xffd9a8, 0.3, 0.0),
    key(19.75, 0xc08c9c, 0.85, 0.3),
    key(21.0, 0x6070a8, 1.0, 1.0),
    key(24.0, 0x6070a8, 1.0, 1.0),
];

/// Indoors, the lamps are lit whenever it gets dark, so a room only ever dims to lamplight.
const LAMPLIGHT: [f32; 3] = [0.86, 0.74, 0.62];

/// Where the Hill's hours come from.
#[derive(Clone, Copy, Debug)]
pub enum Clock {
    /// The person's own time of day, this far from UTC. Read once at start-up, before Hill has
    /// more than one thread, as reading it later cannot be done safely everywhere.
    Local(UtcOffset),
    /// One hour, held all visit: for seeing how the Hill looks at it.
    Held(f32),
}

impl Clock {
    pub fn daylight(&self) -> Daylight {
        match *self {
            Clock::Local(offset) => Daylight::now(offset),
            Clock::Held(hour) => Daylight::at_hour(hour),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Daylight {
    hour: f32,
}

impl Default for Daylight {
    /// Midday: how every scene is painted.
    fn default() -> Self {
        Self::at_hour(12.0)
    }
}

impl Daylight {
    pub fn at_hour(hour: f32) -> Self {
        Self {
            hour: if hour.is_finite() {
                hour.rem_euclid(24.0)
            } else {
                12.0
            },
        }
    }

    /// The hour where the person is.
    fn now(offset: UtcOffset) -> Self {
        let local = OffsetDateTime::now_utc().to_offset(offset);
        Self::at_hour(
            f32::from(local.hour())
                + f32::from(local.minute()) / 60.0
                + f32::from(local.second()) / 3600.0,
        )
    }

    pub fn hour(&self) -> f32 {
        self.hour
    }

    fn key(&self) -> Key {
        let next = DAY
            .iter()
            .position(|key| key.hour > self.hour)
            .unwrap_or(DAY.len() - 1);
        let (a, b) = (DAY[next.saturating_sub(1)], DAY[next]);
        let t = ((self.hour - a.hour) / (b.hour - a.hour).max(f32::EPSILON)).clamp(0.0, 1.0);
        let lerp = |x: f32, y: f32| x + (y - x) * t;
        Key {
            hour: self.hour,
            ambient: [
                lerp(a.ambient[0], b.ambient[0]),
                lerp(a.ambient[1], b.ambient[1]),
                lerp(a.ambient[2], b.ambient[2]),
            ],
            lamps: lerp(a.lamps, b.lamps),
            stars: lerp(a.stars, b.stars),
        }
    }

    /// What every colour outdoors is multiplied by.
    pub fn ambient(&self) -> [f32; 3] {
        self.key().ambient
    }

    /// The same indoors: dimmer as the day goes, but warm, and never darker than lamplight.
    pub fn indoors(&self) -> [f32; 3] {
        let outside = self.ambient();
        let mut inside = [0.0; 3];
        for channel in 0..3 {
            inside[channel] = outside[channel].max(LAMPLIGHT[channel]);
        }
        inside
    }

    /// How lit lamps and windows look, 0 by day and 1 by night.
    pub fn lamps(&self) -> f32 {
        self.key().lamps
    }

    /// How many of the stars are out, 0 by day and 1 by night.
    pub fn stars(&self) -> f32 {
        self.key().stars
    }

    /// How much darker than midday it is outdoors, from 0 by day to about a half at night.
    pub fn darkness(&self) -> f32 {
        (1.0 - self.ambient().iter().sum::<f32>() / 3.0).clamp(0.0, 1.0)
    }
}

/// What gives off light of its own in a scene, to be laid back over the hour's light: lamps, lit
/// windows and fires, shown as strongly as lamps are lit; and the night sky, deepened, with its
/// stars and moon, shown as strongly as the stars are out.
pub struct Nightlights {
    pub lamps: Canvas,
    pub sky: Canvas,
    /// Whether the scene is a room, lit by lamps whenever it gets dark.
    pub indoors: bool,
}

impl Nightlights {
    /// Lights a finished picture for the hour. `painted` is what the scene shows with nobody in
    /// it, so that a lamp or a star never shines through someone standing in front of it.
    pub fn light(&self, frame: &mut Canvas, painted: &Canvas, daylight: Daylight) {
        let ambient = if self.indoors {
            daylight.indoors()
        } else {
            daylight.ambient()
        };
        light(
            frame,
            ambient,
            painted,
            &[
                (&self.sky, daylight.stars()),
                (&self.lamps, daylight.lamps()),
            ],
        );
    }
}

/// The night sky over `painted`, where it still shows `sky` (the sky as painted on its own,
/// clouds and all): a deep blue wash, lighter towards `horizon`; stars in the clear parts,
/// thinning as they go down; and the moon, if it is given and in clear sky.
pub fn night_sky(painted: &Canvas, sky: &Canvas, horizon: i32, moon: Option<(i32, i32)>) -> Canvas {
    let (width, height) = (painted.width() as i32, painted.height() as i32);
    let mut night = Canvas::new(painted.width(), painted.height());
    let is_sky = |x: i32, y: i32| {
        x >= 0 && y >= 0 && x < width && y < height && {
            let pixel = painted.get(x, y);
            pixel.a > 0 && pixel == sky.get(x, y)
        }
    };
    // Clouds take the wash but hide the stars.
    let clear = |x: i32, y: i32| {
        let pixel = painted.get(x, y);
        u32::from(pixel.r) + u32::from(pixel.g) + u32::from(pixel.b) < 3 * 236
            || pixel.b > pixel.r.saturating_add(8)
    };
    for y in 0..height {
        let depth = (1.0 - y as f32 / horizon.max(1) as f32).clamp(0.0, 1.0);
        for x in 0..width {
            if !is_sky(x, y) {
                continue;
            }
            night.set(x, y, rgba(0x141c3a, (90.0 + depth * 90.0) as u8));
            let hash = noise(x, y, 811) % 1000;
            if clear(x, y) && hash < (7.0 * depth * depth) as u32 {
                let bright = if hash < 2 { 255 } else { 170 };
                night.set(x, y, Rgba::new(0xf4, 0xf0, 0xdc, bright));
            }
        }
    }
    if let Some((cx, cy)) = moon {
        for dy in -6..=6 {
            for dx in -6..=6 {
                let (x, y) = (cx + dx, cy + dy);
                if !is_sky(x, y) {
                    continue;
                }
                let reach = dx * dx + dy * dy;
                if reach <= 20 {
                    // A crescent: the moon's own night side in shadow.
                    let shade = (dx + 2) * (dx + 2) + (dy + 1) * (dy + 1) <= 12;
                    night.set(x, y, if shade { rgb(0x9aa0bc) } else { rgb(0xfdf6dc) });
                } else if reach <= 36 {
                    night.set(x, y, rgba(0xc8d0ec, (130 - (reach - 20) * 6) as u8));
                }
            }
        }
    }
    night
}

/// Fireflies after dark: in each patch `(left, top, wide, tall, how many)`, points of green-gold
/// light with a little glow about each.
pub fn fireflies(canvas: &mut Canvas, haunts: &[(i32, i32, i32, i32, i32)], salt: u32) {
    for (patch, &(left, top, wide, tall, count)) in haunts.iter().enumerate() {
        let salt = salt + patch as u32 * 7;
        for index in 0..count {
            let x = left + (noise(index, 0, salt) % wide.max(1) as u32) as i32;
            let y = top + (noise(index, 1, salt) % tall.max(1) as u32) as i32;
            glow(canvas, (x, y), 5, rgb(0xd8f07a));
            put(canvas, x, y, rgb(0xf2ffb8));
        }
    }
}

/// A soft round glow for a lamp's light on whatever is around it, strongest in the middle and
/// falling away smoothly, a little flattened as light on the ground is.
pub fn glow(canvas: &mut Canvas, (cx, cy): (i32, i32), radius: i32, color: Rgba) {
    let radius = radius.max(1);
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let reach = ((dx * dx) as f32 + (dy * dy) as f32 * 1.6).sqrt() / radius as f32;
            if reach >= 1.0 {
                continue;
            }
            let alpha = f32::from(color.a) * 0.7 * (1.0 - reach).powi(2);
            if alpha >= 1.0 {
                put(
                    canvas,
                    cx + dx,
                    cy + dy,
                    Rgba::new(color.r, color.g, color.b, alpha as u8),
                );
            }
        }
    }
}

/// Lays `light` over a finished picture, then gives back what glows of its own accord: each
/// layer of `glows` is laid on, as strongly as it says, wherever the picture still shows
/// `painted` (that is, wherever nobody stands in front of it).
pub fn light(frame: &mut Canvas, light: [f32; 3], painted: &Canvas, glows: &[(&Canvas, f32)]) {
    let unchanged = light.iter().all(|channel| *channel >= 0.999);
    let glows: Vec<(&Canvas, f32)> = glows
        .iter()
        .filter(|(_, strength)| *strength > 0.0)
        .map(|(canvas, strength)| (*canvas, strength.min(1.0)))
        .collect();
    if unchanged && glows.is_empty() {
        return;
    }
    let (width, height) = (frame.width() as i32, frame.height() as i32);
    let scale = |value: u8, by: f32| (f32::from(value) * by).round().clamp(0.0, 255.0) as u8;
    for y in 0..height {
        for x in 0..width {
            let pixel = frame.get(x, y);
            let mut lit = Rgba::new(
                scale(pixel.r, light[0]),
                scale(pixel.g, light[1]),
                scale(pixel.b, light[2]),
                pixel.a,
            );
            if !glows.is_empty() && pixel == painted.get(x, y) {
                for (layer, strength) in &glows {
                    let shine = layer.get(x, y);
                    if shine.a > 0 {
                        let alpha = (f32::from(shine.a) * strength).round() as u8;
                        lit = over(Rgba::new(shine.r, shine.g, shine.b, alpha), lit);
                    }
                }
            }
            frame.set(x, y, lit);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn midday_leaves_a_picture_as_it_was_painted() {
        let day = Daylight::at_hour(12.0);
        assert_eq!(day.ambient(), [1.0, 1.0, 1.0]);
        assert_eq!(day.darkness(), 0.0);
        let mut frame = Canvas::new(4, 4);
        frame.fill_rect(0, 0, 4, 4, rgb(0x86bb7c));
        let before = frame.clone();
        let none = Canvas::new(4, 4);
        light(&mut frame, day.ambient(), &before, &[(&none, day.lamps())]);
        assert_eq!(frame, before);
    }

    #[test]
    fn night_is_dark_and_blue_but_not_black() {
        let night = Daylight::at_hour(1.0);
        let [r, g, b] = night.ambient();
        assert!(r < 0.5 && g < 0.5);
        assert!(b > r && b > g, "night should lean blue");
        assert!(
            r > 0.3,
            "night should never go so dark that outlines stop reading"
        );
        assert_eq!(night.lamps(), 1.0);
        assert_eq!(night.stars(), 1.0);
    }

    #[test]
    fn the_day_wraps_round_midnight_without_a_jump() {
        let before = Daylight::at_hour(23.99).ambient();
        let after = Daylight::at_hour(0.01).ambient();
        for channel in 0..3 {
            assert!((before[channel] - after[channel]).abs() < 0.01);
        }
        assert_eq!(Daylight::at_hour(24.0), Daylight::at_hour(0.0));
        assert_eq!(Daylight::at_hour(-1.0).hour(), 23.0);
    }

    #[test]
    fn the_light_changes_gradually_through_the_day() {
        let mut last = Daylight::at_hour(0.0).ambient();
        let mut minute = 0;
        while minute < 24 * 60 {
            minute += 5;
            let now = Daylight::at_hour(minute as f32 / 60.0).ambient();
            for channel in 0..3 {
                assert!(
                    (now[channel] - last[channel]).abs() < 0.03,
                    "a jump in the light at {minute} minutes past midnight"
                );
            }
            last = now;
        }
    }

    #[test]
    fn stars_come_out_only_in_the_sky() {
        let mut painted = Canvas::new(40, 40);
        painted.fill_rect(0, 0, 40, 20, rgb(0x6fa3d8));
        painted.fill_rect(0, 20, 40, 20, rgb(0x86bb7c));
        let mut only_sky = Canvas::new(40, 40);
        only_sky.fill_rect(0, 0, 40, 20, rgb(0x6fa3d8));
        let sky = night_sky(&painted, &only_sky, 20, Some((20, 8)));
        for y in 0..40 {
            for x in 0..40 {
                if y >= 20 {
                    assert_eq!(sky.get(x, y).a, 0, "a star on the grass at {x}, {y}");
                }
            }
        }
        assert!(sky.get(20, 8).a == 255, "no moon");
        assert!(sky.get(2, 2).a > 0, "the sky is not deepened");
    }

    #[test]
    fn indoors_never_dims_past_lamplight() {
        for hour in 0..24 {
            let inside = Daylight::at_hour(hour as f32).indoors();
            for channel in 0..3 {
                assert!(inside[channel] >= LAMPLIGHT[channel]);
            }
        }
    }

    #[test]
    fn what_glows_shows_only_where_nobody_stands_in_front_of_it() {
        let night = Daylight::at_hour(2.0);
        let mut painted = Canvas::new(2, 1);
        painted.fill_rect(0, 0, 2, 1, rgb(0x404040));
        let mut glow = Canvas::new(2, 1);
        glow.fill_rect(0, 0, 2, 1, rgb(0xffd77a));
        let mut frame = painted.clone();
        // Someone stands in front of the second pixel.
        frame.set(1, 0, rgb(0x202020));
        light(
            &mut frame,
            night.ambient(),
            &painted,
            &[(&glow, night.lamps())],
        );
        assert_eq!(frame.get(0, 0), rgb(0xffd77a));
        assert_ne!(frame.get(1, 0), rgb(0xffd77a));
    }
}

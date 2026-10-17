//! The card Hill opens on when someone opens it themselves, from Applications or the Start menu,
//! rather than Formiga Desktop sending a colony. Desktop's sample colony arrives as it always
//! has, and the card over it says where a colony of the person's own comes from: by train, from
//! the Formiga icon. Looking around with the sample puts the card away; closing closes the Hill.

use super::{Card, HillApp, paper};
use eframe::egui;

/// The card's title.
const TITLE: &str = "Your colony comes from Formiga";

/// The card's two buttons.
const LOOK_AROUND: &str = "Look around with a sample colony";
const CLOSE: &str = "Close";

/// How wide the card is, as a share of the scene: room for the title on one line.
const WIDTH: f32 = 0.56;

/// What the card says, paragraph by paragraph, in the words Desktop's own menu uses.
fn words() -> [String; 2] {
    // Desktop's icon is in the menu bar on a Mac, and in the notification area on Windows.
    let icon = if cfg!(target_os = "macos") {
        "the menu bar"
    } else {
        "the notification area"
    };
    [
        "Formiga Hill opens when your colony takes the train from Formiga.".to_owned(),
        format!(
            "Open the Formiga icon in {icon} and choose \u{201c}Go to Formiga Hill\u{2026}\u{201d}."
        ),
    ]
}

/// Where Formiga is to be had, for anyone who hasn't it yet.
const GET_FORMIGA: &str = "Don't have Formiga yet? It's at formigaworld.com.";

impl HillApp {
    pub(super) fn welcome_window(&mut self, ctx: &egui::Context) {
        if !self.showing(Card::Welcome) {
            return;
        }
        let mut look_around = false;
        let mut close = false;
        self.show_card(ctx, Card::Welcome, TITLE, WIDTH, |_, ui| {
            for paragraph in words() {
                paper::words(ui, paragraph);
            }
            paper::aside(ui, GET_FORMIGA);
            paper::rule(ui);
            ui.horizontal_wrapped(|ui| {
                look_around = paper::button(ui, LOOK_AROUND).clicked();
                close = paper::button(ui, CLOSE).clicked();
            });
        });
        if look_around {
            self.card = None;
        }
        if close {
            // A rehearsal has nobody waiting for it, so closing writes nothing for Desktop.
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lettering;
    use crate::station::SCENE_WIDTH;

    #[test]
    fn the_welcome_card_names_desktops_own_menu_item() {
        let [opens, how] = words();
        assert!(opens.contains("train from Formiga"));
        assert!(how.contains("\u{201c}Go to Formiga Hill\u{2026}\u{201d}"));
        let icon = if cfg!(target_os = "macos") {
            "menu bar"
        } else {
            "notification area"
        };
        assert!(how.contains(icon), "{how}");
        assert!(GET_FORMIGA.contains("formigaworld.com"));
    }

    #[test]
    fn the_welcome_cards_title_and_buttons_fit_on_it() {
        // The card's room inside its margins, in paper pixels, at the scene's own size.
        let room = (SCENE_WIDTH as f32 * WIDTH) as i32 - 6 * 2 - 6 * 2;
        let close_button = 2 + 4 * 2 + lettering::width("\u{d7}");
        assert!(
            lettering::width(&TITLE.to_uppercase()) + 3 + close_button <= room,
            "the title wraps"
        );
        for label in [LOOK_AROUND, CLOSE] {
            assert!(
                2 + 4 * 2 + lettering::width(&label.to_uppercase()) <= room,
                "{label} is wider than the card"
            );
        }
    }
}

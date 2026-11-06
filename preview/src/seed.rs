//! Placeholder text for the terminal viewports in the browser preview.
//!
//! No shell runs in a browser, so a feed's live output cannot appear here;
//! the text says so. It is sized to each viewport: the LCARS bridge
//! terminal is 10 columns by 2 rows, so a long line would wrap into
//! fragments and only the last rows show.

use crate::core::panel::Panel;

/// Fill every terminal viewport with its labelled placeholder text.
pub(crate) fn seed_terminals(panel: &mut Panel) {
    let sizes: Vec<(String, usize)> = panel
        .terms()
        .iter()
        .map(|term| (term.name().to_string(), term.columns()))
        .collect();
    for (name, columns) in sizes {
        let text = match name.as_str() {
            "bridge" if columns >= 30 => {
                "bridge — browser preview\nno shell runs here\npress 1-4: the command is shown, not run"
            }
            "bridge" => "LCARS 4711\npreview",
            "telemetry" if columns >= 30 => "telemetry — browser preview\nfeeds are disabled here",
            "telemetry" => "preview",
            "clock" => "--:--:--",
            _ => "preview",
        };
        panel.write(&name, text);
    }
}

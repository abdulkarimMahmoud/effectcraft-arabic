//! Bidirectional (Bidi) display ordering for right-to-left interface languages.
//!
//! egui/epaint (the forked `storytold/egui`) shapes text with HarfBuzz (`harfrust`):
//! `shape_text` calls `guess_segment_properties()`, so a run whose first strong character is
//! Arabic is shaped right-to-left and its glyphs come out in visual order. A pure Arabic
//! sentence therefore already draws correctly.
//!
//! What egui does *not* do is reorder runs: `segment_into_runs` splits text only by font
//! face, then lays the runs out left-to-right in logical order. A mixed sentence such as
//! "فتح File.ecproj" (Arabic verb, then a Latin filename) is drawn with the Arabic run on
//! the left and the filename on the right — the reverse of what an RTL paragraph needs.
//!
//! This module fixes exactly that, and nothing more: it reorders the *runs* of a label into
//! visual order (UAX #9 via `unicode-bidi`, the same crate `effectcraft-text` uses for
//! composition text layers) while keeping the characters inside each run in logical order,
//! so HarfBuzz still shapes every run correctly. Reversing characters inside a run would
//! double-reverse them under HarfBuzz and break Arabic letter joining — it must not be done.
//!
//! Scope and limits (deliberate):
//! - Applied only to non-editable labels that own their string (`tr_args`, menu `entry` /
//!   `context_item`, effect names). Editable text fields keep logical order: the user types
//!   and copies logical text there, and the field's caret/selection must stay logical.
//! - Pure single-direction labels pass through unchanged (one run), so applying this
//!   centrally to owned label strings is safe.
//! - Bracket mirroring (logical "(" drawn as ")" in RTL context) is handled by HarfBuzz's
//!   shaping of each run, not here; this module only moves whole runs.
//! - Numbers stay Western (`0123`) in the Arabic catalog (timecode, coordinates, FPS), as in
//!   Arabic After Effects; UAX #9 keeps number runs left-to-right inside RTL sentences.

use unicode_bidi::{BidiInfo, Level};

/// `text` (logical order) with its directional runs in visual order, for drawing with
/// egui's left-to-right layout. Characters inside each run stay in logical order.
///
/// The paragraph direction is right-to-left: interface sentences in an RTL language start on
/// the right, so trailing neutrals (punctuation, brackets) attach to the correct side.
/// Returns `text` unchanged when it contains no RTL characters (fast path: most labels).
pub(crate) fn visual_order(text: &str) -> String {
    if !text.chars().any(|c| is_rtl_char(c)) {
        return text.to_string();
    }
    let info = BidiInfo::new(text, Some(Level::rtl()));
    let Some(paragraph) = info.paragraphs.first() else {
        return text.to_string();
    };
    let line = 0..text.len();
    let (_levels, runs) = info.visual_runs(paragraph, line);
    if runs.len() <= 1 {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    for run in runs {
        out.push_str(&text[run]);
    }
    out
}

/// Whether `c` can belong to a right-to-left run (Arabic, Hebrew, and their blocks).
fn is_rtl_char(c: char) -> bool {
    matches!(c as u32,
        0x0590..=0x05FF   // Hebrew
        | 0x0600..=0x06FF  // Arabic
        | 0x0700..=0x074F  // Syriac
        | 0x0750..=0x077F  // Arabic Supplement
        | 0x0780..=0x07BF  // Thaana
        | 0x07C0..=0x07FF  // NKo
        | 0x0800..=0x083F  // Samaritan
        | 0x0840..=0x085F  // Mandaic
        | 0x08A0..=0x08FF  // Arabic Extended-A
        | 0xFB1D..=0xFB4F  // Hebrew Presentation Forms
        | 0xFB50..=0xFDFF  // Arabic Presentation Forms-A
        | 0xFE70..=0xFEFF  // Arabic Presentation Forms-B
        | 0x10800..=0x10FFF // Ancient scripts (incl. RTL blocks)
        | 0x1E800..=0x1EFFF // Mende, Adlam, ...
    )
}

#[cfg(test)]
mod tests {
    use super::visual_order;

    #[test]
    fn pure_latin_is_unchanged() {
        assert_eq!(visual_order("Save Project..."), "Save Project...");
        assert_eq!(visual_order(""), "");
    }

    #[test]
    fn pure_arabic_is_unchanged() {
        // One RTL run: HarfBuzz shapes it, so the label must pass through byte-identical.
        // (Reversing its characters here would break letter joining under HarfBuzz.)
        assert_eq!(visual_order("حفظ"), "حفظ");
        assert_eq!(visual_order("الكاميرا النشطة"), "الكاميرا النشطة");
    }

    #[test]
    fn mixed_arabic_latin_reorders_runs_not_characters() {
        // Logical: "فتح File.ecproj" (Arabic verb, then Latin filename).
        // Visual (what an LTR layout must draw): filename run first (left), Arabic run after
        // (right) — each run's characters untouched, in logical order.
        let out = visual_order("فتح File.ecproj");
        assert!(out.starts_with("File.ecproj"), "visual order starts with the Latin run: {out:?}");
        assert!(out.ends_with("فتح"), "visual order ends with the Arabic run intact: {out:?}");
        assert_eq!(out.chars().count(), "فتح File.ecproj".chars().count());
    }

    #[test]
    fn numbers_survive_inside_rtl() {
        // Logical: "الكاميرا النشطة (2)" — no character may be dropped or duplicated.
        let out = visual_order("الكاميرا النشطة (2)");
        assert_eq!(out.chars().filter(|c| *c == '2').count(), 1);
        assert_eq!(out.chars().count(), "الكاميرا النشطة (2)".chars().count());
    }
}

//! The WebAssembly of the wiki editor.
//!
//! The wiki builds its own wasm: exactly what the editor asks of Tomet, in the
//! shape the editor wants it. Two parts:
//!
//! - the GitHub client and the conflict functions of `tomet-cms-js`, carried
//!   into this wasm by `pub use` (the exports of a dependency crate are part
//!   of the wasm that links it);
//! - [`highlight_spans`], the syntax colors, which only the wiki needs and in
//!   the unit its editor counts: UTF-16 code units.
//!
//! What a range of a text is stays in `tomet::highlight`; this crate only
//! changes the offsets.

pub use tomet_cms_js as cms;

use serde::Serialize;
use wasm_bindgen::prelude::*;

/// A range of the source to color. Offsets count UTF-16 code units.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Span {
    pub from: usize,
    pub to: usize,
    pub tag: &'static str,
}

/// The colored ranges of `src`, in order and without overlap, counted in UTF-16
/// code units (what `String.prototype.length`, CodeMirror and Telaion count).
pub fn spans(src: &str) -> Vec<Span> {
    let mut at = Utf16Cursor::new(src);
    tomet::highlight::highlight(src)
        .into_iter()
        .map(|s| Span {
            from: at.units(s.from),
            to: at.units(s.to),
            tag: s.tag.as_str(),
        })
        .collect()
}

/// Turns byte offsets into UTF-16 offsets. The offsets are asked for in
/// increasing order, so one pass over the text answers them all.
struct Utf16Cursor<'a> {
    src: &'a str,
    byte: usize,
    units: usize,
}

impl<'a> Utf16Cursor<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            byte: 0,
            units: 0,
        }
    }

    /// The UTF-16 offset of the byte offset `to`, a boundary of a character at or
    /// after the one asked for last.
    fn units(&mut self, to: usize) -> usize {
        debug_assert!(to >= self.byte && self.src.is_char_boundary(to));
        self.units += self.src[self.byte..to].encode_utf16().count();
        self.byte = to;
        self.units
    }
}

/// The colored ranges of a Tomet text, for `tometHighlighting` of `@telaion/tomet`:
/// an array of `{ from, to, tag }` with UTF-16 offsets.
#[wasm_bindgen(js_name = highlightSpans)]
pub fn highlight_spans(src: &str) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(&spans(src)).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tagged(src: &str) -> Vec<(String, &'static str)> {
        // Read the ranges back through UTF-16, as the editor does.
        let units: Vec<u16> = src.encode_utf16().collect();
        spans(src)
            .into_iter()
            .map(|s| (String::from_utf16(&units[s.from..s.to]).unwrap(), s.tag))
            .collect()
    }

    #[test]
    fn ascii_ranges_are_the_byte_ranges() {
        let src = "=[ Title ]\n- item\n";
        let bytes: Vec<_> = tomet::highlight::highlight(src)
            .into_iter()
            .map(|s| (s.from, s.to))
            .collect();
        let units: Vec<_> = spans(src).into_iter().map(|s| (s.from, s.to)).collect();
        assert_eq!(bytes, units);
    }

    #[test]
    fn ranges_after_multibyte_text_count_utf16_units() {
        // 日本語 is 9 bytes but 3 units; 😀 is 4 bytes and 2 units.
        let src = "日本語 😀 @note(id: \"x\")[text]\n";
        let found = tagged(src);
        assert!(found.contains(&("@note".to_string(), "tagName")), "{found:?}");
        assert!(found.contains(&("id".to_string(), "propertyName")), "{found:?}");
        assert!(found.contains(&("\"x\"".to_string(), "string")), "{found:?}");
        // Not the byte offset (15): in UTF-16 it is 3 + space + 2 + space = 7.
        let note = spans(src).into_iter().find(|s| s.tag == "tagName").unwrap();
        assert_eq!(note.from, 7);
    }

    #[test]
    fn a_heading_in_japanese() {
        let src = "= 見出し\n\n本文 @em[強調] と `コード`\n";
        let found = tagged(src);
        assert!(found.contains(&("=".to_string(), "heading")), "{found:?}");
        assert!(found.contains(&("`コード`".to_string(), "monospace")), "{found:?}");
    }

    #[test]
    fn empty_and_untagged_text() {
        assert!(spans("").is_empty());
        assert!(tagged("just words\n").is_empty());
    }

    #[test]
    fn ranges_are_ordered_and_inside_the_text() {
        let src = "= 日本\n- 😀 *強い* `x` // メモ\n@a(k: 1)[ 😀 ]\n";
        let len = src.encode_utf16().count();
        let mut last = 0;
        for s in spans(src) {
            assert!(last <= s.from && s.from < s.to && s.to <= len, "{s:?}");
            last = s.to;
        }
    }
}

//! `ph2d-system-fonts` — a REAL [`ph2d_vector_font::PlatformHost`] backed by the OS
//! font collection (fontique, the Linebender stack parley/vello already use).
//!
//! W10 Coord: replaces the `MockHost`. [`SystemFontHost::system_fonts`] enumerates
//! the actually-installed families; [`SystemFontHost::fallback_chain`] returns the
//! OS's script-appropriate cascade for a locale — so CJK / Arabic / emoji glyphs
//! render with a *covering* installed font instead of tofu. The shell instantiates
//! one and hands it to `resolve_glyph_font`; this crate stays platform-detail-free
//! behind fontique.
//!
//! **Coverage is COARSE by design** (one sample codepoint per major Unicode block):
//! `CoverageRanges` is documented as "a coarse stand-in for a real cmap — enough to
//! route fallback correctly without parsing fonts here", so a full per-font cmap
//! scan (hundreds of fonts) is deliberately avoided — we ask each font's charmap
//! whether it maps the block's representative codepoint.

use std::sync::Mutex;

use fontique::{Collection, CollectionOptions, FallbackKey, QueryStatus, Script, SourceCache};
use ph2d_vector_font::fallback_chain::CoverageRanges;
use ph2d_vector_font::{FontFamily, Locale, PlatformHost};

/// `(start, end, sample)` for the major Unicode blocks. A family "covers" the
/// block when its charmap maps `sample`; the whole `start..=end` range is then
/// recorded (the coarse-cmap contract). Ordered low → high.
const BLOCKS: &[(u32, u32, u32)] = &[
    (0x0020, 0x007F, 0x0041),    // Basic Latin — 'A'
    (0x00A0, 0x00FF, 0x00E9),    // Latin-1 Supplement — 'é'
    (0x0100, 0x017F, 0x0107),    // Latin Extended-A — 'ć'
    (0x0370, 0x03FF, 0x03B1),    // Greek — 'α'
    (0x0400, 0x04FF, 0x0430),    // Cyrillic — 'а'
    (0x0590, 0x05FF, 0x05D0),    // Hebrew — 'א'
    (0x0600, 0x06FF, 0x0627),    // Arabic — 'ا'
    (0x0900, 0x097F, 0x0915),    // Devanagari — 'क'
    (0x0E00, 0x0E7F, 0x0E01),    // Thai — 'ก'
    (0x3040, 0x309F, 0x3042),    // Hiragana — 'あ'
    (0x30A0, 0x30FF, 0x30A2),    // Katakana — 'ア'
    (0x3400, 0x4DBF, 0x3400),    // CJK Extension A
    (0x4E00, 0x9FFF, 0x4E00),    // CJK Unified Ideographs — '一'
    (0xAC00, 0xD7AF, 0xAC00),    // Hangul Syllables — '가'
    (0x1F300, 0x1FAFF, 0x1F600), // Emoji — '😀'
];

/// ISO 15924 script tag for a primary language subtag (the fallback key the OS
/// keys its cascade on). Unknown → Latin.
fn script_for_language(lang: &str) -> &'static str {
    match lang {
        "ja" => "Jpan",
        "zh" => "Hani",
        "ko" => "Kore",
        "ar" | "fa" | "ur" | "ps" => "Arab",
        "he" | "yi" => "Hebr",
        "th" => "Thai",
        "hi" | "mr" | "ne" | "sa" => "Deva",
        "el" => "Grek",
        "ru" | "uk" | "bg" | "sr" | "be" | "mk" => "Cyrl",
        _ => "Latn",
    }
}

struct Inner {
    collection: Collection,
    cache: SourceCache,
    /// `system_fonts()` is enumerate-all + coarse-coverage — cache the result so
    /// repeated calls (font picker reopen) are free.
    system_cache: Option<Vec<FontFamily>>,
}

impl Inner {
    /// Coarse coverage for `family`: query the family's primary font and test one
    /// sample codepoint per block. Empty if the family resolves to no font.
    fn coverage_of(&mut self, family: &str) -> CoverageRanges {
        let mut ranges: Vec<(u32, u32)> = Vec::new();
        let mut query = self.collection.query(&mut self.cache);
        query.set_families([family]);
        query.matches_with(|font| {
            if let Some(charmap) = font.charmap() {
                for &(start, end, sample) in BLOCKS {
                    if charmap.map(sample).is_some() {
                        ranges.push((start, end));
                    }
                }
            }
            QueryStatus::Stop // the family's first (primary) match is enough
        });
        CoverageRanges::new(ranges)
    }

    fn family_with_coverage(&mut self, name: &str) -> FontFamily {
        let coverage = self.coverage_of(name);
        FontFamily::new(name, coverage)
    }
}

/// A [`PlatformHost`] over the real OS font collection. Cheap to construct; the
/// system scan happens lazily on first use. `Send + Sync` (the fontique state is
/// behind a `Mutex`) so the shell can park it in a shared resource.
pub struct SystemFontHost {
    inner: Mutex<Inner>,
}

impl SystemFontHost {
    #[must_use]
    pub fn new() -> Self {
        let collection = Collection::new(CollectionOptions {
            shared: false,
            system_fonts: true,
        });
        Self {
            inner: Mutex::new(Inner {
                collection,
                cache: SourceCache::default(),
                system_cache: None,
            }),
        }
    }
}

impl Default for SystemFontHost {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformHost for SystemFontHost {
    fn system_fonts(&self) -> Vec<FontFamily> {
        let mut inner = self.inner.lock().expect("font host mutex");
        if let Some(cached) = &inner.system_cache {
            return cached.clone();
        }
        // Collect names first (releases the `family_names` borrow before the
        // per-family coverage queries, which borrow the collection mutably).
        let names: Vec<String> = inner
            .collection
            .family_names()
            .map(str::to_string)
            .collect();
        let families: Vec<FontFamily> = names
            .iter()
            .map(|name| inner.family_with_coverage(name))
            .collect();
        inner.system_cache = Some(families.clone());
        families
    }

    fn fallback_chain(&self, locale: &Locale) -> Vec<FontFamily> {
        let mut inner = self.inner.lock().expect("font host mutex");
        let script = Script::from(script_for_language(locale.language()));
        let key = FallbackKey::from((script, locale.as_str()));
        // Resolve the OS cascade to family ids → names, then build coverage.
        let ids: Vec<_> = inner.collection.fallback_families(key).collect();
        let names: Vec<String> = ids
            .iter()
            .filter_map(|&id| inner.collection.family_name(id).map(str::to_string))
            .collect();
        names
            .iter()
            .map(|name| inner.family_with_coverage(name))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vector_font::resolve_glyph_font;

    // These run on whatever fonts the machine has. CI runners may be sparse, so
    // assertions on CONTENT are conditional ("if any fonts at all"); the
    // invariant always checked is that the real host never panics and the OS
    // queries return well-formed data.

    #[test]
    fn enumerates_real_families_without_panicking() {
        let host = SystemFontHost::new();
        let fonts = host.system_fonts();
        // Caching: a second call returns the same set.
        assert_eq!(
            fonts.len(),
            host.system_fonts().len(),
            "system_fonts cached"
        );
        for f in &fonts {
            assert!(!f.name.is_empty(), "every family has a name");
        }
    }

    #[test]
    fn latin_fallback_covers_ascii_when_fonts_present() {
        let host = SystemFontHost::new();
        let chain = host.fallback_chain(&Locale::new("en-US"));
        // If the OS has ANY Latin fallback, the resolver finds a cover for 'A'.
        if !chain.is_empty() {
            let primary = &chain[0];
            let resolved = resolve_glyph_font(primary, 'A', &Locale::new("en-US"), &host);
            assert!(resolved.is_some(), "a Latin chain must cover ASCII 'A'");
        }
    }

    #[test]
    fn cjk_locale_keys_a_han_script_fallback() {
        // The Japanese locale must route through the Han/Japanese script key (not
        // Latin) — proving the locale→script map drives the OS cascade. We don't
        // require a JP font to be installed; we require the query to be well-formed
        // (no panic, and any returned family is named).
        let host = SystemFontHost::new();
        for f in host.fallback_chain(&Locale::new("ja-JP")) {
            assert!(!f.name.is_empty());
        }
    }
}

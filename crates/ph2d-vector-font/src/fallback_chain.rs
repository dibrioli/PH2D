//! HR-15 locale-aware font fallback (ADR-0066 §2.5): when the primary font lacks
//! a glyph for a codepoint, walk the host's locale fallback chain (CJK /
//! Devanagari / Thai / …) to find one that covers it.
//!
//! The host abstraction ([`PlatformHost`]) keeps this crate platform-free — the
//! shell supplies system fonts + per-locale chains; this module is the pure
//! resolution logic, mockable for the gate.

/// A BCP-47-ish locale tag (e.g. `"ja-JP"`, `"zh-Hans"`, `"en"`). Only the
/// primary language subtag is interpreted here; the host owns full matching.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Locale(String);

impl Locale {
    pub fn new(tag: impl Into<String>) -> Self {
        Self(tag.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The primary language subtag (`"zh"` from `"zh-Hans-CN"`).
    pub fn language(&self) -> &str {
        self.0.split(['-', '_']).next().unwrap_or(&self.0)
    }
}

/// Inclusive Unicode codepoint ranges a family covers. A coarse stand-in for a
/// real `cmap` — enough to route fallback correctly without parsing fonts here.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CoverageRanges(Vec<(u32, u32)>);

impl CoverageRanges {
    pub fn new(ranges: impl IntoIterator<Item = (u32, u32)>) -> Self {
        Self(ranges.into_iter().collect())
    }

    /// Basic Latin + Latin-1 (the common primary-font coverage).
    pub fn latin() -> Self {
        Self::new([(0x0000, 0x024F)])
    }

    pub fn covers_cp(&self, cp: u32) -> bool {
        self.0.iter().any(|&(lo, hi)| (lo..=hi).contains(&cp))
    }
}

/// A resolvable font family: a name + its codepoint coverage.
#[derive(Clone, Debug, PartialEq)]
pub struct FontFamily {
    pub name: String,
    pub coverage: CoverageRanges,
}

impl FontFamily {
    pub fn new(name: impl Into<String>, coverage: CoverageRanges) -> Self {
        Self {
            name: name.into(),
            coverage,
        }
    }

    pub fn covers(&self, ch: char) -> bool {
        self.coverage.covers_cp(ch as u32)
    }
}

/// The platform's font services (the shell implements this; the crate stays
/// platform-free).
pub trait PlatformHost {
    /// All installed families (for enumeration / picking a primary).
    fn system_fonts(&self) -> Vec<FontFamily>;
    /// The ordered fallback families to try for `locale` (script-appropriate).
    fn fallback_chain(&self, locale: &Locale) -> Vec<FontFamily>;
}

/// Resolve which family supplies `ch` (ADR-0066 §2.5): the `primary` if it covers
/// `ch`, else the first family in the **locale-specific** fallback chain that
/// does. `None` when nothing covers it (tofu).
pub fn resolve_glyph_font(
    primary: &FontFamily,
    ch: char,
    locale: &Locale,
    host: &dyn PlatformHost,
) -> Option<FontFamily> {
    if primary.covers(ch) {
        return Some(primary.clone());
    }
    host.fallback_chain(locale)
        .into_iter()
        .find(|f| f.covers(ch))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A host whose fallback chain depends on the locale's language: `ja` adds a
    /// JP family, `zh` a SC family, everything else just Latin.
    struct MockHost;

    impl MockHost {
        fn jp() -> FontFamily {
            // Hiragana + CJK Unified Ideographs.
            FontFamily::new(
                "Noto Sans JP",
                CoverageRanges::new([(0x3040, 0x30FF), (0x4E00, 0x9FFF)]),
            )
        }
        fn sc() -> FontFamily {
            FontFamily::new("Noto Sans SC", CoverageRanges::new([(0x4E00, 0x9FFF)]))
        }
    }

    impl PlatformHost for MockHost {
        fn system_fonts(&self) -> Vec<FontFamily> {
            vec![
                FontFamily::new("Inter", CoverageRanges::latin()),
                Self::jp(),
                Self::sc(),
            ]
        }
        fn fallback_chain(&self, locale: &Locale) -> Vec<FontFamily> {
            let latin = FontFamily::new("Inter", CoverageRanges::latin());
            match locale.language() {
                "ja" => vec![latin, Self::jp()],
                "zh" => vec![latin, Self::sc()],
                _ => vec![latin],
            }
        }
    }

    fn inter() -> FontFamily {
        FontFamily::new("Inter", CoverageRanges::latin())
    }

    #[test]
    fn latin_resolves_to_primary() {
        let got = resolve_glyph_font(&inter(), 'A', &Locale::new("en"), &MockHost);
        assert_eq!(got.unwrap().name, "Inter");
    }

    #[test]
    fn cjk_routes_by_locale() {
        // Gate `variable_font_fallback_chain_locale_aware`: the SAME ideograph
        // routes to the locale-appropriate family.
        let kanji = '漢'; // U+6F22, in CJK Unified
        let ja = resolve_glyph_font(&inter(), kanji, &Locale::new("ja-JP"), &MockHost);
        assert_eq!(ja.unwrap().name, "Noto Sans JP");
        let zh = resolve_glyph_font(&inter(), kanji, &Locale::new("zh-Hans"), &MockHost);
        assert_eq!(zh.unwrap().name, "Noto Sans SC");
    }

    #[test]
    fn hiragana_only_in_jp_chain() {
        let hira = 'あ'; // U+3042, JP-only in this mock
        assert!(resolve_glyph_font(&inter(), hira, &Locale::new("ja"), &MockHost).is_some());
        // zh chain has no hiragana coverage → tofu.
        assert!(resolve_glyph_font(&inter(), hira, &Locale::new("zh"), &MockHost).is_none());
    }

    #[test]
    fn uncovered_is_none() {
        assert!(resolve_glyph_font(&inter(), '😀', &Locale::new("en"), &MockHost).is_none());
    }
}

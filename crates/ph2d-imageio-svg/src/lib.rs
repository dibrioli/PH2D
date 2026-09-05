#![forbid(unsafe_code)]
//! `ph2d-imageio-svg` — SVG decode + (deferred) encode (W3.T5).
//!
//! Pure-Rust via `usvg` 0.43 (parser + simplifier). W3.T5 ships
//! parse-only — the resulting tree validates the SVG is well-formed
//! and within size bounds, but the canonical `VectorDoc` body is
//! reserved for W3+ when `ph2d-vector` lands the `kurbo::BezPath`
//! paint stack + paint server + transform amendment.
//!
//! ### What W3.T5 covers
//!
//! - Magic: heuristic on first 256 bytes containing `<svg` or
//!   `<?xml` followed by `<svg` (handles BOM + DOCTYPE prefixes).
//! - Decode: `usvg::Tree::from_data` validates the SVG, returns
//!   `DecodedImage::Vector(VectorDoc::default())`. The VectorDoc
//!   body is intentionally empty (W3+ amendment).
//! - HR-1: pure-Rust, no C bindings.
//! - HR-13: size cap via `ph2d_imageio::MAX_ARCHIVE_TEXT_BYTES`
//!   (SVG is text/XML; 16 MiB ceiling covers any plausible icon
//!   set, blocks billion-laughs / external-entity attacks at the
//!   read boundary).
//!
//! Audit-13 (2026-05-27): fan-out drop-crate per ADR-0054 §3.8.

use ph2d_imageio::{
    DecodedImage, Error, ExportFormat, ExportOpts, ExporterRegistry, ImageExporter, ImageImporter,
    ImportOpts, ImporterRegistry, MAX_ARCHIVE_TEXT_BYTES, MagicHint, MagicMatch, VectorDoc,
};

/// Register the SVG importer.
pub fn register_importer(reg: &mut ImporterRegistry) {
    reg.register(Box::new(SvgImporter));
}

/// Register the SVG exporter.
pub fn register_exporter(reg: &mut ExporterRegistry) {
    reg.register(Box::new(SvgExporter));
}

/// SVG import driver.
pub struct SvgImporter;

/// Heuristic: does the first 256 bytes look like SVG? Handles BOM,
/// XML prolog, DOCTYPE preamble, and direct `<svg` opening.
fn is_svg_heuristic(b: &[u8]) -> bool {
    let head = &b[..b.len().min(256)];
    let head_lower: Vec<u8> = head.iter().map(|c| c.to_ascii_lowercase()).collect();
    // `<svg` anywhere in the first 256 bytes — covers `<svg xmlns=…>`,
    // `<?xml …?><!DOCTYPE …><svg …>`, BOM + XML prolog, etc.
    head_lower.windows(4).any(|w| w == b"<svg")
}

impl ImageImporter for SvgImporter {
    fn supports(&self, hint: MagicHint<'_>) -> MagicMatch {
        match hint {
            MagicHint::Bytes(b) if is_svg_heuristic(b) => MagicMatch::Strong,
            MagicHint::Bytes(_) => MagicMatch::None,
            MagicHint::Extension(ext) if ext.eq_ignore_ascii_case("svg") => MagicMatch::Weak,
            MagicHint::Extension(_) => MagicMatch::None,
        }
    }

    fn import(&self, src: &[u8], _opts: &ImportOpts) -> Result<DecodedImage, Error> {
        if src.is_empty() {
            return Err(Error::Truncated);
        }
        // HR-13: cap source size before usvg parses. SVG is text/XML
        // so the cap is large enough for any plausible icon set
        // (~16 MiB) but blocks billion-laughs / hostile expansion.
        if (src.len() as u64) > MAX_ARCHIVE_TEXT_BYTES {
            return Err(Error::Decode(format!(
                "SVG source {} bytes > MAX_ARCHIVE_TEXT_BYTES={MAX_ARCHIVE_TEXT_BYTES} \
                 (DoS defence)",
                src.len()
            )));
        }
        // Parse via usvg. Audit-13 Lens HH FIN-1 (2026-05-27): the
        // default `image_href_resolver` calls `std::fs::read(href)`
        // for `<image href="…">` references — a hostile SVG with
        // `<image href="/etc/passwd"/>` would touch the filesystem
        // (no leak to caller, but page-cache footprint + timing
        // side-channel). Override with neutral resolvers: data: URIs
        // pass through; string hrefs are NOT resolved (return None).
        let opts = usvg::Options {
            image_href_resolver: usvg::ImageHrefResolver {
                resolve_data: usvg::ImageHrefResolver::default_data_resolver(),
                resolve_string: Box::new(|_href, _opts| None),
            },
            ..Default::default()
        };
        // Default options resolve <use>, <style>, gradients into a
        // flat tree; the result is dropped here because VectorDoc
        // body is reserved for W3+ ph2d-vector amendment.
        let _tree = usvg::Tree::from_data(src, &opts)
            .map_err(|e| Error::from_decoder_message(format!("SVG parse: {e}")))?;
        // ⚠️⚠️ **A recusa passou a NOMEAR a porta certa** (2026-09-05, estudo 42 item 3). Ela dizia
        // *"lands in W3+ when ph2d-vector is callable from imageio"*, e isso deixou de ser o
        // estado do mundo: um `.svg` **entra** neste app como formas editáveis, pelo
        // `ph2d-vec-svg` (largar na janela ou *File > Import…*).
        //
        // ⛔ O que esta porta faz continua a ser outra coisa: ela pertence ao registo de IMAGENS e
        // devolve um `DecodedImage`, que é um contentor de pixels — o `VectorDoc` daqui é o
        // modelo vectorial ANTIGO (congelado, §6), e traduzir para ele para depois traduzir outra
        // vez seria uma segunda tradução a envelhecer ao lado da primeira.
        //
        // ⇒ O corpo fica vazio **de propósito**, e o `.svg` não está no
        // `SUPPORTED_IMAGE_EXTENSIONS` justamente para nenhum gesto do produto chegar aqui — se
        // chegasse, o artista receberia pixels onde pediu curvas.
        Ok(DecodedImage::Vector(VectorDoc::default()))
    }
}

/// SVG export driver. Deferred until ph2d-vector canonicalises.
pub struct SvgExporter;

impl ImageExporter for SvgExporter {
    fn supports_format(&self, fmt: ExportFormat) -> bool {
        matches!(fmt, ExportFormat::Svg)
    }

    /// ⚠️⚠️ **A recusa passou a NOMEAR a porta certa** (2026-09-02). Ela dizia *"deferred to W3+"*,
    /// e a partir de hoje isso é **falso na metade que importa**: o desenho vectorial exporta-se
    /// por **File > Export SVG…** ([`vec_svg_export`](../../../shells/desktop/src/vec_svg_export.rs)).
    ///
    /// ⛔ **O que continua a não existir é RASTERIZAR-PARA-SVG**, que é o que esta porta faz — ela
    /// recebe uma imagem já decodificada, e uma imagem não tem curva nenhuma para escrever. *Uma
    /// recusa que não diz onde está a coisa que o artista quer é indistinguível de uma ferramenta
    /// partida.*
    fn export(&self, _img: &DecodedImage, _opts: &ExportOpts) -> Result<Vec<u8>, Error> {
        Err(Error::Unsupported(
            "SVG cannot carry an IMAGE: this exporter receives decoded pixels, and pixels have \
             no curves. To export the VECTOR drawing, use File > Export SVG... instead."
                .into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_recognizes_svg_open_tag_strong() {
        assert_eq!(
            SvgImporter.supports(MagicHint::Bytes(
                b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>"
            )),
            MagicMatch::Strong
        );
    }

    #[test]
    fn supports_recognizes_svg_with_xml_prolog_strong() {
        assert_eq!(
            SvgImporter.supports(MagicHint::Bytes(
                b"<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\"/>"
            )),
            MagicMatch::Strong
        );
    }

    #[test]
    fn supports_rejects_html_with_none() {
        assert_eq!(
            SvgImporter.supports(MagicHint::Bytes(b"<!DOCTYPE html><html></html>")),
            MagicMatch::None
        );
    }

    #[test]
    fn supports_recognizes_svg_extension_weak() {
        assert_eq!(
            SvgImporter.supports(MagicHint::Extension("svg")),
            MagicMatch::Weak
        );
    }

    #[test]
    fn import_rejects_empty_as_truncated() {
        let err = SvgImporter
            .import(&[], &ImportOpts::default())
            .expect_err("empty");
        assert!(matches!(err, Error::Truncated));
    }

    #[test]
    fn import_minimal_svg_returns_vector() {
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1"/>"#;
        let decoded = SvgImporter
            .import(svg, &ImportOpts::default())
            .expect("minimal SVG must parse");
        assert!(matches!(decoded, DecodedImage::Vector(_)));
    }

    #[test]
    fn import_rejects_malformed_svg_with_decode() {
        let bad = b"<svg this is not closed";
        let err = SvgImporter
            .import(bad, &ImportOpts::default())
            .expect_err("malformed");
        assert!(matches!(err, Error::Decode(_) | Error::Truncated));
    }

    /// Audit-13 Lens HH FIN-1 (2026-05-27): hostile SVG with
    /// `<image href="/etc/passwd"/>` must NOT touch the filesystem.
    /// With the neutral `resolve_string` resolver installed, the
    /// href is dropped (resolver returns None) and the parse still
    /// succeeds — VectorDoc body is empty either way in W3.T5.
    #[test]
    fn import_with_filesystem_href_does_not_read_file() {
        // We can't directly assert "no syscall" from unit test, but
        // we can confirm the parse succeeds without error AND
        // returns Vector (proving the resolver short-circuited
        // rather than failing on missing file).
        let svg = br#"<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1">
            <image href="/etc/passwd" x="0" y="0" width="1" height="1"/>
        </svg>"#;
        let decoded = SvgImporter
            .import(svg, &ImportOpts::default())
            .expect("hostile-href SVG must parse without FS read");
        assert!(matches!(decoded, DecodedImage::Vector(_)));
    }

    /// Audit-13 Lens HH FIN-3 (2026-05-27): hostile SVG with
    /// billion-laughs-style entity expansion (DOCTYPE + recursive
    /// entities). roxmltree 0.20 caps depth ≤ 10 and references ≤
    /// 255, so the parse either rejects or returns small tree. We
    /// assert it does NOT panic / OOM / hang.
    #[test]
    fn import_billion_laughs_does_not_explode() {
        let svg = br#"<?xml version="1.0"?>
<!DOCTYPE lolz [
  <!ENTITY lol "lol">
  <!ENTITY lol1 "&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;&lol;">
  <!ENTITY lol2 "&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;&lol1;">
  <!ENTITY lol3 "&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;&lol2;">
]>
<svg xmlns="http://www.w3.org/2000/svg" width="1" height="1">
    <desc>&lol3;</desc>
</svg>"#;
        // Either Decode error (roxmltree rejects entity bomb) or Ok
        // (small expansion within caps) — both are non-failure
        // outcomes. The point is: no panic, no OOM, no hang.
        let _ = SvgImporter.import(svg, &ImportOpts::default());
    }

    /// Audit-13 Lens Z #1 / HH defence-in-depth: oversized SVG (>16
    /// MiB) must be rejected at the read boundary, before usvg parses.
    #[test]
    fn import_oversized_svg_rejected_pre_parse() {
        let mut svg = Vec::with_capacity(20 * 1024 * 1024);
        svg.extend_from_slice(br#"<svg xmlns="http://www.w3.org/2000/svg">"#);
        // Pad with whitespace to push past MAX_ARCHIVE_TEXT_BYTES.
        svg.resize(17 * 1024 * 1024, b' ');
        svg.extend_from_slice(b"</svg>");
        let err = SvgImporter
            .import(&svg, &ImportOpts::default())
            .expect_err("oversized must be rejected");
        match err {
            Error::Decode(msg) => assert!(
                msg.contains("MAX_ARCHIVE_TEXT_BYTES"),
                "expected MAX_ARCHIVE_TEXT_BYTES message: {msg}"
            ),
            other => panic!("expected Decode with cap message, got: {other:?}"),
        }
    }

    /// ⭐⭐ **OS DOIS TECTOS SÃO O MESMO NÚMERO, e esta é a única crate que vê os dois.**
    ///
    /// O `ph2d-vec-svg` (o importador vectorial a sério, estudo 42 item 3) declara o próprio
    /// `MAX_SVG_BYTES` porque não pode depender do `ph2d-imageio` — mas a LEI é uma só: quanto
    /// texto XML este app aceita antes de o parser lhe tocar.
    ///
    /// ⚠️ *Duas constantes para a mesma lei divergem no primeiro dia em que alguém mexe numa
    /// delas*, e a divergência aqui seria silenciosa: um ficheiro entraria por uma porta e seria
    /// recusado pela outra, com a mesma mensagem de defesa.
    #[test]
    fn the_two_xml_size_ceilings_are_one_law() {
        assert_eq!(
            ph2d_vec_svg::MAX_SVG_BYTES,
            MAX_ARCHIVE_TEXT_BYTES,
            "o tecto do importador vectorial e o do texto de arquivo tem de ser o MESMO numero"
        );
    }

    #[test]
    fn exporter_returns_unsupported() {
        let err = SvgExporter
            .export(
                &DecodedImage::Vector(VectorDoc::default()),
                &ExportOpts {
                    format: ExportFormat::Svg,
                    ..ExportOpts::default()
                },
            )
            .expect_err("export deferred");
        assert!(matches!(err, Error::Unsupported(_)));
    }
}

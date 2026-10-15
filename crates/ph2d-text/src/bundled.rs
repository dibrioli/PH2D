//! ⭐⭐ **As fontes EMBUTIDAS e o ESTILO do texto da interface** — o que o artista escolhe em
//! `Settings ▸ Interface font / Font weight / Font size` (2026-10-01).
//!
//! Irmão do [`super::system`] por RESPONSABILIDADE: lá mora a moldagem (o layout e a cache), aqui
//! o que entra nela — que bytes de fonte existem, com que nome são registados, e o estilo que a
//! porta única de layout aplica a todo texto.

use parley::FontContext;
use parley::fontique::{Blob, FontInfoOverride};
use std::sync::{Arc, OnceLock};

/// Inter Variable (v4.0, SIL OFL) — bundled so chrome text rasterizes
/// to the same glyphs everywhere, independent of installed system fonts.
/// Inter was designed for screen rendering without LCD subpixel AA,
/// which matches Vello's glyph pipeline (vs. system fonts like SF that
/// are tuned for CoreText's subpixel rendering and look soft here).
/// Source: <https://github.com/rsms/inter/releases/tag/v4.0> (LICENSE.txt
/// in this directory).
pub(crate) const INTER_VARIABLE_TTF: &[u8] = include_bytes!("../fonts/InterVariable.ttf");

/// Os bytes da fonte embutida (InterVariable, OFL). Exposto para quem precisa dos
/// contornos crus dos glyphs — ex. texto VETORIAL (skrifa → `VecPath`), que não passa
/// pelo pipeline parley/vello de UI.
#[must_use]
pub fn inter_variable_ttf() -> &'static [u8] {
    INTER_VARIABLE_TTF
}

/// Family name registered when bundled Inter loads successfully.
/// Falls back to `sans-serif` if registration fails (corrupted bytes,
/// future fontique breaking change, etc.) so we never panic at startup.
pub(crate) const INTER_FAMILY: &str = "InterVariable";

/// ⭐ **Noto Sans Variable (SIL OFL 1.1)** — a 2.ª fonte da interface ([`ph2d_tokens::UiFont`]).
/// Fonte: <https://github.com/google/fonts/tree/main/ofl/notosans> (licença ao lado).
const NOTO_SANS_TTF: &[u8] = include_bytes!("../fonts/NotoSansVariable.ttf");
/// ⭐ **Atkinson Hyperlegible Next (SIL OFL 1.1)** — a 3.ª, desenhada para leitura fácil.
/// Fonte: <https://github.com/google/fonts/tree/main/ofl/atkinsonhyperlegiblenext>.
const ATKINSON_TTF: &[u8] = include_bytes!("../fonts/AtkinsonHyperlegibleNextVariable.ttf");

/// ⭐ **Virgil (SIL OFL 1.1)** — a letra À MÃO do Quadro: o texto das formas e setas em RASCUNHO
/// (MiroClone W4, pedido do dono 07/10). Do artefacto instalado do Excalidraw 0.18.1, convertida de
/// woff2 para TTF (licença e origem em `fonts/Virgil-LICENSE.txt`). ⛔ A Excalifont, a letra actual
/// do Excalidraw, não: o ficheiro instalado diz «All rights reserved» e não traz licença.
const VIRGIL_TTF: &[u8] = include_bytes!("../fonts/Virgil-Regular.ttf");
/// O nome com que a Virgil é registada (forçado, como as da interface).
pub const HAND_FAMILY: &str = "PH2D Virgil";

/// ⭐ **JetBrains Mono 2.304 (SIL OFL 1.1, variável `wght`)** — a letra MONOESPAÇADA do editor de
/// código. Origem, sha256 e a decisão do nome em `fonts/JetBrainsMono-LICENSE.txt`.
const JETBRAINS_MONO_TTF: &[u8] = include_bytes!("../fonts/JetBrainsMonoVariable.ttf");
/// O nome com que a JetBrains Mono é registada — forçado, para a embutida nunca perder para uma
/// instalada. A OFL dela não declara «Reserved Font Name», então o nome próprio é conforme.
/// ⚠️ [`ph2d_tokens::FONT_MONO`] começa por ele (gate em `tests/it/the_mono_font_is_ours.rs`).
pub const MONO_FAMILY: &str = "PH2D JetBrains Mono";

/// Quantas fontes da interface há — o comprimento de [`ph2d_tokens::UiFont::ALL`], para uma
/// escolha nova não compilar sem a sua entrada em [`BUNDLED`].
pub(crate) const UI_FONTS: usize = ph2d_tokens::UiFont::ALL.len();

/// ⭐⭐⭐ **As fontes EMBUTIDAS, pela ordem de [`ph2d_tokens::UiFont::ALL`]** — os bytes e o nome
/// de família com que cada uma é REGISTADA.
///
/// ⛔⛔ **O nome é SEMPRE forçado, nos dois construtores** (2026-10-01, medido): o nome que a Inter
/// traz na tabela dela é *«Inter Variable»*, com espaço, e a pilha pedia `InterVariable` ⇒ com as
/// fontes do sistema carregadas a pilha **não a achava** e caía no `sans-serif` — nesta máquina a
/// `NotoSans-Medium.ttf` ESTÁTICA de `/usr/share/fonts` (`619 976` bytes, contra os `862 936` da
/// Inter). *O app desenhava há meses uma fonte que não era a dele*, e os testes (que correm sem as
/// fontes do sistema, onde o nome era forçado) mediam a Inter: o produto e a régua liam fontes
/// diferentes. ⚠️ E forçar um nome PRÓPRIO evita a colisão inversa: a Noto embutida e a Noto
/// instalada não podem responder pelo mesmo nome.
const BUNDLED: [(&[u8], &str); UI_FONTS] = [
    (INTER_VARIABLE_TTF, INTER_FAMILY),
    (NOTO_SANS_TTF, "PH2D Noto Sans"),
    (ATKINSON_TTF, "PH2D Atkinson Hyperlegible"),
];

thread_local! {
    /// ⭐⭐ **O estilo do texto da interface** ([`ph2d_tokens::UiTextStyle`]) — mora ao lado da
    /// estratégia de nitidez e pela mesma razão: quem MEDE (o `prefix_width`, a elisão) e quem
    /// PINTA leem o mesmo valor, senão o cursor e as reticências caem no sítio errado.
    static ACTIVE_TEXT_STYLE: std::cell::Cell<ph2d_tokens::UiTextStyle> =
        std::cell::Cell::new(ph2d_tokens::UiTextStyle::default());
}

/// Publica o estilo do texto da interface para esta thread (a shell chama-o uma vez por quadro).
pub fn set_active_text_style(style: ph2d_tokens::UiTextStyle) {
    ACTIVE_TEXT_STYLE.with(|c| c.set(style));
}

/// O estilo do texto da interface desta thread.
#[must_use]
pub fn active_text_style() -> ph2d_tokens::UiTextStyle {
    ACTIVE_TEXT_STYLE.with(std::cell::Cell::get)
}

/// ⭐⭐ **O tamanho em que um texto pedido a `nominal` px é DESENHADO** — a porta de quem conta
/// alturas de linha à mão (a caixa de várias linhas), para elas crescerem com a letra.
#[must_use]
pub fn displayed_font_px(nominal: f32) -> f32 {
    nominal * active_text_style().size.scale()
}

/// Regista as fontes EMBUTIDAS ([`BUNDLED`]) com o nome forçado e devolve a pilha de cada uma —
/// `"<nome><cauda>"`, ou `"sans-serif"` quando o registo não produziu família (bytes rejeitados):
/// o arranque nunca entra em pânico por causa de uma fonte.
pub(crate) fn register_bundled(font_context: &mut FontContext, tail: &str) -> [String; UI_FONTS] {
    // As letras do DOCUMENTO e do CÓDIGO (não são escolha da interface): só se registam, quem as
    // pede nomeia-as (a mono pela pilha [`ph2d_tokens::FONT_MONO`]).
    for (i, (bytes, family)) in [(VIRGIL_TTF, HAND_FAMILY), (JETBRAINS_MONO_TTF, MONO_FAMILY)]
        .into_iter()
        .enumerate()
    {
        let registered = font_context.collection.register_fonts(
            Blob::new(Arc::new(registada(&SEM_PREP[UI_FONTS + i], bytes))),
            Some(FontInfoOverride {
                family_name: Some(family),
                ..Default::default()
            }),
        );
        debug_assert!(
            !registered.is_empty(),
            "a fonte embutida {family} não registou"
        );
    }
    std::array::from_fn(|i| {
        let (bytes, family) = BUNDLED[i];
        let blob = Blob::new(Arc::new(registada(&SEM_PREP[i], bytes)));
        let override_info = FontInfoOverride {
            family_name: Some(family),
            ..Default::default()
        };
        let registered = font_context
            .collection
            .register_fonts(blob, Some(override_info));
        if registered.is_empty() {
            "sans-serif".to_string()
        } else {
            format!("{family}{tail}")
        }
    })
}

/// Os bytes REGISTADOS de cada fonte embutida: as da interface pela ordem de [`BUNDLED`], depois a
/// Virgil e a mono. Calculados uma vez por processo.
static SEM_PREP: [OnceLock<Option<Vec<u8>>>; UI_FONTS + 2] =
    [const { OnceLock::new() }; UI_FONTS + 2];

fn registada(slot: &'static OnceLock<Option<Vec<u8>>>, raw: &'static [u8]) -> &'static [u8] {
    slot.get_or_init(|| without_prep_table(raw))
        .as_deref()
        .unwrap_or(raw)
}

/// ⭐⭐ **A fonte sem a tabela `prep`, quando ela é INERTE** — `None` se não há `prep`, ou se a
/// fonte traz hinting de verdade (`fpgm` ou instruções em algum glifo), que fica intocado.
///
/// O Vello pede ao skrifa `Engine::AutoFallback`, que escolhe o interpretador TrueType sempre que
/// a fonte tem `fpgm`, `prep` ou instruções. A Noto, a Atkinson e a JetBrains Mono trazem um
/// `prep` de 7 bytes (`SCANCTRL`/`SCANTYPE`, só *dropout control*) e zero instruções ⇒ o
/// interpretador corre e não mexe em nada, e `hint(true)` é um no-op. Sem o `prep` cai no
/// autohinter. Medição: `docs/CodeEditor/ferramentas/medicoes/igual_perfis.txt`.
///
/// A saída é um sfnt válido: as outras tabelas byte a byte iguais (alinhadas a 4), os
/// registos com o checksum original, e só o `head.checkSumAdjustment` recalculado.
#[must_use]
pub fn without_prep_table(font: &[u8]) -> Option<Vec<u8>> {
    use read_fonts::types::{GlyphId, Tag};
    use read_fonts::{FontRef, TableProvider, tables::glyf::Glyph};
    const PREP: Tag = Tag::new(b"prep");
    const HEAD: Tag = Tag::new(b"head");
    let f = FontRef::new(font).ok()?;
    let records = f.table_directory.table_records();
    if !records.iter().any(|r| r.tag() == PREP) || f.table_data(Tag::new(b"fpgm")).is_some() {
        return None;
    }
    let (loca, glyf) = (f.loca(None).ok()?, f.glyf().ok()?);
    for gid in 0..u32::from(f.maxp().ok()?.num_glyphs()) {
        let instr = match loca.get_glyf(GlyphId::new(gid), &glyf).ok()? {
            Some(Glyph::Simple(g)) => g.instructions(),
            Some(Glyph::Composite(g)) => g.instructions().unwrap_or_default(),
            None => &[],
        };
        if !instr.is_empty() {
            return None;
        }
    }
    let kept: Vec<_> = records.iter().filter(|r| r.tag() != PREP).collect();
    let n = u16::try_from(kept.len()).ok()?;
    let sel = 15 - n.leading_zeros() as u16; // ⌊log2 n⌋
    let range = (1u16 << sel) * 16;
    let mut out = Vec::with_capacity(font.len());
    out.extend_from_slice(&f.table_directory.sfnt_version().to_be_bytes());
    for v in [n, range, sel, n * 16 - range] {
        out.extend_from_slice(&v.to_be_bytes());
    }
    let mut at = out.len() + 16 * kept.len();
    let mut head_at = None;
    for r in &kept {
        out.extend_from_slice(&r.tag().to_be_bytes());
        out.extend_from_slice(&r.checksum().to_be_bytes());
        out.extend_from_slice(&u32::try_from(at).ok()?.to_be_bytes());
        out.extend_from_slice(&r.length().to_be_bytes());
        if r.tag() == HEAD {
            head_at = Some(at);
        }
        at += (r.length() as usize).next_multiple_of(4);
    }
    for r in &kept {
        out.extend_from_slice(f.table_data(r.tag())?.as_bytes());
        out.resize(out.len().next_multiple_of(4), 0);
    }
    let adj = head_at? + 8;
    out[adj..adj + 4].fill(0);
    let soma = sfnt_checksum(&out);
    out[adj..adj + 4].copy_from_slice(&0xB1B0_AFBA_u32.wrapping_sub(soma).to_be_bytes());
    Some(out)
}

/// A soma de `u32` big-endian do sfnt (o `CalcTableChecksum` da OpenType).
#[must_use]
pub fn sfnt_checksum(b: &[u8]) -> u32 {
    b.chunks(4).fold(0u32, |s, c| {
        let mut w = [0u8; 4];
        w[..c.len()].copy_from_slice(c);
        s.wrapping_add(u32::from_be_bytes(w))
    })
}

//! **O que um ID DE PAINEL endereça na pilha de FX** — o dicionário entre a UI e o modelo.
//!
//! Irmão de [`super::fx_live`] pelo teto de LOC, e o corte é por responsabilidade: aquele arquivo é
//! *o que uma pilha É e o que a shell FAZ com ela* (o componente, a resolução em pixels, o cozimento
//! e a edição); este é *o que um id QUER DIZER*. Os ids da seção são hashes de nome derivados por
//! LINHA, então não há aritmética que os inverta — decodifica-se varrendo o teto, e é por isso que
//! a tradução merece uma casa própria em vez de ficar espalhada pelos três sítios que perguntam.

/// **Que controle da pilha um id de painel endereça.** Os ids da seção são derivados por LINHA
/// (hashes de nome), então não há aritmética que os inverta: decodifica-se varrendo o teto.
///
/// Porta única de propósito — a ponte tem TRÊS sítios que perguntam *"este id é da pilha?"* (o
/// comando, o valor e o alvo do picker), e três varreduras escritas à mão divergiriam na primeira
/// linha nova.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum FilterHit {
    /// "Add \<tipo\>" — põe um degrau novo no fim da pilha.
    Add(u8),
    /// ✕ — apaga a linha (a última apaga o componente).
    Remove(usize),
    /// ↑ / ↓ — a ORDEM é a feature.
    Up(usize),
    Down(usize),
    /// 👁 — desarma sem apagar.
    Hide(usize),
    /// A swatch de cor da linha (abre o picker OKLCH partilhado).
    Color(usize),
    /// A SEGUNDA swatch — a ponta CLARA da rampa do Duotone. Mesmo picker, outro campo.
    ColorB(usize),
    /// O chip de MODO da linha (a LEI do degrau).
    Mode(usize, u8),
    /// Uma opção do popover de MISTURA (a lei de como a cor do degrau encosta na de baixo).
    Blend(usize, u8),
    /// Os sliders.
    Radius(usize),
    OffX(usize),
    OffY(usize),
    Opacity(usize),
    /// Os três knobs do RUÍDO (só a turbulência os oferece).
    Scale(usize),
    Detail(usize),
    Seed(usize),
    /// O Amount do Grow / Shrink (bipolar).
    Grow(usize),
    /// Os três knobs do AJUSTE DE COR (bipolares). ⚠️ O `Hue` chega em GRAUS — a conversão para
    /// voltas mora no `apply`, ao lado da que o publica.
    Hue(usize),
    Sat(usize),
    Bright(usize),
}

/// Decodifica um id de painel para o controle da pilha que ele endereça.
pub(crate) fn hit_of(id: ph2d_editor::NodeId) -> Option<FilterHit> {
    use ph2d_editor::ids as vid;
    for k in 0..vid::MAX_FILTER_KINDS {
        if id == vid::filter_add_id(k) {
            #[allow(clippy::cast_possible_truncation)]
            return Some(FilterHit::Add(k as u8));
        }
    }
    for r in 0..vid::MAX_FILTER_ROWS {
        for m in 0..vid::MAX_FILTER_MODES {
            if id == vid::filter_mode_id(r, m) {
                #[allow(clippy::cast_possible_truncation)]
                return Some(FilterHit::Mode(r, m as u8));
            }
        }
        for m in 0..vid::MAX_FILTER_BLENDS {
            if id == vid::filter_blend_option_id(r, m) {
                #[allow(clippy::cast_possible_truncation)]
                return Some(FilterHit::Blend(r, m as u8));
            }
        }
        let hit = if id == vid::filter_remove_id(r) {
            FilterHit::Remove(r)
        } else if id == vid::filter_up_id(r) {
            FilterHit::Up(r)
        } else if id == vid::filter_down_id(r) {
            FilterHit::Down(r)
        } else if id == vid::filter_hide_id(r) {
            FilterHit::Hide(r)
        } else if id == vid::filter_color_id(r) {
            FilterHit::Color(r)
        } else if id == vid::filter_color_b_id(r) {
            FilterHit::ColorB(r)
        } else if id == vid::filter_radius_id(r) {
            FilterHit::Radius(r)
        } else if id == vid::filter_offx_id(r) {
            FilterHit::OffX(r)
        } else if id == vid::filter_offy_id(r) {
            FilterHit::OffY(r)
        } else if id == vid::filter_opacity_id(r) {
            FilterHit::Opacity(r)
        } else if id == vid::filter_scale_id(r) {
            FilterHit::Scale(r)
        } else if id == vid::filter_detail_id(r) {
            FilterHit::Detail(r)
        } else if id == vid::filter_seed_id(r) {
            FilterHit::Seed(r)
        } else if id == vid::filter_grow_id(r) {
            FilterHit::Grow(r)
        } else if id == vid::filter_hue_id(r) {
            FilterHit::Hue(r)
        } else if id == vid::filter_sat_id(r) {
            FilterHit::Sat(r)
        } else if id == vid::filter_bright_id(r) {
            FilterHit::Bright(r)
        } else {
            continue;
        };
        return Some(hit);
    }
    None
}

/// **A cor de um degrau em BYTES**, como a swatch do painel a mostra.
///
/// Porta única, e ela nasceu com o Duotone: a partir de duas pontas a conversão passou a ter dois
/// chamadores, e duas cópias de um arredondamento divergem exactamente onde ninguém lê um número —
/// numa swatch.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn colour_bytes(c: [f32; 4]) -> [u8; 4] {
    [
        (c[0].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        (c[1].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        (c[2].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
        (c[3].clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
    ]
}

/// **Qual PONTA de cor este id nomeia** — `(linha, é_a_segunda)`.
///
/// Porta única do readback do picker, e ela existe porque o picker é o ÚNICO consumidor que precisa
/// distinguir as duas swatches: para todo o resto (o dispatch, a varredura de seam) as duas são o
/// mesmo tipo de controle. Escrita aqui, a shell não repete a enumeração dos dois variants.
pub(crate) fn colour_target(id: ph2d_editor::NodeId) -> Option<(usize, bool)> {
    match hit_of(id)? {
        FilterHit::Color(r) => Some((r, false)),
        FilterHit::ColorB(r) => Some((r, true)),
        _ => None,
    }
}

//! ⭐⭐ **A PEGA de uma secção** — os dez pontos na ponta direita do cabeçalho, que arrastam a
//! secção para outro sítio do painel.
//!
//! Ordem do dono, 2026-09-29, com o *Properties Editor* do Blender ao lado: *«o blender tem no
//! topo de cada seção um ícone de 10 pontos que serve para arrastar e reorganizar as seções.
//! Vamos criar isso.»*
//!
//! ⭐ **Dez pontos, em duas filas de cinco** — o número que ele contou. O passo é o `Spacing::Xs`
//! da casa e o ponto metade dele (`Spacing::Xxs`): *uma pega não é um ícone, é uma TEXTURA de
//! aderência*, e o que a torna legível é o ritmo regular, não o desenho de cada ponto.
//!
//! ⚠️ **Geometria pura, lida pelo pintor e pelo hit-index** — a pega pinta-se num sítio e agarra-se
//! noutro se as duas contas viverem em dois ficheiros.

use crate::paint::fill_rounded_rect;
use crate::zones::Rect;
use ph2d_a11y::{Action, Node, NodeBuilder, Role};
use ph2d_tokens::Spacing;
use ph2d_vector::{Color as VelloColor, VectorScene};

/// Colunas da pega.
const COLUNAS: usize = 5;
/// Filas da pega — `COLUNAS × FILAS` é os dez pontos do dono.
const FILAS: usize = 2;

fn passo() -> f32 {
    Spacing::Xs.px()
}

fn ponto() -> f32 {
    Spacing::Xxs.px()
}

/// A largura da pega desenhada: `COLUNAS − 1` passos e um ponto.
#[must_use]
pub fn grip_w_px() -> f32 {
    passo() * (COLUNAS - 1) as f32 + ponto()
}

/// A altura da pega desenhada.
#[must_use]
pub fn grip_h_px() -> f32 {
    passo() * (FILAS - 1) as f32 + ponto()
}

/// Onde a pega é DESENHADA dentro do rect do cabeçalho: encostada à direita (com a folga do
/// título, `Spacing::Md`) e centrada na altura.
#[must_use]
pub fn grip_rect(header: Rect) -> Rect {
    let w = grip_w_px();
    let h = grip_h_px();
    Rect::new(
        header.x + header.w - Spacing::Md.px() - w,
        header.y + (header.h - h) * 0.5,
        w,
        h,
    )
}

/// ⭐ **Onde a pega é AGARRADA** — a banda inteira da direita do cabeçalho, da altura dele.
///
/// ⚠️ Maior que o desenho de propósito: dez pontos de `2 px` são um alvo de `18 × 6`, e um alvo
/// dessa altura é o que o artista falha. A banda vai da borda até meia folga à esquerda da pega.
#[must_use]
pub fn grip_hit_rect(header: Rect) -> Rect {
    let g = grip_rect(header);
    let x = g.x - Spacing::Xs.px();
    Rect::new(x, header.y, header.x + header.w - x, header.h)
}

/// ⭐ **A largura que a pega ocupa na ponta direita do cabeçalho** — a do alvo dela. Quem põe um
/// controlo à esquerda da pega (o botão de repor da Transform) começa aqui.
#[must_use]
pub fn grip_slot_w_px() -> f32 {
    Spacing::Md.px() + grip_w_px() + Spacing::Xs.px()
}

/// Os dez pontos, em coordenadas do ecrã.
#[must_use]
pub fn grip_dots(r: Rect) -> Vec<Rect> {
    let (p, d) = (passo(), ponto());
    let mut out = Vec::with_capacity(COLUNAS * FILAS);
    for fila in 0..FILAS {
        for col in 0..COLUNAS {
            out.push(Rect::new(r.x + col as f32 * p, r.y + fila as f32 * p, d, d));
        }
    }
    out
}

/// ⭐ **A pega para quem não a vê** — um botão com o nome da secção que ela move. O nome é o
/// título da secção (o chamador já o traduziu); a acção é o clique, que é o Down que semeia o
/// arrasto.
#[must_use]
pub fn grip_a11y(section_label: &str, hit: Rect) -> Node {
    NodeBuilder::new(Role::Button)
        .label(section_label)
        .bounds(hit.x.into(), hit.y.into(), hit.w.into(), hit.h.into())
        .focusable(true)
        .action(Action::Click)
        .build()
}

/// Pinta a pega dentro de `r` (o [`grip_rect`]).
pub fn paint_grip(scene: &mut VectorScene, r: Rect, color: VelloColor) {
    for dot in grip_dots(r) {
        fill_rounded_rect(scene, dot, dot.w * 0.5, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ⭐ **Dez pontos, dentro do desenho, sem se tocarem** — o número do dono. *Mutação: três
    /// colunas ⇒ 6; passo igual ao ponto ⇒ os pontos colam-se.*
    #[test]
    fn a_pega_tem_dez_pontos_separados() {
        let r = grip_rect(Rect::new(0.0, 0.0, 300.0, 26.0));
        let dots = grip_dots(r);
        assert_eq!(dots.len(), 10, "o dono contou DEZ pontos");
        for d in &dots {
            assert!(
                d.x >= r.x - 1e-4 && d.x + d.w <= r.x + r.w + 1e-4,
                "ponto fora do desenho"
            );
            assert!(
                d.y >= r.y - 1e-4 && d.y + d.h <= r.y + r.h + 1e-4,
                "ponto fora do desenho"
            );
        }
        assert!(
            passo() > ponto(),
            "os pontos colam-se: a pega le-se como uma barra"
        );
    }

    /// A pega anuncia-se como botão com o nome da secção.
    #[test]
    fn a_pega_e_um_botao_com_o_nome_da_seccao() {
        let n = grip_a11y("Transform", Rect::new(0.0, 0.0, 30.0, 26.0));
        assert_eq!(n.role(), Role::Button);
        assert!(n.supports_action(Action::Click));
    }

    /// ⭐ **O alvo contém o desenho e tem a altura do cabeçalho** — senão o rato acerta nos
    /// pontos e o arrasto não nasce. *Mutação: devolver o `grip_rect` ⇒ `6 px` de alto.*
    #[test]
    fn o_alvo_da_pega_contem_o_desenho_e_a_altura_do_cabecalho() {
        let head = Rect::new(10.0, 40.0, 300.0, 26.0);
        let (g, hit) = (grip_rect(head), grip_hit_rect(head));
        assert!(
            hit.x <= g.x && hit.x + hit.w >= g.x + g.w,
            "o alvo nao cobre o desenho"
        );
        assert!((hit.h - head.h).abs() < 1e-4 && (hit.y - head.y).abs() < 1e-4);
        assert!(
            hit.x + hit.w <= head.x + head.w + 1e-4,
            "o alvo sai do cabecalho"
        );
    }
}

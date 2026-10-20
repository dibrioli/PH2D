//! **O Fill no tutorial «fazer»** (W13, `docs/Macros/11_W13_o_tutorial_ensina_o_fill.md`): o ColorDrop
//! gravado (W11) é uma etapa cuja REGIÃO é o que ele pintou na reprodução, e que se cumpre com essa
//! região na cor dele ([`parecida`]). Uma pincelada por cima de um Fill mede-se contra a cor dele, não
//! contra o papel.

use super::{Pinc, TipoDoCriterio, toca};
use crate::macro_track::passo::Passo;
use crate::macro_track::tocar::{ErroDaMacro, TocadorDoPainter};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_macro::Track;

/// ⭐ **A diferença por canal (`0..255`, a métrica do próprio balde) a que uma cor ainda é a do Fill**.
/// MEDIDA (W13 doc §2) entre o que o seletor de cor do C&F deixa alcançar e o que ele deixa distinguir:
///
/// | Limite | De onde | Valor |
/// |---|---|---|
/// | o rato alcança | a faixa do matiz tem 256 px lógicos (280 − 2 × 12): 1,41°/px, até 5,98 níveis por canal num píxel ⇒ metade | ≥ 3 |
/// | a paleta distingue | as duas amostras mais próximas da paleta por omissão (`(240,96,0)` × `(220,50,50)`) | < 50 |
///
/// O meio do intervalo: 26,5 ⇒ 26 (o hex, a amostra e o conta-gotas dão a cor exacta).
pub const TOLERANCIA_DA_COR: u8 = 26;

/// O texel RGBA `px` é a cor `cor` a menos de [`TOLERANCIA_DA_COR`] por canal, opaco.
#[must_use]
pub fn parecida(px: &[u8], cor: [u8; 3]) -> bool {
    px[3] >= u8::MAX - TOLERANCIA_DA_COR && rgb_parecida([px[0], px[1], px[2]], cor)
}

/// As duas cores a menos de [`TOLERANCIA_DA_COR`] por canal.
#[must_use]
pub fn rgb_parecida(a: [u8; 3], b: [u8; 3]) -> bool {
    a.iter()
        .zip(b)
        .all(|(&x, y)| x.abs_diff(y) <= TOLERANCIA_DA_COR)
}

/// O passo pertence ao gesto do ColorDrop (o que o «Mostra-me» refaz): o ponteiro, as portas do C&F e
/// o modal do limiar.
pub(super) fn e_do_colordrop(p: &Passo) -> bool {
    use ph2d_editor_core::ids::{
        PAINTER_FILL_MODAL_CANCEL, PAINTER_FILL_MODAL_DONE, PAINTER_FILL_MODAL_SLIDER,
    };
    match p {
        Passo::Ponteiro { .. }
        | Passo::ColorDropComeca(_)
        | Passo::LimiarDoFill(_)
        | Passo::FillCommit
        | Passo::ColorDropRestaura => true,
        Passo::Painel(PanelEvent::Click(id) | PanelEvent::SetValue(id, _)) => [
            PAINTER_FILL_MODAL_SLIDER,
            PAINTER_FILL_MODAL_DONE,
            PAINTER_FILL_MODAL_CANCEL,
        ]
        .contains(id),
        _ => false,
    }
}

/// **A região de cada Fill** de `ps` na reprodução de `tocavel` (`None` nas pinceladas): os texels que
/// o gesto mudou e deixou na cor dele, opacos — a água que anda ao lado nunca dá a cor ao byte. A
/// reprodução pára no fecho do último.
pub(super) fn regioes_dos_fills(
    tocavel: &Track,
    ps: &[Pinc],
    w: u32,
    h: u32,
) -> Result<Vec<Option<Vec<u32>>>, ErroDaMacro> {
    let mut out = vec![None; ps.len()];
    let Some(ultimo) = ps
        .iter()
        .rposition(|p| p.tipo == TipoDoCriterio::Preenchimento)
    else {
        return Ok(out);
    };
    let mut toc = TocadorDoPainter::novo(
        &tocavel.initial,
        tocavel.events.iter().map(|e| e.payload.as_slice()),
    )?;
    let n = w as usize * h as usize;
    let mut antes: Vec<(u64, Vec<u8>)> = Vec::new();
    let mut j = 0;
    for (i, ev) in tocavel.events.iter().enumerate() {
        while j < ps.len() && ps[j].tipo != TipoDoCriterio::Preenchimento {
            j += 1;
        }
        if j > ultimo {
            break;
        }
        if i == ps[j].inicio {
            antes = camadas(toc.tool());
        }
        toca(&mut toc, &ev.payload)?;
        if i == ps[j].fim {
            let cor = [ps[j].cor[0], ps[j].cor[1], ps[j].cor[2], u8::MAX];
            let depois = camadas(toc.tool());
            let regiao = (0..n)
                .filter(|&x| {
                    let k = x * 4;
                    depois.iter().any(|(id, px)| {
                        px.len() >= k + 4
                            && px[k..k + 4] == cor
                            && antes
                                .iter()
                                .find(|(a, _)| a == id)
                                .is_none_or(|(_, a)| a[k..k + 4] != cor)
                    })
                })
                .map(|x| u32::try_from(x).expect("tela < 4 G texels"))
                .collect();
            out[j] = Some(regiao);
            j += 1;
        }
    }
    Ok(out)
}

fn camadas(t: &PainterTool) -> Vec<(u64, Vec<u8>)> {
    t.camadas_rgba()
        .into_iter()
        .map(|(id, px)| (id, px.to_vec()))
        .collect()
}

/// **O «Mostra-me» de um Fill** no tool vivo `t`: o gesto gravado com a cor e o limiar do toque; o C&F
/// volta ao modo do aluno, e a cor e o limiar dele voltam no fim.
pub(super) fn mostra(t: &mut PainterTool, p: Pinc) -> Result<(), ErroDaMacro> {
    let antes = (t.brush_color_srgb8(), t.fill_threshold());
    t.set_brush_color_srgb8(p.cor);
    t.set_fill_threshold(p.limiar);
    let r = p.passos.into_iter().try_for_each(|passo| {
        let passo = match passo {
            Passo::ColorDropComeca(_) => Passo::ColorDropComeca(t.active_paint_mode_id().into()),
            x => x,
        };
        TocadorDoPainter::aplica_ao_tool(t, passo).map(drop)
    });
    t.set_brush_color_srgb8(antes.0);
    t.set_fill_threshold(antes.1);
    r
}

#[cfg(test)]
#[path = "tutorial_fill_tests.rs"]
mod tests;

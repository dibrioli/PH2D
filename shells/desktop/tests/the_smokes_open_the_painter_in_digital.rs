//! **Arch-gate: os smokes do Painter abrem em DIGITAL — nenhum força um meio.**
//!
//! ## O defeito (Enio, 2026-07-22)
//!
//! *"Quando abro o painter o Wet paint ainda é o que aparece primeiro mas deveria ser o digital como o
//! padrão inicial do app."*
//!
//! O `PainterTool::default()` abre em Digital (pinado por `tool::paint::media::tests`), mas cada smoke
//! chamava `set_paint_media(<seu meio>)` no `arm_brush_once` — então rodar `PH2D_WETPAINT_SMOKE=1`
//! abria o Painter em Wet Paint, e `PH2D_IMPASTO_SMOKE=1` em Impasto. Era exatamente a cicatriz que o
//! DOC do impasto_smoke já descrevia (*"nothing here is armed in code … the smoke that arms state under
//! the table skips exactly the seam it was supposed to prove"*) — e ainda pulava o seletor de Paint
//! Mode, o controle real que o smoke deveria exercitar.
//!
//! ## Por que um gate de TEXTO
//!
//! O override do smoke é código de shell, atrás de `PH2D_*_SMOKE`. O gate de COMPORTAMENTO
//! (`the_painter_opens_on_the_plain_digital_brush`, na `ph2d-tool-painter`) prova o DEFAULT do tool,
//! mas não vê o que o smoke faz por cima dele. Este lê a fonte dos dois smokes e afirma que nenhum
//! seleciona um meio — a única razão de chamar `set_paint_media` num smoke era forçar a abertura, que é
//! o que estamos proibindo. Já reincidiu duas vezes (a cicatriz), então é gate, não confiança.

const IMPASTO: &str = include_str!("../src/impasto_smoke.rs");
const WETPAINT: &str = include_str!("../src/wetpaint_smoke.rs");

/// Nenhum smoke chama `set_paint_media(PaintMedia::<não-Digital>)`.
///
/// **Mutação que deve sangrar:** colocar de volta
/// `painter.set_paint_media(ph2d_tool_painter::PaintMedia::WetPaint)` (ou `::Impasto`) em qualquer um
/// dos dois `arm_brush_once`.
#[test]
fn no_painter_smoke_forces_a_medium() {
    for (name, src) in [("impasto", IMPASTO), ("wetpaint", WETPAINT)] {
        for medium in ["Impasto", "Watercolor", "WetPaint"] {
            // A chamada exata que abre num meio; a prosa dos docs cita os meios em texto, nunca nesta
            // forma de chamada, então isto não pega comentário.
            let call = format!("set_paint_media(ph2d_tool_painter::PaintMedia::{medium})");
            assert!(
                !src.contains(&call),
                "o smoke `{name}` chama `{call}` — ele abre o Painter em {medium} em vez do Digital que \
                 é o padrão do app. O smoke dá o canvas; o meio é escolhido no dropdown de Paint Mode."
            );
        }
    }
}

/// Controle positivo: ambos os smokes AINDA fazem o setup que os torna úteis (o canvas + a cor/tamanho),
/// para o gate acima não passar simplesmente porque alguém esvaziou os arquivos.
#[test]
fn the_smokes_still_prepare_a_canvas() {
    for (name, src) in [("impasto", IMPASTO), ("wetpaint", WETPAINT)] {
        assert!(
            src.contains("fn spawn_if_enabled")
                && src.contains("fn arm_brush_once")
                && src.contains("spawn_blank_canvas"),
            "o smoke `{name}` perdeu o preparo do canvas — este gate ficaria verde por vacuidade se o \
             arquivo fosse esvaziado; se o smoke foi renomeado/removido, atualize-o de propósito"
        );
    }
    // …e cada um ainda diz ao artista para escolher o SEU meio no dropdown.
    assert!(
        IMPASTO.contains("pick Impasto from the Paint Mode dropdown"),
        "o impasto_smoke não instrui mais a escolher Impasto no dropdown"
    );
    assert!(
        WETPAINT.contains("pick Wet Paint from the Paint Mode dropdown"),
        "o wetpaint_smoke não instrui mais a escolher Wet Paint no dropdown"
    );
}

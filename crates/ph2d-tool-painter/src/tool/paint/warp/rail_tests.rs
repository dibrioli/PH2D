//! **O chip É a ferramenta** — os gates da promoção do Liquify ao rail (2026-08-08).
//!
//! O [`super::compose_tests`] julga a LEI e o [`super::list_tests`] o ESTADO. Este julga a **PORTA**:
//! que o fio que um chip do rail publica entrega o artista *dentro* de uma metade do warp, e não numa
//! antessala; e que a viagem de volta (`active_paint_mode_id`, que o arrasto de Fill usa para capturar
//! e restaurar a ferramenta) nomeia a metade certa.
//!
//! ⚠️ **A antessala foi MEDIDA, não deduzida** (`measure_rail_chips`, um arrasto pela porta do produto
//! no meio que o Painter abre): o chip `Deform` movia **0** pixels, porque entrar no modo abria o
//! temperamento em `NONE` e o roteador de canvas consumia o evento sem agir (`_ => true`). O mesmo
//! chip movia **26 964** depois de um clique a mais no painel.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

fn striped_tool() -> PainterTool {
    let side = 128u32;
    let mut px = vec![255u8; (side * side) as usize * 4];
    for y in 0..side {
        for x in 0..side {
            if (x / 6) % 2 == 0 {
                let b = ((y * side + x) * 4) as usize;
                px[b] = 0;
                px[b + 1] = 0;
                px[b + 2] = 0;
            }
        }
    }
    let mut t = PainterTool::default();
    t.set_source(px, side, side);
    t.set_brush_size_px(40.0);
    t
}

/// **O fio do chip entrega a ferramenta, não a antessala.**
///
/// O oráculo é o que o ARTISTA vê: pegar o chip e arrastar tem de mover pixels. Um oráculo que
/// perguntasse `temperament == RESHAPE` seria um espelho da linha que acabei de escrever; este passa
/// pela porta do produto e falharia igual se o roteador de canvas voltasse a consumir sem agir.
///
/// **Mutação que tem de sangrar:** tirar o braço que arma o temperamento em `set_paint_tool_mode`. O
/// modo entra, o painel pinta, o chip acende — e o arrasto move zero.
#[test]
fn the_liquify_chip_lands_in_the_tool_not_in_a_lobby() {
    let mut t = striped_tool();
    // Exatamente o que `rail_painter_tools::push_paint_mode` põe no barramento para o chip Liquify.
    t.set_paint_tool_mode("liquify");
    let before = t.canvas_rgba.as_ref().clone();
    t.on_canvas_pointer(cp([40.0, 64.0], PointerPhase::Down));
    let mut x = 40.0f32;
    while x < 88.0 {
        x += 4.0;
        t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Up));
    let moved = before
        .iter()
        .zip(t.canvas_rgba.as_ref())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        moved > 1000,
        "o chip Liquify moveu {moved} bytes — ele caiu numa antessala, que é o defeito que a promoção \
         existe para remover"
    );
}

/// **As duas metades são duas ferramentas, e a volta nomeia a certa.**
///
/// O arrasto de Fill CAPTURA `active_paint_mode_id()` e o restaura depois; se as duas metades
/// reportassem o mesmo texto, um ColorDrop devolveria o artista à outra ferramenta em silêncio — e o
/// rail acenderia o chip errado no mesmo movimento.
///
/// **Mutação que tem de sangrar:** devolver um texto só para `PaintMode::Deform`.
#[test]
fn each_half_of_the_warp_round_trips_to_its_own_chip() {
    for wire in ["liquify", "transform"] {
        let mut t = striped_tool();
        t.set_paint_tool_mode(wire);
        assert_eq!(
            t.active_paint_mode_id(),
            wire,
            "a metade `{wire}` não sobrevive a uma captura/restauração do modo ativo"
        );
    }
}

/// **Entrar em Transform vindo de outra ferramenta levanta um gizmo NOVO** — a cerca de Chesterton
/// que a promoção tinha de preservar.
///
/// A cerca (Enio, 2026-07-04) é sobre a ENTRADA: *entrar em Deform abre o temperamento sem escolha,
/// para que escolher Transform sempre levante um gizmo fresco*. Tirar a antessala do caminho do rail
/// poderia tê-la perdido de duas maneiras — esquecer o braço que arma o temperamento, ou armá-lo
/// ANTES do reset a `NONE` que a entrada faz, caso em que o reset o apagaria e o chip cairia na
/// antessala de novo.
///
/// ⚠️ **DUAS versões deste gate estavam erradas antes desta, e as duas por ORÁCULO.** A primeira
/// afirmava mais do que o produto promete — que *re-clicar o chip que já está aceso* re-levanta —, e
/// falhou porque `set_deform_temperament` sai cedo em `t == old`: clicar duas vezes no chip do Deform
/// nunca fez nada, nem antes desta wave. A segunda esperava o gizmo fresco de volta no **centro do
/// levante original**, e falhou sobre um produto correto: sair do Transform ASSA o patch, e o levante
/// enquadra na *bbox de conteúdo opaco* (`begin_transform`), que o próprio arrasto moveu — um levante
/// fresco depois de um arrasto de +26 px senta em 77, não em 64. O oráculo honesto não é *onde* o
/// gizmo novo aparece, é que ele **não é o velho**: é exatamente isso que a cerca protege.
///
/// **Mutação que tem de sangrar:** mover o braço `"liquify" | "transform"` para ANTES da linha
/// `self.paint.paint_mode = new_mode;` — o reset a `NONE` o come, e o chip volta a ser uma antessala.
#[test]
fn entering_transform_from_another_tool_lifts_a_fresh_gizmo() {
    let mut t = striped_tool();
    t.set_paint_tool_mode("transform");
    let fresh = t
        .deform_gizmo()
        .expect(
            "o chip Transform tem de levantar o gizmo — sem isso ele é a antessala com outro nome",
        )
        .center;
    // Arrasta o patch para longe da pose de levante, para que um levante NOVO seja distinguível.
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([90.0, 64.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([90.0, 64.0], PointerPhase::Up));
    let dragged = t
        .deform_gizmo()
        .expect("o arrasto manteve o gizmo vivo")
        .center;
    assert!(
        (dragged[0] - fresh[0]).abs() > 1.0,
        "fixture: o arrasto não moveu o patch ({dragged:?} contra {fresh:?}), então um levante fresco \
         seria indistinguível de nenhum levante"
    );
    // O artista sai para outra ferramenta (o que assa o transform) e volta pelo chip.
    t.set_paint_tool_mode("brush");
    t.set_paint_tool_mode("transform");
    let relifted = t
        .deform_gizmo()
        .expect("voltar pelo chip tem de deixar um gizmo — sem ele o chip virou antessala")
        .center;
    assert!(
        (relifted[0] - dragged[0]).abs() > 1.0,
        "voltar ao Transform devolveu o gizmo ANTIGO (centro {relifted:?}, o arrastado estava em \
         {dragged:?}) — a garantia que a antessala dava se perdeu na promoção"
    );
}

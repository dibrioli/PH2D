//! Gates da **peça colada que flutua** ([`super`]).
//!
//! O invariante que os organiza: **nada vira tinta antes do Enter**. Tudo o mais — a transformação, o
//! rastro, o undo — é consequência dele.

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Falloff};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Tela branca com um quadrado PRETO de 24 px já cortado para o clipboard.
///
/// ⚠️ **24 px e não 8:** a peça tem de ser MAIOR que o dobro da tolerância de grab, senão o quadrado
/// central cobre as quinas e o gate de escala mede o caso degenerado em vez da lei.
fn armed() -> PainterTool {
    let mut t = PainterTool::default();
    let n = 48usize;
    t.set_source(vec![255u8; n * n * 4], n as u32, n as u32);
    t.paint.brush = BrushSpec {
        radius_px: 4.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.0, 0.0, 0.0],
        space_attenuation: false,
        ..Default::default()
    };
    t.set_rect_selection(8, 8, 24, 24);
    t.selection_color_fill(); // o quadrado preto
    t.selection_cut(); // levou os pixels E limpou (o clipboard esta cheio)
    t
}

fn px(t: &PainterTool, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * 48 + x) * 4) as usize;
    [
        t.canvas_rgba[i],
        t.canvas_rgba[i + 1],
        t.canvas_rgba[i + 2],
        t.canvas_rgba[i + 3],
    ]
}

fn opaque(t: &PainterTool, x: u32, y: u32) -> bool {
    px(t, x, y)[3] > 128
}

/// **Colar não commita: a peça FLUTUA.** É a mudança de comportamento inteira num gate — antes o
/// Paste compositava e gravava undo na hora.
///
/// **Mutação que sangra:** `selection_paste` voltando a compositar direto — `paste_patch_live` fica
/// falso e o `paste_cancel` não tem o que descartar.
#[test]
fn paste_arms_a_floating_patch_instead_of_committing() {
    let mut t = armed();
    t.selection_paste();
    assert!(t.paste_patch_live(), "a peca esta viva depois do Paste");
    assert!(
        t.paste_patch_gizmo().is_some(),
        "e ela publica um gizmo para o artista a transformar"
    );
}

/// **Esc devolve a tela EXATAMENTE como estava** — e sem gastar um passo de undo, porque nada foi
/// commitado. As duas metades juntas: uma peça que "cancelasse" via undo deixaria o Ctrl+Z do artista
/// apontando para o gesto errado.
///
/// **Mutação que sangra:** o `paste_cancel` não restaurar o pristino (fica o desenho da peça), ou o
/// `redraw_paste_patch` não guardar o pristino (fica o rastro).
#[test]
fn esc_gives_the_canvas_back_untouched_and_spends_no_undo() {
    let mut t = armed();
    let before: Vec<u8> = t.canvas_rgba.to_vec();
    t.selection_paste();
    assert!(opaque(&t, 20, 20), "a peca esta desenhada na tela");
    assert!(t.paste_cancel(), "havia peca para descartar");
    assert_eq!(t.canvas_rgba.to_vec(), before, "Esc devolve a tela ao byte");
    // ⚠️ O oraculo NAO e "nao ha undo": a fixture ja gastou passos (selecao, fill, cut). O que se
    // afirma e que o par paste+cancel nao acrescentou NENHUM — entao UM undo tem de voltar ao estado
    // anterior ao CUT, com o quadrado preto de volta.
    assert!(t.undo_last(), "o passo do Cut continua no topo da fila");
    assert!(
        px(&t, 20, 20)[0] < 128,
        "UM Ctrl+Z desfez o CUT, e nao um passo espurio do paste"
    );
}

/// **Arrastar a peça não deixa rastro.** A dança guarda-desenha-restaura: depois de mover, o lugar de
/// onde ela saiu tem de estar limpo.
///
/// **Mutação que sangra:** tirar o `restore_paste_pristine` do topo do `redraw_paste_patch` — a peça
/// aparece nas DUAS posições.
#[test]
fn dragging_the_patch_leaves_no_trail() {
    let mut t = armed();
    t.selection_paste();
    assert!(opaque(&t, 20, 20), "a peca comeca onde foi copiada");
    // Pega o centro e leva para (30, 30).
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([34.0, 34.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([34.0, 34.0], PointerPhase::Up));
    assert!(opaque(&t, 34, 34), "a peca esta na posicao NOVA");
    assert!(
        !opaque(&t, 20, 20),
        "e a posicao ANTIGA voltou a ser tela limpa — sem rastro"
    );
}

/// **Enter aplica, e é UM passo de undo que tira a peça inteira.** A ordem interna é o que se está
/// provando: o snapshot tem de ser tirado com o pristino de volta, senão o `before` do passo já
/// contém a peça e o Ctrl+Z não a remove.
///
/// **Mutação que sangra:** tirar o `restore_paste_pristine` de dentro do `paste_commit` — o undo
/// devolve uma tela que ainda tem a peça.
#[test]
fn enter_applies_it_as_one_undo_step_that_removes_the_whole_patch() {
    let mut t = armed();
    let before: Vec<u8> = t.canvas_rgba.to_vec();
    t.selection_paste();
    t.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([34.0, 34.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([34.0, 34.0], PointerPhase::Up));
    assert!(t.paste_commit(), "havia peca para aplicar");
    assert!(!t.paste_patch_live(), "e ela deixou de flutuar");
    assert!(opaque(&t, 34, 34), "a tinta ficou onde a peca foi largada");
    assert!(t.undo_last(), "o Enter deixou UM passo de undo");
    assert_eq!(
        t.canvas_rgba.to_vec(),
        before,
        "e UM Ctrl+Z tira a peca inteira"
    );
}

/// **A peça é reamostrada da FONTE, então girar em N passos dá o mesmo que girar de uma vez.**
///
/// ⚠️ É a lei que esta linha pagou quatro vezes no relevo: reamostrar repetidamente o RESULTADO é um
/// PRODUTO sobre a lista de gestos, e o desenho degrada com o número de passos. Aqui o que compõe é a
/// moldura; a imagem é lida uma vez, sempre do original.
///
/// **Mutação que sangra:** `transformed` computar a partir da peça VIVA em vez da `initial` do grab —
/// os dois caminhos deixam de coincidir.
#[test]
fn the_patch_is_resampled_from_the_source_so_a_drag_does_not_degrade() {
    let mut fine = armed();
    fine.selection_paste();
    fine.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    for k in 1..=20u8 {
        let s = 20.0 + 14.0 * f32::from(k) / 20.0;
        fine.on_canvas_pointer(cp([s, s], PointerPhase::Move));
    }
    fine.on_canvas_pointer(cp([34.0, 34.0], PointerPhase::Up));

    let mut coarse = armed();
    coarse.selection_paste();
    coarse.on_canvas_pointer(cp([20.0, 20.0], PointerPhase::Down));
    coarse.on_canvas_pointer(cp([34.0, 34.0], PointerPhase::Move));
    coarse.on_canvas_pointer(cp([34.0, 34.0], PointerPhase::Up));

    assert_eq!(
        fine.canvas_rgba.to_vec(),
        coarse.canvas_rgba.to_vec(),
        "vinte passos e um passo tem de pousar a MESMA peca"
    );
}

/// **Escalar por uma quina prega o lado OPOSTO** — é o que faz uma caixa ser puxada em vez de inflada
/// em torno do centro, e é o comportamento do gizmo de sprite que o artista já conhece.
///
/// **Mutação que sangra:** tirar a correção de centro do `transformed` — o lado oposto anda junto.
#[test]
fn scaling_by_a_corner_pins_the_opposite_one() {
    let mut t = armed();
    t.selection_paste();
    let before = t.paste_patch_gizmo().expect("gizmo").box_corners;
    let grab = before[2]; // a quina `++`
    let pinned = before[0]; // a `--`, que tem de ficar parada
    t.on_canvas_pointer(cp(grab, PointerPhase::Down));
    t.on_canvas_pointer(cp([grab[0] + 8.0, grab[1] + 8.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([grab[0] + 8.0, grab[1] + 8.0], PointerPhase::Up));
    let after = t.paste_patch_gizmo().expect("gizmo").box_corners;
    let moved = (after[2][0] - grab[0]).abs() + (after[2][1] - grab[1]).abs();
    let drift = (after[0][0] - pinned[0]).abs() + (after[0][1] - pinned[1]).abs();
    assert!(moved > 4.0, "a quina arrastada andou ({moved:.2} px)");
    assert!(
        drift < 0.01,
        "a quina OPOSTA tem de ficar pregada, e andou {drift:.4} px"
    );
}

/// **Colar com o clipboard vazio não arma nada** — o caso degenerado que separa *"há o que colar"* de
/// *"o Paste sempre faz alguma coisa"*.
#[test]
fn paste_with_an_empty_clipboard_arms_nothing() {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; 48 * 48 * 4], 48, 48);
    t.selection_paste();
    assert!(!t.paste_patch_live(), "sem clipboard nao ha peca");
    assert!(!t.paste_cancel(), "e nada a cancelar");
}

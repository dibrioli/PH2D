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

// ── O CORPO viaja com a cor ──────────────────────────────────────────────────────────────────────────

/// Uma tela com uma pincelada de IMPASTO real — a rota de dab do produto, o depósito do produto —, que é
/// a única fonte dos três planos (`heights`/`covers`/`mats`). Um relevo sintético não os teria todos, e
/// meio relevo é uma fixture que não contém o fenômeno.
fn impasto_canvas() -> (PainterTool, crate::tool::RtLayerId) {
    let mut t = PainterTool::default();
    let n = 96usize;
    t.set_source(vec![255u8; n * n * 4], n as u32, n as u32);
    let b = BrushSpec {
        radius_px: 8.0,
        hardness: 1.0,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.9, 0.1, 0.1],
        space_attenuation: false,
        impasto: true,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_tool_mode("brush");
    t.set_brush_impasto_depth(1.0);
    let layer = t.layers.active().expect("uma camada");
    t.on_canvas_pointer(cp([20.0, 24.0], PointerPhase::Down));
    let mut x = 24.0;
    while x <= 40.0 {
        t.on_canvas_pointer(cp([x, 24.0], PointerPhase::Move));
        x += 4.0;
    }
    t.on_canvas_pointer(cp([40.0, 24.0], PointerPhase::Up));
    (t, layer)
}

fn body_at(t: &PainterTool, layer: crate::tool::RtLayerId, x: u32, y: u32) -> (f32, u8) {
    let i = (y * 96 + x) as usize;
    (
        t.heights.get(&layer).map_or(0.0, |p| p[i]),
        t.covers.get(&layer).map_or(0, |p| p[i]),
    )
}

/// **O Copy/Paste leva o RELEVO, não só a cor** (Enio, 2026-08-07: *"O Copy/Paste não levou o relevo do
/// impasto, apenas a cor"*).
///
/// Sob impasto uma pincelada é espessura + cobertura + material tanto quanto pigmento, e um clipboard que
/// leva um quarto do fato cola uma decalcomania chapada de algo que o artista esculpiu — e a luz, que lê
/// `∇h`, mostra a diferença imediatamente.
///
/// **Mutação que sangra:** `copy_relief` devolvendo `None` (ou o composite pulando os três planos) — a
/// cor pousa no destino e o corpo lá é zero.
#[test]
fn a_pasted_piece_carries_the_body_not_only_the_colour() {
    let (mut t, layer) = impasto_canvas();
    t.selection_from_layer_contents();
    t.selection_copy();
    let (h_src, c_src) = body_at(&t, layer, 30, 24);
    assert!(
        h_src > 0.0 && c_src > 0,
        "fixture: a origem tem corpo ({h_src}, {c_src})"
    );
    assert_eq!(
        body_at(&t, layer, 30, 64).1,
        0,
        "fixture: o destino esta nu"
    );
    t.selection_paste();
    // Leva a peça 40 px para baixo e aplica.
    t.on_canvas_pointer(cp([30.0, 24.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([30.0, 64.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([30.0, 64.0], PointerPhase::Up));
    assert!(t.paste_commit(), "havia peca para aplicar");
    let (h, c) = body_at(&t, layer, 30, 64);
    assert!(
        h > 0.0 && c > 0,
        "a peca pousou CHAPADA: altura {h}, cobertura {c} — o relevo nao viajou com a cor"
    );
    assert!(
        (h - h_src).abs() < h_src * 0.25,
        "a espessura que pousou ({h}) nao se parece com a de origem ({h_src})"
    );
}

/// **Colar uma cópia opaca NO MESMO LUGAR é a identidade** — e é esta propriedade que decide a lei da
/// altura.
///
/// O depósito SOMA altura (mais tinta É mais grossa, e um traço acrescenta matéria nova). Um paste não
/// acrescenta matéria: ele **coloca uma imagem** da tinta, e é por isso que o RGBA dele compõe `over` em
/// vez de somar. Somando, este gesto — que o artista lê como *"nada aconteceu"* — dobraria a espessura.
///
/// **Mutação que sangra:** trocar o `over` da altura por `+=` — a espessura dobra.
#[test]
fn pasting_a_copy_in_place_is_the_identity() {
    let (mut t, layer) = impasto_canvas();
    t.selection_from_layer_contents();
    t.selection_copy();
    let before_h = (**t.heights.get(&layer).expect("altura")).clone();
    let before_c = (**t.covers.get(&layer).expect("cobertura")).clone();
    t.selection_paste();
    assert!(t.paste_commit(), "havia peca para aplicar");
    let now_h = t.heights.get(&layer).expect("altura");
    let now_c = t.covers.get(&layer).expect("cobertura");
    let moved = before_h
        .iter()
        .zip(now_h.iter())
        .filter(|(a, b)| (*a - *b).abs() > 1e-4)
        .count();
    assert_eq!(
        moved, 0,
        "{moved} texels mudaram de espessura ao colar uma copia sobre si mesma"
    );
    assert_eq!(&before_c, &**now_c, "e a cobertura mudou");
}

/// **Arrastar a peça não deixa RASTRO DE CORPO** — o irmão exato do
/// [`dragging_the_patch_leaves_no_trail`], no plano que a luz lê.
///
/// A dança de um quadro é *restaura → guarda → compõe*, e ela só devolve a tela ao que era se o pristino
/// guardar tudo o que o composite escreve. Guardar só o RGBA enquanto o composite passou a escrever
/// relevo deixa, no caminho por onde a peça passou, o corpo que ela depositou lá — tinta invisível com
/// espessura, que a luz desenha como um sulco fantasma.
///
/// ⚠️ **A versão anterior deste gate media a ESPESSURA NO DESTINO e sobrevivia à mutação**: a lei da
/// altura é um `over`, que é **idempotente**, então re-compor vinte vezes no mesmo lugar dá o mesmo
/// número que compor uma. O que o pristino protege não é o destino — é tudo o que a peça ATRAVESSOU.
///
/// **Mutação que sangra:** o `PastePristine.relief` sempre `None` (o pristino guardando só os pixels).
#[test]
fn dragging_the_piece_leaves_no_trail_of_body() {
    let (mut t, layer) = impasto_canvas();
    t.selection_from_layer_contents();
    t.selection_copy();
    // Um ponto que a peça vai ATRAVESSAR e onde ela não vai parar.
    let midway = (30u32, 44u32);
    assert_eq!(
        body_at(&t, layer, midway.0, midway.1),
        (0.0, 0),
        "fixture: o meio do caminho comeca nu"
    );
    t.selection_paste();
    t.on_canvas_pointer(cp([30.0, 24.0], PointerPhase::Down));
    for k in 1..=20u8 {
        t.on_canvas_pointer(cp([30.0, 24.0 + 2.0 * f32::from(k)], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([30.0, 64.0], PointerPhase::Up));
    assert!(t.paste_commit(), "havia peca para aplicar");
    assert!(
        body_at(&t, layer, 30, 64).0 > 0.0,
        "CONTROLE: a peca pousou com corpo no destino"
    );
    let (h, c) = body_at(&t, layer, midway.0, midway.1);
    assert_eq!(
        (h, c),
        (0.0, 0),
        "a peca deixou corpo ({h}, {c}) por onde apenas PASSOU — o pristino nao devolve o relevo"
    );
}

/// **Uma peça CHAPADA não toca o corpo do destino.** Ela não sabe nada sobre ele: colar tinta digital
/// sobre uma pincelada de impasto muda a cor e deixa a espessura onde está.
///
/// É o guarda de regressão do caminho digital — sem ele, a wave poderia ter zerado o relevo do destino
/// "por simetria" e ninguém veria até um smoke.
///
/// **Mutação que sangra:** o composite escrever os planos com relevo vazio (zeros) em vez de pular.
#[test]
fn a_flat_piece_leaves_the_bodys_relief_alone() {
    let (mut t, layer) = impasto_canvas();
    // ⚠️ O impasto sai ANTES do fill, e nao depois: desde a wave do Fill com corpo, um `selection_color_fill`
    // com o pincel em impasto DEPOSITA relevo — a primeira versao desta fixture copiava uma peca que tinha
    // corpo e o gate reprovava codigo correto (436 texels), que e a fixture nao contendo o que declara.
    t.paint.brush.impasto = false;
    for slot in &mut t.paint.brush_by_mode {
        slot.impasto = false;
    }
    t.set_rect_selection(10, 60, 30, 20);
    t.selection_color_fill(); // tinta chapada sobre tela nua
    t.selection_copy();
    assert!(
        t.paint
            .selection_clipboard
            .as_ref()
            .is_some_and(|c| c.relief.is_none()),
        "fixture: a peca TEM de sair sem corpo, senao este gate nao e sobre nada"
    );
    let before_h = (**t.heights.get(&layer).expect("altura")).clone();
    t.selection_paste();
    t.on_canvas_pointer(cp([25.0, 70.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([30.0, 24.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([30.0, 24.0], PointerPhase::Up));
    assert!(t.paste_commit(), "havia peca para aplicar");
    let now_h = t.heights.get(&layer).expect("altura");
    let moved = before_h
        .iter()
        .zip(now_h.iter())
        .filter(|(a, b)| (*a - *b).abs() > 1e-4)
        .count();
    assert_eq!(
        moved, 0,
        "{moved} texels de espessura mudaram sob uma peca que nao tem corpo nenhum"
    );
    // E a outra metade, que e a observavel: uma peca chapada nao DA corpo a uma camada que nao tem.
    let mut plain = PainterTool::default();
    plain.set_source(vec![255u8; 96 * 96 * 4], 96, 96);
    let plain_layer = plain.layers.active().expect("uma camada");
    plain.paint.brush = BrushSpec {
        radius_px: 6.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.0, 0.0, 0.0],
        space_attenuation: false,
        ..Default::default()
    };
    plain.set_rect_selection(20, 20, 20, 20);
    plain.selection_color_fill();
    plain.selection_copy();
    plain.selection_paste();
    assert!(plain.paste_commit(), "havia peca para aplicar");
    assert!(
        !plain.heights.contains_key(&plain_layer),
        "colar tinta chapada alocou os 12 B/px de relevo numa camada que nao tem corpo"
    );
}

/// **O Cut leva o CORPO junto com a cor** — o mesmo fato do report, visto do outro lado.
///
/// Sem isto o recorte deixa tinta **invisível com espessura**: alfa 0 e o par `(altura, cobertura)`
/// intacto, e a luz — que pesa a altura pela cobertura — desenha um sulco fantasma onde não há mais
/// pigmento nenhum.
///
/// ⚠️ **A cobertura recua e a altura fica**, e o gate afirma as duas metades: é a mesma assimetria do
/// alfa (a cor de um texto meio-cortado também fica onde está).
///
/// **Mutação que sangra:** tirar o `erase_relief` do `selection_cut` — cobertura 255 sob alfa 0.
#[test]
fn cutting_takes_the_body_with_the_colour() {
    let (mut t, layer) = impasto_canvas();
    t.selection_from_layer_contents();
    let (h0, c0) = body_at(&t, layer, 30, 24);
    assert!(
        h0 > 0.0 && c0 > 0,
        "fixture: ha corpo a cortar ({h0}, {c0})"
    );
    t.selection_cut();
    let (h1, c1) = body_at(&t, layer, 30, 24);
    let alpha = t.canvas_rgba[((24 * 96 + 30) * 4 + 3) as usize];
    assert_eq!(alpha, 0, "fixture: a COR foi cortada");
    assert_eq!(
        c1, 0,
        "o corte deixou cobertura {c1} sob alfa 0 — tinta invisivel com espessura"
    );
    assert!(
        (h1 - h0).abs() < 1e-6,
        "a altura da tinta que NAO foi cortada mudou ({h0} -> {h1})"
    );
}

//! **A máscara alcança o RELEVO** — os gates do gate de DEPÓSITO (`height::DepositGate`).
//!
//! Enio, 2026-08-07: *"Mask não consegue mascarar o relevo de impasto (apenas o pigmento)."*
//!
//! ## O mecanismo, e por que o defeito era estrutural
//!
//! O gate de proteção do pigmento (`mask::project_gate_region`) mora no **CANVAS**: ele re-deriva a
//! região do batch como `free·keep + base·(1−keep)`. O relevo não vive lá — ele tem planos próprios
//! (`heights` / `covers` / `mats`) que o depósito escreve DIRETO —, então um texel totalmente
//! protegido guardava o pigmento antigo e ganhava relevo novo: a luz desenhava uma crista onde a tinta
//! não passou.
//!
//! ## Onde a cura entra, e por que ali
//!
//! No **único lugar onde um dab escreve o envelope** (`accumulate_dab_height`). O envelope tem QUATRO
//! leitores a jusante — a luz do traço vivo (`ReliefFields::live_h`/`live_c`), a cobertura no commit, o
//! material no commit, e o re-derive do card Body — e a mordida do Push é tomada DENTRO do mesmo laço.
//! Uma regra escrita em cada leitor é uma regra que o quinto leitor nasce sem.
//!
//! ⚠️ E ela **não compõe entre dabs**, que é o que a torna aplicável no laço mais quente do app: o
//! envelope é um `max`, e `max_k(f·w_k) = f·max_k(w_k)` — um fator constante por texel atravessa o
//! máximo intacto, então nem o número de dabs nem a taxa de polling do mouse muda o resultado. É a
//! mesma lei que o §13.12 pinou para o pigmento (*uma vez por texel*), obtida aqui de graça.

use super::mask_probe::{coverage, cp, vstroke};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{
    CanvasPaintTool, PanelEvent, PointerPhase, RasterEditTool, Tool as _,
};

/// Canvas das fixtures — o irmão do `G` do `mask_gate_tests`, pelo mesmo motivo (cada gate paga a cena
/// duas vezes: a medida e o controle).
const G: u32 = 192;

/// O relevo lido por uma cena que pinta uma faixa de impasto atravessando (ou não) uma proteção.
struct ReliefRun {
    /// `heights` da camada ativa, por texel.
    height: Vec<f32>,
    /// `covers` da camada ativa — a grandeza que a LUZ pesa; relevo sobre cobertura zero não acende.
    cover: Vec<u8>,
    /// `keep = 1 − cobertura da máscara`, o número que o overlay mostra ao artista.
    keep: Vec<f32>,
}

/// Pinta uma **faixa horizontal de impasto** em `y = 96`, atravessando toda a largura, sob uma
/// proteção vertical em `x ≈ 96`.
///
/// `mask_x` põe a proteção: **96** a cruza no meio (o caso medido), **10** a põe fora do caminho da
/// medição (o CONTROLE — mesma cena, mesmo pincel, mesma história de rng, `keep == 1` onde medimos).
fn impasto_through_protection(mask_x: f32) -> ReliefRun {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (G * G * 4) as usize], G, G);
    // A proteção: um traço vertical macio de máscara (o feather é o ponto — uma borda dura não tem
    // texel de keep parcial, e é no parcial que a lei se distingue de um simples recorte).
    t.handle_panel_event(PanelEvent::SelectOption(
        ph2d_editor_core::ids::PAINTER_PAINT_MODE,
        "mask".to_string(),
    ));
    t.set_brush_size_px(24.0);
    vstroke(&mut t, mask_x, 20.0, 172.0, 24);
    let keep: Vec<f32> = coverage(&t, G).iter().map(|c| 1.0 - c).collect();
    // …e agora o impasto, pela porta do artista.
    t.set_paint_tool_mode("brush");
    t.set_brush_color_srgb8([0, 0, 0]);
    t.set_brush_size_px(14.0);
    t.paint.brush.impasto = true;
    t.on_canvas_pointer(cp([20.0, 96.0], PointerPhase::Down));
    for i in 1..=16u8 {
        let x = 20.0 + 152.0 * f32::from(i) / 16.0;
        t.on_canvas_pointer(cp([x, 96.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([172.0, 96.0], PointerPhase::Up));
    let _ = t.take_preview_arc();
    let layer = t.layers.active().expect("uma camada ativa");
    let n = (G as usize) * (G as usize);
    ReliefRun {
        height: t
            .heights
            .get(&layer)
            .map_or_else(|| vec![0.0; n], |f| f.as_ref().clone()),
        cover: t
            .covers
            .get(&layer)
            .map_or_else(|| vec![0u8; n], |c| c.as_ref().clone()),
        keep,
    }
}

/// O índice do texel `(x, y)`.
fn at(x: u32, y: u32) -> usize {
    (y * G + x) as usize
}

/// **O gate da wave:** sob proteção TOTAL o relevo não sobe — nem a altura, nem a cobertura que a luz
/// pesa.
///
/// O controle é a MESMA cena com a proteção fora do caminho: ele prova que o pincel de fato deposita
/// relevo ali, senão o gate estaria verde sobre um impasto que nunca ligou.
///
/// **Mutação que deve sangrar:** `gate: None` no `HeightFields` do `stamp_dabs_height`.
#[test]
fn a_protected_texel_gains_no_relief() {
    let run = impasto_through_protection(96.0);
    let ctl = impasto_through_protection(10.0);
    let i = at(96, 96);
    assert!(
        run.keep[i] < 0.05,
        "premissa da fixture: o texel medido tem de estar PROTEGIDO (keep {:.3})",
        run.keep[i]
    );
    assert!(
        ctl.height[i].abs() > 0.05 && ctl.cover[i] > 32,
        "controle: sem proteção o pincel DEPOSITA relevo aqui (h {:.3}, cover {}) — sem isto o gate \
         mediria um impasto desligado",
        ctl.height[i],
        ctl.cover[i]
    );
    assert!(
        run.height[i].abs() < 0.01,
        "a máscara não segurou a ALTURA (h {:.4}) — a luz desenha uma crista onde a tinta não passou",
        run.height[i]
    );
    assert!(
        run.cover[i] < 8,
        "a máscara não segurou a COBERTURA (cover {}) — ela é o peso da luz, e o material monta nela",
        run.cover[i]
    );
}

/// …e fora da proteção o MESMO traço deposita normalmente: o gate segura o que tem de segurar e não
/// mais que isso.
///
/// **Mutação que deve sangrar:** um `k` que ignore o plano e devolva 0.
#[test]
fn the_same_stroke_still_lays_relief_outside_the_protection() {
    let run = impasto_through_protection(96.0);
    let i = at(40, 96);
    assert!(
        run.keep[i] > 0.95,
        "premissa: o texel de fora está LIVRE (keep {:.3})",
        run.keep[i]
    );
    assert!(
        run.height[i].abs() > 0.05 && run.cover[i] > 32,
        "o relevo sumiu FORA da proteção (h {:.3}, cover {}) — a máscara virou um interruptor",
        run.height[i],
        run.cover[i]
    );
}

/// **A borda macia é macia no relevo também:** onde a proteção deixa passar metade, passa cerca de
/// metade — e não tudo, nem nada.
///
/// ⚠️ O oráculo é uma **razão contra o controle**, nunca um valor absoluto: a altura depende do Depth,
/// do raio e do falloff, e um literal aqui seria um gate que reprova no dia em que um default de
/// pincel mudar. A banda `0,15..0,85` é folgada de propósito — o que se afirma é *"é uma rampa"*, e a
/// mutação que a mata é qualquer lei binária (recortar em vez de escalar).
#[test]
fn the_protections_feather_reaches_the_relief_as_a_ramp() {
    let run = impasto_through_protection(96.0);
    let ctl = impasto_through_protection(10.0);
    let mut found = None;
    for x in 60..96 {
        let i = at(x, 96);
        if (0.35..=0.65).contains(&run.keep[i]) && ctl.height[i].abs() > 0.05 {
            found = Some((x, run.height[i] / ctl.height[i]));
            break;
        }
    }
    let (x, ratio) = found.expect(
        "a fixture tem de conter um texel de keep intermediário sob relevo — sem ele não há rampa a medir",
    );
    assert!(
        (0.15..=0.85).contains(&ratio),
        "em x={x} o keep é ~0,5 e o relevo saiu {ratio:.3} do livre — a proteção está recortando em vez \
         de escalar"
    );
}

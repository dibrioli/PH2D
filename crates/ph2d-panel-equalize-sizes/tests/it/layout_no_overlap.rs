//! **Duas linhas deste painel nunca se sobrepõem** — o defeito que o Enio fotografou.
//!
//! Enio, 2026-08-19: *"o painel de equalize sizes: fixed está todo embolado"*. No modo **Fixed** os
//! campos `W`/`H` caíam por cima do toggle `Upscale if smaller`, e um retângulo preto aparecia à
//! esquerda do número.
//!
//! ## O mecanismo, e por que um teste de comportamento não o via
//!
//! A linha `W`/`H` reusava o painter de **slider com chip**, com o track posto a largura zero —
//! um truque que funcionava *enquanto a row coubesse numa linha*. Nas duas metades estreitas deste
//! painel o painter **empilha** (rótulo em cima, controlo em baixo), e aí:
//!
//! 1. o track deixa de estar escondido atrás do rótulo e desenha-se à largura toda (o retângulo
//!    preto);
//! 2. a row passa a ocupar **duas** linhas, e o chamador avançava `y` por **uma** — a linha
//!    seguinte entrava por cima.
//!
//! O `seam.rs` irmão prova que escrever no chip chega aos params do tool, e continuou verde o
//! tempo todo: *ele mede o FIO, e isto é uma questão de ESPAÇO*. As duas perguntas precisam de
//! dois testes.
//!
//! ## O que este gate afirma, e o que ele NÃO consegue afirmar
//!
//! Ele afirma que **a linha condicional do modo não invade a linha seguinte** — a invariante que
//! de facto quebrou, e a que quebra outra vez quando um modo novo crescer.
//!
//! ⚠️ A primeira versão era mais ambiciosa: *nenhum par de rectângulos se sobrepõe, em modo
//! nenhum*. Ela reprovou **sobre layout correto** — um agrupador desenha-se por baixo dos próprios
//! filhos, e o `paint` devolve os dois no mesmo saco. Distinguir contentor de controlo exigiria
//! uma taxonomia que o testkit não dá, e um gate que precisa de exceções para cada agrupador
//! deixa de ser lido. *Um gate ruidoso é desligado; um gate estreito e verdadeiro sobrevive.*

use ph2d_editor_core::zones::Rect;
use ph2d_panel_equalize_sizes::{
    EqualizeSizesPanel, EqualizeSizesPanelState, ids, set_current_equalize_sizes_snapshot,
};
use ph2d_tool_equalize_sizes::params::{EqualizeSizesUiSnapshot, TargetMode};
use ph2d_ui_testkit::MockPanelHost;

/// Área da interseção de dois rectângulos — `0.0` quando não se tocam.
fn overlap_area(a: Rect, b: Rect) -> f32 {
    let w = (a.x + a.w).min(b.x + b.w) - a.x.max(b.x);
    let h = (a.y + a.h).min(b.y + b.h) - a.y.max(b.y);
    if w > 0.0 && h > 0.0 { w * h } else { 0.0 }
}

fn painted_rects(mode: TargetMode, width: f32) -> Vec<(ph2d_a11y::NodeId, Rect)> {
    let mut host = MockPanelHost::with_panel::<EqualizeSizesPanel>();
    let mut state = EqualizeSizesPanelState;
    let snapshot = EqualizeSizesUiSnapshot {
        target_mode: mode,
        ..EqualizeSizesUiSnapshot::default()
    };
    set_current_equalize_sizes_snapshot(Some(snapshot));
    host.settle_section_folds();
    host.paint::<EqualizeSizesPanel>(&mut state, Rect::new(0.0, 0.0, width, 1200.0))
}

/// ⚠️ **A largura VARRE-SE.** O defeito só aparecia abaixo do ponto em que a row empilha —
/// testar uma largura só (a confortável) é como ele passou despercebido.
#[test]
fn the_mode_row_never_invades_the_row_below_it() {
    // Os widgets que cada modo desenha na faixa condicional, e o toggle que vem logo a seguir —
    // o par que o Enio viu embolado.
    let below = ids::EQS_UPSCALE_IF_SMALLER;
    let mut failures: Vec<String> = Vec::new();
    for (mode, conditional) in [
        (TargetMode::Fixed, vec![ids::EQS_FIXED_W, ids::EQS_FIXED_H]),
        (
            TargetMode::GridUnit,
            vec![ids::EQS_GRID_OFFSET, ids::EQS_GRID_OFFSET_NUM],
        ),
        (TargetMode::MaxOfSelection, vec![]),
    ] {
        for width in [220.0_f32, 260.0, 300.0, 340.0, 420.0] {
            let rects = painted_rects(mode, width);
            let find =
                |id: ph2d_a11y::NodeId| rects.iter().find(|(rid, _)| *rid == id).map(|(_, r)| *r);
            let Some(toggle) = find(below) else {
                failures.push(format!(
                    "  {mode:?} @ {width}px: o toggle `Upscale if smaller` nem foi pintado"
                ));
                continue;
            };
            for id in &conditional {
                let Some(r) = find(*id) else {
                    failures.push(format!(
                        "  {mode:?} @ {width}px: o widget condicional {id:?} nao foi pintado"
                    ));
                    continue;
                };
                let area = overlap_area(r, toggle);
                if area > 1.0 {
                    failures.push(format!(
                        "  {mode:?} @ {width}px: {id:?} {r:?} sobrepoe o toggle {toggle:?} \
                         ({area:.0} px2)"
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "a faixa condicional do modo invade a linha seguinte — um dos dois fica por baixo do \
         outro e nao pode ser clicado:\n{}",
        failures.join("\n")
    );
}

/// **Controle positivo:** o modo Fixed pinta MESMO os dois campos, em todas as larguras varridas.
///
/// ⚠️ Sem isto, o teste acima passaria por os widgets condicionais não existirem — que é o modo de
/// falha mais fácil de introduzir num painel com faixas condicionais.
#[test]
fn fixed_mode_paints_both_fields_at_every_width() {
    for width in [220.0_f32, 260.0, 300.0, 340.0, 420.0] {
        let rects = painted_rects(TargetMode::Fixed, width);
        for id in [ids::EQS_FIXED_W, ids::EQS_FIXED_H] {
            let hit = rects.iter().find(|(rid, _)| *rid == id);
            assert!(
                hit.is_some_and(|(_, r)| r.w > 0.0 && r.h > 0.0),
                "a {width} px o campo {id:?} do modo Fixed nao foi pintado com area"
            );
        }
    }
}

//! Os gates da **TRANSAÇÃO** do `Style: Solid` — o irmão do [`super::solid_deposit_tests`], cortado
//! por responsabilidade quando o pai bateu o teto de LOC.
//!
//! ⚠️ **A linha do corte é a pergunta, não o tamanho.** O pai pergunta *o que a mancha DESENHA* (a
//! região cercada, o pincel na borda, as formas, a simetria, a costura); aqui pergunta-se *o que a
//! transação GARANTE* — que ela não apaga tinta cumulativa, que não escreve fora do retângulo que
//! salvou, e que o pool de threads é só agendamento.

use super::measure_shape_system::{cp, tool};
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::Tool;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

/// Quantos texels do canvas deixaram de ser o branco de fundo.
fn inked(t: &crate::tool::PainterTool) -> usize {
    t.canvas_rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| p[0] < 250)
        .count()
}

/// **A TEIA SOBREVIVE AO PREENCHIMENTO** — os fios do Sketchy / Wire são tinta CUMULATIVA, e a
/// transação da mancha não pode apagá-los (auditoria do Enio, 2026-08-15: *"os demais traços não
/// ficaram bons … o efeito do traço não acontece"*).
///
/// ⚠️ **O mecanismo era a ORDEM do ciclo de traço.** Num evento o produto faz `stamp_dabs` — que
/// abria a transação, **salvava** o retângulo e escrevia a mancha — e só DEPOIS `park_stroke` →
/// `stamp_threads`. Os fios caíam **fora** do instantâneo, e o `peel` do evento seguinte restaurava
/// exactamente aquele retângulo: **sobravam 11,9% da teia**. A mancha passou a fechar o evento
/// (`super::stamp_route` arma, `super::thread_deposit::park_stroke` consome).
///
/// ⚠️ **A fixture põe a `Strength` do pincel em ZERO, e é ela que torna a pergunta respondível.** Um
/// fio que cai DENTRO da região cheia é invisível por construção (mesma cor sobre mesma cor), então
/// um oráculo que conte texels sobre a mancha **não distingue apagado de invisível** — a primeira
/// versão mediu `0 de 117` e não podia dizer qual dos dois. A tinta do fio sai por um canal PRÓPRIO
/// (`thread_ink` lê `thread_width_px`/`thread_opacity`, nunca a `strength`), então com a força a zero
/// a mancha e os dabs não escrevem um byte e o que estiver na tela é a teia e só ela.
///
/// **Mutação que sangra:** devolver o `stamp_solid_preview()` ao `stamp_dabs` (261 de 2186).
#[test]
fn the_web_survives_the_fill() {
    use ph2d_painter_brush::StrokeMethod;
    use ph2d_painter_brush::line_kind::LineKind;

    let side = 256u32;
    let run = |kind: LineKind, solid: bool| -> usize {
        let mut t = tool(side, PaintMedia::Digital, 3.0);
        t.paint.brush.line_kind = kind;
        t.paint.brush.strength = 0.0; // a mancha e os dabs ficam mudos; a teia não
        t.paint.brush.sketchy_reach = 3.0;
        t.paint.brush.sketchy_density = 1.0;
        t.paint.brush.thread_width_px = 1.0;
        t.paint.brush.thread_opacity = 0.5;
        t.paint.brush.stroke_method = StrokeMethod::Space;
        if solid {
            t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
        }
        let c = 128.0f32;
        t.on_canvas_pointer(cp([c - 40.0, c], PointerPhase::Down));
        for leg in 0..6 {
            #[allow(clippy::cast_precision_loss)]
            let x = c - 40.0 + (leg as f32) * 16.0;
            let up = if leg % 2 == 0 { 1.0 } else { -1.0 };
            for k in 1..=8 {
                #[allow(clippy::cast_precision_loss)]
                let y = c + up * (k as f32) * 4.0;
                t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
            }
            t.on_canvas_pointer(cp([x + 16.0, c], PointerPhase::Move));
        }
        t.on_canvas_pointer(cp([c + 56.0, c], PointerPhase::Up));
        inked(&t)
    };

    let control = run(LineKind::None, true);
    assert_eq!(
        control, 0,
        "com Strength 0 e sem fios a tela tem de ficar limpa ({control} texels): a fixture esta \
         medindo outra coisa"
    );
    let without = run(LineKind::Sketchy, false);
    assert!(
        without > 500,
        "a fixture nao costurou teia nenhuma ({without} texels): o oraculo nao mede nada"
    );
    let with_solid = run(LineKind::Sketchy, true);
    assert!(
        with_solid * 10 >= without * 9,
        "o preenchimento APAGOU a teia: {with_solid} texels sob Solid contra {without} sem ele"
    );
}

/// **A TRANSAÇÃO NÃO ESCREVE FORA DO RETÂNGULO QUE ELA SALVOU** — senão cada evento deixa um
/// fantasma que nenhum restore volta a alcançar (auditoria do Enio, 2026-08-15).
///
/// ⚠️ **O Tiling tem régua PRÓPRIA, e era essa a discordância.** Um laço é replicado quando a CAIXA
/// dele passa a costura; um dab, quando `centro ± raio` passa. Um caminho colado à borda tem a caixa
/// DENTRO da tela e dabs de corda cuja pegada passa dela: a cópia envolvida cai na borda OPOSTA, a um
/// span inteiro do retângulo salvo. Medido, o desenho passava a depender da TAXA DE EVENTOS.
///
/// ⚠️ **O oráculo é EXATO e o controle é obrigatório:** descascar o preview no fim do gesto devolve a
/// tinta cumulativa, que é exactamente o que o MESMO gesto sem Solid pinta. Um oráculo por *"o
/// desenho muda com o número de eventos?"* seria contaminado — o próprio caminho é amostrado
/// diferente (medido: 718 texels de piso já com o Tiling desligado).
///
/// **Mutação que sangra:** devolver a folga de meia-espessura no lugar da [`tiled_chord_region`]
/// (197 fantasmas, todos na faixa envolvida).
#[test]
fn the_fill_writes_nothing_outside_the_rect_it_saved() {
    let side = 256u32;
    // Um caminho colado à borda direita: a caixa não cruza a costura, a pegada dos dabs sim.
    let path = |k: usize, n: usize| -> [f32; 2] {
        #[allow(clippy::cast_precision_loss)]
        let f = k as f32 / n as f32;
        [
            248.0 - 8.0 * (f * std::f32::consts::TAU).sin(),
            40.0 + 170.0 * f,
        ]
    };
    let bare = |solid: bool| -> Vec<u8> {
        let events = 40usize;
        let mut t = tool(side, PaintMedia::Digital, 5.0);
        if solid {
            t.handle_panel_event(PanelEvent::Click(core_ids::PAINTER_LINE_SOLID));
        }
        t.toggle_brush_tiling(0);
        t.on_canvas_pointer(cp(path(0, events), PointerPhase::Down));
        for k in 1..events {
            t.on_canvas_pointer(cp(path(k, events), PointerPhase::Move));
        }
        t.peel_drag_preview(); // fora a mancha e a corda deste evento
        t.canvas_rgba.to_vec()
    };
    let a = bare(true);
    let b = bare(false);
    let painted = b.as_chunks::<4>().0.iter().filter(|p| p[0] < 250).count();
    assert!(
        painted > 1_000,
        "a fixture nao pintou nada ({painted} texels): o oraculo nao mede nada"
    );
    let ghosts = a
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.as_chunks::<4>().0.iter())
        .filter(|(x, y)| x[0].abs_diff(y[0]) > 8)
        .count();
    assert_eq!(
        ghosts, 0,
        "a transacao escreveu {ghosts} texels fora do retangulo que salvou — eles sobrevivem a \
         todo restore e o desenho passa a depender da taxa de eventos"
    );
}

/// **AS DUAS ROTAS DO `over` DA MANCHA ESCREVEM O MESMO** — a rede que torna o pool de threads uma
/// escolha de agendamento em vez de uma segunda resposta (ADR-0109; auditoria de 2026-08-15).
///
/// ⚠️ **Ele existe porque nenhum outro gate alcança a rota paralela:** o piso do pool é ~131 k
/// texels e toda a fixture de Solid deste arquivo roda a 256² (65 k) — a rota rápida shipava
/// **sem gate nenhum**. Aqui as duas são chamadas sobre o MESMO estado e comparadas ao byte.
///
/// ⚠️ **O que ele PODE provar é o mapeamento `linha → y`**; o corpo é partilhado
/// ([`super::solid_deposit::blend_solid_rows`] escolhe o walker, nunca a aritmética), então um
/// defeito DENTRO da linha é invisível aqui e é dos gates de aparência
/// ([[feedback_an_identity_gate_cannot_see_a_defect_in_the_shared_body]]).
///
/// **Mutação que sangra:** trocar o `enumerate()` da rota paralela por um índice que não seja a
/// linha (a banda sai deslocada).
#[test]
fn both_walkers_of_the_solid_over_write_the_same_bytes() {
    use crate::tool::paint::solid_deposit::{SolidBand, blend_solid_rows};

    // Uma banda deliberadamente ESTRUTURADA: cobertura que varia por linha E por coluna, senão um
    // erro de mapeamento de linha é invisível por vácuo.
    let (rows, cols, row_bytes, x0) = (37usize, 29usize, 64usize * 4, 11usize);
    let cov: Vec<u8> = (0..rows * cols)
        .map(|i| {
            #[allow(clippy::cast_possible_truncation)]
            {
                ((i / cols) * 7 + (i % cols) * 3) as u8
            }
        })
        .collect();
    let base: Vec<u8> = (0..rows * row_bytes)
        .map(|i| {
            #[allow(clippy::cast_possible_truncation)]
            {
                (i % 251) as u8
            }
        })
        .collect();
    let band = |par: bool| -> Vec<u8> {
        let mut buf = base.clone();
        blend_solid_rows(
            &mut buf,
            SolidBand {
                cov: &cov,
                cov_stride: cols,
                row_bytes,
                x0,
                cols,
                rgb: [200, 40, 90],
                strength: 178,
            },
            par,
        );
        buf
    };
    let serial = band(false);
    let parallel = band(true);
    assert_ne!(
        serial, base,
        "a fixture nao escreveu um byte: o oraculo nao mede nada"
    );
    let diff = serial
        .iter()
        .zip(parallel.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        diff, 0,
        "as duas rotas do `over` divergem em {diff} bytes — o pool deixou de ser so' agendamento"
    );
}

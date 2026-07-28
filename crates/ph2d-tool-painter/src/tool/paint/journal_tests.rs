//! **O JOURNAL DESCREVE A TELA, E SÓ A TELA** — os gates do degrau 2 (doc 28 §5.23).
//!
//! O degrau 1 instalou o journal por tile e a porta [`super::plane_fork::fork_canvas`], e o censo
//! (`PH2D_UNDO_AUDIT=1`) acusou **12 divergências em 894 commits**. Nenhuma era um escritor que
//! esqueceu a porta: eram **três mecanismos**, e cada um destes gates pina um.
//!
//! ⚠️ **Por que gates próprios e não a rede sempre-ligada.** A rede varre o canvas INTEIRO por commit;
//! ligada por padrão ela entra no relógio de dois gates de razão que medem *janela contra canvas*
//! (`the_fold_costs_what_the_window_costs_not_what_the_canvas_costs` e o irmão do gate de proteção) e
//! os derruba — **uma rede de verificação não pode viver dentro do relógio da coisa que observa**.
//! Então a varredura fica opt-in (censo) e a PROPRIEDADE fica aqui, sem relógio nenhum.

use super::mask::mask_probe::{mask_tool, vstroke};
use super::measure_stroke_owners::{armed, cp, stroke};
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

/// O que o journal do canvas diz sobre o elemento `i`, ou `None` se o tile não foi capturado.
fn journal_at(t: &crate::tool::PainterTool, i: usize) -> Option<u8> {
    t.undo.write_state.canvas_before(i)
}

/// **A máscara pinta num plano que NÃO é a tela, e o journal não pode capturar dele.**
///
/// `stamp_dabs_mask` troca o scratch para dentro do campo `canvas_rgba` para que o pipeline de stamp
/// inteiro o edite. Enquanto a troca está de pé, um `fork_canvas` capturaria bytes do SCRATCH — e como
/// *a primeira captura de cada tile é a que vale*, a poluição seria permanente: a projeção que escreve
/// a tela logo depois encontraria o tile já tomado e o journal juraria que a tela começou o passo com
/// os bytes do scratch. Foi isto que o censo pegou em 11 dos 12 casos.
///
/// ⚠️ **Mutação que sangra:** tirar o `toggle_foreign_plane` da porta
/// [`super::plane_fork::swap_canvas_plane`] — o journal passa a descrever o scratch (preto/branco de
/// máscara) onde a tela é papel branco intocado.
#[test]
fn a_mask_stroke_never_teaches_the_journal_about_the_scratch() {
    let mut t = mask_tool(256);
    let before: Vec<u8> = (*t.canvas_rgba).clone();

    // ⚠️ **O pen-up COMMITA, e um commit ZERA o journal** (`set_cursor`) — medir depois dele é medir
    // um journal sempre vazio, e a asserção "nada divergente" seria verdadeira por construção. A
    // primeira versão deste gate fazia isso e a mutação passou por cima dela. O traço fica ABERTO.
    t.on_canvas_pointer(cp([128.0, 40.0], PointerPhase::Down));
    for k in 1..=8u8 {
        t.on_canvas_pointer(cp([128.0, 40.0 + f32::from(k) * 20.0], PointerPhase::Move));
    }

    // Controle: o scratch (branco) e a tela (vermelha) TÊM de diferir, senão capturar do plano errado
    // seria indistinguível de capturar do certo.
    assert_ne!(
        t.paint.mask_scratch_rgba[0], before[0],
        "controle: o fixture nao distingue o scratch da tela"
    );

    // Ou o tile não foi capturado (a tela não foi tocada), ou o que foi capturado É a tela.
    let mut wrong = 0usize;
    for (i, &want) in before.iter().enumerate() {
        if journal_at(&t, i).is_some_and(|got| got != want) {
            wrong += 1;
        }
    }
    assert_eq!(
        wrong, 0,
        "o journal aprendeu {wrong} byte(s) de um plano que nao e a tela (o scratch da mascara \
         estava trocado para dentro do campo `canvas_rgba` durante o stamp)"
    );
}

/// **Uma SUBSTITUIÇÃO de plano guarda o plano velho inteiro.**
///
/// `Fill`, crop, o Reset do warp e todo bind trocam o `Arc` por outro — não há escrita incremental que
/// um fork pudesse capturar, e sem esta porta o passo perderia, **em silêncio**, tudo que ainda não
/// tinha sido capturado.
///
/// ⚠️ **Mutação que sangra:** tirar o `capture_canvas` de
/// [`super::plane_fork::replace_canvas`] — o journal fica vazio e não sabe descrever o antes.
#[test]
fn replacing_the_canvas_wholesale_keeps_the_old_plane() {
    let mut t = armed(256);
    stroke(&mut t, 100.0); // dá ao histórico um cursor, e à tela algo distinguível
    let before: Vec<u8> = (*t.canvas_rgba).clone();

    let n = t.canvas_rgba.len();
    t.replace_canvas(std::sync::Arc::new(vec![7u8; n]));

    // O plano velho tem de estar todo lá — e é o VELHO, não o novo.
    for i in [0usize, 4, n / 3, n / 2, n - 8, n - 1] {
        assert_eq!(
            journal_at(&t, i),
            Some(before[i]),
            "elemento {i}: a substituicao nao guardou o plano velho"
        );
    }
    assert_ne!(
        before[0], 7,
        "controle: o fixture tem de distinguir o plano velho do novo"
    );
}

/// **Uma reinstalação de modelo (um undo) esquece o passo.**
///
/// `restore_model` troca TODO plano por outro, então os bytes que o journal guardava descrevem um
/// passado que não existe mais — e pior: as substituições passam pela porta, logo elas CAPTURAM o
/// estado de antes do undo, que é exatamente o que o undo está desfazendo. Foram as 3 últimas
/// divergências do censo.
///
/// ⚠️ **Mutação que sangra:** tirar o `reset_journal` do fim de `restore_model`.
#[test]
fn reinstalling_a_model_forgets_what_the_step_had_captured() {
    let mut t = armed(256);
    stroke(&mut t, 100.0);
    let mid = t.snapshot_model();
    stroke(&mut t, 160.0);

    t.restore_model(mid);

    assert_eq!(
        journal_at(&t, 0),
        None,
        "o journal sobreviveu a uma reinstalacao de modelo — ele descreve planos que ja nao existem"
    );
}

/// **A troca de plano é PAREADA, e um traço comum nunca a deixa de pé.**
///
/// O contador é a única coisa entre o journal e o plano errado; se um sítio trocasse e não voltasse, a
/// captura ficaria desligada para sempre e o censo ficaria verde por não capturar NADA — o modo de
/// falha silencioso deste desenho.
#[test]
fn the_plane_swap_is_balanced() {
    let mut m = mask_tool(256);
    vstroke(&mut m, 128.0, 40.0, 200.0, 8);
    assert!(
        !m.undo.write_state.on_foreign_plane(),
        "a mascara deixou uma troca de plano aberta"
    );

    let mut t = armed(256);
    t.on_canvas_pointer(cp([60.0, 60.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([120.0, 60.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([180.0, 60.0], PointerPhase::Up));
    assert!(
        !t.undo.write_state.on_foreign_plane(),
        "um traco comum deixou uma troca de plano aberta"
    );
}

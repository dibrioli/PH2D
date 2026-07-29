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

/// **O journal é ancorado no PASSO, não no último commit** — e é isso que o torna utilizável como lado
/// `before` (doc 28 §5.26).
///
/// Entre dois passos pode haver uma escrita de canvas **sem entrada de undo**: é o que a sim do Wet
/// Paint faz a cada tick depois do pen-up, e é literalmente o que um *escorrido* é. Ancorado no último
/// **commit**, o journal captura os bytes de antes da gota; o passo seguinte então encontraria, nos
/// tiles que os dois tocam, um `before` do passo **anterior** — e o undo devolveria uma tela que nunca
/// existiu. Ancorado no **passo** (`begin_undo_step`), ele descreve o que aquele passo de fato encontrou.
///
/// ⚠️ **Mutação que sangra:** tirar o `self.begin_undo_step()` do `paint_begin` — o journal segue com a
/// captura de antes da gota e o gate acusa o byte pré-gota onde deveria estar o pós.
#[test]
fn a_foreign_write_between_two_steps_does_not_leak_into_the_second_ones_before() {
    let mut t = armed(256);
    stroke(&mut t, 100.0); // passo 1, commitado — o commit zera o journal

    // **A gota**: escrita de canvas pela porta, sem entrada de undo nenhuma.
    let (w, _h) = t.source_size;
    let stride = w as usize * 4;
    let probes: Vec<usize> = (96usize..112)
        .map(|x| 200 * stride + x * 4) // pixels (96..112, 200) — dentro do traço 2 E da gota
        .collect();
    let pre: Vec<u8> = probes.iter().map(|&i| t.canvas_rgba[i]).collect();
    {
        let buf = super::plane_fork::fork_canvas(&mut t.canvas_rgba, &t.undo.write_state, w, None);
        for &i in &probes {
            buf[i] = 33; // a tinta que a sim composita depois do pen-up
        }
    }
    t.mark_dirty(crate::compositor::Region {
        x: 96,
        y: 200,
        w: 16,
        h: 1,
    });
    assert!(
        pre.iter().all(|&b| b != 33),
        "controle: a gota tem de mudar os bytes, senao pre e pos sao indistinguiveis"
    );

    // **Passo 2**, deixado ABERTO — o pen-up commitaria e um commit zera o journal, e aí a asserção
    // seria verdadeira por vacuidade (a armadilha que o gate da máscara já pagou).
    t.on_canvas_pointer(cp([60.0, 200.0], PointerPhase::Down));
    for k in 1..=6u8 {
        t.on_canvas_pointer(cp([60.0 + f32::from(k) * 30.0, 200.0], PointerPhase::Move));
    }

    let mut known = 0usize;
    for &i in &probes {
        if let Some(got) = journal_at(&t, i) {
            known += 1;
            assert_eq!(
                got, 33,
                "elemento {i}: o journal do passo 2 devolveu o byte de ANTES da gota — ele esta \
                 ancorado no ultimo commit, nao neste passo"
            );
        }
    }
    assert!(
        known > 0,
        "controle: o traco 2 tem de tocar os tiles da gota, senao o gate nao pergunta nada"
    );
}

/// **O CURSOR é reconstruível de `vivo + journal`** — o fato que autoriza o último degrau do S3.
///
/// O `cursor` do histórico é um dono **permanente** do canvas, e `make_mut` copia com qualquer coisa
/// acima de um: é ele, junto com o `stroke_undo`, que faz a primeira escrita de todo gesto pagar uma
/// cópia do documento. Ele existe por dois motivos, e os dois se dissolveram: ser a **base do delta**
/// (o 3a mediu o vivo como sendo o cursor em 92 de 92 undos) e ser o **alvo da absorção** — que é o que
/// este gate fecha.
///
/// O cursor é o estado do último commit; o journal é zerado *naquele mesmo commit* e guarda os bytes
/// velhos de toda escrita desde então. Logo `cursor[i] == journal.get(i).unwrap_or(vivo[i])` — e no caso
/// comum (journal vazio) a reconstrução é o próprio `Arc` vivo, de graça.
///
/// ⚠️ **A fixture TEM de conter uma escrita estrangeira**, senão os dois lados são o mesmo buffer e o
/// gate afirma `x == x`. A gota é escrita pela porta, sem entrada de undo — o que a sim do Wet Paint faz
/// a cada tick depois do pen-up.
///
/// ⚠️ **Mutação que sangra:** tirar o `capture_canvas` de [`super::plane_fork::fork_canvas`] — o journal
/// deixa de conhecer a gota e a reconstrução devolve o byte do VIVO onde o cursor tem o velho.
#[test]
fn the_cursor_is_reconstructible_from_the_live_plane_and_the_journal() {
    let mut t = armed(256);
    stroke(&mut t, 100.0); // um commit: há cursor, e ele zerou o journal

    let (w, _h) = t.source_size;
    let stride = w as usize * 4;
    let probes: Vec<usize> = (96usize..112).map(|x| 200 * stride + x * 4).collect();
    {
        let buf = super::plane_fork::fork_canvas(&mut t.canvas_rgba, &t.undo.write_state, w, None);
        for &i in &probes {
            buf[i] = 33;
        }
    }

    let cursor = t
        .undo
        .cursor_for_audit()
        .expect("um traco commitado deixa cursor")
        .canvas_rgba
        .clone();
    assert_ne!(
        *cursor, *t.canvas_rgba,
        "controle: sem escrita estrangeira o gate afirmaria x == x"
    );

    // A reconstrução, elemento a elemento — dentro da gota E fora dela.
    let mut differ = 0usize;
    for (i, &want) in cursor.iter().enumerate() {
        let got = t
            .undo
            .write_state
            .canvas_before(i)
            .unwrap_or(t.canvas_rgba[i]);
        differ += usize::from(got != want);
    }
    assert_eq!(
        differ, 0,
        "o cursor NAO e reconstruivel de vivo+journal — enquanto isso nao valer ele tem de SEGURAR o \
         canvas, e segura-lo e o que faz a 1a escrita de todo gesto copiar o documento"
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

/// **Todo texel que o traço mudou está no journal** — o gate que o pré-requisito (c) existe para ter.
///
/// ⚠️ **Com TILING ligado, e é essa a premissa sob teste.** O Tiling replica um dab que cruza a borda
/// numa cópia deslocada para a borda OPOSTA, e a região que [`super::region::dabs_bounds`] soma só a cobre porque a
/// replicação acontece **na lista** (`tiling::tiled_dabs_grouped`, em `stamp_route`), antes de a rota de
/// depósito ser chamada. Se algum dia alguém mover o wrap para *dentro* do blit, esta função passa a
/// devolver um subconjunto — e é este gate que falha, em vez de o undo passar a esquecer a borda oposta
/// em silêncio.
///
/// ⚠️ **O traço fica ABERTO.** O pen-up commita, e um commit **zera o journal** (`set_cursor`) — medir
/// depois dele mede um journal vazio, e "nenhum texel divergente" seria verdade por construção. Foi
/// exatamente esse o defeito da primeira versão da sonda de memória (doc 28 §7).
#[test]
fn every_texel_the_stroke_changed_is_described_by_the_journal() {
    let side = 256u32;
    let mut t = armed(side);
    t.paint.tiling = [true, true]; // wrap nos dois eixos: as cópias vão para as bordas opostas
    let before: Vec<u8> = (*t.canvas_rgba).clone();

    // Um traço COLADO na borda esquerda, para o Tiling de fato produzir cópias do outro lado.
    //
    // ⚠️ **UM salto longo, não seis passos curtos** — e a diferença decide o gate. Cada evento de
    // ponteiro é um BATCH, e cada batch faz o seu próprio fork; com passos de 18 px os dabs de um
    // batch caem todos no mesmo tile, e uma região que cobrisse **só o primeiro dab** ainda os
    // conteria. A mutação sobreviveu exatamente assim. Um salto de 200 px emite a fila inteira de
    // dabs num batch só, e aí a região tem de somá-los.
    t.on_canvas_pointer(cp([4.0, 30.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([4.0, 230.0], PointerPhase::Move));

    let after: &[u8] = &t.canvas_rgba;
    let mut changed = 0usize;
    let mut undescribed = 0usize;
    let mut wrapped_changed = 0usize;
    for (i, (&a, &b)) in before.iter().zip(after.iter()).enumerate() {
        if a == b {
            continue;
        }
        changed += 1;
        // O texel mudou ⇒ o journal TEM de saber o valor velho dele.
        match t.undo.write_state.canvas_before(i) {
            Some(got) if got == a => {}
            _ => undescribed += 1,
        }
        // A metade direita da tela só pode ter mudado pelas cópias do Tiling.
        let x = (i / 4) % (side as usize);
        if x > (side as usize) * 3 / 4 {
            wrapped_changed += 1;
        }
    }

    // Controles: sem eles o gate passa sobre um traço que não pintou nada, ou sobre um Tiling inerte.
    assert!(
        changed > 500,
        "controle: o traco mal pintou ({changed} texels)"
    );
    assert!(
        wrapped_changed > 0,
        "controle: o Tiling nao produziu copia na borda oposta — a premissa nao esta sob teste"
    );
    assert_eq!(
        undescribed, 0,
        "{undescribed} de {changed} texels mudaram e o journal nao guardou o valor velho deles"
    );
}

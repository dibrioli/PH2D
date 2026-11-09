//! ⭐ **As folhas no editor de animações** (plano GEF 26 §3.4, V4; maquete 26, painéis 4 e 6): o dropdown da folha,
//! a mistura na mesma animação, a tira com a folha de cada quadro, o vazio destacado e a limpeza ao pé da lista.

use super::{Mesa, centre};
use ph2d_ecs::{Entity, ExtraSheet, ShownSheet, SpriteSheets};
use ph2d_editor_core::{AnimFrameRef, ImageUseEdit, row_cells};

/// A Mesa do herói (folha `0`, 8 × 4) com uma folha `1` de 2 × 1 («run», 64 × 32 px na base de assets).
fn duas_folhas() -> Mesa {
    let mut m = Mesa::new();
    junta_folha(&mut m);
    m
}

/// Junta a folha `1` («run», 2 × 1) ao herói.
fn junta_folha(m: &mut Mesa) {
    let run = m.db.insert_image_rgba8(64, 32, vec![255; 64 * 32 * 4]);
    let extra = ExtraSheet {
        asset: run,
        size: [1.0, 1.0],
        anchor: [0.5, 0.5],
        hframes: 2,
        vframes: 1,
        offsets: Default::default(),
    };
    let e = Entity::from_bits(m.e);
    m.sim
        .world_mut()
        .entity_mut(e)
        .insert(SpriteSheets(std::iter::once(extra).collect()));
    m.frame();
}

/// O dropdown → a folha `k`, e um quadro para a pintura ler.
fn mostra(m: &mut Mesa, k: usize) {
    let picker = m.doc().sheet_ui.picker.expect("o ▾ com duas folhas");
    m.click(centre(picker));
    m.paint();
    let item = m.doc().sheet_ui.items[k];
    m.click(centre(item));
    m.frame();
}

/// Os quadros da animação aberta como o snapshot os tem.
fn abertos(m: &mut Mesa) -> Vec<AnimFrameRef> {
    let d = m.doc();
    let info = d.snap.info.as_ref().expect("animações");
    row_cells(&info.rows[d.selected], &info.sheets)
}

/// Com uma folha, o título é o de sempre (sem ▾); com duas, o ▾ escolhe a folha, que é a MESMA do cartão Image
/// (`ShownSheet`), e a grelha e a imagem passam a ser as dela.
/// *Mutação: o ▾ com uma folha só (`sheets.len() >= 1`) ⇒ `picker` existe na Mesa de uma folha.*
#[test]
fn o_editor_mostra_a_folha_escolhida() {
    let mut uma = Mesa::new();
    assert_eq!(uma.doc().sheet_ui.picker, None, "uma folha: sem dropdown");
    let mut m = duas_folhas();
    assert_eq!(m.doc().snap.shown_sheet, 0);
    mostra(&mut m, 1);
    let e = Entity::from_bits(m.e);
    assert_eq!(m.sim.world().get::<ShownSheet>(e), Some(&ShownSheet(1)));
    let d = m.doc();
    assert_eq!(d.snap.shown_sheet, 1);
    assert_eq!(
        (d.snap.hframes, d.snap.vframes),
        (2, 1),
        "a grelha da folha 1"
    );
    assert_eq!(d.snap.sheet.as_ref().map(|s| (s.w, s.h)), Some((64, 32)));
}

/// Trocar de folha e continuar a clicar junta quadros da folha nova à MESMA animação aberta.
/// *Mutação: o clique ignora a folha mostrada (`frame_of` com `sheet: 0`) ⇒ o quadro novo é `(0, 1)`.*
#[test]
fn clicar_noutra_folha_mistura_a_mesma_animacao() {
    let mut m = duas_folhas();
    let antes = abertos(&mut m);
    mostra(&mut m, 1);
    let c = m.cell(1);
    m.click(c);
    m.frame();
    let depois = abertos(&mut m);
    assert_eq!(depois.len(), antes.len() + 1);
    assert_eq!(
        depois[..antes.len()],
        antes[..],
        "os quadros de antes ficam"
    );
    assert_eq!(depois.last(), Some(&AnimFrameRef { sheet: 1, cell: 1 }));
}

/// A tira pinta cada quadro com a imagem da SUA folha e, como a animação mistura, a marca da folha em cada um.
/// *Mutação: a tira pinta todos com uma folha só (`let k = 0` no `paint_cell`) ⇒ o último diz `Some(0)`.*
#[test]
fn a_tira_pinta_cada_quadro_com_a_sua_folha() {
    let mut m = duas_folhas();
    mostra(&mut m, 1);
    let c = m.cell(0);
    m.click(c);
    m.frame();
    let pintado = m.doc().sheet_ui.painted.borrow().clone();
    let n = abertos(&mut m).len();
    assert_eq!(pintado.len(), n);
    assert_eq!(pintado[0].sheet, Some(0));
    assert_eq!(
        pintado[n - 1].sheet,
        Some(1),
        "o quadro da folha 1 com a imagem dela"
    );
    assert!(
        pintado.iter().all(|p| p.mark.is_some()),
        "mistura: a marca em cada um"
    );
    assert_ne!(pintado[0].mark, pintado[n - 1].mark);
}

/// Apagar a folha deixa o quadro dela VAZIO no lugar: a tira tracejado de aviso, a linha da lista em alerta.
/// *Mutação: o vazio não se destaca (`empty: false`) ⇒ nenhum quadro pintado como vazio.*
#[test]
fn o_vazio_fica_destacado_na_tira() {
    let mut m = duas_folhas();
    mostra(&mut m, 1);
    let c = m.cell(1);
    m.click(c);
    m.frame();
    m.doc().sheet_ui.pending.push(ImageUseEdit::DeleteSheet);
    m.frame();
    let fim = abertos(&mut m);
    assert!(
        fim.last().is_some_and(|f| f.is_empty()),
        "o buraco no lugar"
    );
    let pintado = m.doc().sheet_ui.painted.borrow().clone();
    let ultimo = pintado.last().expect("quadros");
    assert!(ultimo.empty && ultimo.sheet.is_none());
    assert_eq!(pintado.iter().filter(|p| p.empty).count(), 1);
    let sel = m.doc().selected;
    assert_eq!(*m.doc().sheet_ui.alerts.borrow(), vec![sel]);
}

/// Ao pé da lista: **Remove empty frames** com algum vazio; **Delete empty animations** só com uma animação TODA
/// vazia. Cada botão empurra a sua edição e o mundo fica limpo.
/// *Mutação: o 2.º botão sempre (`take(2)`) ⇒ `clean[1]` existe sem nenhuma animação toda vazia.*
#[test]
fn os_botoes_de_limpeza_do_editor() {
    let mut m = duas_folhas();
    assert_eq!(
        m.doc().sheet_ui.clean,
        [None, None],
        "sem vazios, sem botões"
    );
    mostra(&mut m, 1);
    let c = m.cell(1);
    m.click(c);
    m.frame();
    m.doc().sheet_ui.pending.push(ImageUseEdit::DeleteSheet);
    m.frame();
    let [remove, delete] = m.doc().sheet_ui.clean;
    assert!(remove.is_some());
    assert_eq!(delete, None, "nenhuma animação toda vazia");
    // Uma animação toda da folha 1: o «+» e um clique nela, depois apagar a folha.
    junta_folha(&mut m);
    mostra(&mut m, 1);
    let add = m.doc().layout.add;
    m.click(centre(add));
    m.frame();
    let c = m.cell(0);
    m.click(c);
    m.frame();
    m.doc().sheet_ui.pending.push(ImageUseEdit::DeleteSheet);
    m.frame();
    let rows = |m: &mut Mesa| m.doc().snap.info.as_ref().map_or(0, |i| i.rows.len());
    let n = rows(&mut m);
    let delete = m.doc().sheet_ui.clean[1].expect("uma toda vazia: o 2.º botão");
    m.click(centre(delete));
    m.frame();
    assert_eq!(
        rows(&mut m),
        n - 1,
        "Delete empty animations apagou a toda vazia"
    );
    let remove = m.doc().sheet_ui.clean[0].expect("ainda há vazios");
    m.click(centre(remove));
    m.frame();
    assert_eq!(
        m.doc().sheet_ui.clean,
        [None, None],
        "sem vazios, sem botões"
    );
    assert!(abertos(&mut m).iter().all(|f| !f.is_empty()));
}

/// ⭐ Smoke do dono 09/11 (Two Sheets): com a folha «run» à vista, o «+» cria a animação NA run e sem quadros; a
/// primeira célula clicada é da run. *Mutação: o «+» na folha 0 ⇒ `sheet == 0`.*
#[test]
fn o_mais_cria_vazia_na_folha_a_vista() {
    let mut m = duas_folhas();
    mostra(&mut m, 1);
    let add = centre(m.doc().layout.add);
    m.click(add);
    m.frame();
    let row = {
        let d = m.doc();
        d.snap.info.as_ref().expect("animações").rows[d.selected].clone()
    };
    assert_eq!((row.sheet, row.frames), (1, Some(Vec::new())));
    assert!(abertos(&mut m).is_empty());
    let c = m.cell(1);
    m.click(c);
    m.frame();
    assert_eq!(abertos(&mut m), [AnimFrameRef { sheet: 1, cell: 1 }]);
}

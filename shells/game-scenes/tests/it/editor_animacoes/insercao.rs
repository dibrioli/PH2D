//! **A marca de inserção da tira** (dono 08/11): as células da folha entram ONDE a marca está, e ela anda para depois
//! delas. Filhos de [`super`] (a `Mesa`).

use super::*;

/// ⭐ A marca nasce no fim; arrastada para entre o 2.º e o 3.º quadro, o clique numa célula entra ALI, e o seguinte
/// entra logo depois. *Mutações: `spliced` que junta no fim ⇒ `[0..5, 20]`; não avançar a marca ⇒ o 21 entra antes
/// do 20; a marca que não segue o arrasto ⇒ fica no fim.*
#[test]
fn as_celulas_entram_onde_esta_a_marca_e_ela_anda_para_depois_delas() {
    let mut m = Mesa::new();
    m.paint();
    let l = m.doc().layout.clone();
    assert_eq!(m.doc().playhead_gap(), 6, "nasce no fim da idle (0..=5)");
    assert!(l.playhead.x > l.frames[5].0.x, "pintada depois do último");
    let entre = (l.frames[1].0.x + l.frames[1].0.w, centre(l.frames[1].0).1);
    m.pointer(PointerKind::Down, centre(l.playhead));
    m.drag_to(entre);
    m.pointer(PointerKind::Up, entre);
    assert_eq!(m.doc().insert_at, Some(2));
    let at = m.cell(20);
    m.click(at);
    assert_eq!(
        m.pending(),
        [AnimFieldEdit::Frames(0, fr(&[0, 1, 20, 2, 3, 4, 5]))]
    );
    m.frame();
    m.paint();
    let at = m.cell(21);
    m.click(at);
    assert_eq!(
        m.pending(),
        [AnimFieldEdit::Frames(0, fr(&[0, 1, 20, 21, 2, 3, 4, 5]))],
        "o seguinte entra depois do anterior"
    );
}

/// Clicar num vão da tira põe a marca ali; apagar um quadro antes dela recua-a, e trocar de animação volta ao fim.
/// *Mutações: sem `playhead_follow_delete` ⇒ fica 3; sem o reset em `pick_row` ⇒ fica 2.*
#[test]
fn clicar_num_vao_poe_a_marca_e_ela_segue_o_apagar() {
    let mut m = Mesa::new();
    m.paint();
    let l = m.doc().layout.clone();
    let vao = (
        l.frames[2].0.x + l.frames[2].0.w + 1.0,
        centre(l.frames[2].0).1,
    ); // LITERAL-PX-OK: dentro do vão
    m.click(vao);
    assert_eq!(m.doc().insert_at, Some(3));
    m.paint();
    let x = m.doc().layout.frames[0].1;
    m.click(centre(x));
    assert_eq!(m.doc().insert_at, Some(2), "o quadro 1 saiu antes dela");
    m.frame();
    m.paint();
    let row = centre(m.doc().layout.list_rows[1]);
    m.click(row);
    assert_eq!(m.doc().insert_at, None, "outra animação: no fim");
}

/// O ◆ pertence ao desenho: células que entram antes dele empurram-no. *Mutação: sem `markers_follow_insert` ⇒ o ◆
/// fica no 3 e passa a marcar a célula nova.*
#[test]
fn o_marcador_depois_da_marca_anda_com_o_seu_quadro() {
    let mut m = Mesa::new();
    let r = m.doc().layout.frames[3].0;
    m.click(centre(r));
    m.frame();
    let r = m.doc().layout.mark_toggle;
    m.click(centre(r));
    m.key(Key::Enter, None);
    m.frame();
    m.paint();
    let l = m.doc().layout.clone();
    m.click((
        l.frames[0].0.x + l.frames[0].0.w + 1.0,
        centre(l.frames[0].0).1,
    )); // LITERAL-PX-OK: dentro do vão
    let at = m.cell(20);
    m.click(at);
    m.frame();
    let marcas = m
        .sim
        .world()
        .get::<ph2d_ecs::SpriteAnimMarkers>(Entity::from_bits(m.e))
        .map(|k| k.0.iter().map(|x| x.step).collect::<Vec<_>>());
    assert_eq!(marcas, Some(vec![4]));
}

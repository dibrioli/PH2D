//! Gates do arrasto de nota — o gesto inteiro sobre um hit-index montado.

use super::*;
use crate::ids;

const P: NodeId = ids::INSP_PANEL;
const A: NodeId = ids::INSP_LIVE_TRANSFORM_SECTION;
const B: NodeId = ids::INSP_LIVE_RENDER_SECTION;

/// Um painel com dois cabeçalhos (`A` em 100, `B` em 300) e duas notas em `A` (em 150 e 200).
fn cena() -> (WidgetStore, HitIndex) {
    let mut s = WidgetStore::with_capacity(8);
    s.set_panel_rect(P, Rect::new(0.0, 0.0, 300.0, 600.0));
    s.notes_push(P, 0, Some(A));
    s.notes_push(P, 1, Some(A));
    let mut hit = HitIndex::default();
    hit.register(A, Rect::new(0.0, 100.0, 300.0, 20.0));
    hit.register(B, Rect::new(0.0, 300.0, 300.0, 20.0));
    hit.register(ids::NOTE_SLOT_IDS[0], Rect::new(0.0, 150.0, 300.0, 40.0));
    hit.register(ids::NOTE_SLOT_IDS[1], Rect::new(0.0, 200.0, 300.0, 40.0));
    (s, hit)
}

/// ⭐⭐ **Arrastar a pega da 1.ª nota para dentro de `B` muda-a de secção; um clique parado não
/// move nada.** *Mutação: não exigir `active` ⇒ o clique parado move-a.*
#[test]
fn arrastar_a_pega_da_nota_para_outra_seccao_move_a() {
    let (mut s, hit) = cena();
    seed(&mut s, ids::NOTE_GRIP_IDS[0], 280.0, 160.0);
    drop(&mut s, &hit, 160.0);
    assert_eq!(
        s.notes_for_panel(P)[0].section,
        Some(A),
        "o clique parado moveu"
    );
    seed(&mut s, ids::NOTE_GRIP_IDS[0], 280.0, 160.0);
    s.update_note_drag(280.0, 350.0);
    drop(&mut s, &hit, 350.0);
    let notas = s.notes_for_panel(P);
    assert_eq!(notas[1].section, Some(B));
    assert_eq!(notas[0].section, Some(A));
}

/// ⭐ A posição entre as notas da secção sai do meio delas: largar acima do meio da 1.ª põe a
/// arrastada à frente. *Mutação: contar pelo topo ⇒ a posição muda.*
#[test]
fn a_posicao_na_seccao_sai_do_meio_das_notas() {
    let (s, hit) = cena();
    let drag = NoteDrag {
        panel: P,
        index: 1,
        down_x: 0.0,
        down_y: 220.0,
        cursor_x: 0.0,
        cursor_y: 160.0,
        active: true,
    };
    assert_eq!(lugar_da_queda(&s, &hit, &drag, 160.0), Some((Some(A), 0)));
    assert_eq!(lugar_da_queda(&s, &hit, &drag, 175.0), Some((Some(A), 1)));
}

/// A secção sob um `y` — acima de todas, a primeira; num painel sem secções, nenhuma.
#[test]
fn a_seccao_sob_um_y() {
    let heads = [(A, 100.0), (B, 300.0)];
    assert_eq!(seccao_sob(&heads, 50.0), Some(A));
    assert_eq!(seccao_sob(&heads, 299.0), Some(A));
    assert_eq!(seccao_sob(&heads, 301.0), Some(B));
    assert_eq!(seccao_sob(&[], 10.0), None);
}

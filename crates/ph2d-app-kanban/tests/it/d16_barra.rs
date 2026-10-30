//! D16 — a barra da lista como a do Trello (referência do dono `smoke/D16`): margem própria, cartões
//! mais estreitos na lista que rola, setas que rolam um passo.

use crate::{Mesa, centro};
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_editor_core::zones::Rect;
use ph2d_host::Modifiers;
use ph2d_kanban_model::{KanbanId, ListId};

/// A «To Do» com oito cartões novos (e o compositor fechado): rola.
fn longa() -> (Mesa, KanbanId, ListId) {
    let (mut m, id) = Mesa::cena1();
    let todo = m.lista(id, "To Do");
    let (add, _, _) = m.docs().probe_list_buttons(id.0, todo).expect("botões");
    m.clica(centro(add));
    for i in 0..8 {
        m.escreve(&format!("Shot {i}"));
        m.enter();
    }
    m.tecla(BoardKey::Escape, Modifiers::default(), None);
    m.quadros(30);
    (m, id, todo)
}

fn rolagem(m: &mut Mesa, id: KanbanId, l: ListId) -> f64 {
    m.docs()
        .probe_view(id.0)
        .unwrap()
        .scroll_y
        .get(&l)
        .copied()
        .unwrap_or(0.0)
}

/// A lista que rola dá a margem direita à barra: os seus cartões estreitam (15 px, 256 → 241 como no
/// Trello) e nenhum toca a calha; a lista que não rola fica com os cartões largos e sem barra.
#[test]
fn a_lista_que_rola_estreita_os_cartoes_e_a_barra_nao_lhes_toca() {
    let (mut m, id, todo) = longa();
    let doing = m.lista(id, "Doing");
    assert!(
        m.docs().probe_bar(id.0, Some(doing)).is_none(),
        "Doing não rola"
    );
    let largo = m.rect_cartao(id, m.cartoes(id, doing)[0]);
    let (track, _) = m
        .docs()
        .probe_bar(id.0, Some(todo))
        .expect("a barra da To Do");
    let lista = m.rect_lista(id, todo);
    for c in m.cartoes(id, todo) {
        let r = m.rect_cartao(id, c);
        assert_eq!(largo.w - r.w, 15.0, "o cartão estreita: {r:?} × {largo:?}");
        assert!(
            r.x + r.w + 7.0 <= track.x,
            "o cartão não toca a barra: {r:?} {track:?}"
        );
    }
    assert!(
        track.x + track.w + 7.0 <= lista.x + lista.w,
        "a barra dentro da lista"
    );
    assert_eq!(track.w, 9.0, "a espessura da captura");
}

/// As setas: a de cima rola um passo para trás, a de baixo volta; carregar nelas não agarra o
/// polegar (arrastar a seguir não rola).
#[test]
fn as_setas_da_barra_rolam_um_passo_e_nao_agarram() {
    let (mut m, id, todo) = longa();
    let s0 = rolagem(&mut m, id, todo);
    assert!(s0 > 40.0, "a lista desceu: {s0}");
    let (track, _) = m.docs().probe_bar(id.0, Some(todo)).unwrap();
    let cima = centro(Rect::new(track.x, track.y + 1.0, track.w, 2.0));
    let baixo = centro(Rect::new(track.x, track.y + track.h - 3.0, track.w, 2.0));
    m.clica(cima);
    m.quadros(2);
    let s = rolagem(&mut m, id, todo);
    assert!((s0 - 40.0 - s).abs() < 1e-3, "um passo acima: {s0} → {s}");
    m.carrega(baixo.0, baixo.1);
    m.move_(baixo.0, track.y + track.h * 0.2);
    m.larga(baixo.0, track.y + track.h * 0.2);
    m.quadros(2);
    let s = rolagem(&mut m, id, todo);
    assert!(
        (s - s0).abs() < 1e-3,
        "a seta de baixo volta e não agarra: {s}"
    );
}

/// 📷 A lista longa com a barra nova (a margem, as setas), sem e com o rato no polegar, em
/// `docs/TrelloClone/smoke/D16/`. `#[ignore]`: a placa.
#[test]
#[ignore = "a placa: foto para docs/TrelloClone/smoke/D16/"]
fn d16_foto_barra_da_lista() {
    let (mut m, id, todo) = longa();
    m.ts = ph2d_text::TextSystem::new();
    m.quadros(10);
    crate::j2_fotos::grava_em("D16", "lista_longa_barra", &m.cena());
    let (_, thumb) = m.docs().probe_bar(id.0, Some(todo)).unwrap();
    m.move_(thumb.x + thumb.w / 2.0, thumb.y + thumb.h / 2.0);
    m.quadros(4);
    crate::j2_fotos::grava_em("D16", "lista_longa_barra_sob_o_rato", &m.cena());
}

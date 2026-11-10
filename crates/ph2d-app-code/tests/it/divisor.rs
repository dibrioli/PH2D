//! ⭐ O divisor da coluna da esquerda (dono 10/11): o vão entre a coluna Scripts + «In this script» e a
//! do documento arrasta-se de lado e muda a largura das duas, pelo rato real da aba.

use crate::{Mesa, centro};
use ph2d_editor_core::doc_surface::{DocumentSurface, Key, PointerLook};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{Modifiers, PointerButton};
use ph2d_panel_code::bancada::Aim;
use ph2d_panel_code::{ScriptListView, cards};
use ph2d_script::ScriptId;
use ph2d_tokens::PANEL_MIN_W_PX;

const TEXTO: &str = "ph2d.property(\"speed\", 220, { min = 0, max = 1000 })\n\
                     function on_update(self, dt)\n\
                     \x20 print(self.speed)\n\
                     end\n";

fn aberta() -> (Mesa, ScriptId) {
    let (m, ids) = Mesa::new(&[("a.luau", TEXTO)]);
    let id = ids[0];
    let mut m = m.com_cena(id, 1);
    m.pede(id);
    m.quadro();
    m.pinta(id);
    m.pinta(id);
    (m, id)
}

/// A coluna da esquerda, o cartão do código e a entrada `on_update` do «In this script».
fn rects(m: &mut Mesa, id: ScriptId) -> (Rect, Rect, Rect) {
    let [col, code, _] = m.docs().card_rects(id).expect("cartões");
    let entry = m.docs().bench.target(Aim::Entry(1)).expect("a entrada");
    (col, code, entry)
}

/// O meio do vão entre a coluna da esquerda e o documento.
fn vao(col: Rect) -> (f32, f32) {
    (col.x + col.w + cards::margin() * 0.5, col.y + col.h * 0.5)
}

fn input(ts: &mut ph2d_text::TextSystem) -> ph2d_editor_core::doc_surface::PointerInput<'_> {
    ph2d_editor_core::doc_surface::PointerInput {
        text: ts,
        mods: Modifiers::default(),
        now_ns: 1,
    }
}

fn arrasta(m: &mut Mesa, id: ScriptId, de: (f32, f32), dx: f32) {
    let ate = (de.0 + dx, de.1);
    m.arrasta_com(id, de, ate, Modifiers::default(), PointerButton::Primary);
    m.pinta(id);
}

#[test]
fn o_vao_mostra_o_cursor_de_redimensionar_e_os_cartoes_nao() {
    let (mut m, id) = aberta();
    let (col, code, entry) = rects(&mut m, id);
    let (x, y) = vao(col);
    let look = |m: &mut Mesa, (x, y): (f32, f32)| m.docs().pointer_look(id.0, x, y);
    assert_eq!(look(&mut m, (x, y)), Some(PointerLook::ResizeEw));
    for (nome, p) in [("lista", centro(col)), ("código", centro(code))] {
        assert_ne!(look(&mut m, p), Some(PointerLook::ResizeEw), "{nome}");
    }
    assert_ne!(look(&mut m, centro(entry)), Some(PointerLook::ResizeEw));
}

#[test]
fn arrastar_o_vao_alarga_a_esquerda_e_estreita_o_documento() {
    let (mut m, id) = aberta();
    let (col, code, entry) = rects(&mut m, id);
    let d = 60.0;
    arrasta(&mut m, id, vao(col), d);
    let (col2, code2, entry2) = rects(&mut m, id);
    assert_eq!((col2.x, col2.w), (col.x, col.w + d), "a coluna da esquerda");
    assert_eq!((code2.x, code2.w), (code.x + d, code.w - d), "o código");
    assert_eq!(
        (entry2.x, entry2.w),
        (entry.x, entry.w + d),
        "In this script"
    );
    // O Esc a meio de um arrasto devolve a largura do carregar.
    let (x, y) = vao(col2);
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let (down, prim) = (ph2d_host::PointerKind::Down, PointerButton::Primary);
    assert!(m.docs().pointer(id.0, input(&mut ts), down, prim, x, y));
    let _ = m.docs().pointer_move(id.0, input(&mut ts), x + d, y);
    m.tecla(id, Key::Escape, Modifiers::default(), None);
    m.pinta(id);
    assert_eq!(rects(&mut m, id).0.w, col2.w, "o Esc devolve a largura");
}

#[test]
fn o_arrasto_para_no_minimo_e_no_maximo() {
    let (mut m, id) = aberta();
    let (col, ..) = rects(&mut m, id);
    arrasta(&mut m, id, vao(col), -5000.0);
    let (min_col, ..) = rects(&mut m, id);
    let fit = m.docs().bench.outline_fit_w();
    assert!(fit > 0.0, "controlo: o «In this script» mediu-se");
    assert!(
        min_col.w >= fit && min_col.w < ScriptListView::width(true),
        "mínimo {} (cabe {fit})",
        min_col.w
    );
    arrasta(&mut m, id, vao(min_col), -100.0);
    assert_eq!(rects(&mut m, id).0.w, min_col.w, "o mínimo segura");
    arrasta(&mut m, id, vao(min_col), 5000.0);
    let (max_col, max_code, _) = rects(&mut m, id);
    assert!(max_col.w > col.w, "alargou");
    assert!(
        max_code.w >= PANEL_MIN_W_PX,
        "o documento fica com {}",
        max_code.w
    );
    arrasta(&mut m, id, vao(max_col), 100.0);
    assert_eq!(rects(&mut m, id).0.w, max_col.w, "o máximo segura");
}

#[test]
fn o_clique_no_in_this_script_segue_a_largura_nova() {
    let (mut m, id) = aberta();
    let (col, ..) = rects(&mut m, id);
    arrasta(&mut m, id, vao(col), 120.0);
    let (.., entry) = rects(&mut m, id);
    // O fim da entrada alargada: fora da caixa de antes, dentro da nova.
    let (x, y) = (entry.x + entry.w - cards::margin(), entry.y + entry.h * 0.5);
    assert!(
        x > col.x + col.w,
        "controlo: o alvo está para lá da largura antiga"
    );
    m.clica(id, x, y, Modifiers::default());
    m.quadro();
    assert_eq!(
        m.docs().cursor_of(id).map(|c| c.0),
        Some(1),
        "saltou a on_update"
    );
}

#[test]
fn o_duplo_clique_no_vao_devolve_a_largura_de_fabrica() {
    let (mut m, id) = aberta();
    let (col, ..) = rects(&mut m, id);
    assert_eq!(
        col.w,
        ScriptListView::width(true),
        "controlo: nasce na de fábrica"
    );
    arrasta(&mut m, id, vao(col), 60.0);
    let (largo, ..) = rects(&mut m, id);
    assert_eq!(largo.w, col.w + 60.0, "controlo: o arrasto alargou");
    // Um clique só no vão não mexe na largura.
    let (x, y) = vao(largo);
    m.clica(id, x, y, Modifiers::default());
    m.pinta(id);
    assert_eq!(rects(&mut m, id).0.w, largo.w, "um clique só");
    // O segundo clique, sem arrasto, faz o duplo: volta a de fábrica.
    m.clica(id, x, y, Modifiers::default());
    m.pinta(id);
    assert_eq!(rects(&mut m, id).0.w, col.w, "o duplo-clique");
}

#[test]
fn o_duplo_clique_num_cartao_nao_mexe_no_divisor() {
    let (mut m, id) = aberta();
    let (col, ..) = rects(&mut m, id);
    arrasta(&mut m, id, vao(col), 60.0);
    let (largo, code, _) = rects(&mut m, id);
    let (cx, cy) = centro(code);
    // Um clique no vão e o segundo no cartão também não é duplo.
    let (x, y) = vao(largo);
    m.clica(id, x, y, Modifiers::default());
    for _ in 0..2 {
        m.clica(id, cx, cy, Modifiers::default());
        m.pinta(id);
    }
    m.clica(id, x, y, Modifiers::default());
    m.pinta(id);
    assert_eq!(rects(&mut m, id).0.w, largo.w, "o cartão não reinicia");
}

/// No mínimo os títulos dos dois cartões («SCRIPTS», «IN THIS SCRIPT», maiúsculas espaçadas)
/// pintam-se inteiros: o mínimo conta-os, não só as linhas.
#[test]
fn no_minimo_os_titulos_dos_cartoes_cabem_inteiros() {
    let (mut m, id) = aberta();
    let (col, ..) = rects(&mut m, id);
    arrasta(&mut m, id, vao(col), -5000.0);
    let (min_col, ..) = rects(&mut m, id);
    assert!(min_col.w < col.w, "controlo: estreitou ({})", min_col.w);
    let (_, medidos) = ph2d_editor_core::text_elide::elisao::medindo(|| m.pinta(id));
    for k in ["code.list.title", "code.bancada.outline.title"] {
        let t = ph2d_i18n::tr(k);
        let titulo: Vec<_> = medidos
            .iter()
            .filter(|x| x.texto.eq_ignore_ascii_case(t))
            .collect();
        assert!(!titulo.is_empty(), "controlo: «{t}» foi medido");
        for x in titulo {
            assert!(x.coube(), "cortado no mínimo {}: {x:?}", min_col.w);
        }
    }
}

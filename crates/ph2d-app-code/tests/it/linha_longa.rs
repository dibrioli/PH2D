//! ⛔ **Uma linha muito longa não custa a linha inteira por quadro** (auditoria do fecho, item 9): a
//! pintura preparava o texto inteiro da linha (cópia, tabs, cores, a chave da cache) e entregava à
//! cena TODOS os glifos dela — uma linha de 1 MB colada custava 1 MB e um milhão de glifos em cada
//! quadro, com a vista a mostrar duzentas colunas.

use crate::{Mesa, ctrl};
use ph2d_editor_core::doc_surface::{DocumentSurface, Key};
use ph2d_host::Modifiers;

/// O trabalho de um quadro já moldado, com uma linha de `n` bytes COLADA no meio de um script.
fn trabalho_por_quadro(n: usize) -> ph2d_panel_code::PaintWork {
    let (mut m, ids) = Mesa::new(&[("bob.luau", "local a = 1\nlocal b = 2\n")]);
    let id = ids[0];
    m.pede(id);
    m.tecla(id, Key::End, Modifiers::default(), None);
    let longa = format!(" -- {}", "ab cd é ".repeat(n / 9));
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let input = ph2d_editor_core::doc_surface::KeyInput {
        key: Key::Char('v'),
        mods: ctrl(),
        pressed: true,
        text: Some("v"),
        paste: Some(longa),
    };
    let _ = m.docs().key(id.0, &mut ts, input);
    m.pinta(id);
    let antes = ph2d_panel_code::paint_work();
    m.pinta(id);
    m.pinta(id);
    let depois = ph2d_panel_code::paint_work();
    ph2d_panel_code::PaintWork {
        row_bytes: (depois.row_bytes - antes.row_bytes) / 2,
        glyphs: (depois.glyphs - antes.glyphs) / 2,
    }
}

#[test]
fn uma_linha_de_1_mb_colada_custa_por_quadro_o_mesmo_que_uma_de_64_kb() {
    let curta = trabalho_por_quadro(64 << 10);
    let longa = trabalho_por_quadro(1 << 20);
    eprintln!("por quadro: 64 KB → {curta:?} · 1 MB → {longa:?}");
    assert!(
        curta.row_bytes > 0 && curta.glyphs > 0,
        "controlo: a régua conta a pintura"
    );
    assert!(
        longa.row_bytes <= curta.row_bytes && longa.glyphs <= curta.glyphs,
        "a linha de 1 MB custa mais por quadro: {longa:?} contra {curta:?}"
    );
}

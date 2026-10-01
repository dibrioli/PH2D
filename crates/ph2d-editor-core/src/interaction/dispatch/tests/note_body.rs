//! ⭐⭐ **O botão de MINIMIZAR e o clique no corpo QUEBRADO de uma nota** — pelo despacho real
//! (2026-10-01, ordem do dono: *«crie um botão nas notas que possibilite minimizar as notas. O card
//! das notas não cresce … e não há quebra automática de linha»*).

use super::*;
use crate::ids::{self, NoteFace, note_ids};
use crate::widget::showcase::{linhas_visuais, texto_da_linha};
use ph2d_text::TextSystem;

const P: NodeId = ids::PHYSICS_PANEL;

fn com_uma_nota() -> WidgetStore {
    let mut store = WidgetStore::with_capacity(64);
    store.set_panel_rect(P, Rect::new(0.0, 0.0, 400.0, 600.0));
    assert_eq!(store.notes_push(P, 0, None), Some(0));
    store
}

/// ⭐⭐ **Um Down no botão troca a nota entre aberta e minimizada, e não foca o título por baixo.**
/// *Mutação: o braço do `Fold` fora do `premiu_a_nota` ⇒ a nota fica aberta e o Down cai na porta
/// da focabilidade.*
#[test]
fn o_botao_minimiza_e_volta_a_abrir() {
    let mut store = com_uma_nota();
    let caixas = note_ids(P);
    assert_eq!(store.nota_de(caixas.fold[0]), Some((P, 0, NoteFace::Fold)));
    let mut hits = HitIndex::new();
    let titulo = Rect::new(20.0, 10.0, 300.0, 24.0);
    hits.register(caixas.title[0], titulo);
    hits.register(caixas.fold[0], Rect::new(20.0, 14.0, 16.0, 16.0));
    let arena = Bump::new();
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 28.0, 22.0),
        &arena,
    );
    assert!(
        store.notes_for_panel(P)[0].minimized,
        "o botão não minimizou"
    );
    assert_ne!(
        store.focus_id(),
        Some(caixas.title[0]),
        "o Down no botão focou o título por baixo"
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Up, 28.0, 22.0),
        &arena,
    );
    let _ = dispatch_pointer(
        &mut store,
        &hits,
        pointer(PointerKind::Down, 28.0, 22.0),
        &arena,
    );
    assert!(
        !store.notes_for_panel(P)[0].minimized,
        "o 2.º Down não voltou a abrir"
    );
}

/// ⭐⭐ **Clicar na 2.ª linha VISUAL de uma frase quebrada põe o caret nela** — pela mesma lei que
/// o pintor desenha. *Mutação: o braço `corpo_de_nota` fora do `byte_offset_from_click_xy` ⇒ sem
/// `\n` o texto é UMA linha lógica e o caret cai na 1.ª.*
#[test]
fn o_clique_na_segunda_linha_visual_poe_o_caret_nela() {
    let mut store = com_uma_nota();
    let corpo = note_ids(P).body[0];
    let texto = "uma frase comprida que tem de partir em varias linhas no corpo da nota";
    if let Some(InteractiveState::TextInput { text, .. }) = store.get_mut(corpo) {
        *text = texto.to_string();
    }
    let rect = Rect::new(20.0, 40.0, 160.0, 200.0);
    let mut hits = HitIndex::new();
    hits.register(corpo, rect);
    let mut ts = TextSystem::without_system_fonts();
    let fonte = ph2d_tokens::TypeToken::Base.px();
    let m = crate::widget::text_area_metrics(rect);
    let linhas = linhas_visuais(texto, m.inner_w, |s| ts.prefix_width(s, fonte));
    assert!(
        linhas.len() >= 3,
        "fixtura: a frase tem de partir ({linhas:?})"
    );
    let (a, b) = linhas[1];
    let arena = Bump::new();
    let _ = dispatch_pointer_with_text(
        &mut store,
        &hits,
        pointer(
            PointerKind::Down,
            m.inner_x + 1.0,
            m.inner_y + m.line_h * 1.5,
        ),
        Some(&mut ts),
        &arena,
    );
    let caret = match store.get(corpo) {
        Some(InteractiveState::TextInput { caret, .. }) => *caret,
        _ => panic!("o corpo não é uma caixa de texto"),
    };
    assert!(
        caret >= a && caret <= a + texto_da_linha(texto, (a, b)).len(),
        "o caret ({caret}) não caiu na 2.ª linha visual ({:?})",
        texto_da_linha(texto, (a, b))
    );
}

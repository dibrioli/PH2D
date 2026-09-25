//! ⭐⭐ **OS GATES DA CAMADA DA GRADE** — report do dono de 2026-09-24 (*«Behind deixa o grid mais
//! discreto mas não atrás dos objetos»*).
//!
//! A régua é o que cada cena EMITE (palavras do buffer do Vello, `probe_bin_info_words`), não o que
//! o código diz que faz: a grade tem de sair de UMA das duas cenas e nunca das duas, e o `Behind`
//! tem de a pôr na cena própria — que é a que a shell desenha antes dos sprites.

use super::*;
use crate::NodeId;
use crate::grid::GridView;
use crate::zones::Rect;
use ph2d_text::TextSystem;

fn hero(em_frente: bool) -> HeroScreen {
    let mut h = HeroScreen::new(NodeId(1));
    h.view.grid_visible = true;
    h.set_grid_view(Some(GridView {
        camera_center: [0.0, 0.0],
        camera_height_world: 10.0,
        window_w: 1280.0,
        window_h: 800.0,
        canvas: Rect::new(0.0, 0.0, 1280.0, 800.0),
    }));
    h.grid.snap_state.grid_in_front = em_frente;
    // Um quadro inteiro, para o `last_layout` existir como no produto.
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    crate::screens::paint_hero_screen(
        &mut h,
        Rect::new(0.0, 0.0, 1280.0, 800.0),
        &mut scene,
        &mut text,
    );
    h
}

fn palavras(s: &VectorScene) -> u32 {
    s.probe_bin_info_words()
}

/// ⭐ **Com `Behind`, a grade sai da cena PRÓPRIA e não da do chrome** — e com `In front`, o
/// contrário. *Mutação: o `paint_in_chrome` a pintar sempre (o defeito do report, com a grade por
/// cima dos objectos) ⇒ a 1.ª metade reprova; o `paint_behind` a devolver `false` ⇒ a 2.ª.*
#[test]
fn a_grade_sai_de_uma_camada_so_e_o_behind_e_a_de_tras() {
    let vazia = palavras(&VectorScene::new());
    let layout_de = |h: &HeroScreen| h.last_layout.expect("o quadro publica o layout");

    // Atrás: nada no chrome, tudo na cena própria.
    let atras = hero(false);
    let layout = layout_de(&atras);
    let mut chrome = VectorScene::new();
    paint_in_chrome(&atras, &layout, &mut chrome);
    assert_eq!(
        palavras(&chrome),
        vazia,
        "com `Behind` a grade continua a ser pintada no CHROME — que o compositor poe por cima \
         dos objectos: e' o report do dono"
    );
    let mut propria = VectorScene::new();
    assert!(
        paint_behind(&atras, &mut propria),
        "com `Behind` a porta disse que nao ha' grade para pintar atras"
    );
    assert!(
        palavras(&propria) > vazia,
        "com `Behind` a cena propria da grade saiu vazia"
    );
    assert!(is_behind(&atras));

    // À frente: tudo no chrome, nada na cena própria (senão sai duas vezes).
    let frente = hero(true);
    let layout = layout_de(&frente);
    let mut chrome = VectorScene::new();
    paint_in_chrome(&frente, &layout, &mut chrome);
    assert!(
        palavras(&chrome) > vazia,
        "com `In front` a grade sumiu do chrome"
    );
    let mut propria = VectorScene::new();
    assert!(!paint_behind(&frente, &mut propria));
    assert_eq!(
        palavras(&propria),
        vazia,
        "com `In front` a grade tambem foi para tras: sai DUAS vezes"
    );
    assert!(!is_behind(&frente));
}

/// ⭐ **A grade ATRÁS é o MESMO desenho, com a MESMA força** — o `× 0,4` era a aproximação que o
/// dono leu como *«mais discreto»*, e ele saiu.
///
/// ⚠️ **Duas metades, porque a régua das palavras é cega à COR:** ela prova que as duas cenas
/// emitem a mesma geometria, e um `opacity *= 0,4` de volta não muda uma palavra. A metade da força
/// lê os dois ficheiros por onde a grade passa — o pintor do quadro e esta porta — e exige que
/// nenhum volte a escalar a opacidade. *Mutação: repor o `× 0,4` em qualquer um dos dois.*
#[test]
fn atras_a_grade_e_o_mesmo_desenho_e_nao_fica_mais_fraca() {
    let agulha = concat!("opacity ", "*=");
    for (nome, fonte) in [
        ("paint.rs", include_str!("paint.rs")),
        ("grid_layer.rs", include_str!("grid_layer.rs")),
    ] {
        assert!(
            !fonte.contains(agulha),
            "`{nome}` voltou a escalar a opacidade da grade — o `Behind` volta a ser «mais \
             discreto» em vez de «atras»"
        );
    }
    let atras = hero(false);
    let frente = hero(true);
    let mut a = VectorScene::new();
    let _ = paint_behind(&atras, &mut a);
    let mut f = VectorScene::new();
    paint_in_chrome(&frente, &frente.last_layout.expect("layout"), &mut f);
    assert_eq!(
        palavras(&a),
        palavras(&f),
        "a grade de tras nao e' o mesmo desenho que a da frente"
    );
    // e com a grade desligada, nenhuma das duas pinta
    let mut off = hero(false);
    off.view.grid_visible = false;
    let mut s = VectorScene::new();
    assert!(!paint_behind(&off, &mut s));
}

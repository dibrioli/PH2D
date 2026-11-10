//! R6a — as diferenças das primeiras fotos do Try it contra a maquete C: a tecla PREMIDA enche-se da cor da acção
//! (C2), e a manche diz a direcção por ícone, marca os campos com «D»/«P» e só pinta o «Pressed» de verde (C3).

use super::try_it_censo::cena;
use ph2d_app_input_map::palette::of_action;
use ph2d_app_input_map::view_keyboard::state::KbHit;
use ph2d_editor_core::icons::IconId;
use ph2d_i18n::tr;
use ph2d_input::Key;
use ph2d_tokens::ColorToken;

fn up() -> Key {
    Key::named_in_script("up")
        .and_then(|v| v.first().copied())
        .expect("a seta para cima")
}

#[test]
fn a_tecla_premida_no_try_it_enche_se_da_cor_da_accao_e_a_da_b_so_tem_contorno() {
    let m = cena("C2");
    let doc = &m.tab().doc;
    let jump = m.hero.input_map.id("jump").expect("jump");
    let key = doc.rect_of(KbHit::Key(up()).id()).expect("a ↑ desenhada");
    let [(r, cor)] = doc.census.filled.as_slice() else {
        panic!("C2: só a ↑ enchida, e não {:?}", doc.census.filled);
    };
    assert_eq!(*cor, of_action(jump), "na cor do jump");
    let perto = |a: f32, b: f32| (a - b).abs() < 1e-3;
    assert!(
        perto(r.x, key.x) && perto(r.y, key.y) && perto(r.w, key.w) && perto(r.h, key.h),
        "a enchida é a ↑: {r:?} × {key:?}"
    );
    let b1 = cena("B1");
    assert!(
        b1.tab().doc.census.filled.is_empty(),
        "B1: a acção aberta acende por contorno, não enche"
    );
}

#[test]
fn a_manche_diz_a_direccao_por_icone_marca_os_campos_e_so_o_pressed_e_verde() {
    let mut m = cena("C3");
    // A manche da cena (a shell empurra-a pelo comando ao vivo: `manche`).
    let (eixo, v) =
        ph2d_app_game::smoke::input_map::manche(ph2d_app_game::smoke::input_map::PRIMEIRA + 10)
            .expect("a C3 empurra a manche");
    m.live.gamepad.handle_axis(eixo, v);
    m.frame();
    let c = &m.tab().doc.census;
    let titulo = tr("input_map_try.stick_of")
        .replace("{action}", "move_left")
        .replace("{stick}", tr("input_map_try.left_stick"));
    assert!(c.texts.iter().any(|t| t.1 == titulo), "C3: «{titulo}»");
    assert_eq!(
        c.icons.iter().map(|i| i.1).collect::<Vec<_>>(),
        vec![IconId::ChevronLeft],
        "a direcção da manche é um ícone"
    );
    for k in ["input_map_try.dead_mark", "input_map_try.press_mark"] {
        assert!(c.texts.iter().any(|t| t.1 == tr(k)), "a marca {k} no campo");
    }
    let verdes: Vec<&str> = (c.runs.iter())
        .filter(|r| r.1 == ColorToken::Success)
        .map(|r| r.0.as_str())
        .collect();
    assert_eq!(
        verdes,
        vec![tr("input_map_try.pressed")],
        "só o «Pressed» é verde"
    );
    let valores: Vec<&str> = (c.runs.iter())
        .filter(|r| r.2 && r.1 == ColorToken::Text1)
        .map(|r| r.0.as_str())
        .collect();
    assert_eq!(
        valores,
        vec!["0.62", "0.55"],
        "os valores em negrito, no texto normal"
    );
}

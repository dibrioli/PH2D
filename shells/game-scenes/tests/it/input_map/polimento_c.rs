//! R3-C (onda 2): a vista C no registo de controlos (plano §13.3) — receitas, Replace/Cancel, caixas, «Opens with»,
//! Forget e o polegar: hover no quadro só no pairado, premido no Down, arrastar para fora cancela, dispara no Up
//! dentro, o Tab chega com o anel, o Enter activa, o nó a11y com papel e nome, e o cursor.

use ph2d_app_input_map::TAB_ID;
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_try::state::TryHit;
use ph2d_editor_core::doc_surface::{DocumentSurface, Key as K, PointerLook};
use ph2d_input::recipes::Recipe;

use super::Mesa;
use super::lista::onze;

fn key(n: &str) -> ph2d_input::Key {
    ph2d_input::Key::named_in_script(n).expect("tecla")[0]
}

fn mesa() -> Mesa {
    let mut m = Mesa::aberta(View::TryIt);
    m.hero.input_map = onze();
    m.frame();
    m
}

fn longe(m: &Mesa, id: ph2d_a11y::NodeId) -> (f32, f32) {
    let r = m.tab().doc.rect_of(id).expect("pintado");
    (r.x + r.w * 0.5, r.y - r.h * 4.0)
}

fn node(m: &Mesa, id: ph2d_a11y::NodeId) -> ph2d_a11y::Node {
    m.tab()
        .a11y(TAB_ID)
        .into_iter()
        .find(|(i, _)| *i == id)
        .map(|(_, n)| n)
        .expect("o nó do controlo")
}

/// Os alvos da C que se carregam, com o papel e o nome lido.
fn kinds(m: &Mesa) -> Vec<(TryHit, ph2d_a11y::Role, String)> {
    let jump = m.hero.input_map.id("jump").expect("jump");
    vec![
        (
            TryHit::Recipe(Recipe::Platformer),
            ph2d_a11y::Role::Button,
            "Start from Platformer".into(),
        ),
        (
            TryHit::PlayersCanChange,
            ph2d_a11y::Role::CheckBox,
            "Players can change keys".into(),
        ),
        (
            TryHit::Shown(jump),
            ph2d_a11y::Role::CheckBox,
            "jump".into(),
        ),
        (
            TryHit::OpenKey,
            ph2d_a11y::Role::Button,
            "Opens with F1".into(),
        ),
    ]
}

#[test]
fn cada_controlo_da_c_acende_so_ele_no_quadro() {
    let mut m = mesa();
    for (h, _, _) in kinds(&m) {
        let at = m.at(h.id());
        m.hover(at);
        assert_eq!(
            m.tab().doc.hover_live(h.id()),
            1.0,
            "{h:?} acende no quadro"
        );
        for (o, _, _) in kinds(&m) {
            if o != h {
                assert_eq!(m.tab().doc.hover_live(o.id()), 0.0, "{o:?} frio sob {h:?}");
            }
        }
        m.hover(longe(&m, h.id()));
        assert_eq!(m.tab().doc.hover_live(h.id()), 0.0, "{h:?} apaga no quadro");
    }
}

#[test]
fn cada_controlo_da_c_prime_no_down_cancela_fora_e_dispara_no_up_dentro() {
    let mut m = mesa();
    for (h, _, _) in kinds(&m) {
        let at = m.at(h.id());
        let before = (
            m.hero.input_map.clone(),
            m.tab().doc.tryit.listening_open_key,
        );
        assert!(m.down(at));
        assert!(m.tab().doc.is_pressed(h.id()), "{h:?} premido");
        m.hover(longe(&m, h.id()));
        assert!(
            !m.tab().doc.is_pressed(h.id()),
            "{h:?}: fora apaga o premido"
        );
        m.up(longe(&m, h.id()));
        let after = (
            m.hero.input_map.clone(),
            m.tab().doc.tryit.listening_open_key,
        );
        assert_eq!(after, before, "{h:?}: o Up fora cancela");
        assert_eq!(m.tab().doc.tryit.confirm, None, "{h:?}: nada pedido");
        m.click(at);
        let fired = m.hero.input_map != before.0
            || m.tab().doc.tryit.listening_open_key != before.1
            || m.tab().doc.tryit.confirm.is_some();
        assert!(fired, "{h:?}: o Up dentro dispara");
        // Desfaz o gesto para o seguinte partir do mesmo sítio.
        m.hero.input_map = before.0;
        m.tab_mut().doc.tryit.listening_open_key = false;
        m.tab_mut().doc.tryit.confirm = None;
        m.frame();
    }
}

#[test]
fn o_tab_chega_a_cada_controlo_da_c_e_o_enter_activa() {
    let mut m = mesa();
    let id = TryHit::Recipe(Recipe::Platformer).id();
    let mut seen = false;
    for _ in 0..64 {
        m.key(K::Tab, None);
        if m.tab().doc.focused() == Some(id) {
            seen = true;
            break;
        }
    }
    assert!(seen, "o Tab chega à receita");
    m.key(K::Enter, None);
    assert_eq!(
        m.tab().doc.tryit.confirm,
        Some(Recipe::Platformer),
        "o Enter activa a receita focada"
    );
}

#[test]
fn as_teclas_ao_vivo_sao_so_leitura_fora_do_tab() {
    let mut m = mesa();
    let w = ph2d_app_input_map::view_keyboard::state::KbHit::Key(key("w")).id();
    assert!(
        m.tab().doc.rect_of(w).is_some(),
        "a tecla W está pintada na C"
    );
    let _ = node(&m, w);
    for _ in 0..200 {
        m.key(K::Tab, None);
        assert_ne!(
            m.tab().doc.focused(),
            Some(w),
            "a tecla W não recebe foco na C"
        );
    }
}

#[test]
fn cada_controlo_da_c_tem_no_a11y_e_cursor() {
    let m = mesa();
    for (h, role, label) in kinds(&m) {
        let n = node(&m, h.id());
        assert_eq!(n.role(), role, "{h:?}");
        assert_eq!(n.label(), Some(label.as_str()), "{h:?}");
        let (x, y) = m.at(h.id());
        assert_eq!(
            m.tab().pointer_look(TAB_ID, x, y),
            Some(PointerLook::Hand),
            "{h:?}"
        );
    }
}

#[test]
fn replace_e_cancel_premem_tem_nos_e_esc_enter_respondem() {
    let mut m = mesa();
    let before = m.hero.input_map.clone();
    m.click(m.at(TryHit::Recipe(Recipe::Shooter).id()));
    for (h, label) in [(TryHit::Replace, "Replace"), (TryHit::Cancel, "Cancel")] {
        let n = node(&m, h.id());
        assert_eq!(n.role(), ph2d_a11y::Role::Button);
        assert_eq!(n.label(), Some(label));
        let at = m.at(h.id());
        m.down(at);
        assert!(m.tab().doc.is_pressed(h.id()), "{h:?} premido");
        m.hover(longe(&m, h.id()));
        m.up(longe(&m, h.id()));
        assert_eq!(
            m.tab().doc.tryit.confirm,
            Some(Recipe::Shooter),
            "{h:?}: fora cancela"
        );
    }
    m.physical(key("escape"));
    assert_eq!(m.tab().doc.tryit.confirm, None, "Esc = Cancel");
    assert_eq!(m.hero.input_map, before);
    m.click(m.at(TryHit::Recipe(Recipe::Shooter).id()));
    m.physical(key("enter"));
    m.frame();
    assert_eq!(m.tab().doc.tryit.confirm, None, "Enter = Replace");
    assert_ne!(m.hero.input_map, before, "o Enter substituiu");
}

#[test]
fn o_forget_e_uma_ligacao_que_sublinha_e_dispara() {
    let mut m = mesa();
    m.tab_mut().doc.tryit.player_changed = 2;
    m.frame();
    let id = TryHit::Forget.id();
    let n = node(&m, id);
    assert_eq!(n.role(), ph2d_a11y::Role::Link);
    assert_eq!(n.label(), Some("Forget them"));
    let (x, y) = m.at(id);
    assert_eq!(m.tab().pointer_look(TAB_ID, x, y), Some(PointerLook::Hand));
    m.hover((x, y));
    assert_eq!(m.tab().doc.hover_live(id), 1.0);
}

#[test]
fn o_anel_da_escuta_pulsa_e_fica_parado_com_reduced_motion() {
    let mut m = mesa();
    m.hero
        .motion
        .set_character(ph2d_editor_core::motion::UiCharacter::Expressive);
    m.click(m.at(TryHit::OpenKey.id()));
    assert!(m.tab().doc.tryit.listening_open_key);
    assert!(m.tab().wants_frame(), "à escuta, o pulso pede quadro");
    m.hero.motion.set_reduced_motion(true);
    m.frame();
    assert!(
        !m.tab().wants_frame(),
        "reduced motion: anel parado, sem quadro"
    );
    assert!(
        !m.tab().doc.census.dashed.is_empty(),
        "o anel tracejado continua lá"
    );
}

#[test]
fn o_polegar_da_c_e_uma_barra_que_desvanece() {
    let mut m = mesa();
    m.paint_in(ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1280.0, 720.0)); // LITERAL-PX-OK: a janela do plano §8
    let id = ph2d_app_input_map::view_try::THUMB;
    let n = node(&m, id);
    assert_eq!(n.role(), ph2d_a11y::Role::ScrollBar);
    assert_eq!(m.tab().doc.thumb_fade(id), 0.0, "frio sem o ponteiro");
}

/// O censo da C nas áreas REAIS da aba (1600×1000 e 1280×720), nos dois modos de texto, com a pergunta aberta,
/// à escuta e com o Forget à vista: zero texto cortado.
#[test]
fn o_censo_da_c_fica_a_zero_nas_areas_reais() {
    use ph2d_editor_core::zones::Rect;
    // LITERAL-PX-OK: as áreas reais da aba (lição 1 da missão)
    let areas = [
        Rect::new(308.0, 28.0, 988.0, 890.0),
        Rect::new(297.0, 28.0, 690.0, 604.0),
    ];
    super::em_cada_modo_de_texto(|| {
        for area in areas {
            let mut m = mesa();
            m.tab_mut().doc.tryit.player_changed = 3;
            m.tab_mut().doc.tryit.confirm = Some(Recipe::Shooter);
            m.tab_mut().doc.tryit.listening_open_key = true;
            m.frame();
            m.paint_in(area);
            let c = &m.tab().doc.census;
            assert_eq!(c.elided(), Vec::<&str>::new(), "{area:?}: texto cortado");
        }
    });
}

//! Auditoria independente da onda 2 (lente «correcção e fios»): as asserções-vermelhas que faltavam ao §13.1–§13.3
//! do plano `docs/InputMap/01_plano.md`. Cada teste afirma a propriedade pela porta real da aba.

use super::Mesa;
use super::lista::onze;
use super::teclado_censo::AREAS;
use ph2d_app_input_map::View;
use ph2d_app_input_map::ids::{self, ListHit};
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_editor_core::doc_surface::Key as DocKey;
use ph2d_input::{Binding, Key};

/// (4) A 1280 a B empilha e a lista rola: o Tab nunca pousa num controlo que o recorte escondeu (sem alvo de
/// clique pintado). Pousar lá pinta o anel fora do cartão e o Enter «clica» no centro de um rect invisível.
#[test]
fn o_tab_da_b_a_1280_nunca_pousa_num_controlo_escondido_pelo_recorte() {
    let area = AREAS[1];
    let mut m = Mesa::aberta(View::Keyboard);
    m.hero.input_map = onze();
    m.frame();
    m.click(m.at(KbHit::Sub(Sub::Keyboard).id()));
    m.paint_in(area);
    assert!(m.tab().doc.kb.stacked, "a 1280 a B empilha");
    for i in 0..300 {
        m.key(DocKey::Tab, None);
        m.paint_in(area);
        let doc = &m.tab().doc;
        if let Some(f) = doc.focused() {
            assert!(
                doc.rect_of(f).is_some(),
                "Tab n.º {i}: o foco {f:?} pousou num controlo sem alvo visível (recortado)"
            );
        }
    }
}

/// (1) O Numpad Enter confirma onde o Enter confirma: na busca vazia o Enter não vira «procurar pela tecla Enter»,
/// e o Numpad Enter também não.
#[test]
fn o_numpad_enter_na_busca_vazia_faz_o_que_o_enter_faz() {
    let run = |k: Key| {
        let mut m = Mesa::aberta(View::List);
        m.hero.input_map = onze();
        m.frame();
        m.click(m.at(ids::SEARCH));
        let took = m.physical(k);
        (took, m.tab().doc.list.search_key.is_some())
    };
    let enter = run(Key::ENTER);
    let kp = run(Key::NUMPAD_ENTER);
    assert_eq!(enter, (false, false), "o Enter não é procurado");
    assert_eq!(kp, enter, "o Numpad Enter divergiu do Enter na busca vazia");
}

/// (6) A frase do limite diz o valor que FICA no mapa: «Kept at 1.00» só se a zona morta gravada é 1,00; e um
/// valor dentro de 0..1 que o mapa muda tem de avisar.
#[test]
fn a_zona_morta_escrita_e_a_que_o_mapa_guarda_ou_avisa() {
    let mut m = Mesa::aberta(View::List);
    let mut map = ph2d_input::InputMap::new();
    let a = map.create("steer");
    if let Some(act) = map.get_mut(a) {
        act.bindings.push(Binding::PadAxis {
            axis: ph2d_input::GamepadAxis::LeftStickX,
            positive: true,
        });
        act.set_zone(0.2, 0.5);
    }
    m.hero.input_map = map;
    m.frame();
    for typed in ["1.5", "1"] {
        m.click(m.at(ListHit::Dead(a).id()));
        m.type_text(typed);
        m.key(DocKey::Enter, None);
        let dead = m.hero.input_map.get(a).expect("a").dead_zone;
        let warn = m.tab().doc.list.warn.map(|w| w.2.i18n_key());
        let ok = match warn {
            Some("input_map_list.kept_max") => (dead - 1.0).abs() < 1e-6,
            None => (dead - typed.parse::<f32>().expect("n")).abs() < 1e-6,
            _ => false,
        };
        assert!(
            ok,
            "Dead «{typed}»: o mapa guardou {dead}, a frase diz {warn:?} (ph2d_input::action::set_zone prende a 0,99)"
        );
    }
}

/// (4) Tirar a ligação pelo × focado não deixa o foco num controlo que já não existe (o Enter seguinte seria
/// engolido e o Tab recomeçaria do início).
#[test]
fn o_foco_nao_fica_num_controlo_que_sumiu() {
    let mut m = Mesa::aberta(View::Keyboard);
    m.hero.input_map = onze();
    m.frame();
    m.click(m.at(KbHit::Sub(Sub::Keyboard).id()));
    let fire = m.hero.input_map.id("fire").expect("fire");
    m.click(m.at(KbHit::Row(fire).id()));
    assert_eq!(m.hero.input_map.get(fire).expect("f").bindings.len(), 1);
    let want = KbHit::TileX(0).id();
    let mut reached = false;
    for _ in 0..400 {
        m.key(DocKey::Tab, None);
        if m.tab().doc.is_focused(want) {
            reached = true;
            break;
        }
    }
    assert!(reached, "o Tab chega ao × do único ladrilho");
    m.key(DocKey::Enter, None);
    assert!(m.hero.input_map.get(fire).expect("f").bindings.is_empty());
    let doc = &m.tab().doc;
    if let Some(f) = doc.focused() {
        assert!(
            doc.rect_of(f).is_some(),
            "o foco ficou em {f:?}, que já não está pintado"
        );
    }
}

/// (7) Sem nada a animar a aba não pede quadro; o caret da busca focada pede; com reduced motion, nem ele.
#[test]
fn a_aba_parada_nao_pede_quadro_e_o_reduced_motion_para_o_caret() {
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = onze();
    m.frame();
    m.frame();
    assert!(!m.tab().doc.wants_frame(), "parada: sem repintura contínua");
    m.hover(m.segment(View::Keyboard));
    assert!(!m.tab().doc.wants_frame(), "o hover é instantâneo (D32)");
    m.click(m.at(ids::SEARCH));
    assert!(m.tab().doc.wants_frame(), "o caret da busca focada pisca");
    m.hero.motion.set_reduced_motion(true);
    m.frame();
    assert!(
        !m.tab().doc.wants_frame(),
        "reduced motion: caret fixo, sem quadro"
    );
}

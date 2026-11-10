//! R3-B (plano `docs/InputMap/01_plano.md` §13.2–§13.3): a vista B empilhada a 1280 e os controlos dela no registo
//! (hover instantâneo só no pairado, premido com o botão em baixo, arrastar para fora cancela, dispara no Up dentro,
//! Tab com o anel, Enter activa, nó a11y com papel e nome, cursor) e o Remove desligado com o porquê.

use super::Mesa;
use super::lista::onze;
use super::teclado_censo::AREAS;
use ph2d_app_input_map::view_keyboard::geometry::{Arrangement, drawn_keys};
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_app_input_map::{InputMapEdit, View};
use ph2d_editor_core::doc_surface::{Key as DocKey, PointerLook};
use ph2d_editor_core::zones::Rect;

fn inside(r: Rect, a: Rect) -> bool {
    r.x >= a.x - 0.01
        && r.y >= a.y - 0.01
        && r.x + r.w <= a.x + a.w + 0.01
        && r.y + r.h <= a.y + a.h + 0.01
}

fn mesa_b() -> Mesa {
    let mut m = Mesa::aberta(View::Keyboard);
    m.hero.input_map = onze();
    m.frame();
    m.click(m.at(KbHit::Sub(Sub::Keyboard).id()));
    m
}

/// A 1280 a coluna não segura o Wide: empilha (o teclado na largura inteira, Wide); a 1600 fica ao lado. Cada
/// tecla e cada linha da lista (rolando) cabe na área; o cartão aberto inteiro dentro dela.
#[test]
fn a_1280_a_b_empilha_e_tudo_se_alcanca() {
    super::em_cada_modo_de_texto(|| {
        for (i, area) in AREAS.into_iter().enumerate() {
            for sub in [Sub::Keyboard, Sub::Gamepad] {
                let mut m = mesa_b();
                m.click(m.at(KbHit::Sub(sub).id()));
                m.paint_in(area);
                let doc = &m.tab().doc;
                let body = doc.body();
                let fit = doc.kb.fit;
                eprintln!(
                    "{}×{} {sub:?}: stacked {} fit {fit:?} body {body:?}",
                    area.w, area.h, doc.kb.stacked
                );
                assert_eq!(doc.kb.stacked, i == 1, "{}: empilha só a 1280", area.w);
                if sub == Sub::Keyboard {
                    let f = fit.expect("o teclado pintado");
                    assert_eq!(
                        f.arrangement,
                        Arrangement::Wide,
                        "{}: o arranjo da foto",
                        area.w
                    );
                    let rs: Vec<Rect> = drawn_keys()
                        .into_iter()
                        .map(|k| doc.rect_of(KbHit::Key(k).id()).expect("tecla pintada"))
                        .collect();
                    let top = rs.iter().map(|r| r.y).fold(f32::INFINITY, f32::min);
                    let bot = rs.iter().map(|r| r.y + r.h).fold(0.0, f32::max);
                    eprintln!(
                        "  pitch {:.1} · altura do teclado {:.1}",
                        f.pitch,
                        bot - top
                    );
                    for r in &rs {
                        assert!(inside(*r, body), "tecla {r:?} fora de {body:?}");
                    }
                }
                // O cartão aberto: todo texto dele dentro do cartão da vista.
                for (r, s, _) in &doc.census.texts {
                    let cut = doc.census.clip.filter(|c| r.x >= c.x && r.x < c.x + c.w);
                    if cut.is_none() {
                        assert!(
                            r.y + r.h <= body.y + body.h + 0.01,
                            "«{s}» {r:?} abaixo de {body:?}"
                        );
                    }
                }
                // Cada linha alcança-se: rola até ela caber no recorte.
                let ids: Vec<_> = m.hero.input_map.actions().iter().map(|a| a.id).collect();
                for a in ids {
                    let mut seen = false;
                    m.tab_mut().doc.wheel(f32::MAX);
                    for _ in 0..64 {
                        m.paint_in(area);
                        let doc = &m.tab().doc;
                        let view = doc.census.clip.unwrap_or(body);
                        if doc
                            .rect_of(KbHit::Row(a).id())
                            .is_some_and(|r| inside(r, view))
                        {
                            seen = true;
                            break;
                        }
                        if !m.tab_mut().doc.wheel(-24.0)
                        // LITERAL-PX-OK: um entalhe da roda
                        {
                            break;
                        }
                    }
                    assert!(
                        seen,
                        "{}×{} {sub:?}: a linha {a:?} nunca se alcança",
                        area.w, area.h
                    );
                }
            }
        }
    });
}

/// Os alvos de cada espécie de controlo da B (com a acção aberta analógica para a barra).
fn kinds(m: &Mesa) -> Vec<(&'static str, KbHit, ph2d_a11y::Role)> {
    use ph2d_a11y::Role as R;
    let doc = &m.tab().doc;
    let open = doc.kb.open.expect("aberta");
    let key = drawn_keys()
        .into_iter()
        .find(|k| k.system_owned().is_none())
        .expect("tecla");
    vec![
        ("segment", KbHit::Sub(Sub::Gamepad), R::Tab),
        ("row", KbHit::Row(open), R::ListItem),
        ("add", KbHit::Add, R::Button),
        ("remove", KbHit::Remove, R::Button),
        ("key", KbHit::Key(key), R::Button),
        ("listen", KbHit::Listen, R::Button),
    ]
}

#[test]
fn cada_controlo_da_b_paira_preme_cancela_dispara_e_tem_no() {
    let all = kinds(&mesa_b());
    for (what, h, role) in all.clone() {
        let mut m = mesa_b();
        let id = h.id();
        let at = m.at(id);
        m.hover(at);
        let doc = &m.tab().doc;
        assert!(doc.is_hovered(id), "{what}: pairado");
        assert!(
            (doc.hover_live(id) - 1.0).abs() < f32::EPSILON,
            "{what}: hover instantâneo"
        );
        for (_, o, _) in all.iter().filter(|o| o.1 != h) {
            assert!(
                doc.hover_live(o.id()) < f32::EPSILON,
                "{what}: {o:?} também acende"
            );
        }
        assert_eq!(
            doc.look_at(at.0, at.1),
            Some(PointerLook::Hand),
            "{what}: cursor"
        );
        let node = doc
            .a11y()
            .into_iter()
            .find(|(n, _)| *n == id)
            .expect("nó a11y")
            .1;
        assert_eq!(node.role(), role, "{what}: papel");
        assert!(node.label().is_some_and(|l| !l.is_empty()), "{what}: nome");
        // Premido com o botão em baixo; fora do alvo apaga e o Up lá fora não dispara.
        m.tab_mut().doc.pending.clear();
        m.down(at);
        assert!(m.tab().doc.is_pressed(id), "{what}: premido");
        let far = (at.0, m.tab().doc.body().y + m.tab().doc.body().h - 1.0); // LITERAL-PX-OK: dentro da aresta
        m.hover(far);
        assert!(
            !m.tab().doc.is_pressed(id),
            "{what}: arrastar para fora apaga o premido"
        );
        let before = (m.tab().doc.kb.sub, m.tab().doc.kb.adding);
        m.up(far);
        assert!(
            m.tab().doc.pending.is_empty(),
            "{what}: o Up fora não dispara"
        );
        assert_eq!(
            (m.tab().doc.kb.sub, m.tab().doc.kb.adding),
            before,
            "{what}: nada mudou"
        );
    }
}

#[test]
fn tab_chega_a_cada_controlo_com_o_anel_e_enter_activa() {
    let mut m = mesa_b();
    let want = KbHit::Sub(Sub::Gamepad).id();
    let mut reached = false;
    for _ in 0..400 {
        m.key(DocKey::Tab, None);
        if m.tab().doc.is_focused(want) {
            reached = true;
            break;
        }
    }
    assert!(reached, "o Tab chega ao segmento Gamepad");
    m.key(DocKey::Enter, None);
    assert_eq!(m.tab().doc.kb.sub, Sub::Gamepad, "Enter activa o foco");
}

#[test]
fn o_remove_sem_accao_aberta_fica_desligado_e_diz_porque() {
    let mut m = Mesa::aberta(View::Keyboard);
    m.hero.input_map = ph2d_input::InputMap::default();
    m.frame();
    let id = KbHit::Remove.id();
    let at = m.at(id);
    m.hover(at);
    let doc = &m.tab().doc;
    assert_eq!(
        doc.look_at(at.0, at.1),
        Some(PointerLook::Arrow),
        "desligado: seta"
    );
    assert_eq!(
        doc.disabled_why_here().map(|w| w.1),
        Some("input_map_keyboard.remove_off_none")
    );
    m.tab_mut().doc.pending.clear();
    m.click(at);
    assert!(
        !m.tab()
            .doc
            .pending
            .iter()
            .any(|e| matches!(e, InputMapEdit::Remove(_))),
        "o desligado não dispara"
    );
}

#[test]
fn clicar_no_remove_com_accao_aberta_apaga_a_aberta() {
    let mut m = mesa_b();
    let open = m.tab().doc.kb.open.expect("aberta");
    let n = m.hero.input_map.actions().len();
    m.click(m.at(KbHit::Remove.id()));
    // O quadro da família aplica o pedido: a aberta sai do mapa.
    assert!(m.hero.input_map.get(open).is_none(), "a aberta ficou");
    assert_eq!(m.hero.input_map.actions().len(), n - 1);
}

// ── Os números Dead/Press da B: o campo de `field.rs`, como a A ──────────────────────────────────────────────────

fn key_mods(m: &mut Mesa, key: DocKey, mods: ph2d_host::Modifiers) {
    let input = ph2d_editor_core::doc_surface::KeyInput {
        key,
        mods,
        pressed: true,
        text: None,
        paste: None,
    };
    let ts = &mut m.ts;
    if let Some((f, id)) = m.hero.documents.active_surface_mut() {
        let _ = f.key(id, ts, input);
    }
    m.frame();
}

/// A B com uma acção de manche aberta (a barra do curso e os dois números).
fn b_com_manche() -> (Mesa, ph2d_input::ActionId) {
    let mut m = Mesa::aberta(View::Keyboard);
    let mut map = ph2d_input::InputMap::new();
    let a = map.create("steer");
    if let Some(act) = map.get_mut(a) {
        act.bindings.push(ph2d_input::Binding::PadAxis {
            axis: ph2d_input::GamepadAxis::LeftStickX,
            positive: true,
        });
        act.set_zone(0.2, 0.5);
    }
    m.hero.input_map = map;
    m.frame();
    (m, a)
}

#[test]
fn os_numeros_da_b_editam_como_os_da_a() {
    use ph2d_app_input_map::ids::ListHit;
    let (mut m, a) = b_com_manche();
    let open = |m: &Mesa| m.tab().doc.list.typing.as_ref().map(|t| t.buffer.clone());
    let dead = |m: &Mesa| m.hero.input_map.get(a).expect("a").dead_zone;
    let id = ListHit::Dead(a).id();
    let at = m.at(id);
    assert_eq!(
        m.tab().doc.look_at(at.0, at.1),
        Some(PointerLook::Text),
        "cursor de texto"
    );
    let node = m
        .tab()
        .doc
        .a11y()
        .into_iter()
        .find(|(n, _)| *n == id)
        .expect("nó")
        .1;
    assert_eq!(node.role(), ph2d_a11y::Role::SpinButton);
    m.click(at);
    assert_eq!(open(&m).as_deref(), Some("0.20"), "abre com o valor");
    assert!(
        m.tab().doc.list.caret.selection().is_some(),
        "tudo seleccionado"
    );
    assert!(
        m.tab().doc.wants_frame(),
        "o caret vivo pede quadro (pisca)"
    );
    m.key(DocKey::Up, None);
    assert_eq!(open(&m).as_deref(), Some("0.21"), "↑ um passo");
    let shift = ph2d_host::Modifiers {
        shift: true,
        ..Default::default()
    };
    key_mods(&mut m, DocKey::Up, shift);
    assert_eq!(open(&m).as_deref(), Some("0.31"), "Shift+↑ ×10");
    let alt = ph2d_host::Modifiers {
        alt: true,
        ..Default::default()
    };
    key_mods(&mut m, DocKey::Down, alt);
    assert_eq!(open(&m).as_deref(), Some("0.309"), "Alt+↓ ×0,1");
    assert_eq!(dead(&m), 0.2, "o passo ainda não aplica");
    m.key(DocKey::Escape, None);
    assert_eq!((open(&m), dead(&m)), (None, 0.2), "Esc devolve");
    m.click(m.at(id));
    m.key(DocKey::Up, None);
    m.key(DocKey::Enter, None);
    assert_eq!(open(&m), None, "Enter aplica e larga");
    assert!((dead(&m) - 0.21).abs() < 1e-5, "Enter aplica: {}", dead(&m));
    // Tab aplica e segue para o Press; o limite diz a frase no lugar do rótulo.
    m.click(m.at(id));
    m.key(DocKey::Tab, None);
    assert_eq!(
        open(&m).as_deref(),
        Some("0.50"),
        "Tab chega ao Press aberto"
    );
    m.type_text("7");
    m.key(DocKey::Enter, None);
    assert_eq!(m.hero.input_map.get(a).expect("a").press_point, 1.0);
    let all: String = m
        .tab()
        .doc
        .census
        .texts
        .iter()
        .map(|t| t.1.clone())
        .collect();
    assert!(
        all.contains(ph2d_i18n::tr("input_map_list.kept_max")),
        "a frase do limite: {all}"
    );
}

#[test]
fn o_tab_abre_o_numero_da_b() {
    use ph2d_app_input_map::ids::ListHit;
    let (mut m, a) = b_com_manche();
    let mut opened = false;
    for _ in 0..200 {
        m.key(DocKey::Tab, None);
        if m.tab().doc.list.typing.is_some() {
            opened = true;
            break;
        }
    }
    assert!(opened, "o Tab abre um número da B");
    let t = m.tab().doc.list.typing.as_ref().expect("aberto");
    assert_eq!(t.action, a);
    let _ = ListHit::Dead(a);
}

#[test]
fn o_tab_chega_ao_x_do_ladrilho_e_enter_tira_a_ligacao() {
    let mut m = mesa_b();
    let open = m.tab().doc.kb.open.expect("aberta");
    let n = m.hero.input_map.get(open).expect("a").bindings.len();
    assert!(n > 0, "a aberta tem ligações");
    let want = KbHit::TileX(0).id();
    let mut reached = false;
    for _ in 0..400 {
        m.key(DocKey::Tab, None);
        if m.tab().doc.is_focused(want) {
            reached = true;
            break;
        }
    }
    assert!(reached, "o Tab chega ao × do 1.º ladrilho");
    let node = m
        .tab()
        .doc
        .a11y()
        .into_iter()
        .find(|(i, _)| *i == want)
        .expect("nó")
        .1;
    assert_eq!(node.role(), ph2d_a11y::Role::Button);
    m.key(DocKey::Enter, None);
    assert_eq!(
        m.hero.input_map.get(open).expect("a").bindings.len(),
        n - 1,
        "Enter tira a ligação"
    );
}

#[test]
fn o_segmento_gamepad_escolhido_diz_se_seleccionado() {
    let mut m = mesa_b();
    m.click(m.at(KbHit::Sub(Sub::Gamepad).id()));
    let sel = |m: &Mesa, s: Sub| {
        let id = KbHit::Sub(s).id();
        m.tab()
            .doc
            .a11y()
            .into_iter()
            .find(|(i, _)| *i == id)
            .expect("nó")
            .1
            .is_selected()
    };
    assert_eq!(sel(&m, Sub::Gamepad), Some(true), "Gamepad seleccionado");
    assert_ne!(sel(&m, Sub::Keyboard), Some(true), "Keyboard não");
}

#[test]
fn a_accao_recem_aberta_rola_para_a_vista_a_1280() {
    let area = AREAS[1];
    let mut m = mesa_b();
    m.paint_in(area);
    m.tab_mut().doc.wheel(f32::MIN);
    m.paint_in(area);
    let first = m.hero.input_map.actions()[0].clone();
    assert!(m.tab().doc.kb.scroll > 0.0, "a região rolou para o fundo");
    m.tab_mut().doc.kb.select_name = Some(first.name.clone());
    m.paint_in(area);
    let doc = &m.tab().doc;
    let view = doc.census.clip.expect("a região rola");
    let r = doc.rect_of(KbHit::Row(first.id).id()).expect("linha");
    assert!(inside(r, view), "a linha aberta {r:?} à vista em {view:?}");
}

/// R4: o número da B abre no DOWN, todo seleccionado (como na A); a alça D/P continua a arrastar.
#[test]
fn o_numero_da_b_foca_no_down() {
    use ph2d_app_input_map::ids::ListHit;
    let (mut m, a) = b_com_manche();
    let id = ListHit::Press(a).id();
    m.down(m.at(id));
    let open = m.tab().doc.list.typing.as_ref().map(|t| t.buffer.clone());
    assert_eq!(open.as_deref(), Some("0.50"), "aberto no Down");
    assert_eq!(
        m.tab().doc.list.caret.selection(),
        Some((0, 4)),
        "tudo seleccionado"
    );
    m.up(m.at(id));
    assert!(m.tab().doc.list.typing.is_some(), "o Up mantém-no aberto");
}

fn meets(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// F1: a acção aberta por uma PORTA (o smoke da B4 abre o reload) rola-se para a vista já na 1.ª pintura; a barra
/// mora na calha reservada (nenhum texto nela, dentro da região), a borda cortada desvanece e os chips da acção
/// aberta têm UM token em repouso — nas duas áreas reais, nos dois modos de texto.
#[test]
fn f1_a_porta_rola_a_aberta_e_a_barra_nao_tapa_texto() {
    super::em_cada_modo_de_texto(|| {
        for area in AREAS {
            for name in ["reload", "fire", "laser", "move_left"] {
                let mut m = mesa_b();
                let id = m.hero.input_map.id(name).expect("a acção da cena");
                m.tab_mut().doc.smoke_open(Some(id));
                m.paint_in(area);
                let doc = &m.tab().doc;
                let row = doc.rect_of(KbHit::Row(id).id()).expect("a linha pintada");
                let view = doc.census.clip.unwrap_or(area);
                assert!(
                    inside(row, view),
                    "{}: {name} à vista ({row:?} em {view:?})",
                    area.w
                );
                for (thumb, region, gutter) in &doc.census.scrollbars {
                    assert!(
                        inside(*thumb, *region),
                        "{}: o polegar dentro da região",
                        area.w
                    );
                    assert!(inside(*thumb, *gutter), "{}: o polegar na calha", area.w);
                    for (r, s, _) in &doc.census.texts {
                        let shown = inside(*r, *region) || meets(*r, *region);
                        assert!(
                            !(shown && meets(*r, *gutter)),
                            "{}: «{s}» na calha da barra",
                            area.w
                        );
                    }
                    assert!(
                        !doc.census.fades.is_empty(),
                        "{}: a borda cortada desvanece",
                        area.w
                    );
                }
                let toks: Vec<_> = doc.census.chips.iter().map(|c| c.1).collect();
                assert!(
                    toks.windows(2).all(|w| w[0] == w[1]),
                    "{}: {name} chips {toks:?}",
                    area.w
                );
            }
        }
    });
}

/// F1: a escuta sem ligação centra a frase na caixa; ao lado de ladrilhos, a frase fica na linha das legendas. O
/// corpo é Regular e o título da acção e os nomes da lista SemiBold.
#[test]
fn f1_a_escuta_alinha_e_o_corpo_e_regular() {
    super::em_cada_modo_de_texto(|| {
        for area in AREAS {
            for (name, empty) in [("reload", true), ("fire", false)] {
                let mut m = mesa_b();
                let id = m.hero.input_map.id(name).expect("a acção");
                m.tab_mut().doc.smoke_open(Some(id));
                m.paint_in(area);
                let doc = &m.tab().doc;
                let slot = (doc.rect_of(KbHit::Listen.id()))
                    .unwrap_or_else(|| panic!("{}: a escuta de {name}", area.w));
                let msg = (doc.census.texts.iter())
                    .find(|(r, _, _)| inside(*r, slot))
                    .expect("a frase da escuta");
                if empty {
                    let (a, b) = (msg.0.y - slot.y, slot.y + slot.h - msg.0.y - msg.0.h);
                    assert!((a - b).abs() < 0.51, "{}: centrada ({a} × {b})", area.w);
                } else {
                    let cap = (doc.census.texts.iter())
                        .find(|(r, s, _)| s == "Keyboard" && r.x < slot.x)
                        .expect("a legenda do ladrilho");
                    assert!(
                        (cap.0.y - msg.0.y).abs() < 0.01,
                        "{}: a mesma linha",
                        area.w
                    );
                    assert!(
                        doc.census.regular.iter().any(|s| s == "Keyboard"),
                        "legenda Regular"
                    );
                }
                assert!(
                    doc.census.semibold.iter().any(|s| s == name),
                    "{name}: o nome SemiBold"
                );
                assert!(
                    doc.census.semibold.iter().any(|s| s.starts_with(name)),
                    "o título SemiBold"
                );
            }
        }
    });
}

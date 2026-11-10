//! W3–W5 (onda 2): o inventário controlo × estado — hover, premido, desactivado, foco, cursor, movimento e
//! acessibilidade; cada célula é um teste que dirige o evento real e afirma a pintura. Plano: §13.3.

use ph2d_app_input_map::interact::{Control, ControlRole};
use ph2d_app_input_map::{TAB_ID, View, ids};
use ph2d_editor_core::doc_surface::{DocumentSurface, PointerLook};

use super::Mesa;

/// ⭐ O BUG (INFRA-1): o seletor pinta pelo `CardCtx`, que lê `hover_live(id) > 0`; a aba nunca o alimentava e um
/// id nunca visto lê `SETTLED = 1.0` — os segmentos não escolhidos pintavam-se SEMPRE em hover. Agora: frio sem
/// o ponteiro, aceso NO QUADRO em que ele entra e apagado no quadro em que sai (D32, sem fade).
#[test]
fn o_seletor_so_acende_sob_o_ponteiro_e_no_quadro() {
    let mut m = Mesa::aberta(View::Keyboard);
    for v in View::ALL {
        assert_eq!(m.tab().doc.hover_live(v.id()), 0.0, "{v:?} sem o ponteiro");
    }
    let list = m.segment(View::List);
    m.hover(list);
    assert_eq!(
        m.tab().doc.hover_live(ids::VIEW_LIST),
        1.0,
        "acende no quadro"
    );
    assert_eq!(m.tab().doc.hover_live(ids::VIEW_TRY), 0.0);
    m.hover((list.0, list.1 + 300.0)); // LITERAL-PX-OK: para baixo do seletor
    assert_eq!(
        m.tab().doc.hover_live(ids::VIEW_LIST),
        0.0,
        "apaga no quadro"
    );
}

/// O premido nativo: o Down captura e não troca a vista; arrastar para fora apaga o premido; o Up dentro dispara;
/// o Up fora cancela.
#[test]
fn o_segmento_dispara_no_up_dentro_e_cancela_fora() {
    let mut m = Mesa::aberta(View::Keyboard);
    let list = m.segment(View::List);
    let fora = (list.0, list.1 + 300.0); // LITERAL-PX-OK: para baixo do seletor
    assert!(m.down(list), "o Down é tomado");
    assert_eq!(m.tab().doc.view(), View::Keyboard, "o Down não dispara");
    assert!(m.tab().doc.is_pressed(ids::VIEW_LIST));
    m.hover(fora);
    assert!(
        !m.tab().doc.is_pressed(ids::VIEW_LIST),
        "fora apaga o premido"
    );
    m.up(fora);
    assert_eq!(m.tab().doc.view(), View::Keyboard, "o Up fora cancela");
    m.down(list);
    m.hover(fora);
    m.hover(list);
    assert!(
        m.tab().doc.is_pressed(ids::VIEW_LIST),
        "de volta, premido outra vez"
    );
    m.up(list);
    assert_eq!(m.tab().doc.view(), View::List, "o Up dentro dispara");
    assert!(m.click(m.segment(View::TryIt)));
    assert_eq!(m.tab().doc.view(), View::TryIt, "o clique inteiro troca");
}

/// O foco: Tab / Shift+Tab percorrem os segmentos pela ordem da pintura, Enter activa, Esc larga.
#[test]
fn tab_percorre_o_seletor_enter_activa_e_esc_larga() {
    let mut m = Mesa::aberta(View::Keyboard);
    let k = ph2d_editor_core::doc_surface::Key::Tab;
    assert!(m.key(k, None), "o Tab é tomado");
    assert_eq!(
        m.tab().doc.focused(),
        Some(ids::VIEW_LIST),
        "o primeiro da pintura"
    );
    m.key(k, None);
    assert_eq!(m.tab().doc.focused(), Some(ids::VIEW_KEYBOARD));
    let named = |n: &str| ph2d_input::Key::named_in_script(n).expect("no teclado")[0];
    m.physical(named("left_shift"));
    m.physical(named("tab"));
    assert_eq!(
        m.tab().doc.focused(),
        Some(ids::VIEW_LIST),
        "Shift+Tab volta"
    );
    m.key(ph2d_editor_core::doc_surface::Key::Enter, None);
    assert_eq!(m.tab().doc.view(), View::List, "Enter activa o focado");
    assert!(m.key(ph2d_editor_core::doc_surface::Key::Escape, None));
    assert_eq!(m.tab().doc.focused(), None, "Esc larga");
}

/// A acessibilidade e o cursor: um nó `Tab` por segmento (o da vista escolhida, seleccionado) e a mão sobre eles.
#[test]
fn o_seletor_tem_nos_de_tab_e_a_mao() {
    let m = Mesa::aberta(View::Keyboard);
    let nodes = m.tab().a11y(TAB_ID);
    assert_eq!(nodes[0].1.role(), ph2d_a11y::Role::Document);
    for v in View::ALL {
        let (_, n) = nodes
            .iter()
            .find(|(id, _)| *id == v.id())
            .expect("o nó do segmento");
        assert_eq!(n.role(), ph2d_a11y::Role::Tab, "{v:?}");
        assert_eq!(n.label(), Some(ph2d_i18n::tr(v.label_key())));
        assert_eq!(
            n.is_selected(),
            (v == View::Keyboard).then_some(true),
            "{v:?}"
        );
    }
    let (x, y) = m.segment(View::List);
    assert_eq!(m.tab().pointer_look(TAB_ID, x, y), Some(PointerLook::Hand));
    assert!(!m.tab().wants_frame(), "em repouso, nenhum quadro pedido");
}

/// Um controlo desligado: o Down não captura, o Up não dispara, o cursor é a seta.
#[test]
fn um_controlo_desligado_nao_dispara_e_mostra_a_seta() {
    let mut m = Mesa::aberta(View::Keyboard);
    let r = m.tab().doc.segment_rect(View::List).expect("pintado");
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let off = Control::new(ids::VIEW_LIST, r, ControlRole::Segment, "List")
        .disabled(Some("input_map_tab.view.list_tip"));
    let doc = &mut m.tab_mut().doc;
    doc.control(off);
    assert!(doc.press(x, y), "dentro da aba");
    assert!(!doc.is_pressed(ids::VIEW_LIST), "desligado não captura");
    doc.release_at(x, y);
    assert_eq!(doc.view(), View::Keyboard, "nem dispara");
    assert_eq!(doc.look_at(x, y), Some(PointerLook::Arrow));
}

/// O relógio do substrato: o caret pisca pelo `UiMotion::elapsed` e fica fixo com reduced motion; o pulso é
/// `Role::Decoration` — ausente em Discreto e no reduced motion, presente no Expressivo (e pede quadro).
#[test]
fn caret_e_pulso_seguem_o_relogio_e_o_reduced_motion() {
    let mut m = Mesa::aberta(View::Keyboard);
    let id = ids::SEARCH;
    m.hero
        .motion
        .set_character(ph2d_editor_core::motion::UiCharacter::Discrete);
    m.frame();
    assert_eq!(m.tab_mut().doc.pulse(id), None, "Discreto: sem enfeite");
    m.hero
        .motion
        .set_character(ph2d_editor_core::motion::UiCharacter::Expressive);
    m.frame();
    assert!(
        m.tab_mut().doc.pulse(id).is_some(),
        "Expressivo: o pulso existe"
    );
    assert!(m.tab().doc.wants_frame(), "um pulso vivo pede quadro");
    let a = m.tab_mut().doc.caret_on(id);
    m.hero.motion.advance(0.6);
    m.frame();
    assert_ne!(
        m.tab_mut().doc.caret_on(id),
        a,
        "meio período depois, o caret trocou"
    );
    m.hero.motion.set_reduced_motion(true);
    m.frame();
    assert_eq!(m.tab_mut().doc.pulse(id), None, "reduced: sem pulso");
    assert!(m.tab_mut().doc.caret_on(id), "reduced: caret fixo, aceso");
}

/// O Add com o nome vazio fica DESLIGADO: o clique não cria nada, a dica diz porquê, o cursor é a seta e o nó
/// a11y está desligado; com um nome novo, liga e o clique (Up dentro) cria a acção.
#[test]
fn o_add_vazio_esta_desligado_e_diz_porque() {
    let mut m = Mesa::aberta(View::List);
    let add = m.at(ids::ADD);
    m.hover(add);
    assert_eq!(
        m.tab().doc.disabled_why_here().map(|w| w.1),
        Some("input_map_list.add_off_empty")
    );
    assert_eq!(m.tab().doc.look_at(add.0, add.1), Some(PointerLook::Arrow));
    m.click(add);
    assert!(
        m.tab_mut().doc.take_pending().is_empty(),
        "desligado não dispara"
    );
    m.click(m.at(ids::NEW_NAME));
    m.type_text("taunt");
    m.down(add);
    assert!(m.tab().doc.is_pressed(ids::ADD), "premido à vista");
    m.up(add);
    assert!(m.hero.input_map.id("taunt").is_some(), "o Up dentro cria");
}

/// O número da zona: o foco selecciona tudo; ↑ um passo, Shift+↑ dez, Alt+↑ um décimo; Esc devolve; Enter
/// aplica UMA vez; fora de 0..1 fica no limite com a frase, que some na edição válida seguinte.
#[test]
fn o_numero_da_zona_edita_como_um_campo_pro() {
    use ph2d_app_input_map::ids::ListHit;
    use ph2d_editor_core::doc_surface::Key as K;
    let mut m = Mesa::aberta(View::List);
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
    let dead0 = m.hero.input_map.get(a).expect("a").dead_zone;
    m.click(m.at(ListHit::Dead(a).id()));
    let open = |m: &Mesa| m.tab().doc.list.typing.as_ref().map(|t| t.buffer.clone());
    assert_eq!(open(&m), Some(format!("{dead0:.2}")));
    assert!(
        m.tab().doc.list.caret.selection().is_some(),
        "tudo seleccionado"
    );
    m.key(K::Up, None);
    assert_eq!(open(&m).as_deref(), Some("0.21"), "↑ um passo");
    assert_eq!(
        m.hero.input_map.get(a).expect("a").dead_zone,
        dead0,
        "↑ ainda não aplica"
    );
    m.key(K::Escape, None);
    assert_eq!(open(&m), None, "Esc larga");
    assert_eq!(
        m.hero.input_map.get(a).expect("a").dead_zone,
        dead0,
        "Esc devolve"
    );
    m.click(m.at(ListHit::Dead(a).id()));
    m.type_text("1.5");
    m.key(K::Enter, None);
    // A zona morta prende ao que o mapa guarda (`MAX_DEAD_ZONE`) e o Press acompanha-a.
    assert_eq!(
        m.hero.input_map.get(a).expect("a").press_point,
        ph2d_input::MAX_DEAD_ZONE
    );
    assert_eq!(
        m.tab().doc.list.warn.map(|w| w.2.i18n_key()),
        Some("input_map_list.kept_max"),
        "a frase do limite"
    );
    m.click(m.at(ListHit::Press(a).id()));
    m.type_text("0.5");
    m.key(K::Tab, None);
    assert_eq!(m.tab().doc.list.warn, None, "a edição válida apaga a frase");
}

// ── R3-A2: o polimento pro da vista A ──────────────────────────────────────────────────────────────────────────

/// Uma tecla com modificadores pela porta `key` da aba, seguida do quadro.
fn key_mods(m: &mut Mesa, key: ph2d_editor_core::doc_surface::Key, mods: ph2d_host::Modifiers) {
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

/// A lista com uma acção de manche (os dois números) e uma de tecla (o chip ×, o lixo e o «+»).
fn mesa_com_manche() -> (Mesa, ph2d_input::ActionId, ph2d_input::ActionId) {
    let mut m = Mesa::aberta(View::List);
    let mut map = ph2d_input::InputMap::new();
    let a = map.create("steer");
    if let Some(act) = map.get_mut(a) {
        act.bindings.push(ph2d_input::Binding::PadAxis {
            axis: ph2d_input::GamepadAxis::LeftStickX,
            positive: true,
        });
        act.set_zone(0.2, 0.5);
    }
    let k = map.create("jump");
    if let Some(act) = map.get_mut(k) {
        act.bindings
            .push(ph2d_input::Binding::Key(ph2d_input::Key(u32::from('J'))));
    }
    m.hero.input_map = map;
    m.frame();
    (m, a, k)
}

/// O campo de texto foca no DOWN (não no Up) e o caret cai onde se carregou; arrastar selecciona; soltar acaba.
#[test]
fn o_campo_de_texto_foca_no_down_e_o_caret_cai_onde_se_carrega() {
    let mut m = Mesa::aberta(View::List);
    m.click(m.at(ids::NEW_NAME));
    m.type_text("taunt");
    let r = m.tab().doc.rect_of(ids::NEW_NAME).expect("pintado");
    let (esq, dir, meio_y) = (r.x + 1.0, r.x + r.w - 2.0, r.y + r.h * 0.5); // LITERAL-PX-OK: dentro da borda
    m.down(m.at(ids::SEARCH));
    assert_eq!(
        m.tab().doc.focus(),
        Some(ids::SEARCH),
        "a busca foca no Down"
    );
    m.up(m.at(ids::SEARCH));
    m.down((esq, meio_y));
    assert_eq!(
        m.tab().doc.focus(),
        Some(ids::NEW_NAME),
        "o nome foca no Down"
    );
    assert_eq!(
        m.tab().doc.field_caret(ids::NEW_NAME).map(|c| c.0),
        Some(0),
        "carregar antes da 1.ª letra põe o caret no início"
    );
    m.hover((dir, meio_y));
    assert_eq!(
        m.tab().doc.field_caret(ids::NEW_NAME).map(|c| (c.0, c.1)),
        Some((5, Some(0))),
        "arrastar selecciona do Down ao ponteiro"
    );
    m.up((dir, meio_y));
    assert_eq!(m.tab().doc.list.text_drag, None, "soltar acaba o arrasto");
    m.hover((esq, meio_y));
    assert_eq!(
        m.tab().doc.field_caret(ids::NEW_NAME).map(|c| c.0),
        Some(5),
        "sem botão, mover não mexe no caret"
    );
}

/// O caret do campo focado pisca pelo relógio do movimento (e pede quadro) e fica fixo, aceso, com reduced motion.
#[test]
fn o_caret_do_campo_pisca_e_fica_fixo_com_reduced_motion() {
    let mut m = Mesa::aberta(View::List);
    m.click(m.at(ids::NEW_NAME));
    let fase = |m: &Mesa| {
        m.tab()
            .doc
            .field_caret(ids::NEW_NAME)
            .map(|c| c.2)
            .expect("campo")
    };
    let a = fase(&m);
    assert!(m.tab().wants_frame(), "o caret vivo pede quadro");
    m.hero.motion.advance(0.6);
    m.frame();
    assert_ne!(fase(&m), a, "meio período depois, o caret trocou");
    m.hero.motion.advance(0.6);
    m.frame();
    assert_eq!(fase(&m), a, "e volta");
    m.hero.motion.set_reduced_motion(true);
    for _ in 0..3 {
        m.hero.motion.advance(0.6);
        m.frame();
        assert!(fase(&m), "reduced motion: fixo, aceso");
    }
    assert!(
        m.tab().doc.field_caret(ids::SEARCH).is_some_and(|c| c.2),
        "o campo sem foco não pisca"
    );
}

/// Shift+↑ dez passos, Alt+↑ um décimo; Tab do Dead vai ao Press e Shift+Tab volta; Esc devolve.
#[test]
fn o_numero_da_zona_passo_largo_fino_e_tab_entre_os_dois() {
    use ph2d_app_input_map::ids::ListHit;
    use ph2d_app_input_map::view_list::state::Zone;
    use ph2d_editor_core::doc_surface::Key as K;
    let (mut m, a, _) = mesa_com_manche();
    let shift = ph2d_host::Modifiers {
        shift: true,
        ..Default::default()
    };
    let alt = ph2d_host::Modifiers {
        alt: true,
        ..Default::default()
    };
    let open = |m: &Mesa| {
        m.tab()
            .doc
            .list
            .typing
            .as_ref()
            .map(|t| (t.zone, t.buffer.clone()))
    };
    m.click(m.at(ListHit::Dead(a).id()));
    key_mods(&mut m, K::Up, shift);
    assert_eq!(
        open(&m),
        Some((Zone::Dead, "0.30".into())),
        "Shift+↑ = dez passos"
    );
    m.key(K::Escape, None);
    assert_eq!(
        m.hero.input_map.get(a).expect("a").dead_zone,
        0.2,
        "Esc devolve"
    );
    m.click(m.at(ListHit::Dead(a).id()));
    key_mods(&mut m, K::Up, alt);
    assert_eq!(
        open(&m),
        Some((Zone::Dead, "0.201".into())),
        "Alt+↑ = um décimo"
    );
    m.key(K::Escape, None);
    m.click(m.at(ListHit::Dead(a).id()));
    m.key(K::Tab, None);
    assert_eq!(open(&m).map(|o| o.0), Some(Zone::Press), "Tab vai ao Press");
    key_mods(&mut m, K::Tab, shift);
    assert_eq!(
        open(&m).map(|o| o.0),
        Some(Zone::Dead),
        "Shift+Tab volta ao Dead"
    );
}

/// O hover acende SÓ o controlo pairado, no mesmo quadro.
#[test]
fn o_hover_acende_so_o_controlo_pairado_da_lista() {
    let (mut m, _, k) = mesa_com_manche();
    use ph2d_app_input_map::ids::ListHit;
    let alvos = [
        ids::NEW_NAME,
        ids::ADD,
        ids::SEARCH,
        ListHit::Listen(k).id(),
    ];
    for quem in alvos {
        m.hover(m.at(quem));
        for outro in alvos {
            let esperado = if outro == quem { 1.0 } else { 0.0 };
            assert_eq!(
                m.tab().doc.hover_live(outro),
                esperado,
                "pairado {quem:?}, lido {outro:?}"
            );
        }
    }
}

/// O premido à vista e o arrasto para fora que cancela, no × do chip, no lixo e no «+».
#[test]
fn chip_lixo_e_mais_premem_e_cancelam_ao_sair() {
    use ph2d_app_input_map::ids::ListHit;
    let (mut m, _, k) = mesa_com_manche();
    let row = m.at(ListHit::Row(k).id());
    for alvo in [
        ListHit::RemoveBinding(k, 0).id(),
        ListHit::Trash(k).id(),
        ListHit::Listen(k).id(),
    ] {
        m.hover(row);
        let at = m.at(alvo);
        m.down(at);
        assert!(m.tab().doc.is_pressed(alvo), "{alvo:?}: premido à vista");
        assert!(
            m.tab_mut().doc.take_pending().is_empty(),
            "o Down não dispara"
        );
        let fora = (at.0, at.1 + 400.0); // LITERAL-PX-OK: muito abaixo da linha
        m.hover(fora);
        assert!(
            !m.tab().doc.is_pressed(alvo),
            "{alvo:?}: fora apaga o premido"
        );
        m.up(fora);
        assert!(
            m.tab_mut().doc.take_pending().is_empty(),
            "{alvo:?}: o Up fora cancela"
        );
    }
    assert_eq!(
        m.hero.input_map.get(k).map(|a| a.bindings.len()),
        Some(1),
        "nada mudou"
    );
}

/// O cursor por espécie de controlo da A: texto, mão, ↔ no número, seta no Add desligado.
#[test]
fn o_cursor_por_especie_de_controlo_da_lista() {
    use ph2d_app_input_map::ids::ListHit;
    let (mut m, a, k) = mesa_com_manche();
    let look = |m: &Mesa, id| {
        let (x, y) = m.at(id);
        m.tab().doc.look_at(x, y)
    };
    assert_eq!(look(&m, ids::NEW_NAME), Some(PointerLook::Text));
    assert_eq!(look(&m, ids::SEARCH), Some(PointerLook::Text));
    assert_eq!(
        look(&m, ids::ADD),
        Some(PointerLook::Arrow),
        "Add desligado"
    );
    assert_eq!(
        look(&m, ListHit::Dead(a).id()),
        Some(PointerLook::Text),
        "o corpo escreve"
    );
    assert_eq!(
        look(&m, ListHit::DeadHandle(a).id()),
        Some(PointerLook::ResizeEw),
        "a alça arrasta"
    );
    assert_eq!(look(&m, ListHit::Listen(k).id()), Some(PointerLook::Hand));
    m.hover(m.at(ListHit::Row(k).id()));
    assert_eq!(look(&m, ListHit::Trash(k).id()), Some(PointerLook::Hand));
    assert_eq!(
        look(&m, ListHit::RemoveBinding(k, 0).id()),
        Some(PointerLook::Hand)
    );
}

/// A acessibilidade da A: um nó por controlo pintado, cada um com papel e nome.
#[test]
fn cada_controlo_da_lista_tem_no_com_papel_e_nome() {
    use ph2d_a11y::Role as R;
    use ph2d_app_input_map::ids::ListHit;
    let (m, a, k) = mesa_com_manche();
    let nodes = m.tab().a11y(TAB_ID);
    assert_eq!(nodes[0].1.role(), R::Document);
    for (id, n) in &nodes[1..] {
        assert!(n.label().is_some_and(|l| !l.is_empty()), "{id:?} sem nome");
        assert!(
            m.tab().doc.rect_of(*id).is_some(),
            "{id:?}: nó sem controlo pintado"
        );
    }
    let papel = |id| nodes.iter().find(|(o, _)| *o == id).map(|(_, n)| n.role());
    assert_eq!(papel(ids::NEW_NAME), Some(R::TextInput));
    assert_eq!(papel(ids::SEARCH), Some(R::TextInput));
    assert_eq!(papel(ids::ADD), Some(R::Button));
    assert_eq!(papel(ListHit::Dead(a).id()), Some(R::SpinButton));
    assert_eq!(papel(ListHit::Listen(k).id()), Some(R::Button));
    let ids_unicos: std::collections::BTreeSet<_> = nodes.iter().map(|(id, _)| *id).collect();
    assert_eq!(
        ids_unicos.len(),
        nodes.len(),
        "um nó por controlo, sem repetidos"
    );
}

// ── R4: o número foca no Down, o caret recomeça a cada tecla, duplo clique, campo rolado ────────────────────────

/// Foca o nome novo pelo Tab (sem Down: nenhum duplo clique a contar).
fn foca_o_nome_pelo_tab(m: &mut Mesa) {
    for _ in 0..64 {
        if m.tab().doc.focus() == Some(ids::NEW_NAME) {
            return;
        }
        m.key(ph2d_editor_core::doc_surface::Key::Tab, None);
    }
    panic!("o Tab não chegou ao nome");
}

/// O número da zona abre no DOWN, todo seleccionado (como o campo de texto); a alça continua a arrastar.
#[test]
fn o_numero_da_a_foca_no_down_e_a_alca_arrasta() {
    use ph2d_app_input_map::ids::ListHit;
    let (mut m, a, _) = mesa_com_manche();
    m.down(m.at(ListHit::Dead(a).id()));
    let t = m.tab().doc.list.typing.as_ref().map(|t| t.buffer.clone());
    assert_eq!(t.as_deref(), Some("0.20"), "aberto no Down, antes do Up");
    assert_eq!(
        m.tab().doc.list.caret.selection(),
        Some((0, 4)),
        "tudo seleccionado"
    );
    m.up(m.at(ListHit::Dead(a).id()));
    assert_eq!(
        m.tab().doc.list.caret.selection(),
        Some((0, 4)),
        "o Up não desfaz"
    );
    m.key(ph2d_editor_core::doc_surface::Key::Escape, None);
    m.down(m.at(ListHit::DeadHandle(a).id()));
    assert!(m.tab().doc.list.typing.is_none(), "a alça não abre o texto");
    assert!(m.tab().doc.gesture_in_progress(), "a alça arrasta");
    m.release();
}

/// O caret acende a cada tecla e o piscar recomeça dali, no campo de texto e no número.
#[test]
fn o_caret_recomeca_a_piscar_a_cada_tecla() {
    use ph2d_app_input_map::ids::ListHit;
    use ph2d_editor_core::doc_surface::Key as K;
    let (mut m, a, _) = mesa_com_manche();
    foca_o_nome_pelo_tab(&mut m);
    let on = |m: &Mesa| m.tab().doc.field_caret(ids::NEW_NAME).expect("campo").2;
    assert!(on(&m), "aceso ao focar");
    m.hero.motion.advance(0.6);
    m.frame();
    assert!(!on(&m), "meio período depois, apagado");
    m.type_text("x");
    assert!(on(&m), "a tecla acende-o já");
    m.hero.motion.advance(0.3);
    m.frame();
    assert!(on(&m), "e o período recomeçou da tecla");
    m.hero.motion.advance(0.3);
    m.frame();
    assert!(!on(&m), "um meio período depois da tecla, apaga");
    m.click(m.at(ListHit::Dead(a).id()));
    m.hero.motion.advance(0.6);
    m.frame();
    let id = ListHit::Dead(a).id();
    assert!(!m.tab_mut().doc.caret_on(id), "o do número também pisca");
    m.key(K::Up, None);
    assert!(m.tab_mut().doc.caret_on(id), "a tecla acende o do número");
}

/// O duplo clique no campo de texto selecciona tudo.
#[test]
fn o_duplo_clique_no_campo_selecciona_tudo() {
    let mut m = Mesa::aberta(View::List);
    foca_o_nome_pelo_tab(&mut m);
    m.type_text("taunt");
    let r = m.tab().doc.rect_of(ids::NEW_NAME).expect("pintado");
    let esq = (r.x + 1.0, r.y + r.h * 0.5); // LITERAL-PX-OK: dentro da borda
    m.click(esq);
    assert_eq!(
        m.tab().doc.field_caret(ids::NEW_NAME).map(|c| (c.0, c.1)),
        Some((0, None))
    );
    m.click(esq);
    assert_eq!(
        m.tab().doc.field_caret(ids::NEW_NAME).map(|c| (c.0, c.1)),
        Some((5, Some(0))),
        "o duplo clique selecciona tudo"
    );
}

/// Com o texto focado maior que a caixa, a linha desliza para o caret caber: o clique junto à ponta direita
/// cai junto ao FIM do texto (as fronteiras levam o deslize), não a meio.
#[test]
fn o_clique_no_campo_rolado_cai_onde_o_texto_esta() {
    let mut m = Mesa::aberta(View::List);
    foca_o_nome_pelo_tab(&mut m);
    let long = "w".repeat(400);
    m.type_text(&long);
    let r = m.tab().doc.rect_of(ids::NEW_NAME).expect("pintado");
    let pad = ph2d_editor_core::widget::field_pad_x();
    m.down((r.x + pad, r.y + r.h * 0.5));
    let first = m.tab().doc.field_caret(ids::NEW_NAME).expect("campo").0;
    assert!(
        first > 100,
        "a ponta esquerda mostra o meio do texto: {first}"
    );
    assert!(first < long.len(), "e não no fim: {first}");
    m.up((r.x + pad, r.y + r.h * 0.5));
}

/// F1 (A): o × da busca é um alvo de 24 px (a caixa de ícone do cartão) e o Add desligado tem a aresta do botão
/// desligado (lê-se como botão, não como campo).
#[test]
fn o_x_da_busca_tem_24_px_e_o_add_desligado_tem_aresta() {
    let mut m = Mesa::aberta(View::List);
    m.click(m.at(ids::SEARCH));
    m.type_text("jump");
    let doc = &m.tab().doc;
    let x = doc.rect_of(ids::SEARCH_CLEAR).expect("o × da busca");
    let alvo = 24.0; // LITERAL-PX-OK: o alvo mínimo do × (HEAD_ICON_BOX)
    assert!(x.w >= alvo - 0.01 && x.h >= alvo - 0.01, "o × tem {x:?}");
    let add = doc.rect_of(ids::ADD).expect("o Add");
    assert!(
        (doc.census.filled.iter())
            .any(|(r, t)| *r == add && *t == ph2d_tokens::ColorToken::BorderStrong),
        "o Add desligado com aresta: {add:?}"
    );
}

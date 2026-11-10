//! Provas de mutação da onda 2 do Input Map: o que sobreviveu à primeira corrida e o que o mata. A lei «uma tecla do
//! sistema nunca liga» mora em CINCO guardas em camadas (a escuta, o filtro `never_binds`, o `add` do clique, o
//! `apply` do mapa e o controlo desligado); cada uma, tirada sozinha, deixava as outras taparem o buraco — por isso
//! cada guarda tem de ser vista por um teste que NÃO passa pelas outras.

use super::Mesa;
use ph2d_app_input_map::View;
use ph2d_app_input_map::view_keyboard::geometry::drawn_keys;
use ph2d_app_input_map::view_keyboard::state::{KbHit, Sub};
use ph2d_input::Key;

/// ⭐ Lei 2 (o controlo desligado): cada tecla do SISTEMA desenhada na B é um controlo DESLIGADO (não clica, não entra
/// na ordem do Tab, o leitor de ecrã di-lo), e uma tecla do jogo é um controlo LIGADO. Sem isto, o `add` do clique era
/// a única barreira e o `disabled(system_owned)` podia cair sem ninguém ver.
#[test]
fn a_tecla_do_sistema_e_um_controlo_desligado_e_a_do_jogo_nao() {
    let mut m = Mesa::aberta(View::Keyboard);
    m.click(m.at(KbHit::Sub(Sub::Keyboard).id()));
    let mut sistema = 0;
    let mut jogo = 0;
    for k in drawn_keys() {
        let Some(r) = m.tab().doc.rect_of(KbHit::Key(k).id()) else {
            continue;
        };
        let c = m
            .tab()
            .doc
            .control_at(r.x + r.w * 0.5, r.y + r.h * 0.5)
            .expect("a tecla desenhada é um controlo");
        if k.system_owned().is_some() {
            sistema += 1;
            assert!(!c.enabled(), "{}: a do sistema está desligada", k.label());
        } else if k == Key::named_in_script("a").expect("a")[0] {
            jogo += 1;
            assert!(c.enabled(), "{}: a do jogo está ligada", k.label());
        }
    }
    assert!(sistema >= 3, "Fn, Windows e PrtSc desenhadas: {sistema}");
    assert_eq!(jogo, 1, "o controlo: a tecla A está ligada");
}

/// Os bytes RGBA de um token na tinta da lista, como o vello os põe nos dados de desenho.
fn bytes_de(t: ph2d_tokens::ColorToken) -> u32 {
    let c = ph2d_editor_core::widget::card_ink::CardInk::of_list(ph2d_tokens::Theme::default())
        .color(t)
        .to_rgba8();
    u32::from_le_bytes([c.r, c.g, c.b, c.a])
}

/// A pintura da aba activa numa cena nova: os dados de desenho (as cores dos pincéis) que ela empurrou.
fn dados_de_desenho(m: &mut Mesa) -> Vec<u32> {
    let mut scene = ph2d_vector::VectorScene::new();
    let ts = &mut m.ts;
    if let Some((f, id)) = m.hero.documents.active_surface_mut() {
        f.paint(
            id,
            super::AREA,
            &mut scene,
            ts,
            &ph2d_tokens::Theme::default(),
        );
    }
    scene.inner().encoding().draw_data.clone()
}

/// ⭐ Lei 4: o número FOCADO tem o anel de foco (`KanbanBlueBold`, o do campo do cartão) e em repouso não tem. A
/// pintura não deixa rasto no censo; lê-se na cor do pincel da cena.
#[test]
fn o_campo_numerico_focado_pinta_o_anel_e_em_repouso_nao() {
    use ph2d_app_input_map::ids::ListHit;
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = super::lista::onze();
    m.frame();
    let ring = bytes_de(ph2d_tokens::ColorToken::KanbanBlueBold);
    let tem = |d: &[u32]| d.contains(&ring);
    assert!(!tem(&dados_de_desenho(&mut m)), "em repouso: sem anel");
    let ml = m.hero.input_map.id("move_left").expect("move_left");
    let dead = m.at(ListHit::Dead(ml).id());
    m.click(dead);
    m.release();
    assert!(tem(&dados_de_desenho(&mut m)), "focado: o anel de foco");
}

/// ⭐ Lei 4: o Esc num número já ESCRITO devolve o valor do FOCO (0.15), não o texto escrito (0.7). Os gates de antes
/// carregavam Esc sem ter escrito nada — com o texto igual ao do foco, as duas leituras coincidem.
#[test]
fn o_esc_no_numero_escrito_devolve_o_valor_do_foco_nao_o_texto() {
    use ph2d_app_input_map::ids::ListHit;
    use ph2d_editor_core::doc_surface::Key as DocKey;
    let mut m = Mesa::aberta(View::List);
    m.hero.input_map = super::lista::onze();
    m.frame();
    let ml = m.hero.input_map.id("move_left").expect("move_left");
    let antes = m.hero.input_map.get(ml).expect("move_left").dead_zone;
    let dead = m.at(ListHit::Dead(ml).id());
    m.click(dead);
    m.release();
    for _ in 0..4 {
        m.key(DocKey::Backspace, None);
    }
    m.type_text("0.7");
    m.key(DocKey::Escape, None);
    let depois = m.hero.input_map.get(ml).expect("move_left").dead_zone;
    assert!(
        (depois - antes).abs() < 1e-4,
        "Esc devolve {antes}, ficou {depois}"
    );
}

/// ⭐ Lei 4: um valor EXACTAMENTE no limite que o mapa guarda (Dead 0,99, Press 1) é aceite sem aviso — o aviso é só
/// para o que o mapa de facto muda. (`v >= max` avisava «Kept at 0.99» sobre um 0.99 escrito.)
#[test]
fn o_valor_escrito_no_limite_exacto_nao_avisa() {
    use ph2d_app_input_map::ids::ListHit;
    use ph2d_editor_core::doc_surface::Key as DocKey;
    use ph2d_input::{Binding, GamepadAxis, InputMap};
    let mut m = Mesa::aberta(View::List);
    let mut map = InputMap::new();
    let a = map.create("steer");
    if let Some(act) = map.get_mut(a) {
        act.bindings.push(Binding::PadAxis {
            axis: GamepadAxis::LeftStickX,
            positive: true,
        });
        act.set_zone(0.2, 0.5);
    }
    m.hero.input_map = map;
    m.frame();
    for (id, typed) in [
        (ListHit::Dead(a).id(), "0.99"),
        (ListHit::Press(a).id(), "1"),
    ] {
        m.click(m.at(id));
        m.type_text(typed);
        m.key(DocKey::Enter, None);
        assert!(
            m.tab().doc.list.warn.is_none(),
            "«{typed}» no limite: sem aviso"
        );
        let act = m.hero.input_map.get(a).expect("a");
        let got = if typed == "1" {
            act.press_point
        } else {
            act.dead_zone
        };
        assert!(
            (got - typed.parse::<f32>().expect("n")).abs() < 1e-6,
            "«{typed}» ficou {got}"
        );
    }
}

/// ⭐ Lei 6: a 1280 (B empilhada) o Tab ROLA a região até ao controlo focado: o controlo que o recorte mostrava
/// pela metade fica INTEIRO na vista quando recebe o foco (sem isto o anel pintava-se só no que o recorte deixa).
#[test]
fn o_tab_rola_a_regiao_ate_ao_controlo_focado_inteiro() {
    use super::teclado_censo::AREAS;
    use ph2d_editor_core::doc_surface::Key as DocKey;
    let area = AREAS[1];
    let mut m = Mesa::aberta(View::Keyboard);
    m.hero.input_map = super::lista::onze();
    m.frame();
    m.click(m.at(KbHit::Sub(Sub::Gamepad).id()));
    let ml = m.hero.input_map.id("move_left").expect("move_left");
    m.click(m.at(KbHit::Row(ml).id()));
    m.paint_in(area);
    assert!(m.tab().doc.kb.stacked, "a 1280 a B empilha");
    let mut revelados = 0;
    for i in 0..300 {
        m.key(DocKey::Tab, None);
        m.paint_in(area);
        let doc = &m.tab().doc;
        let (Some(f), Some(clip)) = (doc.focused(), doc.census.clip) else {
            continue;
        };
        let Some(r) = doc.rect_of(f) else { continue };
        let na_coluna = r.x >= clip.x && r.x + r.w <= clip.x + clip.w;
        let toca = r.y + r.h > clip.y && r.y < clip.y + clip.h;
        if na_coluna && toca {
            revelados += 1;
            assert!(
                r.y >= clip.y - 0.5 && r.y + r.h <= clip.y + clip.h + 0.5,
                "Tab n.º {i}: o foco {f:?} {r:?} ficou pela metade no recorte {clip:?}"
            );
        }
    }
    assert!(revelados > 0, "o teste viu algum controlo da região");
}

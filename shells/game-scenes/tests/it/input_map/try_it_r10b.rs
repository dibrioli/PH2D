//! **R10b · C · Try it pela revisão**: C4 com a tela do jogo no TOPO e o resto por baixo (o «Opens with» e o
//! «Forget them» discretos); C3 com o ponto no destaque, o disco Dead mais claro e o texto ao centro do círculo;
//! C1 com ladrilhos compactos encostados à esquerda; C2 com o teclado encostado à esquerda; e o «Forget them» com a
//! tela do jogo aberta, que a deixa aberta e o relógio parado.

use super::Mesa;
use super::try_it_censo::cena;
use ph2d_app_input_map::ph2d_player_controls::{self as pc, GameId};
use ph2d_app_input_map::view_keyboard::state::KbHit;
use ph2d_app_input_map::view_try::state::TryHit;
use ph2d_app_input_map::view_try::stick::{DEAD_INK, DOT_INK, OUTER_INK};
use ph2d_app_input_map::{InputMapTab, View};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_input::recipes::Recipe;
use ph2d_input::{Binding, Key, PlayerKeys};
use ph2d_tokens::{ColorToken, Theme};

fn key(name: &str) -> Key {
    Key::named_in_script(name).expect("tecla com nome")[0]
}

fn text_rect(m: &Mesa, s: &str) -> Rect {
    (m.tab().doc.census.texts.iter())
        .find(|t| t.1 == s)
        .unwrap_or_else(|| panic!("«{s}» pintado"))
        .0
}

#[test]
fn c4_a_tela_do_jogo_vai_ao_topo_e_o_resto_desce_por_baixo_dela() {
    let m = cena("C4");
    let doc = &m.tab().doc;
    let label = text_rect(&m, tr("input_map_try.controls"));
    let at = |h: TryHit| doc.rect_of(h.id()).expect("o alvo pintado");
    let players = at(TryHit::PlayersCanChange);
    // A tela 16:9 inteira (16 campos de largura) fica entre o rótulo e a caixa.
    let screen_h = 28.0 * 16.0 * 9.0 / 16.0; // LITERAL-PX-OK: FIELD_H × 16 × 9/16, a tela da prévia
    assert!(
        players.y - (label.y + label.h) >= screen_h,
        "a caixa por baixo da tela: {players:?} × rótulo {label:?}"
    );
    assert!(
        (players.x - label.x).abs() < 1.0,
        "a caixa encosta à esquerda, sem coluna ao lado"
    );
    let shown: Vec<Rect> = (m.hero.input_map.actions().iter())
        .map(|a| at(TryHit::Shown(a.id)))
        .collect();
    let shown_bottom = shown.iter().map(|r| r.y + r.h).fold(0.0_f32, f32::max);
    assert!(shown.iter().all(|r| r.y > players.y), "as caixas por baixo");
    let open = at(TryHit::OpenKey);
    let forget = at(TryHit::Forget);
    assert!(open.y >= shown_bottom, "o «Opens with» depois das caixas");
    assert!(forget.y > open.y, "o «Forget them» na linha de baixo");
    assert!(
        open.h < 28.0, // LITERAL-PX-OK: FIELD_H — o chip discreto é mais baixo que um campo
        "o chip da tecla é discreto: {open:?}"
    );
    assert!(
        forget.x > label.x,
        "o «Forget them» é uma ligação no fim da frase"
    );
}

#[test]
fn c3_o_ponto_no_destaque_o_disco_dead_mais_claro_e_o_texto_ao_centro() {
    assert_eq!(DOT_INK, ColorToken::InputMapHighlight);
    let lum = |t: ColorToken| {
        let c = t.resolve(Theme::default());
        u32::from(c.r) * 299 + u32::from(c.g) * 587 + u32::from(c.b) * 114
    };
    assert!(
        lum(DEAD_INK) > lum(OUTER_INK),
        "o disco Dead mais claro que o círculo"
    );
    let m = cena("C3");
    let doc = &m.tab().doc;
    let label = text_rect(&m, tr("input_map_try.stick"));
    let cap = tr("input_map_try.stick_caption");
    // Só os textos do cartão da manche (à direita do rótulo dele e por baixo).
    let in_card = |t: &&(Rect, String, bool)| t.0.x >= label.x - 1.0 && t.0.y > label.y;
    let first = (doc.census.texts.iter())
        .filter(in_card)
        .find(|t| t.1.len() > 3 && cap.starts_with(t.1.as_str()))
        .expect("a legenda")
        .0;
    let gap = 6.0; // LITERAL-PX-OK: FIELD_ROW_GAP, entre o círculo e a legenda
    let circle_mid = (label.y + label.h + first.y - gap) * 0.5;
    let a = (m.hero.input_map.actions().iter())
        .find(|a| a.is_analog())
        .expect("a manche");
    let title = (doc.census.texts.iter())
        .filter(in_card)
        .find(|t| t.1.starts_with(&a.name) && t.1.len() > a.name.len())
        .expect("o título da manche")
        .0;
    let state = [tr("input_map_try.pressed"), tr("input_map_try.not_pressed")];
    let last = (doc.census.texts.iter())
        .rev()
        .find(|t| state.contains(&t.1.as_str()))
        .expect("a leitura")
        .0;
    let block_mid = (title.y + last.y + last.h) * 0.5;
    assert!(
        (block_mid - circle_mid).abs() <= 1.0,
        "o texto ao centro do círculo: {block_mid} × {circle_mid} (título {title:?}, leitura {last:?}, rótulo {label:?}, legenda {first:?})"
    );
}

#[test]
fn c1_os_ladrilhos_sao_compactos_e_encostam_a_esquerda() {
    let m = cena("C1");
    let doc = &m.tab().doc;
    let label = text_rect(&m, tr("input_map_try.start_from"));
    let tiles: Vec<Rect> = (Recipe::ALL.iter())
        .map(|r| doc.rect_of(TryHit::Recipe(*r).id()).expect("o ladrilho"))
        .collect();
    assert!((tiles[0].x - label.x).abs() < 0.5, "o primeiro à esquerda");
    for (i, t) in tiles.iter().enumerate() {
        assert!(
            (150.0..=170.0).contains(&t.w), // LITERAL-PX-OK: ~160 px, a maquete C1
            "ladrilho {i} compacto: {t:?}"
        );
        if i > 0 && (t.y - tiles[0].y).abs() < 0.5 {
            let gap = t.x - (tiles[i - 1].x + tiles[i - 1].w);
            assert!(gap > 0.0 && gap < 12.0, "lado a lado, sem esticar: {gap}"); // LITERAL-PX-OK: o vão
        }
    }
    let names: Vec<String> = Recipe::ALL
        .iter()
        .filter(|r| !r.actions().is_empty())
        .map(|r| r.actions()[0].name.to_owned())
        .collect();
    for n in names {
        assert!(
            !doc.census.semibold.iter().any(|s| s.starts_with(&n)),
            "a descrição «{n}…» no regular, não no negrito"
        );
    }
}

#[test]
fn c2_o_teclado_encosta_a_esquerda_do_cartao() {
    let m = cena("C2");
    let doc = &m.tab().doc;
    let q = doc.rect_of(KbHit::Key(key("q")).id()).expect("o Q");
    let label = text_rect(&m, tr("input_map_try.try_it"));
    let pad = 10.0; // LITERAL-PX-OK: CARD_PAD_X, o recuo do desenho
    assert!(
        (q.x - (label.x + pad)).abs() < 0.5,
        "o Q no recuo esquerdo do desenho: {q:?} × {label:?}"
    );
}

/// As linhas da shell depois do `frame` da família.
fn shell_forget(m: &mut Mesa) {
    let tab = m.hero.documents.surfaces_mut().find_mut::<InputMapTab>();
    if let Some(t) = tab {
        t.doc.forget_reaches(&mut m.jogador, &m.hero.input_map);
    }
}

#[test]
fn forget_com_a_tela_do_jogo_aberta_deixa_a_aberta_e_o_relogio_parado() {
    let dir = std::env::temp_dir().join(format!("ph2d-try-r10b-{}", std::process::id()));
    let mut m = Mesa::new(View::TryIt);
    m.tab_mut().doc.tryit.base = Some(dir.clone());
    m.sim.world_mut().insert_resource(GameId(10));
    m.hero.input_map.controls_mut().players_can_change_keys = true;
    let file = pc::player_file(&dir, 10);
    let mut keys = PlayerKeys::new();
    keys.set("jump", vec![Binding::Key(key("x"))]);
    pc::file::save(&file, &keys).expect("grava");
    m.menu();
    let project = m.hero.input_map.clone();
    m.sim.world_mut().insert_resource(ph2d_ecs::RunOnCopy);
    m.jogador.start(&project, Some(file.clone()));
    let opened = m.jogador.key(key("f1"), true, &project);
    assert_eq!(m.jogador.clock_after(opened, true), Some(false), "abriu e parou");
    assert!(m.jogador.screen().is_open());
    m.frame();
    m.click(m.at(TryHit::Forget.id()));
    shell_forget(&mut m);
    assert!(m.jogador.screen().is_open(), "a tela continua aberta");
    assert_eq!(
        *m.jogador.map(&m.hero.input_map),
        m.hero.input_map,
        "o mapa efectivo = o do projecto"
    );
    assert!(m.jogador.keys().is_empty(), "as trocas saíram");
    let closed = m.jogador.key(key("escape"), true, &project);
    assert_eq!(
        m.jogador.clock_after(closed, false),
        Some(true),
        "ao fechar, o relógio retoma: estava parado pela tela, não pelo Forget"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

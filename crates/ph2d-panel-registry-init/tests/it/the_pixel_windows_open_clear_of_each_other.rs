//! ⭐⭐ **A janela acessória que abre não cai sobre outra** e **a busca dos atalhos tem o «×»** (smoke
//! 2 do dono 08/11: *«Ao abrir Undo History ele apareceu sobre Slices»* · *«Falta um x para limpar a
//! busca de atalhos»*). A tela inteira pintada, um ponteiro real.

use bumpalo::Bump;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::cards::{borda, busca};
use ph2d_editor_core::screens::hero::{
    PANEL_PIXEL_HISTORY, PANEL_PIXEL_KEYS, PANEL_PIXEL_PREFS, PANEL_PIXEL_PREVIEW,
    PANEL_PIXEL_SLICES, paint_hero_screen,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId, ids};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_pixel_history::canal::{HistoryView, Row, publish_view};
use ph2d_pixel_core::{SliceId, StepState};

fn ptr(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        // Um segundo por gesto: dois cliques do mesmo teste nunca viram duplo clique.
        timestamp_ns: RELOGIO.with(|r| {
            r.set(r.get() + 1_000_000_000);
            r.get()
        }),
    }
}

thread_local! {
    static RELOGIO: std::cell::Cell<u128> = const { std::cell::Cell::new(0) };
}

fn assenta(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem) {
    // O ecrã do dono (1 930 × 1 040): é nele que as quatro cabem lado a lado.
    quadros(hero, ts, ECRA_DO_DONO, 30);
}

fn quadros(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem, ecra: (f32, f32), n: usize) {
    let vp = Rect::new(0.0, 0.0, ecra.0, ecra.1);
    for _ in 0..n {
        let mut scene = ph2d_vector::VectorScene::new();
        ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || paint_hero_screen(hero, vp, &mut scene, ts),
        );
    }
}

fn clica(hero: &mut HeroScreen, id: NodeId, nome: &str) {
    let r = hero
        .hit_index
        .rect_for(id)
        .unwrap_or_else(|| panic!("{nome} não foi pintado"));
    let arena = Bump::new();
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut ev = hero
        .handle_pointer(ptr(PointerKind::Down, x, y), &arena)
        .to_vec();
    ev.extend_from_slice(hero.handle_pointer(ptr(PointerKind::Up, x, y), &arena));
    assert!(
        ev.contains(&WidgetEvent::Click(id)),
        "{nome}: o ponteiro deu {ev:?}; em ({x},{y}) acerta {:?}",
        hero.hit_index.hit_with_rect(x, y)
    );
    for e in ev {
        hero.apply_event(e);
    }
}

const PASSOS: [&str; 4] = [
    "pixel.history.initial",
    "pixel.history.pencil",
    "pixel.history.paint_bucket",
    "pixel.history.draw",
];

fn publica_conteudo() {
    publish_view(HistoryView {
        rows: PASSOS
            .iter()
            .map(|&label| Row {
                label,
                state: StepState::Done,
            })
            .collect(),
        current: PASSOS.len() - 1,
        bytes: 5400,
    });
    use ph2d_panel_pixel_slices::canal::{self, Contexto, Linha, Vista};
    canal::publish_contexto(Contexto {
        com_sprite: true,
        na_mao: true,
        linhas: vec![Linha {
            id: SliceId(1),
            nome: "button".to_owned(),
            selecionada: true,
        }],
        vista: Some(Vista {
            nome: Some("button".to_owned()),
            bounds: [Some(4), Some(6), Some(20), Some(10)],
            ..Vista::default()
        }),
        transform: false,
        user: None,
    });
}

fn hero(ts: &mut ph2d_text::TextSystem, aberta: &'static str) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
    publica_conteudo();
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(aberta, true);
    assenta(&mut hero, ts);
    hero
}

const ECRA_DO_DONO: (f32, f32) = (1930.0, 1040.0);

const JANELAS: [(&str, NodeId); 4] = [
    (PANEL_PIXEL_SLICES, ids::PIXEL_SLICES_PANEL),
    (PANEL_PIXEL_HISTORY, ids::PIXEL_HISTORY_PANEL),
    (PANEL_PIXEL_KEYS, ids::PIXEL_KEYS_PANEL),
    (PANEL_PIXEL_PREFS, ids::PIXEL_PREFS_PANEL),
];

fn sobrepoe(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

fn nenhuma_sobre_outra(hero: &HeroScreen, quando: &str) {
    let abertas: Vec<(&str, Rect)> = JANELAS
        .iter()
        .filter_map(|&(p, n)| hero.store.panel_rect(n).map(|r| (p, r)))
        .collect();
    for (i, &(pa, a)) in abertas.iter().enumerate() {
        for &(pb, b) in &abertas[i + 1..] {
            assert!(
                !sobrepoe(a, b),
                "{quando}: {pa} {a:?} cai sobre {pb} {b:?}; todas {abertas:?}"
            );
        }
    }
}

/// Cada janela abre depois das outras já visíveis, em qualquer ordem de abertura, e nenhuma cai
/// sobre outra; a que se fecha pelo «×» e se reabre também não (o passo do dono: Undo History).
#[test]
fn a_janela_que_abre_nao_cai_sobre_outra() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for ordem in [[0, 1, 2, 3], [3, 2, 1, 0], [1, 0, 3, 2]] {
        let _ = ph2d_panel_registry_init::register_all_panels();
        publica_conteudo();
        let mut hero = HeroScreen::new(NodeId(1));
        assenta(&mut hero, &mut ts);
        for &k in &ordem {
            let (painel, node) = JANELAS[k];
            hero.panel_visibility.insert(painel, true);
            assenta(&mut hero, &mut ts);
            assert!(hero.store.panel_rect(node).is_some(), "{painel} não abriu");
            nenhuma_sobre_outra(&hero, &format!("{ordem:?}, abriu {painel}"));
        }
        let (painel, node) = JANELAS[1];
        clica(&mut hero, borda::fecho_de(node), painel);
        assenta(&mut hero, &mut ts);
        assert!(hero.store.panel_rect(node).is_none(), "o «×» não fechou");
        hero.panel_visibility.insert(painel, true);
        assenta(&mut hero, &mut ts);
        nenhuma_sobre_outra(&hero, &format!("{ordem:?}, reabriu {painel}"));
    }
}

/// ⭐ U6 (fecho P12): os obstáculos são TODAS as janelas no ecrã, não só as do «×» acessório — a
/// Preview (F7) aberta antes, no mesmo canto à esquerda da coluna, não fica tapada por nenhuma.
#[test]
fn a_janela_que_abre_nao_cai_sobre_a_preview() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    for &(painel, node) in &JANELAS {
        let _ = ph2d_panel_registry_init::register_all_panels();
        publica_conteudo();
        let mut hero = HeroScreen::new(NodeId(1));
        hero.panel_visibility.insert(PANEL_PIXEL_PREVIEW, true);
        assenta(&mut hero, &mut ts);
        let preview = hero
            .store
            .panel_rect(ids::PIXEL_PREVIEW_PANEL)
            .expect("a Preview abriu");
        hero.panel_visibility.insert(painel, true);
        assenta(&mut hero, &mut ts);
        let r = hero.store.panel_rect(node).expect("a janela abriu");
        assert!(
            !sobrepoe(r, preview),
            "{painel} {r:?} cai sobre a Preview {preview:?}"
        );
    }
}

/// O «×» da busca dos atalhos: só com texto; o clique esvazia e o campo fica com o teclado.
#[test]
fn a_busca_dos_atalhos_limpa_pelo_x() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero(&mut ts, PANEL_PIXEL_KEYS);
    let campo = ph2d_panel_pixel_keys::ids::SEARCH;
    let x = busca::limpa_de(campo);
    assert!(hero.hit_index.rect_for(x).is_none(), "vazia, sem «×»");
    clica(&mut hero, campo, "a busca");
    if let Some(InteractiveState::TextInput { text, caret, .. }) = hero.store.get_mut(campo) {
        "flip".clone_into(text);
        *caret = text.len();
    }
    assenta(&mut hero, &mut ts);
    let caixa = hero.hit_index.rect_for(campo).expect("a busca pintada");
    let r = hero.hit_index.rect_for(x).expect("com texto, o «×»");
    assert!(
        r.x >= caixa.x && r.x + r.w <= caixa.x + caixa.w + 0.5 && r.y >= caixa.y - 0.5,
        "o «×» {r:?} fica dentro da caixa {caixa:?}"
    );
    // U7: o texto (em repouso e a editar) pára antes do «×» — a reserva cobre a ponta inteira.
    let borda = caixa.x + caixa.w - (r.x + r.w);
    assert_eq!(
        busca::ponta(caixa, borda),
        r,
        "o «×» pintado é a ponta da régua"
    );
    assert!(
        busca::reserva(caixa, borda) >= caixa.x + caixa.w - r.x,
        "o texto desliza por baixo do «×» {r:?}"
    );
    clica(&mut hero, x, "o «×» da busca");
    assenta(&mut hero, &mut ts);
    assert_eq!(hero.store.text(campo), Some(""), "o «×» não esvaziou");
    assert_eq!(
        hero.store.focus_id(),
        Some(campo),
        "a busca perdeu o teclado"
    );
    assert!(
        hero.hit_index.rect_for(x).is_none(),
        "vazia outra vez, sem «×»"
    );
}

const ECRA_PEQUENO: (f32, f32) = (1024.0, 768.0);

/// As janelas da cena 25 e as outras do menu *Window*, todas a abrir no MESMO quadro.
const TODAS: [(&str, NodeId); 5] = [
    (PANEL_PIXEL_PREVIEW, ids::PIXEL_PREVIEW_PANEL),
    (PANEL_PIXEL_SLICES, ids::PIXEL_SLICES_PANEL),
    (PANEL_PIXEL_HISTORY, ids::PIXEL_HISTORY_PANEL),
    (PANEL_PIXEL_KEYS, ids::PIXEL_KEYS_PANEL),
    (PANEL_PIXEL_PREFS, ids::PIXEL_PREFS_PANEL),
];

/// O rect de `node` no ecrã `ecra`: inteiro dentro dele e com mais que o cabeçalho à vista.
fn no_ecra(hero: &HeroScreen, painel: &str, node: NodeId, ecra: (f32, f32), quando: &str) -> Rect {
    use ph2d_editor_core::panel::cards::banda::metrics as m;
    let r = hero
        .store
        .panel_rect(node)
        .unwrap_or_else(|| panic!("{quando}: {painel} marcada e sem rect"));
    assert!(
        r.x >= -0.5 && r.y >= -0.5 && r.x + r.w <= ecra.0 + 0.5 && r.y + r.h <= ecra.1 + 0.5,
        "{quando}: {painel} {r:?} sai do ecrã {ecra:?}"
    );
    assert!(
        r.w >= m::LIST_HEAD_H && r.h > m::LIST_HEAD_H,
        "{quando}: {painel} {r:?} não se vê"
    );
    if node == ids::PIXEL_PREVIEW_PANEL {
        return r; // o «×» da Preview é o do seu cabeçalho próprio
    }
    let hit = hero
        .hit_index
        .rect_for(borda::fecho_de(node))
        .unwrap_or_else(|| panic!("{quando}: o «×» de {painel} não foi pintado"));
    let (cx, cy) = (hit.x + hit.w * 0.5, hit.y + hit.h * 0.5);
    assert_eq!(
        hero.hit_index.hit_with_rect(cx, cy).map(|(id, _)| id),
        Some(borda::fecho_de(node)),
        "{quando}: o «×» de {painel} {hit:?} está tapado"
    );
    r
}

/// ⭐ F1 (fecho P12, ecrã de 1 024 × 768): no PRIMEIRO quadro, toda janela marcada no menu está no
/// ecrã, inteira, com o corpo à vista e o «×» por cima de tudo — a marca é a janela no ecrã.
#[test]
fn no_ecra_pequeno_toda_janela_marcada_esta_no_ecra_no_primeiro_quadro() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let _ = ph2d_panel_registry_init::register_all_panels();
    publica_conteudo();
    let mut hero = HeroScreen::new(NodeId(1));
    quadros(&mut hero, &mut ts, ECRA_PEQUENO, 30);
    for &(p, _) in &TODAS {
        hero.panel_visibility.insert(p, true);
    }
    quadros(&mut hero, &mut ts, ECRA_PEQUENO, 1);
    for &(p, n) in &TODAS {
        no_ecra(&hero, p, n, ECRA_PEQUENO, "1.º quadro");
    }
    quadros(&mut hero, &mut ts, ECRA_PEQUENO, 30);
    for &(p, n) in &TODAS {
        no_ecra(&hero, p, n, ECRA_PEQUENO, "assente");
    }
}

/// ⭐ F1: sem sítio livre do tamanho inteiro, a janela que (re)abre não tapa o título (nem o «×»)
/// de nenhuma outra — em cascata, sempre no ecrã.
#[test]
fn sem_sitio_livre_a_janela_reaberta_nao_tapa_o_titulo_de_outra() {
    use ph2d_editor_core::panel::cards::banda::metrics as m;
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let _ = ph2d_panel_registry_init::register_all_panels();
    publica_conteudo();
    let mut hero = HeroScreen::new(NodeId(1));
    quadros(&mut hero, &mut ts, ECRA_PEQUENO, 30);
    for &(p, _) in &TODAS {
        hero.panel_visibility.insert(p, true);
        quadros(&mut hero, &mut ts, ECRA_PEQUENO, 30);
    }
    let (painel, node) = TODAS[2];
    clica(&mut hero, borda::fecho_de(node), painel);
    quadros(&mut hero, &mut ts, ECRA_PEQUENO, 30);
    hero.panel_visibility.insert(painel, true);
    quadros(&mut hero, &mut ts, ECRA_PEQUENO, 30);
    let r = no_ecra(&hero, painel, node, ECRA_PEQUENO, "reaberta");
    for &(p, n) in &TODAS {
        if n == node {
            continue;
        }
        let o = no_ecra(&hero, p, n, ECRA_PEQUENO, "reaberta");
        let titulo = Rect::new(o.x, o.y, o.w, m::LIST_HEAD_H);
        assert!(
            !sobrepoe(r, titulo),
            "{painel} {r:?} tapa o título de {p} {o:?}"
        );
    }
}

/// ⭐ Smoke do dono 09/11 (*«ao abrir o app os painéis acessórios estão sobrepostos»*): a cena 25 pede
/// a Undo History e os Keyboard Shortcuts no MESMO quadro. Nenhuma das abertas no mesmo quadro cai
/// sobre outra — nem no 1.º quadro, nem assente.
#[test]
fn as_janelas_que_abrem_no_mesmo_quadro_nao_caem_umas_sobre_as_outras() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let pares: [&[usize]; 3] = [&[1, 2], &[0, 1, 2, 3], &[3, 2, 1, 0]];
    for abre in pares {
        let _ = ph2d_panel_registry_init::register_all_panels();
        publica_conteudo();
        let mut hero = HeroScreen::new(NodeId(1));
        assenta(&mut hero, &mut ts);
        for &k in abre {
            hero.panel_visibility.insert(JANELAS[k].0, true);
        }
        quadros(&mut hero, &mut ts, ECRA_DO_DONO, 1);
        nenhuma_sobre_outra(&hero, &format!("{abre:?} no 1.º quadro"));
        assenta(&mut hero, &mut ts);
        nenhuma_sobre_outra(&hero, &format!("{abre:?} assente"));
    }
}

/// ⭐ Smoke do dono 09/11 (*«a largura do painel ficou travada»*): no Pixel o lápis tomava o premir
/// na borda da coluna. A borda é chrome — `costura_fisica` a responde, com as janelas da cena 25
/// abertas — e a largura pedida chega ao painel PIXEL até ao piso.
#[test]
fn a_borda_da_coluna_pixel_e_chrome_e_estreita_o_painel() {
    use ph2d_editor_core::interaction::WidgetStore;
    use ph2d_editor_core::screens::hero::PANEL_PIXEL;
    use ph2d_editor_core::screens::layout::DockSide;
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let _ = ph2d_panel_registry_init::register_all_panels();
    publica_conteudo();
    let mut hero = HeroScreen::new(NodeId(1));
    for p in [PANEL_PIXEL, PANEL_PIXEL_HISTORY, PANEL_PIXEL_KEYS] {
        hero.panel_visibility.insert(p, true);
    }
    assenta(&mut hero, &mut ts);
    let antes = hero
        .store
        .panel_rect(ids::PIXEL_PANEL)
        .expect("o PIXEL abriu");
    let s = hero.last_layout.expect("layout").dock_seam(DockSide::Right);
    assert!(s.w > 0.0 && s.h > 0.0, "a coluna PIXEL sem borda {s:?}");
    for dy in [0.1_f32, 0.5, 0.9] {
        let p = (s.x + s.w * 0.5, s.y + s.h * dy);
        assert_eq!(
            hero.costura_fisica(p.0, p.1),
            Some(DockSide::Right),
            "{p:?} na borda {s:?} não é a borda"
        );
    }
    hero.store
        .set_dock_width(DockSide::Right, Some(WidgetStore::DOCK_W_MIN));
    assenta(&mut hero, &mut ts);
    let depois = hero.store.panel_rect(ids::PIXEL_PANEL).expect("o PIXEL");
    assert!(
        depois.w + 40.0 < antes.w,
        "o PIXEL não estreitou: {antes:?} → {depois:?}"
    );
}

/// ⭐ A coluna ① de ferramentas nunca fica tapada (foto `P12_fecho_04`, ecrã de 1 024 × 768: a Undo
/// History tapava metade dela, a Slices a coluna inteira). As janelas acessórias ficam à direita dela
/// como ficam à direita da lista docada — a regra da ④ (dono 02/11, F50) — e o «×» de cada uma
/// continua a clicar (F1): a cena 25 (Undo History e Keyboard Shortcuts no mesmo quadro) e todas.
#[test]
fn nenhuma_janela_tapa_a_coluna_de_ferramentas() {
    use ph2d_editor_core::icons::IconId;
    use ph2d_editor_core::interaction::{BarButton, BarFace, BarFamily, ToolBars};
    use ph2d_editor_core::screens::hero::{PANEL_PIXEL, PANEL_PIXEL_TILES};
    let b = |i: u64| BarButton {
        id: NodeId(0x7001_0000 + i),
        face: BarFace::Icon(IconId::Select),
        label: String::new(),
        on: false,
        family: BarFamily::Neutral,
    };
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let _ = ph2d_panel_registry_init::register_all_panels();
    publica_conteudo();
    let todas: Vec<(&str, NodeId)> = (TODAS.iter().copied())
        .chain([(PANEL_PIXEL_TILES, ids::PIXEL_TILES_PANEL)])
        .collect();
    let cena_25 = [JANELAS[1], JANELAS[2]];
    // A cena 25 e o menu Edit abrem pelo PEDIDO: a janela fica visível dentro do próprio pintor,
    // depois de a ordem z do quadro estar feita — o caminho que a foto `c25_1024` apanhou.
    for ecra in [ECRA_PEQUENO, ECRA_DO_DONO] {
        for (janelas, pelo_pedido) in [
            (&cena_25[..], true),
            (&cena_25[..], false),
            (&todas[..], false),
        ] {
            let mut hero = HeroScreen::new(NodeId(1));
            hero.panel_visibility.insert(PANEL_PIXEL, true);
            hero.store.set_tool_bars(Some(ToolBars {
                column: vec![(0..11).map(b).collect(), (11..19).map(b).collect()],
                ..ToolBars::default()
            }));
            quadros(&mut hero, &mut ts, ecra, 30);
            let coluna = hero
                .store
                .tool_column()
                .expect("a coluna ① não foi pintada");
            if pelo_pedido {
                ph2d_panel_pixel_history::canal::pede_abrir();
                ph2d_panel_pixel_keys::pede_abrir();
            } else {
                for &(p, _) in janelas {
                    hero.panel_visibility.insert(p, true);
                }
            }
            for (quando, n) in [("1.º quadro", 1), ("assente", 30)] {
                quadros(&mut hero, &mut ts, ecra, n);
                let quando = format!(
                    "ecrã {ecra:?}, {} janelas, pedido {pelo_pedido}, {quando}",
                    janelas.len()
                );
                for &(p, n) in janelas {
                    let r = no_ecra(&hero, p, n, ecra, &quando);
                    assert!(
                        !sobrepoe(r, coluna),
                        "{quando}: {p} {r:?} tapa a coluna ① {coluna:?}"
                    );
                }
            }
        }
    }
}

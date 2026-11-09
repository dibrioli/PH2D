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
    let vp = Rect::new(0.0, 0.0, ECRA_DO_DONO.0, ECRA_DO_DONO.1);
    for _ in 0..30 {
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

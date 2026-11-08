//! **A janela SLICES alarga-se pela borda de FORA, como a coluna do Inspector** (dono, 08/11: *«O
//! painel Slices e outros painéis acessórios devem ter a alcinha de redimensionamento como o
//! inspector»*). A tela inteira pintada, um ponteiro real: Down na faixa da esquerda, Move, Up.

use bumpalo::Bump;
use ph2d_editor_core::interaction::section_prefs_text;
use ph2d_editor_core::panel::cards::borda;
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, PANEL_PIXEL_SLICES, paint_hero_screen,
};
use ph2d_editor_core::screens::layout::{DOCK_SEAM_PX, DockSide};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId, ids as nucleo};
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_pixel_slices::canal::{self, Contexto, Linha, Vista};
use ph2d_pixel_core::SliceId;
use ph2d_tokens::PANEL_MIN_W_PX;

const JANELA: NodeId = nucleo::PIXEL_SLICES_PANEL;

fn ptr(kind: PointerKind, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        x,
        y,
        pressure: 1.0,
        kind,
        source: PointerSource::Mouse,
        button: PointerButton::Primary,
        timestamp_ns: 0,
    }
}

fn assenta(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem) {
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    for _ in 0..30 {
        let mut scene = ph2d_vector::VectorScene::new();
        ph2d_editor_core::aquecimento::com_passo_fixo(
            ph2d_editor_core::aquecimento::PASSO_DE_QUADRO_S,
            || paint_hero_screen(hero, vp, &mut scene, ts),
        );
    }
}

fn hero(ts: &mut ph2d_text::TextSystem) -> HeroScreen {
    let _ = ph2d_panel_registry_init::register_all_panels();
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
    let mut hero = HeroScreen::new(NodeId(1));
    hero.panel_visibility.insert(PANEL_PIXEL_SLICES, true);
    assenta(&mut hero, ts);
    hero
}

/// Arrasta a borda de fora `dx` px e devolve o rect da janela depois.
fn arrasta(hero: &mut HeroScreen, ts: &mut ph2d_text::TextSystem, dx: f32) -> Rect {
    let r = hero.store.panel_rect(JANELA).expect("a janela pintou-se");
    let (x, y) = (r.x + DOCK_SEAM_PX * 0.5, r.y + r.h * 0.5);
    let alvo = hero.hit_index.hit(x, y).expect("a faixa tem dono");
    assert!(
        borda::e_alca(&hero.store, alvo),
        "a faixa da esquerda não é a alça"
    );
    assert_eq!(
        hero.chrome_panel_at(x, y),
        Some(JANELA),
        "a alça é da janela"
    );
    let arena = Bump::new();
    let _ = hero.handle_pointer(ptr(PointerKind::Down, x, y), &arena);
    let _ = hero.handle_pointer(ptr(PointerKind::Move, x + dx * 0.5, y), &arena);
    let _ = hero.handle_pointer(ptr(PointerKind::Move, x + dx, y), &arena);
    let _ = hero.handle_pointer(ptr(PointerKind::Up, x + dx, y), &arena);
    assenta(hero, ts);
    hero.store.panel_rect(JANELA).expect("a janela continua")
}

#[test]
fn a_borda_de_fora_alarga_a_janela_e_a_direita_fica_presa() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero(&mut ts);
    let antes = hero.store.panel_rect(JANELA).expect("pintou");
    // A lista pinta-se um pouco para dentro do encaixe; a folga é a mesma a qualquer largura.
    let folga = PANEL_MIN_W_PX - antes.w;
    assert!(
        hero.store.accessory_width(JANELA).is_none(),
        "nasce na largura de fábrica"
    );
    let x_campo = hero
        .hit_index
        .rect_for(ph2d_panel_pixel_slices::ids::BOUNDS_X);

    let depois = arrasta(&mut hero, &mut ts, -150.0);
    assert_eq!(
        depois.w,
        antes.w + 150.0,
        "puxar 150 px para fora alarga 150"
    );
    assert_eq!(
        depois.x + depois.w,
        antes.x + antes.w,
        "a direita fica presa à coluna"
    );
    let x_novo = hero
        .hit_index
        .rect_for(ph2d_panel_pixel_slices::ids::BOUNDS_X);
    assert_ne!(x_campo, x_novo, "os cartões refluem para a largura nova");

    // Para dentro, o piso: a janela nunca fica mais estreita que o mínimo do painel.
    let estreita = arrasta(&mut hero, &mut ts, 400.0);
    assert_eq!(
        estreita.w + folga,
        PANEL_MIN_W_PX,
        "o piso é o mínimo do painel"
    );
    assert_eq!(estreita.x + estreita.w, antes.x + antes.w);

    // Para fora sem fim, o tecto: metade do vão à esquerda da coluna.
    let larga = arrasta(&mut hero, &mut ts, -4000.0);
    let area = hero.last_layout.expect("layout").draw_area;
    let coluna = hero.last_layout.expect("layout").inspector.x;
    assert_eq!(larga.w + folga, borda::tecto(area, coluna), "o tecto");
    assert!(larga.x >= area.x, "a janela não sai da área");
}

#[test]
fn a_largura_grava_se_e_volta() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero(&mut ts);
    let _ = arrasta(&mut hero, &mut ts, -100.0);
    let w = hero.store.accessory_width(JANELA).expect("escolheu");
    let texto = section_prefs_text(&hero.store);
    assert!(texto.contains("accessory_w."), "não gravou: {texto}");
    let mut outro = HeroScreen::new(NodeId(1));
    ph2d_editor_core::interaction::restore_section_prefs_text(&mut outro.store, &texto);
    assert_eq!(outro.store.accessory_width(JANELA), Some(w));
}

/// A costura da coluna do Inspector continua no sítio, e a alça nova não a cobre.
#[test]
fn a_costura_da_coluna_continua_a_responder() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero(&mut ts);
    let layout = hero.last_layout.expect("layout");
    let costura = layout.dock_seam(DockSide::Right);
    let p = (costura.x + costura.w * 0.5, costura.y + costura.h * 0.5);
    assert_eq!(layout.dock_seam_at(p), Some(DockSide::Right));
    let janela = arrasta(&mut hero, &mut ts, -120.0);
    assert!(
        janela.x + janela.w <= costura.x,
        "a janela invadiu a coluna"
    );
    let layout = hero.last_layout.expect("layout");
    assert_eq!(layout.dock_seam_at(p), Some(DockSide::Right));
}

/// ⭐ A largura é função da posição do ponteiro desde o Down: passar do tecto e voltar devolve a
/// borda para debaixo do ponteiro (antes, o excesso somado ao limitado escorregava).
#[test]
fn passar_do_tecto_e_voltar_nao_escorrega() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero(&mut ts);
    let antes = hero.store.panel_rect(JANELA).expect("pintou");
    let (x, y) = (antes.x + DOCK_SEAM_PX * 0.5, antes.y + antes.h * 0.5);
    let arena = Bump::new();
    let _ = hero.handle_pointer(ptr(PointerKind::Down, x, y), &arena);
    let _ = hero.handle_pointer(ptr(PointerKind::Move, x - 4000.0, y), &arena);
    assenta(&mut hero, &mut ts);
    let _ = hero.handle_pointer(ptr(PointerKind::Move, x - 50.0, y), &arena);
    assenta(&mut hero, &mut ts);
    let _ = hero.handle_pointer(ptr(PointerKind::Up, x - 50.0, y), &arena);
    assenta(&mut hero, &mut ts);
    let depois = hero.store.panel_rect(JANELA).expect("continua");
    assert_eq!(depois.w, antes.w + 50.0, "a borda não ficou sob o ponteiro");
}

/// ⭐ Arrastada pelo título para longe, a janela fica DENTRO da área de desenho e fora da lista que
/// flutua sobre a coluna da esquerda (a Hierarchy): nunca tapada por ela, nunca a tapar.
#[test]
fn arrastada_pelo_titulo_fica_na_area_e_fora_da_hierarchy() {
    let mut ts = ph2d_text::TextSystem::without_system_fonts();
    let mut hero = hero(&mut ts);
    for (dx, dy) in [(-4000.0, 4000.0), (4000.0, -4000.0), (-4000.0, -4000.0)] {
        let r = hero
            .hit_index
            .rect_for(ph2d_panel_pixel_slices::ids::SLICES_DRAG);
        let r = r.expect("a alça do título");
        let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        let arena = Bump::new();
        let _ = hero.handle_pointer(ptr(PointerKind::Down, x, y), &arena);
        let _ = hero.handle_pointer(ptr(PointerKind::Move, x + dx, y + dy), &arena);
        let _ = hero.handle_pointer(ptr(PointerKind::Up, x + dx, y + dy), &arena);
        assenta(&mut hero, &mut ts);
        let layout = hero.last_layout.expect("layout");
        let (area, j) = (
            layout.draw_area,
            hero.store.panel_rect(JANELA).expect("pintou"),
        );
        let dentro = j.x >= area.x
            && j.y >= area.y
            && j.x + j.w <= layout.inspector.x
            && j.y + j.h <= area.y + area.h;
        assert!(dentro, "({dx},{dy}): {j:?} saiu da área {area:?}");
        let hier = layout.float_lists[0];
        let tapa = hier.h > 0.0 && j.x < hier.x + hier.w && hier.x < j.x + j.w;
        assert!(!tapa, "({dx},{dy}): {j:?} sobre a Hierarchy {hier:?}");
        let (cx, cy) = (j.x + j.w * 0.5, j.y + j.h * 0.5);
        assert_eq!(hero.chrome_panel_at(cx, cy), Some(JANELA));
    }
}

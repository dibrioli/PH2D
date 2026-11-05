//! As fotos da fatia F7, por pintura real do hero: `PH2D_FOTO_DIR=<pasta>`, `--ignored`, por nome `f7_foto`.
//! Bones com Reach + Handles · «Stroke» com a seta · o painel do traço sobre outro painel · o gizmo a
//! abrir um eixo a zero. O gesto da alça não existe em janela virtual: a última é o gizmo real, em duas poses.
#![cfg(all(feature = "panel-skeleton", feature = "panel-vector"))]

use super::o_cartao_solto_no_canvas::{clica, hero_com_o_bones, osso_em_foco, pinta};
use ph2d_editor_core::gizmo::{GizmoView, MODEL_IDENTITY, paint_sprite_gizmo};
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, NodeId, ids};
use ph2d_panel_skeleton as skel;
use ph2d_tool_vector::{DrawMode, VectorStyleSnapshot};
use ph2d_vector::VectorScene;

fn guarda(cena: &VectorScene, nome: &str, janela: Rect, amplia: u32) {
    let dir = std::env::var("PH2D_FOTO_DIR").expect("PH2D_FOTO_DIR");
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("a foto precisa da placa");
    let (w, h) = (janela.w as u32, janela.h as u32);
    let mut pass =
        ph2d_render::VelloPass::new(&gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (w, h)).unwrap();
    let mut recorte = VectorScene::new();
    recorte.append_transformed(
        cena,
        ph2d_vector::Affine::translate((-f64::from(janela.x), -f64::from(janela.y))),
    );
    let px = pass
        .render_and_readback(&gpu, recorte.inner(), (w, h))
        .unwrap();
    let img = image::RgbaImage::from_raw(w, h, px).unwrap();
    let img = image::imageops::resize(
        &img,
        w * amplia,
        h * amplia,
        image::imageops::FilterType::Nearest,
    );
    img.save(format!("{dir}/{nome}.png")).unwrap();
}

fn quadro(h: &mut HeroScreen, ts: &mut ph2d_text::TextSystem) -> VectorScene {
    let mut cena = VectorScene::new();
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    paint_hero_screen(h, vp, &mut cena, ts);
    cena
}

fn bones(ts: &mut ph2d_text::TextSystem) {
    let mut h = hero_com_o_bones();
    osso_em_foco(7, "Bone", 0.5);
    let mut osso = ph2d_skeleton::bend::BoneSpec::straight(20.0, 1.0);
    osso.segments = 4;
    skel::state::set_current_bone(Some(osso));
    skel::state::set_current_bone_handles(Some(0));
    skel::state::set_current_envelope_manda(true);
    pinta(&mut h, 60);
    let cena = quadro(&mut h, ts);
    let r = h
        .hit_index
        .rect_for(ids::VECTOR_BONE_STRENGTH)
        .expect("Reach");
    let janela = Rect::new(r.x - 16.0, r.y - 150.0, r.w + 32.0, 340.0);
    guarda(&cena, "f7_bones_reach_handles", janela, 2);
    skel::state::set_current_bone(None);
}

fn traco(ts: &mut ph2d_text::TextSystem) {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut h = HeroScreen::new(NodeId(1));
    let publica = |h: &mut HeroScreen| {
        ph2d_panel_vector::set_current_vector_style(Some(VectorStyleSnapshot {
            mode: DrawMode::Select,
            ..VectorStyleSnapshot::default()
        }));
        ph2d_panel_vector::set_current_selection_count(1);
        ph2d_panel_vector::set_current_vertex_count(0);
        let barras = ph2d_panel_vector::bars::tool_bars(&h.store, None, Vec::new());
        h.store.set_tool_bars(Some(barras));
    };
    for _ in 0..40 {
        publica(&mut h);
        let _ = quadro(&mut h, ts);
    }
    publica(&mut h);
    let cena = quadro(&mut h, ts);
    let chip = ph2d_panel_vector::ids::VECTOR_BAR_STROKE;
    let r = h.hit_index.rect_for(chip).expect("o chip Stroke");
    let barra = Rect::new(
        (r.x - 180.0).max(0.0),
        (r.y - 8.0).max(0.0),
        520.0,
        r.h + 16.0,
    );
    guarda(&cena, "f7_stroke_seta", barra, 3);
    clica(&mut h, chip);
    for _ in 0..30 {
        publica(&mut h);
        let _ = quadro(&mut h, ts);
    }
    publica(&mut h);
    let cena = quadro(&mut h, ts);
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    guarda(&cena, "f7_stroke_popover", vp, 1);
    ph2d_panel_vector::set_current_vector_style(None);
    ph2d_panel_vector::set_current_selection_count(0);
    h.store.set_tool_bars(None);
}

/// Em cima, a forma a zero em X (uma linha, as alças juntas); em baixo, depois de abrir pela alça.
fn gizmo() {
    let vista = |y: f32, meio_x: f32| GizmoView {
        bbox_min_world: [-meio_x, y - 1.0],
        bbox_max_world: [meio_x, y + 1.0],
        rotation: 0.0,
        camera_center: [0.0, 0.0],
        camera_height_world: 10.0,
        window_w: 800.0,
        window_h: 600.0,
        canvas: Rect::new(0.0, 0.0, 800.0, 600.0),
        cursor_screen: None,
        pivot_world: [0.0, y],
        pivot_tool_active: false,
        model: MODEL_IDENTITY,
    };
    let mut cena = VectorScene::new();
    let mut hits = ph2d_editor_core::interaction::HitIndex::new();
    let tema = ph2d_tokens::Theme::Dark;
    let fundo = ph2d_vector::Rect::new(0.0, 0.0, 800.0, 600.0);
    cena.fill_rect(
        fundo,
        ph2d_editor_core::paint::resolve(ph2d_tokens::ColorToken::Bg, tema),
    );
    paint_sprite_gizmo(&mut cena, &vista(2.2, 0.0), tema, &mut hits);
    paint_sprite_gizmo(&mut cena, &vista(-2.2, 1.5), tema, &mut hits);
    guarda(
        &cena,
        "f7_gizmo_abre_o_zero",
        Rect::new(0.0, 0.0, 800.0, 600.0),
        1,
    );
}

#[test]
#[ignore = "fotos: PH2D_FOTO_DIR=<pasta> e --ignored"]
fn f7_foto() {
    let mut ts = ph2d_text::TextSystem::new();
    bones(&mut ts);
    traco(&mut ts);
    gizmo();
}

//! ⚠️ MEDIÇÃO (W4, lote V): o custo por quadro de cada nível do Kanban no canvas, com a cena GRANDE
//! (8 listas × 250 cartões) num Quadro, e a altura do texto no ecrã a cada zoom. Os três níveis à
//! MESMA escala, intercalados em rondas no mesmo processo; o resumo é o MÍNIMO
//! (`docs/DevOps/MEDIR_VELOCIDADE.md`). A tabela vive em `docs/TrelloClone/loop/MEDICOES.md`.
//!
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-app-kanban --profile smoke --test it medida -- --ignored --nocapture`

use ph2d_board_render::{KanbanLod, force_kanban_lod};
use ph2d_editor_core::NodeId;
use ph2d_editor_core::screens::hero::board_video::kanban::link;
use ph2d_editor_core::screens::hero::{
    HERO_VIEWPORT_H, HERO_VIEWPORT_W, HeroScreen, paint_hero_screen,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_vector::VectorScene;
use std::time::{Duration, Instant};

/// O tamanho do elemento medido (unidades do mundo).
const TAMANHO: [f64; 2] = [1900.0, 1000.0];

fn quadro(h: &mut HeroScreen, ts: &mut TextSystem, agora: &mut u128) -> Duration {
    *agora += crate::QUADRO_NS;
    h.store.note_clock(*agora);
    ph2d_app_kanban::frame(h);
    ph2d_editor_core::screens::hero::document_tabs::follow_surfaces(h);
    ph2d_board_render::install_kanbans(h.documents.kanbans_shared());
    let mut scene = VectorScene::new();
    let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
    let t = Instant::now();
    paint_hero_screen(h, vp, &mut scene, ts);
    t.elapsed()
}

#[test]
#[ignore = "medição: corre-se à mão (perfil smoke)"]
fn medida_dos_niveis_do_kanban_no_canvas() {
    ph2d_editor_core::test_support::ensure_panel_registry();
    let mut h = HeroScreen::new(NodeId(1));
    ph2d_app_kanban::install(&mut h);
    let mut ts = TextSystem::without_system_fonts();
    let mut agora = 1_000_000_000_000u128;
    let id = ph2d_app_kanban::smoke::stage(&mut h, 2).expect("cena grande");
    for _ in 0..2 {
        quadro(&mut h, &mut ts, &mut agora);
    }
    h.documents.create("Board".into());
    link::request(link::BoardRequest::Place { kanban: id });
    for _ in 0..3 {
        quadro(&mut h, &mut ts, &mut agora);
    }
    let el = link::placements(&h.documents, id)[0].element;
    // Um Kanban do tamanho de uma janela grande no canvas (o de nascença cabe em meia vista).
    if let Some(b) = h.documents.active_board_mut() {
        let mut e = b.doc.get(el).unwrap().clone();
        [e.w, e.h] = TAMANHO;
        ph2d_board_model::BoardOp::Put(e).apply(&mut b.doc);
    }
    let (w, hh) = {
        let e = h.documents.active_board().unwrap().doc.get(el).unwrap();
        (e.w, e.h)
    };
    let xs = f64::from(ph2d_tokens::TypeToken::Xs.px());
    println!(
        "elemento {w:.0}×{hh:.0} · TypeToken::Xs = {xs} px · janela {HERO_VIEWPORT_W}×{HERO_VIEWPORT_H}"
    );
    println!(
        "| zoom | texto do cartão (px ecrã) | título de lista (px ecrã) | nível pelo zoom | Baixo ms | Médio ms | Alto ms |"
    );
    println!("|---|---|---|---|---|---|---|");
    let lods = [KanbanLod::Low, KanbanLod::Mid, KanbanLod::High];
    for zoom in [0.15, 0.25, 0.4, 0.55, 0.7, 0.786, 0.9, 1.0, 1.5] {
        if let Some(b) = h.documents.active_board_mut() {
            b.camera.zoom = zoom;
            b.camera.center_x = 0.0;
            b.camera.center_y = 0.0;
        }
        let mut best = [Duration::MAX; 3];
        // Aquecer cada nível (a cache do texto), depois rondas rodadas.
        for l in lods {
            force_kanban_lod(Some(l));
            for _ in 0..3 {
                quadro(&mut h, &mut ts, &mut agora);
            }
        }
        for ronda in 0..12 {
            for k in 0..3 {
                let i = (k + ronda) % 3;
                force_kanban_lod(Some(lods[i]));
                best[i] = best[i].min(quadro(&mut h, &mut ts, &mut agora));
            }
        }
        force_kanban_lod(None);
        let ms = |d: Duration| d.as_secs_f64() * 1e3;
        println!(
            "| {zoom} | {:.1} | {:.1} | {:?} | {:.2} | {:.2} | {:.2} |",
            14.0 * zoom,
            15.0 * zoom,
            KanbanLod::at(zoom),
            ms(best[0]),
            ms(best[1]),
            ms(best[2])
        );
    }
}

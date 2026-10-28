//! 📷 As fotos das vistas sobre a cena 1, claro e escuro (`#[ignore]`: a placa).
//! `bash scripts/cargo-test-narrow.sh ph2d-kanban-views photo_ -- --ignored`

use crate::support::{Shots, World};
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_views::{ViewKind, Views};
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::Affine;

const W: u32 = 1440;
const H: u32 = 900;

/// Fotografa a vista `kind` nos dois temas, com `prep` a pôr a vista no estado da foto.
pub fn photo(name: &str, kind: ViewKind, prep: impl Fn(&mut Views, &World)) {
    let w = World::scene1();
    let mut shots = Shots::new();
    for (theme, tag) in [(Theme::Light, "claro"), (Theme::Dark, "escuro")] {
        let mut views = Views::default();
        views.kind = kind;
        prep(&mut views, &w);
        let cx = w.cx();
        #[allow(clippy::cast_precision_loss)]
        let rect = Rect::new(0.0, 0.0, W as f32, H as f32);
        shots.shot(&format!("{name}_{tag}"), (W, H), |scene, ts| {
            scene.fill_rect(
                ph2d_vector::Rect::new(0.0, 0.0, f64::from(W), f64::from(H)),
                resolve(ColorToken::KanbanList, theme),
            );
            views.paint(scene, Affine::IDENTITY, rect, &w.set, w.id, ts, theme, &cx);
        });
    }
}


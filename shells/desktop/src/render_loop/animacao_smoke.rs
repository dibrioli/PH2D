//! `PH2D_ANIMACAO_SMOKE=1` — **a Animação por objecto** (doc ObjectsAndComponents 01 §6, onda 2):
//! o Herói tem a animação DELE («andar», «pular»; o «andar» também gira a Espada, filha dele), o
//! Inimigo mexe-se pela Animação da Cena («Main»), e o Inimigo não tem Animação — o ▾ oferece-a.
//! A Timeline e a Hierarquia abrem com o Herói seleccionado (`=1`), ou com o Inimigo e o ▾ da
//! Timeline aberto (`=2`: «Animation: Scene» e «Add animation to Inimigo»).

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{ChildOf, Name, Transform};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_render::Sprite;
use ph2d_timeline::PropKind;

static FEITO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn key(doc: &mut ph2d_timeline::TimelineDoc, bits: u64, prop: PropKind, t: f64, v: f32) {
    doc.upsert_key(
        bits,
        prop,
        RationalTime::from_seconds(t),
        AnimValue::Float(v),
        Interp::Linear,
    );
}

impl crate::App {
    pub(crate) fn animacao_smoke(&mut self) {
        let Some(nivel) = std::env::var("PH2D_ANIMACAO_SMOKE").ok() else {
            return;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return; // ainda sem mundo; tenta no próximo quadro
        };
        // A trava vive aqui e não na `App` (o tecto de campos dela só desce).
        if FEITO.swap(true, std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        let world = gfx.sim.world_mut();
        let sprite = |w: f32, h: f32, rgba: [f32; 4]| Sprite::atlas(0, [w, h], rgba);
        let heroi = world
            .spawn((
                Transform::from_translation(Vec2::new(-2.0, 0.0)),
                sprite(1.0, 1.0, [1.0, 0.45, 0.15, 1.0]),
                Name::new("Herói"),
            ))
            .id();
        let espada = world
            .spawn((
                Transform::from_translation(Vec2::new(0.8, 0.0)),
                sprite(0.25, 1.2, [0.75, 0.78, 0.85, 1.0]),
                Name::new("Espada"),
                ChildOf(heroi),
            ))
            .id()
            .to_bits();
        let inimigo = world
            .spawn((
                Transform::from_translation(Vec2::new(2.5, 0.0)),
                sprite(1.0, 1.0, [0.55, 0.3, 0.85, 1.0]),
                Name::new("Inimigo"),
            ))
            .id()
            .to_bits();
        let heroi = heroi.to_bits();

        let doc = &mut self.timeline.doc;
        // A Animação da CENA: o clip de sempre, a mexer o Inimigo.
        key(doc, inimigo, PropKind::TranslationY, 0.0, 0.0);
        key(doc, inimigo, PropKind::TranslationY, 1.0, 0.8);
        key(doc, inimigo, PropKind::TranslationY, 2.0, 0.0);
        doc.set_clip_length_override(0, Some(2.0));
        // A do HERÓI: «andar» (anda e gira a Espada) e «pular».
        let andar = doc.add_clip_for("andar".into(), heroi).expect("andar");
        doc.set_active(andar);
        key(doc, heroi, PropKind::TranslationX, 0.0, -3.0);
        key(doc, heroi, PropKind::TranslationX, 1.0, -1.0);
        key(doc, heroi, PropKind::TranslationX, 2.0, -3.0);
        key(doc, espada, PropKind::Rotation, 0.0, 0.0);
        key(doc, espada, PropKind::Rotation, 1.0, 0.8);
        key(doc, espada, PropKind::Rotation, 2.0, 0.0);
        doc.set_clip_length_override(andar, Some(2.0));
        let pular = doc.add_clip_for("pular".into(), heroi).expect("pular");
        doc.set_active(pular);
        key(doc, heroi, PropKind::TranslationY, 0.0, 0.0);
        key(doc, heroi, PropKind::TranslationY, 0.5, 1.5);
        key(doc, heroi, PropKind::TranslationY, 1.0, 0.0);
        doc.set_clip_length_override(pular, Some(1.0));
        doc.set_active(andar);

        if let Some(hero) = gfx.hero_screen.as_mut() {
            let segundo = nivel == "2";
            hero.gizmo
                .replace_selection(Some(if segundo { inimigo } else { heroi }));
            // O ▾ aberto: o estado do chip vive no `store` do ecrã (o `populate` regista-o no arranque).
            if segundo
                && let Some(ph2d_editor_core::interaction::InteractiveState::Dropdown {
                    open, ..
                }) = hero
                    .store
                    .get_mut(ph2d_panel_timeline::ids::TIMELINE_ANIM_DD)
            {
                *open = true;
            }
            hero.set_panel_visible("timeline", true);
            hero.set_panel_visible("hierarchy", true);
        }
        self.playhead.rewind();
        self.playhead.set_loop(0.0, 2.0);
        self.playhead.play();
        eprintln!(
            "[animacao-smoke] Herói ▸ Clips (andar, pular) · Espada (filha) · Inimigo (Animação da Cena). \
             A Timeline diz «Animation: Herói»; clique no Inimigo e ela mostra a da Cena."
        );
    }
}

//! `PH2D_ANIMACAO_SMOKE=1` — **a Animação por objecto** (doc ObjectsAndComponents 01 §6): o Herói
//! tem «andar» (gira também a Espada, filha) e «pular»; o Inimigo mexe-se pela Animação da Cena e
//! ouve o sinal «land» do marcador do quadro 24 (Signal Actions; doc Timeline 17, B5). `=2`: o
//! Inimigo e o ▾ da Timeline aberto. `=4`: o Herói com a Signal Action «Play Clip» e a «Bandeira»,
//! sprite com SKEW (`=5`: seleccionada; `=6`: sem rotação, com a Caixa, as duas seleccionadas).

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
                ph2d_ecs::SignalActions(vec![ph2d_ecs::SignalAction {
                    on: "land".into(),
                    verb: ph2d_ecs::SignalVerb::Show,
                    ..Default::default()
                }]),
            ))
            .id()
            .to_bits();
        let (mut bandeira, mut caixa) = (None, None);
        if matches!(nivel.as_str(), "4" | "5" | "6") {
            world
                .entity_mut(heroi)
                .insert(ph2d_ecs::SignalActions(vec![ph2d_ecs::SignalAction {
                    on: "pulo".into(),
                    verb: ph2d_ecs::SignalVerb::PlayClip,
                    arg: "pular".into(),
                    ..Default::default()
                }]));
            let mut torta = Transform::from_translation(Vec2::new(0.5, -1.2));
            torta.skew_x = 0.5;
            torta.rotation = if nivel == "6" { 0.0 } else { 0.3 };
            bandeira = Some(
                world
                    .spawn((
                        torta,
                        sprite(2.0, 1.0, [0.2, 0.7, 0.4, 1.0]),
                        Name::new("Bandeira"),
                    ))
                    .id()
                    .to_bits(),
            );
            if nivel == "6" {
                let lado = Transform::from_translation(Vec2::new(3.5, -1.2));
                caixa = Some(
                    world
                        .spawn((
                            lado,
                            sprite(1.0, 1.0, [0.9, 0.8, 0.2, 1.0]),
                            Name::new("Caixa"),
                        ))
                        .id()
                        .to_bits(),
                );
            }
        }
        let heroi = heroi.to_bits();

        let doc = &mut self.timeline.doc;
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
        key(doc, heroi, PropKind::TranslationX, 0.0, -3.0);
        key(doc, heroi, PropKind::TranslationX, 1.0, -1.0);
        doc.set_clip_length_override(pular, Some(1.0));
        let land = doc.add_marker(RationalTime::from_seconds(1.0), "land");
        doc.set_marker_signal(land, Some("land".into()));
        doc.set_active(andar);

        if let Some(hero) = gfx.hero_screen.as_mut() {
            let segundo = nivel == "2";
            let alvo = match (segundo, bandeira) {
                (true, _) => inimigo,
                (false, Some(b)) if nivel == "5" || nivel == "6" => b,
                _ => heroi,
            };
            hero.gizmo.replace_selection(Some(alvo));
            hero.gizmo.extra_selection.extend(caixa);
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
        // `=3`: o «andar» do Herói e o «Main» da Cena com Autoplay, e a cena a CORRER como pelo Play
        // da barra de cima (onda 4, item 1). Em `=1` nada vem marcado — como no Godot.
        if nivel == "3" {
            self.timeline.doc.set_autoplay(andar, true);
            self.timeline.doc.set_autoplay(0, true);
            self.timeline_intents
                .push(ph2d_timeline::TimelineIntent::RunScene);
        }
        eprintln!(
            "[animacao-smoke] Herói ▸ Clips (andar, pular) · Espada (filha) · Inimigo (Animação da Cena). \
             A Timeline diz «Animation: Herói»; clique no Inimigo e ela mostra a da Cena."
        );
    }
}

//! O onion da timeline é **puro** (ADR-0142): dado o mundo, o doc, os alvos e o instante
//! vivo, os fantasmas são função disso — respondível headless. Estes gates pinam *quantos*
//! fantasmas, *onde* (a pose de `t±k`, não a viva), *de que cor* (frio atrás, quente à
//! frente), *quão fortes* (falloff) e *que forma* (silhueta plana).

use super::{GHOST_MIN_ALPHA, OnionSettings, build_ghosts};
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Transform, World};
use ph2d_render::RenderInstance;
use ph2d_timeline::{PropKind, TimelineDoc, pose_at};

/// Um `RenderInstance` neutro (branco, basis identidade) — o "vivo" que os fantasmas
/// herdam os campos de sprite (aqui só a forma importa).
fn template() -> RenderInstance {
    let mut t: RenderInstance = bytemuck::Zeroable::zeroed();
    t.tint = [1.0, 1.0, 1.0, 1.0];
    t.per_corner_tint = [[1.0, 1.0, 1.0, 1.0]; 4];
    t.size = [1.0, 1.0];
    t.basis = RenderInstance::IDENTITY_BASIS;
    t.opacity = 1.0;
    t.texture_id = 1;
    t
}

/// Sim + doc: um objeto cujo X vai de 0 a 10 em 0..4 s (linear ⇒ `x = 2,5·t`).
fn rig() -> (World, u64, TimelineDoc) {
    let mut w = World::new();
    let e = w.spawn(Transform::from_translation(Vec2::ZERO)).id();
    let b = e.to_bits();
    let mut doc = TimelineDoc::new();
    doc.insert_key(b, PropKind::TranslationX, RationalTime::from_seconds(0.0), AnimValue::Float(0.0), Interp::Linear);
    doc.insert_key(b, PropKind::TranslationX, RationalTime::from_seconds(4.0), AnimValue::Float(10.0), Interp::Linear);
    (w, b, doc)
}

/// `fps = 4` ⇒ `dt = 0,25 s` ⇒ passos de X de `0,625` bem separados, ligado.
fn settings() -> OnionSettings {
    OnionSettings {
        enabled: true,
        frames_before: 2,
        frames_after: 2,
        fps: 4.0,
        ..OnionSettings::default()
    }
}

#[test]
fn the_onion_ghosts_the_frames_before_and_after() {
    let (w, e, doc) = rig();
    let s = settings();
    let mut out = Vec::new();
    build_ghosts(&s, &w, &doc, &[(e, template())], 2.0, &mut out);
    assert_eq!(
        out.len(),
        (s.frames_before + s.frames_after) as usize,
        "um fantasma por quadro antes e depois"
    );
}

#[test]
fn past_ghosts_are_cool_and_future_ghosts_are_warm() {
    let (w, e, doc) = rig();
    let s = settings();
    let live_x = pose_at(&w, &doc, e, 2.0).unwrap().translation.x; // 5.0
    let mut out = Vec::new();
    build_ghosts(&s, &w, &doc, &[(e, template())], 2.0, &mut out);
    for g in &out {
        let cool = g.world_pos[0] < live_x; // passado = X menor (objeto ia para a direita)
        let want = if cool { s.color_before } else { s.color_after };
        assert_eq!(
            [g.tint[0], g.tint[1], g.tint[2]],
            want,
            "fantasma em x={} usou a cor errada",
            g.world_pos[0]
        );
    }
    // E os dois lados de fato existem (senão "todos frios" passaria vazio).
    assert!(out.iter().any(|g| g.world_pos[0] < live_x), "sem fantasma passado");
    assert!(out.iter().any(|g| g.world_pos[0] > live_x), "sem fantasma futuro");
}

#[test]
fn a_farther_ghost_is_fainter() {
    let (w, e, doc) = rig();
    let s = settings();
    let live_x = pose_at(&w, &doc, e, 2.0).unwrap().translation.x;
    let mut out = Vec::new();
    build_ghosts(&s, &w, &doc, &[(e, template())], 2.0, &mut out);
    // Do lado futuro: o mais próximo (menor |x-live|) é mais forte que o mais distante.
    let mut future: Vec<_> = out.iter().filter(|g| g.world_pos[0] > live_x).collect();
    future.sort_by(|a, b| a.world_pos[0].partial_cmp(&b.world_pos[0]).unwrap());
    assert!(future.len() >= 2);
    assert!(
        future[0].tint[3] > future[1].tint[3],
        "o fantasma mais próximo ({}) não é mais forte que o distante ({})",
        future[0].tint[3],
        future[1].tint[3]
    );
    for g in &out {
        assert!(g.tint[3] >= GHOST_MIN_ALPHA, "abaixo do piso de opacidade");
    }
}

#[test]
fn a_ghost_stands_where_the_object_would_be_not_where_it_is() {
    // A prova de que a pose vem de `pose_at(t±k)`, não do instante vivo: NENHUM fantasma
    // fica onde o objeto está agora. Mutação (amostrar em `live_clip_t`) faz todos
    // caírem em x=live ⇒ RED.
    let (w, e, doc) = rig();
    let s = settings();
    let live_x = pose_at(&w, &doc, e, 2.0).unwrap().translation.x;
    let mut out = Vec::new();
    build_ghosts(&s, &w, &doc, &[(e, template())], 2.0, &mut out);
    for g in &out {
        assert!(
            (g.world_pos[0] - live_x).abs() > 1e-3,
            "um fantasma ({}) caiu sobre a pose viva ({live_x}) — a pose não veio de t±k",
            g.world_pos[0]
        );
    }
}

#[test]
fn a_ghost_is_a_flat_silhouette() {
    let (w, e, doc) = rig();
    let s = settings();
    let mut out = Vec::new();
    build_ghosts(&s, &w, &doc, &[(e, template())], 2.0, &mut out);
    assert!(!out.is_empty());
    for g in &out {
        assert!(
            g.flip_uv & RenderInstance::TINT_FILL_BIT != 0,
            "o fantasma não está em modo silhueta (tint_fill)"
        );
        assert_eq!(g.per_corner_tint, [[1.0, 1.0, 1.0, 1.0]; 4], "per-corner não-neutro tingiria a silhueta");
        assert_eq!(g.opacity, 1.0, "a opacity herdada apagaria o falloff da tinta");
    }
}

#[test]
fn the_onion_is_off_by_default_and_when_disabled() {
    assert!(!OnionSettings::default().enabled, "o onion nasce desligado");
    let (w, e, doc) = rig();
    let s = OnionSettings { enabled: false, ..settings() };
    let mut out = Vec::new();
    build_ghosts(&s, &w, &doc, &[(e, template())], 2.0, &mut out);
    assert!(out.is_empty(), "desligado não desenha fantasma");
}

#[test]
fn no_targets_no_ghosts() {
    let (w, _e, doc) = rig();
    let mut out = Vec::new();
    build_ghosts(&settings(), &w, &doc, &[], 2.0, &mut out);
    assert!(out.is_empty());
}

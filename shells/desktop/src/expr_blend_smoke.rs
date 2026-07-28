//! **Smoke das expressões que FADEIAM no blend** (ADR-0146). `PH2D_EXPR_BLEND_SMOKE=1`:
//!
//! O `expr_smoke` (ADR-0144/0145) mostra o comportamento ANTIGO — a expressão liga/desliga
//! seco na borda da strip. ESTA cena mostra o novo: a expressão é a FONTE que a strip entrega
//! ao blend, então ela **fadeia, cruza e é seguida** como uma animação keyada. Abra em ARRANGE
//! (o default) e deixe o playhead correr pelo loop `[0, 6]s`.
//!
//! Quatro objetos, em unidades de MUNDO (~±6):
//!   - **"Cross"** (laranja, y=+3.5) — o TÍTULO. X dirigido pela expressão CONSTANTE `+4`
//!     (clip "Expr") numa strip `[0,3)`, cruzando com uma X keyada FLAT `-4` (clip "Key") numa
//!     strip `[2,5)`. Na sobreposição `[2,3)` o X **cruza suavemente de +4 para −4** — o motor
//!     antigo daria um SALTO seco na borda; agora é um crossfade.
//!   - **"Follow"** (azul, y=+1.5) — X = `Cross.x`. Um PROP-LINK que acompanha o valor JÁ
//!     FADEADO do Cross, no MESMO frame (duplo fade): durante o crossfade do Cross, o Follow
//!     desliza junto de +4 a −4.
//!   - **"Wiggle"** (verde, y=−0.5) — X keyado (rampa −4→4) + Y = `value + wiggle(3, 0.8)` sobre
//!     uma Y keyada FLAT. Anda na horizontal e treme na vertical, determinístico.
//!   - **"Add"** (âmbar, y=−2.5) — X keyado FLAT 0 numa lane base + uma lane ADITIVA com
//!     `time*0.8`: a lane aditiva contribui o DELTA (`E(t) − E(início)`), então o objeto desliza
//!     para a direita por cima da base.
//!
//! **O que provar** (playhead correndo em ARRANGE):
//! 1. Cross e Follow **cruzam suave** de +4 a −4 na janela `[2,3)` (não saltam). Pare o playhead
//!    dentro da sobreposição e arraste a régua devagar: o valor varia continuamente.
//! 2. **Sem chave-fantasma:** com AutoKey armado e a cena parada, NADA é keyado sozinho (o
//!    Follow prop-linkado fica quieto).
//! 3. **Keying honesto** (R-CLICK na track / K):
//!    - `value + g(time)` **pré-compensa** (a pose pousa onde você a largou).
//!    - Escreva `wiggle(3, 1)` numa track e tente K → **RECUSA** com *"clean or rewrite the
//!      formula"* (não "delete a lane").
//! 4. **`value` é per-STRIP** — edite a fórmula do Wiggle para `value + wiggle(6, 2)`: treme mais
//!    forte, sobre a amostra keyada DAQUELA strip.
//!
//! ⚠️ Se a linha `[expr-blend-smoke]` não aparecer, PARE: a cena não montou.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_timeline::{
    LaneMode, PropKind, StackHost, StripSource, TimelineDoc, TimelineIntent, apply_intent,
};

/// Author `src` as the PER-CLIP expression of `(bits, prop)` in clip `clip` (ADR-0145/0146).
fn expr(doc: &mut TimelineDoc, clip: usize, bits: u64, prop: PropKind, src: &str) {
    let target = doc.bind(bits, prop);
    doc.set_clip_expr(clip, target, Some(src.to_string()));
}

/// Key `(bits, prop)` FLAT at `v` in clip `clip` (Hold).
fn key_flat(doc: &mut TimelineDoc, clip: usize, bits: u64, prop: PropKind, v: f32) {
    let was = doc.active_index();
    doc.set_active(clip);
    doc.insert_key(
        bits,
        prop,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(v),
        Interp::Hold,
    );
    doc.set_active(was);
}

/// Key `(bits, prop)` linearly `v0 -> v1` over `0..dur` in clip `clip`.
fn ramp(doc: &mut TimelineDoc, clip: usize, bits: u64, prop: PropKind, v0: f32, v1: f32, dur: f64) {
    let was = doc.active_index();
    doc.set_active(clip);
    let s = RationalTime::from_seconds;
    doc.insert_key(bits, prop, s(0.0), AnimValue::Float(v0), Interp::Linear);
    doc.insert_key(bits, prop, s(dur), AnimValue::Float(v1), Interp::Linear);
    doc.set_active(was);
}

impl crate::App {
    /// In the frame prologue, once. No-op without the env.
    pub(crate) fn expr_blend_smoke(&mut self) {
        if self.expr_blend_smoke_done {
            return;
        }
        if std::env::var_os("PH2D_EXPR_BLEND_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // no world yet; try next frame
        }
        self.expr_blend_smoke_done = true;

        let spawn = |app: &mut crate::App, name: &str, y: f32, rgb: [f32; 3]| {
            let gfx = app.gfx.as_mut().expect("gfx");
            gfx.sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(0.0, y)),
                    Sprite::atlas(0, [0.8, 0.8], [rgb[0], rgb[1], rgb[2], 1.0]),
                    Name::new(name),
                ))
                .id()
                .to_bits()
        };
        let cross = spawn(self, "Cross", 3.5, [1.0, 0.5, 0.3]);
        let follow = spawn(self, "Follow", 1.5, [0.4, 0.8, 1.0]);
        let wiggle = spawn(self, "Wiggle", -0.5, [0.7, 1.0, 0.4]);
        let add = spawn(self, "Add", -2.5, [1.0, 0.8, 0.4]);

        let add_lane_idx = {
            let doc = &mut self.timeline.doc;
            doc.rename_clip(0, "Expr".into());
            let key_clip = doc.add_clip("Key".into()); // 1
            let base = doc.add_clip("Base".into()); // 2
            let addmove = doc.add_clip("AddMove".into()); // 3

            // Cross: clip 0 drives X by a CONSTANT expression +4; clip 1 keys X flat -4.
            expr(doc, 0, cross, PropKind::TranslationX, "4.0");
            key_flat(doc, key_clip, cross, PropKind::TranslationX, -4.0);

            // Follow: a prop-link tracking Cross's FADED value (in the always-on base lane).
            expr(doc, base, follow, PropKind::TranslationX, "Cross.x");

            // Wiggle: keyed X ramp (-4 -> 4) + Y = value + wiggle over a KEYED flat Y.
            ramp(doc, base, wiggle, PropKind::TranslationX, -4.0, 4.0, 4.0);
            key_flat(doc, base, wiggle, PropKind::TranslationY, 0.0);
            expr(doc, base, wiggle, PropKind::TranslationY, "value + wiggle(3, 0.8)");

            // Add: keyed flat X=0 in the base; the ADDITIVE lane adds time*0.8 (a moving delta).
            key_flat(doc, base, add, PropKind::TranslationX, 0.0);
            expr(doc, addmove, add, PropKind::TranslationX, "time*0.8");

            let clip = |i: usize| StripSource::Clip(u16::try_from(i).unwrap_or(0));
            // cross lane (Override): clip 0 [0,3) crossfades with clip 1 [2,5) -> Cross.X +4 -> -4.
            if let Some(l) = doc.add_lane("cross".into()) {
                let _ = doc.add_strip_to(StackHost::Document, l, clip(0), 0.0, 3.0);
                let _ = doc.add_strip_to(StackHost::Document, l, clip(key_clip), 2.0, 5.0);
            }
            // base lane (Override): clip 2 [0,6) always on (Follow/Wiggle/Add base).
            if let Some(l) = doc.add_lane("base".into()) {
                let _ = doc.add_strip_to(StackHost::Document, l, clip(base), 0.0, 6.0);
            }
            // add lane (Additive): clip 3 [0,6) adds time*0.8 to Add.X (mode set below).
            let add_lane = doc.add_lane("add".into());
            if let Some(l) = add_lane {
                let _ = doc.add_strip_to(StackHost::Document, l, clip(addmove), 0.0, 6.0);
            }

            doc.set_active(base); // the Keys tab shows the always-on objects
            // The scene is a 6 s composition — matching the [0,6) content and the loop below.
            // (`with_default_duration` opens at 4 s; left there, `cut_scene` would FREEZE every
            // object from 4-6 s while the playhead ran on, which reads as a bug — the veil and
            // the content must agree.)
            doc.set_scene_length(Some(6.0));
            // Every clip opens with an AUTHORED duration — exactly what the AddClip
            // authoring intent stamps on a user-made clip (`intent_apply.rs`). The raw
            // `doc.add_clip` above is the DATA layer and stays DERIVED, so without this
            // the Keys tab shows the active clip with NO veil, and the smoke misrepresents
            // the product (Enio: *"o véu deve estar visível antes de tocar na caixa de
            // duração do clip"* — plain boot and user-made clips already carry 4 s; only a
            // smoke's raw clips were bare). **4 s = o padrão do produto** (Enio, 2026-07-28:
            // *"todo clip criado é criado com dur 4 e com véu visível"*): a cena fica em 6 s
            // (o arranjo é [0,6)), mas cada CLIP mostra o mesmo 4 s + véu que um clip criado
            // pelo botão "+". Arrange usa o corte da CENA + as fatias das strips, não o
            // override do clip, então nada muda no crossfade — só a régua da aba Keys.
            let clip_count = doc.clips().len();
            for i in 0..clip_count {
                doc.set_clip_length_override(i, Some(ph2d_timeline::DEFAULT_DURATION_SECONDS));
            }
            doc.set_active_loop_for(false, Some((0.0, 6.0))); // the ARRANGE loop
            add_lane
        };

        // The additive lane's mode, via the intent (the `doc` borrow is dropped now).
        if let Some(l) = add_lane_idx {
            apply_intent(
                &mut self.timeline,
                &mut self.playhead,
                TimelineIntent::SetLaneMode {
                    lane: l,
                    mode: LaneMode::Additive,
                },
            );
        }

        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(cross));
            hero.panel_visibility.insert("timeline", true);
        }
        self.playhead.seek(0.0);
        self.playhead.play();

        eprintln!(
            "[expr-blend-smoke] 4 objetos, expressoes FADEIAM no blend (ADR-0146), loop [0,6] em \
             ARRANGE: Cross (y=+3.5) X=expr +4 (strip [0,3)) CRUZANDO com X keyada -4 (strip \
             [2,5)) -> na janela [2,3) cruza suave de +4 a -4 (o motor antigo SALTAVA). Follow \
             (y=+1.5) X=Cross.x -> segue o valor FADEADO (duplo fade). Wiggle (y=-0.5) X rampa \
             -4..4 + Y=value+wiggle(3,0.8). Add (y=-2.5) X flat 0 + lane ADITIVA time*0.8 (delta). \
             Pare o playhead dentro de [2,3s] e arraste a regua: Cross/Follow variam continuo. \
             AutoKey armado + cena parada -> ZERO chave fantasma. K num canal wiggle -> RECUSA. \
             5. DURACAO 0 = INFINITO (Enio 2026-07-28): a caixa Dur mostra o simbolo de infinito \
             e o veu SOME quando a duracao e apagada (digite 0). Digite um numero > 0 -> veu \
             volta com a mascara pos-fim. Criar clip/container (+) ou abrir o app: nasce com 4 s \
             + veu. A regra vale igual para clip, container e a cena (Arrange)."
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_ecs::{Entity, World};
    use ph2d_timeline::apply_from_doc;

    /// The scene's headline behaviour, headless: the expression CROSSFADES with the keyed pose,
    /// and the prop-link tracks the FADED value — the same doc the smoke builds. Guards that the
    /// scene mounts and behaves (the APIs are gate-proven in `ph2d-timeline`; this pins the wiring).
    #[test]
    fn the_smoke_scene_crossfades_and_the_prop_link_follows() {
        let mut world = World::new();
        let cross = world
            .spawn((Transform::default(), Name::new("Cross")))
            .id()
            .to_bits();
        let follow = world
            .spawn((Transform::default(), Name::new("Follow")))
            .id()
            .to_bits();
        let mut doc = TimelineDoc::new();
        let key_clip = doc.add_clip("Key".into()); // 1
        let base = doc.add_clip("Base".into()); // 2

        expr(&mut doc, 0, cross, PropKind::TranslationX, "4.0");
        key_flat(&mut doc, key_clip, cross, PropKind::TranslationX, -4.0);
        expr(&mut doc, base, follow, PropKind::TranslationX, "Cross.x");

        let clip = |i: usize| StripSource::Clip(u16::try_from(i).unwrap());
        let l = doc.add_lane("cross".into()).unwrap();
        doc.add_strip_to(StackHost::Document, l, clip(0), 0.0, 3.0)
            .unwrap();
        doc.add_strip_to(StackHost::Document, l, clip(key_clip), 2.0, 5.0)
            .unwrap();
        let b = doc.add_lane("base".into()).unwrap();
        doc.add_strip_to(StackHost::Document, b, clip(base), 0.0, 6.0)
            .unwrap();

        let x = |w: &World, e: u64| {
            w.get::<Transform>(Entity::from_bits(e))
                .unwrap()
                .translation
                .x
        };

        // Full weight (t=1, only the expression strip): Cross = +4, Follow tracks it.
        apply_from_doc(&mut world, &mut doc, 1.0);
        assert!((x(&world, cross) - 4.0).abs() < 1e-3, "expr side: +4");
        assert!((x(&world, follow) - 4.0).abs() < 1e-3, "follow tracks +4");

        // Mid-crossfade (t=2.5): Cross is STRICTLY between +4 and -4 (a fade, not a snap), and
        // Follow reads that faded value in the same frame.
        apply_from_doc(&mut world, &mut doc, 2.5);
        let cx = x(&world, cross);
        assert!(
            cx > -4.0 && cx < 4.0,
            "Cross crossfades (strictly between +4 and -4), got {cx}"
        );
        assert!(
            (x(&world, follow) - cx).abs() < 1e-3,
            "Follow reads the FADED Cross.x in the same frame, got {}",
            x(&world, follow)
        );
    }
}

//! **Smoke do C4** (ADR-0152): ler uma propriedade de volta é o inverso EXATO de
//! escrevê-la. `PH2D_MORPH_FADE_SMOKE=1`.
//!
//! Dois canais nunca souberam responder *"que valor você tem no mundo?"* — Morph e
//! Position — porque a porta de leitura tomava só o `PropKind`, enquanto a de escrita
//! sempre tomou o BINDING inteiro. O que isso custava, medido no produto:

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_timeline::{PropKind, TimelineDoc};
use ph2d_vec_scene::ShapeKind;

/// A pose AUTORADA do morph — o número que o `rest` tem de capturar. Longe da key
/// (0.15) de propósito: é a distância entre os dois que torna um estalo visível.
const AUTHORED_T: f32 = 0.85;
/// O valor que a track keya, quase na outra ponta.
const KEYED_T: f32 = 0.15;

fn key_flat(doc: &mut TimelineDoc, clip: usize, bits: u64, prop: PropKind, v: f32) {
    let was = doc.active_index();
    doc.set_active(clip);
    for t in [0.0, 4.0] {
        doc.insert_key(
            bits,
            prop,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
    doc.set_active(was);
}

impl crate::App {
    /// No prólogo do frame, uma vez. No-op sem a env.
    pub(crate) fn morph_fade_smoke(&mut self) {
        if self.morph_fade_smoke_done {
            return;
        }
        if std::env::var_os("PH2D_MORPH_FADE_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // sem mundo ainda; tenta no próximo frame
        }
        self.morph_fade_smoke_done = true;

        // As duas fontes do morph, visualmente DISTINTAS: um estalo entre elas se vê.
        let (ia, ib) = {
            let scene = &mut self.gfx.as_mut().expect("gfx").vec_scene;
            let a = scene.push_path(crate::build_smoke::shape(
                ShapeKind::Ellipse,
                [-5.0, 1.0],
                [-3.0, 3.0],
                &[],
                [90, 170, 255],
            ));
            let b = scene.push_path(crate::build_smoke::shape(
                ShapeKind::Star,
                [3.0, 1.0],
                [5.0, 3.0],
                &[5.0, 0.45],
                [255, 140, 90],
            ));
            (a, b)
        };

        // O morph nasce vazio (a geometria é DERIVADA pelo recook) e só ganha entidade
        // no `sync` — daí a dança: sync, cria, sync outra vez, então pendura.
        // ⚠️ O mapa `path -> entidade` mora no `App` e a cena/mundo no `AppGfx`: empréstimo
        // disjunto de campos, então o `gfx` é re-pegado a cada passo em vez de segurado.
        let morph_bits = {
            let gfx = self.gfx.as_mut().expect("gfx");
            ph2d_vec_entities::entities::sync(
                &mut gfx.sim,
                &mut gfx.vec_scene,
                &mut self.vec.entities,
            );
            let (id, mut morph) = crate::morph_live::create(&mut gfx.vec_scene, ia, ib);
            ph2d_vec_entities::entities::sync(
                &mut gfx.sim,
                &mut gfx.vec_scene,
                &mut self.vec.entities,
            );
            morph.t = AUTHORED_T; // a POSE AUTORADA — o que o `rest` tem de capturar
            let attached = crate::morph_live::attach(&mut gfx.sim, &self.vec.entities, id, &morph);
            assert!(attached, "[morph-fade-smoke] o morph nao pendurou");
            let bits = self.vec.entities[&id];
            gfx.sim
                .world_mut()
                .entity_mut(ph2d_ecs::Entity::from_bits(bits))
                .insert(Name::new("Morpher"));
            bits
        };

        // A AGULHA: o mostrador do prop-link. Sem o fix fica cravada embaixo.
        let needle = {
            let gfx = self.gfx.as_mut().expect("gfx");
            gfx.sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(0.0, -3.0)),
                    Sprite::atlas(0, [0.6, 0.6], [1.0, 0.8, 0.4, 1.0]),
                    Name::new("Needle"),
                ))
                .id()
                .to_bits()
        };

        let doc = &mut self.timeline.doc;
        doc.rename_clip(0, "Morph".into());
        key_flat(doc, 0, morph_bits, PropKind::Morph, KEYED_T);

        // A agulha lê o canal do morph por um prop-link GLOBAL (o campo Expression da
        // track): Y = t*6 - 3, então t=0.85 põe a agulha no alto e t=0.15 embaixo.
        let tgt = doc.bind(needle, PropKind::TranslationY);
        doc.bindings_mut()
            .iter_mut()
            .find(|b| b.target == tgt)
            .expect("acabou de bindar")
            .expr = Some("Morpher.morph * 6 - 3".into());

        // A strip que só cobre [2,6): antes dela a pose é a AUTORADA, e o `ease_in` é o
        // quadro em que o teleporte acontecia.
        let lane = doc.add_lane("L".into()).expect("lane");
        doc.add_strip(lane, 0, 2.0, 6.0);
        doc.stack_mut()[lane].strips[0].ease_in = 1.0;

        eprintln!(
            "[morph-fade-smoke] morph autorado t={AUTHORED_T} -> key {KEYED_T}; \
             strip [2,6) com fade-in de 1,0 s; agulha por prop-link `Morpher.morph`. \
             ARRASTE a regua devagar por t=2 (nao de play): a forma tem de COMECAR a \
             caminhar dali, sem estalo, e a agulha desce junto."
        );
    }
}

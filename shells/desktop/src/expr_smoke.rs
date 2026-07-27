//! **Smoke das expressões de propriedade** (Wave C / ADR-0144). `PH2D_EXPR_SMOKE=1`:
//!
//! Três objetos dirigidos por FÓRMULA (em unidades de MUNDO, ~±6), tocando em loop,
//! cada um demonstrando uma capacidade — para o artista VER expressões ao abrir:
//!   - **"Slider"** — X = `time*1.2`. Uma propriedade dirigida SÓ pela fórmula, sem
//!     keyframe nenhum: desliza a 1.2 u/s.
//!   - **"Wiggler"** — X keyado (rampa −4→4) + Y = `value + wiggle(3, 1.2)` sobre uma
//!     Y keyada FLAT (`value`=0). O wiggle soma jitter POR CIMA: o objeto anda na
//!     horizontal e treme na vertical, determinístico. ⚠️ `value` só é estável numa
//!     prop KEYADA (numa prop sem keys ele realimenta o próprio output).
//!   - **"Follower"** — X = `Slider.x + 2.5`. Um PROP-LINK: segue o Slider deslocado
//!     de 2.5 (sem lag — o Slider é lido no MESMO frame).
//!
//! O passe roda DEPOIS da composição das keys e NUNCA toca o fade — um documento sem
//! expressão é byte-idêntico.
//!
//! O que provar (playhead correndo; R-CLICK na LABEL de uma track):
//! 1. Os três já se movem ao abrir (Slider desliza, Wiggler treme subindo a rampa,
//!    Follower acompanha o Slider).
//! 2. **Expression\u{2026}** no menu abre um campo de texto. Edite a fórmula do
//!    Wiggler para `value + wiggle(6, 2)` -> treme mais rápido e mais forte.
//! 3. Escreva `time*3` numa track qualquer -> ela dispara; ESVAZIE o campo -> volta
//!    aos keyframes.
//! 4. `Slider.x` num objeto novo -> ele passa a seguir o Slider (prop-link).
//! 5. Uma fórmula inválida (`time * (`) mantém o valor keyado (não quebra o frame).
//!
//! ⚠️ Se a linha `[expr-smoke]` não aparecer, PARE: a cena não montou.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_core::Vec2;
use ph2d_ecs::{Name, Transform};
use ph2d_render::Sprite;
use ph2d_timeline::{PropKind, TimelineDoc};

/// Bind `(entity, prop)` and stamp `src` as its expression (creating the binding if
/// the prop has no keys — a pure-expression property).
fn drive(doc: &mut TimelineDoc, bits: u64, prop: PropKind, src: &str) {
    let target = doc.bind(bits, prop);
    if let Some(b) = doc.bindings_mut().iter_mut().find(|b| b.target == target) {
        b.expr = Some(src.to_string());
    }
}

/// Key `(entity, prop)` linearly `v0 -> v1` over `0..dur` (Linear).
fn ramp(doc: &mut TimelineDoc, bits: u64, prop: PropKind, v0: f32, v1: f32, dur: f64) {
    let s = RationalTime::from_seconds;
    doc.insert_key(bits, prop, s(0.0), AnimValue::Float(v0), Interp::Linear);
    doc.insert_key(bits, prop, s(dur), AnimValue::Float(v1), Interp::Linear);
}

impl crate::App {
    /// In the frame prologue, once. No-op without the env.
    pub(crate) fn expr_smoke(&mut self) {
        if self.expr_smoke_done {
            return;
        }
        if std::env::var_os("PH2D_EXPR_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // no world yet; try next frame
        }
        self.expr_smoke_done = true;

        // ⚠️ Coordenadas em UNIDADES DE MUNDO visíveis (~±6, o mesmo alcance do
        // motion_path_smoke) — não px. Sprite de 0.8 unidade.
        let spawn = |app: &mut crate::App, name: &str, x: f32, y: f32| {
            let gfx = app.gfx.as_mut().expect("gfx");
            gfx.sim
                .world_mut()
                .spawn((
                    Transform::from_translation(Vec2::new(x, y)),
                    Sprite::atlas(0, [0.8, 0.8], [1.0, 0.7, 0.3, 1.0]),
                    Name::new(name),
                ))
                .id()
                .to_bits()
        };
        let slider = spawn(self, "Slider", 0.0, 3.0);
        let wiggler = spawn(self, "Wiggler", 0.0, 0.0);
        let follower = spawn(self, "Follower", 0.0, -3.0);

        let doc = &mut self.timeline.doc;
        // Slider: pure-expression X (no keys) sliding at 1.2 u/s (0 -> 4.8 over 4 s).
        drive(doc, slider, PropKind::TranslationX, "time*1.2");
        // Wiggler: a keyed X ramp (-4 -> 4) so it slides, plus wiggle ON TOP of a
        // keyed FLAT Y (`value` = 0). ⚠️ `value` rides the keyed value, so Y must be
        // KEYED — on a keyless prop `value` reads last frame's own output and drifts.
        ramp(doc, wiggler, PropKind::TranslationX, -4.0, 4.0, 4.0);
        ramp(doc, wiggler, PropKind::TranslationY, 0.0, 0.0, 4.0); // flat -> value = 0
        drive(
            doc,
            wiggler,
            PropKind::TranslationY,
            "value + wiggle(3, 1.2)",
        );
        // Follower: a prop-link — follows the Slider offset by 2.5 (no lag).
        drive(doc, follower, PropKind::TranslationX, "Slider.x + 2.5");

        doc.set_active_loop_for(false, Some((0.0, 4.0)));
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(wiggler));
            hero.panel_visibility.insert("timeline", true);
        }
        self.playhead.seek(0.0);
        self.playhead.play();

        eprintln!(
            "[expr-smoke] 3 objetos dirigidos por FORMULA (unidades de mundo ~±6), loop [0,4] \
             tocando: Slider X=time*1.2 (so-expressao, sem keys, y=3), Wiggler X keyado -4..4 + \
             Y=value+wiggle(3,1.2) (jitter vertical sobre Y keyada flat, y=0), Follower \
             X=Slider.x+2.5 (prop-link, sem lag, y=-3). R-CLICK na LABEL de uma track -> \
             Expression... abre o campo. Nenhum strip/fade se mexe (documento sem expr e \
             byte-identico)."
        );
    }
}

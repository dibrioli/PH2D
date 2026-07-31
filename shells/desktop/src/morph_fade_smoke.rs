//! **Smoke do C4** (ADR-0146): ler uma propriedade de volta é o inverso EXATO de
//! escrevê-la. `PH2D_MORPH_FADE_SMOKE=1`.
//!
//! Dois canais nunca souberam responder *"que valor você tem no mundo?"* — Morph e
//! Position — porque a porta de leitura tomava só o `PropKind`, enquanto a de escrita
//! sempre tomou o BINDING inteiro. O que isso custava, medido no produto:
//!
//! - um **Morph sob um fade-in teleportava 0,700 num frame** (estalava na forma A);
//! - um prop-link `Nome.morph` / `Nome.position` resolvia **0** em silêncio, com o
//!   parser aceitando o token e a aresta de dependência montada — um controle que MENTE.
//!
//! A cena mostra os dois de uma vez, e é uma cena de **SCRUB**, não de play: o defeito
//! vivia num único quadro, e um playhead correndo passa por cima dele.
//!
//! **A cena** (unidades de mundo, ~±6):
//!   - **"Morpher"** — um morph REAL entre um círculo (esquerda) e uma estrela (direita).
//!     A pose autorada é `t = 0.85` (quase a estrela). Uma track de **Morph** keya `0.15`
//!     (quase o círculo) num clip, e a strip que a toca vive em `[2, 6)s` com fade-in.
//!   - **"Needle"** (âmbar) — um sprite cujo **Y é dirigido pelo prop-link
//!     `Morpher.morph * 6 - 3`**: ele é o mostrador do canal. Sem o fix ele fica cravado
//!     em −3 para sempre, porque o link resolve 0.
//!
//! **O que provar** (arraste a régua devagar — NÃO dê play):
//! 1. **Em `t < 2`** (antes do fade abrir) a forma está na pose AUTORADA (perto da
//!    estrela) e a agulha está no ALTO. Este é o quadro que quebrava.
//! 2. **Atravesse `t = 2.0` devagar.** A forma tem de COMEÇAR a caminhar dali — sem estalo.
//!    Antes do fix ela saltava para o círculo no primeiro quadro do fade.
//! 3. **`t` de 2 a 3** — a forma caminha até quase o círculo e a **agulha desce junto**,
//!    continuamente. A agulha se mexendo É o prop-link funcionando.
//! 4. **`t > 3`** — segura em `0.15`, agulha embaixo.
//!
//! **A trajetória, MEDIDA headless nesta configuração exata** (não é estimativa):
//!
//! ```text
//!      t    morph.t   agulha.y
//!   0.00     0.8500     2.1000   <- a pose AUTORADA; sem o fix, 0.0000 / -3.0000
//!   1.95     0.8500     2.1000
//!   2.00     0.8500     2.1000   <- o fade ABRE aqui e parte DAQUI (era o teleporte)
//!   2.25     0.7406     1.4438
//!   2.50     0.5000     0.0000
//!   2.75     0.2594    -1.4437
//!   3.00     0.1500    -2.1000   <- o fade fecha
//!   4.00     0.1500    -2.1000
//!   pior passo num frame (20 Hz): 0.0523   (o defeito era um salto de 0,7000)
//! ```
//!
//! ⚠️ Se a linha `[morph-fade-smoke]` não aparecer, PARE: a cena não montou, e o resto
//! do smoke não significa nada.

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
            crate::vec_entities::sync(&mut gfx.sim, &mut gfx.vec_scene, &mut self.vec_entities);
            let (id, mut morph) = crate::morph_live::create(&mut gfx.vec_scene, ia, ib);
            crate::vec_entities::sync(&mut gfx.sim, &mut gfx.vec_scene, &mut self.vec_entities);
            morph.t = AUTHORED_T; // a POSE AUTORADA — o que o `rest` tem de capturar
            let attached = crate::morph_live::attach(&mut gfx.sim, &self.vec_entities, id, &morph);
            assert!(attached, "[morph-fade-smoke] o morph nao pendurou");
            let bits = self.vec_entities[&id];
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

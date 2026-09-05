//! ⭐⭐⭐ **A LINHA DO TEMPO DESVANECE UM CAMINHO VETORIAL** — `PH2D_VEC_FADE_SMOKE=1`.
//!
//! # O defeito que a cena fecha
//!
//! Até 2026-09-04 o canal `Opacity` existia para **todo** objecto e era **MUDO** num vetor: a row
//! aparecia no *+ Track*, aceitava chaves, desenhava a curva no editor de gráfico — e não movia um
//! pixel. O braço da escrita exigia um `ph2d_render::Sprite`, e a entidade de um caminho vetorial
//! nasce com `(Transform, Name, VecPathRef, RootOrder)` e nunca com um.
//!
//! # A cena (unidades de mundo, ~±6)
//!
//! Duas estrelas iguais, com a **mesma** curva de opacidade — `1 → 0 → 1` em 4 s:
//!
//! - **"Fade"** (esquerda, azul) — uma forma simples. Ela prova a metade nova: a linha do tempo
//!   alcança a aparência de um vetor.
//! - **"Fade FX"** (direita, laranja) — a mesma forma **com um brilho** (`FxOp::GLOW`). Ela prova
//!   a segunda metade, e é a que **falhava em silêncio**: o memo do FX guardava a forma AUTORADA,
//!   e a opacidade é uma camada de VISTA aplicada depois — então a textura acertava o memo e
//!   ficava com os pixels opacos da era anterior. *Pixels velhos que ninguém vê que são velhos.*
//!
//! # O que provar (dê PLAY, ou arraste a régua)
//!
//! 1. **As duas** desvanecem juntas, e voltam juntas. Uma que não desvanece é o defeito.
//! 2. **A da direita desvanece com o brilho** — o halo some junto com a forma, não depois nem
//!    "por degraus". Se ela ficar cravada opaca enquanto a esquerda desaparece, o vão do memo
//!    voltou.
//! 3. **No topo (`t = 0` e `t = 4`) as duas voltam a OPACO**, não a um degrau abaixo. Um `254` em
//!    vez de `255` é invisível numa cor chapada e visível na borda contra o fundo.
//!
//! ⚠️ Se a linha `[vec-fade-smoke]` não aparecer, PARE: a cena não montou e o resto não significa
//! nada.

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{FxOp, Name, VecFilter};
use ph2d_timeline::{PropKind, TimelineDoc};
use ph2d_vec_scene::{ShapeKind, VecPathId};

/// A curva: opaca, invisível ao meio, opaca outra vez. Quatro segundos.
fn fade_track(doc: &mut TimelineDoc, bits: u64) {
    for (t, v) in [(0.0, 1.0_f32), (2.0, 0.0), (4.0, 1.0)] {
        doc.insert_key(
            bits,
            PropKind::Opacity,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }
}

/// Um brilho externo — o degrau que põe a forma da direita a passar pelo assador de FX.
fn glow() -> FxOp {
    FxOp {
        radius: 0.16,
        color: [1.0, 0.7, 0.25, 1.0],
        opacity: 1.0,
        ..FxOp::new(FxOp::GLOW)
    }
}

impl crate::App {
    /// No prólogo do frame, uma vez. No-op sem a env.
    pub(crate) fn vec_fade_smoke(&mut self) {
        if self.vec_fade_smoke_done || std::env::var_os("PH2D_VEC_FADE_SMOKE").is_none() {
            return;
        }
        if self.gfx.is_none() {
            return; // sem mundo ainda; tenta no próximo frame
        }
        self.vec_fade_smoke_done = true;

        // As duas formas, iguais de propósito: o que difere é o FILTRO, e é isso que a cena mede.
        let (plain, filtered): (VecPathId, VecPathId) = {
            let scene = &mut self.gfx.as_mut().expect("gfx").vec_scene;
            (
                scene.push_path(crate::build_smoke::shape(
                    ShapeKind::Star,
                    [-5.0, -2.0],
                    [-1.0, 2.0],
                    &[5.0, 0.45],
                    [90, 170, 255],
                )),
                scene.push_path(crate::build_smoke::shape(
                    ShapeKind::Star,
                    [1.0, -2.0],
                    [5.0, 2.0],
                    &[5.0, 0.45],
                    [255, 150, 70],
                )),
            )
        };

        // O `sync` é quem cria as entidades — sem ele não há a quem pendurar nome nem filtro.
        let (bits_plain, bits_fx) = {
            let gfx = self.gfx.as_mut().expect("gfx");
            crate::vec_entities::sync(&mut gfx.sim, &mut gfx.vec_scene, &mut self.vec_entities);
            for (id, nome) in [(plain, "Fade"), (filtered, "Fade FX")] {
                let bits = self.vec_entities[&id];
                gfx.sim
                    .world_mut()
                    .entity_mut(ph2d_ecs::Entity::from_bits(bits))
                    .insert(Name::new(nome));
            }
            // ⭐ O brilho só na da DIREITA — a esquerda é o controlo.
            let armed = crate::fx_live::set_filter(
                &mut gfx.sim,
                &self.vec_entities,
                &[filtered],
                Some(VecFilter {
                    ops: vec![glow()],
                }),
            );
            assert_eq!(armed, 1, "[vec-fade-smoke] o filtro nao pendurou");
            (self.vec_entities[&plain], self.vec_entities[&filtered])
        };

        let doc = &mut self.timeline.doc;
        doc.rename_clip(0, "Fade".into());
        fade_track(doc, bits_plain);
        fade_track(doc, bits_fx);

        // A régua já aberta — o smoke é sobre o que a curva faz, não sobre achar o painel.
        // (`L` alterna, e é o mesmo gesto que a cena da física usa.)
        if let Some(hero) = self.gfx.as_mut().expect("gfx").hero_screen.as_mut() {
            hero.panel_visibility.insert("timeline", true);
        }

        eprintln!(
            "[vec-fade-smoke] duas estrelas com a MESMA curva de opacidade (1 -> 0 -> 1 em 4 s); a \
             da direita tem um brilho. DE' PLAY: as duas tem de desvanecer JUNTAS e voltar a \
             opaco. A da direita cravada opaca = o vao do memo de FX voltou."
        );
    }
}

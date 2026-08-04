//! **As consultas de LEITURA do mundo** — o que o overlay pergunta, e nada mais.
//!
//! Irmão de `sensors` e `contacts`, e pelo mesmo motivo: uma pergunta que a shell faz ao
//! mundo sem nunca escrever nele. Nasceu quando a linha d'água levou `world.rs` a 703 dos
//! seus 700 — e o corte é o que o arquivo já vinha fazendo, porque `spawn`/`step`/`rewind`
//! (o que MOVE) e "onde está a superfície desta poça?" (o que se OLHA) não são a mesma
//! responsabilidade.

use rapier2d::dynamics::RigidBodyHandle;

use super::PhysicsWorld;
use super::buoyancy;

impl PhysicsWorld {
    /// **A linha d'água de cada zona com empuxo** — o segmento onde a superfície corta
    /// o collider dela, em unidades de mundo.
    ///
    /// A metade VISÍVEL do empuxo. Uma zona de força ganha uma seta porque *para que
    /// lado sopra* não é inferível; um arrasto não tem direção para desenhar; mas uma
    /// poça **tem um lugar**, e até aqui o artista posicionava o tronco no olho.
    ///
    /// Sai da MESMA função que o empuxo usa (`buoyancy::waterline` → `surface_level`),
    /// nunca de uma re-derivação: duas respostas para *"onde está a água?"* divergiriam
    /// numa poça rotacionada ou sob gravidade lateral, que é precisamente onde ninguém
    /// confere. Vazio numa cena sem empuxo, e vazio sem gravidade.
    #[must_use]
    pub fn waterlines(&self) -> Vec<([f32; 2], [f32; 2])> {
        self.effectors
            .iter()
            .filter(|(_, e, _)| e.density > 0.0)
            .filter_map(|(handle, _, _)| {
                let collider = self
                    .bodies
                    .get(*handle)
                    .and_then(|b| b.colliders().first().copied())
                    .and_then(|h| self.colliders.get(h))?;
                buoyancy::waterline(collider.shape(), collider.position(), self.gravity)
            })
            .collect()
    }

    /// **A caixa que envolve TODAS as formas de um corpo**, em mundo —
    /// `(mínimos, máximos)`. `None` se o handle morreu ou se o corpo não tem
    /// forma nenhuma.
    ///
    /// ⚠️ **Todas as formas, e é a lição de 02/08 escrita como código:** a
    /// W-Compound deu a um corpo várias, e a frase *"um corpo tem exatamente um
    /// collider"* — que estava escrita em quatro lugares — virou quatro defeitos
    /// de classes diferentes. Uma caixa é sempre UMA, seja o corpo simples ou
    /// composto, e por isso é a forma certa de perguntar *"qual é a largura da
    /// cabeça dele?"* (o sensor de quina do player, W10).
    ///
    /// ⚠️ E ela é **conservadora**: a caixa de uma cápsula é mais larga que os
    /// ombros dela perto do topo, então uma assistência que a use dispara um
    /// pouco antes do estritamente necessário — o lado seguro de errar, e o
    /// preço de não precisar de um shapecast por forma.
    #[must_use]
    pub fn body_aabb(&self, handle: RigidBodyHandle) -> Option<([f32; 2], [f32; 2])> {
        use rapier2d::parry::bounding_volume::{Aabb, BoundingVolume};
        let rb = self.bodies.get(handle)?;
        let mut merged: Option<Aabb> = None;
        for &ch in rb.colliders() {
            let Some(c) = self.colliders.get(ch) else {
                continue;
            };
            let box_ = c.shape().compute_aabb(c.position());
            merged = Some(merged.map_or(box_, |a| a.merged(&box_)));
        }
        let a = merged?;
        Some(([a.mins.x, a.mins.y], [a.maxs.x, a.maxs.y]))
    }
}

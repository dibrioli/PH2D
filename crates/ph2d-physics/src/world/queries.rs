//! **As consultas de LEITURA do mundo** — o que o overlay pergunta, e nada mais.
//!
//! Irmão de `sensors` e `contacts`, e pelo mesmo motivo: uma pergunta que a shell faz ao
//! mundo sem nunca escrever nele. Nasceu quando a linha d'água levou `world.rs` a 703 dos
//! seus 700 — e o corte é o que o arquivo já vinha fazendo, porque `spawn`/`step`/`rewind`
//! (o que MOVE) e "onde está a superfície desta poça?" (o que se OLHA) não são a mesma
//! responsabilidade.

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
            .filter(|(_, e)| e.density > 0.0)
            .filter_map(|(handle, _)| {
                let collider = self
                    .bodies
                    .get(*handle)
                    .and_then(|b| b.colliders().first().copied())
                    .and_then(|h| self.colliders.get(h))?;
                buoyancy::waterline(collider.shape(), collider.position(), self.gravity)
            })
            .collect()
    }
}

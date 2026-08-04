//! **O componente do PLAYER DE PLATAFORMA** (W2) — a config autorada.
//!
//! Módulo irmão dos outros de `components/` pelo motivo de sempre (LOC e
//! isolamento), e a fronteira dele é a regra mais importante desta wave:
//!
//! # ⚠️ CONFIG, nunca estado vivo
//!
//! Um controlador de plataforma carrega estado por-tick — `is_grounded`, o
//! coyote timer, o jump buffer. **Nada disso mora aqui.** O `canonicalize` do
//! undo ordena as entidades pelos BYTES dos componentes, então um campo que
//! muda por tick faria **cada frame virar um passo de undo** (o ADR-0131 escreve
//! essa lei, e o `PhysicsJoint` já a honra).
//!
//! O estado vivo mora na PONTE, ao lado do `grab` — e a partir da W7 ele deixa
//! de ser guardado e passa a ser **derivado da fita de entrada**, que é o que o
//! torna reproduzível num scrub.

use bevy_ecs::component::Component;
use ph2d_platformer::RideConfig;
use serde::{Deserialize, Serialize};

/// **Este corpo é um player de plataforma.**
///
/// Presença = o comportamento existe; os campos são os ganhos da perna
/// ([`RideConfig`], a lei pura). Ausente é o mundo de antes desta wave, byte a
/// byte — nenhum corpo sem o componente muda de trajetória.
///
/// ⚠️ **Só faz sentido num corpo `Dynamic`**, e por FÍSICA, não por gosto: a
/// mola é um impulso, e um impulso não move um corpo estático nem um kinematic
/// (massa infinita — o fato que a W-BakeJoint mediu e que faz a MÃO recusar os
/// dois). A row do Inspector será oferecida para Dynamic apenas, e a ponte
/// recusa em silêncio o resto.
///
/// ⚠️ **Componente NOVO ⇒ blob-key própria ⇒ `PROJECT_SCHEMA` NÃO bumpa** (o
/// precedente do `PhysicsJoint`/W3): um arquivo antigo simplesmente não o tem, e
/// o load o deixa ausente. O que bumpa é apendar campo a um componente que já
/// existe, porque o postcard é posicional.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlatformPlayer {
    /// A que altura o personagem paira, medida do CENTRO do corpo para baixo.
    pub float_height: f32,
    /// Quanto acima da altura de repouso a mola ainda age (ver [`RideConfig`]).
    pub cling_distance: f32,
    /// Rigidez da perna, em aceleração-por-metro.
    pub spring_strength: f32,
    /// Amortecimento, em fração da velocidade relativa removida por tick.
    ///
    /// ⚠️ Tem TETO medido ([`RideConfig::MAX_DAMPING`]) — acima dele o boost
    /// inverte a velocidade em vez de matá-la, e o personagem pipoca.
    pub spring_damping: f32,
}

impl PlatformPlayer {
    /// A config da lei que este componente descreve.
    ///
    /// Existe para que a ponte **não** remonte o `RideConfig` campo a campo:
    /// duas cópias da mesma tradução divergem no dia em que um campo novo entra
    /// só numa delas.
    #[must_use]
    pub fn ride(&self) -> RideConfig {
        RideConfig {
            float_height: self.float_height,
            cling_distance: self.cling_distance,
            spring_strength: self.spring_strength,
            spring_damping: self.spring_damping,
        }
    }
}

impl Default for PlatformPlayer {
    /// ⚠️ **Ponto de partida, não default de produto** — os números que shipam
    /// saem da varredura da wave, com a tabela ao lado (CLAUDE.md §0).
    fn default() -> Self {
        let r = RideConfig::STARTING_POINT;
        Self {
            float_height: r.float_height,
            cling_distance: r.cling_distance,
            spring_strength: r.spring_strength,
            spring_damping: r.spring_damping,
        }
    }
}

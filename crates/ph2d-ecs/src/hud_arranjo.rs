//! **O arranjo do HUD** — as relações entre IRMÃOS de um [`crate::UiCanvas`] (ao lado, centrado, não invade,
//! largura do texto), resolvidas pelo Cassowary em [`ph2d_hud_layout`] DEPOIS das âncoras. Mora no canvas e cita
//! os filhos pelo [`StableId`] (o `Entity` é de alocação). As folgas estão em unidades LOCAIS do canvas.
//! Desenho: `docs/BlenderConstraints/06_W5_resto.md` §3.3; porquê: ADR-0194.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

pub use ph2d_hud_layout::{Eixo, Forca, Lado, Relacao, na_grelha};

use crate::{Entity, SimComponent, StableId};

/// As relações de um canvas, pela ordem em que o artista as fez (a ordem é a de inserção no solver).
#[derive(Component, Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct UiArranjo {
    pub lacos: Vec<(Relacao<StableId>, Forca)>,
}

impl SimComponent for UiArranjo {}

impl UiArranjo {
    /// ⭐ A porta da ferramenta: junta `nova`, trocando a que o mesmo par já tinha NO MESMO EIXO (uma seta nova
    /// entre os mesmos dois substitui a antiga; ao lado em X e centrado em Y convivem). Recusa — e não muda nada — o que fecharia um círculo de
    /// lados ([`ph2d_hud_layout::contradiz`]): o solver aceitá-lo-ia e violaria um `REQUIRED` em silêncio.
    pub fn junta(&mut self, nova: Relacao<StableId>, forca: Forca) -> bool {
        let par = |r: &Relacao<StableId>| {
            let (a, b) = r.ids();
            (a, b.unwrap_or(StableId::NONE), r.eixo())
        };
        let (a, b, eixo) = par(&nova);
        let mesmo = |r: &Relacao<StableId>| {
            let (x, y, e) = par(r);
            e == eixo && ((x, y) == (a, b) || (x, y) == (b, a))
        };
        let restantes: Vec<_> = self
            .lacos
            .iter()
            .filter(|(r, _)| !mesmo(r))
            .cloned()
            .collect();
        let so: Vec<_> = restantes.iter().map(|(r, _)| *r).collect();
        if ph2d_hud_layout::contradiz(&so, &nova) {
            return false;
        }
        self.lacos = restantes;
        self.lacos.push((nova, forca));
        true
    }

    /// Apaga a relação `i`; `false` se não existe.
    pub fn apaga(&mut self, i: usize) -> bool {
        (i < self.lacos.len())
            .then(|| self.lacos.remove(i))
            .is_some()
    }
}

/// ⭐ O que o passe do arranjo VIU num quadro, por canvas — o retrato que a ferramenta do HUD desenha e acerta
/// (ela não tem o mundo no desenho). Preview: nunca gravado.
#[derive(Clone, Debug, PartialEq)]
pub struct Visto {
    pub canvas: Entity,
    /// Mundo por unidade local do canvas (as folgas gravam-se em unidades locais).
    pub escala: [f64; 2],
    pub lacos: Vec<(Relacao<StableId>, Forca)>,
    pub filhos: Vec<FilhoVisto>,
}

/// Um filho do canvas como se DESENHA neste quadro (depois das âncoras e do arranjo).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FilhoVisto {
    pub kid: Entity,
    pub id: StableId,
    /// Caixa de mundo `(min, max)`.
    pub caixa: ([f64; 2], [f64; 2]),
    /// É um texto: ao entrar numa relação ganha «largura do texto» (encolhe antes de invadir).
    pub texto: bool,
}

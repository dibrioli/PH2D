//! **A forma gravada da [`SkinBind`] até ao `PROJECT_SCHEMA` `210`** — o [`Tendon`] sem o
//! `comprimento` do bind. ⛔ Congelada: só se lê; mapeia para `comprimento = 0` (a pele não estica,
//! e o ficheiro antigo continua igual ao que era).

use serde::Deserialize;

use ph2d_ecs::StableId;

use crate::{CorreccaoDePeso, SkinBind, SkinLaw, Tendon};

/// O último `PROJECT_SCHEMA` com o tendão sem comprimento.
pub const ATE: u32 = 210;

/// O [`Tendon`] da v210.
#[derive(Deserialize)]
pub struct TendonV210 {
    pub bone: StableId,
    pub rest: [f64; 6],
}

/// A [`SkinBind`] da v210 — os mesmos campos, pela mesma ordem.
#[derive(Deserialize)]
pub struct SkinBindV210 {
    pub source: Vec<u8>,
    pub tendons: Vec<TendonV210>,
    pub law: SkinLaw,
    pub correcoes: Vec<CorreccaoDePeso>,
}

impl From<SkinBindV210> for SkinBind {
    fn from(v: SkinBindV210) -> Self {
        Self {
            source: v.source,
            tendons: (v.tendons.into_iter())
                .map(|t| Tendon {
                    bone: t.bone,
                    rest: t.rest,
                    comprimento: 0.0,
                })
                .collect(),
            law: v.law,
            correcoes: v.correcoes,
        }
    }
}

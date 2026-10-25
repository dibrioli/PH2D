//! **As entradas da pilha lêem-se pela VERSÃO de quem as gravou** (F31 §B.1, v23 → v24).
//!
//! A v24 deu a cada [`PaintEntry`] e [`FxEntry`] um `id` ([`ItemId`], o endereço durável da chave
//! do Auto Key). O postcard é posicional ⇒ bytes v23 lidos com o tipo v24 são lixo bem-formado.
//! O idioma é o do [`crate::stroke_wire`]: quem conhece a versão do ficheiro abre o âmbito
//! [`with_pilha_layout_v23`], e dentro dele as duas entradas lêem-se sem o id (`NONE`, que o
//! passe de atribuição do `ph2d-app-vec` preenche no primeiro quadro).
//!
//! ⚠️ O âmbito é da THREAD e só vale durante a chamada; os `VecPath` guardados em blobs de
//! componentes (Envelope, pele) regravam-se no load (`migra_pilha_v23`).

use std::cell::Cell;

use ph2d_blend_mode::BlendMode;
use ph2d_item_id::ItemId;
use serde::{Deserialize, Deserializer};

use crate::effect::{FxEntry, PathEffect};
use crate::{Opacity, PaintEntry, PaintKind};

thread_local! {
    static V23: Cell<u32> = const { Cell::new(0) };
}

/// Corre `f` a ler toda entrada da pilha na forma do `VEC_SCENE_SCHEMA_VERSION` 23 (sem id).
/// Reentrante; o âmbito fecha mesmo num `panic`.
pub fn with_pilha_layout_v23<T>(f: impl FnOnce() -> T) -> T {
    let _v23 = PilhaLayoutV23::enter();
    f()
}

/// O mesmo âmbito como GUARDA (o leitor do `.ph2dproj`).
#[must_use = "o âmbito fecha quando o guarda cai"]
pub struct PilhaLayoutV23(());

impl PilhaLayoutV23 {
    pub fn enter() -> Self {
        V23.with(|c| c.set(c.get() + 1));
        Self(())
    }
}

impl Drop for PilhaLayoutV23 {
    fn drop(&mut self) {
        V23.with(|c| c.set(c.get() - 1));
    }
}

fn reading_v23() -> bool {
    V23.with(|c| c.get() > 0)
}

/// ⛔ Congelada: a forma v23.
#[derive(Deserialize)]
struct PaintEntryV23 {
    kind: PaintKind,
    enabled: bool,
    opacity: Opacity,
    blend: BlendMode,
    offset: [f64; 2],
    dilate: f64,
    dilate_join: u8,
}

/// O espelho da forma VIVA (a ordem do `Serialize` derivado; gate `a_forma_viva_da_pilha_le_o_que_grava`).
#[derive(Deserialize)]
struct PaintEntryV24 {
    kind: PaintKind,
    enabled: bool,
    opacity: Opacity,
    blend: BlendMode,
    offset: [f64; 2],
    dilate: f64,
    dilate_join: u8,
    id: ItemId,
}

#[derive(Deserialize)]
struct FxEntryV23 {
    effect: PathEffect,
    enabled: bool,
}

#[derive(Deserialize)]
struct FxEntryV24 {
    effect: PathEffect,
    enabled: bool,
    id: ItemId,
}

impl From<PaintEntryV23> for PaintEntry {
    fn from(o: PaintEntryV23) -> Self {
        Self {
            kind: o.kind,
            enabled: o.enabled,
            opacity: o.opacity,
            blend: o.blend,
            offset: o.offset,
            dilate: o.dilate,
            dilate_join: o.dilate_join,
            id: ItemId::NONE,
        }
    }
}

impl From<PaintEntryV24> for PaintEntry {
    fn from(o: PaintEntryV24) -> Self {
        Self {
            kind: o.kind,
            enabled: o.enabled,
            opacity: o.opacity,
            blend: o.blend,
            offset: o.offset,
            dilate: o.dilate,
            dilate_join: o.dilate_join,
            id: o.id,
        }
    }
}

impl<'de> Deserialize<'de> for PaintEntry {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        if reading_v23() {
            PaintEntryV23::deserialize(d).map(Into::into)
        } else {
            PaintEntryV24::deserialize(d).map(Into::into)
        }
    }
}

impl<'de> Deserialize<'de> for FxEntry {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        if reading_v23() {
            let o = FxEntryV23::deserialize(d)?;
            Ok(Self {
                effect: o.effect,
                enabled: o.enabled,
                id: ItemId::NONE,
            })
        } else {
            let o = FxEntryV24::deserialize(d)?;
            Ok(Self {
                effect: o.effect,
                enabled: o.enabled,
                id: o.id,
            })
        }
    }
}

#[cfg(test)]
#[path = "pilha_wire_tests.rs"]
mod tests;

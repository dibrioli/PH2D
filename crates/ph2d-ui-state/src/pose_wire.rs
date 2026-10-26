//! **A pose lê-se pela VERSÃO de quem a gravou** (F32 §2, `PROJECT_SCHEMA` 196 → 197).
//!
//! O `197` deu à pose o id de cada filtro ([`crate::ObjectPose::filter_ids`], no FIM). O postcard é
//! posicional ⇒ uma pose `196` lida com o tipo de agora come os bytes da pose seguinte. O idioma é o
//! do `ph2d_vec_scene::PilhaLayoutV23`: quem conhece a versão do ficheiro abre o âmbito
//! [`PoseLayoutV196`], e dentro dele o campo lê-se sem tocar nos bytes (vazio) — e grava-se sem
//! nenhum, que é como um teste escreve um ficheiro antigo sem mentir sobre a forma dele.

use std::cell::Cell;

use ph2d_item_id::ItemId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

thread_local! {
    static V196: Cell<u32> = const { Cell::new(0) };
}

/// O âmbito como GUARDA (o leitor do `.ph2dproj`). Reentrante; fecha mesmo num `panic`.
#[must_use = "o âmbito fecha quando o guarda cai"]
pub struct PoseLayoutV196(());

impl PoseLayoutV196 {
    pub fn enter() -> Self {
        V196.with(|c| c.set(c.get() + 1));
        Self(())
    }
}

impl Drop for PoseLayoutV196 {
    fn drop(&mut self) {
        V196.with(|c| c.set(c.get() - 1));
    }
}

#[allow(clippy::ptr_arg)] // a assinatura do `serialize_with` é `&Vec<_>`
pub(crate) fn grava_filter_ids<S: Serializer>(ids: &Vec<ItemId>, s: S) -> Result<S::Ok, S::Error> {
    if V196.with(|c| c.get() > 0) {
        return ().serialize(s);
    }
    ids.serialize(s)
}

pub(crate) fn filter_ids<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<ItemId>, D::Error> {
    if V196.with(|c| c.get() > 0) {
        return Ok(Vec::new());
    }
    Vec::deserialize(d)
}

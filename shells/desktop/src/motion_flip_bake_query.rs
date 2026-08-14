//! **Consultar o bake de Flip** — a metade que responde *quais tiles existem* e *qual
//! delas serve este pedido*, irmã de `motion_flip_bake` (que responde *como uma tile é
//! produzida*). Cortado por assunto pelo teto de LOC da shell (HR-18).
//!
//! FILHO via `#[path]`, então `super` é o módulo do bake e o `impl` alcança os campos
//! privados `cache`/`asked`.

use super::{FlipObjectBake, FlipTile};
use ph2d_flip::FlipObjectId;

impl FlipObjectBake {
    /// The `name -> tile` map the membrane publishes individually (the picker path).
    /// Read-only — the bake ran at the fx phase. Only NAMED entries are yielded: an
    /// unnamed group child has a tile (for the group stamp) but nothing to type into a
    /// node.
    pub(crate) fn tiles(&self) -> impl Iterator<Item = (&str, FlipTile)> {
        // Só o pedido NÃO deslocado (bits `0`): o nome cru significa "agora", e um
        // deslocado publicado aqui sobrescreveria o canal de todo mundo.
        self.asked
            .iter()
            .filter(|((_, bits), _)| *bits == 0)
            .filter_map(|((oid, _), frame)| self.cache.get(&(*oid, *frame)))
            .filter_map(|b| {
                b.name.as_deref().map(|n| {
                    (
                        n,
                        FlipTile {
                            texture_id: b.texture_id,
                            size: b.size,
                        },
                    )
                })
            })
    }

    /// A tile de um objeto NOMEADO **num offset** — a metade que o canal deslocado
    /// publica. `None` quando o nome não é um Flip, ou quando o bake não produziu
    /// geometria naquele quadro (pulado, nunca adivinhado).
    pub(crate) fn tile_named_shifted(&self, name: &str, offset: f32) -> Option<FlipTile> {
        let oid = self
            .cache
            .iter()
            .find(|(_, b)| b.name.as_deref() == Some(name))
            .map(|((id, _), _)| *id)?;
        let frame = *self.asked.get(&(oid, offset.to_bits()))?;
        self.cache.get(&(oid, frame)).map(|b| FlipTile {
            texture_id: b.texture_id,
            size: b.size,
        })
    }

    /// The baked tile for ONE Flip object by its `FlipObjectId` (the raw `u64` a
    /// `FlipObjectRef` carries), or `None` if it isn't baked (doc 86 §2 A4). A group
    /// child that is a Flip object resolves its appearance here — by its drawing id, so
    /// an unnamed child still resolves.
    pub(crate) fn tile_for_id(&self, id: u64) -> Option<FlipTile> {
        let oid = FlipObjectId(id);
        let frame = *self.asked.get(&(oid, 0u32))?;
        self.cache.get(&(oid, frame)).map(|b| FlipTile {
            texture_id: b.texture_id,
            size: b.size,
        })
    }

    /// Semeia uma tile NOMEADA respondendo a um pedido DESLOCADO — para os gates
    /// headless da membrana, que dirigem `publish_shifted` sem GPU. O quadro é `1`
    /// (qualquer um distinto do `0` do pedido cru) porque o que este seed encena é a
    /// resolução `(objeto, offset) -> quadro`, não a aritmética que a produz.
    #[cfg(test)]
    pub(crate) fn seed_named_shift_for_test(
        &mut self,
        id: u64,
        name: &str,
        offset: f32,
        texture_id: u32,
        size: [f32; 2],
    ) {
        let oid = FlipObjectId(id);
        let frame: ph2d_flip::Frame = 1;
        self.asked.insert((oid, offset.to_bits()), frame);
        self.cache.insert(
            (oid, frame),
            super::FlipBaked {
                name: Some(name.to_string()),
                key: 0,
                texture_id,
                size,
                thumb: ph2d_panel_motion_graph::PreviewThumb {
                    rgba: std::sync::Arc::new(Vec::new()),
                    w: 0,
                    h: 0,
                },
            },
        );
    }

    /// Seed a fake tile under `id` — for headless membrane gates that drive
    /// `resolve_leaf` without a GPU bake.
    #[cfg(test)]
    pub(crate) fn seed_for_test(&mut self, id: u64, texture_id: u32, size: [f32; 2]) {
        self.asked.insert((FlipObjectId(id), 0u32), 0);
        self.cache.insert(
            (FlipObjectId(id), 0),
            super::FlipBaked {
                name: None,
                key: 0,
                texture_id,
                size,
                thumb: ph2d_panel_motion_graph::PreviewThumb {
                    rgba: std::sync::Arc::new(Vec::new()),
                    w: 0,
                    h: 0,
                },
            },
        );
    }

    /// The baked-tile THUMBNAIL for `texture_id`, or `None` (doc 86 A5) — the Flip
    /// twin of `ObjectBake::thumbnail_for`, so the membrane looks a source node's tid
    /// up in BOTH bakes without caring which medium answered.
    pub(crate) fn thumbnail_for(
        &self,
        texture_id: u32,
    ) -> Option<ph2d_panel_motion_graph::PreviewThumb> {
        self.cache
            .values()
            .find(|b| b.texture_id == texture_id)
            .map(|b| b.thumb.clone())
    }
}

//! ⭐ **As camadas de cada imagem, para a HIERARQUIA** (doc 01 §4.3 — o conteúdo da Imagem). O
//! Painter é quem as tem: a do documento em mãos e as dos documentos guardados (`doc_cache`).

use super::*;
use ph2d_editor_core::hier_rows::ContentItem;

impl PainterTool {
    /// As imagens de que o Painter tem documento em camadas: a ligada e as guardadas.
    pub fn layered_entities(&self) -> impl Iterator<Item = u64> + '_ {
        self.bound_doc
            .filter(|_| !self.canvas_rgba.is_empty())
            .into_iter()
            .chain(self.doc_cache.keys().copied())
    }

    /// As camadas do documento de `entity`, de CIMA para baixo como no painel Layers (grupos
    /// aninhados, máscaras fora — são do dono). `None` = o Painter não tem documento em camadas
    /// dela: uma imagem nunca pintada é só os píxeis da sprite.
    #[must_use]
    pub fn layer_outline_of(&self, entity: u64) -> Option<Vec<ContentItem>> {
        let stack = if self.bound_doc == Some(entity) && !self.canvas_rgba.is_empty() {
            &self.layers
        } else {
            self.doc_cache.get(&entity)?.layers()
        };
        let mut out = Vec::with_capacity(stack.len());
        push_outline(stack, stack.root(), 0, &mut out);
        Some(out)
    }

    /// ⭐ **Põe a camada `layer` de `entity` em mãos.** O documento pode ainda não estar ligado (o
    /// modo acabou de abrir e a ligação vem no mesmo quadro, mais adiante): então fica pendente e
    /// o [`Self::bind_document`] cumpre-o. `false` = a camada já não existe.
    pub fn open_layer_of(&mut self, entity: u64, layer: u64) -> bool {
        let id = RtLayerId(layer);
        if self.bound_doc == Some(entity) && !self.canvas_rgba.is_empty() {
            let there = self.layers.get(id).is_some();
            self.select_layer(id);
            return there;
        }
        let there = self
            .doc_cache
            .get(&entity)
            .is_some_and(|d| d.layers().get(id).is_some());
        if there {
            self.pending_layer = Some((entity, id));
        }
        there
    }

    /// O pedido do [`Self::open_layer_of`], cumprido quando `entity` ficou ligado — e LARGADO quando
    /// é outra a que liga: voltar mais tarde à primeira por outro caminho não abre a camada antiga.
    pub(crate) fn serve_pending_layer(&mut self, entity: u64) {
        if let Some((e, id)) = self.pending_layer.take()
            && e == entity
        {
            self.select_layer(id);
        }
    }

    /// ⭐ **Larga o pedido** — o Paint que o fez saiu antes de o documento ligar (`paint_mode::leave`).
    pub fn forget_pending_layer(&mut self) {
        self.pending_layer = None;
    }
}

fn push_outline(stack: &LayerStack, ids: &[RtLayerId], depth: usize, out: &mut Vec<ContentItem>) {
    if depth > crate::layers::MAX_GROUP_DEPTH {
        return;
    }
    let active = stack.active();
    for &id in ids {
        let Some(layer) = stack.get(id) else { continue };
        if matches!(layer.kind, LayerKind::Mask(_)) {
            continue;
        }
        out.push(ContentItem {
            key: id.0,
            name: layer.name.clone(),
            depth: u8::try_from(depth).unwrap_or(u8::MAX),
            active: active == Some(id),
        });
        if let LayerKind::Group(g) = &layer.kind {
            push_outline(stack, &g.children, depth + 1, out);
        }
    }
}

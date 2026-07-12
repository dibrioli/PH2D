//! **Persistência** do documento pintado — o que sobrevive a um restart.
//!
//! O Painter guarda seus documentos por `Entity::to_bits()` (o id de ALOCAÇÃO do ECS), que morre no
//! save/load: o restore despawna tudo e recria com bits novos. Então a shell carimba cada sprite
//! pintado com um `ph2d_ecs::PaintedDoc(u32)` — uma identidade **estável**, que viaja no snapshot — e
//! é por ela que os documentos são coletados aqui e devolvidos no load.
//!
//! [`PaintedDocument`] é o documento inteiro e nada mais: as camadas, os pixels de cada uma, o relevo
//! e a cobertura. **Não** carrega undo (histórico é da sessão), nem caches (reconstruídos), nem o
//! composite (é derivado). É o mesmo recorte que o `StashedDoc` já faz para trocar de sprite — a
//! diferença é só que este atravessa o disco, e por isso é serde.
//!
//! **O relevo é a razão de isto não ser opcional.** Um sprite é RGBA e nada mais; o `Apply` assa a
//! luz nele e joga a altura fora ("bake the look, lose the editability" — é o que Apply *é*). Se o
//! projeto salvasse só o sprite, todo impasto voltaria como uma foto de tinta grossa: sem camadas,
//! sem espessura, sem como continuar esculpindo.

use crate::compositor::LayerImage;
use crate::layers::LayerStack;
use crate::tool::PainterTool;
use crate::tool::RtLayerId;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Um documento pintado, pronto para o disco. Postcard é posicional: **campo novo entra no FIM**.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PaintedDocument {
    /// A identidade estável (o `PaintedDoc(u32)` que a shell carimbou no sprite).
    pub id: u32,
    /// Estrutura: camadas, grupos, máscaras, ajustes, blend, opacidade, visibilidade.
    pub layers: LayerStack,
    /// Pixels da camada ATIVA (o `canvas_rgba` do tool — a única que não vive em `images`).
    pub canvas_rgba: Vec<u8>,
    /// Pixels das demais camadas.
    pub images: BTreeMap<RtLayerId, LayerImage>,
    /// **Impasto**: o relevo por camada. Vazio ⇒ documento sem escultura (custo zero).
    pub heights: BTreeMap<RtLayerId, Vec<f32>>,
    /// **Impasto**: a cobertura de tinta por camada — o que a luz pesa. Anda junto com o relevo.
    pub covers: BTreeMap<RtLayerId, Vec<u8>>,
    /// Tamanho do canvas em pixels.
    pub size: (u32, u32),
}

impl PainterTool {
    /// Todos os documentos que o Painter tem em mãos: o LIGADO (os campos de topo) e os guardados
    /// (`doc_cache`), na ordem dos ids.
    ///
    /// `ids` mapeia `entity bits → PaintedDoc(u32)`, e vem da shell (o ECS é quem sabe). Um documento
    /// cuja entidade não está no mapa é **pulado, não inventado**: um id atribuído aqui não estaria no
    /// snapshot, e o load não teria como devolvê-lo a sprite nenhum.
    #[must_use]
    pub fn collect_documents(&self, ids: &BTreeMap<u64, u32>) -> Vec<PaintedDocument> {
        let mut out = Vec::new();
        if let Some(bound) = self.bound_doc
            && let Some(&id) = ids.get(&bound)
        {
            out.push(PaintedDocument {
                id,
                layers: self.layers.clone(),
                canvas_rgba: self.canvas_rgba.as_ref().clone(),
                images: self.images.clone(),
                heights: self.heights.clone(),
                covers: self.covers.clone(),
                size: self.source_size,
            });
        }
        for (bits, doc) in &self.doc_cache {
            if let Some(&id) = ids.get(bits) {
                out.push(doc.to_painted(id));
            }
        }
        out.sort_by_key(|d| d.id);
        out
    }

    /// As entidades para as quais o Painter tem documento: a LIGADA e as guardadas. É a lista que a
    /// shell usa para carimbar a identidade estável antes de coletar.
    #[must_use]
    pub fn document_entities(&self) -> Vec<u64> {
        let mut v: Vec<u64> = self.doc_cache.keys().copied().collect();
        if let Some(b) = self.bound_doc
            && !v.contains(&b)
        {
            v.push(b);
        }
        v.sort_unstable();
        v
    }

    /// Devolve um documento carregado do disco ao sprite `entity`.
    ///
    /// Vai para o `doc_cache` (não para os campos de topo): o sprite ainda não está ligado, e é o
    /// `bind_document` — quando o artista selecionar esse sprite com o Painter — que o traz para a
    /// frente, pelo caminho normal. Assim o load não precisa saber nada sobre qual documento está
    /// ligado, e não existe um segundo caminho de restore para divergir do primeiro.
    pub fn install_document(&mut self, entity: u64, doc: PaintedDocument) {
        self.doc_cache.insert(
            entity,
            crate::tool::documents::StashedDoc::from_painted(doc),
        );
        // Se o sprite já está ligado (a shell pode carregar com o Painter ativo), o documento vivo é o
        // do projeto ANTIGO — solte-o, para que o próximo `bind_document` puxe o que acabou de chegar.
        if self.bound_doc == Some(entity) {
            self.bound_doc = None;
        }
    }

    /// Os pixels que o sprite deve mostrar depois de um load — o documento instalado, composto e (se
    /// houver relevo) **iluminado**.
    ///
    /// Deliberadamente NÃO existe um segundo caminho de composite aqui: isto liga o documento pelo
    /// `bind_document` de sempre e lê o preview de sempre (`take_preview_arc`, que já assa a luz — o
    /// fast-path trivial é justamente gateado por `impasto_visible`). Escrever um bake próprio para o
    /// load seria criar um segundo caminho para a mesma imagem, e é assim que dois caminhos divergem
    /// seis meses depois — a lição que esta linha já pagou duas vezes.
    ///
    /// `None` quando não há documento instalado para essa entidade (ou o canvas é vazio).
    #[must_use]
    pub fn baked_document_pixels(&mut self, entity: u64) -> Option<(Vec<u8>, u32, u32)> {
        let (w, h) = self.doc_cache.get(&entity)?.size();
        if w == 0 || h == 0 {
            return None;
        }
        // Os pixels passados aqui são ignorados: o `bind_document` acha o documento no cache e restaura
        // aquele — este é o mesmo caminho que trocar de sprite percorre.
        self.bind_document(entity, Vec::new(), w, h);
        self.preview_dirty = true; // um documento recém-restaurado ainda não foi composto nesta sessão
        let (px, pw, ph) = self.take_preview_arc()?;
        Some((px.as_ref().clone(), pw, ph))
    }
}

/// O `canvas_rgba` de um documento vem como `Arc` no tool e como `Vec` no disco — uma conversão só,
/// aqui, para os dois lados não precisarem se conhecer.
pub(super) fn arc_pixels(v: Vec<u8>) -> Arc<Vec<u8>> {
    Arc::new(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};

    fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
        CanvasPointer {
            pos,
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        }
    }

    /// A painted document — layers, pixels AND relief — survives a real postcard round trip and comes
    /// back attached to its sprite, still editable.
    ///
    /// This is the gate for the whole feature. Until now the project file saved a sprite pointing at a
    /// runtime texture id that dies with the process: painting + Ctrl+S + reopen came back EMPTY. And a
    /// save that only kept the flattened sprite would be little better — the sculpting would return as
    /// a photograph of thick paint, with no layers and no height to keep working on. So the assertion
    /// is deliberately about the RELIEF, not just the pixels.
    #[test]
    fn a_painted_document_survives_the_disk_with_its_relief() {
        const A: u64 = 7; // sprite entity bits in the "old" session
        const B: u64 = 999_001; // …and the DIFFERENT bits the same sprite gets after a load
        const DOC: u32 = 3; // the stable `PaintedDoc(u32)` the shell stamps on it

        // ── Session 1: paint a sculpted stroke on a second layer ───────────────────────────────
        let mut t = PainterTool::default();
        t.bind_document(A, vec![255u8; 32 * 32 * 4], 32, 32);
        t.add_raster_layer("Layer 2"); // a real multi-layer doc (the case a bake would flatten)
        t.set_brush_size_px(6.0);
        t.toggle_brush_impasto();
        t.set_brush_impasto_depth(0.9);
        t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Down));
        t.on_canvas_pointer(cp([16.0, 16.0], PointerPhase::Up));

        // …and then TUNE that relief at the layer (§10.8): a depth and a composite mode. These are
        // composite parameters, so nothing about the height field records them — if the file forgot
        // them, a reopened painting would come back with its relief at full strength and piling up,
        // which reads to the artist as "my settings were thrown away".
        let sculpted = t.layers.active().expect("a layer");
        t.set_layer_impasto_depth(sculpted, -0.4);
        t.set_layer_impasto_composite(sculpted, crate::layers::ReliefComposite::Level);

        let layers_before = t.layers.root().len();
        assert!(layers_before >= 2, "the document really is multi-layer");
        let relief_before: Vec<f32> = t
            .heights
            .values()
            .next()
            .cloned()
            .expect("the stroke sculpted relief");
        assert!(
            relief_before.iter().any(|&h| h.abs() > 0.2),
            "…and the relief is real"
        );

        // ── Save: collect through the shell's stable-id map, then serialise for real ───────────
        let ids: BTreeMap<u64, u32> = [(A, DOC)].into_iter().collect();
        let docs = t.collect_documents(&ids);
        assert_eq!(docs.len(), 1, "the bound document was collected");
        let bytes = postcard::to_allocvec(&docs).expect("a painted document serialises");
        let back: Vec<PaintedDocument> =
            postcard::from_bytes(&bytes).expect("…and comes back off the disk");
        assert_eq!(back, docs, "byte-for-byte the same document");

        // ── Session 2: a FRESH tool, and the sprite has different entity bits (the restore

        // respawns everything). The stable id is what carries the document across. ─────────────
        let mut t2 = PainterTool::default();
        assert!(t2.heights.is_empty(), "a fresh tool knows nothing");
        let doc = back
            .into_iter()
            .find(|d| d.id == DOC)
            .expect("by stable id");
        t2.install_document(B, doc);

        // The sprite can be shown before the Painter is ever activated…
        let (px, w, h) = t2
            .baked_document_pixels(B)
            .expect("the loaded document composites");
        assert_eq!((w, h), (32, 32));
        assert_eq!(px.len(), (32 * 32 * 4) as usize);

        // …and, the point of all this, it is still a DOCUMENT: the layers are there, and so is the
        // height field, so the artist can keep sculpting where they left off.
        assert_eq!(
            t2.layers.root().len(),
            layers_before,
            "the layers came back — not a flattened bake"
        );
        let relief_after: Vec<f32> = t2
            .heights
            .values()
            .next()
            .cloned()
            .expect("the relief came back too");
        assert_eq!(relief_after, relief_before, "…exactly as it was sculpted");

        // The per-layer composite came back with it. (Postcard is positional, which is why the project
        // schema had to go 3 → 4 the moment `Layer` grew these fields: a v3 file would have read the
        // NEXT field's bytes as this one's.)
        let restored = t2
            .layers
            .all_ids()
            .find_map(|id| t2.layers.get(id).filter(|l| l.has_relief))
            .expect("the sculpted layer came back, and it knows it carries relief");
        assert!(
            (restored.impasto_depth - (-0.4)).abs() < 1e-6,
            "the layer's Impasto depth survived the disk (got {})",
            restored.impasto_depth
        );
        assert_eq!(
            restored.impasto_composite,
            crate::layers::ReliefComposite::Level,
            "…and so did how it meets the relief below it"
        );
    }
}

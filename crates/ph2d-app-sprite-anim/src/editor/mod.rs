//! **O editor de animações** — uma aba por sprite (plano GameEngineFuncional 07, «Desenho aprovado» 2–3).
//!
//! Uma família de documentos ([`DocumentSurface`]) no molde do editor de scripts: a shell instala
//! [`AnimDocs`] uma vez e chama [`frame`] uma vez por quadro, ANTES dos snapshots. O «Editar animações…»
//! do cartão chega como `AnimFieldEdit::OpenEditor`, que a shell entrega a [`request_open`].

pub mod doc;
pub mod paint;

use std::collections::BTreeMap;

use doc::{AnimDoc, EditorSnapshot};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue, apply_editor_commands};
use ph2d_ecs::{Entity, Resource, SimWorld};
use ph2d_editor_core::doc_surface::{
    Compose, DocTab, DocumentSurface, Key, KeyInput, KeyOutcome, PointerInput,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{AnimFieldEdit, HeroScreen, Toast, ToastQueue};
use ph2d_host::{PointerButton, PointerKind};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// As sprites cujo editor alguém pediu neste quadro (o «Editar animações…» do cartão).
#[derive(Resource, Default, Debug)]
pub struct AnimEditorRequests(pub Vec<u64>);

/// **A porta da shell para o `OpenEditor`.** Devolve `true` se a edição era esse pedido (e já foi
/// registada): a shell salta então o `apply_anim_edit`.
pub fn request_open(sim: &mut SimWorld, entity_bits: u64, edit: &AnimFieldEdit) -> bool {
    if !matches!(edit, AnimFieldEdit::OpenEditor) {
        return false;
    }
    sim.world_mut()
        .get_resource_or_init::<AnimEditorRequests>()
        .0
        .push(entity_bits);
    true
}

/// **Uma imagem acabou de ser largada sobre a sprite `bits` (ou de nascer nela)**: se a transparência
/// propõe uma grelha de várias células diferente da que a sprite tem, abre o editor dela — a faixa
/// «Parece C × L — Usar» é a proposta, e o artista confirma (plano 07, «Desenho aprovado» 3).
/// `rgba` é a imagem reta, `w × h × 4`. Devolve se pediu o editor.
pub fn propose_after_drop(sim: &mut SimWorld, bits: u64, rgba: &[u8], w: u32, h: u32) -> bool {
    if rgba.len() != (w as usize) * (h as usize) * 4 {
        return false;
    }
    let alpha: Vec<u8> = rgba.chunks_exact(4).map(|p| p[3]).collect();
    let Some(g) = ph2d_sprite_sheet::guess_grid(&alpha, w, h) else {
        return false;
    };
    let grid = sim
        .world()
        .get::<ph2d_ecs::SpriteGrid>(Entity::from_bits(bits))
        .copied()
        .unwrap_or(ph2d_ecs::SpriteGrid::SINGLE);
    if g.cols * g.rows <= 1 || (g.cols, g.rows) == (grid.hframes, grid.vframes) {
        return false;
    }
    request_open(sim, bits, &AnimFieldEdit::OpenEditor)
}

/// A família: as abas abertas.
#[derive(Debug, Default)]
pub struct AnimDocs {
    pub docs: Vec<AnimDoc>,
    focus: Option<u64>,
}

impl AnimDocs {
    /// Abre (ou traz à frente) a aba da sprite `entity`.
    pub fn open(&mut self, entity: u64, name: String) {
        if !self.docs.iter().any(|d| d.entity == entity) {
            self.docs.push(AnimDoc::new(entity, name));
        }
        self.focus = Some(entity);
    }

    #[must_use]
    pub fn doc(&self, entity: u64) -> Option<&AnimDoc> {
        self.docs.iter().find(|d| d.entity == entity)
    }

    pub fn doc_mut(&mut self, entity: u64) -> Option<&mut AnimDoc> {
        self.docs.iter_mut().find(|d| d.entity == entity)
    }
}

impl DocumentSurface for AnimDocs {
    fn tabs(&self) -> Vec<DocTab> {
        self.docs
            .iter()
            .map(|d| DocTab {
                id: d.entity,
                title: ph2d_i18n::tr_with("anim_editor.tab", &[("name", &d.name)]),
                dirty: false,
            })
            .collect()
    }

    fn paint(
        &mut self,
        id: u64,
        area: Rect,
        scene: &mut VectorScene,
        text: &mut TextSystem,
        theme: &Theme,
    ) {
        if let Some(d) = self.doc_mut(id) {
            paint::paint_doc(d, area, scene, text, *theme);
        }
    }

    fn pointer(
        &mut self,
        id: u64,
        _input: PointerInput<'_>,
        kind: PointerKind,
        button: PointerButton,
        x: f32,
        y: f32,
    ) -> bool {
        let Some(d) = self.doc_mut(id) else {
            return false;
        };
        if button != PointerButton::Primary {
            return false;
        }
        match kind {
            PointerKind::Down => d.press(x, y),
            PointerKind::Up => d.release(x, y),
            PointerKind::Move => d.motion(x, y),
        }
    }

    fn pointer_move(&mut self, id: u64, _input: PointerInput<'_>, x: f32, y: f32) -> bool {
        self.doc_mut(id).is_some_and(|d| d.motion(x, y))
    }

    fn wheel(&mut self, id: u64, x: f32, y: f32, dy: f32, _shift: bool) -> bool {
        let Some(d) = self.doc_mut(id) else {
            return false;
        };
        let s = d.layout.strip;
        if !(x >= s.x && x < s.x + s.w && y >= s.y && y < s.y + s.h) {
            return false;
        }
        let n = d.layout.frames.len() as f32;
        let thumb = d
            .layout
            .frames
            .first()
            .map_or(0.0, |f| f.0.w + ph2d_tokens::control_gap_px());
        let max = (n * thumb - s.w).max(0.0);
        d.strip_scroll = (d.strip_scroll - dy).clamp(0.0, max);
        true
    }

    fn key(&mut self, id: u64, _ts: &mut TextSystem, input: KeyInput<'_>) -> KeyOutcome {
        let Some(d) = self.doc_mut(id) else {
            return KeyOutcome::default();
        };
        let consumed = |c: bool| KeyOutcome {
            consumed: c,
            copy: None,
        };
        if !input.pressed {
            return consumed(d.editing.is_some());
        }
        match input.key {
            Key::Enter if d.editing.is_some() => {
                d.close_field(true);
                consumed(true)
            }
            Key::Escape if d.editing.is_some() => {
                d.close_field(false);
                consumed(true)
            }
            Key::Backspace | Key::Delete => consumed(d.backspace_or_delete()),
            // Um atalho (Ctrl/Cmd) passa, mesmo com um campo aberto: o Ctrl+Z é do projecto.
            _ if d.editing.is_some() && !(input.mods.ctrl || input.mods.meta) => {
                if let Some(t) = input.text {
                    d.type_text(t);
                }
                consumed(true)
            }
            _ => consumed(false),
        }
    }

    fn compose(&mut self, _id: u64, _c: Compose<'_>) -> bool {
        // Os campos lêem o texto da tecla (`KeyInput::text`), como a lista do editor de scripts.
        false
    }

    fn close(&mut self, id: u64) {
        self.docs.retain(|d| d.entity != id);
    }

    fn undo(&mut self, _id: u64, _redo: bool) -> bool {
        // O desfazer é o do projecto: as edições desta aba entram na mesma fila por diff.
        false
    }

    fn take_focus_request(&mut self) -> Option<u64> {
        self.focus.take()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// O nome do objecto para o título da aba.
fn name_of(sim: &SimWorld, bits: u64) -> String {
    sim.world()
        .get::<ph2d_ecs::Name>(Entity::from_bits(bits))
        .map_or_else(
            || ph2d_i18n::tr("anim_editor.unnamed").to_string(),
            |n| n.0.clone(),
        )
}

/// O snapshot de uma aba: a §11 com o cartão (que já traz a folha da [`crate::sheet::SheetCache`]).
#[must_use]
pub fn snapshot(sim: &SimWorld, bits: u64) -> EditorSnapshot {
    let w = sim.world();
    let e = Entity::from_bits(bits);
    let info = crate::inspector::build_anim_info(w, bits, 1);
    let grid = w
        .get::<ph2d_ecs::SpriteGrid>(e)
        .copied()
        .unwrap_or(ph2d_ecs::SpriteGrid::SINGLE);
    let entry = w
        .get_resource::<crate::sheet::SheetCache>()
        .and_then(|c| c.of_entity(bits))
        .cloned();
    EditorSnapshot {
        info,
        sheet: entry.as_ref().map(|e| e.sheet.clone()),
        guess: entry.and_then(|e| e.guess),
        hframes: grid.hframes,
        vframes: grid.vframes,
    }
}

/// **O quadro da família**: abre o que se pediu, resolve as folhas, aplica as edições das abas pela
/// porta do Inspector e entrega a cada aba o snapshot novo.
pub fn frame(
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    registry: &ComponentRegistry,
    toasts: &mut ToastQueue,
) {
    let wanted = sim
        .world_mut()
        .get_resource_mut::<AnimEditorRequests>()
        .map(|mut r| std::mem::take(&mut r.0))
        .unwrap_or_default();
    let names: Vec<(u64, String)> = wanted.iter().map(|&b| (b, name_of(sim, b))).collect();
    let selection = hero.gizmo.selection;
    let Some(docs) = hero.documents.surfaces_mut().find_mut::<AnimDocs>() else {
        return;
    };
    for (b, n) in names {
        docs.open(b, n);
    }
    // Uma aba cuja sprite saiu da cena fecha-se (nada a editar).
    docs.docs.retain(|d| {
        sim.world()
            .get::<ph2d_render::Sprite>(Entity::from_bits(d.entity))
            .is_some()
    });
    let mut look: Vec<u64> = docs.docs.iter().map(|d| d.entity).collect();
    look.extend(selection);
    crate::sheet::refresh(sim.world_mut(), &look, db, atlas);
    for d in &mut docs.docs {
        for edit in std::mem::take(&mut d.pending) {
            let queue = EditorCommandQueue::new();
            if let Some(t) =
                crate::inspector::apply_anim_edit(sim, d.entity, &edit, &queue, registry)
            {
                toasts.push(t);
                continue;
            }
            if let Err(e) = apply_editor_commands(sim.world_mut(), &queue, registry) {
                toasts.push(Toast::error(ph2d_i18n::tr_with(
                    "anim_editor.commit_failed",
                    &[("e", &format!("{e:?}"))],
                )));
            }
        }
        d.set_snapshot(snapshot(sim, d.entity));
    }
}

#[cfg(test)]
mod tests;

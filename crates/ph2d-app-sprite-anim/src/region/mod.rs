//! **O editor de Region** ([plano GEF 16](../../../../docs/GameEngineFuncional/16_uso_region_plano.md) D2) — uma
//! aba por sprite, irmã do [`crate::editor`]: a imagem inteira com zoom e arrasto, a grelha Columns × Rows e a
//! área com 8 alças, que encaixa nas linhas (`Alt`: ao píxel). **OK** aplica num passo de desfazer
//! (`ImageUseEdit::SetRegion`, pela porta do Inspector); **Cancel** / `Esc` não muda nada. A shell instala
//! [`RegionDocs`] uma vez e chama [`frame`] uma vez por quadro, ANTES dos snapshots.

pub mod doc;
pub mod paint;

use std::collections::BTreeMap;

use doc::{Outcome, RegionDoc};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor_core::doc_surface::{
    Compose, DocTab, DocumentSurface, Key, KeyInput, KeyOutcome, PointerInput,
};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{HeroScreen, ImageUseEdit};
use ph2d_host::{PointerButton, PointerKind};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

/// A família: as abas abertas.
#[derive(Debug, Default)]
pub struct RegionDocs {
    pub docs: Vec<RegionDoc>,
    focus: Option<u64>,
}

impl RegionDocs {
    #[must_use]
    pub fn doc(&self, entity: u64) -> Option<&RegionDoc> {
        self.docs.iter().find(|d| d.entity == entity)
    }

    pub fn doc_mut(&mut self, entity: u64) -> Option<&mut RegionDoc> {
        self.docs.iter_mut().find(|d| d.entity == entity)
    }
}

impl DocumentSurface for RegionDocs {
    fn tabs(&self) -> Vec<DocTab> {
        self.docs
            .iter()
            .map(|d| DocTab {
                id: d.entity,
                title: ph2d_i18n::tr_with("region_editor.tab", &[("name", &d.name)]),
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
        input: PointerInput<'_>,
        kind: PointerKind,
        button: PointerButton,
        x: f32,
        y: f32,
    ) -> bool {
        let Some(d) = self.doc_mut(id) else {
            return false;
        };
        d.set_keys(input.mods.alt, None);
        match (kind, button) {
            (PointerKind::Down, PointerButton::Primary) => d.press(x, y),
            (PointerKind::Down, PointerButton::Middle) => d.press_middle(x, y),
            (PointerKind::Up, _) => d.release(x, y),
            (PointerKind::Move, _) => d.motion(x, y),
            _ => false,
        }
    }

    fn pointer_move(&mut self, id: u64, input: PointerInput<'_>, x: f32, y: f32) -> bool {
        self.doc_mut(id).is_some_and(|d| {
            d.set_keys(input.mods.alt, None);
            d.motion(x, y)
        })
    }

    fn wheel(&mut self, id: u64, x: f32, y: f32, dy: f32, _shift: bool) -> bool {
        self.doc_mut(id).is_some_and(|d| d.wheel(x, y, dy))
    }

    fn key(&mut self, id: u64, _ts: &mut TextSystem, input: KeyInput<'_>) -> KeyOutcome {
        let Some(d) = self.doc_mut(id) else {
            return KeyOutcome::default();
        };
        let consumed = |c: bool| KeyOutcome {
            consumed: c,
            copy: None,
        };
        let space = (input.key == Key::Space).then_some(input.pressed);
        d.set_keys(input.mods.alt, space);
        if !input.pressed {
            return consumed(space.is_some() || d.editing.is_some());
        }
        let campo = d.editing.is_some();
        match input.key {
            Key::Enter if campo => d.close_field(true),
            Key::Enter => d.ok(),
            Key::Escape if campo => d.close_field(false),
            Key::Escape => d.outcome = Some(Outcome::Cancel),
            Key::Backspace | Key::Delete if campo => {
                d.backspace();
            }
            Key::Space => {}
            // Um atalho (Ctrl/Cmd) passa: o Ctrl+Z é do projecto.
            _ if campo && !(input.mods.ctrl || input.mods.meta) => {
                if let Some(t) = input.text {
                    d.type_text(t);
                }
            }
            _ => return consumed(false),
        }
        consumed(true)
    }

    fn compose(&mut self, _id: u64, _c: Compose<'_>) -> bool {
        false
    }

    fn close(&mut self, id: u64) {
        self.docs.retain(|d| d.entity != id);
    }

    fn undo(&mut self, _id: u64, _redo: bool) -> bool {
        // O desfazer é o do projecto: o OK entra na mesma fila por diff, num passo.
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
fn name_of(w: &ph2d_ecs::World, e: Entity) -> String {
    w.get::<ph2d_ecs::Name>(e).map_or_else(
        || ph2d_i18n::tr("anim_editor.unnamed").to_string(),
        |n| n.0.clone(),
    )
}

/// Abre (ou traz à frente) a aba de `e`; nada se `e` não está em Region.
pub fn open(
    docs: &mut RegionDocs,
    w: &ph2d_ecs::World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
) -> bool {
    let bits = e.to_bits();
    if docs.doc(bits).is_none() {
        let Some((px, rect, grid)) = crate::image_use::region_seed(w, e, db, atlas, ppm) else {
            return false;
        };
        let stable = w.get::<ph2d_ecs::StableId>(e).map_or(0, |s| s.0);
        docs.docs
            .push(RegionDoc::new(bits, stable, name_of(w, e), px, rect, grid));
    }
    docs.focus = Some(bits);
    true
}

/// **O quadro da família**: abre o que se pediu, dá a cada aba a imagem e aplica o OK (um passo de desfazer)
/// pela porta do Inspector; a aba que acabou (OK, Cancel, a sprite saiu de Region ou da cena) fecha-se.
pub fn frame(
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
) {
    let wanted = sim
        .world_mut()
        .get_resource_mut::<crate::image_use::RegionEditorRequests>()
        .map(|mut r| std::mem::take(&mut r.0))
        .unwrap_or_default();
    let Some(docs) = hero.documents.surfaces_mut().find_mut::<RegionDocs>() else {
        return;
    };
    if docs.docs.is_empty() && wanted.is_empty() {
        return;
    }
    for b in wanted {
        if let Some(e) = Entity::try_from_bits(b) {
            open(docs, sim.world(), e, db, atlas, ppm);
        }
    }
    follow_identity(sim.world(), &mut docs.docs);
    let look: Vec<u64> = docs.docs.iter().map(|d| d.entity).collect();
    crate::sheet::refresh(sim.world_mut(), &look, db, atlas);
    for d in &mut docs.docs {
        if d.image.is_none() {
            d.image = sim
                .world()
                .get_resource::<crate::sheet::SheetCache>()
                .and_then(|c| c.of_entity(d.entity))
                .map(|s| s.sheet.clone());
        }
        if let Some(Outcome::Ok { rect, grid }) = d.outcome {
            crate::image_use::apply_edit(
                sim.world_mut(),
                Entity::from_bits(d.entity),
                db,
                atlas,
                ppm,
                ImageUseEdit::SetRegion { rect, grid },
            );
        }
    }
    let w = sim.world();
    docs.docs.retain(|d| {
        d.outcome.is_none()
            && Entity::try_from_bits(d.entity).is_some_and(|e| {
                crate::image_use::image_use(w, e) == Some(crate::image_use::ImageUse::Region)
            })
    });
}

/// O gesto da mão sobre a aba de Region aberta: um clique no CENTRO da célula `cell` (coluna, linha) da grelha
/// (o condutor do smoke, `PH2D_GAME_SMOKE_USOS`). `false` sem aba ou antes de ela estar pintada.
pub fn click_cell(hero: &mut HeroScreen, cell: [u32; 2]) -> bool {
    let Some(d) = hero
        .documents
        .surfaces_mut()
        .find_mut::<RegionDocs>()
        .and_then(|f| f.docs.first_mut())
    else {
        return false;
    };
    if d.layout.canvas.w <= 0.0 {
        return false;
    }
    let p: [f32; 2] = std::array::from_fn(|i| {
        let meio = (d.grid_line(i, cell[i]) + d.grid_line(i, cell[i] + 1)) * 0.5;
        d.origin[i] + meio * d.zoom
    });
    d.press(p[0], p[1]);
    d.release(p[0], p[1])
}

/// O gesto da mão: um clique no botão OK da aba de Region aberta.
pub fn click_ok(hero: &mut HeroScreen) -> bool {
    let Some(d) = hero
        .documents
        .surfaces_mut()
        .find_mut::<RegionDocs>()
        .and_then(|f| f.docs.first_mut())
    else {
        return false;
    };
    let ok = d.layout.ok;
    d.press(ok.x + ok.w * 0.5, ok.y + ok.h * 0.5)
}

/// A aba segue a sprite pela identidade, não pelos bits (o restauro do desfazer respawna o mundo).
fn follow_identity(w: &ph2d_ecs::World, docs: &mut [RegionDoc]) {
    for d in docs.iter_mut() {
        let vivo = Entity::try_from_bits(d.entity)
            .is_some_and(|e| w.get::<ph2d_render::Sprite>(e).is_some());
        if vivo || d.stable == 0 {
            continue;
        }
        if let Some(e) = ph2d_ecs::entity_with_stable_id(w, ph2d_ecs::StableId(d.stable)) {
            d.entity = e.to_bits();
        }
    }
}

#[cfg(test)]
mod tests;

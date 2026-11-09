//! **O editor de animações** — uma aba por sprite (plano GameEngineFuncional 07, «Desenho aprovado» 2–3).
//!
//! Uma família de documentos ([`DocumentSurface`]) no molde do editor de scripts: a shell instala
//! [`AnimDocs`] uma vez e chama [`frame`] uma vez por quadro, ANTES dos snapshots. O «Editar animações…»
//! do cartão chega como `AnimFieldEdit::OpenEditor`, que a shell entrega a [`request_open`].

// O documento da aba mora em `ph2d-anim-editor` (catraca das `ph2d-app-*`, auditoria 08/11); aqui fica a ligação.
pub use ph2d_anim_editor::{bands, doc, layout, paint, paint_sheet, split, strip_bar};

use std::collections::BTreeMap;

use doc::{AnimDoc, EditorSnapshot};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue, apply_editor_commands};
use ph2d_ecs::{Entity, Resource, SimWorld};
use ph2d_editor_core::doc_surface::{
    Compose, DocTab, DocumentSurface, Key, KeyInput, KeyOutcome, PointerInput, PointerLook,
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

/// **Uma imagem acabou de ser largada sobre a sprite ANIMADA `bits`**: se a transparência propõe uma
/// grelha de várias células diferente da que a sprite tem, abre o editor dela — a faixa «Parece C × L —
/// Usar» é a proposta, e o artista confirma (plano 07, «Desenho aprovado» 3). Parada e repetida mostram a
/// imagem inteira (plano GEF 15): nada a propor. `rgba` é a imagem reta, `w × h × 4`. Devolve se pediu o editor.
pub fn propose_after_drop(sim: &mut SimWorld, bits: u64, rgba: &[u8], w: u32, h: u32) -> bool {
    let animada = crate::image_use::image_use(sim.world(), Entity::from_bits(bits))
        == Some(crate::image_use::ImageUse::Animated);
    if !animada || rgba.len() != (w as usize) * (h as usize) * 4 {
        return false;
    }
    let alpha: Vec<u8> = rgba.as_chunks::<4>().0.iter().map(|p| p[3]).collect();
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
    /// O relógio do quadro anterior (o voo da inserção anda a diferença).
    clock_ns: Option<u128>,
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
        input: PointerInput<'_>,
        kind: PointerKind,
        button: PointerButton,
        x: f32,
        y: f32,
    ) -> bool {
        let Some(d) = self.doc_mut(id) else {
            return false;
        };
        d.mods = input.mods;
        if button == PointerButton::Secondary && kind == PointerKind::Down {
            return d.align_secondary(x, y);
        }
        if button != PointerButton::Primary {
            return false;
        }
        match kind {
            PointerKind::Down => d.press(x, y),
            PointerKind::Up => d.release(x, y),
            PointerKind::Move => d.motion(x, y),
        }
    }

    fn pointer_move(&mut self, id: u64, input: PointerInput<'_>, x: f32, y: f32) -> bool {
        self.doc_mut(id).is_some_and(|d| {
            d.mods = input.mods;
            d.motion(x, y)
        })
    }

    fn pointer_look(&self, id: u64, x: f32, y: f32) -> Option<PointerLook> {
        self.doc(id)?.pointer_look(x, y)
    }

    fn wheel(&mut self, id: u64, x: f32, y: f32, dy: f32, _shift: bool) -> bool {
        self.doc_mut(id).is_some_and(|d| d.wheel_strip(x, y, dy))
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
            Key::Escape if d.cancel_scrub() => consumed(true),
            Key::Escape if d.align.picking => {
                d.align.picking = false;
                consumed(true)
            }
            Key::Enter if d.editing.is_some() => {
                d.close_field(true);
                consumed(true)
            }
            Key::Escape if d.editing.is_some() => {
                d.close_field(false);
                consumed(true)
            }
            Key::Backspace | Key::Delete => consumed(d.backspace_or_delete()),
            // A prévia Align: as setas acertam o quadro aberto (GEF 28).
            Key::Left | Key::Right | Key::Up | Key::Down if d.editing.is_none() => {
                let (dx, dy) = match input.key {
                    Key::Left => (-1.0, 0.0),
                    Key::Right => (1.0, 0.0),
                    Key::Up => (0.0, 1.0),
                    _ => (0.0, -1.0),
                };
                consumed(d.align_arrow(dx, dy, input.mods.shift))
            }
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
        false
    }

    /// As edições desta aba são diffs da cena. Sem isto a aba engolia o Ctrl+Z (medido em 07/11, ficha 02).
    fn undo_is_the_projects(&self) -> bool {
        true
    }

    fn take_focus_request(&mut self) -> Option<u64> {
        self.focus.take()
    }

    /// Um voo de inserção em curso pede quadros até assentar.
    fn wants_frame(&self) -> bool {
        self.docs.iter().any(|d| d.flight.is_some())
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

/// O snapshot de uma aba: a §11 com o cartão e as folhas da [`crate::sheet::SheetCache`]; a grelha é a da visível.
#[must_use]
pub fn snapshot(sim: &SimWorld, bits: u64, ppm: f32) -> EditorSnapshot {
    let (sheets, shown) = ph2d_anim_sheets::editor_ui::editor_sheets(sim.world(), bits);
    let s = sheets.get(usize::from(shown)).cloned().unwrap_or_default();
    EditorSnapshot {
        info: crate::inspector::build_anim_info(sim.world(), bits, 1),
        sheet: s.image,
        guess: s.guess,
        hframes: s.hframes,
        vframes: s.vframes,
        shown_sheet: shown,
        sheets,
        geo: Entity::try_from_bits(bits)
            .map(|e| ph2d_anim_sheets::align::sheet_geos(sim.world(), e, ppm))
            .unwrap_or_default(),
    }
}

/// As famílias de abas desta crate: o editor de animações e o de Region (plano GEF 16).
pub fn install(hero: &mut HeroScreen) {
    hero.documents
        .install_surface(Box::new(AnimDocs::default()));
    hero.documents
        .install_surface(Box::new(crate::region::RegionDocs::default()));
}

/// **O quadro da família**: abre o que se pediu, resolve as folhas, aplica as edições das abas pela
/// porta do Inspector e entrega a cada aba o snapshot novo. O editor de Region corre primeiro, aqui.
pub fn frame(
    sim: &mut SimWorld,
    hero: &mut HeroScreen,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    registry: &ComponentRegistry,
    toasts: &mut ToastQueue,
) {
    let ppm = hero.project.pixels_per_meter;
    crate::region::frame(sim, hero, db, atlas, ppm);
    let wanted = sim
        .world_mut()
        .get_resource_mut::<AnimEditorRequests>()
        .map(|mut r| std::mem::take(&mut r.0))
        .unwrap_or_default();
    let names: Vec<(u64, String)> = wanted.iter().map(|&b| (b, name_of(sim, b))).collect();
    let selection = hero.gizmo.selection;
    let (now, law) = (
        hero.store.clock_ns(),
        hero.motion.law(ph2d_editor_core::motion::Role::Travel),
    );
    let active = hero.documents.surface_active().filter(|t| {
        let f = hero.documents.surfaces_mut().get(t.family);
        f.is_some_and(|f| f.as_any().is::<AnimDocs>())
    });
    let Some(docs) = hero.documents.surfaces_mut().find_mut::<AnimDocs>() else {
        return;
    };
    // O voo anda o tempo real entre dois quadros (preso a um décimo: uma pausa longa não o teletransporta).
    let dt = docs.clock_ns.map_or(0, |t| now.saturating_sub(t)) as f64 * 1e-9; // LITERAL-PX-OK: ns → s
    docs.clock_ns = Some(now);
    for d in &mut docs.docs {
        d.tick(dt.min(0.1), law); // LITERAL-PX-OK: segundos
    }
    for (b, n) in names {
        docs.open(b, n);
    }
    let moved = follow_identity(sim.world(), &mut docs.docs);
    let mut look: Vec<u64> = docs.docs.iter().map(|d| d.entity).collect();
    look.extend(selection);
    crate::sheet::refresh(sim.world_mut(), &look, db, atlas);
    for d in &mut docs.docs {
        if !has_sprite(sim.world(), d.entity) {
            continue; // à espera do Stop
        }
        let e = Entity::from_bits(d.entity);
        for edit in std::mem::take(&mut d.sheet_ui.pending) {
            ph2d_anim_sheets::apply::apply(sim.world_mut(), e, db, atlas, ppm, edit);
        }
        let mut edits: Vec<AnimFieldEdit> = std::mem::take(&mut d.pending);
        edits.reverse();
        while let Some(edit) = edits.pop() {
            // O «+» escolhe a nova E põe-na no ecrã: o cartão e o editor mostram a mesma. O nome é o
            // que a biblioteca dá ANTES de a receber; o SetCurrent vai depois de o Add pegar.
            let added = (edit == AnimFieldEdit::Add).then(|| {
                sim.world()
                    .get::<ph2d_ecs::SpriteAnimations>(Entity::from_bits(d.entity))
                    .map_or_else(
                        || ph2d_ecs::SpriteAnimations::new().next_free_name(),
                        ph2d_ecs::SpriteAnimations::next_free_name,
                    )
            });
            // Cols/Rows de uma folha extra visível editam o corte DELA (plano GEF 26 §3.4).
            let side = |n: u32| n.clamp(1, crate::card::GRID_SIDE_MAX);
            if let AnimFieldEdit::Grid(h, v) = edit
                && ph2d_anim_sheets::apply::set_shown_grid(sim.world_mut(), e, side(h), side(v))
            {
                let k = ph2d_ecs::shown_sheet(sim.world(), e);
                crate::image_use::recut_sheet_users(sim.world_mut(), e, k);
                continue;
            }
            let queue = EditorCommandQueue::new();
            crate::previa::nota_o_gesto(sim.world_mut(), d.entity, &edit);
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
                continue;
            }
            // O corte é da imagem: quem a mostra segue, e os quadros de fora viram buraco (plano GEF 26).
            if matches!(edit, AnimFieldEdit::Grid(..)) {
                crate::image_use::recut_users(sim.world_mut(), e);
            }
            edits.extend(added.map(AnimFieldEdit::SetCurrent));
        }
        d.set_snapshot(snapshot(sim, d.entity, ppm));
    }
    // A aba à frente cuja sprite mudou de bits continua à frente.
    if let Some(t) = active
        && let Some(&(_, id)) = moved.iter().find(|(de, _)| *de == t.id)
    {
        hero.documents
            .activate_surface(ph2d_editor_core::doc_surface::SurfaceTab { id, ..t });
    }
}

fn has_sprite(w: &ph2d_ecs::World, bits: u64) -> bool {
    Entity::try_from_bits(bits).is_some_and(|e| w.get::<ph2d_render::Sprite>(e).is_some())
}

/// ⭐ **A aba segue a sprite pela identidade, não pelos bits** (report do dono 25/10: fechar a janela do
/// jogo fechava a aba). O restauro do undo — o Stop do jogo, Ctrl+Z — respawna o mundo com bits novos: a
/// aba passa aos bits que o `StableId` dela tem agora. Uma sprite que a corrida tirou (sobre a cópia) deixa
/// a aba à espera do Stop; fora da corrida, a aba cuja sprite saiu da cena fecha-se. Devolve `de → para`.
fn follow_identity(w: &ph2d_ecs::World, docs: &mut Vec<AnimDoc>) -> Vec<(u64, u64)> {
    let mut moved = Vec::new();
    for d in docs.iter_mut() {
        if has_sprite(w, d.entity) {
            let sid = Entity::try_from_bits(d.entity).and_then(|e| w.get::<ph2d_ecs::StableId>(e));
            d.stable = sid.map_or(d.stable, |s| s.0);
            continue;
        }
        let back = ph2d_ecs::entity_with_stable_id(w, ph2d_ecs::StableId(d.stable))
            .map(Entity::to_bits)
            .filter(|&b| has_sprite(w, b));
        if let Some(b) = back {
            moved.push((d.entity, b));
            d.entity = b;
        }
    }
    let on_copy = ph2d_ecs::run_is_on_copy(w);
    docs.retain(|d| on_copy || has_sprite(w, d.entity));
    moved
}

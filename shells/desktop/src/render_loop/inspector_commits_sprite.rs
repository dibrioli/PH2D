//! **O commit da §2 Sprite** — o que uma [`SpriteFieldEdit`] escreve no componente.
//!
//! ⚠️ **Irmão de [`super::inspector_commits`] por CAP de LOC** (HR-18, 600 no shell): aquele
//! ficheiro é o **roteador** de todos os commits do Inspector e cresce uma família por seção — a
//! §11 Animation levou-o a 611 em 2026-08-23. A regra da casa é cortar para o irmão, nunca
//! declarar exceção, e o que sai é o que não é roteamento.

use ph2d_ecs::{SpriteCornerTint, SpriteGrid, SpriteRegion};
use ph2d_editor::SpriteFieldEdit;
use ph2d_render::Sprite;

/// **Os quatro editáveis da §2**, lidos da entidade com o default benigno no lugar do que ela
/// não tem (ADR-0164 F1 passo 6).
///
/// ⚠️ **A `region` é `Option` e as outras não**, e a assimetria é a lei: a PRESENÇA do
/// [`SpriteRegion`] é o antigo `region_enabled`, então «não há região» tem de ser exprimível.
/// Uma grelha de uma célula e cantos brancos, ao contrário, dizem-se com um VALOR.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct SpriteEditables {
    pub sprite: Sprite,
    pub grid: SpriteGrid,
    pub region: Option<SpriteRegion>,
    pub corner_tint: SpriteCornerTint,
}

impl SpriteEditables {
    /// Lê os quatro da entidade. `None` só quando ela não é uma sprite.
    pub(super) fn read(world: &ph2d_ecs::World, entity: ph2d_ecs::Entity) -> Option<Self> {
        Some(Self {
            sprite: *world.get::<Sprite>(entity)?,
            grid: world
                .get::<SpriteGrid>(entity)
                .copied()
                .unwrap_or(SpriteGrid::SINGLE),
            region: world.get::<SpriteRegion>(entity).copied(),
            corner_tint: world
                .get::<SpriteCornerTint>(entity)
                .copied()
                .unwrap_or(SpriteCornerTint::IDENTITY),
        })
    }
}

/// **Que componente uma edição escreveu** — o que o commit tem de gravar.
///
/// ⚠️ Existe porque uma edição da §2 deixou de ter um destino só. Devolver isto, em vez de o
/// chamador adivinhar pelo variante da `SpriteFieldEdit`, mantém a lei num sítio: um variante
/// novo que se esqueça de dizer o destino é **erro de compilação** aqui, e não um commit
/// silencioso no componente errado.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum SpriteEditTarget {
    Sprite,
    Grid,
    /// A região passou a existir ou mudou de valor.
    Region,
    /// ⭐ A região deixou de existir — o commit **REMOVE** o componente. É o antigo
    /// `region_enabled = false`, dito da única maneira que ele hoje se diz.
    RegionRemoved,
    CornerTint,
}

/// Apply one [`SpriteFieldEdit`], enforcing the schema invariants the Inspector widgets can't
/// (anatomia §1.6): `hframes`/`vframes >= 1`, `frame < hframes*vframes`, `opacity ∈ [0, 1]`.
/// The frame index is re-clamped whenever the grid shrinks so a stale frame can never index
/// past the sheet. This is the single authoring write boundary for the editable sprite fields.
///
/// ⚠️ **Uma edição de campo da região MATERIALIZA o componente quando ele falta**, em vez de
/// ser um no-op: hoje o painel ainda mostra as linhas da região a toda sprite, e um campo que
/// aceita o gesto e não faz nada é o defeito que a DIRETIVA §2 proíbe. Quando a F3 passar a
/// pintar por presença, a linha deixa de existir sem o componente e esta rota fica inalcançável
/// pela UI — mas continua correta.
pub(super) fn apply_sprite_field(
    t: &mut SpriteEditables,
    edit: SpriteFieldEdit,
) -> SpriteEditTarget {
    match edit {
        SpriteFieldEdit::FlipX(b) => {
            t.sprite.flip_x = b;
            SpriteEditTarget::Sprite
        }
        SpriteFieldEdit::FlipY(b) => {
            t.sprite.flip_y = b;
            SpriteEditTarget::Sprite
        }
        SpriteFieldEdit::Centered(b) => {
            t.sprite.centered = b;
            SpriteEditTarget::Sprite
        }
        // Per-axis: preserve the OTHER axis (so a bulk edit of one axis
        // can't stomp a diverging sibling — audit D-1).
        SpriteFieldEdit::OffsetX(x) => {
            t.sprite.offset[0] = x;
            SpriteEditTarget::Sprite
        }
        SpriteFieldEdit::OffsetY(y) => {
            t.sprite.offset[1] = y;
            SpriteEditTarget::Sprite
        }
        SpriteFieldEdit::Hframes(n) => {
            t.grid.hframes = n.max(1);
            clamp_frame(&mut t.grid);
            SpriteEditTarget::Grid
        }
        SpriteFieldEdit::Vframes(n) => {
            t.grid.vframes = n.max(1);
            clamp_frame(&mut t.grid);
            SpriteEditTarget::Grid
        }
        SpriteFieldEdit::Frame(f) => {
            t.grid.frame = f;
            clamp_frame(&mut t.grid);
            SpriteEditTarget::Grid
        }
        // ⭐ Ligar/desligar a região É anexar/retirar o componente.
        SpriteFieldEdit::RegionEnabled(b) => {
            if b {
                t.region
                    .get_or_insert_with(|| region_for(&t.sprite, [0.0; 4]));
                SpriteEditTarget::Region
            } else {
                t.region = None;
                SpriteEditTarget::RegionRemoved
            }
        }
        SpriteFieldEdit::RegionRect(r) => {
            // Schema invariant (anatomia §1.6): w/h kept `>= 0`. A negative
            // extent would invert the sampled UV; x/y may be negative (the
            // extract clamps the rect into the source).
            region_mut(t).rect = [r[0], r[1], r[2].max(0.0), r[3].max(0.0)];
            SpriteEditTarget::Region
        }
        // Per-axis: preserve the other three components (audit D-1). W/H
        // floor at 0 like the whole-vector path.
        SpriteFieldEdit::RegionX(x) => {
            region_mut(t).rect[0] = x;
            SpriteEditTarget::Region
        }
        SpriteFieldEdit::RegionY(y) => {
            region_mut(t).rect[1] = y;
            SpriteEditTarget::Region
        }
        SpriteFieldEdit::RegionW(w) => {
            region_mut(t).rect[2] = w.max(0.0);
            SpriteEditTarget::Region
        }
        SpriteFieldEdit::RegionH(h) => {
            region_mut(t).rect[3] = h.max(0.0);
            SpriteEditTarget::Region
        }
        SpriteFieldEdit::RegionFilterClip(b) => {
            region_mut(t).filter_clip = b;
            SpriteEditTarget::Region
        }
        SpriteFieldEdit::Tint(c) => {
            t.sprite.tint = c;
            SpriteEditTarget::Sprite
        }
        SpriteFieldEdit::SelfTint(c) => {
            t.sprite.self_tint = c;
            SpriteEditTarget::Sprite
        }
        SpriteFieldEdit::TintFill(b) => {
            t.sprite.tint_fill = b;
            SpriteEditTarget::Sprite
        }
        SpriteFieldEdit::Opacity(o) => {
            t.sprite.opacity = o.clamp(0.0, 1.0);
            SpriteEditTarget::Sprite
        }
        // ⚠️ Um canto, lido e escrito **na sprite desta iteração** — é isso que faz o fan-out
        // preservar os cantos divergentes das outras em vez de os atropelar.
        SpriteFieldEdit::PerCornerTintAt(i, rgba) => {
            if let Some(slot) = t.corner_tint.0.get_mut(usize::from(i)) {
                *slot = rgba;
            }
            SpriteEditTarget::CornerTint
        }
        // Cada sprite iguala pelo SEU próprio TL — «igualar» é uma operação, não um valor.
        SpriteFieldEdit::EqualizeCorners => {
            t.corner_tint.0 = [t.corner_tint.0[0]; 4];
            SpriteEditTarget::CornerTint
        }
    }
}

/// A região da entidade, criando-a se faltar — ver a nota do [`apply_sprite_field`].
fn region_mut(t: &mut SpriteEditables) -> &mut SpriteRegion {
    let sprite = t.sprite;
    t.region
        .get_or_insert_with(|| region_for(&sprite, [0.0; 4]))
}

/// Uma região nova para ESTA sprite: o `filter_clip` sai da FONTE dos pixels, que é onde a
/// escolha sempre pertenceu (ver o cabeçalho do [`SpriteRegion`]).
fn region_for(sprite: &Sprite, rect: [f32; 4]) -> SpriteRegion {
    if matches!(sprite.source, ph2d_render::SpriteSource::Atlas { .. }) {
        SpriteRegion::for_atlas(rect)
    } else {
        SpriteRegion::individual(rect)
    }
}

/// Clamp `frame` into `[0, cells - 1]`. `hframes`/`vframes` are always `>= 1` here (set via
/// [`apply_sprite_field`]), so the grid has at least one cell.
pub(super) fn clamp_frame(grid: &mut SpriteGrid) {
    let cells = grid.cells().max(1);
    if grid.frame >= cells {
        grid.frame = cells - 1;
    }
}

// §7 ordering commit handler lives in the sibling `inspector_ordering`
// module (HR-18 LOC + separation): `apply_ordering_edit`.

/// Os nomes canónicos que este commit pode gravar (ADR-0164 F1 passo 6). ⚠️ **Literais e não
/// `type_name`**: o registo é indexado pelo nome CANÓNICO, que é dado à mão no `register_*` e não
/// derivado do tipo Rust — renomear o módulo não pode mudar o que o arquivo diz.
const SPRITE_TYPE: &str = "ph2d::render::Sprite";
const GRID_TYPE: &str = "ph2d::ecs::SpriteGrid";
const REGION_TYPE: &str = "ph2d::ecs::SpriteRegion";
const CORNER_TINT_TYPE: &str = "ph2d::ecs::SpriteCornerTint";

/// ⭐ **A família da SPRITE do commit do Inspector** — devolve `true` se o título ficou sujo.
///
/// ⚠️ Vive aqui e não em [`super::inspector_commits`] porque é aqui que mora o que uma sprite
/// editável É (o [`SpriteEditables`], os quatro componentes em que ela foi cortada na F1.6) — e
/// porque aquele ficheiro estava no teto de 600 LOC. *O corte é por assunto.*
pub(super) fn apply_sprite_edits(
    sprite_edits: &[(u64, ph2d_editor::screens::hero::SpriteFieldEdit)],
    sim: &mut ph2d_ecs::SimWorld,
    editor_queue: &ph2d_ecs::scene::EditorCommandQueue,
    component_registry: &ph2d_ecs::scene::ComponentRegistry,
    sprite_type_id: u64,
    toasts: &mut ph2d_editor::ToastQueue,
) -> bool {
    use ph2d_ecs::scene::{EditorCommand, apply_editor_commands};
    use ph2d_editor::Toast;
    use ph2d_render::Sprite;
    let mut dirty = false;
    // W2 Sprite Inspector v2: drain editable Sprite field edits (flip,
    // region, sprite-sheet, tint channels, opacity, …). For each, read
    // the entity's current Sprite, apply the one field (clamped), and
    // commit the whole struct through the SAME SetComponent path as
    // Transform. Grouped per-entity isn't necessary — applying edits
    // sequentially re-reads the just-written Sprite each iteration.
    for &(entity_bits, edit) in sprite_edits {
        let entity = ph2d_ecs::Entity::from_bits(entity_bits);
        let Some(mut editables) = SpriteEditables::read(sim.world(), entity) else {
            continue;
        };
        // `Sprite.premultiplied` is `#[serde(skip)]` — a runtime hint set
        // by BG-Removal Apply, NOT on the wire. The SetComponent round
        // trip (postcard → from_bytes) would reset it to `false` and
        // silently reintroduce the straight-alpha edge fringe. Capture
        // the live flag and re-assert it after the commit (audit F1).
        let was_premultiplied = editables.sprite.premultiplied;
        let target = apply_sprite_field(&mut editables, edit);
        // ⚠️ **Retirar um componente não passa pelo `SetComponent`** (ADR-0164 F1 passo 6):
        // desligar a região é uma AUSÊNCIA, e o comando que grava bytes não sabe exprimi-la.
        // O `remove` é direto no mundo, e o passo de undo nasce do diff do quadro como sempre.
        if target == SpriteEditTarget::RegionRemoved {
            sim.world_mut()
                .entity_mut(entity)
                .remove::<ph2d_ecs::SpriteRegion>();
            continue;
        }
        let encoded = match target {
            SpriteEditTarget::Sprite => {
                postcard::to_allocvec(&editables.sprite).map(|d| (SPRITE_TYPE, d))
            }
            SpriteEditTarget::Grid => {
                postcard::to_allocvec(&editables.grid).map(|d| (GRID_TYPE, d))
            }
            SpriteEditTarget::Region => {
                postcard::to_allocvec(&editables.region.expect("o alvo Region garante-a"))
                    .map(|d| (REGION_TYPE, d))
            }
            SpriteEditTarget::CornerTint => {
                postcard::to_allocvec(&editables.corner_tint).map(|d| (CORNER_TINT_TYPE, d))
            }
            SpriteEditTarget::RegionRemoved => unreachable!(),
        };
        match encoded {
            Ok((type_name, data)) => {
                let type_id = if type_name == SPRITE_TYPE {
                    sprite_type_id
                } else {
                    ph2d_ecs::scene::stable_type_id(type_name)
                };
                let push_res = editor_queue.push(EditorCommand::SetComponent {
                    entity: entity_bits,
                    type_id,
                    data,
                });
                if let Err(e) = push_res {
                    toasts.push(Toast::error(format!("Editor queue full: {e}")));
                    dirty = true;
                } else if let Err(e) =
                    apply_editor_commands(sim.world_mut(), editor_queue, component_registry)
                {
                    toasts.push(Toast::error(format!("Sprite commit failed: {e}")));
                    dirty = true;
                } else if was_premultiplied
                    && let Some(mut s) = sim.world_mut().get_mut::<Sprite>(entity)
                {
                    // Re-assert the serde(skip) runtime hint the wire dropped.
                    s.premultiplied = true;
                }
            }
            Err(e) => {
                toasts.push(Toast::error(format!("Sprite encode failed: {e}")));
                dirty = true;
            }
        }
    }
    dirty
}

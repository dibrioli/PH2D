//! Sim tick + extract phase.
//!
//! Wave 3.2 stage A — extracted from `render_loop::mod.rs` as a free
//! function. Runs the M5 demo's bouncing-motion sim tick, then the
//! ADR-0021 / ADR-0025 extract pass that propagates Transforms +
//! emits `RenderInstance`s into PresentWorld. Behavior-preserving
//! lift.
//!
//! HR-3: `worklist`'s capacity is reused across frames so this hot
//! path stays zero-alloc after warm-up (`tests/propagate_no_alloc.rs`).
//!
// Tecto de LOC NUMERADO em `tests/it/file_loc_caps.rs` — per-sprite RenderInstance build accreted every W3 render
// feature (tint cascade / region / sheet / anchor / flip / sampling /
// UV tiling / visibility cull / sort). 5 LOC over after amendment-6;
// follow-up = lift the build closure body into a sibling module (the
// hot-path context threading makes a mid-session split risky).

use crate::{Velocity, WORLD_HALF};
use ph2d_ecs::sort_key::{SortInput, SortScratch, compute_sort_ranks_into};
use ph2d_ecs::{
    ChildOf, ClipChildren, ClipMode, Entity, FilterMode, Mask2D, MaskInteraction, MaskMode,
    PresentWorld, RepeatMode, SimRef, SimWorld, Transform, TransformPropagationState, UvTransform,
    WorklistBuf, World, propagate_transforms, resolve_texture_filter, resolve_texture_repeat,
};
use ph2d_render::{RenderInstance, Sprite, SpriteRenderer};

/// A emissão de uma sprite (a instância e os quads dela) — filho por assunto, num ficheiro irmão.
#[path = "sim_extract_emit.rs"]
mod emit;

/// Render tint folding the ancestor modulate chain (anatomia §4.3):
/// `self_tint × tint × Π(ancestor.tint)`. Each ancestor contributes its
/// `tint` — the inheriting "modulate" (Godot) — NOT its `self_tint`,
/// which is local-only. Non-sprite ancestors (empty parents, pure
/// transforms) contribute nothing. Walks `ChildOf` bottom-up (O(depth));
/// the common no-parent case is one component miss + the per-sprite
/// `collapsed_tint`. No allocation (HR-3). Alpha cascades too (spec §4.4
/// `Π(modulate_ancestors.a)`); `opacity` stays per-sprite and is NOT
/// folded here.
fn cascade_tint_with_ancestors(world: &World, entity: Entity, sprite: &Sprite) -> [f32; 4] {
    let mut tint = sprite.collapsed_tint(); // self_tint × own tint
    let mut cur = world.get::<ChildOf>(entity).map(|c| c.parent());
    while let Some(parent) = cur {
        if let Some(ps) = world.get::<Sprite>(parent) {
            tint[0] *= ps.tint[0];
            tint[1] *= ps.tint[1];
            tint[2] *= ps.tint[2];
            tint[3] *= ps.tint[3];
        }
        cur = world.get::<ChildOf>(parent).map(|c| c.parent());
    }
    tint
}

/// Resolve the clip-stencil grouping for one emitted instance
/// (ADR-0070-amendment-7, spec §6.2). Returns `(clip_group, clip_meta)`
/// packed for [`RenderInstance`]:
///
/// - The instance is a **clip-parent** (has [`ClipChildren`] with a
///   non-`Disabled` mode) → it is the mask SOURCE of its own group. The
///   group id is `rank(self) + 1` (unique, never the `0` sentinel) and
///   the role encodes `ClipOnly` / `ClipAndDraw` + the quantized cutoff.
/// - Else the instance is a descendant of a clip-parent → it is a MEMBER,
///   tagged with the NEAREST (innermost) clip-ancestor's group id.
/// - Else no clip → `(CLIP_GROUP_NONE, 0)` (normal-pass identity).
///
/// **Single-level (W3):** a clip-parent nested under another clip-parent
/// owns its own independent group — the outer silhouette is NOT
/// intersected with the inner. True nested intersection is a future wave
/// (handoff §6); the regression gate marks the single-level contract.
///
/// `ranks` must already be filled by the sort pipeline. The renderer's
/// **clip-anchor sort** (renderer.rs) keeps every clip group contiguous so
/// it batches into one stencil mark→test→draw triple — we do NOT rely on
/// raw `z_order`/DFS adjacency (a divergent member Z or an interloping
/// sprite would otherwise split the span; W3 §8 audit fix).
///
/// **Hidden / culled clip-parent (known W3 behavior, audit LOW):** a
/// clip-parent that is `Visibility.hidden` or `VisibilityLayer`-culled
/// emits no `SortInput`, so `ranks.rank(parent)` is `None` and its members
/// fall through to `CLIP_GROUP_NONE` → they render UNCLIPPED (Visibility is
/// per-entity, it does not propagate to descendants). Graceful (children
/// don't vanish) but means hiding a mould un-clips its children. Proper
/// subtree-hide = visibility propagation, a future wave.
fn resolve_clip_grouping(sim: &World, entity: Entity, ranks: &SortScratch) -> (u32, u32) {
    // This entity owns a clip? → it is the mask source for its group.
    if let Some(cc) = sim.get::<ClipChildren>(entity)
        && cc.mode != ClipMode::Disabled
        && let Some(rank) = ranks.rank(entity)
    {
        let role = match cc.mode {
            ClipMode::ClipOnly => RenderInstance::CLIP_ROLE_MASK_CLIP_ONLY,
            ClipMode::ClipAndDraw => RenderInstance::CLIP_ROLE_MASK_CLIP_AND_DRAW,
            // Filtered out by the `!= Disabled` guard above.
            ClipMode::Disabled => RenderInstance::CLIP_ROLE_MEMBER,
        };
        let meta = RenderInstance::pack_clip_meta(role, cc.clamped().alpha_cutoff);
        return (rank + 1, meta);
    }
    // Otherwise: descendant of a clip-parent? Walk up ChildOf, take the
    // nearest clip-ancestor (innermost wins → single-level).
    let mut cur = sim.get::<ChildOf>(entity).map(|c| c.parent());
    while let Some(p) = cur {
        if let Some(cc) = sim.get::<ClipChildren>(p)
            && cc.mode != ClipMode::Disabled
            && let Some(rank) = ranks.rank(p)
        {
            let meta = RenderInstance::pack_clip_meta(RenderInstance::CLIP_ROLE_MEMBER, 0.0);
            return (rank + 1, meta);
        }
        cur = sim.get::<ChildOf>(p).map(|c| c.parent());
    }
    (RenderInstance::CLIP_GROUP_NONE, 0)
}

/// Fold the Mask2D / MaskInteraction role into `clip_meta` (bits 16-17),
/// reusing the stencil grouping packer (spec §6.4/§6.6). The mask feature
/// is GLOBAL (no `clip_group`); the role just routes the instance to the
/// mask pass.
///
/// **Precedence:** a clip-parent/member (`clip_group != 0`) is left
/// untouched — in W3 a sprite is either a clip participant OR a mask
/// participant, not both (the stencil buffer is reused per pass, so
/// combining them on one sprite is out of scope).
///
/// - [`Mask2D`] source → role SOURCE; its `alpha_cutoff` is packed into
///   the shared cutoff bits so the mark pass thresholds the silhouette.
/// - [`MaskInteraction`] responder → role INSIDE / OUTSIDE.
///
/// **Hidden / culled Mask2D source (known W3 behavior, audit LOW):** a
/// hidden/culled source emits no instance → it marks nothing → responders
/// behave as "no mask present" (Inside shows nowhere, Outside everywhere).
/// Intuitive (no source = no mask), but silent. Matches the hidden
/// clip-parent semantics in [`resolve_clip_grouping`].
fn resolve_mask_meta(sim: &World, entity: Entity, clip_group: u32, clip_meta: u32) -> u32 {
    if clip_group != RenderInstance::CLIP_GROUP_NONE {
        return clip_meta;
    }
    if let Some(m) = sim.get::<Mask2D>(entity) {
        // Source: pack the mask cutoff into the (currently-empty) cutoff
        // bits, then stamp the SOURCE role.
        let base = RenderInstance::pack_clip_meta(0, m.clamped().alpha_cutoff);
        return RenderInstance::with_mask_role(base, RenderInstance::MASK_ROLE_SOURCE);
    }
    if let Some(mi) = sim.get::<MaskInteraction>(entity) {
        let role = match mi.mode {
            MaskMode::VisibleInside => RenderInstance::MASK_ROLE_INSIDE,
            MaskMode::VisibleOutside => RenderInstance::MASK_ROLE_OUTSIDE,
            MaskMode::None => return clip_meta,
        };
        return RenderInstance::with_mask_role(clip_meta, role);
    }
    clip_meta
}

/// Select the sprite-sheet cell `frame` from a base UV rect
/// `[u_min, v_min, u_max, v_max]`, dividing it into an `hframes × vframes`
/// grid (anatomia §03 §3.4). Frame 0 = top-left, `col = frame % hframes`,
/// `row = frame / hframes` (row increases downward, matching V=0 = top).
/// `hframes`/`vframes` floor at 1 and `frame` is clamped into the grid,
/// so the default 1×1 sheet returns the input rect unchanged (no-op for
/// every legacy sprite). Render-only (PresentWorld), HR-5 exempt.
/// ⚠️ **`pub(crate)` desde 2026-09-01: o RETRATO de um prefab é o segundo leitor.** Ele compõe as
/// peças de uma receita e tem de mostrar **a mesma célula** que a tela mostra — sem isto, uma
/// sprite de folha aparecia no cartão com a grelha inteira espremida na célula.
pub(crate) fn sprite_sheet_subrect(
    uv: [f32; 4],
    hframes: u32,
    vframes: u32,
    frame: u32,
) -> [f32; 4] {
    let hf = hframes.max(1);
    let vf = vframes.max(1);
    if hf == 1 && vf == 1 {
        return uv;
    }
    let cells = hf.saturating_mul(vf).max(1);
    let frame = frame.min(cells - 1);
    let col = frame % hf;
    let row = frame / hf;
    let [u0, v0, u1, v1] = uv;
    let cw = (u1 - u0) / hf as f32;
    let ch = (v1 - v0) / vf as f32;
    let nu0 = u0 + col as f32 * cw;
    let nv0 = v0 + row as f32 * ch;
    [nu0, nv0, nu0 + cw, nv0 + ch]
}

/// Narrow a UV rect to a sprite's pixel-space `region_rect` (anatomia
/// §03 §3.5). `region` is `[x, y, w, h]` in SOURCE pixels; `(src_w,
/// src_h)` are the source image's pixel dimensions, so the rect maps to
/// the fraction `region / src` of the base `uv`. A zero/negative region
/// or unknown source dims leaves `uv` untouched (region = no-op). When
/// `filter_clip_half_texel` is `Some((htu, htv))`, the result is inset
/// by half a texel per side (Godot `region_filter_clip`) so bilinear
/// sampling can't bleed past the region edge into neighbouring atlas
/// content. The `htu`/`htv` are in the SAMPLED texture's UV space.
/// ⚠️ **`pub(crate)` pela razão da irmã acima** — o retrato é o segundo leitor.
pub(crate) fn region_subrect(
    uv: [f32; 4],
    region: [f32; 4],
    src_w: f32,
    src_h: f32,
    filter_clip_half_texel: Option<(f32, f32)>,
) -> [f32; 4] {
    let [u0, v0, u1, v1] = uv;
    let [rx, ry, rw, rh] = region;
    if rw <= 0.0 || rh <= 0.0 || src_w <= 0.0 || src_h <= 0.0 {
        return uv;
    }
    let du = u1 - u0;
    let dv = v1 - v0;
    // Region beyond the source edges is clamped to the base rect.
    let mut nu0 = (u0 + du * (rx / src_w)).clamp(u0, u1);
    let mut nv0 = (v0 + dv * (ry / src_h)).clamp(v0, v1);
    let mut nu1 = (u0 + du * ((rx + rw) / src_w)).clamp(u0, u1);
    let mut nv1 = (v0 + dv * ((ry + rh) / src_h)).clamp(v0, v1);
    if let Some((htu, htv)) = filter_clip_half_texel {
        nu0 += htu;
        nv0 += htv;
        nu1 -= htu;
        nv1 -= htv;
        // A region thinner than one texel would invert under the inset;
        // collapse it to its center so the sample stays inside.
        if nu1 < nu0 {
            let m = 0.5 * (nu0 + nu1);
            nu0 = m;
            nu1 = m;
        }
        if nv1 < nv0 {
            let m = 0.5 * (nv0 + nv1);
            nv0 = m;
            nv1 = m;
        }
    }
    [nu0, nv0, nu1, nv1]
}

/// Per-frame override that swaps a sprite entity's texture binding
/// for a transient one — used by the BG-Removal live preview (Lens F,
/// 2026-05-26) so the preview pixels render through the SAME sprite
/// pipeline (`Rgba8UnormSrgb` + `sprite.wgsl` + premul blend) as the
/// Apply bake. Replaces the previous Vello-overlay path, which
/// diverged from Apply in gamma + blend space and produced a visible
/// halo at edge pixels.
///
/// When the extract pass emits a `RenderInstance` for `entity_bits`,
/// it substitutes `texture_id` + `premultiplied` while leaving every
/// other instance field (world position, size, rotation, anchor,
/// tint, z_order) intact — so the override paints the SAME quad the
/// source sprite would have, but sampling from the preview texture.
#[derive(Copy, Clone, Debug)]
pub(crate) struct PreviewOverride {
    pub entity_bits: u64,
    pub texture_id: u32,
    pub premultiplied: bool,
}

/// Sim tick → extract pass. Caller provides the destructured
/// `AppGfx` refs.
#[allow(clippy::too_many_arguments)]
/// ⭐⭐ **A REGRA: esta entidade ocupa um rank como FORMA VETORIAL?** (ADR-0154 Fase 2)
///
/// `Some(id)` = sim, e este é o `VecPathRef` dela; `None` = não é forma (é sprite, ou não desenha
/// nada).
///
/// ⚠️ **`Sprite` VENCE, e a assimetria é deliberada.** Uma entidade com os dois é uma sprite que
/// referencia um path (proveniência de autoria, não geometria a desenhar) — contá-la duas vezes
/// dar-lhe-ia dois ranks e o objecto apareceria em duas faixas.
///
/// ⚠️ **A visibilidade NÃO se pergunta aqui**: quem a decide é o `off_canvas::draws_this_frame`, no
/// chamador, pela mesma porta dos sprites. *Uma segunda pergunta de visibilidade aqui seria a
/// segunda resposta que diverge.*
pub(super) fn vector_participant(sim: &World, entity: Entity) -> Option<u64> {
    if sim.get::<Sprite>(entity).is_some() {
        return None;
    }
    sim.get::<ph2d_ecs::VecPathRef>(entity).map(|vp| vp.0)
}

/// ⭐⭐⭐ **ESTA SPRITE É DESENHADA DEFORMADA, pelo Vello?** — a 2.ª mídia do esqueleto.
///
/// ⚠️ **Ela é a irmã da [`vector_participant`]**, e responde à mesma pergunta de outra família:
/// *quem ocupa o lugar dele na ordem mas não emite instância nenhuma, porque o Vello o desenha?*
///
/// ⚠️ **DERIVADA, nunca guardada:** uma sprite com pele é uma sprite que o esqueleto deforma, e a
/// pergunta responde-se olhando os dois componentes. Um terceiro componente a dizê-lo seria uma
/// fonte de verdade que pode discordar dos outros dois.
#[must_use]
pub(super) fn skinned_image(sim: &World, entity: Entity) -> bool {
    sim.get::<Sprite>(entity).is_some() && sim.get::<ph2d_skeleton_ecs::SkinBind>(entity).is_some()
}

/// ⭐⭐ **A CONVERSÃO: dos ranks que o ordenador decidiu para a ordem que o presente lê.**
///
/// ⛔ Ela não reordena nada — é leitura. E é uma função com nome, e não um laço inline, porque é
/// aqui que mora o defeito silencioso: **um rank de SPRITE que não seja registado** deixa aquele
/// índice no preenchimento por omissão, e uma forma que caia nele seria desenhada como sprite —
/// isto é, não seria desenhada.
pub(super) fn build_frame_order(
    sort_inputs: &[SortInput],
    vector_inputs: &[(Entity, u64)],
    scratch: &SortScratch,
    out: &mut crate::draw_bands::FrameOrder,
) {
    out.clear();
    for input in sort_inputs {
        if let Some(rank) = scratch.rank(input.entity) {
            let path = vector_inputs
                .iter()
                .find(|(e, _)| *e == input.entity)
                .map(|(_, id)| *id);
            out.record(rank, path);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    dt: f32,
    sim: &mut SimWorld,
    present: &mut PresentWorld,
    renderer: &SpriteRenderer,
    prop_state: &mut TransformPropagationState,
    worklist: &mut WorklistBuf,
    // Reusable buffers for the W3.T3.8 canonical sorting pipeline: the
    // scratch + the per-frame `SortInput` collection (both cleared and
    // reused — HR-3).
    sort_scratch: &mut SortScratch,
    sort_inputs: &mut Vec<SortInput>,
    // Tool live-preview overrides: for each sprite with a matching entry,
    // its `RenderInstance` is emitted with `texture_id` + `premultiplied`
    // replaced. Empty = emit every sprite from its own `Sprite` source. A
    // slice (not one `Option`) so SEVERAL sprites can preview at once — the
    // ACTIVE painted sprite AND a non-selected sprite used as the brush Shape.
    preview_overrides: &[PreviewOverride],
    // Project pixels-per-meter — converts a sprite's intrinsic-px
    // `offset` into the LOCAL meters `resolve_anchor` works in.
    pixels_per_meter: f32,
    // Active camera's visibility-layer cull mask (W3.T3.12): a sprite
    // with a `VisibilityLayer` that doesn't intersect this mask is
    // skipped. `u32::MAX` (default) = no culling.
    cull_mask: u32,
    // Project-default sampling (W3.T3.11): the filter/repeat an
    // all-`Inherit` sprite resolves to (from the project image filter).
    default_filter: FilterMode,
    default_repeat: RepeatMode,
    // **A FOLHA ABERTA** (Enio, 2026-08-23): a entidade cuja grelha se desdobra em células
    // fantasma no canvas. `None` = ninguém, e o frame é byte-idêntico ao de antes desta feature.
    //
    // ⚠️ **Um parâmetro, e não um componente**, pela razão do `preview_overrides` ao lado: isto é
    // uma vista, não conteúdo. Um componente entraria no undo, no save e no snapshot — e o artista
    // reabriria o projeto com a folha aberta, sem se lembrar de a ter aberto.
    sheet_preview: Option<ph2d_ecs::Entity>,
    // ⭐⭐ **A ordem TOTAL deste quadro** (ADR-0154 Fase 2) — as duas famílias numa lista só,
    // indexada pelo rank que o `compute_sort_ranks_into` acabou de decidir. O presente parte-a em
    // faixas e desenha faixa a faixa, alternando de motor.
    //
    // ⚠️ **Out-param e não valor de retorno**: esta função já devolve por escrita em `present`, e
    // um `Vec` novo por quadro seria uma alocação por quadro (HR-3) — o `clear` mantém a
    // capacidade.
    order_out: &mut crate::draw_bands::FrameOrder,
) {
    // As formas vetoriais que entraram na ordem, na travessia: `(entidade, VecPathRef.0)`.
    let mut vector_inputs: Vec<(ph2d_ecs::Entity, u64)> = Vec::new();
    // Sim tick: bouncing motion. Single substep per frame for the
    // M5 demo (we don't yet honor the FixedStep substep count for
    // gameplay — that lands in M10 with the physics integrator).
    {
        let mut q = sim.world_mut().query::<(&mut Transform, &mut Velocity)>();
        for (mut t, mut vel) in q.iter_mut(sim.world_mut()) {
            let mut p = t.translation;
            let mut v = vel.0;
            p += v * dt;
            if p.x.abs() > WORLD_HALF {
                v.x = -v.x;
                p.x = p.x.clamp(-WORLD_HALF, WORLD_HALF);
            }
            if p.y.abs() > WORLD_HALF {
                v.y = -v.y;
                p.y = p.y.clamp(-WORLD_HALF, WORLD_HALF);
            }
            t.translation = p;
            vel.0 = v;
        }
    }

    // Extract (ADR-0021 + ADR-0025): hierarchical Transform →
    // GlobalTransform propagation plus per-entity sprite emit.
    // `propagate_transforms` walks the `ChildOf` tree once, and the
    // closure spawns one mirror entity per sim entity in PresentWorld
    // carrying `(SimRef, GlobalTransform)` plus an optional
    // `RenderInstance` for sprite-bearing entities.
    let atlas = renderer.atlas();
    present.clear();
    // Canonical sorting pipeline (W3.T3.8): the old per-frame DFS
    // counter is replaced by the full 7-stage order (SortingLayer / Z /
    // YSort / SortingGroup / ShowBehindParent / DFS). We can't know an
    // entity's rank until the whole tree is walked, so the walk emits
    // each `RenderInstance` with a placeholder `z_order` + records a
    // `SortInput` (entity + world position); after the walk the pipeline
    // ranks them and we stamp `z_order` via a `SimRef` query. Reusing
    // `sort_inputs` (cleared, capacity retained) keeps this allocation-
    // free (HR-3). The DFS fallback still protects bakes that promote an
    // Atlas sprite (texture_id 0) to Individual (>0) from floating up.
    sort_inputs.clear();
    ph2d_ecs::extract!(*sim => *present, |sim_w, present_w| {
        propagate_transforms(
            sim_w,
            prop_state,
            present_w,
            worklist,
            |sim, present, sim_entity, gt| {
                let builder_id = present.spawn((SimRef(sim_entity), gt)).id();
                // ⭐⭐⭐ **UMA porta, TRÊS razões** — o olho da Hierarquia / a peça de uma receita,
                // a máscara de camadas contra a da câmara, e o rect do `OnScreenEnabler`. As três
                // vivem em [`ph2d_entity_visibility::off_canvas::draws_this_frame`], onde cada uma tem gate e prova
                // de mutação; soltas aqui dentro do closure, nenhuma delas era observável.
                let t = gt.translation();
                let drawn =
                    ph2d_entity_visibility::off_canvas::draws_this_frame(sim, sim_entity, cull_mask, [t.x, t.y]);
                let override_for_entity = preview_overrides
                    .iter()
                    .copied()
                    .find(|o| o.entity_bits == sim_entity.to_bits());
                // W2.T4: CookedTexture sprites are now LIVE. The
                // `cooked_texture_bridge` loader pass (run just before this
                // extract) resolved each `logical_id` for the device tier and
                // uploaded its KTX2 into the renderer's cooked-texture cache;
                // the `CookedTexture` arm below reads back the cached
                // `texture_id`. A sprite whose logical texture wasn't uploaded
                // (no cooked tier on this device) returns early → no
                // RenderInstance → invisible, the same shape as a
                // hidden/culled sprite (the W2.T2 skip-guard's behavior, now
                // resolved per-sprite instead of blanket).
                // ⭐⭐⭐ **A FORMA VETORIAL ENTRA NA MESMA ORDEM** (ADR-0154 Fase 2, report do
                // Enio de 2026-08-30). Ela não emite `RenderInstance` nenhum — o Vello desenha-a —,
                // mas ocupa o **lugar** dela na ordem total, e é isso que faltava: as duas famílias
                // eram ordenadas por dois pipelines independentes e coladas por um `over` fixo.
                //
                // ⚠️ **Tem de ser AQUI, dentro da travessia**, e não num laço à parte: o
                // `compute_sort_ranks_into` exige os `inputs` em **DFS pre-order**, e é essa ordem
                // que semeia o `draw_order` e o desempate entre irmãos. Um laço de query por fora
                // daria a ordem do arquétipo, que muda com qualquer `insert`.
                //
                // ⚠️ E ela passa pela MESMA porta de visibilidade (`drawn`): uma forma escondida
                // pelo olho da Hierarquia não pode ocupar um rank, senão ela abre uma faixa que
                // não desenha nada e paga a colagem à mesma.
                if drawn && let Some(id) = vector_participant(sim, sim_entity) {
                    vector_inputs.push((sim_entity, id));
                    sort_inputs.push(SortInput {
                        entity: sim_entity,
                        world_pos: t,
                    });
                }
                // ⭐⭐⭐ **UMA IMAGEM PRESA AO ESQUELETO NÃO EMITE INSTÂNCIA** — quem a desenha
                // é o Vello, deformada ([`crate::skeleton_skin_image`]), exactamente como já
                // acontece com uma forma vectorial oito linhas acima.
                //
                // ⛔⛔ **Sem isto a arte aparece DUAS vezes:** a original, por deformar, fica por
                // baixo — e assim que o artista dobra o braço ela espreita por fora da deformada.
                //
                // ⚠️ **É um facto DERIVADO, e não uma bandeira guardada.** Escrever `Visibility`
                // aqui poria a shell a discutir com o olho da Hierarquia — *duas fontes de verdade
                // para o mesmo bool, e a que o artista toca é a que perde*, que é a lei que o
                // menu *Window* desta mesma linha já pagou.
                if drawn
                    && !skinned_image(sim, sim_entity)
                    && let Some(spr) = sim.get::<Sprite>(sim_entity)
                {
                    emit::sprite(
                        sim,
                        present,
                        builder_id,
                        sim_entity,
                        gt,
                        spr,
                        atlas,
                        renderer,
                        override_for_entity,
                        pixels_per_meter,
                        default_filter,
                        default_repeat,
                        sort_inputs,
                        sheet_preview,
                    );
                }
            },
        );
        // Rank every emitted sprite through the canonical pipeline, then
        // stamp the rank onto each present `RenderInstance` (matched back
        // to its sim entity via `SimRef`). The renderer sorts by
        // `(z_order, texture_id)`; ranks are unique so the texture_id
        // tie-break never fires.
        compute_sort_ranks_into(sort_scratch, sim_w, sort_inputs);
        // ⭐⭐ **A ORDEM TOTAL, na forma de que o presente precisa** (ADR-0154 Fase 2). Ela é a
        // LEITURA do que o ordenador decidiu — nada aqui reordena.
        //
        // ⚠️ Percorre-se `sort_inputs` (a lista de entrada, em DFS) e não `vector_inputs` sozinha:
        // o rank de um SPRITE também tem de ser registado, senão a família daquele rank fica no
        // preenchimento por omissão e uma forma vetorial que caia lá seria desenhada como sprite —
        // isto é, não seria desenhada.
        build_frame_order(sort_inputs, &vector_inputs, sort_scratch, order_out);
        let mut q = present_w.query::<(&SimRef, &mut RenderInstance)>();
        for (sim_ref, mut ri) in q.iter_mut(present_w) {
            if let Some(rank) = sort_scratch.rank(sim_ref.0) {
                ri.z_order = rank;
            }
            // ADR-0070-amendment-7: tag the clip-stencil group from the
            // now-final ranks (clip_group = clip_parent rank + 1), then
            // fold in the Mask2D / MaskInteraction role (bits 16-17).
            let (clip_group, clip_meta) = resolve_clip_grouping(sim_w, sim_ref.0, sort_scratch);
            ri.clip_group = clip_group;
            ri.clip_meta = resolve_mask_meta(sim_w, sim_ref.0, clip_group, clip_meta);
        }
    });
}

#[cfg(test)]
#[path = "sim_extract_sprite_sheet_tests.rs"]
mod sprite_sheet_tests;

#[cfg(test)]
#[path = "sim_extract_region_subrect_tests.rs"]
mod region_subrect_tests;

#[cfg(test)]
#[path = "sim_extract_cascade_tint_tests.rs"]
mod cascade_tint_tests;

#[cfg(test)]
#[path = "sim_extract_band_rule_tests.rs"]
mod band_rule_tests;

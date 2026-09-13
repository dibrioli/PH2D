//! **A emissão de UMA sprite no extract** — a instância (a ligação à textura, o quad desdobrado, a amostragem, a
//! região e a grelha) e os quads que ela gera (o 9-slice, a folha aberta, a pré-visualização animada). Filho do
//! [`super`] por `#[path]` (OBRA 3 da `line/render-bodies`): o `run` chama [`sprite`] no sítio exacto do bloco, dentro
//! da mesma condição (`drawn`, não presa ao esqueleto, com `Sprite`), que fica lá — os gates leem-na ali.

use super::*;
use ph2d_ecs::GlobalTransform;
use ph2d_render::TextureAtlas;

/// A sprite `spr` desta entidade: resolve a instância e põe os quads dela no `present`. Sai cedo, sem emitir nada,
/// quando a textura cozida ainda não subiu — como o `return` do closure fazia.
#[allow(clippy::too_many_arguments)]
pub(super) fn sprite(
    sim: &World,
    present: &mut World,
    builder_id: Entity,
    sim_entity: Entity,
    gt: GlobalTransform,
    spr: &Sprite,
    atlas: &TextureAtlas,
    renderer: &SpriteRenderer,
    override_for_entity: Option<PreviewOverride>,
    pixels_per_meter: f32,
    default_filter: FilterMode,
    default_repeat: RepeatMode,
    sort_inputs: &mut Vec<SortInput>,
    sheet_preview: Option<Entity>,
) {
    let p = gt.translation();
    // ADR-0070-amendment-4: pass the FULL 2x2 world basis
    // (col0, col1) to the shader instead of decomposing it
    // to atan2(col0) + per-column scale. The old
    // decomposition collapsed any skew (a non-orthogonal
    // basis) into a rotated rectangle — skew read as
    // rotation + stretched scale. The basis carries
    // rotation + scale + skew EXACTLY, and the shader maps
    // the local quad through it (sheared parallelogram).
    // `size`/`anchor` stay LOCAL (the basis applies scale),
    // so no double-scaling. Column-major affine:
    //   col0 = basis.xy (x axis), col1 = basis.zw (y axis).
    let affine = gt.affine();
    let basis = [affine[0], affine[1], affine[2], affine[3]];
    // M14.5 C: branch on the sprite source. Atlas
    // sprites resolve UV via `region_uv`; individual
    // sprites use the full (0..1) UV rect and carry
    // the renderer-side texture_id so the batcher
    // can pick the right bind group at draw time.
    let (atlas_uv, texture_id) = match spr.source {
        ph2d_render::SpriteSource::Atlas { key } => (
            atlas.region_uv(key),
            ph2d_render::RenderInstance::ATLAS_TEXTURE_ID,
        ),
        ph2d_render::SpriteSource::Individual { texture_id } => ([0.0, 0.0, 1.0, 1.0], texture_id),
        // W2.T4: resolve the tier-agnostic logical id to the
        // cooked `texture_id` the loader pass uploaded. The id
        // is in the `COOKED_TEXTURE_ID_BIT` namespace so the
        // renderer's draw loop binds the cooked store. Not yet
        // uploaded (no device tier / asset) → skip this sprite
        // this frame (no RenderInstance, invisible).
        ph2d_render::SpriteSource::CookedTexture { logical_id } => {
            match renderer.cooked_texture_id(logical_id) {
                Some(id) => ([0.0, 0.0, 1.0, 1.0], id),
                None => return,
            }
        }
    };
    // Lens F (2026-05-26): if a tool's live preview
    // claims this entity, substitute the texture binding
    // for the preview's transient Individual texture +
    // its premultiplied flag. UV stays the full unit
    // rect (preview textures are atlas-free); every
    // other instance field (transform, size, anchor,
    // tint, z) is identical so the override paints the
    // SAME quad the source sprite would have.
    let (atlas_uv, texture_id, premultiplied_flag) = if let Some(ov) = override_for_entity {
        ([0.0, 0.0, 1.0, 1.0], ov.texture_id, ov.premultiplied)
    } else {
        (atlas_uv, texture_id, spr.premultiplied)
    };
    // ⚠️ **E NUMA FOLHA, O QUAD DESDOBRA-SE** (Enio, 2026-08-23, com foto). A UV
    // acima passou a ser o rect INTEIRO da textura de pré-visualização — que é o
    // bake da imagem toda —, mas o quad continuava a ser o de UMA célula: as oito
    // saíam esmagadas 8:1 dentro dela.
    //
    // ⚠️ O caminho do PONTEIRO faz a mesma conta
    // (`ph2d_sprite_screen::sprite_image_to_screen_affine`), e é por isso que ele
    // chama a MESMA função: pintar-se-ia num sítio e ver-se-ia noutro.
    // A grelha (ausente = uma célula), lida uma vez para os três consumidores
    // do quad desdobrado abaixo.
    let sheet_grid = sim.get::<ph2d_ecs::SpriteGrid>(sim_entity).copied();
    let quad_size = override_for_entity
        .and(sheet_grid)
        .and_then(|g| crate::render_loop::sim_extract_sheet::unfolded_quad(spr, g))
        .unwrap_or(spr.size);
    let quad_anchor = spr.resolve_anchor(pixels_per_meter);
    // Record this sprite for the sort pipeline; `z_order`
    // is patched to the computed rank after the walk.
    sort_inputs.push(SortInput {
        entity: sim_entity,
        world_pos: p,
    });
    let z_order = 0u32;
    let (sampling, uv_xform, cascade_tint, flip_uv) =
        style(sim, sim_entity, spr, default_filter, default_repeat);
    // Region sub-UV (anatomia §03 §3.5): when enabled,
    // narrow the base rect to `region_rect` (source pixels
    // → UV). Applied BEFORE the sheet grid so the grid
    // divides the region, not the whole source. Skipped
    // under a tool preview override (the preview texture is
    // a full-frame bake whose pixel space doesn't match the
    // authored source's `region_rect`). Default
    // `region_enabled = false` → no-op for legacy sprites.
    // Dimensoes em pixels da FONTE. Hoisted (2026-08-21): a regiao ja' as
    // pedia, e o 9-slice precisa das MESMAS para medir as bordas. Duas copias
    // deste match divergiriam no dia em que nascesse uma quarta `SpriteSource`.
    let src_dims = match spr.source {
        ph2d_render::SpriteSource::Atlas { key } => atlas.region_px(key),
        ph2d_render::SpriteSource::Individual { texture_id } => {
            renderer.individual().dims(texture_id)
        }
        ph2d_render::SpriteSource::CookedTexture { .. } => renderer.cooked().dims(texture_id),
    };
    // ADR-0164 F1 passo 6: janela e grelha são componentes; ausentes = a
    // textura inteira e uma célula, que é o que os campos v4 significavam.
    let region = sim.get::<ph2d_ecs::SpriteRegion>(sim_entity).copied();
    let grid = sim
        .get::<ph2d_ecs::SpriteGrid>(sim_entity)
        .copied()
        .unwrap_or(ph2d_ecs::SpriteGrid::SINGLE);
    let atlas_uv = region_uv(atlas_uv, region, override_for_entity, src_dims, spr, atlas);
    // Sprite-sheet sub-UV (anatomia §03 §3.4): divide the
    // (possibly region-narrowed) atlas_uv rect into an
    // hframes×vframes grid and select `frame`'s cell. The
    // default 1×1 grid is a no-op, so legacy sprites render
    // unchanged. Skipped under a tool preview override
    // (audit E-3): the transient preview texture is a
    // full-frame bake, not a sheet, so slicing it would
    // show only one cell.
    // ⚠️ **A UV da GRELHA INTEIRA**, guardada antes de a célula ser escolhida —
    // é ela que a folha aberta percorre. Derivá-la de volta a partir da célula
    // seria multiplicar por `hframes` uma conta que este sítio já tem exacta.
    let sheet_uv = atlas_uv;
    let atlas_uv = if override_for_entity.is_some() {
        atlas_uv
    } else {
        sprite_sheet_subrect(atlas_uv, grid.hframes, grid.vframes, grid.frame)
    };
    let base = RenderInstance {
        world_pos: [p.x, p.y],
        // LOCAL size — the basis applies world scale. ⚠️ Numa folha sob
        // pré-visualização isto é o quad DESDOBRADO (ver acima).
        size: quad_size,
        atlas_uv,
        tint: cascade_tint,
        basis,
        texture_id,
        // Flag the BG-Removal-baked premultiplied texture
        // so the fragment skips its post-sample premultiply
        // (fringe fix). Straight for every other sprite.
        premultiplied: if premultiplied_flag { 1.0 } else { 0.0 },
        // Effective pivot offset in LOCAL meters — folds
        // the Godot `centered`/`offset` authoring onto the
        // tool `anchor` (resolve_anchor). The basis maps it
        // to world with the quad corners, so the quad orbits
        // `world_pos` (the pivot) under skew too. Default
        // (centered, offset 0, anchor 0) = [0,0] (legacy).
        anchor: quad_anchor,
        per_corner_tint: sim
            .get::<ph2d_ecs::SpriteCornerTint>(sim_entity)
            .map_or(ph2d_ecs::SpriteCornerTint::IDENTITY.0, |c| c.0),
        opacity: spr.opacity,
        flip_uv,
        z_order,
        sampling,
        uv_xform,
        // Clip-stencil grouping needs the final z_order
        // rank (clip_group = clip_parent rank + 1), so it is
        // stamped in the post-walk pass below, like z_order
        // itself (ADR-0070-amendment-7). Placeholder here.
        clip_group: RenderInstance::CLIP_GROUP_NONE,
        // ADR-0070-amendment-9: a cena tem UM lugar por objecto na
        // hierarquia, então o `z_order` já a ordena por inteiro e a
        // sub-ordem não tem quem a autore aqui. Ela existe para o único
        // produtor que emite N linhas na MESMA fatia de z — um sink de
        // Motion. `0` ⇒ o desempate por textura, byte-idêntico.
        sub_order: 0,
        clip_meta: 0,
    };
    spawn(
        sim,
        present,
        builder_id,
        sim_entity,
        gt,
        spr,
        base,
        region,
        grid,
        src_dims,
        atlas_uv,
        pixels_per_meter,
        basis,
        sheet_preview,
        override_for_entity,
        sheet_grid,
        sheet_uv,
        quad_anchor,
    );
}

/// Os quads da instância: ela sozinha (e os extras da folha aberta e da pré-visualização animada), ou os nove do
/// 9-slice. O `builder` é a entidade que o closure já gerou para esta `SimRef`, re-obtida pelo id.
#[allow(clippy::too_many_arguments)]
fn spawn(
    sim: &World,
    present: &mut World,
    builder_id: Entity,
    sim_entity: Entity,
    gt: GlobalTransform,
    spr: &Sprite,
    base: RenderInstance,
    region: Option<ph2d_ecs::SpriteRegion>,
    grid: ph2d_ecs::SpriteGrid,
    src_dims: Option<(u32, u32)>,
    atlas_uv: [f32; 4],
    pixels_per_meter: f32,
    basis: [f32; 4],
    sheet_preview: Option<Entity>,
    override_for_entity: Option<PreviewOverride>,
    sheet_grid: Option<ph2d_ecs::SpriteGrid>,
    sheet_uv: [f32; 4],
    quad_anchor: [f32; 2],
) {
    let mut builder = present.entity_mut(builder_id);
    // **9-SLICE** (spec Sprite 03 §3.5): um sprite fatiado desenha-se como até
    // NOVE quads. `patches_for` devolve `None` para todo sprite normal — e para
    // um 9-slice que não produziria quad nenhum —, e aí o caminho abaixo é o de
    // sempre, byte-idêntico.
    //
    // ⚠️ Os nove partilham o `SimRef`: é isso que faz o passe pós-caminhada
    // (z_order + clip) servir os nove sem saber que 9-slice existe. Os oito
    // extra levam `SlicePatchMirror` para o HUD não os contar como entidades.
    match crate::render_loop::sim_extract_slice::patches_for(
        sim.get::<ph2d_ecs::SliceNine>(sim_entity),
        spr,
        crate::render_loop::sim_extract_slice::SliceSource {
            region,
            grid,
            src_dims,
        },
        atlas_uv,
        pixels_per_meter,
        basis,
    ) {
        None => {
            builder.insert(base);
            // **OS EXTRAS DE PRÉ-VISUALIZAÇÃO** (Enio, 2026-08-23), os dois de uma
            // vez porque partilham o `drop` do empréstimo:
            //
            // 1. **A FOLHA ABERTA** — as outras células da grelha, esmaecidas e no
            //    lugar delas, para o artista VER onde os cortes caem.
            // 2. **A PRÉ-VISUALIZAÇÃO ANIMADA** — com o quad desdobrado a mostrar
            //    a folha inteira enquanto se pinta, o `Sprite::frame` deixa de ter
            //    efeito visível: o artista pinta oito desenhos e não vê a animação
            //    que eles formam. Uma célula, por fora da folha, ao ritmo do tocador.
            //
            // ⚠️ A 1 não corre sob pré-visualização de ferramenta e a 2 corre **só**
            // sob ela: são os dois lados do mesmo momento — ou se está a inspecionar
            // a grelha, ou se está a pintá-la.
            let ghosts = crate::render_loop::sim_extract_sheet::should_open(
                sheet_preview,
                sim_entity,
                override_for_entity.is_some(),
                sheet_grid,
            );
            // ⚠️ **A base é `[0,0,1,1]`, e NÃO o `sheet_uv`.** Sob override a
            // textura é a transitória da ferramenta — o bake da imagem inteira —,
            // e o `sheet_uv` é um rect do ATLAS. Fatiar um pelo outro amostraria
            // uma lasca arbitrária: um sprite de atlas mostraria lixo na
            // pré-visualização, e o de textura própria acertava por os dois serem
            // o rect unitário. *Um acerto que depende da fonte não é um acerto.*
            let anim_preview = override_for_entity.and_then(|_| {
                crate::render_loop::sim_extract_sheet::anim_preview_quad(
                    spr,
                    grid,
                    [0.0, 0.0, 1.0, 1.0],
                )
            });
            if ghosts.is_some() || anim_preview.is_some() {
                // O `drop` é pelo EMPRÉSTIMO, como no braço do 9-slice ao lado.
                #[allow(clippy::drop_non_drop)]
                drop(builder);
                for i in 0..ghosts.unwrap_or(0) {
                    if let Some((uv, off)) =
                        crate::render_loop::sim_extract_sheet::cell(spr, grid, sheet_uv, i)
                    {
                        present.spawn((
                            SimRef(sim_entity),
                            gt,
                            crate::render_loop::sim_extract_sheet::ghost(&base, uv, off),
                            ph2d_render::nine_slice::SlicePatchMirror,
                        ));
                    }
                }
                if let Some((uv, off)) = anim_preview {
                    let mut ri = base;
                    ri.atlas_uv = uv;
                    // ⚠️ UMA célula, e não o quad desdobrado: é a imagem que a
                    // sprite mostra quando a folha volta a fechar.
                    ri.size = spr.size;
                    ri.anchor = [quad_anchor[0] + off[0], quad_anchor[1] + off[1]];
                    present.spawn((
                        SimRef(sim_entity),
                        gt,
                        ri,
                        ph2d_render::nine_slice::SlicePatchMirror,
                    ));
                }
            }
        }
        Some(patches) => {
            // O fan-out em si é uma função PURA (e testada); aqui só resta
            // colocá-las. Sem alocar: array fixo + contagem (HR-3).
            let (insts, n) = crate::render_loop::sim_extract_slice::instances(&base, &patches);
            builder.insert(insts[0]);
            // ⚠️ **O `drop` é pelo EMPRÉSTIMO, não pelo valor.** `builder` tem o
            // `present` emprestado mutavelmente; sem o largar aqui, os oito
            // `present.spawn` abaixo não compilam. O lint `drop_non_drop` avisa
            // que o tipo não tem `Drop` — verdade, e irrelevante: o que acaba
            // aqui é o empréstimo.
            #[allow(clippy::drop_non_drop)]
            drop(builder);
            for ri in insts.iter().take(n).skip(1) {
                present.spawn((
                    SimRef(sim_entity),
                    gt,
                    *ri,
                    ph2d_render::nine_slice::SlicePatchMirror,
                ));
            }
        }
    }
}

/// A janela da região estreitada na UV da fonte (com o meio-texel do filtro) — fora de uma pré-visualização de
/// ferramenta, cuja textura é o bake inteiro.
fn region_uv(
    atlas_uv: [f32; 4],
    region: Option<ph2d_ecs::SpriteRegion>,
    override_for_entity: Option<PreviewOverride>,
    src_dims: Option<(u32, u32)>,
    spr: &Sprite,
    atlas: &TextureAtlas,
) -> [f32; 4] {
    if let Some(region) = region.filter(|_| override_for_entity.is_none()) {
        match src_dims {
            Some((sw, sh)) => {
                // Half-texel size in the SAMPLED texture's UV
                // space: atlas sprites sample the shared atlas
                // (1 / size_px); individual sprites sample
                // their own texture (1 / w, 1 / h).
                let half_texel = if region.filter_clip {
                    let (tw, th) = match spr.source {
                        ph2d_render::SpriteSource::Atlas { .. } => {
                            let s = atlas.size_px.max(1) as f32;
                            (s, s)
                        }
                        // Cooked + Individual both sample their
                        // OWN native-resolution texture, so the
                        // half-texel is `1 / (w, h)` (W2.T4).
                        ph2d_render::SpriteSource::Individual { .. }
                        | ph2d_render::SpriteSource::CookedTexture { .. } => {
                            (sw.max(1) as f32, sh.max(1) as f32)
                        }
                    };
                    Some((0.5 / tw, 0.5 / th))
                } else {
                    None
                };
                region_subrect(atlas_uv, region.rect, sw as f32, sh as f32, half_texel)
            }
            None => atlas_uv,
        }
    } else {
        atlas_uv
    }
}

/// A amostragem, a transformação de UV, a tinta em cascata e as flags empacotadas da instância.
fn style(
    sim: &World,
    sim_entity: Entity,
    spr: &Sprite,
    default_filter: FilterMode,
    default_repeat: RepeatMode,
) -> (u32, [f32; 4], [f32; 4], u32) {
    // W3.T3.11: per-node filter (→ sampler) + repeat (→ shader
    // wrap, in flip_uv) + UV tiling, resolved up the ChildOf chain.
    let rfilter = resolve_texture_filter(sim, sim_entity, default_filter);
    let rrepeat = resolve_texture_repeat(sim, sim_entity, default_repeat);
    let sampling = ph2d_render::RenderInstance::pack_sampling(rfilter as u8, rrepeat as u8);
    let uv_xform = sim
        .get::<UvTransform>(sim_entity)
        .map_or(ph2d_render::RenderInstance::IDENTITY_UV_XFORM, |t| {
            [t.scale[0], t.scale[1], t.offset[0], t.offset[1]]
        });
    // Sprite-Inspector-v2 v4 channel collapse (W1.T1.8/T1.10,
    // anatomia §4.2/§4.3): `self_tint × tint × Π(ancestor.tint)`.
    // The per-sprite `collapsed_tint` (self_tint × tint) lives +
    // is unit-tested in ph2d-render; `cascade_tint_with_ancestors`
    // folds in the inherited modulate chain (W2 — so a parent's
    // `tint` actually tints its children, while `self_tint` does
    // not). Skipped under a preview override (the override emits
    // the SAME tint as the source). per_corner_tint + opacity
    // pass through unchanged.
    let cascade_tint = cascade_tint_with_ancestors(sim, sim_entity, spr);
    // Packed flags (amendment-3/-6): bit0=flip_x · bit1=flip_y ·
    // bit2=tint_fill · bits3-4=resolved repeat (single-sourced
    // helpers keep the bit layout in sync with the WGSL decode).
    // §10 BlendMode: per-sprite, absent = Mix (tag 0 =
    // zero-regression default). Packed into flip_uv bits 5-7
    // (CPU-only — the renderer keys draw runs on it).
    let blend_tag = sim
        .get::<ph2d_ecs::BlendMode>(sim_entity)
        .copied()
        .unwrap_or_default()
        .tag();
    let flip_uv =
        ph2d_render::RenderInstance::pack_flip_flags(spr.flip_x, spr.flip_y, spr.tint_fill)
            | ph2d_render::RenderInstance::pack_repeat_bits(rrepeat as u8)
            | ph2d_render::RenderInstance::pack_blend_bits(blend_tag);
    (sampling, uv_xform, cascade_tint, flip_uv)
}

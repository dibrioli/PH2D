//! A metade de GPU e de overlay da ponte da Remoção de fundo — o upload da prévia e as dicas por
//! cima dela. Filho por ASSUNTO (por `#[path]`) do [`super`], que guarda o downcast (ADR-0040 §3)
//! e chama estas duas funções no sítio exacto onde os blocos estavam.

use crate::app_state::{BgremovalPreview, BgremovalPreviewGpu};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::toast::{Toast, ToastQueue};
use ph2d_host::WindowSize;
use ph2d_i18n::tr_with;
use ph2d_render::{Camera2d, Sprite, SpriteRenderer};
// ⭐ O afim saiu para uma FOLHA porque quatro assuntos o partilhavam (HOWTO §1.2).
use ph2d_sprite_screen::sprite_image_to_screen_affine;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::{Affine, Brush, Circle, Color, Stroke, VectorScene};
use std::sync::Arc;

/// O ciclo de vida da textura de GPU da prévia (Lens F): sobe os pixels premultiplicados quando o
/// `Arc` da cache muda, e liberta a ranhura quando a prévia some.
pub(super) fn upload_preview(
    bgremoval_preview: &Option<BgremovalPreview>,
    bgremoval_preview_gpu: &mut Option<BgremovalPreviewGpu>,
    renderer: &mut SpriteRenderer,
    toasts: &mut ToastQueue,
) {
    // ── GPU lifecycle for the live-preview texture (Lens F, 2026-05-26) ──
    // Replaces the old Vello image draw of the preview RGBA. Owns a
    // transient `IndividualTextureStore` slot; uploads the
    // premultiplied preview pixels into it whenever the CPU-side
    // cache gets a fresh `Arc` (or the selection drifts to a
    // different sprite, or the size changes). NEXT frame's
    // `sim_extract` reads `bgremoval_preview_gpu` directly to emit a
    // `PreviewOverride` — the sprite pipeline samples from THIS
    // texture instead of the source sprite's original binding, so
    // the live preview goes through the SAME wgpu sprite shader as
    // Apply (`Rgba8UnormSrgb` + premul blend) → byte-for-byte parity,
    // no Vello-internal gamma/blend divergence.
    //
    // Premultiplication: byte-space `premultiply_rgba8` mirrors
    // EXACTLY what the Apply path does in
    // `SpriteImage::into_premultiplied`. Both produce identical bytes
    // that, when uploaded to `Rgba8UnormSrgb`, the GPU decodes to
    // identical linear values for the sprite shader's bilinear
    // sample. The gamma-correct variant from earlier (Fix C) is
    // intentionally NOT used here — its job was to compensate for
    // Vello's `Rgba8Unorm` raw-byte interpretation, which no longer
    // applies once the preview leaves the Vello path entirely.
    //
    // The 1-frame lag introduced by reading `bgremoval_preview_gpu`
    // on the NEXT frame's extract (this dispatch runs after the
    // current frame's `sim_extract`) is imperceptible: the live
    // preview is a continuous animation and a single ~16ms delay
    // between parameter change and visible response is below the
    // human flicker threshold.
    match bgremoval_preview.as_ref() {
        Some(preview) => {
            let cache_token = Arc::as_ptr(&preview.rgba) as usize;
            let needs_upload = match *bgremoval_preview_gpu {
                None => true,
                Some(gpu) => {
                    gpu.arc_token != cache_token
                        || gpu.entity_bits != preview.entity_bits
                        || gpu.width != preview.width
                        || gpu.height != preview.height
                }
            };
            if needs_upload {
                let mut premul_bytes = (*preview.rgba).clone();
                ph2d_render::premultiply_rgba8(&mut premul_bytes);
                let upload_result: Result<u32, _> = match *bgremoval_preview_gpu {
                    Some(gpu) => renderer
                        .replace_individual_pixels(
                            gpu.texture_id,
                            preview.width,
                            preview.height,
                            &premul_bytes,
                        )
                        .map(|()| gpu.texture_id),
                    None => {
                        renderer.acquire_individual(preview.width, preview.height, &premul_bytes)
                    }
                };
                match upload_result {
                    Ok(texture_id) => {
                        *bgremoval_preview_gpu = Some(BgremovalPreviewGpu {
                            texture_id,
                            width: preview.width,
                            height: preview.height,
                            arc_token: cache_token,
                            entity_bits: preview.entity_bits,
                        });
                    }
                    Err(e) => {
                        // Audit T1.6 R7 J1-3: surface GPU upload errors
                        // via toast instead of an eprintln the user
                        // never reads. The next frame retries
                        // automatically (we drop the stale slot below).
                        toasts.push(Toast::error(tr_with(
                            "shell.bgremoval_preview_gpu.preview_upload_failed",
                            &[("e", &e)],
                        )));
                        // Drop the stale slot; next frame retries.
                        release_preview_texture(renderer, bgremoval_preview_gpu);
                    }
                }
            }
        }
        None => {
            release_preview_texture(renderer, bgremoval_preview_gpu);
        }
    }
}

/// As duas dicas que a ferramenta publicou para o canvas: `(a tinta da máscara, o anel do pincel)`.
type DicasDoCanvas<'a> = (Option<&'a (Arc<Vec<u8>>, u32, u32)>, Option<(f32, u32)>);

/// ⭐⭐⭐ **O QUE ESTE QUADRO DESENHA NO CANVAS** — a tinta da máscara (que vai pelo passe de
/// SPRITES, com a malha da arte) e o anel do pincel (que é uma dica de UI e fica no Vello).
///
/// ⚠️ **As duas numa porta porque são a mesma FASE**, e não porque partilhem código: o
/// `dispatch` do bridge é o painel + a cache + o Apply, e isto é o canvas. *Uma fase, um assunto.*
#[allow(clippy::too_many_arguments)]
pub(super) fn paint_canvas(
    dicas: DicasDoCanvas<'_>,
    bgr: &mut crate::bgremoval_shell::BgremovalShell,
    // `(apresentação, simulação)` — a malha posada vive no primeiro, a pose e a sprite no segundo.
    mundos: (&ph2d_ecs::World, &SimWorld),
    // O enquadramento do ecrã e a cena onde o anel é desenhado.
    ecra: (&Camera2d, WindowSize, &mut VectorScene, Theme),
    renderer: &mut SpriteRenderer,
    toasts: &mut ToastQueue,
) {
    let (protect_tint, brush_ring) = dicas;
    let (present, sim) = mundos;
    let (camera, window_size, vector_scene, theme) = ecra;
    drive_tint(
        protect_tint,
        bgr.preview.as_ref().map(|p| p.entity_bits),
        present,
        (&mut bgr.tint_gpu, &mut bgr.tint_extra),
        renderer,
        toasts,
    );
    draw_overlays(
        &bgr.preview,
        brush_ring,
        sim,
        camera,
        window_size,
        vector_scene,
        theme,
    );
}

/// **A TINTA DA MÁSCARA, das duas metades numa chamada só** — a ranhura sobe e a instância
/// nasce. ⚠️ Ela só existe com PRÉVIA, como antes do passe de sprites: *é a prévia que ela anota*,
/// e sem dono a ranhura é libertada.
fn drive_tint(
    protect_tint: Option<&(Arc<Vec<u8>>, u32, u32)>,
    dono: Option<u64>,
    present: &ph2d_ecs::World,
    // ⚠️ As duas metades viajam como PAR: a instância indexa a ranhura, e uma sem a outra desenha
    // uma textura libertada ou nada.
    tinta: (
        &mut Option<BgremovalPreviewGpu>,
        &mut ph2d_render::LiftedInstances,
    ),
    renderer: &mut SpriteRenderer,
    toasts: &mut ToastQueue,
) {
    let (gpu, extra) = tinta;
    upload_tint(
        dono.and(protect_tint),
        dono.unwrap_or_default(),
        gpu,
        renderer,
        toasts,
    );
    tint_instances(present, *gpu, extra);
}

/// **A ranhura de GPU da tinta** — gémea do [`upload_preview`] — o ciclo de vida da textura
/// dela, gémeo do [`upload_preview`].
///
/// ⛔⛔ **A alternativa está MEDIDA e REFUTADA** (sonda `skin_pieces_gpu_cost`, corrida 2026-09-15):
/// desenhá-la por cima da arte dobrada com um recorte do Vello por triângulo deixa **costuras**
/// (`10 580` px fora da barra numa arte translúcida com `216` peças, `41 732` com `3 456`) e a cura
/// barata das costuras — dilatar os recortes — **piora** exactamente no caso translúcido, porque a
/// faixa sobreposta compõe-se duas vezes (`55 978` px, pior desvio `124`). ⛔ E acima disso os
/// buffers do Vello são de tamanho FIXO: o que os estoura degrada **em silêncio**, que é o
/// *«Smooth bugado quebrando a forma»* que este mesmo módulo já pagou.
///
/// ⭐ No passe de sprites não há costura **por construção**: dois triângulos que partilham uma
/// aresta são rasterizados pela regra de canto, e cada centro de pixel pertence a UM deles.
fn upload_tint(
    protect_tint: Option<&(Arc<Vec<u8>>, u32, u32)>,
    entity_bits: u64,
    tint_gpu: &mut Option<BgremovalPreviewGpu>,
    renderer: &mut SpriteRenderer,
    toasts: &mut ToastQueue,
) {
    let Some((rgba, tw, th)) = protect_tint else {
        release_preview_texture(renderer, tint_gpu);
        return;
    };
    let token = Arc::as_ptr(rgba) as usize;
    let precisa = match *tint_gpu {
        None => true,
        Some(g) => {
            g.arc_token != token
                || g.entity_bits != entity_bits
                || g.width != *tw
                || g.height != *th
        }
    };
    if !precisa {
        return;
    }
    // ⚠️ **Pré-multiplicada, como a prévia** — o passe de sprites compõe com `src + dst·(1−a)`, e
    // uma fonte de alfa DIRECTO ali sai clara na borda de cada dab (o mesmo halo que a prévia
    // pagou em 2026-05-26).
    let mut bytes = (**rgba).clone();
    ph2d_render::premultiply_rgba8(&mut bytes);
    let subida = match *tint_gpu {
        Some(g) => renderer
            .replace_individual_pixels(g.texture_id, *tw, *th, &bytes)
            .map(|()| g.texture_id),
        None => renderer.acquire_individual(*tw, *th, &bytes),
    };
    match subida {
        Ok(texture_id) => {
            *tint_gpu = Some(BgremovalPreviewGpu {
                texture_id,
                width: *tw,
                height: *th,
                arc_token: token,
                entity_bits,
            });
        }
        Err(e) => {
            toasts.push(Toast::error(format!(
                "Bg Removal: upload da tinta da máscara falhou ({e}). \
                 Tentando novamente no próximo frame."
            )));
            release_preview_texture(renderer, tint_gpu);
        }
    }
}

/// ⭐⭐⭐ **A tinta como UMA instância do passe de sprites, com a MALHA da arte por baixo.**
///
/// ⚠️ **O `sub_order` é o que a põe POR CIMA**, e é o campo que existe exactamente para isto (o
/// doc dele: *«a grandeza que faltava não era “mais fundo”, era “mais à frente dentro do mesmo
/// fundo”»*). ⛔ Empatar em tudo e confiar na ordem de inserção **não** serve: a chave de ordenação
/// desempata por `texture_id`, e o da ranhura da tinta tanto pode ser maior como menor que o da
/// arte — a tinta desapareceria POR BAIXO dela, dependendo da ordem em que as ranhuras foram pedidas.
///
/// ⚠️ Todos os outros campos são COPIADOS da instância da arte (pose, base, âncora, recorte,
/// opacidade, tinta de objecto): a tinta é uma dica **daquela** sprite, e um objecto escondido não
/// mostra a máscara dele.
fn tint_instances(
    present: &ph2d_ecs::World,
    tint_gpu: Option<BgremovalPreviewGpu>,
    out: &mut ph2d_render::LiftedInstances,
) {
    out.clear();
    let Some(gpu) = tint_gpu else { return };
    let Some((arte, malha)) = ph2d_render::drawn_instance_of(present, gpu.entity_bits) else {
        return;
    };
    let mut inst = *arte;
    inst.texture_id = gpu.texture_id;
    // A textura da tinta é individual — o rect INTEIRO, como a prévia.
    inst.atlas_uv = [0.0, 0.0, 1.0, 1.0];
    inst.premultiplied = 1.0;
    inst.uv_xform = ph2d_render::RenderInstance::IDENTITY_UV_XFORM;
    inst.sub_order = inst.sub_order.saturating_add(1);
    out.push(inst, malha);
}

/// O anel do pincel por cima da prévia (Vello, dica de UI).
#[allow(clippy::too_many_arguments)]
fn draw_overlays(
    bgremoval_preview: &Option<BgremovalPreview>,
    brush_ring: Option<(f32, u32)>,
    sim: &SimWorld,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    theme: Theme,
) {
    // ── Protection-mask tint + brush-size ring (Vello, UI hints) ──────────
    // The two remaining Vello overlays. They are UI affordances, not
    // image data — alpha-blended hints on top of the live preview.
    // They can stay in Vello because they don't need byte-for-byte
    // parity with anything. Gated on the preview being loaded so they
    // disappear in sync with the sprite-pipeline live preview.
    if let Some(preview) = bgremoval_preview {
        let entity = ph2d_ecs::Entity::from_bits(preview.entity_bits);
        // ⚠️ Pose de MUNDO — vide o doc do `sprite_image_to_screen_affine`.
        if let (Some(tr), Some(sprite)) = (
            ph2d_ecs::world_transform(sim.world(), entity),
            sim.world().get::<Sprite>(entity),
        ) {
            // A grelha desta sprite (ADR-0164 F1 passo 6) — ausente = uma célula.
            let grid = sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied();
            // ⭐⭐⭐ **A TINTA SAIU DO VELLO** (2026-09-15) — ela era desenhada aqui com o afim do
            // QUAD DE REPOUSO, logo sobre uma arte presa ao esqueleto e DOBRADA ela aparecia num
            // sítio e a prévia (que já vai pelo passe de sprites, deformada) noutro. Hoje é uma
            // INSTÂNCIA do passe de sprites com a MESMA malha — ver [`tint_instances`], e a recusa
            // medida do caminho por recortes do Vello no doc da `ph2d_render::drawn_instance_of`.
            // Brush-size ring at the cursor — the source-px radius
            // mapped to screen via the footprint scale (extracted
            // from the affine's per-axis magnitude).
            if let (Some((r_src, src_w)), Some((cur_x, cur_y))) = (
                brush_ring,
                crate::input_dispatch::protect_brush::brush_cursor(),
            ) && src_w > 0
            {
                // `Affine` matrix is [a b c; d e f]; the column vector
                // `[a, d]` is image-X mapped to screen — its magnitude
                // is the per-pixel scale on the X axis.
                let m = sprite_image_to_screen_affine(
                    preview.width,
                    preview.height,
                    tr,
                    sprite,
                    grid,
                    camera,
                    window_size,
                )
                .as_coeffs();
                let pixel_scale = (m[0] * m[0] + m[1] * m[1]).sqrt() as f32; // |col 0|
                let src_to_screen = pixel_scale * preview.width as f32 / src_w as f32;
                let r_screen = r_src * src_to_screen;
                let accent = ColorToken::Accent.resolve(theme);
                let color = Color::from_rgba8(accent.r, accent.g, accent.b, 255);
                vector_scene.inner_mut().stroke(
                    &Stroke::new(StrokeToken::Default.px() as f64),
                    Affine::IDENTITY,
                    &Brush::Solid(color),
                    None,
                    &Circle::new((cur_x as f64, cur_y as f64), r_screen as f64),
                );
            }
        }
    }
}

/// Release the live-preview's `IndividualTextureStore` slot (if any)
/// and zero the cache. Called when the preview cache turns `None`
/// (tool deactivated, source unavailable, post-Apply transition) and
/// when an upload errors out — next frame's lifecycle re-acquires
/// from scratch.
fn release_preview_texture(
    renderer: &mut SpriteRenderer,
    bgremoval_preview_gpu: &mut Option<BgremovalPreviewGpu>,
) {
    if let Some(gpu) = bgremoval_preview_gpu.take() {
        renderer.individual_mut().release(gpu.texture_id);
    }
}

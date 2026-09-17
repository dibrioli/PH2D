//! ⭐ **O que o SELECTOR DE COR pede à shell** — irmão do [`super::forwarding`] pelo tecto de LOC,
//! cortado por RESPONSABILIDADE.
//!
//! O selector é chrome puro: ele sabe desenhar uma roda de cor e não sabe fazer as duas coisas que
//! um utilizador lhe pede à mesma — **ler um pixel da GPU** (o conta-gotas) e **abrir um diálogo de
//! ficheiro** (importar/exportar paleta). As duas aterram aqui, e as duas são pedidos que o
//! `forward_to_hero` DRENA, nunca chamadas que o chrome faça.

/// Service a pending palette import/export (opens an `rfd` file dialog, then applies via the
/// `ph2d_color::palette` engine). Split out of [`forward_to_hero`] to keep that hot path readable.
pub(super) fn handle_palette_io(
    hero: &mut ph2d_editor_core::HeroScreen,
    parent: ph2d_editor_core::NodeId,
    io_kind: ph2d_editor_core::interaction::PaletteIoKind,
) {
    use ph2d_color::palette::{self, PaletteData, PaletteFormat};
    use ph2d_editor_core::interaction::PaletteIoKind;
    let fmt_of = |path: &std::path::Path| {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(PaletteFormat::from_extension)
            .unwrap_or(PaletteFormat::Gpl)
    };
    match io_kind {
        PaletteIoKind::Import => {
            let Some(path) = rfd::FileDialog::new()
                .add_filter(
                    ph2d_i18n::tr("shell.forwarding.colour_palette"),
                    &["gpl", "hex", "txt", "css", "ase", "aco"],
                )
                .pick_file()
            else {
                return;
            };
            match std::fs::read(&path).map(|b| palette::parse(fmt_of(&path), &b)) {
                Ok(Ok(p)) => {
                    let colors = p
                        .colors
                        .iter()
                        .map(|c| ph2d_tokens::ColorValue::from_rgba8(c[0], c[1], c[2], c[3]))
                        .collect();
                    // Import ADDS a named palette (the file's name) + activates it.
                    hero.store.blender_import_palette(parent, &p.name, colors);
                    hero.store.sync_blender_palette_name_buffer(parent);
                }
                Ok(Err(e)) => eprintln!("[ph2d] palette import: {e}"),
                Err(e) => eprintln!("[ph2d] palette read: {e}"),
            }
        }
        PaletteIoKind::Export => {
            let colors: Vec<[u8; 4]> = hero
                .store
                .blender_palette(parent)
                .map(|s| s.iter().map(|c| c.rgba).collect())
                .unwrap_or_default();
            if colors.is_empty() {
                return;
            }
            let Some(path) = rfd::FileDialog::new()
                .add_filter(ph2d_i18n::tr("shell.forwarding.gimp_palette"), &["gpl"])
                .add_filter(ph2d_i18n::tr("shell.forwarding.hex_list"), &["hex"])
                .add_filter(
                    ph2d_i18n::tr("shell.forwarding.adobe_swatch_exchange"),
                    &["ase"],
                )
                .add_filter(ph2d_i18n::tr("shell.forwarding.adobe_color"), &["aco"])
                .set_file_name("palette.gpl")
                .save_file()
            else {
                return;
            };
            let data = PaletteData {
                name: ph2d_i18n::tr("shell.forwarding.palette").to_string(),
                colors,
            };
            if let Err(e) = std::fs::write(&path, palette::write(fmt_of(&path), &data)) {
                eprintln!("[ph2d] palette export: {e}");
            }
        }
    }
}

/// Sample the painted layer COMPOSITE under the screen pixel `(px, py)` for the colour-picker
/// eyedropper, when the Painter is active and the click lands on the selected sprite (not a panel).
/// Devolve a cor **AUTORADA** ali (a composição de camadas do Painter), ou `None` para cair na
/// leitura do ECRÃ ([`ph2d_render::screen_color`]). ⚠️ **As duas existem, e não é redundância:** o
/// ecrã passa pelo tonemap e pelo dither da descida, logo escolher uma cor que se acabou de pintar
/// e recebê-la de volta com `±1` por canal seria um round-trip que não fecha. Este caminho é o que
/// integra o conta-gotas com o sistema de camadas; o outro é o que responde em **todo o resto do
/// ecrã**, que é onde ele devolvia `#00000000`. Mirrors the footprint mapping in `painter_canvas_input` / the BgRemoval
/// eyedropper; takes disjoint `AppGfx` fields by ref so it composes with the live `&mut hero_screen`.
#[allow(clippy::too_many_arguments)]
pub(super) fn painter_eyedropper_sample(
    tools: &mut ph2d_editor_core::ToolRegistry,
    sim: &ph2d_ecs::SimWorld,
    present: &mut ph2d_ecs::World,
    camera: &ph2d_render::Camera2d,
    window: ph2d_host::WindowSize,
    selection: Option<u64>,
    on_panel: bool,
    px: f32,
    py: f32,
) -> Option<[u8; 4]> {
    if on_panel {
        return None; // a click on a panel is panel-targeted, not a canvas sample
    }
    let painter_active = tools
        .active()
        .map(|t| t.id() == ph2d_editor_core::ToolId::new("painter"))
        .unwrap_or(false);
    if !painter_active {
        return None;
    }
    let bits = selection?;
    let entity = ph2d_ecs::Entity::from_bits(bits);
    // ⚠️ Pose de MUNDO: um sprite filho tem a cadeia do pai por cima, e sem ela o afim mapeia o
    // ponteiro para fora da pegada dele.
    let tr = ph2d_ecs::world_transform(sim.world(), entity)?;
    let sprite = sim.world().get::<ph2d_render::Sprite>(entity)?;
    // A grelha desta sprite (ADR-0164 F1 passo 6) — ausente = uma célula.
    let sprite_grid = sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied();
    let painter = tools
        .active_mut()?
        .as_any_mut()
        .downcast_mut::<ph2d_tool_painter::PainterTool>()?;
    let (iw, ih) = painter.canvas_size();
    if iw == 0 || ih == 0 {
        return None;
    }
    // Screen → image-px via the FULL sprite affine (size · scale · rotation · anchor · camera) — the
    // same geometry the brush uses, so the eyedropper tracks the sprite under any resize, AR change OR
    // rotation. `u`/`v` is the image fraction; not clamped (a Repeat-Image neighbour lands outside `[0,1]`).
    let affine = ph2d_sprite_screen::sprite_image_to_screen_affine(
        iw,
        ih,
        tr,
        sprite,
        sprite_grid,
        camera,
        window,
    );
    let img = affine.inverse() * ph2d_vector::Point::new(f64::from(px), f64::from(py));
    // ⭐⭐⭐ **A ARTE DOBRADA MANDA NO CONTA-GOTAS TAMBÉM** (2026-09-15). O afim acima é o do QUAD DE
    // REPOUSO: numa arte presa ao esqueleto e dobrada ele aponta para o texel errado, e o artista
    // recolhe uma cor que não é a que está debaixo do dedo — *o mesmo defeito que a pincelada tinha,
    // na ferramenta ao lado*. A lei dos três estados é a [`ph2d_render::mesh_uv`], e `Quad` deixa o
    // caminho de sempre intocado (é ele que carrega a grelha da folha e o *Repeat Image*).
    //
    // ⚠️ **`starting = true` e pegada `[0, 0]`:** um clique é sempre o primeiro ponto de um gesto, e
    // uma porta que APONTA pergunta por um PONTO — a pegada só tem sentido para quem vai pintar um
    // disco. ⇒ fora da arte desenhada a porta RECUSA, e aqui isso é `None`, que é exactamente o que
    // o chamador já faz com um clique fora da sprite: cair na leitura do ecrã.
    let malha = ph2d_render::mesh_uv(
        present,
        bits,
        camera.screen_to_world((px, py), window),
        true,
        [0.0, 0.0],
    );
    if malha == ph2d_render::MeshUv::Refuse {
        return None;
    }
    let (u, v) = match malha {
        ph2d_render::MeshUv::Use { u, v, .. } => (u, v), // a UV de repouso do texel desenhado ALI
        _ => (
            (img.x / f64::from(iw)) as f32,
            (img.y / f64::from(ih)) as f32,
        ),
    };
    // **Repeat Image**: the preview tiles the sprite 3×3, so the eyedropper works on any of the 8
    // neighbour tiles AND the original — accept the 3×3 UV grid and WRAP the sample back onto the
    // canvas (`rem_euclid`), exactly like the neighbour-paint hit region. Without Repeat, only the
    // central sprite is sampleable.
    let repeat = painter.repeat_image();
    let (lo, hi) = if repeat { (-1.0, 2.0) } else { (0.0, 1.0) };
    if !((lo..=hi).contains(&u) && (lo..=hi).contains(&v)) {
        return None; // outside the sampleable region → fall back to the rendered-overlay readback
    }
    let (su, sv) = if repeat {
        (u.rem_euclid(1.0), v.rem_euclid(1.0))
    } else {
        (u, v)
    };
    painter.sample_composite_at_uv(su, sv)
}

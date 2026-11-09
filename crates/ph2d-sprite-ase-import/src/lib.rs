//! **O IMPORT de um `.ase`/`.aseprite`** — largar o ficheiro NATIVO do Aseprite na janela (Enio,
//! 2026-08-23). Movido da shell (P14e): a shell só compõe.
//!
//! O ficheiro lê-se pelo MESMO leitor do Pixel e deriva-se pelas portas do Pixel Sprite
//! ([`ph2d_pixel_cena::importa_ase`]): tags (`tag_do_aseprite`), pivô e formas por célula
//! ([`ph2d_ecs::SpriteCells`], P14e D1), a cor sem as camadas Normal Map e a normal (que a família liga
//! à imagem pela porta `normal_map`). Daí nasce **uma** sprite com grelha e biblioteca.
//!
//! ⚠️ **A ORDEM DOS QUADROS É O CONTRATO**: uma `AnimationTag` é um intervalo sobre as CÉLULAS, e
//! a folha empacota-se em linha, da esquerda para a direita e de cima para baixo ([`pack`]).
//!
//! A nota ao artista diz o ritmo por quadro que entrou: nenhum campo do Inspector o edita.

use ph2d_ecs::{AnimTagError, AnimationTag, SimWorld, SpriteAnimations, SpriteAnimator};
use ph2d_i18n::tr_with;
use ph2d_render::SpriteRenderer;
use std::path::Path;

/// O lado máximo de uma folha, em pixels: o `max_texture_dimension_2d` do default do wgpu (o
/// recurso é a GPU).
pub const MAX_SHEET_EDGE_PX: u32 = 8192;

/// As duas extensões que o Aseprite escreve (uma LISTA: o diálogo de ficheiro enumera-as).
/// ⚠️ `.ase` é disputado com o *Swatch Exchange* da Adobe (paleta): a porta é que separa.
pub const ASE_EXTENSIONS: &[&str] = &["ase", "aseprite"];

/// Este caminho reconhece o ficheiro?
#[must_use]
pub fn is_ase_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|e| ASE_EXTENSIONS.iter().any(|x| e.eq_ignore_ascii_case(x)))
}

/// **Como os N quadros se arrumam**: `(colunas, linhas)`. Uma tira sempre que couber; senão
/// quase-quadrada, nunca mais larga do que cabe.
#[must_use]
pub fn grid_for(frames: usize, frame_w: u32, _frame_h: u32) -> (u32, u32) {
    let n = frames.max(1) as u32;
    let fit_w = (MAX_SHEET_EDGE_PX / frame_w.max(1)).max(1);
    if n <= fit_w {
        return (n, 1);
    }
    let cols = ((n as f64).sqrt().ceil() as u32).clamp(1, fit_w);
    // Não cabe de todo: devolve o que cabe, e quem chama recusa com o número.
    (cols, n.div_ceil(cols))
}

/// **Empacota os quadros na folha**, em linha; as células que sobram ficam transparentes.
#[must_use]
pub fn pack(frames: &[Vec<u8>], fw: u32, fh: u32, cols: u32, rows: u32) -> Vec<u8> {
    let (fw, fh) = (fw as usize, fh as usize);
    let (cw, ch) = (cols as usize, rows as usize);
    let sheet_w = fw * cw;
    let mut out = vec![0_u8; sheet_w * fh * ch * 4];
    for (i, frame) in frames.iter().enumerate() {
        let (cx, cy) = (i % cw, i / cw);
        if cy >= ch {
            break;
        }
        for row in 0..fh {
            let src = row * fw * 4;
            let Some(line) = frame.get(src..src + fw * 4) else {
                break;
            };
            let dst = ((cy * fh + row) * sheet_w + cx * fw) * 4;
            out[dst..dst + fw * 4].copy_from_slice(line);
        }
    }
    out
}

/// A linha ao artista quando a tag tem ritmo por quadro (diz **o que entrou**).
#[must_use]
pub fn nota_do_ritmo(tag: &AnimationTag) -> Option<String> {
    let per = &tag.per_frame_ms;
    (!per.is_empty()).then(|| {
        tr_with(
            "shell.ase_import.has_per_frame_timing",
            &[
                ("t", &(tag.name)),
                ("per_frame_ms", &(per.len())),
                ("into", &(per.iter().copied().min().unwrap_or(tag.frame_ms))),
                (
                    "into2",
                    &(per.iter().copied().max().unwrap_or(tag.frame_ms)),
                ),
            ],
        )
    })
}

/// As notas da biblioteca: o clip inventado (sem tags), o ritmo por quadro de cada tag e cada tag
/// recusada, NOMEADA com o motivo.
#[must_use]
pub fn notas(
    lib: &SpriteAnimations,
    sem_tags: bool,
    recusas: &[(String, AnimTagError)],
    stem: &str,
    quadros: usize,
) -> Vec<String> {
    let mut out = Vec::new();
    if sem_tags {
        out.push(tr_with(
            "shell.ase_import.the_file_has_no_tags",
            &[("stem", &stem), ("frames", &quadros)],
        ));
        return out;
    }
    out.extend(lib.iter().filter_map(nota_do_ritmo));
    out.extend(recusas.iter().map(|(name, e)| {
        tr_with(
            "shell.ase_import.tag_was_not_imported",
            &[("name", name), ("e_", &format!("{e:?}"))],
        )
    }));
    out
}

/// O que aconteceu a um `.ase` largado.
pub enum AseImportResult {
    Ok {
        name: String,
        frames: usize,
        animations: usize,
        bits: u64,
        /// O que ficou por trás, em linguagem de artista. Uma linha por assunto.
        notes: Vec<String>,
        /// A folha da normal (camada Normal Map), com a grelha da cor: `(w, h, rgba)`. Quem a liga à
        /// imagem é a família (`ph2d_app_sprite_anim::ase_import`, porta `normal_map`).
        normal: Option<(u32, u32, Vec<u8>)>,
    },
    Err {
        name: String,
        error: String,
    },
}

/// Importa um `.ase`: lê pelo leitor do Pixel, empacota numa folha, sobe UMA textura, e nasce
/// **uma** sprite com a grelha, a biblioteca (a tocar a primeira), o pivô e as formas por célula
/// e a normal da imagem.
#[allow(clippy::too_many_arguments)]
pub fn import_ase(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &ph2d_asset::AssetDb,
    next_cell: &mut u32,
    atlas_asset_map: &mut std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
    path: &Path,
    anchor_world: [f32; 2],
    pixels_per_meter: f32,
) -> AseImportResult {
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("sprite")
        .to_owned();
    let fail = |error: String| AseImportResult::Err {
        name: name.clone(),
        error,
    };
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => return fail(tr_with("shell.ase_import.read", &[("e", &e)])),
    };
    let m_por_px = 1.0 / f64::from(pixels_per_meter.max(f32::MIN_POSITIVE));
    let imp = match ph2d_pixel_cena::importa_ase(&bytes, &name, m_por_px) {
        Ok(i) => i,
        Err(e) => return fail(e),
    };
    let (w, h) = (imp.largura, imp.altura);
    let (cols, rows) = grid_for(imp.cor.len(), w, h);
    let (sheet_w, sheet_h) = (cols * w, rows * h);
    if sheet_w > MAX_SHEET_EDGE_PX || sheet_h > MAX_SHEET_EDGE_PX {
        return fail(tr_with(
            "shell.ase_import.frames_of_x_need_a_x",
            &[
                ("frames", &(imp.cor.len())),
                ("width", &w),
                ("height", &h),
                ("sheet_w", &sheet_w),
                ("sheet_h", &sheet_h),
                ("MAX_SHEET_EDGE_PX", &MAX_SHEET_EDGE_PX),
            ],
        ));
    }
    let pixels = pack(&imp.cor, w, h, cols, rows);
    let cell = *next_cell;
    let (_, bits) = match ph2d_image_import::spawn_rgba(
        sim,
        renderer,
        asset_db,
        cell,
        sheet_w,
        sheet_h,
        pixels,
        ph2d_core::Vec2::new(anchor_world[0], anchor_world[1]),
        pixels_per_meter,
        atlas_asset_map,
        &name,
    ) {
        Ok(v) => v,
        Err(e) => return fail(e),
    };
    *next_cell += 1;
    let entity = ph2d_ecs::Entity::from_bits(bits);
    let mut notes: Vec<String> = imp
        .avisos
        .iter()
        .map(|&a| ph2d_panel_pixel_export::textos::aviso_ase(a))
        .collect();
    notes.extend(notas(
        &imp.clips,
        imp.sem_tags,
        &imp.recusas,
        &name,
        imp.cor.len(),
    ));
    liga(sim, entity, &imp, cols, rows, m_por_px);
    AseImportResult::Ok {
        name,
        frames: imp.cor.len(),
        animations: imp.clips.iter().count(),
        bits,
        notes,
        normal: imp
            .normal
            .as_ref()
            .map(|n| (sheet_w, sheet_h, pack(n, w, h, cols, rows))),
    }
}

/// O que o import liga à sprite já nascida: a grelha (o tamanho de UMA célula), a biblioteca a
/// tocar a primeira, o filtro de pixel art, o pivô da célula 0 como o da folha, e o pivô e as
/// formas por célula ([`ph2d_pixel_cena::liga_celulas`]).
pub fn liga(
    sim: &mut SimWorld,
    entity: ph2d_ecs::Entity,
    imp: &ph2d_pixel_cena::SpriteImportado,
    cols: u32,
    rows: u32,
    m_por_px: f64,
) {
    let w = sim.world_mut();
    if let Some(mut s) = w.get_mut::<ph2d_render::Sprite>(entity) {
        s.size = [s.size[0] / cols as f32, s.size[1] / rows as f32];
        let size = s.size;
        if let Some(a) = imp.celulas.anchor(0, size) {
            s.anchor = a;
        }
    }
    let first = imp
        .clips
        .iter()
        .next()
        .map(|t| t.name.clone())
        .unwrap_or_default();
    let mut player = SpriteAnimator::new(first);
    player.playing = true;
    player.autoplay = true;
    w.entity_mut(entity).insert((
        ph2d_ecs::SpriteGrid {
            hframes: cols,
            vframes: rows,
            frame: 0,
            sheet: 0,
        },
        imp.clips.clone(),
        player,
        // Um `.ase` é pixel art: ampliar não pode borrar (smoke do dono, 05/11).
        ph2d_ecs::TextureFilter(ph2d_ecs::FilterMode::Nearest),
    ));
    ph2d_pixel_cena::liga_celulas(w, entity, imp.celulas.clone(), m_por_px);
}

#[cfg(test)]
mod tests;

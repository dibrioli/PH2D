//! **O IMPORT de um `.ase`/`.aseprite`** — largar o ficheiro NATIVO do Aseprite na janela (Enio,
//! 2026-08-23). Movido da shell (P14e): a shell só compõe.
//!
//! O ficheiro lê-se pelo MESMO leitor do Pixel e deriva-se pelas portas do Pixel Sprite
//! ([`ph2d_pixel_cena::importa_ase`]): tags (`tag_do_aseprite`), pivô e formas por célula
//! ([`ph2d_ecs::SpriteCells`], P14e D1), a cor sem as camadas Normal Map e a normal, ligada à
//! imagem pela porta [`EscolherNormal`] (`normal_map` da família). Daí nasce **uma** sprite com grelha e biblioteca.
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
    escolher_normal: EscolherNormal,
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
    let porta = Porta {
        sim,
        renderer,
        asset_db,
        next_cell,
        atlas_asset_map,
        anchor_world,
        pixels_per_meter,
        escolher_normal,
    };
    nasce(porta, name, &imp)
}

/// O que o import precisa do app para nascer a sprite.
pub struct Porta<'a> {
    pub sim: &'a mut SimWorld,
    pub renderer: &'a mut SpriteRenderer,
    pub asset_db: &'a ph2d_asset::AssetDb,
    pub next_cell: &'a mut u32,
    pub atlas_asset_map: &'a mut std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
    pub anchor_world: [f32; 2],
    pub pixels_per_meter: f32,
    pub escolher_normal: EscolherNormal,
}

/// Nasce **uma** sprite do import (de um `.ase` ou de uma folha): a folha empacotada numa textura,
/// a grelha, a biblioteca, o pivô e as formas por célula e a normal.
pub fn nasce(
    p: Porta<'_>,
    name: String,
    imp: &ph2d_pixel_cena::SpriteImportado,
) -> AseImportResult {
    let Porta {
        sim,
        renderer,
        asset_db,
        next_cell,
        atlas_asset_map,
        anchor_world,
        pixels_per_meter,
        escolher_normal,
    } = p;
    let fail = |error: String| AseImportResult::Err {
        name: name.clone(),
        error,
    };
    let m_por_px = 1.0 / f64::from(pixels_per_meter.max(f32::MIN_POSITIVE));
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
        .map(|&a| ph2d_pixel_io::frases::aviso_ase(a))
        .collect();
    notes.extend(notas(
        &imp.clips,
        imp.sem_tags,
        &imp.recusas,
        &name,
        imp.cor.len(),
    ));
    liga(sim, entity, imp, cols, rows, m_por_px);
    if let Some(n) = &imp.normal {
        let id = asset_db.insert_image_rgba8(sheet_w, sheet_h, pack(n, w, h, cols, rows));
        if let Err(e) = escolher_normal(sim.world(), asset_db, atlas_asset_map, entity, Some(id)) {
            notes.push(e);
        }
    }
    AseImportResult::Ok {
        name,
        frames: imp.cor.len(),
        animations: imp.clips.iter().count(),
        bits,
        notes,
    }
}

/// O `.json` é de uma folha ANIMADA (a do Pixel/Aseprite: os quadros têm `duration`)? Os atlas
/// hand-packed (TexturePacker) não têm, e continuam o import de N sprites.
#[must_use]
pub fn e_folha_animada(json: &[u8]) -> bool {
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(json) else {
        return false;
    };
    let tem = |q: &serde_json::Value| q.get("duration").is_some();
    match &v["frames"] {
        serde_json::Value::Array(a) => a.first().is_some_and(tem),
        serde_json::Value::Object(o) => o.values().next().is_some_and(tem),
        _ => false,
    }
}

/// **Importa a folha que o Pixel exporta** (`folha.json` + a imagem dele + `<imagem>_n.png` ao
/// lado): o MESMO Sprite do `.ase` ([`ph2d_pixel_cena::importa_folha`] + [`nasce`]).
pub fn import_folha(p: Porta<'_>, json_path: &Path) -> AseImportResult {
    let m_por_px = 1.0 / f64::from(p.pixels_per_meter.max(f32::MIN_POSITIVE));
    match le_folha(json_path, m_por_px) {
        Ok((name, imp)) => nasce(p, name, &imp),
        Err((name, error)) => AseImportResult::Err { name, error },
    }
}

/// Lê a folha largada: o nome e o import; o erro é a frase ao artista (inglês, por chave).
///
/// # Errors
/// `(nome, frase)`: um ficheiro que não se lê, ou a folha recusada (Split).
pub fn le_folha(
    json_path: &Path,
    m_por_px: f64,
) -> Result<(String, ph2d_pixel_cena::SpriteImportado), (String, String)> {
    let name = json_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("sheet")
        .to_owned();
    let ler =
        |c: &Path| std::fs::read(c).map_err(|e| tr_with("shell.ase_import.read", &[("e", &e)]));
    let json = ler(json_path)
        .and_then(|b| {
            String::from_utf8(b).map_err(|e| tr_with("shell.ase_import.read", &[("e", &e)]))
        })
        .map_err(|e| (name.clone(), e))?;
    let imagem = serde_json::from_str::<serde_json::Value>(&json)
        .ok()
        .and_then(|v| v["meta"]["image"].as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("{name}.png"));
    let pasta = json_path.parent().unwrap_or_else(|| Path::new("."));
    let png = ler(&pasta.join(&imagem)).map_err(|e| (name.clone(), e))?;
    let raiz = Path::new(&imagem)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&name);
    let normal = std::fs::read(pasta.join(format!("{raiz}_n.png"))).ok();
    ph2d_pixel_cena::importa_folha(&png, &json, normal.as_deref(), &name, m_por_px)
        .map(|imp| (name.clone(), imp))
        .map_err(|e| (name.clone(), ph2d_pixel_io::frases::erro_abrir(&e)))
}

/// As frases `(é erro, texto)` de um import e os bits da sprite nascida: a linha do nome (quadros,
/// animações) e as notas, ou o erro com o nome.
#[must_use]
pub fn frases_do_import(r: AseImportResult) -> (Vec<(bool, String)>, Option<u64>) {
    match r {
        AseImportResult::Ok {
            name,
            frames,
            animations,
            bits,
            notes,
        } => {
            let linha = tr_with(
                "shell.import_router.frames_animations",
                &[
                    ("name", &name),
                    ("frames", &frames),
                    ("animations", &animations),
                ],
            );
            let todas = std::iter::once(linha).chain(notes).map(|t| (false, t));
            (todas.collect(), Some(bits))
        }
        AseImportResult::Err { name, error } => (
            vec![(
                true,
                tr_with(
                    "shell.input_drop.sheet",
                    &[("name", &name), ("error", &error)],
                ),
            )],
            None,
        ),
    }
}

/// A porta da normal da IMAGEM (`ph2d_app_sprite_anim::normal_map::escolher_normal`): a família
/// passa-a, porque ela mora na crate que depende desta.
pub type EscolherNormal = fn(
    &ph2d_ecs::World,
    &ph2d_asset::AssetDb,
    &std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
    ph2d_ecs::Entity,
    Option<ph2d_asset::AssetId>,
) -> Result<(), String>;

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

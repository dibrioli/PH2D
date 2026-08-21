//! Image-tool texture chokepoint — the SINGLE sanctioned path for
//! reading a sprite's source pixels and committing an edited result.
//!
//! Every Image Tool (Trim, Make Square, BG-Removal, and the many to come)
//! reads a sprite's texture, transforms the pixels, and uploads a new one.
//! The alpha REPRESENTATION (straight vs premultiplied) must survive that
//! round-trip: a bug where Trim / Make-Square un-premultiplied a
//! BG-Removal result and silently dropped the `premultiplied` flag
//! re-introduced the edge fringe. Routing every tool through
//! [`read_sprite_source`] (pixels are a [`SpriteImage`] that CARRIES its
//! [`AlphaMode`]) and [`commit_edited_texture`] (writes
//! `Sprite.premultiplied` FROM the result's `AlphaMode`, never a
//! hand-typed bool) makes that class of bug impossible.
//!
//! An arch gate (`tests/texture_edit_chokepoint.rs`) forbids
//! `readback_individual` anywhere else in the shell, so a future tool
//! cannot obtain individual-texture pixels without the mode-carrying
//! wrapper.

use std::collections::BTreeMap;

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_render::{AlphaMode, Sprite, SpriteImage, SpriteRenderer, SpriteSource};

/// A sprite's source pixels (native alpha mode) plus the pre-edit
/// transform metadata image tools need for undo + recenter.
pub(crate) struct SourceRead {
    pub image: SpriteImage,
    pub old_size_world: [f32; 2],
    pub old_translation: [f32; 2],
    pub old_source: SpriteSource,
    pub old_premultiplied: bool,
    /// `Sprite.anchor` (intrinsic-local pivot offset) before the edit.
    /// Padding's Keep mode rebases it so content + pivot stay world-
    /// fixed; undo restores it. `[0,0]` for any sprite the TOOL_PIVOT
    /// tool hasn't moved the pivot on.
    pub old_anchor: [f32; 2],
}

/// Read `entity`'s sprite source as a [`SpriteImage`] carrying its native
/// [`AlphaMode`] — Atlas resolves to the straight asset pixels; Individual
/// reads the texture back, premultiplied iff `Sprite.premultiplied`.
/// `None` if the sprite is missing, the atlas key/asset is absent, or the
/// readback fails.
///
/// THE single sanctioned `readback_individual` call site (arch-gated).
pub(crate) fn read_sprite_source(
    entity: Entity,
    sim: &SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
) -> Option<SourceRead> {
    let world = sim.world();
    let sprite = world.get::<Sprite>(entity)?;
    let old_size_world = sprite.size;
    let old_source = sprite.source;
    let old_premultiplied = sprite.premultiplied;
    let old_anchor = sprite.anchor;
    let old_translation = world
        .get::<Transform>(entity)
        .map(|t| [t.translation.x, t.translation.y])
        .unwrap_or([0.0, 0.0]);
    let image = match sprite.source {
        SpriteSource::Atlas { key } => {
            let aid = atlas_asset_map.get(&key)?;
            let asset = asset_db.get(aid)?;
            // ⚠️ `image_rgba8` e não um `match` na variante: uma sprite de Atlas é de 8 bits por
            // construção, e as Image Tools que consomem isto entregam 8 bits de volta. Casar a
            // variante fazia um asset de 16 bits abrir a ferramenta com a imagem VAZIA
            // (plano `docs/Sprite_projeto/18`, auditoria da W2).
            let (width, height, px) = asset.image_rgba8()?;
            SpriteImage::from_bytes(width, height, px.into_owned(), AlphaMode::Straight)
        }
        SpriteSource::Individual { texture_id } => {
            let (w, h, pixels) = renderer.readback_individual(texture_id).ok()?;
            let full = SpriteImage::from_bytes(
                w,
                h,
                pixels,
                AlphaMode::from_premultiplied_flag(old_premultiplied),
            );
            // ⚠️ **UM SPRITE DE REGIÃO NÃO É A TEXTURA INTEIRA** (Enio, 2026-08-19: a folha
            // exportada saiu *"com múltiplas repetições"*). Uma peça ligada a uma folha partilha a
            // textura com as vizinhas e usa só uma janela dela — ler a textura toda devolve a
            // folha INTEIRA como se fossem os pixels daquele sprite. No bake isso carimbava a
            // folha completa no lugar de cada peça, uma vez por peça; num Trim daria a folha
            // inteira recortada e re-atribuída a um sprite só.
            //
            // ⚠️ **O defeito é ANTIGO e estava latente:** esta função nasceu quando `Individual`
            // significava sempre "uma textura, um sprite", e o `region_enabled` chegou com o
            // import de folhas sem que ninguém revisitasse o leitor. Ele só ficou ALCANÇÁVEL
            // quando a folha passou a ser produzida aqui dentro. *Curar no ponto de
            // estrangulamento cobre as oito ferramentas de uma vez* — que é a razão de ele existir.
            crop_region(full, sprite)
        }
        // W2.T2: a tier-cooked KTX2 texture is GPU-compressed (BC/ASTC/
        // ETC2) with no CPU-side RGBA — the raster image tools (Trim,
        // Make Square, Bg Removal) can't read it back to edit. Unsupported.
        SpriteSource::CookedTexture { .. } => return None,
    };
    Some(SourceRead {
        image,
        old_size_world,
        old_translation,
        old_source,
        old_premultiplied,
        old_anchor,
    })
}

/// Recorta a imagem à janela que o sprite de facto usa (`region_rect`), ou devolve-a inteira.
///
/// ⚠️ **Recusa silenciosamente um retângulo impossível** devolvendo a imagem inteira: um rect que
/// sai da textura significa que alguém a re-uploadou mais pequena por baixo do sprite, e cortar
/// com aritmética que dá a volta produziria lixo. Devolver o todo é errado de forma VISÍVEL —
/// devolver lixo é errado de forma silenciosa.
fn crop_region(img: SpriteImage, sprite: &Sprite) -> SpriteImage {
    if !sprite.region_enabled {
        return img;
    }
    let [rx, ry, rw, rh] = sprite.region_rect;
    // Os quatro vêm de um `[f32; 4]` que carrega pixels inteiros; qualquer coisa que não seja um
    // pixel inteiro positivo não é uma janela.
    let (x, y, w, h) = (
        rx.round().max(0.0) as u64,
        ry.round().max(0.0) as u64,
        rw.round().max(0.0) as u64,
        rh.round().max(0.0) as u64,
    );
    if w == 0 || h == 0 || x + w > u64::from(img.width) || y + h > u64::from(img.height) {
        return img;
    }
    let (x, y, w, h) = (x as usize, y as usize, w as usize, h as usize);
    let src_row = img.width as usize * 4;
    let mut out = vec![0u8; w * h * 4];
    for r in 0..h {
        let from = (y + r) * src_row + x * 4;
        let to = r * w * 4;
        out[to..to + w * 4].copy_from_slice(&img.pixels[from..from + w * 4]);
    }
    SpriteImage::from_bytes(w as u32, h as u32, out, img.alpha)
}

/// Upload `img` as a fresh Individual texture and repoint `entity`'s
/// Sprite at it: `source` + `new_size_world` + `premultiplied` derived
/// from `img.alpha`. Returns the new texture id.
///
/// THE single place an image-tool result writes `Sprite.premultiplied`
/// — derived from the bytes' actual [`AlphaMode`], so the flag can never
/// drift from the representation (the original Trim/Make-Square bug).
///
/// ## It is also where the pixels get a DURABLE NAME
///
/// `texture_id` is a GPU allocation id and the store restarts numbering at `1` every process, so
/// a sprite that only carried it came back **invisible** — or showing another sprite's pixels —
/// after save/load. The same bytes therefore also go into the [`AssetDb`] (blake3 content hash,
/// HR-6) and the entity is stamped with [`ph2d_ecs::SpritePixels`], which travels in the
/// `WorldSnapshot` and is what `project_sprite_pixels` writes to (and restores from) the file.
///
/// Doing it HERE is the whole point: an arch gate pins this function as the single door every
/// image tool leaves through, so eight tools are covered by one stamp rather than eight.
pub(crate) fn commit_edited_texture(
    entity: Entity,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    img: &SpriteImage,
    new_size_world: [f32; 2],
    toasts: &mut ph2d_editor::ToastQueue,
) -> Result<u32, String> {
    // **A PERDA DE PRECISÃO tem de ser dita** (plano `docs/Sprite_projeto/18` W4).
    //
    // ⚠️ Toda ferramenta de imagem trabalha em 8 bits — o `SpriteImage` é `Vec<u8>` — e este funil
    // é por onde todas escrevem de volta. Sem este aviso, correr um Trim numa sprite de 16 bits
    // **rebaixa-a em silêncio**: sem erro, sem log, e o único vestígio é a linha `Format` do
    // Inspector, que o artista não estava a olhar.
    //
    // ⛔ **A alternativa recusada** era converter a ferramenta para 16 bits: nenhuma delas pede
    // isso hoje, e emendar o contrato congelado `RasterEditTool` por simetria — em vez de por
    // necessidade medida — é como se paga um contrato duas vezes.
    //
    // O aviso é DEPOIS e não antes de propósito: antes exigiria interceptar a activação de cada
    // ferramenta (nove sítios) para dizer o que este único sítio sabe de facto.
    if let SpriteSource::Individual { texture_id } = sim
        .world()
        .get::<Sprite>(entity)
        .map(|s| s.source)
        .unwrap_or(SpriteSource::Atlas { key: 0 })
        && renderer.individual_format(texture_id)
            == Some(ph2d_render::IndividualTextureStore::FORMAT_16)
    {
        toasts.push(ph2d_editor::Toast::info(
            "Converted to RGBA8 — image tools work in 8-bit",
        ));
    }
    let texture_id = renderer
        .acquire_individual(img.width, img.height, &img.pixels)
        .map_err(|e| e.to_string())?;
    // The durable name of these exact bytes. `insert_image_rgba8` takes `&self` (interior
    // mutability) and hashes dims + content, so re-committing an identical edit is idempotent
    // and two sprites that end up with the same pixels share one entry in the project file.
    let pixels_id = asset_db.insert_image_rgba8(img.width, img.height, img.pixels.clone());
    rebind_to_individual(
        entity,
        sim,
        texture_id,
        pixels_id,
        new_size_world,
        img.alpha.is_premultiplied(),
    );
    Ok(texture_id)
}

/// **A cauda do [`commit_edited_texture`] — as cinco invariantes de "esta sprite passou a ter
/// pixels próprios", num sítio só.**
///
/// ⚠️ Extraída em 2026-08-20 (plano `docs/Sprite_projeto/18` W5) porque a conversão de precisão
/// precisa **exatamente** delas e escrevê-las outra vez seria pedir que as duas cópias
/// concordassem para sempre. Duas delas só falham **depois de fechar e reabrir o projeto**, que é o
/// pior sítio para descobrir uma divergência.
pub(crate) fn rebind_to_individual(
    entity: Entity,
    sim: &mut SimWorld,
    texture_id: u32,
    pixels_id: ph2d_asset::AssetId,
    new_size_world: [f32; 2],
    premultiplied: bool,
) {
    if let Some(mut sprite) = sim.world_mut().get_mut::<Sprite>(entity) {
        sprite.source = SpriteSource::Individual { texture_id };
        sprite.size = new_size_world;
        sprite.premultiplied = premultiplied;
        // ⚠️ **A JANELA MORRE COM A EDIÇÃO, e é a outra metade do bug das repetições.** Se o
        // sprite era uma região de uma folha, a textura que acabou de subir é a imagem INTEIRA
        // dele — amostrá-la pela janela antiga mostraria um recorte arbitrário dessa imagem nova.
        // O `region_rect` fica como está de propósito (é ignorado enquanto `region_enabled` é
        // falso, e zerá-lo só acrescentaria uma escrita que ninguém lê).
        sprite.region_enabled = false;
    }
    // Stamped AFTER the sprite write and unconditionally: an entity whose `Sprite` vanished
    // mid-frame gets no stamp because `insert` on a dead entity is the caller's bug, so guard on
    // the same lookup the write used.
    if sim.world().get::<Sprite>(entity).is_some() {
        sim.world_mut()
            .entity_mut(entity)
            .insert(ph2d_ecs::SpritePixels(pixels_id));
        // ⚠️ **E a AUTORIA morre com ela.** `SpriteSheetRef` diz *"os meus pixels são a região R da
        // folha F"*, e isso deixou de ser verdade: os pixels agora são próprios, com nome durável
        // (o `SpritePixels` acima). Deixá-lo faria o `restore_sprite_sheets` re-ligar o sprite à
        // folha no load seguinte e **apagar a edição** — o defeito só apareceria depois de fechar
        // e reabrir o projeto, que é o pior sítio para o descobrir.
        sim.world_mut()
            .entity_mut(entity)
            .remove::<ph2d_ecs::SpriteSheetRef>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Uma imagem `w × h` em que cada pixel carrega o próprio `(x, y)` nos dois primeiros canais —
    /// assim um recorte errado é legível: o pixel diz de onde veio.
    fn tagged(w: u32, h: u32) -> SpriteImage {
        let mut px = Vec::with_capacity((w * h * 4) as usize);
        for y in 0..h {
            for x in 0..w {
                px.extend_from_slice(&[x as u8, y as u8, 0, 255]);
            }
        }
        SpriteImage::from_bytes(w, h, px, AlphaMode::Straight)
    }

    fn region_sprite(rect: [f32; 4]) -> Sprite {
        let mut s = Sprite::individual(1, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]);
        s.region_enabled = true;
        s.region_rect = rect;
        s
    }

    /// ⚠️ **O bug que o Enio viu na folha exportada** (*"com múltiplas repetições"*): sem o
    /// recorte, um sprite de região devolvia a TEXTURA INTEIRA — a folha toda —, e o bake
    /// carimbava-a no lugar de cada peça, uma vez por peça.
    #[test]
    fn a_region_sprite_reads_only_its_window() {
        let img = crop_region(tagged(16, 16), &region_sprite([4.0, 8.0, 3.0, 2.0]));
        assert_eq!((img.width, img.height), (3, 2));
        // O primeiro pixel do recorte tem de ser o `(4, 8)` da origem.
        assert_eq!(&img.pixels[0..2], &[4, 8]);
        // E o último, o `(6, 9)`.
        let last = img.pixels.len() - 4;
        assert_eq!(&img.pixels[last..last + 2], &[6, 9]);
    }

    /// Controle positivo: sem janela, a imagem passa inteira. Sem isto o teste acima passaria com
    /// um `crop_region` que devolvesse sempre um recorte fixo.
    #[test]
    fn a_plain_sprite_reads_whole() {
        let mut s = Sprite::individual(1, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]);
        s.region_enabled = false;
        s.region_rect = [4.0, 8.0, 3.0, 2.0];
        let img = crop_region(tagged(16, 16), &s);
        assert_eq!((img.width, img.height), (16, 16));
    }

    /// ⚠️ Um retângulo que sai da textura devolve o TODO, não lixo: significa que alguém
    /// re-uploadou a textura mais pequena por baixo do sprite, e é um erro que tem de ser visível.
    #[test]
    fn an_impossible_window_falls_back_to_the_whole_image() {
        for rect in [
            [14.0, 0.0, 4.0, 4.0], // sai pela direita
            [0.0, 14.0, 4.0, 4.0], // sai por baixo
            [0.0, 0.0, 0.0, 4.0],  // largura zero
            [-1.0, 0.0, 4.0, 4.0], // canto negativo (o `max(0)` traz para 0, mas fica dentro)
        ] {
            let img = crop_region(tagged(16, 16), &region_sprite(rect));
            let cropped = (img.width, img.height) != (16, 16);
            // O canto negativo é o único que PODE recortar (vira 0,0 4x4) — os outros três não.
            if rect[0] >= 0.0 {
                assert!(
                    !cropped,
                    "rect {rect:?} devia ter devolvido a imagem inteira"
                );
            }
        }
    }

    /// ⚠️ **O RECORTE ESTÁ LIGADO** — e este gate existe porque os quatro testes acima **não o
    /// provam**: eles chamam `crop_region` diretamente, e uma prova de mutação mostrou-os todos
    /// verdes com a chamada removida do `read_sprite_source`. Unidade verde, costura morta — a
    /// forma canónica do apodrecimento neste repositório.
    ///
    /// `read_sprite_source` precisa de uma GPU (faz `readback_individual`), então a costura não
    /// tem teste de comportamento possível aqui; o que se pode afirmar é que o braço `Individual`
    /// **passa pelo recorte**. É o mesmo instrumento que o gate do pivô de joint usa, e pela mesma
    /// razão: quando o comportamento é inalcançável, a estrutura é a afirmação que resta.
    #[test]
    fn the_individual_branch_goes_through_the_crop() {
        let src = include_str!("texture_edit.rs");
        let arm = src
            .find("SpriteSource::Individual { texture_id } =>")
            .expect("o braco `Individual` do `read_sprite_source` mudou de forma");
        // ⚠️ A fronteira e' o braco SEGUINTE, nao uma janela de N bytes. A 1a versao deste gate
        // usava `arm + 1200` e reprovou sobre codigo correto assim que o comentario do braco
        // cresceu (a distancia real era 1362). *Um numero magico num gate apodrece na primeira
        // edicao do que ele mede.*
        let end = src[arm..]
            .find("SpriteSource::CookedTexture")
            .map_or(src.len(), |o| arm + o);
        let body = &src[arm..end];
        assert!(
            body.contains("crop_region("),
            "o braco `Individual` deixou de recortar a regiao: um sprite ligado a uma folha volta \n\
             a devolver a TEXTURA INTEIRA, e o bake carimba a folha toda no lugar de cada peca \n\
             (Enio, 2026-08-19: a folha exportada saiu «com multiplas repeticoes»)"
        );
    }

    /// A janela preserva o MODO de alfa — recortar não é converter.
    #[test]
    fn the_crop_keeps_the_alpha_mode() {
        let mut img = tagged(8, 8);
        img.alpha = AlphaMode::Premultiplied;
        let out = crop_region(img, &region_sprite([1.0, 1.0, 2.0, 2.0]));
        assert_eq!(out.alpha, AlphaMode::Premultiplied);
    }
}

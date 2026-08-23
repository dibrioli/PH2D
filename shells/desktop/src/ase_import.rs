//! **O IMPORT de um `.ase`/`.aseprite`** — largar o ficheiro NATIVO do Aseprite na janela.
//!
//! Enio, 2026-08-23: *«Precisamos Importar Aseprite (.ase)»*. Irmão de
//! [`crate::sheet_import`] (o par `.png` + `.json`, que já existia) e de
//! [`crate::image_import`] (uma imagem = um sprite). ⚠️ **O corte é o que cada porta SABE**: o par
//! exportado traz rectângulos com nome; o `.ase` traz a **autoria** — camadas, tags e a duração de
//! cada quadro. É por isso que este caminho produz **UMA** sprite com grelha e uma biblioteca de
//! animações (o modelo da §11), e não N sprites soltas como o hand-packed.
//!
//! ## A leitura é da crate-folha; aqui mora a COSTURA
//!
//! [`ph2d_aseprite::parse`] é pura (bytes → quadros + tags). O que este módulo faz é o que ela
//! deliberadamente não sabe: arrumar os quadros numa folha, subir a textura, e traduzir as tags
//! para [`ph2d_ecs::AnimationTag`]. As três decisões que isso pede estão em funções **puras**
//! ([`grid_for`], [`pack`], [`tag_from_ase`]), porque a costura que precisa de GPU não é alcançável
//! de um teste.
//!
//! ## ⚠️ A ORDEM DOS QUADROS É O CONTRATO
//!
//! Uma `AnimationTag` é um intervalo `[from, to]` sobre as **células** da grelha, e uma tag do
//! Aseprite é um intervalo sobre os **quadros** do ficheiro. Os dois só coincidem enquanto a folha
//! for empacotada **em linha, da esquerda para a direita e de cima para baixo** — que é a ordem que
//! o extract usa para converter uma célula em sub-UV. Empacotar por colunas daria uma folha bonita
//! e animações trocadas.
//!
//! ## ⛔ A duração por-quadro: a recusa que este ficheiro reabre
//!
//! A §11 guarda **um** `frame_ms` por tag, e a recusa de então (spec §8.12) dizia *«não há quem
//! produza durações por-quadro»*. Há: é este importador. O que se faz **hoje** é aproximar pela
//! duração mais comum da tag **e dizê-lo ao artista**, nomeando a tag. ⚠️ *Aproximar em silêncio
//! seria a resposta errada com a certeza da resposta certa* — o artista veria o *hold* de
//! antecipação dele desaparecer sem uma linha a explicar. A decisão de pôr a duração por-quadro no
//! modelo é de produto, e move o `PROJECT_SCHEMA`.

use ph2d_aseprite::{AseDoc, AseFrame, AseTag};
use ph2d_ecs::{AnimDirection, AnimationTag, SimWorld, SpriteAnimations, SpriteAnimator};
use ph2d_render::SpriteRenderer;
use std::path::Path;

/// O lado máximo de uma folha, em pixels. ⚠️ **É o `max_texture_dimension_2d` do default do wgpu**
/// — o recurso é a GPU, não o gosto: uma tira de 300 quadros de 128 px teria 38 400 px de largura e
/// a subida falharia com uma mensagem que não fala de animação nenhuma.
const MAX_SHEET_EDGE_PX: u32 = 8192;

/// As duas extensões que o Aseprite escreve. ⚠️ **Uma LISTA, e o predicado abaixo é derivado dela**
/// — quem constrói o diálogo de ficheiro precisa de as ENUMERAR, e um predicado não se enumera.
/// Foi exactamente essa duplicação que deixou o `.ase` invisível no «Import…» (Enio, 2026-08-23).
///
/// ⚠️ **`.ase` é um nome DISPUTADO:** a Adobe usa-o para *Swatch Exchange*, uma paleta — e este app
/// já lê esse formato noutro sítio ([`crate::forwarding`], o import de paletas). São dois ficheiros
/// diferentes com a mesma extensão, e o que os separa é a **porta**: uma paleta entra pelo botão de
/// paletas, um sprite entra pelo canvas. Quem largar uma paleta aqui recebe *«not an Aseprite file
/// (bad magic number)»*, que é a mensagem certa.
pub(crate) const ASE_EXTENSIONS: &[&str] = &["ase", "aseprite"];

/// Este caminho reconhece o ficheiro?
#[must_use]
pub(crate) fn is_ase_file(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|e| ASE_EXTENSIONS.iter().any(|x| e.eq_ignore_ascii_case(x)))
}

/// **Como os N quadros se arrumam.** Devolve `(colunas, linhas)`.
///
/// ⚠️ **Uma tira, sempre que ela couber.** É o que o artista espera ver, é o que o Aseprite exporta
/// por omissão, e faz o `hframes` do inspector ser legível (`8` em vez de `3 × 3`). Só quando a
/// tira não cabe na textura é que a folha vira quase-quadrada — e o limite é de MEMÓRIA de GPU, com
/// o número ao lado.
///
/// Um quadro maior que o teto sozinho dá uma coluna: a folha vai falhar na subida, e falhar com o
/// tamanho certo é melhor que falhar com um tamanho inventado.
#[must_use]
pub(crate) fn grid_for(frames: usize, frame_w: u16, frame_h: u16) -> (u32, u32) {
    let n = frames.max(1) as u32;
    let (w, h) = (u32::from(frame_w).max(1), u32::from(frame_h).max(1));
    let fit_w = (MAX_SHEET_EDGE_PX / w).max(1);
    if n <= fit_w {
        return (n, 1);
    }
    // Quase-quadrada, mas nunca mais larga do que cabe — e nunca mais alta do que cabe.
    let cols = (n as f64).sqrt().ceil() as u32;
    let cols = cols.clamp(1, fit_w);
    let rows = n.div_ceil(cols);
    let fit_h = (MAX_SHEET_EDGE_PX / h).max(1);
    if rows <= fit_h {
        return (cols, rows);
    }
    // Não cabe de todo: devolve o que cabe, e quem chama recusa com o número.
    (cols, rows)
}

/// **Empacota os quadros na folha**, em linha, da esquerda para a direita e de cima para baixo.
///
/// Células sobrando na última linha ficam transparentes — a grelha é rectangular e o número de
/// quadros raramente é.
#[must_use]
pub(crate) fn pack(frames: &[AseFrame], fw: u16, fh: u16, cols: u32, rows: u32) -> Vec<u8> {
    let (fw, fh) = (usize::from(fw), usize::from(fh));
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
            let Some(line) = frame.rgba.get(src..src + fw * 4) else {
                break;
            };
            let dst = ((cy * fh + row) * sheet_w + cx * fw) * 4;
            out[dst..dst + fw * 4].copy_from_slice(line);
        }
    }
    out
}

/// **Uma tag do Aseprite vira uma da §11.** Devolve também a linha a dizer ao artista quando a
/// conversão perdeu alguma coisa.
///
/// ⚠️ **A direcção passa pelo construtor, e não pelo número.** As duas ordens coincidem hoje
/// (`Forward`/`Reverse`/`PingPong`/`PingPongReverse`), e essa coincidência é do formato — escrever
/// `direction: unsafe_cast(t.direction)` faria uma reordenação futura de um dos dois lados trocar
/// animações em silêncio.
#[must_use]
pub(crate) fn tag_from_ase(t: &AseTag, frames: &[AseFrame]) -> (AnimationTag, Option<String>) {
    let uniform = t.uniform_duration_ms(frames);
    let ms = uniform.unwrap_or_else(|| t.dominant_duration_ms(frames));
    let note = uniform.is_none().then(|| {
        format!(
            "\"{}\" has per-frame durations; every frame now lasts {ms} ms",
            t.name
        )
    });
    (
        AnimationTag {
            frame_ms: u32::from(ms).max(1),
            direction: AnimDirection::from_tag(t.direction),
            // Aseprite: `0` = para sempre. A §11 diz o mesmo com `None`.
            repeat: (t.repeat > 0).then(|| u32::from(t.repeat)),
            ..AnimationTag::new(t.name.clone(), u32::from(t.from), u32::from(t.to))
        },
        note,
    )
}

/// **A biblioteca inteira**, com as notas que a conversão produziu.
///
/// ⚠️ **Um ficheiro SEM tags recebe uma**, que cobre todos os quadros e leva o nome do ficheiro:
/// sem ela a sprite nasce com uma grelha e nada para tocar, e a §11 fica muda sobre um ficheiro que
/// é uma animação. É o que o próprio Aseprite faz quando exporta um ficheiro sem tags.
#[must_use]
pub(crate) fn library(doc: &AseDoc, stem: &str) -> (SpriteAnimations, Vec<String>) {
    let mut lib = SpriteAnimations::new();
    let mut notes = Vec::new();
    let last = doc.frames.len().saturating_sub(1) as u32;
    if doc.tags.is_empty() {
        let mut all = AnimationTag::new(stem, 0, last);
        all.frame_ms = doc
            .frames
            .first()
            .map_or(100, |f| u32::from(f.duration_ms))
            .max(1);
        let _ = lib.insert(all);
        notes.push(format!(
            "the file has no tags — one animation named \"{stem}\" covers all {} frames",
            doc.frames.len()
        ));
        return (lib, notes);
    }
    for t in &doc.tags {
        let (tag, note) = tag_from_ase(t, &doc.frames);
        let name = tag.name.clone();
        match lib.insert(tag) {
            Ok(()) => notes.extend(note),
            // ⚠️ NOMEIA a tag e o motivo: «alguma coisa não entrou» manda o artista adivinhar
            // entre um nome repetido, um nome vazio e uma lista cheia — três consertos diferentes.
            Err(e) => notes.push(format!("tag \"{name}\" was not imported ({e:?})")),
        }
    }
    (lib, notes)
}

/// O que aconteceu a um `.ase` largado.
pub(crate) enum AseImportResult {
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

/// Importa um `.ase`: lê, empacota numa folha, sobe UMA textura, e nasce **uma** sprite com a
/// grelha e a biblioteca de animações do ficheiro, a tocar a primeira.
#[allow(clippy::too_many_arguments)]
pub(crate) fn import_ase(
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
        Err(e) => return fail(format!("read: {e}")),
    };
    let doc = match ph2d_aseprite::parse(&bytes) {
        Ok(d) => d,
        Err(e) => return fail(e.to_string()),
    };
    let (cols, rows) = grid_for(doc.frames.len(), doc.width, doc.height);
    let sheet_w = cols * u32::from(doc.width);
    let sheet_h = rows * u32::from(doc.height);
    if sheet_w > MAX_SHEET_EDGE_PX || sheet_h > MAX_SHEET_EDGE_PX {
        // ⚠️ A mensagem traz OS DOIS números: o que a folha precisaria e o que a placa aceita.
        return fail(format!(
            "{} frames of {}x{} need a {sheet_w}x{sheet_h} sheet, and the limit is {MAX_SHEET_EDGE_PX}",
            doc.frames.len(),
            doc.width,
            doc.height
        ));
    }
    let pixels = pack(&doc.frames, doc.width, doc.height, cols, rows);
    let cell = *next_cell;
    let (_, bits) = match crate::image_import::spawn_rgba(
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
    // **A GRELHA é o pool da §11.** E o tamanho no mundo é o de UMA célula — o `spawn_rgba` deu à
    // sprite o tamanho da folha inteira, que é o que ela mostraria sem grelha.
    if let Some(mut s) = sim.world_mut().get_mut::<ph2d_render::Sprite>(entity) {
        s.hframes = cols;
        s.vframes = rows;
        s.frame = 0;
        s.size = [s.size[0] / cols as f32, s.size[1] / rows as f32];
    }
    let (lib, mut notes) = library(&doc, &name);
    notes.splice(0..0, doc.notes.iter().cloned());
    let animations = lib.iter().count();
    // Toca a primeira, e fica a tocar ao abrir o projeto: um ficheiro de animação largado que
    // aparece parado lê-se como um import que falhou.
    let first = lib
        .iter()
        .next()
        .map(|t| t.name.clone())
        .unwrap_or_default();
    let mut player = SpriteAnimator::new(first);
    player.playing = true;
    player.autoplay = true;
    if let Ok(mut ent) = sim.world_mut().get_entity_mut(entity) {
        ent.insert((lib, player));
    }
    AseImportResult::Ok {
        name,
        frames: doc.frames.len(),
        animations,
        bits,
        notes,
    }
}

#[cfg(test)]
#[path = "ase_import_tests.rs"]
mod tests;

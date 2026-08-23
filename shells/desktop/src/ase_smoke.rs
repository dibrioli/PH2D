//! `PH2D_ASE_SMOKE` — **o import de um `.ase` a acontecer**, sem precisar do Aseprite instalado.
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Sprite \
//!   && env PH2D_ASE_SMOKE=1 cargo run -p ph2d-host-desktop --release
//! ```
//!
//! ⚠️ **Ele ESCREVE um `.ase` de verdade** (numa pasta temporária) e depois larga-o pela porta do
//! produto — o mesmo [`crate::ase_import::import_ase`] que o drag & drop chama. Não há caminho
//! paralelo: se o smoke funciona, largar um ficheiro do artista funciona.
//!
//! O ficheiro que ele escreve foi desenhado para exercer o que costuma partir:
//!
//! * **duas camadas**, uma delas em `Multiply` — o modo tem de chegar aos pixels;
//! * **um cel LIGADO** — o quadro 5 repete o 4, que é como o Aseprite guarda o que não foi
//!   redesenhado (tratá-lo como vazio faz a animação piscar);
//! * **duas tags** com direcções diferentes, uma delas com o **HOLD** de um quadro mais longo — é
//!   a que produz o aviso da duração aproximada;
//! * **um cel comprimido**, que é a forma que todo ficheiro real usa.
//!
//! ⛔ **Ele não é um oráculo do Aseprite.** Ele prova que sabemos ler o que descrevemos; a
//! fidelidade ao programa real vem de largar um ficheiro do artista.

use std::path::PathBuf;

const CELLS: u32 = 8;
const PX: u16 = 24;

pub(crate) fn enabled() -> bool {
    std::env::var_os("PH2D_ASE_SMOKE").is_some()
}

fn w16(v: &mut Vec<u8>, x: u16) {
    v.extend_from_slice(&x.to_le_bytes());
}
fn w32(v: &mut Vec<u8>, x: u32) {
    v.extend_from_slice(&x.to_le_bytes());
}
fn wstr(v: &mut Vec<u8>, s: &str) {
    w16(v, s.len() as u16);
    v.extend_from_slice(s.as_bytes());
}

/// Um chunk: `DWORD` de tamanho (que inclui estes 6 bytes) + `WORD` de tipo + corpo.
fn chunk(out: &mut Vec<u8>, kind: u16, body: &[u8]) {
    w32(out, (body.len() + 6) as u32);
    w16(out, kind);
    out.extend_from_slice(body);
}

fn layer_chunk(name: &str, blend: u16, opacity: u8) -> Vec<u8> {
    let mut b = Vec::new();
    w16(&mut b, 1); // flags: visível
    w16(&mut b, 0); // camada de imagem
    w16(&mut b, 0); // nível de aninhamento
    w16(&mut b, 0);
    w16(&mut b, 0);
    w16(&mut b, blend);
    b.push(opacity);
    b.extend_from_slice(&[0; 3]);
    wstr(&mut b, name);
    b
}

fn cel_chunk(layer: u16, x: i16, y: i16, w: u16, h: u16, rgba: &[u8]) -> Vec<u8> {
    let mut b = Vec::new();
    w16(&mut b, layer);
    w16(&mut b, x as u16);
    w16(&mut b, y as u16);
    b.push(255);
    w16(&mut b, 2); // imagem comprimida — a forma que todo ficheiro real usa
    b.extend_from_slice(&[0; 7]);
    w16(&mut b, w);
    w16(&mut b, h);
    b.extend_from_slice(&miniz_oxide::deflate::compress_to_vec_zlib(rgba, 6));
    b
}

fn link_chunk(layer: u16, src_frame: u16) -> Vec<u8> {
    let mut b = Vec::new();
    w16(&mut b, layer);
    w16(&mut b, 0);
    w16(&mut b, 0);
    b.push(255);
    w16(&mut b, 1); // cel LIGADO
    b.extend_from_slice(&[0; 7]);
    w16(&mut b, src_frame);
    b
}

fn tags_chunk(list: &[(&str, u16, u16, u8, u16)]) -> Vec<u8> {
    let mut b = Vec::new();
    w16(&mut b, list.len() as u16);
    b.extend_from_slice(&[0; 8]);
    for (name, from, to, dir, repeat) in list {
        w16(&mut b, *from);
        w16(&mut b, *to);
        b.push(*dir);
        w16(&mut b, *repeat);
        b.extend_from_slice(&[0; 6]);
        b.extend_from_slice(&[0; 3]);
        b.push(0);
        wstr(&mut b, name);
    }
    b
}

/// Matiz em `[0, 1]` → RGB saturado. LITERAL-COLOR-OK: conteúdo de CENA, não chrome.
fn hue(t: f32) -> [u8; 3] {
    let h = (t * 6.0).clamp(0.0, 5.999);
    let f = h - h.floor();
    let q = ((1.0 - f) * 255.0) as u8;
    let p = (f * 255.0) as u8;
    match h as u32 {
        0 => [255, p, 0],
        1 => [q, 255, 0],
        2 => [0, 255, p],
        3 => [0, q, 255],
        4 => [p, 0, 255],
        _ => [255, 0, q],
    }
}

/// A camada de baixo do quadro `i`: um quadrado colorido que sobe.
fn base_cel(i: u32) -> Vec<u8> {
    let n = usize::from(PX);
    let mut px = vec![0_u8; n * n * 4];
    let c = hue(i as f32 / CELLS as f32);
    let lift = (i * (PX as u32 / 2) / CELLS) as usize;
    for y in (n / 3)..(2 * n / 3) {
        for x in (n / 4)..(3 * n / 4) {
            let yy = y.saturating_sub(lift);
            let d = (yy * n + x) * 4;
            px[d..d + 4].copy_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
    px
}

/// A camada de cima: uma faixa cinzenta que, em `Multiply`, escurece metade do quadrado.
fn shade_cel() -> Vec<u8> {
    let n = usize::from(PX);
    let mut px = vec![255_u8; n * n * 4];
    for y in 0..n {
        for x in 0..n {
            let v = if x < n / 2 { 120 } else { 255 };
            let d = (y * n + x) * 4;
            px[d..d + 4].copy_from_slice(&[v, v, v, 255]);
        }
    }
    px
}

/// Escreve o ficheiro e devolve o caminho.
fn write_demo() -> std::io::Result<PathBuf> {
    let mut out = Vec::new();
    w32(&mut out, 0);
    w16(&mut out, 0xA5E0);
    w16(&mut out, CELLS as u16);
    w16(&mut out, PX);
    w16(&mut out, PX);
    w16(&mut out, 32); // RGBA
    w32(&mut out, 0);
    w16(&mut out, 100);
    w32(&mut out, 0);
    w32(&mut out, 0);
    out.push(0);
    out.extend_from_slice(&[0; 3]);
    w16(&mut out, 0);
    out.resize(128, 0);

    let shade = shade_cel();
    for i in 0..CELLS {
        let mut body = Vec::new();
        let mut n_chunks = 0_u32;
        if i == 0 {
            chunk(&mut body, 0x2004, &layer_chunk("art", 0, 255));
            chunk(&mut body, 0x2004, &layer_chunk("shade", 1, 255)); // 1 = Multiply
            chunk(
                &mut body,
                0x2018,
                &tags_chunk(&[
                    // A `walk` percorre a tira toda e repete para sempre.
                    ("walk", 0, 7, 0, 0),
                    // A `idle` vai-e-volta e tem um HOLD no quadro 2 — é ela que produz o aviso.
                    ("idle", 0, 3, 2, 0),
                ]),
            );
            n_chunks += 3;
        }
        // ⚠️ O quadro 5 NÃO é redesenhado: ele LIGA-SE ao 4. É assim que o Aseprite guarda o que
        // ficou igual, e é o modo de falha nº 1 de um leitor ingénuo.
        if i == 5 {
            chunk(&mut body, 0x2005, &link_chunk(0, 4));
        } else {
            chunk(&mut body, 0x2005, &cel_chunk(0, 0, 0, PX, PX, &base_cel(i)));
        }
        chunk(&mut body, 0x2005, &cel_chunk(1, 0, 0, PX, PX, &shade));
        n_chunks += 2;

        // O quadro 2 dura muito mais: o *hold* de antecipação da `idle`.
        let duration = if i == 2 { 400 } else { 90 };
        w32(&mut out, (body.len() + 16) as u32);
        w16(&mut out, 0xF1FA);
        w16(&mut out, 0);
        w16(&mut out, duration);
        out.extend_from_slice(&[0; 2]);
        w32(&mut out, n_chunks);
        out.extend_from_slice(&body);
    }
    let path = std::env::temp_dir().join("ph2d-smoke-hero.ase");
    std::fs::write(&path, out)?;
    Ok(path)
}

/// Escreve o `.ase` e importa-o **pela porta do produto**. Devolve os bits da sprite e as linhas a
/// mostrar.
pub(crate) fn spawn_if_enabled(
    sim: &mut ph2d_ecs::SimWorld,
    renderer: &mut ph2d_render::SpriteRenderer,
    asset_db: &ph2d_asset::AssetDb,
    next_cell: &mut u32,
    atlas_asset_map: &mut std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
    pixels_per_meter: f32,
) -> Option<(u64, Vec<String>)> {
    let path = match write_demo() {
        Ok(p) => p,
        Err(e) => return Some((0, vec![format!("could not write the demo file: {e}")])),
    };
    match crate::ase_import::import_ase(
        sim,
        renderer,
        asset_db,
        next_cell,
        atlas_asset_map,
        &path,
        [0.0, 0.0],
        pixels_per_meter,
    ) {
        crate::ase_import::AseImportResult::Ok {
            name,
            frames,
            animations,
            bits,
            notes,
        } => {
            let mut lines = vec![format!(
                "{name}: {frames} frames, {animations} animations — from {}",
                path.display()
            )];
            lines.extend(notes);
            Some((bits, lines))
        }
        crate::ase_import::AseImportResult::Err { name, error } => {
            Some((0, vec![format!("{name}: {error}")]))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O ficheiro que este smoke escreve LÊ-SE**, e traz o que ele promete.
    ///
    /// ⚠️ Este gate é o que impede o smoke de ser uma promessa: ele corre o escritor E o leitor,
    /// então um desalinhamento de um byte no cabeçalho reprova aqui, e não com o Enio à frente.
    #[test]
    fn the_demo_file_reads_back_with_everything_it_promises() {
        let mut bytes = Vec::new();
        // Reconstrói pelo mesmo caminho do produto, mas sem tocar no disco.
        let path = write_demo().expect("escrever o demo");
        bytes.extend_from_slice(&std::fs::read(&path).expect("ler o demo"));
        let doc = ph2d_aseprite::parse(&bytes).expect("o demo tem de ler");
        assert_eq!(doc.frames.len(), CELLS as usize);
        assert_eq!((doc.width, doc.height), (PX, PX));
        assert_eq!(doc.tags.len(), 2);
        assert_eq!(doc.tags[1].direction, 2, "a `idle` vai-e-volta");
        assert!(doc.notes.is_empty(), "nada ficou por tras: {:?}", doc.notes);

        // O cel LIGADO: o quadro 5 tem de ser igual ao 4, e DIFERENTE do 6.
        assert_eq!(
            doc.frames[5].rgba, doc.frames[4].rgba,
            "o quadro ligado tinha de repetir o 4"
        );
        assert_ne!(doc.frames[5].rgba, doc.frames[6].rgba);

        // O HOLD: o quadro 2 dura mais, e é isso que faz a `idle` avisar.
        assert_eq!(doc.frames[2].duration_ms, 400);
        assert_eq!(doc.tags[1].uniform_duration_ms(&doc.frames), None);

        // A camada `Multiply`: a metade esquerda tem de sair MAIS ESCURA que a direita.
        let row = usize::from(PX) / 2;
        let at = |x: usize| {
            let i = (row * usize::from(PX) + x) * 4;
            [
                doc.frames[0].rgba[i],
                doc.frames[0].rgba[i + 1],
                doc.frames[0].rgba[i + 2],
            ]
        };
        let (left, right) = (at(usize::from(PX) / 3), at(2 * usize::from(PX) / 3));
        let sum = |c: [u8; 3]| u32::from(c[0]) + u32::from(c[1]) + u32::from(c[2]);
        assert!(
            sum(left) < sum(right),
            "a camada Multiply nao escureceu a metade esquerda: {left:?} vs {right:?}"
        );
        let _ = std::fs::remove_file(path);
    }
}

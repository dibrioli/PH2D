//! O PNG à mão: só os chunks que o preset usa (PNG 1.2 / ISO 15948).
//!
//! Escrita: `IHDR` · `tEXt ph2d-origin` · `iTXt ph2d-brush` (comprimido) · `phTx`* · `IDAT` · `IEND`.
//! `iTXt` e não `zTXt`: o `zTXt` é Latin-1 por norma e os nomes são UTF-8. `phTx` é um chunk
//! privado, auxiliar e seguro de copiar (minúsculas nas posições 1, 2 e 4): um editor que não o
//! conhece pode deixá-lo ficar.

use crate::modelo::{Erro, Miniatura, Origem, Parametros, Preset, SCHEMA};
use std::collections::BTreeMap;

const ASSINATURA: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
const CHAVE_PARAMETROS: &str = "ph2d-brush";
const CHAVE_ORIGEM: &str = "ph2d-origin";
const CHUNK_TEXTURA: &[u8; 4] = b"phTx";
/// Nível do deflate. O maior: a escrita é rara e o ficheiro vive no repositório.
const NIVEL_ZLIB: u8 = 9;
/// Tecto do JSON descomprimido. Recurso: memória ao ler um ficheiro alheio (um preset tem ~5 KB;
/// isto recusa uma bomba de zlib sem recusar nada legítimo).
const MAX_TEXTO: usize = 16 << 20;
/// Tecto do lado da imagem lida. Recurso: memória (4096² RGBA = 64 MiB); mais do que isto não é
/// miniatura e é recusado antes de inflar.
const MAX_LADO_LIDO: u32 = 4096;

fn chunk(out: &mut Vec<u8>, tipo: &[u8; 4], dados: &[u8]) {
    let n = u32::try_from(dados.len()).expect("chunk PNG > 4 GiB");
    out.extend_from_slice(&n.to_be_bytes());
    let mut crc = crc32fast::Hasher::new();
    crc.update(tipo);
    crc.update(dados);
    out.extend_from_slice(tipo);
    out.extend_from_slice(dados);
    out.extend_from_slice(&crc.finalize().to_be_bytes());
}

/// O JSON canónico: chaves pela ordem do struct e dos `BTreeMap`, `f64` pelo menor texto que volta
/// ao mesmo bit. Recusa o que não voltaria igual (um `NaN` escreve-se `null` e não se lê como número).
fn json_canonico(p: &Parametros) -> Result<String, Erro> {
    let texto = serde_json::to_string_pretty(p).map_err(|e| Erro::Json(e.to_string()))?;
    let volta: Parametros = serde_json::from_str(&texto).map_err(|e| Erro::Json(e.to_string()))?;
    if &volta != p {
        return Err(Erro::Json(
            "um valor não volta igual depois de escrito (NaN ou infinito?)".to_owned(),
        ));
    }
    Ok(texto)
}

/// Escreve um PNG de 8 bits (`cor` 0 = cinzento, 6 = RGBA) com `antes_do_idat` entre o `IHDR` e o
/// `IDAT`. Filtro 0 em todas as linhas: o mais simples que é determinístico por construção.
fn png_8bits(cor: u8, canais: usize, w: u32, h: u32, px: &[u8], antes_do_idat: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(px.len() / 4);
    out.extend_from_slice(&ASSINATURA);
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&w.to_be_bytes());
    ihdr.extend_from_slice(&h.to_be_bytes());
    // 8 bits · cor · deflate · filtros adaptativos · sem entrelaçado.
    ihdr.extend_from_slice(&[8, cor, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    out.extend_from_slice(antes_do_idat);
    let linha = w as usize * canais;
    let mut cru = Vec::with_capacity(h as usize * (linha + 1));
    for y in 0..h as usize {
        cru.push(0);
        cru.extend_from_slice(&px[y * linha..(y + 1) * linha]);
    }
    chunk(
        &mut out,
        b"IDAT",
        &miniz_oxide::deflate::compress_to_vec_zlib(&cru, NIVEL_ZLIB),
    );
    chunk(&mut out, b"IEND", &[]);
    out
}

/// Escreve o ficheiro. Determinístico: o mesmo [`Preset`] dá sempre os mesmos bytes.
///
/// # Errors
/// [`Erro::Json`] se um parâmetro não for finito; [`Erro::MiniaturaIlegivel`] se a miniatura não
/// tiver `largura × altura × 4` bytes.
pub fn escrever(p: &Preset) -> Result<Vec<u8>, Erro> {
    let m = &p.miniatura;
    let esperado = m.largura as usize * m.altura as usize * 4;
    if m.largura == 0 || m.altura == 0 || m.rgba.len() != esperado {
        return Err(Erro::MiniaturaIlegivel(format!(
            "{}×{} com {} bytes (esperava {esperado})",
            m.largura,
            m.altura,
            m.rgba.len()
        )));
    }
    let json = json_canonico(&p.parametros)?;
    let mut meio = Vec::new();
    let mut origem = CHAVE_ORIGEM.as_bytes().to_vec();
    origem.push(0);
    origem.extend_from_slice(p.origem.texto().as_bytes());
    chunk(&mut meio, b"tEXt", &origem);
    // iTXt: chave \0 · comprimido (1) · método (0) · língua \0 · chave traduzida \0 · texto.
    let mut itxt = CHAVE_PARAMETROS.as_bytes().to_vec();
    itxt.extend_from_slice(&[0, 1, 0, 0, 0]);
    itxt.extend_from_slice(&miniz_oxide::deflate::compress_to_vec_zlib(
        json.as_bytes(),
        NIVEL_ZLIB,
    ));
    chunk(&mut meio, b"iTXt", &itxt);
    for (id, bytes) in &p.texturas {
        let mut d = id.as_bytes().to_vec();
        d.push(0);
        d.extend_from_slice(bytes);
        chunk(&mut meio, CHUNK_TEXTURA, &d);
    }
    Ok(png_8bits(6, 4, m.largura, m.altura, &m.rgba, &meio))
}

/// Uma textura NOSSA (a máscara cinzenta de uma forma ou de um grão) como PNG cinzento de 8 bits — o
/// conteúdo de um chunk `phTx`, que qualquer programa abre se o extraírem.
///
/// # Panics
/// Se `lum` não tiver `w × h` bytes (erro de quem chama: a máscara do pincel tem sempre).
#[must_use]
pub fn escrever_textura(lum: &[u8], w: u32, h: u32) -> Vec<u8> {
    assert_eq!(lum.len(), w as usize * h as usize, "textura {w}×{h}");
    png_8bits(0, 1, w, h, lum, &[])
}

/// Lê uma textura escrita por [`escrever_textura`] (ou qualquer PNG de 8 bits: a cor vira luminância
/// pelo canal vermelho, que numa máscara cinzenta é o próprio valor).
///
/// # Errors
/// Os de [`ler`] para a parte da imagem.
pub fn ler_textura(bytes: &[u8]) -> Result<(Vec<u8>, u32, u32), Erro> {
    let (cab, cru) = imagem_crua(bytes)?;
    let px = desfiltrar(&cab, &cru)?;
    let lum = if cab.canais == 1 {
        px
    } else {
        px.chunks_exact(4).map(|p| p[0]).collect()
    };
    Ok((lum, cab.largura, cab.altura))
}

fn inflar(dados: &[u8], limite: usize, o_que: &str) -> Result<Vec<u8>, Erro> {
    miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(dados, limite)
        .map_err(|e| Erro::ChunkCorrompido(format!("{o_que}: zlib {:?}", e.status)))
}

struct Cabecalho {
    largura: u32,
    altura: u32,
    canais: usize,
}

fn cabecalho(d: &[u8]) -> Result<Cabecalho, Erro> {
    if d.len() != 13 {
        return Err(Erro::ChunkCorrompido("IHDR".to_owned()));
    }
    let largura = u32::from_be_bytes([d[0], d[1], d[2], d[3]]);
    let altura = u32::from_be_bytes([d[4], d[5], d[6], d[7]]);
    let (profundidade, cor, entrelacado) = (d[8], d[9], d[12]);
    let canais = match cor {
        6 => 4,
        2 => 3,
        0 => 1,
        c => {
            return Err(Erro::MiniaturaIlegivel(format!("tipo de cor {c}")));
        }
    };
    if profundidade != 8 || entrelacado != 0 {
        return Err(Erro::MiniaturaIlegivel(format!(
            "{profundidade} bits, entrelaçado {entrelacado}"
        )));
    }
    if largura == 0 || altura == 0 || largura > MAX_LADO_LIDO || altura > MAX_LADO_LIDO {
        return Err(Erro::MiniaturaIlegivel(format!("{largura}×{altura}")));
    }
    Ok(Cabecalho {
        largura,
        altura,
        canais,
    })
}

/// Paeth (PNG §9.4).
fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let p = i16::from(a) + i16::from(b) - i16::from(c);
    let (pa, pb, pc) = (
        (p - i16::from(a)).abs(),
        (p - i16::from(b)).abs(),
        (p - i16::from(c)).abs(),
    );
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

/// Desfaz os cinco filtros por linha: RGBA8 (RGB ganha alfa 255) ou, num PNG cinzento, a luminância.
fn desfiltrar(h: &Cabecalho, cru: &[u8]) -> Result<Vec<u8>, Erro> {
    let bpp = h.canais;
    let linha = h.largura as usize * bpp;
    if cru.len() != h.altura as usize * (linha + 1) {
        return Err(Erro::MiniaturaIlegivel(format!(
            "IDAT com {} bytes para {}×{}",
            cru.len(),
            h.largura,
            h.altura
        )));
    }
    let mut img = vec![0u8; h.altura as usize * linha];
    for y in 0..h.altura as usize {
        let filtro = cru[y * (linha + 1)];
        let src = &cru[y * (linha + 1) + 1..(y + 1) * (linha + 1)];
        let (antes, resto) = img.split_at_mut(y * linha);
        let acima: &[u8] = if y == 0 { &[] } else { &antes[(y - 1) * linha..] };
        let cur = &mut resto[..linha];
        for i in 0..linha {
            let a = if i >= bpp { cur[i - bpp] } else { 0 };
            let b = acima.get(i).copied().unwrap_or(0);
            let c = if i >= bpp {
                acima.get(i - bpp).copied().unwrap_or(0)
            } else {
                0
            };
            let pred = match filtro {
                0 => 0,
                1 => a,
                2 => b,
                3 => ((u16::from(a) + u16::from(b)) / 2) as u8,
                4 => paeth(a, b, c),
                f => return Err(Erro::MiniaturaIlegivel(format!("filtro {f}"))),
            };
            cur[i] = src[i].wrapping_add(pred);
        }
    }
    if bpp == 4 || bpp == 1 {
        return Ok(img);
    }
    Ok(img
        .chunks_exact(3)
        .flat_map(|p| [p[0], p[1], p[2], 255])
        .collect())
}

fn parametros(json: &[u8]) -> Result<Parametros, Erro> {
    let v: serde_json::Value =
        serde_json::from_slice(json).map_err(|e| Erro::Json(e.to_string()))?;
    let schema = v
        .get("schema")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| Erro::Json("sem `schema`".to_owned()))?;
    if schema > u64::from(SCHEMA) {
        return Err(Erro::SchemaFuturo(
            u32::try_from(schema).unwrap_or(u32::MAX),
        ));
    }
    serde_json::from_value(v).map_err(|e| Erro::Json(e.to_string()))
}

fn texto_itxt(d: &[u8]) -> Result<Option<Vec<u8>>, Erro> {
    let corrompido = || Erro::ChunkCorrompido("iTXt".to_owned());
    let fim_chave = d.iter().position(|&b| b == 0).ok_or_else(corrompido)?;
    if &d[..fim_chave] != CHAVE_PARAMETROS.as_bytes() {
        return Ok(None);
    }
    let resto = d.get(fim_chave + 1..).ok_or_else(corrompido)?;
    let [comprimido, _metodo, resto @ ..] = resto else {
        return Err(corrompido());
    };
    let fim_lingua = resto.iter().position(|&b| b == 0).ok_or_else(corrompido)?;
    let resto = &resto[fim_lingua + 1..];
    let fim_trad = resto.iter().position(|&b| b == 0).ok_or_else(corrompido)?;
    let texto = &resto[fim_trad + 1..];
    Ok(Some(if *comprimido == 1 {
        inflar(texto, MAX_TEXTO, "iTXt")?
    } else {
        texto.to_vec()
    }))
}

/// Os chunks até ao `IEND`, com o CRC de cada um conferido.
fn chunks(bytes: &[u8]) -> Result<Vec<([u8; 4], &[u8])>, Erro> {
    if bytes.len() < 8 || bytes[..8] != ASSINATURA {
        return Err(Erro::NaoEPng);
    }
    let mut out = Vec::new();
    let mut i = 8;
    while i < bytes.len() {
        let corte = || Erro::ChunkCorrompido(format!("no byte {i}"));
        let n = bytes
            .get(i..i + 4)
            .ok_or_else(corte)?
            .try_into()
            .map(u32::from_be_bytes)
            .map_err(|_| corte())? as usize;
        let tipo: [u8; 4] = bytes
            .get(i + 4..i + 8)
            .ok_or_else(corte)?
            .try_into()
            .map_err(|_| corte())?;
        let nome = || Erro::ChunkCorrompido(String::from_utf8_lossy(&tipo).into_owned());
        let dados = bytes.get(i + 8..i + 8 + n).ok_or_else(nome)?;
        let crc_lido: [u8; 4] = bytes
            .get(i + 8 + n..i + 12 + n)
            .ok_or_else(nome)?
            .try_into()
            .map_err(|_| nome())?;
        let mut crc = crc32fast::Hasher::new();
        crc.update(&tipo);
        crc.update(dados);
        if crc.finalize() != u32::from_be_bytes(crc_lido) {
            return Err(Erro::ChunkCorrompido(format!(
                "{} (CRC)",
                String::from_utf8_lossy(&tipo)
            )));
        }
        out.push((tipo, dados));
        if &tipo == b"IEND" {
            break;
        }
        i += 12 + n;
    }
    Ok(out)
}

/// O cabeçalho e o `IDAT` inflado (ainda filtrado).
fn imagem_crua(bytes: &[u8]) -> Result<(Cabecalho, Vec<u8>), Erro> {
    let cs = chunks(bytes)?;
    let (_, ihdr) = cs
        .iter()
        .find(|(t, _)| t == b"IHDR")
        .ok_or_else(|| Erro::ChunkCorrompido("IHDR ausente".to_owned()))?;
    let cab = cabecalho(ihdr)?;
    let idat: Vec<u8> = cs
        .iter()
        .filter(|(t, _)| t == b"IDAT")
        .flat_map(|(_, d)| d.iter().copied())
        .collect();
    let limite = cab.altura as usize * (cab.largura as usize * cab.canais + 1);
    let cru = inflar(&idat, limite, "IDAT")?;
    Ok((cab, cru))
}

/// Lê um ficheiro de preset.
///
/// # Errors
/// Ver [`Erro`]: cada variante diz porque o ficheiro não se leu.
pub fn ler(bytes: &[u8]) -> Result<Preset, Erro> {
    let mut origem = Origem::Importado;
    let mut json = None;
    let mut texturas = BTreeMap::new();
    for (tipo, dados) in chunks(bytes)? {
        match &tipo {
            b"tEXt" => {
                if let Some(z) = dados.iter().position(|&b| b == 0)
                    && &dados[..z] == CHAVE_ORIGEM.as_bytes()
                {
                    origem = std::str::from_utf8(&dados[z + 1..])
                        .ok()
                        .and_then(Origem::de_texto)
                        .unwrap_or(Origem::Importado);
                }
            }
            b"iTXt" => {
                if let Some(t) = texto_itxt(dados)? {
                    json = Some(t);
                }
            }
            t if t == CHUNK_TEXTURA => {
                let z = dados
                    .iter()
                    .position(|&b| b == 0)
                    .ok_or_else(|| Erro::ChunkCorrompido("phTx".to_owned()))?;
                let id = String::from_utf8_lossy(&dados[..z]).into_owned();
                texturas.insert(id, dados[z + 1..].to_vec());
            }
            _ => {}
        }
    }
    let parametros = parametros(&json.ok_or(Erro::SemParametros)?)?;
    let (cab, cru) = imagem_crua(bytes)?;
    let px = desfiltrar(&cab, &cru)?;
    let rgba = if cab.canais == 1 {
        px.iter().flat_map(|&l| [l, l, l, 255]).collect()
    } else {
        px
    };
    Ok(Preset {
        parametros,
        origem,
        miniatura: Miniatura {
            largura: cab.largura,
            altura: cab.altura,
            rgba,
        },
        texturas,
    })
}

#[cfg(test)]
#[path = "png_tests.rs"]
mod tests;

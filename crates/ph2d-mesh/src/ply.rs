//! **Leitor de PLY** — ASCII e binário little-endian, com a COR.
//!
//! O PLY é o único dos três que carrega cor por vértice **no padrão** (não numa
//! extensão tolerada, como o `v x y z r g b` do OBJ), e é o formato que sai de
//! todo scanner fotogramétrico. É por isso que ele é a rota de ida-e-volta que
//! preserva o que o artista pintou.
//!
//! ## O cabeçalho é uma DECLARAÇÃO, e é preciso lê-la
//!
//! ⚠️ **A ordem e o tipo das propriedades são do ARQUIVO, não nossos.** Um leitor
//! que assuma `x y z red green blue` porque é isso que ele mesmo escreve lê
//! errado o primeiro PLY de terceiro que tiver `nx ny nz` no meio — e não falha:
//! ele devolve **normais como cor** e coordenadas deslocadas. O cabeçalho diz o
//! deslocamento de cada propriedade e o tamanho de cada uma; nós obedecemos.
//!
//! ⚠️ **Só o elemento `vertex` e o `face` importam**, mas os OUTROS têm de ser
//! PULADOS pelo tamanho declarado — um PLY com `element edge` entre eles
//! desalinharia o corpo binário inteiro se fosse ignorado em vez de pulado.
//!
//! ⚠️ **`format binary_big_endian` é RECUSADO com nome**, em vez de lido ao
//! contrário. Ele existe e é raro; ler os bytes trocados produziria coordenadas
//! astronômicas e uma malha que "carregou" — a recusa nomeia o arquivo.

use crate::face::Face;
use crate::mesh::{Mesh, MeshError};

/// O que pode dar errado lendo um PLY.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlyError {
    /// Não começa com `ply`, ou o `end_header` nunca chega.
    BadHeader,
    /// `format` que este leitor não implementa (ex.: big-endian).
    UnsupportedFormat(String),
    /// Um tipo de propriedade fora do vocabulário do PLY.
    UnknownType(String),
    /// Faltam `x`/`y`/`z` no elemento `vertex`.
    NoPositions,
    /// Os bytes acabaram antes do que o cabeçalho promete.
    Truncated,
    /// A geometria carregou, mas não forma uma malha.
    Mesh(MeshError),
}

impl core::fmt::Display for PlyError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BadHeader => write!(f, "cabeçalho PLY ausente ou incompleto"),
            Self::UnsupportedFormat(s) => write!(f, "formato PLY não suportado: {s}"),
            Self::UnknownType(s) => write!(f, "tipo de propriedade desconhecido: {s}"),
            Self::NoPositions => write!(f, "o elemento vertex não declara x/y/z"),
            Self::Truncated => write!(f, "PLY truncado: os dados acabam antes do declarado"),
            Self::Mesh(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for PlyError {}

/// Quantos bytes um tipo escalar do PLY ocupa.
fn width(ty: &str) -> Option<usize> {
    match ty {
        "char" | "uchar" | "int8" | "uint8" => Some(1),
        "short" | "ushort" | "int16" | "uint16" => Some(2),
        "int" | "uint" | "int32" | "uint32" | "float" | "float32" => Some(4),
        "double" | "float64" => Some(8),
        _ => None,
    }
}

/// Um `element` do cabeçalho: o nome, quantos registros, as propriedades
/// escalares e — se houver — a de LISTA.
///
/// ⚠️ Um `struct` e não uma tupla porque os quatro campos são perguntados em
/// pontos diferentes, e `e.2` num laço de leitura binária é onde um leitor
/// troca as propriedades pela contagem sem o compilador reclamar.
struct Element {
    name: String,
    count: usize,
    props: Vec<Prop>,
    /// `(tipo da contagem, tipo do item)`.
    list: Option<(String, String)>,
}

/// Uma propriedade escalar declarada no cabeçalho.
struct Prop {
    name: String,
    ty: String,
    /// Byte em que ela começa, dentro de um registro de vértice.
    at: usize,
}

/// Lê um escalar como `f64`, do jeito que o tipo dele manda.
fn scalar(bytes: &[u8], at: usize, ty: &str) -> Option<f64> {
    let w = width(ty)?;
    let s = bytes.get(at..at + w)?;
    Some(match ty {
        "char" | "int8" => s[0] as i8 as f64,
        "uchar" | "uint8" => s[0] as f64,
        "short" | "int16" => i16::from_le_bytes([s[0], s[1]]) as f64,
        "ushort" | "uint16" => u16::from_le_bytes([s[0], s[1]]) as f64,
        "int" | "int32" => i32::from_le_bytes([s[0], s[1], s[2], s[3]]) as f64,
        "uint" | "uint32" => u32::from_le_bytes([s[0], s[1], s[2], s[3]]) as f64,
        "float" | "float32" => f32::from_le_bytes([s[0], s[1], s[2], s[3]]) as f64,
        _ => f64::from_le_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]]),
    })
}

/// Uma cor de arquivo vira `[0,1]`: inteiro é 0..255, ponto flutuante já vem
/// normalizado.
///
/// ⚠️ **A regra é do TIPO, não do valor** — decidir por *"é maior que 1, então
/// deve ser 0..255"* faria um PLY em `float` com um canal levemente acima de 1
/// (o que HDR produz) virar preto quase puro.
fn to_unit(v: f64, ty: &str) -> f32 {
    let x = if matches!(ty, "float" | "float32" | "double" | "float64") {
        v
    } else {
        v / 255.0
    };
    x.clamp(0.0, 1.0) as f32
}

/// Lê um PLY (ASCII ou binário little-endian).
pub fn import_ply(bytes: &[u8]) -> Result<Mesh, PlyError> {
    let head_end = find_header_end(bytes).ok_or(PlyError::BadHeader)?;
    let header = String::from_utf8_lossy(&bytes[..head_end]);
    if !header.trim_start().starts_with("ply") {
        return Err(PlyError::BadHeader);
    }

    let mut ascii = None;
    // (nome, contagem, propriedades escalares, propriedade de lista)
    let mut elements: Vec<Element> = Vec::new();
    for line in header.lines() {
        let mut it = line.split_ascii_whitespace();
        match it.next() {
            Some("format") => {
                let f = it.next().unwrap_or_default();
                ascii = Some(match f {
                    "ascii" => true,
                    "binary_little_endian" => false,
                    other => return Err(PlyError::UnsupportedFormat(other.to_string())),
                });
            }
            Some("element") => {
                let name = it.next().unwrap_or_default().to_string();
                let n = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                elements.push(Element {
                    name,
                    count: n,
                    props: Vec::new(),
                    list: None,
                });
            }
            Some("property") => {
                let Some(Element { props, list, .. }) = elements.last_mut() else {
                    continue;
                };
                let ty = it.next().unwrap_or_default();
                if ty == "list" {
                    let count_ty = it.next().unwrap_or_default().to_string();
                    let item_ty = it.next().unwrap_or_default().to_string();
                    if width(&count_ty).is_none() || width(&item_ty).is_none() {
                        return Err(PlyError::UnknownType(format!("{count_ty}/{item_ty}")));
                    }
                    *list = Some((count_ty, item_ty));
                } else {
                    let w = width(ty).ok_or_else(|| PlyError::UnknownType(ty.to_string()))?;
                    let at = props
                        .last()
                        .map_or(0, |p: &Prop| p.at + width(&p.ty).unwrap_or(0));
                    let _ = w;
                    props.push(Prop {
                        name: it.next().unwrap_or_default().to_string(),
                        ty: ty.to_string(),
                        at,
                    });
                }
            }
            _ => {}
        }
    }
    let ascii = ascii.ok_or(PlyError::BadHeader)?;

    let body = &bytes[head_end..];
    let (positions, colors, faces) = if ascii {
        read_ascii(body, &elements)?
    } else {
        read_binary(body, &elements)?
    };
    if positions.is_empty() {
        return Err(PlyError::NoPositions);
    }
    let mut mesh = Mesh::from_parts(positions, faces).map_err(PlyError::Mesh)?;
    if let Some(c) = colors {
        mesh.colors_mut().copy_from_slice(&c);
    }
    Ok(mesh)
}

fn find_header_end(bytes: &[u8]) -> Option<usize> {
    let needle = b"end_header";
    bytes
        .windows(needle.len())
        .position(|w| w == needle)
        .and_then(|at| {
            // Consome o resto da linha (aceita `\n` e `\r\n`).
            bytes[at..]
                .iter()
                .position(|&b| b == b'\n')
                .map(|nl| at + nl + 1)
        })
}

type Parsed = (Vec<[f32; 3]>, Option<Vec<[f32; 3]>>, Vec<Face>);

/// Onde estão x/y/z e r/g/b dentro de um registro de vértice.
fn locate(props: &[Prop]) -> (Option<[usize; 3]>, Option<[usize; 3]>) {
    let find = |names: [&str; 3]| -> Option<[usize; 3]> {
        let mut out = [usize::MAX; 3];
        for (k, want) in names.iter().enumerate() {
            out[k] = props.iter().position(|p| p.name == *want)?;
        }
        Some(out)
    };
    (
        find(["x", "y", "z"]),
        find(["red", "green", "blue"]).or_else(|| find(["r", "g", "b"])),
    )
}

/// Uma face vira `Face`: tri e quad direto, n-gon como leque.
///
/// ⚠️ **O leque é a MESMA escolha do `import_obj`**, e ela tem de ser a mesma:
/// dois leitores que triangulassem diferente dariam malhas diferentes para o
/// mesmo modelo dependendo da extensão do arquivo.
fn push_face(idx: &[u32], out: &mut Vec<Face>) {
    match idx.len() {
        3 => out.push(Face::tri(idx[0], idx[1], idx[2])),
        4 => out.push(Face(
            ([idx[0], idx[1], idx[2], idx[3]])[..].try_into().unwrap(),
        )),
        n if n > 4 => {
            for k in 1..n - 1 {
                out.push(Face::tri(idx[0], idx[k], idx[k + 1]));
            }
        }
        _ => {}
    }
}

fn read_ascii(body: &[u8], elements: &[Element]) -> Result<Parsed, PlyError> {
    let text = String::from_utf8_lossy(body);
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let mut positions = Vec::new();
    let mut colors: Option<Vec<[f32; 3]>> = None;
    let mut faces = Vec::new();
    for Element {
        name,
        count: n,
        props,
        list,
    } in elements
    {
        if name == "vertex" {
            let (xyz, rgb) = locate(props);
            let xyz = xyz.ok_or(PlyError::NoPositions)?;
            if rgb.is_some() {
                colors = Some(Vec::with_capacity(*n));
            }
            for _ in 0..*n {
                let line = lines.next().ok_or(PlyError::Truncated)?;
                let t: Vec<&str> = line.split_ascii_whitespace().collect();
                let num =
                    |i: usize| -> f64 { t.get(i).and_then(|s| s.parse().ok()).unwrap_or(0.0) };
                positions.push([num(xyz[0]) as f32, num(xyz[1]) as f32, num(xyz[2]) as f32]);
                if let (Some(c), Some(rgb)) = (colors.as_mut(), rgb) {
                    c.push([
                        to_unit(num(rgb[0]), &props[rgb[0]].ty),
                        to_unit(num(rgb[1]), &props[rgb[1]].ty),
                        to_unit(num(rgb[2]), &props[rgb[2]].ty),
                    ]);
                }
            }
        } else if name == "face" && list.is_some() {
            for _ in 0..*n {
                let line = lines.next().ok_or(PlyError::Truncated)?;
                let t: Vec<u32> = line
                    .split_ascii_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
                if t.is_empty() {
                    continue;
                }
                let k = t[0] as usize;
                push_face(&t[1..(1 + k).min(t.len())], &mut faces);
            }
        } else {
            for _ in 0..*n {
                lines.next().ok_or(PlyError::Truncated)?;
            }
        }
    }
    Ok((positions, colors, faces))
}

fn read_binary(body: &[u8], elements: &[Element]) -> Result<Parsed, PlyError> {
    let mut at = 0usize;
    let mut positions = Vec::new();
    let mut colors: Option<Vec<[f32; 3]>> = None;
    let mut faces = Vec::new();
    for Element {
        name,
        count: n,
        props,
        list,
    } in elements
    {
        let stride: usize = props.iter().map(|p| width(&p.ty).unwrap_or(0)).sum();
        if name == "vertex" && list.is_none() {
            let (xyz, rgb) = locate(props);
            let xyz = xyz.ok_or(PlyError::NoPositions)?;
            if rgb.is_some() {
                colors = Some(Vec::with_capacity(*n));
            }
            for _ in 0..*n {
                let rec = body.get(at..at + stride).ok_or(PlyError::Truncated)?;
                let get =
                    |i: usize| -> f64 { scalar(rec, props[i].at, &props[i].ty).unwrap_or(0.0) };
                positions.push([get(xyz[0]) as f32, get(xyz[1]) as f32, get(xyz[2]) as f32]);
                if let (Some(c), Some(rgb)) = (colors.as_mut(), rgb) {
                    c.push([
                        to_unit(get(rgb[0]), &props[rgb[0]].ty),
                        to_unit(get(rgb[1]), &props[rgb[1]].ty),
                        to_unit(get(rgb[2]), &props[rgb[2]].ty),
                    ]);
                }
                at += stride;
            }
        } else if let Some((count_ty, item_ty)) = list {
            let (cw, iw) = (width(count_ty).unwrap_or(1), width(item_ty).unwrap_or(4));
            for _ in 0..*n {
                at += stride; // escalares antes da lista, se houver
                let k = scalar(body, at, count_ty).ok_or(PlyError::Truncated)? as usize;
                at += cw;
                let mut idx = Vec::with_capacity(k);
                for j in 0..k {
                    idx.push(scalar(body, at + j * iw, item_ty).ok_or(PlyError::Truncated)? as u32);
                }
                at += k * iw;
                if name == "face" {
                    push_face(&idx, &mut faces);
                }
            }
        } else {
            // ⚠️ Elemento que não nos interessa: PULADO pelo tamanho, nunca
            // ignorado — ignorar desalinha todo o resto do corpo.
            at += stride * *n;
        }
    }
    Ok((positions, colors, faces))
}

#[cfg(test)]
#[path = "ply_tests.rs"]
mod tests;

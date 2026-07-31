//! Import de OBJ — o mínimo para haver o que esculpir.
//!
//! Adaptado de `reference/sculptgl/src/files/ImportOBJ.js`, MIT — ver
//! `LICENSES/sculptgl-MIT.txt`.
//!
//! **Escopo, dito na porta:** geometria e cor por vértice. UV, materiais,
//! grupos e suavização são **ignorados em silêncio** — não por descuido: a W1
//! existe para haver uma malha na tela, e um importador que finge entender
//! `usemtl` cria a expectativa de que o material sobreviveu à viagem. Quando
//! houver dono para UV (a doação, W3), ele entra aqui com gate próprio.
//!
//! ⚠️ **Quads são PRESERVADOS.** Triangular na porta de entrada jogaria fora
//! exatamente a topologia que a multiresolução (`docs/3D/04.3`) precisa — o
//! `Face` guarda tri e quad justamente para isso. N-gons acima de 4 viram leque
//! de triângulos, porque não há representação para eles.

use crate::face::Face;
use crate::mesh::{Mesh, MeshError};

/// O que pode dar errado lendo um OBJ.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ObjError {
    /// `f` referenciou um vértice que o arquivo não declarou (índice 0, fora de
    /// alcance, ou negativo além do começo).
    BadFaceIndex { line: usize, token: String },
    /// A geometria carregou, mas não forma uma malha (ver `MeshError`).
    Mesh(MeshError),
}

impl core::fmt::Display for ObjError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BadFaceIndex { line, token } => {
                write!(f, "linha {line}: índice de face inválido {token:?}")
            }
            Self::Mesh(e) => write!(f, "{e}"),
        }
    }
}

impl core::error::Error for ObjError {}

/// Lê um OBJ de texto.
///
/// Devolve a malha **já construída** (normais, adjacência e octree derivados),
/// e a cor por vértice só é materializada se o arquivo de fato trouxer cor — a
/// preguiça dos planos, honrada desde a porta de entrada.
pub fn import_obj(text: &str) -> Result<Mesh, ObjError> {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 3]> = Vec::new();
    let mut has_colors = false;
    let mut faces: Vec<Face> = Vec::new();
    let mut idx: Vec<u32> = Vec::new();

    for (lineno, raw) in text.lines().enumerate() {
        let line = raw.trim();
        let mut it = line.split_ascii_whitespace();
        match it.next() {
            Some("v") => {
                let n: Vec<f32> = it.filter_map(|t| t.parse::<f32>().ok()).collect();
                if n.len() < 3 {
                    continue;
                }
                positions.push([n[0], n[1], n[2]]);
                if n.len() >= 6 {
                    has_colors = true;
                    colors.push([n[3], n[4], n[5]]);
                } else {
                    colors.push(crate::mesh::DEFAULT_COLOR);
                }
            }
            Some("f") => {
                idx.clear();
                for token in it {
                    // `a`, `a/b`, `a//c`, `a/b/c` — só o primeiro campo é a
                    // posição, e é o único que esta wave consome.
                    let first = token.split('/').next().unwrap_or("");
                    let Ok(raw_i) = first.parse::<i64>() else {
                        return Err(ObjError::BadFaceIndex {
                            line: lineno + 1,
                            token: token.to_string(),
                        });
                    };
                    // OBJ é 1-based, e negativo conta de trás para a frente a
                    // partir do que já foi declarado.
                    let resolved = if raw_i > 0 {
                        raw_i - 1
                    } else if raw_i < 0 {
                        positions.len() as i64 + raw_i
                    } else {
                        -1
                    };
                    if resolved < 0 || resolved >= positions.len() as i64 {
                        return Err(ObjError::BadFaceIndex {
                            line: lineno + 1,
                            token: token.to_string(),
                        });
                    }
                    idx.push(resolved as u32);
                }
                push_face(&idx, &mut faces);
            }
            _ => {}
        }
    }

    let mut mesh = Mesh::from_parts(positions, faces).map_err(ObjError::Mesh)?;
    if has_colors {
        mesh.colors_mut().copy_from_slice(&colors);
    }
    Ok(mesh)
}

/// Um polígono do arquivo vira 1 face (tri/quad) ou um leque de triângulos.
fn push_face(idx: &[u32], faces: &mut Vec<Face>) {
    match idx.len() {
        0..=2 => {} // linha ou ponto: não é superfície
        3 => faces.push(Face::tri(idx[0], idx[1], idx[2])),
        4 => faces.push(Face::quad(idx[0], idx[1], idx[2], idx[3])),
        _ => {
            for k in 1..idx.len() - 1 {
                faces.push(Face::tri(idx[0], idx[k], idx[k + 1]));
            }
        }
    }
}

#[cfg(test)]
#[path = "obj_tests.rs"]
mod tests;

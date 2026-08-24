//! **A PORTA DE DADOS** — o formato de texto em que um mapa de grade inteira viaja.
//!
//! ⚠️ **A imagem é POR CANTO, não por vértice** — é isso que permite a cada
//! triângulo ter a sua **carta**, e é de comparar as duas imagens de uma aresta
//! partilhada que sai a função de transição.
//!
//! ```text
//! malha <nV> <nF>
//! v <x> <y> <z>              # a superfície, em R³
//! f <a> <b> <c>              # triângulos, índices base 0
//! canto <face> <k> <u> <v>   # a imagem do canto k (0..2) daquela face
//! ```
//!
//! ⚠️ **Linhas desconhecidas são IGNORADAS e contadas**, nunca rejeitadas em
//! silêncio: um produtor que acrescente uma directiva nova tem de aparecer no
//! relatório de quem a não entendeu.

/// Um mapa lido de texto.
#[derive(Debug, Clone, Default)]
pub struct Mapa {
    /// Os vértices em `R³`.
    pub pos: Vec<[f32; 3]>,
    /// Os triângulos.
    pub tris: Vec<[u32; 3]>,
    /// Por face, por canto, a imagem no domínio.
    pub uv: Vec<[[f64; 2]; 3]>,
    /// Directivas que o leitor não conhece.
    pub unknown: usize,
}

/// O que impede um texto de ser um mapa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapaError {
    /// Uma directiva com campos a menos, ou um número que não é número.
    Malformed(usize),
    /// Um `canto` que aponta para uma face ou um canto que não existem.
    CornerOutOfRange(usize),
    /// Uma face sem os três cantos.
    MissingCorner(usize),
    /// Um triângulo que cita um vértice que não existe.
    VertexOutOfRange(usize),
}

impl core::fmt::Display for MapaError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Malformed(l) => write!(f, "linha {l}: directiva malformada"),
            Self::CornerOutOfRange(l) => write!(f, "linha {l}: canto fora de alcance"),
            Self::MissingCorner(i) => write!(f, "face {i}: falta um canto"),
            Self::VertexOutOfRange(i) => write!(f, "face {i}: vértice fora de alcance"),
        }
    }
}

impl core::error::Error for MapaError {}

impl Mapa {
    /// Lê um mapa de texto.
    ///
    /// # Errors
    /// Ver [`MapaError`].
    pub fn parse(text: &str) -> Result<Self, MapaError> {
        let mut out = Self::default();
        let mut seen: Vec<[bool; 3]> = Vec::new();
        for (n, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut it = line.split_whitespace();
            let Some(head) = it.next() else { continue };
            match head {
                "v" => {
                    let v = triple_f32(&mut it).ok_or(MapaError::Malformed(n + 1))?;
                    out.pos.push(v);
                }
                "f" => {
                    let t = triple_u32(&mut it).ok_or(MapaError::Malformed(n + 1))?;
                    out.tris.push(t);
                    out.uv.push([[0.0; 2]; 3]);
                    seen.push([false; 3]);
                }
                "canto" => {
                    let f: usize = it
                        .next()
                        .and_then(|s| s.parse().ok())
                        .ok_or(MapaError::Malformed(n + 1))?;
                    let k: usize = it
                        .next()
                        .and_then(|s| s.parse().ok())
                        .ok_or(MapaError::Malformed(n + 1))?;
                    let u: f64 = it
                        .next()
                        .and_then(|s| s.parse().ok())
                        .ok_or(MapaError::Malformed(n + 1))?;
                    let w: f64 = it
                        .next()
                        .and_then(|s| s.parse().ok())
                        .ok_or(MapaError::Malformed(n + 1))?;
                    if f >= out.uv.len() || k >= 3 {
                        return Err(MapaError::CornerOutOfRange(n + 1));
                    }
                    out.uv[f][k] = [u, w];
                    seen[f][k] = true;
                }
                "malha" => {}
                _ => out.unknown += 1,
            }
        }
        for (i, s) in seen.iter().enumerate() {
            if !s.iter().all(|x| *x) {
                return Err(MapaError::MissingCorner(i));
            }
        }
        let nv = u32::try_from(out.pos.len()).unwrap_or(u32::MAX);
        for (i, t) in out.tris.iter().enumerate() {
            if t.iter().any(|&v| v >= nv) {
                return Err(MapaError::VertexOutOfRange(i));
            }
        }
        Ok(out)
    }

    /// A vista que a extracção consome.
    #[must_use]
    pub fn as_map(&self) -> crate::CornerMap<'_> {
        crate::CornerMap {
            pos: &self.pos,
            tris: &self.tris,
            uv: &self.uv,
        }
    }
}

fn triple_f32<'a>(it: &mut impl Iterator<Item = &'a str>) -> Option<[f32; 3]> {
    Some([
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
    ])
}

fn triple_u32<'a>(it: &mut impl Iterator<Item = &'a str>) -> Option<[u32; 3]> {
    Some([
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
    ])
}

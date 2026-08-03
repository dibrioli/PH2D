//! **Leitor de STL** — binário e ASCII.
//!
//! O STL é o formato que sai de todo scanner e entra em toda impressora, e é o
//! mais pobre dos três: **triângulos soltos**, sem índices, sem cor, sem nome.
//! Ler um é reconstruir a informação que o formato jogou fora.
//!
//! ⚠️ **A SOLDA de vértices é obrigatória, não uma otimização.** Um STL de um
//! cubo traz 12 triângulos × 3 vértices = 36 posições, das quais 8 são
//! distintas; sem soldar, a malha resultante **não tem adjacência** — cada
//! triângulo é uma ilha —, e adjacência é o que o suavizar, o subdividir, o
//! espelho e o fechar-buraco leem. A malha desenharia certo e **toda ferramenta
//! de escultura seria inerte nela**, que é a forma mais cara de um import estar
//! errado.
//!
//! ⚠️ **A chave da solda são os BITS do `f32`, não o valor.** Comparar por
//! igualdade de ponto flutuante seria o mesmo, mas `f32` não é `Hash`/`Eq`; e um
//! épsilon seria pior: ele funde vértices que o autor pôs perto de propósito, e
//! a distância "perto" depende da escala do modelo, que não conhecemos. Um STL
//! bem-formado repete o **mesmo bit padrão** para o mesmo canto — é o que o
//! escritor faz, porque ele escreve o mesmo `f32` três vezes.
//!
//! ⚠️ **`-0.0` e `+0.0` têm bits diferentes** e são o mesmo ponto, então o zero é
//! normalizado antes de virar chave — sem isso um vértice sobre um plano de
//! simetria pode ficar dividido em dois, e a costura abre exatamente onde o
//! espelho trabalha.

use std::collections::BTreeMap;

use crate::face::Face;
use crate::mesh::{Mesh, MeshError};

/// O que pode dar errado lendo um STL.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StlError {
    /// Os bytes acabaram no meio de um triângulo, ou o cabeçalho promete mais
    /// do que o arquivo tem.
    Truncated { expected: usize, got: usize },
    /// Um `facet`/`vertex` do ASCII que não descreve três números finitos.
    BadVertex { line: usize, text: String },
    /// A geometria carregou, mas não forma uma malha.
    Mesh(MeshError),
}

impl core::fmt::Display for StlError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Truncated { expected, got } => {
                write!(f, "STL truncado: esperava {expected} bytes, li {got}")
            }
            Self::BadVertex { line, text } => {
                write!(f, "linha {line}: vértice malformado {text:?}")
            }
            Self::Mesh(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for StlError {}

/// Zero com sinal normalizado — ver a nota do módulo.
fn key(p: [f32; 3]) -> [u32; 3] {
    [
        (if p[0] == 0.0 { 0.0 } else { p[0] }).to_bits(),
        (if p[1] == 0.0 { 0.0 } else { p[1] }).to_bits(),
        (if p[2] == 0.0 { 0.0 } else { p[2] }).to_bits(),
    ]
}

/// Solda triângulos soltos numa malha indexada.
fn weld(tris: &[[[f32; 3]; 3]]) -> Result<Mesh, StlError> {
    // ⚠️ **`BTreeMap`, e a lint que o exige tem razão aqui**: a ordem em que
    // os vértices entram DECIDE os índices da malha, e um arquivo tem de dar a
    // mesma malha em toda máquina. (A ordem de inserção do `HashMap` também
    // seria determinística por acidente da implementação; o `BTreeMap` a torna
    // uma propriedade do TIPO, e é isso que a lint compra.)
    let mut seen: BTreeMap<[u32; 3], u32> = BTreeMap::new();
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut faces: Vec<Face> = Vec::with_capacity(tris.len());
    for t in tris {
        let mut idx = [0u32; 3];
        for (k, v) in t.iter().enumerate() {
            idx[k] = *seen.entry(key(*v)).or_insert_with(|| {
                positions.push(*v);
                (positions.len() - 1) as u32
            });
        }
        // ⚠️ Um triângulo cujos três cantos soldaram no mesmo vértice é uma
        // face DEGENERADA — o `from_parts` a aceitaria (os índices são válidos)
        // e ela envenenaria a normal e a adjacência. Descartar é o que o
        // arquivo já diz: um triângulo de área zero não desenha nada.
        if idx[0] != idx[1] && idx[1] != idx[2] && idx[0] != idx[2] {
            faces.push(Face::tri(idx[0], idx[1], idx[2]));
        }
    }
    Mesh::from_parts(positions, faces).map_err(StlError::Mesh)
}

/// Lê um STL, **binário ou ASCII**, e devolve uma malha indexada.
///
/// ⚠️ **A escolha do ramo NÃO é pela palavra `solid`**, e essa é a armadilha
/// clássica deste formato: o cabeçalho binário tem 80 bytes livres e muitos
/// escritores põem `solid <nome>` neles, então um leitor que decide pelo prefixo
/// lê um arquivo binário como texto e devolve uma malha vazia. Decidimos pelo
/// **TAMANHO**, que é uma propriedade que o formato binário garante:
/// `84 + 50·n` bytes exatos.
///
/// ⚠️ **E a recusa por truncamento é o ÚLTIMO recurso, não o primeiro** — a
/// primeira versão desta função a punha antes do ASCII e **engoliu um arquivo
/// ASCII legítimo**: num texto de 109 bytes, os bytes 80..84 são letras, leem
/// como uma contagem enorme, e a condição *"menor que o esperado"* casa. O
/// tamanho só distingue os dois formatos quando ele bate EXATAMENTE; em todo o
/// resto, quem decide é o ASCII **ter achado triângulos**, e o erro fica para
/// quando nenhum dos dois leu nada.
pub fn import_stl(bytes: &[u8]) -> Result<Mesh, StlError> {
    if let Some(tris) = binary_triangles(bytes) {
        return weld(&tris);
    }
    let tris = ascii_triangles(bytes)?;
    if tris.is_empty()
        && let Some(err) = truncated_binary(bytes)
    {
        return Err(err);
    }
    weld(&tris)
}

/// Um arquivo que promete `84 + 50·n` e entrega menos, sem ser texto legível.
fn truncated_binary(bytes: &[u8]) -> Option<StlError> {
    if bytes.len() <= 84 {
        return None;
    }
    let n = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]) as usize;
    let expected = 84usize.saturating_add(n.saturating_mul(50));
    (n > 0 && bytes.len() < expected).then_some(StlError::Truncated {
        expected,
        got: bytes.len(),
    })
}

/// `None` = não é binário; deixe o ASCII tentar.
fn binary_triangles(bytes: &[u8]) -> Option<Vec<[[f32; 3]; 3]>> {
    if bytes.len() < 84 {
        return None;
    }
    let n = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]) as usize;
    if bytes.len() != 84usize.saturating_add(n.saturating_mul(50)) {
        return None;
    }
    let mut tris = Vec::with_capacity(n);
    for i in 0..n {
        let at = 84 + i * 50 + 12; // pula a normal: derivamos das posições.
        let f = |o: usize| f32::from_le_bytes([bytes[o], bytes[o + 1], bytes[o + 2], bytes[o + 3]]);
        tris.push([
            [f(at), f(at + 4), f(at + 8)],
            [f(at + 12), f(at + 16), f(at + 20)],
            [f(at + 24), f(at + 28), f(at + 32)],
        ]);
    }
    Some(tris)
}

fn ascii_triangles(bytes: &[u8]) -> Result<Vec<[[f32; 3]; 3]>, StlError> {
    let text = String::from_utf8_lossy(bytes);
    let mut tris = Vec::new();
    let mut cur: Vec<[f32; 3]> = Vec::with_capacity(3);
    for (n, line) in text.lines().enumerate() {
        let mut it = line.split_ascii_whitespace();
        if it.next() != Some("vertex") {
            continue;
        }
        let v: Vec<f32> = it.filter_map(|t| t.parse::<f32>().ok()).collect();
        if v.len() < 3 || !v[..3].iter().all(|c| c.is_finite()) {
            return Err(StlError::BadVertex {
                line: n + 1,
                text: line.trim().to_string(),
            });
        }
        cur.push([v[0], v[1], v[2]]);
        if cur.len() == 3 {
            tris.push([cur[0], cur[1], cur[2]]);
            cur.clear();
        }
    }
    Ok(tris)
}

#[cfg(test)]
#[path = "stl_tests.rs"]
mod tests;

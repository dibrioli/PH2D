//! **A PORTA DE SAÍDA** — a escultura vira um arquivo que outro programa abre.
//!
//! A W8.4 deu a entrada e a W8.3 deu o documento; sem esta, o trabalho **entra,
//! salva e não sai**. Um arquivo `.ph2d` só abre aqui: levar a malha ao Blender,
//! ao ZBrush ou a uma impressora 3D é o que torna o módulo parte de um fluxo em
//! vez de um beco.
//!
//! ## A decisão que espelha o import: a geometria sai em MUNDO
//!
//! ⚠️ Desde a W8.1 a cena é uma LISTA, e a posição de cada peça vive na
//! [`Pose`], não na geometria (foi exatamente isso que a W8.4 estabeleceu ao
//! CENTRAR cada peça importada). Escrever a geometria local seria escrever
//! **todas as peças empilhadas na origem** — o defeito espelho do que o import
//! curou. Toda posição atravessa [`Pose::point_to_world`].
//!
//! ## O que cada formato PERDE, e por que isso é uma pergunta de CÓDIGO
//!
//! | | quads | cor | peças |
//! |---|---|---|---|
//! | **OBJ** | ✅ | ✅ (`v x y z r g b`) | ✅ (`o <nome>`) |
//! | **PLY** | ✅ (lista de tamanho livre) | ✅ (padrão, `uchar` RGB) | ❌ funde |
//! | **STL** | ❌ só triângulos | ❌ | ❌ funde |
//!
//! ⚠️ **A tabela é [`MeshFormat`], não prosa.** Quem escreve o arquivo e quem
//! AVISA o artista perguntam à mesma coisa; duas cópias dariam um toast dizendo
//! *"cor preservada"* sobre um STL, que é a resposta errada com a confiança da
//! certa.
//!
//! ⚠️ **A MÁSCARA não sobrevive a NENHUM deles, e por isso não é uma pergunta** —
//! nenhum dos três formatos tem campo para ela. Quem a preserva é o documento
//! (W8.3), e é isso que o aviso diz.
//!
//! ## Binário na escrita, os dois na leitura
//!
//! STL e PLY existem em ASCII e em binário. Escrevemos **binário** (é o que
//! Blender e scanners produzem por default, e o ASCII de uma malha de 100k
//! triângulos passa de 50 MB); os leitores aceitam **os dois**, porque o que
//! chega de fora não é escolha nossa.

use crate::face::TRI;
use crate::mesh::{DEFAULT_COLOR, Mesh};
use crate::pose::Pose;

/// Uma peça a caminho do arquivo.
///
/// ⚠️ Empresta em vez de possuir: a cena é a dona das malhas, e um exportador
/// que clonasse a pilha inteira para escrevê-la duplicaria dezenas de MB por um
/// arquivo que já vai ser serializado.
pub struct ExportPiece<'a> {
    /// O nome que vira `o <nome>` no OBJ. Os outros dois formatos não têm onde
    /// pô-lo.
    pub name: Option<&'a str>,
    /// O nível VIVO da pilha — o que está na tela é o que sai.
    pub mesh: &'a Mesh,
    /// Onde a peça está no mundo.
    pub pose: Pose,
}

/// Os formatos que a porta de saída conhece.
///
/// ⚠️ **A ÚNICA fonte sobre o que cada um carrega.** As perguntas
/// [`keeps_colour`](Self::keeps_colour) e [`keeps_pieces`](Self::keeps_pieces)
/// existem para o aviso ao artista sair do mesmo lugar que o escritor.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MeshFormat {
    /// Wavefront OBJ — texto, o formato que o módulo já LÊ.
    Obj,
    /// Stanford PLY — binário little-endian, com cor por vértice.
    Ply,
    /// STL binário — triângulos soltos, o formato da impressão 3D.
    Stl,
}

impl MeshFormat {
    /// O formato que esta extensão nomeia.
    ///
    /// ⚠️ **É a extensão que decide, e isso é desenho.** Um seletor de formato
    /// ao lado do nome do arquivo seriam DUAS portas para a mesma pergunta, e
    /// elas divergem no primeiro `retrato.obj` salvo com "STL" no dropdown.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "obj" => Some(Self::Obj),
            "ply" => Some(Self::Ply),
            "stl" => Some(Self::Stl),
            _ => None,
        }
    }

    /// A extensão canônica.
    pub fn extension(self) -> &'static str {
        match self {
            Self::Obj => "obj",
            Self::Ply => "ply",
            Self::Stl => "stl",
        }
    }

    /// Todos, na ordem em que um seletor os oferece.
    pub const ALL: [MeshFormat; 3] = [Self::Obj, Self::Ply, Self::Stl];

    /// A cor que o artista pintou atravessa?
    pub fn keeps_colour(self) -> bool {
        matches!(self, Self::Obj | Self::Ply)
    }

    /// As peças continuam separadas do outro lado?
    pub fn keeps_pieces(self) -> bool {
        matches!(self, Self::Obj)
    }

    /// Escreve a cena inteira neste formato.
    pub fn write(self, pieces: &[ExportPiece<'_>]) -> Vec<u8> {
        match self {
            Self::Obj => write_obj(pieces).into_bytes(),
            Self::Ply => write_ply(pieces),
            Self::Stl => write_stl(pieces),
        }
    }
}

/// A cor de um vértice, com o default explícito.
fn colour_at(mesh: &Mesh, i: usize) -> [f32; 3] {
    mesh.colors().map_or(DEFAULT_COLOR, |c| c[i])
}

/// Quantos vértices e quantas faces a cena inteira tem.
///
/// ⚠️ Existe porque PLY escreve as contagens no **cabeçalho**, antes de qualquer
/// dado — e um cabeçalho que discorde do corpo produz um arquivo que todo leitor
/// recusa longe da causa.
fn totals(pieces: &[ExportPiece<'_>]) -> (usize, usize) {
    pieces.iter().fold((0, 0), |(v, f), p| {
        (v + p.mesh.positions().len(), f + p.mesh.faces().len())
    })
}

/// **OBJ** — o único dos três que preserva peça, quad e cor ao mesmo tempo.
///
/// ⚠️ **Os índices de `f` são 1-based e GLOBAIS ao arquivo**, então a segunda
/// peça é escrita sobre um offset acumulado. É o espelho exato da compactação
/// que o import teve de fazer: lá o pool global vira peças, aqui as peças viram
/// um pool global.
pub fn write_obj(pieces: &[ExportPiece<'_>]) -> String {
    let mut out = String::from("# PH2D Sculpt\n");
    let mut base = 0usize;
    for (i, p) in pieces.iter().enumerate() {
        let name = p.name.unwrap_or("Piece");
        out.push_str(&format!("o {name}_{i}\n"));
        let coloured = p.mesh.colors().is_some();
        for (v, pos) in p.mesh.positions().iter().enumerate() {
            let w = p.pose.point_to_world(*pos);
            if coloured {
                let c = colour_at(p.mesh, v);
                out.push_str(&format!(
                    "v {} {} {} {} {} {}\n",
                    w[0], w[1], w[2], c[0], c[1], c[2]
                ));
            } else {
                out.push_str(&format!("v {} {} {}\n", w[0], w[1], w[2]));
            }
        }
        for f in p.mesh.faces() {
            out.push('f');
            for &idx in f.verts() {
                out.push_str(&format!(" {}", base + idx as usize + 1));
            }
            out.push('\n');
        }
        base += p.mesh.positions().len();
    }
    out
}

/// **PLY binário little-endian** — cor por vértice como padrão do formato.
///
/// ⚠️ **Uma lista de faces de tamanho LIVRE** (`property list uchar uint`), então
/// um quad sai quad; só as peças se perdem, porque o PLY tem um único elemento
/// `vertex` e um único `face`.
pub fn write_ply(pieces: &[ExportPiece<'_>]) -> Vec<u8> {
    let (nv, nf) = totals(pieces);
    let mut out = Vec::with_capacity(84 + nv * 15 + nf * 17);
    out.extend_from_slice(
        format!(
            "ply\nformat binary_little_endian 1.0\ncomment PH2D Sculpt\n\
             element vertex {nv}\n\
             property float x\nproperty float y\nproperty float z\n\
             property uchar red\nproperty uchar green\nproperty uchar blue\n\
             element face {nf}\n\
             property list uchar uint vertex_indices\n\
             end_header\n"
        )
        .as_bytes(),
    );
    for p in pieces {
        for (v, pos) in p.mesh.positions().iter().enumerate() {
            let w = p.pose.point_to_world(*pos);
            for c in w {
                out.extend_from_slice(&c.to_le_bytes());
            }
            // ⚠️ `clamp` ANTES do cast: em Rust um `as u8` satura, mas uma cor
            // fora de [0,1] indica que alguém escreveu HDR num canal que o
            // formato define como 8 bits — o clamp é o lugar onde isso vira um
            // número honesto em vez de um wrap.
            for ch in colour_at(p.mesh, v) {
                out.push((ch.clamp(0.0, 1.0) * 255.0).round() as u8);
            }
        }
    }
    let mut base = 0u32;
    for p in pieces {
        for f in p.mesh.faces() {
            let vs = f.verts();
            out.push(vs.len() as u8);
            for &idx in vs {
                out.extend_from_slice(&(base + idx).to_le_bytes());
            }
        }
        base += p.mesh.positions().len() as u32;
    }
    out
}

/// **STL binário** — triângulos SOLTOS, sem índices, sem cor, sem peças.
///
/// ⚠️ **Cada triângulo repete os três vértices**, que é o formato, não uma
/// escolha nossa: um STL de uma malha indexada custa ~6× o PLY equivalente. É o
/// preço de falar com uma impressora.
pub fn write_stl(pieces: &[ExportPiece<'_>]) -> Vec<u8> {
    let mut tris: Vec<[[f32; 3]; 3]> = Vec::new();
    let mut scratch = Vec::new();
    for p in pieces {
        for f in p.mesh.faces() {
            scratch.clear();
            f.triangles(&mut scratch);
            for t in &scratch {
                tris.push([
                    p.pose.point_to_world(p.mesh.positions()[t[0] as usize]),
                    p.pose.point_to_world(p.mesh.positions()[t[1] as usize]),
                    p.pose.point_to_world(p.mesh.positions()[t[2] as usize]),
                ]);
            }
        }
    }
    let mut out = Vec::with_capacity(84 + tris.len() * 50);
    out.extend_from_slice(&[0u8; 80]); // cabeçalho livre, e um STL binário que
    // comece com "solid" é lido como ASCII por alguns leitores — zeros são a
    // escolha segura.
    out.extend_from_slice(&(tris.len() as u32).to_le_bytes());
    for t in &tris {
        for c in face_normal(t) {
            out.extend_from_slice(&c.to_le_bytes());
        }
        for v in t {
            for c in *v {
                out.extend_from_slice(&c.to_le_bytes());
            }
        }
        out.extend_from_slice(&0u16.to_le_bytes());
    }
    out
}

/// A normal de um triângulo, normalizada; `[0,0,0]` num degenerado.
///
/// ⚠️ **Zero é o valor CERTO para um triângulo degenerado**, e é o que a spec do
/// STL prescreve: o leitor deve então derivar a normal da regra da mão direita.
/// Normalizar um vetor nulo daria `NaN`, que atravessa o arquivo e reaparece
/// como geometria ausente três programas adiante.
fn face_normal(t: &[[f32; 3]; 3]) -> [f32; 3] {
    let u = [t[1][0] - t[0][0], t[1][1] - t[0][1], t[1][2] - t[0][2]];
    let v = [t[2][0] - t[0][0], t[2][1] - t[0][1], t[2][2] - t[0][2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len > 0.0 {
        [n[0] / len, n[1] / len, n[2] / len]
    } else {
        [0.0; 3]
    }
}

/// Quantos triângulos a cena tem — a contagem que o STL escreve.
///
/// Pública porque o gate e o log do produto perguntam a mesma coisa, e uma
/// segunda contagem divergiria do arquivo.
pub fn triangle_count(pieces: &[ExportPiece<'_>]) -> usize {
    pieces
        .iter()
        .flat_map(|p| p.mesh.faces())
        .map(|f| if f.0[3] == TRI { 1 } else { 2 })
        .sum()
}

#[cfg(test)]
#[path = "export_tests.rs"]
mod tests;

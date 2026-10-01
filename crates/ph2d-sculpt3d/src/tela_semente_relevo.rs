//! ⭐⭐⭐ **O RELEVO DA PEÇA visto da vista** — a semente de relevo com que a
//! tela do Painter começa uma pincelada que molda a espessura (`docs/3D/29` §6).
//!
//! # Porque existe (report do dono, 01/10)
//!
//! *«smooth, knife e outras tools não funcionam no relevo»*: a tela começava
//! cada pincelada com o RETRATO da cor ([`crate::tela_semente`]) e com o
//! relevo VAZIO, logo o alisar, a faca e os verbos de esculpir do Painter
//! trabalhavam sobre uma tela lisa — e a espessura que já estava na peça não
//! existia para eles. Este é o retrato da espessura, na unidade que a tela do
//! Painter usa (PÍXEIS), e do corpo.
//!
//! # A lei, por píxel
//!
//! A MESMA rasterização do retrato de cor ([`crate::tela_semente::rasteriza_com`]
//! — de frente, profundidade por `1/w`, baricêntricas corrigidas pela
//! perspectiva), e o par lê-se pela porta do plano
//! ([`Tinta::espessura_tri`] · [`Tinta::espessura_quad`]). A altura passa de
//! unidades de objecto a píxeis dividindo pelo tamanho de um píxel NAQUELE
//! ponto ([`Vista::mundo_por_pixel`]), interpolado dos cantos — a inversa da
//! conversão que a pousada faz.
//!
//! ⚠️ **Ela só precisa de ser CONSISTENTE, não exacta**, pela razão do retrato
//! de cor: a lei da pousada é a DIFERENÇA entre a tela e esta semente, e quem
//! semeia guarda o que a tela DEVOLVE depois de semeada.

use ph2d_mesh::Mesh;
use ph2d_mesh_colors::Tinta;

use crate::tela_na_malha::{Vista, de_frente};
use crate::tela_semente::rasteriza_com;

/// O relevo de uma peça numa vista — a tela inteira, por píxel.
pub struct RelevoDaPeca {
    /// A espessura em PÍXEIS (a unidade da tela do Painter).
    pub px: Vec<f32>,
    /// O corpo, `0..1`.
    pub corpo: Vec<f32>,
}

/// ⭐ **O relevo de `mesh` visto por `vista`** — `None` quando o plano não tem
/// relevo (não há o que semear, e a tela começa lisa como antes).
#[must_use]
pub fn semente_relevo(mesh: &Mesh, tinta: &Tinta, vista: &Vista) -> Option<RelevoDaPeca> {
    if !tinta.tem_relevo() {
        return None;
    }
    let (w, h) = vista.tamanho();
    let n = (w as usize) * (h as usize);
    let mut out = RelevoDaPeca {
        px: vec![0.0; n],
        corpo: vec![0.0; n],
    };
    let mut perto = vec![0.0f32; n];
    let pos = mesh.positions();
    let projectados: Vec<Option<([f32; 2], f32)>> =
        pos.iter().map(|&p| vista.ecra_e_inverso(p)).collect();
    // O tamanho de um píxel em cada vértice — calculado só para os que uma
    // face de frente usa, e uma vez.
    let mut wpp: Vec<Option<f32>> = vec![None; pos.len()];
    for (fi, face) in mesh.faces().iter().enumerate() {
        let cantos = face.verts();
        if !de_frente(pos, cantos, vista.olho()) {
            continue;
        }
        for &v in cantos {
            if wpp[v as usize].is_none() {
                wpp[v as usize] = vista.mundo_por_pixel(pos[v as usize]);
            }
        }
        let tris: &[[usize; 3]] = if cantos.len() == 3 {
            &[[0, 1, 2]]
        } else {
            &[[0, 1, 2], [0, 2, 3]]
        };
        for t in tris {
            let Some(ps) = t
                .iter()
                .map(|&c| projectados[cantos[c] as usize])
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            let Some(ws) = t
                .iter()
                .map(|&c| wpp[cantos[c] as usize])
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            rasteriza_com(&ps, (w, h), &mut perto, |o, bar| {
                let e = if cantos.len() == 3 {
                    tinta.espessura_tri(fi, cantos, bar)
                } else {
                    // O uv de cada canto do quad, e o do ponto por baricêntricas
                    // — a conversão do retrato de cor.
                    const UV: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
                    let uv = [
                        bar[0] * UV[t[0]][0] + bar[1] * UV[t[1]][0] + bar[2] * UV[t[2]][0],
                        bar[0] * UV[t[0]][1] + bar[1] * UV[t[1]][1] + bar[2] * UV[t[2]][1],
                    ];
                    tinta.espessura_quad(fi, cantos, uv)
                };
                let m = bar[0] * ws[0] + bar[1] * ws[1] + bar[2] * ws[2];
                out.px[o] = if m > 0.0 { e[0] / m } else { 0.0 };
                out.corpo[o] = e[1].clamp(0.0, 1.0);
            });
        }
    }
    Some(out)
}

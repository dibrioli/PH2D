//! ⭐⭐ **ONDE CADA PÍXEL ESTAVA** — o mapa que leva a tinta molhada de uma
//! vista para a outra (report do dono, 29/09: *«rotacionar e pintar em
//! seguida está pausando a simulação»*).
//!
//! A água da tinta molhada vive nos píxeis da tela do Painter, e a tela é o
//! ECRÃ. Quando o artista roda a peça e dá o traço seguinte, a tela passa a
//! mostrar a peça doutro lado — e a água do traço de antes está nos píxeis de
//! ANTES. Este módulo responde, píxel a píxel da vista NOVA, **que píxel da
//! vista de antes via o mesmo ponto da superfície** — e `None` onde a vista de
//! antes não o via (estava de costas, tapado, ou fora da tela). O Painter usa
//! a resposta para levar a água consigo
//! (`PainterTool::reproject_screen_canvas`), e ela continua a correr.
//!
//! # A lei, por píxel
//!
//! O mesmo rasterizador do retrato ([`crate::tela_semente`]): na vista nova,
//! cada face de FRENTE cobre os centros de píxel com as baricêntricas
//! corrigidas pela perspectiva, e o ponto da superfície é a combinação dos
//! cantos. Esse ponto é projectado na vista de antes, e conta se ela o VIA:
//! o ponto cai dentro da tela e é a superfície mais perto nesse píxel (o `1/w`
//! dele não perde para o que a vista de antes desenhou ali por mais do que
//! [`FOLGA_DE_PROFUNDIDADE`]).
//!
//! ⚠️ **Não há um teste «a face estava de frente para o olho de antes»** ao
//! lado deste: o mapa de profundidade de antes só tem faces de frente, logo um
//! ponto de uma face de costas ou não tem nada no píxel dele (`None`) ou perde
//! para a superfície da frente que o tapa. Uma segunda cerca que não muda
//! nenhuma resposta seria uma linha que nenhuma prova consegue matar.
//!
//! ⚠️ **A folga é relativa e declarada.** Dentro de um píxel a profundidade de
//! uma face inclinada varia, e o ponto de um píxel da vista nova não cai no
//! centro de um píxel da de antes; sem folga, a própria superfície perderia
//! para si mesma. Uma folga maior deixaria uma superfície por trás de outra
//! MUITO próxima passar como vista; a de `2e-3` do `1/w` é `~2 mm` a `1 m` da
//! câmera. Perto da silhueta, onde a inclinação é rasante, a folga pode não
//! chegar — ali o píxel lê-se «não visto» e a água desse ponto não viaja, que
//! é a resposta conservadora (a tinta que já pousou fica na peça na mesma).

use ph2d_mesh::Mesh;

use crate::tela_na_malha::{Vista, de_frente};
use crate::tela_semente::rasteriza_com;

/// A folga relativa do teste de profundidade da vista de antes (ver o
/// cabeçalho do módulo).
pub const FOLGA_DE_PROFUNDIDADE: f32 = 2e-3;

/// ⭐ **Por píxel da vista `nova`, o ponto CONTÍNUO da tela da vista `antiga`
/// que vê o mesmo sítio da superfície** — `None` onde a vista antiga não o
/// via. Tem `largura × altura` da vista nova, na ordem das linhas.
#[must_use]
pub fn origem(mesh: &Mesh, nova: &Vista, antiga: &Vista) -> Vec<Option<[f32; 2]>> {
    let (w, h) = nova.tamanho();
    let (wa, ha) = antiga.tamanho();
    let pos = mesh.positions();
    // 1. O que a vista de antes via, píxel a píxel (o `1/w` mais perto).
    let perto_antes = profundidade(mesh, antiga);
    // 2. Na vista nova, o ponto da superfície de cada píxel, levado à de antes.
    let mut perto = vec![0.0f32; w as usize * h as usize];
    let mut saida: Vec<Option<[f32; 2]>> = vec![None; w as usize * h as usize];
    let projectados: Vec<Option<([f32; 2], f32)>> =
        pos.iter().map(|&p| nova.ecra_e_inverso(p)).collect();
    for face in mesh.faces() {
        let cantos = face.verts();
        if !de_frente(pos, cantos, nova.olho()) {
            continue;
        }
        for t in triangulos(cantos.len()) {
            let Some(ps) = t
                .iter()
                .map(|&c| projectados[cantos[c] as usize])
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            rasteriza_com(&ps, (w, h), &mut perto, |o, bar| {
                let mut p = [0.0f32; 3];
                for (k, &c) in t.iter().enumerate() {
                    let v = pos[cantos[c] as usize];
                    for e in 0..3 {
                        p[e] += v[e] * bar[k];
                    }
                }
                saida[o] = na_vista(antiga, (wa, ha), &perto_antes, p);
            });
        }
    }
    saida
}

/// O ponto `p` visto pela vista `antiga`, ou `None` se ela não o via.
fn na_vista(antiga: &Vista, (wa, ha): (u32, u32), perto: &[f32], p: [f32; 3]) -> Option<[f32; 2]> {
    let (q, inv) = antiga.ecra_e_inverso(p)?;
    if !(q[0] >= 0.0 && q[1] >= 0.0 && q[0] < wa as f32 && q[1] < ha as f32) {
        return None;
    }
    let o = q[1] as usize * wa as usize + q[0] as usize;
    let ali = perto[o];
    (ali > 0.0 && inv >= ali * (1.0 - FOLGA_DE_PROFUNDIDADE)).then_some(q)
}

/// O `1/w` da superfície mais perto em cada píxel de `vista` (`0` onde não há
/// nenhuma) — o mesmo teste de profundidade do retrato, sem a cor.
fn profundidade(mesh: &Mesh, vista: &Vista) -> Vec<f32> {
    let (w, h) = vista.tamanho();
    let pos = mesh.positions();
    let mut perto = vec![0.0f32; w as usize * h as usize];
    let projectados: Vec<Option<([f32; 2], f32)>> =
        pos.iter().map(|&p| vista.ecra_e_inverso(p)).collect();
    for face in mesh.faces() {
        let cantos = face.verts();
        if !de_frente(pos, cantos, vista.olho()) {
            continue;
        }
        for t in triangulos(cantos.len()) {
            let Some(ps) = t
                .iter()
                .map(|&c| projectados[cantos[c] as usize])
                .collect::<Option<Vec<_>>>()
            else {
                continue;
            };
            rasteriza_com(&ps, (w, h), &mut perto, |_, _| {});
        }
    }
    perto
}

/// Os triângulos de uma face (um quad parte-se em `(0,1,2)` e `(0,2,3)` — a
/// mesma partição do retrato).
fn triangulos(n: usize) -> &'static [[usize; 3]] {
    if n == 3 {
        &[[0, 1, 2]]
    } else {
        &[[0, 1, 2], [0, 2, 3]]
    }
}

#[cfg(test)]
#[path = "tela_origem_tests.rs"]
mod tests;

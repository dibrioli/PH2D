//! ⭐ **A FORÇA POR PASSO DAS ÂNCORAS É ZERADA EM TODA A MALHA** (espec §4.3).
//!
//! ⚠️ **Este ficheiro existe porque a lei sobreviveu a uma mutação.** Apagar o
//! zeramento não mexeu em NENHUM dos 50 traços do oráculo, e a razão é
//! estrutural: a relaxação não filtra por vértice activo — quem apaga a
//! contribuição de um vértice longe é o `φ`, que já é zero fora da banda. As
//! duas leis tapam-se uma à outra em todo o corpus.
//!
//! ⚠️ **Mas elas medem coisas DIFERENTES, e o caso que as separa existe:** a
//! área é decidida sobre as posições **ACTUAIS** e o `φ` sobre as de
//! **REPOUSO**. Um vértice bastante deformado pode sair da área (posição actual
//! longe do cursor) e continuar com `φ > 0` (repouso perto) — e aí a marca
//! velha dele seria aplicada outra vez, contra a espec. *Um corpus que nunca
//! deforma o bastante para separar duas leis não testa nenhuma das duas.*

use crate::V3;
use crate::verlet_gesto::{Area, Modo, Passo, Pincel, PincelTecido};

/// Uma grelha `n × n` no plano `z = 0`, com passo `h`.
fn grelha(n: usize, h: f64) -> (Vec<V3>, Vec<Vec<u32>>) {
    let meio = (n - 1) as f64 * h * 0.5;
    let mut p = Vec::with_capacity(n * n);
    for j in 0..n {
        for i in 0..n {
            p.push([i as f64 * h - meio, j as f64 * h - meio, 0.0]);
        }
    }
    let idx = |i: usize, j: usize| u32::try_from(j * n + i).unwrap_or(u32::MAX);
    let mut faces = Vec::new();
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            faces.push(vec![
                idx(i, j),
                idx(i + 1, j),
                idx(i + 1, j + 1),
                idx(i, j + 1),
            ]);
        }
    }
    (p, faces)
}

fn aneis(n: usize, faces: &[Vec<u32>]) -> Vec<Vec<u32>> {
    let mut a = vec![Vec::new(); n];
    for f in faces {
        for k in 0..f.len() {
            let (p, q) = (f[k] as usize, f[(k + 1) % f.len()] as usize);
            a[p].push(u32::try_from(q).unwrap_or(u32::MAX));
            a[q].push(u32::try_from(p).unwrap_or(u32::MAX));
        }
    }
    for l in &mut a {
        l.sort_unstable();
        l.dedup();
    }
    a
}

/// **GATE — nenhum vértice guarda a força por passo de um passo anterior.**
///
/// O arnês monta o caso que o corpus não tem: área *Dynamic* (o disco anda com
/// o cursor), modo de âncora, e um traço longo o bastante para que vértices
/// puxados no início fiquem para trás. Depois do último passo, todo vértice
/// fora do disco DESTE passo tem de ter `σ = 0`.
///
/// ⚠️ **A régua é a espec, não a nossa aritmética:** `σ` é *«a força por passo,
/// zerada em todo o objecto antes de ser reescrita»*, então o que se afirma é
/// uma propriedade do estado no fim do passo, e não um número.
#[test]
fn nenhum_vertice_guarda_a_forca_por_passo_de_um_passo_anterior() {
    for modo in [Modo::Agarrar, Modo::Gancho] {
        let (rest, faces) = grelha(41, 0.05);
        let an = aneis(rest.len(), &faces);
        let anel = |v: u32| an[v as usize].clone();
        let pincel = Pincel {
            modo,
            area: Area::Dinamica,
            raio: 0.20,
            ..Pincel::default()
        };
        let r = pincel.raio;
        let mut pos = rest.clone();
        let inicio = [-0.5, 0.0, 0.0];
        let mut tecido = PincelTecido::pen_down(pincel, &pos, inicio);
        let mut cursor = inicio;
        let normais = vec![[0.0, 0.0, 1.0]; rest.len()];
        for k in 0..24 {
            let delta = if k == 0 { [0.0; 3] } else { [0.05, 0.0, 0.10] };
            cursor = [
                cursor[0] + delta[0],
                cursor[1] + delta[1],
                cursor[2] + delta[2],
            ];
            let passo = Passo {
                cursor,
                delta,
                parado: k == 0,
                normal_area: [0.0, 0.0, 1.0],
                normais: &normais,
                pressao: 1.0,
            };
            if tecido.passo(&pos, &anel, &passo) {
                for (v, act) in tecido.sim.activo.iter().enumerate() {
                    if *act {
                        pos[v] = tecido.sim.x[v];
                    }
                }
            }
        }
        // Controlo anti-vácuo: o traço tem de ter DEFORMADO, senão «σ = 0 em
        // toda a parte» é verdade num pincel morto.
        let movidos = pos
            .iter()
            .zip(&rest)
            .filter(|(a, b)| crate::verlet::norm([a[0] - b[0], a[1] - b[1], a[2] - b[2]]) > 1e-9)
            .count();
        assert!(movidos > 100, "{modo:?}: só {movidos} movidos — vácuo");
        // E tem de haver quem esteja FORA do disco deste passo com σ escrito
        // num passo anterior, senão o gate não olha para nada.
        let mut fora = 0usize;
        for (v, p) in pos.iter().enumerate() {
            let d = crate::verlet::norm([p[0] - cursor[0], p[1] - cursor[1], p[2] - cursor[2]]);
            if d >= r {
                fora += 1;
                assert!(
                    tecido.sim.sigma[v] == 0.0,
                    "{modo:?}: vertice {v} esta a {d:.4} do cursor (raio {r}) e guarda \
                     forca por passo {} -- a espec §4.3 manda zerar em TODO o objecto",
                    tecido.sim.sigma[v]
                );
            }
        }
        assert!(
            fora > 100,
            "{modo:?}: só {fora} vértices fora do disco — vácuo"
        );
    }
}

#![forbid(unsafe_code)]
//! ⭐⭐ **O CONTACTO ENTRE PEÇAS** — doc 109, ordem do dono (2026-09-13): *«colidem sozinhas»*.
//!
//! Discos que se sobrepõem são afastados até apenas se tocarem. É a lei do `motion.collide`
//! (restrição de não-penetração do *Position Based Dynamics*, Müller et al. 2007), com o mesmo
//! **Jacobi com média** que o tornou independente da ordem do stream (Macklin & Müller, *Unified
//! Particle Physics*, 2014): cada varredura lê UMA fotografia das posições, cada disco soma o que
//! os contactos dele pedem, e aplica a MÉDIA.
//!
//! ## Porque é uma folha
//!
//! Dois integradores a pedem — o `sim.step` (W2) e o `motion.integrate` (W4). Copiar a lei seria a
//! segunda resposta à mesma pergunta; um nó a depender de outro nó quebraria o isolamento.
//!
//! ## A grelha dá os MESMOS BITS que todos-os-pares, e isso é uma escolha
//!
//! O `motion.collide` de CPU é `O(n² · varreduras)`. Aqui cada disco procura parceiros só nas 9
//! células vizinhas de uma grelha de lado `2 · r_max` — um parceiro a `r_i + r_j ≤ 2 · r_max` nunca
//! está mais longe que uma célula.
//!
//! ⚠️ **E os parceiros são somados em ordem CRESCENTE de índice**, porque é essa a ordem em que o
//! laço de todos-os-pares (`i < j`, `i` por fora) entrega as contribuições a um disco: primeiro as
//! dos pares `(i, k)` com `i < k`, depois as dos `(k, j)`. Um par visto do outro lado dá os mesmos
//! bits — `b − a = −(a − b)` e a divisão pelo mesmo `d` são exactas em IEEE-754, e a soma e o
//! produto dos pesos são comutativos. ⇒ o gate [`tests`] exige **igualdade ao bit** com a
//! referência, e não uma tolerância que esconderia uma ordem trocada.
//!
//! ## O que conta como peça
//!
//! Um elemento só entra com **raio finito e positivo e posição finita**. Os outros não empurram nem
//! são empurrados — é a coluna `collider` ausente vista por dentro. Um peso `w = 0` (o `inv_mass` do
//! `motion.pin_constraint`) é um **obstáculo**: não se move e os outros contornam-no.

use std::collections::BTreeMap;

use ph2d_nodegraph::attr::par_build;

/// Abaixo disto dois centros coincidem e a normal não existe (o `EPS` do `motion.collide`).
const EPS: f32 = 1e-9;

/// Afasta os discos sobrepostos, `varreduras` vezes. `p` é reescrito no sítio.
///
/// # Panics
///
/// Se `raios` ou `pesos` não tiverem o comprimento de `p` — três colunas de uma mesma corrente
/// com comprimentos diferentes não são uma pergunta com resposta.
pub fn separate(p: &mut [[f32; 2]], raios: &[f32], pesos: &[f32], varreduras: usize) {
    let n = p.len();
    assert_eq!(raios.len(), n, "um raio por peca");
    assert_eq!(pesos.len(), n, "um peso por peca");
    let ativo: Vec<bool> = (0..n).map(|i| ativo(p[i], raios[i])).collect();
    let r_max = (0..n)
        .filter(|&i| ativo[i])
        .map(|i| raios[i])
        .fold(0.0_f32, f32::max);
    if r_max <= 0.0 {
        return;
    }
    let lado = 2.0 * r_max;
    for _ in 0..varreduras {
        let foto = p.to_vec();
        let grelha = grelha(&foto, &ativo, lado);
        let novas: Vec<Option<[f32; 2]>> = par_build(n, |k| {
            if !ativo[k] {
                return None;
            }
            let (cx, cy) = celula(foto[k], lado);
            let mut parceiros: Vec<usize> = Vec::new();
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if let Some(v) = grelha.get(&(cx + dx, cy + dy)) {
                        parceiros.extend_from_slice(v);
                    }
                }
            }
            parceiros.sort_unstable();
            corrigida(k, parceiros.into_iter(), &foto, raios, pesos, &ativo)
        });
        for (k, nova) in novas.into_iter().enumerate() {
            if let Some(q) = nova {
                p[k] = q;
            }
        }
    }
}

/// **A referência**: a mesma lei por todos-os-pares, na ordem do laço `i < j`. `O(n²)`.
///
/// Pública para que o gate de cada cliente possa comparar-se com ela; nenhum caminho de produto a
/// chama.
pub fn separate_all_pairs(p: &mut [[f32; 2]], raios: &[f32], pesos: &[f32], varreduras: usize) {
    let n = p.len();
    assert_eq!(raios.len(), n, "um raio por peca");
    assert_eq!(pesos.len(), n, "um peso por peca");
    let ativo: Vec<bool> = (0..n).map(|i| ativo(p[i], raios[i])).collect();
    for _ in 0..varreduras {
        let foto = p.to_vec();
        let novas: Vec<Option<[f32; 2]>> = (0..n)
            .map(|k| {
                if ativo[k] {
                    corrigida(k, 0..n, &foto, raios, pesos, &ativo)
                } else {
                    None
                }
            })
            .collect();
        for (k, nova) in novas.into_iter().enumerate() {
            if let Some(q) = nova {
                p[k] = q;
            }
        }
    }
}

fn ativo(p: [f32; 2], r: f32) -> bool {
    r.is_finite() && r > 0.0 && p[0].is_finite() && p[1].is_finite()
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "uma celula de grelha; uma coordenada fora de i64 satura, e so' agrupa pior"
)]
fn celula(p: [f32; 2], lado: f32) -> (i64, i64) {
    ((p[0] / lado).floor() as i64, (p[1] / lado).floor() as i64)
}

/// As peças activas por célula, cada lista em ordem crescente de índice.
fn grelha(foto: &[[f32; 2]], ativo: &[bool], lado: f32) -> BTreeMap<(i64, i64), Vec<usize>> {
    let mut g: BTreeMap<(i64, i64), Vec<usize>> = BTreeMap::new();
    for (i, q) in foto.iter().enumerate() {
        if ativo[i] {
            g.entry(celula(*q, lado)).or_default().push(i);
        }
    }
    g
}

/// A posição de `k` depois desta varredura, ou `None` se nada lhe tocou. Os `parceiros` têm de vir
/// em ordem CRESCENTE — ver o cabeçalho.
fn corrigida(
    k: usize,
    parceiros: impl Iterator<Item = usize>,
    foto: &[[f32; 2]],
    raios: &[f32],
    pesos: &[f32],
    ativo: &[bool],
) -> Option<[f32; 2]> {
    let mut delta = [0.0_f32; 2];
    let mut contactos = 0_u32;
    for j in parceiros {
        if j == k || !ativo[j] {
            continue;
        }
        // Dois obstáculos (ou dois pesos infinitos) não têm correcção a repartir.
        let soma_w = pesos[k] + pesos[j];
        if soma_w <= 0.0 {
            continue;
        }
        let min_dist = raios[k] + raios[j];
        let min_d2 = min_dist * min_dist;
        let dx = foto[j][0] - foto[k][0];
        let dy = foto[j][1] - foto[k][1];
        let d2 = dx * dx + dy * dy;
        if d2 >= min_d2 {
            continue;
        }
        let (nx, ny, penetracao) = if d2 > EPS {
            let d = d2.sqrt();
            (dx / d, dy / d, min_dist - d)
        } else {
            // Centros coincidentes: o eixo sai do PAR (o do `motion.collide`), e a normal aponta
            // do índice menor para o maior — visto de qualquer dos dois lados.
            let (lo, hi) = (k.min(j), k.max(j));
            let (ax, ay) = if (lo + hi) % 2 == 0 {
                (1.0, 0.0)
            } else {
                (0.0, 1.0)
            };
            if k == lo {
                (ax, ay, min_dist)
            } else {
                (-ax, -ay, min_dist)
            }
        };
        // A parte da penetração que cabe a `k`: `w_k / (w_k + w_j)`.
        let empurra = penetracao * (pesos[k] / soma_w);
        delta[0] -= nx * empurra;
        delta[1] -= ny * empurra;
        contactos += 1;
    }
    (contactos > 0).then(|| {
        #[expect(
            clippy::cast_precision_loss,
            reason = "uma contagem de contactos de um disco, muito abaixo de 2^24"
        )]
        let inv = 1.0 / contactos as f32;
        [foto[k][0] + delta[0] * inv, foto[k][1] + delta[1] * inv]
    })
}

#[cfg(test)]
mod tests;

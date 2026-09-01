//! ⭐⭐⭐ **SOLDAR** (plano 39) — linhas cruzadas passam a partilhar o nó.
//!
//! Ideia do Enio (2026-08-31): *"e se pudéssemos soldar linhas cruzadas? Ou seja: linhas cruzadas
//! compartilham o mesmo nó de modo que criem várias áreas fechadas interligadas?"*
//!
//! # A lei
//!
//! > **Cada contorno parte-se em ARCOS nos pontos onde encontra os outros** — e as pontas dos arcos
//! > vizinhos caem exactamente no mesmo sítio, porque saem do mesmo cruzamento.
//!
//! O grafo não é uma estrutura de dados: ele é **implícito nas coordenadas coincidentes**. É o
//! modelo do desenho de CAD (um esboço do Fusion é uma rede desde sempre, e é por isso que o Trim
//! de lá parece natural) e a metade barata do *vector network* da Figma.
//!
//! # ⛔ O que isto NÃO é
//!
//! **Não é o `Pathfinder > Divide` do Illustrator.** Aquele *"perde as partes de caminhos abertos
//! que ficam de fora"* — é a queixa documentada dele. Aqui **todo arco sobrevive**, incluindo o toco
//! que sobra para fora: o que sai é a rede inteira, não só as faces.
//!
//! **Não é a rede da Figma.** Lá um ponto é um nó de um multigrafo e sobrevive à edição; aqui a
//! soldadura é um **acto**, e arrastar um nó depois separa outra vez as duas pontas. ⚠️ Manter a
//! rede soldada durante a edição é modelo novo — a Figma conta que refazer o tipo *caminho* teve
//! *"becos sem saída"* e que quase desistiram. **Decisão do Enio: soldar CONSOME os traços.**
//!
//! # ⛔ E não é automático
//!
//! Se cruzar duas linhas as colasse sozinho, seria impossível apenas **sobrepor** dois traços. O
//! gesto é um verbo explícito sobre a selecção.

use crate::VecVertex;

/// Duas fracções mais próximas que isto são o mesmo corte.
const EPS: f64 = 1e-9;

/// **OS ARCOS em que este contorno se parte** nos `cruzamentos` dados (fracções de arco `0..=1`,
/// em qualquer ordem).
///
/// Sem cruzamento nenhum devolve o contorno **intacto** — e intacto quer dizer os mesmos vértices,
/// não uma reconstrução: um caminho que não encontra ninguém não pode pagar o custo de ser cortado
/// e recosturado.
///
/// | o contorno | os cruzamentos | o que sai |
/// |---|---|---|
/// | qualquer | nenhum | ele próprio, intacto |
/// | aberto | `n` | `n + 1` arcos abertos |
/// | fechado | `n` | `n` arcos abertos (o último dá a volta pela emenda) |
///
/// ⚠️ **Um fechado com UM cruzamento vira UM arco aberto** — o anel é cortado num ponto e passa a
/// ter duas pontas, que caem no mesmo sítio. Não é degenerado: é um anel aberto.
#[must_use]
pub fn split_at(
    verts: &[VecVertex],
    closed: bool,
    cruzamentos: &[f64],
) -> Vec<(Vec<VecVertex>, bool)> {
    let mut cortes: Vec<f64> = cruzamentos
        .iter()
        .copied()
        .filter(|f| f.is_finite() && *f > EPS && *f < 1.0 - EPS)
        .collect();
    cortes.sort_by(f64::total_cmp);
    cortes.dedup_by(|a, b| (*a - *b).abs() < EPS);
    if cortes.is_empty() {
        return vec![(verts.to_vec(), closed)];
    }
    // As FRONTEIRAS dos arcos. Num aberto as duas pontas entram; num fechado a lista fecha-se
    // sobre si mesma e o último arco dá a volta pela emenda.
    let mut arcos = Vec::with_capacity(cortes.len() + 1);
    let mut emitir = |de: f64, ate: f64| {
        if let Some(v) = crate::trim_tool::piece_geometry(verts, closed, de, ate) {
            arcos.push((v, false));
        }
    };
    if closed {
        for w in 0..cortes.len() {
            emitir(cortes[w], cortes[(w + 1) % cortes.len()]);
        }
    } else {
        let mut anterior = 0.0;
        for &c in &cortes {
            emitir(anterior, c);
            anterior = c;
        }
        emitir(anterior, 1.0);
    }
    arcos
}

/// ⭐⭐⭐ **FUNDE as pontas que caem quase no mesmo sítio** — cada aglomerado passa a ter **UMA**
/// coordenada, bit a bit igual em todos os arcos que o partilham.
///
/// # Porque não basta cortar
///
/// Report do Enio (2026-08-31, com foto): *"weld dividiu e não soldou (eu que afastei os pontos)"*.
/// E ele estava certo — **cortar não é soldar**. As duas metades de um cruzamento nascem de
/// contornos DIFERENTES: cada um converte a mesma travessia para a **sua** fracção de arco e depois
/// avalia a **sua** cúbica ali. Os dois pontos ficam perto e **não iguais** — e dois pontos perto
/// não são um nó, são dois nós.
///
/// ⚠️ **A tolerância é o erro de AMOSTRAGEM** (`trim_tool::sampling_error`), a mesma régua que diz
/// se uma ponta está *sobre* uma curva. Um número escolhido colaria pontas que o artista quis
/// separadas, ou deixaria de colar as que ele quis juntas.
///
/// ⛔ Só PONTAS: um vértice interior de um arco não é junta de nada.
pub fn fuse_endpoints(arcos: &mut [(Vec<VecVertex>, bool)], tol: f64) -> usize {
    // Onde está cada ponta: `(índice do arco, índice do vértice)`.
    let mut pontas: Vec<(usize, usize)> = Vec::new();
    for (i, (verts, closed)) in arcos.iter().enumerate() {
        if *closed || verts.len() < 2 {
            continue;
        }
        pontas.push((i, 0));
        pontas.push((i, verts.len() - 1));
    }
    let t2 = tol * tol;
    let mut visto = vec![false; pontas.len()];
    let mut fundidos = 0usize;
    for a in 0..pontas.len() {
        if visto[a] {
            continue;
        }
        let pa = arcos[pontas[a].0].0[pontas[a].1].anchor;
        let mut grupo = vec![a];
        for (b, vb) in visto.iter().enumerate().skip(a + 1) {
            if *vb {
                continue;
            }
            let pb = arcos[pontas[b].0].0[pontas[b].1].anchor;
            if (pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) <= t2 {
                grupo.push(b);
            }
        }
        for &g in &grupo {
            visto[g] = true;
        }
        if grupo.len() < 2 {
            continue; // uma ponta sozinha não é junta
        }
        // ⚠️ **O CENTROIDE, e não a primeira**: escolher uma das pontas faria a junta depender da
        // ordem em que os arcos saíram, e a mesma solda dava geometria diferente conforme a ordem
        // da selecção.
        let (mut sx, mut sy) = (0.0, 0.0);
        for &g in &grupo {
            let p = arcos[pontas[g].0].0[pontas[g].1].anchor;
            sx += p[0];
            sy += p[1];
        }
        #[allow(clippy::cast_precision_loss)]
        let n = grupo.len() as f64;
        let no = [sx / n, sy / n];
        for &g in &grupo {
            let (ai, vi) = pontas[g];
            // ⚠️ **A alça acompanha a âncora** — mover só a âncora mudaria a CURVA em vez de a
            // deslocar, e o arco descolaria da forma que ele tinha.
            let v = &mut arcos[ai].0[vi];
            let d = [no[0] - v.anchor[0], no[1] - v.anchor[1]];
            v.anchor = no;
            v.in_handle = [v.in_handle[0] + d[0], v.in_handle[1] + d[1]];
            v.out_handle = [v.out_handle[0] + d[0], v.out_handle[1] + d[1]];
        }
        fundidos += 1;
    }
    fundidos
}

#[cfg(test)]
#[path = "weld_tests.rs"]
mod tests;

//! ⭐⭐⭐ **O PASSE DE DESEMARANHAMENTO** — desfaz dobras do mapa **depois do arredondamento**,
//! retalho a retalho, com a fronteira do retalho **e as imagens inteiras** presas.
//!
//! ⛔⛔ **A medição que o nomeou** (2026-08-30, peça do artista): o mapa dobra `3,12 %` dos
//! triângulos no **ombro** de um espinho contra `0,14 %` no corpo — `23×` — e a extracção emite
//! isso como face a `177°` e gravata. Plano e literatura:
//! `docs/3D/quad-remesh/PLANO_desdobrar_o_mapa.md`.
//!
//! # ⛔⛔ ONDE ele corre — e a 1.ª redacção disto dizia o CONTRÁRIO, com convicção
//!
//! Ele corre **DEPOIS** da escada de arredondamento, com as **imagens inteiras presas**.
//!
//! ⚠️ **A versão anterior corria ANTES**, e a razão escrita ao lado dela era boa: *«depois seria
//! destruir o trabalho da escada — ela prega imagens em pontos inteiros, e um passe que move
//! vértices continuamente tira-as de lá»*. ⛔ **A medição matou-a:** desemaranhar o mapa contínuo
//! desfaz `62,4 %` das dobras dele (`149 → 56`, `193 ms`) — e a escada **re-dobra**, e re-dobra
//! **MAIS** partindo de um mapa desemaranhado (`149 → 169` dobras no mapa final, `54 → 71` na
//! casca do corpo). *Uma restrição imposta numa fase e não na seguinte não é uma restrição; é um
//! ponto de partida* — a mesma lei que o §23.18 desta crate já tinha escrito, noutro assunto.
//!
//! ⭐ A objecção da 1.ª redacção resolve-se **prendendo os inteiros**, não escolhendo outra fase.
//!
//! # ⚠️ E porque a fronteira do retalho fica PRESA
//!
//! As transições de carta vivem nas fronteiras. Prendê-las deixa-as **intactas** ⇒ a propriedade
//! `GP` — costura por rotação de 90° e translação inteira, que a obra de 24/08 comprou por
//! eliminação de variável — fica preservada **por construção**, e não por promessa.
//!
//! ⛔ **É uma versão RESTRITA**, e as dobras que sobram vivem **na** fronteira. Libertá-las é a
//! wave seguinte — o conjunto reduzido do `ClosureSystem` é exactamente o espaço onde ela cabe.
//!
//! ⛔⛔ **E ele NASCE DESLIGADO** — ver [`enabled`] para a tabela que o decidiu.

use crate::cut::CutMesh;
use crate::solve::GridMap;
use ph2d_mesh::Mesh;
use std::collections::BTreeMap;

/// O que o passe fez.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UntangleReport {
    /// Retalhos que entraram com pelo menos uma dobra.
    pub patches: usize,
    /// Dobras antes.
    pub before: usize,
    /// Dobras depois. ⭐ A diferença é o que o passe comprou.
    pub after: usize,
    /// Retalhos que **não** fecharam — os que sobram têm dobra na fronteira.
    pub gave_up: usize,
}

/// ⛔⛔ **NASCE DESLIGADO, e a razão é a medição pela PORTA.** `PH2D_GRIDMAP_UNTANGLE=1` liga.
///
/// Medido em 2026-08-30 na peça do artista (`Detail 0,85` + `Follow Curvature 1`), com o passe
/// no sítio certo (depois da escada, inteiros presos):
///
/// | coluna | desligado | ⭐ ligado |
/// |---|---|---|
/// | dobras da saída | `27` | ⭐ **`21`** |
/// | aspecto p99 | `1,70` | ⭐ `1,64` |
/// | enviesamento p99 | `33,5°` | ⭐ `30,9°` |
/// | faces `>60°` | `12` | ⭐ **`10`** |
/// | irregulares | `50` | ⭐ `48` |
/// | `χ` · bordo · não-manifold | `2` · `0` · `0` | `2` · `0` · `0` |
/// | ⚠️ gravatas | `1` | ⚠️ **`2`** |
/// | ⚠️ ponta pior | `−6,4 %` | ⚠️ `−6,6 %` |
/// | ⛔ **relógio** | `21,4 s` | ⛔ **`33,4 s`** |
///
/// ⭐ **Todas as colunas de forma melhoram**, e as dobras caem `22 %`. ⛔ **E mesmo assim ele
/// não shipa ligado**, por três razões medidas:
///
/// 1. ⛔ **Custa `+56 %` do relógio** — e o relógio é do artista, a cada clique. ⚠️ *A medição
///    foi tirada com a máquina sob carga de outra linha*, então o número é um tecto, não um
///    facto; mas mesmo o tecto é grande demais para uma melhoria que ele não vê.
/// 2. ⛔ **Não cura a foto.** A torção `p99` do ombro anda `35,7° → 34,8°` e o máximo continua
///    em `179°` — *o defeito que o dono fotografou continua lá*.
/// 3. ⚠️ Duas colunas **pioram** (gravatas `1 → 2`, ponta pior `−6,4 → −6,6 %`).
///
/// ⇒ *Uma melhoria real que o dono não vê, paga com metade de um relógio que ele vê, não é um
/// degrau — é uma troca, e a troca é dele.*
#[must_use]
pub fn enabled() -> bool {
    std::env::var("PH2D_GRIDMAP_UNTANGLE").as_deref() == Ok("1")
}

/// ⭐⭐⭐ **Desemaranha o mapa, no sítio.**
///
/// ⚠️ **O repouso é o triângulo 3D achatado ISOMETRICAMENTE** (`p0` na origem, `p1` no eixo
/// `x`): ele preserva os comprimentos das arestas, então a energia mede a distorção **do mapa** e
/// não a de um achatamento que já distorce.
///
/// ⚠️ **Um retalho sem dobra nenhuma é SALTADO** — não é uma optimização, é uma cerca: correr a
/// descida sobre um mapa já válido mexia-o para baixar a energia, e *este passe não existe para
/// melhorar um mapa bom; existe para desfazer uma dobra.*
pub fn untangle_patches(mesh: &Mesh, cut: &CutMesh, map: &mut GridMap) -> UntangleReport {
    let mut rep = UntangleReport::default();
    let pos = mesh.positions();
    for (p, tris) in cut.tris.iter().enumerate() {
        let (Some(origin), Some(uvp)) = (cut.origin.get(p), map.uv.get(p)) else {
            continue;
        };
        if tris.is_empty() || uvp.is_empty() {
            continue;
        }
        let mut elements = Vec::with_capacity(tris.len());
        for t in tris {
            if let Some(el) = element_of(pos, origin, *t) {
                elements.push(el);
            }
        }
        let mut uv: Vec<[f64; 2]> = uvp
            .iter()
            .map(|c| [f64::from(c[0]), f64::from(c[1])])
            .collect();
        let before = ph2d_untangle::flipped(&elements, &uv);
        if before == 0 {
            continue;
        }
        rep.patches += 1;
        rep.before += before;

        let locked = locked_of(tris, uvp, uvp.len());
        let r = ph2d_untangle::untangle(
            &elements,
            &mut uv,
            &locked,
            ph2d_untangle::Settings::default(),
        );
        rep.after += r.flipped_after;
        if r.gave_up {
            rep.gave_up += 1;
        }
        // ⚠️ **Só se escreve de volta o que MELHOROU.** A descida nunca sobe (a busca linear
        // recusa), mas o retorno a `f32` pode: *um passe que só pode ajudar tem de o provar na
        // régua do consumidor, e a régua dele é `f32`.*
        let out: Vec<[f32; 2]> = uv
            .iter()
            .map(|c| {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "o mapa do consumidor e' f32; a descida corre em f64 e volta"
                )]
                [c[0] as f32, c[1] as f32]
            })
            .collect();
        let back: Vec<[f64; 2]> = out
            .iter()
            .map(|c| [f64::from(c[0]), f64::from(c[1])])
            .collect();
        if ph2d_untangle::flipped(&elements, &back) < before
            && let Some(slot) = map.uv.get_mut(p)
        {
            *slot = out;
        }
    }
    rep
}

/// O elemento de um triângulo, com o repouso achatado isometricamente.
fn element_of(pos: &[[f32; 3]], origin: &[u32], t: [u32; 3]) -> Option<ph2d_untangle::Element> {
    let q: Vec<[f64; 3]> = t
        .iter()
        .map(|&l| {
            let g = *origin.get(l as usize).unwrap_or(&0) as usize;
            let v = pos.get(g).copied().unwrap_or([0.0; 3]);
            [f64::from(v[0]), f64::from(v[1]), f64::from(v[2])]
        })
        .collect();
    let e1 = [q[1][0] - q[0][0], q[1][1] - q[0][1], q[1][2] - q[0][2]];
    let e2 = [q[2][0] - q[0][0], q[2][1] - q[0][1], q[2][2] - q[0][2]];
    let l1 = e1[0]
        .mul_add(e1[0], e1[1].mul_add(e1[1], e1[2] * e1[2]))
        .sqrt();
    if !l1.is_finite() || l1 <= 0.0 {
        return None;
    }
    let u = [e1[0] / l1, e1[1] / l1, e1[2] / l1];
    let x = e2[0].mul_add(u[0], e2[1].mul_add(u[1], e2[2] * u[2]));
    let sq = e2[0].mul_add(e2[0], e2[1].mul_add(e2[1], e2[2] * e2[2])) - x * x;
    let y = if sq > 0.0 { sq.sqrt() } else { 0.0 };
    ph2d_untangle::Element::from_rest(t, [0.0, 0.0], [l1, 0.0], [x, y])
}

/// ⭐⭐ **Tolerância para «esta imagem já é INTEIRA»**, em células.
///
/// ⚠️ A escada prega em inteiros **exactos**; a folga existe só para o retorno a `f32`, e é
/// deliberadamente apertada — *uma folga larga prenderia vértices que a escada nunca pregou.*
const INTEGER_TOL: f32 = 1e-4;

/// Os vértices locais que **não se podem mexer**: a fronteira do retalho **e** as imagens que já
/// são inteiras.
///
/// ⛔⛔ **A segunda metade é o que torna este passe seguro DEPOIS do arredondamento** — e é a
/// única posição em que ele serve para alguma coisa. Medido em 2026-08-30: correndo **antes** da
/// escada, ela re-dobra o mapa e re-dobra **mais** partindo de um mapa desemaranhado
/// (`149 → 169` dobras finais). *Uma restrição imposta numa fase e não na seguinte não é uma
/// restrição; é um ponto de partida* — a mesma lei que o §23.18 desta crate já tinha escrito.
///
/// ⚠️ **Antes do arredondamento esta metade é INERTE** (quase nada é inteiro), então a função
/// serve as duas posições sem um interruptor.
fn locked_of(tris: &[[u32; 3]], uv: &[[f32; 2]], n: usize) -> Vec<bool> {
    let mut count: BTreeMap<(u32, u32), usize> = BTreeMap::new();
    for t in tris {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            *count
                .entry(if a < b { (a, b) } else { (b, a) })
                .or_default() += 1;
        }
    }
    let mut locked = vec![false; n];
    for (e, c) in &count {
        if *c == 1 {
            if let Some(s) = locked.get_mut(e.0 as usize) {
                *s = true;
            }
            if let Some(s) = locked.get_mut(e.1 as usize) {
                *s = true;
            }
        }
    }
    for (i, c) in uv.iter().enumerate() {
        if (c[0] - c[0].round()).abs() < INTEGER_TOL
            && (c[1] - c[1].round()).abs() < INTEGER_TOL
            && let Some(s) = locked.get_mut(i)
        {
            *s = true;
        }
    }
    locked
}

#[cfg(test)]
#[path = "untangle_pass_tests.rs"]
mod tests;

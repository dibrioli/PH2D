//! ⭐⭐⭐ **A INJECTIVIDADE COMO OBJECTIVO DA FASE** — a obra que o `PLANO_desdobrar_o_mapa`
//! §10 nomeou, depois de as três alternativas terem sido medidas e nenhuma chegar.
//!
//! # ⛔ Porque nenhum passe serve, e isto é aritmética
//!
//! Medido em 2026-08-30 na peça do artista (`Detail 0,85`), com a mesma régua dos dois lados:
//!
//! | | dobras |
//! |---|---|
//! | mapa **contínuo** (G3) | `120` |
//! | mapa **final** (pós-escada) | `149` |
//!
//! ⇒ **`80 %` das dobras nascem no contínuo.** Curar só a escada deixa `120`; curar só o
//! contínuo deixa `29`; e **curar o contínuo e depois arredondar dá `169`** — pior que não
//! curar nada, porque a escada re-dobra a partir de outro ponto de partida.
//!
//! ⇒ ⭐ *A injectividade não é um passe. Tem de ser uma propriedade do OBJECTIVO que a fase
//! optimiza* — e hoje o G3 minimiza `‖∇f − R/h‖²`, uma Poisson por mínimos quadrados **sem
//! termo nenhum de injectividade**.
//!
//! # ⭐⭐⭐ A variável é a RAIZ DA CLASSE, e é isso que preserva a costura
//!
//! A costura já entra por **eliminação de variável** (obra A, 2026-08-24): cada cópia de um
//! vértice é `uv = R^k · raiz + t`, com `k` e `t` **constantes** durante a resolução. ⇒ descer
//! nas **raízes** produz, por construção, um mapa que satisfaz a costura **exactamente** — não
//! há projecção nenhuma a desfazer o trabalho da descida.
//!
//! ⛔⛔ **É essa a diferença para a sonda `seam_free_probe`**, que media a mesma ideia por
//! **projecção** (`derive` a esmagar as cópias não-raiz) e estagnava a oscilar: *a descida
//! desfazia e a projecção refazia, para sempre.*
//!
//! ⚠️ **O `t` de cada cópia extrai-se UMA vez do mapa consistente que entra** (`t = uv − R^k ·
//! raiz`), e não se lê de dentro da `Weld`: ele é uma composição de translações de costura ao
//! longo do caminho até à raiz, e reconstruí-la aqui seria uma segunda aritmética a divergir da
//! primeira.
//!
//! ⚠️ **Clean-room da literatura pública** — a energia e o calendário de `ε` vivem em
//! [`ph2d_untangle`]; aqui mora só a mudança de variável.

use crate::cut::CutMesh;
use crate::solve::GridMap;
use crate::weld::Weld;
use ph2d_mesh::Mesh;
use ph2d_untangle::{Element, History, Settings, energy, energy_and_gradient, lbfgs_direction};

/// O que a resolução injectiva fez.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct InjectiveReport {
    /// Triângulos dobrados antes.
    pub flipped_before: usize,
    /// Triângulos dobrados depois. ⭐ `0` é o objectivo.
    pub flipped_after: usize,
    /// `min det J` antes e depois.
    pub min_det: (f64, f64),
    /// Iterações externas (uma por valor de `ε`).
    pub outer: usize,
    /// Iterações internas ao todo.
    pub inner: usize,
    /// ⛔ Ficou com dobras.
    pub gave_up: bool,
}

/// ⛔⛔⛔ **NASCE DESLIGADO, e a tabela da recusa está aqui.** `PH2D_GRIDMAP_INJECTIVE=1` liga.
///
/// ⭐⭐⭐ **O que ela ENTREGA (peça do artista, `Detail 0,85`, 2026-08-30):**
///
/// | | dobras no mapa contínuo | `min det` | iterações | relógio |
/// |---|---|---|---|---|
/// | sem ela | `120` | `−2,870` | — | — |
/// | ⭐ **com ela** | **`0`** | ⭐ **`+1,245e−4`** | `5` externas (tecto `64`) | **`352 ms`** |
///
/// ⇒ **o mapa contínuo fica LOCALMENTE INJECTIVO**, que é exactamente o que a literatura
/// promete, e converge — não é truncagem (gasta `5` das `64` externas disponíveis).
///
/// ⛔⛔⛔ **E o PRODUTO sai PIOR, com o mesmo A/B pelo botão:**
///
/// | coluna | controlo | com ela |
/// |---|---|---|
/// | quads | `9 598` | ⛔ `14 521` (`1,51×`) |
/// | enviesamento p50 | `6,4°` | ⛔ **`21,3°`** |
/// | faces `>60°` | `2` | ⛔ **`1 191`** |
/// | defeitos locais | `0,48 %` | ⛔ **`4,83 %`** |
/// | `χ` · bordo | `1` · `4` | ⛔ `0` · `12` |
/// | faces dobradas na extracção | `22` | ⛔ **`415`** |
/// | ⭐ pontas cortadas | `2` de `12` | ⭐ **`1` de `12`** |
/// | ⭐ cobertura p50 (fidelidade) | `0,271 %` | ⭐ **`0,061 %`** |
/// | relógio | `57,7 s` | `80,1 s` |
///
/// ⭐⭐⭐ **A ARITMÉTICA LOCALIZA O DEFEITO SEM MARGEM:** o mapa que entra na escada é
/// **perfeito** (`0` dobras) e o que sai da extracção tem `415` faces dobradas ⇒ *o dano é
/// todo a jusante*. A escada gulosa prega os inteiros um a um e re-relaxa; a partir de outro
/// ponto de partida ela faz outras escolhas, e piores.
///
/// ⚠️ **E há um segundo mecanismo, de desenho:** o G3 minimiza `‖∇f − R/h‖²`, que fixa a
/// escala **e a ORIENTAÇÃO contra o campo cruzado**. Esta energia fixa a escala (pelo termo
/// `g`) e a conformidade (pelo `f`), e **não tem termo nenhum a amarrar o mapa ao campo** ⇒ as
/// linhas de grade podem rodar em relação a ele, e o enviesamento é o que se lê.
///
/// ⇒ ⭐⭐ **A obra seguinte está NOMEADA e não é esta:** a barreira tem de entrar **somada** ao
/// objectivo do G3 (`‖∇f − R/h‖² + w · barreira`), nunca a **substituí-lo**. É o que o
/// `PLANO_desdobrar_o_mapa` §10 pedia — *propriedade do objectivo que a fase optimiza* — e o
/// que aqui se construiu e provou é a **maquinaria** dela: a costura como variável, o
/// calendário de `ε`, o repouso em células, e a prova de que a partir de emaranhado ela chega
/// a zero.
///
/// ⚠️ **Duas colunas MELHORARAM, e não se apagam da tabela:** a peça perde **menos** pontas
/// (`1` contra `2`) e a fidelidade à escultura fica **`4,4×` melhor**. *A direcção está certa;
/// o que falta é não pagar por ela na forma dos quads.*
#[must_use]
pub fn enabled() -> bool {
    std::env::var("PH2D_GRIDMAP_INJECTIVE").as_deref() == Ok("1")
}

/// Uma rotação de `k·90°` no plano.
fn turn(v: [f64; 2], k: i32) -> [f64; 2] {
    match k.rem_euclid(4) {
        1 => [-v[1], v[0]],
        2 => [-v[0], -v[1]],
        3 => [v[1], -v[0]],
        _ => v,
    }
}

/// ⭐⭐⭐ **Torna o mapa localmente injectivo, descendo nas RAÍZES das classes.**
///
/// O mapa que entra tem de ser **consistente** com a costura (é o que o G3 devolve). O que sai
/// continua a sê-lo, por construção.
#[must_use]
pub fn make_injective(
    mesh: &Mesh,
    cut: &CutMesh,
    w: &Weld,
    map: &mut GridMap,
    step: crate::solve::Step<'_>,
    set: Settings,
) -> InjectiveReport {
    // ── O índice plano das cópias: `base[p] + local`.
    let mut base = Vec::with_capacity(cut.tris.len() + 1);
    let mut total = 0usize;
    for uvp in &map.uv {
        base.push(total);
        total += uvp.len();
    }
    let mut flat: Vec<[f64; 2]> = Vec::with_capacity(total);
    for uvp in &map.uv {
        flat.extend(uvp.iter().map(|c| [f64::from(c[0]), f64::from(c[1])]));
    }

    // ── Os elementos, em índices planos.
    let pos = mesh.positions();
    let mut elements: Vec<Element> = Vec::new();
    for (p, tris) in cut.tris.iter().enumerate() {
        let Some(origin) = cut.origin.get(p) else {
            continue;
        };
        let b = base[p];
        for t in tris {
            // ⚠️ O passo é o **do triângulo**, pela porta da [`crate::solve::Step`] — é a
            // mesma média que o G3 usa, e um passo por vértice (`Follow Curvature`) chega
            // aqui sem esta função saber que ele existe.
            let g: Vec<u32> = t
                .iter()
                .map(|&l| origin.get(l as usize).copied().unwrap_or(0))
                .collect();
            let Some(el) = element_of(pos, origin, *t, b, f64::from(step.at(&g))) else {
                continue;
            };
            elements.push(el);
        }
    }
    let flipped_before = ph2d_untangle::flipped(&elements, &flat);
    let det_before = ph2d_untangle::min_det(&elements, &flat);
    let mut rep = InjectiveReport {
        flipped_before,
        flipped_after: flipped_before,
        min_det: (det_before, det_before),
        ..InjectiveReport::default()
    };
    if elements.is_empty() || flipped_before == 0 {
        return rep;
    }

    // ── A mudança de variável: por cópia, a classe, a rotação e a translação.
    //
    // ⚠️ **O `t` sai do mapa que entrou**, que é consistente por hipótese. *Reconstruí-lo da
    // `Weld` seria uma segunda aritmética a divergir da primeira.*
    let classes = w.classes();
    let mut roots: Vec<[f64; 2]> = vec![[0.0; 2]; classes];
    for (c, r) in roots.iter_mut().enumerate() {
        let v = w.value_pub(map, c);
        *r = [f64::from(v[0]), f64::from(v[1])];
    }
    let mut owner: Vec<Option<(u32, i32, [f64; 2])>> = vec![None; total];
    for (p, uvp) in map.uv.iter().enumerate() {
        for l in 0..uvp.len() {
            let Some((c, k)) = w.of(p, l) else { continue };
            let Ok(ci) = u32::try_from(c) else { continue };
            let i = base[p] + l;
            let r = turn(roots[c], k);
            owner[i] = Some((ci, k, [flat[i][0] - r[0], flat[i][1] - r[1]]));
        }
    }
    // ⛔ Uma cópia sem classe não tem variável — ela não existe no espaço reduzido, e mexer
    // nela seria escrever fora do domínio. *Conta-se e fica onde está.*
    let forward = |roots: &[[f64; 2]], flat: &mut [[f64; 2]]| {
        for (i, o) in owner.iter().enumerate() {
            if let Some((c, k, t)) = o {
                let r = turn(roots[*c as usize], *k);
                flat[i] = [r[0] + t[0], r[1] + t[1]];
            }
        }
    };
    // ⭐ O gradiente volta pela transposta da rotação, que é a rotação inversa.
    let pull = |g: &[[f64; 2]], out: &mut [[f64; 2]]| {
        for v in out.iter_mut() {
            *v = [0.0, 0.0];
        }
        for (i, o) in owner.iter().enumerate() {
            if let Some((c, k, _)) = o {
                let b = turn(g[i], -*k);
                out[*c as usize][0] += b[0];
                out[*c as usize][1] += b[1];
            }
        }
    };

    let livre = vec![false; classes];
    let mut gflat = vec![[0.0f64; 2]; total];
    let mut groot = vec![[0.0f64; 2]; classes];
    let mut hist: History = Vec::new();
    let mut prev = f64::INFINITY;

    for outer in 0..set.max_outer {
        rep.outer = outer + 1;
        forward(&roots, &mut flat);
        let worst = ph2d_untangle::min_det(&elements, &flat);
        let eps = 4e-2f64.mul_add(worst.min(0.0).powi(2), 1e-12).sqrt();
        hist.clear();
        let mut e_now = {
            let e = energy_and_gradient(&elements, &flat, eps, set.lambda, &mut gflat);
            pull(&gflat, &mut groot);
            e
        };
        for _ in 0..set.max_inner {
            rep.inner += 1;
            let dir = lbfgs_direction(&hist, &groot, &livre);
            let slope: f64 = dir
                .iter()
                .zip(groot.iter())
                .map(|(d, g)| d[0].mul_add(g[0], d[1] * g[1]))
                .sum();
            let step: Vec<[f64; 2]> = if slope.is_finite() && slope > 0.0 {
                dir.iter().map(|d| [-d[0], -d[1]]).collect()
            } else {
                hist.clear();
                groot.iter().map(|g| [-g[0], -g[1]]).collect()
            };
            let before = roots.clone();
            let gbefore = groot.clone();
            if !line_search(
                &elements, &mut roots, &step, e_now, eps, set.lambda, &forward, &mut flat,
                &mut e_now,
            ) {
                break;
            }
            energy_and_gradient(&elements, &flat, eps, set.lambda, &mut gflat);
            pull(&gflat, &mut groot);
            let s: Vec<[f64; 2]> = roots
                .iter()
                .zip(before.iter())
                .map(|(a, b)| [a[0] - b[0], a[1] - b[1]])
                .collect();
            let y: Vec<[f64; 2]> = groot
                .iter()
                .zip(gbefore.iter())
                .map(|(a, b)| [a[0] - b[0], a[1] - b[1]])
                .collect();
            if dot(&s, &y) > 1e-30 {
                hist.push((s, y));
                if hist.len() > 8 {
                    hist.remove(0);
                }
            }
        }
        let after = ph2d_untangle::min_det(&elements, &flat);
        if after > 0.0 && e_now > (1.0 - set.rel_tol) * prev {
            break;
        }
        prev = e_now;
    }

    forward(&roots, &mut flat);
    rep.flipped_after = ph2d_untangle::flipped(&elements, &flat);
    rep.min_det.1 = ph2d_untangle::min_det(&elements, &flat);
    rep.gave_up = rep.flipped_after > 0;

    // ⭐⭐ **Só se escreve de volta o que MELHOROU** — e pela porta da costura (`Weld::set`),
    // que escreve a raiz e deriva o resto. *Escrever as cópias à mão seria a segunda aritmética
    // outra vez.*
    if rep.flipped_after < rep.flipped_before {
        for (c, r) in roots.iter().enumerate() {
            #[expect(
                clippy::cast_possible_truncation,
                reason = "o mapa e' f32; a descida corre em f64 e volta"
            )]
            w.set(map, c, [r[0] as f32, r[1] as f32]);
        }
    }
    rep
}

fn dot(a: &[[f64; 2]], b: &[[f64; 2]]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| x[0].mul_add(y[0], x[1] * y[1]))
        .sum()
}

/// Busca linear de Armijo nas RAÍZES — cada tentativa reconstrói o mapa pela mudança de
/// variável, que é o que mantém a costura exacta a cada avaliação.
#[expect(
    clippy::too_many_arguments,
    reason = "e' uma busca linear com mudanca de variavel: malha, estado, direccao, energia, os dois parametros, o mapa directo e a saida"
)]
fn line_search(
    elements: &[Element],
    roots: &mut [[f64; 2]],
    dir: &[[f64; 2]],
    e0: f64,
    eps: f64,
    lambda: f64,
    forward: &impl Fn(&[[f64; 2]], &mut [[f64; 2]]),
    flat: &mut [[f64; 2]],
    out: &mut f64,
) -> bool {
    let before: Vec<[f64; 2]> = roots.to_vec();
    let mut t = 1.0f64;
    for _ in 0..40 {
        for (i, r) in roots.iter_mut().enumerate() {
            *r = [
                t.mul_add(dir[i][0], before[i][0]),
                t.mul_add(dir[i][1], before[i][1]),
            ];
        }
        forward(roots, flat);
        let e = energy(elements, flat, eps, lambda);
        if e.is_finite() && e < e0 {
            *out = e;
            return true;
        }
        t *= 0.5;
    }
    roots.copy_from_slice(&before);
    forward(roots, flat);
    false
}

/// O elemento de um triângulo, com o repouso achatado isometricamente, **em unidades de
/// célula da grade**, e os cantos em índice **plano**.
///
/// ⛔⛔⛔ **A DIVISÃO POR `h` É LOAD-BEARING, e a 1.ª redacção não a tinha** (medido em
/// 2026-08-30, A/B ponta a ponta pelo botão na peça do artista):
///
/// | coluna | controlo | sem a divisão |
/// |---|---|---|
/// | quads | `9 598` | ⛔ `21 868` (**`2,3×`**) |
/// | enviesamento p50 | `6,4°` | ⛔ **`30,3°`** |
/// | faces `>60°` | `2` | ⛔ **`2 955`** |
/// | defeitos locais | `0,48 %` | ⛔ **`7,69 %`** |
///
/// ⚠️ **O mecanismo é aritmético e não afinação:** o termo `g(J) = (det²J + 1)/det J` da
/// energia é minimizado em **`det J = 1`**. Com o repouso em unidades do **mundo**, isso pede
/// *uma célula por unidade de área do mundo* — e o alvo do G3 é *uma célula por `h`*, com
/// `h ≈ 0,038`. ⇒ a barreira puxava contra a densidade que o artista pediu, e ganhava.
///
/// ⭐ Com o repouso dividido pelo passo, `det J = 1` é **exactamente** a densidade pedida, e o
/// termo deixa de ter opinião sobre ela. *Um termo sem escala num problema com escala não é
/// neutro: ele impõe a escala `1`.*
fn element_of(
    pos: &[[f32; 3]],
    origin: &[u32],
    t: [u32; 3],
    base: usize,
    h: f64,
) -> Option<Element> {
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
    let flat = [
        u32::try_from(base + t[0] as usize).ok()?,
        u32::try_from(base + t[1] as usize).ok()?,
        u32::try_from(base + t[2] as usize).ok()?,
    ];
    if !h.is_finite() || h <= 0.0 {
        return None;
    }
    Element::from_rest(flat, [0.0, 0.0], [l1 / h, 0.0], [x / h, y / h])
}

#[cfg(test)]
#[path = "injective_solve_tests.rs"]
mod tests;

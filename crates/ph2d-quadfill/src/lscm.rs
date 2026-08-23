//! ⭐⭐⭐ **O ACHATAMENTO DE FRONTEIRA LIVRE** — mínimos quadrados conformes (LSCM).
//!
//! Clean-room a partir de **Lévy, Petitjean, Ray, Maillot, *Least Squares Conformal
//! Maps for Automatic Texture Atlas Generation*, SIGGRAPH 2002** — algoritmo
//! publicado, e a energia abaixo está deduzida de raiz. ⛔ Nenhuma linha de fonte
//! GPL — ver ADR-0162.
//!
//! # ⛔ O que ele substitui, e por que as duas tentativas anteriores não chegaram lá
//!
//! | achatamento | condição de fronteira | o que mediu |
//! |---|---|---|
//! | **Tutte** ([`crate::param`]) | fronteira **pregada** no polígono, por `τ` | domínio `1,0°` → superfície `16°`: o mapa **acrescenta `15°`** |
//! | **quadrilátero extremal** ([`crate::rectangle`]) | Dirichlet em dois lados, natural nos outros | conforme (`12,4°` → `14°`, folga `1,6°`) ⛔ **mas RECUSA-SE em patch grande** (fronteira livre não-monótona) e só serve `n = 4` |
//! | ⭐ **LSCM** (aqui) | **nenhuma** — só dois pinos | *é o que esta constante existe para medir* |
//!
//! ⭐⭐ **A diferença de espécie: o LSCM não tem condição de fronteira para errar.**
//! Os dois anteriores impõem onde o bordo cai e pagam-no no interior; este minimiza a
//! não-conformalidade **sobre o patch inteiro** e deixa o bordo ir para onde a
//! conformalidade o manda. *É a construção-padrão para cartas de atlas de textura —
//! isto é, exactamente para «um pedaço grande e curvo de superfície».*
//!
//! # A energia, deduzida
//!
//! Cada triângulo achata-se **isometricamente** numa moldura local `(x, y)`. Sobre
//! ele, `u` e `v` são lineares, logo os gradientes são constantes:
//!
//! ```text
//!     ∂u/∂x = Σ_j dx_j·u_j      dx_j = (y_{j+1} − y_{j+2}) / 2A
//!     ∂u/∂y = Σ_j dy_j·u_j      dy_j = (x_{j+2} − x_{j+1}) / 2A
//! ```
//!
//! Cauchy–Riemann pede `∂u/∂x = ∂v/∂y` e `∂u/∂y = −∂v/∂x`; o resíduo disso, pesado
//! pela área, é a energia:
//!
//! ```text
//!     E = Σ_t A_t · [ (∂u/∂x − ∂v/∂y)² + (∂u/∂y + ∂v/∂x)² ]
//! ```
//!
//! # ⭐⭐⭐ E o passo de Gauss–Seidel sai em FORMA FECHADA, com os eixos desacoplados
//!
//! Derivando em ordem a `u_k` e `v_k` e somando sobre os triângulos incidentes, os
//! termos cruzados **cancelam-se**: o que multiplica `v` na equação de `u` é
//! `Σ A(−dy·dx + dx·dy) = 0`, e vice-versa. ⇒ o sistema local `2×2` é **diagonal**:
//!
//! ```text
//!     u_k = −Σ A(a₁·dx + a₂·dy) / Σ A(dx² + dy²)
//!     v_k = −Σ A(−a₁·dy + a₂·dx) / Σ A(dx² + dy²)
//! ```
//!
//! onde `a₁, a₂` são os resíduos que os **outros dois** vértices do triângulo já
//! trazem. ⭐ *Nenhum solver esparso entra aqui, e o denominador é uma soma de
//! quadrados — sempre positivo, ao contrário do cotangente.*
//!
//! # ⚠️ Os dois PINOS, e por que são exactamente dois
//!
//! A energia é invariante por **similaridade** do plano: translação (2), rotação (1)
//! e escala (1) — quatro graus de liberdade, e o mínimo é `E = 0` no colapso a um
//! ponto. Pregar **dois** vértices fixa os quatro de uma vez e mais nada.
//! ⭐ **Os escolhidos são o par de fronteira mais afastado em `ℝ³`**, que é a escolha
//! mais estável (dois pinos próximos deixam a escala mal condicionada) e é
//! determinística.

/// ⛔⛔⛔ **DESLIGADO — MEDIDO E REJEITADO** (2026-08-23). E é a rejeição mais
/// informativa desta linha: ela **refuta a premissa que atravessou a semana inteira**.
///
/// ⚠️ **Com `false` o achatamento é byte-idêntico ao de sempre.**
///
/// # ⭐⭐⭐ A tabela, e a terceira coluna é a que conta
///
/// Esfera lisa, `d = 0,55`:
///
/// | | Tutte (shipa) | LSCM a `4 000` rondas | ⭐ LSCM **convergido** |
/// |---|---|---|---|
/// | **erro conforme** ([`conformal_error`]) | ⛔ `4,32` | `1,82` | ⭐ **`1,01`** |
/// | enviesamento p50 | **`18°`** | `18°` | ⛔ **`28°`** |
/// | domínio dos rectângulos | `1,0°` | `1,9°` | ⛔ **`21,4°`** |
/// | domínio dos leques | `18,7°` | `18,9°` | ⛔ **`50,8°`** |
/// | dobras | **`0`** | `5` | ⛔ **`68`** |
/// | aspecto p50 | **`1,26`** | `1,29` | ⛔ `1,43` |
///
/// ⭐⭐⭐ **Um mapa PRATICAMENTE CONFORME (`1,01`) dá o PIOR resultado dos três.**
///
/// ⛔⛔ ⇒ **A conformalidade não é o objectivo.** A premissa que motivou o
/// [`crate::rectangle`], o `CONFORMAL_MAP` e este ficheiro — *«um mapa mais conforme dá
/// quads mais quadrados»* — está **refutada por medição direta**, com a régua que mede
/// exactamente a promessa do mapa ao lado do resultado.
///
/// # ⭐⭐⭐ E o mecanismo está à vista, na coluna do DOMÍNIO
///
/// Com o mapa conforme, o enviesamento **do domínio** salta de `1,0°` para `21,4°` nos
/// rectângulos e de `18,7°` para `50,8°` nos leques. ⇒ *num domínio conforme, os pontos
/// de bordo — que continuam a ser postos por **comprimento de arco** — caem em posições
/// muito desiguais, e a grade de Coons entre eles nasce torta.*
///
/// ⭐ **O Tutte pregado não é um defeito: ele está a MASCARAR a discordância.** Ele
/// redistribui os pontos de bordo uniformemente no domínio e paga isso em
/// não-conformalidade — e o líquido é **melhor**. *A conformalidade não remove a
/// discordância da subdivisão; ela deixa de a esconder.*
///
/// ⇒ ⭐⭐⭐ **O constrangimento é a SUBDIVISÃO DO ARCO, e ele é maior do que estava
/// medido:** não os `12,4°` que o [`crate::rectangle`] viu num mapa parcial, mas
/// `21,4°` (rectângulos) e `50,8°` (leques). **Nenhum mapa o cura, e um mapa melhor
/// expõe-no mais.**
///
/// ⚠️ Preço secundário, medido: a sonda da esfera lisa passou de `~15 s` para
/// **`7 min 22 s`** — Gauss–Seidel a `100 000` rondas por patch. *Mesmo que a qualidade
/// tivesse ganho, isto não shipava sem outro solver.*
///
/// ⛔ **A rede é uma CONTAGEM:** o LSCM **não** garante ausência de dobras (o teorema
/// de Tutte precisa de fronteira convexa pregada, que é justamente o que aqui não há),
/// então quem virar triângulos no domínio **recua** para o Tutte.
pub(crate) const LSCM_MAP: bool = false;

/// ⚠️⚠️ **QUANTAS RONDAS o LSCM corre, e por que NÃO são as do Tutte.**
///
/// ⛔ **Medido 2026-08-23, e apanhou uma medição minha já escrita:** este achatamento
/// corria com as `4 000` rondas do [`crate::param`], e numa faixa alongada isso deixa
/// o erro conforme em `1,0929` quando o correcto é `1,0000` — o mapa que a sonda da
/// cadeia mediu estava **sub-convergido**, e a conclusão que eu ia tirar dele assentava
/// nisso.
///
/// | rondas | erro conforme (faixa `6 × 1` plana) | resíduo |
/// |---|---|---|
/// | `4 000` (as do Tutte) | ⛔ `1,0929` | `3,4e−4` |
/// | `20 000` | `1,0033` | `1,5e−5` |
/// | ⭐ **`100 000`** | **`1,0000`** | `1,2e−7` |
/// | `400 000` | `1,0000` | `1,2e−7` |
///
/// ⭐ **O número é onde a tabela deixa de mudar**, não uma folga escolhida: Gauss–Seidel
/// sobre uma faixa alongada converge devagar porque o condicionamento cresce com o
/// quadrado do alongamento, e o Tutte não tem esse problema (a fronteira pregada
/// prende-o de imediato). *Dois solvers diferentes não partilham um teto de espera.*
pub(crate) const LSCM_ROUNDS: usize = 100_000;

/// O que o achatamento devolve.
pub(crate) struct Flat {
    /// Por vértice local, o ponto no domínio, já normalizado para `[−1, 1]²`.
    pub(crate) uv: Vec<[f32; 2]>,
    /// Quantas rondas gastou.
    pub(crate) rounds: usize,
    /// O resíduo com que parou.
    pub(crate) residual: f32,
}

/// Os coeficientes de gradiente de um triângulo achatado isometricamente.
struct Tri {
    v: [u32; 3],
    dx: [f32; 3],
    dy: [f32; 3],
    area: f32,
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2]))
}

/// **ACHATA UM TRIÂNGULO ISOMETRICAMENTE** — `p0` na origem, `p1` no eixo `x`.
///
/// ⚠️ **Isometricamente e não por projecção numa normal média:** o que a energia
/// mede é a distorção *em relação ao triângulo verdadeiro*, e uma projecção já teria
/// distorcido antes de a conta começar.
fn local(p: [[f32; 3]; 3]) -> Option<([[f32; 2]; 3], f32)> {
    let e1 = sub(p[1], p[0]);
    let len = dot(e1, e1).sqrt();
    if len <= 1.0e-12 {
        return None;
    }
    let ex = [e1[0] / len, e1[1] / len, e1[2] / len];
    let e2 = sub(p[2], p[0]);
    let x2 = dot(e2, ex);
    let y2 = dot(e2, e2).mul_add(1.0, -(x2 * x2)).max(0.0).sqrt();
    let area = 0.5 * len * y2;
    (area > 1.0e-16).then_some(([[0.0, 0.0], [len, 0.0], [x2, y2]], area))
}

/// ⭐⭐⭐ **RESOLVE.** `boundary` são os vértices **locais** da fronteira do patch, em
/// qualquer ordem — só servem para escolher os dois pinos.
///
/// ⚠️ **`None` é uma resposta e não uma falha** — o chamador fica com o Tutte.
pub(crate) fn solve(
    tris: &[[u32; 3]],
    pos: &[[f32; 3]],
    boundary: &[u32],
    rounds_cap: usize,
    tol: f32,
) -> Option<Flat> {
    let nv = pos.len();
    if tris.is_empty() || boundary.len() < 2 || nv < 3 {
        return None;
    }
    // ── Os coeficientes de cada triângulo, e a vizinhança por vértice.
    let mut cells: Vec<Tri> = Vec::with_capacity(tris.len());
    let mut incident: Vec<Vec<u32>> = vec![Vec::new(); nv];
    for t in tris {
        let Some((q, area)) = local([
            *pos.get(t[0] as usize)?,
            *pos.get(t[1] as usize)?,
            *pos.get(t[2] as usize)?,
        ]) else {
            continue;
        };
        let inv = 1.0 / (2.0 * area);
        let mut dx = [0.0f32; 3];
        let mut dy = [0.0f32; 3];
        for j in 0..3 {
            dx[j] = (q[(j + 1) % 3][1] - q[(j + 2) % 3][1]) * inv;
            dy[j] = (q[(j + 2) % 3][0] - q[(j + 1) % 3][0]) * inv;
        }
        let id = u32::try_from(cells.len()).ok()?;
        for &v in t {
            incident.get_mut(v as usize)?.push(id);
        }
        cells.push(Tri {
            v: *t,
            dx,
            dy,
            area,
        });
    }
    if cells.is_empty() {
        return None;
    }

    // ── ⭐ **OS DOIS PINOS: o par de fronteira mais afastado.** Ver o doc do módulo.
    //
    // ⚠️ A varredura é `O(b²)` sobre a FRONTEIRA (dezenas a centenas de vértices),
    // não sobre o patch — e é determinística, que é o que importa aqui.
    let (mut best, mut pins) = (-1.0f32, (boundary[0], boundary[1]));
    for (i, &a) in boundary.iter().enumerate() {
        for &b in &boundary[i + 1..] {
            let d = sub(*pos.get(a as usize)?, *pos.get(b as usize)?);
            let d = dot(d, d);
            if d > best {
                best = d;
                pins = (a, b);
            }
        }
    }
    if best <= 0.0 {
        return None;
    }
    let span = best.sqrt();
    let mut uv = vec![[0.0f32; 2]; nv];
    let mut fixed = vec![false; nv];
    uv[pins.0 as usize] = [0.0, 0.0];
    uv[pins.1 as usize] = [span, 0.0];
    fixed[pins.0 as usize] = true;
    fixed[pins.1 as usize] = true;

    // ── Gauss–Seidel, com o passo em forma fechada do doc do módulo.
    let (mut rounds, mut residual) = (0usize, f32::INFINITY);
    for r in 0..rounds_cap {
        let mut worst = 0.0f32;
        for k in 0..nv {
            if fixed[k] || incident[k].is_empty() {
                continue;
            }
            let (mut n0, mut n1, mut den) = (0.0f32, 0.0f32, 0.0f32);
            for &c in &incident[k] {
                let t = &cells[c as usize];
                let Some(slot) = t.v.iter().position(|&x| x as usize == k) else {
                    continue;
                };
                // Os resíduos que os OUTROS dois vértices já trazem.
                let (mut a1, mut a2) = (0.0f32, 0.0f32);
                for j in 0..3 {
                    if j == slot {
                        continue;
                    }
                    let w = uv[t.v[j] as usize];
                    a1 = t.dx[j].mul_add(w[0], a1) - t.dy[j] * w[1];
                    a2 = t.dy[j].mul_add(w[0], a2) + t.dx[j] * w[1];
                }
                let (dx, dy) = (t.dx[slot], t.dy[slot]);
                n0 += t.area * a1.mul_add(dx, a2 * dy);
                n1 += t.area * a2.mul_add(dx, -(a1 * dy));
                den += t.area * dx.mul_add(dx, dy * dy);
            }
            if den <= 1.0e-20 {
                continue;
            }
            let next = [-n0 / den, -n1 / den];
            if !next[0].is_finite() || !next[1].is_finite() {
                return None;
            }
            worst = worst.max((next[0] - uv[k][0]).abs().max((next[1] - uv[k][1]).abs()));
            uv[k] = next;
        }
        rounds = r + 1;
        residual = worst;
        if worst < tol * span {
            break;
        }
    }

    // ── ⭐ **Normalizar para `[−1, 1]²` — e é uma SIMILARIDADE, logo continua conforme.**
    //
    // ⚠️ O balde de localização do [`crate::param`] mapeia `[−1, 1]` em células e não
    // sabe de domínios livres; um `uv` fora do alcance faria a amostragem falhar na
    // borda, **em silêncio**.
    let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
    for p in &uv {
        for k in 0..2 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    let scale = 2.0 / (hi[0] - lo[0]).max(hi[1] - lo[1]).max(1.0e-20);
    if !scale.is_finite() {
        return None;
    }
    let mid = [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5];
    for p in &mut uv {
        p[0] = (p[0] - mid[0]) * scale;
        p[1] = (p[1] - mid[1]) * scale;
    }
    Some(Flat {
        uv,
        rounds,
        residual,
    })
}

/// ⭐⭐ **QUANTO O MAPA SE AFASTA DE CONFORME** — a razão entre os valores singulares
/// do jacobiano, por triângulo, em mediana.
///
/// ⛔ **Ela existe porque «é conforme» é uma afirmação e não uma esperança.** Um mapa
/// perfeitamente conforme dá `1,0`; o Tutte sobre um patch grande dá muito mais. *Sem
/// esta régua, trocar de achatamento seria trocar uma crença por outra* — e é o mesmo
/// erro que custou a leitura do `deslizou 1/2`.
pub(crate) fn conformal_error(tris: &[[u32; 3]], pos: &[[f32; 3]], uv: &[[f32; 2]]) -> f32 {
    let mut all: Vec<f32> = Vec::with_capacity(tris.len());
    for t in tris {
        let (Some(&a), Some(&b), Some(&c)) = (
            pos.get(t[0] as usize),
            pos.get(t[1] as usize),
            pos.get(t[2] as usize),
        ) else {
            continue;
        };
        let Some((q, area)) = local([a, b, c]) else {
            continue;
        };
        let inv = 1.0 / (2.0 * area);
        // O jacobiano do mapa `(x,y) -> (u,v)`, constante no triângulo.
        let (mut j00, mut j01, mut j10, mut j11) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        for j in 0..3 {
            let Some(&w) = uv.get(t[j] as usize) else {
                continue;
            };
            let dx = (q[(j + 1) % 3][1] - q[(j + 2) % 3][1]) * inv;
            let dy = (q[(j + 2) % 3][0] - q[(j + 1) % 3][0]) * inv;
            j00 = dx.mul_add(w[0], j00);
            j01 = dy.mul_add(w[0], j01);
            j10 = dx.mul_add(w[1], j10);
            j11 = dy.mul_add(w[1], j11);
        }
        // ⭐ Os valores singulares de uma 2×2 em forma fechada: `σ = |E| ± |F|` com
        // `E = ½|(j00+j11, j10−j01)|` e `F = ½|(j00−j11, j10+j01)|`.
        let e = 0.5 * (j00 + j11).hypot(j10 - j01);
        let f = 0.5 * (j00 - j11).hypot(j10 + j01);
        let (hi, lo) = (e + f, (e - f).abs());
        if hi > 1.0e-20 && lo > 1.0e-20 {
            all.push(hi / lo);
        }
    }
    all.sort_by(f32::total_cmp);
    all.get(all.len() / 2).copied().unwrap_or(0.0)
}

/// A fronteira de um patch, em índices locais e sem repetições.
pub(crate) fn boundary_of(chains: &[Vec<u32>]) -> Vec<u32> {
    let mut out: Vec<u32> = chains.iter().flatten().copied().collect();
    out.sort_unstable();
    out.dedup();
    out
}

/// **O que uma tentativa bem-sucedida devolve:** o domínio, as rondas, o resíduo e a
/// régua de cada lado.
pub(crate) type Flattened = (Vec<[f32; 2]>, usize, f32, Vec<Vec<(f32, [f32; 2])>>);

/// ⭐⭐⭐ **A TENTATIVA INTEIRA** — achata, confere a rede e devolve a régua de cada lado.
///
/// ⚠️ **Ela vive aqui e não no [`crate::param`] por causa do teto de LOC** (HR-18,
/// 700) — e por assunto: quem sabe o que o LSCM promete é quem sabe conferi-lo.
///
/// ⛔ **A rede é a contagem de triângulos VIRADOS**, e ela não é opcional: o teorema
/// de Tutte exige fronteira convexa pregada, que é exactamente o que aqui não há.
/// *`None` é a resposta normal, e o chamador fica com o achatamento de sempre.*
pub(crate) fn try_flatten(
    tris: &[[u32; 3]],
    pos: &[[f32; 3]],
    chains: &[Vec<u32>],
    tau: &[Vec<f32>],
    rounds_cap: usize,
    tol: f32,
) -> Option<Flattened> {
    let flat = solve(tris, pos, &boundary_of(chains), rounds_cap, tol)?;
    if crate::aligned::flipped(tris, &flat.uv) > 0 {
        return None;
    }
    // A régua de cada lado: a fracção de `τ` de cada vértice de malha, com o `uv` que
    // o achatamento lhe deu. É a MESMA estrutura que o mapa do rectângulo usa — ver
    // [`crate::param::PatchParam::uv_on_side`].
    let mut per_side: Vec<Vec<(f32, [f32; 2])>> = Vec::with_capacity(chains.len());
    for (i, c) in chains.iter().enumerate() {
        let side_tau = tau.get(i)?;
        if side_tau.len() != c.len() {
            return None;
        }
        let total = side_tau.last().copied().unwrap_or(0.0);
        per_side.push(
            c.iter()
                .zip(side_tau)
                .map(|(&v, &t)| {
                    let f = if total > 0.0 { t / total } else { 0.0 };
                    (f, flat.uv[v as usize])
                })
                .collect(),
        );
    }
    Some((flat.uv, flat.rounds, flat.residual, per_side))
}

#[cfg(test)]
mod tests {
    use super::{conformal_error, solve};

    /// ⭐⭐⭐ **O CONTROLO DO CONTROLO.** A régua da conformalidade é ela própria uma
    /// afirmação, e a primeira versão dela devolveu `0,00` — um valor que **não existe**
    /// (o mínimo é `1,0`, o mapa conforme perfeito). ⚠️ *Uma régua sem controlo positivo
    /// é uma opinião com casas decimais.*
    ///
    /// A identidade sobre um quadrado plano é conforme por construção.
    #[test]
    fn the_identity_on_a_flat_square_is_perfectly_conformal() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let tris = vec![[0, 1, 2], [0, 2, 3]];
        let uv: Vec<[f32; 2]> = pos.iter().map(|p| [p[0], p[1]]).collect();
        let e = conformal_error(&tris, &pos, &uv);
        assert!(
            (e - 1.0).abs() < 1.0e-3,
            "a identidade num quadrado plano mediu {e:.4} em vez de 1,0 -- a regua esta' \
             partida, e todo numero que ela produziu nao vale"
        );
        // ⛔ **O controlo NEGATIVO**: um esticão de `3×` num eixo é a menos conforme
        // que existe, e tem de dar exactamente `3`. *Sem ele a régua podia devolver
        // `1,0` para tudo.*
        let flat: Vec<[f32; 2]> = pos.iter().map(|p| [p[0] * 3.0, p[1]]).collect();
        let e = conformal_error(&tris, &pos, &flat);
        assert!(
            (e - 3.0).abs() < 1.0e-3,
            "um esticao de 3x mediu {e:.4} em vez de 3,0"
        );
    }

    /// ⭐⭐⭐ **A PROMESSA, medida: o LSCM é mais conforme que o Tutte.**
    ///
    /// ⛔ **Sem este gate, a conclusão desta jornada não vale.** O LSCM foi construído,
    /// medido e **desligado** porque não moveu o enviesamento (`18° → 18°` na esfera
    /// lisa) — e essa é uma afirmação sobre *o mapa não ser o constrangimento* **só se o
    /// mapa novo tiver de facto sido mais conforme**. Se ele tivesse um bug, o mesmo
    /// número contaria a história oposta. *Medido na cadeia: `4,32 → 1,82`.*
    ///
    /// # ⚠️ A fixtura teve de conter o fenómeno, e a primeira não continha
    ///
    /// A primeira tentativa foi uma calota parabólica suave: **Tutte `1,046`, LSCM
    /// `1,049`** — os dois quase conformes, empate, gate vermelho sobre código correcto.
    /// *Uma peça pouco distorcida não distingue dois mapas.*
    ///
    /// ⭐ **A que distingue é uma FAIXA ALONGADA e PLANA.** Ser plana torna o alvo
    /// exacto (o LSCM tem de dar `1,0` — é a identidade a menos de similaridade), e ser
    /// alongada é o que castiga o Tutte: pregar os quatro cantos de um `6 × 1` num
    /// **círculo** é exactamente a condição de fronteira errada que este ficheiro existe
    /// para dispensar. ⇒ *o gate mede o mecanismo, não a curvatura.*
    #[test]
    fn on_a_stretched_patch_lscm_beats_tutte_at_being_conformal() {
        // Uma faixa plana 6 × 1, em grade.
        let (nx, ny) = (18usize, 3usize);
        let mut pos = Vec::new();
        for j in 0..=ny {
            for i in 0..=nx {
                pos.push([6.0 * i as f32 / nx as f32, j as f32 / ny as f32, 0.0]);
            }
        }
        let at = |i: usize, j: usize| (j * (nx + 1) + i) as u32;
        let mut tris = Vec::new();
        for j in 0..ny {
            for i in 0..nx {
                tris.push([at(i, j), at(i + 1, j), at(i + 1, j + 1)]);
                tris.push([at(i, j), at(i + 1, j + 1), at(i, j + 1)]);
            }
        }
        // A fronteira, no sentido do contorno.
        let mut rim: Vec<u32> = (0..=nx).map(|i| at(i, 0)).collect();
        rim.extend((1..=ny).map(|j| at(nx, j)));
        rim.extend((0..nx).rev().map(|i| at(i, ny)));
        rim.extend((1..ny).rev().map(|j| at(0, j)));

        // ── O CONTROLO: Tutte com a fronteira pregada no CÍRCULO, valor médio.
        let nb = crate::weights::mean_value_weights(&tris, &pos);
        let mut uv = vec![[0.0f32; 2]; pos.len()];
        let mut fixed = vec![false; pos.len()];
        for (k, &v) in rim.iter().enumerate() {
            let a = std::f32::consts::TAU * k as f32 / rim.len() as f32;
            uv[v as usize] = [a.cos(), a.sin()];
            fixed[v as usize] = true;
        }
        for _ in 0..4_000 {
            for v in 0..pos.len() {
                if fixed[v] || nb[v].is_empty() {
                    continue;
                }
                let (mut s, mut w) = ([0.0f32; 2], 0.0f32);
                for &(u, k) in &nb[v] {
                    s[0] = k.mul_add(uv[u as usize][0], s[0]);
                    s[1] = k.mul_add(uv[u as usize][1], s[1]);
                    w += k;
                }
                if w > 0.0 {
                    uv[v] = [s[0] / w, s[1] / w];
                }
            }
        }
        let tutte = conformal_error(&tris, &pos, &uv);
        // ⚠️⚠️ **AS RONDAS SÃO 100 000 e o número é MEDIDO, não escolhido.** Gauss–Seidel
        // sobre uma faixa alongada converge devagar (o condicionamento cresce com o
        // quadrado do alongamento):
        //
        // | rondas | erro conforme | resíduo |
        // |---|---|---|
        // | `4 000` | ⛔ `1,0929` | `3,4e−4` |
        // | `20 000` | `1,0033` | `1,5e−5` |
        // | ⭐ `100 000` | **`1,0000`** | `1,2e−7` |
        // | `400 000` | `1,0000` | `1,2e−7` |
        //
        // ⛔ **E isto apanhou um erro na cadeia**: o [`crate::param`] corria o LSCM com
        // as mesmas `4 000` rondas do Tutte, logo o mapa que a sonda mediu estava
        // **sub-convergido** — ver [`LSCM_ROUNDS`].
        let flat = solve(&tris, &pos, &rim, 100_000, 1.0e-9).expect("o LSCM achata a faixa");
        let lscm = conformal_error(&tris, &pos, &flat.uv);
        // ⚠️ **A fixtura contém o fenómeno**, e isso é asserido ANTES do resultado.
        assert!(
            tutte > 1.5,
            "o Tutte mediu {tutte:.3} nesta faixa -- ela deixou de o castigar, e o gate \
             compararia dois mapas ja' conformes (foi o que matou a primeira fixtura)"
        );
        assert!(
            lscm < tutte,
            "o LSCM mediu {lscm:.3} de erro conforme e o Tutte {tutte:.3} -- ele nao cumpriu \
             a unica coisa que promete, e entao a conclusao «o mapa nao e' o constrangimento» \
             perde o seu controlo"
        );
        assert!(
            lscm < 1.01,
            "a faixa e' PLANA, logo o LSCM tinha de ser a identidade a menos de similaridade \
             ({lscm:.3}) -- se nao e', ha' bug nele e nao na conclusao"
        );
    }

    /// ⭐⭐ **E o LSCM de um quadrado plano é a identidade, a menos de similaridade.**
    #[test]
    fn lscm_of_a_flat_patch_is_conformal() {
        let pos = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ];
        let tris = vec![[0, 1, 2], [0, 2, 3]];
        let flat = solve(&tris, &pos, &[0, 1, 2, 3], 4_000, 1.0e-6).expect("achata");
        let e = conformal_error(&tris, &pos, &flat.uv);
        assert!(
            (e - 1.0).abs() < 1.0e-2,
            "o LSCM de um quadrado PLANO mediu {e:.4} de erro conforme -- ele tinha de ser \
             a identidade a menos de similaridade"
        );
    }
}

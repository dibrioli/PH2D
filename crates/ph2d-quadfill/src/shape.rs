//! ⭐⭐⭐ **A FORMA DE CADA QUAD** — a régua POR-FACE, e a que faltava.
//!
//! ⚠️ **Irmã do [`crate::report`] pelo teto de LOC (HR-18, 700) e por ASSUNTO:**
//! lá o que a montagem **diz** (recusa, proveniência, contagens); aqui a única
//! grandeza deste crate que olha **um quad de cada vez**.
//!
//! ⛔⛔ **Todas as outras réguas geométricas desta linha medem UM EXTREMO.**
//! [`crate::FillReport::edge_max`] é a aresta mais longa da malha inteira;
//! `edge_median` é a mediana de todas as arestas. *Um quad de `0,02 × 0,30` não
//! move nenhuma das duas* — a longa dele está muito abaixo da máxima e a curta
//! afunda-se na mediana de dezenas de milhares.
//!
//! ⚠️ **E foi assim que a malha de 2026-08-22 passou em `edge_max ≤ 20 %` — depois
//! de esse número ter caído de `57 %` da peça para `5,5 %` no mesmo dia — e o
//! artista escreveu «péssimo» a olhar para ela.**

use ph2d_mesh::Mesh;

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn norm(a: [f32; 3]) -> f32 {
    a[0].mul_add(a[0], a[1].mul_add(a[1], a[2] * a[2])).sqrt()
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1].mul_add(b[2], -(a[2] * b[1])),
        a[2].mul_add(b[0], -(a[0] * b[2])),
        a[0].mul_add(b[1], -(a[1] * b[0])),
    ]
}

/// ⭐⭐⭐ **A FORMA DE CADA QUAD, em percentis** — a régua por-face.
///
/// ⚠️ **Percentis, não médias.** O defeito é uma **faixa** de faces más numa malha
/// de dezenas de milhares de boas, e uma média dilui-a até desaparecer. *Medido em
/// 2026-08-22 na `eared_sphere`: aspecto médio `1,4`, p99,9 de `73`.*
///
/// ⭐ **As três grandezas são precisas JUNTAS**, e nenhuma substitui outra:
///
/// | grandeza | o que é | o que ela apanha | a que ela é cega |
/// |---|---|---|---|
/// | **aspecto** | longa ÷ curta **do mesmo quad** | o rectângulo `1 × 10` | o losango, que tem aspecto `1` |
/// | **enviesamento** | maior desvio de 90° nos cantos | o losango de 30° | o rectângulo, que tem cantos rectos |
/// | **área** | espalhamento p99 ÷ p50 | a orelha grossa ao lado da calota fina | as duas de cima |
///
/// ⭐⭐ **A BARRA é o oráculo**, medida com este mesmo código sobre a saída dele
/// (`ph2d-quadbench`, 2026-08-22):
///
/// | peça | aspecto p50 | p99 | `> 4×` | enviesamento p50 | p99 | `> 60°` |
/// |---|---|---|---|---|---|---|
/// | orelha | `1,08` | `1,4` | **0** | `6°` | `20°` | **0** |
/// | gancho | `1,19` | `2,8` | **0** | `6°` | `48°` | 4 |
/// | enrugada | `1,08` | `1,3` | **0** | `5°` | `17°` | **0** |
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct QuadShape {
    /// Aspecto (longa ÷ curta do mesmo quad) — mediana.
    pub aspect_p50: f32,
    /// Aspecto — percentil 99.
    pub aspect_p99: f32,
    /// Aspecto — o pior quad da malha.
    pub aspect_max: f32,
    /// Quantas faces passam de `4×`. ⭐ O oráculo entrega **zero** em toda peça
    /// orgânica do corpus.
    pub aspect_over_4: usize,
    /// Maior desvio de 90° num canto, em graus — mediana.
    pub skew_p50: f32,
    /// Enviesamento — percentil 99.
    pub skew_p99: f32,
    /// Enviesamento — o pior canto da malha.
    pub skew_max: f32,
    /// Quantas faces têm um canto pior que 60° de desvio (abaixo de 30° ou acima
    /// de 150°). ⭐ O oráculo entrega `0` a `4`.
    pub skew_over_60: usize,
    /// Área p99 ÷ área p50 — o espalhamento da densidade.
    pub area_spread: f32,
}

fn pct(sorted: &[f32], p: f32) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss, clippy::cast_sign_loss)]
    let i = ((sorted.len() - 1) as f32 * p).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}

/// **MEDE A FORMA DE CADA FACE** — ver [`QuadShape`].
///
/// ⚠️ Aceita faces de qualquer valência: um triângulo perdido na saída tem aspecto
/// e enviesamento como qualquer outra face, e [`FillReport::non_quads`] é quem
/// guarda a promessa de que ele não existe.
#[must_use]
pub fn quad_shape(mesh: &Mesh) -> QuadShape {
    let pos = mesh.positions();
    let mut aspect: Vec<f32> = Vec::with_capacity(mesh.faces().len());
    let mut skew: Vec<f32> = Vec::with_capacity(mesh.faces().len());
    let mut area: Vec<f32> = Vec::with_capacity(mesh.faces().len());
    for f in mesh.faces() {
        let v = f.verts();
        let n = v.len();
        if n < 3 {
            continue;
        }
        let p: Vec<[f32; 3]> = v.iter().map(|&i| pos[i as usize]).collect();
        let mut lo = f32::MAX;
        let mut hi = 0.0f32;
        let mut worst = 0.0f32;
        for k in 0..n {
            let e = norm(sub(p[(k + 1) % n], p[k]));
            lo = lo.min(e);
            hi = hi.max(e);
            let a = sub(p[(k + n - 1) % n], p[k]);
            let b = sub(p[(k + 1) % n], p[k]);
            let (la, lb) = (norm(a).max(1.0e-12), norm(b).max(1.0e-12));
            let c = a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2])) / (la * lb);
            worst = worst.max((c.clamp(-1.0, 1.0).acos().to_degrees() - 90.0).abs());
        }
        aspect.push(hi / lo.max(1.0e-12));
        skew.push(worst);
        // Área por leque desde o primeiro vértice — exacta para um quad planar e a
        // melhor aproximação para um alabeado.
        let mut acc = 0.0f32;
        for k in 1..n - 1 {
            acc += 0.5 * norm(cross(sub(p[k], p[0]), sub(p[k + 1], p[0])));
        }
        area.push(acc);
    }
    let aspect_over_4 = aspect.iter().filter(|a| **a > 4.0).count();
    let skew_over_60 = skew.iter().filter(|s| **s > 60.0).count();
    aspect.sort_by(f32::total_cmp);
    skew.sort_by(f32::total_cmp);
    area.sort_by(f32::total_cmp);
    QuadShape {
        aspect_p50: pct(&aspect, 0.50),
        aspect_p99: pct(&aspect, 0.99),
        aspect_max: aspect.last().copied().unwrap_or(0.0),
        aspect_over_4,
        skew_p50: pct(&skew, 0.50),
        skew_p99: pct(&skew, 0.99),
        skew_max: skew.last().copied().unwrap_or(0.0),
        skew_over_60,
        area_spread: pct(&area, 0.99) / pct(&area, 0.50).max(1.0e-12),
    }
}

/// ⭐⭐⭐ **ONDE MORA O ENVIESAMENTO** — a mediana por **fase de origem** dos cantos
/// da face. Ver [`crate::report::FillReport::skew_prov`].
///
/// ⛔ **Ela nasceu porque duas curas teoricamente correctas não moveram o número**
/// (2026-08-23): pôr o campo cruzado no interior do achatamento, e pôr os lados do
/// domínio na proporção dos segmentos. As duas deixaram o enviesamento mediano da
/// orelha em `27°`.
///
/// ⚠️ **Quando duas hipóteses boas falham, o modelo do defeito está errado** — e a
/// resposta é parar de supor e perguntar à malha *onde* ele mora. O que esta régua
/// respondeu, na orelha a `d = 1,0`:
///
/// ```text
///     canto (F3) 0°   arco 26°   centro (F3) 0°   raio 56°   grade 26°
/// ```
///
/// ⇒ **está em TODA a parte, e a grade interior mede o mesmo que o resto.** Não é
/// o leque, não é a costura, não é um caso raro. *Isso exclui de uma vez toda a
/// família de hipóteses «uma construção local está errada».*
///
/// ⚠️ **A face é classificada pela proveniência DOMINANTE dos cantos dela** — um
/// quad com dois cantos de arco e dois de grade conta para o lado que tiver mais, e
/// o empate vai para o menor índice. *É uma atribuição, não uma medição exacta;
/// serve para localizar, não para julgar uma face.*
#[must_use]
pub fn skew_by_provenance(
    mesh: &Mesh,
    prov: &[crate::report::Provenance],
) -> [f32; crate::report::Provenance::COUNT] {
    use crate::report::Provenance;
    let mut per: [Vec<f32>; Provenance::COUNT] = Default::default();
    let pos = mesh.positions();
    for f in mesh.faces() {
        let v = f.verts();
        let n = v.len();
        if n < 3 {
            continue;
        }
        let mut tally = [0usize; Provenance::COUNT];
        for &i in v {
            if let Some(p) = prov.get(i as usize) {
                tally[*p as usize] += 1;
            }
        }
        let Some(win) = (0..Provenance::COUNT).max_by_key(|&k| tally[k]) else {
            continue;
        };
        let mut worst = 0.0f32;
        for k in 0..n {
            let p0 = pos[v[k] as usize];
            let a = sub(pos[v[(k + n - 1) % n] as usize], p0);
            let b = sub(pos[v[(k + 1) % n] as usize], p0);
            let (la, lb) = (norm(a).max(1.0e-12), norm(b).max(1.0e-12));
            let c = a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2])) / (la * lb);
            worst = worst.max((c.clamp(-1.0, 1.0).acos().to_degrees() - 90.0).abs());
        }
        per[win].push(worst);
    }
    let mut out = [0.0f32; Provenance::COUNT];
    for (k, list) in per.iter_mut().enumerate() {
        list.sort_by(f32::total_cmp);
        if !list.is_empty() {
            out[k] = list[list.len() / 2];
        }
    }
    out
}

/// ⭐⭐⭐ **O ENVIESAMENTO, separado por QUEM CONSTRUIU a face** — a grade de um
/// rectângulo (`n = 4`) contra a de um sector de LEQUE (`n ≠ 4`).
///
/// Devolve `(rectângulo, leque)`, as duas medianas em graus.
///
/// ⛔ **É o número que decide se vale a pena reescrever o F3.** O arnês 2D provou que
/// um sector de leque **obriga** `|360/n − 90|` de enviesamento máximo e metade disso
/// de mediana, num domínio ideal (`tests/fan_sector.rs`); ⚠️ *o que ele não pode dizer
/// é quanto disso sobrevive à malha real, à curvatura e ao alisamento.* Esta régua
/// diz.
///
/// | se der | então |
/// |---|---|
/// | ⭐ rectângulo `~6°`, leque `~20°` | a previsão confirma-se, e **fazer o F3 entregar só quadriláteros** é a obra certa |
/// | os dois `~18°` | ⛔ o leque é inocente e o defeito é de outra coisa — **não construa** |
///
/// ⚠️ **Uma malha sem patches de quatro lados devolve `0,0` na primeira coluna**, e
/// `0` não é «perfeito», é «não medido». Quem a lê tem de olhar a contagem ao lado.
#[must_use]
pub fn skew_by_fan(mesh: &Mesh, from_fan: &[bool]) -> (f32, f32) {
    let pos = mesh.positions();
    let mut rect: Vec<f32> = Vec::new();
    let mut fan: Vec<f32> = Vec::new();
    for (i, f) in mesh.faces().iter().enumerate() {
        let v = f.verts();
        let n = v.len();
        if n < 3 {
            continue;
        }
        let mut worst = 0.0f32;
        for k in 0..n {
            let p0 = pos[v[k] as usize];
            let a = sub(pos[v[(k + n - 1) % n] as usize], p0);
            let b = sub(pos[v[(k + 1) % n] as usize], p0);
            let (la, lb) = (norm(a).max(1.0e-12), norm(b).max(1.0e-12));
            let c = a[0].mul_add(b[0], a[1].mul_add(b[1], a[2] * b[2])) / (la * lb);
            worst = worst.max((c.clamp(-1.0, 1.0).acos().to_degrees() - 90.0).abs());
        }
        if from_fan.get(i).copied().unwrap_or(false) {
            fan.push(worst);
        } else {
            rect.push(worst);
        }
    }
    let med = |v: &mut Vec<f32>| {
        v.sort_by(f32::total_cmp);
        v.get(v.len() / 2).copied().unwrap_or(0.0)
    };
    (med(&mut rect), med(&mut fan))
}

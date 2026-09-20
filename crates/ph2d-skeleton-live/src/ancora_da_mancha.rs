//! ⭐⭐⭐ **ONDE UMA MANCHA DE PESO POUSA** — a geometria do contorno, e a tradução
//! *«o dedo está aqui» → «no repouso, isso é ali»*.
//!
//! # ⛔⛔⛔ Por que este módulo existe: a F30 shipou uma LEI SEM GESTO
//!
//! Em 2026-09-19 a [`ph2d_vec_skin::curva`] fez a arte seguir o peso **entre** dois nós, e o gate
//! que o mede (`pintar_peso_entre_dois_nos_move_a_arte`) constrói a [`ph2d_skeleton_ecs::CorreccaoDePeso`]
//! **à mão**, com `centro` no meio de uma aresta. ⛔ Nada no repo perguntava se o PINCEL consegue
//! produzir esse centro — e ele **não conseguia**: o gesto ancorava a mancha no **nó mais perto** e
//! recusava (`ForaDaArte`) quando o dedo estava mais longe do que o raio do pincel.
//!
//! Medido na barra do smoke (`RoundRect` de 7×1, os oito nós nas duas pontas, raio de pincel de
//! omissão `40 px` = `0,40` de mundo):
//!
//! | o dedo | nó mais perto | veredito |
//! |---|---|---|
//! | no MEIO da barra | **`3,041`** de distância | **`ForaDaArte`** |
//! | idem, com o raio a `400 px` | idem | `Pintada`, **com o centro na QUINA** `(-8,0 · 2,0)` |
//!
//! ⇒ *a lei da F30 era inexprimível por gesto nenhum, e a cena do dono é exactamente onde isso
//! aparece.* É o terceiro elo do `CLAUDE.md` §5.0 outra vez — a porta faz efeito, o clique chega ao
//! barramento, e **nada juntava as duas pontas**.
//!
//! # ⭐⭐ A lei que fica
//!
//! A mancha pousa no **ponto do CONTORNO** mais perto do dedo, e o repouso dele sai do **MESMO
//! parâmetro** da curva. ⚠️ Não é o cursor cru: ancorar no cursor poria a correcção no vazio quando
//! o dedo passa ao lado, e ela deixaria de corrigir exactamente quando o artista pensa que a pôs.
//!
//! ⚠️⚠️ **O achatamento é o MESMO nos dois lados**, e é isso que torna a tradução honesta: a
//! amostra `i` do contorno posado e a amostra `i` do contorno de repouso são o mesmo ponto da
//! mesma curva, em duas poses. *Dois achatamentos diferentes dariam um repouso plausível e errado.*

/// O quadrado da distância entre dois pontos.
pub(crate) fn d2(a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (a[0] - b[0], a[1] - b[1]);
    dy.mul_add(dy, dx * dx)
}

/// Quantas amostras por segmento o contorno achatado leva.
///
/// ⚠️ **Grosseiro de propósito e SUFICIENTE por construção:** numa aresta RECTA — que é o que uma
/// barra, um rectângulo e os lados de um `RoundRect` têm — o achatamento é **exacto** a qualquer
/// densidade, e a projecção dentro de cada corda é contínua. O erro só existe onde a curva dobra,
/// e ali ele é uma fracção do raio do pincel.
const SEGMENTOS: usize = 8;

/// **O contorno de uma forma, achatado em segmentos** — a porta que o hit-test e a âncora
/// partilham.
///
/// ⚠️ **As triplas são `(âncora, alça de entrada, alça de saída)`** — a ordem que a
/// [`ph2d_vec_skin::aplica_corrigido`] percorre —, logo o troço entre o vértice `k` e o `k+1` é a
/// cúbica `(a_k, out_k, in_{k+1}, a_{k+1})`.
///
/// ⛔ **Achatada à mão e não pela `kurbo`:** esta crate é uma folha e não depende de geometria
/// vectorial.
///
/// ⚠️ **Ela serve as DUAS poses** (o nome dizia `_posado` e mentia sobre metade dos chamadores): o
/// contorno de repouso sai dela com as triplas de repouso, e é por serem a mesma função que a
/// amostra `i` de um corresponde à amostra `i` do outro.
pub(crate) fn contorno(pts: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let n = pts.len() / 3;
    if n < 2 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(n * SEGMENTOS);
    for k in 0..n {
        let (a, o) = (pts[3 * k], pts[3 * k + 2]);
        let j = (k + 1) % n;
        let (i, b) = (pts[3 * j + 1], pts[3 * j]);
        for s in 0..SEGMENTOS {
            #[expect(clippy::cast_precision_loss, reason = "s < SEGMENTOS, um punhado")]
            let t = s as f64 / SEGMENTOS as f64;
            let u = 1.0 - t;
            let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            out.push([
                w3.mul_add(b[0], w2.mul_add(i[0], w1.mul_add(o[0], w0 * a[0]))),
                w3.mul_add(b[1], w2.mul_add(i[1], w1.mul_add(o[1], w0 * a[1]))),
            ]);
        }
    }
    out
}

/// **O ponto está dentro do polígono?** — regra par/ímpar por cruzamentos.
pub(crate) fn dentro(poli: &[[f64; 2]], q: [f64; 2]) -> bool {
    let mut dentro = false;
    let n = poli.len();
    for i in 0..n {
        let (a, b) = (poli[i], poli[(i + 1) % n]);
        if (a[1] > q[1]) != (b[1] > q[1]) {
            let t = (q[1] - a[1]) / (b[1] - a[1]);
            if q[0] < t.mul_add(b[0] - a[0], a[0]) {
                dentro = !dentro;
            }
        }
    }
    dentro
}

/// A fracção do segmento `ab` onde `q` se projecta, presa a `[0,1]`.
fn fraccao(a: [f64; 2], b: [f64; 2], q: [f64; 2]) -> f64 {
    let (vx, vy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = vy.mul_add(vy, vx * vx);
    if l2 <= f64::EPSILON {
        return 0.0;
    }
    (((q[1] - a[1]) * vy + (q[0] - a[0]) * vx) / l2).clamp(0.0, 1.0)
}

/// A distância ao segmento `ab` — o que cobre um caminho ABERTO, que não tem interior.
pub(crate) fn d2_segmento(a: [f64; 2], b: [f64; 2], q: [f64; 2]) -> f64 {
    let t = fraccao(a, b, q);
    d2(
        [t.mul_add(b[0] - a[0], a[0]), t.mul_add(b[1] - a[1], a[1])],
        q,
    )
}

/// **Onde a mancha pousa** — o ponto do contorno, nas duas poses.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Ancora {
    /// No espaço da coisa no bind — o que a [`ph2d_skeleton_ecs::CorreccaoDePeso::centro`] guarda.
    pub repouso: [f64; 2],
    /// Onde esse mesmo ponto está AGORA, em mundo — contra o que o raio do pincel se mede.
    pub mundo: [f64; 2],
}

/// ⭐⭐⭐ **O PONTO DO CONTORNO SOB O CURSOR**, com o repouso tirado do MESMO parâmetro.
///
/// `repousos` e `posados` são as triplas da mesma forma nas duas poses, na mesma ordem.
///
/// ⚠️ Devolve `None` quando a forma não tem contorno (menos de dois vértices) — ali não há
/// parâmetro nenhum a traduzir, e quem chama cai no caminho do ponto mais perto.
pub(crate) fn no_contorno(
    repousos: &[[f64; 2]],
    posados: &[[f64; 2]],
    mundo: [f64; 2],
) -> Option<Ancora> {
    let (r, p) = (contorno(repousos), contorno(posados));
    if r.len() != p.len() || p.len() < 2 {
        return None;
    }
    let n = p.len();
    let mut melhor: Option<(f64, usize, f64)> = None;
    for i in 0..n {
        let j = (i + 1) % n;
        let t = fraccao(p[i], p[j], mundo);
        let d = d2(
            [
                t.mul_add(p[j][0] - p[i][0], p[i][0]),
                t.mul_add(p[j][1] - p[i][1], p[i][1]),
            ],
            mundo,
        );
        if melhor.is_none_or(|(b, _, _)| d < b) {
            melhor = Some((d, i, t));
        }
    }
    let (_, i, t) = melhor?;
    let j = (i + 1) % n;
    Some(Ancora {
        repouso: [
            t.mul_add(r[j][0] - r[i][0], r[i][0]),
            t.mul_add(r[j][1] - r[i][1], r[i][1]),
        ],
        mundo: [
            t.mul_add(p[j][0] - p[i][0], p[i][0]),
            t.mul_add(p[j][1] - p[i][1], p[i][1]),
        ],
    })
}

#[cfg(test)]
#[path = "ancora_da_mancha_tests.rs"]
mod ancora_da_mancha_tests;

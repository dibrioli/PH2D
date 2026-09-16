//! ⭐⭐⭐ **O PADRÃO-OURO PARA UMA FORMA VECTORIAL** — a segunda mídia a deixar a lei euclidiana.
//!
//! # A dívida que isto paga
//!
//! Em 2026-09-15 os *Bounded Biharmonic Weights* entraram no produto **só para imagens**, porque
//! eles precisam de uma **malha do domínio** e uma Bézier não tem uma. ⛔ O preço foi um rig com
//! **duas leis**: a imagem deformava-se por uma e a forma vectorial por outra, e o sintoma seria *«o
//! braço desenhado não acompanha o braço vectorial»* — o mesmo defeito que o [`tendons_for`] do
//! esqueleto já nomeia por escrito, um nível acima.
//!
//! [`tendons_for`]: ph2d_skeleton
//!
//! # ⭐⭐ O domínio de um caminho é o INTERIOR dele, e a malha é a MESMA da imagem
//!
//! ```text
//! contornos FECHADOS ──▶ achatados em polígonos   (a fronteira, do documento)
//!                    ──▶ grelha graduada + orçamento   (ph2d_poly2d::grid_mesh_com)
//!                    ──▶ Bounded Biharmonic Weights    (ph2d_skin_weights)
//!                    ──▶ um peso por PONTO DE CONTROLO (baricêntrico na malha)
//! ```
//!
//! ⚠️ **A malha é a da imagem, e é de propósito:** a graduação pelas articulações, a conformidade, o
//! orçamento em triângulos e a renormalização são do GRID e não da mídia. *Uma segunda grelha
//! escrita para o vector divergiria desta no primeiro ajuste* — e a divergência seria exactamente o
//! defeito que esta wave existe para fechar.
//!
//! ⛔ **A malha NÃO é guardada nem desenhada.** Ela é um andaime do *bind*: o que sobrevive é um
//! peso por ponto de controlo. A forma continua a ser uma Bézier exacta e editável, que é a razão
//! de o vector usar LBS e não um envelope.
//!
//! # ⛔ Um caminho ABERTO não tem interior, e a resposta é *não sei*
//!
//! Uma linha de construção tem área zero: não há domínio, não há energia, não há pesos. Ela cai na
//! **lei derivada**, que é onde já estava — e a tabela vazia diz isso em voz alta, pela mesma porta
//! que uma imagem sem pesos usa. *Recusar é a resposta honesta; inventar um domínio para uma linha
//! seria inventar uma arte que o artista não desenhou.*

use ph2d_skin_weights::Handle;
use ph2d_vec_scene::VecPath;

/// Quantos pontos por segmento de cúbica ao achatar o contorno — o mesmo número que a
/// `ph2d_vec_scene::boundary` usa, para a fronteira do domínio ser a que o resto do app já enxerga.
const AMOSTRAS: usize = 16;

/// ⭐ **Quantos triângulos a malha do bind de um caminho tem.**
///
/// ⚠️ **Menos que os `3 000` de uma imagem, e o motivo é o que se mede sobre ela:** aqui os pesos
/// são amostrados em **pontos de controlo** (dezenas), não em cada vértice desenhado (milhares) —
/// a malha é um andaime que morre no fim do bind, e refiná-la só paga a solução, nunca o desenho.
const ALVO_DE_TRIANGULOS: usize = 1_200;

/// ⭐⭐⭐ **OS PESOS DE UMA FORMA VECTORIAL, pelo padrão-ouro** — um por ponto de controlo, achatado.
///
/// A ordem é a de [`VecPath::for_each_vert_mut`] — para o vértice `k`, as três casas
/// `3k`, `3k+1`, `3k+2` são **âncora**, **alça de entrada** e **alça de saída**, cada uma com
/// `ossos` pesos. ⚠️ Ela é estável porque o caminho GUARDADO é que a define, e ele não muda depois
/// do bind.
///
/// `ossos` são os eixos no espaço LOCAL do caminho (o mesmo em que os vértices vivem).
///
/// `None` quando não há domínio (caminho aberto, área nula) ou quando o solver não responde — ⛔ nos
/// dois casos a resposta certa é *não sei*, e quem chama cai na lei derivada.
#[must_use]
pub fn pesos_do_caminho(path: &VecPath, ossos: &[Handle]) -> Option<Vec<f64>> {
    if ossos.is_empty() {
        return None;
    }
    let aneis = contornos_fechados(path);
    if aneis.is_empty() {
        return None;
    }
    let (malha, para_malha) = malha_do_dominio(&aneis, ossos)?;
    let handles: Vec<Handle> = ossos
        .iter()
        .map(|o| Handle {
            a: para_malha(o.a),
            b: para_malha(o.b),
        })
        .collect();
    let w = ph2d_skin_weights::bounded_biharmonic(
        &malha,
        &handles,
        ph2d_skin_weights::Options::default(),
    )?;
    let n = ossos.len();
    let mut out = Vec::new();
    let mut faltou = 0usize;
    for v in path.verts_all() {
        for p in [v.anchor, v.in_handle, v.out_handle] {
            match amostra(&malha, &w.por_vertice, para_malha(p), n) {
                Some(ws) => out.extend_from_slice(&ws),
                None => {
                    faltou += 1;
                    // ⚠️ **Uma ALÇA pode viver FORA da forma** (ela é uma tangente, não um ponto do
                    // desenho), e ali não há domínio. A resposta é o vértice da malha mais próximo —
                    // ⛔ nunca zeros, que a normalização a jusante leria como *«este ponto não é de
                    // ninguém»* e entregaria ao primeiro osso.
                    out.extend_from_slice(&mais_proximo(&malha, &w.por_vertice, para_malha(p), n));
                }
            }
        }
    }
    if faltou > 0 {
        eprintln!(
            "[bone] {faltou} pontos de controlo caem FORA do interior da forma (alcas tangentes) — \
             cada um herdou os pesos do vertice mais proximo da malha"
        );
    }
    Some(out)
}

/// Os contornos **fechados** do caminho cozido, achatados em polígonos de espaço LOCAL.
///
/// ⛔ Só os fechados: uma linha de construção não é fronteira de nada, e é essa a mesma regra que a
/// `ph2d_vec_scene::boundary::outline` já aplica.
fn contornos_fechados(path: &VecPath) -> Vec<Vec<[f64; 2]>> {
    let cozido = path.cooked();
    let mut out = Vec::new();
    for c in 0..cozido.contour_count() {
        let Some((verts, closed)) = cozido.contour(c) else {
            continue;
        };
        if !closed || verts.len() < 2 {
            continue;
        }
        let mut poly = Vec::with_capacity(verts.len() * AMOSTRAS);
        for i in 0..verts.len() {
            let a = &verts[i];
            let b = &verts[(i + 1) % verts.len()];
            for k in 0..AMOSTRAS {
                let t = k as f64 / AMOSTRAS as f64;
                poly.push(cubica(a.anchor, a.out_handle, b.in_handle, b.anchor, t));
            }
        }
        if ph2d_poly2d::signed_area(&poly).abs() > f64::EPSILON {
            out.push(poly);
        }
    }
    out
}

/// Um ponto da cúbica em `t`.
fn cubica(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        w0 * p0[0] + w1 * p1[0] + w2 * p2[0] + w3 * p3[0],
        w0 * p0[1] + w1 * p1[1] + w2 * p2[1] + w3 * p3[1],
    ]
}

/// `true` quando `p` está DENTRO dos anéis, pela regra par-ímpar.
///
/// ⭐ **Par-ímpar e não *nonzero*, e a diferença é um FURO:** um caminho composto escreve o anel de
/// dentro com a mesma orientação do de fora tantas vezes quanto o artista quiser, e é a paridade que
/// faz o furo ser furo. *Um domínio que tape o furo dá peso a uma região que não é arte.*
fn dentro(aneis: &[Vec<[f64; 2]>], p: [f64; 2]) -> bool {
    let mut cruz = 0usize;
    for anel in aneis {
        for i in 0..anel.len() {
            let (a, b) = (anel[i], anel[(i + 1) % anel.len()]);
            if (a[1] > p[1]) != (b[1] > p[1]) {
                let x = (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0];
                if x > p[0] {
                    cruz += 1;
                }
            }
        }
    }
    cruz % 2 == 1
}

/// A malha do domínio e a régua `local → malha`.
///
/// ⚠️ **A malha vive em coordenadas próprias, com a origem no canto da caixa** — a `ph2d_poly2d`
/// trabalha em pixels de imagem (`y` para baixo, origem no canto), e traduzir na fronteira é mais
/// barato que ensinar a grelha a viver noutro referencial.
fn malha_do_dominio(
    aneis: &[Vec<[f64; 2]>],
    ossos: &[Handle],
) -> Option<(ph2d_poly2d::Mesh2d, impl Fn([f64; 2]) -> [f64; 2] + use<>)> {
    let mut caixa = [
        f64::INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::NEG_INFINITY,
    ];
    for p in aneis.iter().flatten() {
        caixa[0] = caixa[0].min(p[0]);
        caixa[1] = caixa[1].min(p[1]);
        caixa[2] = caixa[2].max(p[0]);
        caixa[3] = caixa[3].max(p[1]);
    }
    let (larg, alt) = (caixa[2] - caixa[0], caixa[3] - caixa[1]);
    if !(larg > 0.0 && alt > 0.0) {
        return None;
    }
    // A malha é indexada em «pixels» de uma caixa de `LADO` de lado maior — a grelha é adimensional
    // e o orçamento é em triângulos, então a escala só tem de ser a mesma nos dois eixos.
    const LADO: f64 = 512.0;
    let escala = LADO / larg.max(alt);
    let para_malha = move |p: [f64; 2]| [(p[0] - caixa[0]) * escala, (p[1] - caixa[1]) * escala];
    let aneis_malha: Vec<Vec<[f64; 2]>> = aneis
        .iter()
        .map(|a| a.iter().map(|&p| para_malha(p)).collect())
        .collect();
    let focos: Vec<[f64; 2]> = ossos
        .iter()
        .flat_map(|o| [para_malha(o.a), para_malha(o.b)])
        .collect();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a caixa foi escalada para um lado maior de 512"
    )]
    let (w, h) = (
        (larg * escala).ceil() as u32 + 1,
        (alt * escala).ceil() as u32 + 1,
    );
    let malha = ph2d_poly2d::grid_mesh_com(
        &|x0, y0, x1, y1, folga| {
            // A célula tem arte se o centro OU algum canto (com a folga) estiver dentro — cinco
            // amostras, que é o que separa uma ponta fina de uma célula vazia.
            let (a, b) = (x0 - folga, y0 - folga);
            let (c, d) = (x1 + folga, y1 + folga);
            [
                [(a + c) / 2.0, (b + d) / 2.0],
                [a, b],
                [c, b],
                [c, d],
                [a, d],
            ]
            .iter()
            .any(|&p| dentro(&aneis_malha, p))
        },
        w,
        h,
        &focos,
        ph2d_poly2d::GridOptions {
            target_tris: ALVO_DE_TRIANGULOS,
            ..ph2d_poly2d::GridOptions::default()
        },
    )?;
    Some((malha, para_malha))
}

/// Os pesos num ponto qualquer, por coordenadas baricêntricas. `None` fora da malha.
fn amostra(
    m: &ph2d_poly2d::Mesh2d,
    por_vertice: &[Vec<f64>],
    p: [f64; 2],
    n: usize,
) -> Option<Vec<f64>> {
    for t in &m.tris {
        let (a, b, c) = (
            m.rest[t[0] as usize],
            m.rest[t[1] as usize],
            m.rest[t[2] as usize],
        );
        let den = (b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]);
        if den.abs() < 1e-12 {
            continue;
        }
        let u = ((p[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (p[1] - a[1])) / den;
        let v = ((b[0] - a[0]) * (p[1] - a[1]) - (p[0] - a[0]) * (b[1] - a[1])) / den;
        if u < -1e-9 || v < -1e-9 || u + v > 1.0 + 1e-9 {
            continue;
        }
        let (pa, pb, pc) = (
            &por_vertice[t[0] as usize],
            &por_vertice[t[1] as usize],
            &por_vertice[t[2] as usize],
        );
        return Some(
            (0..n)
                .map(|k| (1.0 - u - v) * pa[k] + u * pb[k] + v * pc[k])
                .collect(),
        );
    }
    None
}

/// Os pesos do vértice da malha mais próximo — a resposta para uma alça que vive fora da forma.
fn mais_proximo(
    m: &ph2d_poly2d::Mesh2d,
    por_vertice: &[Vec<f64>],
    p: [f64; 2],
    n: usize,
) -> Vec<f64> {
    let mut melhor = (f64::INFINITY, 0usize);
    for (i, q) in m.rest.iter().enumerate() {
        let d = (q[0] - p[0]).hypot(q[1] - p[1]);
        if d < melhor.0 {
            melhor = (d, i);
        }
    }
    por_vertice
        .get(melhor.1)
        .cloned()
        .unwrap_or_else(|| vec![1.0 / n as f64; n])
}

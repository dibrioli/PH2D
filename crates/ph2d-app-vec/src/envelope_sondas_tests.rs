//! ⭐⭐ **AS SONDAS DO ENVELOPE** — *«como é que o `Effects: Arc` com tão poucos pontos fica tão
//! perfeito?»* (report do dono, 2026-09-20, com a estrela de `10` nós na foto).
//!
//! Elas IMPRIMEM; os gates são o irmão [`super::tests`].

use super::*;
use ph2d_ecs::{EnvelopeWarp, VecEnvelope};
use ph2d_vec_envelope::{CoonsWarp, Warp};
use ph2d_vec_scene::{ShapeKind, cook};

/// Quantos pontos de controlo um caminho tem, somando todos os contornos.
fn nos(p: &VecPath) -> usize {
    p.verts_all().count()
}

/// O maior afastamento GEOMÉTRICO de `saida` à curva VERDADEIRA — a imagem exacta da fonte pelo
/// mapa, `W(C(t))`, amostrada densa.
///
/// ⚠️ **Geométrico e não no mesmo `t`**, que é a 1.ª das duas disciplinas de oráculo que o
/// `split_invariance.rs` desta casa escreve: comparar `A(t)` com `B(t)` mede deriva de
/// PARAMETRIZAÇÃO, que é invisível na tela.
fn desvio_a_verdade(fonte: &VecPath, saida: &VecPath, warp: &impl Warp) -> f64 {
    let verdade: Vec<[f64; 2]> = amostra(fonte, 128).iter().map(|&p| warp.map(p)).collect();
    let nossa = amostra(saida, 128);
    verdade
        .iter()
        .map(|&q| dist_a_polilinha(q, &nossa))
        .fold(0.0_f64, f64::max)
}

/// A distância de `q` à POLILINHA `poli` — ponto a SEGMENTO, nunca ponto a ponto.
///
/// ⛔⛔ **A 1.ª redacção media ponto a PONTO e lia o próprio espaçamento da amostragem:** ela deu
/// `1,28` a `3,19` numa peça de `400` com a tolerância pedida em `0,46`, e a conclusão seria *«o
/// Arc não cumpre a tolerância dele»* — que é falso. *Uma régua de distância a um conjunto
/// DISCRETO tem o espaçamento dele como piso.*
fn dist_a_polilinha(q: [f64; 2], poli: &[[f64; 2]]) -> f64 {
    let n = poli.len();
    (0..n)
        .map(|i| {
            let (a, b) = (poli[i], poli[(i + 1) % n]);
            let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
            let l2 = dx.mul_add(dx, dy * dy);
            let t = if l2 > 0.0 {
                (((q[0] - a[0]) * dx + (q[1] - a[1]) * dy) / l2).clamp(0.0, 1.0)
            } else {
                0.0
            };
            (t.mul_add(dx, a[0]) - q[0]).hypot(t.mul_add(dy, a[1]) - q[1])
        })
        .fold(f64::INFINITY, f64::min)
}

/// Amostra `n` pontos por segmento de todo contorno.
fn amostra(p: &VecPath, n: usize) -> Vec<[f64; 2]> {
    let c = p.cooked();
    let mut out = Vec::new();
    for k in 0..c.contour_count() {
        let Some((v, fechado)) = c.contour(k) else {
            continue;
        };
        let m = v.len();
        let segs = if fechado { m } else { m.saturating_sub(1) };
        for s in 0..segs {
            let (a, b) = (&v[s], &v[(s + 1) % m]);
            for i in 0..n {
                #[expect(clippy::cast_precision_loss, reason = "i < n, um punhado")]
                let t = i as f64 / n as f64;
                let u = 1.0 - t;
                let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
                out.push([
                    w3.mul_add(
                        b.anchor[0],
                        w2.mul_add(
                            b.in_handle[0],
                            w1.mul_add(a.out_handle[0], w0 * a.anchor[0]),
                        ),
                    ),
                    w3.mul_add(
                        b.anchor[1],
                        w2.mul_add(
                            b.in_handle[1],
                            w1.mul_add(a.out_handle[1], w0 * a.anchor[1]),
                        ),
                    ),
                ]);
            }
        }
    }
    out
}

/// A lei INGÉNUA — mover os pontos de controlo, que é o que a pele do esqueleto faz e o que o
/// cabeçalho da [`ph2d_vec_envelope`] proíbe por escrito. É o CONTROLO desta sonda.
fn ingenua(fonte: &VecPath, warp: &impl Warp) -> VecPath {
    let mut out = fonte.clone();
    out.for_each_vert_mut(|v| {
        for p in [&mut v.anchor, &mut v.in_handle, &mut v.out_handle] {
            *p = warp.map(*p);
        }
    });
    out
}

/// ⭐⭐⭐ **SONDA — O ARC COM POUCOS PONTOS: quantos entram, quantos SAEM, e a que distância da
/// verdade.**
#[test]
fn diag_e_o_arc_com_poucos_pontos() {
    println!("\n{:=<96}", "");
    println!("SONDA · O `Effects: Arc` — o que ENTRA, o que SAI e o desvio à curva verdadeira");
    println!("{:=<96}", "");
    println!(
        "{:>22} | {:>5} {:>6} | {:>11} | {:>12} {:>12}",
        "forma", "entra", "sai", "tolerância", "desvio ARC", "desvio ingén."
    );
    for (nome, fonte) in [
        (
            "estrela de 5 pontas",
            cook(
                ShapeKind::Star,
                [0.0, 0.0],
                [400.0, 400.0],
                &[5.0, 0.4, 0.0, 0.0],
            ),
        ),
        (
            "rectângulo (4 nós)",
            cook(ShapeKind::Rectangle, [0.0, 0.0], [400.0, 400.0], &[]),
        ),
        (
            "elipse (4 nós)",
            cook(ShapeKind::Ellipse, [0.0, 0.0], [400.0, 400.0], &[]),
        ),
    ] {
        let entra = nos(&fonte);
        let (mut sim, mut scene, _m, bits, ids) = super::tests::envelope_over(vec![fonte.clone()]);
        // O `Bend` da foto do dono: `51,22` no painel ⇒ `0,5122` na faixa `[-1, 1]`.
        super::apply_preset(&mut sim, bits, EnvelopeWarp::Arc, 0.5122);
        super::recook(&mut sim, &mut scene);
        let saida = scene
            .paths()
            .iter()
            .find(|p| p.id == ids[0])
            .expect("o filho")
            .clone();
        // A VERDADE: o mesmo mapa que o recook montou, relido do mundo.
        let env = {
            let mut q = sim.world_mut().query::<&VecEnvelope>();
            q.iter(sim.world()).next().cloned().expect("envelope")
        };
        let src = postcard::from_bytes::<VecPath>(&env.children[0].source).expect("fonte");
        let (origem, tam) = super::union_control_bbox(std::iter::once(&src)).expect("bbox");
        let warp = CoonsWarp::new(origem, tam, env.corners, &env.edges).expect("mapa");
        let tol = 0.001 * tam[0].hypot(tam[1]).max(1.0);
        println!(
            "{nome:>22} | {entra:>5} {:>6} | {tol:>11.4} | {:>12.6} {:>12.6}",
            nos(&saida),
            desvio_a_verdade(&src, &saida, &warp),
            desvio_a_verdade(&src, &ingenua(&src, &warp), &warp),
        );
    }
    println!("{:=<96}", "");
}

/// ⭐⭐⭐ **SONDA — A CONTAGEM DE PONTOS DO ARC AO LONGO DO CURSO DO SLIDER.**
///
/// A pergunta que separa o Arc da pele: *o refit muda a representação enquanto a mão arrasta?*
#[test]
fn diag_e_a_contagem_do_arc_ao_longo_do_slider() {
    let fonte = cook(
        ShapeKind::Star,
        [0.0, 0.0],
        [400.0, 400.0],
        &[5.0, 0.4, 0.0, 0.0],
    );
    println!("\n{:=<72}", "");
    println!("SONDA · quantos pontos o ARC emite ao longo do curso do `Bend`");
    println!("{:=<72}", "");
    let (mut sim, mut scene, _m, bits, ids) = super::tests::envelope_over(vec![fonte]);
    let mut linha = String::new();
    let mut ant = 0usize;
    let mut saltos = 0usize;
    for i in 0..=40 {
        let bend = f64::from(i) / 40.0;
        super::apply_preset(&mut sim, bits, EnvelopeWarp::Arc, bend);
        super::recook(&mut sim, &mut scene);
        let n = nos(scene
            .paths()
            .iter()
            .find(|p| p.id == ids[0])
            .expect("filho"));
        if i > 0 && n != ant {
            saltos += 1;
        }
        ant = n;
        if i % 4 == 0 {
            linha.push_str(&format!("  bend {bend:.2} → {n:>3} nós\n"));
        }
    }
    print!("{linha}");
    println!("  ⇒ a contagem MUDOU {saltos} vezes em 40 passos do slider");
    println!("{:=<72}", "");
}

/// ⭐⭐⭐ **SONDA — QUANTOS PONTOS O CAMINHO INGÉNUO PRECISA PARA IGUALAR O ARC.**
///
/// A lei ingénua (mover os pontos de controlo) é a da pele do esqueleto. Ela **não pode acrescentar
/// pontos** — o que ela pode é receber uma fonte já mais subdividida, que é o que o `Bind` faz.
/// Esta sonda responde à conta: *para chegar onde o Arc chega com `26`, de quantos ela precisa?*
#[test]
fn diag_e_quantos_pontos_a_lei_ingenua_precisa() {
    let base = cook(
        ShapeKind::Star,
        [0.0, 0.0],
        [400.0, 400.0],
        &[5.0, 0.4, 0.0, 0.0],
    );
    let (mut sim, mut scene, _m, bits, ids) = super::tests::envelope_over(vec![base]);
    super::apply_preset(&mut sim, bits, EnvelopeWarp::Arc, 0.5122);
    super::recook(&mut sim, &mut scene);
    let arc = scene
        .paths()
        .iter()
        .find(|p| p.id == ids[0])
        .expect("filho")
        .clone();
    let env = {
        let mut q = sim.world_mut().query::<&VecEnvelope>();
        q.iter(sim.world()).next().cloned().expect("envelope")
    };
    let src = postcard::from_bytes::<VecPath>(&env.children[0].source).expect("fonte");
    let (origem, tam) = super::union_control_bbox(std::iter::once(&src)).expect("bbox");
    let warp = CoonsWarp::new(origem, tam, env.corners, &env.edges).expect("mapa");
    let tol = 0.001 * tam[0].hypot(tam[1]).max(1.0);
    let alvo = desvio_a_verdade(&src, &arc, &warp);

    println!("\n{:=<80}", "");
    println!("SONDA · quantos pontos a LEI INGÉNUA precisa para igualar o ARC");
    println!(
        "  o ARC: {} nós, desvio {alvo:.6} (tolerância pedida {tol:.4})",
        nos(&arc)
    );
    println!("{:=<80}", "");
    println!("{:>9} {:>7} | {:>12}", "rondas", "nós", "desvio");
    let mut cur = src.clone();
    for ronda in 0..=5 {
        if ronda > 0 {
            // Parte TODO segmento ao meio. ⚠️ De trás para a frente: partir insere um vértice e
            // renumera os segmentos à frente dele.
            let segs = cur.verts_all().count();
            for k in (0..segs).rev() {
                ph2d_vec_scene::split_segment(&mut cur, k, 0.5);
            }
        }
        let d = desvio_a_verdade(&src, &ingenua(&cur, &warp), &warp);
        println!(
            "{ronda:>9} {:>7} | {d:>12.6} {}",
            nos(&cur),
            if d <= alvo { "← iguala o ARC" } else { "" }
        );
        if d <= alvo {
            break;
        }
    }
    println!("{:=<80}", "");
}

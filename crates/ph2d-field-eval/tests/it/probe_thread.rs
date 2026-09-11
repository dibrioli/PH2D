//! Sonda (`#[ignore]`) da W135: a rosca entrega o perfil que promete? o divisor segura a família?
//! e onde é que o preço do quadro deixa de pagar mais uma entrada?
use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform, thread_depth_ceiling};
use ph2d_field_eval::{Field, ops_thread};

/// O raio da SUPERFÍCIE que o perfil autorado promete, no `w` dado.
///
/// ⚠️ **É o oráculo, e ele não partilha uma linha com o produto** — sai da geometria escrita no
/// painel (`núcleo`, `depth`, `α`), e não da árvore.
fn raio_prometido(w: f64, nucleo: f64, depth: f64, alpha: f64) -> f64 {
    let a = depth * alpha.tan();
    nucleo + depth * (1.0 - (w.abs() / a).min(1.0))
}

/// O primeiro cruzamento da superfície ao andar do eixo para fora, em passos pequenos.
///
/// ⚠️ **Passo a passo, e não bissecção de intervalo largo** — foi a lição que a W134 pagou: com um
/// intervalo largo a bissecção agarra o fio VIZINHO e devolve um defeito numa peça perfeita.
fn primeiro_cruzamento(f: &Field, phi: f64, z: f64, r0: f64, r1: f64) -> Option<f64> {
    let n = 4000;
    let (mut anterior, mut r_ant) = (f.at(r0 * phi.cos(), r0 * phi.sin(), z), r0);
    for i in 1..=n {
        let r = r0 + (r1 - r0) * (i as f64) / (n as f64);
        let v = f.at(r * phi.cos(), r * phi.sin(), z);
        if anterior < 0.0 && v >= 0.0 {
            // interpolação linear entre os dois passos
            return Some(r_ant + (r - r_ant) * (-anterior) / (v - anterior));
        }
        anterior = v;
        r_ant = r;
    }
    None
}

fn campo(radius: f64, half: f64, pitch: f64, depth: f64, flank: f64, s: u32, h: u32) -> Field {
    Field::from_tree(&ops_thread::sd_thread(
        radius, half, pitch, depth, flank, s, h, 0.0, 0.0,
    ))
}

/// O pior `‖∇f‖` da caixa e o pior junto da pele — as duas perguntas são diferentes (ver a sonda do
/// nó): um gradiente alto longe da peça custa passos, um junto da pele custa buracos.
fn pior_gradiente(f: &Field, e: f64) -> (f64, f64, [f64; 3]) {
    let (mut caixa, mut banda, mut onde) = (0.0_f64, 0.0_f64, [0.0; 3]);
    let mut varre = |n: usize, e: f64, so_pele: bool| {
        let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / n as f64;
        for i in 0..n {
            for j in 0..n {
                for k in 0..n {
                    let (x, y, z) = (at(i), at(j), at(k));
                    let v = f.at(x, y, z);
                    if !v.is_finite() || (so_pele && v.abs() > 0.01) {
                        continue;
                    }
                    let g = f.gradient_norm(x, y, z, 1.0e-5);
                    if !g.is_finite() {
                        continue;
                    }
                    if so_pele {
                        banda = banda.max(g);
                    }
                    if g > caixa {
                        caixa = g;
                        onde = [x, y, z];
                    }
                }
            }
        }
    };
    varre(34, e, false);
    varre(110, e * 0.98, true);
    (caixa, banda, onde)
}

/// ⭐ **A SECÇÃO** — a superfície entregue contra o triângulo autorado, ao longo de um período.
#[test]
#[ignore = "sonda"]
fn probe_thread_section() {
    println!("perfil entregue contra o autorado (erro em % da profundidade)");
    for (nome, pitch, flank, starts) in [
        ("fino  s=1", 0.045_f64, 30.0_f64, 1_u32),
        ("grosso s=1", 0.090, 30.0, 1),
        ("s=4", 0.090, 30.0, 4),
        ("s=12", 0.090, 30.0, 12),
        ("flanco 60", 0.090, 60.0, 1),
        ("flanco 10", 0.030, 10.0, 1),
    ] {
        let radius = 0.17_f64;
        #[allow(clippy::cast_possible_truncation)]
        let depth = f64::from(thread_depth_ceiling(
            radius as f32,
            pitch as f32,
            flank as f32,
        )) * 0.75;
        let nucleo = radius - depth;
        let alpha = flank.to_radians();
        let b = f64::from(starts) * pitch / std::f64::consts::TAU;
        let f = campo(radius, 0.24, pitch, depth, flank, starts, 1);
        let (mut pior, mut soma, mut n) = (0.0_f64, 0.0_f64, 0_u32);
        for iw in 0..40 {
            for iphi in 0..7 {
                let phi = -2.6 + 5.2 * f64::from(iphi) / 6.0;
                let w = -pitch * 0.5 + pitch * f64::from(iw) / 39.0;
                let z = w + b * phi;
                if z.abs() > 0.18 {
                    continue;
                }
                let Some(r) = primeiro_cruzamento(&f, phi, z, nucleo * 0.8, radius * 1.4) else {
                    continue;
                };
                let erro = (r - raio_prometido(w, nucleo, depth, alpha)) / depth * 100.0;
                pior = pior.max(erro.abs());
                soma += erro.abs();
                n += 1;
            }
        }
        println!(
            "  {nome:11}  pior {pior:7.3} %   medio {:7.3} %   ({n} amostras)",
            soma / f64::from(n)
        );
    }
}

/// ⭐ **O DIVISOR** — `‖∇f‖` sobre a região permitida de `(starts, hands, flanco, profundidade)`.
#[test]
#[ignore = "sonda"]
fn probe_thread_grad() {
    println!("starts hands flanco fraccao |  grad caixa   grad pele");
    for starts in [1_u32, 4, 16, 32, 64, 96] {
        for hands in [1_u32, 2] {
            for flank in [10.0_f64, 30.0, 60.0] {
                for fraccao in [0.30_f64, 0.99] {
                    let (radius, pitch) = (0.17_f64, 0.09_f64);
                    #[allow(clippy::cast_possible_truncation)]
                    let depth = f64::from(thread_depth_ceiling(
                        radius as f32,
                        pitch as f32,
                        flank as f32,
                    )) * fraccao;
                    let f = campo(radius, 0.24, pitch, depth, flank, starts, hands);
                    let (caixa, pele, onde) = pior_gradiente(&f, 0.30);
                    let alarme = if pele > 1.0001 { " <<<" } else { "" };
                    println!(
                        "  {starts:2}    {hands}    {flank:4.0}    {fraccao:.2}  |  {caixa:8.4}  \
                         {pele:8.4}  em ({:.3},{:.3},{:.3}){alarme}",
                        onde[0], onde[1], onde[2]
                    );
                }
            }
        }
    }
}

/// ⭐ **O PREÇO** — quanto custa um quadro por número de entradas e de mãos.
#[test]
#[ignore = "sonda"]
fn probe_thread_price() {
    let doc = |starts: u32, hands: u32| {
        let (radius, pitch, flank) = (0.5_f32, 0.16_f32, 30.0_f32);
        Primitive::Thread {
            radius,
            half_height: 0.45,
            pitch,
            depth: thread_depth_ceiling(radius, pitch, flank) * 0.75,
            flank,
            starts,
            hands,
            round: 0.0,
            chamfer: 0.0,
        }
    };
    println!("(a contagem de nos da arvore, que e o que o preco do quadro segue)");
    for starts in [1_u32, 2, 4, 8, 12, 16, 24] {
        for hands in [1_u32, 2] {
            let p = doc(starts, hands);
            let arvore = FieldDoc::new(
                vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
                NodeId(0),
            )
            .expect("a peça");
            let t = std::time::Instant::now();
            let f = Field::new(&arvore);
            let montagem = t.elapsed();
            let t = std::time::Instant::now();
            let mut soma = 0.0;
            for i in 0..40_000 {
                let a = f64::from(i) * 0.000_37;
                soma += f.at(a.sin() * 0.4, a.cos() * 0.4, (a * 0.7).sin() * 0.4);
            }
            let amostra = t.elapsed();
            println!(
                "  starts {starts:2}  hands {hands}  montagem {:6.2} us  40k amostras {:7.2} ms \
                 ({:5.1} ns/ponto)  [{soma:.1}]",
                montagem.as_secs_f64() * 1.0e6,
                amostra.as_secs_f64() * 1.0e3,
                amostra.as_secs_f64() * 1.0e9 / 40_000.0
            );
        }
    }
}

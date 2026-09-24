//! ⏱️⭐⭐⭐⭐ **A OCLUSÃO DO CÉU MARCHADA NUMA GRADE ASSADA** — a sonda que decide se ela paga
//! (`docs/Render3d/03_o_plano.md` §W9, «a oclusão na grade»).
//!
//! Medido antes dela: no modo Render a mexer, a oclusão é `60`–`80 %` do quadro (`48` cones por
//! pixel, cada um uma marcha sobre a árvore inteira). A pergunta: marchar os cones numa grade
//! assada na placa — o *Distance Field Ambient Occlusion* dos motores de jogo — compra quanto, e
//! quanto move a imagem? O raio primário continua EXACTO (a silhueta e a normal são as da árvore);
//! só a luz do céu muda.

/// ⏱️ **Sonda: relógio e imagem, oclusão exacta contra grade de `n`**, pela porta do produto
/// (`paint_com`, o quadro de movimento). A imagem compara-se byte a byte com a exacta.
#[test]
#[ignore = "sonda de GPU"]
fn diag_a_oclusao_na_grade() {
    const W: u32 = 1920;
    const H: u32 = 1080;
    let cam = ph2d_field_render::Orbit::default();
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    let materiais = [ph2d_material::OpenPbr::default().prepare()];
    let surfaces = ph2d_field_render::Surfaces {
        all: &materiais,
        owners: None,
    };
    let olhar = ph2d_view_transform::Look::default();
    let pres = ph2d_field_render::Presentation::of(olhar);
    let luz = [crate::gpu_frame::tests_lampada(&cam)];
    println!(
        "\n  {}\n  cena · grade · ms [mín 5] · píxeis >2 B · >8 B · p99 B · máx B",
        super::super::super::super::contexto()
    );
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let quadro = |ceu: Option<u32>| {
            let sonda = crate::gpu_frame::Sonda {
                ceu_na_grade: ceu,
                ..crate::gpu_frame::Sonda::default()
            };
            crate::gpu_frame::paint_com(
                t, &doc, &reg, &cam, &luz, &surfaces, &pres, [0, 0, 0, 0], None, W, H, false,
                sonda,
            )
            .expect("o pintor")
            .rgba
        };
        let relogio = |ceu: Option<u32>| {
            let _ = quadro(ceu);
            let mut m = f64::INFINITY;
            for _ in 0..5 {
                let t0 = std::time::Instant::now();
                let _ = quadro(ceu);
                m = m.min(t0.elapsed().as_secs_f64() * 1e3);
            }
            m
        };
        let exacta = quadro(None);
        println!("  {cena:4} ·  exacta · {:>7.2}", relogio(None));
        for res in [128u32, 192, 256, 384] {
            let img = quadro(Some(res));
            let mut d: Vec<u8> = exacta
                .as_chunks::<4>()
                .0
                .iter()
                .zip(img.as_chunks::<4>().0)
                .map(|(a, b)| (0..3).map(|c| a[c].abs_diff(b[c])).max().unwrap_or(0))
                .collect();
            let acima = |b: u8| d.iter().filter(|x| **x > b).count();
            let (a2, a8) = (acima(2), acima(8));
            d.sort_unstable();
            let p99 = d[d.len() * 99 / 100];
            let max = d.last().copied().unwrap_or(0);
            println!(
                "  {cena:4} · {res:>6} · {:>7.2} · {a2:>8} · {a8:>6} · {p99:>5} · {max:>5}",
                relogio(Some(res))
            );
        }
    }
}

/// ⏱️ **Sonda: o CANAL do céu, exacto contra a grade** — o G-buffer volta pela `frame`, e compara-se
/// o valor da oclusão pixel a pixel (média com sinal, fracção mais escura, p99 do módulo).
#[test]
#[ignore = "sonda de GPU"]
#[allow(clippy::cast_precision_loss)]
fn diag_o_canal_do_ceu_na_grade() {
    const W: u32 = 480;
    const H: u32 = 270;
    let cam = ph2d_field_render::Orbit::default();
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    let luz = [crate::gpu_frame::tests_lampada(&cam).world];
    println!("\n  cena · grade · acertos · média(grade−exacta) · fracção mais escura · p99 |Δ| · máx |Δ|");
    for cena in [26u32, 5, 28] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let ceu = |g: Option<u32>| {
            let sonda = crate::gpu_frame::Sonda {
                ceu_na_grade: g,
                ..crate::gpu_frame::Sonda::default()
            };
            let (c, f, setup) = crate::gpu_frame::pedido(
                &doc,
                &reg,
                &cam,
                &luz,
                None,
                ph2d_field_gpu::trace::MAX_LAMPS,
                sonda,
                W,
                H,
                None,
                true,
            )
            .expect("o pedido");
            let gb = t.lock().expect("o traçador").frame(&f, c.sculpts(), setup, W, H);
            (gb.t, gb.ambient)
        };
        let (tt, exacta) = ceu(None);
        for res in [32u32, 128, 512] {
            let (_, grade) = ceu(Some(res));
            let mut d: Vec<f32> = Vec::new();
            let mut soma = 0.0f64;
            let mut escura = 0usize;
            for i in 0..exacta.len() {
                if tt[i] < 0.0 {
                    continue;
                }
                let x = grade[i] - exacta[i];
                soma += f64::from(x);
                if x < -1e-3 {
                    escura += 1;
                }
                d.push(x.abs());
            }
            let n = d.len().max(1);
            d.sort_by(f32::total_cmp);
            println!(
                "  {cena:4} · {res:>5} · {n:>7} · {:>+9.4} · {:>6.3} · {:>7.4} · {:>7.4}",
                soma / n as f64,
                escura as f64 / n as f64,
                d[n * 99 / 100],
                d.last().copied().unwrap_or(0.0)
            );
        }
    }
}

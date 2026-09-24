//! ⏱️⭐⭐⭐⭐ **A GRADE DE LONGE e o RECORTE DA CAIXA** — as sondas da recusa medida.
//!
//! ⛔ **A grade de longe foi construída, medida e RECUSADA** (`docs/Render3d/03_o_plano.md` §W9,
//! «a grade de longe»): ela não compra nada sobre o recorte da caixa sozinho, e o recorte sozinho é o
//! que shipa. Estas sondas são o instrumento da recusa, e é por isso que ficam — a grade continua
//! alcançável por `PH2D_FIELD_LONGE=<n>`, e é aqui que se volta a medi-la.
//!
//! ⚠️ **Filha do módulo da grade assada** e não irmã: ela usa a `Grade` e a interpolação dele, que
//! são privadas, e um filho vê os itens privados do pai sem que eles passem a ser API.

use super::Grade;

/// ⏱️⭐⭐⭐⭐ **Sonda: a GRADE DE LONGE no produto** — ver [`ph2d_field_gpu::longe`].
///
/// A mesma imagem do modo de omissão (o matcap, `1920×1080`) com a grade desligada e em três
/// resoluções: o relógio da placa (mínimo de 4) e **quanto da IMAGEM muda** — píxeis que diferem
/// mais de `1` byte e o pior byte. ⚠️ A imagem é a régua porque a lei promete a MESMA superfície
/// (o raio só pára pela árvore); a coluna que mediria outra coisa é o relógio.
///
/// ```text
/// PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-field3d --lib --release -- \
///   --ignored --exact preview::device_tests::sondas_w9::grade::longe::diag_a_grade_de_longe --nocapture
/// ```
#[test]
#[ignore = "sonda de relógio; precisa de adaptador e de máquina calma"]
fn diag_a_grade_de_longe() {
    let reg = crate::smoke::sampled_registry();
    let cam = ph2d_field_render::Orbit::default();
    const W: u32 = 1920;
    const H: u32 = 1080;
    let (lado, foto) = crate::smoke::matcap_para_sonda();
    let olhar = ph2d_view_transform::Look::default();
    let mc = ph2d_field_gpu::matcap::MatcapSetup {
        rgb_linear: &foto,
        side: lado,
        chave: 1,
        stops: olhar.exposure_stops,
        view: ph2d_view_transform::wgsl::view_code(olhar.view),
        background: [0, 0, 0, 0],
    };
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    let quadro = |doc: &ph2d_field::FieldDoc, res: Option<u32>| {
        let sonda = crate::gpu_frame::Sonda {
            longe: res,
            ..crate::gpu_frame::Sonda::default()
        };
        let mut melhor = f32::INFINITY;
        let mut img = Vec::new();
        for _ in 0..=super::super::super::QUADROS_MEDIDOS {
            let t0 = std::time::Instant::now();
            let p = crate::gpu_frame::pinta_matcap_com(t, doc, &reg, &cam, &mc, W, H, sonda)
                .expect("o quadro");
            #[allow(clippy::cast_possible_truncation)]
            let ms = t0.elapsed().as_secs_f32() * 1e3;
            melhor = melhor.min(ms);
            img = p.rgba;
        }
        (melhor, img)
    };
    println!(
        "\n  {}\n  cena · longe · placa ms · píxeis que mudam (>1 byte) · pior byte",
        super::super::super::contexto()
    );
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let (ms0, ref_img) = quadro(&doc, None);
        println!("  {cena:>4} · árvore · {ms0:>7.2} · — · —");
        // ⚠️ `0` é SÓ o recorte pela caixa — separa o que o recorte compra do que a grade compra.
        for res in [0u32, 32, 64, 128] {
            let (ms, img) = quadro(&doc, Some(res));
            let mut mudam = 0usize;
            let mut pior = 0u8;
            for (a, b) in ref_img
                .as_chunks::<4>()
                .0
                .iter()
                .zip(img.as_chunks::<4>().0)
            {
                let d = a
                    .iter()
                    .zip(b)
                    .map(|(x, y)| x.abs_diff(*y))
                    .max()
                    .unwrap_or(0);
                pior = pior.max(d);
                if d > 1 {
                    mudam += 1;
                }
            }
            println!(
                "  {cena:>4} · {res:>6} · {ms:>7.2} · {mudam} · {pior}  ({:.2}×)",
                ms0 / ms
            );
        }
    }
    println!();
}

/// ⏱️⭐⭐⭐⭐ **Sonda: a grade de longe CONTRA A REFERÊNCIA** — quem está certo quando os dois
/// lados do dispositivo discordam na silhueta?
///
/// A mesma régua do gate `o_gbuffer_do_dispositivo_e_o_da_cpu` (silhueta em desacordo e `Δt`
/// p99 contra o traçado da CPU), corrida com a grade desligada e ligada. ⚠️ Se a grade só
/// mudasse pixels que a árvore já errava, a coluna «ligada» leria MENOS desacordo, não mais.
#[test]
#[ignore = "sonda: precisa de adaptador"]
fn diag_a_grade_de_longe_contra_a_referencia() {
    use ph2d_field_render::{Orbit, Screen, trace};
    const W: u32 = 384;
    const H: u32 = 216;
    let cam = Orbit::default();
    let (right, up, fwd) = cam.basis();
    let screen = Screen::new(W, H, cam.half_extent);
    let Some(t) = crate::gpu_frame::shared() else {
        println!("sem adaptador");
        return;
    };
    println!("\n  cena · longe · silhueta≠cpu · Δt p99 · Δt max");
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let Some(campo) = ph2d_field_eval::device::DeviceField::new(&doc, &reg) else {
            continue;
        };
        let Some(fita) = campo.tape_wgsl() else {
            continue;
        };
        let g = trace(&doc, &reg, &cam, W, H);
        let bola = ph2d_field_eval::bounds::bounding_ball(&doc, &reg)
            .unwrap_or(ph2d_field_eval::bounds::Ball::EMPTY);
        let passo = ph2d_field_eval::safe_march_step(&doc);
        let shrink = ph2d_field_eval::field_shrink(&doc, &reg);
        let sharp = ph2d_field_render::Sharpness::for_frame(cam.half_extent, W.min(H) as usize);
        // ⚠️ TRÊS colunas: sem nada · SÓ o recorte da caixa (`res = 0`) · o recorte e a grade.
        // A do meio separa o que a GRADE muda do que mudar o PONTO DE PARTIDA do raio muda.
        for res in [u32::MAX, 0, 64] {
            let longe = crate::gpu_frame::a_caixa_da_marcha(bola, (res != u32::MAX).then_some(res));
            let mut lamps = [[0.0f32; 3]; ph2d_field_gpu::trace::MAX_LAMPS];
            lamps[0] = [0.4, 0.8, 0.45];
            let setup = ph2d_field_gpu::trace::MarchSetup {
                half_extent: cam.half_extent,
                half_px: screen.half(),
                target: cam.target,
                right,
                up,
                fwd,
                ortho_start: ph2d_field_render::ORTHO_START,
                eye_distance: cam.eye_distance().unwrap_or(0.0),
                hit_eps: sharp.hit,
                normal_eps: sharp.normal,
                lamps,
                n_lamps: 1,
                ball_center: bola.center,
                ball_radius: bola.radius,
                ao_rays: ph2d_field_render::OCCLUSION_PASSES,
                ao_reach: ph2d_field_render::OCCLUSION_REACH * cam.half_extent,
                ground: None,
                antialias: true,
                edge_cos: ph2d_field_render::EDGE_COS,
                mole: None,
                longe,
                step: passo,
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                budget: ((ph2d_field_render::MAX_STEPS as f32) * shrink.max(1.0)
                    / passo.clamp(f32::EPSILON, 1.0))
                .ceil() as u32,
                t_max: ph2d_field_render::T_MAX,
            };
            let dev = t
                .lock()
                .expect("o traçador")
                .frame(&fita, campo.sculpts(), setup, W, H);
            let mut difere = 0usize;
            let mut dts: Vec<f32> = Vec::new();
            for i in 0..g.hit.len() {
                if g.hit[i] != dev.hit(i) {
                    difere += 1;
                    continue;
                }
                if !g.hit[i] {
                    continue;
                }
                let (sx, sy) = ((i % W as usize) as f32 + 0.5, (i / W as usize) as f32 + 0.5);
                let (u, v) = screen.plane_at(sx, sy);
                let (o, d) = cam.ray_at_plane(u, v);
                let p = g.point[i];
                let t_cpu = (p[0] - o[0]) * d[0] + (p[1] - o[1]) * d[1] + (p[2] - o[2]) * d[2];
                dts.push((t_cpu - dev.t[i]).abs());
            }
            dts.sort_by(f32::total_cmp);
            let p99 = dts
                .get(((dts.len().max(1) - 1) as f64 * 0.99) as usize)
                .copied();
            let longe_txt = if res == u32::MAX {
                "nada".to_string()
            } else {
                res.to_string()
            };
            println!(
                "  {cena:4} · {longe_txt:>5} · {difere:11} · {:9.2e} · {:9.2e}",
                p99.unwrap_or(0.0),
                dts.last().copied().unwrap_or(0.0)
            );
        }
    }
}

/// ⏱️⭐⭐⭐⭐ **Sonda de CPU: a desigualdade do salto, medida sem dispositivo nenhum.**
///
/// Assa a grade de longe na CPU com o MESMO `Longe::grade` e o avaliador do produto, e mede em
/// pontos aleatórios da caixa (a) a violação `s·f̃ − 0,866·h − s·f`, que tem de ser `≤ 0`, e
/// (b) o maior `s·‖∇f‖` por diferenças, que a desigualdade supõe `≤ 1`. ⚠️ Se (a) viola e (b)
/// passa de `1`, o defeito é o PASSO declarado e não o assar do dispositivo.
#[test]
#[ignore = "sonda de CPU"]
fn diag_a_desigualdade_do_salto_na_cpu() {
    println!("\n  cena · s · max s·|∇f| · violações · pior violação/h");
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let Some(bola) = ph2d_field_eval::bounds::bounding_ball(&doc, &reg) else {
            continue;
        };
        let (lo, hi) = ph2d_field_eval::bounds_clip::march_clip(bola);
        let l = ph2d_field_gpu::longe::Longe {
            lo,
            hi,
            res: 64,
            perto: crate::preview::LONGE_PERTO,
        };
        let g = l.grade().expect("a grade");
        let [dx, dy, dz] = g.dims.map(|d| d as usize);
        let total = dx * dy * dz;
        let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
        for k in 0..dz {
            for j in 0..dy {
                for i in 0..dx {
                    #[allow(clippy::cast_precision_loss)]
                    {
                        xs.push((i as f32).mul_add(g.celula, g.origem[0]));
                        ys.push((j as f32).mul_add(g.celula, g.origem[1]));
                        zs.push((k as f32).mul_add(g.celula, g.origem[2]));
                    }
                }
            }
        }
        let mut h = ph2d_field_eval::hybrid::Hybrid::new(&doc, &reg);
        let mut vals = Vec::with_capacity(total);
        const LOTE: usize = 1 << 16;
        for c in (0..total).step_by(LOTE) {
            let f = (c + LOTE).min(total);
            vals.extend_from_slice(h.eval(&xs[c..f], &ys[c..f], &zs[c..f]).expect("assar"));
        }
        let grade = Grade {
            dims: [dx, dy, dz],
            origin: g.origem,
            step: g.celula,
            values: vals,
            raio: 0.0,
        };
        let s = ph2d_field_eval::safe_march_step(&doc);
        // Pontos pseudo-aleatórios na caixa da grade (xorshift, semente fixa).
        let mut st = 0x9E37_79B9_7F4A_7C15u64;
        let mut rnd = || {
            st ^= st << 13;
            st ^= st >> 7;
            st ^= st << 17;
            #[allow(clippy::cast_precision_loss)]
            {
                (st >> 11) as f32 / (1u64 << 53) as f32
            }
        };
        let topo = grade.hi();
        const N: usize = 1 << 16;
        let (mut px, mut py, mut pz) = (Vec::new(), Vec::new(), Vec::new());
        let (mut qx, mut qy, mut qz) = (Vec::new(), Vec::new(), Vec::new());
        let dh = g.celula * 0.05;
        for _ in 0..N {
            let p: [f32; 3] =
                std::array::from_fn(|a| grade.origin[a] + rnd() * (topo[a] - grade.origin[a]));
            let mut u = [rnd() - 0.5, rnd() - 0.5, rnd() - 0.5];
            let n = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt().max(1e-9);
            for c in &mut u {
                *c /= n;
            }
            px.push(p[0]);
            py.push(p[1]);
            pz.push(p[2]);
            qx.push(p[0] + dh * u[0]);
            qy.push(p[1] + dh * u[1]);
            qz.push(p[2] + dh * u[2]);
        }
        let fp = h.eval(&px, &py, &pz).expect("f(p)").to_vec();
        let fq = h.eval(&qx, &qy, &qz).expect("f(q)").to_vec();
        let (mut grad, mut viol, mut pior) = (0.0f32, 0usize, f32::NEG_INFINITY);
        for i in 0..N {
            grad = grad.max(s * (fq[i] - fp[i]).abs() / dh);
            let lb = grade.dentro([px[i], py[i], pz[i]]) * s - 0.866_025_4 * g.celula;
            let v = lb - s * fp[i];
            if v > 0.0 {
                viol += 1;
            }
            pior = pior.max(v / g.celula);
        }
        println!("  {cena:4} · {s:.3} · {grad:9.3} · {viol:10} · {pior:8.3}");
    }
}

/// ⏱️⭐⭐⭐⭐ **Sonda de CPU: a caixa da peça CONTÉM a peça?** O recorte supõe que sim. Amostra
/// uma caixa `3×` maior, fora da caixa da marcha, e conta os pontos DENTRO da peça (`f < 0`).
///
/// ⛔ **A 1.ª redacção contava também `s·f < distância à caixa` como violação, e isso não é
/// violação nenhuma:** `s·f` é um LIMITE INFERIOR da distância à superfície, logo pode ser menor do
/// que a distância à caixa sem que o salto de fora deixe de ser seguro. Ela lia `130 000` de
/// `131 072` e não dizia nada. *A desigualdade de segurança é `dist ≥ fora`, e é a contenção que a
/// prova — não o campo.*
#[test]
#[ignore = "sonda de CPU"]
fn diag_a_caixa_contem_a_peca() {
    println!("\n  cena · pontos DENTRO da peça e FORA da caixa (tem de ser 0)");
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let Some(bola) = ph2d_field_eval::bounds::bounding_ball(&doc, &reg) else {
            continue;
        };
        let (lo, hi) = ph2d_field_eval::bounds_clip::march_clip(bola);
        let lado = (0..3).map(|a| hi[a] - lo[a]).fold(0.0f32, f32::max);
        let mut st = 0x2545_F491_4F6C_DD1Du64;
        let mut rnd = || {
            st ^= st << 13;
            st ^= st >> 7;
            st ^= st << 17;
            #[allow(clippy::cast_precision_loss)]
            {
                (st >> 11) as f32 / (1u64 << 53) as f32
            }
        };
        const N: usize = 1 << 17;
        let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
        while xs.len() < N {
            let p: [f32; 3] = std::array::from_fn(|a| {
                let c = 0.5 * (lo[a] + hi[a]);
                c + (rnd() - 0.5) * 3.0 * lado
            });
            let d = (0..3)
                .map(|a| (lo[a] - p[a]).max(p[a] - hi[a]).max(0.0))
                .map(|x| x * x)
                .sum::<f32>()
                .sqrt();
            if d > 0.0 {
                xs.push(p[0]);
                ys.push(p[1]);
                zs.push(p[2]);
            }
        }
        let mut h = ph2d_field_eval::hybrid::Hybrid::new(&doc, &reg);
        let f = h.eval(&xs, &ys, &zs).expect("f").to_vec();
        let dentro = f.iter().filter(|v| **v < 0.0).count();
        println!("  {cena:4} · {dentro}");
    }
}

/// ⏱️⭐⭐⭐⭐ **A desigualdade do salto com os números do DISPOSITIVO** (GPU). A irmã de CPU lê
/// `0` violações; esta assa a grade com o `field()` da PLACA e mede a mesma desigualdade com a
/// placa dos dois lados, mais o `s·‖∇f‖` da placa e o desvio placa-CPU nos nós.
#[test]
#[ignore = "sonda de GPU"]
fn diag_a_desigualdade_do_salto_no_dispositivo() {
    println!("\n  cena · s · max s·|∇f| placa · violações · pior/h · pior |placa−CPU| nos nós");
    for cena in [5u32, 28, 1, 11, 26, 27, 29, 30] {
        let doc = crate::smoke::scene(cena);
        let reg = crate::smoke::sampled_registry();
        let Some(bola) = ph2d_field_eval::bounds::bounding_ball(&doc, &reg) else {
            continue;
        };
        let (lo, hi) = ph2d_field_eval::bounds_clip::march_clip(bola);
        let l = ph2d_field_gpu::longe::Longe {
            lo,
            hi,
            res: 64,
            perto: crate::preview::LONGE_PERTO,
        };
        let g = l.grade().expect("a grade");
        let [dx, dy, dz] = g.dims.map(|d| d as usize);
        let total = dx * dy * dz;
        let mut pts: Vec<[f32; 3]> = Vec::with_capacity(total + 2 * (1 << 16));
        for k in 0..dz {
            for j in 0..dy {
                for i in 0..dx {
                    #[allow(clippy::cast_precision_loss)]
                    pts.push([
                        (i as f32).mul_add(g.celula, g.origem[0]),
                        (j as f32).mul_add(g.celula, g.origem[1]),
                        (k as f32).mul_add(g.celula, g.origem[2]),
                    ]);
                }
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let topo: [f32; 3] =
            std::array::from_fn(|a| ((g.dims[a] - 1) as f32).mul_add(g.celula, g.origem[a]));
        let mut st = 0x9E37_79B9_7F4A_7C15u64;
        let mut rnd = || {
            st ^= st << 13;
            st ^= st >> 7;
            st ^= st << 17;
            #[allow(clippy::cast_precision_loss)]
            {
                (st >> 11) as f32 / (1u64 << 53) as f32
            }
        };
        const N: usize = 1 << 16;
        let dh = g.celula * 0.05;
        for _ in 0..N {
            let p: [f32; 3] =
                std::array::from_fn(|a| g.origem[a] + rnd() * (topo[a] - g.origem[a]));
            let mut u = [rnd() - 0.5, rnd() - 0.5, rnd() - 0.5];
            let n = (u[0] * u[0] + u[1] * u[1] + u[2] * u[2]).sqrt().max(1e-9);
            for c in &mut u {
                *c /= n;
            }
            pts.push(p);
            pts.push([p[0] + dh * u[0], p[1] + dh * u[1], p[2] + dh * u[2]]);
        }
        let Some(par) = ph2d_field_gpu::parity::compare(&doc, &pts) else {
            println!("  {cena:4} · sem placa ou sem fita");
            continue;
        };
        let pior_nos = par.gpu[..total]
            .iter()
            .zip(&par.cpu[..total])
            .map(|(a, b)| (f64::from(*a) - b).abs())
            .fold(0.0f64, f64::max);
        let grade = Grade {
            dims: [dx, dy, dz],
            origin: g.origem,
            step: g.celula,
            values: par.gpu[..total].to_vec(),
            raio: 0.0,
        };
        let s = ph2d_field_eval::safe_march_step(&doc);
        let (mut grad, mut viol, mut pior) = (0.0f32, 0usize, f32::NEG_INFINITY);
        for i in 0..N {
            let p = pts[total + 2 * i];
            let fp = par.gpu[total + 2 * i];
            let fq = par.gpu[total + 2 * i + 1];
            grad = grad.max(s * (fq - fp).abs() / dh);
            let v = grade.dentro(p) * s - 0.866_025_4 * g.celula - s * fp;
            if v > 0.0 {
                viol += 1;
            }
            pior = pior.max(v / g.celula);
        }
        println!("  {cena:4} · {s:.3} · {grad:9.3} · {viol:10} · {pior:8.3} · {pior_nos:.3e}");
    }
}

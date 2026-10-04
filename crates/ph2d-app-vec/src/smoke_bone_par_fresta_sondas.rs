//! ⏱️ **As SONDAS da costura da imagem** — `#[ignore]`, instrumentos e não gates; filhas dos gates
//! ([`super`]) para herdar o palco e as réguas. O corte é o tecto de LOC, por responsabilidade: lá a
//! lei, aqui as medições que a escolheram (os números delas vivem nos doc-comments das consts e na
//! fila §F49).

use super::*;

/// ⏱️ **SONDA — o que o fecho ACRESCENTA numa pose**: cada triângulo a mais, com a área em px² de
/// ecrã a `100 %` e o sítio.
///
/// `SONDA_G1=<°> SONDA_G2=<°> cargo test -p ph2d-app-vec --lib --profile smoke -- --ignored --nocapture diag_o_que_o_fecho_acrescenta`
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_o_que_o_fecho_acrescenta() {
    let g = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let p = palco((
        g("SONDA_G1", POSE_DO_REPORT.0),
        g("SONDA_G2", POSE_DO_REPORT.1),
    ));
    let (sem, com) = (desenhada(&p, false, false), desenhada(&p, true, false));
    let px2 = f64::from(PPM).powi(2);
    let mut total = 0.0;
    for t in &com.tris[sem.tris.len()..] {
        let q = t.map(|i| {
            let v = com.local[i as usize];
            [f64::from(v[0]), f64::from(v[1])]
        });
        let a = ((q[1][0] - q[0][0]) * (q[2][1] - q[0][1])
            - (q[2][0] - q[0][0]) * (q[1][1] - q[0][1]))
            .abs()
            / 2.0
            * px2;
        total += a;
        let uv = t.map(|i| com.uv[i as usize]);
        println!("  tri {:?} area {a:.3} px2 uv {:?}", q[0], uv[0]);
    }
    println!(
        "acrescentou {} triangulos, {total:.3} px2",
        com.tris.len() - sem.tris.len()
    );
}

/// ⏱️ **SONDA — de onde vem o risquinho**: nós PENDURADOS (fenda entre vizinhos) e, onde a malha não
/// cobre, se os triângulos de cima e de baixo são vizinhos no repouso (fenda) ou de membros
/// diferentes (vão). Medido a `(36°, −144°)`: `0` pendurados em `9 091` triângulos, e os dois lados
/// de cada buraco a `~350 px` no repouso ⇒ um VÃO.
///
/// `SONDA_G1=<°> SONDA_G2=<°> cargo test -p ph2d-app-vec --lib --profile smoke -- --ignored --nocapture diag_de_onde_vem_o_risquinho`
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_de_onde_vem_o_risquinho() {
    let g = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let p = palco((
        g("SONDA_G1", POSE_DO_REPORT.0),
        g("SONDA_G2", POSE_DO_REPORT.1),
    ));
    let m = desenhada(&p, false, false);
    let rest = &p.mesh.rest;
    let mut pendurados = 0;
    for t in &p.mesh.tris {
        for k in 0..3 {
            let (a, b) = (rest[t[k] as usize], rest[t[(k + 1) % 3] as usize]);
            let l2 = (b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2);
            pendurados += rest
                .iter()
                .enumerate()
                .filter(|&(v, q)| {
                    let u = ((q[0] - a[0]) * (b[0] - a[0]) + (q[1] - a[1]) * (b[1] - a[1])) / l2;
                    let x = (q[0] - a[0]) * (b[1] - a[1]) - (q[1] - a[1]) * (b[0] - a[0]);
                    v as u32 != t[k]
                        && v as u32 != t[(k + 1) % 3]
                        && (1e-6..1.0 - 1e-6).contains(&u)
                        && x.abs() / l2.sqrt() < 1e-6
                })
                .count();
        }
    }
    println!(
        "malha: {} vertices, {} triangulos; pendurados: {pendurados}; buracos na janela do vao: {}",
        rest.len(),
        p.mesh.tris.len(),
        buracos(&m, JANELA_DO_VAO, 4.0)
    );
    let pos = |i: u32| {
        let q = m.local[i as usize];
        [f64::from(q[0]), f64::from(q[1])]
    };
    let quem = |q: [f64; 2]| {
        m.tris
            .iter()
            .position(|t| dentro(q, pos(t[0]), pos(t[1]), pos(t[2])))
    };
    let centro = |i: usize| {
        let t = p.mesh.tris[i];
        let r = t.map(|v| rest[v as usize]);
        [
            (r[0][0] + r[1][0] + r[2][0]) / 3.0,
            (r[0][1] + r[1][1] + r[2][1]) / 3.0,
        ]
    };
    let [x0, y0, x1, y1] = JANELA_DO_VAO;
    let h = 0.5 / f64::from(PPM);
    let mut y = y0;
    while y <= y1 {
        let mut x = x0;
        while x <= x1 {
            if quem([x, y]).is_none() {
                let viz =
                    |s: f64| (1..=16).find_map(|k| quem([x, y + s * 0.04 * f64::from(k) / 16.0]));
                if let (Some(a), Some(b)) = (viz(1.0), viz(-1.0)) {
                    let (ca, cb) = (centro(a), centro(b));
                    println!(
                        "  buraco ({x:.4}, {y:.4}) m: em cima o tri {a} (repouso {:.1},{:.1} px), em baixo o {b} ({:.1},{:.1} px) — {:.1} px no repouso",
                        ca[0],
                        ca[1],
                        cb[0],
                        cb[1],
                        (ca[0] - cb[0]).hypot(ca[1] - cb[1])
                    );
                }
            }
            x += h;
        }
        y += h;
    }
}

/// ⏱️ **SONDA — quanto custa a malha desenhada por quadro**, por porta (fecho on/off × placa on/off)
/// e por pose. `cargo test -p ph2d-app-vec --lib --profile smoke -- --ignored --nocapture diag_o_custo_da_malha_desenhada`
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_o_custo_da_malha_desenhada() {
    for pose in [
        (0.0, 0.0),
        (40.0, 40.0),
        (36.0, -131.25),
        (36.0, -144.0),
        (DOBRA_FORTE, DOBRA_FORTE),
    ] {
        let p = palco(pose);
        // ⚠️ Os anéis fora do relógio: o produto guarda-os por malha (`bordas_da`).
        let todos = ph2d_skeleton_live::skin_image_fecho::aneis_da_borda(&p.mesh.tris);
        for (costura, placa) in [(false, true), (true, true), (false, false), (true, false)] {
            let aneis: &[Vec<u32>] = if costura { &todos } else { &[] };
            let mut melhor = std::time::Duration::MAX;
            let mut cosidos = 0;
            for _ in 0..7 {
                let t = std::time::Instant::now();
                let m = ph2d_skeleton_live::skin_image_fecho::malha_desenhada_com(
                    p.mesh.clone(),
                    p.p2l,
                    &p.pele,
                    &p.pesos,
                    p.quad,
                    &p.correcoes,
                    aneis,
                    placa,
                );
                melhor = melhor.min(t.elapsed());
                cosidos = m.as_ref().map_or(0, |m| m.tris.len() - p.mesh.tris.len());
                std::hint::black_box(m);
            }
            eprintln!(
                "{pose:?} costura {costura} placa {placa}: {melhor:?} (+{cosidos} triangulos)"
            );
        }
    }
}

/// ⏱️ **SONDA — a largura dos vãos entre partes da borda que se ENCARAM**, em texels, e o custo das
/// peças (anéis, borda posada). Por nó da borda: a distância ao segmento mais perto de OUTRA parte
/// (a mais de `SONDA_L` texels ao longo do anel), que fica do lado de FORA dos dois.
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_a_largura_dos_vaos() {
    let g = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let (g1, de, ate, passo) = (
        g("SONDA_G1", 36.0),
        g("SONDA_DE", -120.0),
        g("SONDA_ATE", -160.0),
        g("SONDA_PASSO", 4.0),
    );
    let l_tx = f64::from(g("SONDA_L", 6.0));
    let n = ((ate - de) / passo).abs().round() as i32;
    for k in 0..=n {
        let g2 = de + (ate - de).signum() * passo * k as f32;
        let p = palco((g1, g2));
        let t = std::time::Instant::now();
        let aneis = ph2d_skeleton_live::skin_image_fecho::aneis_da_borda(&p.mesh.tris);
        let t_aneis = t.elapsed();
        let m = desenhada(&p, false, false);
        let pos = |v: u32| {
            [
                f64::from(m.local[v as usize][0]),
                f64::from(m.local[v as usize][1]),
            ]
        };
        let [a, b, c, d, _, _] = p.p2l.0;
        let texel = (a * d - b * c).abs().sqrt();
        let area = |r: &[u32]| {
            (0..r.len())
                .map(|i| {
                    let (p0, p1) = (pos(r[i]), pos(r[(i + 1) % r.len()]));
                    p0[0] * p1[1] - p1[0] * p0[1]
                })
                .sum::<f64>()
                / 2.0
        };
        let sinal = aneis
            .iter()
            .map(|r| area(r))
            .max_by(|x, y| x.abs().total_cmp(&y.abs()))
            .unwrap_or(1.0)
            .signum();
        // Segmentos: (anel, índice, p0, p1, s0 = arco acumulado no anel).
        let mut segs = Vec::new();
        for (ri, r) in aneis.iter().enumerate() {
            let mut s = 0.0;
            for i in 0..r.len() {
                let (p0, p1) = (pos(r[i]), pos(r[(i + 1) % r.len()]));
                segs.push((ri, i, p0, p1, s));
                s += (p1[0] - p0[0]).hypot(p1[1] - p0[1]);
            }
        }
        let perim: Vec<f64> = aneis
            .iter()
            .enumerate()
            .map(|(ri, _)| {
                segs.iter()
                    .filter(|s| s.0 == ri)
                    .map(|s| (s.3[0] - s.2[0]).hypot(s.3[1] - s.2[1]))
                    .sum()
            })
            .collect();
        let normal = |p0: [f64; 2], p1: [f64; 2]| {
            let (tx, ty) = (p1[0] - p0[0], p1[1] - p0[1]);
            let l = tx.hypot(ty).max(1e-30);
            [ty / l * sinal, -tx / l * sinal]
        };
        let mut bins = [0usize; 6];
        let mut perfil = Vec::new();
        for (si, &(ri, _, p0, p1, s0)) in segs.iter().enumerate() {
            let v = p0;
            let nv = {
                let prev = segs[if si == 0 { segs.len() - 1 } else { si - 1 }];
                let a = normal(prev.2, prev.3);
                let b = normal(p0, p1);
                [a[0] + b[0], a[1] + b[1]]
            };
            let mut melhor = f64::INFINITY;
            for &(rj, _, q0, q1, s1) in &segs {
                if rj == ri {
                    let ds = (s1 - s0).abs();
                    if ds.min(perim[ri] - ds) < l_tx * texel {
                        continue;
                    }
                }
                let dd = [q1[0] - q0[0], q1[1] - q0[1]];
                let l2 = dd[0] * dd[0] + dd[1] * dd[1];
                let t = if l2 > 0.0 {
                    (((v[0] - q0[0]) * dd[0] + (v[1] - q0[1]) * dd[1]) / l2).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let q = [q0[0] + t * dd[0], q0[1] + t * dd[1]];
                let w = [q[0] - v[0], q[1] - v[1]];
                let nq = normal(q0, q1);
                if w[0] * nv[0] + w[1] * nv[1] <= 0.0 || -(w[0] * nq[0] + w[1] * nq[1]) <= 0.0 {
                    continue;
                }
                melhor = melhor.min(w[0].hypot(w[1]) / texel);
            }
            let bin = [0.5, 1.0, 1.5, 2.0, 3.0, 6.0]
                .iter()
                .position(|&x| melhor < x);
            if let Some(b) = bin {
                bins[b] += 1;
            }
            if melhor < 6.0 {
                perfil.push(format!("{melhor:.2}"));
            }
        }
        let t = std::time::Instant::now();
        let ids: Vec<u32> = aneis.iter().flatten().copied().collect();
        let ossos = p.pesos.len() / p.mesh.rest.len();
        let linhas: Vec<f64> = ids
            .iter()
            .flat_map(|&v| {
                p.pesos[v as usize * ossos..(v as usize + 1) * ossos]
                    .iter()
                    .copied()
            })
            .collect();
        let so = ph2d_poly2d::Mesh2d {
            rest: ids.iter().map(|&v| p.mesh.rest[v as usize]).collect(),
            tris: Vec::new(),
            size: p.mesh.size,
        };
        let _ = ph2d_skeleton_live::skin_image::posed_sprite_mesh_corrigida(
            so,
            p.p2l,
            &p.pele,
            &linhas,
            p.quad[0],
            p.quad[1],
            &p.correcoes,
        );
        let t_posa = t.elapsed();
        eprintln!(
            "({g1}, {g2:.1}) aneis {t_aneis:?} posa-borda {t_posa:?} ({} nos)  <0.5:{} <1:{} <1.5:{} <2:{} <3:{} <6:{}  {:?}",
            ids.len(),
            bins[0],
            bins[1],
            bins[2],
            bins[3],
            bins[4],
            bins[5],
            if perfil.len() < 40 {
                perfil
            } else {
                vec![format!("{} nos", perfil.len())]
            }
        );
    }
}

/// ⏱️ **SONDA — a área cosida ao longo de uma varredura**, em texel².
/// `SONDA_DE=-146 SONDA_ATE=-140 SONDA_PASSO=0.1 cargo test -p ph2d-app-vec --lib --profile smoke -- --ignored --nocapture diag_a_area_cosida`
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_a_area_cosida() {
    let g = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let (g1, de, ate, passo) = (
        g("SONDA_G1", 36.0),
        g("SONDA_DE", -160.0),
        g("SONDA_ATE", -138.0),
        g("SONDA_PASSO", 1.0),
    );
    let n = ((ate - de) / passo).abs().round() as i32;
    for k in 0..=n {
        let g2 = de + (ate - de).signum() * passo * k as f32;
        eprintln!("({g1}, {g2:.2}) {:.3}", area_cosida(&palco((g1, g2))));
    }
}

/// ⏱️ **SONDA — o que se VÊ na janela do vão**, por pose: fundo entalado a `1 px` (o fio) e a `4 px`
/// (a baía), com e sem costura.
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_o_que_se_ve_no_vao() {
    let g = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let (g1, de, ate, passo) = (
        g("SONDA_G1", 36.0),
        g("SONDA_DE", -147.0),
        g("SONDA_ATE", -142.0),
        g("SONDA_PASSO", 0.05),
    );
    let n = ((ate - de) / passo).abs().round() as i32;
    for k in 0..=n {
        let g2 = de + (ate - de).signum() * passo * k as f32;
        let p = palco((g1, g2));
        let (sem, com) = (desenhada(&p, false, false), desenhada(&p, true, false));
        let j = [
            g("SONDA_X0", -1.8).into(),
            g("SONDA_Y0", 0.3).into(),
            g("SONDA_X1", -0.5).into(),
            g("SONDA_Y1", 0.75).into(),
        ];
        eprintln!(
            "({g1}, {g2:.2}) fio {}->{} baia {}->{}",
            buracos_de(&sem, j, 1.0, true),
            buracos_de(&com, j, 1.0, true),
            buracos_de(&sem, j, 4.0, true),
            buracos_de(&com, j, 4.0, true)
        );
    }
}

/// ⏱️ **SONDA — continuidade e sobreposição**: a área cosida a passos de `SONDA_PASSO` e os centros
/// cosidos que caem em tinta.
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_continuidade_e_sobreposicao() {
    let g = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let (de, passo, n) = (
        g("SONDA_DE", -146.0),
        g("SONDA_PASSO", 0.0025),
        g("SONDA_N", 20.0) as i32,
    );
    let mut antes: Option<f64> = None;
    let mut pior = 0.0_f64;
    for k in 0..=n {
        let a = area_cosida(&palco((36.0, de + passo * k as f32)));
        if let Some(b) = antes {
            pior = pior.max((a - b).abs());
        }
        antes = Some(a);
    }
    eprintln!("continuidade: pior salto {pior:.4} texel2 em passos de {passo}°");
    for g2 in [-150.0, -152.0, -155.0, -158.0, -160.0, -144.0, -143.0] {
        let (s, t) = cosidos_sobre_tinta(&palco((36.0, g2)));
        eprintln!("sobreposicao (36, {g2}): {s} de {t} centros cosidos sobre tinta");
    }
}

/// ⏱️ **SONDA — o maior vão cosido** numa varredura.
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_o_maior_vao_cosido() {
    let mut pior = (0.0_f64, 0.0_f32);
    for k in 0..=120 {
        let g2 = -150.0 + 0.1 * k as f32;
        let v = maior_vao_cosido(&palco((36.0, g2)));
        if v > pior.0 {
            pior = (v, g2);
        }
    }
    eprintln!("maior vao cosido: {:.4} texel em (36, {})", pior.0, pior.1);
}

/// ⏱️ **SONDA — a FOTO na CPU da malha desenhada** (A5-a: a cúspide junto à tampa), sem a placa:
/// rasteriza os triângulos POR ORDEM (o último por cima), textura ao texel mais perto, sobre branco,
/// `SONDA_ESCALA` amostras por px de ecrã na janela `SONDA_X0..Y1`; com e sem costura, em
/// `target/prova/imagem_<g2>_{sem,com}.ppm`.
#[test]
#[ignore = "SONDA, nao gate"]
fn diag_a_foto_da_imagem() {
    let g = |k: &str, d: f32| {
        std::env::var(k)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(d)
    };
    let (g1, g2, escala) = (
        g("SONDA_G1", 36.0),
        g("SONDA_G2", -144.0),
        g("SONDA_ESCALA", 8.0),
    );
    let j = [
        g("SONDA_X0", -1.8),
        g("SONDA_Y0", 0.3),
        g("SONDA_X1", -0.5),
        g("SONDA_Y1", 0.75),
    ]
    .map(f64::from);
    let p = palco((g1, g2));
    let px = pixels();
    std::fs::create_dir_all("target/prova").expect("pasta");
    for (nome, costura) in [("sem", false), ("com", true)] {
        let m = desenhada(&p, costura, false);
        let passo = 1.0 / (f64::from(PPM) * f64::from(escala));
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "pixels"
        )]
        let (w, h) = (
            ((j[2] - j[0]) / passo) as usize,
            ((j[3] - j[1]) / passo) as usize,
        );
        let mut img = vec![[255.0_f64; 3]; w * h];
        for t in &m.tris {
            let q = t.map(|i| m.local[i as usize].map(f64::from));
            let uv = t.map(|i| m.uv[i as usize].map(f64::from));
            let area = (q[1][0] - q[0][0]) * (q[2][1] - q[0][1])
                - (q[2][0] - q[0][0]) * (q[1][1] - q[0][1]);
            if area == 0.0 {
                continue;
            }
            let lo = [
                q[0][0].min(q[1][0]).min(q[2][0]),
                q[0][1].min(q[1][1]).min(q[2][1]),
            ];
            let hi = [
                q[0][0].max(q[1][0]).max(q[2][0]),
                q[0][1].max(q[1][1]).max(q[2][1]),
            ];
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "pixels"
            )]
            let ix = |x: f64| (((x - j[0]) / passo).max(0.0) as usize).min(w);
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "pixels"
            )]
            let iy = |y: f64| (((j[3] - y) / passo).max(0.0) as usize).min(h);
            for yy in iy(hi[1])..(iy(lo[1]) + 1).min(h) {
                for xx in ix(lo[0])..(ix(hi[0]) + 1).min(w) {
                    #[expect(clippy::cast_precision_loss, reason = "pixels")]
                    let s = [
                        j[0] + (xx as f64 + 0.5) * passo,
                        j[3] - (yy as f64 + 0.5) * passo,
                    ];
                    let b1 = ((s[0] - q[0][0]) * (q[2][1] - q[0][1])
                        - (q[2][0] - q[0][0]) * (s[1] - q[0][1]))
                        / area;
                    let b2 = ((q[1][0] - q[0][0]) * (s[1] - q[0][1])
                        - (s[0] - q[0][0]) * (q[1][1] - q[0][1]))
                        / area;
                    let b0 = 1.0 - b1 - b2;
                    if b0 < 0.0 || b1 < 0.0 || b2 < 0.0 {
                        continue;
                    }
                    let u = b0 * uv[0][0] + b1 * uv[1][0] + b2 * uv[2][0];
                    let v = b0 * uv[0][1] + b1 * uv[1][1] + b2 * uv[2][1];
                    #[expect(
                        clippy::cast_possible_truncation,
                        clippy::cast_sign_loss,
                        reason = "texel"
                    )]
                    let (tx, ty) = (
                        ((u * f64::from(IMG_W)) as u32).min(IMG_W - 1),
                        ((v * f64::from(IMG_H)) as u32).min(IMG_H - 1),
                    );
                    let k = ((ty * IMG_W + tx) * 4) as usize;
                    let a = f64::from(px[k + 3]) / 255.0;
                    let o = &mut img[yy * w + xx];
                    for c in 0..3 {
                        o[c] = f64::from(px[k + c]).mul_add(a, o[c] * (1.0 - a));
                    }
                }
            }
        }
        let mut f = format!("P6\n{w} {h}\n255\n").into_bytes();
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "um canal"
        )]
        f.extend(img.iter().flat_map(|c| c.map(|x| x.round() as u8)));
        std::fs::write(format!("target/prova/imagem_{g2}_{nome}.ppm"), f).expect("ppm");
    }
}

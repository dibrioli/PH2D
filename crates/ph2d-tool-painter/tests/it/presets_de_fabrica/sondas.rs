//! As SONDAS da fábrica (medição à mão, `--ignored`) — partidas de `presets_de_fabrica.rs` pelo tecto de 700 linhas.

use super::*;

/// **A largura sem o Real Oil, medida** (J6): o preset de fábrica `PH2D_SONDA_PRESET` («Knife Oily») a cada 8.º raio
/// da lei, com e sem o óleo, cada raio num fio, as duas ordens intercaladas — o relógio e a largura de cada.
#[test]
#[ignore = "sonda: mede a lei da largura com e sem o Real Oil; corre-se à mão"]
fn sonda_largura_sem_oleo() {
    let nome = std::env::var("PH2D_SONDA_PRESET").unwrap_or_else(|_| "Knife Oily".into());
    let p = crate::midias_de_fabrica::de_fabrica(&nome);
    let raios: Vec<f32> = raios_da_lei().into_iter().step_by(8).collect();
    let medidas: Vec<(f32, [(f32, f64); 2])> = std::thread::scope(|s| {
        let h: Vec<_> = raios
            .iter()
            .enumerate()
            .map(|(k, &raio)| {
                let p = &p;
                s.spawn(move || {
                    let mut m = [(0.0, 0.0); 2];
                    for j in 0..2 {
                        let com = (j + k) % 2 == 0;
                        LEI_COM_OLEO.set(com);
                        let t0 = std::time::Instant::now();
                        let l = largura_de(p, raio, true);
                        m[usize::from(com)] = (l, t0.elapsed().as_secs_f64());
                    }
                    (raio, m)
                })
            })
            .collect();
        h.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let (mut t_sem, mut t_com) = (0.0, 0.0);
    for (raio, [(ls, ts), (lc, tc)]) in &medidas {
        println!("[sonda] {nome} raio {raio}: sem óleo {ls} px {ts:.2} s · com {lc} px {tc:.2} s");
        t_sem += ts;
        t_com += tc;
    }
    println!(
        "[sonda] {nome}: soma com óleo {t_com:.1} s → sem {t_sem:.1} s ({:.2}×)",
        t_com / t_sem
    );
    let mudam: Vec<_> = medidas
        .iter()
        .filter(|(_, [(a, _), (b, _)])| a != b)
        .collect();
    println!(
        "[sonda] {} de {} larguras mudam sem o óleo",
        mudam.len(),
        medidas.len()
    );
    // ONDE mudam: o píxel que é papel sem o óleo e não é com ele (a luz/sombra fora da tinta), e o contrário.
    for raio in [18.5, 98.9] {
        let img = |com: bool| {
            LEI_COM_OLEO.set(com);
            imagem_da_largura(&p, raio, true, Relogio::PorQuadro, 0).0
        };
        let (sem, com) = (img(false), img(true));
        let papel = |px: &[u8; 4]| px[..3] == [255; 3];
        let (mut so_com, mut so_sem, mut tinta) = (0, 0, 0);
        for (a, b) in sem.as_chunks::<4>().0.iter().zip(com.as_chunks::<4>().0) {
            match (papel(a), papel(b)) {
                (true, false) => so_com += 1,
                (false, true) => so_sem += 1,
                (false, false) => tinta += 1,
                _ => {}
            }
        }
        println!(
            "[sonda] raio {raio}: papel sem óleo que o óleo escurece {so_com} px · tinta sem óleo que com ele é papel \
             {so_sem} px · tinta nos dois {tinta} px"
        );
    }
}

/// **Sonda do relógio** ([`Relogio::DeUmaVez`], 24/10): cada cena dos dois portões da fábrica — a da largura (cada preset
/// do Wet a cada raio da promessa, e o controlo cru do Wet Round) e a cena seca das mídias (cada preset do Wet à largura
/// da sua foto, ou 20, e o Wet Round a cada largura de foto) — pintada com um tique por quadro e com o mesmo tempo num
/// tique só: as telas têm de ser iguais ao byte. É ela que deixa os portões passarem o tempo de uma vez.
#[test]
#[ignore = "sonda: prova o relógio DeUmaVez dos portões da fábrica (custa ~2× os dois portões)"]
fn o_tempo_de_uma_vez_e_o_por_quadro() {
    use crate::midias_de_fabrica::{de_fabrica, imagem_seca, raio_da_midia};
    use std::sync::atomic::{AtomicUsize, Ordering};
    enum Cena {
        Largura(usize, f32, bool),
        Seca(usize, f32),
        SecaDoRound(f32),
    }
    let presets: Vec<(String, Preset)> = ficheiros_do_wet()
        .into_iter()
        .map(|(rel, b)| (rel, ler(&b).unwrap()))
        .collect();
    let round = de_fabrica("Wet Round");
    let i_round = presets
        .iter()
        .position(|(rel, _)| rel.ends_with("Wet Round.ph2d-brush.png"))
        .expect("o Wet Round de fábrica");
    let mut cenas = Vec::new();
    let mut raios_das_fotos = Vec::new();
    for (i, (_, p)) in presets.iter().enumerate() {
        cenas.extend(
            RAIOS_DA_PROMESSA
                .iter()
                .map(|&r| Cena::Largura(i, r, false)),
        );
        let foto = raio_da_midia(&p.parametros.name);
        raios_das_fotos.extend(foto);
        cenas.push(Cena::Seca(i, foto.unwrap_or(20.0)));
    }
    cenas.extend(
        RAIOS_DA_PROMESSA
            .iter()
            .map(|&r| Cena::Largura(i_round, r, true)),
    );
    raios_das_fotos.sort_by(f32::total_cmp);
    raios_das_fotos.dedup();
    cenas.extend(raios_das_fotos.into_iter().map(Cena::SecaDoRound));
    let proxima = AtomicUsize::new(0);
    let resultados: Vec<(String, usize)> = std::thread::scope(|s| {
        let h: Vec<_> = (0..8)
            .map(|_| {
                s.spawn(|| {
                    let mut feitas = Vec::new();
                    while let Some(c) = cenas.get(proxima.fetch_add(1, Ordering::Relaxed)) {
                        let (nome, a, b) = match *c {
                            Cena::Largura(i, r, crua) => {
                                let (rel, p) = &presets[i];
                                let f = |rl| imagem_da_largura(p, r, crua, rl, 0).0;
                                (
                                    format!("largura {rel} raio {r} crua {crua}"),
                                    f(Relogio::PorQuadro),
                                    f(Relogio::DeUmaVez),
                                )
                            }
                            Cena::Seca(i, r) => {
                                let (rel, p) = &presets[i];
                                let f = |rl| imagem_seca(p, r, 0, rl).0;
                                (
                                    format!("seca {rel} raio {r}"),
                                    f(Relogio::PorQuadro),
                                    f(Relogio::DeUmaVez),
                                )
                            }
                            Cena::SecaDoRound(r) => {
                                let f = |rl| imagem_seca(&round, r, 0, rl).0;
                                (
                                    format!("seca do controlo Wet Round raio {r}"),
                                    f(Relogio::PorQuadro),
                                    f(Relogio::DeUmaVez),
                                )
                            }
                        };
                        let n = a.iter().zip(&b).filter(|(x, y)| x != y).count()
                            + a.len().abs_diff(b.len());
                        println!("[relogio] {nome}: {n} bytes diferem");
                        feitas.push((nome, n));
                    }
                    feitas
                })
            })
            .collect();
        h.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });
    assert_eq!(resultados.len(), cenas.len());
    let diferem: Vec<_> = resultados.iter().filter(|(_, n)| *n > 0).collect();
    assert!(
        diferem.is_empty(),
        "o tempo de uma vez pinta outra tela: {diferem:#?}"
    );
}

/// **Sonda do custo da largura** — o CPU do processo (todas as threads, `/proc/self/stat`) que cada preset do Wet custa
/// ao [`a_largura_e_a_prometida`], um preset de cada vez (os raios em paralelo), e por raio no `PH2D_SONDA_PRESET`.
#[test]
#[ignore = "sonda: o custo da largura por preset"]
fn sonda_custo_da_largura() {
    fn cpu_s() -> f64 {
        let st = std::fs::read_to_string("/proc/self/stat").unwrap();
        let f: Vec<&str> = st.rsplit_once(')').unwrap().1.split_whitespace().collect();
        (f[11].parse::<f64>().unwrap() + f[12].parse::<f64>().unwrap()) / 100.0
    }
    let alvo = std::env::var("PH2D_SONDA_PRESET").ok();
    for (rel, b) in ficheiros_do_wet() {
        let p = ler(&b).unwrap();
        if alvo.as_deref().is_some_and(|a| a != p.parametros.name) {
            continue;
        }
        if alvo.is_some() {
            for &r in &RAIOS_DA_PROMESSA {
                let c0 = cpu_s();
                let _ = largura_de_com(&p, r, false, Relogio::DeUmaVez);
                println!("[custo-largura] {rel} raio {r}: {:.1} s CPU", cpu_s() - c0);
            }
        } else {
            let c0 = cpu_s();
            let _ = larguras_da_promessa(&p, false);
            println!("[custo-largura] {rel}: {:.1} s CPU", cpu_s() - c0);
        }
    }
}

/// **Sonda de custo da miniatura** — o número que decide onde o preview vivo corre (no quadro ou numa
/// thread). Mínimo de 5 corridas por preset, a tela inteira e só o traço (sem a água depois).
#[test]
#[ignore = "sonda: mede o custo de pintar a miniatura"]
fn sonda_custo_da_miniatura() {
    println!(
        "[custo] load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    for (rel, b) in ficheiros_de_fabrica() {
        let p = ler(&b).unwrap();
        let min = (0..5)
            .map(|_| {
                let t0 = std::time::Instant::now();
                let _ = miniatura_do_preset(&p);
                t0.elapsed().as_secs_f64() * 1e3
            })
            .fold(f64::INFINITY, f64::min);
        println!("[custo] {rel:<60} {min:7.1} ms");
    }
}

/// **Sonda do SORTEIO da largura**: o preset `PH2D_SONDA_PRESET` a cada raio da promessa, com a tabela dele, uma
/// largura por semente dos traços (0‥8) — quanto o sorteio (o Jitter) move a largura no MESMO raio.
#[test]
#[ignore = "sonda: a largura por semente"]
fn sonda_sorteio_da_largura() {
    let nome = std::env::var("PH2D_SONDA_PRESET").unwrap_or_else(|_| "Blender Textured".into());
    let p = crate::midias_de_fabrica::de_fabrica(&nome);
    for &r in &RAIOS_DA_PROMESSA {
        let ls: Vec<f32> = std::thread::scope(|s| {
            let h: Vec<_> = (0..9u64)
                .map(|k| {
                    let p = &p;
                    s.spawn(move || {
                        let (px, w, h, regua) =
                            imagem_da_largura(p, r, false, Relogio::DeUmaVez, k);
                        regua(&px, w, h)
                    })
                })
                .collect();
            h.into_iter().map(|h| h.join().unwrap()).collect()
        });
        println!(
            "[sorteio] {nome} raio {r} pedido {} → {ls:?} mediana {}",
            2.0 * r,
            mediana(ls.clone())
        );
    }
}

/// **Sonda do RUÍDO da largura** (W2): o mesmo pincel ao mesmo raio prometido, cinco traços que só
/// diferem na posição vertical (a tela cresce 60 px de cada vez; o traço desce 30) — o quanto a largura de um traço varia
/// por si. É o chão de qualquer promessa: nenhuma tabela a acerta melhor do que isto.
#[test]
#[ignore = "sonda: o ruído da largura por traço (W2)"]
fn sonda_ruido_da_largura() {
    for (rel, b) in ficheiros_do_wet() {
        let p = direito(&ler(&b).unwrap());
        for &r in &RAIOS_DA_PROMESSA {
            let ls: Vec<f32> = std::thread::scope(|s| {
                let h: Vec<_> = (0..5)
                    .map(|k| {
                        let p = &p;
                        s.spawn(move || {
                            let (px, w, h) =
                                traco_recto_alto(r, r + 10.0 * k as f32, QUADROS_DA_LEI, |t| {
                                    t.set_wet_relogio_fixo(true);
                                    assert!(t.wet_preset_aplicar(p).is_empty());
                                });
                            largura_que_se_ve(&px, w, h)
                        })
                    })
                    .collect();
                h.into_iter().map(|h| h.join().unwrap()).collect()
            });
            println!(
                "[ruido] {rel:<55} raio {r:>5} pedido {:>5} → {ls:?}",
                2.0 * r
            );
        }
    }
}

/// **Sonda: de que a cena da largura é feita** — o gesto e os `QUADROS_DA_LEI` da água, por raio, num pincel do Wet
/// (`PH2D_SONDA_PRESET`, omissão Wet Round): a água passo a passo pelos estágios do motor que o PRESET montou, o tempo
/// de cada estágio somado na cena inteira e a janela média.
#[test]
#[ignore = "sonda de medicao (perfil smoke, --test-threads=1)"]
fn sonda_de_que_a_cena_da_largura_e_feita() {
    use ph2d_editor_core::tool::RasterEditTool;
    use std::time::Instant;
    let nome = std::env::var("PH2D_SONDA_PRESET").unwrap_or_else(|_| "Wet Round".into());
    let (_, b) = ficheiros_do_wet()
        .into_iter()
        .find(|(rel, _)| rel.contains(&nome))
        .expect("o preset");
    let p = direito(&ler(&b).unwrap());
    let passos = (QUADROS_DA_LEI as f64 / 60.0 * 40.0) as usize;
    println!(
        "\n  {nome}, {passos} passos: raio  gesto ms  agua ms  ns/cel·passo  janela média  | ms por estágio 0..6"
    );
    for &r in &RAIOS_DA_PROMESSA {
        let h = ((r * 6.0) as u32 + 80).max(120);
        let cy = h as f32 * 0.5;
        let mut t = PainterTool::default();
        t.set_source(
            vec![255u8; (crate::sonda_gestos::W * h * 4) as usize],
            crate::sonda_gestos::W,
            h,
        );
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(&p).is_empty());
        t.set_brush_size_px(r);
        t.set_brush_color_srgb8(crate::sonda_gestos::AZUL);
        let t0 = Instant::now();
        crate::sonda_gestos::gesto(&mut t, &[[100.0, cy], [900.0, cy]], 3.0, |_| 1.0);
        let gesto = t0.elapsed().as_secs_f64() * 1e3;
        let mut est = [0.0f64; 8];
        let (mut celulas, mut dados) = (0usize, 0usize);
        let t1 = Instant::now();
        t.wet_sonda_motor(|e| {
            for _ in 0..passos {
                if !e.sim_should_run() {
                    break;
                }
                let g = e.active_grid();
                if g.has_fluid {
                    celulas += ((g.bx1 - g.bx0 + 1).max(0) * (g.by1 - g.by0 + 1).max(0)) as usize;
                }
                dados += 1;
                let mut k = 0;
                loop {
                    let s0 = Instant::now();
                    let fim = e.step_stage();
                    est[k.min(7)] += s0.elapsed().as_secs_f64() * 1e3;
                    k += 1;
                    if fim.is_some() {
                        break;
                    }
                }
            }
        });
        let agua = t1.elapsed().as_secs_f64() * 1e3;
        let media = celulas / dados.max(1);
        let ns = agua * 1e6 / celulas.max(1) as f64;
        let e = est
            .iter()
            .map(|v| format!("{v:6.0}"))
            .collect::<Vec<_>>()
            .join(" ");
        println!("  {r:>30}  {gesto:>8.1}  {agua:>7.1}  {ns:>12.1}  {media:>12}  | {e}");
        let _ = t.run_full();
    }
}

/// **Sonda: o piso da secagem paralela na cena da fábrica** — o motor que o preset montou (`PH2D_SONDA_PRESET`, omissão
/// Wet Round), depois do gesto e de 100 passos, por raio: a secagem serial × paralela intercaladas, mínimo de 7, estado
/// restaurado. É o joelho do `MIN_CELLS_DRYING` com a célula da lei do Rebelle.
#[test]
#[ignore = "sonda de medicao (perfil smoke, --test-threads=1)"]
fn sonda_o_piso_da_secagem_na_cena_da_fabrica() {
    use ph2d_editor_core::tool::RasterEditTool;
    use ph2d_wet_paint::drying::drying_pass_jacobi_rows;
    use ph2d_wet_paint::grid::{restore_grid, snapshot_grid};
    use ph2d_wet_paint::par::Rows;
    use ph2d_wet_paint::solver;
    use std::time::Instant;
    let nome = std::env::var("PH2D_SONDA_PRESET").unwrap_or_else(|_| "Wet Round".into());
    let (_, b) = ficheiros_do_wet()
        .into_iter()
        .find(|(rel, _)| rel.contains(&nome))
        .expect("o preset");
    let p = direito(&ler(&b).unwrap());
    let la = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "\n  loadavg inicio: {}\n  {nome}: largura raio celulas | passe serial/paralela ms = razão",
        la.trim()
    );
    // As telas estreitas descem a janela abaixo da do raio 2 da fábrica (~20 mil células): é lá que o joelho mora.
    let estreitas = [
        (60u32, 2.0f32),
        (120, 2.0),
        (250, 2.0),
        (500, 2.0),
        (250, 7.0),
        (500, 7.0),
    ];
    let cenas = estreitas.into_iter().chain(
        RAIOS_DA_PROMESSA
            .iter()
            .map(|&r| (crate::sonda_gestos::W, r)),
    );
    for (w, r) in cenas {
        let h = ((r * 6.0) as u32 + 80).max(120);
        let cy = h as f32 * 0.5;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(&p).is_empty());
        t.set_brush_size_px(r);
        t.set_brush_color_srgb8(crate::sonda_gestos::AZUL);
        let wf = w as f32;
        crate::sonda_gestos::gesto(&mut t, &[[0.1 * wf, cy], [0.9 * wf, cy]], 3.0, |_| 1.0);
        t.wet_sonda_motor(|e| {
            for _ in 0..100 {
                e.step_simulation();
            }
            let params = e.sim.gather_params(&e.tuning);
            let (evap, rewet) = (e.sim.evap_scale, e.sim.rewet_base);
            let [gx, gy] = e.sim.gravity(&e.tuning);
            let sed = params.sedimenta * (gx * gx + gy * gy).sqrt();
            let km = params.km_mixing;
            let g = e.active_grid();
            let celulas = ((g.by1 - g.by0 + 1).max(0) * (g.bx1 - g.bx0 + 1)).max(0) as usize;
            let snap = snapshot_grid(g);
            type Passe<'a> = (
                &'a str,
                Box<dyn Fn(&mut ph2d_wet_paint::grid::Grid, Rows) + 'a>,
            );
            let passes: [Passe; 6] = [
                (
                    "drying",
                    Box::new(|g, m| drying_pass_jacobi_rows(g, &params, evap, rewet, false, m)),
                ),
                (
                    "advect",
                    Box::new(|g, m| {
                        solver::advect_jacobi_rows(g, &params, gx, gy, m);
                    }),
                ),
                (
                    "sedimenta",
                    Box::new(|g, m| solver::sedimenta_rows(g, sed, gx, gy, km, m)),
                ),
                (
                    "flow",
                    Box::new(|g, m| {
                        solver::build_flow_field_jacobi_rows(g, &params, gx, gy, false, m)
                    }),
                ),
                (
                    "project",
                    Box::new(|g, m| solver::project_rows(g, &params, m)),
                ),
                (
                    "smooth",
                    Box::new(|g, m| solver::smooth_velocity_rows(g, &params, m)),
                ),
            ];
            let mut linha = format!("  {w:>5} {r:>5} {celulas:>7} |");
            for (nome, f) in &passes {
                let mut best = [f64::MAX; 2];
                for _ in 0..7 {
                    for (k, mode) in [Rows::Serial, Rows::Parallel].into_iter().enumerate() {
                        let g = e.active_grid_mut();
                        restore_grid(g, &snap);
                        let t0 = Instant::now();
                        f(g, mode);
                        best[k] = best[k].min(t0.elapsed().as_secs_f64() * 1e3);
                    }
                }
                linha += &format!(
                    " {nome} {:.3}/{:.3}={:.2} |",
                    best[0],
                    best[1],
                    best[0] / best[1]
                );
            }
            println!("{linha}");
        });
        let _ = t.run_full();
    }
    let la = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("  loadavg fim: {}", la.trim());
}

/// **Sonda: o piso de TEMPO da água** (`ph2d_wet_paint::par::PISO_NS_AGUA`) — o ciclo de 12 passos do produto sobre o
/// motor que o preset montou, por cena, a cada piso candidato no MESMO processo (∞ = a rota de antes: série abaixo dos
/// pisos de células), intercalados, mínimo de 7, estado restaurado.
#[test]
#[ignore = "sonda de medicao (perfil smoke, --test-threads=1)"]
fn sonda_o_piso_de_tempo_da_agua() {
    use ph2d_editor_core::tool::RasterEditTool;
    use ph2d_wet_paint::grid::{restore_grid, snapshot_grid};
    use ph2d_wet_paint::par::PISO_NS_AGUA;
    use std::sync::atomic::Ordering;
    use std::time::Instant;
    const PISOS: [u64; 6] = [u64::MAX, 0, 10_000, 20_000, 40_000, 80_000];
    let nome = std::env::var("PH2D_SONDA_PRESET").unwrap_or_else(|_| "Wet Round".into());
    let (_, b) = ficheiros_do_wet()
        .into_iter()
        .find(|(rel, _)| rel.contains(&nome))
        .expect("o preset");
    let p = direito(&ler(&b).unwrap());
    let antes = PISO_NS_AGUA.load(Ordering::Relaxed);
    let la = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "\n  loadavg inicio: {}\n  {nome}: ms por ciclo de 12 passos, piso ∞ · 0 · 10 · 20 · 40 · 80 µs",
        la.trim()
    );
    let estreitas = [
        (60u32, 2.0f32),
        (120, 2.0),
        (250, 2.0),
        (500, 2.0),
        (250, 7.0),
        (500, 7.0),
    ];
    let cenas = estreitas.into_iter().chain(
        RAIOS_DA_PROMESSA
            .iter()
            .map(|&r| (crate::sonda_gestos::W, r)),
    );
    for (w, r) in cenas {
        let h = ((r * 6.0) as u32 + 80).max(120);
        let cy = h as f32 * 0.5;
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
        t.set_wet_relogio_fixo(true);
        assert!(t.wet_preset_aplicar(&p).is_empty());
        t.set_brush_size_px(r);
        t.set_brush_color_srgb8(crate::sonda_gestos::AZUL);
        let wf = w as f32;
        crate::sonda_gestos::gesto(&mut t, &[[0.1 * wf, cy], [0.9 * wf, cy]], 3.0, |_| 1.0);
        t.wet_sonda_motor(|e| {
            for _ in 0..100 {
                e.step_simulation();
            }
            let g = e.active_grid();
            let celulas = ((g.by1 - g.by0 + 1).max(0) * (g.bx1 - g.bx0 + 1)).max(0) as usize;
            let (snap, sim0) = (snapshot_grid(g), e.sim.clone());
            let mut best = [f64::MAX; PISOS.len()];
            for _ in 0..7 {
                for (k, &piso) in PISOS.iter().enumerate() {
                    PISO_NS_AGUA.store(piso, Ordering::Relaxed);
                    e.sim = sim0.clone();
                    restore_grid(e.active_grid_mut(), &snap);
                    let t0 = Instant::now();
                    for _ in 0..12 {
                        e.step_simulation();
                    }
                    best[k] = best[k].min(t0.elapsed().as_secs_f64() * 1e3);
                }
            }
            let v = best
                .iter()
                .map(|b| format!("{b:8.2}"))
                .collect::<Vec<_>>()
                .join(" ");
            println!(
                "  {w:>5} {r:>5} {celulas:>7} | {v} | ∞÷melhor {:.2}",
                best[0] / best.iter().copied().fold(f64::MAX, f64::min)
            );
        });
        let _ = t.run_full();
    }
    PISO_NS_AGUA.store(antes, Ordering::Relaxed);
    let la = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("  loadavg fim: {}", la.trim());
}

/// **Sonda: o portão da largura com e sem o piso de tempo da água** — o corpo de [`a_largura_e_a_prometida`] (todos os
/// presets do Wet, cada raio, em paralelo como o portão) com `PISO_NS_AGUA` = ∞ (a rota de antes) e o de fábrica,
/// alternados, duas rondas; as larguras das duas rotas têm de ser as mesmas.
#[test]
#[ignore = "sonda de medicao (perfil smoke): ~4 corridas do portão da largura"]
fn sonda_o_portao_da_largura_com_o_piso_de_tempo() {
    use ph2d_wet_paint::par::PISO_NS_AGUA;
    use std::sync::atomic::Ordering;
    use std::time::Instant;
    let fabrica = PISO_NS_AGUA.load(Ordering::Relaxed);
    let ficheiros = ficheiros_do_wet();
    let corre = || -> Vec<Vec<f32>> {
        std::thread::scope(|s| {
            let h: Vec<_> = ficheiros
                .iter()
                .map(|(_, b)| s.spawn(move || larguras_da_promessa(&ler(b).unwrap(), false)))
                .collect();
            h.into_iter().map(|h| h.join().unwrap()).collect()
        })
    };
    let la = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!(
        "\n  loadavg inicio: {}  ({} presets do Wet)",
        la.trim(),
        ficheiros.len()
    );
    let mut res: Vec<Vec<Vec<f32>>> = Vec::new();
    for ronda in 0..2 {
        for piso in [u64::MAX, fabrica] {
            PISO_NS_AGUA.store(piso, Ordering::Relaxed);
            let t0 = Instant::now();
            res.push(corre());
            println!(
                "  ronda {ronda} piso {piso:>20}: {:6.1} s",
                t0.elapsed().as_secs_f64()
            );
        }
    }
    PISO_NS_AGUA.store(fabrica, Ordering::Relaxed);
    assert!(
        res.iter().all(|r| *r == res[0]),
        "as larguras mudaram com a rota"
    );
    let la = std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    println!("  loadavg fim: {}", la.trim());
}

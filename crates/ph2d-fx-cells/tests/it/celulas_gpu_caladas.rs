//! ⭐ **A linha CALADA não é célula** (doc 132 §7 P4): o quad de tamanho zero — a linha de forma que a
//! placa cala no buffer das sprites (doc 121 W3), a imagem de grupo que a separação limpou e a linha de
//! forma que ela leva com a faixa (P4) — não cobre pixel nenhum. A caixa dele é VAZIA e ele não entra
//! no plano: sem a regra, cada uma era uma célula de ~2 px mais a vala, relocada, desenhada e devolvida
//! a pintar nada.

use super::*;

/// A sprite da fixtura CALADA como a placa a cala: o mesmo registo com o tamanho a zero.
fn calada(b: &Banca, x: f32, y: f32) -> RenderInstance {
    RenderInstance {
        size: [0.0, 0.0],
        ..sprite(b, x, y)
    }
}

/// As células do último plano.
fn celulas(b: &Banca) -> usize {
    b.celulas
        .ultimas_paginas()
        .iter()
        .map(|p| p.celulas.len())
        .sum()
}

/// ⭐⭐ **Gate:** discos e uma sprite visível, com e sem `2·n` linhas caladas (metade com a pose da
/// sprite, metade zeradas como a separação as deixa), pelas DUAS rotas das sprites. Com a regra, as
/// células são SÓ as que desenham e a camada é a mesma ao byte. ⚠️ Controlo: sem a regra
/// (`calada_vazia: false`) as caladas viram células — a fixtura contém o que se mede.
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn uma_linha_calada_nao_e_celula() {
    let Some(gpu) = gpu() else {
        return;
    };
    let mut b = Banca::new(gpu);
    let ds: Vec<ShapeInstance> = (0..6).map(|k| disco(-6.0 + 2.0 * k as f32, -3.0)).collect();
    let ops = pilhas_de_fora()[1].1.clone();
    let visivel = sprite(&b, 2.0, 4.0);
    let mut com_caladas = vec![visivel];
    for d in &ds {
        com_caladas.push(calada(&b, d.pos[0], d.pos[1]));
        com_caladas.push(bytemuck::Zeroable::zeroed());
    }
    let so = [visivel];
    let desenham = ds.len() + 1;
    for placa in [false, true] {
        let rota = |v| if placa { Rota::Placa(v) } else { Rota::Cpu(v) };
        b.celulas.escolhe(Variante::default());
        let referencia = b.cada_um(&ds, rota(&so), &ops);
        assert_eq!(celulas(&b), desenham, "placa={placa}: a referência");
        let lida = b.cada_um(&ds, rota(&com_caladas), &ops);
        let n = celulas(&b);
        b.celulas.escolhe(Variante {
            calada_vazia: false,
            ..Variante::default()
        });
        let sem_regra = b.cada_um(&ds, rota(&com_caladas), &ops);
        let n_sem = celulas(&b);
        b.celulas.escolhe(Variante::default());
        eprintln!(
            "[caladas] placa={placa}: células {n} (sem a regra {n_sem}) · desenham {desenham} · \
             igual ao byte {} (sem a regra {})",
            lida == referencia,
            sem_regra == referencia
        );
        assert!(
            n_sem > desenham,
            "placa={placa}: controlo — sem a regra as caladas viram células ({n_sem})"
        );
        assert_eq!(n, desenham, "placa={placa}: só as que desenham são células");
        assert!(
            lida == referencia,
            "placa={placa}: a camada muda com as caladas"
        );
    }
}

/// **A régua das caladas** (doc 132 §7 P4, `MEDIR_VELOCIDADE.md`): discos «cada forma» na bancada da
/// C.10 com `r·n` linhas caladas no buffer da placa — `r = 1` é a forma calada nas sprites (doc 121
/// W3, antes do P4), `r = 3` a composição da `=140` (as formas caladas e as imagens limpas nas sprites,
/// as formas caladas que a separação leva para as imagens). As duas variantes (`calada_vazia`) no MESMO
/// processo, blocos de 10 quadros com a ordem RODADA, 7 rodadas; o MÍNIMO da placa e da parede.
/// Imprime; não reprova.
///
/// `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --release -p ph2d-fx-cells --test it relogio_das_linhas_caladas -- --ignored --nocapture`
#[test]
#[ignore = "medição de GPU"]
fn relogio_das_linhas_caladas() {
    let Some(gpu) = gpu() else {
        return;
    };
    ph2d_gpu::pass_profiler::init_forced(&gpu.device, &gpu.queue);
    let mut b = Banca::new(gpu);
    let mesa = Mesa::new(&b.gpu);
    let carga0 = carga();
    let variantes = [false, true].map(|calada_vazia| Variante {
        calada_vazia,
        ..Variante::default()
    });
    for (pilha, ops) in pilhas_de_fora().into_iter().take(3) {
        for (cols, rows) in [(32, 16), (64, 64)] {
            for r in [1usize, 3] {
                let ds = grelha_de(cols, rows);
                let mut caladas = Vec::with_capacity(r * ds.len());
                for k in 0..r {
                    for d in &ds {
                        caladas.push(if k == 1 {
                            bytemuck::Zeroable::zeroed()
                        } else {
                            calada(&b, d.pos[0], d.pos[1])
                        });
                    }
                }
                let s_buf = b.sprites_na_placa(&caladas);
                let buf = b.copias(&ds);
                let n = ds.len() as u32;
                let corre = |b: &mut Banca| {
                    b.celulas.corre(Entrada {
                        gpu: &b.gpu,
                        renderer: &mut b.renderer,
                        camera: &mesa.cam,
                        window: WindowSize {
                            width: mesa.lw,
                            height: mesa.lh,
                        },
                        scene_viewport: None,
                        sprites: Sprites::Placa {
                            buffer: &s_buf,
                            n: caladas.len() as u32,
                            runs: &[],
                        },
                        formas: Some(Formas {
                            buffer: &buf,
                            n,
                            vista: mesa.vista,
                            corridas: &[],
                        }),
                        ops: &ops,
                        pilha: &mut b.pilha,
                        camada: &mesa.vc,
                        tamanho: (mesa.lw, mesa.lh),
                        imagens: None,
                    });
                };
                for v in variantes {
                    b.celulas.escolhe(v);
                    for _ in 0..3 {
                        corre(&mut b);
                        ph2d_gpu::pass_profiler::end_frame(&b.gpu.device, &b.gpu.queue);
                    }
                }
                let _ = ph2d_gpu::pass_profiler::drain(&b.gpu.device);
                let mut placa: [Vec<f64>; 2] = Default::default();
                let mut parede: [Vec<f64>; 2] = Default::default();
                let mut contas = [(0usize, 0usize); 2];
                for rodada in 0..7 {
                    for k in 0..2 {
                        let i = (k + rodada) % 2;
                        b.celulas.escolhe(variantes[i]);
                        let mut minimo = f64::MAX;
                        for _ in 0..10 {
                            let t0 = std::time::Instant::now();
                            corre(&mut b);
                            let _ = b.gpu.device.poll(wgpu::PollType::wait_indefinitely());
                            minimo = minimo.min(t0.elapsed().as_secs_f64() * 1e3);
                            ph2d_gpu::pass_profiler::end_frame(&b.gpu.device, &b.gpu.queue);
                        }
                        let t = ph2d_gpu::pass_profiler::drain(&b.gpu.device).unwrap_or_default();
                        placa[i].push(t.span_ns as f64 / f64::from(t.frames.max(1)) / 1e6);
                        parede[i].push(minimo);
                        contas[i] = (celulas(&b), b.celulas.ultimas_paginas().len());
                    }
                }
                let mut linha = format!(
                    "[caladas] {pilha} · {n} discos + {} caladas (load {}):",
                    caladas.len(),
                    carga()
                );
                for i in 0..2 {
                    placa[i].sort_by(f64::total_cmp);
                    parede[i].sort_by(f64::total_cmp);
                    linha.push_str(&format!(
                        "\n[caladas]   calada_vazia {} · {} células · {} pág · placa {:.2} (med {:.2}) \
                         ms · parede {:.2} (med {:.2}) ms",
                        u8::from(variantes[i].calada_vazia),
                        contas[i].0,
                        contas[i].1,
                        placa[i][0],
                        placa[i][3],
                        parede[i][0],
                        parede[i][3]
                    ));
                }
                eprintln!("{linha}");
            }
        }
    }
    eprintln!("[caladas] load {carga0} → {}", carga());
    ph2d_gpu::pass_profiler::shutdown();
}

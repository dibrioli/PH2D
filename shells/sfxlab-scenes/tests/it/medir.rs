//! Sonda de velocidade (`docs/DevOps/MEDIR_VELOCIDADE.md`): variantes no mesmo processo, intercaladas, o mínimo. O resultado
//! vai para `docs/SfxLab/04_W3_plano.md` §6.
//! `cargo test --profile smoke -p ph2d-panel-sfxlab --test it diag_ -- --ignored --nocapture --test-threads=1`

use crate::Mesa;
use ph2d_audio::dsp::integrated_lufs;
use ph2d_panel_sfxlab::medidor::{espectro, pico_db};
use ph2d_sfx_recipe::{Envelope, Noise, NoiseFilter, Recipe, SAMPLE_RATE, Source};
use std::hint::black_box;
use std::time::Instant;

const BLOCOS: usize = 7;
const QUADROS_POR_BLOCO: usize = 30;

fn com_camadas(n: usize) -> Recipe {
    let mut r = Recipe::new(3, 1.0);
    for _ in 0..n {
        r.add(
            0.1,
            Source::Noise(Noise {
                filter: NoiseFilter::Band,
                hz: 800.0,
                hz_end: 800.0,
                q: 1.0,
            }),
            Envelope::hit(0.001, 0.4, 0.01),
        );
    }
    r
}

#[test]
#[ignore = "sonda: mede, não julga"]
fn diag_tempo_por_quadro_com_8_e_64_camadas() {
    let mut mesas: Vec<(usize, Mesa)> = [8, 64]
        .map(|n| {
            let mut m = Mesa::nova();
            let id = m.abre("Sonda", com_camadas(n));
            m.espera(id);
            (n, m)
        })
        .into_iter()
        .collect();
    let mut minimo = [f64::INFINITY; 2];
    for bloco in 0..BLOCOS {
        for k in 0..mesas.len() {
            let i = (k + bloco) % mesas.len();
            let t = Instant::now();
            for _ in 0..QUADROS_POR_BLOCO {
                mesas[i].1.quadro();
            }
            let ms = t.elapsed().as_secs_f64() * 1000.0 / QUADROS_POR_BLOCO as f64;
            minimo[i] = minimo[i].min(ms);
        }
    }
    for (i, (n, _)) in mesas.iter().enumerate() {
        println!(
            "quadro, {n:>2} camadas: {:.3} ms (mínimo de {BLOCOS} blocos)",
            minimo[i]
        );
    }
}

#[test]
#[ignore = "sonda: mede, não julga"]
fn diag_custo_dos_medidores_por_render() {
    let de = |s: f32| -> Vec<f32> {
        let mut x = 0x9E37_79B9_u32;
        (0..(s * SAMPLE_RATE as f32) as usize)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                (x as f32 / u32::MAX as f32) - 0.5
            })
            .collect()
    };
    for (nome, buf) in [("2 s", de(2.0)), ("120 s", de(120.0))] {
        let mut minimo = [f64::INFINITY; 3];
        for bloco in 0..BLOCOS {
            for k in 0..3 {
                let i = (k + bloco) % 3;
                let t = Instant::now();
                match i {
                    0 => {
                        black_box(pico_db(&buf));
                    }
                    1 => {
                        black_box(integrated_lufs(&buf, 1, SAMPLE_RATE));
                    }
                    _ => {
                        black_box(espectro(&buf, SAMPLE_RATE));
                    }
                }
                minimo[i] = minimo[i].min(t.elapsed().as_secs_f64() * 1000.0);
            }
        }
        println!(
            "{nome:>6}: pico {:.3} ms · LUFS {:.3} ms · espectro {:.3} ms",
            minimo[0], minimo[1], minimo[2]
        );
    }
}

/// Passo 5: da edição até o som novo tocável, e o que o limitador e a impressão digital custam por render.
#[test]
#[ignore = "medição, passo 5"]
fn medir_latencia_e_limitador() {
    use ph2d_app_sfxlab::Alvo;
    use ph2d_sfx_dsp::{limitar_escuta, render};
    use ph2d_sfx_lab::aprovar::impressao;
    use ph2d_sfx_recipe::{ENGINE_V1, SAMPLE_RATE, Takes, factory};
    use std::time::Instant;

    const BLOCOS: usize = 7;
    let receita = |s: f32| {
        let (_, mut r) = factory::all().remove(0);
        r.seconds = s;
        r
    };
    for s in [2.0_f32, 120.0] {
        let r = receita(s);
        let mut pcm = render(&r, r.seed, ENGINE_V1, &Takes::default())
            .unwrap()
            .samples;
        let pico = pcm.iter().fold(0.0_f32, |a, x| a.max(x.abs())).max(1e-6);
        // Estourado, para o limitador trabalhar de verdade (abaixo do teto ele só copia).
        pcm.iter_mut().for_each(|x| *x *= 2.0 / pico);
        let (mut lim, mut imp) = (f64::MAX, f64::MAX);
        for _ in 0..BLOCOS {
            let t = Instant::now();
            std::hint::black_box(limitar_escuta(&pcm, SAMPLE_RATE));
            lim = lim.min(t.elapsed().as_secs_f64() * 1e3);
            let t = Instant::now();
            std::hint::black_box(impressao(&pcm));
            imp = imp.min(t.elapsed().as_secs_f64() * 1e3);
        }
        println!("{s:>5} s | limitador {lim:.3} ms | impressão {imp:.3} ms");
    }
    let mut m = Mesa::nova();
    let aba = m.abre("Medida", receita(2.0));
    let camada = m.receita(aba).layers[0].id;
    let mut melhor = f64::MAX;
    for _ in 0..BLOCOS * 2 {
        m.quadro();
        let t = Instant::now();
        m.clica_na_aba(aba, Alvo::Mudo(camada));
        while m.docs().em_voo(aba) {
            m.quadro();
        }
        melhor = melhor.min(t.elapsed().as_secs_f64() * 1e3);
    }
    println!("edição (M) até o som tocável, 2 s, com os quadros da tela: {melhor:.2} ms");
}

/// O lote da W4 (`docs/SfxLab/05_W4_plano.md` §6), com o sorteio de verdade.
/// `cargo test --profile smoke -p ph2d-host-sfxlab-scenes --test it medir_exploracao -- --ignored --nocapture --test-threads=1`
#[test]
#[ignore = "sonda: mede, não reprova"]
fn medir_exploracao() {
    use ph2d_sfx_explore::{Exploracao, FILHOS, Sessao, TAMANHO_MUTATE, TAMANHO_ROLL, varia};
    use ph2d_sfx_recipe::{Body, Como, ModeLaw, Source, Takes, factory};
    use std::sync::Arc;

    let takes = Arc::new(Takes::default());
    let ms = |t: Instant| t.elapsed().as_secs_f64() * 1e3;
    let minimo = |v: &[f64]| v.iter().copied().fold(f64::INFINITY, f64::min);
    let oito = |base: &Recipe, tamanho: f64| {
        let mut x = Exploracao::default();
        let t = Instant::now();
        x.semeia(base, tamanho, &takes);
        loop {
            let _ = x.serve(base, false, &takes);
            if x.grade
                .as_ref()
                .is_some_and(|g| g.filhos.iter().all(Option::is_some))
            {
                return ms(t);
            }
            std::thread::yield_now();
        }
    };
    println!(
        "== oito variações, do pedido à última pronta, ms (mínimo de {BLOCOS} intercalado; varia real, com medida e limitador)"
    );
    for (nome, base) in factory::all() {
        let (mut a, mut b) = (vec![], vec![]);
        for bloco in 0..BLOCOS {
            if bloco % 2 == 0 {
                a.push(oito(&base, TAMANHO_MUTATE));
                b.push(oito(&base, TAMANHO_ROLL));
            } else {
                b.push(oito(&base, TAMANHO_ROLL));
                a.push(oito(&base, TAMANHO_MUTATE));
            }
        }
        println!(
            "{nome:<14} Mutate {:>7.2}   Roll {:>7.2}",
            minimo(&a),
            minimo(&b)
        );
    }

    let mut pior = Recipe::new(1, 120.0);
    pior.add(
        1.0,
        Source::Body(Body {
            law: ModeLaw::Bar,
            base_hz: 20.0,
            modes: 1080,
            t60: 60.0,
            damping: 0.0,
            rolloff: 0.5,
            contact_ms: 1.0,
            noise: 0.5,
        }),
        Envelope::default(),
    );
    let t = oito(&pior, TAMANHO_MUTATE);
    println!("pior corpo de 120 s: {t:.0} ms até a oitava");

    let pico = || {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("VmHWM"))
                    .and_then(|l| l.split_whitespace().nth(1)?.parse::<u64>().ok())
            })
            .unwrap_or(0)
    };
    let antes = pico();
    let mut oito_camadas = Recipe::new(1, 120.0);
    for _ in 0..8 {
        oito_camadas.add(
            0.1,
            Source::Noise(ph2d_sfx_recipe::Noise {
                filter: NoiseFilter::Band,
                hz: 800.0,
                hz_end: 800.0,
                q: 1.0,
            }),
            Envelope::default(),
        );
    }
    let t = oito(&oito_camadas, TAMANHO_MUTATE);
    println!(
        "8 camadas × 120 s: {t:.0} ms; memória de pico do processo subiu {} MB (a sonda já tinha {} MB)",
        (pico() - antes) / 1024,
        antes / 1024
    );

    let impact = factory::all()
        .into_iter()
        .find(|(n, _)| *n == "Impact")
        .expect("Impact")
        .1;
    for n in [1_000usize, 10_000] {
        let mut s = Sessao::default();
        for i in 0..n as u64 {
            s.ouvidos.push(ph2d_sfx_recipe::Ouvido {
                receita: varia(&impact, i + 1, TAMANHO_MUTATE),
                como: Como::Roll(i + 1),
                estrela: false,
            });
        }
        let (mut novo, mut copia) = (vec![], vec![]);
        for i in 0..BLOCOS as u64 {
            let t = Instant::now();
            s.ouviu(&varia(&impact, 1_000_000 + i, TAMANHO_MUTATE), Como::Mao);
            novo.push(ms(t));
            let t = Instant::now();
            black_box(s.ouvidos.clone());
            copia.push(ms(t));
        }
        println!(
            "Heard {n}: entrar uma linha {:.3} ms · copiar para o mundo {:.3} ms",
            minimo(&novo),
            minimo(&copia)
        );
    }

    let mut m = Mesa::nova();
    let id = m.abre("Impact", impact);
    let quadros = |m: &mut Mesa| {
        let t = Instant::now();
        for _ in 0..QUADROS_POR_BLOCO {
            m.quadro();
        }
        ms(t) / QUADROS_POR_BLOCO as f64
    };
    let (mut normal, mut grade) = (vec![], vec![]);
    for _ in 0..BLOCOS {
        normal.push(quadros(&mut m));
        assert_eq!(FILHOS, 8);
        let k = ph2d_editor_core::screens::hero::board_keys::BoardKey::Tab;
        let mods = ph2d_host::Modifiers::default();
        let (h, ts) = (&mut m.hero, &mut m.ts);
        let _ = ph2d_editor_core::screens::hero::board_keys::key(h, ts, k, mods, true, None, None);
        while m.docs().exploracao(id).expect("aba").em_voo() {
            m.quadro();
        }
        grade.push(quadros(&mut m));
        let (h, ts) = (&mut m.hero, &mut m.ts);
        let _ = ph2d_editor_core::screens::hero::board_keys::key(h, ts, k, mods, true, None, None);
    }
    println!(
        "quadro da tela, CPU: normal {:.3} ms · Explore {:.3} ms",
        minimo(&normal),
        minimo(&grade)
    );
}

/// A pergunta do limite da `Ninhada` (`docs/SfxLab/05_W4_plano.md` §6): pedidos seguidos do pior corpo de 120 s. `R` preso são oito
/// por tecla; 1000 pedidos dizem se o pico cresce com o tamanho da rajada.
/// `cargo test --profile smoke -p ph2d-host-sfxlab-scenes --test it medir_ninhada -- --ignored --nocapture --test-threads=1`
#[test]
#[ignore = "sonda: mede, não reprova"]
fn medir_ninhada() {
    use ph2d_sfx_explore::{FILHOS, Ninhada};
    use ph2d_sfx_recipe::{Body, ModeLaw, Source, Takes};
    use std::sync::Arc;
    use std::time::Duration;

    let vivas = || {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|s| {
                s.lines()
                    .find(|l| l.starts_with("Threads"))
                    .and_then(|l| l.split_whitespace().nth(1)?.parse::<i64>().ok())
            })
            .unwrap_or(0)
    };
    let mut pior = Recipe::new(1, 120.0);
    pior.add(
        1.0,
        Source::Body(Body {
            law: ModeLaw::Bar,
            base_hz: 20.0,
            modes: 1080,
            t60: 60.0,
            damping: 0.0,
            rolloff: 0.5,
            contact_ms: 1.0,
            noise: 0.5,
        }),
        Envelope::default(),
    );
    let takes = Arc::new(Takes::default());
    let corrida = |pedidos: usize, lugares: usize, por_vez: usize, intervalo: Duration| {
        let antes = vivas();
        let mut n = Ninhada::default();
        let mut pico = 0;
        let t = Instant::now();
        for i in 0..pedidos {
            n.pede(i % lugares, pior.clone(), takes.clone());
            pico = pico.max(vivas() - antes);
            if !intervalo.is_zero() && (i + 1) % por_vez == 0 {
                std::thread::sleep(intervalo);
            }
        }
        let pedir = t.elapsed().as_secs_f64() * 1e3;
        let t = Instant::now();
        drop(n);
        (pico, pedir, t.elapsed().as_secs_f64() * 1e3)
    };
    println!("== o pior corpo de 120 s (o maior de {BLOCOS} blocos intercalados)");
    let quadro = Duration::from_millis(16);
    let casos = [
        ("200, 1 lugar, sem pausa", 200, 1, 1, Duration::ZERO),
        ("1000, 1 lugar, sem pausa", 1000, 1, 1, Duration::ZERO),
        ("200, 1 lugar, 1 por quadro", 200, 1, 1, quadro),
        ("200, 8 lugares, sem pausa", 200, FILHOS, 1, Duration::ZERO),
        (
            "1000, 8 lugares, sem pausa",
            1000,
            FILHOS,
            1,
            Duration::ZERO,
        ),
        ("200, 8 lugares, 8 por quadro", 200, FILHOS, FILHOS, quadro),
    ];
    let mut maximos = [(0, 0.0f64, 0.0f64); 6];
    for bloco in 0..BLOCOS {
        for k in 0..casos.len() {
            let i = (k + bloco) % casos.len();
            let (pico, pedir, soltar) = corrida(casos[i].1, casos[i].2, casos[i].3, casos[i].4);
            let m = &mut maximos[i];
            *m = (m.0.max(pico), m.1.max(pedir), m.2.max(soltar));
        }
    }
    for (c, m) in casos.iter().zip(maximos) {
        println!(
            "{:<30} pico de threads {:>4} · pedir tudo {:>8.1} ms · Drop {:>7.1} ms",
            c.0, m.0, m.1, m.2
        );
    }
}

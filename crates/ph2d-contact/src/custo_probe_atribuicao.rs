//! **ONDE, DENTRO DE UMA VARREDURA, O TEMPO MORA** — irmã do [`super`] pelo tecto de LOC (HR-18) e
//! por ASSUNTO: ali mora QUANTO a separação custa, aqui mora de que é feito esse custo.
//!
//! ⚠️ **As duas atribuições não são redundantes:** uma corre com CAIXAS num campo espalhado e a
//! outra com DISCOS encostados — *a forma decide onde o tempo mora* (o par disco-disco tem saída
//! rápida; caixa-caixa faz SAT), e foi a segunda que mostrou que a escrituração já não é o alvo.
//!
//! ⛔⛔ E as duas do PARALELO existem porque a razão série/paralelo **mente sozinha**: `1,5×` em 32
//! núcleos lê-se igual quer o trabalho não esteja a ser espalhado, quer a máquina esteja ocupada. O
//! discriminador é o **tempo de CPU contra o de parede**, e ele diz que o passe ocupa `4`–`5`
//! núcleos e gasta `5×` o CPU da série — *o que se paga é o `fork/join` por varredura*.

use super::*;
use std::time::Instant;

/// ⭐⭐⭐ **ONDE O TEMPO MORA DENTRO DE UMA VARREDURA** — a atribuição antes de qualquer cura.
///
/// ⚠️ Ela chama as MESMAS funções do produto (`grelha`, `celula`, `corrigida`), nunca uma segunda
/// cópia da lei: o que ela faz é cronometrar as fases **separadamente**, somando ao lado.
#[test]
#[ignore = "sonda de medição, não gate"]
fn onde_o_tempo_mora_dentro_de_uma_varredura() {
    const N: usize = 500;
    const REPS: usize = 200;
    eprintln!("\n  ═══ ATRIBUIÇÃO DE UMA VARREDURA ({N} peças) ═══\n");
    let (p0, c, w) = campo(N, ESPACO_DE_CENA);
    let inv = vec![0.0; N];
    let pecas = Pecas::novas(&c, &w, &inv);
    let ativo: Vec<bool> = (0..N).map(|i| ativo(p0[i], c[i].as_ref())).collect();
    let alcance = (0..N)
        .filter_map(|i| c[i].map(|x| x.alcance()))
        .fold(0.0f32, f32::max);
    let lado = 2.0 * alcance;

    let cron = |f: &mut dyn FnMut()| {
        let mut melhor = f64::INFINITY;
        for _ in 0..CORRIDAS {
            let agora = Instant::now();
            for _ in 0..REPS {
                f();
            }
            melhor = melhor.min(agora.elapsed().as_secs_f64() * 1e6 / REPS as f64);
        }
        melhor
    };

    // (a) só construir a grelha
    let t_grelha = cron(&mut || {
        let mut g = crate::grelha::Grelha::default();
        g.planeia_numa_camada(&ativo, lado);
        g.constroi(&p0, &ativo);
        std::hint::black_box(&g);
    });
    // (b) construir + colher os vizinhos das 9 células, sem tocar na lei
    let t_colher = cron(&mut || {
        let mut g = crate::grelha::Grelha::default();
        g.planeia_numa_camada(&ativo, lado);
        g.constroi(&p0, &ativo);
        let mut viz: Vec<u32> = Vec::new();
        let mut total = 0usize;
        for k in 0..N {
            g.vizinhos_de(k, &mut viz);
            total += viz.len();
        }
        std::hint::black_box(total);
    });
    // (c) a varredura inteira, pela porta do produto
    let t_tudo = cron(&mut || {
        let mut p = p0.clone();
        let mut g = vec![0.0; N];
        separate(&mut p, &mut Saida { giro: &mut g }, &pecas, 1);
        std::hint::black_box(&p);
    });

    // Quantos parceiros cada peça de facto vê — o que a LEI custa é proporcional a isto.
    let mut g = crate::grelha::Grelha::default();
    g.planeia_numa_camada(&ativo, lado);
    g.constroi(&p0, &ativo);
    let mut viz: Vec<u32> = Vec::new();
    let mut soma = 0usize;
    for k in 0..N {
        g.vizinhos_de(k, &mut viz);
        soma += viz.len();
    }
    eprintln!(
        "  parceiros por peça (média) .... {:.1}",
        soma as f64 / N as f64
    );
    eprintln!("  (a) construir a grelha ........ {t_grelha:>8.1} µs");
    eprintln!("  (b) (a) + colher e ordenar .... {t_colher:>8.1} µs");
    eprintln!("  (c) a varredura inteira ....... {t_tudo:>8.1} µs");
    eprintln!(
        "\n  ⇒ achar os pares: {:.0}%   ·   a LEI: {:.0}%",
        t_colher / t_tudo * 100.0,
        (t_tudo - t_colher) / t_tudo * 100.0
    );
    eprintln!("\n  load durante a corrida: {}\n", carga());
}

/// ⭐⭐⭐ **A PARTIR DE QUANTAS PEÇAS O PARALELO PAGA** — de onde sai a [`PECAS_PARA_PARALELIZAR`].
///
/// ⚠️ A régua é a RAZÃO entre as duas colunas, e não um relógio absoluto: sob carga as duas sobem
/// juntas. O ponto de equilíbrio é onde a razão cruza `1`.
#[test]
#[ignore = "sonda de medição, não gate"]
fn onde_o_paralelo_passa_a_pagar() {
    const V: usize = 64;
    eprintln!("\n  ═══ ONDE O PARALELO PASSA A PAGAR ({V} varreduras) ═══\n");
    eprintln!(
        "  {:<8} │ {:>12} │ {:>12} │ {:>9}",
        "peças", "1 núcleo", "N núcleos", "razão"
    );
    eprintln!("  ---------|--------------|--------------|----------");
    for n in [16usize, 32, 64, 128, 256, 500, 1000, 4000] {
        let (p0, c, w) = campo(n, ESPACO_DE_CENA);
        let inv: Vec<f32> = (0..n)
            .map(|i| c[i].map_or(0.0, |x| x.inv_inercia(w[i])))
            .collect();
        let pecas = Pecas::novas(&c, &w, &inv);
        let mut col = [0.0f64; 2];
        for (i, paralelo) in [false, true].into_iter().enumerate() {
            let mut melhor = f64::INFINITY;
            for _ in 0..CORRIDAS {
                let mut p = p0.clone();
                let mut g = vec![0.0; n];
                let agora = Instant::now();
                separate_com(
                    &mut p,
                    &mut Saida { giro: &mut g },
                    &pecas,
                    V,
                    paralelo,
                    REPOUSO_VISIVEL,
                );
                melhor = melhor.min(agora.elapsed().as_secs_f64() * 1e3);
            }
            col[i] = melhor;
        }
        eprintln!(
            "  {n:<8} │ {:>9.3} ms │ {:>9.3} ms │ {:>8.2}×",
            col[0],
            col[1],
            col[0] / col[1]
        );
    }
    eprintln!("\n  load durante a corrida: {}\n", carga());
}

/// ⭐⭐⭐ **ATRIBUIÇÃO NA CENA DO DONO — discos encostados** (report de 2026-09-18: *«roda bem com
/// Sweeps 64 (500 objetos = 100 FPS). Mas não vamos tentar chegar nos 1000?»*).
///
/// ⚠️ A atribuição irmã mede CAIXAS num campo espalhado. Esta mede o que ele tem: **discos**, e
/// encostados — *e a forma decide onde o tempo mora*, porque o par disco-disco tem saída rápida e a
/// caixa-caixa faz SAT.
#[test]
#[ignore = "sonda de medição, não gate"]
fn onde_o_tempo_mora_com_discos() {
    const N: usize = 500;
    const RAIO: f32 = 100.0;
    const REPS: usize = 40;
    let (p0, c, w) = campo_de_discos(N, RAIO, 1.8);
    let inv: Vec<f32> = (0..N)
        .map(|i| c[i].map_or(0.0, |x| x.inv_inercia(w[i])))
        .collect();
    let pecas = Pecas::novas(&c, &w, &inv);
    let ativo: Vec<bool> = (0..N).map(|i| ativo(p0[i], c[i].as_ref())).collect();
    let cron = |f: &mut dyn FnMut()| {
        let mut melhor = f64::INFINITY;
        for _ in 0..CORRIDAS {
            let agora = Instant::now();
            for _ in 0..REPS {
                f();
            }
            #[expect(clippy::cast_precision_loss, reason = "uma contagem de repeticoes")]
            let reps = REPS as f64;
            melhor = melhor.min(agora.elapsed().as_secs_f64() * 1e6 / reps);
        }
        melhor
    };
    let t_grelha = cron(&mut || {
        let mut g = crate::grelha::Grelha::default();
        g.planeia_numa_camada(&ativo, 2.0 * RAIO);
        g.constroi(&p0, &ativo);
        std::hint::black_box(&g);
    });
    let t_colher = cron(&mut || {
        let mut g = crate::grelha::Grelha::default();
        g.planeia_numa_camada(&ativo, 2.0 * RAIO);
        g.constroi(&p0, &ativo);
        let mut viz: Vec<u32> = Vec::new();
        let mut total = 0usize;
        for k in 0..N {
            g.vizinhos_de(k, &mut viz);
            total += viz.len();
        }
        std::hint::black_box(total);
    });
    // ⭐ O `girado` de cada peça: para um DISCO sem desvio ele devolve o mesmo colisor — e paga
    // duas chamadas de trigonometria para isso.
    let t_girado = cron(&mut || {
        let mut s = 0.0f32;
        for (i, x) in c.iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "um indice de fixtura")]
            let g = 0.3 + i as f32 * 1e-4;
            if let Some(v) = x.map(|x| x.girado(g)) {
                s += v.alcance();
            }
        }
        std::hint::black_box(s);
    });
    let t_tudo = cron(&mut || {
        let mut p = p0.clone();
        let mut g = vec![0.0; N];
        separate_com(
            &mut p,
            &mut Saida { giro: &mut g },
            &pecas,
            1,
            false,
            REPOUSO_VISIVEL,
        );
        std::hint::black_box(&p);
    });
    eprintln!("\n  ═══ ATRIBUIÇÃO COM DISCOS ENCOSTADOS ({N} peças, série) ═══\n");
    eprintln!("  (a) construir a grelha ........ {t_grelha:>8.1} µs");
    eprintln!("  (b) (a) + colher e ordenar .... {t_colher:>8.1} µs");
    eprintln!("  (c) girar os colisores ........ {t_girado:>8.1} µs  ⇠ um DISCO não muda ao rodar");
    eprintln!("  (d) a varredura inteira ....... {t_tudo:>8.1} µs");
    eprintln!(
        "\n  ⇒ achar os pares {:.0}%  ·  girar {:.0}%  ·  a LEI {:.0}%",
        t_colher / t_tudo * 100.0,
        t_girado / t_tudo * 100.0,
        (t_tudo - t_colher - t_girado) / t_tudo * 100.0
    );
    eprintln!("\n  load: {}\n", carga());
}

/// ⭐⭐⭐ **QUANTO O PARALELO RENDE NA CENA DO DONO** — e quanto dele o FORK come.
///
/// ⚠️ O laço faz `fork/join` **por varredura**. Esta sonda separa as duas coisas: a razão
/// série/paralelo por varredura, e o custo de uma varredura PARALELA contra o trabalho que ela
/// distribui — se a razão não subir com o `n`, o que manda é o arranque e não o trabalho.
#[test]
#[ignore = "sonda de medição, não gate"]
fn quanto_o_paralelo_rende_com_discos() {
    const RAIO: f32 = 100.0;
    eprintln!("\n  ═══ O PARALELO NA CENA DO DONO (discos encostados) ═══\n");
    eprintln!(
        "  {:<8} │ {:<7} │ {:>12} │ {:>12} │ {:>8} │ {:>14}",
        "discos", "varred.", "1 núcleo", "N núcleos", "razão", "µs/varredura"
    );
    eprintln!("  ---------|---------|--------------|--------------|----------|---------------");
    for (n, v) in [
        (500usize, 64usize),
        (500, 1024),
        (1000, 64),
        (1000, 1024),
        (2000, 64),
        (4000, 64),
    ] {
        let (p0, c, w) = campo_de_discos(n, RAIO, 1.8);
        let inv: Vec<f32> = (0..n)
            .map(|i| c[i].map_or(0.0, |x| x.inv_inercia(w[i])))
            .collect();
        let pecas = Pecas::novas(&c, &w, &inv);
        let mut col = [0.0f64; 2];
        for (i, paralelo) in [false, true].into_iter().enumerate() {
            let mut melhor = f64::INFINITY;
            for _ in 0..5 {
                let mut p = p0.clone();
                let mut g = vec![0.0; n];
                let agora = Instant::now();
                separate_com(
                    &mut p,
                    &mut Saida { giro: &mut g },
                    &pecas,
                    v,
                    paralelo,
                    REPOUSO_VISIVEL,
                );
                melhor = melhor.min(agora.elapsed().as_secs_f64() * 1e3);
            }
            col[i] = melhor;
        }
        #[expect(clippy::cast_precision_loss, reason = "uma contagem de varreduras")]
        let vf = v as f64;
        eprintln!(
            "  {n:<8} │ {v:<7} │ {:>9.2} ms │ {:>9.2} ms │ {:>7.2}× │ {:>11.1} µs",
            col[0],
            col[1],
            col[0] / col[1],
            col[1] / vf * 1e3
        );
    }
    eprintln!("\n  load: {}\n", carga());
}

/// ⭐⭐⭐ **O PARALELO ACONTECE, OU A MÁQUINA ESTÁ OCUPADA?** — o discriminador que a CARGA não
/// estraga.
///
/// ⚠️ Uma razão série/paralelo de `1,5×` em 32 núcleos lê-se igual nos dois casos. O que os separa é
/// o **TEMPO DE CPU**: se a corrida paralela consome `N×` o tempo de parede dela, houve `N` núcleos
/// a trabalhar — e o pouco ganho é contenção com os vizinhos. Se consome `~1×`, o trabalho **não
/// está a ser espalhado**, e o defeito é nosso.
#[test]
#[ignore = "sonda de medição, não gate"]
fn o_paralelo_acontece_ou_a_maquina_esta_ocupada() {
    const RAIO: f32 = 100.0;
    eprintln!("\n  ═══ NÚCLEOS DE FACTO OCUPADOS PELO PASSE ═══\n");
    eprintln!(
        "  {:<8} │ {:<9} │ {:>11} │ {:>11} │ {:>12}",
        "discos", "rota", "parede", "CPU", "núcleos"
    );
    eprintln!("  ---------|-----------|-------------|-------------|-------------");
    for n in [1000usize, 4000] {
        let (p0, c, w) = campo_de_discos(n, RAIO, 1.8);
        let inv: Vec<f32> = (0..n)
            .map(|i| c[i].map_or(0.0, |x| x.inv_inercia(w[i])))
            .collect();
        let pecas = Pecas::novas(&c, &w, &inv);
        for (rota, paralelo) in [("série", false), ("paralelo", true)] {
            let mut p = p0.clone();
            let mut g = vec![0.0; n];
            let (c0, t0) = (cpu_segundos(), Instant::now());
            separate_com(
                &mut p,
                &mut Saida { giro: &mut g },
                &pecas,
                256,
                paralelo,
                REPOUSO_VISIVEL,
            );
            let parede = t0.elapsed().as_secs_f64();
            let cpu = cpu_segundos() - c0;
            eprintln!(
                "  {n:<8} │ {rota:<9} │ {:>8.1} ms │ {:>8.1} ms │ {:>11.1}×",
                parede * 1e3,
                cpu * 1e3,
                cpu / parede.max(1e-9)
            );
        }
    }
    eprintln!("\n  ⚠️ `núcleos` = CPU/parede: quantos núcleos o passe de facto teve a trabalhar.");
    eprintln!("  load: {}\n", carga());
}

/// ⭐⭐⭐ **DENTRO DA LEI: quanto é a GEOMETRIA do par, e quanto é a restrição?**
///
/// ⚠️ **Cada par é avaliado DUAS vezes** — uma do lado de `k`, outra do lado de `j` —, e a
/// geometria (o [`manifesto`]) é a mesma nas duas. Esta sonda mede o que se ganharia calculando-a
/// uma vez só: se ela for metade da lei, o ganho vale um refactor; se for um décimo, não vale.
#[test]
#[ignore = "sonda de medição, não gate"]
fn quanto_da_lei_e_a_geometria_do_par() {
    const N: usize = 1000;
    const RAIO: f32 = 100.0;
    const REPS: usize = 40;
    let (p0, c, w) = campo_de_discos(N, RAIO, 1.8);
    let inv: Vec<f32> = (0..N)
        .map(|i| c[i].map_or(0.0, |x| x.inv_inercia(w[i])))
        .collect();
    let pecas = Pecas::novas(&c, &w, &inv);
    let ativo: Vec<bool> = (0..N).map(|i| ativo(p0[i], c[i].as_ref())).collect();
    let mut grade = crate::grelha::Grelha::default();
    grade.planeia_numa_camada(&ativo, 2.0 * RAIO);
    grade.constroi(&p0, &ativo);
    // A lista de candidatos, colhida uma vez — ela é comum às duas colunas.
    let mut candidatos: Vec<(usize, usize)> = Vec::new();
    let mut viz: Vec<u32> = Vec::new();
    for k in 0..N {
        grade.vizinhos_de(k, &mut viz);
        for &j in &viz {
            candidatos.push((k, j as usize));
        }
    }
    let cron = |f: &mut dyn FnMut()| {
        let mut melhor = f64::INFINITY;
        for _ in 0..CORRIDAS {
            let agora = Instant::now();
            for _ in 0..REPS {
                f();
            }
            #[expect(clippy::cast_precision_loss, reason = "uma contagem de repeticoes")]
            let reps = REPS as f64;
            melhor = melhor.min(agora.elapsed().as_secs_f64() * 1e6 / reps);
        }
        melhor
    };
    // (a) só a geometria de cada candidato — o que um cache por par evitaria repetir.
    let t_geo = cron(&mut || {
        let mut toques = 0usize;
        for &(k, j) in &candidatos {
            if k == j {
                continue;
            }
            let (lo, hi) = (k.min(j), k.max(j));
            if let (Some(a), Some(b)) = (c[lo], c[hi])
                && manifesto(&a, p0[lo], &b, p0[hi], (lo + hi) % 2 == 0).is_some()
            {
                toques += 1;
            }
        }
        std::hint::black_box(toques);
    });
    // (b) a varredura inteira, pela porta do produto.
    let t_tudo = cron(&mut || {
        let mut p = p0.clone();
        let mut g = vec![0.0; N];
        separate_com(
            &mut p,
            &mut Saida { giro: &mut g },
            &pecas,
            1,
            false,
            REPOUSO_VISIVEL,
        );
        std::hint::black_box(&p);
    });
    eprintln!("\n  ═══ A GEOMETRIA DENTRO DA LEI ({N} discos) ═══\n");
    eprintln!("  candidatos por varredura ...... {}", candidatos.len());
    eprintln!("  (a) só a geometria ............ {t_geo:>8.1} µs");
    eprintln!("  (b) a varredura inteira ....... {t_tudo:>8.1} µs");
    eprintln!(
        "\n  ⇒ a geometria é {:.0}% da varredura; calculá-la UMA vez por par pouparia ~{:.0}%.",
        t_geo / t_tudo * 100.0,
        t_geo / t_tudo * 50.0
    );
    eprintln!("\n  load: {}\n", carga());
}

/// Uma grelha hexagonal apertada de discos — a cena da foto.
pub(super) fn campo_de_discos(
    n: usize,
    raio: f32,
    passo_r: f32,
) -> (Vec<[f32; 2]>, Vec<Option<Colisor>>, Vec<f32>) {
    let passo = passo_r * raio;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss,
        reason = "um lado de grelha de fixtura"
    )]
    let cols = ((n as f32).sqrt() * 1.15).ceil() as usize;
    let mut p = Vec::with_capacity(n);
    let mut c = Vec::with_capacity(n);
    for i in 0..n {
        let (gx, gy) = (i % cols, i / cols);
        #[expect(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            reason = "coordenadas de fixtura"
        )]
        let (fx, fy, k) = (gx as f32, gy as f32, i as u32);
        #[expect(clippy::cast_precision_loss, reason = "paridade da linha")]
        let desloca = (gy % 2) as f32 * passo * 0.5;
        p.push([
            fx * passo + desloca + (acaso(k, 1) - 0.5) * passo * 0.08,
            fy * passo * 0.87 + (acaso(k, 2) - 0.5) * passo * 0.08,
        ]);
        c.push(Some(Colisor::disco(raio)));
    }
    (p, c, vec![1.0; n])
}

/// O tempo de CPU deste processo (utime + stime), em segundos.
fn cpu_segundos() -> f64 {
    let s = std::fs::read_to_string("/proc/self/stat").unwrap_or_default();
    // O campo 14 e o 15, contando depois do `)` do nome (que pode conter espaços).
    let Some(resto) = s.rsplit_once(')').map(|(_, r)| r) else {
        return 0.0;
    };
    let campos: Vec<&str> = resto.split_whitespace().collect();
    let tick = |i: usize| {
        campos
            .get(i)
            .and_then(|x| x.parse::<f64>().ok())
            .unwrap_or(0.0)
    };
    // `resto` começa no campo 3, logo utime (14) e stime (15) estão em 11 e 12.
    (tick(11) + tick(12)) / 100.0
}

/// ⭐⭐⭐ **O PISO DA TAREFA** — de onde sai a [`crate::PISO_DA_TAREFA`].
///
/// ⚠️⚠️ **DUAS escolhas minhas caíram aqui.** A 1.ª foi um número redondo (`64` peças por tarefa):
/// com `1000` peças são **16 tarefas** numa máquina de 32 núcleos. A 2.ª derivava o número de
/// PEDAÇOS do `n` e dos núcleos, e **piorou o app do dono de `18,6` para `30,4 ms`** — fixar os
/// pedaços tira ao rayon a decisão de partir só quando há quem trabalhe, e isso só se nota com a
/// máquina PARADA (as minhas tabelas saíram todas a `load 22`–`32`).
///
/// A coluna que decide é a dos **núcleos** (CPU/parede): ela diz quantos de facto trabalharam, e a
/// carga da máquina não a estraga como estraga o relógio.
#[test]
#[ignore = "sonda de medição, não gate"]
fn o_grao_da_tarefa() {
    const RAIO: f32 = 100.0;
    const V: usize = 64;
    let nucleos = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    eprintln!("\n  ═══ O PISO DA TAREFA ({nucleos} núcleos na máquina) ═══\n");
    eprintln!(
        "  {:<8} │ {:<7} │ {:>8} │ {:>11} │ {:>11} │ {:>9}",
        "discos", "piso", "n/piso", "parede", "CPU", "núcleos"
    );
    eprintln!("  ---------|---------|----------|-------------|-------------|----------");
    for n in [1000usize, 4000] {
        let (p0, c, w) = campo_de_discos(n, RAIO, 0.5);
        let inv: Vec<f32> = (0..n)
            .map(|i| c[i].map_or(0.0, |x| x.inv_inercia(w[i])))
            .collect();
        let pecas = Pecas::novas(&c, &w, &inv);
        for grao in [8usize, 16, 32, 64, 128, 256] {
            let mut melhor = (f64::INFINITY, 0.0f64);
            for _ in 0..3 {
                let mut p = p0.clone();
                let mut g = vec![0.0; n];
                let (c0, t0) = (cpu_segundos(), Instant::now());
                separate_grao(
                    &mut p,
                    &mut Saida { giro: &mut g },
                    &pecas,
                    V,
                    &Cercas {
                        paralelo: true,
                        repouso: REPOUSO_VISIVEL,
                        grao,
                        duas_camadas: false,
                    },
                );
                let parede = t0.elapsed().as_secs_f64();
                if parede < melhor.0 {
                    melhor = (parede, cpu_segundos() - c0);
                }
            }
            eprintln!(
                "  {n:<8} │ {grao:<7} │ {:>8} │ {:>8.1} ms │ {:>8.1} ms │ {:>8.1}×",
                n.div_ceil(grao),
                melhor.0 * 1e3,
                melhor.1 * 1e3,
                melhor.1 / melhor.0.max(1e-9)
            );
        }
    }
    eprintln!("\n  load: {}\n", carga());
}

/// ⭐⭐⭐ **QUANTO AS DUAS CAMADAS COMPRAM NO RELÓGIO** — a contagem já disse `11,3 ×` em
/// candidatos; isto diz o que isso vale em microssegundos.
///
/// ⚠️ **O A/B é entre dois PLANOS da MESMA grelha** ([`Grelha::planeia`] contra
/// [`Grelha::planeia_numa_camada`]), e não entre duas versões do ficheiro — *uma medição contra um
/// binário antigo não é reproduzível por quem vier a seguir*. Tudo o resto — a construção, a colheita
/// e a [`crate::varredura::corrigida`] — é literalmente o mesmo código.
#[test]
#[ignore = "sonda de medição, não gate"]
fn quanto_as_duas_camadas_compram_no_relogio() {
    const N: usize = 1000;
    const RAIO: f32 = 100.0;
    const REPS: usize = 20;
    const VARR: f64 = 64.0;
    let (p0, mut c, w) = campo_de_discos(N, RAIO, 1.8);
    c[0] = Some(Colisor::disco(4.0 * RAIO)); // a dispersão que a cena do dono tem
    let inv: Vec<f32> = (0..N)
        .map(|i| c[i].map_or(0.0, |x| x.inv_inercia(w[i])))
        .collect();
    let pecas = Pecas::novas(&c, &w, &inv);
    let ativo: Vec<bool> = (0..N).map(|i| ativo(p0[i], c[i].as_ref())).collect();
    let alcances = crate::grelha::alcances_de(&c, &ativo);
    let alcance_max = alcances.iter().fold(0.0_f32, |a, b| a.max(*b));
    let uma_vez = |duas: bool| {
        let mut g = crate::grelha::Grelha::default();
        if duas {
            g.planeia(&p0, &ativo, &alcances);
        } else {
            g.planeia_numa_camada(&ativo, 2.0 * alcance_max);
        }
        g.constroi(&p0, &ativo);
        let (mut viz, mut total, mut cand) = (Vec::<u32>::new(), 0usize, 0usize);
        for k in 0..N {
            g.vizinhos_de(k, &mut viz);
            cand += viz.len();
            if crate::varredura::corrigida(
                k,
                viz.iter().map(|&j| j as usize),
                &p0,
                &c,
                &pecas,
                &ativo,
            )
            .is_some()
            {
                total += 1;
            }
        }
        std::hint::black_box(total);
        cand
    };
    let cron = |f: &mut dyn FnMut() -> usize| {
        let mut melhor = f64::INFINITY;
        let mut cand = 0usize;
        for _ in 0..CORRIDAS {
            let agora = Instant::now();
            for _ in 0..REPS {
                cand = f();
            }
            #[expect(clippy::cast_precision_loss, reason = "uma contagem de repeticoes")]
            let reps = REPS as f64;
            melhor = melhor.min(agora.elapsed().as_secs_f64() * 1e6 / reps);
        }
        (melhor, cand)
    };
    let (t_uma, c_uma) = cron(&mut || uma_vez(false));
    let (t_duas, c_duas) = cron(&mut || uma_vez(true));
    eprintln!("\n  ═══ O QUE AS DUAS CAMADAS COMPRAM ({N} discos, UMA peça a 4 × R) ═══\n");
    eprintln!("   plano        │ candidatos │ uma varredura │ 64 varreduras");
    eprintln!("  ──────────────┼────────────┼───────────────┼──────────────");
    eprintln!(
        "   uma camada   │ {c_uma:>10} │ {t_uma:>10.1} µs │ {:>9.2} ms",
        t_uma * VARR / 1000.0
    );
    eprintln!(
        "   duas camadas │ {c_duas:>10} │ {t_duas:>10.1} µs │ {:>9.2} ms",
        t_duas * VARR / 1000.0
    );
    eprintln!("\n  ⇒ {:.2} × no relógio de uma varredura.", t_uma / t_duas);
    // ⭐ E o mesmo pela PORTA DO PRODUTO, com o paralelo — o número que o quadro do dono sente.
    // ⚠️ O A/B desta linha faz-se correndo a sonda DUAS vezes, com a [`crate::MARGEM_DO_CORTE`]
    // mutada para `1e9` na segunda (é a mutação `R1` do arnês): por-passe o plano não é parâmetro,
    // e uma 8.ª entrada no `separate_grao` acordava o `too_many_arguments`.
    let mut melhor = f64::INFINITY;
    for _ in 0..CORRIDAS {
        let mut p = p0.clone();
        let mut g = vec![0.0_f32; N];
        let agora = Instant::now();
        let v = separate(&mut p, &mut Saida { giro: &mut g }, &pecas, 64);
        melhor = melhor.min(agora.elapsed().as_secs_f64() * 1e3);
        std::hint::black_box((v, &p));
    }
    eprintln!("\n  de ponta a ponta (separate, 64 varreduras, paralelo): {melhor:>7.2} ms");
    eprintln!("\n  load: {}\n", carga());
}

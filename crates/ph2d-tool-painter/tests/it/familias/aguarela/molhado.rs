//! **Molhado × molhado** (J5, a FILA 1; `docs/Painter/Pinceis/exames/aguarela.md` «Motor J5»): a sonda da cena `sobre`
//! e o tubo da cena `largoazul` (oráculo da janela 9).

use crate::exame_bancada::{Fonte, Pincel, bateria, pinceis, pinta_cena};

/// A cena `cena` (foto `foto`) pintada por `nome` (preset da biblioteca ou caminho) com o mapa `engine` sobreposto.
pub(super) fn foto(
    nome: &str,
    engine: &[(String, f64)],
    cena: &str,
    foto: usize,
) -> (Vec<u8>, usize) {
    let p = pincel(nome, engine);
    let bat = bateria();
    let c = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == cena)
        .unwrap()
        .clone();
    foto_da_cena(p, &c, foto)
}

/// A cena `c` (JSON da bateria) pintada por `p`, na tela e no papel da bateria.
fn foto_da_cena(p: Pincel, c: &serde_json::Value, foto: usize) -> (Vec<u8>, usize) {
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    (
        pinta_cena(&p, None, c, tela, papel, foto).1,
        tela[0] as usize,
    )
}

/// `PH2D_MOLHADO_PINCEIS` (`a|b|…`) × `PH2D_MOLHADO_VARIANTES` (JSON `[{knob: v}, …]`) → `$PH2D_MOLHADO_OUT/<i>_<v>.ppm`.
#[test]
#[ignore = "sonda: o cruzamento molhado (J5); corre-se à mão"]
fn sonda_o_molhado() {
    let out = std::env::var("PH2D_MOLHADO_OUT").unwrap_or_else(|_| "/tmp".into());
    let nomes = std::env::var("PH2D_MOLHADO_PINCEIS").unwrap_or_else(|_| "Wet Liner".into());
    let vs: Vec<serde_json::Map<String, serde_json::Value>> = serde_json::from_str(
        &std::env::var("PH2D_MOLHADO_VARIANTES").unwrap_or_else(|_| "[{}]".into()),
    )
    .unwrap();
    let cena = std::env::var("PH2D_MOLHADO_CENA").unwrap_or_else(|_| "sobre".into());
    let n_foto: usize = std::env::var("PH2D_MOLHADO_FOTO")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    type Tarefa<'a> = (usize, &'a str, usize, Vec<(String, f64)>);
    let tarefas: Vec<Tarefa> = nomes
        .split('|')
        .enumerate()
        .flat_map(|(i, n)| {
            vs.iter().enumerate().map(move |(k, m)| {
                (
                    i,
                    n,
                    k,
                    m.iter()
                        .map(|(a, b)| (a.clone(), b.as_f64().unwrap()))
                        .collect::<Vec<_>>(),
                )
            })
        })
        .collect();
    std::thread::scope(|s| {
        for (i, n, k, e) in &tarefas {
            let out = &out;
            let cena = &cena;
            s.spawn(move || {
                let (px, w) = foto(n, e, cena, n_foto);
                let h = px.len() / 4 / w;
                let mut f = format!("P6\n{w} {h}\n255\n").into_bytes();
                for p in px.as_chunks::<4>().0 {
                    f.extend_from_slice(&p[..3]);
                }
                std::fs::write(format!("{out}/{i}_{k}.ppm"), f).unwrap();
                println!("[molhado] {i}_{k} = {n} {e:?}");
            });
        }
    });
}

/// A MANCHA do cruzamento molhado (cena `sobre`, foto A; a régua da mancha do relatório), a média dos três verticais:
/// `(redondo, escuro)` — a largura da mancha no meio da faixa (y 150: a distância de cor à faixa longe passa metade da do
/// centro, contínua a partir dele) ÷ a do vertical no papel (y 240), e a escuridão do miolo ÷ a do azul no papel.
/// O Rebelle `Liner` 1,42 · 1,06; `Mop` 1,44 · 1,06; `Wash` 1,17 · 1,30.
pub(super) fn mancha(px: &[u8], w: usize) -> (f64, f64) {
    let rgb = |x: usize, y: usize| {
        let i = (y * w + x) * 4;
        [0, 1, 2].map(|k| f64::from(px[i + k]))
    };
    let esc = |x: usize, y: usize| {
        let c = rgb(x, y);
        255.0 - (0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2])
    };
    let media = |xs: std::ops::Range<usize>,
                 ys: std::ops::Range<usize>,
                 f: &dyn Fn(usize, usize) -> f64| {
        let n = (xs.len() * ys.len()) as f64;
        ys.clone()
            .flat_map(|y| xs.clone().map(move |x| (x, y)))
            .map(|(x, y)| f(x, y))
            .sum::<f64>()
            / n
    };
    let (mut redondo, mut escuro) = (0.0, 0.0);
    for c in [250usize, 500, 750] {
        let longe: [f64; 3] =
            [0, 1, 2].map(|k| media(c + 100..c + 126, 140..161, &|x, y| rgb(x, y)[k]));
        let d: Vec<f64> = (c - 70..=c + 70)
            .map(|x| {
                let m = [0, 1, 2].map(|k| media(x..x + 1, 149..152, &|x, y| rgb(x, y)[k]));
                (0..3)
                    .map(|k| (m[k] - longe[k]).powi(2))
                    .sum::<f64>()
                    .sqrt()
            })
            .collect();
        let dc = d[64..77].iter().sum::<f64>() / 13.0;
        let (mut l, mut r) = (70, 70);
        while l > 0 && d[l - 1] > 0.5 * dc {
            l -= 1;
        }
        while r < 140 && d[r + 1] > 0.5 * dc {
            r += 1;
        }
        let papel = (c - 60..=c + 60)
            .filter(|&x| media(x..x + 1, 238..243, &esc) > 20.0)
            .count();
        redondo += (r - l + 1) as f64 / papel.max(1) as f64;
        escuro +=
            media(c - 6..c + 7, 145..156, &esc) / media(c - 6..c + 7, 230..251, &esc).max(1.0);
    }
    (redondo / 3.0, escuro / 3.0)
}

/// ⭐ **Molhado × molhado** (J5, `Knob::WetOpen`): o azul que atravessa a faixa vermelha AINDA MOLHADA abre nela uma mancha
/// REDONDA, mais larga que o traço, e empurra o vermelho para fora dela (o Rebelle `Liner` 1,42 × a largura, escuro 1,06);
/// sem a lei, o quadrado da largura do traço com as duas tintas somadas (1,00 · 1,27). CONTROLO: o mesmo sem a lei.
#[test]
fn o_azul_abre_uma_mancha_redonda_no_vermelho_molhado() {
    // O Wet Liner de fábrica liga a lei (`MANCHA_DO_LINER`, 0,42 · 0,5); o controlo desliga-a.
    let lei: [(String, f64); 0] = [];
    let sem = [("wetOpen".to_owned(), 0.0)];
    let [(com_lei, w), (controlo, _)] = std::thread::scope(|s| {
        let a = s.spawn(|| foto("Wet Liner", &lei, "sobre", 0));
        let b = s.spawn(|| foto("Wet Liner", &sem, "sobre", 0));
        [a.join().unwrap(), b.join().unwrap()]
    });
    let ((rl, el), (rc, ec)) = (mancha(&com_lei, w), mancha(&controlo, w));
    println!(
        "[molhado] Wet Liner: mancha {rl:.2} × escuro {el:.2} (Rebelle 1,42 · 1,06) · sem a lei {rc:.2} · {ec:.2}"
    );
    assert!(
        rc < 1.1 && ec > 1.18,
        "o controlo (sem a lei) já abre a mancha: {rc:.2} · {ec:.2}"
    );
    assert!(
        rl > 1.3,
        "o azul não abre a mancha no vermelho molhado: {rl:.2} × a largura (Rebelle 1,42)"
    );
    assert!(
        el < 1.15,
        "a mancha tem as duas tintas somadas: escuro {el:.2} (Rebelle 1,06)"
    );
}

/// O preset `nome` com o mapa `engine` sobreposto.
fn pincel(nome: &str, engine: &[(String, f64)]) -> Pincel {
    let p = pinceis(nome).remove(0);
    let Fonte::Preset { preset, relativo } = p.fonte else {
        unreachable!()
    };
    let mut pr = preset;
    for (k, v) in engine {
        pr.parametros.engine.insert(k.clone(), *v);
    }
    Pincel {
        nome: p.nome,
        fonte: Fonte::Preset {
            preset: pr,
            relativo,
        },
    }
}

/// A cena `largoazul` do oráculo da janela 9 (`docs/Painter/smoke_2026-10-24/oraculo_janela9/README.md`): uma faixa
/// vermelha MOLHADA de 84 px (dois traços de 60) e o azul de 40 que COMEÇA dentro dela e corre ao longo até passar o
/// fim; e um azul igual no papel (y 520), a régua da largura.
fn cena_largoazul() -> serde_json::Value {
    let traco = |nome: &str, size: u32, de: [f64; 2], ate: [f64; 2]| {
        let n = ((ate[0] - de[0]).abs() / 3.0).ceil() as usize;
        let pts: Vec<[f64; 3]> = (0..=n)
            .map(|k| {
                let a = k as f64 / n as f64;
                [
                    de[0] + (ate[0] - de[0]) * a,
                    de[1] + (ate[1] - de[1]) * a,
                    1.0,
                ]
            })
            .collect();
        serde_json::json!({"op": "traco", "nome": nome, "size_px": size, "passo": 3.0, "pts": pts})
    };
    serde_json::json!({"nome": "largoazul", "ops": [
        {"op": "pincel", "quem": "base"},
        {"op": "cor", "rgb": [191, 0, 18]},
        traco("faixa_cima", 60, [300.0, 305.0], [700.0, 305.0]),
        traco("faixa_baixo", 60, [300.0, 335.0], [700.0, 335.0]),
        {"op": "pincel", "quem": "preset"},
        {"op": "cor", "rgb": [23, 101, 179]},
        traco("dentro", 40, [433.0, 325.0], [850.0, 325.0]),
        traco("papel", 40, [300.0, 520.0], [850.0, 520.0]),
        {"op": "simula", "quadros": 240},
        {"op": "foto", "nome": "A"},
    ]})
}

/// A largura do azul na coluna `x`, nas linhas `ys` (a régua do oráculo: `B − R > −30` e fora do branco).
fn largura_azul(px: &[u8], w: usize, x: usize, ys: std::ops::Range<usize>) -> usize {
    ys.filter(|&y| {
        let i = (y * w + x) * 4;
        let (r, g, b) = (i32::from(px[i]), i32::from(px[i + 1]), i32::from(px[i + 2]));
        b - r > -30 && r + g + b < 3 * 235
    })
    .count()
}

/// ⭐ **O tubo** (oráculo da janela 9, smoke 25/10 dos retângulos brancos): o azul que COMEÇA dentro da faixa vermelha
/// molhada e corre ao longo dela alarga POR TODO O CAMINHO, de largura constante — o Rebelle `Liner` 1,47–1,53 × a
/// largura no papel, de x ≈ 435 a 700 —, e não num disco a meio da faixa com arestas retas. A régua: a largura em cada
/// coluna de x 450 a 690 ÷ a do mesmo azul no papel; o salto entre colunas vizinhas (o degrau do disco era uma aresta).
/// CONTROLO: sem a lei, a largura do papel.
#[test]
fn o_azul_que_corre_dentro_do_vermelho_molhado_alarga_como_um_tubo() {
    let mede = |engine: &[(String, f64)]| {
        let (px, w) = foto_da_cena(pincel("Wet Liner", engine), &cena_largoazul(), 0);
        let papel = (590..610)
            .map(|x| largura_azul(&px, w, x, 470..570))
            .sum::<usize>() as f64
            / 20.0;
        let ls: Vec<f64> = (450..=690)
            .map(|x| largura_azul(&px, w, x, 250..400) as f64)
            .collect();
        let media = ls.iter().sum::<f64>() / ls.len() as f64 / papel;
        let (lo, hi) = ls
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), &v| (a.min(v), b.max(v)));
        let salto = ls
            .windows(2)
            .map(|p| (p[1] - p[0]).abs())
            .fold(0.0, f64::max);
        (media, (hi - lo) / papel, salto, papel)
    };
    let [(media, faixa, salto, papel), (sem, ..)] = std::thread::scope(|s| {
        let a = s.spawn(|| mede(&[]));
        let b = s.spawn(|| mede(&[("wetOpen".to_owned(), 0.0)]));
        [a.join().unwrap(), b.join().unwrap()]
    });
    println!(
        "[tubo] Wet Liner: {media:.2} × o papel ({papel:.0} px), faixa {faixa:.2}, maior salto {salto:.0} px · sem a lei {sem:.2} (Rebelle 1,47–1,53)"
    );
    assert!(
        sem < 1.15,
        "CONTROLO: sem a lei o azul já alarga ({sem:.2})"
    );
    assert!(
        (1.3..1.7).contains(&media),
        "o azul dentro do vermelho molhado não alarga como o Rebelle: {media:.2} × (1,47–1,53)"
    );
    assert!(
        faixa < 0.25 && salto <= 4.0,
        "a largura não é constante pelo caminho: faixa {faixa:.2} × o papel, salto {salto:.0} px (o degrau do disco)"
    );
}

/// A BORDA da mancha do cruzamento molhado (cena `sobre`, foto A, y 145–155, os três cruzamentos e os dois lados):
/// `(degradê, aro)` — as células entre 80 % e 20 % da distância de cor do miolo à faixa longe, e a escuridão máxima na
/// banda da borda (8 células para dentro do meio-caminho, 2 para fora) ÷ a do miolo. O Rebelle `Liner` 12,2 · 1,10 (o
/// aro dele é o vermelho EMPURRADO, por fora), `Mop` 12,7 · 1,03, `Wash Round` 40,7 · 1,03.
pub(super) fn borda(px: &[u8], w: usize) -> (f64, f64) {
    let canal = |x: usize, k: usize| {
        (145..156)
            .map(|y| f64::from(px[(y * w + x) * 4 + k]))
            .sum::<f64>()
            / 11.0
    };
    let esc = |x: usize| 255.0 - (0.299 * canal(x, 0) + 0.587 * canal(x, 1) + 0.114 * canal(x, 2));
    let (mut deg, mut aro) = (0.0, 0.0);
    for c in [250usize, 500, 750] {
        let longe: [f64; 3] =
            [0, 1, 2].map(|k| (c + 100..c + 126).map(|x| canal(x, k)).sum::<f64>() / 26.0);
        let d = |x: usize| {
            (0..3)
                .map(|k| (canal(x, k) - longe[k]).powi(2))
                .sum::<f64>()
                .sqrt()
        };
        let dc = (c - 6..=c + 6).map(d).sum::<f64>() / 13.0;
        for lado in [-1i64, 1] {
            let em = |i: usize| (c as i64 + lado * i as i64) as usize;
            let ate = |f: f64| (0..=80).find(|&i| d(em(i)) < f * dc).unwrap_or(80);
            deg += (ate(0.2) - ate(0.8)) as f64;
            let i50 = ate(0.5);
            let miolo = (0..6).map(|i| esc(em(i))).sum::<f64>() / 6.0;
            let banda = (i50.saturating_sub(8)..i50 + 3)
                .map(|i| esc(em(i)))
                .fold(0.0, f64::max);
            aro += banda / miolo;
        }
    }
    (deg / 6.0, aro / 6.0)
}

/// ⭐ **A borda da mancha molhada ESBATE** (`Knob::WetOpenEdge`, J6): o Rebelle `Liner` esbate a mancha que o azul abre no
/// vermelho molhado em 12,2 células (80 → 20 %), sem aro por dentro; a lei de sempre corta-a em 2–3 células, com o aro
/// das duas tintas somadas (1,16). Com a pena de 20 células: o degradê dele e sem o aro. CONTROLO: o Wet Liner sem a pena.
#[test]
fn a_borda_da_mancha_molhada_esbate() {
    let [(com, w), (sem, _)] = std::thread::scope(|s| {
        let a = s.spawn(|| foto("Wet Liner", &[("wetOpenEdge".to_owned(), 20.0)], "sobre", 0));
        let b = s.spawn(|| foto("Wet Liner", &[("wetOpenEdge".to_owned(), 0.0)], "sobre", 0));
        [a.join().unwrap(), b.join().unwrap()]
    });
    let ((dl, al), (dc, ac)) = (borda(&com, w), borda(&sem, w));
    let (rl, _) = mancha(&com, w);
    println!(
        "[borda] Wet Liner: degradê {dl:.1} · aro {al:.3} · mancha {rl:.2} (Rebelle 12,2 · 1,10 · 1,42) · sem a pena {dc:.1} · {ac:.3}"
    );
    assert!(
        dc < 5.0 && ac > 1.10,
        "o controlo (a borda de sempre) já esbate: {dc:.1} · aro {ac:.3}"
    );
    assert!(
        (9.0..16.0).contains(&dl),
        "a borda da mancha não esbate como a do Rebelle: {dl:.1} células (12,2)"
    );
    assert!(al < 1.06, "o aro das duas tintas somadas ficou: {al:.3}");
    assert!(
        rl > 1.3,
        "a mancha deixou de abrir: {rl:.2} × a largura (1,42)"
    );
}

/// As RISCAS ao longo do traço na mancha (cena `sobre`, foto A, y 130–170): o desvio do vermelho de cada coluna à média
/// das 5 vizinhas, RMS nos três cruzamentos (x ±30). O Rebelle `Liner` 0,45.
fn riscas(px: &[u8], w: usize) -> f64 {
    let (mut t, mut n) = (0.0, 0.0);
    for c in [250usize, 500, 750] {
        let col: Vec<f64> = (c - 30..=c + 30)
            .map(|x| {
                (130..171)
                    .map(|y| f64::from(px[(y * w + x) * 4]))
                    .sum::<f64>()
                    / 41.0
            })
            .collect();
        for i in 2..col.len() - 2 {
            t += (col[i] - col[i - 2..=i + 2].iter().sum::<f64>() / 5.0).powi(2);
            n += 1.0;
        }
    }
    (t / n).sqrt()
}

/// ⭐ **A pena larga não risca** (`Knob::WetOpenEdge`, J6): os destinos da borda a ~4 células uns dos outros, na MESMA fase
/// em todas as células, pousavam nas mesmas colunas linha após linha — riscas ao longo do traço (o Wet Wash com a pena de
/// 60: 1,56). A fase sorteada por célula do grid desfá-las (0,84). CONTROLO: a mancha de borda dura de sempre (2,0+).
#[test]
fn a_pena_larga_nao_risca() {
    let (px, w) = foto("Wet Wash", &[("wetOpenEdge".to_owned(), 60.0)], "sobre", 0);
    let r = riscas(&px, w);
    println!("[riscas] Wet Wash com a pena de 60: {r:.2} (Rebelle Liner 0,45)");
    assert!(
        r < 1.2,
        "a pena larga risca a mancha ao longo do traço: {r:.2}"
    );
}

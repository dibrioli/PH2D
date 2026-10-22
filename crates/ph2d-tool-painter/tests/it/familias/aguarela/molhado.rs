//! **Molhado × molhado** (J5, a FILA 1; `docs/Painter/Pinceis/exames/aguarela.md` «Motor J5»): a sonda da cena `sobre`.

use crate::exame_bancada::{Fonte, Pincel, bateria, pinceis, pinta_cena};

/// A cena `cena` (foto `foto`) pintada por `nome` (preset da biblioteca ou caminho) com o mapa `engine` sobreposto.
pub(super) fn foto(
    nome: &str,
    engine: &[(String, f64)],
    cena: &str,
    foto: usize,
) -> (Vec<u8>, usize) {
    let p = pinceis(nome).remove(0);
    let Fonte::Preset { preset, relativo } = p.fonte else {
        unreachable!()
    };
    let mut pr = preset;
    for (k, v) in engine {
        pr.parametros.engine.insert(k.clone(), *v);
    }
    let p = Pincel {
        nome: p.nome,
        fonte: Fonte::Preset {
            preset: pr,
            relativo,
        },
    };
    let bat = bateria();
    let tela = [0, 1].map(|i| bat["tela"][i].as_u64().unwrap() as u32);
    let papel = [0, 1, 2].map(|i| bat["papel"][i].as_u64().unwrap() as u8);
    let c = bat["cenas"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["nome"] == cena)
        .unwrap()
        .clone();
    (
        pinta_cena(&p, None, &c, tela, papel, foto).1,
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
    let tarefas: Vec<(usize, &str, usize, Vec<(String, f64)>)> = nomes
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

/// A MANCHA do cruzamento molhado (cena `sobre`, foto A; o `blot.py` do relatório), a média dos três verticais:
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

//! Os gates do que o gesto do Solid CERCA (`solid_cercado`), pela porta única `fill_coverage`.

use crate::solid::fill_coverage;

const LADO: usize = 160;

/// O rabisco da ferramenta (`solido_papel_tests`): duas voltas, um só laço. Os vazios dele tocam o
/// lado de fora só nos CRUZAMENTOS — por isso a régua abaixo tapa o caminho.
fn rabisco() -> Vec<[f32; 2]> {
    (0..=120)
        .map(|k| {
            #[allow(clippy::cast_precision_loss)]
            let s = k as f32 / 120.0 * std::f32::consts::TAU * 2.0;
            [80.0 + 55.0 * (s * 1.5).sin(), 80.0 + 50.0 * s.cos()]
        })
        .collect()
}

/// Um círculo de raio `r` em `c`, no sentido `sentido` (`±1`).
fn circulo(c: [f32; 2], r: f32, sentido: f32) -> Vec<[f32; 2]> {
    (0..48)
        .map(|k| {
            #[allow(clippy::cast_precision_loss)]
            let a = sentido * k as f32 / 48.0 * std::f32::consts::TAU;
            [c[0] + r * a.cos(), c[1] + r * a.sin()]
        })
        .collect()
}

fn area(l: &[[f32; 2]]) -> f32 {
    (0..l.len())
        .map(|i| {
            let (a, b) = (l[i], l[(i + 1) % l.len()]);
            a[0] * b[1] - b[0] * a[1]
        })
        .sum()
}

/// Os píxeis vazios (`0`) que não se ligam à borda por vazios (4-vizinhança), com o CAMINHO dos
/// laços a tapar (o recorte exacto de cada segmento — outro método que o percurso da lei): sem
/// isso a régua vaza pelos cruzamentos do rabisco, onde a cobertura dos dois sentidos se anula.
fn cercados(cov: &[u8], w: usize, h: usize, lacos: &[Vec<[f32; 2]>]) -> Vec<usize> {
    let (fora, vazio) = fora_de(cov, w, h, lacos);
    (0..w * h).filter(|&i| vazio[i] && !fora[i]).collect()
}

/// O vazio (`0`, fora do caminho) e o que dele se liga à borda — a régua de [`cercados`].
fn fora_de(cov: &[u8], w: usize, h: usize, lacos: &[Vec<[f32; 2]>]) -> (Vec<bool>, Vec<bool>) {
    // O caminho: todo píxel cujo quadrado FECHADO o segmento toca — o recorte de Liang–Barsky de
    // cada segmento contra cada píxel da caixa dele (exacto nos raspões de canto).
    let toca = |a: [f32; 2], b: [f32; 2], x: f32, y: f32| {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let (mut t0, mut t1) = (0.0f32, 1.0f32);
        for (p, q) in [
            (-dx, a[0] - x),
            (dx, x + 1.0 - a[0]),
            (-dy, a[1] - y),
            (dy, y + 1.0 - a[1]),
        ] {
            if p == 0.0 {
                if q < 0.0 {
                    return false;
                }
            } else {
                let r = q / p;
                if p < 0.0 {
                    t0 = t0.max(r);
                } else {
                    t1 = t1.min(r);
                }
            }
        }
        t0 <= t1
    };
    let mut caminho = vec![false; w * h];
    for l in lacos {
        for i in 0..l.len() {
            let (a, b) = (l[i], l[(i + 1) % l.len()]);
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let (x0, x1, y0, y1) = (
                a[0].min(b[0]).floor().max(0.0) as usize,
                (a[0].max(b[0]).floor() as usize).min(w - 1),
                a[1].min(b[1]).floor().max(0.0) as usize,
                (a[1].max(b[1]).floor() as usize).min(h - 1),
            );
            for y in y0..=y1 {
                for x in x0..=x1 {
                    #[allow(clippy::cast_precision_loss)]
                    if toca(a, b, x as f32, y as f32) {
                        caminho[y * w + x] = true;
                    }
                }
            }
        }
    }
    // Vazio: cobertura 0, fora do caminho e winding 0 no CENTRO (por um raio, outro método que as
    // linhas de varrimento da lei) — um píxel de anulação (dois sentidos no mesmo píxel) não é vazio.
    let winding = |i: usize| {
        #[allow(clippy::cast_precision_loss)]
        let (px, py) = ((i % w) as f32 + 0.5, (i / w) as f32 + 0.5);
        let mut n = 0i32;
        for l in lacos {
            for k in 0..l.len() {
                let (a, b) = (l[k], l[(k + 1) % l.len()]);
                if (a[1] <= py) != (b[1] <= py)
                    && a[0] + (b[0] - a[0]) * (py - a[1]) / (b[1] - a[1]) > px
                {
                    n += if b[1] > a[1] { 1 } else { -1 };
                }
            }
        }
        n
    };
    let vazio: Vec<bool> = (0..w * h)
        .map(|i| cov[i] == 0 && !caminho[i] && winding(i) == 0)
        .collect();
    let mut fora = vec![false; w * h];
    let mut pilha: Vec<usize> = (0..w * h)
        .filter(|&i| {
            let (x, y) = (i % w, i / w);
            (x == 0 || y == 0 || x == w - 1 || y == h - 1) && vazio[i]
        })
        .collect();
    pilha.iter().for_each(|&i| fora[i] = true);
    while let Some(i) = pilha.pop() {
        let (x, y) = (i % w, i / w);
        for k in [
            (x > 0).then(|| i - 1),
            (x + 1 < w).then(|| i + 1),
            (y > 0).then(|| i - w),
            (y + 1 < h).then(|| i + w),
        ]
        .into_iter()
        .flatten()
        {
            if !fora[k] && vazio[k] {
                fora[k] = true;
                pilha.push(k);
            }
        }
    }
    (fora, vazio)
}

/// A regra não-zero SEM o que o gesto cerca — o controlo (o `fill_coverage` de antes).
fn nao_zero(loops: &[Vec<[f32; 2]>]) -> Vec<u8> {
    crate::solid::fill_coverage_sem_cercado(loops, LADO, LADO, [0.0, 0.0])
}

/// ⭐ **O RABISCO QUE VOLTA PARA TRÁS NÃO DEIXA VAZIO CERCADO** — e o controlo: a regra não-zero
/// deixava (milhares de píxeis).
#[test]
fn o_rabisco_nao_deixa_vazio_cercado() {
    let l = vec![rabisco()];
    let antes = cercados(&nao_zero(&l), LADO, LADO, &l).len();
    assert!(
        antes > 1000,
        "controlo: a regra não-zero deixava buracos ({antes})"
    );
    let depois = cercados(&fill_coverage(&l, LADO, LADO, [0.0, 0.0]), LADO, LADO, &l).len();
    assert_eq!(
        depois, 0,
        "o rabisco deixou {depois} píxeis vazios cercados"
    );
}

/// ⭐ **O VAZIO QUE CADA LAÇO CERCA ENCHE TAMBÉM COM OUTROS LAÇOS NA LISTA** (as cópias da simetria):
/// o rabisco com um disco ao lado, e com a sua cópia espelhada (sentido preservado, como o
/// `symmetric_loops` a faz) a cruzá-lo — no par, todo píxel que UM dos laços cerca sozinho fica cheio.
///
/// ⚠️ **Aberto, medido:** no espelho sobram `129` píxeis vazios que só o PAR cerca — lóbulos de
/// sentidos contrários das duas cópias a anularem-se, e bolsos que nenhuma cópia fecha sozinha. Enchê-los
/// pede a UNIÃO das cópias de um gesto, que contraria o `Remove` das formas (que fura por soma):
/// decisão à parte, não pedida.
#[test]
fn o_vazio_de_cada_laco_enche_com_outros_lacos() {
    let r = rabisco();
    let espelho: Vec<[f32; 2]> = r.iter().rev().map(|p| [160.0 - p[0] + 6.0, p[1]]).collect();
    for lacos in [
        vec![r.clone(), circulo([150.0, 150.0], 6.0, 1.0)],
        vec![r.clone(), espelho],
    ] {
        let par = fill_coverage(&lacos, LADO, LADO, [0.0, 0.0]);
        let mut vistos = 0;
        for l in &lacos {
            let so = std::slice::from_ref(l);
            for i in cercados(&nao_zero(so), LADO, LADO, so) {
                vistos += 1;
                assert_eq!(
                    par[i],
                    255,
                    "({}, {}): o que um laço cerca sozinho ficou vazio no par",
                    i % LADO,
                    i / LADO
                );
            }
        }
        assert!(
            vistos > 1000,
            "controlo: os laços cercam vazio sozinhos ({vistos})"
        );
    }
}

/// ⭐ **«CERCADO» É DA CAIXA INTEIRA, NUNCA DA JANELA** — qualquer janela é, ao byte, o recorte da
/// cobertura da tela inteira (a aguada recompõe por janelas, BUGS #38).
#[test]
fn qualquer_janela_e_o_recorte_da_tela() {
    let l = vec![rabisco()];
    let tela = fill_coverage(&l, LADO, LADO, [0.0, 0.0]);
    for (x0, y0, w, h) in [
        (30, 30, 40, 40),
        (60, 50, 17, 33),
        (0, 0, 80, 160),
        (95, 70, 50, 9),
    ] {
        #[allow(clippy::cast_precision_loss)]
        let jan = fill_coverage(&l, w, h, [x0 as f32, y0 as f32]);
        for j in 0..h {
            for i in 0..w {
                assert_eq!(
                    jan[j * w + i],
                    tela[(y0 + j) * LADO + x0 + i],
                    "janela ({x0}, {y0}, {w}, {h}): ({i}, {j}) difere da tela"
                );
            }
        }
    }
}

/// ⭐ **UM LAÇO QUE NÃO CERCA VAZIO É O DE ANTES, AO BYTE** — um círculo, uma estrela que se cruza
/// (winding 2 no miolo, nada vazio) e dois laços com o furo de um `Remove`.
#[test]
fn sem_vazio_cercado_nada_muda() {
    let estrela: Vec<[f32; 2]> = (0..5)
        .map(|k| {
            #[allow(clippy::cast_precision_loss)]
            let a = k as f32 * 2.0 * std::f32::consts::TAU / 5.0;
            [80.0 + 60.0 * a.cos(), 80.0 + 60.0 * a.sin()]
        })
        .collect();
    let disco = circulo([80.0, 80.0], 50.0, 1.0);
    let casos = [
        vec![disco.clone()],
        vec![estrela],
        vec![disco, circulo([80.0, 80.0], 20.0, -1.0)],
    ];
    for l in casos {
        assert_eq!(fill_coverage(&l, LADO, LADO, [0.0, 0.0]), nao_zero(&l));
    }
}

/// SONDA — o rabisco pela regra não-zero: quantos píxeis cercados por cobertura baixa (`< k`).
#[test]
#[ignore = "diagnóstico"]
fn diag_o_rabisco_na_regra_nao_zero() {
    let cov = nao_zero(&[rabisco()]);
    for k in [1u8, 2, 4, 8, 32] {
        let baixa: Vec<u8> = cov.iter().map(|&c| if c < k { 0 } else { 255 }).collect();
        eprintln!(
            "cobertura < {k}: {} cercados",
            cercados(&baixa, LADO, LADO, &[rabisco()]).len()
        );
    }
    let r = rabisco();
    eprintln!("pontos {} · área com sinal {}", r.len(), area(&r));
}

/// SONDA — o preço por evento de um laço de ~1000 px que cresce (o rabisco escalado ×4, um ponto a
/// cada 2 px), antes (não-zero) e depois (com o que cerca), no mesmo processo, 3 rodadas intercaladas:
/// a caixa inteira uma vez por evento (Digital) e três janelas de 256² por evento (a aguada).
/// `cargo test -p ph2d-painter-brush --profile smoke --lib diag_o_preco_do_cercado -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico"]
fn diag_o_preco_do_cercado() {
    let mut pts: Vec<[f32; 2]> = Vec::new();
    for k in 0..=2400 {
        #[allow(clippy::cast_precision_loss)]
        let s = k as f32 / 2400.0 * std::f32::consts::TAU * 2.0;
        let p = [500.0 + 420.0 * (s * 1.5).sin(), 500.0 + 380.0 * s.cos()];
        if pts
            .last()
            .is_none_or(|q: &[f32; 2]| (p[0] - q[0]).hypot(p[1] - q[1]) >= 2.0)
        {
            pts.push(p);
        }
    }
    if std::env::var("CERCADO_CIRCULO").is_ok() {
        // O caso comum: um laço que NÃO cerca nada (um círculo de raio 400 a crescer).
        pts = (0..2000)
            .map(|k| {
                #[allow(clippy::cast_precision_loss)]
                let a = k as f32 / 2000.0 * std::f32::consts::TAU * 0.98;
                [500.0 + 400.0 * a.cos(), 500.0 + 400.0 * a.sin()]
            })
            .collect();
    }
    let eventos: Vec<usize> = (1..=40).map(|e| pts.len() * e / 40).collect();
    let corre = |com: bool, janelas: bool| -> Vec<f64> {
        eventos
            .iter()
            .map(|&n| {
                let l = vec![pts[..n].to_vec()];
                let t0 = std::time::Instant::now();
                if janelas {
                    for k in 0..3usize {
                        #[allow(clippy::cast_precision_loss)]
                        let o = [(200 + 150 * k) as f32, (300 + 100 * k) as f32];
                        let _ = if com {
                            fill_coverage(&l, 256, 256, o)
                        } else {
                            crate::solid::fill_coverage_sem_cercado(&l, 256, 256, o)
                        };
                    }
                } else if com {
                    let _ = fill_coverage(&l, 1000, 1000, [0.0, 0.0]);
                } else {
                    let _ = crate::solid::fill_coverage_sem_cercado(&l, 1000, 1000, [0.0, 0.0]);
                }
                t0.elapsed().as_secs_f64() * 1e3
            })
            .collect()
    };
    for (nome, janelas) in [
        ("caixa 1000² por evento", false),
        ("3 janelas de 256² por evento", true),
    ] {
        let (mut a, mut b) = (Vec::new(), Vec::new());
        for _ in 0..3 {
            let mut x = corre(false, janelas);
            x.sort_by(f64::total_cmp);
            a.push(x[x.len() / 2]);
            let mut y = corre(true, janelas);
            y.sort_by(f64::total_cmp);
            b.push(y[y.len() / 2]);
        }
        a.sort_by(f64::total_cmp);
        b.sort_by(f64::total_cmp);
        eprintln!(
            "{nome}: antes {:.3} ms (mín) · {:.3} (med) · depois {:.3} (mín) · {:.3} (med) · {} pontos · loadavg {}",
            a[0],
            a[1],
            b[0],
            b[1],
            pts.len(),
            std::fs::read_to_string("/proc/loadavg")
                .unwrap_or_default()
                .trim()
        );
    }
}

/// ⭐ **O BURACO ENCHE SEM COSTURA E O CONTORNO DE FORA NÃO MUDA** — a orla (a barreira encostada ao
/// vazio cercado e a nenhum de fora) enche: sem ela ficava um anel de píxeis meio-cobertos à volta do
/// buraco cheio, a costura que o papel escuro mostra. E todo píxel encostado ao vazio de FORA tem a
/// cobertura da regra não-zero, ao byte (o contorno do gesto é o de antes, também nos cruzamentos onde
/// o buraco tocava o lado de fora).
#[test]
fn o_buraco_enche_sem_costura_e_o_contorno_de_fora_nao_muda() {
    let l = vec![rabisco()];
    let (antes, depois) = (nao_zero(&l), fill_coverage(&l, LADO, LADO, [0.0, 0.0]));
    let buracos = cercados(&antes, LADO, LADO, &l);
    let ainda = cercados(&depois, LADO, LADO, &l);
    assert!(ainda.is_empty(), "controlo: os buracos encheram");
    // De fora, no resultado: os vazios ligados à borda, com o caminho a tapar (um cruzamento não liga).
    let (fora, _) = fora_de(&depois, LADO, LADO, &l);
    let vizinhos = |i: usize| {
        let (x, y) = ((i % LADO) as i64, (i / LADO) as i64);
        (-1i64..=1)
            .flat_map(move |dy| (-1i64..=1).map(move |dx| (x + dx, y + dy)))
            .filter(|&(vx, vy)| vx >= 0 && vy >= 0 && vx < LADO as i64 && vy < LADO as i64)
            .map(|(vx, vy)| vy as usize * LADO + vx as usize)
    };
    let e_buraco = |i: usize| buracos.binary_search(&i).is_ok();
    let (mut costura, mut contorno) = (0, 0);
    for i in 0..LADO * LADO {
        // Encostado ao lado de fora PELO LADO (4-vizinhos): no píxel do cruzamento por onde o buraco
        // tocava o lado de fora só em diagonal, a régua (caminho amostrado) e a lei (percurso exacto)
        // discordam sobre o vizinho diagonal — e encher esse píxel é o que fecha o buraco.
        let (x, y) = (i % LADO, i / LADO);
        let encosta_fora = [
            (x > 0).then(|| i - 1),
            (x + 1 < LADO).then(|| i + 1),
            (y > 0).then(|| i - LADO),
            (y + 1 < LADO).then(|| i + LADO),
        ]
        .into_iter()
        .flatten()
        .any(|k| fora[k]);
        if encosta_fora {
            contorno += 1;
            assert_eq!(
                depois[i],
                antes[i],
                "({}, {}): o contorno de fora mudou",
                i % LADO,
                i / LADO
            );
        } else if vizinhos(i).any(e_buraco) {
            costura += 1;
            assert_eq!(
                depois[i],
                255,
                "({}, {}): costura à volta do buraco",
                i % LADO,
                i / LADO
            );
        }
    }
    assert!(
        costura > 100 && contorno > 100,
        "controlo: {costura} · {contorno}"
    );
}

/// SONDA — a vizinhança 5×5 de um píxel do contorno: cobertura antes · depois · (F de fora, v vazio).
#[test]
#[ignore = "diagnóstico"]
fn diag_a_vizinhanca() {
    let l = vec![rabisco()];
    let (antes, depois) = (nao_zero(&l), fill_coverage(&l, LADO, LADO, [0.0, 0.0]));
    let (fora, vazio) = fora_de(&depois, LADO, LADO, &l);
    let (fora0, vazio0) = fora_de(&antes, LADO, LADO, &l);
    for y in 67..=71usize {
        let linha: Vec<String> = (54..=58usize)
            .map(|x| {
                let i = y * LADO + x;
                format!(
                    "{:>3}/{:>3}{}{}|{}{}",
                    antes[i],
                    depois[i],
                    if fora[i] { 'F' } else { '.' },
                    if vazio[i] { 'v' } else { '.' },
                    if fora0[i] { 'F' } else { '.' },
                    if vazio0[i] { 'v' } else { '.' }
                )
            })
            .collect();
        eprintln!("y{y}: {}", linha.join("  "));
    }
}

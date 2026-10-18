//! **A régua das mídias em Rust** — a `cobertura` e a `textura` de `ferramentas/mede_traco.py`, sobre o
//! recorte do recto da bancada (03 §1), para o portão `cada_preset_parece_a_sua_midia` correr sem o
//! Python. Conferida contra o Python nas mesmas imagens (`a_regua_rust_e_a_do_python`, ignorado: precisa
//! de `PH2D_SONDA_OUT` com os PPM e os números do Python ao lado).

/// O recorte do recto da bancada (`ferramentas/midias_regua.py::RECTO`): x, y, largura, altura.
pub const RECTO: [u32; 4] = [20, 60, 580, 120];

/// A cobertura do miolo e a textura (%) de um traço — as duas métricas de `mede_traco.py`.
#[derive(Clone, Copy, Debug)]
pub struct Medida {
    pub cobertura: f64,
    pub textura: f64,
    /// A cobertura na orla de 3 px ÷ a do miolo (< 1 = o grão abre na borda; > 1 = a orla escura).
    pub borda: f64,
    /// A mediana da transição 10 → 90 % do máximo da coluna, em cima e em baixo (px).
    pub orla: f64,
    /// A uniformidade ao longo do traço: o desvio ÷ a média da cobertura de cada coluna (> 0,08), fora dos
    /// 10 % de cada ponta (`uniformidade_cv`).
    pub cv: f64,
    /// A mediana da largura a meio do máximo de cada coluna (px, `largura_mediana_px`).
    pub largura: f64,
}

/// `mede_traco.mede(png, crop)`: `c = max|px − papel| / 255` (papel = mediana das 2 linhas de cima e
/// de baixo do recorte), máscara `c > 0,08`, miolo = máscara erodida 3× (vizinhança 4, com a volta do
/// `np.roll`), resíduo = `c − média 5×5` (bordo replicado); textura = desvio do resíduo no miolo ÷ a
/// cobertura média do miolo.
pub fn mede(rgba: &[u8], w: u32, recorte: [u32; 4]) -> Medida {
    let [x0, y0, cw, ch] = recorte;
    let (cw, ch) = (cw as usize, ch as usize);
    let px = |x: usize, y: usize| {
        let i = ((y0 as usize + y) * w as usize + x0 as usize + x) * 4;
        [rgba[i], rgba[i + 1], rgba[i + 2]].map(f64::from)
    };
    let mut papel = [0.0; 3];
    for (k, p) in papel.iter_mut().enumerate() {
        let mut v: Vec<f64> = [0, 1, ch - 2, ch - 1]
            .iter()
            .flat_map(|&y| (0..cw).map(move |x| (x, y)))
            .map(|(x, y)| px(x, y)[k])
            .collect();
        v.sort_by(f64::total_cmp);
        let n = v.len();
        *p = if n % 2 == 1 {
            v[n / 2]
        } else {
            0.5 * (v[n / 2 - 1] + v[n / 2])
        };
    }
    let c: Vec<f64> = (0..ch)
        .flat_map(|y| (0..cw).map(move |x| (x, y)))
        .map(|(x, y)| {
            let a = px(x, y);
            (0..3).map(|k| (a[k] - papel[k]).abs()).fold(0.0, f64::max) / 255.0
        })
        .collect();
    let at = |x: isize, y: isize| c[(y as usize) * cw + x as usize];
    let mut miolo: Vec<bool> = c.iter().map(|&v| v > 0.08).collect();
    for _ in 0..3 {
        let m = miolo.clone();
        let w_ = |x: isize, y: isize| {
            m[(y.rem_euclid(ch as isize) as usize) * cw + x.rem_euclid(cw as isize) as usize]
        };
        for y in 0..ch as isize {
            for x in 0..cw as isize {
                miolo[y as usize * cw + x as usize] =
                    w_(x, y) && w_(x, y - 1) && w_(x, y + 1) && w_(x - 1, y) && w_(x + 1, y);
            }
        }
    }
    let (mut soma, mut n) = (0.0f64, 0.0f64);
    let mut residuos = Vec::new();
    for y in 0..ch as isize {
        for x in 0..cw as isize {
            if !miolo[y as usize * cw + x as usize] {
                continue;
            }
            let mut m = 0.0;
            for dy in -2..=2 {
                for dx in -2..=2 {
                    m += at(
                        (x + dx).clamp(0, cw as isize - 1),
                        (y + dy).clamp(0, ch as isize - 1),
                    );
                }
            }
            let v = at(x, y);
            residuos.push(v - m / 25.0);
            soma += v;
            n += 1.0;
        }
    }
    let cobertura = soma / n.max(1.0);
    // A orla de 3 px: a máscara menos o miolo.
    let (mut so, mut no) = (0.0f64, 0.0f64);
    for (k, &v) in c.iter().enumerate() {
        if v > 0.08 && !miolo[k] {
            so += v;
            no += 1.0;
        }
    }
    let borda = (so / no.max(1.0)) / cobertura.max(1e-6);
    let mut orlas = Vec::new();
    // As colunas do traço (a primeira e a última com alguma célula > 0,08) e, nelas, o perfil e a largura.
    let tem = |x: usize| (0..ch).any(|y| c[y * cw + x] > 0.08);
    let (xa, xb) = ((0..cw).find(|&x| tem(x)), (0..cw).rfind(|&x| tem(x)));
    let mut perfil = Vec::new();
    let mut larguras = Vec::new();
    for x in xa.unwrap_or(0)..=xb.unwrap_or(0) {
        let col: Vec<f64> = (0..ch).map(|y| c[y * cw + x]).collect();
        let m = col.iter().copied().fold(0.0, f64::max);
        if m < 0.08 {
            perfil.push(0.0);
            continue;
        }
        let tinta: Vec<f64> = col.iter().copied().filter(|&v| v > 0.08).collect();
        perfil.push(tinta.iter().sum::<f64>() / tinta.len() as f64);
        let meio = |t: f64| {
            (
                col.iter().position(|&v| v >= t),
                col.iter().rposition(|&v| v >= t),
            )
        };
        if let (Some(a), Some(b)) = meio(0.5 * m) {
            larguras.push((b - a + 1) as f64);
        }
        let (lo, hi) = (0.1 * m, 0.9 * m);
        let primeiro = |t: f64| col.iter().position(|&v| v >= t).unwrap();
        let ultimo = |t: f64| col.iter().rposition(|&v| v >= t).unwrap();
        orlas.push((primeiro(hi) - primeiro(lo)) as f64);
        orlas.push((ultimo(lo) - ultimo(hi)) as f64);
    }
    orlas.sort_by(f64::total_cmp);
    let k = orlas.len();
    let orla = if k == 0 {
        0.0
    } else if k % 2 == 1 {
        orlas[k / 2]
    } else {
        0.5 * (orlas[k / 2 - 1] + orlas[k / 2])
    };
    let mediana = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        let k = v.len();
        match k {
            0 => 0.0,
            _ if k % 2 == 1 => v[k / 2],
            _ => 0.5 * (v[k / 2 - 1] + v[k / 2]),
        }
    };
    let np = perfil.len();
    let corpo = &perfil[(0.1 * np as f64) as usize..(0.9 * np as f64) as usize];
    let mc = corpo.iter().sum::<f64>() / corpo.len().max(1) as f64;
    let dc =
        (corpo.iter().map(|v| (v - mc).powi(2)).sum::<f64>() / corpo.len().max(1) as f64).sqrt();
    let media = residuos.iter().sum::<f64>() / n.max(1.0);
    let var = residuos.iter().map(|r| (r - media).powi(2)).sum::<f64>() / n.max(1.0);
    Medida {
        cobertura,
        textura: 100.0 * var.sqrt() / cobertura.max(1e-6),
        borda,
        orla,
        cv: dc / mc.max(1e-6),
        largura: mediana(&mut larguras),
    }
}

/// As pontas isoladas da bancada (`midias_regua.py::PONTAS`): o traço RÁPIDO e o vermelho recto — x0, y0, x1, y1.
pub const PONTAS: [[u32; 4]; 2] = [[575, 260, 990, 340], [650, 455, 990, 505]];

/// **O RECUO da ponta** (prompt 44, `midias_regua.py::ponta`): quanto a ponta a ±0,7 da meia-largura fica atrás
/// da do eixo, ÷ a meia-largura, na média das 4 pontas de [`PONTAS`] — redonda ≈ 0,29, reta ≈ 0. A máscara é
/// a cobertura acima de metade do p90 da tinta (> 0,08); a ponta de uma linha é a 1.ª/última coluna onde ≥ 2
/// das 3 linhas da faixa estão na máscara.
pub fn ponta(rgba: &[u8], w: u32) -> f64 {
    let mut recuos = Vec::new();
    for [x0, y0, x1, y1] in PONTAS {
        let (cw, ch) = ((x1 - x0) as usize, (y1 - y0) as usize);
        let px = |x: usize, y: usize| {
            let i = ((y0 as usize + y) * w as usize + x0 as usize + x) * 4;
            [rgba[i], rgba[i + 1], rgba[i + 2]].map(f64::from)
        };
        let mut papel = [0.0; 3];
        for (k, p) in papel.iter_mut().enumerate() {
            let mut v: Vec<f64> = [0, 1, ch - 2, ch - 1]
                .iter()
                .flat_map(|&y| (0..cw).map(move |x| px(x, y)[k]))
                .collect();
            v.sort_by(f64::total_cmp);
            let n = v.len();
            *p = if n % 2 == 1 {
                v[n / 2]
            } else {
                0.5 * (v[n / 2 - 1] + v[n / 2])
            };
        }
        let c: Vec<f64> = (0..ch)
            .flat_map(|y| (0..cw).map(move |x| (x, y)))
            .map(|(x, y)| {
                let a = px(x, y);
                (0..3).map(|k| (a[k] - papel[k]).abs()).fold(0.0, f64::max) / 255.0
            })
            .collect();
        // O p90 da tinta, interpolado como o `np.percentile`.
        let mut tinta: Vec<f64> = c.iter().copied().filter(|&v| v > 0.08).collect();
        tinta.sort_by(f64::total_cmp);
        let pos = 0.9 * (tinta.len() - 1) as f64;
        let (i, f) = (pos.floor() as usize, pos.fract());
        let p90 = tinta[i] + f * (tinta[(i + 1).min(tinta.len() - 1)] - tinta[i]);
        let mask: Vec<bool> = c.iter().map(|&v| v > 0.5 * p90).collect();
        let prof: Vec<usize> = (0..ch)
            .map(|y| (0..cw).filter(|&x| mask[y * cw + x]).count())
            .collect();
        let max = *prof.iter().max().unwrap();
        let linhas: Vec<usize> = (0..ch).filter(|&y| prof[y] * 2 > max).collect();
        let (topo, base) = (linhas[0] as f64, *linhas.last().unwrap() as f64);
        let (yc, hw) = ((topo + base) / 2.0, (base - topo + 1.0) / 2.0);
        let pontas = |dy: f64| {
            let lados: &[f64] = if dy == 0.0 { &[0.0] } else { &[-1.0, 1.0] };
            let (mut e, mut d) = (0.0, 0.0);
            for &sg in lados {
                let r = (yc + sg * dy).round() as usize;
                let dentro = |x: usize| (r - 1..=r + 1).filter(|&y| mask[y * cw + x]).count() >= 2;
                e += (0..cw).find(|&x| dentro(x)).unwrap() as f64;
                d += (0..cw).rfind(|&x| dentro(x)).unwrap() as f64;
            }
            (e / lados.len() as f64, d / lados.len() as f64)
        };
        let (eixo, fora) = (pontas(0.0), pontas(0.7 * hw));
        recuos.push((fora.0 - eixo.0) / hw);
        recuos.push((eixo.1 - fora.1) / hw);
    }
    recuos.iter().sum::<f64>() / recuos.len() as f64
}

/// ⭐ **A régua da ponta em Rust é a do Python** e distingue o redondo do recto: os dois traços de [`PONTAS`]
/// desenhados sem AA (raio 17, tampas de disco × rectas), os números do `midias_regua.py::ponta` sobre a MESMA
/// imagem (17/10): `0,228571…` e `0`.
#[test]
fn a_regua_da_ponta_e_a_do_python() {
    let sintetica = |redonda: bool| {
        let (w, h) = (1000usize, 640usize);
        let mut px = vec![255u8; w * h * 4];
        for (xa, xb, yc) in [(600i64, 960i64, 300i64), (700, 960, 480)] {
            let r = 17i64;
            for y in 0..h as i64 {
                for x in 0..w as i64 {
                    let corpo = x >= xa && x <= xb && (y - yc).abs() <= r;
                    let tampa = if redonda {
                        (x - xa).pow(2) + (y - yc).pow(2) <= r * r
                            || (x - xb).pow(2) + (y - yc).pow(2) <= r * r
                    } else {
                        x >= xa - r && x <= xb + r && (y - yc).abs() <= r
                    };
                    if corpo || tampa {
                        let i = (y as usize * w + x as usize) * 4;
                        px[i..i + 3].copy_from_slice(&[40, 60, 200]);
                    }
                }
            }
        }
        ponta(&px, w as u32)
    };
    let (redonda, recta) = (sintetica(true), sintetica(false));
    assert!(
        (redonda - 0.228_571_428_571_428_56).abs() < 1e-12,
        "redonda {redonda}"
    );
    assert!(recta.abs() < 1e-12, "recta {recta}");
}

/// **A RAMPA da pressão** (prompt 44): na linha 2 da bancada (x 60 → 560, pressão `0,15 + 0,85·sen(πt)`), a
/// largura a meio do máximo e a cobertura mediana (> 0,08) na pressão ~0,46 (x 120) ÷ as da pressão 1 (x 310),
/// medianas de 9 colunas em y 250..350. Papel = a mediana das 3 linhas de cima e de baixo da IMAGEM. Rebelle:
/// o Round estreita (0,44 / 0,59), o Flat esmaece (0,83 / 0,42).
pub fn rampa(rgba: &[u8], w: u32, h: u32) -> (f64, f64) {
    let (w, h) = (w as usize, h as usize);
    let px = |x: usize, y: usize| {
        let i = (y * w + x) * 4;
        [rgba[i], rgba[i + 1], rgba[i + 2]].map(f64::from)
    };
    let mut papel = [0.0; 3];
    for (k, p) in papel.iter_mut().enumerate() {
        let mut v: Vec<f64> = [0, 1, 2, h - 3, h - 2, h - 1]
            .iter()
            .flat_map(|&y| (0..w).map(move |x| px(x, y)[k]))
            .collect();
        v.sort_by(f64::total_cmp);
        let n = v.len();
        *p = 0.5 * (v[(n - 1) / 2] + v[n / 2]);
    }
    let c = |x: usize, y: usize| {
        let a = px(x, y);
        (0..3).map(|k| (a[k] - papel[k]).abs()).fold(0.0, f64::max) / 255.0
    };
    let mediana = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        let n = v.len();
        0.5 * (v[(n - 1) / 2] + v[n / 2])
    };
    let coluna = |xc: usize| {
        let (mut larg, mut cob) = (Vec::new(), Vec::new());
        for x in xc - 4..=xc + 4 {
            let col: Vec<f64> = (250..350).map(|y| c(x, y)).collect();
            let m = col.iter().copied().fold(0.0, f64::max);
            if m < 0.08 {
                larg.push(0.0);
                cob.push(0.0);
                continue;
            }
            let a = col.iter().position(|&v| v >= 0.5 * m).unwrap();
            let b = col.iter().rposition(|&v| v >= 0.5 * m).unwrap();
            larg.push((b - a + 1) as f64);
            cob.push(mediana(col.iter().copied().filter(|&v| v > 0.08).collect()));
        }
        (mediana(larg), mediana(cob))
    };
    let ((l0, c0), (l1, c1)) = (coluna(120), coluna(310));
    (l0 / l1.max(1e-6), c0 / c1.max(1e-6))
}

/// O recorte da RAMPA da bancada (linha 2 esquerda, `mede_todos.sh`): x, y, largura, altura.
pub const RAMPA: [u32; 4] = [20, 240, 560, 120];

/// ⭐ **As PONTAS da rampa** (W6, `ferramentas/pontas_rampa.py`): a largura a 5 % e a 95 % do comprimento ÷ a
/// largura a 50 %, na média das duas — o `largura_5_50_95` de `mede_traco.py` (papel = mediana das 2 linhas
/// de cima e de baixo do recorte, colunas da 1.ª à última com cobertura > 0,08, largura de cada uma a meio do
/// máximo dela, e `0` onde o máximo é < 0,08). Rebelle: óleo 0,81, tinta 0,68, lápis 0,56.
pub fn pontas(rgba: &[u8], w: u32) -> f64 {
    let [x0, y0, cw, ch] = RAMPA;
    let (cw, ch) = (cw as usize, ch as usize);
    let px = |x: usize, y: usize| {
        let i = ((y0 as usize + y) * w as usize + x0 as usize + x) * 4;
        [rgba[i], rgba[i + 1], rgba[i + 2]].map(f64::from)
    };
    let mut papel = [0.0; 3];
    for (k, p) in papel.iter_mut().enumerate() {
        let mut v: Vec<f64> = [0, 1, ch - 2, ch - 1]
            .iter()
            .flat_map(|&y| (0..cw).map(move |x| (x, y)))
            .map(|(x, y)| px(x, y)[k])
            .collect();
        v.sort_by(f64::total_cmp);
        let n = v.len();
        *p = 0.5 * (v[(n - 1) / 2] + v[n / 2]);
    }
    let c = |x: usize, y: usize| {
        let a = px(x, y);
        (0..3).map(|k| (a[k] - papel[k]).abs()).fold(0.0, f64::max) / 255.0
    };
    let tem = |x: usize| (0..ch).any(|y| c(x, y) > 0.08);
    let (Some(xa), Some(xb)) = ((0..cw).find(|&x| tem(x)), (0..cw).rfind(|&x| tem(x))) else {
        return 0.0;
    };
    let larg: Vec<f64> = (xa..=xb)
        .map(|x| {
            let col: Vec<f64> = (0..ch).map(|y| c(x, y)).collect();
            let m = col.iter().copied().fold(0.0, f64::max);
            if m < 0.08 {
                return 0.0;
            }
            let a = col.iter().position(|&v| v >= 0.5 * m).unwrap();
            let b = col.iter().rposition(|&v| v >= 0.5 * m).unwrap();
            (b - a + 1) as f64
        })
        .collect();
    let n = larg.len();
    let at = |f: f64| larg[(f * n as f64) as usize];
    (at(0.05) + at(0.95)) / (2.0 * at(0.5)).max(1e-6)
}

/// A régua das pontas é a do Python: uma rampa sintética (largura `8 + 32·sen(πt)`, x 60 → 560, sem AA) mede
/// o mesmo que `pontas_rampa.py` sobre a MESMA imagem (18/10): `15/41/15` ⇒ `0,365853…`.
#[test]
fn a_regua_das_pontas_e_a_do_python() {
    let (w, h) = (1000usize, 640usize);
    let mut px = vec![255u8; w * h * 4];
    for x in 60..=560usize {
        let t = (x - 60) as f64 / 500.0;
        let meia = ((8.0 + 32.0 * (t * std::f64::consts::PI).sin()) / 2.0).round() as usize;
        for y in 300 - meia..=300 + meia {
            px[(y * w + x) * 4..(y * w + x) * 4 + 3].copy_from_slice(&[40, 60, 200]);
        }
    }
    let p = pontas(&px, w as u32);
    assert!((p - PONTAS_DA_SINTETICA).abs() < 1e-9, "{p}");
}

/// O número do `pontas_rampa.py` sobre a rampa sintética de [`a_regua_das_pontas_e_a_do_python`].
const PONTAS_DA_SINTETICA: f64 = 0.365_853_658_536_585_36;

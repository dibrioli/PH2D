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

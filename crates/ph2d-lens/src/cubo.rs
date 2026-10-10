//! **A LUT `.cube`** — o leitor da spec pública *Adobe Cube LUT 1.0* e as duas amostragens 3D.
//!
//! As duas batem com o `lut3d` do ffmpeg ao último bit de 16 (`1,5e-5`, doc 129 §5.1). O look
//! amostra-se sobre os valores CODIFICADOS (sRGB): é o domínio em que um `.cube` é autorado.

/// Uma LUT 3D lida: `n³` entradas, o vermelho a variar mais depressa (a ordem da spec).
#[derive(Clone, Debug, PartialEq)]
pub struct Cubo {
    pub n: usize,
    pub dados: Vec<[f32; 3]>,
    pub dominio_min: [f32; 3],
    pub dominio_max: [f32; 3],
}

/// Porque um ficheiro foi recusado — a frase que o artista lê sai desta enumeração.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErroCubo {
    /// Só LUTs 3D (a `LUT_1D_SIZE` é outra ferramenta).
    So1d,
    SemTamanho,
    /// Fora de `2..=256` (o tecto da spec).
    TamanhoInvalido(usize),
    /// Uma linha que não é número nem palavra-chave conhecida.
    LinhaInvalida(usize),
    /// O número de entradas não é `n³`.
    Contagem {
        esperado: usize,
        lido: usize,
    },
}

/// O tecto da spec 1.0 para `LUT_3D_SIZE`.
pub const N_MAX: usize = 256;

impl Cubo {
    /// Lê o texto de um `.cube`.
    ///
    /// # Errors
    /// O [`ErroCubo`] que nomeia a primeira recusa.
    pub fn ler(texto: &str) -> Result<Self, ErroCubo> {
        let mut n = None;
        let mut dmin = [0.0; 3];
        let mut dmax = [1.0; 3];
        let mut dados = Vec::new();
        for (i, linha) in texto.lines().enumerate() {
            let l = linha.trim();
            if l.is_empty() || l.starts_with('#') || l.starts_with("TITLE") {
                continue;
            }
            let mut partes = l.split_whitespace();
            let chave = partes.next().unwrap_or_default();
            let tres = |p: std::str::SplitWhitespace<'_>| -> Option<[f32; 3]> {
                let v: Vec<f32> = p.map(str::parse).collect::<Result<_, _>>().ok()?;
                (v.len() == 3).then(|| [v[0], v[1], v[2]])
            };
            match chave {
                "LUT_1D_SIZE" => return Err(ErroCubo::So1d),
                "LUT_3D_SIZE" => {
                    let v: usize = partes
                        .next()
                        .and_then(|s| s.parse().ok())
                        .ok_or(ErroCubo::LinhaInvalida(i + 1))?;
                    if !(2..=N_MAX).contains(&v) {
                        return Err(ErroCubo::TamanhoInvalido(v));
                    }
                    n = Some(v);
                }
                "DOMAIN_MIN" => dmin = tres(partes).ok_or(ErroCubo::LinhaInvalida(i + 1))?,
                "DOMAIN_MAX" => dmax = tres(partes).ok_or(ErroCubo::LinhaInvalida(i + 1))?,
                _ => dados.push(tres(l.split_whitespace()).ok_or(ErroCubo::LinhaInvalida(i + 1))?),
            }
        }
        let n = n.ok_or(ErroCubo::SemTamanho)?;
        if dados.len() != n * n * n {
            return Err(ErroCubo::Contagem {
                esperado: n * n * n,
                lido: dados.len(),
            });
        }
        Ok(Self {
            n,
            dados,
            dominio_min: dmin,
            dominio_max: dmax,
        })
    }

    fn em(&self, r: usize, g: usize, b: usize) -> [f32; 3] {
        self.dados[r + self.n * (g + self.n * b)]
    }

    /// A posição na grelha `[0, n-1]` e a célula (o canto de baixo, `≤ n-2`).
    fn celula(&self, c: [f32; 3]) -> ([usize; 3], [f32; 3]) {
        let top = (self.n - 1) as f32;
        let mut i = [0; 3];
        let mut f = [0.0; 3];
        for k in 0..3 {
            let t = ((c[k] - self.dominio_min[k]) / (self.dominio_max[k] - self.dominio_min[k]))
                .clamp(0.0, 1.0)
                * top;
            let base = (t.floor() as usize).min(self.n - 2);
            i[k] = base;
            f[k] = t - base as f32;
        }
        (i, f)
    }

    /// Tetraédrica: a célula partida em seis tetraedros pela ordem das três frações.
    #[must_use]
    pub fn tetraedrica(&self, c: [f32; 3]) -> [f32; 3] {
        let ([i, j, k], [r, g, b]) = self.celula(c);
        let p = |a: usize, bb: usize, cc: usize| self.em(i + a, j + bb, k + cc);
        let (c000, c111) = (p(0, 0, 0), p(1, 1, 1));
        let (a, bv, w) = if r > g {
            if g > b {
                (p(1, 0, 0), p(1, 1, 0), [r, g, b])
            } else if r > b {
                (p(1, 0, 0), p(1, 0, 1), [r, b, g])
            } else {
                (p(0, 0, 1), p(1, 0, 1), [b, r, g])
            }
        } else if b > g {
            (p(0, 0, 1), p(0, 1, 1), [b, g, r])
        } else if b > r {
            (p(0, 1, 0), p(0, 1, 1), [g, b, r])
        } else {
            (p(0, 1, 0), p(1, 1, 0), [g, r, b])
        };
        let [x, y, z] = w;
        [0, 1, 2].map(|q| (1.0 - x) * c000[q] + (x - y) * a[q] + (y - z) * bv[q] + z * c111[q])
    }

    /// Trilinear: os oito cantos pesados pelas três frações.
    #[must_use]
    pub fn trilinear(&self, c: [f32; 3]) -> [f32; 3] {
        let ([i, j, k], [r, g, b]) = self.celula(c);
        let mut out = [0.0; 3];
        for (dr, wr) in [(0, 1.0 - r), (1, r)] {
            for (dg, wg) in [(0, 1.0 - g), (1, g)] {
                for (db, wb) in [(0, 1.0 - b), (1, b)] {
                    let v = self.em(i + dr, j + dg, k + db);
                    let w = wr * wg * wb;
                    for q in 0..3 {
                        out[q] += w * v[q];
                    }
                }
            }
        }
        out
    }
}

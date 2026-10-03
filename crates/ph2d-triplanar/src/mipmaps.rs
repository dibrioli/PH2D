//! ⭐ **A imagem com a cadeia de mips**, em bytes RGBA8 — os MESMOS bytes que sobem para a placa e
//! que a CPU lê. Linha 0 = a de BAIXO (`v = 0`), como o Blender e a placa a leem.

/// O sRGB de 8 bits em linear (a curva do padrão), por tabela.
#[must_use]
pub fn srgb_para_linear(b: u8) -> f32 {
    TABELA.with(|t| t[b as usize])
}

thread_local! {
    static TABELA: [f32; 256] = std::array::from_fn(|i| {
        let c = i as f64 / 255.0;
        (if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }) as f32
    });
}

fn linear_para_srgb8(l: f32) -> u8 {
    let l = f64::from(l.clamp(0.0, 1.0));
    let c = if l <= 0.003_130_8 {
        l * 12.92
    } else {
        1.055 * l.powf(1.0 / 2.4) - 0.055
    };
    (c * 255.0).round() as u8
}

/// ⭐ Uma imagem quadrada `lado × lado` (potência de dois) e os mips dela até `1 × 1`.
#[derive(Clone, Debug, PartialEq)]
pub struct Mipmaps {
    lado: u32,
    niveis: Vec<Vec<[u8; 4]>>,
}

impl Mipmaps {
    /// ⭐ A partir de pixels RGBA8 `w × h` com a origem em CIMA (o PNG/JPG), levados a `lado²`.
    /// `srgb` = a imagem é cor (reamostra e faz a média em linear); senão os bytes são números.
    ///
    /// # Panics
    /// Se `lado` não for potência de dois ou os pixels não forem `w × h`.
    #[must_use]
    pub fn de_rgba8(w: u32, h: u32, px: &[[u8; 4]], lado: u32, srgb: bool) -> Self {
        assert!(lado.is_power_of_two(), "o lado é potência de dois");
        assert_eq!(px.len(), (w * h) as usize, "pixels w × h");
        let base: Vec<[u8; 4]> = if w == lado && h == lado {
            // Sem reamostrar: os bytes do ficheiro, só virados.
            (0..lado)
                .flat_map(|j| {
                    let r = (lado - 1 - j) as usize * lado as usize;
                    px[r..r + lado as usize].iter().copied()
                })
                .collect()
        } else {
            let dec = |b: [u8; 4]| -> [f32; 4] {
                if srgb {
                    [
                        srgb_para_linear(b[0]),
                        srgb_para_linear(b[1]),
                        srgb_para_linear(b[2]),
                        f32::from(b[3]) / 255.0,
                    ]
                } else {
                    b.map(|c| f32::from(c) / 255.0)
                }
            };
            let lin: Vec<[f32; 4]> = px.iter().map(|&b| dec(b)).collect();
            let horiz = reamostra(&lin, w as usize, h as usize, lado as usize, true);
            let tudo = reamostra(&horiz, lado as usize, h as usize, lado as usize, false);
            (0..lado as usize)
                .flat_map(|j| {
                    let r = (lado as usize - 1 - j) * lado as usize;
                    tudo[r..r + lado as usize].to_vec()
                })
                .map(|c| codifica(c, srgb))
                .collect()
        };
        let mut niveis = vec![base];
        let mut l = lado;
        while l > 1 {
            let ant = niveis.last().expect("nível");
            let m = l / 2;
            let mut prox = Vec::with_capacity((m * m) as usize);
            for j in 0..m {
                for i in 0..m {
                    let q = |di: u32, dj: u32| ant[((2 * j + dj) * l + 2 * i + di) as usize];
                    let quatro = [q(0, 0), q(1, 0), q(0, 1), q(1, 1)];
                    let mut s = [0.0f32; 4];
                    for b in quatro {
                        for c in 0..4 {
                            s[c] += if srgb && c < 3 {
                                srgb_para_linear(b[c])
                            } else {
                                f32::from(b[c]) / 255.0
                            };
                        }
                    }
                    prox.push(codifica(s.map(|v| v * 0.25), srgb));
                }
            }
            niveis.push(prox);
            l = m;
        }
        Self { lado, niveis }
    }

    #[must_use]
    pub fn lado(&self) -> u32 {
        self.lado
    }

    #[must_use]
    pub fn contagem(&self) -> u32 {
        self.niveis.len() as u32
    }

    /// Os bytes do nível `k` (linha 0 = baixo), prontos a subir.
    #[must_use]
    pub fn nivel(&self, k: u32) -> &[[u8; 4]] {
        &self.niveis[k as usize]
    }

    /// ⭐ **A leitura trilinear da placa**, repetindo: `lod ≤ 0` lê o nível 0 bilinear; acima
    /// mistura os dois níveis vizinhos. `srgb` = decodifica cada texel ANTES de filtrar.
    #[must_use]
    pub fn amostra(&self, srgb: bool, u: f32, v: f32, lod: f32) -> [f32; 4] {
        let ler = |k: u32, i: u32, j: u32| -> [f32; 4] {
            let b = self.niveis[k as usize][(j * (self.lado >> k) + i) as usize];
            if srgb {
                [
                    srgb_para_linear(b[0]),
                    srgb_para_linear(b[1]),
                    srgb_para_linear(b[2]),
                    f32::from(b[3]) / 255.0,
                ]
            } else {
                b.map(|c| f32::from(c) / 255.0)
            }
        };
        let topo = (self.contagem() - 1) as f32;
        let lod = lod.clamp(0.0, topo);
        if lod <= 0.0 {
            return self.bilinear_com(0, u, v, &ler);
        }
        let k0 = lod.floor();
        let f = lod - k0;
        let a = self.bilinear_com(k0 as u32, u, v, &ler);
        if f <= 0.0 {
            return a;
        }
        let b = self.bilinear_com((k0 as u32 + 1).min(topo as u32), u, v, &ler);
        std::array::from_fn(|c| a[c] + (b[c] - a[c]) * f)
    }

    /// O bilinear de um nível com repetição; quem lê o texel é `ler(nível, i, j)`.
    pub(crate) fn bilinear_com(
        &self,
        k: u32,
        u: f32,
        v: f32,
        ler: &dyn Fn(u32, u32, u32) -> [f32; 4],
    ) -> [f32; 4] {
        let s = self.lado >> k;
        let sf = s as f32;
        let x = u * sf - 0.5;
        let y = v * sf - 0.5;
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let w = |a: f32| a.rem_euclid(sf) as u32 % s;
        let (i0, i1, j0, j1) = (w(x0), w(x0 + 1.0), w(y0), w(y0 + 1.0));
        let (a, b, c, d) = (ler(k, i0, j0), ler(k, i1, j0), ler(k, i0, j1), ler(k, i1, j1));
        std::array::from_fn(|q| {
            let baixo = a[q] + (b[q] - a[q]) * fx;
            let cima = c[q] + (d[q] - c[q]) * fx;
            baixo + (cima - baixo) * fy
        })
    }
}

fn codifica(c: [f32; 4], srgb: bool) -> [u8; 4] {
    let n = |x: f32| (x.clamp(0.0, 1.0) * 255.0).round() as u8;
    if srgb {
        [
            linear_para_srgb8(c[0]),
            linear_para_srgb8(c[1]),
            linear_para_srgb8(c[2]),
            n(c[3]),
        ]
    } else {
        c.map(n)
    }
}

/// Reamostra numa direcção (`horizontal` = as colunas) de `n` para `m` amostras: média de área a
/// encolher, bilinear a esticar.
fn reamostra(px: &[[f32; 4]], w: usize, h: usize, m: usize, horizontal: bool) -> Vec<[f32; 4]> {
    let n = if horizontal { w } else { h };
    let (ow, oh) = if horizontal { (m, h) } else { (w, m) };
    let escala = n as f64 / m as f64;
    // Os pesos de cada saída, uma vez.
    let pesos: Vec<Vec<(usize, f32)>> = (0..m)
        .map(|o| {
            if escala > 1.0 {
                let (a, b) = (o as f64 * escala, (o + 1) as f64 * escala);
                let mut v = Vec::new();
                let mut i = a.floor() as usize;
                while (i as f64) < b && i < n {
                    let cobre = (b.min(i as f64 + 1.0) - a.max(i as f64)) / escala;
                    v.push((i, cobre as f32));
                    i += 1;
                }
                v
            } else {
                let x = ((o as f64 + 0.5) * escala - 0.5).clamp(0.0, (n - 1) as f64);
                let i0 = x.floor() as usize;
                let f = (x - i0 as f64) as f32;
                vec![(i0, 1.0 - f), ((i0 + 1).min(n - 1), f)]
            }
        })
        .collect();
    let mut out = vec![[0.0f32; 4]; ow * oh];
    for j in 0..oh {
        for i in 0..ow {
            let (o, fixo) = if horizontal { (i, j) } else { (j, i) };
            let mut s = [0.0f32; 4];
            for &(k, p) in &pesos[o] {
                let src = if horizontal {
                    px[fixo * w + k]
                } else {
                    px[k * w + fixo]
                };
                for c in 0..4 {
                    s[c] += src[c] * p;
                }
            }
            out[j * ow + i] = s;
        }
    }
    out
}

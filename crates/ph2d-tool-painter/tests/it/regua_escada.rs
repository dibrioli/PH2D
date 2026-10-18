//! **A régua da ESCADA** (W3b, prompt 50; doc 50_medicao §1) — `ferramentas/escada.py::mede` em Rust: num traço
//! de fios, quanto a LUMINÂNCIA salta ao longo dos fios a 1 px (o degrau que corta o fio num traço inclinado), na
//! direcção local dos fios (o tensor de estrutura da cobertura), no miolo do traço. Conferida contra o Python
//! nas mesmas imagens (`a_regua_da_escada_e_a_do_python`, ignorado: precisa das imagens em `PH2D_SONDA_OUT`).

/// O que a régua lê numa região (`0..255` de luminância).
#[derive(Clone, Copy, Debug)]
pub struct Escada {
    /// rms(ΔY ao longo dos fios, 1 px): a ESCADA.
    pub escada: f64,
    /// rms(ΔY através dos fios, 1 px).
    pub atraves: f64,
    /// rms(Y − G₄Y): a textura que se vê (a cobertura, `regua_midia::mede`, satura na tinta escura).
    pub contraste: f64,
    pub n: usize,
}

/// A bancada: o recto da linha 1 (o CONTROLO horizontal) e a curva em S. Os inclinados: o meio de cada recto.
pub const RECTO: [usize; 4] = [110, 40, 510, 220];
pub const CURVA: [usize; 4] = [620, 20, 940, 230];
pub const OITO: [usize; 4] = [250, 100, 750, 330];
pub const VINTE_E_CINCO: [usize; 4] = [180, 300, 480, 560];

/// Um plano `w × h` de `f64`.
struct Plano {
    w: usize,
    h: usize,
    v: Vec<f64>,
}

impl Plano {
    fn at(&self, x: usize, y: usize) -> f64 {
        self.v[y * self.w + x]
    }

    /// O gaussiano separável de `escada.py::gauss` (raio `⌊3σ + 1⌋`, bordo replicado).
    fn gauss(&self, s: f64) -> Plano {
        let r = (3.0 * s + 1.0) as isize;
        let mut k: Vec<f64> = (-r..=r)
            .map(|i| (-0.5 * (i as f64 / s).powi(2)).exp())
            .collect();
        let t: f64 = k.iter().sum();
        k.iter_mut().for_each(|x| *x /= t);
        let (w, h) = (self.w as isize, self.h as isize);
        let passa = |src: &[f64], horizontal: bool| -> Vec<f64> {
            let mut out = vec![0.0; src.len()];
            for y in 0..h {
                for x in 0..w {
                    let mut a = 0.0;
                    for (j, kj) in k.iter().enumerate() {
                        let d = j as isize - r;
                        let (xx, yy) = if horizontal {
                            ((x + d).clamp(0, w - 1), y)
                        } else {
                            (x, (y + d).clamp(0, h - 1))
                        };
                        a += kj * src[(yy * w + xx) as usize];
                    }
                    out[(y * w + x) as usize] = a;
                }
            }
            out
        };
        let v = passa(&passa(&self.v, false), true);
        Plano {
            w: self.w,
            h: self.h,
            v,
        }
    }

    fn bilinear(&self, x: f64, y: f64) -> f64 {
        let x = x.clamp(0.0, self.w as f64 - 1.001);
        let y = y.clamp(0.0, self.h as f64 - 1.001);
        let (x0, y0) = (x.floor() as usize, y.floor() as usize);
        let (fx, fy) = (x - x0 as f64, y - y0 as f64);
        self.at(x0, y0) * (1.0 - fx) * (1.0 - fy)
            + self.at(x0 + 1, y0) * fx * (1.0 - fy)
            + self.at(x0, y0 + 1) * (1.0 - fx) * fy
            + self.at(x0 + 1, y0 + 1) * fx * fy
    }
}

/// A régua na região `[x0, y0, x1, y1)` de uma imagem RGBA8 `w × h` (papel = a mediana das 2 linhas de cima e de
/// baixo da imagem, como `escada.py::cobertura`).
pub fn mede(rgba: &[u8], w: usize, h: usize, regiao: [usize; 4]) -> Escada {
    let px = |i: usize, k: usize| f64::from(rgba[i * 4 + k]);
    let mut papel = [0.0; 3];
    for (k, p) in papel.iter_mut().enumerate() {
        let mut v: Vec<f64> = [0, 1, h - 2, h - 1]
            .iter()
            .flat_map(|&y| (0..w).map(move |x| y * w + x))
            .map(|i| px(i, k))
            .collect();
        v.sort_by(f64::total_cmp);
        let n = v.len();
        *p = if n % 2 == 1 {
            v[n / 2]
        } else {
            0.5 * (v[n / 2 - 1] + v[n / 2])
        };
    }
    let c = Plano {
        w,
        h,
        v: (0..w * h)
            .map(|i| {
                (0..3)
                    .map(|k| (px(i, k) - papel[k]).abs())
                    .fold(0.0, f64::max)
                    / 255.0
            })
            .collect(),
    };
    let lum = Plano {
        w,
        h,
        v: (0..w * h)
            .map(|i| 0.2126 * px(i, 0) + 0.7152 * px(i, 1) + 0.0722 * px(i, 2))
            .collect(),
    };
    // O miolo: `c > 0,08` erodido 6× (vizinhança 4, com a volta do `np.roll`), dentro da região.
    let mut miolo: Vec<bool> = c.v.iter().map(|&v| v > 0.08).collect();
    for _ in 0..6 {
        let m = miolo.clone();
        let q = |x: isize, y: isize| {
            m[y.rem_euclid(h as isize) as usize * w + x.rem_euclid(w as isize) as usize]
        };
        for y in 0..h as isize {
            for x in 0..w as isize {
                miolo[y as usize * w + x as usize] =
                    q(x, y) && q(x, y - 1) && q(x, y + 1) && q(x - 1, y) && q(x + 1, y);
            }
        }
    }
    let [x0, y0, x1, y1] = regiao;
    // O tensor de estrutura: o gradiente de `c` a σ 1,5, os produtos a σ 6; o ângulo é o da NORMAL dos fios.
    let g = c.gauss(1.5);
    let mut gx = Plano {
        w,
        h,
        v: vec![0.0; w * h],
    };
    let mut gy = Plano {
        w,
        h,
        v: vec![0.0; w * h],
    };
    for y in 0..h {
        for x in 0..w {
            if x > 0 && x + 1 < w {
                gx.v[y * w + x] = 0.5 * (g.at(x + 1, y) - g.at(x - 1, y));
            }
            if y > 0 && y + 1 < h {
                gy.v[y * w + x] = 0.5 * (g.at(x, y + 1) - g.at(x, y - 1));
            }
        }
    }
    let prod = |a: &Plano, b: &Plano| Plano {
        w,
        h,
        v: a.v.iter().zip(&b.v).map(|(p, q)| p * q).collect(),
    };
    let (jxx, jxy, jyy) = (
        prod(&gx, &gx).gauss(6.0),
        prod(&gx, &gy).gauss(6.0),
        prod(&gy, &gy).gauss(6.0),
    );
    let g4 = lum.gauss(4.0);
    let (mut st, mut sn, mut sr, mut n) = (0.0, 0.0, 0.0, 0usize);
    for y in y0..y1.min(h) {
        for x in x0..x1.min(w) {
            if !miolo[y * w + x] {
                continue;
            }
            let a = 0.5 * (2.0 * jxy.at(x, y)).atan2(jxx.at(x, y) - jyy.at(x, y));
            let (nx, ny) = (a.cos(), a.sin());
            let (tx, ty) = (-ny, nx);
            let (xf, yf) = (x as f64, y as f64);
            let d = |ux: f64, uy: f64| {
                lum.bilinear(xf + 0.5 * ux, yf + 0.5 * uy)
                    - lum.bilinear(xf - 0.5 * ux, yf - 0.5 * uy)
            };
            st += d(tx, ty).powi(2);
            sn += d(nx, ny).powi(2);
            sr += (lum.at(x, y) - g4.at(x, y)).powi(2);
            n += 1;
        }
    }
    let k = n.max(1) as f64;
    Escada {
        escada: (st / k).sqrt(),
        atraves: (sn / k).sqrt(),
        contraste: (sr / k).sqrt(),
        n,
    }
}

/// ⭐ **A régua MORDE a escada** (o controlo dela): o MESMO padrão de fios a 8° — faixas de 3 px que alternam claro
/// e escuro —, desenhado com o degrau por píxel (cada píxel toma o fio do seu centro) e anti-aliased (a média de
/// 8×8 amostras por píxel). O degrau lê a escada 3,0× o liso (7,8 · 2,6; o liso não é zero: a bilinear de
/// faixas de 3 px); através, os dois são parecidos (24,4 · 22,4).
#[test]
fn a_regua_da_escada_morde_o_degrau() {
    let (w, h) = (300usize, 200usize);
    let ang = 8f64.to_radians();
    let (nx, ny) = (-ang.sin(), ang.cos());
    let fio = |x: f64, y: f64| -> f64 {
        let u = (x * nx + y * ny) / 3.0;
        if (u.floor() as i64).rem_euclid(2) == 0 {
            60.0
        } else {
            0.0
        }
    };
    let pinta = |aa: bool| -> Vec<u8> {
        let mut px = vec![255u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                if !(40..160).contains(&y) {
                    continue;
                }
                let v = if aa {
                    let mut s = 0.0;
                    for j in 0..8 {
                        for i in 0..8 {
                            s += fio(
                                x as f64 + (i as f64 + 0.5) / 8.0,
                                y as f64 + (j as f64 + 0.5) / 8.0,
                            );
                        }
                    }
                    s / 64.0
                } else {
                    fio(x as f64 + 0.5, y as f64 + 0.5)
                };
                let o = (y * w + x) * 4;
                px[o..o + 3].copy_from_slice(&[
                    (23.0 + v) as u8,
                    (101.0 + v) as u8,
                    (179.0 + v) as u8,
                ]);
            }
        }
        px
    };
    let r = [40, 40, 260, 160];
    let (degrau, liso) = (mede(&pinta(false), w, h, r), mede(&pinta(true), w, h, r));
    assert!(degrau.n > 5000 && liso.n == degrau.n, "{degrau:?} {liso:?}");
    assert!(
        degrau.escada >= 2.5 * liso.escada,
        "degrau {degrau:?} · liso {liso:?}"
    );
    assert!(
        (0.6..1.6).contains(&(degrau.atraves / liso.atraves)),
        "degrau {degrau:?} · liso {liso:?}"
    );
}

/// **A régua Rust é a do Python** (ignorado): sobre os PPM de `sonda_a_escada` em `PH2D_SONDA_OUT`, imprime os
/// números para os pôr ao lado de `ferramentas/escada.py --tabela` (doc 50_medicao §1: iguais a 0,01).
#[test]
#[ignore = "conferência: precisa dos PPM em PH2D_SONDA_OUT e do Python ao lado"]
fn a_regua_da_escada_e_a_do_python() {
    let dir = std::env::var("PH2D_SONDA_OUT").expect("PH2D_SONDA_OUT");
    let mut fs: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "ppm"))
        .collect();
    fs.sort();
    for f in fs {
        let b = std::fs::read(&f).unwrap();
        let mut partes = b.splitn(4, |&c| c == b'\n');
        let _ = partes.next();
        let dims = String::from_utf8_lossy(partes.next().unwrap()).to_string();
        let _ = partes.next();
        let corpo = partes.next().unwrap();
        let mut d = dims.split_whitespace().map(|v| v.parse::<usize>().unwrap());
        let (w, h) = (d.next().unwrap(), d.next().unwrap());
        let rgba: Vec<u8> = corpo[..w * h * 3]
            .chunks(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect();
        let nome = f.file_stem().unwrap().to_string_lossy().to_string();
        let regs: &[(&str, [usize; 4])] = if nome.contains("_incl_") {
            &[("8°", OITO), ("25°", VINTE_E_CINCO)]
        } else {
            &[("recto", RECTO), ("curva", CURVA)]
        };
        for (rn, r) in regs {
            let m = mede(&rgba, w, h, *r);
            println!(
                "[escada-rs] {nome:<34} {rn:<6} escada {:5.2} · através {:5.2} · contraste {:5.2} · n {}",
                m.escada, m.atraves, m.contraste, m.n
            );
        }
    }
}

//! ⭐⭐⭐ **A TEXTURA é a lei da casa** — a placa contra a CPU ([`ph2d_triplanar::avalia`] +
//! `Surface::at_base_color` / `at_roughness` + a normal), sob o céu fotográfico.
//!
//! Quatro quadrados PLANOS inclinados (posição, normal e derivadas exactas: uma esfera facetada
//! mudaria o texel lido), um regime cada: minificação com três vistas, ampliação num metal, verniz
//! com duas vistas, e a matriz mundo → folha girada, escalada e deslocada.

use crate::tests::{ID, ambiente, camera, cena, ceu_de, srgb};
use crate::{Forward, Foto, Instancia, Malha, TexturaMaterial};
use ph2d_triplanar::{Mapas, Mipmaps, Triplanar};

type V3 = [f32; 3];

fn png(b: &[u8]) -> ph2d_triplanar::Imagem {
    use ph2d_imageio::ImageImporter;
    let ph2d_imageio::DecodedImage::Flat(img) = ph2d_imageio_png::PngImporter
        .import(b, &ph2d_imageio::ImportOpts::default())
        .expect("png")
    else {
        panic!("png plano");
    };
    (img.width, img.height, img.pixels.iter().map(|p| p.0).collect())
}

/// A textura de teste COLORIDA, a normal de teste e uma rugosidade (o verde da colorida).
fn mapas() -> Mapas {
    let cor = png(include_bytes!("../../ph2d-triplanar/fixtures/teste_colorida.png"));
    let nor = png(include_bytes!("../../ph2d-triplanar/fixtures/teste_normal.png"));
    let rug = (cor.0, cor.1, cor.2.iter().map(|c| [c[1]; 4]).collect());
    Mapas {
        cor: Mipmaps::de_rgba8(cor.0, cor.1, &cor.2, cor.0, true),
        nrh: ph2d_triplanar::junta_nrh(&nor, Some(&rug), nor.0),
        tem_normal: true,
        tem_rugosidade: true,
    }
}

fn normaliza(v: V3) -> V3 {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    v.map(|c| c / l)
}

fn linha(m: &[[f32; 4]; 3], v: V3) -> V3 {
    std::array::from_fn(|i| m[i][0] * v[0] + m[i][1] * v[1] + m[i][2] * v[2])
}

fn coluna(m: &[[f32; 4]; 3], v: V3) -> V3 {
    std::array::from_fn(|i| m[0][i] * v[0] + m[1][i] * v[1] + m[2][i] * v[2])
}

struct Quadrado {
    x: (f32, f32),
    y: (f32, f32),
    n: V3,
    material: ph2d_material::OpenPbr,
    tex: TexturaMaterial,
}

impl Quadrado {
    fn z(&self, x: f32, y: f32) -> f32 {
        let c = ((self.x.0 + self.x.1) * 0.5, (self.y.0 + self.y.1) * 0.5);
        -(self.n[0] * (x - c.0) + self.n[1] * (y - c.1)) / self.n[2]
    }
}

fn girada() -> [[f32; 4]; 3] {
    // 1,3 × a rotação de 0,6 rad em torno de (1, 1, 0)/√2, mais uma translação.
    let (s, c) = 0.6f32.sin_cos();
    let a = std::f32::consts::FRAC_1_SQRT_2;
    let (x, y) = (a, a);
    let r = [
        [c + x * x * (1.0 - c), x * y * (1.0 - c), y * s],
        [y * x * (1.0 - c), c + y * y * (1.0 - c), -x * s],
        [-y * s, x * s, c],
    ];
    let t = [0.2, -0.1, 0.4];
    std::array::from_fn(|i| [r[i][0] * 1.3, r[i][1] * 1.3, r[i][2] * 1.3, t[i]])
}

fn quadrados() -> Vec<Quadrado> {
    let id = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
    ];
    let tex = |tamanho, blend, relevo, m| TexturaMaterial {
        camada: 0,
        triplanar: Triplanar {
            tamanho,
            aspecto: 1.0,
            blend,
            relevo,
        },
        tem_normal: true,
        tem_rugosidade: true,
        mundo_para_folha: m,
    };
    let base = ph2d_material::OpenPbr::default();
    vec![
        // Três vistas, minificação (≈ 1,8 níveis).
        Quadrado {
            x: (-0.95, -0.05),
            y: (0.05, 0.95),
            n: normaliza([0.6, 0.5, 0.62]),
            material: ph2d_material::OpenPbr {
                base_color: [1.0; 3],
                specular_roughness: 0.4,
                ..base
            },
            tex: tex(0.37, 0.5, 1.0, id),
        },
        // Ampliação num metal, uma vista quase sempre.
        Quadrado {
            x: (0.05, 0.95),
            y: (0.05, 0.95),
            n: normaliza([0.3, 0.4, 0.87]),
            material: ph2d_material::OpenPbr {
                base_color: [0.95, 0.75, 0.35],
                base_metalness: 1.0,
                ..base
            },
            tex: tex(3.0, 0.2, 0.6, id),
        },
        // Verniz (sem escurecer: a fronteira declarada do `mx_at_base_color`), duas vistas.
        Quadrado {
            x: (-0.95, -0.05),
            y: (-0.95, -0.05),
            n: normaliza([-0.7, 0.2, 0.68]),
            material: ph2d_material::OpenPbr {
                base_color: [0.8, 0.2, 0.15],
                coat_weight: 0.6,
                coat_roughness: 0.3,
                coat_darkening: 0.0,
                ..base
            },
            tex: tex(0.37, 1.0, 1.0, id),
        },
        // A folha girada, escalada e deslocada.
        Quadrado {
            x: (0.05, 0.95),
            y: (-0.95, -0.05),
            n: normaliza([0.2, -0.6, 0.77]),
            material: ph2d_material::OpenPbr {
                base_color: [0.3, 0.5, 0.95],
                specular_roughness: 0.2,
                ..base
            },
            tex: tex(0.5, 0.5, 1.0, girada()),
        },
    ]
}

/// ⭐⭐⭐ **A textura é a lei da casa, na placa.** Medido (03/10): p50 `0`, p99 `1`, máx `1–4` B. **Mutações que sangram:**
/// a rugosidade do mapa ignorada, a normal não devolvida ao mundo, a matriz da folha ignorada, o
/// nível do mip a zero.
#[test]
#[ignore = "precisa de aparelho"]
fn a_textura_e_a_lei_da_casa() {
    let Some(mut fw) = Forward::no_aparelho(&ambiente()) else {
        eprintln!("sem aparelho");
        return;
    };
    let qs = quadrados();
    let (mut p, mut n, mut mat, mut idx) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for (k, q) in qs.iter().enumerate() {
        let o = p.len() as u32;
        for (x, y) in [(q.x.0, q.y.0), (q.x.1, q.y.0), (q.x.1, q.y.1), (q.x.0, q.y.1)] {
            p.push([x, y, q.z(x, y)]);
            n.push(q.n);
            mat.push(k as u32);
        }
        idx.extend_from_slice(&[o, o + 1, o + 2, o, o + 2, o + 3]);
    }
    let ao = vec![1.0; p.len()];
    fw.sobe(
        1,
        &Malha {
            posicoes: &p,
            normais: &n,
            ao: &ao,
            material: &mat,
            indices: &idx,
        },
    );
    let m = mapas();
    fw.sobe_textura(0, &m);
    let ceu = ceu_de(ph2d_sky::Embarcado::Por);
    fw.sobe_ceu(ceu);
    let (th, forca) = (1.1f32, 0.6f32);
    let foto = Foto {
        giro: [th.cos(), th.sin()],
        forca,
        caixa: 1.0,
        fundo: None,
    };
    let env = ph2d_sky::Orientado {
        ceu,
        giro: foto.giro,
        forca,
        sol: 1.0,
    };
    let surf: Vec<ph2d_material::Surface> = qs.iter().map(|q| q.material.prepare()).collect();
    let mats: Vec<[f32; 48]> = surf
        .iter()
        .map(|s| ph2d_material::wgsl::pack(s, ph2d_material::wgsl::EnvLobe::of(s)))
        .collect();
    let texs: Vec<Option<TexturaMaterial>> = qs.iter().map(|q| Some(q.tex)).collect();
    let objs = [Instancia {
        malha: 1,
        modelo: ID,
    }];
    let img = fw
        .quadro(&crate::Cena {
            foto: Some(foto),
            texturas: &texs,
            ..cena(&objs, &mats, camera(1.0, 0.0))
        })
        .expect("quadro");
    let sem = fw
        .quadro(&crate::Cena {
            foto: Some(foto),
            ..cena(&objs, &mats, camera(1.0, 0.0))
        })
        .expect("quadro sem textura");
    let h = 2.0 / 96.0;
    let vista = ph2d_view_transform::ViewTransform::Standard;
    let mut dif: Vec<Vec<f32>> = vec![Vec::new(); qs.len()];
    let mut mexe = vec![0usize; qs.len()];
    for yy in 0..96 {
        for xx in 0..96 {
            let wx = (xx as f32 + 0.5) / 96.0 * 2.0 - 1.0;
            let wy = 1.0 - (yy as f32 + 0.5) / 96.0 * 2.0;
            let Some(k) = qs.iter().position(|q| {
                wx > q.x.0 + 2.0 * h && wx < q.x.1 - 2.0 * h && wy > q.y.0 + 2.0 * h && wy < q.y.1 - 2.0 * h
            }) else {
                continue;
            };
            let q = &qs[k];
            let pm = [wx, wy, q.z(wx, wy)];
            let dx = [h, 0.0, -h * q.n[0] / q.n[2]];
            let dy = [0.0, -h, h * q.n[1] / q.n[2]];
            let mf = &q.tex.mundo_para_folha;
            let lp = linha(mf, pm);
            let pf = [lp[0] + mf[0][3], lp[1] + mf[1][3], lp[2] + mf[2][3]];
            let r = ph2d_triplanar::avalia(
                &m,
                &q.tex.triplanar,
                pf,
                normaliza(linha(mf, q.n)),
                linha(mf, dx),
                linha(mf, dy),
            );
            let b = q.material.base_color;
            let s = surf[k]
                .at_base_color([b[0] * r.cor[0], b[1] * r.cor[1], b[2] * r.cor[2]])
                .at_roughness(r.rugosidade);
            let nn = normaliza(coluna(mf, r.normal));
            let c = s.indirect(nn, [0.0, 0.0, 1.0], &env);
            let d = ph2d_view_transform::to_display(c, 0.0, vista);
            let i = (yy * 96 + xx) * 4;
            let pior = (0..3)
                .map(|t| (f32::from(img[i + t]) - (srgb(d[t]) * 255.0 + 0.5).floor()).abs())
                .fold(0.0, f32::max);
            dif[k].push(pior);
            if (0..3).any(|t| img[i + t].abs_diff(sem[i + t]) > 8) {
                mexe[k] += 1;
            }
        }
    }
    for (k, d) in dif.iter_mut().enumerate() {
        d.sort_by(f32::total_cmp);
        let q = |p: f32| d[((d.len() - 1) as f32 * p) as usize];
        eprintln!(
            "quadrado {k}: {} px · p50 {} p99 {} max {} · a textura mexe {} px",
            d.len(),
            q(0.5),
            q(0.99),
            d[d.len() - 1],
            mexe[k]
        );
        assert!(d.len() > 300, "quadrado {k}: pixels de sobra");
        assert!(mexe[k] * 2 > d.len(), "o controlo: a textura muda o quadrado {k}");
        assert!(q(0.5) <= 1.0 && q(0.99) <= 1.0, "quadrado {k}: a placa diverge da CPU");
    }
}

/// ⭐⭐ **Trocar de textura não compila nada, e o quadro não depende do anterior** — os dois
/// inegociáveis da linha (`nada_compila_ao_editar`, `quadro_pronto_na_hora`) com a textura ligada:
/// subir outra camada (a matriz cresce e copia), trocar de camada e de números, e voltar.
#[test]
#[ignore = "precisa de aparelho"]
fn trocar_de_textura_nao_compila_nem_acumula() {
    let Some(mut fw) = Forward::no_aparelho(&ambiente()) else {
        eprintln!("sem aparelho");
        return;
    };
    let (p, n, idx) = crate::tests::esfera(0.5);
    let ao = vec![1.0; p.len()];
    let mat = vec![0u32; p.len()];
    fw.sobe(
        1,
        &Malha {
            posicoes: &p,
            normais: &n,
            ao: &ao,
            material: &mat,
            indices: &idx,
        },
    );
    let m = mapas();
    fw.sobe_textura(0, &m);
    let antes = fw.pipelines_compilados();
    let objs = [Instancia {
        malha: 1,
        modelo: ID,
    }];
    let s = ph2d_material::OpenPbr::default().prepare();
    let mats = [ph2d_material::wgsl::pack(&s, ph2d_material::wgsl::EnvLobe::of(&s))];
    let tex = |camada, tamanho| {
        [Some(TexturaMaterial {
            camada,
            triplanar: Triplanar {
                tamanho,
                aspecto: 1.0,
                blend: 0.3,
                relevo: 1.0,
            },
            tem_normal: true,
            tem_rugosidade: true,
            mundo_para_folha: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ],
        })]
    };
    let quadro = |fw: &mut Forward, t: &[Option<TexturaMaterial>]| {
        fw.quadro(&crate::Cena {
            texturas: t,
            ..cena(&objs, &mats, camera(1.0, 0.0))
        })
        .expect("quadro")
    };
    let a = quadro(&mut fw, &tex(0, 0.3));
    // Outra camada (a matriz cresce de 1 para 2 e copia a 0), outros números.
    let mut outra = m.clone();
    outra.tem_normal = false;
    fw.sobe_textura(1, &outra);
    let b = quadro(&mut fw, &tex(1, 0.7));
    let sem = quadro(&mut fw, &[]);
    let a2 = quadro(&mut fw, &tex(0, 0.3));
    assert_eq!(fw.pipelines_compilados(), antes, "trocar de textura compilou");
    assert_eq!(a, a2, "a camada 0 voltou diferente depois de a matriz crescer");
    assert_ne!(a, b, "o controlo: outra camada e outro ladrilho são outra imagem");
    assert_ne!(a, sem, "o controlo: a textura muda a imagem");
}

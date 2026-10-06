//! **A Strength é a opacidade do traço, uma vez só, em toda rota** (decisão do dono, 2026-10-06:
//! *«Strength = a opacidade exata»*). O `Dab::coverage` já nasce com `strength × pressão × overlap`
//! (`stroke/dab_build.rs`) e os núcleos multiplicavam `spec.strength` outra vez: o miolo de um traço a
//! Strength 0,4 tinha opacidade `0,163` (BUGS #41).

use super::measure_shape_system::{cp, tool};
use super::{CompositeLayer, CompositeOp};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool};

const LADO: u32 = 128;

/// Um traço reto de `a` a `b` em 12 eventos.
fn linha(t: &mut PainterTool, a: [f32; 2], b: [f32; 2]) {
    t.on_canvas_pointer(cp(a, PointerPhase::Down));
    for k in 1..=12 {
        #[allow(clippy::cast_precision_loss)]
        let f = k as f32 / 12.0;
        t.on_canvas_pointer(cp(
            [a[0] + (b[0] - a[0]) * f, a[1] + (b[1] - a[1]) * f],
            PointerPhase::Move,
        ));
    }
    t.on_canvas_pointer(cp(b, PointerPhase::Up));
}

/// Um toque: um dab só, no centro.
fn toque(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([64.0, 64.0], PointerPhase::Up));
}

/// A opacidade do texel mais opaco: tinta PRETA sobre a tela branca, `(255 − G) / 255`.
fn opacidade_maxima(t: &PainterTool) -> f32 {
    let g = t.canvas_rgba.chunks(4).map(|p| p[1]).min().expect("tela");
    f32::from(255 - g) / 255.0
}

fn altura_maxima(t: &PainterTool) -> f32 {
    t.layers
        .active()
        .and_then(|a| t.heights.get(&a))
        .map_or(0.0, |h| h.iter().copied().fold(0.0f32, f32::max))
}

/// FNV-1a da tela e do relevo — a impressão digital do que o gesto deixou.
fn impressao(t: &PainterTool) -> u64 {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut mistura = |b: u8| {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    };
    t.canvas_rgba.iter().copied().for_each(&mut mistura);
    if let Some(r) = t.layers.active().and_then(|a| t.heights.get(&a)) {
        r.iter()
            .flat_map(|v| v.to_bits().to_le_bytes())
            .for_each(&mut mistura);
    }
    h
}

/// Um pincel preto de fábrica no meio `meio`, raio 6, Strength `s`.
fn pincel(meio: PaintMedia, s: f32) -> PainterTool {
    let mut t = tool(LADO, meio, 6.0);
    t.set_brush_color_srgb8([0, 0, 0]);
    t.paint.brush.strength = s;
    t
}

/// O traço com Accumulate desligado (o tecto armado em `s < 1`): rota por píxel.
fn digital_por_pixel(s: f32) -> PainterTool {
    let mut t = pincel(PaintMedia::Digital, s);
    t.paint.brush.accumulate = false;
    linha(&mut t, [20.0, 64.0], [108.0, 64.0]);
    t
}

/// Um dab com Accumulate ligado (sem tecto, sem atenuação): a rota em banda / o laço por dab.
fn digital_dab(s: f32, f: &dyn Fn(&mut PainterTool)) -> PainterTool {
    let mut t = pincel(PaintMedia::Digital, s);
    t.paint.brush.accumulate = true;
    t.paint.brush.space_attenuation = false;
    f(&mut t);
    toque(&mut t);
    t
}

/// A pilha do Composite com uma camada Brush de Strength `s`.
fn composite(s: f32, acumula: bool) -> PainterTool {
    let mut t = pincel(PaintMedia::Digital, 1.0);
    t.paint.brush.accumulate = acumula;
    t.paint.brush.space_attenuation = false;
    t.paint.composite_enabled = true;
    t.paint.composite[0] = CompositeLayer {
        op: CompositeOp::Brush,
        strength: s,
        ..CompositeLayer::default()
    };
    for pos in 1..t.paint.composite.len() {
        t.paint.composite[pos].strength = 0.0;
    }
    if acumula {
        toque(&mut t);
    } else {
        linha(&mut t, [20.0, 64.0], [108.0, 64.0]);
    }
    t
}

fn meio_linha(meio: PaintMedia, s: f32) -> PainterTool {
    let mut t = pincel(meio, s);
    linha(&mut t, [20.0, 64.0], [108.0, 64.0]);
    t
}

/// O Sculpt (o verbo de fábrica) sobre um relevo que o Impasto pôs: `Σ|Δaltura|` do toque.
fn sculpt(s: f32) -> (PainterTool, f32) {
    let mut t = pincel(PaintMedia::Impasto, 1.0);
    linha(&mut t, [20.0, 64.0], [108.0, 64.0]);
    let relevo = |t: &PainterTool| {
        t.layers
            .active()
            .and_then(|a| t.heights.get(&a))
            .map(|h| h.to_vec())
            .unwrap_or_default()
    };
    let antes = relevo(&t);
    t.set_paint_tool_mode("sculpt");
    let mut b = t.paint.brush;
    b.strength = s;
    t.paint.brush = b;
    t.paint.brush_by_mode[super::PaintMode::Sculpt.slot()] = b;
    toque(&mut t);
    let depois = relevo(&t);
    let mudou = antes.iter().zip(&depois).map(|(a, b)| (a - b).abs()).sum();
    (t, mudou)
}

/// `Σ|tela − base|` — quanto o gesto mudou uma tela lisa de valor `base`, em qualquer cor.
fn soma_delta(t: &PainterTool, base: u8) -> f32 {
    #[allow(clippy::cast_precision_loss)]
    let d = t
        .canvas_rgba
        .chunks(4)
        .flat_map(|p| p[..3].iter())
        .map(|&v| u64::from(v.abs_diff(base)))
        .sum::<u64>() as f32;
    d
}

/// Um toque com Accumulate ligado, sobre a tela lisa `base`, depois de `arma` — as rotas que o
/// roteador escolhe pela Shape e pelo Grain (`stamp_route::stamp_dabs_inner`). ⚠️ A base é BRANCA
/// (tinta preta, a resolução inteira de 255 níveis) e PRETA só onde a rampa de fábrica pinta branco:
/// num cinzento `128` o arredondamento da borda fraca vicia a soma (razão `0,236` em Strength 0,2).
fn toque_armado(s: f32, base: u8, arma: fn(&mut PainterTool)) -> f32 {
    let mut t = pincel(PaintMedia::Digital, s);
    // OPACA: numa base transparente a tinta pinta a cor cheia com alfa = Strength, e o RGB não mede.
    let tela: Vec<u8> = (0..LADO * LADO)
        .flat_map(|_| [base, base, base, 255])
        .collect();
    t.set_source(tela, LADO, LADO);
    t.paint.brush.accumulate = true;
    t.paint.brush.space_attenuation = false;
    arma(&mut t);
    t.paint.brush.strength = s;
    toque(&mut t);
    soma_delta(&t, base)
}

/// A silhueta da Shape: um quadrado branco `n²` (a imagem e as duas camadas da cor por camada).
fn arma_shape(t: &mut PainterTool) {
    let n = 16u32;
    t.set_brush_shape_image(vec![255u8; (n * n) as usize], n, n);
}

fn arma_cor_por_camada(t: &mut PainterTool) {
    let n = 16u32;
    arma_shape(t);
    // UMA camada: com duas, cada uma pinta a sua Strength por cima da outra e o gesto lê
    // `1 − (1 − s)²` (`0,361` em Strength 0,2) — a lei de cada camada é a mesma, uma vez.
    t.set_brush_shape_layers(vec![(vec![255u8; (n * n) as usize], n, n)]);
    t.toggle_brush_shape_per_layer_color();
}

/// A cor por camada com a COR DA TEXTURA (o RGB capturado da camada, cinzento `128`): a rota
/// `stamp_dabs_cached_color_rgba`.
fn arma_cor_da_textura(t: &mut PainterTool) {
    let n = 16usize;
    arma_cor_por_camada(t);
    t.paint
        .shape_layers
        .set_layers_meta(vec![vec![128u8; n * n * 3]], vec![1.0], vec![0], vec![1]);
}

/// A borracha do Impasto sobre um relevo uniforme `0,6`: `Σ|Δaltura|` do toque.
fn borracha_do_relevo(s: f32) -> f32 {
    let mut t = pincel(PaintMedia::Impasto, s);
    let camada = t.layers.active().expect("camada");
    let n = (LADO * LADO) as usize;
    t.heights
        .insert(camada, std::sync::Arc::new(vec![0.6f32; n]));
    t.covers.insert(camada, std::sync::Arc::new(vec![255u8; n]));
    t.sync_relief_flags();
    t.paint.eraser = true;
    toque(&mut t);
    t.heights
        .get(&camada)
        .expect("relevo")
        .iter()
        .map(|h| (0.6 - h).abs())
        .sum()
}

/// O Clone de uma metade preta para a branca: `Σ|Δ|` na metade branca.
fn clone(s: f32) -> f32 {
    let mut img = vec![255u8; (LADO * LADO * 4) as usize];
    for (i, px) in img.chunks_mut(4).enumerate() {
        if i % LADO as usize <= 64 {
            px[..3].copy_from_slice(&[0, 0, 0]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(img, LADO, LADO);
    t.set_brush_size_px(6.0);
    t.set_paint_tool_mode("clone");
    t.paint.brush.strength = s;
    t.set_clone_source([32.0, 64.0]);
    toque(&mut t);
    #[allow(clippy::cast_precision_loss)]
    let d = t
        .canvas_rgba
        .chunks(4)
        .enumerate()
        .filter(|(i, _)| i % LADO as usize > 70)
        .map(|(_, p)| u64::from(255 - p[1]))
        .sum::<u64>() as f32;
    d
}

/// O Blur sobre uma aresta preta/branca: quanto o toque mudou a tela.
fn blur(s: f32) -> (PainterTool, u64) {
    let mut img = vec![255u8; (LADO * LADO * 4) as usize];
    for (i, px) in img.chunks_mut(4).enumerate() {
        if i % LADO as usize <= 64 {
            px[..3].copy_from_slice(&[0, 0, 0]);
        }
    }
    let mut t = PainterTool::default();
    t.set_source(img, LADO, LADO);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(6.0);
    let antes = t.canvas_rgba.to_vec();
    t.paint.paint_mode = super::PaintMode::Blur;
    t.paint.brush.strength = s;
    toque(&mut t);
    let mudou = antes
        .iter()
        .zip(t.canvas_rgba.iter())
        .map(|(a, b)| u64::from(a.abs_diff(*b)))
        .sum();
    (t, mudou)
}

/// SONDA — a opacidade (e a altura, onde há) contra a Strength em cada rota e em cada meio, com a
/// razão para Strength 1 e a impressão digital (que prova os meios que NÃO mudam).
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_a_strength_em_cada_rota -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico"]
fn diag_a_strength_em_cada_rota() {
    let ss = [0.2f32, 0.4, 0.7, 1.0];
    let tabela = |nome: &str, v: &dyn Fn(f32) -> (f32, u64)| {
        let medidas: Vec<(f32, u64)> = ss.iter().map(|&s| v(s)).collect();
        let um = medidas[3].0.max(1e-9);
        let linha: Vec<String> = ss
            .iter()
            .zip(&medidas)
            .map(|(s, (m, _))| format!("S{s}: {m:.3} ({:.3})", m / um))
            .collect();
        let hashes: Vec<String> = medidas.iter().map(|(_, h)| format!("{h:016x}")).collect();
        eprintln!("{nome:<44} {} · {}", linha.join(" · "), hashes.join(" "));
    };
    let op = |t: PainterTool| (opacidade_maxima(&t), impressao(&t));
    let alt = |t: PainterTool| (altura_maxima(&t), impressao(&t));
    tabela("Digital por píxel (Accumulate off, linha)", &|s| {
        op(digital_por_pixel(s))
    });
    tabela("Digital por dab (Accumulate on, toque)", &|s| {
        op(digital_dab(s, &|_| {}))
    });
    tabela("Digital Flow 0,5 (Accumulate on, toque)", &|s| {
        op(digital_dab(s, &|t| t.paint.brush.flow = 0.5))
    });
    tabela("Digital Grain (cache, toque)", &|s| {
        op(digital_dab(s, &|t| t.set_brush_texture_kind(26)))
    });
    tabela("Digital Grain + Color Ramp (toque)", &|s| {
        op(digital_dab(s, &|t| {
            t.set_brush_texture_kind(26);
            t.set_texture_ramp_enabled(true);
        }))
    });
    tabela("Composite Brush (Accumulate off, linha)", &|s| {
        op(composite(s, false))
    });
    tabela("Composite Brush (Accumulate on, toque)", &|s| {
        op(composite(s, true))
    });
    tabela("Impasto cor (linha)", &|s| {
        op(meio_linha(PaintMedia::Impasto, s))
    });
    tabela("Impasto altura (linha)", &|s| {
        alt(meio_linha(PaintMedia::Impasto, s))
    });
    tabela("Sculpt Σ|Δaltura| (toque sobre o traço)", &|s| {
        let (t, d) = sculpt(s);
        (d, impressao(&t))
    });
    tabela("Blur Σ|Δ| (toque na aresta)", &|s| {
        let (t, d) = blur(s);
        #[allow(clippy::cast_precision_loss)]
        let d = d as f32;
        (d, impressao(&t))
    });
    tabela("Aquarela cor (linha)", &|s| {
        op(meio_linha(PaintMedia::Watercolor, s))
    });
    tabela("Wet Paint cor (linha)", &|s| {
        op(meio_linha(PaintMedia::WetPaint, s))
    });
}

/// Os gestos dos portões: horizontal em píxel inteiro, diagonal, deslocado MEIO píxel e um laço que
/// FECHA no ponto de partida (a cobertura fraccionária e o fecho que as fixturas alinhadas escondem).
fn gestos(t: &mut PainterTool, k: usize) {
    match k {
        0 => linha(t, [20.0, 64.0], [108.0, 64.0]),
        1 => linha(t, [22.0, 26.0], [104.0, 101.0]),
        2 => linha(t, [20.5, 40.5], [108.5, 88.5]),
        4 => {
            // O laço LENTO: 10 eventos por aresta com um tique entre eles e o fecho a 4 px do
            // começo — os cantos e o fecho empilham dezenas de dabs no mesmo texel.
            let c = [
                [24.0f32, 24.0],
                [104.0, 24.0],
                [104.0, 104.0],
                [24.0, 104.0],
                [24.0, 28.0],
            ];
            t.on_canvas_pointer(cp(c[0], PointerPhase::Down));
            for w in c.windows(2) {
                for i in 1..=10 {
                    #[allow(clippy::cast_precision_loss)]
                    let f = i as f32 / 10.0;
                    t.on_canvas_pointer(cp(
                        [
                            w[0][0] + (w[1][0] - w[0][0]) * f,
                            w[0][1] + (w[1][1] - w[0][1]) * f,
                        ],
                        PointerPhase::Move,
                    ));
                    t.paint_tick(1.0 / 60.0);
                }
            }
            t.on_canvas_pointer(cp(c[4], PointerPhase::Up));
        }
        _ => {
            let c = [
                [30.0f32, 30.5],
                [98.5, 30.0],
                [98.0, 98.5],
                [30.5, 98.0],
                [30.0, 30.5],
            ];
            t.on_canvas_pointer(cp(c[0], PointerPhase::Down));
            for w in c.windows(2) {
                for i in 1..=8 {
                    #[allow(clippy::cast_precision_loss)]
                    let f = i as f32 / 8.0;
                    t.on_canvas_pointer(cp(
                        [
                            w[0][0] + (w[1][0] - w[0][0]) * f,
                            w[0][1] + (w[1][1] - w[0][1]) * f,
                        ],
                        PointerPhase::Move,
                    ));
                }
            }
            t.on_canvas_pointer(cp(c[4], PointerPhase::Up));
        }
    }
}

/// ⭐ **O MIOLO DE UM TRAÇO COM ACCUMULATE DESLIGADO TEM A OPACIDADE DA STRENGTH** (decisão do dono,
/// 2026-10-06: *«Strength = a opacidade exata»*) — ±1/255, em Strength 0,2 · 0,4 · 0,7 · 1, no Digital
/// e no Impasto, em quatro gestos. Vermelho antes: `0,039` · `0,161` · `0,490` (a Strength ao quadrado).
#[test]
fn o_miolo_do_traco_tem_a_opacidade_da_strength() {
    for meio in [PaintMedia::Digital, PaintMedia::Impasto] {
        for k in 0..4 {
            for s in [0.2f32, 0.4, 0.7, 1.0] {
                let mut t = pincel(meio, s);
                t.paint.brush.accumulate = false;
                gestos(&mut t, k);
                let g = t.canvas_rgba.chunks(4).map(|p| p[1]).min().expect("tela");
                let alvo = 255.0 * s;
                assert!(
                    (f32::from(255 - g) - alvo).abs() <= 1.0,
                    "{meio:?} gesto {k} Strength {s}: o miolo tem opacidade {}/255, a Strength pede {alvo}/255",
                    255 - g
                );
            }
        }
    }
}

/// ⭐ **A STRENGTH ENTRA UMA VEZ SÓ EM TODA ROTA** — a razão `medida(s) / medida(1)` é `s` no dab
/// do Accumulate ligado (com Flow, Grain e Color Ramp), na pilha do Composite, na altura do Impasto, no
/// Sculpt e no Blur. Vermelho antes: `s²` em todas menos o Composite (que trocava a Strength na hora
/// do carimbo e por isso a levava uma vez — o gate prende que continua a levar).
#[test]
fn a_strength_entra_uma_vez_em_toda_rota() {
    // O 3.º campo é o degrau de quantização da medida (`1/255` numa opacidade, `0` numa soma): a Color
    // Ramp lê `25/255` em Strength 1, e `0,7 × 25 = 17,5` arredonda para `18` (razão `0,720`).
    type Rota = (&'static str, fn(f32) -> f32, f32);
    const NIVEL: f32 = 1.0 / 255.0;
    let rotas: [Rota; 19] = [
        (
            "Grain View (cache)",
            |s| {
                toque_armado(s, 255, |t| {
                    t.set_brush_texture_kind(26);
                    t.set_brush_texture_mapping(0);
                })
            },
            0.0,
        ),
        (
            "Grain Tiled (cache da tela)",
            |s| {
                toque_armado(s, 255, |t| {
                    t.set_brush_texture_kind(26);
                    t.set_brush_texture_mapping(1);
                })
            },
            0.0,
        ),
        ("Shape", |s| toque_armado(s, 255, arma_shape), 0.0),
        (
            "Shape + Color Ramp",
            |s| {
                toque_armado(s, 0, |t| {
                    arma_shape(t);
                    t.set_shape_ramp_enabled(true);
                })
            },
            0.0,
        ),
        (
            "cor por camada",
            |s| toque_armado(s, 255, arma_cor_por_camada),
            0.0,
        ),
        (
            "cor por camada + Jitter Rotate",
            |s| {
                toque_armado(s, 255, |t| {
                    arma_cor_por_camada(t);
                    t.set_brush_jitter_rotate(1.0);
                })
            },
            0.0,
        ),
        (
            "Grain View + Color Ramp (cache)",
            |s| {
                toque_armado(s, 255, |t| {
                    t.set_brush_texture_kind(26);
                    t.set_brush_texture_mapping(0);
                    t.set_texture_ramp_enabled(true);
                })
            },
            0.0,
        ),
        (
            "cor da textura por camada",
            |s| toque_armado(s, 255, arma_cor_da_textura),
            0.0,
        ),
        ("borracha do relevo", borracha_do_relevo, 0.0),
        ("Clone", clone, 0.0),
        ("dab", |s| opacidade_maxima(&digital_dab(s, &|_| {})), NIVEL),
        (
            "dab Flow 0,5",
            |s| opacidade_maxima(&digital_dab(s, &|t| t.paint.brush.flow = 0.5)),
            NIVEL,
        ),
        (
            "dab Grain",
            |s| opacidade_maxima(&digital_dab(s, &|t| t.set_brush_texture_kind(26))),
            NIVEL,
        ),
        (
            "dab Grain + Color Ramp",
            |s| {
                opacidade_maxima(&digital_dab(s, &|t| {
                    t.set_brush_texture_kind(26);
                    t.set_texture_ramp_enabled(true);
                }))
            },
            NIVEL,
        ),
        (
            "Composite linha",
            |s| opacidade_maxima(&composite(s, false)),
            NIVEL,
        ),
        (
            "Composite dab",
            |s| opacidade_maxima(&composite(s, true)),
            NIVEL,
        ),
        (
            "Impasto altura",
            |s| altura_maxima(&meio_linha(PaintMedia::Impasto, s)),
            0.0,
        ),
        ("Sculpt", |s| sculpt(s).1, 0.0),
        (
            "Blur",
            |s| {
                #[allow(clippy::cast_precision_loss)]
                let d = blur(s).1 as f32;
                d
            },
            0.0,
        ),
    ];
    for (nome, mede, degrau) in rotas {
        let um = mede(1.0);
        assert!(um > 0.0, "controlo: {nome} mede alguma coisa em Strength 1");
        for s in [0.2f32, 0.4, 0.7] {
            let razao = mede(s) / um;
            eprintln!("{nome:<32} Strength {s}: razão {razao:.4}");
            assert!(
                (razao - s).abs() <= 0.02 + degrau / um,
                "{nome} Strength {s}: a medida é {razao:.3}× a de Strength 1 (a Strength pede {s})"
            );
        }
    }
}

/// SONDA — quanto o texel mais opaco passa da Strength (em níveis de 1/255, tinta preta), por gesto e
/// Strength de 0,1 a 1, no Digital.
#[test]
#[ignore = "diagnóstico"]
fn diag_o_miolo_contra_a_strength_por_gesto() {
    for k in 0..5 {
        let linha: Vec<String> = (1..=10)
            .map(|i| {
                #[allow(clippy::cast_precision_loss)]
                let s = i as f32 / 10.0;
                let mut t = pincel(PaintMedia::Digital, s);
                t.paint.brush.accumulate = false;
                gestos(&mut t, k);
                let g = t.canvas_rgba.chunks(4).map(|p| p[1]).min().expect("tela");
                format!("{s:.1}:{:+.1}", f32::from(255 - g) - 255.0 * s)
            })
            .collect();
        eprintln!("gesto {k}: {}", linha.join(" "));
    }
}

// ── O tecto da mancha do Solid depois da cura (o #40 regravado) ──────────────────────────────────

/// SONDA — o texel mais opaco do gesto com Solid contra o do traço sem Solid (G, tinta vermelha), na
/// Strength de 0,1 a 1: quantos níveis a mancha passa do tecto.
#[test]
#[ignore = "diagnóstico"]
fn diag_o_tecto_da_mancha_por_strength() {
    for (cor, rgb) in [("vermelho", [220u8, 40, 40]), ("preto", [0, 0, 0])] {
        for meio in [PaintMedia::Digital, PaintMedia::Impasto] {
            let linha: Vec<String> = (1..=10)
                .map(|k| {
                    #[allow(clippy::cast_precision_loss)]
                    let s = k as f32 / 10.0;
                    let com =
                        super::solido_meios_tests::laco_com(meio, 1.0, &|t: &mut PainterTool| {
                            t.paint.brush.strength = s;
                            t.set_brush_color_srgb8(rgb);
                        });
                    let sem =
                        super::solido_meios_tests::laco_com(meio, 1.0, &|t: &mut PainterTool| {
                            t.paint.brush.strength = s;
                            t.paint.brush.style_solid = false;
                            t.set_brush_color_srgb8(rgb);
                        });
                    format!(
                        "{s:.1}:{:+}",
                        i32::from(super::solido_meios_tests::mais_opaco(&sem))
                            - i32::from(super::solido_meios_tests::mais_opaco(&com))
                    )
                })
                .collect();
            eprintln!("{cor} {meio:?} níveis além do tecto: {}", linha.join(" "));
        }
    }
}

/// SONDA — onde a mancha passa do tecto (Digital, Strength 0,4): o texel mais opaco com Solid, o G
/// dele sem Solid, e a vizinhança.
#[test]
#[ignore = "diagnóstico"]
fn diag_onde_a_mancha_passa_do_tecto() {
    let com =
        super::solido_meios_tests::laco_com(PaintMedia::Digital, 1.0, &|t: &mut PainterTool| {
            t.paint.brush.strength = 0.4
        });
    let sem =
        super::solido_meios_tests::laco_com(PaintMedia::Digital, 1.0, &|t: &mut PainterTool| {
            t.paint.brush.strength = 0.4;
            t.paint.brush.style_solid = false;
        });
    let g = |t: &PainterTool, i: usize| t.canvas_rgba[i * 4 + 1];
    let (i, gc) = (0..128 * 128)
        .map(|i| (i, g(&com, i)))
        .min_by_key(|p| p.1)
        .expect("tela");
    let (j, gs) = (0..128 * 128)
        .map(|i| (i, g(&sem, i)))
        .min_by_key(|p| p.1)
        .expect("tela");
    let piores: Vec<(usize, usize, u8, u8)> = (0..128 * 128)
        .filter(|&k| g(&com, k) < gs)
        .map(|k| (k % 128, k / 128, g(&com, k), g(&sem, k)))
        .take(12)
        .collect();
    eprintln!(
        "com: ({}, {}) G {gc} · sem: ({}, {}) G {gs} · além do tecto (x, y, G com, G sem): {piores:?}",
        i % 128,
        i / 128,
        j % 128,
        j / 128
    );
}

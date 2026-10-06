//! **O fio claro dos traços pintados ANTES do papel** (decisão do dono, 2026-10-06: corrigir). Sobre o
//! branco a orla anti-aliased de um traço guarda a mistura com ele, e quando o papel passa a ter cor
//! ela aparecia como um fio claro de 1 px (BUGS #39). O critério é o da ordem: pintar e DEPOIS
//! escolher o papel dá a imagem de escolher o papel e DEPOIS pintar, ±1 nível — e a tinta opaca do
//! miolo continua opaca.

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use crate::tool::papel::{separa_o_branco, sobre_o_papel};
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase, Tool};

const LADO: usize = 128;
/// Um papel ESCURO: é onde o branco da orla mais aparece.
const PAPEL: [u8; 3] = [60, 90, 140];

/// Um traço: o pincel, a Strength, a dureza e o caminho.
#[derive(Clone, Copy)]
struct Traco {
    nome: &'static str,
    cor: [u8; 3],
    strength: f32,
    dureza: f32,
    raio: f32,
    de: [f32; 2],
    ate: [f32; 2],
}

const TRACOS: [Traco; 5] = [
    Traco {
        nome: "mole vermelho",
        cor: [220, 40, 40],
        strength: 1.0,
        dureza: 0.0,
        raio: 10.0,
        de: [20.0, 64.0],
        ate: [108.0, 64.0],
    },
    Traco {
        nome: "duro vermelho",
        cor: [220, 40, 40],
        strength: 1.0,
        dureza: 1.0,
        raio: 10.0,
        de: [20.0, 40.0],
        ate: [108.0, 40.0],
    },
    Traco {
        nome: "mole diagonal, meio píxel",
        cor: [30, 120, 60],
        strength: 1.0,
        dureza: 0.3,
        raio: 7.0,
        de: [20.5, 20.5],
        ate: [100.5, 107.5],
    },
    Traco {
        nome: "claro opaco largo",
        cor: [250, 210, 215],
        strength: 1.0,
        dureza: 1.0,
        raio: 24.0,
        de: [40.0, 70.0],
        ate: [88.0, 70.0],
    },
    Traco {
        nome: "Strength 0,5 (ambíguo)",
        cor: [220, 40, 40],
        strength: 0.5,
        dureza: 0.0,
        raio: 10.0,
        de: [20.0, 90.0],
        ate: [108.0, 90.0],
    },
];

fn pinta(t: &mut PainterTool, tr: &Traco) {
    t.set_brush_color_srgb8(tr.cor);
    t.paint.brush.strength = tr.strength;
    t.paint.brush.hardness = tr.dureza;
    t.set_brush_size_px(tr.raio);
    t.on_canvas_pointer(cp(tr.de, PointerPhase::Down));
    for k in 1..=16 {
        #[allow(clippy::cast_precision_loss)]
        let f = k as f32 / 16.0;
        t.on_canvas_pointer(cp(
            [
                tr.de[0] + (tr.ate[0] - tr.de[0]) * f,
                tr.de[1] + (tr.ate[1] - tr.de[1]) * f,
            ],
            PointerPhase::Move,
        ));
    }
    t.on_canvas_pointer(cp(tr.ate, PointerPhase::Up));
}

fn escolhe_o_papel(t: &mut PainterTool) {
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", PAPEL[0], PAPEL[1], PAPEL[2]),
    ));
    assert_eq!(t.papel(), Some(PAPEL), "controlo: o papel foi aplicado");
}

/// A imagem mostrada de uma camada única sobre o papel (o que o composite faz, sem relevo).
fn mostrada(camada: &[u8]) -> Vec<u8> {
    let mut v = camada.to_vec();
    for px in v.as_chunks_mut::<4>().0 {
        sobre_o_papel(px, PAPEL);
    }
    v
}

/// O papel escolhido ANTES do traço: a camada que o produto deixa.
fn papel_primeiro(tr: &Traco) -> Vec<u8> {
    let mut t = tool(LADO as u32, PaintMedia::Digital, tr.raio);
    escolhe_o_papel(&mut t);
    pinta(&mut t, tr);
    t.canvas_rgba.to_vec()
}

/// O traço pintado sobre o BRANCO, antes de qualquer papel.
fn traco_no_branco(tr: &Traco) -> Vec<u8> {
    let mut t = tool(LADO as u32, PaintMedia::Digital, tr.raio);
    pinta(&mut t, tr);
    t.canvas_rgba.to_vec()
}

/// A lei antiga: só o branco PURO sai.
fn so_o_branco_puro(rgba: &mut [u8]) {
    for px in rgba.as_chunks_mut::<4>().0 {
        if *px == [255, 255, 255, 255] {
            *px = [0, 0, 0, 0];
        }
    }
}

/// O «color to alpha» do GIMP contra o branco (`a = 1 − min(c)`, `c' = branco + (c − branco)/a`), só na
/// faixa AA: os píxeis a 1 px (8-vizinhança) do branco puro. O resto é a lei antiga.
fn gimp_na_faixa_aa(rgba: &mut [u8]) {
    let branco = |r: &[u8], i: usize| r[i * 4..i * 4 + 4] == [255, 255, 255, 255];
    let original = rgba.to_vec();
    for i in 0..LADO * LADO {
        if branco(&original, i) || original[i * 4 + 3] != 255 {
            continue;
        }
        let (x, y) = ((i % LADO) as i64, (i / LADO) as i64);
        let encosta = (-1i64..=1).any(|dy| {
            (-1i64..=1).any(|dx| {
                let (vx, vy) = (x + dx, y + dy);
                vx >= 0
                    && vy >= 0
                    && vx < LADO as i64
                    && vy < LADO as i64
                    && branco(&original, vy as usize * LADO + vx as usize)
            })
        });
        if !encosta {
            continue;
        }
        let c: Vec<f32> = original[i * 4..i * 4 + 3]
            .iter()
            .map(|&v| f32::from(v) / 255.0)
            .collect();
        let a = 1.0 - c.iter().copied().fold(1.0f32, f32::min);
        if a <= 0.0 {
            continue;
        }
        for ch in 0..3 {
            let v = 1.0 + (c[ch] - 1.0) / a;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            {
                rgba[i * 4 + ch] = (v.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            }
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            rgba[i * 4 + 3] = (a * 255.0 + 0.5) as u8;
        }
    }
    so_o_branco_puro(rgba);
}

/// Pior diferença (níveis) e quantos bytes passam de 1, entre duas imagens.
fn compara(a: &[u8], b: &[u8]) -> (u8, usize) {
    a.iter().zip(b).fold((0, 0), |(pior, n), (x, y)| {
        let d = x.abs_diff(*y);
        (pior.max(d), n + usize::from(d > 1))
    })
}

/// O alfa mais alto que a lei deixou na camada (a tinta opaca do miolo tem de continuar a `255`).
fn alfa_maximo(c: &[u8]) -> u8 {
    c.chunks(4).map(|p| p[3]).max().unwrap_or(0)
}

/// SONDA — a orla antes e depois de cada lei: pior diferença e bytes acima de 1 nível entre as duas
/// ordens, e o alfa máximo que sobra na camada.
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_a_orla_do_papel -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico"]
fn diag_a_orla_do_papel() {
    type Lei = (&'static str, fn(&mut [u8]));
    let leis: [Lei; 3] = [
        ("só o branco puro (antes)", so_o_branco_puro),
        ("GIMP color-to-alpha na faixa AA", gimp_na_faixa_aa),
        ("separa_o_branco (matting)", |r| {
            separa_o_branco(r, LADO, LADO)
        }),
    ];
    for tr in &TRACOS {
        let alvo = mostrada(&papel_primeiro(tr));
        for (nome, lei) in leis {
            let mut c = traco_no_branco(tr);
            lei(&mut c);
            let (pior, n) = compara(&mostrada(&c), &alvo);
            eprintln!(
                "{:<28} · {nome:<34}: pior {pior:>3} níveis · {n:>5} bytes > 1 · alfa máx {}",
                tr.nome,
                alfa_maximo(&c)
            );
        }
    }
}

/// O laço MOLE que volta ao ponto de partida (o fecho põe a orla sobre ela mesma).
fn laco_mole(t: &mut PainterTool) {
    t.set_brush_color_srgb8([200, 30, 120]);
    t.paint.brush.hardness = 0.0;
    t.set_brush_size_px(8.0);
    let c = [
        [30.0f32, 30.5],
        [97.5, 30.0],
        [98.0, 97.5],
        [30.5, 98.0],
        [30.0, 30.5],
    ];
    t.on_canvas_pointer(cp(c[0], PointerPhase::Down));
    for w in c.windows(2) {
        for k in 1..=8 {
            #[allow(clippy::cast_precision_loss)]
            let f = k as f32 / 8.0;
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

/// A imagem que o PRODUTO mostra (o composite sobre o papel).
fn imagem(t: &mut PainterTool) -> Vec<u8> {
    t.invalidate_composite();
    t.take_preview_arc().expect("o preview").0.to_vec()
}

/// O ORÁCULO que conhece a tinta `f`: cada píxel opaco não-branco vira `(f, a)` com `a` a projecção
/// exacta na reta branco→`f`; o branco puro sai. É o melhor que qualquer lei pode fazer.
fn oraculo(rgba: &[u8], f: [u8; 3]) -> Vec<u8> {
    let fw: Vec<f32> = f.iter().map(|&v| 255.0 - f32::from(v)).collect();
    let n2: f32 = fw.iter().map(|v| v * v).sum();
    let mut v = rgba.to_vec();
    for px in v.as_chunks_mut::<4>().0 {
        if *px == [255, 255, 255, 255] {
            *px = [0, 0, 0, 0];
        } else if px[3] == 255 {
            let a = px[..3]
                .iter()
                .zip(&fw)
                .map(|(&c, k)| (255.0 - f32::from(c)) * k)
                .sum::<f32>()
                / n2;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let a8 = (a.clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            *px = [f[0], f[1], f[2], a8];
        }
    }
    v
}

/// ⭐ **PINTAR E DEPOIS ESCOLHER O PAPEL DÁ A IMAGEM DE ESCOLHER O PAPEL E DEPOIS PINTAR** (decisão do
/// dono, 2026-10-06), pela porta do produto (o seletor da cor do papel): traço mole, duro, mole na
/// diagonal a meio píxel, uma tinta clara opaca larga e um laço mole que FECHA no ponto de partida,
/// sobre um papel escuro. Vermelho antes: pior `194` níveis em `4 162` bytes (o fio claro da orla AA).
///
/// Duas metades, porque o «±1 nível» pedido tem um PISO que não é da lei, MEDIDO
/// (`diag_o_piso_das_duas_ordens`): o mesmo gesto deixa no branco e numa camada transparente
/// opacidades até `1,8` · `2,2` · `2,7` níveis de alfa apart (mole · diagonal · laço; cada dab
/// arredonda num canal diferente), mesmo com a tinta CONHECIDA. Por isso:
/// - contra o ORÁCULO que conhece a tinta ([`oraculo`]), ±1 — a lei não acrescenta nada;
/// - contra a outra ordem, ≤ `2` — o piso.
///
/// E a tinta opaca do miolo continua opaca: no interior da tinta clara o alfa é `255`.
#[test]
fn pintar_antes_ou_depois_do_papel_da_a_mesma_imagem() {
    type Gesto = (&'static str, [u8; 3], fn(&mut PainterTool));
    let gestos: [Gesto; 5] = [
        ("mole vermelho", TRACOS[0].cor, |t| pinta(t, &TRACOS[0])),
        ("duro vermelho", TRACOS[1].cor, |t| pinta(t, &TRACOS[1])),
        ("mole diagonal, meio píxel", TRACOS[2].cor, |t| {
            pinta(t, &TRACOS[2])
        }),
        ("claro opaco largo", TRACOS[3].cor, |t| pinta(t, &TRACOS[3])),
        ("laço mole que fecha", [200, 30, 120], laco_mole),
    ];
    for (nome, f, gesto) in gestos {
        let mut antes = tool(LADO as u32, PaintMedia::Digital, 8.0);
        gesto(&mut antes);
        let no_branco = antes.canvas_rgba.to_vec();
        escolhe_o_papel(&mut antes);
        let mut depois = tool(LADO as u32, PaintMedia::Digital, 8.0);
        escolhe_o_papel(&mut depois);
        gesto(&mut depois);
        let (a, b) = (imagem(&mut antes), imagem(&mut depois));
        assert!(
            b.chunks(4).any(|p| p[..3] != PAPEL),
            "controlo: {nome}: o gesto pintou"
        );
        let (pior, n) = compara(&a, &mostrada(&oraculo(&no_branco, f)));
        assert!(
            pior <= 1,
            "{nome}: a lei difere do oráculo que conhece a tinta — pior {pior} níveis, {n} bytes > 1"
        );
        let (pior, n) = compara(&a, &b);
        assert!(
            pior <= 2,
            "{nome}: pintar antes do papel difere de pintar depois — pior {pior} níveis, {n} bytes > 1"
        );
        if nome == "claro opaco largo" {
            let i = (70 * LADO + 64) * 4 + 3;
            assert_eq!(
                antes.canvas_rgba[i], 255,
                "a tinta clara opaca virou translúcida no miolo"
            );
        }
    }
}

/// SONDA — onde o laço que fecha difere entre as duas ordens: o píxel, a imagem nas duas, e a camada
/// pintada no branco antes da lei.
#[test]
#[ignore = "diagnóstico"]
fn diag_onde_o_laco_difere() {
    let mut no_branco = tool(LADO as u32, PaintMedia::Digital, 8.0);
    laco_mole(&mut no_branco);
    let cru = no_branco.canvas_rgba.to_vec();
    let mut antes = tool(LADO as u32, PaintMedia::Digital, 8.0);
    laco_mole(&mut antes);
    escolhe_o_papel(&mut antes);
    let mut depois = tool(LADO as u32, PaintMedia::Digital, 8.0);
    escolhe_o_papel(&mut depois);
    laco_mole(&mut depois);
    let (a, b) = (imagem(&mut antes), imagem(&mut depois));
    for i in 0..LADO * LADO {
        let (pa, pb) = (&a[i * 4..i * 4 + 4], &b[i * 4..i * 4 + 4]);
        if pa.iter().zip(pb).any(|(x, y)| x.abs_diff(*y) > 1) {
            eprintln!(
                "({}, {}): antes {pa:?} depois {pb:?} · no branco {:?} · camada antes {:?} · camada depois {:?}",
                i % LADO,
                i / LADO,
                &cru[i * 4..i * 4 + 4],
                &antes.canvas_rgba[i * 4..i * 4 + 4],
                &depois.canvas_rgba[i * 4..i * 4 + 4]
            );
        }
    }
}

/// SONDA — o PISO do critério, sem lei nenhuma: o mesmo gesto pintado no branco e numa camada
/// transparente guarda opacidades diferentes (cada dab arredonda num canal diferente). Com a tinta
/// CONHECIDA, a opacidade exacta do píxel no branco é a projecção na reta branco→tinta; contra o alfa
/// do traço na camada transparente, a pior diferença em níveis e quantos píxeis passam de 1.
#[test]
#[ignore = "diagnóstico"]
fn diag_o_piso_das_duas_ordens() {
    type Gesto = (&'static str, [u8; 3], fn(&mut PainterTool));
    let gestos: [Gesto; 3] = [
        ("mole vermelho", TRACOS[0].cor, |t| pinta(t, &TRACOS[0])),
        ("mole diagonal", TRACOS[2].cor, |t| pinta(t, &TRACOS[2])),
        ("laço mole que fecha", [200, 30, 120], laco_mole),
    ];
    for (nome, f, gesto) in gestos {
        let mut branco = tool(LADO as u32, PaintMedia::Digital, 8.0);
        gesto(&mut branco);
        let mut transparente = tool(LADO as u32, PaintMedia::Digital, 8.0);
        escolhe_o_papel(&mut transparente);
        gesto(&mut transparente);
        let fw: Vec<f32> = f.iter().map(|&v| 255.0 - f32::from(v)).collect();
        let n2: f32 = fw.iter().map(|v| v * v).sum();
        let (mut pior, mut acima) = (0.0f32, 0);
        for i in 0..LADO * LADO {
            let c = &branco.canvas_rgba[i * 4..i * 4 + 3];
            let a_branco = c
                .iter()
                .zip(&fw)
                .map(|(&v, k)| (255.0 - f32::from(v)) * k)
                .sum::<f32>()
                / n2
                * 255.0;
            let a_transp = f32::from(transparente.canvas_rgba[i * 4 + 3]);
            let d = (a_branco - a_transp).abs();
            pior = pior.max(d);
            acima += usize::from(d > 1.5);
        }
        eprintln!(
            "{nome:<22}: pior {pior:.2} níveis de alfa entre as duas telas · {acima} píxeis acima de 1,5"
        );
    }
}

/// SONDA — o preço de separar o branco (corre UMA vez, na 1.ª cor do papel): uma tela branca com 12
/// traços moles, a 2048² e 4096², três corridas, o mínimo e a mediana.
#[test]
#[ignore = "diagnóstico"]
fn diag_o_preco_de_separar_o_branco() {
    for lado in [2048u32, 4096] {
        let mut t = tool(lado, PaintMedia::Digital, 24.0);
        t.paint.brush.hardness = 0.0;
        for k in 0..12u8 {
            t.set_brush_color_srgb8([40 + k * 15, 200 - k * 12, 90]);
            let y = (f32::from(k) + 1.0) * lado as f32 / 13.0;
            t.on_canvas_pointer(cp([40.0, y], PointerPhase::Down));
            for i in 1..=24 {
                #[allow(clippy::cast_precision_loss)]
                let x = 40.0 + (lado as f32 - 80.0) * i as f32 / 24.0;
                t.on_canvas_pointer(cp(
                    [x, y + 30.0 * (i as f32 * 0.7).sin()],
                    PointerPhase::Move,
                ));
            }
            t.on_canvas_pointer(cp([lado as f32 - 40.0, y], PointerPhase::Up));
        }
        let base = t.canvas_rgba.to_vec();
        let mut ms: Vec<f64> = (0..3)
            .map(|_| {
                let mut c = base.clone();
                let t0 = std::time::Instant::now();
                separa_o_branco(&mut c, lado as usize, lado as usize);
                t0.elapsed().as_secs_f64() * 1e3
            })
            .collect();
        ms.sort_by(f64::total_cmp);
        let pintados = base.chunks(4).filter(|p| p[..3] != [255, 255, 255]).count();
        eprintln!(
            "{lado}²: {pintados} píxeis pintados · {:.1} ms (mín) · {:.1} ms (mediana) · loadavg {}",
            ms[0],
            ms[1],
            std::fs::read_to_string("/proc/loadavg")
                .unwrap_or_default()
                .trim()
        );
    }
}

/// ⭐ **UMA TINTA CLARA OPACA ENCOSTADA A UMA ESCURA DA MESMA FAMÍLIA CONTINUA OPACA** — a mancha rosa
/// larga (opaca) cruzada por um traço vermelho duro pintado depois, no branco: o rosa está quase na
/// reta branco→vermelho, e a subida até à crista não pode lê-lo como orla do vermelho (ficaria
/// translúcido, contra a escolha do dono: a tinta opaca cobre o papel). Em todo o interior do rosa —
/// longe da orla dele com o branco — o alfa continua `255` depois de escolher o papel.
#[test]
fn a_tinta_clara_encostada_a_escura_continua_opaca() {
    let mut t = tool(LADO as u32, PaintMedia::Digital, 8.0);
    pinta(&mut t, &TRACOS[3]);
    let rosa = t.canvas_rgba.to_vec();
    pinta(
        &mut t,
        &Traco {
            nome: "vermelho duro",
            cor: [220, 40, 40],
            strength: 1.0,
            dureza: 1.0,
            raio: 4.0,
            de: [64.0, 30.0],
            ate: [64.0, 110.0],
        },
    );
    let antes = t.canvas_rgba.to_vec();
    escolhe_o_papel(&mut t);
    let rosa_cor = [250u8, 210, 215, 255];
    let mut vistos = 0;
    for i in 0..LADO * LADO {
        let (x, y) = ((i % LADO) as i64, (i / LADO) as i64);
        // Interior do rosa: rosa puro agora e em toda a vizinhança 3×3 na pintura SÓ do rosa.
        let interior = (-2i64..=2).all(|dy| {
            (-2i64..=2).all(|dx| {
                let (vx, vy) = (x + dx, y + dy);
                vx >= 0
                    && vy >= 0
                    && vx < LADO as i64
                    && vy < LADO as i64
                    && rosa[(vy as usize * LADO + vx as usize) * 4..][..4] == rosa_cor
            })
        });
        if interior && antes[i * 4..i * 4 + 4] == rosa_cor {
            vistos += 1;
            assert_eq!(
                t.canvas_rgba[i * 4 + 3],
                255,
                "({x}, {y}): o rosa opaco encostado ao vermelho virou translúcido"
            );
        }
    }
    assert!(
        vistos > 500,
        "controlo: a fixtura tem interior rosa ({vistos})"
    );
}

/// Uma tira sintética `w × 3` toda branca com a linha do meio dada (cada píxel como «branco − cor»).
fn tira(meio: &[[f32; 3]]) -> Vec<u8> {
    let w = meio.len();
    let mut v = vec![255u8; w * 3 * 4];
    for (x, d) in meio.iter().enumerate() {
        let i = (w + x) * 4;
        for c in 0..3 {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            {
                v[i + c] = (255.0 - d[c]).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    v
}

/// ⭐ **A ORLA SOBE PARA A CRISTA MAIS OPACA** — um vale entre duas cristas da MESMA tinta (a mais
/// clara pode ser ela própria uma mistura com o branco): o píxel do vale fica com a tinta da crista
/// mais escura, a estimativa mais verdadeira da tinta. Subir pelo vizinho mais claro deixava-o com a
/// cor da crista clara.
#[test]
fn a_orla_sobe_para_a_crista_mais_opaca() {
    let dir = [0.14f32, 0.88, 0.45];
    let em = |t: f32| [dir[0] * t, dir[1] * t, dir[2] * t];
    // W · B (crista clara) · p (vale) · A (crista escura) · W
    let mut v = tira(&[[0.0; 3], em(120.0), em(60.0), em(220.0), [0.0; 3]]);
    let antes = v.clone();
    separa_o_branco(&mut v, 5, 3);
    let (p, a) = (5 + 2, 5 + 3);
    assert_eq!(
        v[p * 4..p * 4 + 3],
        antes[a * 4..a * 4 + 3],
        "o vale ficou com a tinta {:?}, e a crista mais opaca é {:?}",
        &v[p * 4..p * 4 + 3],
        &antes[a * 4..a * 4 + 3]
    );
    assert!(v[p * 4 + 3] < 255, "controlo: o vale foi separado");
}

/// ⭐ **COM PAPEL BRANCO A SEPARAÇÃO NÃO MUDA A IMAGEM** — a camada separada, composta sobre o branco,
/// devolve cada píxel dentro da tolerância da reta. A tira deriva de cor aos poucos (cada passo na
/// reta do vizinho, o conjunto não): o píxel cuja cadeia acaba numa tinta FORA da sua reta não pode
/// ser separado contra ela (dava `~26` níveis). Também nos traços reais da fixtura.
#[test]
fn com_papel_branco_a_separacao_nao_muda_a_imagem() {
    let deriva: Vec<[f32; 3]> = std::iter::once([0.0; 3])
        .chain((1..=8).map(|k| {
            #[allow(clippy::cast_precision_loss)]
            let (t, th) = (20.0 + 20.0 * k as f32, (10.0 + 6.0 * k as f32).to_radians());
            [t * th.cos(), t * th.sin(), 0.3 * t]
        }))
        .collect();
    let w = deriva.len();
    let mut casos: Vec<(String, Vec<u8>, usize, usize)> =
        vec![("tira que deriva".into(), tira(&deriva), w, 3)];
    for tr in &TRACOS[..4] {
        casos.push((tr.nome.into(), traco_no_branco(tr), LADO, LADO));
    }
    for (nome, original, w, h) in casos {
        let mut v = original.clone();
        separa_o_branco(&mut v, w, h);
        let mut sobre_branco = v.clone();
        for px in sobre_branco.as_chunks_mut::<4>().0 {
            sobre_o_papel(px, [255, 255, 255]);
        }
        for i in 0..w * h {
            let c: Vec<f32> = (0..3)
                .map(|k| 255.0 - f32::from(original[i * 4 + k]))
                .collect();
            let tol = 3.0 + 0.2 * c.iter().map(|x| x * x).sum::<f32>().sqrt();
            for k in 0..3 {
                let d = f32::from(sobre_branco[i * 4 + k].abs_diff(original[i * 4 + k]));
                assert!(
                    d <= tol,
                    "{nome}: ({}, {}) canal {k}: sobre o branco a separação mudou {d} níveis",
                    i % w,
                    i / w
                );
            }
        }
    }
}

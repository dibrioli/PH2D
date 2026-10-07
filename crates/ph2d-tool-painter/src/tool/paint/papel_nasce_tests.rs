//! **O papel existe desde o primeiro instante** (pedido do dono 2026-10-06, com fotos: *«após pintar
//! com papel branco e depois escurecer o papel, as áreas pintadas não sofreram nenhum escurecimento e
//! em wet paint pontos brancos apareceram ao redor. Corrija. Permita o papel escurecer a tinta como no
//! mundo real.»*). O critério é o do Rebelle, medido (doc 47): o papel nunca é tinta, a tinta guarda o
//! alfa que o pincel depositou, e a ordem não importa.
#![allow(clippy::cast_precision_loss)]

use super::measure_shape_system::{cp, tool};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PanelEvent, PointerPhase, RasterEditTool, Tool};

const LADO: usize = 128;
const MEIOS: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];
const BRANCO: [u8; 3] = [255, 255, 255];
/// O castanho do papel do Rebelle (doc 47) — escuro: é onde «não escureceu» e o branco mais aparecem.
const CASTANHO: [u8; 3] = [112, 88, 52];
const TINTA: [u8; 3] = [30, 60, 220];

/// Escolhe o papel pela porta do produto: o seletor da cor do papel.
fn escolhe_o_papel(t: &mut PainterTool, p: [u8; 3]) {
    t.handle_panel_event(PanelEvent::SelectOption(
        crate::ids::PAINTER_WATERCOLOR_PAPER_COLOR_THUMB,
        format!("{},{},{}", p[0], p[1], p[2]),
    ));
}

/// Um gesto: nome e pontos (um evento por ponto, um quadro por evento).
type Gesto = (&'static str, fn() -> Vec<[f32; 2]>);

const GESTOS: [Gesto; 3] = [
    ("horizontal", || {
        (0..=22)
            .map(|k| [20.0 + 4.0 * k as f32, 64.5 + 0.3 * (k % 3) as f32])
            .collect()
    }),
    ("diagonal a meio píxel", || {
        (0..=24)
            .map(|k| {
                let f = k as f32 / 24.0;
                [20.5 + 80.0 * f, 20.5 + 87.0 * f]
            })
            .collect()
    }),
    ("laço que fecha", || {
        let c = [
            [30.0f32, 30.5],
            [97.5, 30.0],
            [98.0, 97.5],
            [30.5, 98.0],
            [30.0, 30.5],
        ];
        let mut v = vec![c[0]];
        for w in c.windows(2) {
            for k in 1..=8 {
                let f = k as f32 / 8.0;
                v.push([
                    w[0][0] + (w[1][0] - w[0][0]) * f,
                    w[0][1] + (w[1][1] - w[0][1]) * f,
                ]);
            }
        }
        v
    }),
];

/// O gesto com o relógio do Wet Paint fixo, e 30 quadros parados depois do pen-up.
fn pinta(t: &mut PainterTool, strength: f32, pts: &[[f32; 2]]) {
    t.set_wet_relogio_fixo(true);
    t.set_brush_color_srgb8(TINTA);
    t.paint.brush.strength = strength;
    t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
    for p in &pts[1..] {
        t.on_canvas_pointer(cp(*p, PointerPhase::Move));
        t.paint_tick(1.0 / 60.0);
    }
    t.on_canvas_pointer(cp(*pts.last().expect("pontos"), PointerPhase::Up));
    for _ in 0..30 {
        t.paint_tick(1.0 / 60.0);
    }
}

/// **Um desenho NOVO pela porta do produto** — a sprite branca do *New Image…* ligada ao pintor.
fn novo(meio: PaintMedia, raio: f32) -> PainterTool {
    let mut t = PainterTool::default();
    t.bind_document(1, vec![255u8; LADO * LADO * 4], LADO as u32, LADO as u32);
    t.set_paint_media(meio);
    t.set_brush_size_px(raio);
    t
}

/// A arte que já nasceu branca (a fonte crua: a camada É a tela branca opaca, sem papel).
fn arte_branca(meio: PaintMedia, raio: f32) -> PainterTool {
    tool(LADO as u32, meio, raio)
}

/// A imagem que o produto mostra.
fn imagem(t: &mut PainterTool) -> Vec<u8> {
    t.invalidate_composite();
    t.take_preview_arc().expect("o preview").0.to_vec()
}

fn lum(p: &[u8]) -> f32 {
    0.2126 * f32::from(p[0]) + 0.7152 * f32::from(p[1]) + 0.0722 * f32::from(p[2])
}

/// Pior diferença (níveis) e quantos bytes passam de `tol`, entre duas imagens.
fn compara(a: &[u8], b: &[u8], tol: u8) -> (u8, usize) {
    a.iter().zip(b).fold((0, 0), |(pior, n), (x, y)| {
        let d = x.abs_diff(*y);
        (pior.max(d), n + usize::from(d > tol))
    })
}

/// Os pontos brancos: píxeis quase brancos OPACOS na camada (fora do branco puro).
fn quase_brancos_opacos(camada: &[u8]) -> usize {
    camada
        .chunks(4)
        .filter(|p| p[3] == 255 && p[..3].iter().all(|&v| v >= 230) && p[..3] != BRANCO)
        .count()
}

/// Quanto a tinta escurece com o papel (`1` = como o alvo; `0` = ficou como no branco): nos texels
/// que o alvo escurece, a queda de luminância desde o branco relativa à do alvo.
fn escurece(w: &[u8], a: &[u8], b: &[u8]) -> f32 {
    let (mut q, mut qa) = (0.0f32, 0.0f32);
    for k in 0..w.len() / 4 {
        let (pw, pa, pb) = (
            &w[k * 4..k * 4 + 3],
            &a[k * 4..k * 4 + 3],
            &b[k * 4..k * 4 + 3],
        );
        if pw != BRANCO && lum(pw) - lum(pb) > 2.0 {
            q += lum(pw) - lum(pa);
            qa += lum(pw) - lum(pb);
        }
    }
    q / qa.max(1e-6)
}

/// SONDA — antes de construir (CLAUDE.md §0.10, uma rodada):
/// 1. o alfa que cada meio deposita num desenho novo (camada TRANSPARENTE), Strength 1 e 0,5;
/// 2. a cena do dono: pintar no branco e escurecer o papel, na lei de hoje (`hoje`: a tela branca
///    opaca) e na nova (`novo`: o desenho que nasce transparente sobre o papel branco), contra pintar
///    sobre o papel já escuro (o alvo): quanto escurece, a pior diferença, os pontos brancos (quase
///    brancos opacos na camada) e os texels MOSTRADOS mais claros que o alvo por mais de 40;
/// 3. a ponte: o desenho novo no branco mostra-se como o de hoje (pior · bytes > 1).
///
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_o_papel_desde_o_inicio -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico"]
fn diag_o_papel_desde_o_inicio() {
    eprintln!("\n[alfa] meio        strength  alfa miolo  alfa máx  alfa médio  texels  cor miolo");
    for meio in MEIOS {
        for s in [1.0f32, 0.5] {
            let mut t = novo(meio, 6.0);
            pinta(&mut t, s, &(GESTOS[0].1)());
            let c = &t.canvas_rgba;
            let i = (64 * LADO + 64) * 4;
            let pintados: Vec<u8> = c.chunks(4).map(|p| p[3]).filter(|&a| a > 0).collect();
            let medio =
                pintados.iter().map(|&a| f32::from(a)).sum::<f32>() / pintados.len().max(1) as f32;
            eprintln!(
                "[alfa] {meio:<11?} {s:<8} {:>10.3}  {:>8.3}  {:>10.3}  {:>6}  {:?}",
                f32::from(c[i + 3]) / 255.0,
                f32::from(*pintados.iter().max().unwrap_or(&0)) / 255.0,
                medio / 255.0,
                pintados.len(),
                &c[i..i + 3]
            );
        }
    }
    eprintln!(
        "\n[cena] meio        s    raio  gesto                  lei   escurece  pior  bytes>1  pontos  claros  | ponte pior  bytes>1"
    );
    for meio in MEIOS {
        for (s, raio) in [(1.0f32, 6.0f32), (0.5, 6.0), (1.0, 14.0)] {
            for (nome, gesto) in GESTOS {
                let pts = gesto();
                let mut alvo = novo(meio, raio);
                escolhe_o_papel(&mut alvo, CASTANHO);
                pinta(&mut alvo, s, &pts);
                let b = imagem(&mut alvo);
                let mut no_branco = novo(meio, raio);
                pinta(&mut no_branco, s, &pts);
                let w = imagem(&mut no_branco);
                let mut hoje_branco = arte_branca(meio, raio);
                pinta(&mut hoje_branco, s, &pts);
                let (ponte, nponte) = compara(&imagem(&mut hoje_branco), &w, 1);
                let mut hoje = arte_branca(meio, raio);
                pinta(&mut hoje, s, &pts);
                escolhe_o_papel(&mut hoje, CASTANHO);
                let mut agora = novo(meio, raio);
                pinta(&mut agora, s, &pts);
                escolhe_o_papel(&mut agora, CASTANHO);
                for (lei, t) in [("hoje", &mut hoje), ("novo", &mut agora)] {
                    let a = imagem(t);
                    let (pior, n) = compara(&a, &b, 1);
                    let claros = (0..LADO * LADO)
                        .filter(|&k| lum(&a[k * 4..k * 4 + 3]) > lum(&b[k * 4..k * 4 + 3]) + 40.0)
                        .count();
                    eprintln!(
                        "[cena] {meio:<11?} {s:<4} {raio:<4}  {nome:<22} {lei:<5} {:>8.3}  {pior:>4}  {n:>7}  {:>6}  {claros:>6}  | {ponte:>10}  {nponte:>7}",
                        escurece(&w, &a, &b),
                        quase_brancos_opacos(&t.canvas_rgba),
                    );
                }
            }
        }
    }
}

/// SONDA — **o quadro de um desenho novo**: o papel branco tira a camada da pista trivial (sem
/// cópia) e a manda compor sobre o papel. Por movimento (o evento + a drenagem do preview), nos quatro
/// meios, a 2048² e 4096², a lei de hoje (a tela branca opaca) contra a nova (camada transparente
/// sobre o papel branco), no MESMO processo, três rodadas intercaladas: o mínimo das medianas, com a
/// mediana das medianas ao lado.
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_o_quadro_de_um_desenho_novo -- --ignored --nocapture --test-threads=1`
#[test]
#[ignore = "medição"]
fn diag_o_quadro_de_um_desenho_novo() {
    fn por_move(meio: PaintMedia, lado: u32, novo: bool) -> f64 {
        let mut t = PainterTool::default();
        let branco = vec![255u8; (lado * lado * 4) as usize];
        if novo {
            t.bind_document(1, branco, lado, lado);
        } else {
            t.set_source(branco, lado, lado);
        }
        assert_eq!(t.papel().is_some(), novo, "controlo: a porta");
        t.set_paint_media(meio);
        t.set_brush_size_px(40.0);
        t.set_wet_relogio_fixo(true);
        let mid = (lado / 2) as f32;
        t.on_canvas_pointer(cp([80.0, mid], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let mut ms = Vec::new();
        for k in 1..=12u8 {
            let x = 80.0 + 30.0 * f32::from(k);
            let t0 = std::time::Instant::now();
            t.on_canvas_pointer(cp([x, mid], PointerPhase::Move));
            t.paint_tick(1.0 / 60.0);
            let _ = t.take_preview_arc();
            ms.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        t.on_canvas_pointer(cp([80.0 + 30.0 * 13.0, mid], PointerPhase::Up));
        ms.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        ms[ms.len() / 2]
    }
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("\n[quadro] loadavg {}", carga().trim());
    let casos: Vec<(PaintMedia, u32)> = MEIOS
        .iter()
        .flat_map(|&m| [(m, 2048u32), (m, 4096)])
        .collect();
    let mut hoje = vec![Vec::new(); casos.len()];
    let mut agora = vec![Vec::new(); casos.len()];
    for _rodada in 0..3 {
        for (k, &(meio, lado)) in casos.iter().enumerate() {
            hoje[k].push(por_move(meio, lado, false));
            agora[k].push(por_move(meio, lado, true));
        }
    }
    eprintln!("[quadro] meio        lado   hoje ms (mín · med)   novo ms (mín · med)   novo/hoje");
    for (k, &(meio, lado)) in casos.iter().enumerate() {
        let mm = |v: &mut Vec<f64>| {
            v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            (v[0], v[v.len() / 2])
        };
        let (hm, hd) = mm(&mut hoje[k]);
        let (am, ad) = mm(&mut agora[k]);
        eprintln!(
            "[quadro] {meio:<11?} {lado:<6} {hm:>7.3} · {hd:>7.3}      {am:>7.3} · {ad:>7.3}      {:>5.2}×",
            am / hm.max(1e-6)
        );
    }
    eprintln!("[quadro] loadavg {}", carga().trim());
    // E o preço de ABRIR a sprite (uma vez, ao escolhê-la): a varredura do branco e a camada a zero.
    for lado in [2048u32, 4096] {
        let mut ms = [Vec::new(), Vec::new()];
        for _ in 0..5 {
            for (k, novo) in [false, true].into_iter().enumerate() {
                let branco = vec![255u8; (lado * lado * 4) as usize];
                let mut t = PainterTool::default();
                let t0 = std::time::Instant::now();
                if novo {
                    t.bind_document(1, branco, lado, lado);
                } else {
                    t.set_source(branco, lado, lado);
                }
                ms[k].push(t0.elapsed().as_secs_f64() * 1e3);
            }
        }
        for v in &mut ms {
            v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        }
        eprintln!(
            "[abrir] {lado}²: a fonte crua {:.2} ms · a sprite em branco vira papel {:.2} ms (medianas de 5)",
            ms[0][2], ms[1][2]
        );
    }
}

/// Os casos de pincel dos gates: (Strength, raio).
const PINCEIS: [(f32, f32); 3] = [(1.0, 6.0), (0.5, 6.0), (1.0, 14.0)];

/// ⭐ **PINTAR NO BRANCO E DEPOIS ESCURECER O PAPEL = PINTAR SOBRE O PAPEL ESCURO** (pedido do dono,
/// 2026-10-06), pela porta do produto: o desenho novo (a sprite branca) e o seletor da cor do papel,
/// nos quatro meios, Strength 1 e 0,5, raio 6 e 14, três gestos (horizontal, diagonal a meio píxel,
/// laço que fecha). ±1 nível. Vermelho antes: a tinta escurecia só `0,23`–`0,94` do que devia, pior
/// `202` níveis (`diag_o_papel_desde_o_inicio`, lei `hoje`).
#[test]
fn pintar_e_depois_escurecer_o_papel_e_pintar_sobre_ele() {
    for meio in MEIOS {
        for (s, raio) in PINCEIS {
            for (nome, gesto) in GESTOS {
                let pts = gesto();
                let mut alvo = novo(meio, raio);
                escolhe_o_papel(&mut alvo, CASTANHO);
                pinta(&mut alvo, s, &pts);
                let mut depois = novo(meio, raio);
                pinta(&mut depois, s, &pts);
                escolhe_o_papel(&mut depois, CASTANHO);
                assert_eq!(
                    depois.papel(),
                    Some(CASTANHO),
                    "controlo: o seletor aplicou o papel"
                );
                let (a, b) = (imagem(&mut depois), imagem(&mut alvo));
                assert!(
                    b.chunks(4).any(|p| p[..3] != CASTANHO),
                    "controlo: {meio:?} {nome}: o gesto pintou"
                );
                let (pior, n) = compara(&a, &b, 1);
                assert!(
                    pior <= 1,
                    "{meio:?} s {s} raio {raio} {nome}: escurecer o papel depois difere de pintar sobre \
                     ele — pior {pior} níveis, {n} bytes > 1 (escurece {:.3})",
                    escurece(&imagem(&mut novo(meio, raio)), &a, &b)
                );
            }
        }
    }
}

/// ⭐ **OS PONTOS BRANCOS DO WET PAINT** (a 2.ª foto do dono): pintado no branco e escurecido o papel,
/// nenhum píxel quase branco OPACO fica na camada, e nenhum texel se mostra mais claro que o alvo por
/// mais de 40 níveis. Vermelho antes (raio 14): `63` · `19` · `83` pontos, `325` · `125` · `1 062` texels
/// claros (horizontal · diagonal · laço).
#[test]
fn no_wet_paint_o_papel_escuro_nao_deixa_pontos_brancos() {
    for (nome, gesto) in GESTOS {
        let pts = gesto();
        let mut alvo = novo(PaintMedia::WetPaint, 14.0);
        escolhe_o_papel(&mut alvo, CASTANHO);
        pinta(&mut alvo, 1.0, &pts);
        let b = imagem(&mut alvo);
        let mut t = novo(PaintMedia::WetPaint, 14.0);
        pinta(&mut t, 1.0, &pts);
        assert!(
            t.canvas_rgba.chunks(4).any(|p| p[3] > 0 && p[3] < 255),
            "controlo: {nome}: o fluido deixou orla"
        );
        escolhe_o_papel(&mut t, CASTANHO);
        let a = imagem(&mut t);
        let pontos = quase_brancos_opacos(&t.canvas_rgba);
        let claros = (0..LADO * LADO)
            .filter(|&k| lum(&a[k * 4..k * 4 + 3]) > lum(&b[k * 4..k * 4 + 3]) + 40.0)
            .count();
        assert_eq!(
            (pontos, claros),
            (0, 0),
            "{nome}: pontos brancos na orla do Wet Paint (quase brancos opacos · texels claros)"
        );
    }
}

/// ⭐ **UM DESENHO NOVO NASCE TRANSPARENTE SOBRE O PAPEL BRANCO, E NO BRANCO MOSTRA-SE COMO O DE HOJE**
/// — a sprite branca vira papel branco com a camada vazia; mostra-se e assa-se branca, ao byte; e o
/// mesmo traço mostra-se como na tela branca opaca de antes até ao PISO medido das duas representações
/// (`diag_o_papel_desde_o_inicio`, coluna «ponte»): cada dab arredonda num canal diferente quando a tinta
/// cai no branco e quando guarda o seu alfa — pior `4` no Digital e no Impasto (o mesmo piso do #42),
/// `1` na Aquarela, `0` no Wet Paint.
#[test]
fn um_desenho_novo_nasce_transparente_sobre_o_papel_branco() {
    let mut t = novo(PaintMedia::Digital, 6.0);
    assert_eq!(
        t.papel(),
        Some(BRANCO),
        "o desenho novo não nasceu com o papel branco"
    );
    assert!(
        t.canvas_rgba.chunks(4).all(|p| p == [0, 0, 0, 0]),
        "o papel entrou na camada"
    );
    assert!(
        imagem(&mut t).chunks(4).all(|p| p == [255, 255, 255, 255]),
        "o desenho novo não se mostra branco"
    );
    t.invalidate_composite();
    assert!(
        t.current_preview()
            .expect("a porta genérica do preview")
            .0
            .chunks(4)
            .all(|p| p == [255, 255, 255, 255]),
        "a porta genérica do preview não compõe sobre o papel"
    );
    assert!(
        t.run_full().0.chunks(4).all(|p| p == [255, 255, 255, 255]),
        "o desenho novo não se assa branco"
    );
    for meio in MEIOS {
        let piso = match meio {
            PaintMedia::Digital | PaintMedia::Impasto => 4,
            PaintMedia::Watercolor => 1,
            PaintMedia::WetPaint => 0,
        };
        for (s, raio) in PINCEIS {
            for (nome, gesto) in GESTOS {
                let pts = gesto();
                let mut agora = novo(meio, raio);
                pinta(&mut agora, s, &pts);
                let mut antes = arte_branca(meio, raio);
                pinta(&mut antes, s, &pts);
                let a = imagem(&mut agora);
                assert!(
                    a.chunks(4).any(|p| p[..3] != BRANCO),
                    "controlo: {meio:?} {nome}: o gesto pintou"
                );
                let (pior, n) = compara(&a, &imagem(&mut antes), 1);
                assert!(
                    pior <= piso,
                    "{meio:?} s {s} raio {raio} {nome}: o desenho novo mostra-se diferente do de hoje no \
                     branco — pior {pior} níveis (piso {piso}), {n} bytes > 1"
                );
                assert_eq!(
                    agora.run_full().0,
                    a,
                    "{meio:?} {nome}: o Apply não assa o que se mostra"
                );
            }
        }
    }
}

/// **SÓ A SPRITE TODA BRANCA NASCE PAPEL** — uma sprite com arte (um píxel que seja) é a camada, sem
/// papel, como chegou (e a 1.ª cor do papel separa-lhe o branco, BUGS #42); a transparente do *New
/// Image…* fica transparente, sem papel.
#[test]
fn so_a_sprite_toda_branca_nasce_papel() {
    let mut arte = vec![255u8; LADO * LADO * 4];
    arte[(70 * LADO + 33) * 4..(70 * LADO + 33) * 4 + 4].copy_from_slice(&[254, 255, 255, 255]);
    let mut t = PainterTool::default();
    t.bind_document(1, arte.clone(), LADO as u32, LADO as u32);
    assert_eq!(t.papel(), None, "a arte ganhou papel");
    assert_eq!(
        t.canvas_rgba.as_slice(),
        arte.as_slice(),
        "a arte não chegou como estava"
    );
    let vazia = vec![0u8; LADO * LADO * 4];
    t.bind_document(2, vazia.clone(), LADO as u32, LADO as u32);
    assert_eq!(t.papel(), None, "a sprite transparente ganhou papel");
    assert_eq!(t.canvas_rgba.as_slice(), vazia.as_slice());
}

/// **O DESENHO NOVO COM TINTA GUARDA O ALFA DELA NA TROCA DE SPRITE** — intocado, ele reconstrói-se da
/// sprite branca e é descartado na troca (nada fica guardado); com tinta, o alfa só vive no documento
/// (a sprite tem a imagem assada, opaca), então ele é guardado, e ao voltar o papel ainda escurece a
/// tinta.
#[test]
fn o_desenho_novo_com_tinta_guarda_o_alfa_na_troca_de_sprite() {
    let branco = || vec![255u8; LADO * LADO * 4];
    let mut t = novo(PaintMedia::Digital, 6.0);
    t.bind_document(2, branco(), LADO as u32, LADO as u32);
    assert!(
        !t.doc_cache.contains_key(&1),
        "o desenho intocado foi guardado"
    );
    t.bind_document(1, branco(), LADO as u32, LADO as u32);
    pinta(&mut t, 0.5, &(GESTOS[0].1)());
    let antes = t.canvas_rgba.to_vec();
    assert!(
        antes.chunks(4).any(|p| p[3] > 0 && p[3] < 255),
        "controlo: tinta a meio alfa"
    );
    t.bind_document(2, branco(), LADO as u32, LADO as u32);
    assert!(
        t.doc_cache.contains_key(&1),
        "o desenho com tinta não foi guardado"
    );
    t.bind_document(1, branco(), LADO as u32, LADO as u32);
    assert_eq!(t.papel(), Some(BRANCO));
    assert_eq!(
        t.canvas_rgba.as_slice(),
        antes.as_slice(),
        "o alfa da tinta perdeu-se na troca"
    );
}

/// **NO DESENHO NOVO, O BRANCO NO SELETOR NÃO FAZ NADA E O DESFAZER VOLTA AO PAPEL BRANCO** — a camada
/// nunca é tocada pelo papel.
#[test]
fn no_desenho_novo_o_desfazer_da_cor_volta_ao_papel_branco() {
    let mut t = novo(PaintMedia::Digital, 6.0);
    escolhe_o_papel(&mut t, BRANCO);
    assert!(
        !t.undo_last(),
        "o branco no seletor fez um passo de desfazer"
    );
    escolhe_o_papel(&mut t, CASTANHO);
    assert_eq!(t.papel(), Some(CASTANHO));
    assert!(t.undo_last(), "há o que desfazer");
    assert_eq!(
        t.papel(),
        Some(BRANCO),
        "o desfazer não voltou ao papel branco"
    );
    assert!(
        t.canvas_rgba.chunks(4).all(|p| p == [0, 0, 0, 0]),
        "o papel tocou a camada"
    );
    assert!(imagem(&mut t).chunks(4).all(|p| p == [255, 255, 255, 255]));
}

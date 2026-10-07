//! **A aguada é um FILTRO sobre o papel de cor** (doc 48, BUGS #46; pedido do dono 2026-10-07: *«eu
//! quero o estado da arte»*): o vidrado de Kubelka–Munk (Curtis et al., SIGGRAPH 1997) — a pilha
//! sobre o papel é `R + papel·T` por canal, com `R` o que a aguada devolve sobre o preto e `T` o que
//! o chão atravessa. O oráculo é a própria óptica sobre DOIS chãos (uma camada opaca branca e uma
//! preta por baixo), a forma como Curtis especifica um pigmento.

#![allow(clippy::cast_precision_loss)]

use super::measure_shape_system::cp;
use super::papel_nasce_tests::{GESTOS, LADO, escolhe_o_papel, imagem, novo, pinta_com_a_cor};
use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase};

/// O PISO medido das duas contas (`a_aguada_e_um_filtro_sobre_o_papel`, 24 casos): o oráculo arredonda
/// as DUAS camadas que pinta (sobre o branco e sobre o preto), o produto arredonda o alfa de cada canal
/// em `u8` — pior `2` níveis, num texel opaco de corpo 1.
const PISO: i32 = 2;
/// O papel castanho da foto do dono.
pub(super) const PAPEL: [u8; 3] = [153, 121, 91];
const CORES: [[u8; 3]; 4] = [[255, 0, 0], [220, 40, 40], [30, 60, 220], [240, 200, 40]];
const CORPOS: [f32; 3] = [0.0, 0.4, 1.0];

/// Os traços de um caso: um gesto, ou dois que se sobrepõem com o 1.º SECO entre eles (o 2.º pinta
/// sobre a base do 1.º) — o VIDRADO de Curtis, um vidro sobre outro. ⚠️ Molhado sobre molhado a sessão
/// re-renderiza a união com os campos da re-molhagem, que leem a presença de tinta CONTRA o chão: o
/// oráculo do preto recalcula-os contra o preto e deixa de ser oráculo (medido: `8` níveis, amarelo e
/// depois ciano, corpo 0); o produto decide-os sobre o branco de referência.
fn tracos(t: &mut PainterTool, cor: [u8; 3], corpo: f32, dois: bool) {
    t.paint.brush.opacity = corpo;
    t.set_brush_color_srgb8(cor);
    pinta_com_a_cor(t, 1.0, &(GESTOS[2].1)());
    if dois {
        t.dry_session_now();
        t.set_brush_color_srgb8([cor[2], cor[0], cor[1]]);
        pinta_com_a_cor(t, 1.0, &(GESTOS[1].1)());
    }
}

/// O ORÁCULO: os mesmos traços numa camada sobre um chão OPACO (`chao`) — a imagem mostrada.
fn sobre_chao(chao: [u8; 3], cor: [u8; 3], corpo: f32, dois: bool) -> Vec<u8> {
    let mut t = PainterTool::default();
    t.bind_document(1, vec![255u8; LADO * LADO * 4], LADO as u32, LADO as u32);
    let fundo: Vec<u8> = (0..LADO * LADO)
        .flat_map(|_| [chao[0], chao[1], chao[2], 255])
        .collect();
    t.add_raster_layer_with_pixels("chão", fundo)
        .expect("o chão");
    t.add_raster_layer("aguada").expect("a aguada");
    t.set_paint_media(PaintMedia::Watercolor);
    t.set_brush_size_px(14.0);
    tracos(&mut t, cor, corpo, dois);
    imagem(&mut t)
}

/// O que o produto mostra: um desenho novo, os traços, e DEPOIS o papel de cor pelo seletor.
fn no_papel(t: &mut PainterTool, cor: [u8; 3], corpo: f32, dois: bool) -> Vec<u8> {
    t.set_paint_media(PaintMedia::Watercolor);
    t.set_brush_size_px(14.0);
    tracos(t, cor, corpo, dois);
    escolhe_o_papel(t, PAPEL);
    imagem(t)
}

/// ⭐ **A AGUADA É UM FILTRO SOBRE O PAPEL DE COR** — nas quatro cores, nos três corpos (0, o de fábrica
/// 0,4, e 1) e com um e dois traços sobrepostos, o que se mostra sobre o papel castanho é `R + papel·T`
/// do oráculo de dois chãos, até ao [`PISO`]; e o vermelho PURO da foto do dono, com corpo 0, nenhum canal
/// acima do papel (a óptica devolve `pigmento·(1 − T)`, então só um pigmento de canais 0/255 é filtro
/// puro: um azul `30,60,220` sobe o azul do papel pelo que o próprio pigmento devolve, e o oráculo diz o
/// mesmo). Vermelho antes (a ablação `sem_vidro`, a lei de um alfa): o vermelho puro de corpo 0
/// mostrava `221,111,101` — `+68` acima do papel — contra `153,74,55`.
#[test]
fn a_aguada_e_um_filtro_sobre_o_papel() {
    for cor in CORES {
        for corpo in CORPOS {
            for dois in [false, true] {
                let w = sobre_chao([255, 255, 255], cor, corpo, dois);
                let r = sobre_chao([0, 0, 0], cor, corpo, dois);
                let mut t = novo(PaintMedia::Watercolor, 14.0);
                let a = no_papel(&mut t, cor, corpo, dois);
                let (mut pior, mut acima) = (0i32, 0i32);
                let mut onde = (0usize, 0usize, 0i32);
                for k in 0..LADO * LADO {
                    for c in 0..3 {
                        let (rv, wv) = (f32::from(r[k * 4 + c]), f32::from(w[k * 4 + c]));
                        let o = (rv + f32::from(PAPEL[c]) / 255.0 * (wv - rv)).round() as i32;
                        let d = (i32::from(a[k * 4 + c]) - o).abs();
                        if d > pior {
                            onde = (k, c, o);
                        }
                        pior = pior.max(d);
                        if t.canvas_rgba[k * 4 + 3] > 0 {
                            acima = acima.max(i32::from(a[k * 4 + c]) - i32::from(PAPEL[c]));
                        }
                    }
                }
                assert!(
                    w.chunks(4).any(|p| p[..3] != [255, 255, 255]),
                    "controlo: {cor:?} pintou"
                );
                assert!(
                    pior <= PISO,
                    "{cor:?} corpo {corpo} dois {dois}: o papel não atravessa a aguada como filtro — \
                     pior {pior} níveis contra R + papel·T (texel {} canal {}: mostrado {:?} · oráculo {} · \
                     branco {:?} · preto {:?} · camada {:?})",
                    onde.0,
                    onde.1,
                    &a[onde.0 * 4..onde.0 * 4 + 3],
                    onde.2,
                    &w[onde.0 * 4..onde.0 * 4 + 3],
                    &r[onde.0 * 4..onde.0 * 4 + 3],
                    &t.canvas_rgba[onde.0 * 4..onde.0 * 4 + 4],
                );
                if corpo == 0.0 && cor == [255, 0, 0] && !dois {
                    assert!(
                        acima <= 1,
                        "{cor:?} dois {dois}: a aguada transparente ACENDE o papel em {acima} níveis"
                    );
                }
            }
        }
    }
}

/// **O SELO DEVOLVE A TINTA DE UM ALFA** — um traço Digital opaco por cima da aguada, na mesma camada:
/// ali o papel castanho não se vê (a tinta opaca cobre), e o texel mostra a cor do Digital.
#[test]
fn o_selo_devolve_a_tinta_de_um_alfa() {
    let mut t = novo(PaintMedia::Watercolor, 14.0);
    t.paint.brush.opacity = 0.0;
    t.set_brush_color_srgb8([255, 0, 0]);
    pinta_com_a_cor(&mut t, 1.0, &(GESTOS[2].1)());
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(10.0);
    t.paint.brush.hardness = 1.0;
    t.set_brush_color_srgb8([20, 200, 40]);
    pinta_com_a_cor(&mut t, 1.0, &(GESTOS[1].1)());
    escolhe_o_papel(&mut t, PAPEL);
    let a = imagem(&mut t);
    let mut opacos = 0;
    for k in 0..LADO * LADO {
        if t.canvas_rgba[k * 4..k * 4 + 4] == [20, 200, 40, 255] {
            opacos += 1;
            assert_eq!(
                a[k * 4..k * 4 + 3],
                [20, 200, 40],
                "texel {k}: o papel vazou pela tinta opaca do Digital por cima da aguada"
            );
        }
    }
    assert!(
        opacos > 100,
        "controlo: o Digital cobriu a aguada ({opacos} texels)"
    );
}

/// **O VIDRO VOLTA COM O DESFAZER, A TROCA DE SPRITE E O FICHEIRO** — uma 2.ª aguada sobre a 1.ª e o
/// desfazer dela; trocar de sprite e voltar; gravar e reabrir: o papel castanho mostra sempre a mesma
/// imagem, ao byte.
#[test]
fn o_vidro_volta_com_o_desfazer_a_troca_e_o_ficheiro() {
    let branco = || vec![255u8; LADO * LADO * 4];
    let mut t = novo(PaintMedia::Watercolor, 14.0);
    t.set_brush_color_srgb8([255, 0, 0]);
    pinta_com_a_cor(&mut t, 1.0, &(GESTOS[2].1)());
    escolhe_o_papel(&mut t, PAPEL);
    let antes = imagem(&mut t);
    t.set_brush_color_srgb8([30, 60, 220]);
    pinta_com_a_cor(&mut t, 1.0, &(GESTOS[1].1)());
    assert_ne!(imagem(&mut t), antes, "controlo: a 2.ª aguada pintou");
    assert!(t.undo_last(), "há o que desfazer");
    assert_eq!(
        imagem(&mut t),
        antes,
        "o desfazer não devolveu o vidro da 1.ª aguada"
    );
    t.bind_document(2, branco(), LADO as u32, LADO as u32);
    t.bind_document(1, branco(), LADO as u32, LADO as u32);
    assert_eq!(imagem(&mut t), antes, "a troca de sprite perdeu o vidro");
    let docs = t.collect_documents(&[(1u64, 7u32)].into_iter().collect());
    let mut outro = PainterTool::default();
    outro.install_document(1, docs.into_iter().next().expect("o documento"));
    outro.bind_document(1, branco(), LADO as u32, LADO as u32);
    assert_eq!(imagem(&mut outro), antes, "o ficheiro perdeu o vidro");
}

/// SONDA — **a aguada vermelha no papel castanho parece fluorescente** (smoke do dono, 2026-10-07, com
/// foto: miolo `230,95,85` sobre o papel `153,121,91`). Por meio, o texel do miolo: o que a camada
/// guarda (cor, alfa), o que se mostra no branco e no castanho, e — nos texels pintados — quantos se
/// mostram com o VERMELHO acima do papel por mais de 20 (luz que um filtro não tem) e a média desse
/// excesso. A referência física é o filtro: o que se mostra no branco multiplicado pelo papel.
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_a_aguada_vermelha_no_castanho -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico"]
fn diag_a_aguada_vermelha_no_castanho() {
    const PAPEL: [u8; 3] = [153, 121, 91];
    for cor in [[255u8, 0, 0], [220, 40, 40]] {
        for (meio, s) in [
            (PaintMedia::Watercolor, 1.0f32),
            (PaintMedia::Digital, 0.5),
            (PaintMedia::WetPaint, 1.0),
        ] {
            let mut t = novo(meio, 14.0);
            t.set_wet_relogio_fixo(true);
            t.set_brush_color_srgb8(cor);
            t.paint.brush.strength = s;
            let pts = (GESTOS[0].1)();
            t.on_canvas_pointer(cp(pts[0], PointerPhase::Down));
            for p in &pts[1..] {
                t.on_canvas_pointer(cp(*p, PointerPhase::Move));
                t.paint_tick(1.0 / 60.0);
            }
            t.on_canvas_pointer(cp(*pts.last().expect("pontos"), PointerPhase::Up));
            for _ in 0..30 {
                t.paint_tick(1.0 / 60.0);
            }
            let branco = imagem(&mut t);
            escolhe_o_papel(&mut t, PAPEL);
            let castanho = imagem(&mut t);
            let i = (64 * LADO + 64) * 4;
            let (mut n, mut acima, mut excesso) = (0usize, 0usize, 0.0f32);
            for k in 0..LADO * LADO {
                if t.canvas_rgba[k * 4 + 3] < 8 {
                    continue;
                }
                n += 1;
                let d = f32::from(castanho[k * 4]) - f32::from(PAPEL[0]);
                if d > 20.0 {
                    acima += 1;
                    excesso += d;
                }
            }
            let filtro: Vec<u8> = (0..3)
                .map(|c| (u32::from(branco[i + c]) * u32::from(PAPEL[c]) / 255) as u8)
                .collect();
            eprintln!(
                "[fluor] cor {cor:?} {meio:<10?} s {s}: camada {:?} · no branco {:?} · no castanho {:?} · filtro {filtro:?} · R acima do papel +20: {acima}/{n} (média +{:.0})",
                &t.canvas_rgba[i..i + 4],
                &branco[i..i + 3],
                &castanho[i..i + 3],
                excesso / acima.max(1) as f32
            );
        }
    }
}

/// SONDA — **a aguada como VIDRADO** (doc 48 §4, antes de construir): a óptica de hoje sobre um chão
/// BRANCO e sobre um chão PRETO (uma camada de baixo opaca; a aguada numa camada transparente por
/// cima), o par da referência de Kubelka–Munk (Curtis 1997 especifica o pigmento assim). Em tons de
/// ecrã: `R = o mostrado no preto` (o que o pigmento devolve) e `T = branco − preto` (o que passa do
/// chão). Nos texels pintados: quantos têm `T < 0` em algum canal (a lei «R + chão·T» não os
/// descreve), quantos não cabem em `cor·alfa + chão·filtro·(1 − alfa)` (`máx R > 1 − máx T`), e na
/// cena do dono (papel `153,121,91`) o miolo previsto `R + papel·T` contra o de hoje, e o pior
/// excesso de um canal sobre o papel.
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_a_aguada_como_vidrado -- --ignored --nocapture`
#[test]
#[ignore = "diagnóstico"]
fn diag_a_aguada_como_vidrado() {
    const PAPEL: [u8; 3] = [153, 121, 91];
    let pinta_sobre = |chao: [u8; 3], cor: [u8; 3], corpo: f32| {
        let mut t = PainterTool::default();
        t.bind_document(1, vec![255u8; LADO * LADO * 4], LADO as u32, LADO as u32);
        let fundo: Vec<u8> = (0..LADO * LADO)
            .flat_map(|_| [chao[0], chao[1], chao[2], 255])
            .collect();
        t.add_raster_layer_with_pixels("chão", fundo)
            .expect("o chão");
        t.add_raster_layer("aguada").expect("a aguada");
        t.set_paint_media(PaintMedia::Watercolor);
        t.set_brush_size_px(14.0);
        t.paint.brush.opacity = corpo;
        t.set_brush_color_srgb8(cor);
        pinta_com_a_cor(&mut t, 1.0, &(GESTOS[2].1)());
        (imagem(&mut t), t.canvas_rgba.to_vec())
    };
    let hoje_t = |cor: [u8; 3], corpo: f32| {
        let mut t = novo(PaintMedia::Watercolor, 14.0);
        t.paint.brush.opacity = corpo;
        t.set_brush_color_srgb8(cor);
        pinta_com_a_cor(&mut t, 1.0, &(GESTOS[2].1)());
        escolhe_o_papel(&mut t, PAPEL);
        imagem(&mut t)
    };
    eprintln!(
        "\n[vidrado] cor            corpo  texels  T<0  não cabe  miolo hoje        previsto (R+papel·T)  pior canal acima do papel: hoje · vidrado"
    );
    for cor in [[255u8, 0, 0], [220, 40, 40], [30, 60, 220], [240, 200, 40]] {
        for corpo in [0.0f32, 0.4, 1.0] {
            let (w, camada) = pinta_sobre([255, 255, 255], cor, corpo);
            let (b, _) = pinta_sobre([0, 0, 0], cor, corpo);
            let hoje = hoje_t(cor, corpo);
            let (mut n, mut negativo, mut nao_cabe) = (0usize, 0usize, 0usize);
            let (mut pior_hoje, mut pior_vidrado) = (0i32, 0i32);
            let mut miolo = None;
            let (mut erro, mut f_max) = ([0.0f32; 3], 0.0f32);
            for k in 0..LADO * LADO {
                if camada[k * 4 + 3] < 8 {
                    continue;
                }
                n += 1;
                let r: Vec<f32> = (0..3).map(|c| f32::from(b[k * 4 + c]) / 255.0).collect();
                let tt: Vec<f32> = (0..3)
                    .map(|c| (f32::from(w[k * 4 + c]) - f32::from(b[k * 4 + c])) / 255.0)
                    .collect();
                negativo += usize::from(tt.iter().any(|&v| v < -0.5 / 255.0));
                let max_r = r.iter().copied().fold(0.0f32, f32::max);
                let max_t = tt.iter().copied().fold(0.0f32, f32::max);
                nao_cabe += usize::from(max_r > 1.0 - max_t + 1.0 / 255.0);
                // O erro no BRANCO de cada encaixe: alfa = 1 − máx T (a cor corta) · alfa = máx R (o
                // filtro corta) · o filtro em `u8` sobre [0, 2] (alfa = máx R, filtro até 2).
                let a1 = 1.0 - max_t;
                let e1 = (0..3)
                    .map(|c| r[c] - r[c].min(a1.max(0.0)))
                    .fold(0.0f32, f32::max);
                let e2 = (0..3)
                    .map(|c| tt[c] - tt[c].min(1.0 - max_r))
                    .fold(0.0f32, f32::max);
                let e3 = (0..3)
                    .map(|c| {
                        let f = (tt[c] / (1.0 - max_r).max(1e-6)).clamp(0.0, 2.0);
                        let fq = (f * 127.5).round() / 127.5;
                        (fq * (1.0 - max_r) - tt[c]).abs()
                    })
                    .fold(0.0f32, f32::max);
                erro[0] = erro[0].max(e1 * 255.0);
                erro[1] = erro[1].max(e2 * 255.0);
                erro[2] = erro[2].max(e3 * 255.0);
                f_max = f_max.max(
                    (0..3)
                        .map(|c| tt[c] / (1.0 - max_r).max(1e-6))
                        .fold(0.0, f32::max),
                );
                let previsto: Vec<i32> = (0..3)
                    .map(|c| ((r[c] + f32::from(PAPEL[c]) / 255.0 * tt[c]) * 255.0).round() as i32)
                    .collect();
                for c in 0..3 {
                    pior_hoje = pior_hoje.max(i32::from(hoje[k * 4 + c]) - i32::from(PAPEL[c]));
                    pior_vidrado = pior_vidrado.max(previsto[c] - i32::from(PAPEL[c]));
                }
                if k == 98 * LADO + 64 {
                    miolo = Some((hoje[k * 4..k * 4 + 3].to_vec(), previsto));
                }
            }
            eprintln!(
                "[vidrado] {cor:<14?} {corpo:<5}  {n:>6}  {negativo:>4}  {nao_cabe:>8}  {:?}  pior: {pior_hoje:>4} · {pior_vidrado:>4}  | erro no branco (níveis): cor corta {:.1} · filtro corta {:.1} · filtro u8 [0,2] {:.1} · filtro máx {f_max:.2}",
                miolo, erro[0], erro[1], erro[2]
            );
        }
    }
}

/// **O VIDRO ATRAVESSA OS GRUPOS E VAI COM A CÓPIA DA CAMADA** — a aguada numa camada própria, sobre o
/// papel castanho: metê-la num grupo (modo Normal) não muda a imagem, e duplicar a camada e apagar a
/// original também não (a cópia leva o vidro dela).
#[test]
fn o_vidro_atravessa_grupos_e_copias() {
    let mut t = novo(PaintMedia::Watercolor, 14.0);
    let aguada = t.add_raster_layer("aguada").expect("a camada da aguada");
    t.set_paint_media(PaintMedia::Watercolor);
    t.set_brush_size_px(14.0);
    t.paint.brush.opacity = 0.0;
    t.set_brush_color_srgb8([255, 0, 0]);
    pinta_com_a_cor(&mut t, 1.0, &(GESTOS[2].1)());
    escolhe_o_papel(&mut t, PAPEL);
    let antes = imagem(&mut t);
    assert!(t.papel_atravessa_vidro(), "controlo: a aguada deixou vidro");
    let copia = t.duplicate_layer(aguada).expect("a cópia");
    assert!(t.delete_layer(aguada), "a original sai");
    assert_eq!(imagem(&mut t), antes, "a cópia da camada não levou o vidro");
    t.select_layer(copia);
    t.group_active().expect("o grupo");
    assert_eq!(
        imagem(&mut t),
        antes,
        "o grupo não deixa o papel atravessar canal a canal"
    );
}

/// **A PRÉ-VISUALIZAÇÃO QUE SE MOVE DEVOLVE O VIDRO DA BASE** — um ponto (Drag Dot) premido sobre
/// uma aguada seca e arrastado para longe: a posição velha volta à base, e o vidro dela volta com ela
/// (o quadro anterior tinha selado ali o ponto). Sobre o papel castanho, a zona da posição velha é a
/// da aguada sozinha, ao byte.
#[test]
fn a_previsualizacao_que_se_move_devolve_o_vidro_da_base() {
    let aguada = || {
        let mut t = novo(PaintMedia::Watercolor, 14.0);
        t.paint.brush.opacity = 0.0;
        t.set_brush_color_srgb8([255, 0, 0]);
        pinta_com_a_cor(&mut t, 1.0, &(GESTOS[0].1)());
        t.dry_session_now();
        t
    };
    let mut so = aguada();
    escolhe_o_papel(&mut so, PAPEL);
    let alvo = imagem(&mut so);
    let mut t = aguada();
    t.paint.brush.stroke_method = ph2d_painter_brush::StrokeMethod::DragDot;
    t.paint.brush_by_mode.fill(t.paint.brush);
    t.set_brush_size_px(6.0);
    t.set_brush_color_srgb8([30, 60, 220]);
    t.on_canvas_pointer(cp([64.0, 64.5], PointerPhase::Down));
    t.paint_tick(1.0 / 60.0);
    t.on_canvas_pointer(cp([20.0, 100.0], PointerPhase::Move));
    t.paint_tick(1.0 / 60.0);
    t.on_canvas_pointer(cp([20.0, 100.0], PointerPhase::Up));
    escolhe_o_papel(&mut t, PAPEL);
    let a = imagem(&mut t);
    for y in 54..76 {
        for x in 54..76 {
            let k = (y * LADO + x) * 4;
            assert_eq!(
                a[k..k + 3],
                alvo[k..k + 3],
                "({x}, {y}): a posição velha do ponto não devolveu o vidro da aguada"
            );
        }
    }
}

/// SONDA — **o preço do vidro**, por movimento (o evento, o tique e a drenagem do preview), num desenho
/// novo sobre o papel castanho: a Aquarela (o núcleo avalia a óptica duas vezes e escreve o plano) e o
/// Digital por cima de uma aguada (cada quadro compõe pela passada com a transparência por canal), a
/// 2048² e 4096², com o vidro e com a ablação `sem_vidro` (a lei de um alfa), no MESMO processo, três
/// rodadas intercaladas: o mínimo das medianas, a mediana das medianas ao lado.
/// `cargo test -p ph2d-tool-painter --profile smoke --lib diag_o_preco_do_vidro -- --ignored --nocapture --test-threads=1`
#[test]
#[ignore = "medição"]
fn diag_o_preco_do_vidro() {
    fn por_move(meio: PaintMedia, lado: u32, vidro: bool) -> (f64, f64, f64, f64) {
        let mut t = PainterTool {
            sem_vidro: !vidro,
            ..Default::default()
        };
        t.bind_document(1, vec![255u8; (lado * lado * 4) as usize], lado, lado);
        t.set_wet_relogio_fixo(true);
        t.set_brush_size_px(40.0);
        let mid = (lado / 2) as f32;
        // Uma aguada por baixo (o Digital pinta por cima dela) e o papel castanho.
        t.set_paint_media(PaintMedia::Watercolor);
        t.on_canvas_pointer(cp([60.0, mid], PointerPhase::Down));
        t.on_canvas_pointer(cp([600.0, mid], PointerPhase::Move));
        t.on_canvas_pointer(cp([600.0, mid], PointerPhase::Up));
        let _ = t.take_preview_arc();
        let t0 = std::time::Instant::now();
        escolhe_o_papel(&mut t, PAPEL);
        let _ = t.take_preview_arc();
        let quadro_do_papel = t0.elapsed().as_secs_f64() * 1e3;
        assert_eq!(t.papel_atravessa_vidro(), vidro, "controlo: o vidro");
        t.set_paint_media(meio);
        let t0 = std::time::Instant::now();
        t.on_canvas_pointer(cp([80.0, mid + 10.0], PointerPhase::Down));
        let _ = t.take_preview_arc();
        let toque = t0.elapsed().as_secs_f64() * 1e3;
        let mut ms = Vec::new();
        for k in 1..=12u8 {
            let x = 80.0 + 30.0 * f32::from(k);
            let t0 = std::time::Instant::now();
            t.on_canvas_pointer(cp([x, mid + 10.0], PointerPhase::Move));
            t.paint_tick(1.0 / 60.0);
            let _ = t.take_preview_arc();
            ms.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        let t0 = std::time::Instant::now();
        t.on_canvas_pointer(cp([500.0, mid + 10.0], PointerPhase::Up));
        let largar = t0.elapsed().as_secs_f64() * 1e3;
        let primeiro = toque + ms[0];
        ms.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
        (ms[ms.len() / 2], primeiro, quadro_do_papel, largar)
    }
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!("\n[vidro] loadavg {}", carga().trim());
    let casos: Vec<(PaintMedia, u32)> = [PaintMedia::Watercolor, PaintMedia::Digital]
        .iter()
        .flat_map(|&m| [(m, 2048u32), (m, 4096)])
        .collect();
    let (mut sem, mut com) = (vec![Vec::new(); casos.len()], vec![Vec::new(); casos.len()]);
    let (mut sem1, mut com1) = (vec![Vec::new(); casos.len()], vec![Vec::new(); casos.len()]);
    let (mut semp, mut comp) = (vec![Vec::new(); casos.len()], vec![Vec::new(); casos.len()]);
    let (mut seml, mut coml) = (vec![Vec::new(); casos.len()], vec![Vec::new(); casos.len()]);
    for _ in 0..3 {
        for (k, &(meio, lado)) in casos.iter().enumerate() {
            let (s, s1, sp, sl) = por_move(meio, lado, false);
            let (c, c1, cpp, cl) = por_move(meio, lado, true);
            seml[k].push(sl);
            coml[k].push(cl);
            semp[k].push(sp);
            comp[k].push(cpp);
            sem[k].push(s);
            com[k].push(c);
            sem1[k].push(s1);
            com1[k].push(c1);
        }
    }
    eprintln!(
        "[vidro] meio        lado   sem vidro ms (mín · med)   com vidro ms (mín · med)   com/sem"
    );
    for (k, &(meio, lado)) in casos.iter().enumerate() {
        let mm = |v: &mut Vec<f64>| {
            v.sort_by(|a, b| a.partial_cmp(b).expect("finite"));
            (v[0], v[v.len() / 2])
        };
        let (sm, sd) = mm(&mut sem[k]);
        let (cm, cd) = mm(&mut com[k]);
        let (s1, _) = mm(&mut sem1[k]);
        let (c1, _) = mm(&mut com1[k]);
        let (sp, _) = mm(&mut semp[k]);
        let (cpp, _) = mm(&mut comp[k]);
        let (sl, _) = mm(&mut seml[k]);
        let (cl, _) = mm(&mut coml[k]);
        eprintln!(
            "[vidro] {meio:<11?} {lado:<6} {sm:>7.3} · {sd:>7.3}            {cm:>7.3} · {cd:>7.3}            {:>5.2}×   toque + 1.º movimento {s1:.2} → {c1:.2} ms · quadro da troca de papel {sp:.1} → {cpp:.1} ms · largar {sl:.2} → {cl:.2} ms",
            cm / sm.max(1e-6)
        );
    }
    eprintln!("[vidro] loadavg {}", carga().trim());
}

/// **A SELEÇÃO TAMBÉM GUARDA O VIDRO** (auditoria do fecho): uma aguada que atravessa a borda de uma
/// seleção — fora dela o texel volta à base, e o vidro com ele. No papel castanho, a metade de fora da
/// seleção mostra só o papel (e a aguada pinta a de dentro).
#[test]
fn a_selecao_tambem_guarda_o_vidro() {
    let mut t = novo(PaintMedia::Watercolor, 14.0);
    t.set_rect_selection(0, 0, 64, LADO as u32);
    t.set_brush_color_srgb8([255, 0, 0]);
    pinta_com_a_cor(&mut t, 1.0, &(GESTOS[0].1)());
    escolhe_o_papel(&mut t, PAPEL);
    let a = imagem(&mut t);
    let fora = (0..LADO)
        .flat_map(|y| (66..LADO).map(move |x| (y * LADO + x) * 4))
        .map(|k| {
            (0..3)
                .map(|c| a[k + c].abs_diff(PAPEL[c]))
                .max()
                .unwrap_or(0)
        })
        .max()
        .unwrap_or(0);
    assert!(fora <= 1, "fora da seleção o papel escureceu {fora} níveis");
    let dentro = (0..LADO)
        .flat_map(|y| (20..60).map(move |x| (y * LADO + x) * 4))
        .any(|k| a[k..k + 3] != PAPEL);
    assert!(dentro, "controlo: dentro da seleção a aguada pintou");
}

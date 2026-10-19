//! **O artista escreveu aqui mesmo sem mudar o byte** (smoke do dono, 19/10: faixas brancas no Play onde a
//! pincelada nova cruza o miolo de uma gravada). Preto sobre o preto que a gravação deixou no fim não muda o
//! píxel; a régua «o que mudou» deixava esse texel sem marca e o Play mostrava ali a gravação daquele
//! instante. W10 (`docs/Macros/08_plano_W10_cobertura.md`): no Digital a marca é a FORÇA que o carimbo
//! mediu em cada texel — a régua é o byte da mesma pincelada sobre o branco (preto por cima:
//! `byte = 255·(1 − força)`); nos meios sem força medida (a aquarela) fica a lei da W9, o miolo ≥ ½.

use super::marca::MarcaDoArtista;
use crate::PaintMedia;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool};

const LADO: u32 = 128;
const L: usize = LADO as usize;
/// A faixa preta que a gravação deixou no fim (linhas).
const FAIXA: std::ops::Range<usize> = 60..68;

fn i(x: usize, y: usize) -> usize {
    y * L + x
}

/// A tela branca (com a faixa preta opaca se `faixa`), o Painter em `meio` a pincel preto de raio 16
/// acertado por `pincel`, a marca posta.
fn tela(meio: PaintMedia, faixa: bool, pincel: fn(&mut PainterTool)) -> PainterTool {
    let mut src = vec![255u8; L * L * 4];
    if faixa {
        for y in FAIXA {
            for x in 0..L {
                src[i(x, y) * 4..i(x, y) * 4 + 3].fill(0);
            }
        }
    }
    let mut t = PainterTool::default();
    t.set_source(src, LADO, LADO);
    t.set_paint_media(meio);
    t.poe_a_marca(Some(MarcaDoArtista::nova(LADO, LADO, 1)));
    t.set_brush_size_px(16.0);
    t.set_brush_color_srgb8([0, 0, 0]);
    pincel(&mut t);
    t
}

fn de_fabrica(_: &mut PainterTool) {}

/// Continuous Spacing com os dabs a um diâmetro (o máximo): o meio entre dois dabs só tem miolo pela
/// cápsula.
fn esparso_continuo(t: &mut PainterTool) {
    t.toggle_brush_continuous_spacing();
    t.set_brush_spacing(1.0);
}

/// Uma pincelada vertical em `x`, de `y0` a `y1`, um quadro por evento.
fn vertical(t: &mut PainterTool, x: f32, (y0, y1): (f32, f32)) {
    let n = 12;
    for k in 0..=n {
        #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
        let y = y0 + (y1 - y0) * k as f32 / n as f32;
        let phase = match k {
            0 => PointerPhase::Down,
            k if k == n => PointerPhase::Up,
            _ => PointerPhase::Move,
        };
        t.on_canvas_pointer(CanvasPointer {
            pos: [x, y],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        drop(t.take_preview_arc());
        t.on_tick(16.667);
    }
}

const DE_CIMA_A_BAIXO: (f32, f32) = (20.0, 110.0);

fn imagem(t: &PainterTool) -> Vec<u8> {
    t.imagem_aplicada().expect("documento").0
}

fn px(v: &[u8], k: usize) -> Vec<u8> {
    v[k * 4..k * 4 + 4].to_vec()
}

/// A régua: a pincelada Digital de `pincel` sobre o branco (linha 40) — a força dela em cada coluna lê-se
/// no byte (preto sobre branco, composto no espaço do píxel: força ≥ ½ ⇔ byte ≤ 127). `true` = miolo
/// (≤ 120), `false` = borda fraca (135..255), `None` = perto do limiar ou fora do pincel.
fn regua(pincel: fn(&mut PainterTool)) -> (Vec<Option<bool>>, Vec<u8>) {
    let mut t = tela(PaintMedia::Digital, true, pincel);
    vertical(&mut t, 64.0, DE_CIMA_A_BAIXO);
    let depois = imagem(&t);
    let r: Vec<Option<bool>> = (0..L)
        .map(|x| match depois[i(x, 40) * 4] {
            v if v <= 120 => Some(true),
            135..=254 => Some(false),
            _ => None,
        })
        .collect();
    let miolo = r.iter().filter(|c| **c == Some(true)).count();
    let borda = r.iter().filter(|c| **c == Some(false)).count();
    assert!(
        miolo > 4 && borda > 4,
        "controlo: a régua tem miolo ({miolo}) e borda ({borda})"
    );
    (r, (0..L).map(|x| depois[i(x, 40) * 4]).collect())
}

/// O piso do arredondamento de cada dab ao byte numa pincelada macia (medido 19/10 na régua dos
/// cruzamentos, `palcos_cobertura_tests`: 2 a 3 níveis), em níveis.
const PISO: f32 = 3.0;

/// A cobertura do texel `k` em níveis de 255.
fn niveis(m: &MarcaDoArtista, k: usize) -> f32 {
    255.0 * f32::from(m.cobertura(k)) / f32::from(super::marca::CHEIA)
}

/// A pincelada de `pincel` em `meio` por cima da faixa. Digital: em TODAS as linhas da faixa a cobertura
/// é a força que a régua mostra na coluna (± o piso). Aquarela (a lei da W9): o miolo marcado, a borda
/// fraca não.
fn confere(meio: PaintMedia, pincel: fn(&mut PainterTool)) {
    let (r, r_bytes) = regua(pincel);
    let mut t = tela(meio, true, pincel);
    let antes = imagem(&t);
    vertical(&mut t, 64.0, DE_CIMA_A_BAIXO);
    let depois = imagem(&t);
    assert_ne!(
        px(&antes, i(64, 40)),
        px(&depois, i(64, 40)),
        "controlo {meio:?}: pintou o branco"
    );
    let m = t.marca_do_artista().expect("a marca").clone();
    for (x, c) in r.iter().enumerate() {
        for y in FAIXA {
            let k = i(x, y);
            assert_eq!(
                px(&antes, k),
                px(&depois, k),
                "controlo {meio:?}: preto sobre o preto ({x}, {y})"
            );
            if meio == PaintMedia::Digital {
                let forca = 255.0 - f32::from(r_bytes[x]);
                assert!(
                    (niveis(&m, k) - forca).abs() <= PISO,
                    "{meio:?}: a cobertura de ({x}, {y}) é {:.1} níveis e a força que o pincel pôs ali {forca}",
                    niveis(&m, k)
                );
                continue;
            }
            match c {
                Some(true) => assert!(
                    m.marcado(k),
                    "{meio:?}: o miolo sobre a faixa ({x}, {y}) não ficou do artista — o Play mostra ali a gravação"
                ),
                Some(false) => assert!(
                    !m.marcado(k),
                    "{meio:?}: a borda fraca ({x}, {y}) marcou — o Play mostraria ali o fim antes do tempo"
                ),
                None => {}
            }
        }
    }
}

/// ⭐ Digital e Watercolor (os meios que só marcavam o que mudou; medido 19/10: 136 de 136 texels do
/// miolo sobre a faixa sem marca nos dois — Impasto muda o relevo e o Wet Paint marca o disco inteiro).
#[test]
fn pintar_por_cima_do_que_ja_e_o_resultado_marca_o_miolo_do_pincel() {
    confere(PaintMedia::Digital, de_fabrica);
    confere(PaintMedia::Watercolor, de_fabrica);
}

/// Dabs afastados com Continuous Spacing: o carimbo enche o meio pela cápsula, e a força também — sem os
/// sub-dabs a faixa entre dois dabs voltava a abrir.
#[test]
fn o_miolo_entre_dabs_afastados_e_o_da_capsula() {
    confere(PaintMedia::Digital, esparso_continuo);
}

/// ⭐ **A cobertura de cada texel é a força que o pincel pôs ali** (W10): no branco, preto por cima, a
/// régua é o byte (`255·(1 − força)`) — pincel de fábrica, ponta ACHATADA (a elipse: o perfil redondo da
/// W9 mentia aqui) e com grão (o carimbo mede-o; a W9 recortava-o). Tudo o que mudou está marcado, e longe
/// do traço nada. ⛔ VERMELHO com a marca binária: cada texel mudado ficava com a imagem atual inteira.
#[test]
fn a_cobertura_e_a_forca_que_o_pincel_pos_no_texel() {
    let casos: [(&str, fn(&mut PainterTool)); 3] = [
        ("de fábrica", de_fabrica),
        ("achatada", |t| t.set_brush_dab_flatten(0.7)),
        ("com grão", |t| t.new_brush_texture()),
    ];
    for (nome, pincel) in casos {
        let mut t = tela(PaintMedia::Digital, false, pincel);
        let antes = imagem(&t);
        vertical(&mut t, 64.0, DE_CIMA_A_BAIXO);
        let depois = imagem(&t);
        let m = t.marca_do_artista().expect("a marca").clone();
        let mudou: Vec<usize> = (0..L * L)
            .filter(|&k| px(&antes, k) != px(&depois, k))
            .collect();
        assert!(mudou.len() > 100, "controlo {nome}: a pincelada pintou");
        assert!(
            mudou.iter().all(|&k| m.marcado(k)),
            "{nome}: um texel pintado sem marca"
        );
        let parciais = mudou
            .iter()
            .filter(|&&k| (1..254).contains(&depois[k * 4]))
            .count();
        assert!(
            parciais > 20,
            "controlo {nome}: a borda tem forças parciais ({parciais})"
        );
        let (mut pior, mut onde) = (0.0f32, 0);
        for k in 0..L * L {
            let d = (niveis(&m, k) - (255.0 - f32::from(depois[k * 4]))).abs();
            if d > pior {
                (pior, onde) = (d, k);
            }
        }
        assert!(
            pior <= PISO,
            "{nome}: a cobertura de ({}, {}) difere {pior:.1} níveis da força que o pincel pôs",
            onde % L,
            onde / L
        );
        assert!(!m.marcado(i(5, 5)), "{nome}: longe do traço marcou");
    }
}

/// A força é do PASSO: desfeita a pincelada por cima da faixa, a seguinte (noutro sítio) não a herda.
#[test]
fn a_forca_de_um_passo_desfeito_nao_passa_ao_seguinte() {
    let mut t = tela(PaintMedia::Digital, true, de_fabrica);
    vertical(&mut t, 64.0, DE_CIMA_A_BAIXO);
    let m = t.marca_do_artista().expect("a marca").clone();
    assert!(
        m.marcado(i(64, 64)),
        "controlo: o miolo sobre a faixa marcou"
    );
    assert!(t.undo_last(), "o desfazer");
    vertical(&mut t, 20.0, (5.0, 40.0));
    let m = t.marca_do_artista().expect("a marca").clone();
    assert!(m.marcado(i(20, 20)), "controlo: a 2.ª pincelada marcou");
    for y in FAIXA {
        assert!(
            !m.marcado(i(64, y)),
            "o miolo da pincelada desfeita voltou com a seguinte ({y})"
        );
    }
}

/// Os modos que dão o que estava por baixo (Blur, Smear) não depõem tinta: numa tela toda preta não
/// mudam nada e não marcam nada. (A máscara: `palcos_fim_tests::o_filme_da_mascara_…`, no app.)
#[test]
fn quem_nao_depoe_tinta_nao_marca_o_miolo() {
    for modo in ["blur", "smear"] {
        let mut t = PainterTool::default();
        let mut preta = vec![0u8; L * L * 4];
        preta
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .for_each(|p| p[3] = 255);
        t.set_source(preta, LADO, LADO);
        t.poe_a_marca(Some(MarcaDoArtista::nova(LADO, LADO, 1)));
        t.set_brush_size_px(16.0);
        t.handle_panel_event(ph2d_editor_core::tool::PanelEvent::SelectOption(
            ph2d_editor_core::ids::PAINTER_PAINT_MODE,
            modo.into(),
        ));
        let antes = imagem(&t);
        vertical(&mut t, 64.0, DE_CIMA_A_BAIXO);
        assert_eq!(
            antes,
            imagem(&t),
            "controlo {modo}: preto sobre preto não muda"
        );
        let m = t.marca_do_artista().expect("a marca").clone();
        assert_eq!(m.quantos(), 0, "{modo}: não depõe tinta e marcou");
    }
}

/// ⭐ **A cobertura e o que estava por baixo vão e voltam com o desfazer** (W10): duas pinceladas macias
/// que se sobrepõem — a 2.ª compõe a cobertura por cima da 1.ª e não troca o que estava por baixo (o
/// 1.º toque); desfazer devolve a marca da 1.ª e depois nenhuma; refazer devolve as duas.
#[test]
fn a_cobertura_e_o_por_baixo_vao_e_voltam_com_o_desfazer() {
    let mut t = tela(PaintMedia::Digital, true, de_fabrica);
    let fundo = imagem(&t);
    vertical(&mut t, 60.0, DE_CIMA_A_BAIXO);
    let m1 = t.marca_do_artista().expect("a marca").clone();
    let parciais: Vec<usize> = (0..L * L)
        .filter(|&k| (1..super::marca::CHEIA).contains(&m1.cobertura(k)))
        .collect();
    assert!(
        parciais.len() > 100,
        "controlo: a borda macia tem cobertura parcial"
    );
    assert!(
        parciais
            .iter()
            .all(|&k| m1.por_baixo(k)[..] == fundo[k * 4..k * 4 + 4]),
        "o que estava por baixo não é a imagem antes da pincelada"
    );
    vertical(&mut t, 66.0, DE_CIMA_A_BAIXO);
    let m2 = t.marca_do_artista().expect("a marca").clone();
    let juntos: Vec<usize> = parciais
        .iter()
        .copied()
        .filter(|&k| m2.cobertura(k) > m1.cobertura(k))
        .collect();
    assert!(
        juntos.len() > 50,
        "controlo: a 2.ª compôs por cima da 1.ª ({})",
        juntos.len()
    );
    assert!(
        juntos.iter().all(|&k| m2.por_baixo(k) == m1.por_baixo(k)),
        "o 2.º toque trocou o que estava por baixo"
    );
    // Fora da faixa (o branco), preto por cima: a cobertura das duas é a que o byte mostra — a força da
    // 1.ª não entra outra vez no passo da 2.ª.
    let depois = imagem(&t);
    let pior = juntos
        .iter()
        .filter(|&&k| !FAIXA.contains(&(k / L)))
        .map(|&k| (niveis(&m2, k) - (255.0 - f32::from(depois[k * 4]))).abs())
        .fold(0.0f32, f32::max);
    assert!(
        pior <= PISO,
        "a cobertura das duas pinceladas difere {pior:.1} níveis do que elas pintaram"
    );
    assert!(t.undo_last(), "o desfazer");
    assert_eq!(
        t.marca_do_artista().expect("a marca").ladrilhos(),
        m1.ladrilhos()
    );
    assert!(t.undo_last(), "o 2.º desfazer");
    assert!(t.marca_do_artista().expect("a marca").vazia());
    assert!(t.redo_last() && t.redo_last(), "os refazeres");
    assert_eq!(
        t.marca_do_artista().expect("a marca").ladrilhos(),
        m2.ladrilhos()
    );
}

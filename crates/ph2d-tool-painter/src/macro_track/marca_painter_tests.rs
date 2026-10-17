//! W9 — **a marca do artista no Painter** (`docs/Macros/07_plano_W9.md`): a água que corre sozinha
//! depois do Parar não marca; a pincelada do artista marca onde o pincel passou, com a água a correr; o
//! desfazer desliga-a; a reescrita de uma etapa e o «Recomeçar» não marcam e a reescrita salta o que
//! está marcado. A água corre no WORKER de verdade (o editor), nunca no relógio fixo.

use super::etapas::imagens_da_gravacao;
use super::marca::{MarcaDoArtista, epoca_da_gravacao};
use super::roteiro::grava_no_papel_ligado_com_o_worker;
use crate::PaintMedia;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool, Tool};
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

const LADO: u32 = 128;
/// O alcance de um dab do pincel de raio 6 com a ponta de fábrica (`radius × tip_reach + ½`), por cima.
const ALCANCE: f32 = 6.0 * 2.0 + 1.0;

/// A gravação no papel (duas pinceladas de Wet Paint) com o worker real, e o editor que a gravou com a
/// marca da gravação posta (vazia).
fn gravado() -> PainterTool {
    let (mut t, bytes, _) = grava_no_papel_ligado_com_o_worker(LADO, 7, 2).expect("grava");
    let track = ph2d_macro::codec::read(&bytes)
        .expect("lê")
        .tracks
        .remove(0);
    t.poe_a_marca(Some(MarcaDoArtista::nova(
        LADO,
        LADO,
        epoca_da_gravacao(&track),
    )));
    t
}

fn imagem(t: &PainterTool) -> Vec<u8> {
    t.imagem_aplicada().expect("documento").0
}

/// Um quadro do editor.
fn quadro(t: &mut PainterTool) {
    drop(t.take_preview_arc());
    t.on_tick(16.667);
    std::thread::sleep(Duration::from_millis(1));
}

/// Quadros até a água parar; devolve quantos.
fn espera_a_agua(t: &mut PainterTool) -> usize {
    let t0 = Instant::now();
    let mut q = 0;
    while !t.agua_parada() {
        assert!(t0.elapsed().as_secs() < 120, "a água não parou em 120 s");
        quadro(t);
        q += 1;
    }
    q
}

/// Um traço recto de `de` a `ate` (texels), um quadro por evento.
fn traco(t: &mut PainterTool, de: [f32; 2], ate: [f32; 2]) {
    const N: usize = 8;
    for i in 0..=N {
        #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
        let f = i as f32 / N as f32;
        let phase = match i {
            0 => PointerPhase::Down,
            N => PointerPhase::Up,
            _ => PointerPhase::Move,
        };
        t.on_canvas_pointer(CanvasPointer {
            pos: [de[0] + (ate[0] - de[0]) * f, de[1] + (ate[1] - de[1]) * f],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        quadro(t);
    }
}

/// A distância do centro do texel `i` ao segmento `de`–`ate` (a régua do pincel, à parte).
fn distancia(i: usize, de: [f32; 2], ate: [f32; 2]) -> f32 {
    #[expect(clippy::cast_precision_loss, reason = "uma tela pequena")]
    let p = [
        (i % LADO as usize) as f32 + 0.5,
        (i / LADO as usize) as f32 + 0.5,
    ];
    let d = [ate[0] - de[0], ate[1] - de[1]];
    let l2 = d[0] * d[0] + d[1] * d[1];
    let s = (((p[0] - de[0]) * d[0] + (p[1] - de[1]) * d[1]) / l2).clamp(0.0, 1.0);
    let q = [de[0] + d[0] * s - p[0], de[1] + d[1] * s - p[1]];
    (q[0] * q[0] + q[1] * q[1]).sqrt()
}

fn diferem(a: &[u8], b: &[u8]) -> Vec<usize> {
    (0..a.len() / 4)
        .filter(|&k| a[k * 4..k * 4 + 4] != b[k * 4..k * 4 + 4])
        .collect()
}

fn marca(t: &PainterTool) -> MarcaDoArtista {
    t.marca_do_artista().expect("a marca").clone()
}

/// A1 no Painter — ⭐ **a água que corre sozinha depois do Parar não marca nada**: o editor corre a água
/// no worker até ela parar (a imagem muda — o controlo) e a marca continua vazia. ⛔ Uma marca escrita
/// pelo `mark_dirty` (o que a água também chama) marcaria a gravação inteira — a régua W8 «≠ fim» lia o
/// mesmo minuto como pintura do artista.
#[test]
fn a_agua_que_corre_sozinha_depois_do_parar_nao_marca() {
    let mut t = gravado();
    let no_parar = imagem(&t);
    let q = espera_a_agua(&mut t);
    assert!(
        !diferem(&no_parar, &imagem(&t)).is_empty(),
        "controlo: a água não correu depois do Parar ({q} quadros)"
    );
    let m = marca(&t);
    assert!(
        m.vazia(),
        "a água marcou {} texels em {q} quadros",
        m.quantos()
    );
}

/// ⭐ **A pincelada do artista com a água da gravação a correr marca onde o pincel passou, e só aí**: o
/// núcleo do traço fica marcado (a tinta molhada aparece pela água, às vezes depois de levantar); nada
/// além do alcance do pincel — raio 6 × o alcance da ponta (≤ 2) — se marca, nem onde a água da gravação
/// continua a mudar a imagem (o controlo). Diagnóstico de 17/10: o 1.º dab marca 468 texels (raio ≈ 12,2:
/// o alcance da ponta de fábrica ≈ 1,95) e nada mais nasce em 2 000 quadros de água. O desfazer
/// desliga a marca do passo (depois de a água dele ser absorvida nele) e o refazer liga-a de novo.
#[test]
fn a_pincelada_do_artista_marca_onde_o_pincel_passou_e_o_desfazer_a_desliga() {
    let mut t = gravado();
    let antes = imagem(&t);
    let (de, ate) = ([12.0, 118.0], [60.0, 118.0]);
    t.set_brush_color_srgb8([20, 20, 200]);
    t.set_brush_size_px(6.0);
    traco(&mut t, de, ate);
    espera_a_agua(&mut t);
    let m = marca(&t);
    let n = (LADO * LADO) as usize;
    let longe: Vec<usize> = (0..n)
        .filter(|&i| m.marcado(i) && distancia(i, de, ate) > ALCANCE)
        .collect();
    assert!(
        longe.is_empty(),
        "{} texels marcados longe do pincel (1.º: {:?})",
        longe.len(),
        longe.first()
    );
    let nucleo: Vec<usize> = (0..n).filter(|&i| distancia(i, de, ate) <= 1.5).collect();
    let fora: Vec<&usize> = nucleo.iter().filter(|&&i| !m.marcado(i)).collect();
    assert!(
        fora.is_empty(),
        "{} de {} texels do núcleo do traço sem marca",
        fora.len(),
        nucleo.len()
    );
    let agua_longe = diferem(&antes, &imagem(&t))
        .into_iter()
        .filter(|&i| distancia(i, de, ate) > ALCANCE + 4.0)
        .count();
    assert!(
        agua_longe > 0,
        "controlo: a água da gravação não mudou nada longe do traço"
    );

    let marcada = m.palavras().as_ref().clone();
    assert!(t.undo_last(), "o desfazer");
    let m = marca(&t);
    assert!(
        m.vazia(),
        "o desfazer deixou {} texels marcados",
        m.quantos()
    );
    assert!(t.redo_last(), "o refazer");
    assert_eq!(
        *marca(&t).palavras().as_ref(),
        marcada,
        "o refazer não devolveu a marca do traço"
    );
}

/// **Um traço Digital (sem água) marca exactamente os texels que mudaram** — a régua é o diff das duas
/// imagens, à parte.
#[test]
fn um_traco_digital_marca_exactamente_o_que_mudou() {
    let mut t = PainterTool::default();
    t.set_source(vec![255; (LADO * LADO * 4) as usize], LADO, LADO);
    t.set_paint_media(PaintMedia::Digital);
    t.poe_a_marca(Some(MarcaDoArtista::nova(LADO, LADO, 1)));
    t.set_brush_size_px(8.0);
    let antes = imagem(&t);
    traco(&mut t, [20.0, 30.0], [90.0, 70.0]);
    let mudou = diferem(&antes, &imagem(&t));
    assert!(mudou.len() > 100, "controlo: o traço pintou");
    let m = marca(&t);
    let marcados: Vec<usize> = (0..(LADO * LADO) as usize)
        .filter(|&i| m.marcado(i))
        .collect();
    assert_eq!(marcados, mudou, "a marca não é o que o traço mudou");
}

/// **A reescrita de uma etapa e o «Recomeçar» são da macro**: não marcam; e a reescrita deixa os texels
/// marcados como estão, mesmo onde ainda são o fim antigo.
#[test]
fn a_reescrita_e_o_recomecar_nao_marcam_e_a_reescrita_salta_o_marcado() {
    use super::etapas::edita_tamanho;
    use super::roteiro::{SPLASH, grava_nos_meios_e_o_tool};
    let (mut t, bytes) =
        grava_nos_meios_e_o_tool(LADO, &SPLASH, &[PaintMedia::Impasto]).expect("grava");
    let crua = ph2d_macro::codec::read(&bytes)
        .expect("lê")
        .tracks
        .remove(0);
    let mut editada = crua.clone();
    edita_tamanho(&mut editada, 1, 30.0).expect("edita");
    let nao = AtomicBool::new(false);
    let de = imagens_da_gravacao(&crua, &nao).expect("toca").expect("fim");
    let para = imagens_da_gravacao(&editada, &nao)
        .expect("toca")
        .expect("fim");
    let muda = diferem(&de.fim, &para.fim);
    assert!(muda.len() > 20, "controlo: a edição muda o fim");
    // Uma marca sobre metade do que a edição muda (o artista mexeu ali e deixou o fim antigo).
    let mut m = MarcaDoArtista::nova(LADO, LADO, epoca_da_gravacao(&crua));
    let metade = &muda[..muda.len() / 2];
    for &i in metade {
        #[expect(clippy::cast_precision_loss, reason = "uma tela pequena")]
        let (x, y) = (
            (i % LADO as usize) as f32 + 0.5,
            (i / LADO as usize) as f32 + 0.5,
        );
        m.marca_disco(x, y, 0.5);
    }
    let marcados: Vec<usize> = (0..(LADO * LADO) as usize)
        .filter(|&i| m.marcado(i))
        .collect();
    assert_eq!(marcados, metade, "controlo: a régua do disco de meio texel");
    t.poe_a_marca(Some(m.clone()));
    // Os planos da tela (a imagem acesa mistura o relevo dos vizinhos).
    let tela = |t: &PainterTool| t.snapshot_model().canvas_rgba.as_ref().clone();
    let (antes, fim_novo) = (tela(&t), para.modelo.0.canvas_rgba.as_ref().clone());
    assert!(t.reescreve_a_gravacao(&de.modelo, &para.modelo, 3) > 0);
    let depois = tela(&t);
    for &i in metade {
        assert_eq!(
            depois[i * 4..i * 4 + 4],
            antes[i * 4..i * 4 + 4],
            "o texel marcado {i} foi reescrito"
        );
    }
    assert!(
        diferem(&antes, &depois)
            .iter()
            .all(|&i| depois[i * 4..i * 4 + 4] == fim_novo[i * 4..i * 4 + 4]),
        "controlo: o que muda passa ao fim novo"
    );
    assert!(!diferem(&antes, &depois).is_empty(), "controlo: a reescrita mudou a tela");
    assert_eq!(
        marca(&t).palavras(),
        m.palavras(),
        "a reescrita marcou ou desmarcou"
    );
    let mut t0 = PainterTool::default();
    t0.set_source(vec![255; (LADO * LADO * 4) as usize], LADO, LADO);
    t.recomeca_com(&t0);
    assert!(
        diferem(&depois, &tela(&t)).len() > 20,
        "controlo: recomeçar mudou a imagem"
    );
    assert_eq!(marca(&t).palavras(), m.palavras(), "recomeçar marcou");
}


/// W9 A3 — **a imagem na caixa é o recorte da inteira, ao byte**: no papel (Wet Paint, a composição
/// sobre o papel), no Impasto (a luz do relevo lê os vizinhos fora da caixa) e numa tela simples; e
/// fora do documento é `None`.
#[test]
fn a_imagem_na_caixa_e_o_recorte_da_inteira() {
    use super::roteiro::{SPLASH, grava_nos_meios_e_o_tool};
    let recorte = |img: &[u8], l: u32, (x, y, w, h): (u32, u32, u32, u32)| -> Vec<u8> {
        (y..y + h)
            .flat_map(|j| img[((j * l + x) * 4) as usize..((j * l + x + w) * 4) as usize].to_vec())
            .collect()
    };
    let (papel, _, _) = grava_no_papel_ligado_com_o_worker(LADO, 7, 2).expect("grava");
    let (impasto, _) =
        grava_nos_meios_e_o_tool(LADO, &SPLASH, &[PaintMedia::Impasto]).expect("grava");
    let mut simples = PainterTool::default();
    simples.set_source(vec![90; (LADO * LADO * 4) as usize], LADO, LADO);
    for (nome, t) in [("papel", &papel), ("impasto", &impasto), ("simples", &simples)] {
        let inteira = imagem(t);
        for caixa in [(0, 0, LADO, LADO), (3, 5, 40, 17), (LADO - 9, LADO - 4, 9, 4)] {
            assert_eq!(
                t.imagem_aplicada_na_caixa(caixa),
                Some(recorte(&inteira, LADO, caixa)),
                "{nome}: a caixa {caixa:?} não é o recorte da inteira"
            );
        }
        assert_eq!(t.imagem_aplicada_na_caixa((LADO - 2, 0, 3, 1)), None);
    }
}

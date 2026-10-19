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
    traco_de(t, de, ate, 8, false);
}

/// [`traco`] com `n` movimentos, um quadro do editor (16 ms) por evento (`em_baixo` = o Down já foi).
fn traco_de(t: &mut PainterTool, de: [f32; 2], ate: [f32; 2], n: usize, em_baixo: bool) {
    for i in 0..=n {
        #[expect(clippy::cast_precision_loss, reason = "poucos passos")]
        let f = i as f32 / n as f32;
        let phase = match i {
            0 if !em_baixo => PointerPhase::Down,
            i if i == n => PointerPhase::Up,
            _ => PointerPhase::Move,
        };
        t.on_canvas_pointer(CanvasPointer {
            pos: [de[0] + (ate[0] - de[0]) * f, de[1] + (ate[1] - de[1]) * f],
            pressure: 1.0,
            tilt: [0.0, 0.0],
            phase,
        });
        quadro(t);
        if n > 8 {
            std::thread::sleep(Duration::from_millis(15));
        }
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
    let s = if l2 > 0.0 {
        (((p[0] - de[0]) * d[0] + (p[1] - de[1]) * d[1]) / l2).clamp(0.0, 1.0)
    } else {
        0.0
    };
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
    // O raio do disco do pincel, calibrado no 1.º dab (os discos provisórios), e um traço longo ao ritmo
    // do editor: a água da tinta nova escorre DENTRO do passo, e o que ela muda fora do disco não é marca.
    t.on_canvas_pointer(CanvasPointer {
        pos: de,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase: PointerPhase::Down,
    });
    quadro(&mut t);
    let n = (LADO * LADO) as usize;
    let r0 = {
        let m = marca(&t);
        (0..n)
            .filter(|&i| m.marcado(i))
            .map(|i| distancia(i, de, de))
            .fold(0.0_f32, f32::max)
    };
    assert!(r0 > 3.0, "controlo: o 1.º dab marcou (raio {r0})");
    traco_de(&mut t, de, ate, 40, true);
    let halo = |m: &MarcaDoArtista| {
        (0..n)
            .filter(|&i| m.marcado(i) && distancia(i, de, ate) > r0 + 0.75)
            .count()
    };
    assert_eq!(halo(&marca(&t)), 0, "a água da tinta fora do disco marcou");
    espera_a_agua(&mut t);
    let m = marca(&t);
    assert_eq!(
        halo(&m),
        0,
        "depois da água, marcas fora do disco do pincel"
    );
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

    let marcada = m.ladrilhos().as_ref().clone();
    assert!(t.undo_last(), "o desfazer");
    let m = marca(&t);
    assert!(
        m.vazia(),
        "o desfazer deixou {} texels marcados",
        m.quantos()
    );
    assert!(t.redo_last(), "o refazer");
    assert_eq!(
        *marca(&t).ladrilhos().as_ref(),
        marcada,
        "o refazer não devolveu a marca do traço"
    );
}

/// **Um traço Digital (sem água) sobre o BRANCO marca todos os texels que mudaram, e só onde o pincel
/// pôs tinta** — a régua é o diff das duas imagens, à parte; a força de cada um é
/// `marca_cobertura_tests`. Longe do traço nada.
#[test]
fn um_traco_digital_marca_o_que_mudou_e_nada_longe_dele() {
    let mut t = PainterTool::default();
    t.set_source(vec![255; (LADO * LADO * 4) as usize], LADO, LADO);
    t.set_paint_media(PaintMedia::Digital);
    t.poe_a_marca(Some(MarcaDoArtista::nova(LADO, LADO, 1)));
    t.set_brush_size_px(8.0);
    let antes = imagem(&t);
    let (de, ate) = ([20.0, 30.0], [90.0, 70.0]);
    traco(&mut t, de, ate);
    let mudou = diferem(&antes, &imagem(&t));
    assert!(mudou.len() > 100, "controlo: o traço pintou");
    let m = marca(&t);
    assert!(
        mudou.iter().all(|&i| m.marcado(i)),
        "um texel pintado sem marca"
    );
    let longe = (0..(LADO * LADO) as usize)
        .filter(|&i| m.marcado(i) && distancia(i, de, ate) > 8.0 * 2.0 + 2.0)
        .count();
    assert_eq!(longe, 0, "texels marcados longe do traço");
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
    let de = imagens_da_gravacao(&crua, &nao)
        .expect("toca")
        .expect("fim");
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
    assert!(
        !diferem(&antes, &depois).is_empty(),
        "controlo: a reescrita mudou a tela"
    );
    assert_eq!(
        marca(&t).ladrilhos(),
        m.ladrilhos(),
        "a reescrita marcou ou desmarcou"
    );
    let mut t0 = PainterTool::default();
    t0.set_source(vec![255; (LADO * LADO * 4) as usize], LADO, LADO);
    t.recomeca_com(&t0);
    assert!(
        diferem(&depois, &tela(&t)).len() > 20,
        "controlo: recomeçar mudou a imagem"
    );
    assert_eq!(marca(&t).ladrilhos(), m.ladrilhos(), "recomeçar marcou");
}

/// ⭐ W10 — **a reescrita passa o fim novo por baixo da tinta parcial do artista, e o desfazer devolve o
/// antigo**: gravado em Digital; uma pincelada preta macia por cima do fim; a etapa editada (a 2.ª
/// pincelada mais fina). Nos texels onde a pincelada cobre em parte e o fim muda, a imagem passa a ser a
/// mesma pincelada pintada sobre o fim NOVO (a régua: um tool à parte, até ao piso do carimbo) e o que
/// está por baixo na marca passa ao fim novo; o desfazer da reescrita devolve a imagem e a marca.
#[test]
fn a_reescrita_passa_o_fim_novo_por_baixo_da_tinta_parcial() {
    use super::etapas::edita_tamanho;
    use super::roteiro::{SPLASH, grava_nos_meios_e_o_tool};
    let (mut t, bytes) =
        grava_nos_meios_e_o_tool(LADO, &SPLASH, &[PaintMedia::Digital]).expect("grava");
    let crua = ph2d_macro::codec::read(&bytes)
        .expect("lê")
        .tracks
        .remove(0);
    let mut editada = crua.clone();
    edita_tamanho(&mut editada, 1, 30.0).expect("edita");
    let nao = AtomicBool::new(false);
    let de = imagens_da_gravacao(&crua, &nao)
        .expect("toca")
        .expect("fim");
    let para = imagens_da_gravacao(&editada, &nao)
        .expect("toca")
        .expect("fim");
    assert_eq!(
        imagem(&t),
        de.fim,
        "controlo: o Painter parou no fim gravado"
    );
    t.poe_a_marca(Some(MarcaDoArtista::nova(
        LADO,
        LADO,
        epoca_da_gravacao(&crua),
    )));
    let pinta = |t: &mut PainterTool| {
        t.set_paint_media(PaintMedia::Digital);
        t.set_brush_size_px(14.0);
        t.set_brush_color_srgb8([0, 0, 0]);
        traco(t, [64.0, 4.0], [64.0, 124.0]);
    };
    pinta(&mut t);
    let (antes, m) = (imagem(&t), t.marca_do_artista().expect("a marca").clone());
    let cheia = super::marca::CHEIA;
    let alvo: Vec<usize> = (0..(LADO * LADO) as usize)
        .filter(|&i| (1..cheia).contains(&m.cobertura(i)))
        .filter(|&i| de.fim[i * 4..i * 4 + 4] != para.fim[i * 4..i * 4 + 4])
        .collect();
    assert!(
        alvo.len() > 10,
        "controlo: a borda da pincelada cruza o que a edição muda ({})",
        alvo.len()
    );
    assert!(t.reescreve_a_gravacao(&de.modelo, &para.modelo, 3) > 0);
    let depois = imagem(&t);
    let mut r = PainterTool::default();
    r.set_source(para.fim.clone(), LADO, LADO);
    pinta(&mut r);
    let regua = imagem(&r);
    let m2 = t.marca_do_artista().expect("a marca").clone();
    for &i in &alvo {
        let d = (0..3)
            .map(|k| depois[i * 4 + k].abs_diff(regua[i * 4 + k]))
            .max()
            .unwrap_or(0);
        assert!(
            d <= 4,
            "texel {i}: {d} níveis da pincelada sobre o fim novo"
        );
        assert_eq!(
            m2.por_baixo(i)[..],
            para.fim[i * 4..i * 4 + 4],
            "o que está por baixo de {i}"
        );
    }
    assert!(t.undo_last(), "o desfazer da reescrita");
    assert_eq!(imagem(&t), antes, "o desfazer não devolveu a imagem");
    assert_eq!(
        t.marca_do_artista().expect("a marca").ladrilhos(),
        m.ladrilhos(),
        "o desfazer não devolveu o que estava por baixo"
    );
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
    for (nome, t) in [
        ("papel", &papel),
        ("impasto", &impasto),
        ("simples", &simples),
    ] {
        let inteira = imagem(t);
        for caixa in [
            (0, 0, LADO, LADO),
            (3, 5, 40, 17),
            (LADO - 9, LADO - 4, 9, 4),
        ] {
            assert_eq!(
                t.imagem_aplicada_na_caixa(caixa),
                Some(recorte(&inteira, LADO, caixa)),
                "{nome}: a caixa {caixa:?} não é o recorte da inteira"
            );
        }
        assert_eq!(t.imagem_aplicada_na_caixa((LADO - 2, 0, 3, 1)), None);
    }
}

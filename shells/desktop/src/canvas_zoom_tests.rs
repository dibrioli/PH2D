//! Os gates do **zoom do canvas**.
//!
//! ⚠️ Eles dirigem a LEI (o módulo irmão), não o `on_mouse_wheel` — aquele precisa de `gfx`
//! (janela + GPU) e nenhum teste de unidade o alcança; quem afirma a FIAÇÃO é o arch-gate em
//! `layout_scroll_gesture_tests.rs`. Os dois não são redundantes: um diz *que lei*, o outro diz
//! *que ela é a perguntada*.

use super::{CANVAS_ZOOM, CanvasZoom};
use ph2d_editor::motion::UiMotion;
use ph2d_render::Camera2d;

/// Um entalhe de roda para dentro, na régua do `on_mouse_wheel` (`0,9^(dy/16)` com `dy = 16`).
const NOTCH_IN: f32 = 0.9;
/// O quadro de 60 fps, que é o relógio com que a mola é integrada.
const FRAME: f64 = 1.0 / 60.0;
/// Um `height_world` cujo destino **não sobrevive** ao round-trip `ln`→`exp` em `f32`.
///
/// ⚠️ **A fixture tem de conter o fenómeno, e a primeira não continha:** os dois gates de
/// exactidão nasceram com `10,0`, cujo destino (`9,0`) round-trippa EXACTO — a mutação que publica
/// `exp(ln(destino))` no repouso passava por eles em silêncio. Com `50,0` o destino é `45,0` e o
/// round-trip dá `45,0000038`.
const LOSSY_START: f32 = 50.0;

/// Corre o gesto até assentar, devolvendo cada `height_world` publicado.
fn run(zoom: &mut CanvasZoom, motion: &mut UiMotion, start: f32) -> Vec<f32> {
    let mut live = start;
    let mut seen = Vec::new();
    for _ in 0..600 {
        let Some(next) = zoom.tick(live, motion) else {
            break;
        };
        live = next;
        seen.push(next);
        motion.advance(FRAME);
    }
    seen
}

/// **A roda escreve o DESTINO; a câmera só se mexe no tique.**
///
/// *Mutação que deve sangrar:* a roda a publicar `live * factor` — o salto que esta wave remove.
#[test]
fn a_notch_moves_the_target_and_the_camera_only_follows_on_the_tick() {
    let mut zoom = CanvasZoom::default();
    assert_eq!(
        zoom.target(),
        None,
        "em repouso a camera nao e' deste modulo"
    );

    zoom.wheel(10.0, NOTCH_IN);
    assert_eq!(zoom.target(), Some(9.0), "o entalhe pousa no destino");

    let mut motion = UiMotion::default();
    let first = zoom.tick(10.0, &mut motion).expect("o tique publica");
    assert!(
        (first - 10.0).abs() < 1e-6,
        "o PRIMEIRO quadro tem de sair do valor VIVO (a semeadura), e saiu de {first}"
    );
    assert!(
        first > 9.0,
        "e nao pode ja' estar no destino: isso e' o salto de volta"
    );
}

/// **Uma rajada compõe no DESTINO, nunca no vivo.**
///
/// ⚠️ É a lição que a rolagem de painel já pagou (*«cinco voltas de 100 px somam 230,56 em vez de
/// 500»*), aqui com o agravante de o zoom ser MULTIPLICATIVO: compor sobre um valor a meio
/// caminho perde uma fracção de cada entalhe, e a perda acumula.
///
/// *Mutação que deve sangrar:* `let base = live;` em vez de `self.target.unwrap_or(live)`.
#[test]
fn a_burst_of_notches_compounds_on_the_target_never_on_the_live() {
    let mut zoom = CanvasZoom::default();
    let mut motion = UiMotion::default();
    let mut live = 10.0_f32;
    for _ in 0..5 {
        zoom.wheel(live, NOTCH_IN);
        // Um quadro entre entalhes: é exactamente aqui que compor sobre o vivo perde terreno.
        if let Some(next) = zoom.tick(live, &mut motion) {
            live = next;
        }
        motion.advance(FRAME);
    }
    let want = 10.0 * NOTCH_IN.powi(5);
    let got = zoom.target().expect("o gesto continua em voo");
    assert!(
        (got - want).abs() < 1e-4,
        "cinco entalhes valem {want}, e o destino diz {got}"
    );
}

/// **O destino é cravado, então descer cinquenta entalhes e subir um MEXE.**
///
/// ⚠️ É a metade que o artista sente: sem o clamp no destino ele afundaria muito abaixo do
/// mínimo, e o caminho de volta seria dezenas de entalhes de nada a acontecer.
///
/// *Mutação que deve sangrar:* tirar o `.clamp(..)` do [`CanvasZoom::wheel`].
#[test]
fn fifty_notches_down_and_one_up_still_moves_the_picture() {
    let mut zoom = CanvasZoom::default();
    let mut live = 10.0_f32;
    for _ in 0..50 {
        zoom.wheel(live, NOTCH_IN);
        live = zoom.target().expect("em voo");
    }
    assert!(
        (live - Camera2d::ZOOM_MIN_HEIGHT_WORLD).abs() < 1e-6,
        "cinquenta entalhes tem de parar no minimo, e pararam em {live}"
    );
    zoom.wheel(live, 1.0 / NOTCH_IN);
    let up = zoom.target().expect("em voo");
    assert!(
        up > Camera2d::ZOOM_MIN_HEIGHT_WORLD * 1.05,
        "um entalhe de volta tem de afastar de verdade, e deu {up}"
    );
}

/// **O vivo nunca sai do intervalo, e não é uma segunda checagem — é a mola.**
///
/// `Role::Surface` é criticamente amortecida nos dois carácteres ⇒ não ultrapassa ⇒ um percurso
/// entre dois pontos do intervalo fica no intervalo. O gate mede o percurso INTEIRO, não os
/// extremos.
#[test]
fn the_live_zoom_never_leaves_the_clamp() {
    for (start, factor) in [(10.0_f32, NOTCH_IN), (10.0, 1.0 / NOTCH_IN)] {
        let mut zoom = CanvasZoom::default();
        let mut motion = UiMotion::default();
        for _ in 0..40 {
            zoom.wheel(zoom.target().unwrap_or(start), factor);
        }
        for v in run(&mut zoom, &mut motion, start) {
            assert!(
                (Camera2d::ZOOM_MIN_HEIGHT_WORLD..=Camera2d::ZOOM_MAX_HEIGHT_WORLD).contains(&v),
                "o vivo saiu do intervalo em {v}"
            );
        }
    }
}

/// **Assentado, a câmera volta a ser de quem a escrever — e no valor EXACTO.**
///
/// ⚠️ As duas metades são o mesmo desenho: o destino é um `Option`, então enquanto ninguém dá
/// zoom este módulo não toca na câmera (o *fit-to-view*, o load e as cenas de smoke continuam
/// donos dela), e o valor em que ele a larga é o destino, não o resíduo de `exp(ln(destino))`.
///
/// *Mutação que deve sangrar:* nunca largar o destino ⇒ o tique seguinte devolve `Some` e come a
/// escrita alheia.
#[test]
fn a_settled_zoom_gives_the_camera_back_at_the_exact_target() {
    let mut zoom = CanvasZoom::default();
    let mut motion = UiMotion::default();
    zoom.wheel(LOSSY_START, NOTCH_IN);
    let seen = run(&mut zoom, &mut motion, LOSSY_START);
    assert_eq!(
        *seen.last().expect("publicou"),
        Camera2d::zoomed(LOSSY_START, NOTCH_IN),
        "o repouso e' o destino, ao bit"
    );
    assert_eq!(zoom.target(), None, "e a camera volta a ser dos outros");
    assert_eq!(
        zoom.tick(3.0, &mut motion),
        None,
        "com o gesto acabado o modulo nao toca numa camera que outro dono escreveu"
    );
}

/// **Uma escrita ESTRANGEIRA no meio de um voo GANHA.**
///
/// ⚠️ A testemunha é o valor que este módulo publicou no quadro anterior — *enumerar os
/// escritores de `height_world` apodrece; uma testemunha não*.
///
/// *Mutação que deve sangrar:* tirar a comparação `live != self.published` ⇒ o zoom continua a
/// puxar a câmera de volta e o *fit-to-view* é comido em silêncio.
#[test]
fn a_foreign_write_mid_flight_wins() {
    let mut zoom = CanvasZoom::default();
    let mut motion = UiMotion::default();
    zoom.wheel(10.0, NOTCH_IN);
    let live = zoom.tick(10.0, &mut motion).expect("em voo");
    motion.advance(FRAME);
    assert!(zoom.target().is_some(), "o voo continua");
    // O `View · All` escreveu na camera entre os dois quadros.
    assert_eq!(
        zoom.tick(live * 3.0, &mut motion),
        None,
        "quem escreveu por fora e' a verdade"
    );
    assert_eq!(zoom.target(), None, "e o voo e' largado, nao adiado");
}

/// **Sob *reduced motion* a roda é instantânea, AO BIT.**
///
/// ⚠️ O controle desta wave: uma definição de acessibilidade que perturbasse o número que promete
/// não animar seria a garantia a mentir. É por isso que o caminho instantâneo **não passa pelo
/// log** — a ida-e-volta custaria ~1e-6 relativo.
///
/// *Mutação que deve sangrar:* tirar o atalho e deixar o `exp(ln(target))` responder.
#[test]
fn reduced_motion_keeps_the_wheel_instant_to_the_bit() {
    let mut motion = UiMotion::default();
    motion.set_reduced_motion(true);
    let mut zoom = CanvasZoom::default();
    zoom.wheel(LOSSY_START, NOTCH_IN);
    assert_eq!(
        zoom.tick(LOSSY_START, &mut motion),
        Some(Camera2d::zoomed(LOSSY_START, NOTCH_IN)),
        "com reduced motion um entalhe pousa no destino no MESMO quadro"
    );
    assert_eq!(zoom.target(), None, "e nao sobra voo nenhum");
    assert_eq!(
        motion.get(CANVAS_ZOOM),
        None,
        "nem entrada no substrato: o modo mais acessivel e' tambem o mais barato"
    );
}

/// O ritmo PERCEBIDO do gesto, quadro a quadro: que fracção do zoom já aconteceu aos olhos.
fn rhythm(start: f32, factor: f32) -> Vec<f32> {
    let mut zoom = CanvasZoom::default();
    let mut motion = UiMotion::default();
    zoom.wheel(start, factor);
    let target = Camera2d::zoomed(start, factor);
    run(&mut zoom, &mut motion, start)
        .into_iter()
        .map(|h| (h / start).ln() / (target / start).ln())
        .collect()
}

/// **Aproximar e afastar têm o MESMO ritmo** — a propriedade que o espaço logarítmico compra, e a
/// única que a derivação sustenta.
///
/// ⚠️ **É o gate que substituiu um que teria ficado verde por vácuo.** O primeiro afirmava que um
/// entalhe mede o mesmo percurso em qualquer nível de zoom — verdade sobre o NÚMERO, inerte sobre
/// o comportamento (o espaço linear é escala-invariante: `h₀` sai do valor, da velocidade e do
/// span) — e nem sequer chamava o módulo, então animar em espaço linear o deixaria verde.
///
/// *Mutação que deve sangrar:* animar `target` em vez de `target.ln()` (e publicar `now` em vez de
/// `now.exp()`) ⇒ a meio da mola um fator de 2 lê 41,5% a aproximar contra 58,5% a afastar.
#[test]
fn zooming_in_and_out_have_the_same_rhythm() {
    let inwards = rhythm(10.0, 0.5);
    let outwards = rhythm(5.0, 2.0);
    assert!(
        !inwards.is_empty() && inwards.len() == outwards.len(),
        "os dois sentidos tem de levar os mesmos quadros ({} contra {})",
        inwards.len(),
        outwards.len()
    );
    for (i, (a, b)) in inwards.iter().zip(&outwards).enumerate() {
        assert!(
            (a - b).abs() < 1e-3,
            "no quadro {i} o gesto lê {:.1}% a aproximar e {:.1}% a afastar",
            a * 100.0,
            b * 100.0
        );
    }
}

/// **A sonda que escolheu o espaço** — a assimetria do espaço linear, em números.
///
/// `cargo test -p ph2d-host-desktop --bin ph2d-host-desktop -- --ignored measure_the_zoom_space`
#[test]
#[ignore = "sonda de medicao"]
fn measure_the_zoom_space() {
    // A meio caminho da mola (x = 0,5), quanto do zoom PERCEBIDO ja' aconteceu?
    let half = |h0: f32, h1: f32| {
        let h = h0 + (h1 - h0) * 0.5;
        (h / h0).ln() / (h1 / h0).ln()
    };
    println!("espaco | fator | a meio da mola");
    for (name, h0, h1) in [
        ("aproximar 2x", 10.0_f32, 5.0_f32),
        ("afastar   2x", 5.0, 10.0),
        ("um entalhe in ", 10.0, 9.0),
        ("um entalhe out", 9.0, 10.0),
    ] {
        println!("linear | {name} | {:.1}% percorrido", half(h0, h1) * 100.0);
    }
    println!("log    | qualquer   | 50.0% percorrido (por construcao)");
    println!();
    // ⚠️ O CONTROLE que derrubou a minha primeira justificacao: o espaco LINEAR ja' e'
    // escala-invariante, entao "um entalhe mede 200x mais no topo" e' verdade sobre o numero e
    // INERTE sobre o comportamento. Se estes tres nao coincidirem, a nota do modulo esta' errada.
    println!("o espaco LINEAR e' escala-invariante (o mesmo entalhe, tres niveis de zoom):");
    for h0 in [0.6_f32, 10.0, 90.0] {
        let h = h0 + (h0 * NOTCH_IN - h0) * 0.5;
        println!(
            "  h0={h0:>5.1}  a meio da mola: {:.4}% percorrido",
            (h / h0).ln() / NOTCH_IN.ln() * 100.0
        );
    }
}

//! **QUEM segura os planos** — a pergunta que reescreveu o plano da porta única de escrita.
//!
//! O irmão [`super::measure_commit_cost`] responde a outra metade (*de que é feito o CUSTO*); a linha
//! de corte é POSSE contra PREÇO.
//!
//! A decomposição anterior (doc 28 §5.13) disse que o pen-up custa 38,9 ms a 4096² **porque o
//! `commit_stroke_height` forka os três planos de relevo**, e fechou a conta somando `Vec::clone` dos
//! três: 0,40 + 9,94 + 18,13 = 28,47 contra 32,8 medidos. As sondas deste arquivo derrubaram os dois
//! lados dessa frase.
//!
//! ## 1. O dono extra é PERMANENTE, e é o histórico
//!
//! Em repouso, com dois traços commitados e nenhum gesto aberto, **os quatro planos canvas-shaped têm
//! DOIS donos**; limpar o histórico leva a contagem a **um**. Ou seja o `cursor` que a U1 instalou como
//! base de todo delta é um segundo dono permanente — e a hipótese fácil (*"o dono extra é o snapshot de
//! pen-down"*) descrevia só metade. Consequência de projeto: um journal que substituísse apenas o
//! `paint.stroke_undo` deixaria a contagem em 2 e **não mudaria um milissegundo**.
//!
//! ### 1b. DENTRO do gesto são TRÊS, e cada um tem nome (doc 28 §5.20)
//!
//! `tool` (irredutível) · `cursor` · `paint.stroke_undo` — este último porque `snapshot_model` clona os
//! `BTreeMap`, o que bumpa um `Arc` por plano. O **quarto** que aparece logo após o PRIMEIRO traço é a
//! entrada dele: um traço que **cria** os planos de relevo não tem lado `before` a diferenciar, então o
//! delta grava `Whole { before, after }` e segura o plano vivo; do segundo traço em diante a entrada é
//! `Patch` e não segura nada. ⚠️ **`make_mut` copia com qualquer coisa acima de um** ⇒ remover UMA das
//! três não compra milissegundo nenhum: o S3 é tudo-ou-nada, e o alvo são as três.
//!
//! (As seções sobre o CUSTO — o fork, o pen-up, o commit — mudaram-se para o irmão junto com as sondas
//! que as mediram; o que segue é a atribuição de posse, que é o assunto deste arquivo.)
//!
//! ## 2. O fork custa um TERÇO do que a aritmética dizia
//!
//! `Vec::clone` é um memcpy **serial**; o produto usa [`super::plane_fork::fork_par`], que é paralelo.
//! Medido pela porta do produto a 4096²: **0,29 + 3,16 + 5,79 = 9,25 ms**, não 28,47. A soma que
//! "fechava" fechava por **coincidência** — e uma atribuição que casa com o total por acidente é pior
//! que nenhuma, porque encerra a investigação no lugar errado.
//!
//! ## 3. O pen-up é o COMMIT DE UNDO, e o `commit_stroke_height` é footprint-bound
//!
//! Ablacionando pela entrada (`paint.stroke_undo = None` faz o `close_stroke` pular o commit
//! estrutural), a 4096² com impasto: **40,20 ms completo contra 3,49 sem o commit** — e o que sobra é
//! **plano na tela** (3,35 / 3,43 / 3,49 a 1024²/2048²/4096²), que é a forma correta para trabalho
//! limitado pela pegada. **91% do pen-up era o histórico**, cujo `PlaneDeltas::split` varre
//! `diff_window` sobre os quatro planos: ~256 MB de comparação por traço.

use super::*;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_painter_brush::Falloff;

pub(super) fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Um canvas com pincel de impasto — o modo cujo pen-up paga os três planos de relevo.
pub(super) fn armed(size: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: 40.0,
        hardness: 0.5,
        falloff: Falloff::Sphere,
        strength: 1.0,
        color: [0.2, 0.3, 0.8],
        impasto: true,
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t
}

pub(super) fn stroke(t: &mut PainterTool, y: f32) {
    t.on_canvas_pointer(cp([60.0, y], PointerPhase::Down));
    for k in 1..=6u8 {
        let x = 60.0 + f32::from(k) * 30.0;
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([260.0, y], PointerPhase::Up));
}

fn plane_owners<T>(
    m: &std::collections::BTreeMap<crate::layers::LayerId, std::sync::Arc<Vec<T>>>,
    active: Option<crate::layers::LayerId>,
) -> usize {
    active
        .and_then(|a| m.get(&a).map(std::sync::Arc::strong_count))
        .unwrap_or(0)
}

fn owners(t: &PainterTool) -> (usize, usize, usize, usize) {
    let a = t.layers.active();
    (
        std::sync::Arc::strong_count(&t.canvas_rgba),
        plane_owners(&t.heights, a),
        plane_owners(&t.covers, a),
        plane_owners(&t.mats, a),
    )
}

/// **A contagem de donos em REGIME, e de quem é cada um.**
///
/// Roda dois traços (o primeiro instala o histórico) e conta os donos de cada plano canvas-shaped no
/// repouso entre eles — depois derruba os suspeitos, um a um, para atribuir.
#[test]
#[ignore = "medicao — rode com --release --ignored"]
fn who_holds_the_planes_when_a_stroke_begins() {
    let mut t = armed(1024);
    stroke(&mut t, 200.0);
    stroke(&mut t, 300.0);

    let (c, h, cv, m) = owners(&t);
    eprintln!("\n[donos] REGIME (2 tracos commitados, nenhum gesto aberto)");
    eprintln!("  canvas_rgba {c} · heights {h} · covers {cv} · mats {m}");
    eprintln!("  (1 = so o tool ⇒ a proxima escrita e IN PLACE; ≥2 ⇒ ela COPIA o plano inteiro)");

    // Suspeito 1: o snapshot de pen-down. Em repouso ele nem existe.
    eprintln!("  stroke_undo vivo? {}", t.paint.stroke_undo.is_some());

    // Suspeito 2: o histórico (o `cursor` da U1 + as entradas).
    t.undo.clear();
    let (c2, h2, cv2, m2) = owners(&t);
    eprintln!("[donos] depois de LIMPAR o historico");
    eprintln!("  canvas_rgba {c2} · heights {h2} · covers {cv2} · mats {m2}");

    // E dentro do gesto: quantos donos o primeiro dab encontra?
    //
    // ⚠️ A contagem depende de QUANTOS traços vieram antes, e não por acaso: o PRIMEIRO traço numa
    // camada CRIA os planos de relevo, então o `split` não tem janela a diferenciar (o lado `before`
    // é vazio) e a entrada guarda `Whole { before, after }` — um `Arc` do plano VIVO, para sempre.
    // Do segundo em diante a entrada é `Patch` e não segura plano nenhum.
    for warm in [1usize, 2, 4] {
        let mut t2 = armed(1024);
        for k in 0..warm {
            stroke(&mut t2, 200.0 + (k as f32) * 40.0);
        }
        t2.on_canvas_pointer(cp([60.0, 600.0], PointerPhase::Down));
        let (c, h, cv, m) = owners(&t2);
        eprintln!(
            "[donos] apos {warm} traco(s), dentro do gesto: canvas {c} · heights {h} · covers {cv} · mats {m}"
        );
    }

    let mut t2 = armed(1024);
    stroke(&mut t2, 200.0);
    t2.on_canvas_pointer(cp([60.0, 300.0], PointerPhase::Down));
    let (c3, h3, cv3, m3) = owners(&t2);
    eprintln!("[donos] DENTRO do gesto (logo apos o pen-down)");
    eprintln!("  canvas_rgba {c3} · heights {h3} · covers {cv3} · mats {m3}");

    // …e de QUEM são, um a um. A ordem é a das duas referências que o S3 removeria (§7 do doc 28):
    // primeiro o snapshot de pen-down, depois o `cursor` do histórico. O que sobrar depois das duas
    // é o que decide se a wave é viável — `make_mut` copia com qualquer coisa acima de um.
    t2.paint.stroke_undo = None;
    let (c4, h4, cv4, m4) = owners(&t2);
    eprintln!(
        "  - sem o snapshot de pen-down:  canvas {c4} · heights {h4} · covers {cv4} · mats {m4}"
    );
    t2.undo.clear();
    let (c5, h5, cv5, m5) = owners(&t2);
    eprintln!(
        "  - …e sem o historico:          canvas {c5} · heights {h5} · covers {cv5} · mats {m5}"
    );
    eprintln!("  (sobra 1 = o TOOL, irredutivel ⇒ o S3 chega la, mas so alcancando as tres)\n");
}

/// **O QUE O JOURNAL RETÉM NUM TRAÇO REAL — o número que decide se ele pode sair do `cfg(debug)`.**
///
/// A troca do S3 substitui, em cada gesto, um **fork do plano inteiro** (que só existe porque há um
/// segundo dono) por uma **captura da região**. A §5.25 mediu a região contra o fork no primitivo —
/// 15–73× mais barata —, mas o primitivo não diz quanto um TRAÇO retém, e é o traço que paga.
///
/// ⚠️ **A comparação certa não é contra o documento, é contra o que o modelo de HOJE já retém.** O fork
/// aloca um plano inteiro por gesto e o histórico guarda um delta por passo; se a captura for da mesma
/// ordem do delta, ela é gratuita em memória — ela substitui trabalho, não o acrescenta.
#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture"]
fn what_the_journal_retains_for_one_real_stroke() {
    println!(
        "\n{:<8} {:>10} {:>12} {:>12} {:>10}",
        "tela", "doc (MB)", "journal MB", "delta MB", "j/doc"
    );
    for side in [1024u32, 2048, 4096] {
        let mut t = armed(side);
        stroke(&mut t, 40.0); // o 1º instala o histórico (e o cursor)
        let before = t.undo.retained_bytes();

        // O passo seguinte, medido com o journal ancorado nele.
        // ⚠️ **Lido ANTES do pen-up**, e a 1ª versão desta sonda não era: o pen-up commita, o commit
        // move o cursor e `set_cursor` **zera o journal** — a tabela saía com 0,00 MB em toda tela, que
        // é o journal *depois* de ele ter cumprido o seu papel, não o pico que ele retém.
        t.begin_undo_step();
        t.on_canvas_pointer(cp([60.0, 80.0], PointerPhase::Down));
        for k in 1..=6u8 {
            t.on_canvas_pointer(cp([60.0 + f32::from(k) * 30.0, 80.0], PointerPhase::Move));
        }
        let journal = t.undo.write_state.journal_heap_bytes();
        t.on_canvas_pointer(cp([260.0, 80.0], PointerPhase::Up));

        let n = (side as usize) * (side as usize);
        // Os quatro planos canvas-shaped de uma camada tocada: rgba(4) + heights(4) + covers(1) + mats(7).
        let doc = (n * 16) as f64 / 1e6;
        let delta = (t.undo.retained_bytes() - before) as f64 / 1e6;
        println!(
            "{side:<8} {doc:>10.1} {:>12.2} {delta:>12.2} {:>9.1}%",
            journal as f64 / 1e6,
            100.0 * journal as f64 / (doc * 1e6)
        );
    }
    println!();
}

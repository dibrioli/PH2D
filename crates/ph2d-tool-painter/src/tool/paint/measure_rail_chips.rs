//! **O que cada chip do rail FAZ quando o artista o pega e arrasta** — a medição que decide qual pill
//! recebe o Reshape (o escopo que o [ADR-0156] deixou para wave própria).
//!
//! O rail é a barra universal: ela é a mesma nos quatro meios de pintura, e o meio que o Painter abre
//! é o **Digital** (Enio, 2026-07-22: *"o modo que aparece ao abrir o painter deve ser o digital
//! normal"*). Um chip que não faz nada ALI é um chip que a maioria dos artistas nunca vê funcionar.
//!
//! ⚠️ Esta sonda não afirma nada; ela IMPRIME, e passa pela porta do produto (`on_canvas_pointer`) —
//! não por um kernel escolhido a dedo. Cada linha é *um artista pegando o chip e arrastando*.
//!
//! Rodar: `cargo test -p ph2d-tool-painter --release the_rail_chips -- --ignored --nocapture`
//!
//! [ADR-0156]: ../../../../../docs/architecture/decisions/0156-liquify-is-an-authored-dab-list-cooked-on-the-device-never-a-stored-dense-field.md

use crate::tool::PainterTool;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};

const SIDE: u32 = 128;

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Uma tela com listras — arte que TODA operação pode mexer de forma medível. Numa tela chapada o
/// Smear e o Blur seriam invariantes por construção, e a tabela diria "inerte" sobre eles também.
fn striped_tool() -> PainterTool {
    let mut px = vec![255u8; (SIDE * SIDE) as usize * 4];
    for y in 0..SIDE {
        for x in 0..SIDE {
            if (x / 6) % 2 == 0 {
                let b = ((y * SIDE + x) * 4) as usize;
                px[b] = 0;
                px[b + 1] = 0;
                px[b + 2] = 0;
            }
        }
    }
    let mut t = PainterTool::default();
    t.set_source(px, SIDE, SIDE);
    t.set_brush_size_px(24.0);
    t
}

fn drag(t: &mut PainterTool) {
    t.on_canvas_pointer(cp([30.0, 64.0], PointerPhase::Down));
    let mut x = 30.0f32;
    while x < 96.0 {
        x += 4.0;
        t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([x, 64.0], PointerPhase::Up));
}

fn bytes_changed(a: &[u8], b: &[u8]) -> usize {
    a.iter().zip(b).filter(|(x, y)| x != y).count()
}

/// Uma cópia de TODOS os planos de altura, para diferenciar depois.
///
/// ⚠️ Contar texels NÃO-ZERO seria o oráculo errado: o Smooth do Sculpt *redistribui* altura sem
/// criar nem apagar texel nenhum, então a contagem fica idêntica sobre uma esculpida que funcionou.
/// A pergunta é *quantos texels mudaram de VALOR*.
fn relief(t: &PainterTool) -> Vec<Vec<f32>> {
    t.heights.values().map(|h| h.as_ref().clone()).collect()
}

fn relief_changed(a: &[Vec<f32>], b: &[Vec<f32>]) -> usize {
    a.iter()
        .zip(b)
        .map(|(x, y)| x.iter().zip(y).filter(|(p, q)| p != q).count())
        .sum()
}

/// **A tabela que decide o pill.** Para cada modo que um chip do rail publica, um arrasto no meio
/// Digital: quantos bytes do que o artista VÊ (`canvas_rgba`) mudaram, e quantos texels de relevo
/// (`heights`) foram escritos.
///
/// ⚠️ A coluna do relevo existe porque as duas perguntas são diferentes: o Sculpt escreve `h` e SÓ
/// `h` (doc 18 §5), e **relevo sobre cobertura zero não acende** — então ele pode escrever muito e
/// mostrar nada. É essa distinção que a palavra "inerte" precisa carregar para não ser injusta.
#[test]
#[ignore = "measurement, not a gate — run explicitly"]
fn the_rail_chips_are_measured_in_the_medium_the_painter_opens_in() {
    // Os modos que um chip do rail publica (`rail_painter_tools::push_paint_mode`), na ordem do rail.
    let modes = [
        "brush",
        "eraser",
        "clone",
        "smear",
        "blur",
        "liquify",
        "transform",
        "sculpt",
        "mask",
        "inpaint",
    ];
    println!("[rail] chip        pixels mudados   texels de relevo");
    for mode in modes {
        let mut t = striped_tool();
        t.set_paint_tool_mode(mode);
        let before = t.canvas_rgba.as_ref().clone();
        let relief_before = relief(&t);
        drag(&mut t);
        let after = t.canvas_rgba.as_ref().clone();
        println!(
            "[rail] {mode:<10} {:>14} {:>18}",
            bytes_changed(&before, &after),
            relief_changed(&relief_before, &relief(&t)),
        );
    }

    // ⚠️ **O CONTROLE do Transform** — a linha dele na tabela diz `0` pixels, e isso é HONESTO: entrar
    // no Transform LEVANTA um patch flutuante e a transformação identidade é byte-idêntica por
    // construção (`deform_transform_identity_is_byte_identical`). A evidência de que aquele chip é uma
    // ferramenta e não uma antessala não é pixel, é o GIZMO — e sem esta linha a tabela convidaria a
    // próxima pessoa a concluir "inerte" do mesmo jeito que ela quase me fez concluir sobre o Sculpt.
    let mut t = striped_tool();
    t.set_paint_tool_mode("transform");
    println!(
        "[rail] transform      {:>10}  (o CONTROLE: 0 pixels é a identidade; o que ele levanta é o gizmo)",
        if t.deform_gizmo().is_some() {
            "gizmo"
        } else {
            "NADA"
        },
    );

    // ⚠️ **O que esta sonda mediu ANTES da wave de 2026-08-08**, e que decidiu o desenho: havia um chip
    // só, `Deform`, publicando um fio só, `"deform"`, e ele movia **0** pixels — o temperamento abria
    // em `NONE` e o roteador de canvas consumia o arrasto sem agir. O mesmo chip movia **26 964**
    // depois de UM clique a mais no painel. Hoje o fio `"deform"` não existe: cada metade tem chip e
    // fio próprios, e as duas linhas acima são a prova viva de que nenhuma delas é uma antessala.

    // ⚠️ **O CONTROLE do Sculpt** — sem ele o zero acima seria uma acusação sem prova. O verbo não
    // está quebrado: ele reshapeia RELEVO, e no Digital não existe relevo para reshapear. Aqui o
    // mesmo gesto roda depois de uma pincelada de impasto ter posto corpo na tela.
    let mut t = striped_tool();
    t.toggle_brush_impasto();
    drag(&mut t); // deposita corpo
    t.set_paint_tool_mode("sculpt");
    let before = t.canvas_rgba.as_ref().clone();
    let relief_before = relief(&t);
    drag(&mut t);
    println!(
        "[rail] sculpt/impasto {:>10} {:>18}  (o CONTROLE: o mesmo verbo, com corpo na tela)",
        bytes_changed(&before, t.canvas_rgba.as_ref()),
        relief_changed(&relief_before, &relief(&t)),
    );
}

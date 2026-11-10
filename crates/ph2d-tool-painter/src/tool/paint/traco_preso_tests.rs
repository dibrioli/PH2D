//! **O `Up` que nunca chegou** (B03, DF-40/O2-1): soltar o botão fora da janela deixa o traço aberto, e o
//! `Down` seguinte abria outro por cima — o passo de undo do anterior perdia-se e o Ctrl+Z não o tirava.

use super::*;

const SIZE: u32 = 64;

/// Os texels que deixaram de ser o branco da tela.
fn pintados(t: &PainterTool) -> usize {
    t.canvas_rgba
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|p| **p != [255; 4])
        .count()
}

/// Um traço horizontal em `y`, de `x = 10` a `50`; `solta = false` é o `Up` perdido.
fn traco(t: &mut PainterTool, y: f32, solta: bool) {
    t.on_canvas_pointer(cp([10.0, y], PointerPhase::Down));
    for x in (12..=50).step_by(2) {
        t.on_canvas_pointer(cp([x as f32, y], PointerPhase::Move));
    }
    if solta {
        t.on_canvas_pointer(cp([50.0, y], PointerPhase::Up));
    }
}

fn dois_tracos_com_o_up_perdido() -> PainterTool {
    let mut t = white_canvas(SIZE, 3.0);
    traco(&mut t, 12.0, false);
    traco(&mut t, 44.0, true);
    t
}

/// Mutação que sangra: tirar o fecho do traço aberto no `Down` (`canvas_pointer.rs`).
#[test]
fn um_down_sobre_traco_aberto_nao_apaga_o_passo_anterior() {
    let mut t = dois_tracos_com_o_up_perdido();
    assert!(px(&t, SIZE, 30, 12) != [255; 4] && px(&t, SIZE, 30, 44) != [255; 4]);
    assert_eq!(
        px(&t, SIZE, 30, 28),
        [255; 4],
        "o 2.º Down não liga os traços"
    );
    assert!(t.undo_last(), "o 2.º traço desfaz-se");
    assert_eq!(
        px(&t, SIZE, 30, 44),
        [255; 4],
        "o 1.º Ctrl+Z tira o 2.º traço"
    );
    assert_ne!(px(&t, SIZE, 30, 12), [255; 4], "…e só ele");
    assert!(t.undo_last(), "o 1.º traço tem o SEU passo de undo");
}

#[test]
fn desfazer_tudo_deixa_zero_px() {
    let mut t = dois_tracos_com_o_up_perdido();
    assert!(pintados(&t) > 200, "fixtura: os dois traços pintam");
    let mut passos = 0;
    while t.undo_last() {
        passos += 1;
        assert!(passos < 8, "o undo não acaba");
    }
    assert_eq!((passos, pintados(&t)), (2, 0), "(passos, texels pintados)");
}

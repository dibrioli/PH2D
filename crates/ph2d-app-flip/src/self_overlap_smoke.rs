//! **A cena pronta para o smoke do SELF OVERLAP** (`PH2D_FLIP_SELF_OVERLAP_SMOKE=1`, 03 §8).
//!
//! Auto-sobreposição com ACÚMULO: quando um traço cruza a si mesmo, a tinta ESCURECE no
//! cruzamento (marcador/nanquim expressivo) em vez de ficar a união chapada. Só se VÊ com
//! **opacidade < 1** (a tinta opaca já satura), então os dois traços são desenhados a
//! **opacity 0.5**.
//!
//! A cena põe o MESMO traço — um LAÇO que cruza a si mesmo UMA vez — lado a lado:
//!
//! - **ESQUERDA (OFF)** — `self_overlap = false`, o traço de sempre: o cruzamento fica
//!   IGUAL aos braços (a união chapada). É o default, byte-idêntico ao Flip de sempre.
//! - **DIREITA (ON)** — `self_overlap = true`: o nó do cruzamento fica MAIS ESCURO (duas
//!   camadas compostas), e a passagem mais NOVA aparece por cima.
//!
//! **Números MEDIDOS** (a sonda headless é o gate GPU `a_stroke_with_self_overlap_accumulates_
//! at_the_crossing`, cruzamento em X a opacity 0.5): o braço (1 passagem) fica em **~128 de
//! 255**; o cruzamento OFF **~128** (== braço, sem acúmulo), o cruzamento ON **~191** (= 0.75,
//! a composição `over` de 2 camadas: `0.5 + 0.5·(1−0.5)`). A quina afiada ON também acumula, por
//! design (bleed de marcador); curva suave NÃO (as fitas mitram/abutam).

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::AtomicU32;

pub static FRAME: AtomicU32 = AtomicU32::new(0);

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_SELF_OVERLAP_SMOKE").is_some())
}

/// UM traço que cruza a si mesmo UMA vez (um laço "de rabo de porco"): a linha de baixo e a
/// descida final se cruzam num ponto NÃO-adjacente. `dx` desloca a cópia em x; `self_overlap`
/// liga o acúmulo. Opacity 0.5 — sem isso o acúmulo é invisível (tinta opaca já satura).
pub fn loop_stroke(dx: f32, self_overlap: bool) -> FlipStroke {
    let ink = Rgba::new(0.20, 0.55, 0.85, 1.0); // um azul de tinta, o mesmo nos dois
    let mut s = FlipStroke::new();
    // O laço: entra pela base, sobe pela direita, curva por cima para a esquerda e DESCE
    // cruzando a própria base. O cruzamento cai em ~(dx-0.13, -0.3), não-adjacente.
    for &(x, y) in &[
        (-1.0, -0.3),
        (0.3, -0.3),
        (0.9, 0.5),
        (0.2, 0.9),
        (-0.4, 0.4),
        (0.1, -0.9),
    ] {
        s.push_point(Point {
            pos: Vec2::new(x + dx, y),
            width: 0.12,
            opacity: 0.5,
            color: ink,
        });
    }
    s.hardness = 0.9;
    s.self_overlap = self_overlap;
    s
}

/// **Monta a cena** — porta única (o gate/mensagem encenam por aqui). Uma camada, um quadro:
/// o laço OFF à esquerda, o laço ON à direita. Devolve `(x_off, x_on)` das duas cópias.
pub fn stage(obj: &mut ph2d_flip::FlipObject) -> (f32, f32) {
    obj.fps = 12.0;
    obj.onion.enabled = false; // um quadro só; o onion sujaria a leitura do nó.

    let (x_off, x_on) = (-1.7, 1.7);
    let layer = obj.add_layer("Self Overlap");
    if let Some(d) = obj.insert_frame(layer, 0, Hold::Implicit, KeyKind::Keyframe) {
        let strokes = &mut obj.drawing_mut(d).expect("desenho").strokes;
        strokes.push(loop_stroke(x_off, false)); // ESQUERDA: OFF (o de sempre)
        strokes.push(loop_stroke(x_on, true)); // DIREITA: ON (acumula)
    }
    (x_off, x_on)
}

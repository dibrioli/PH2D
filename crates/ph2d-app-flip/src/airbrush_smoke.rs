//! **A cena pronta para o smoke do AIRBRUSH** (`PH2D_FLIP_AIRBRUSH_SMOKE=1`, 03 §8).
//!
//! O pincel airbrush analítico (Ciallo): o falloff da borda deixa de ser o do Painter e vira
//! a **transmitância física** da tinta por um dab esférico (Beer-Lambert): `A = 1 − exp(−k·√(1−dn²))`,
//! `k = mix(1, 8, hardness)`. É um **domo largo** de núcleo chato e borda SEMPRE macia. O slider
//! Hardness vira a densidade da névoa.
//!
//! A cena põe o MESMO traço grosso lado a lado, na MESMA hardness (0.5):
//!
//! - **ESQUERDA (padrão)** — `airbrush = false`, o falloff do Painter: núcleo CHEIO até `hardness`
//!   e queda `Smooth` na faixa restante, com a borda terminando em zero. É o default.
//!   ⚠️ Antes de 2026-07-28 era o `pow`+smoothstep do GP, um PICO estreito; o contraste desta
//!   cena era com ele. Hoje o discriminante é OUTRO — o airbrush fica muito mais CHEIO perto da
//!   BORDA (delta máximo medido 0,76 em hardness 0,5), em vez de meramente mais largo.
//! - **DIREITA (airbrush)** — `airbrush = true`: um **domo** — a tinta cobre quase toda a largura
//!   antes de rolar suave a zero na borda.
//!
//! **Números MEDIDOS** (a sonda headless é o gate GPU `an_airbrush_has_a_flatter_core_than_the_
//! standard_brush`, banda de raio 10 a hardness 0.5): no EIXO **255 padrão / 252 airbrush** — o
//! padrão é MAIOR, porque tem platô, e o airbrush nem no centro é opaco (`1−exp(−4.5) = 0.989`).
//! O discriminante está no ARO: `dn≈0.8` **55 / 231** · `dn≈0.9` **7 / 192**. Casa com o Self
//! Overlap: a acumulação `over` de airbrush é a multiplicação de transmitâncias (o build-up
//! físico).

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::AtomicU32;

pub static FRAME: AtomicU32 = AtomicU32::new(0);

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_AIRBRUSH_SMOKE").is_some())
}

/// UM traço reto e GROSSO (vertical), na hardness 0.5: onde o padrão vira pico e o airbrush vira
/// domo. `dx` desloca a cópia em x; `airbrush` troca o falloff. Opacity 1.0 (o falloff É o ponto).
pub fn thick_stroke(dx: f32, airbrush: bool) -> FlipStroke {
    let ink = Rgba::new(0.20, 0.55, 0.85, 1.0); // um azul de tinta, o mesmo nos dois
    let mut s = FlipStroke::new();
    for &y in &[-0.9_f32, 0.9] {
        s.push_point(Point {
            pos: Vec2::new(dx, y),
            width: 0.7, // grosso: várias linhas de queda, o perfil aparece
            opacity: 1.0,
            color: ink,
        });
    }
    s.hardness = 0.5;
    s.airbrush = airbrush;
    s
}

/// **Monta a cena** — porta única (o gate/mensagem encenam por aqui). Uma camada, um quadro: o
/// traço padrão à esquerda, o airbrush à direita. Devolve `(x_std, x_air)` das duas cópias.
pub fn stage(obj: &mut ph2d_flip::FlipObject) -> (f32, f32) {
    obj.fps = 12.0;
    obj.onion.enabled = false; // um quadro só; o onion sujaria a leitura do perfil.

    let (x_std, x_air) = (-1.2, 1.2);
    let layer = obj.add_layer("Airbrush");
    if let Some(d) = obj.insert_frame(layer, 0, Hold::Implicit, KeyKind::Keyframe) {
        let strokes = &mut obj.drawing_mut(d).expect("desenho").strokes;
        strokes.push(thick_stroke(x_std, false)); // ESQUERDA: padrão (pico)
        strokes.push(thick_stroke(x_air, true)); // DIREITA: airbrush (domo)
    }
    (x_std, x_air)
}

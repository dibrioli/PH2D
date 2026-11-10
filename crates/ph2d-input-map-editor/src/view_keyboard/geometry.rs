//! As tabelas do desenho: o teclado US (em passos de tecla) e o comando (em fracções da caixa). Fonte das
//! proporções: a maquete `docs/InputMap/maquetes/03_B_teclado_e_comando.png` (B1 e B3), medida em pixels.

use ph2d_input::{Binding, GamepadAxis, GamepadButton, Key};

/// Uma tecla: código, coluna e linha (em passos), largura (em passos).
pub(crate) struct KeyCap {
    pub key: Key,
    pub col: f32,
    pub row: f32,
    pub span: f32,
}

const fn k(code: u32, col: f32, row: f32, span: f32) -> KeyCap {
    KeyCap {
        key: Key(code),
        col,
        row,
        span,
    }
}

/// A parte do passo que a tecla enche (o resto é o vão).
pub(crate) const KEY_FILL: f32 = 0.88; // LITERAL-PX-OK: fração do passo, maquete B1
/// A altura da tecla e o passo entre linhas, em passos de tecla.
pub(crate) const KEY_H: f32 = 0.68; // LITERAL-PX-OK: fração do passo, maquete B1
pub(crate) const ROW_PITCH: f32 = 0.8; // LITERAL-PX-OK: fração do passo, maquete B1
/// A largura do desenho inteiro, em passos (da coluna do Q ao fim da →).
pub(crate) const WIDTH: f32 = 14.7; // LITERAL-PX-OK: passos de tecla, maquete B1
pub(crate) const ROWS: f32 = 4.0; // LITERAL-PX-OK: linhas do desenho
/// O passo não passa deste múltiplo da altura de um campo.
pub(crate) const PITCH_MAX_ROWS: f32 = 1.6; // LITERAL-PX-OK: múltiplo do token de linha

const ARROW: f32 = 1.45; // LITERAL-PX-OK: passos de tecla, maquete B1
const UP: u32 = 0xF700;
const DOWN: u32 = 0xF701;
const LEFT: u32 = 0xF702;
const RIGHT: u32 = 0xF703;

/// Q..P, A..L, Z..M, Space e as quatro setas.
pub(crate) fn keyboard() -> Vec<KeyCap> {
    let rows: [(&str, f32); 3] = [("QWERTYUIOP", 0.0), ("ASDFGHJKL", 0.4), ("ZXCVBNM", 0.8)]; // LITERAL-PX-OK: recuo das filas US, maquete B1
    let mut out = Vec::new();
    for (r, (letters, shift)) in rows.iter().enumerate() {
        for (i, c) in letters.chars().enumerate() {
            out.push(k(u32::from(c), shift + i as f32, r as f32, 1.0));
        }
    }
    out.push(k(0x20, 2.0, 3.0, 6.0)); // LITERAL-PX-OK: a barra de espaço, maquete B1
    out.push(k(UP, 11.75, 2.0, ARROW)); // LITERAL-PX-OK: maquete B1
    out.push(k(LEFT, 10.25, 3.0, ARROW)); // LITERAL-PX-OK: maquete B1
    out.push(k(DOWN, 11.75, 3.0, ARROW)); // LITERAL-PX-OK: maquete B1
    out.push(k(RIGHT, 13.25, 3.0, ARROW)); // LITERAL-PX-OK: maquete B1
    out
}

/// As setas pintam-se com ícone (a fonte não tem os glifos ←↑↓→).
pub(crate) fn arrow_icon(key: Key) -> Option<ph2d_editor_core::icons::IconId> {
    use ph2d_editor_core::icons::IconId;
    match key.0 {
        UP => Some(IconId::ChevronUp),
        DOWN => Some(IconId::ChevronDown),
        LEFT => Some(IconId::ChevronLeft),
        RIGHT => Some(IconId::ChevronRight),
        _ => None,
    }
}

/// A forma de um alvo do comando.
#[derive(Clone, Copy)]
pub(crate) enum PadShape {
    /// Pílula (gatilhos e ombros): centro, meia largura, meia altura.
    Pill,
    /// Círculo (botões e manches): centro, raio = meia altura.
    Round,
    /// Um braço da cruz ou uma metade de manche: quadrado com a seta.
    Arrow(ph2d_editor_core::icons::IconId),
}

/// Um alvo do comando: o que liga, a chave do nome curto, centro e meio-tamanho em fracções da caixa.
pub(crate) struct PadPart {
    pub binding: Binding,
    pub short_key: &'static str,
    pub cx: f32,
    pub cy: f32,
    pub hw: f32,
    pub hh: f32,
    pub shape: PadShape,
}

const fn part(
    binding: Binding,
    short_key: &'static str,
    (cx, cy): (f32, f32),
    (hw, hh): (f32, f32),
    shape: PadShape,
) -> PadPart {
    PadPart {
        binding,
        short_key,
        cx,
        cy,
        hw,
        hh,
        shape,
    }
}

const fn b(p: GamepadButton) -> Binding {
    Binding::PadButton(p)
}

const fn half(axis: GamepadAxis, positive: bool) -> Binding {
    Binding::PadAxis { axis, positive }
}

/// A caixa do comando: largura sobre altura.
pub(crate) const PAD_ASPECT: f32 = 1.75; // LITERAL-PX-OK: razão adimensional, maquete B3
/// O contorno do corpo, em fracções da caixa (um lado; o outro é o espelho em x).
pub(crate) const BODY: [(f32, f32); 9] = [
    (0.5, 0.2),   // LITERAL-PX-OK: maquete B3
    (0.2, 0.2),   // LITERAL-PX-OK: maquete B3
    (0.1, 0.32),  // LITERAL-PX-OK: maquete B3
    (0.03, 0.62), // LITERAL-PX-OK: maquete B3
    (0.02, 0.86), // LITERAL-PX-OK: maquete B3
    (0.08, 0.95), // LITERAL-PX-OK: maquete B3
    (0.16, 0.9),  // LITERAL-PX-OK: maquete B3
    (0.27, 0.72), // LITERAL-PX-OK: maquete B3
    (0.5, 0.72),  // LITERAL-PX-OK: maquete B3
];

// As medidas do comando (fracções da caixa), maquete B3.
const PILL: (f32, f32) = (0.07, 0.035); // LITERAL-PX-OK: maquete B3
const WIDE: (f32, f32) = (0.065, 0.035); // LITERAL-PX-OK: maquete B3
const BTN: (f32, f32) = (0.04, 0.06); // LITERAL-PX-OK: maquete B3
const ARM: (f32, f32) = (0.03, 0.045); // LITERAL-PX-OK: maquete B3
const LT: (f32, f32) = (0.24, 0.06); // LITERAL-PX-OK: maquete B3
const RT: (f32, f32) = (0.76, 0.06); // LITERAL-PX-OK: maquete B3
const LB: (f32, f32) = (0.24, 0.14); // LITERAL-PX-OK: maquete B3
const RB: (f32, f32) = (0.76, 0.14); // LITERAL-PX-OK: maquete B3
const BACK: (f32, f32) = (0.42, 0.4); // LITERAL-PX-OK: maquete B3
const START: (f32, f32) = (0.58, 0.4); // LITERAL-PX-OK: maquete B3
const DPAD: (f32, f32) = (0.4, 0.62); // LITERAL-PX-OK: maquete B3
const FACE: (f32, f32) = (0.78, 0.4); // LITERAL-PX-OK: maquete B3
/// O passo entre botões da face e da cruz, e o alcance das setas à volta da manche.
const STEP: f32 = 0.075; // LITERAL-PX-OK: maquete B3
const REACH: f32 = 0.1; // LITERAL-PX-OK: maquete B3

/// Os alvos do comando (fracções da caixa). As metades de manche são as quatro setas à volta de cada manche.
pub(crate) fn pad() -> Vec<PadPart> {
    use GamepadAxis as A;
    use GamepadButton as P;
    use PadShape::{Arrow, Pill, Round};
    use ph2d_editor_core::icons::IconId as I;
    let (fx, fy, dx) = (FACE.0, FACE.1, STEP * PAD_ASPECT);
    let (px, py, ex) = (DPAD.0, DPAD.1, STEP / PAD_ASPECT);
    let mut out = vec![
        part(
            b(P::LeftTrigger),
            "input_map_keyboard.pad.lt",
            LT,
            PILL,
            Pill,
        ),
        part(
            b(P::RightTrigger),
            "input_map_keyboard.pad.rt",
            RT,
            PILL,
            Pill,
        ),
        part(
            b(P::LeftBumper),
            "input_map_keyboard.pad.lb",
            LB,
            PILL,
            Pill,
        ),
        part(
            b(P::RightBumper),
            "input_map_keyboard.pad.rb",
            RB,
            PILL,
            Pill,
        ),
        part(
            b(P::Select),
            "input_map_keyboard.pad.back",
            BACK,
            WIDE,
            Pill,
        ),
        part(
            b(P::Start),
            "input_map_keyboard.pad.start",
            START,
            WIDE,
            Pill,
        ),
        part(
            b(P::North),
            "input_map_keyboard.pad.y",
            (fx, fy - dx),
            BTN,
            Round,
        ),
        part(
            b(P::South),
            "input_map_keyboard.pad.a",
            (fx, fy + dx),
            BTN,
            Round,
        ),
        part(
            b(P::West),
            "input_map_keyboard.pad.x",
            (fx - STEP, fy),
            BTN,
            Round,
        ),
        part(
            b(P::East),
            "input_map_keyboard.pad.b",
            (fx + STEP, fy),
            BTN,
            Round,
        ),
        part(
            b(P::DPadUp),
            "input_map_keyboard.pad.dpad",
            (px, py - STEP),
            ARM,
            Arrow(I::ChevronUp),
        ),
        part(
            b(P::DPadDown),
            "input_map_keyboard.pad.dpad",
            (px, py + STEP),
            ARM,
            Arrow(I::ChevronDown),
        ),
        part(
            b(P::DPadLeft),
            "input_map_keyboard.pad.dpad",
            (px - ex, py),
            ARM,
            Arrow(I::ChevronLeft),
        ),
        part(
            b(P::DPadRight),
            "input_map_keyboard.pad.dpad",
            (px + ex, py),
            ARM,
            Arrow(I::ChevronRight),
        ),
    ];
    for ((cx, cy), key) in STICKS {
        let (x, y) = if key == STICKS[0].1 {
            (A::LeftStickX, A::LeftStickY)
        } else {
            (A::RightStickX, A::RightStickY)
        };
        let dy = REACH * PAD_ASPECT;
        out.push(part(
            half(y, true),
            key,
            (cx, cy - dy),
            ARM,
            Arrow(I::ChevronUp),
        ));
        out.push(part(
            half(y, false),
            key,
            (cx, cy + dy),
            ARM,
            Arrow(I::ChevronDown),
        ));
        out.push(part(
            half(x, false),
            key,
            (cx - REACH, cy),
            ARM,
            Arrow(I::ChevronLeft),
        ));
        out.push(part(
            half(x, true),
            key,
            (cx + REACH, cy),
            ARM,
            Arrow(I::ChevronRight),
        ));
    }
    out
}

/// O centro das duas manches (o círculo desenhado, que não é alvo: as metades são as setas à volta).
pub(crate) const STICKS: [((f32, f32), &str); 2] = [
    ((0.28, 0.4), "input_map_keyboard.pad.ls"), // LITERAL-PX-OK: maquete B3
    ((0.65, 0.66), "input_map_keyboard.pad.rs"), // LITERAL-PX-OK: maquete B3
];
/// O raio do círculo da manche, em fracções da altura da caixa.
pub(crate) const STICK_R: f32 = 0.1; // LITERAL-PX-OK: maquete B3

/// O ícone da direcção de uma metade de manche — o ladrilho «LS ←» da B e o chip da A (uma porta só).
#[must_use]
pub fn half_icon(b: Binding) -> Option<ph2d_editor_core::icons::IconId> {
    use ph2d_editor_core::icons::IconId as I;
    let Binding::PadAxis { axis, positive } = b else {
        return None;
    };
    Some(match (axis, positive) {
        (GamepadAxis::LeftStickX | GamepadAxis::RightStickX, false) => I::ChevronLeft,
        (GamepadAxis::LeftStickX | GamepadAxis::RightStickX, true) => I::ChevronRight,
        (GamepadAxis::LeftStickY | GamepadAxis::RightStickY, true) => I::ChevronUp,
        (GamepadAxis::LeftStickY | GamepadAxis::RightStickY, false) => I::ChevronDown,
        _ => return None,
    })
}

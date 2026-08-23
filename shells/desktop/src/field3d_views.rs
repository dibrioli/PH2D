//! ⭐⭐ **AS VISTAS NOMEADAS** — frente, trás, lados, topo e base (W47).
//!
//! # O buraco: o módulo não tinha nenhuma
//!
//! Todo modelador tem as seis vistas ortogonais, e este tinha **zero**. A única forma de olhar a
//! peça de frente era arrastar até lá à mão, e a rotação é **livre** (sem polo, de propósito — ver
//! [`ph2d_field_render::Orbit`]), o que significa que *à mão* nunca acerta: chega-se perto, com o
//! horizonte torto.
//!
//! ⚠️ **E o doc da câmera já previa esta wave, à letra:** o [`Orbit::from_yaw_pitch`] existe *"para
//! escrever um **enquadramento nomeado** (o inicial, **a vista de frente, a de topo**)"*. A função
//! estava construída e ninguém a tinha chamado para isso.
//!
//! # ⚠️ Lemos as TECLAS do Blender, não os EIXOS dele
//!
//! O Blender é **Z para cima**; este módulo é **Y para cima** (é o `pitch` que sobe em Y, e o
//! traçado inteiro assenta nisso). Copiar os eixos dele daria uma «frente» que olha para o chão.
//!
//! O que se herda é a **memória de dedo**: `Numpad1` frente · `Numpad3` direita · `Numpad7` topo, e
//! **`Ctrl`** dá o oposto de cada uma — a lei dele, exatamente. `Numpad5` já era a lente (W15), como
//! lá.
//!
//! # ⭐ A vista é um FATO DERIVADO, não um modo guardado
//!
//! [`named_view`] responde *"a câmera está numa vista nomeada?"* olhando para a **orientação**, e
//! não para uma bandeira que alguém pôs. É isso que faz o realce do painel dizer a verdade depois de
//! um arrasto de meio grau: guardar *"estou em Frente"* daria um chip aceso sobre uma vista que já
//! não é aquela — o modo de falha clássico de um espelho de estado.

use ph2d_field_render::Orbit;

/// As seis vistas ortogonais, na ordem em que o painel as oferece.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Standard {
    Front,
    Back,
    Right,
    Left,
    Top,
    Bottom,
}

impl Standard {
    /// ⚠️ **A fonte da contagem** — o painel deriva a fileira daqui, como faz com `Mode::ALL`.
    pub(crate) const ALL: [Self; 6] = [
        Self::Front,
        Self::Back,
        Self::Right,
        Self::Left,
        Self::Top,
        Self::Bottom,
    ];

    /// A chave de i18n do rótulo.
    pub(crate) fn key(self) -> &'static str {
        match self {
            Self::Front => "panel.model3d.view.front",
            Self::Back => "panel.model3d.view.back",
            Self::Right => "panel.model3d.view.right",
            Self::Left => "panel.model3d.view.left",
            Self::Top => "panel.model3d.view.top",
            Self::Bottom => "panel.model3d.view.bottom",
        }
    }

    /// **Onde o OLHO fica** — a direção `fwd` da base da câmera, em coordenadas de mundo.
    ///
    /// ⚠️ Ela existe para o gate: a orientação é escrita em `yaw`/`pitch` (a porta da casa para um
    /// enquadramento nomeado), e um sinal trocado ali dá uma vista que se chama *Frente* e mostra as
    /// costas. *O nome tem de ser conferido contra o EIXO, não contra a aritmética que o produziu.*
    #[cfg(test)]
    pub(crate) fn eye_axis(self) -> [f32; 3] {
        match self {
            Self::Front => [0.0, 0.0, 1.0],
            Self::Back => [0.0, 0.0, -1.0],
            Self::Right => [1.0, 0.0, 0.0],
            Self::Left => [-1.0, 0.0, 0.0],
            Self::Top => [0.0, 1.0, 0.0],
            Self::Bottom => [0.0, -1.0, 0.0],
        }
    }

    /// O `(yaw, pitch)` que põe o olho naquele eixo. Ver [`eye_axis`](Self::eye_axis).
    fn yaw_pitch(self) -> (f32, f32) {
        let q = std::f32::consts::FRAC_PI_2;
        match self {
            Self::Front => (0.0, 0.0),
            Self::Back => (std::f32::consts::PI, 0.0),
            Self::Right => (q, 0.0),
            Self::Left => (-q, 0.0),
            Self::Top => (0.0, q),
            Self::Bottom => (0.0, -q),
        }
    }

    /// A orientação da vista, como quaternion.
    pub(crate) fn rotation(self) -> [f32; 4] {
        let (y, p) = self.yaw_pitch();
        Orbit::from_yaw_pitch(y, p).rotation
    }
}

/// Quão perto a orientação tem de estar para o painel dizer que **é** aquela vista.
///
/// ⚠️ **É uma tolerância de RECONHECIMENTO, não de gesto**, e o único recurso de que ela é feita é o
/// **ruído de `f32`** ao re-normalizar um quaternion. O que ela tem de separar:
///
/// | | `1 − |q·q′|` | em graus |
/// |---|---:|---:|
/// | re-normalizar (o ruído) | ~1e-7 | ~0,05° |
/// | **a barra** | **1e-6** | **0,16°** |
/// | **um pixel** de arrasto (`ORBIT_RAD_PER_PX = 0,01`) | 1,25e-5 | 0,57° |
///
/// ⇒ uma ordem de grandeza acima do ruído, e **12× abaixo** do menor gesto que existe.
///
/// ⛔ **O primeiro número que escrevi aqui foi `1e-4`, com uma justificação que se contradizia**:
/// *"~1,6° de desvio; um arrasto é sempre maior do que isso (0,57° por pixel)"* — 0,57° é **menor**
/// que 1,6°, e eu não fiz a conta. O gate `a_named_view_is_recognised_and_the_smallest_drag_lets_it_go`
/// reprovou à primeira corrida. *Uma justificação com dois números só vale depois de os comparar.*
const RECOGNISE: f32 = 1.0e-6;

/// ⭐ **Em que vista nomeada a câmera está** — ou `None`, que é a vista *livre*.
///
/// ⚠️ **Derivado da orientação**, nunca guardado. Ver a nota do módulo: um modo guardado ficaria
/// aceso depois de o artista arrastar meio grau para longe dele.
///
/// ⚠️ **`|q·q'|`, com módulo:** `q` e `−q` são a **mesma** orientação, e comparar sem o módulo faria
/// metade das vistas certas lerem como livres — dependendo do caminho pelo qual a câmera lá chegou.
pub(crate) fn named_view(cam: &Orbit) -> Option<Standard> {
    Standard::ALL
        .into_iter()
        .find(|s| 1.0 - dot(cam.rotation, s.rotation()).abs() < RECOGNISE)
}

fn dot(a: [f32; 4], b: [f32; 4]) -> f32 {
    a[0].mul_add(b[0], a[1].mul_add(b[1], a[2].mul_add(b[2], a[3] * b[3])))
}

/// **A tecla → a vista.** `Numpad1/3/7`, e `Ctrl` dá o oposto — a lei do Blender.
pub(crate) fn view_for_key(code: winit::keyboard::KeyCode, ctrl: bool) -> Option<Standard> {
    use winit::keyboard::KeyCode as K;
    Some(match (code, ctrl) {
        (K::Numpad1, false) => Standard::Front,
        (K::Numpad1, true) => Standard::Back,
        (K::Numpad3, false) => Standard::Right,
        (K::Numpad3, true) => Standard::Left,
        (K::Numpad7, false) => Standard::Top,
        (K::Numpad7, true) => Standard::Bottom,
        _ => return None,
    })
}

#[cfg(test)]
#[path = "field3d_views_tests.rs"]
mod tests;

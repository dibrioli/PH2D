//! **AS TRÊS ALÇAS do padrão de textura na tela** (plano 33, W6) — irmãs das do gradiente.
//!
//! ⭐ É a lei do **Inkscape**: *um `×` que move, um quadrado que escala, um círculo que roda*. É a
//! diferença entre afinar um padrão **na tela** e preencher um formulário de números — e é
//! literalmente o que a pesquisa do plano 33 (§1.2, ponto 4) apontou como *"o que o Inkscape
//! acertou"*.
//!
//! # As duas leis que isto herda, e que não se reescrevem aqui
//!
//! ⚠️⚠️ **O tamanho passa pela porta ÚNICA** [`PatternFill::set_longer_side`] — a mesma que o slider
//! *Size* usa. A lei escrita nos dois sítios é a lei que um dia muda só num, e aí a alça e o número
//! divergem sob o dedo do artista.
//!
//! ⚠️ **A geometria é LOCAL, como a do path** (ADR-0111): o cursor desce pelo afim da entidade antes
//! de chegar aqui, exactamente como nas alças de gradiente. É a mesma razão de sempre — o que se vê
//! é MUNDO, o que o documento guarda é LOCAL.
//!
//! # E elas somem no `Clamp`
//!
//! Naquele modo a colocação é **derivada** (uma cópia enquadrada na forma): `origin`, `size` e o
//! reticulado não têm quem os leia. Três alças que escrevem campos que ninguém lê seriam três
//! mentiras sob o dedo — é a mesma lei que já esconde os knobs correspondentes no painel.

use ph2d_vec_scene::{Paint, PatternFill, VecPath, VecPathId, VecScene};
use ph2d_vector::{Affine, Brush, Circle, Color, Fill, Point, Rect, Stroke, VectorScene};

/// Qual das três alças está sob o dedo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PatHandle {
    /// O `×`: arrasta a ORIGEM do padrão.
    Move,
    /// O quadrado: arrasta o TAMANHO de uma cópia (o lado maior; o aspecto é preservado).
    Scale,
    /// O círculo: arrasta o ÂNGULO do padrão.
    Rotate,
}

/// O padrão de `path`, **e só quando as alças fazem sentido**.
///
/// ⚠️ `None` no [`ph2d_vec_pattern::PatternMode::Clamp`]: ali a colocação é derivada, e uma alça que
/// escreve um campo que ninguém lê é uma mentira sob o dedo.
#[must_use]
fn live_pattern(path: &VecPath) -> Option<&PatternFill> {
    match path.fill.as_ref() {
        Some(Paint::Pattern(p)) if !matches!(p.mode, ph2d_vec_pattern::PatternMode::Clamp) => {
            Some(p)
        }
        _ => None,
    }
}

/// Qual alça o ponto LOCAL `(lx, ly)` acerta, dentro de `radius` (unidades locais).
///
/// A ordem é a da precedência sob o dedo: **mover primeiro**. Com o padrão muito pequeno as três
/// caem quase no mesmo sítio, e mover é o gesto que o artista quer nesse caso — escalar e rodar
/// exigem ver o que se faz.
#[must_use]
pub fn hit_pattern_handle(path: &VecPath, lx: f64, ly: f64, radius: f64) -> Option<PatHandle> {
    let p = live_pattern(path)?;
    let pts = p.handle_points();
    let near = |q: [f64; 2]| (lx - q[0]).hypot(ly - q[1]) <= radius;
    [
        (PatHandle::Move, pts[0]),
        (PatHandle::Scale, pts[1]),
        (PatHandle::Rotate, pts[2]),
    ]
    .into_iter()
    .find(|(_, q)| near(*q))
    .map(|(h, _)| h)
}

/// Arrasta `handle` para o ponto LOCAL `(lx, ly)`. `true` se algo mudou.
///
/// - **Move** põe a origem no cursor.
/// - **Scale** projecta `cursor − origem` no eixo X do padrão e faz disso o lado maior. ⚠️ A
///   PROJECÇÃO, e não a distância crua: com a distância, arrastar perpendicularmente inflaria o
///   ladrilho sem o artista se mexer na direcção que ele vê.
/// - **Rotate** lê o ângulo de `cursor − origem` e desconta o quarto de volta em que a alça senta.
pub fn drag_pattern_handle(path: &mut VecPath, handle: PatHandle, lx: f64, ly: f64) -> bool {
    let Some(Paint::Pattern(p)) = path.fill.as_mut() else {
        return false;
    };
    if matches!(p.mode, ph2d_vec_pattern::PatternMode::Clamp) {
        return false;
    }
    match handle {
        PatHandle::Move => {
            if p.origin == [lx, ly] {
                return false;
            }
            p.origin = [lx, ly];
        }
        PatHandle::Scale => {
            let (sin, cos) = p.angle.sin_cos();
            let d = [lx - p.origin[0], ly - p.origin[1]];
            let along = d[0] * cos + d[1] * sin;
            let antes = p.size;
            p.set_longer_side(along.max(MIN_SIZE));
            if p.size == antes {
                return false;
            }
        }
        PatHandle::Rotate => {
            let d = [lx - p.origin[0], ly - p.origin[1]];
            if d[0].hypot(d[1]) < MIN_SIZE {
                // Em cima da origem não há direcção nenhuma — e escrever `atan2(0, 0)` daria um
                // salto de ângulo no instante em que o dedo passasse pelo centro.
                return false;
            }
            let a = d[1].atan2(d[0]) - std::f64::consts::FRAC_PI_2;
            if (p.angle - a).abs() < f64::EPSILON {
                return false;
            }
            p.angle = a;
        }
    }
    true
}

/// O piso do tamanho, em unidades de mundo — o mesmo papel do piso da faixa do slider: um ladrilho
/// de tamanho zero é uma divisão por zero na colocação.
const MIN_SIZE: f64 = 1e-4;

/// Desenha as três alças do padrão do path `selected`, em px de TELA.
///
/// A alça activa ganha o anel branco — a mesma gramática das alças de gradiente, e de propósito: o
/// artista não deve ter de aprender duas linguagens de alça no mesmo canvas.
pub fn draw_pattern_handles(
    scene: &VecScene,
    selected: Option<VecPathId>,
    active: Option<PatHandle>,
    transform: Affine,
    target: &mut VectorScene,
) {
    let Some(sel) = selected else { return };
    let Some(path) = scene.paths().iter().find(|p| p.id == sel) else {
        return;
    };
    let Some(p) = live_pattern(path) else { return };
    let pts = p
        .handle_points()
        .map(|q| transform * Point::new(q[0], q[1]));
    let amber = Color::from_rgba8(235, 180, 90, 255);
    let white = Color::from_rgba8(255, 255, 255, 255);
    // Os EIXOS, para o artista ver o que cada alça mede.
    for far in [pts[1], pts[2]] {
        target.inner_mut().stroke(
            &Stroke::new(1.0),
            Affine::IDENTITY,
            &Brush::Solid(amber),
            None,
            &kurbo_line(pts[0], far),
        );
    }
    // MOVER: o `x`.
    let a = active == Some(PatHandle::Move);
    for (p0, p1) in [((-5.0, -5.0), (5.0, 5.0)), ((-5.0, 5.0), (5.0, -5.0))] {
        target.inner_mut().stroke(
            &Stroke::new(if a { 2.5 } else { 1.8 }),
            Affine::IDENTITY,
            &Brush::Solid(if a { white } else { amber }),
            None,
            &kurbo_line(
                Point::new(pts[0].x + p0.0, pts[0].y + p0.1),
                Point::new(pts[0].x + p1.0, pts[0].y + p1.1),
            ),
        );
    }
    // ESCALAR: o quadrado.
    let a = active == Some(PatHandle::Scale);
    target.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(amber),
        None,
        &Rect::new(
            pts[1].x - 4.5,
            pts[1].y - 4.5,
            pts[1].x + 4.5,
            pts[1].y + 4.5,
        ),
    );
    target.inner_mut().stroke(
        &Stroke::new(if a { 2.0 } else { 1.5 }),
        Affine::IDENTITY,
        &Brush::Solid(if a { white } else { amber }),
        None,
        &Rect::new(
            pts[1].x - 4.5,
            pts[1].y - 4.5,
            pts[1].x + 4.5,
            pts[1].y + 4.5,
        ),
    );
    // RODAR: o círculo.
    let a = active == Some(PatHandle::Rotate);
    target.inner_mut().fill(
        Fill::NonZero,
        Affine::IDENTITY,
        &Brush::Solid(amber),
        None,
        &Circle::new(pts[2], 5.0),
    );
    target.inner_mut().stroke(
        &Stroke::new(if a { 2.0 } else { 1.5 }),
        Affine::IDENTITY,
        &Brush::Solid(if a { white } else { amber }),
        None,
        &Circle::new(pts[2], 5.0),
    );
}

/// Um segmento como `BezPath` — o painter do Vello não tem primitiva de linha.
fn kurbo_line(a: Point, b: Point) -> ph2d_vector::BezPath {
    let mut p = ph2d_vector::BezPath::new();
    p.move_to(a);
    p.line_to(b);
    p
}

#[cfg(test)]
#[path = "pattern_handle_tests.rs"]
mod tests;

//! **A pintura do gizmo 3D** — a metade que faz pixels, separada da [lei](crate::field3d_gizmo).
//!
//! ⚠️ A separação é a que o resto do módulo já usa: a lei responde *"onde ficam as alças e o que o
//! arrasto vale?"* sem janela nenhuma, e este arquivo só a traduz para caminhos. Um gate de gesto
//! nunca precisa de abrir uma janela; um erro de pintura nunca pode mudar o que o arrasto faz.
//!
//! # As cores são TOKENS, e os eixos ganharam os seus
//!
//! `axis-x` / `axis-y` / `axis-z` entraram no design system (HR-15: zero hex). ⚠️ Não se reciclou o
//! `curve-r/g/b`: aquilo é o tinto de um **canal de cor**, isto é a identidade de uma **direção do
//! espaço** — e a convenção X=vermelho / Y=verde / Z=azul é a de todo modelador 3D. O dia em que
//! alguém re-vestir o editor de Curvas não pode mover os eixos junto.
//!
//! O realce de quem está sob o cursor é **derivado do próprio token** (a mesma cor, mais clara em
//! OKLCH), e não uma segunda cor escrita à mão: re-vestir um eixo re-veste o realce dele.

use ph2d_tokens::{Color, ColorToken, Theme};
// ⚠️ Os tipos de caminho vêm da `ph2d-vector`, e não do `vello` direto: ela re-exporta-os de
// propósito para não haver duas versões do mesmo `BezPath` a atravessar a fronteira.
use ph2d_vector::{
    Affine, BezPath, Brush, Circle, Color as VelloColor, Point, Shape as _, VectorScene,
};

use crate::field3d_gizmo::{
    GRIP_HALF_PX, HEAD_HALF_W_PX, HEAD_PX, Handle, INNER_PX, Motion, Projected, SHAFT_HALF_W_PX,
    Shape,
};

/// Quanto o realce levanta a luminosidade do token, em OKLCH.
///
/// ⚠️ **Uma fração do que falta para o branco**, e não uma soma: somar 0,2 a um token já claro
/// estoura para branco e o realce desaparece justamente nos temas claros, que é onde ele é mais
/// difícil de ver.
const HOVER_LIFT: f64 = 0.35;

/// A opacidade de uma alça de plano. Ela é uma **superfície**, e uma superfície opaca no meio do
/// gizmo taparia a peça que se está a mover.
const PLANE_ALPHA: f64 = 0.5;

/// Pinta o gizmo. `hot` é a alça sob o cursor (ou a agarrada), se houver.
/// `origin` é o canto da área desenhada: as alças vêm projetadas no referencial dela, e é este
/// deslocamento que as põe na janela. ⚠️ Ele viaja como transformação do caminho, e não somado às
/// coordenadas — assim a lei continua a falar só de área e nunca de janela.
pub(crate) fn paint(
    scene: &mut VectorScene,
    handles: &[Projected],
    hot: Option<Handle>,
    theme: Theme,
    origin: [f32; 2],
) {
    let at = Affine::translate((f64::from(origin[0]), f64::from(origin[1])));
    // ⚠️ **Do fundo para a frente, ao contrário da ordem de apontar.** A lista vem ordenada de
    // dentro para fora porque é assim que o `pick` desempata; desenhar nessa ordem poria o disco
    // central por baixo das hastes que ele tem de tapar.
    for h in handles.iter().rev() {
        if !h.live {
            continue;
        }
        let base = colour_of(h.handle, theme);
        let c = if hot == Some(h.handle) {
            lift(base)
        } else {
            base
        };
        match &h.shape {
            Shape::Arrow { from, to } => arrow(scene, *from, *to, c, at),
            Shape::Quad(q) => quad(scene, *q, with_alpha(c, PLANE_ALPHA), at),
            Shape::Disc { center, radius } => {
                // Um anel, não um disco: cheio ele tapava o ponto da peça que o gizmo marca.
                ring(scene, *center, *radius, c, at);
            }
            Shape::Arc(pts) => ribbon(scene, pts, c, at),
            Shape::Grip { from, to } => grip(scene, *from, *to, c, at),
        }
    }
}

fn colour_of(handle: Handle, theme: Theme) -> Color {
    let token = match handle {
        Handle::Axis(0) | Handle::Plane(0) | Handle::Ring(0) => ColorToken::AxisX,
        Handle::Axis(1) | Handle::Plane(1) | Handle::Ring(1) => ColorToken::AxisY,
        Handle::Axis(2) | Handle::Plane(2) | Handle::Ring(2) => ColorToken::AxisZ,
        // ⚠️ **Nem o disco/argola de vista nem o punho de tamanho são eixos.** Os dois primeiros
        // agem no plano da TELA, que não tem direção no mundo; o terceiro muda o tamanho, que não
        // tem direção nenhuma. Pintá-los com uma das três cores diria uma coisa falsa sobre o que
        // eles fazem — e a cor é a única legenda que um gizmo tem.
        _ => ColorToken::Text1,
    };
    token.resolve(theme)
}

/// A mesma cor, mais clara — **derivada do token**, para o realce seguir uma re-vestida.
fn lift(c: Color) -> Color {
    let (l, chroma, h) = ph2d_tokens::color::srgb_to_oklch(c.r, c.g, c.b);
    let lifted = l + (1.0 - l) * HOVER_LIFT;
    let out = Color::from_oklch(lifted, chroma, h);
    Color { a: c.a, ..out }
}

fn with_alpha(c: Color, a: f64) -> Color {
    Color {
        a: (f64::from(c.a) * a).round() as u8,
        ..c
    }
}

fn brush(c: Color) -> Brush {
    Brush::Solid(VelloColor::from_rgba8(c.r, c.g, c.b, c.a))
}

/// Haste (um quadrilátero fino) + ponta (um triângulo). ⚠️ **Preenchimento, e não traço**: a
/// `VectorScene` da casa preenche caminhos, e uma haste desenhada como retângulo tem a mesma
/// espessura em qualquer direção sem depender de um `stroke` que ela não expõe.
fn arrow(scene: &mut VectorScene, from: [f32; 2], to: [f32; 2], c: Color, at: Affine) {
    let d = [to[0] - from[0], to[1] - from[1]];
    let len = d[0].hypot(d[1]);
    if len <= INNER_PX + HEAD_PX {
        return;
    }
    let u = [d[0] / len, d[1] / len];
    let n = [-u[1], u[0]];
    let pt = |t: f32, off: f32| -> Point {
        Point::new(
            f64::from(from[0] + u[0] * t + n[0] * off),
            f64::from(from[1] + u[1] * t + n[1] * off),
        )
    };

    let shaft_end = len - HEAD_PX;
    let mut p = BezPath::new();
    p.move_to(pt(INNER_PX, -SHAFT_HALF_W_PX));
    p.line_to(pt(shaft_end, -SHAFT_HALF_W_PX));
    p.line_to(pt(shaft_end, SHAFT_HALF_W_PX));
    p.line_to(pt(INNER_PX, SHAFT_HALF_W_PX));
    p.close_path();
    scene.fill_path(&p, &brush(c), at);

    let mut head = BezPath::new();
    head.move_to(pt(len, 0.0));
    head.line_to(pt(shaft_end, -HEAD_HALF_W_PX));
    head.line_to(pt(shaft_end, HEAD_HALF_W_PX));
    head.close_path();
    scene.fill_path(&head, &brush(c), at);
}

fn quad(scene: &mut VectorScene, q: [[f32; 2]; 4], c: Color, at: Affine) {
    let mut p = BezPath::new();
    p.move_to(Point::new(f64::from(q[0][0]), f64::from(q[0][1])));
    for v in &q[1..] {
        p.line_to(Point::new(f64::from(v[0]), f64::from(v[1])));
    }
    p.close_path();
    scene.fill_path(&p, &brush(c), at);
}

/// Um anel de espessura [`SHAFT_HALF_W_PX`]`·2`, como dois círculos com regra par-ímpar.
fn ring(scene: &mut VectorScene, center: [f32; 2], radius: f32, c: Color, at: Affine) {
    let ctr = Point::new(f64::from(center[0]), f64::from(center[1]));
    let outer = Circle::new(ctr, f64::from(radius + SHAFT_HALF_W_PX));
    let inner = Circle::new(ctr, f64::from(radius - SHAFT_HALF_W_PX));
    let mut p = outer.to_path(0.1);
    // ⚠️ O buraco sai da regra NonZero com o furo em sentido CONTRÁRIO — é assim que a
    // `VectorScene` preenche (`Fill::NonZero`), e um furo no mesmo sentido seria simplesmente
    // pintado por cima.
    p.extend(inner.to_path(0.1).reverse_subpaths());
    scene.fill_path(&p, &brush(c), at);
}

/// Uma poligonal com espessura, um quadrilátero por segmento.
///
/// ⚠️ **Sem junta nas dobras, de propósito.** Uma argola amostrada em 48 pedaços dobra ~7,5° por
/// vértice; a fenda que isso deixa mede menos de um décimo de pixel na espessura que se usa aqui, e
/// pagar juntas por ela seria construir o que a medição não pediu.
fn ribbon(scene: &mut VectorScene, pts: &[[f32; 2]], c: Color, at: Affine) {
    for w in pts.windows(2) {
        let (a, b) = (w[0], w[1]);
        let d = [b[0] - a[0], b[1] - a[1]];
        let len = d[0].hypot(d[1]);
        if len <= f32::EPSILON {
            continue;
        }
        let n = [-d[1] / len * SHAFT_HALF_W_PX, d[0] / len * SHAFT_HALF_W_PX];
        quad(
            scene,
            [
                [a[0] + n[0], a[1] + n[1]],
                [b[0] + n[0], b[1] + n[1]],
                [b[0] - n[0], b[1] - n[1]],
                [a[0] - n[0], a[1] - n[1]],
            ],
            c,
            at,
        );
    }
}

/// O punho de tamanho: um traço fino até um quadrado. ⚠️ O traço é **decoração** — quem se agarra é
/// o quadrado (ver `hits`), e desenhá-lo mais grosso prometeria uma alça que não existe.
fn grip(scene: &mut VectorScene, from: [f32; 2], to: [f32; 2], c: Color, at: Affine) {
    ribbon(scene, &[from, to], with_alpha(c, PLANE_ALPHA), at);
    quad(
        scene,
        [
            [to[0] - GRIP_HALF_PX, to[1] - GRIP_HALF_PX],
            [to[0] + GRIP_HALF_PX, to[1] - GRIP_HALF_PX],
            [to[0] + GRIP_HALF_PX, to[1] + GRIP_HALF_PX],
            [to[0] - GRIP_HALF_PX, to[1] + GRIP_HALF_PX],
        ],
        c,
        at,
    );
}

/// ⭐ **O número do gesto**, ao lado do gizmo.
///
/// ⚠️ `motion` é o que o mundo **aplicou**, e não uma segunda conta a partir do cursor — ver a nota
/// no chamador e a lei do `gizmo/readout.rs` da casa. Com o gesto preso à grelha, as duas
/// discordariam e a ficha diria `0,503` enquanto a peça pousou em `0,500`.
pub(crate) fn paint_readout(
    scene: &mut VectorScene,
    text: &mut ph2d_text::TextSystem,
    motion: Motion,
    at: [f32; 2],
    theme: Theme,
) {
    let line = readout(motion);
    if line.is_empty() {
        return;
    }
    let font = ph2d_tokens::TypeToken::Sm.px();
    // Acima e à direita do centro, fora da folga onde as alças vivem: por cima delas a ficha taparia
    // o que ela descreve.
    let (x, y) = (at[0] + READOUT_OFFSET_PX, at[1] - READOUT_OFFSET_PX);
    ph2d_editor::paint::paint_text_block(
        text,
        scene,
        &line,
        x,
        y,
        font,
        READOUT_MAX_W_PX,
        // A mesma porta que todo widget usa para levar um token à cor do vello.
        ph2d_editor::paint::resolve(ColorToken::Text1, theme),
    );
}

/// Quanto a ficha se afasta do centro do gizmo, e a largura máxima dela.
const READOUT_OFFSET_PX: f32 = 26.0; // LITERAL-PX-OK: overlay metric (readout offset from gizmo centre)
const READOUT_MAX_W_PX: f32 = 220.0; // LITERAL-PX-OK: overlay metric (readout wrap width)

/// O texto de um gesto.
///
/// ⚠️ **Só os eixos que se mexeram**, com a letra à frente. Mostrar sempre os três encheria a ficha
/// de zeros num arrasto de eixo, que é o caso comum; mostrar só o comprimento perderia a direção,
/// que é o que se está a controlar.
///
/// ⛔ Nada de `Δ` nem de setas: o repositório já pagou tofu por um caractere que a fonte não tinha.
fn readout(motion: Motion) -> String {
    match motion {
        Motion::Translate(d) => {
            let mut parts: Vec<String> = Vec::new();
            for (k, name) in ["X", "Y", "Z"].into_iter().enumerate() {
                if d[k].abs() >= READOUT_EPS {
                    parts.push(format!("{name} {:+.3}", d[k]));
                }
            }
            parts.join("   ")
        }
        Motion::Rotate { angle, .. } => {
            let deg = angle.to_degrees();
            if deg.abs() >= READOUT_EPS {
                format!("{deg:+.1}°")
            } else {
                String::new()
            }
        }
        Motion::Scale(f) => {
            if (f - 1.0).abs() >= READOUT_EPS {
                format!("x {f:.2}")
            } else {
                String::new()
            }
        }
    }
}

/// Abaixo disto o gesto ainda não disse nada, e uma ficha de `+0,000` é ruído.
const READOUT_EPS: f32 = 1e-4;

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

/// ⭐⭐ **A MOLDURA DO LAÇO** (W58) — o rectângulo que o artista está a arrastar.
///
/// ⚠️ **Uma moldura, e não um preenchimento translúcido.** O que está por baixo é a peça, e é ela
/// que o artista está a mirar: uma manta por cima esconderia exactamente o que o gesto escolhe.
/// ⚠️ E ela é pintada em [`ColorToken::Accent`] — o mesmo tom com que este app diz *seleção*.
pub(crate) fn paint_lasso(
    scene: &mut VectorScene,
    from: [f32; 2],
    to: [f32; 2],
    theme: Theme,
    origin: [f32; 2],
) {
    let at = Affine::translate((f64::from(origin[0]), f64::from(origin[1])));
    let c = ColorToken::Accent.resolve(theme);
    let (lo, hi) = (
        [from[0].min(to[0]), from[1].min(to[1])],
        [from[0].max(to[0]), from[1].max(to[1])],
    );
    // O contorno como quatro fitas — a mesma primitiva das alças, e por isso a mesma espessura.
    ribbon(
        scene,
        &[
            [lo[0], lo[1]],
            [hi[0], lo[1]],
            [hi[0], hi[1]],
            [lo[0], hi[1]],
            [lo[0], lo[1]],
        ],
        c,
        at,
    );
}

/// ⭐⭐⭐ **O CHROME DA DIVISÃO** (W90): as costuras entre os viewports e a moldura do **activo**.
///
/// # Porque a moldura é obrigatória, e não decoração
///
/// Com quatro vistas iguais na tela, *«qual delas o teclado comanda?»* passa a ser uma pergunta — e
/// ela tem resposta (o [`crate::field3d_smoke::Smoke::active`], que o botão do rato escolhe). Sem a
/// moldura, `Home`, `Numpad5` e os verbos do gizmo agiriam numa vista que o artista não sabe qual é:
/// *um estado que muda o que a tecla seguinte faz e não se vê é a definição de uma interface que
/// mente.*
///
/// ⚠️ **As costuras são DERIVADAS dos retângulos**, nunca do `Split`: cada aresta interior é
/// desenhada uma vez, e uma divisão nova (duas vistas, três) entra aqui sem uma linha.
pub(crate) fn paint_split(
    scene: &mut VectorScene,
    rects: &[ph2d_editor::zones::Rect],
    active: usize,
    theme: Theme,
) {
    // A vista única não tem costura nem dono a anunciar — e pintar-lhe uma moldura seria uma
    // afirmação sobre um estado que ali não existe.
    if rects.len() < 2 {
        return;
    }
    let at = Affine::IDENTITY;
    let (x0, y0) = (
        rects.iter().fold(f32::MAX, |a, r| a.min(r.x)),
        rects.iter().fold(f32::MAX, |a, r| a.min(r.y)),
    );
    let (x1, y1) = (
        rects.iter().fold(f32::MIN, |a, r| a.max(r.x + r.w)),
        rects.iter().fold(f32::MIN, |a, r| a.max(r.y + r.h)),
    );
    let costura = ColorToken::Border.resolve(theme);
    for r in rects {
        // A aresta ESQUERDA de um retângulo é interior quando não é a da união — e a de cima idem.
        // ⚠️ Cada uma é desenhada por todos os vizinhos que a partilham; a repetição é invisível
        // (a mesma cor no mesmo sítio) e evita uma lista de arestas com dono.
        if r.x > x0 {
            ribbon(scene, &[[r.x, y0], [r.x, y1]], costura, at);
        }
        if r.y > y0 {
            ribbon(scene, &[[x0, r.y], [x1, r.y]], costura, at);
        }
    }
    if let Some(a) = rects.get(active) {
        ribbon(
            scene,
            &[
                [a.x, a.y],
                [a.x + a.w, a.y],
                [a.x + a.w, a.y + a.h],
                [a.x, a.y + a.h],
                [a.x, a.y],
            ],
            ColorToken::Accent.resolve(theme),
            at,
        );
    }
}

/// ⭐⭐⭐ **O RÓTULO DE UMA VISTA** (W90d) — o que a quina de cada viewport diz.
///
/// # Porque ele existe
///
/// Com quatro vistas na tela, *«qual é qual?»* passa a ser uma pergunta — e a resposta estava só na
/// geometria, que é ambígua exactamente nas peças simétricas em que ela mais importa. É a metade do
/// **cabeçalho** que o plano pede (`03_plano_implicito.md`), na forma que não rouba pixels ao
/// traçado: uma faixa reservada encolheria as quatro imagens e obrigaria a porta do layout a
/// devolver dois retângulos por vista.
///
/// ⚠️ **É um MOSTRADOR, não um controlo.** Trocar a vista de um quadrante já é alcançável — clicar
/// nele (passa a activo) e `Numpad1/3/7` ou o botão do painel. *Antes de construir um controlo,
/// meça se a composição já o exprime.*
///
/// ⚠️ O texto vem da chave i18n derivada da **câmera** ([`crate::field3d_views::label_key`]), nunca
/// do quadrante: orbitar a vista de cima faz dela *User*, que é o que ela passou a ser.
pub(crate) fn paint_view_label(
    scene: &mut VectorScene,
    text: &mut ph2d_text::TextSystem,
    rect: ph2d_editor::zones::Rect,
    key: &str,
    theme: Theme,
) {
    let line = ph2d_i18n::tr(key);
    if line.is_empty() {
        return;
    }
    ph2d_editor::paint::paint_text_block(
        text,
        scene,
        line,
        rect.x + LABEL_INSET_PX,
        rect.y + LABEL_INSET_PX,
        ph2d_tokens::TypeToken::Sm.px(),
        rect.w,
        // ⚠️ **Text2 e não Text1**: ele acompanha a peça o tempo todo, e um rótulo com o mesmo peso
        // do número de um gesto competiria com o que o artista está a fazer.
        ph2d_editor::paint::resolve(ColorToken::Text2, theme),
    );
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
    paint_readout_text(scene, text, &readout(motion), at, theme);
}

/// A mesma ficha, com o texto **dado** — é por aqui que a entrada numérica (W26) mostra o que está a
/// ser escrito.
///
/// ⚠️ Uma segunda função de pintar seria uma segunda posição, um segundo tamanho e uma segunda cor —
/// e a ficha do número digitado tem de aparecer exactamente onde a do gesto aparece, senão o artista
/// olha para dois sítios diferentes para a mesma coisa.
pub(crate) fn paint_readout_text(
    scene: &mut VectorScene,
    text: &mut ph2d_text::TextSystem,
    line: &str,
    at: [f32; 2],
    theme: Theme,
) {
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
        line,
        x,
        y,
        font,
        READOUT_MAX_W_PX,
        // A mesma porta que todo widget usa para levar um token à cor do vello.
        ph2d_editor::paint::resolve(ColorToken::Text1, theme),
    );
}

/// Quanto a ficha se afasta do centro do gizmo, e a largura máxima dela.
/// O recuo do rótulo de vista à quina do viewport — ver [`paint_view_label`].
const LABEL_INSET_PX: f32 = 8.0; // LITERAL-PX-OK: overlay metric (viewport label inset)
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

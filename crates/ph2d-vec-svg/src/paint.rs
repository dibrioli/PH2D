//! **A TINTA de um elemento SVG no vocabulário do documento** — as duas metades (preenchimento e
//! traço), e o que cada uma perde.
//!
//! ⚠️ Tudo aqui produz geometria em espaço **LOCAL** do elemento. Quem a leva ao mundo é o
//! [`ph2d_vec_scene::bake_xform`] no [`crate::import`], que carrega âncoras, geometria de
//! gradiente, raio de quina **e** largura de traço pela MESMA porta — construir aqui já em mundo
//! obrigaria a repetir essa lei, e ela tem quatro sub-casos.

use ph2d_vec_scene::{
    GradientStop, LineCap, LineJoin, Marker, Paint, Rgba8, StrokeAlign, StrokePaint, StrokeSpec,
};

/// O que uma tinta não conseguiu trazer (uma linha por espécie, para a nota do importador).
pub(crate) type Lost = Option<&'static str>;

/// `#rrggbb` + alfa → a cor do documento. O alfa vem do `fill-opacity` / `stop-opacity` do SVG,
/// que são fracções.
fn cor(c: usvg::Color, alpha: f32) -> Rgba8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "alpha vem de um NormalizedF32 (0..=1), e o clamp fecha o NaN"
    )]
    let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    Rgba8::new(c.red, c.green, c.blue, a)
}

/// As paradas de um gradiente, com o alfa do elemento já multiplicado em cada uma.
///
/// ⚠️ **A multiplicação é a lei do SVG** (`stop-opacity` × `fill-opacity`), e tem de acontecer aqui
/// porque o [`Paint`] do documento não tem um alfa de tinta ao lado das paradas — quem carrega
/// transparência é a cor de cada parada.
fn paradas(stops: &[usvg::Stop], alpha: f32) -> Vec<GradientStop> {
    stops
        .iter()
        .map(|s| {
            GradientStop::new(
                f64::from(s.offset().get()),
                cor(s.color(), s.opacity().get() * alpha),
            )
        })
        .collect()
}

/// Leva um ponto pelo `gradientTransform` — o único afim que o documento não guarda, então ele
/// **assa-se** nas coordenadas antes de o resto do caminho as levar ao mundo.
fn g_ponto(t: usvg::Transform, x: f32, y: f32) -> [f64; 2] {
    [
        f64::from(t.sx * x + t.kx * y + t.tx),
        f64::from(t.ky * x + t.sy * y + t.ty),
    ]
}

/// **A cor com que uma tinta inexprimível desenha.** Cinzento neutro — ⛔ e ele só é alcançado
/// quando o padrão não tem uma única forma de cor sólida lá dentro; nesse caso a nota diz que
/// houve aproximação, e *nomear é o que separa uma aproximação de uma mentira*.
const CINZA: Rgba8 = Rgba8 {
    r: 128,
    g: 128,
    b: 128,
    a: 255,
};

/// A primeira cor sólida que um `<pattern>` pinta — a melhor resposta honesta a *"de que cor é
/// isto?"* quando o ladrilho não tem equivalente.
fn primeira_cor(g: &usvg::Group) -> Option<usvg::Color> {
    for n in g.children() {
        match n {
            usvg::Node::Path(p) => {
                if let Some(f) = p.fill()
                    && let usvg::Paint::Color(c) = f.paint()
                {
                    return Some(*c);
                }
            }
            usvg::Node::Group(inner) => {
                if let Some(c) = primeira_cor(inner) {
                    return Some(c);
                }
            }
            _ => {}
        }
    }
    None
}

/// **A tinta como o documento a guarda**, mais o que ela perdeu.
pub(crate) fn tinta(p: &usvg::Paint, alpha: f32) -> (Paint, Lost) {
    match p {
        usvg::Paint::Color(c) => (Paint::Solid(cor(*c, alpha)), None),
        usvg::Paint::LinearGradient(g) => {
            let t = g.transform();
            let perdeu = (g.spread_method() != usvg::SpreadMethod::Pad)
                .then_some("gradiente com spreadMethod (o documento so' tem Pad)");
            (
                Paint::Linear {
                    stops: paradas(g.stops(), alpha),
                    start: g_ponto(t, g.x1(), g.y1()),
                    end: g_ponto(t, g.x2(), g.y2()),
                },
                perdeu,
            )
        }
        usvg::Paint::RadialGradient(g) => {
            let t = g.transform();
            let centro = g_ponto(t, g.cx(), g.cy());
            // ⚠️ O raio sofre a MESMA escala que os pontos, e o afim do gradiente pode não ser
            // uniforme — a média dos eixos é a aproximação que o documento já faz para um raio
            // (`Xform::mean_scale`), e escolher outra aqui poria duas leis para um comprimento.
            let esc = f64::from((t.sx * t.sy - t.kx * t.ky).abs().sqrt());
            // O foco (`fx`/`fy`/`fr`) não existe no documento: um radial daqui é concêntrico.
            let focado = (g.fx() - g.cx()).abs() > f32::EPSILON
                || (g.fy() - g.cy()).abs() > f32::EPSILON
                || g.fr().get() > f32::EPSILON;
            let perdeu = focado.then_some("gradiente radial com FOCO deslocado (fx/fy/fr)");
            (
                Paint::Radial {
                    stops: paradas(g.stops(), alpha),
                    center: centro,
                    radius: f64::from(g.r().get()) * esc,
                },
                perdeu,
            )
        }
        usvg::Paint::Pattern(pat) => (
            Paint::Solid(cor(
                primeira_cor(pat.root()).unwrap_or(usvg::Color::new_rgb(CINZA.r, CINZA.g, CINZA.b)),
                alpha,
            )),
            Some("<pattern> (entrou como a cor solida dele)"),
        ),
    }
}

/// **O traço como o documento o guarda**, em unidades LOCAIS, mais o que ele perdeu.
pub(crate) fn traco(s: &usvg::Stroke) -> (StrokeSpec, Lost) {
    let alpha = s.opacity().get();
    let (paint, perdeu) = tinta(s.paint(), alpha);
    // ⚠️ Um traço com gradiente **não é exprimível**: o `StrokePaint` tem `Solid`/`Pattern`/`Brush`
    // e nada mais, e o doc dele diz porquê (*"um enum que não representa o que o desenho faz
    // produz estado inalcançável, gravado"*). A cor representativa é a que o próprio documento
    // usaria para responder *"de que cor é este traço?"*.
    let (tinta_traco, perdeu) = match &paint {
        Paint::Solid(c) => (StrokePaint::Solid(*c), perdeu),
        outra => (
            StrokePaint::Solid(outra.primary_color()),
            Some("gradiente NO TRACO (entrou como a 1.a cor dele)"),
        ),
    };
    let width = f64::from(s.width().get());
    // ⚠️ **O tracejado do documento é em MÚLTIPLOS da largura**, o do SVG em unidades — e é essa
    // razão que o mantém correcto quando o traço engrossa. ⛔ Um `dasharray` com mais de dois
    // números descreve um ritmo que o documento não tem; entram os dois primeiros, e a nota diz.
    let (dash, perdeu_dash) = match s.dasharray() {
        Some(d) if d.len() >= 2 && width > 0.0 => (
            Some((f64::from(d[0]) / width, f64::from(d[1]) / width)),
            (d.len() > 2).then_some("stroke-dasharray com mais de 2 numeros (entraram os 2 1.os)"),
        ),
        Some(_) | None => (None, None),
    };
    (
        StrokeSpec {
            paint: tinta_traco,
            width,
            cap: match s.linecap() {
                usvg::LineCap::Butt => LineCap::Butt,
                usvg::LineCap::Round => LineCap::Round,
                usvg::LineCap::Square => LineCap::Square,
            },
            join: match s.linejoin() {
                usvg::LineJoin::Round => LineJoin::Round,
                usvg::LineJoin::Bevel => LineJoin::Bevel,
                // ⚠️ O `MiterClip` do SVG 2 é um `Miter` com um tecto próprio; o documento tem um
                // só `Miter`, e a diferença só se vê numa quina mais aguda que o limite.
                usvg::LineJoin::Miter | usvg::LineJoin::MiterClip => LineJoin::Miter,
            },
            dash,
            marker_start: Marker::None,
            marker_end: Marker::None,
            marker_scale: 1.0,
            marker_round: 0.0,
            align: StrokeAlign::Centre,
        },
        perdeu.or(perdeu_dash),
    )
}

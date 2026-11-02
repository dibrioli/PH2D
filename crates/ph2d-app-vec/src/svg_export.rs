//! ⭐⭐⭐ **EXPORTAR O DESENHO VECTORIAL COMO SVG** (plano 40).
//!
//! O porquê e a história: `docs/Vector Module/50_cabecalhos_do_app_vec.md` § `svg_export.rs`.

use ph2d_vec_scene::{Paint, StrokePaint, VecPath, VecScene, VecXforms};
use std::fmt::Write as _;

/// Quantas casas decimais nas coordenadas. `3` chega a `0,001` de unidade de mundo — abaixo do
/// erro de amostragem com que a própria detecção de cruzamentos trabalha.
const CASAS: usize = 3;

/// **Uma unidade de mundo vira uma unidade do ficheiro** — a escolha deste exportador (quem lê o
/// SVG compara os números com a régua do editor). ⛔ O que ele NÃO escolhe é o SINAL do Y, que é a
/// lei da casa e mora na porta.
const EIXOS: f64 = ph2d_vec_svg::EXPORT_PIXELS_PER_UNIT;

/// O resultado: o ficheiro e o que ficou por dizer.
pub struct Svg {
    pub texto: String,
    /// Quantas formas entraram.
    pub formas: usize,
    /// As formas cuja tinta não tem equivalente, `(nome do caminho, o que era)`.
    pub aproximadas: Vec<(u64, &'static str)>,
}

fn num(v: f64) -> String {
    let s = format!("{v:.CASAS$}");
    // `10.000` → `10`; `1.500` → `1.5`. Um ficheiro legível é um ficheiro que se lê.
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn cor(c: ph2d_vec_scene::Rgba8) -> String {
    format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
}

fn alfa(c: ph2d_vec_scene::Rgba8) -> String {
    num(f64::from(c.a) / 255.0)
}

/// A tinta de preenchimento como atributos SVG, mais o que ela perdeu.
/// ⚠️ **O `k` é o índice da CAMADA**, e não enfeite: com a pilha de aparência (v20) uma forma pode
/// ter N gradientes, e um `id` de `<linearGradient>` repetido faz o segundo silenciosamente
/// referenciar o primeiro — todas as camadas sairiam com a mesma rampa.
fn tinta(paint: &Paint, id: u64, k: usize, defs: &mut String) -> (String, Option<&'static str>) {
    match paint {
        Paint::Solid(c) => (
            format!(r#"fill="{}" fill-opacity="{}""#, cor(*c), alfa(*c)),
            None,
        ),
        Paint::Linear { stops, start, end } => {
            let g = format!("g{id}_{k}");
            let _ = write!(
                defs,
                r#"<linearGradient id="{g}" gradientUnits="userSpaceOnUse" x1="{}" y1="{}" x2="{}" y2="{}">"#,
                num(start[0]),
                num(start[1]),
                num(end[0]),
                num(end[1])
            );
            paradas(stops, defs);
            defs.push_str("</linearGradient>");
            (format!(r##"fill="url(#{g})""##), None)
        }
        Paint::Radial {
            stops,
            center,
            radius,
        } => {
            let g = format!("g{id}_{k}");
            let _ = write!(
                defs,
                r#"<radialGradient id="{g}" gradientUnits="userSpaceOnUse" cx="{}" cy="{}" r="{}">"#,
                num(center[0]),
                num(center[1]),
                num(*radius)
            );
            paradas(stops, defs);
            defs.push_str("</radialGradient>");
            (format!(r##"fill="url(#{g})""##), None)
        }
        // ⚠️ A MESMA cor que o renderer põe quando o ladrilho não resolve — quem abre o ficheiro vê
        // o que o app desenharia nesse caso, e não uma cor inventada aqui.
        Paint::MultiPoint { .. } => {
            let c = paint.primary_color();
            (
                format!(r#"fill="{}" fill-opacity="{}""#, cor(c), alfa(c)),
                Some("gradiente multi-ponto"),
            )
        }
        Paint::Pattern(p) => (
            format!(
                r#"fill="{}" fill-opacity="{}""#,
                cor(p.fallback),
                alfa(p.fallback)
            ),
            Some("padrão de textura"),
        ),
    }
}

fn paradas(stops: &[ph2d_vec_scene::GradientStop], defs: &mut String) {
    for s in stops {
        let _ = write!(
            defs,
            r#"<stop offset="{}" stop-color="{}" stop-opacity="{}"/>"#,
            num(s.offset),
            cor(s.color),
            alfa(s.color)
        );
    }
}

/// O traço como atributos SVG, mais o que ele perdeu.
fn traco(s: &ph2d_vec_scene::StrokeSpec) -> (String, Option<&'static str>) {
    let c = s.color();
    let cap = match s.cap {
        ph2d_vec_scene::LineCap::Butt => "butt",
        ph2d_vec_scene::LineCap::Round => "round",
        ph2d_vec_scene::LineCap::Square => "square",
    };
    let join = match s.join {
        // O bico CORTADO (F12) é o `miter-clip` do SVG 2; um visor sem ele cai no `miter`.
        ph2d_vec_scene::LineJoin::Miter if ph2d_vec_scene::miter_clip::enabled() => "miter-clip",
        ph2d_vec_scene::LineJoin::Miter => "miter",
        ph2d_vec_scene::LineJoin::Round => "round",
        ph2d_vec_scene::LineJoin::Bevel => "bevel",
    };
    let mut a = format!(
        r#"stroke="{}" stroke-opacity="{}" stroke-width="{}" stroke-linecap="{cap}" stroke-linejoin="{join}""#,
        cor(c),
        alfa(c),
        num(s.width)
    );
    // ⚠️ O SVG tem `4` por omissão e o documento tem [`ph2d_vec_scene::MITER_LIMIT`]: sem isto o
    // ficheiro exportado cortava o bico que o ecrã mostra.
    if matches!(s.join, ph2d_vec_scene::LineJoin::Miter) {
        let _ = write!(
            a,
            r#" stroke-miterlimit="{}""#,
            num(ph2d_vec_scene::MITER_LIMIT)
        );
    }
    // ⚠️ O tracejado do documento é em MÚLTIPLOS da largura; o do SVG é em unidades.
    // F16c: até três pares, os que o padrão percorre.
    if let Some(p) = s.dash.filter(ph2d_vec_scene::DashPattern::is_dashed) {
        let lista: Vec<String> = p
            .active()
            .iter()
            .flat_map(|[d, g]| [num(d * s.width), num(g * s.width)])
            .collect();
        let _ = write!(a, r#" stroke-dasharray="{}""#, lista.join(","));
    }
    let perdeu = matches!(s.paint, StrokePaint::Pattern(_)).then_some("padrão no traço");
    let perdeu =
        perdeu.or(matches!(s.paint, StrokePaint::Brush(_)).then_some("pincel de contorno"));
    (a, perdeu)
}

/// **O SVG do desenho.**
///
/// `escondido` e `preenchimento` são as MESMAS perguntas que o balde faz
/// ([`crate::bucket`]): uma forma que não se vê não é exportada, e uma que é área de balde
/// leva a marca `data-ph2d-fill` — é ela que deixa quem lê o ficheiro distinguir a LINHA da TINTA
/// sem adivinhar pela cor.
pub fn svg(
    scene: &VecScene,
    xforms: &VecXforms,
    escondido: &dyn Fn(u64) -> bool,
    preenchimento: &dyn Fn(u64) -> bool,
) -> Svg {
    svg_at(scene, xforms, escondido, preenchimento, EIXOS)
}

/// ⭐ O mesmo SVG com `px_por_unidade` unidades do ficheiro por unidade de MUNDO — o «Send to board…»
/// (W15) escreve-o aos `pixels_per_meter` do projecto, para o tamanho natural no quadro ser o da
/// régua do editor em píxeis. A escala passa pela MESMA porta dos eixos (`world_to_svg`).
pub fn svg_at(
    scene: &VecScene,
    xforms: &VecXforms,
    escondido: &dyn Fn(u64) -> bool,
    preenchimento: &dyn Fn(u64) -> bool,
    px_por_unidade: f64,
) -> Svg {
    let mundo: Vec<(u64, VecPath)> = scene
        .paths()
        .iter()
        .filter(|p| !escondido(p.id))
        .map(|p| {
            let mut c = p.cooked().into_owned();
            // ⚠️ DUAS transformações, e a ordem é a única possível: primeiro a POSE do objecto (que
            // vive em mundo), depois a lei dos EIXOS (mundo → ficheiro). Assá-las em separado pela
            // mesma porta mantém a geometria, o gradiente e a largura do traço em acordo.
            ph2d_vec_scene::bake_xform(&mut c, &ph2d_vec_scene::xform_of(xforms, p.id));
            ph2d_vec_scene::bake_xform(&mut c, &ph2d_vec_svg::world_to_svg(px_por_unidade));
            (p.id, c)
        })
        .collect();
    // ⭐ A caixa do que cada forma PINTA (a pilha inteira: camadas deslocadas, dilatadas, traços
    // Inner/Outer — `path_visual_bounds`); só o traço de base cortava uma camada grossa do ficheiro.
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for (_, p) in &mundo {
        if let Some(r) = ph2d_vec_render::path_visual_bounds(p, ph2d_vector::Affine::IDENTITY) {
            lo = [lo[0].min(r.x0), lo[1].min(r.y0)];
            hi = [hi[0].max(r.x1), hi[1].max(r.y1)];
        }
    }
    if !lo[0].is_finite() {
        (lo, hi) = ([0.0, 0.0], [1.0, 1.0]);
    }
    // Uma unidade de mundo de folga (o anti-aliasing da borda), na escala do ficheiro: o tamanho
    // natural cresce na mesma proporção que o desenho.
    let margem = px_por_unidade.abs();
    let (x, y) = (lo[0] - margem, lo[1] - margem);
    let (w, h) = (
        (hi[0] - lo[0] + 2.0 * margem).max(1.0),
        (hi[1] - lo[1] + 2.0 * margem).max(1.0),
    );

    let mut defs = String::new();
    let mut corpo = String::new();
    let mut aproximadas: Vec<(u64, &'static str)> = Vec::new();
    for (id, p) in &mundo {
        let marca = if preenchimento(*id) {
            r#" data-ph2d-fill="1""#
        } else {
            ""
        };
        let regra = match p.fill_rule {
            ph2d_vec_scene::FillRule::EvenOdd => "evenodd",
            ph2d_vec_scene::FillRule::NonZero => "nonzero",
        };
        // ⭐⭐⭐ **UM ELEMENTO POR CAMADA, e a lista sai da PORTA** ([`VecPath::paint_stack`], v20):
        // o chão (preenchimento, depois contorno) e a seguir cada camada da pilha de aparência.
        //
        // ⭐⭐ **E são elementos SEPARADOS, que é a lei do renderer.** O preenchimento só leva os
        // contornos FECHADOS (`build_fill_bezpath`); o traço leva TODOS. Um SVG com um elemento só
        // fecharia implicitamente cada contorno aberto e abriria regiões que o app não pinta.
        for (k, camada) in p.paint_stack().enumerate() {
            // ⭐ O SVG exprime as DUAS propriedades de uma camada, então nada se perde aqui: o
            // `opacity` de um elemento e o `mix-blend-mode` do CSS são exactamente o que a entrada
            // guarda. ⛔ Um modo sem nome em CSS sai NOMEADO em vez de virar `normal` calado.
            let mut extra = String::new();
            if camada.opacity < 1.0 {
                let _ = write!(extra, r#" opacity="{}""#, num(f64::from(camada.opacity)));
            }
            if camada.blend != ph2d_vec_scene::BlendMode::Normal {
                match ph2d_vec_svg::css_blend_name(camada.blend) {
                    Some(nome) => {
                        let _ = write!(extra, r#" style="mix-blend-mode:{nome}""#);
                    }
                    None => aproximadas.push((*id, "modo de mistura sem equivalente em CSS")),
                }
            }
            // ⭐⭐⭐ **E ONDE ela desenha** (v21). O SVG exprime-o directamente — um `transform` de
            // translação no elemento —, então também aqui nada se perde.
            //
            // ⭐⭐ **E ele sai CRU, sem tocar na lei dos eixos — o que é a PROVA de que o sítio dele
            // é o `bake_xform`.** Este `p` já passou pelos DOIS assadores (a pose, depois
            // `world_to_svg`), e o deslocamento viaja com a geometria porque o assador o leva
            // (`apply_vec`) ⇒ quando chega aqui, ele **já está em espaço de ficheiro**.
            //
            // ⛔ A 1.ª redacção desta linha multiplicava por `EIXOS` e invertia o `y` outra vez —
            // a lei do eixo escrita pela **quinta** vez nesta casa, sobre um valor que já a tinha
            // sofrido. *Uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é*, e o
            // cabeçalho deste ficheiro diz exactamente isso, três parágrafos acima.
            if camada.offset != [0.0, 0.0] {
                let [dx, dy] = camada.offset;
                let _ = write!(extra, r#" transform="translate({} {})""#, num(dx), num(dy));
            }
            match camada.paint {
                ph2d_vec_scene::PaintRef::Fill(paint) => {
                    let d = ph2d_vec_render::build_contours(p, Some(true)).to_svg();
                    if d.is_empty() {
                        continue;
                    }
                    let (attrs, perdeu) = tinta(paint, *id, k, &mut defs);
                    if let Some(o) = perdeu {
                        aproximadas.push((*id, o));
                    }
                    let _ = write!(
                        corpo,
                        "\n  <path data-ph2d-id=\"{id}\"{marca}{extra} fill-rule=\"{regra}\" {attrs} stroke=\"none\" d=\"{d}\"/>"
                    );
                }
                ph2d_vec_scene::PaintRef::Stroke(s) => {
                    // ⚠️ Inner/Outer não existem em SVG 1.1: sai a FAIXA que o ecrã pinta
                    // (`align_live`, a mesma porta), preenchida com a cor do traço. Até 15/10 saía
                    // o traço CENTRADO (`BUGS_vector.md` #43).
                    let faixa = if camada.is_base {
                        ph2d_vec_boolean::aligned_stroke(p)
                    } else {
                        None
                    };
                    if let Some(faixa) = faixa {
                        if let Some(o) = traco(s).1 {
                            aproximadas.push((*id, o));
                        }
                        for f in &faixa {
                            let d = ph2d_vec_render::build_contours(f, Some(true)).to_svg();
                            let Some(paint) = f.fill.as_ref().filter(|_| !d.is_empty()) else {
                                continue;
                            };
                            let (attrs, _) = tinta(paint, *id, k, &mut defs);
                            let regra = match f.fill_rule {
                                ph2d_vec_scene::FillRule::EvenOdd => "evenodd",
                                ph2d_vec_scene::FillRule::NonZero => "nonzero",
                            };
                            let _ = write!(
                                corpo,
                                "\n  <path data-ph2d-id=\"{id}\"{marca}{extra} fill-rule=\"{regra}\" {attrs} stroke=\"none\" d=\"{d}\"/>"
                            );
                        }
                        continue;
                    }
                    let d = ph2d_vec_render::build_contours(p, None).to_svg();
                    if d.is_empty() {
                        continue;
                    }
                    let (attrs, perdeu) = traco(s);
                    if let Some(o) = perdeu {
                        aproximadas.push((*id, o));
                    }
                    let _ = write!(
                        corpo,
                        "\n  <path data-ph2d-id=\"{id}\"{marca}{extra} fill=\"none\" {attrs} d=\"{d}\"/>"
                    );
                }
            }
        }
    }

    // ⚠️ A nota diz as DUAS coisas separadamente, porque elas são separadas: a ESCALA é a do mundo
    // (uma unidade é uma unidade) e o EIXO Y é o do SVG (desce). A frase anterior colava-as numa
    // só — *"coordenadas de MUNDO (Y para baixo, como o SVG)"* — e assim afirmava que o mundo tem
    // o Y a descer, que é falso e era exactamente o defeito.
    let mut nota = if px_por_unidade == EIXOS {
        String::from(
            "  Geometria COZIDA. Uma unidade do ficheiro = uma unidade de MUNDO; o eixo Y desce,\n  \
             como manda o SVG (no mundo ele sobe).\n",
        )
    } else {
        format!(
            "  Geometria COZIDA. {} unidades do ficheiro = uma unidade de MUNDO; o eixo Y desce,\n  \
             como manda o SVG (no mundo ele sobe).\n",
            num(px_por_unidade)
        )
    };
    if aproximadas.is_empty() {
        nota.push_str("  Nada foi aproximado.\n");
    } else {
        nota.push_str(
            "  APROXIMADO (sem equivalente em SVG), com a cor de recurso do proprio app:\n",
        );
        for (id, o) in &aproximadas {
            let _ = writeln!(nota, "    caminho {id}: {o}");
        }
    }
    let texto = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!--\n  PH2D — exportacao vectorial\n{nota}-->\n\
         <svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{} {} {} {}\" width=\"{}\" height=\"{}\">\
         {}{}\n</svg>\n",
        num(x),
        num(y),
        num(w),
        num(h),
        num(w),
        num(h),
        if defs.is_empty() {
            String::new()
        } else {
            format!("\n  <defs>{defs}</defs>")
        },
        corpo,
    );
    Svg {
        texto,
        formas: mundo.len(),
        aproximadas,
    }
}

/// ⭐ O SVG de UM objecto vectorial (o caminho `path`), aos `px_por_unidade` — o «Send to board…»
/// (W15). Sai mesmo que o olho da Hierarquia o esconda: foi ESTE o objecto pedido. `None` = o caminho
/// não está na cena ou não desenha nada.
#[must_use]
pub fn svg_of_path(
    scene: &VecScene,
    xforms: &VecXforms,
    path: u64,
    preenchimento: &dyn Fn(u64) -> bool,
    px_por_unidade: f64,
) -> Option<String> {
    let out = svg_at(
        scene,
        xforms,
        &|id| id != path,
        preenchimento,
        px_por_unidade,
    );
    (out.formas > 0).then_some(out.texto)
}

#[cfg(test)]
#[path = "svg_export_tests.rs"]
mod tests;

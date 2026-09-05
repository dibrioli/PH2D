//! ⭐⭐⭐ **EXPORTAR O DESENHO VECTORIAL COMO SVG** (plano 40).
//!
//! Pedido do Enio (2026-09-02): *"precisamos de um meio de exportar o path para que vc possa
//! analisar melhor. Veja se já há exportação no app"*. Não havia: o app exportava **imagem** (16
//! formatos), **folha de sprites**, **peças 3D** e os **tokens DTCG** — nada que levasse uma curva.
//!
//! # A lei
//!
//! > **O que sai é o que se VÊ: a geometria COZIDA, no MUNDO.**
//!
//! ⚠️ **Cozida** — a pilha de Live Path Effects e o raio de quina já correram, e é por isso que um
//! ficheiro exportado de uma forma com efeitos abre igual ao que está no ecrã. ⚠️ **No mundo** — a
//! pose de cada objecto está assada na geometria, então não há um `transform` por elemento a
//! discordar do que a régua do editor mede.
//!
//! ⚠️⚠️ **O `d` sai da MESMA porta que o renderer usa** ([`ph2d_vec_render::build_contours`]) — uma
//! segunda travessia dos contornos daria um ficheiro que discorda do ecrã em curvas que nenhum
//! olho apanha, e essa é a classe de defeito que este módulo existe para NÃO ter.
//!
//! # ⛔⛔ O EIXO Y, e o defeito que esta linha corrigiu em 2026-09-05
//!
//! Até hoje o cabeçalho deste ficheiro dizia *"em coordenadas de MUNDO (Y para baixo, como o
//! SVG)"* — e as duas metades da frase **contradizem-se**. O mundo do PH2D mede o Y para **CIMA**
//! (`ph2d_render::Camera2d::world_to_screen_affine` é `scale(k, **−k**)`), o SVG mede-o para
//! baixo, e escrever as coordenadas cruas fazia **todo ficheiro exportado sair verticalmente
//! espelhado**.
//!
//! ⚠️ **Ninguém o viu porque o consumidor era uma LLM a ler números** (o pedido do Enio de 02/09 foi
//! *"precisamos de um meio de exportar o path para que vc possa analisar melhor"*), e **nenhum
//! gate media orientação** — os seis que existiam mediam tinta, pose, marca e a nota do cabeçalho.
//!
//! ⇒ A conversão passa pela porta ÚNICA ([`ph2d_vec_svg::world_to_svg`]), que é a inversa exacta da
//! que o IMPORTADOR usa. *Uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é.*
//!
//! # ⛔ O que ele NÃO carrega, e diz
//!
//! Um exportador que ignora em silêncio é pior do que um que recusa (a lei do importador `.ase`).
//! Um padrão, um pincel de contorno e um gradiente multi-ponto **não têm equivalente em SVG 1.1**:
//! saem com a **mesma cor sólida que o renderer usa quando o ladrilho não resolve**
//! (`Paint::primary_color` / `PatternFill::fallback`), e o cabeçalho do ficheiro **nomeia** cada
//! forma em que isso aconteceu.

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
pub(crate) struct Svg {
    pub(crate) texto: String,
    /// Quantas formas entraram.
    pub(crate) formas: usize,
    /// As formas cuja tinta não tem equivalente, `(nome do caminho, o que era)`.
    pub(crate) aproximadas: Vec<(u64, &'static str)>,
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
    // ⚠️ O tracejado do documento é em MÚLTIPLOS da largura; o do SVG é em unidades.
    if let Some((d, g)) = s.dash
        && d > 0.0
    {
        let _ = write!(
            a,
            r#" stroke-dasharray="{},{}""#,
            num(d * s.width),
            num(g * s.width)
        );
    }
    let perdeu = matches!(s.paint, StrokePaint::Pattern(_)).then_some("padrão no traço");
    let perdeu =
        perdeu.or(matches!(s.paint, StrokePaint::Brush(_)).then_some("pincel de contorno"));
    (a, perdeu)
}

/// **O SVG do desenho.**
///
/// `escondido` e `preenchimento` são as MESMAS perguntas que o balde faz
/// ([`crate::vec_bucket`]): uma forma que não se vê não é exportada, e uma que é área de balde
/// leva a marca `data-ph2d-fill` — é ela que deixa quem lê o ficheiro distinguir a LINHA da TINTA
/// sem adivinhar pela cor.
pub(crate) fn svg(
    scene: &VecScene,
    xforms: &VecXforms,
    escondido: &dyn Fn(u64) -> bool,
    preenchimento: &dyn Fn(u64) -> bool,
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
            ph2d_vec_scene::bake_xform(&mut c, &ph2d_vec_svg::world_to_svg(EIXOS));
            (p.id, c)
        })
        .collect();
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for (_, p) in &mundo {
        for v in p.verts_all() {
            for q in [v.anchor, v.in_handle, v.out_handle] {
                lo = [lo[0].min(q[0]), lo[1].min(q[1])];
                hi = [hi[0].max(q[0]), hi[1].max(q[1])];
            }
        }
    }
    if !lo[0].is_finite() {
        (lo, hi) = ([0.0, 0.0], [1.0, 1.0]);
    }
    // A margem cobre metade da largura do traço mais gordo — senão um contorno grosso sai cortado.
    let margem = mundo
        .iter()
        .filter_map(|(_, p)| p.stroke.as_ref().map(|s| s.width))
        .fold(1.0_f64, f64::max);
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
    let mut nota = String::from(
        "  Geometria COZIDA. Uma unidade do ficheiro = uma unidade de MUNDO; o eixo Y desce,\n  \
         como manda o SVG (no mundo ele sobe).\n",
    );
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

impl crate::App {
    /// ⭐⭐⭐ **O GESTO de exportar** — *File > Export SVG…*.
    ///
    /// ⚠️ **Pergunta SEMPRE o caminho**, ao contrário do `Save`: um export não tem *"o ficheiro da
    /// sessão"* — o projecto tem, e não é o mesmo ficheiro. Gravar por cima do último SVG sem
    /// perguntar seria o app a decidir onde o trabalho de outra pessoa vive.
    ///
    /// ⚠️ **Sem forma visível ele DIZ**, e não escreve um ficheiro vazio: um SVG de zero formas
    /// abre em branco, e o artista conclui que a exportação se partiu.
    pub(crate) fn export_svg_gesture(&mut self) {
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        let xf = crate::vec_transform::build(&gfx.sim, &self.vec_entities);
        let vista = crate::vec_entities::view_state(&gfx.sim, &self.vec_entities);
        // ⚠️ As DUAS perguntas são as mesmas que o balde faz: o que não se vê não sai, e o que é
        // área de balde sai MARCADO (`VecViewState::is_derived` é populado do `VecBucketFill`).
        let out = svg(&gfx.vec_scene, &xf, &|id| vista.is_hidden(id), &|id| {
            vista.is_derived(id)
        });
        if out.formas == 0 {
            self.toast("Export SVG: the drawing has no visible shape".to_string());
            return;
        }
        let sugerido = self
            .project_path
            .as_deref()
            .and_then(|p| std::path::Path::new(p).file_stem()?.to_str())
            .map_or_else(|| "drawing.svg".to_string(), |s| format!("{s}.svg"));
        let Some(path) = crate::modal::save_file(
            rfd::FileDialog::new()
                .set_file_name(&sugerido)
                .add_filter("SVG (.svg)", &["svg"]),
        ) else {
            return; // o artista desistiu — e desistir não é um erro
        };
        match std::fs::write(&path, out.texto.as_bytes()) {
            Ok(()) => {
                let extra = if out.aproximadas.is_empty() {
                    String::new()
                } else {
                    format!(" ({} approximated)", out.aproximadas.len())
                };
                eprintln!(
                    "[ph2d-vec] SVG: {} ({} forma[s], {} bytes){extra}",
                    path.display(),
                    out.formas,
                    out.texto.len()
                );
                self.toast(format!(
                    "Exported {} shape(s) to {}{extra}",
                    out.formas,
                    path.file_name().unwrap_or_default().to_string_lossy()
                ));
            }
            Err(e) => {
                eprintln!("[ph2d-vec] SVG: erro ao gravar {}: {e}", path.display());
                self.toast(format!("Export SVG FAILED: {e}"));
            }
        }
    }
}

#[cfg(test)]
#[path = "vec_svg_export_tests.rs"]
mod tests;

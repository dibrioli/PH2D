//! Texto vetorial — o CONVERTER puro: transforma o contorno de um glyph (skrifa via
//! `ph2d-vector-font`) em UM `VecPath` compound do motor novo. O contorno externo vai
//! em `verts`, os furos (miolo do `o`, `e`, `a`…) em `subpaths`, com
//! `FillRule::NonZero` — a canônica OT/SVG que resolve buracos. Assim um glyph é uma
//! forma vetorial como qualquer outra: herda render, gizmo, snap, undo e Hierarquia.
//! A sessão de edição que consome isto vive no módulo irmão [`crate::vec_text`].
//!
//! **Coordenadas.** Unidades de design são y-up e o world do editor TAMBÉM é y-up
//! (quem inverte pra tela é a câmera, `world_to_screen_affine` com `scale(k, −k)`).
//! Então mapeia `(x, y) → (x·scale + origin.x, y·scale + origin.y)`, SEM flip — flipar
//! aqui somaria com o da câmera e o texto sairia de cabeça pra baixo.
//!
//! **Handles.** O `VecVertex` guarda handles ABSOLUTOS (estilo Rive `CubicVertex`),
//! não vetores-tangente. Quad `S–Q–E` sobe pra cúbica com controles absolutos
//! `S + ⅔(Q−S)` e `E + ⅔(Q−E)`; a cúbica usa `c1`/`c2` diretos.

use ph2d_tool_vector::TextAlign;
use ph2d_vec_edit::PenStyle;
use ph2d_vec_scene::arc_path::ArcPath;
use ph2d_vec_scene::text_path::GlyphFrame;
use ph2d_vec_scene::{Contour, FillRule, Paint, StrokeSpec, VecPath};
use ph2d_vector_font::{AxisTag, VariableFont};

/// **Onde o bloco de texto assenta** — um ponto de mundo, ou um caminho que ele cavalga.
///
/// É um enum e não um par `(origem, Option<caminho>)` porque as duas coisas são respostas à
/// MESMA pergunta: sobre um caminho a origem do bloco não é ignorada por convenção, ela deixa
/// de existir (o `startOffset` a substitui). Um par deixaria exprimível um estado — *"tenho
/// origem E caminho"* — que nada sabe honrar, e é dele que nasce o bug em que metade do código
/// lê um e metade lê o outro.
#[derive(Clone, Copy, Debug)]
pub(crate) enum TextPlacement<'a> {
    /// Bloco reto, ancorado num ponto de mundo (a baseline da 1ª linha).
    At([f64; 2]),
    /// Cavalgando um caminho parametrizado por arco.
    OnPath {
        path: &'a ArcPath,
        /// Onde a 1ª linha começa, em comprimento de arco (o `startOffset` do SVG).
        start_offset: f64,
        /// Texto do outro lado, a ler no sentido oposto.
        flip: bool,
    },
}

/// **O referencial de UM glyph** (ou do caret, que é um glyph de avanço zero).
///
/// Esta é a porta única do posicionamento: o laço de glyphs e o caret perguntam à MESMA
/// função. É o que impede o defeito óbvio desta feature — *o cursor de digitação ficar no
/// texto reto enquanto as letras já estão na curva* —, que enumerar dois sítios não impede,
/// só promete.
///
/// `pen` é o deslocamento acumulado do bloco: `pen[0]` ao longo da linha (já com o
/// alinhamento somado), `pen[1]` a entrelinha (negativa para baixo, world y-up).
///
/// `None` significa **não desenhe este glyph**, por duas razões distintas e ambas honestas:
/// a âncora caiu **fora** do caminho (a regra normativa do SVG; saturar empilharia as letras
/// que sobram num montinho na ponta) ou caiu numa **cúspide**, onde não há direção e um
/// ângulo inventado poria a letra num rumo que ninguém autorou.
#[must_use]
fn glyph_frame(placement: &TextPlacement<'_>, pen: [f64; 2], advance: f64) -> Option<GlyphFrame> {
    match *placement {
        // ⚠️ A soma é `origem + pen`, exatamente como era quando isto devolvia um ponto —
        // reassociar (pôr o `pen[1]` no y LOCAL, por exemplo) move o layout reto por um ulp.
        TextPlacement::At(o) => Some(GlyphFrame {
            origin: [o[0] + pen[0], o[1] + pen[1]],
            x_axis: [1.0, 0.0],
            y_axis: [0.0, 1.0],
        }),
        TextPlacement::OnPath {
            path,
            start_offset,
            flip,
        } => {
            // A âncora é o MEIO do glyph (o `mid` normativo do `<textPath>`); o contorno é
            // desenhado do pen origin dele, então o referencial recua meio avanço.
            let half = advance * 0.5;
            let s = start_offset + pen[0] + half;
            if s < 0.0 || s > path.total() {
                return None;
            }
            Some(GlyphFrame::on_path(path, s, pen[1], flip)?.shifted_along(-half))
        }
    }
}

/// Os knobs de layout de um bloco de texto (tudo o que NÃO é fonte/eixo/cor). Agrupa
/// tamanho + tipografia num só parâmetro para o converter não estourar o número de
/// argumentos. `line_height` é múltiplo do tamanho; `tracking` é fração do tamanho (em)
/// somada entre glyphs; `align` posiciona cada linha em relação à origem (o clique).
#[derive(Clone, Copy, Debug)]
pub(crate) struct TextLayout {
    pub size: f64,
    pub line_height: f64,
    pub tracking: f64,
    pub align: TextAlign,
}

/// Resolve o Style do Pen (fill/stroke/width em px) no par (fill, stroke) que cada
/// glyph-path recebe — mesma regra das formas (`shape.rs`): sem preenchimento quando
/// alpha 0; traço sempre presente (o render pula width/alpha 0). Assim o texto herda
/// Fill/Stroke/Width/Cap/Join do painel do vetor como qualquer forma.
#[must_use]
pub(crate) fn resolve_style(
    style: &PenStyle,
    px_to_world: f64,
) -> (Option<Paint>, Option<StrokeSpec>) {
    let fill = (style.fill.a != 0).then(|| Paint::solid(style.fill));
    let stroke = Some(style.stroke_spec(style.stroke_w_px * px_to_world));
    (fill, stroke)
}

/// Layout de uma string em `VecPath`s — um por glyph com contorno. Cada linha (`\n`
/// separa) é medida e deslocada pelo alinhamento; `tracking` abre/fecha o espaço entre
/// glyphs; `line_height` (× tamanho) é a entrelinha. `axes` (ex. `wght`) valem no
/// contorno E no avanço (a métrica muda com o peso). Advance-only, sem shaping complexo.
///
/// Sobre um caminho ([`TextPlacement::OnPath`]) **nada disto muda**: alinhamento, tracking e
/// entrelinha continuam a produzir o mesmo `pen`, e só a tradução `pen → mundo` é outra. É
/// por isso que a 2ª linha de um texto em caminho corre paralela à 1ª, de graça — a
/// entrelinha vira deslocamento pela NORMAL da curva sem que este laço saiba disso.
#[must_use]
pub(crate) fn text_to_vec_paths(
    font: &VariableFont,
    text: &str,
    layout: &TextLayout,
    axes: &[(AxisTag, f32)],
    placement: &TextPlacement<'_>,
    fill: &Option<Paint>,
    stroke: &Option<StrokeSpec>,
) -> Vec<VecPath> {
    let scale = layout.size / f64::from(font.units_per_em().max(1));
    let line_h = layout.size * layout.line_height;
    let track_px = layout.size * layout.tracking;
    let mut out = Vec::new();
    let mut pen_y = 0.0;
    for line in text.split('\n') {
        let width = line_advance(font, line, scale, track_px, axes);
        let mut pen_x = align_offset(layout.align, width);
        for ch in line.chars() {
            let Some(gid) = font.glyph_for_char(ch) else {
                continue;
            };
            let advance = f64::from(font.advance(gid, axes).unwrap_or(0.0)) * scale;
            if let Some(frame) = glyph_frame(placement, [pen_x, pen_y], advance)
                && let Ok(outline) = font.outline(gid, axes)
                && let Some(path) =
                    glyph_to_vec_path(&outline, scale, &frame, fill.clone(), *stroke)
            {
                out.push(path);
            }
            // O pen avança MESMO quando o glyph não é desenhado — um glyph fora do caminho
            // (ou sem contorno) não pode encolher o texto e puxar os seguintes para trás.
            pen_x += advance + track_px;
        }
        pen_y -= line_h; // linhas descem = y menor (world y-up)
    }
    out
}

/// O referencial do CARET na ponta da última linha — o mesmo do glyph que ali seria
/// carimbado, porque um cursor é um glyph de avanço zero.
///
/// `None` pelas mesmas razões do [`glyph_frame`]: cursor fora do caminho, ou numa cúspide.
#[must_use]
pub(crate) fn caret_frame(placement: &TextPlacement<'_>, pen: [f64; 2]) -> Option<GlyphFrame> {
    glyph_frame(placement, pen, 0.0)
}

/// O texto inteiro como UM `VecPath` compound (todos os contornos de todos os glyphs
/// — externos + furos — num só path, `NonZero`). É a geometria do texto VIVO: um
/// objeto, um pick, um gizmo. `None` se não houver glyph com área (string vazia/só
/// espaços). Para "Convert to Curves", use [`text_to_vec_paths`] (um path por glyph).
#[must_use]
pub(crate) fn text_to_compound_path(
    font: &VariableFont,
    text: &str,
    layout: &TextLayout,
    axes: &[(AxisTag, f32)],
    placement: &TextPlacement<'_>,
    fill: &Option<Paint>,
    stroke: &Option<StrokeSpec>,
) -> Option<VecPath> {
    let glyphs = text_to_vec_paths(font, text, layout, axes, placement, &None, &None);
    // Concatena o contorno externo + os furos de cada glyph num único compound.
    let mut contours: Vec<Contour> = Vec::new();
    for g in glyphs {
        contours.push(Contour {
            verts: g.verts,
            closed: g.closed,
        });
        contours.extend(g.subpaths);
    }
    let mut iter = contours.into_iter();
    let first = iter.next()?;
    Some(VecPath {
        verts: first.verts,
        closed: true,
        fill: fill.clone(),
        stroke: *stroke,
        subpaths: iter.collect(),
        fill_rule: FillRule::NonZero,
        ..Default::default()
    })
}

/// Centro da bbox (âncoras + alças) de um `VecPath` — o ponto que vira o pivô da
/// forma viva (Live Shapes: a geometria nasce centrada no local 0). `[0,0]` se vazio.
#[must_use]
pub(crate) fn path_center(path: &VecPath) -> [f64; 2] {
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    for v in path.verts_all() {
        for p in [v.anchor, v.in_handle, v.out_handle] {
            lo[0] = lo[0].min(p[0]);
            lo[1] = lo[1].min(p[1]);
            hi[0] = hi[0].max(p[0]);
            hi[1] = hi[1].max(p[1]);
        }
    }
    if lo[0].is_finite() {
        [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]
    } else {
        [0.0, 0.0]
    }
}

/// Desloca toda a geometria de `path` (âncoras + alças, em todos os contornos) por `d`.
pub(crate) fn offset_path(path: &mut VecPath, d: [f64; 2]) {
    path.for_each_vert_mut(|v| {
        v.anchor = [v.anchor[0] + d[0], v.anchor[1] + d[1]];
        v.in_handle = [v.in_handle[0] + d[0], v.in_handle[1] + d[1]];
        v.out_handle = [v.out_handle[0] + d[0], v.out_handle[1] + d[1]];
    });
}

/// Largura visual de UMA linha em world: soma dos avanços + `tracking` entre glyphs
/// (`n−1` gaps). `axes` no mesmo `location` do layout (o avanço muda com o peso).
fn line_advance(
    font: &VariableFont,
    line: &str,
    scale: f64,
    track_px: f64,
    axes: &[(AxisTag, f32)],
) -> f64 {
    let mut width = 0.0;
    let mut glyphs = 0usize;
    for ch in line.chars() {
        if let Some(g) = font.glyph_for_char(ch) {
            width += f64::from(font.advance(g, axes).unwrap_or(0.0)) * scale;
            glyphs += 1;
        }
    }
    width + track_px * glyphs.saturating_sub(1) as f64
}

/// Deslocamento em x da 1ª coluna de uma linha de largura `width`, dado o alinhamento:
/// esquerda começa na origem, centro a centraliza, direita a termina nela.
fn align_offset(align: TextAlign, width: f64) -> f64 {
    match align {
        TextAlign::Left => 0.0,
        TextAlign::Center => -width / 2.0,
        TextAlign::Right => -width,
    }
}

/// Deslocamento em x (a partir de `origin.x`) do CURSOR na ponta de `last_line`: o
/// offset de alinhamento + a largura da linha (com tracking). Usado pelo caret.
#[must_use]
pub(crate) fn caret_x_offset(
    font: &VariableFont,
    last_line: &str,
    layout: &TextLayout,
    axes: &[(AxisTag, f32)],
) -> f64 {
    let scale = layout.size / f64::from(font.units_per_em().max(1));
    let track_px = layout.size * layout.tracking;
    let width = line_advance(font, last_line, scale, track_px, axes);
    align_offset(layout.align, width) + width
}

/// O builder de glyph (contorno → `VecPath` compound) vive no módulo irmão —
/// re-exportado para o layout acima (e a shell) seguirem chamando `vec_glyph::`.
pub(crate) use crate::vec_glyph_build::glyph_to_vec_path;

#[cfg(test)]
#[path = "vec_glyph_tests.rs"]
mod tests;

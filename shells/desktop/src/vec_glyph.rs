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
use ph2d_vec_scene::{Contour, FillRule, Paint, StrokeSpec, VecPath, VecVertex, VertexKind};
use ph2d_vector_font::{AxisTag, GlyphOutline, PathCommand, VariableFont};

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
#[must_use]
pub(crate) fn text_to_vec_paths(
    font: &VariableFont,
    text: &str,
    layout: &TextLayout,
    axes: &[(AxisTag, f32)],
    origin: [f64; 2],
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
            if let Ok(outline) = font.outline(gid, axes)
                && let Some(path) = glyph_to_vec_path(
                    &outline,
                    scale,
                    [origin[0] + pen_x, origin[1] + pen_y],
                    fill.clone(),
                    *stroke,
                )
            {
                out.push(path);
            }
            pen_x += advance + track_px;
        }
        pen_y -= line_h; // linhas descem = y menor (world y-up)
    }
    out
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
    origin: [f64; 2],
    fill: &Option<Paint>,
    stroke: &Option<StrokeSpec>,
) -> Option<VecPath> {
    let glyphs = text_to_vec_paths(font, text, layout, axes, origin, &None, &None);
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

/// Converte o contorno de um glyph em um `VecPath` compound preenchido. `scale` =
/// `font_size / units_per_em`; `origin` = canto do glyph em world (y-down). Devolve
/// `None` quando o glyph não tem área preenchível (espaço, contorno degenerado).
#[must_use]
pub(crate) fn glyph_to_vec_path(
    outline: &GlyphOutline,
    scale: f64,
    origin: [f64; 2],
    fill: Option<Paint>,
    stroke: Option<StrokeSpec>,
) -> Option<VecPath> {
    let mut b = Build::new(scale, origin);
    for cmd in &outline.commands {
        match *cmd {
            PathCommand::MoveTo(p) => b.move_to(p.x, p.y),
            PathCommand::LineTo(p) => b.line_to(p.x, p.y),
            PathCommand::QuadTo { ctrl, to } => b.quad_to(ctrl.x, ctrl.y, to.x, to.y),
            PathCommand::CurveTo { c1, c2, to } => b.curve_to(c1.x, c1.y, c2.x, c2.y, to.x, to.y),
            PathCommand::Close => b.close(),
        }
    }
    b.finish(fill, stroke)
}

/// Estado do builder: contornos fechados já prontos + o que está aberto. Cada
/// vértice sai com handles absolutos já colocados pelo segmento que chega/sai dele.
struct Build {
    scale: f64,
    origin: [f64; 2],
    /// Tolerância² de "mesmo ponto" (¼ de unidade escalada) — só funde a ponta que
    /// volta ao início; âncoras inteiras distintas ficam ≥ 1 unidade apart.
    eps_sq: f64,
    contours: Vec<Vec<VecVertex>>,
    cur: Vec<VecVertex>,
    start: [f64; 2],
    cur_pos: [f64; 2],
    has_seg: bool,
    fused: bool,
}

impl Build {
    fn new(scale: f64, origin: [f64; 2]) -> Self {
        let e = scale * 0.25;
        Self {
            scale,
            origin,
            eps_sq: (e * e).max(1e-12),
            contours: Vec::new(),
            cur: Vec::new(),
            start: [0.0; 2],
            cur_pos: [0.0; 2],
            has_seg: false,
            fused: false,
        }
    }

    /// Unidade de design → world + origem. NÃO flipa Y: o world do editor é Y-up
    /// (igual à fonte); quem inverte pra tela é a câmera (`world_to_screen_affine`
    /// tem `scale(k, −k)`). Flipar aqui deixaria o texto de cabeça pra baixo.
    fn tf(&self, x: f32, y: f32) -> [f64; 2] {
        [
            f64::from(x) * self.scale + self.origin[0],
            f64::from(y) * self.scale + self.origin[1],
        ]
    }

    /// Fecha o contorno corrente e o guarda (≥ 2 vértices = tem área).
    fn flush(&mut self) {
        if self.cur.len() >= 2 {
            self.contours.push(std::mem::take(&mut self.cur));
        } else {
            self.cur.clear();
        }
        self.has_seg = false;
        self.fused = false;
    }

    fn move_to(&mut self, x: f32, y: f32) {
        self.flush();
        let pos = self.tf(x, y);
        self.cur = vec![VecVertex::corner(pos)];
        self.start = pos;
        self.cur_pos = pos;
    }

    /// Fecha o segmento no vértice corrente (out_handle) e cria/funde o de chegada
    /// (in_handle). Funde quando a ponta volta ao início do contorno.
    fn seg(&mut self, end: [f64; 2], out_ctrl: [f64; 2], in_ctrl: [f64; 2]) {
        if self.cur.is_empty() || self.fused {
            return; // sem MoveTo, ou o contorno já voltou ao início
        }
        if let Some(last) = self.cur.last_mut() {
            last.out_handle = out_ctrl;
        }
        if self.has_seg && dist2(end, self.start) <= self.eps_sq {
            if let Some(first) = self.cur.first_mut() {
                first.in_handle = in_ctrl;
            }
            self.cur_pos = self.start;
            self.fused = true;
        } else {
            let mut v = VecVertex::corner(end);
            v.in_handle = in_ctrl;
            self.cur.push(v);
            self.cur_pos = end;
        }
        self.has_seg = true;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let e = self.tf(x, y);
        let s = self.cur_pos;
        self.seg(e, s, e); // reto: handles coincidem com as âncoras
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let q = self.tf(cx, cy);
        let e = self.tf(x, y);
        let s = self.cur_pos;
        let out = [
            s[0] + (q[0] - s[0]) * 2.0 / 3.0,
            s[1] + (q[1] - s[1]) * 2.0 / 3.0,
        ];
        let inn = [
            e[0] + (q[0] - e[0]) * 2.0 / 3.0,
            e[1] + (q[1] - e[1]) * 2.0 / 3.0,
        ];
        self.seg(e, out, inn);
    }

    fn curve_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        let out = self.tf(c1x, c1y);
        let inn = self.tf(c2x, c2y);
        let e = self.tf(x, y);
        self.seg(e, out, inn);
    }

    fn close(&mut self) {
        self.flush();
    }

    fn finish(mut self, fill: Option<Paint>, stroke: Option<StrokeSpec>) -> Option<VecPath> {
        self.flush();
        if self.contours.is_empty() {
            return None;
        }
        // Marca as junções curva↔reta (EXATAMENTE um lado com alça) como Smooth: a
        // alça lateral zero passa a mostrar um ghost agarrável já na criação da letra
        // (Enio 2026-07-11). Não muda geometria/render — só habilita o toco lateral.
        // Quinas retas (ambas as alças zero) e pontos com as duas alças ficam intactos.
        for contour in &mut self.contours {
            for v in contour.iter_mut() {
                let in_zero = dist2(v.in_handle, v.anchor) <= 1e-12;
                let out_zero = dist2(v.out_handle, v.anchor) <= 1e-12;
                if in_zero != out_zero {
                    v.kind = VertexKind::Smooth;
                }
            }
        }
        let mut iter = self.contours.into_iter();
        let verts = iter.next()?;
        let subpaths: Vec<Contour> = iter
            .map(|verts| Contour {
                verts,
                closed: true,
            })
            .collect();
        Some(VecPath {
            verts,
            closed: true,
            fill,
            stroke,
            subpaths,
            fill_rule: FillRule::NonZero,
            ..Default::default()
        })
    }
}

fn dist2(a: [f64; 2], b: [f64; 2]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    dx * dx + dy * dy
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_core::Vec2;
    use ph2d_vec_scene::Rgba8;

    fn black() -> Paint {
        Paint::solid(Rgba8::new(0, 0, 0, 255))
    }

    fn outline(commands: Vec<PathCommand>) -> GlyphOutline {
        GlyphOutline::new(commands, 1000)
    }

    /// Um triângulo fechado vira UM contorno de 3 vértices (a ponta de fecho funde no
    /// início), sem subpaths, preenchido NonZero.
    #[test]
    fn a_triangle_is_one_contour_of_three_corners() {
        let p = glyph_to_vec_path(
            &outline(vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(600.0, 0.0)),
                PathCommand::LineTo(Vec2::new(300.0, 600.0)),
                PathCommand::Close,
            ]),
            1.0 / 1000.0,
            [0.0, 0.0],
            Some(black()),
            None,
        )
        .expect("triângulo tem área");
        assert_eq!(p.verts.len(), 3, "o fecho fundiu, sem 4º vértice");
        assert!(p.subpaths.is_empty());
        assert!(p.closed);
        assert_eq!(p.fill_rule, FillRule::NonZero);
        assert!(p.fill.is_some());
    }

    /// Sem flip: o world é y-up como a fonte (a câmera é que inverte pra tela). y=1000
    /// vira +1.0 em world — flipar aqui deixaria o texto de cabeça pra baixo.
    #[test]
    fn y_is_not_flipped_world_is_yup() {
        let p = glyph_to_vec_path(
            &outline(vec![
                PathCommand::MoveTo(Vec2::new(0.0, 1000.0)),
                PathCommand::LineTo(Vec2::new(1000.0, 1000.0)),
                PathCommand::LineTo(Vec2::new(0.0, 0.0)),
                PathCommand::Close,
            ]),
            1.0 / 1000.0,
            [0.0, 0.0],
            Some(black()),
            None,
        )
        .unwrap();
        assert!(
            p.verts.iter().any(|v| (v.anchor[1] - 1.0).abs() < 1e-6),
            "y=1000 (topo da fonte) -> +1.0 em world (y-up)"
        );
        assert!(p.verts.iter().any(|v| v.anchor[1].abs() < 1e-6));
    }

    /// Glyph com furo ("O"): contorno externo em `verts`, o miolo em `subpaths`.
    #[test]
    fn a_glyph_with_a_hole_puts_the_inner_contour_in_subpaths() {
        let p = glyph_to_vec_path(
            &outline(vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(800.0, 0.0)),
                PathCommand::LineTo(Vec2::new(800.0, 800.0)),
                PathCommand::LineTo(Vec2::new(0.0, 800.0)),
                PathCommand::Close,
                PathCommand::MoveTo(Vec2::new(200.0, 200.0)),
                PathCommand::LineTo(Vec2::new(200.0, 600.0)),
                PathCommand::LineTo(Vec2::new(600.0, 600.0)),
                PathCommand::LineTo(Vec2::new(600.0, 200.0)),
                PathCommand::Close,
            ]),
            1.0 / 1000.0,
            [0.0, 0.0],
            Some(black()),
            None,
        )
        .unwrap();
        assert_eq!(p.verts.len(), 4, "contorno externo");
        assert_eq!(p.subpaths.len(), 1, "um furo");
        assert_eq!(p.subpaths[0].verts.len(), 4);
        assert!(p.subpaths[0].closed);
    }

    /// A quad sobe pra cúbica com handles ABSOLUTOS: out = S + ⅔(Q−S), in = E + ⅔(Q−E).
    #[test]
    fn a_quad_lifts_to_absolute_cubic_handles() {
        let p = glyph_to_vec_path(
            &outline(vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::QuadTo {
                    ctrl: Vec2::new(600.0, 0.0),
                    to: Vec2::new(1200.0, 0.0),
                },
                PathCommand::LineTo(Vec2::new(0.0, 0.0)),
                PathCommand::Close,
            ]),
            1.0, // sem escala pra conta redonda
            [0.0, 0.0],
            Some(black()),
            None,
        )
        .unwrap();
        // S=(0,0), Q=(600,0) → out = (400, 0) absoluto.
        assert!((p.verts[0].out_handle[0] - 400.0).abs() < 1e-4);
        // E=(1200,0), Q=(600,0) → in = 1200 + ⅔(600−1200) = 800 absoluto.
        assert!((p.verts[1].in_handle[0] - 800.0).abs() < 1e-4);
    }

    /// Cúbica: os handles absolutos são os próprios controles (c1/c2).
    #[test]
    fn a_cubic_keeps_its_control_points_as_absolute_handles() {
        let p = glyph_to_vec_path(
            &outline(vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::CurveTo {
                    c1: Vec2::new(300.0, 0.0),
                    c2: Vec2::new(700.0, 0.0),
                    to: Vec2::new(1000.0, 0.0),
                },
                PathCommand::LineTo(Vec2::new(0.0, 0.0)),
                PathCommand::Close,
            ]),
            1.0,
            [0.0, 0.0],
            Some(black()),
            None,
        )
        .unwrap();
        assert!((p.verts[0].out_handle[0] - 300.0).abs() < 1e-4, "out = c1");
        assert!((p.verts[1].in_handle[0] - 700.0).abs() < 1e-4, "in = c2");
    }

    /// Junção curva↔reta vira Smooth (a alça lateral zero mostra ghost na criação da
    /// letra); quina line-line (triângulo) fica Corner. Não muda posições de alça.
    #[test]
    fn curve_line_junctions_are_smooth_so_lateral_ghosts_show() {
        // Curva de v0 a v1 + fecho reto v1→v0: cada ponta tem UM lado com alça.
        let p = glyph_to_vec_path(
            &outline(vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::CurveTo {
                    c1: Vec2::new(0.0, 500.0),
                    c2: Vec2::new(1000.0, 500.0),
                    to: Vec2::new(1000.0, 0.0),
                },
                PathCommand::Close,
            ]),
            1.0 / 1000.0,
            [0.0, 0.0],
            Some(black()),
            None,
        )
        .unwrap();
        assert!(
            p.verts
                .iter()
                .all(|v| v.kind == ph2d_vec_scene::VertexKind::Smooth),
            "juncoes curva-reta = Smooth"
        );
        // Um triângulo (line-line em toda quina) permanece Corner.
        let tri = glyph_to_vec_path(
            &outline(vec![
                PathCommand::MoveTo(Vec2::new(0.0, 0.0)),
                PathCommand::LineTo(Vec2::new(600.0, 0.0)),
                PathCommand::LineTo(Vec2::new(300.0, 600.0)),
                PathCommand::Close,
            ]),
            1.0 / 1000.0,
            [0.0, 0.0],
            Some(black()),
            None,
        )
        .unwrap();
        assert!(
            tri.verts
                .iter()
                .all(|v| v.kind == ph2d_vec_scene::VertexKind::Corner),
            "quinas retas ficam Corner"
        );
    }

    /// Um glyph sem contorno fechável (espaço) devolve `None`.
    #[test]
    fn an_empty_outline_is_none() {
        assert!(
            glyph_to_vec_path(&outline(vec![]), 1.0, [0.0, 0.0], Some(black()), None).is_none()
        );
    }

    /// O Style do painel vira o par (fill, stroke) do texto: sem preenchimento quando
    /// alpha 0, e traço SEMPRE presente (o render pula width/alpha 0) — como nas formas.
    #[test]
    fn the_panel_style_flows_into_the_glyph_paint() {
        let style = PenStyle {
            fill: Rgba8::new(200, 30, 30, 255),
            ..PenStyle::default()
        };
        let (fill, stroke) = resolve_style(&style, 0.01);
        assert!(fill.is_some(), "fill opaco vira Some");
        assert!(stroke.is_some(), "traço sempre presente");
        let clear = PenStyle {
            fill: Rgba8::new(0, 0, 0, 0),
            ..PenStyle::default()
        };
        assert!(
            resolve_style(&clear, 0.01).0.is_none(),
            "fill transparente = None"
        );
    }

    /// Multi-linha: a 2ª linha desce (world y-up → y negativo abaixo da baseline).
    #[test]
    fn a_second_line_sits_below_the_first() {
        let font = VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).expect("embutida");
        let font = &font;
        let min_y = |paths: &[VecPath]| {
            paths
                .iter()
                .flat_map(|p| p.verts.iter())
                .map(|v| v.anchor[1])
                .fold(f64::INFINITY, f64::min)
        };
        let lay = TextLayout {
            size: 1.0,
            line_height: 1.2,
            tracking: 0.0,
            align: TextAlign::Left,
        };
        let one = text_to_vec_paths(font, "A", &lay, &[], [0.0, 0.0], &Some(black()), &None);
        let two = text_to_vec_paths(font, "A\nA", &lay, &[], [0.0, 0.0], &Some(black()), &None);
        assert!(
            min_y(&one) >= -1e-6,
            "linha única: baseline em 0, sem descer"
        );
        assert!(
            min_y(&two) < -0.5,
            "a 2ª linha desce abaixo da baseline da 1ª"
        );
    }

    /// Centralizar uma linha a desloca para a ESQUERDA da origem (o bloco fica
    /// centrado no ponto de clique); alinhar à direita a termina na origem.
    #[test]
    fn alignment_shifts_the_line_horizontally() {
        let font = VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).expect("embutida");
        let min_x = |paths: &[VecPath]| {
            paths
                .iter()
                .flat_map(|p| p.verts.iter())
                .map(|v| v.anchor[0])
                .fold(f64::INFINITY, f64::min)
        };
        let lay = |align| TextLayout {
            size: 1.0,
            line_height: 1.2,
            tracking: 0.0,
            align,
        };
        let left = text_to_vec_paths(
            &font,
            "AA",
            &lay(TextAlign::Left),
            &[],
            [0.0, 0.0],
            &Some(black()),
            &None,
        );
        let center = text_to_vec_paths(
            &font,
            "AA",
            &lay(TextAlign::Center),
            &[],
            [0.0, 0.0],
            &Some(black()),
            &None,
        );
        let right = text_to_vec_paths(
            &font,
            "AA",
            &lay(TextAlign::Right),
            &[],
            [0.0, 0.0],
            &Some(black()),
            &None,
        );
        assert!(
            min_x(&center) < min_x(&left),
            "centralizado começa à esquerda do alinhado à esquerda"
        );
        assert!(
            min_x(&right) < min_x(&center),
            "à direita começa ainda mais à esquerda (termina na origem)"
        );
    }

    /// O texto vivo é UM path compound com todos os contornos: "Hi" = H (1) + ponto e
    /// haste do i (2) → 1 verts + ≥2 subpaths, um objeto só.
    #[test]
    fn compound_path_merges_all_glyph_contours() {
        let font = VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).expect("embutida");
        let lay = TextLayout {
            size: 1.0,
            line_height: 1.2,
            tracking: 0.0,
            align: TextAlign::Left,
        };
        let one = text_to_compound_path(&font, "Hi", &lay, &[], [0.0, 0.0], &Some(black()), &None)
            .unwrap();
        assert!(
            !one.subpaths.is_empty(),
            "vários glyphs/furos viram subpaths do mesmo path"
        );
        assert!(
            text_to_compound_path(&font, "   ", &lay, &[], [0.0, 0.0], &Some(black()), &None)
                .is_none(),
            "só espaços = sem geometria"
        );
    }

    /// Tracking positivo abre o espaço entre glyphs, então a mesma string ocupa mais
    /// largura (o cursor avança mais).
    #[test]
    fn positive_tracking_widens_the_line() {
        let font = VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).expect("embutida");
        let base = TextLayout {
            size: 1.0,
            line_height: 1.2,
            tracking: 0.0,
            align: TextAlign::Left,
        };
        let wide = TextLayout {
            tracking: 0.3,
            ..base
        };
        let narrow = caret_x_offset(&font, "AAA", &base, &[]);
        let opened = caret_x_offset(&font, "AAA", &wide, &[]);
        assert!(
            opened > narrow + 0.5,
            "tracking abre a linha (0.3·size × 2 gaps)"
        );
    }
}

//! O BUILDER de glyph — contorno de fonte (skrifa via `ph2d-vector-font`) → UM `VecPath`
//! compound. Módulo irmão de [`crate::vec_glyph`] (que fica com o LAYOUT: linhas,
//! alinhamento, tracking, centragem); separado pelo teto de 600 LOC/arquivo (HR-18).
//!
//! O contorno externo vai em `verts`, os furos (miolo do `o`, `e`, `a`…) em `subpaths`,
//! com `FillRule::NonZero` — a canônica OT/SVG que resolve buracos. Assim um glyph é uma
//! forma vetorial como qualquer outra: herda render, gizmo, snap, undo e Hierarquia.
//!
//! **Coordenadas.** Unidades de design são y-up e o world do editor TAMBÉM é y-up (quem
//! inverte pra tela é a câmera, `world_to_screen_affine` com `scale(k, −k)`). Mapeia
//! `(x, y) → frame.apply([x·scale, y·scale])`, SEM flip — flipar aqui somaria com o da
//! câmera e o texto sairia de cabeça pra baixo.
//!
//! **O glyph pousa por um AFIM, não por um ponto** ([`GlyphFrame`]). Num texto reto o
//! referencial é a identidade transladada e a aritmética é BYTE-IDÊNTICA à do ponto que
//! havia antes (`1.0.mul_add(a, b)` é exatamente `a + b`, `0.0.mul_add(a, b)` é `b`) —
//! pinado pelo fingerprint em [`crate::vec_glyph`]. Num texto em caminho o mesmo builder
//! recebe o referencial daquele glifo e nada mais muda: é por isso que Live Corners,
//! furos, a marcação Smooth e o `FillRule` não precisaram saber que o caminho existe.
//!
//! **Handles.** O `VecVertex` guarda handles ABSOLUTOS (estilo Rive `CubicVertex`), não
//! vetores-tangente. Quad `S–Q–E` sobe pra cúbica com controles absolutos `S + ⅔(Q−S)` e
//! `E + ⅔(Q−E)`; a cúbica usa `c1`/`c2` diretos.

use ph2d_vec_scene::text_path::GlyphFrame;
use ph2d_vec_scene::{Contour, FillRule, Paint, StrokeSpec, VecPath, VecVertex, VertexKind};
use ph2d_vector_font::{GlyphOutline, PathCommand};

/// Converte o contorno de um glyph em um `VecPath` compound preenchido. `scale` =
/// `font_size / units_per_em`; `frame` = o afim que leva o espaço do glyph ao world,
/// com a origem no **pen origin** dele. Devolve `None` quando o glyph não tem área
/// preenchível (espaço, contorno degenerado).
#[must_use]
pub(crate) fn glyph_to_vec_path(
    outline: &GlyphOutline,
    scale: f64,
    frame: &GlyphFrame,
    fill: Option<Paint>,
    stroke: Option<StrokeSpec>,
) -> Option<VecPath> {
    let mut b = Build::new(scale, *frame);
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
    frame: GlyphFrame,
    /// Tolerância² de "mesmo ponto" (¼ de unidade escalada) — só funde a ponta que
    /// volta ao início; âncoras inteiras distintas ficam ≥ 1 unidade apart.
    ///
    /// Continua em unidades de MUNDO sob um caminho porque o referencial é **rígido**:
    /// ele roda o glyph, não o estica, então distâncias sobrevivem à viagem (é o que o
    /// gate `the_glyph_keeps_its_size_anywhere_on_the_curve` pina). Um referencial
    /// não-rígido — se algum dia entrar — teria de reconsiderar esta linha.
    eps_sq: f64,
    contours: Vec<Vec<VecVertex>>,
    cur: Vec<VecVertex>,
    start: [f64; 2],
    cur_pos: [f64; 2],
    has_seg: bool,
    fused: bool,
}

impl Build {
    fn new(scale: f64, frame: GlyphFrame) -> Self {
        let e = scale * 0.25;
        Self {
            scale,
            frame,
            eps_sq: (e * e).max(1e-12),
            contours: Vec::new(),
            cur: Vec::new(),
            start: [0.0; 2],
            cur_pos: [0.0; 2],
            has_seg: false,
            fused: false,
        }
    }

    /// Unidade de design → world, pelo referencial do glyph. NÃO flipa Y: o world do
    /// editor é Y-up (igual à fonte); quem inverte pra tela é a câmera
    /// (`world_to_screen_affine` tem `scale(k, −k)`). Flipar aqui deixaria o texto de
    /// cabeça pra baixo.
    fn tf(&self, x: f32, y: f32) -> [f64; 2] {
        self.frame
            .apply([f64::from(x) * self.scale, f64::from(y) * self.scale])
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
    // O teste do Style cruza a fronteira: `resolve_style` é do layout (módulo irmão).
    use crate::vec_glyph::resolve_style;
    use ph2d_core::Vec2;
    use ph2d_vec_edit::PenStyle;
    use ph2d_vec_scene::Rgba8;

    fn black() -> Paint {
        Paint::solid(Rgba8::new(0, 0, 0, 255))
    }

    /// O referencial da IDENTIDADE na origem — o texto reto, que é o que estes gates medem.
    fn plain() -> GlyphFrame {
        GlyphFrame {
            origin: [0.0, 0.0],
            x_axis: [1.0, 0.0],
            y_axis: [0.0, 1.0],
        }
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
            &plain(),
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
            &plain(),
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
            &plain(),
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
            &plain(),
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
            &plain(),
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
            &plain(),
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
            &plain(),
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
        assert!(glyph_to_vec_path(&outline(vec![]), 1.0, &plain(), Some(black()), None).is_none());
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
}

//! Texto vetorial — W1 (o converter): transforma o contorno de um glyph (skrifa via
//! `ph2d-vector-font`) em UM `VecPath` compound do motor novo. O contorno externo vai
//! em `verts`, os furos (miolo do `o`, `e`, `a`…) em `subpaths`, com
//! `FillRule::NonZero` — a canônica OT/SVG que resolve buracos. Assim um glyph é uma
//! forma vetorial como qualquer outra: herda render, gizmo, snap, undo e Hierarquia.
//!
//! **Coordenadas.** Unidades de design são y-up e o world do editor TAMBÉM é y-up
//! (quem inverte pra tela é a câmera, `world_to_screen_affine` com `scale(k, −k)`).
//! Então mapeia `(x, y) → (x·scale + origin.x, y·scale + origin.y)`, SEM flip — flipar
//! aqui somaria com o da câmera e o texto sairia de cabeça pra baixo.
//!
//! **Handles.** O `VecVertex` guarda handles ABSOLUTOS (estilo Rive `CubicVertex`),
//! não vetores-tangente. Quad `S–Q–E` sobe pra cúbica com controles absolutos
//! `S + ⅔(Q−S)` e `E + ⅔(Q−E)`; a cúbica usa `c1`/`c2` diretos.

use std::sync::OnceLock;

use ph2d_vec_edit::PenStyle;
use ph2d_vec_scene::{Contour, FillRule, Paint, StrokeSpec, VecPath, VecPathId, VecVertex};
use ph2d_vector_font::{GlyphOutline, PathCommand, VariableFont};

/// Resolve o Style do Pen (fill/stroke/width em px) no par (fill, stroke) que cada
/// glyph-path recebe — mesma regra das formas (`shape.rs`): sem preenchimento quando
/// alpha 0; traço sempre presente (o render pula width/alpha 0). Assim o texto herda
/// Fill/Stroke/Width/Cap/Join do painel do vetor como qualquer forma.
#[must_use]
fn resolve_style(style: &PenStyle, px_to_world: f64) -> (Option<Paint>, Option<StrokeSpec>) {
    let fill = (style.fill.a != 0).then(|| Paint::solid(style.fill));
    let stroke = Some(style.stroke_spec(style.stroke_w_px * px_to_world));
    (fill, stroke)
}

/// A fonte embutida (InterVariable), parseada 1× e cacheada. `None` só se os bytes
/// embutidos falharem o parse — nunca em runtime (é a mesma fonte que a UI usa).
fn font() -> Option<&'static VariableFont> {
    static FONT: OnceLock<Option<VariableFont>> = OnceLock::new();
    FONT.get_or_init(|| VariableFont::new(ph2d_text::inter_variable_ttf().to_vec()).ok())
        .as_ref()
}

/// Layout linear de uma string em `VecPath`s — um por glyph com contorno, posicionado
/// pelo avanço horizontal. `font_size` em unidades de world. `\n` desce uma linha
/// (entrelinha 1.2·size). Sem shaping complexo (kerning/ligaduras) — W1 é advance-only.
#[must_use]
pub(crate) fn text_to_vec_paths(
    font: &VariableFont,
    text: &str,
    font_size: f64,
    origin: [f64; 2],
    fill: &Option<Paint>,
    stroke: &Option<StrokeSpec>,
) -> Vec<VecPath> {
    let upem = f64::from(font.units_per_em().max(1));
    let scale = font_size / upem;
    let line_h = font_size * 1.2;
    let mut out = Vec::new();
    let (mut pen_x, mut pen_y) = (0.0, 0.0);
    for ch in text.chars() {
        if ch == '\n' {
            pen_x = 0.0;
            pen_y -= line_h; // linhas descem = y menor (world y-up)
            continue;
        }
        let Some(gid) = font.glyph_for_char(ch) else {
            continue;
        };
        let advance = f64::from(font.advance(gid, &[]).unwrap_or(0.0));
        if let Ok(outline) = font.outline(gid, &[])
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
        pen_x += advance * scale;
    }
    out
}

/// Largura de avanço total de uma string (uma linha), em unidades de world. Usada
/// para centralizar um bloco de texto horizontalmente.
#[must_use]
pub(crate) fn text_advance_width(font: &VariableFont, text: &str, font_size: f64) -> f64 {
    let scale = font_size / f64::from(font.units_per_em().max(1));
    text.chars()
        .filter(|&c| c != '\n')
        .filter_map(|c| font.glyph_for_char(c))
        .map(|g| f64::from(font.advance(g, &[]).unwrap_or(0.0)) * scale)
        .sum()
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

/// Tamanho default de texto novo, em unidades de world (a vista tem ~10 de altura).
pub(crate) const DEFAULT_TEXT_SIZE: f64 = 1.0;
/// Entrelinha como múltiplo do tamanho.
const LINE_SPACING: f64 = 1.2;

/// Uma sessão de digitação de texto no canvas (`DrawMode::Text`). O texto vive como
/// glyphs (`VecPath`s) na cena; esta struct guarda o ponto de inserção e QUAIS paths
/// são desta sessão, para regenerá-los a cada tecla. Ao finalizar, os glyphs ficam na
/// cena como formas normais e a sessão some.
pub(crate) struct VecTextEdit {
    /// Baseline da PRIMEIRA linha, em world (y-up).
    pub origin: [f64; 2],
    /// Tamanho em unidades de world.
    pub size: f64,
    /// Preenchimento dos glyphs (do Style do painel; `None` = sem fill).
    pub fill: Option<Paint>,
    /// Traço dos glyphs (do Style: cor/largura/cap/join/dash), como nas formas.
    pub stroke: Option<StrokeSpec>,
    /// Conteúdo digitado.
    pub text: String,
    /// Os paths dos glyphs atualmente na cena (regenerados a cada mudança).
    pub ids: Vec<VecPathId>,
}

impl crate::app_state::App {
    /// Tecla `T` na tool Vector: entra no modo Text (de qualquer modo) ou, se já está
    /// nele, finaliza a edição corrente e volta ao Select. Atalho-padrão de ferramenta
    /// de texto; a mesma troca de modo do botão do painel (W1.3 passo 2).
    pub(crate) fn vec_text_toggle_mode(&mut self) {
        use ph2d_tool_vector::DrawMode;
        if self.vec_draw_config.mode == DrawMode::Text {
            self.vec_text_finish();
            self.vec_set_draw_mode(DrawMode::Select);
        } else {
            self.vec_set_draw_mode(DrawMode::Text);
        }
    }

    /// Troca o modo de desenho da tool Vector (a tool é a dona; `vec_draw_config` é
    /// espelho). O downcast fica no `vector_bridge` (allowlist da gate de downcast).
    /// Espelha na hora para o mesmo frame já rotear certo.
    pub(crate) fn vec_set_draw_mode(&mut self, mode: ph2d_tool_vector::DrawMode) {
        if let Some(gfx) = self.gfx.as_mut() {
            crate::render_loop::vector_bridge::set_mode(&mut gfx.tools, mode);
        }
        self.vec_draw_config.mode = mode;
    }

    /// Clique no canvas em modo Text: finaliza a edição anterior (se houver) e começa
    /// uma nova com a baseline no ponto clicado.
    pub(crate) fn vec_text_click(&mut self, world: [f64; 2]) {
        self.vec_text_finish();
        // O texto herda o Style ATIVO do painel do vetor (fill/stroke/width/cap/join)
        // — a mesma regra das formas — capturado no clique.
        let (fill, stroke) = resolve_style(&self.vec_pen.style(), self.vec_px_to_world());
        self.vec_text_edit = Some(VecTextEdit {
            origin: world,
            size: DEFAULT_TEXT_SIZE,
            fill,
            stroke,
            text: String::new(),
            ids: Vec::new(),
        });
    }

    /// Anexa um caractere ao texto em edição e regenera os glyphs. No-op sem edição.
    pub(crate) fn vec_text_append(&mut self, ch: char) {
        if let Some(edit) = self.vec_text_edit.as_mut() {
            edit.text.push(ch);
            self.vec_text_regen();
        }
    }

    /// Apaga o último caractere (Backspace).
    pub(crate) fn vec_text_backspace(&mut self) {
        if let Some(edit) = self.vec_text_edit.as_mut() {
            edit.text.pop();
            self.vec_text_regen();
        }
    }

    /// Enter: quebra de linha dentro da mesma sessão.
    pub(crate) fn vec_text_newline(&mut self) {
        self.vec_text_append('\n');
    }

    /// Finaliza a sessão: os glyphs ficam na cena, o cursor some. Se ficou vazia, não
    /// deixa nada (o `regen` já não gerou paths).
    pub(crate) fn vec_text_finish(&mut self) {
        self.vec_text_edit = None;
    }

    /// Há uma sessão de texto ativa?
    #[must_use]
    pub(crate) fn vec_text_editing(&self) -> bool {
        self.vec_text_edit.is_some()
    }

    /// Regenera os glyphs da sessão: remove os antigos da cena e empurra os novos a
    /// partir do texto corrente. O `vec_entities::sync` reconcilia as entidades.
    fn vec_text_regen(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(edit) = self.vec_text_edit.as_mut() else {
            return;
        };
        regen_into(&mut gfx.vec_scene, edit);
    }
}

/// Remove os glyphs antigos da sessão e empurra os do texto corrente. Recebe a cena
/// explicitamente (não `self.gfx`) para ser chamável tanto do caminho de teclado
/// (`vec_text_regen`) quanto do render loop, onde `gfx` já está com borrow dividido.
fn regen_into(scene: &mut ph2d_vec_scene::VecScene, edit: &mut VecTextEdit) {
    let Some(font) = font() else {
        return;
    };
    for id in edit.ids.drain(..) {
        scene.remove_path(id);
    }
    let paths = text_to_vec_paths(
        font,
        &edit.text,
        edit.size,
        edit.origin,
        &edit.fill,
        &edit.stroke,
    );
    for path in paths {
        edit.ids.push(scene.push_path(path));
    }
}

/// Sincroniza o Style de uma sessão de texto ATIVA com o Style vivo do painel, por
/// frame: se o Fill/Stroke/Width/Cap/Join mudou (o usuário mexeu no painel enquanto
/// digitava), regenera os glyphs com o novo Paint — texto em edição herda o Style em
/// TEMPO REAL, como o Enio pediu. Guardado por igualdade: sem mudança, não regenera.
/// Fn livre (recebe pen + cena + px_to_world) para o render loop poder chamá-la com
/// os campos já emprestados, sem tomar o `App` inteiro.
///
/// **Sair do modo Text COMMITA a sessão** (`*edit = None`): os glyphs viram paths
/// normais e o recolor passa a ser por-seleção (in-place, id estável). Sem isto, uma
/// sessão viva no Select regeneraria o texto INTEIRO a cada mudança de Style —
/// atingindo letras NÃO-selecionadas e trocando as ids dos paths, com o que o gizmo
/// de seleção (preso às entidades antigas) sumia (Enio 2026-07-11). O botão de modo do
/// painel não passa por `vec_text_toggle_mode`, então o commit precisa ser por frame.
pub(crate) fn sync_active_text_style(
    edit: &mut Option<VecTextEdit>,
    mode: ph2d_tool_vector::DrawMode,
    pen: &ph2d_vec_edit::PenTool,
    px_to_world: f64,
    scene: &mut ph2d_vec_scene::VecScene,
) {
    if mode != ph2d_tool_vector::DrawMode::Text {
        *edit = None; // deixou o modo Text ⇒ commita (os glyphs ficam na cena)
        return;
    }
    let Some(edit) = edit.as_mut() else {
        return;
    };
    let (fill, stroke) = resolve_style(&pen.style(), px_to_world);
    if edit.fill != fill || edit.stroke != stroke {
        edit.fill = fill;
        edit.stroke = stroke;
        regen_into(scene, edit);
    }
}

/// Os dois pontos (world) do cursor de texto vertical na ponta da última linha —
/// `None` se não há edição. Fn livre (não método) para o render poder chamá-la lendo
/// só o campo `vec_text_edit`, sem emprestar o `App` inteiro (o `gfx` está vivo lá).
#[must_use]
pub(crate) fn caret_of(edit: Option<&VecTextEdit>) -> Option<([f64; 2], [f64; 2])> {
    let edit = edit?;
    let font = font()?;
    let last_line = edit.text.rsplit('\n').next().unwrap_or("");
    let line_idx = edit.text.matches('\n').count();
    let cx = edit.origin[0] + text_advance_width(font, last_line, edit.size);
    let baseline = edit.origin[1] - line_idx as f64 * edit.size * LINE_SPACING;
    Some((
        [cx, baseline - 0.2 * edit.size],
        [cx, baseline + 0.72 * edit.size],
    ))
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

    /// O cursor de texto fica à direita da origem depois de digitar, e é vertical.
    #[test]
    fn the_caret_advances_as_text_is_typed() {
        let edit = VecTextEdit {
            origin: [5.0, 2.0],
            size: 1.0,
            fill: Some(black()),
            stroke: None,
            text: "Hi".to_string(),
            ids: Vec::new(),
        };
        let (a, b) = caret_of(Some(&edit)).expect("caret com edição ativa");
        assert!(a[0] > 5.0, "o cursor avançou à direita da origem");
        assert!((a[0] - b[0]).abs() < 1e-9, "cursor vertical");
        assert!(b[1] > a[1], "topo acima da base");
        assert!(caret_of(None).is_none(), "sem edição, sem cursor");
    }

    /// Sair do modo Text COMMITA a sessão: `sync_active_text_style` com um modo != Text
    /// zera a sessão (os glyphs ficam na cena). Sem isto, uma sessão viva no Select
    /// regeneraria o texto inteiro a cada mudança de Style — pegando letras não
    /// selecionadas e sumindo com o gizmo (Enio 2026-07-11).
    #[test]
    fn leaving_text_mode_commits_the_session() {
        use ph2d_tool_vector::DrawMode;
        let mut scene = ph2d_vec_scene::VecScene::new();
        let mut edit = Some(VecTextEdit {
            origin: [0.0, 0.0],
            size: 1.0,
            fill: Some(black()),
            stroke: None,
            text: "A".to_string(),
            ids: Vec::new(),
        });
        let pen = ph2d_vec_edit::PenTool::default();
        sync_active_text_style(&mut edit, DrawMode::Select, &pen, 0.01, &mut scene);
        assert!(edit.is_none(), "modo != Text termina a sessão (commit)");
    }

    /// No modo Text, mudar o Style do painel regenera os glyphs vivos com o novo Paint
    /// (herança em tempo real). O glyph troca de fill sem sair da sessão.
    #[test]
    fn active_text_restyles_live_when_the_panel_style_changes() {
        use ph2d_tool_vector::DrawMode;
        use ph2d_vec_edit::{PenStyle, PenTool};
        let mut scene = ph2d_vec_scene::VecScene::new();
        let mut edit = Some(VecTextEdit {
            origin: [0.0, 0.0],
            size: 1.0,
            fill: Some(black()),
            stroke: None,
            text: "A".to_string(),
            ids: Vec::new(),
        });
        regen_into(&mut scene, edit.as_mut().unwrap());
        assert_eq!(scene.paths().len(), 1, "o glyph foi para a cena");
        let mut pen = PenTool::default();
        pen.set_style(PenStyle {
            fill: Rgba8::new(10, 200, 30, 255),
            ..PenStyle::default()
        });
        sync_active_text_style(&mut edit, DrawMode::Text, &pen, 0.0, &mut scene);
        let gid = edit.as_ref().unwrap().ids.first().copied().expect("um glyph");
        let fill = scene
            .paths()
            .iter()
            .find(|p| p.id == gid)
            .and_then(|p| p.fill.clone());
        assert!(
            matches!(fill, Some(Paint::Solid(c)) if c.g == 200),
            "o glyph adotou o novo fill do painel ao vivo"
        );
        assert_eq!(scene.paths().len(), 1, "regen substitui, não acumula");
    }

    /// Multi-linha: a 2ª linha desce (world y-up → y negativo abaixo da baseline).
    #[test]
    fn a_second_line_sits_below_the_first() {
        let font = font().expect("fonte embutida");
        let min_y = |paths: &[VecPath]| {
            paths
                .iter()
                .flat_map(|p| p.verts.iter())
                .map(|v| v.anchor[1])
                .fold(f64::INFINITY, f64::min)
        };
        let one = text_to_vec_paths(font, "A", 1.0, [0.0, 0.0], &Some(black()), &None);
        let two = text_to_vec_paths(font, "A\nA", 1.0, [0.0, 0.0], &Some(black()), &None);
        assert!(
            min_y(&one) >= -1e-6,
            "linha única: baseline em 0, sem descer"
        );
        assert!(
            min_y(&two) < -0.5,
            "a 2ª linha desce abaixo da baseline da 1ª"
        );
    }
}

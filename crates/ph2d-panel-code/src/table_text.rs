//! O texto das células da tabela da casa ([`ph2d_ui_table`]) no editor de scripts: os valores na mono
//! do código com a cor do realce (número, texto, constante…), o «mudou» em âmbar, o resto na letra
//! da interface.

use crate::palette::Palette;
use crate::style::Ink as CodeInk;
use crate::view::CodeView;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_ui_table::{CellText, Ink, UiText};
use ph2d_vector::VectorScene;

/// As células da tabela pintadas com a mono do `view`.
pub struct CodeCells<'a> {
    pub view: &'a mut CodeView,
    pub theme: Theme,
}

/// A cor do realce de cada tom da tabela (`None` = um tom de estado, pintado com o token dele).
fn code_ink(ink: Ink) -> Option<CodeInk> {
    Some(match ink {
        Ink::Plain => CodeInk::Plain,
        Ink::Name => CodeInk::Hint,
        Ink::Text => CodeInk::String,
        Ink::Boolean | Ink::Nil => CodeInk::Constant,
        Ink::Table => CodeInk::Type,
        Ink::Function => CodeInk::Function,
        // O número no azul frio da maquete 14 (o `Info`), não no laranja do realce.
        // `N fields` apagado no `Text3` da casa, como o resto do que é nota.
        Ink::Number | Ink::Muted | Ink::Changed | Ink::Warn | Ink::Danger | Ink::Accent => {
            return None;
        }
    })
}

impl CellText for CodeCells<'_> {
    fn width(&mut self, ts: &mut TextSystem, text: &str, mono: bool) -> f32 {
        if mono {
            text.chars().count() as f32 * self.view.metrics().advance
        } else {
            UiText { theme: self.theme }.width(ts, text, false)
        }
    }

    fn paint(
        &mut self,
        ts: &mut TextSystem,
        scene: &mut VectorScene,
        text: &str,
        ink: Ink,
        mono: bool,
        x: f32,
        (y, h): (f32, f32),
        right: f32,
    ) {
        if !mono {
            UiText { theme: self.theme }.paint(ts, scene, text, ink, false, x, (y, h), right);
            return;
        }
        let mut pal = Palette::new(self.theme);
        let code = code_ink(ink).unwrap_or_else(|| {
            pal.tint(CodeInk::Plain, ph2d_ui_table::ink_color(ink, self.theme));
            CodeInk::Plain
        });
        let row_y = y + (h - self.view.metrics().line_h) * 0.5;
        // A medida é `chars × advance`; o corte do `paint_mono` soma os avanços da letra — à justa,
        // o arredondamento largava a última (medido na foto da cena 14: o «true» sem o «e», «-»).
        let right = right + ph2d_tokens::StrokeToken::Hairline.px();
        self.view
            .paint_mono(ts, scene, &pal, text, code, x, row_y, right);
    }
}

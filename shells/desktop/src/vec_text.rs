//! Texto vetorial — a SESSÃO de edição no canvas (`DrawMode::Text`). Guarda o ponto
//! de inserção, o conteúdo e QUAIS paths são da sessão, regenerando-os a cada tecla /
//! mudança de Style. O converter puro (glyph → `VecPath`) mora no módulo irmão
//! [`crate::vec_glyph`]; aqui é só o estado + as ações do `App`.
//!
//! Ao finalizar (sair do modo Text), os glyphs ficam na cena como formas normais e a
//! sessão some — o recolor passa a ser por-seleção, in-place (ver [`sync_active_text_style`]).

use ph2d_vec_scene::{Paint, StrokeSpec, VecPathId};

use crate::vec_glyph::{font, resolve_style, text_advance_width, text_to_vec_paths};

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
            size: self.vec_text_size, // o tamanho corrente do painel (Size slider)
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

/// Aplica o tamanho vindo do slider Size do painel: atualiza o default corrente da
/// shell (`size_field`, semeia a próxima sessão) e, se há sessão ativa, o tamanho
/// dela + regenera os glyphs ao vivo. Clampeado no mínimo para nunca degenerar o
/// glyph. Fn livre (mirror de `apply_vec_transform`) para o drain do render loop.
pub(crate) fn apply_text_size(
    edit: &mut Option<VecTextEdit>,
    size_field: &mut f64,
    scene: &mut ph2d_vec_scene::VecScene,
    size: f64,
) {
    let size = size.max(ph2d_tool_vector::params::TEXT_SIZE_MIN);
    *size_field = size;
    if let Some(edit) = edit.as_mut() {
        edit.size = size;
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
    use ph2d_vec_scene::Rgba8;

    fn black() -> Paint {
        Paint::solid(Rgba8::new(0, 0, 0, 255))
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
        let gid = edit
            .as_ref()
            .unwrap()
            .ids
            .first()
            .copied()
            .expect("um glyph");
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
}

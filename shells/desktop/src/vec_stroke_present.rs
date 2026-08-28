//! ⭐ **DAR (e TIRAR) o traço de uma forma** — a porta única da caixa *Stroke* do painel (plano 34).
//!
//! # Por que isto existe
//!
//! Toda forma **autorada** nasce com traço (`shape.rs` escreve `path.stroke = Some(..)`
//! incondicionalmente), e o `restyle_selected_strokes` **recusa** quem não tem um. ⇒ uma forma que
//! chegou ao documento por outro caminho — um importador, uma cena montada por código, uma booleana
//! — ficava **sem forma nenhuma de ganhar um traço**, com a secção *Stroke* do painel a oferecer
//! controlos que não a alcançavam. Foi o report do Enio de 2026-08-27.
//!
//! # ⛔ Por que a cura NÃO é no `restyle_selected_strokes`
//!
//! Aquela função corre **por quadro** sobre a selecção, sempre que o estilo da tool difere do dela
//! ([`crate::render_loop::vector_bridge`]). Criar ali daria traço a **toda** forma sem traço que
//! estivesse selecionada, **sem ninguém pedir** — e o comentário que lá está (*"ganhar um traço do
//! nada seria a UI inventando geometria"*) é a cerca certa pelo motivo certo.
//!
//! ⇒ **a criação é um GESTO explícito**, nunca um efeito colateral de um espelhamento.

use ph2d_vec_edit::{History, PenTool};
use ph2d_vec_scene::VecScene;

/// **A forma selecionada tem traço?** `None` quando não há uma resposta — nada selecionado, ou
/// selecção múltipla.
///
/// ⚠️ O `None` é a metade que importa: é ele que impede a caixa de ser pintada. *Uma caixa que
/// descreve um objecto que não está lá é pior que caixa nenhuma* — a mesma lei do `resize_box`.
#[must_use]
pub(crate) fn selected_stroke_present(scene: &VecScene, pen: &PenTool) -> Option<bool> {
    let [id] = pen.selected_paths() else {
        return None;
    };
    scene.path(*id).map(|p| p.stroke.is_some())
}

/// **Inverte o traço da forma selecionada** — dá um se não tem, tira se tem. `true` se mudou.
///
/// ⚠️⚠️ **O traço novo sai da ficha da FERRAMENTA** (`pen.style()`), pela **mesma** porta que a
/// ferramenta de forma usa ao criar (`PenStyle::stroke_spec`, com a largura em px convertida pelo
/// mesmo `px_to_world`). Um default escrito aqui seria uma segunda resposta a *"que traço uma coisa
/// nova recebe?"*, e a forma vestida pela caixa sairia diferente da desenhada.
///
/// ⚠️ **Tirar DESTRÓI a ficha** — é o modelo do Figma, não o do Illustrator (onde a largura
/// persiste por baixo de um `stroke: none`). O ida-e-volta continua honesto porque a ficha da
/// ferramenta é **o que a própria secção mostra**: as rows *Width*/*Color* ficam visíveis, então
/// voltar a marcar devolve o que se está a ver. ⛔ Guardar a ficha removida **no documento** seria
/// estado invisível a envenenar o undo; guardá-la na shell seria estado de sessão que o save não
/// leva.
pub(crate) fn toggle(
    scene: &mut VecScene,
    history: &mut History,
    pen: &PenTool,
    px_to_world: f64,
) -> bool {
    let [id] = pen.selected_paths() else {
        return false;
    };
    let id = *id;
    let Some(tem) = scene.path(id).map(|p| p.stroke.is_some()) else {
        return false;
    };
    let style = pen.style();
    let novo = (!tem).then(|| style.stroke_spec(style.stroke_w_px * px_to_world));
    let pre = scene.clone();
    let Some(path) = scene.path_mut(id) else {
        return false;
    };
    path.stroke = novo;
    history.push_undo(pre);
    true
}

#[cfg(test)]
#[path = "vec_stroke_present_tests.rs"]
mod tests;

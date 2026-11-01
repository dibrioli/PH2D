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

use ph2d_vec_edit::PenTool;
use ph2d_vec_scene::{Paint, VecPath, VecScene};

/// ⭐ **Os quatro gestos da tinta de BASE** (F44b): o «−»/«+» e o olho, do preenchimento e do traço.
/// ⚠️ O olho GUARDA a tinta (`VecPath::fill_hidden`); o «−» tira-a, e o «+» dá uma nova.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BaseVerb {
    /// Dá ou tira o preenchimento de base.
    Fill,
    /// Dá ou tira o traço de base (a caixa *Stroke* de sempre).
    Stroke,
    FillEye,
    StrokeEye,
}

/// Que gesto este id pede (`None` se não é da base).
#[must_use]
pub fn verb_for_id(id: ph2d_editor_core::NodeId) -> Option<BaseVerb> {
    use ph2d_panel_vector::ids as p;
    [
        (p::VECTOR_FILL_PRESENT, BaseVerb::Fill),
        (p::VECTOR_STROKE_PRESENT, BaseVerb::Stroke),
        (p::VECTOR_FILL_EYE, BaseVerb::FillEye),
        (p::VECTOR_STROKE_EYE, BaseVerb::StrokeEye),
    ]
    .into_iter()
    .find_map(|(k, v)| (k == id).then_some(v))
}

/// **Aplica o gesto** à forma selecionada. Devolve se o documento mudou.
pub fn apply(scene: &mut VecScene, pen: &PenTool, verb: BaseVerb) -> bool {
    match verb {
        BaseVerb::Stroke => toggle(scene, pen),
        BaseVerb::Fill => {
            let cor = pen.style().fill;
            edit_one(scene, pen, |p| {
                if p.base_fill().is_some() {
                    p.remove_base_fill()
                } else {
                    p.fill = Some(Paint::solid(cor));
                    true
                }
            })
        }
        BaseVerb::FillEye => edit_one(scene, pen, |p| {
            let esconde = !p.fill_is_hidden();
            p.set_fill_hidden(esconde)
        }),
        BaseVerb::StrokeEye => edit_one(scene, pen, |p| {
            let esconde = !p.stroke_is_hidden();
            p.set_stroke_hidden(esconde)
        }),
    }
}

fn edit_one(scene: &mut VecScene, pen: &PenTool, f: impl FnOnce(&mut VecPath) -> bool) -> bool {
    let [id] = pen.selected_paths() else {
        return false;
    };
    scene.path_mut(*id).is_some_and(f)
}

/// **O que o painel mostra das duas tintas de base** da forma selecionada (`None` sem resposta) —
/// e as pontas do degradê da base, para a amostra.
pub fn publish(scene: &VecScene, pen: &PenTool) {
    use ph2d_panel_vector::state::BasePaint;
    let de = |tem: bool, escondida: bool| match (tem, escondida) {
        (_, true) => BasePaint::Hidden,
        (true, false) => BasePaint::Shown,
        (false, false) => BasePaint::Absent,
    };
    let p = match pen.selected_paths() {
        [id] => scene.path(*id),
        _ => None,
    };
    ph2d_panel_vector::state::set_base_paints(p.map(|p| {
        [
            de(p.fill.is_some(), p.fill_is_hidden()),
            de(p.stroke.is_some(), p.stroke_is_hidden()),
        ]
    }));
    ph2d_panel_vector::state::set_base_fill_ramp(p.and_then(|p| ramp(p.base_fill()?)));
}

/// As duas pontas de um degradê (`None` numa tinta sólida ou num padrão).
#[must_use]
pub fn ramp(paint: &Paint) -> Option<[[u8; 4]; 2]> {
    let c = |c: ph2d_vec_scene::Rgba8| [c.r, c.g, c.b, c.a];
    match paint {
        Paint::Linear { stops, .. } | Paint::Radial { stops, .. } => {
            Some([c(stops.first()?.color), c(stops.last()?.color)])
        }
        Paint::MultiPoint { points } => Some([c(points.first()?.color), c(points.last()?.color)]),
        _ => None,
    }
}

/// **A forma selecionada tem traço?** `None` quando não há uma resposta — nada selecionado, ou
/// selecção múltipla.
///
/// ⚠️ O `None` é a metade que importa: é ele que impede a caixa de ser pintada. *Uma caixa que
/// descreve um objecto que não está lá é pior que caixa nenhuma* — a mesma lei do `resize_box`.
#[must_use]
pub fn selected_stroke_present(scene: &VecScene, pen: &PenTool) -> Option<bool> {
    let [id] = pen.selected_paths() else {
        return None;
    };
    scene.path(*id).map(|p| p.base_stroke().is_some())
}

/// **Inverte o traço da forma selecionada** — dá um se não tem, tira se tem. `true` se mudou.
///
/// ⚠️⚠️ **O traço novo sai da ficha da FERRAMENTA** (`pen.style()`), pela **mesma** porta que a
/// ferramenta de forma usa ao criar (`PenStyle::stroke_spec`, com a largura que a ponte já pôs em
/// mundo). Um default escrito aqui seria uma segunda resposta a *"que traço uma coisa
/// nova recebe?"*, e a forma vestida pela caixa sairia diferente da desenhada.
///
/// ⚠️ **Tirar DESTRÓI a ficha** — é o modelo do Figma, não o do Illustrator (onde a largura
/// persiste por baixo de um `stroke: none`). O ida-e-volta continua honesto porque a ficha da
/// ferramenta é **o que a própria secção mostra**: as rows *Width*/*Color* ficam visíveis, então
/// voltar a marcar devolve o que se está a ver. ⛔ Guardar a ficha removida **no documento** seria
/// estado invisível a envenenar o undo; guardá-la na shell seria estado de sessão que o save não
/// leva.
pub fn toggle(scene: &mut VecScene, pen: &PenTool) -> bool {
    let [id] = pen.selected_paths() else {
        return false;
    };
    let id = *id;
    let Some(tem) = scene.path(id).map(|p| p.base_stroke().is_some()) else {
        return false;
    };
    let style = pen.style();
    let novo = (!tem).then(|| style.stroke_spec(style.stroke_w));
    let Some(path) = scene.path_mut(id) else {
        return false;
    };
    // O «−» tira também a guardada pelo olho (F44b).
    path.stroke_hidden = None;
    path.stroke = novo;
    true
}

#[cfg(test)]
#[path = "stroke_present_tests.rs"]
mod tests;

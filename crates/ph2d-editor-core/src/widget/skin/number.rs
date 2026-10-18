//! A pele do CAMPO NUMÉRICO — saiu do [`super`] pelo tecto de 500 LOC dos widgets (18/10: o
//! pintor ganhou o argumento do «misto» do BulkSelect e o braço já não cabia).

use super::*;

/// ⚠️ `step`/`min`/`max` ficam nos defaults do construtor de propósito: eles governam o que um
/// GESTO produz, e o gesto é do painel — a pele responde *que aparência tem um campo numérico*, e
/// nenhum dos três a muda. Autorá-los seria um parâmetro por-tipo sem consumidor (ver o doc do
/// `ALL`). Sem store, a pele nunca é «misto».
pub(super) fn paint_number_skin(
    id: NodeId,
    label: &str,
    live: Option<(&InteractiveState, f32)>,
    rect: Rect,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    let mut n = NumberInput::new(id, label, f64::from(PREVIEW_VALUE));
    // O buffer do estado vivo é o que o artista está DIGITANDO; sem estado, o pintor
    // formata o `value` sozinho — é o que o `None` significa aqui.
    let mut buf: Option<&str> = None;
    let (mut caret, mut anchor) = (0usize, None);
    if let Some((
        InteractiveState::NumberInput {
            state,
            value,
            buffer,
            caret: c,
            selection_anchor,
            ..
        },
        t,
    )) = live
    {
        n = n.visual((*state, t));
        n.value = *value;
        buf = Some(buffer.as_str());
        caret = *c;
        anchor = *selection_anchor;
    }
    paint_number_input_with_buffer(
        &n,
        buf,
        false,
        caret,
        anchor,
        rect,
        scene,
        text_system,
        theme,
    );
}

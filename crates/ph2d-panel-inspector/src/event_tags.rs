//! ⭐ **O despacho da secção TAGS** (TOP-20 #9, W3a) — irmão do [`crate::event`] por CAP de função.
//!
//! # ⚠️ Três gestos, e cada um é uma variante diferente da edição
//!
//! | gesto | o que vai ao barramento |
//! |---|---|
//! | o `×` de um chip | `TagsFieldEdit::Remove(id)` |
//! | uma opção da lista | `TagsFieldEdit::Add(id)` |
//! | o `+ Create "…"` | `TagsFieldEdit::Create(texto)` |
//!
//! ⛔ **Nenhum deles apaga uma tag da ÁRVORE** — o `×` tira-a DESTE objecto. Apagar a tag do
//! projecto é do painel *Tags* (W4), e confundir os dois aqui faria um clique distraído num chip
//! desmarcar todos os outros objectos da cena.
//!
//! # ⚠️ O que este despacho NÃO escreve
//!
//! O `selected_index` do dropdown. Quem é dono da pertença é o snapshot: o quadro seguinte relê-a
//! da cena. Escrever aqui abriria a segunda porta para o mesmo estado — e ela mentiria exactamente
//! no caso em que a shell recusasse a edição (o objecto cheio).

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::screens::hero::TagsFieldEdit;
use ph2d_editor_core::widget::{ButtonState, TagState};

use crate::state;

/// Despacha um evento da secção TAGS. `true` = consumido.
pub(crate) fn apply_tags_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let Some(info) = state::current_inspector_tags() else {
        return false;
    };
    let WidgetEvent::Click(id) = ev else {
        return false;
    };

    // O `×` de um chip — a posição no array é a posição na lista desenhada.
    if let Some(i) = crate::ids::INSP_TAGS_CHIP.iter().position(|&o| o == id)
        && let Some(row) = info.on_object.get(i)
    {
        push(host, info.entity_bits, TagsFieldEdit::Remove(row.id));
        demote_tag(host, id);
        return true;
    }

    // Uma opção da lista. ⚠️ **A lista é a MESMA que o pintor derivou** (`pick_options`), com o
    // mesmo filtro lido do mesmo campo — uma segunda derivação aqui poria o clique na tag errada
    // no quadro em que o artista está a escrever.
    if let Some(i) = crate::ids::INSP_TAGS_OPT.iter().position(|&o| o == id) {
        let filtro = ph2d_label_fold::fold(&texto(host));
        let arvore = state::current_tag_tree();
        if let Some(opt) =
            crate::sections::tags::pick_options(&arvore, &info.on_object, &filtro).get(i)
        {
            push(host, info.entity_bits, TagsFieldEdit::Add(opt.value));
            limpa_busca(host);
            fecha_popover(host);
            demote(host, id);
            return true;
        }
    }

    if id == crate::ids::INSP_TAGS_CREATE {
        let escrito = texto(host);
        let nome = escrito.trim();
        // ⛔ Um nome vazio não chega aqui (o botão não é pintado), e a cerca fica na mesma: o
        // `TagTree::create` recusaria, e um botão que consome o clique sem fazer nada lê-se como
        // avaria.
        if !nome.is_empty() {
            push(
                host,
                info.entity_bits,
                TagsFieldEdit::Create(nome.to_string()),
            );
            limpa_busca(host);
        }
        demote(host, id);
        return true;
    }
    false
}

fn push(host: &mut dyn PanelHostInternal, entity_bits: u64, edit: TagsFieldEdit) {
    host.bus_mut()
        .push(EditorAction::InspectorTagsEdit { entity_bits, edit });
}

/// O que está escrito no campo de busca/criação.
fn texto(host: &mut dyn PanelHostInternal) -> String {
    match host.store().get(crate::ids::INSP_TAGS_NEW) {
        Some(InteractiveState::TextInput { text, .. }) => text.clone(),
        _ => String::new(),
    }
}

/// **Esvazia a busca depois de uma escolha ou de uma criação.**
///
/// ⚠️ **Não é cosmética:** o campo filtra a lista, e deixá-lo escrito depois de a tag entrar faria
/// a caixa seguinte abrir já estreitada por uma busca que o artista pensa ter terminado — e a tag
/// que ele procura a seguir parecia não existir.
fn limpa_busca(host: &mut dyn PanelHostInternal) {
    if let Some(InteractiveState::TextInput {
        text,
        caret,
        selection_anchor,
        ..
    }) = host.store_mut().get_mut(crate::ids::INSP_TAGS_NEW)
    {
        text.clear();
        *caret = 0;
        *selection_anchor = None;
    }
}

/// Fecha o popover da caixa depois de uma escolha.
fn fecha_popover(host: &mut dyn PanelHostInternal) {
    if let Some(InteractiveState::Dropdown { open, .. }) =
        host.store_mut().get_mut(crate::ids::INSP_TAGS_PICK)
    {
        *open = false;
    }
}

/// Repõe o visual de um botão momentâneo — senão ele fica `Pressed` depois do clique.
fn demote(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Button { state }) = host.store_mut().get_mut(id) {
        *state = ButtonState::Normal;
    }
}

/// O mesmo, para um CHIP — ele é um `Tag`, não um `Button`, e o braço do botão não lhe toca.
fn demote_tag(host: &mut dyn PanelHostInternal, id: ph2d_a11y::NodeId) {
    if let Some(InteractiveState::Tag { state }) = host.store_mut().get_mut(id) {
        *state = TagState::Normal;
    }
}

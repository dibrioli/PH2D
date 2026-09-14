//! **O corpo do painel** — a barra de verbos e a árvore.
//!
//! # ⚠️ Os verbos vivem numa BARRA, não em cada linha
//!
//! A alternativa era `+`/`×` por linha, e ela custa **dois hit-rects por tag** — numa árvore de
//! centenas, centenas de rectângulos por quadro para gestos que só se aplicam a uma linha de cada
//! vez. A barra pergunta uma vez *«qual é a linha em mãos?»* e oferece os verbos DELA.
//!
//! ⭐⭐ **E é isso que deixa o rótulo do `Delete` dizer o estrago**: com um `×` por linha o botão não
//! tem onde escrever *«3 tags, 5 objects»*, e o gesto destrutivo fica mudo.
//!
//! ⚠️ **Sem linha em mãos, a barra só oferece `+ New`** — o painel nunca pinta um verbo que vai ser
//! recusado (a mesma lei que esconde a caixa de escolha do Inspector quando o objecto está cheio).

use ph2d_editor_core::TagsPanelRow;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::widget::{
    Button, ButtonKind, RowHighlight, TextInput, TextInputState, paint_button, paint_row_highlight,
    paint_text_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, ROW_H_PX, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// O recuo de um nível da árvore. ⚠️ **Pela porta do design system** (`list_indent_px`), como a
/// Hierarquia — um literal aqui seria a terceira constante de recuo deste repo, e a tabela dessa
/// porta existe porque já houve duas.
fn indent_px() -> f32 {
    ph2d_tokens::list_indent_px()
}

/// **Um verbo da barra** — `(id, rótulo, precisa de linha em mãos)`.
pub(crate) struct Verb {
    pub id: ph2d_a11y::NodeId,
    pub label: String,
}

/// Os verbos que a barra oferece AGORA. ⚠️ Uma tabela, quatro consumidores (pintar · registar ·
/// despachar · o censo de costura) — a lei que o painel de física escreve no `lib.rs` dele.
pub(crate) fn verbs(focused: Option<&TagsPanelRow>) -> Vec<Verb> {
    let mut out = vec![Verb {
        id: crate::ids::TAGS_NEW,
        label: "+ New".into(),
    }];
    let Some(row) = focused else { return out };
    out.push(Verb {
        id: crate::ids::TAGS_CHILD,
        label: "+ Child".into(),
    });
    out.push(Verb {
        id: crate::ids::TAGS_RENAME,
        label: "Rename".into(),
    });
    if row.depth > 0 {
        out.push(Verb {
            id: crate::ids::TAGS_UNPARENT,
            label: "Move to root".into(),
        });
    }
    out.push(Verb {
        id: crate::ids::TAGS_SELECT,
        label: format!("Select ({})", row.members),
    });
    // ⭐⭐ **O rótulo CARREGA o estrago.** Apagar `Enemy` leva `Flying` e `Boss` junto — e o artista
    // só vê isso se o botão o disser antes de ser carregado.
    out.push(Verb {
        id: crate::ids::TAGS_DELETE,
        label: if row.subtree > 1 {
            format!("Delete ({} tags, {} objects)", row.subtree, row.members)
        } else {
            format!("Delete ({} objects)", row.members)
        },
    });
    out
}

/// A largura que um botão precisa para o rótulo dele caber.
fn verb_w(text_system: &mut TextSystem, label: &str) -> f32 {
    text_system.prefix_width(label, TypeToken::Sm.px()) + Spacing::Lg.px() * 2.0
}

/// **A barra de verbos, quebrada em linhas.** Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn verb_bar(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    verbs: &[Verb],
) -> f32 {
    let gap = ph2d_tokens::control_gap_px();
    let (mut cx, mut cy) = (x, y);
    for v in verbs {
        let bw = verb_w(text_system, &v.label).min(w);
        if cx > x && cx + bw > x + w {
            cx = x;
            cy += ROW_H_PX + gap;
        }
        let rect = Rect::new(cx, cy, bw, ROW_H_PX);
        let kind = if v.id == crate::ids::TAGS_DELETE {
            ButtonKind::Danger
        } else {
            ButtonKind::Default
        };
        let btn = Button::new(v.id, v.label.clone())
            .kind(kind)
            .visual(store.button_visual(v.id));
        paint_button(&btn, rect, scene, text_system, theme);
        hit_index.register(v.id, rect);
        cx += bw + gap;
    }
    cy + ph2d_tokens::row_pitch_px()
}

/// **Uma linha da árvore.** Devolve o `y` seguinte.
#[allow(clippy::too_many_arguments)]
pub(crate) fn tag_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    x: f32,
    w: f32,
    y: f32,
    row: &TagsPanelRow,
    focused: bool,
    renaming: bool,
) -> f32 {
    let id = crate::ids::row_id(row.id);
    let rect = Rect::new(x, y, w, ROW_H_PX);
    paint_row_highlight(
        scene,
        rect,
        theme,
        if focused {
            RowHighlight::Selected
        } else if store.hover_live(id) > 0.0 {
            RowHighlight::Hovered
        } else {
            RowHighlight::None
        },
        0.0,
    );
    hit_index.register(id, rect);
    let recuo = indent_px() * row.depth as f32;
    let label_x = x + recuo + Spacing::Sm.px();
    // A coluna da contagem, à direita — a largura de quatro dígitos, sempre reservada, para os
    // nomes não dançarem quando um número passa de 9 para 10.
    let count_w = text_system.prefix_width("0000", TypeToken::Sm.px());
    let label_w = (w - recuo - Spacing::Sm.px() * 2.0 - count_w).max(1.0);
    if renaming {
        // ⚠️ **O campo tapa o RÓTULO e não a linha inteira** — a contagem continua à vista, porque
        // renomear não muda quantos objectos pertencem e esconder o número ensinaria que muda.
        let input_rect = Rect::new(label_x, y, label_w, ROW_H_PX);
        hit_index.register(crate::ids::TAGS_RENAME_INPUT, input_rect);
        let (st, text, caret, anchor) = match store.get(crate::ids::TAGS_RENAME_INPUT) {
            Some(InteractiveState::TextInput {
                state,
                text,
                caret,
                selection_anchor,
            }) => (*state, text.clone(), *caret, *selection_anchor),
            _ => (TextInputState::Focused, String::new(), 0, None),
        };
        let input = TextInput::new(crate::ids::TAGS_RENAME_INPUT, "")
            .visual((st, store.hover_live(crate::ids::TAGS_RENAME_INPUT)));
        paint_text_input_with_buffer(
            &input,
            Some(text.as_str()),
            Some(caret),
            anchor,
            input_rect,
            scene,
            text_system,
            theme,
        );
    } else {
        paint_text(
            text_system,
            scene,
            &row.label,
            label_x,
            y + (ROW_H_PX - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            label_w,
            resolve(
                if focused {
                    ColorToken::Text1
                } else {
                    ColorToken::Text2
                },
                theme,
            ),
        );
    }
    // ⚠️ **A contagem é a de quem PERTENCE** (com a subárvore), e não a dos ids directos: é a
    // mesma resposta que o `Select` dá e que um sinal alcança. Duas contagens na mesma tela seriam
    // duas respostas à mesma pergunta.
    let n = row.members.to_string();
    let n_w = text_system.prefix_width(&n, TypeToken::Sm.px());
    paint_text(
        text_system,
        scene,
        &n,
        x + w - Spacing::Sm.px() - n_w,
        y + (ROW_H_PX - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        count_w,
        resolve(
            if row.members == 0 {
                ColorToken::TextDisabled
            } else {
                ColorToken::Text3
            },
            theme,
        ),
    );
    y + ROW_H_PX
}

/// ⭐⭐ **A RECUSA, na linha em que ela aconteceu** — nunca num toast.
///
/// A frase vem de `ph2d_tags::TagError::message`, ao lado da lei que a produziu; este painel pinta
/// um texto que não interpreta (a lei do `TextRow.problem` do L-System).
#[allow(clippy::too_many_arguments)]
pub(crate) fn problem_line(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    depth: usize,
    text: &str,
) -> f32 {
    let recuo = indent_px() * depth as f32;
    paint_text(
        text_system,
        scene,
        text,
        x + recuo + Spacing::Sm.px(),
        y + (ROW_H_PX - TypeToken::Xs.px()) * 0.5,
        TypeToken::Xs.px(),
        (w - recuo - Spacing::Sm.px() * 2.0).max(1.0),
        resolve(ColorToken::Danger, theme),
    );
    y + ROW_H_PX
}

/// **A linha do vazio** — o que se lê num projecto sem tag nenhuma.
///
/// ⛔ Um painel em branco lê-se como partido; esta linha diz o gesto que o enche.
pub(crate) fn empty_line(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    paint_text(
        text_system,
        scene,
        "No tags yet. Press + New to make the first one.",
        x + Spacing::Sm.px(),
        y + (ROW_H_PX - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        (w - Spacing::Sm.px() * 2.0).max(1.0),
        resolve(ColorToken::Text3, theme),
    );
    y + ROW_H_PX
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linha(members: usize, subtree: usize, depth: usize) -> TagsPanelRow {
        TagsPanelRow {
            id: 1,
            label: "Enemy".into(),
            depth,
            members,
            subtree,
        }
    }

    /// ⭐⭐ **O rótulo do `Delete` é o TEXTO do estrago** — a metade que a costura mede pela
    /// largura, aqui medida na letra.
    ///
    /// **Mutações que devem sangrar:** o rótulo fixo · trocar `members` por `subtree`.
    #[test]
    fn the_delete_verb_spells_out_what_it_takes() {
        let com_filhos = verbs(Some(&linha(5, 3, 0)));
        let rotulo = |v: &[Verb], id| {
            v.iter()
                .find(|x| x.id == id)
                .map(|x| x.label.clone())
                .unwrap_or_default()
        };
        assert_eq!(
            rotulo(&com_filhos, crate::ids::TAGS_DELETE),
            "Delete (3 tags, 5 objects)"
        );
        // ⚠️ Uma FOLHA não diz «1 tags» — a frase seria verdadeira e ilegível.
        let folha = verbs(Some(&linha(1, 1, 0)));
        assert_eq!(
            rotulo(&folha, crate::ids::TAGS_DELETE),
            "Delete (1 objects)"
        );
        assert_eq!(rotulo(&folha, crate::ids::TAGS_SELECT), "Select (1)");
    }

    /// ⛔ **Sem linha em mãos há UM verbo**, e uma RAIZ não recebe *Move to root*.
    ///
    /// **Mutação que deve sangrar:** devolver a lista inteira sempre.
    #[test]
    fn the_bar_only_offers_verbs_that_have_a_subject() {
        assert_eq!(verbs(None).len(), 1);
        let raiz = verbs(Some(&linha(1, 1, 0)));
        assert!(!raiz.iter().any(|v| v.id == crate::ids::TAGS_UNPARENT));
        let filha = verbs(Some(&linha(1, 1, 1)));
        assert!(filha.iter().any(|v| v.id == crate::ids::TAGS_UNPARENT));
    }
}

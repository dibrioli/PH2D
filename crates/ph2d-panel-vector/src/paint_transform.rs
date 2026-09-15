//! Numeric Transform section painter for the Vector Style panel: the X/Y
//! (position) and W/H (size) fields of the selected path's bbox. Split from
//! `paint_sections` to keep that file under the 600-LOC panel cap; it's an
//! `impl BodyCtx` block over there.

use crate::paint_sections::BodyCtx;
use crate::state;
use ph2d_i18n::tr;

/// ⭐⭐⭐ **A UNIDADE que o artista lê, como [`Unit`] — para ela ir DENTRO da caixa.**
///
/// ⛔⛔ **Ordem do dono, 2026-09-15:** *«Coloque no padrão: Position X/Y Quadro Quadro. Rotation com
/// nome completo. Veja no inspector»*. E a §14 do Inspector já o fazia desde o dia anterior: ali o
/// nome diz `Position X / Y` e a CAIXA diz `12,5 px`.
///
/// ⚠️⚠️ **Isto apaga uma excepção inteira desta secção.** O cabeçalho dizia `Transform (px)` — a
/// unidade reivindicada para a secção toda —, e por isso a rotação tinha de se auto-rotular `R°`
/// para não mentir (*«um campo que se auto-rotula é mais barato que uma excepção escrita num
/// doc-comment que o artista não lê»*). Com a unidade na CAIXA, cada linha diz a sua: os quatro
/// números dizem `px`/`m` e a rotação diz `deg`. ⇒ **o `R°` deixa de ter razão de existir**, e o
/// nome dele passa a ser `Rotation`, por extenso, como o dono pediu.
///
/// ⚠️ O sufixo publicado pela shell é a FONTE (`state::length_suffix`); aqui ele só é traduzido
/// para o vocabulário do campo. ⛔ Guardar a escala aqui seria a segunda cópia da regra.
fn length_unit() -> ph2d_editor_core::widget::Unit {
    match state::length_suffix() {
        "px" => ph2d_editor_core::widget::Unit::Px,
        _ => ph2d_editor_core::widget::Unit::Meters,
    }
}

impl BodyCtx<'_> {
    /// Numeric Transform — X/Y (position) + W/H (size) of the selected path's
    /// anchor bbox, two 2-column rows. Hidden when no path is selected.
    pub(crate) fn transform_section(&mut self, y: f32) -> f32 {
        if state::current_transform().is_none() {
            return y;
        }
        // ⭐ **O cabeçalho deixou de reivindicar a unidade** — ela vive agora em cada caixa.
        let (mut y, collapsed) = self.section_header(
            ph2d_tool_vector::ids::VECTOR_SECTION_TRANSFORM,
            tr("panel.vector.section.transform"),
            y,
        );
        if collapsed {
            return y;
        }
        let comprimento = length_unit();
        // ⭐⭐ **A coluna é da SECÇÃO, medida uma vez sobre os três nomes** — ver
        //    [`ph2d_editor_core::property_row::Seccao`] e o report do dono de 2026-09-15.
        let sec = ph2d_editor_core::property_row::Seccao::medida(
            self.text_system,
            2,
            &[
                tr("panel.vector.transform.position"),
                tr("panel.vector.transform.size"),
                tr("panel.vector.transform.rotation"),
            ],
        );
        // ⭐⭐⭐ **O PADRÃO DO INSPECTOR** — um nome para o PAR, duas caixas na mesma linha.
        y = self.fields_row(
            tr("panel.vector.transform.position"),
            &[
                ph2d_tool_vector::ids::VECTOR_TRANSFORM_X,
                ph2d_tool_vector::ids::VECTOR_TRANSFORM_Y,
            ],
            Some(comprimento),
            y,
            sec,
        );
        y = self.fields_row(
            tr("panel.vector.transform.size"),
            &[
                ph2d_tool_vector::ids::VECTOR_TRANSFORM_W,
                ph2d_tool_vector::ids::VECTOR_TRANSFORM_H,
            ],
            Some(comprimento),
            y,
            sec,
        );
        // A rotação é um arrasto RELATIVO (graus por gesto, em torno do centro da caixa) — uma
        // caixa só, e hoje com o nome por extenso e a unidade dentro dela.
        y = self.fields_row(
            tr("panel.vector.transform.rotation"),
            &[crate::ids::VECTOR_TRANSFORM_R],
            Some(ph2d_editor_core::widget::Unit::Degrees),
            y,
            sec,
        );
        // **Resize Box** (plano UI/UX W3b) — o que a ALÇA do gizmo faz a este objeto: reescrever
        // a caixa, ou escalar a pose (que é herdada pelos filhos — o certo para objeto de game).
        //
        // ⚠️ Ele mora AQUI, ao lado do W/H, e não numa seção própria: o `W`/`H` já reescrevem a
        // caixa, então esta linha diz *"a alça faz o mesmo que estes campos"*. Numa seção Frame
        // ela seria inalcançável para os FILHOS, que é metade do pedido.
        //
        // `None` = a seleção não tem resposta (nada, ou seleção múltipla) e a linha não existe.
        if let Some(on) = state::resize_box() {
            y = self.checkbox_row(
                crate::ids::VECTOR_TRANSFORM_RESIZE_BOX,
                tr("panel.vector.transform.resize_box"),
                on,
                y,
            );
        }
        // "Set Center" — arma a edição de pivô; a próxima pressão no canvas põe a
        // ORIGEM da entidade ali (a geometria desloca junto, a forma não se move).
        // ADR-0112: o pivô nasce no centro da forma; este botão o move.
        let label = if state::pivot_edit_armed() {
            "Click canvas to set center"
        } else {
            "Set Center"
        };
        self.action_button(crate::ids::VECTOR_PIVOT_EDIT, label, y)
    }
}

#[cfg(test)]
mod tests {
    use super::length_unit;
    use crate::state;

    /// ⭐⭐⭐ **A UNIDADE que a shell publicou chega à CAIXA** — e este gate substitui o que
    /// defendia a lei contrária.
    ///
    /// ⛔⛔ **O que estava aqui afirmava que *«o cabeçalho carrega a unidade que a shell
    /// publicou»***, e era a lei certa enquanto o sufixo vivia no título da secção. A ordem do dono
    /// de 2026-09-15 (*«Coloque no padrão … Veja no inspector»*) inverteu-a: a unidade vive na
    /// CAIXA, e o cabeçalho voltou a ser só `Transform`.
    ///
    /// ⚠️ *Um gate que defende o comportamento anterior tem de ser invertido em voz alta, com a
    /// razão ao lado* — apagá-lo em silêncio deixaria a propriedade nova sem quem a cobre.
    ///
    /// **Mutação que tem de sangrar:** o `length_unit` devolver sempre a mesma unidade — aí a caixa
    /// diria `m` sobre números que estão em pixels, que é pior do que não dizer nada.
    #[test]
    fn the_field_shows_the_unit_the_shell_published() {
        state::set_length_suffix("px");
        assert_eq!(length_unit().suffix(), "px", "a shell publicou pixels");
        state::set_length_suffix("m");
        assert_eq!(length_unit().suffix(), "m", "a shell publicou metros");
        // E as duas são DIFERENTES — uma unidade constante seria pior que unidade nenhuma.
        state::set_length_suffix("px");
        let px = length_unit();
        state::set_length_suffix("m");
        assert_ne!(px, length_unit(), "duas unidades, duas caixas");
    }

    /// ⚠️ **E o cabeçalho deixou de a reivindicar** — sem isto, a secção podia voltar a dizer
    /// `Transform (px)` com as caixas a dizerem `m`, e o artista teria duas respostas.
    #[test]
    fn the_section_header_no_longer_claims_a_unit() {
        let head = ph2d_i18n::tr("panel.vector.section.transform");
        assert!(
            !head.contains('('),
            "o cabecalho da seccao nao carrega unidade nenhuma: {head}"
        );
    }
}

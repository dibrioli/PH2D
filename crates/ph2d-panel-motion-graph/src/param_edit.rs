//! ⭐⭐⭐ **ESCREVER O NÚMERO DE UM PARAM NO CARTÃO** — report do Enio, 2026-09-05:
//! *«vários nós não permitem clicar no número para usar o teclado para escrever»*.
//!
//! Um arrasto responde *«um pouco mais»*; só o teclado responde *«exactamente 1250»*. Enquanto
//! o painel lateral existir o artista tem os dois — mas o ciclo 1 tira o painel
//! ([doc 103](../../../docs/Motion%20Nodes/103_dinamica_dos_ciclos.md)), e um cartão que não
//! sabe receber um número escrito faria a saída do painel **perder** uma capacidade. É a mesma
//! régua do gate `no_param_the_panel_offers_falls_off_the_card`, um nível abaixo: não basta o
//! param aparecer, tem de se poder **fazer com ele o que o painel fazia**.
//!
//! ## É a irmã do [`crate::rename`], de propósito
//!
//! As duas são a mesma coisa — *um campo efémero que toma o teclado, vive sobre o sítio onde o
//! valor se lê, e comita por `Enter`* — então têm a mesma forma: o **buffer vive no
//! `WidgetStore`** (nunca uma segunda cópia da string aqui), o foco assenta-se DEPOIS dos
//! gestos (`settle_focus`), o desenho é o último do quadro, e a caixa morre quando o sujeito
//! dela desaparece.
//!
//! ## O gesto é o CLIQUE, e o arrasto continua a varrer
//!
//! A row do cartão não é um slider que salta para onde se toca — ela varre em RELATIVO, que é o
//! *number field* do Blender, e ali um clique sem arrasto entra em edição de texto. Então:
//! **arrastar = varrer · clicar = escrever**, e nenhum dos dois precisa de um modificador.
//! (Um enum e um interruptor ficam com o clique que já tinham: ali *o seguinte* é a resposta,
//! e não há número nenhum para escrever.)
//!
//! ⚠️ **A faixa que o teclado aceita NÃO é a do arrasto** — é a `hard_min..hard_max` do
//! [`CardParam`], a lei do doc 88 que o painel já tinha: o `Radius` arrasta-se de `1` a `20`
//! porque é aí que é útil, e escreve-se até `1 000 000` porque é aí que ainda é um número.

use crate::geom::{self, View};
use crate::hits::param_edit_id;
use crate::snapshot::{CardParam, GraphIntent, GraphViewSnapshot, push_intent};
use crate::state::{MotionGraphPanelState, ParamEdit};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore, format_number};
use ph2d_editor_core::widget::TextInputState;
use ph2d_editor_core::zones::Rect;

/// **A altura da caixa** — a row do app (`ROW_H_PX`), e não a da faixa do cartão: o texto de um
/// campo é chrome de tamanho fixo, então uma caixa que encolhesse com o zoom ficaria com o
/// número a transbordar dela. Ela abre CENTRADA na row, e a `param_row_is_grabbable` garante
/// que ninguém a abre num cartão minúsculo.
fn box_h() -> f32 {
    ph2d_tokens::ROW_H_PX
}

/// A largura mínima de uma caixa — abaixo disto um número de seis dígitos não cabe, e o
/// artista escreveria às cegas.
const MIN_BOX_W: f32 = 96.0; // LITERAL-PX-OK: smallest legible number box

/// ⭐ **ABRIR A CAIXA sobre a row** — semeada com o valor vivo e com a faixa DIGITÁVEL.
///
/// A semente é [`format_number`], a formatação canónica da casa (a mesma do chip do painel), e
/// **não** o texto que a row desenha: a row mostra `0.50` porque a faixa do cartão é estreita,
/// e comitar essa leitura destruiria um `0.503` em quem só abriu a caixa e carregou `Enter`.
pub(crate) fn arm(state: &mut MotionGraphPanelState, node: u32, row: u16, p: &CardParam) {
    state.param_edit = Some(ParamEdit {
        node,
        row,
        param: p.hint.param,
        seed: format_number(f64::from(p.value)),
        min: f64::from(p.hard_min),
        max: f64::from(p.hard_max),
        step: f64::from(p.step),
        face_scale: p.face_scale,
        text: false,
        opened: false,
    });
}

/// ⭐⭐ **ABRIR A CAIXA DE TEXTO** — um nome de coluna, um sinal, uma fórmula.
///
/// ⛔⛔ **A semente é o valor INTEIRO, e não o que a row desenha.** A row carrega um
/// [`crate::RowText`] — o que **cabe** na largura do cartão —, e semear com ele destruiria o
/// resto da string no `Enter`. É a mesma armadilha que o [`arm`] já nomeia para o número
/// (*a row mostra `0.50` e comitar isso mata um `0.503`*), e aqui ela é pior: ali perdiam-se
/// casas decimais, aqui perde-se metade de uma fórmula.
///
/// ⇒ o valor vem do canal lateral que a shell publica ([`crate::snapshot::card_text_of`]).
/// ⚠️ **Sem ele publicado a caixa NÃO abre** — abrir vazia sobre um valor que existe é a forma
/// de apagar o que lá estava com um `Enter` distraído.
pub(crate) fn arm_text(
    state: &mut MotionGraphPanelState,
    node: u32,
    row: u16,
    param: &'static str,
) {
    let Some(seed) = crate::snapshot::card_text_of(node, param) else {
        return;
    };
    state.param_edit = Some(ParamEdit {
        node,
        row,
        param,
        seed,
        min: 0.0,
        max: 0.0,
        step: 0.0,
        face_scale: 1.0,
        text: true,
        opened: false,
    });
}

/// Onde a caixa se desenha e se aponta: **sobre a row**, centrada, com a altura de chrome e uma
/// largura mínima. `None` quando o sujeito já não está no ecrã — o nó foi apagado, a secção
/// dobrou por cima dele, ou o artista entrou noutro nível.
///
/// ⚠️ **Reencontra o param pelo NOME, não pelo índice de faixa.** A faixa é uma coordenada de
/// desenho: dobrar uma secção acima renumera-a, e uma caixa que confiasse no número passaria a
/// escrever noutro param sem uma palavra.
pub(crate) fn box_rect(edit: &ParamEdit, snap: &GraphViewSnapshot, view: &View) -> Option<Rect> {
    let n = snap.nodes.iter().find(|n| n.id == edit.node)?;
    let i = edit.row as usize;
    match geom::band_at(n, i)? {
        geom::BandRow::Param(k) if n.params[k].hint.param == edit.param => {}
        _ => return None,
    }
    let row = geom::param_row_rect(n, view, i);
    let w = row.w.max(MIN_BOX_W);
    let h = box_h();
    Some(Rect::new(
        row.x + (row.w - w) * 0.5,
        row.y + (row.h - h) * 0.5,
        w,
        h,
    ))
}

/// **Quem tem o teclado** — assentado DEPOIS dos gestos do quadro, exactamente como no
/// [`crate::rename`] e pela mesma razão: é um gesto que abre a caixa, e o `WidgetStore` só é
/// escrito depois deles. Fechada a caixa, o foco volta ao grafo — senão `A` deixaria de abrir a
/// paleta e passaria a escrever um "a" num buffer que ninguém vê.
pub(crate) fn settle_focus(
    state: &mut MotionGraphPanelState,
    ctx: &mut ph2d_editor_core::panel::PaintCtx,
) {
    match state.param_edit.as_mut() {
        Some(e) if !e.opened => {
            open_box(ctx.host.store_mut(), e);
            e.opened = true;
        }
        None if ctx.host.store().focus_id() == Some(param_edit_id()) => {
            ctx.host.store_mut().set_focus(None);
        }
        _ => {}
    }
}

/// Regista a caixa **com o texto todo seleccionado** e dá-lhe o teclado — uma vez, no quadro em
/// que abre. Seleccionado inteiro é a convenção de todo campo que abre já cheio: o primeiro
/// caractere substitui, e o número antigo ainda ali está para quem só queria emendar um dígito.
fn open_box(store: &mut WidgetStore, e: &ParamEdit) {
    let id = param_edit_id();
    if e.text {
        // O molde é o do [`crate::rename`] — o mesmo campo efémero, o mesmo buffer no store.
        store.register(
            id,
            InteractiveState::TextInput {
                state: TextInputState::Focused,
                text: e.seed.clone(),
                caret: e.seed.chars().count(),
                selection_anchor: Some(0),
            },
        );
        store.set_focus(Some(id));
        store.mark_cancel_on_escape(id);
        return;
    }
    let value = e.seed.parse::<f64>().unwrap_or(0.0);
    store.register(
        id,
        InteractiveState::NumberInput {
            state: TextInputState::Focused,
            value,
            buffer: e.seed.clone(),
            caret: e.seed.chars().count(),
            last_committed: value,
            selection_anchor: Some(0),
        },
    );
    // A faixa **digitável** (e o passo, que é quem faz as setinhas andarem de um em um num
    // param inteiro). O `NumberInput` clampa a ela ao comitar.
    store.set_number_range(id, e.min, e.max, e.step.max(f64::EPSILON));
    store.set_focus(Some(id));
}

/// Desenha a caixa sobre a row — e, quando o sujeito dela desapareceu, fecha-a.
///
/// Uma caixa a flutuar sobre nada, ainda com o teclado, é a forma de um defeito que come todos
/// os atalhos do editor. Por isso é o desenho que descobre que ela sobreviveu ao sujeito: o
/// snapshot é a verdade sobre o que existe, e este é o quadro que a lê.
pub(crate) fn paint(
    state: &mut MotionGraphPanelState,
    ctx: &mut ph2d_editor_core::panel::PaintCtx,
    snap: &GraphViewSnapshot,
    view: &View,
) {
    let Some(e) = state.param_edit.as_ref() else {
        return;
    };
    let Some(field) = box_rect(e, snap, view) else {
        state.param_edit = None; // o sujeito sumiu; o `settle_focus` devolve o teclado
        return;
    };
    let theme = ctx.host.theme();
    if e.text {
        let (fstate, texto, caret, anchor) = match ctx.host.store().get(param_edit_id()) {
            Some(InteractiveState::TextInput {
                state,
                text,
                caret,
                selection_anchor,
            }) => (*state, text.clone(), *caret, *selection_anchor),
            // No quadro em que abre ela ainda não está registada — desenha-se semeada, para
            // nunca piscar vazia (a mesma nota do `rename`).
            _ => (TextInputState::Focused, e.seed.clone(), 0, None),
        };
        let input = ph2d_editor_core::widget::TextInput::new(param_edit_id(), "")
            .visual((fstate, ctx.host.store().hover_live(param_edit_id())));
        ph2d_editor_core::widget::paint_text_input_with_buffer(
            &input,
            Some(&texto),
            Some(caret),
            anchor,
            field,
            ctx.scene,
            ctx.text_system,
            theme,
        );
        ctx.host.hit_index_mut().register(param_edit_id(), field);
        return;
    }
    let (fstate, value, buf, caret, anchor) =
        ph2d_editor_core::widget::showcase::read_number_input(ctx.host.store(), param_edit_id());
    // No quadro em que a caixa abre ela ainda não está registada (um gesto abriu-a, e o store é
    // escrito depois dos gestos): desenha-se semeada, para nunca piscar vazia.
    let (fstate, value, buf, caret, anchor) = if ctx.host.store().get(param_edit_id()).is_some() {
        (fstate, value, buf.to_string(), caret, anchor)
    } else {
        (
            TextInputState::Focused,
            e.seed.parse::<f64>().unwrap_or(0.0),
            e.seed.clone(),
            e.seed.chars().count(),
            Some(0),
        )
    };
    let input = ph2d_editor_core::widget::NumberInput::new(param_edit_id(), "", value)
        .step(e.step.max(f64::EPSILON))
        .min(e.min)
        .max(e.max)
        .visual((fstate, ctx.host.store().hover_live(param_edit_id())));
    ph2d_editor_core::widget::paint_number_input_with_buffer(
        &input,
        Some(&buf),
        caret,
        anchor,
        field,
        ctx.scene,
        ctx.text_system,
        theme,
    );
    ctx.host.hit_index_mut().register(param_edit_id(), field);
}

/// **O número escrito vira o valor do param.** Sai pela porta que já existe
/// (`GraphIntent::SetParam`), a mesma do arrasto e a mesma da row do painel — o undo, o memo do
/// cook e os limites são os mesmos nas três superfícies.
///
/// ⚠️ **Não fecha a caixa**: quem a fecha é o `Blur`, que chega a seguir ao `Enter` e chega
/// **sozinho** no `Esc`. Fechá-la aqui deixaria o `Esc` a não fechar nada.
pub(crate) fn commit(state: &MotionGraphPanelState, store: &WidgetStore) {
    let Some(e) = state.param_edit.as_ref() else {
        return;
    };
    if e.text {
        let Some(InteractiveState::TextInput { text, .. }) = store.get(param_edit_id()) else {
            return;
        };
        push_intent(GraphIntent::SetTextParam {
            node: e.node,
            param: e.param,
            value: text.clone(),
        });
        return;
    }
    let Some(v) = store.number_value(param_edit_id()) else {
        return;
    };
    // Um param inteiro arredonda, a mesma leitura do passo que o arrasto e a row fazem.
    // ⚠️ **O arredondamento é na FACE**, antes da conversão: escalar e arredondar não comutam, e
    // é a face que o artista vê inteira.
    let v = if e.step >= 1.0 { v.round() } else { v };
    let escala = f64::from(e.face_scale);
    let guardado = if escala.is_finite() && escala > 0.0 {
        v / escala
    } else {
        v
    };
    push_intent(GraphIntent::SetParam {
        node: e.node,
        param: e.param,
        value: guardado as f32,
    });
}

#[cfg(test)]
#[path = "param_edit_tests.rs"]
mod tests;

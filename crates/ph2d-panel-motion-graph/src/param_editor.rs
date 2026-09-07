//! ⭐⭐⭐ **O EDITOR RICO DE UM PARAM, ABERTO SOBRE O CARTÃO** — a curva (e, a seguir, o gradiente
//! e a paleta) que o censo do `panel_exit_probe` conta como inalcançáveis pelo cartão.
//!
//! ⚠️ **Porque FLUTUA em vez de crescer dentro do cartão.** O cartão tem `190 px` de largura em
//! espaço de grafo; o editor de gradiente precisa de `~208` só para os cinco botões do cabeçalho,
//! e o de curva de uma tela quadrada. Um editor embutido teria de encolher — e encolher um
//! controlo que se ARRASTA é tirar-lhe a precisão, que é a única coisa que ele oferece. A janela
//! flutuante é **chrome**: não escala com o zoom, então a caixa é a mesma com o grafo perto ou
//! longe.
//!
//! ⚠️ **E é o precedente da própria casa:** a amostra de cor de um cartão já abre um selector
//! flutuante muito maior que ela. Um gradiente e uma paleta são editores de COR — os irmãos
//! diretos daquela amostra.
//!
//! ⚠️ **A largura deriva dos MESMOS tokens** de que o teto de paradas do gradiente foi medido
//! (`PANEL_MIN_W_PX − 2 × PANEL_HEAD_PAD_PX`), e não é coincidência: aquele teto é *a superfície
//! mais estreita a dividir por um alvo de ponteiro*, e se esta janela fosse mais estreita que a
//! superfície contra a qual ele foi derivado, o número deixaria de descrever o produto.

use crate::snapshot::{CardParam, GraphIntent, push_intent};
use crate::state::MotionGraphPanelState;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{fill_rounded_rect, resolve};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::widget::ButtonState;
use ph2d_editor_core::zones::Rect;
use ph2d_param_editors::EditorKey;
use ph2d_tokens::{ColorToken, PANEL_HEAD_PAD_PX, PANEL_MIN_W_PX, Radius, Theme};

/// A folga entre a janela e a borda do canvas — ela nunca encosta.
const MARGIN: f32 = 8.0; // LITERAL-PX-OK: floating editor margin from the canvas edge

/// **Que editor está aberto** — a espécie decide o que se desenha e o que uma edição escreve.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum EditorKind {
    /// Uma curva de transferência: tela quadrada com pontos que se arrastam.
    Curve,
}

/// **O editor aberto.** Um de cada vez, e é a lei: ele é uma janela modal-por-costume (o clique
/// fora fecha-a), e duas abertas dariam dois donos ao mesmo teclado e ao mesmo arrasto.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Open {
    pub node: u32,
    pub param: &'static str,
    /// O rótulo do param — o cabeçalho diz DE QUE param é a caixa.
    pub title: &'static str,
    pub kind: EditorKind,
    /// Canto superior esquerdo em ECRÃ, fixado na ABERTURA.
    ///
    /// ⚠️ **Fixado, e não seguido do cartão:** o grafo pode ser deslocado com o editor aberto, e
    /// uma janela que perseguisse a row saltaria debaixo do dedo a meio de um arrasto.
    pub screen: (f32, f32),
}

/// A CHAVE deste editor — com o NÓ dentro, porque há vinte cartões visíveis ao mesmo tempo.
/// ⚠️ Devolve as duas strings porque a [`EditorKey`] as empresta.
pub(crate) fn key_of(node: u32, param: &str) -> (String, String) {
    (
        format!("card/editor/{node}/{param}"),
        format!("card/editor_swatch/{node}/{param}"),
    )
}

/// **Este param abre um editor rico?** — a porta única, lida pelo gesto e pelo pintor da row.
pub(crate) fn kind_of(p: &CardParam) -> Option<EditorKind> {
    match p.hint.widget {
        ph2d_node_registry::ParamWidget::Curve => Some(EditorKind::Curve),
        _ => None,
    }
}

/// ⭐ **Abre o editor** — ancorado por baixo da row que o pediu.
///
/// ⚠️ **Sem valor publicado não abre**, e é a mesma lei da caixa de texto: um editor semeado com
/// o vazio sobre um valor que existe apagá-lo-ia no primeiro arrasto. Um param ainda por autorar
/// publica `""`, e nesse caso o editor abre na identidade — que é o que a folha já faz.
pub(crate) fn arm(state: &mut MotionGraphPanelState, node: u32, p: &CardParam, row: Rect) {
    let Some(kind) = kind_of(p) else { return };
    if crate::snapshot::card_text_of(node, p.hint.param).is_none() {
        return;
    }
    state.editor = Some(Open {
        node,
        param: p.hint.param,
        title: p.hint.label,
        kind,
        screen: (row.x, row.y + row.h),
    });
}

/// A janela, em ecrã — recortada para caber no canvas. `None` quando nada está aberto.
pub(crate) fn window(state: &MotionGraphPanelState, canvas: Rect) -> Option<Rect> {
    let open = state.editor.as_ref()?;
    let w = PANEL_MIN_W_PX;
    let h = 2.0f32.mul_add(PANEL_HEAD_PAD_PX, content_h(open));
    // Encostada para dentro quando não cabe onde foi aberta — uma janela meio fora do canvas
    // tem metade dos controlos inalcançáveis.
    let x = open
        .screen
        .0
        .min(canvas.x + canvas.w - w - MARGIN)
        .max(canvas.x + MARGIN);
    let y = open
        .screen
        .1
        .min(canvas.y + canvas.h - h - MARGIN)
        .max(canvas.y + MARGIN);
    Some(Rect::new(x, y, w, h))
}

/// A altura do CONTEÚDO — a folha é quem a sabe, porque é ela que desenha.
fn content_h(open: &Open) -> f32 {
    match open.kind {
        EditorKind::Curve => ph2d_param_editors::curve::height(),
    }
}

/// ⭐⭐ **Desenha a janela e regista os widgets dela.** Corre DEPOIS de tudo o resto, e por isso
/// os widgets do editor ganham qualquer hit do grafo por baixo — a mesma razão da caixa de
/// escrever um número.
pub(crate) fn paint(state: &MotionGraphPanelState, ctx: &mut PaintCtx, canvas: Rect, theme: Theme) {
    let (Some(open), Some(win)) = (state.editor.as_ref(), window(state, canvas)) else {
        return;
    };
    fill_rounded_rect(
        ctx.scene,
        win,
        Radius::Md.px(),
        resolve(ColorToken::Bg2, theme),
    );
    ph2d_editor_core::paint::stroke_frame(
        ctx.scene,
        win,
        Radius::Md.px(),
        theme,
        ph2d_tokens::visuals::Feel::Rest,
        1.0,
        resolve(ColorToken::Border, theme),
    );
    let (own, swatch) = key_of(open.node, open.param);
    let key = EditorKey {
        own: &own,
        swatch: &swatch,
    };
    let valor = crate::snapshot::card_text_of(open.node, open.param).unwrap_or_default();
    let x = win.x + PANEL_HEAD_PAD_PX;
    let w = win.w - 2.0 * PANEL_HEAD_PAD_PX;
    let y = win.y + PANEL_HEAD_PAD_PX;
    let mut sacola = ph2d_param_editors::curve::CurveWidgets::new();
    match open.kind {
        EditorKind::Curve => {
            ph2d_param_editors::curve::paint(
                open.title,
                &valor,
                key,
                x,
                w,
                y,
                ph2d_tokens::TypeToken::Base.px(),
                ctx.host.hit_index_mut(),
                ctx.scene,
                ctx.text_system,
                theme,
                &mut sacola,
            );
        }
    }
    register(ctx.host.store_mut(), &sacola);
}

/// A fase MUTÁVEL: as alças viram `CurvePoint` (o despacho normaliza o arrasto contra a tela) e
/// os botões viram botões. ⚠️ **Sem isto o editor desenha e não obedece a nada** — é a metade que
/// o pintor não consegue fazer, porque ali o store é emprestado imutável.
fn register(store: &mut WidgetStore, sacola: &ph2d_param_editors::curve::CurveWidgets) {
    for &(id, parent, index, canvas) in &sacola.points {
        store.register(
            id,
            InteractiveState::CurvePoint {
                parent,
                channel: 0,
                index,
                canvas,
            },
        );
    }
    for &id in &sacola.buttons {
        store.register(
            id,
            InteractiveState::Button {
                state: ButtonState::Normal,
            },
        );
    }
}

/// ⭐⭐ **O ARRASTO DE UMA ALÇA** — o despacho guardou o ponto normalizado; aqui ele é dobrado no
/// texto e sai pela porta de texto que a row do painel já usa.
///
/// Devolve se o evento era deste editor.
pub(crate) fn on_drag(
    state: &MotionGraphPanelState,
    store: &mut WidgetStore,
    id: ph2d_a11y::NodeId,
) -> bool {
    let Some(open) = state.editor.as_ref() else {
        return false;
    };
    let (own, swatch) = key_of(open.node, open.param);
    let key = EditorKey {
        own: &own,
        swatch: &swatch,
    };
    if id != key.root() {
        return false;
    }
    let valor = crate::snapshot::card_text_of(open.node, open.param).unwrap_or_default();
    if let Some(novo) = ph2d_param_editors::curve::drain_drag(store, key, &valor) {
        push_intent(GraphIntent::SetTextParam {
            node: open.node,
            param: open.param,
            value: novo,
        });
    }
    true
}

/// ⭐⭐ **UM BOTÃO DO EDITOR** (`+` / `−` / interp). Devolve se o clique era deste editor.
pub(crate) fn on_click(state: &MotionGraphPanelState, id: ph2d_a11y::NodeId) -> bool {
    let Some(open) = state.editor.as_ref() else {
        return false;
    };
    let (own, swatch) = key_of(open.node, open.param);
    let key = EditorKey {
        own: &own,
        swatch: &swatch,
    };
    let valor = crate::snapshot::card_text_of(open.node, open.param).unwrap_or_default();
    let novo = if id == key.sub("add") {
        ph2d_param_editors::curve::add_point(&valor)
    } else if id == key.sub("remove") {
        ph2d_param_editors::curve::remove_point(&valor, key)
    } else if id == key.sub("interp") {
        ph2d_param_editors::curve::cycle_interp(&valor, key)
    } else {
        return false;
    };
    push_intent(GraphIntent::SetTextParam {
        node: open.node,
        param: open.param,
        value: novo,
    });
    true
}

#[cfg(test)]
#[path = "param_editor_tests.rs"]
mod tests;

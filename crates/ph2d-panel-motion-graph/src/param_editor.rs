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

/// ⭐⭐ **QUANTO MAIOR ESTA JANELA É QUE A ROW DO PAINEL** — decisão do dono (Enio, 2026-09-07:
/// *«a janela do ramp ficou pequena, aumente uns 30%»*), depois de a ver a funcionar.
///
/// ⚠️ **Ele multiplica a GEOMETRIA e nada mais:** a barra fica mais longa (logo, uma parada
/// arrasta-se com mais precisão) e as amostras ficam maiores (logo, acertam-se melhor) — o
/// número de paradas, as posições e as cores são os mesmos.
///
/// ⛔ **E não move o teto de paradas** (`MAX_GRADIENT_STOPS`): ele é derivado da superfície mais
/// ESTREITA em que o editor vive, que continua a ser a row do painel. *Folga não é licença.*
const JANELA: f32 = 1.3; // LITERAL-PX-OK: FACTOR adimensional, nao uma medida — o «uns 30%» do dono

/// **Que editor está aberto** — a espécie decide o que se desenha e o que uma edição escreve.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum EditorKind {
    /// Uma curva de transferência: tela quadrada com pontos que se arrastam.
    Curve,
    /// Um gradiente: barra com paradas que se arrastam e uma amostra por parada.
    Gradient,
    /// Uma paleta: tira de amostras que embrulha, com `+` e `−`.
    Palette,
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

/// ⭐⭐⭐ **O ID DA `i`-ÉSIMA AMOSTRA de um editor aberto no CARTÃO** — a porta que a SHELL lê
/// para saber que parada / que cor o selector aberto está a editar.
///
/// ⚠️ **Ela é `pub` por necessidade, não por conveniência:** o selector OKLCH abre por REGISTO
/// (o `pointer_down` do `editor-core` vê a marca), e quem lê a escolha de volta para dentro da
/// string é a shell, que tem o documento. Se a shell derivasse o id com uma segunda cópia desta
/// string, a escolha do artista cairia num id que ninguém pintou — e nada no ecrã diria porquê.
#[must_use]
pub fn card_editor_swatch_id(node: u32, param: &str, i: usize) -> ph2d_a11y::NodeId {
    let (own, swatch) = key_of(node, param);
    EditorKey {
        own: &own,
        swatch: &swatch,
    }
    .swatch_id(i)
}

/// **Este param abre um editor rico?** — a porta única, lida pelo gesto e pelo pintor da row.
pub(crate) fn kind_of(p: &CardParam) -> Option<EditorKind> {
    match p.hint.widget {
        ph2d_node_registry::ParamWidget::Curve => Some(EditorKind::Curve),
        ph2d_node_registry::ParamWidget::Gradient => Some(EditorKind::Gradient),
        ph2d_node_registry::ParamWidget::Palette => Some(EditorKind::Palette),
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
    let w = PANEL_MIN_W_PX * JANELA;
    let valor = crate::snapshot::card_text_of(open.node, open.param).unwrap_or_default();
    let h = 2.0f32.mul_add(
        PANEL_HEAD_PAD_PX * JANELA,
        content_h(open, &valor, w - 2.0 * PANEL_HEAD_PAD_PX * JANELA),
    );
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
fn content_h(open: &Open, value: &str, w: f32) -> f32 {
    match open.kind {
        EditorKind::Curve => ph2d_param_editors::curve::height(JANELA),
        EditorKind::Gradient => ph2d_param_editors::gradient::height(JANELA),
        // ⚠️ Só a paleta depende do VALOR: a tira dela embrulha, e a caixa cresce com o número
        // de cores.
        EditorKind::Palette => ph2d_param_editors::palette::height(value, w, JANELA),
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
    let x = win.x + PANEL_HEAD_PAD_PX * JANELA;
    let w = win.w - 2.0 * PANEL_HEAD_PAD_PX * JANELA;
    let y = win.y + PANEL_HEAD_PAD_PX * JANELA;
    // ⚠️ A LETRA cresce com a caixa: um rótulo do tamanho de sempre numa janela 30% maior
    // lê-se como uma janela mal preenchida, e é o mesmo texto que diz de que param ela é.
    let fonte = ph2d_tokens::TypeToken::Base.px() * JANELA;
    let mut curva = ph2d_param_editors::curve::CurveWidgets::new();
    let mut cor = ph2d_param_editors::gradient::ColourRowWidgets::new();
    match open.kind {
        EditorKind::Curve => {
            ph2d_param_editors::curve::paint(
                open.title,
                &valor,
                key,
                x,
                w,
                y,
                fonte,
                JANELA,
                ctx.host.hit_index_mut(),
                ctx.scene,
                ctx.text_system,
                theme,
                &mut curva,
            );
        }
        EditorKind::Gradient => {
            ph2d_param_editors::gradient::paint(
                open.title,
                &valor,
                key,
                x,
                w,
                y,
                fonte,
                JANELA,
                ctx.host.hit_index_mut(),
                ctx.scene,
                ctx.text_system,
                theme,
                &mut cor,
            );
        }
        EditorKind::Palette => {
            ph2d_param_editors::palette::paint(
                open.title,
                &valor,
                key,
                x,
                w,
                y,
                fonte,
                JANELA,
                ctx.host.hit_index_mut(),
                ctx.scene,
                ctx.text_system,
                theme,
                &mut cor,
            );
        }
    }
    register(ctx.host.store_mut(), &curva, &cor);
}

/// A fase MUTÁVEL: as alças viram `CurvePoint` (o despacho normaliza o arrasto contra a tela) e
/// os botões viram botões. ⚠️ **Sem isto o editor desenha e não obedece a nada** — é a metade que
/// o pintor não consegue fazer, porque ali o store é emprestado imutável.
fn register(
    store: &mut WidgetStore,
    curva: &ph2d_param_editors::curve::CurveWidgets,
    cor: &ph2d_param_editors::gradient::ColourRowWidgets,
) {
    // ⭐⭐ **Uma AMOSTRA de parada abre o selector OKLCH** — e a marca é a mesma que a amostra
    // de uma cor simples do cartão usa. ⚠️ **Ela é semeada com a cor de agora**, senão o
    // selector abre no que lá estava da última vez e o artista escolhe a partir do sítio
    // errado.
    for &(id, srgb) in &cor.swatches {
        store.set_widget_color(id, srgb);
        store.register_picker_swatch(id);
    }
    for &(id, parent, index, canvas) in curva.points.iter().chain(cor.markers.iter()) {
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
    for &id in curva.buttons.iter().chain(cor.buttons.iter()) {
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
    let novo = match open.kind {
        EditorKind::Curve => ph2d_param_editors::curve::drain_drag(store, key, &valor),
        EditorKind::Gradient => ph2d_param_editors::gradient::drain_drag(store, key, &valor),
        // ⛔ Uma paleta não tem POSIÇÕES para arrastar — ela é uma lista ordenada, e as
        // amostras dela abrem o selector. Não há arrasto para drenar.
        EditorKind::Palette => None,
    };
    if let Some(novo) = novo {
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
    // ⚠️ **Os NOMES dos botões são partilhados entre as espécies** (`add`, `remove`, `interp`) —
    // e as leis não: acrescentar uma parada não é acrescentar um ponto. A espécie decide, e é
    // por isso que o `match` vem primeiro e o id depois.
    let novo = match open.kind {
        EditorKind::Curve => {
            if id == key.sub("add") {
                ph2d_param_editors::curve::add_point(&valor)
            } else if id == key.sub("remove") {
                ph2d_param_editors::curve::remove_point(&valor, key)
            } else if id == key.sub("interp") {
                ph2d_param_editors::curve::cycle_interp(&valor, key)
            } else {
                return false;
            }
        }
        EditorKind::Gradient => {
            if id == key.sub("add") {
                ph2d_param_editors::gradient::add_stop(&valor)
            } else if id == key.sub("remove") {
                ph2d_param_editors::gradient::remove_stop(&valor, key)
            } else if id == key.sub("interp") {
                ph2d_param_editors::gradient::cycle_interp(&valor, key)
            } else if id == key.sub("space") {
                ph2d_param_editors::gradient::cycle_space(&valor)
            } else if id == key.sub("hue") {
                ph2d_param_editors::gradient::cycle_hue(&valor)
            } else if let Some(p) = (0..ph2d_param_editors::gradient::PRESET_COUNT)
                .find(|&p| id == key.sub(&format!("preset/{p}")))
            {
                // Um molde CARREGA as paradas dele na rampa editável — ele não é um modo.
                ph2d_param_editors::gradient::preset_gradient(p)
            } else {
                return false;
            }
        }
        // ⚠️ A paleta não tem `interp` nem espaço: ela é uma lista de cores, não uma
        // interpolação entre elas.
        EditorKind::Palette => {
            if id == key.sub("add") {
                ph2d_param_editors::palette::add_color(&valor)
            } else if id == key.sub("remove") {
                ph2d_param_editors::palette::remove_color(&valor)
            } else {
                return false;
            }
        }
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

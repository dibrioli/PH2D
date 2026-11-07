//! ⭐ **O modal «Collision Layers»** (onda 8, dono 07/11): a matriz INTEIRA das 32 camadas, no centro da
//! janela, por cima de tudo. Abre do «Edit matrix…» do Inspector e do painel; fecha com Esc, com o X e
//! clicando fora. As células são os MESMOS ids da matriz de sempre: o despacho (`event.rs`), o undo, o
//! Auto Key, o arrasto que pinta e o losango não mudam.
//!
//! Os nomes: o rótulo à esquerda de cada linha e o de cima de cada coluna (rodado) dizem o nome dado ou
//! «Layer n»; duplo clique em qualquer um abre o campo ali mesmo (Enter grava, Esc desiste).
//! Pairar numa célula acende a linha, a coluna e os dois nomes (como a matriz da Unity).

use std::cell::Cell;

use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::camada_das_listas;
use ph2d_editor_core::paint::{
    fill_rounded_rect, frame_radius, paint_text, paint_text_rotated_ccw, resolve,
};
use ph2d_editor_core::widget::{
    Button, ButtonState, Modal, TextInput, paint_modal, paint_text_input_with_buffer,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_physics_ecs::{LayerMatrix, MAX_LAYERS};
use ph2d_tokens::{ColorToken, Radius, Spacing, StrokeToken, TypeToken};

use crate::ids as i;

thread_local! {
    static ABERTO: Cell<bool> = const { Cell::new(false) };
    /// A linha pedida pelo «Edit matrix…» do Inspector: fica acesa enquanto o modal estiver aberto.
    static LINHA: Cell<Option<usize>> = const { Cell::new(None) };
    /// A camada cujo nome se está a escrever.
    static EDITANDO: Cell<Option<usize>> = const { Cell::new(None) };
}

/// O modal está aberto?
#[must_use]
pub fn aberto() -> bool {
    ABERTO.get()
}

/// A linha em destaque no modal.
#[must_use]
pub fn linha_em_destaque() -> Option<usize> {
    LINHA.get().filter(|_| aberto())
}

/// A camada cujo nome está em edição.
#[must_use]
pub fn editando() -> Option<usize> {
    EDITANDO.get().filter(|_| aberto())
}

/// Abre o modal; `linha` = a camada do objeto (Inspector), acesa.
pub fn abre(linha: Option<usize>) {
    ABERTO.set(true);
    LINHA.set(linha);
    EDITANDO.set(None);
    ph2d_editor_core::interaction::dispatch::key::esc_fecha(Some(i::PHYSICS_MATRIX_CLOSE));
}

/// Fecha o modal (Esc, X, fora). Um nome a meio perde-se: Esc nunca grava.
pub fn fecha() {
    ABERTO.set(false);
    LINHA.set(None);
    EDITANDO.set(None);
    ph2d_editor_core::interaction::dispatch::key::esc_fecha(None);
}

pub(crate) fn comeca_a_renomear(camada: usize) {
    EDITANDO.set(Some(camada));
    ph2d_editor_core::interaction::dispatch::key::esc_fecha(None);
}

pub(crate) fn acaba_de_renomear() {
    EDITANDO.set(None);
    if aberto() {
        ph2d_editor_core::interaction::dispatch::key::esc_fecha(Some(i::PHYSICS_MATRIX_CLOSE));
    }
}

/// O que se lê de uma camada: o nome dado, ou «Layer n». O «Layer n» nunca se grava no projeto.
#[must_use]
pub fn rotulo(nomes: &ph2d_physics_ecs::LayerNames, n: usize) -> String {
    match nomes.0.get(n).map(String::as_str) {
        Some(nome) if !nome.is_empty() => nome.to_owned(),
        _ => tr_with("card.physics.layer_default", &[("n", &n.to_string())]),
    }
}

/// O resumo do cartão do painel: quantas camadas à vista e quantos pares não colidem.
#[must_use]
pub fn resumo() -> String {
    let m = LayerMatrix::from_rows(crate::state::current().vivo.layer_matrix);
    let visiveis = crate::state::visible_layers();
    let desligados = visiveis
        .iter()
        .flat_map(|&a| visiveis.iter().filter(move |&&b| b <= a).map(move |&b| (a, b)))
        .filter(|&(a, b)| !m.collides(a, b))
        .count();
    tr_with(
        "card.physics.matrix_summary",
        &[
            ("n", &visiveis.len().to_string()),
            ("off", &desligados.to_string()),
        ],
    )
}

/// ⭐ A porta do painel é a FACE da matriz no painel: um controlo com as células (um valor, como a
/// grelha de bits do Inspector), não um comando a mais.
pub(crate) fn declara_a_porta() {
    ph2d_editor_core::widget::composto::grupo(
        std::iter::once(i::PHYSICS_MATRIX_OPEN).chain(i::PHYSICS_LAYER_CELL.iter().copied()),
    );
}

/// A geometria do modal, numa janela `vp`, com a largura dos rótulos `rotulo_w`.
#[derive(Clone, Copy, Debug)]
pub struct Geometria {
    pub cartao: Rect,
    /// O canto de cima à esquerda do corpo (os rótulos de cima começam aqui, à direita dos da esquerda).
    pub x: f32,
    pub y: f32,
    pub rotulo_w: f32,
    pub passo: f32,
}

impl Geometria {
    #[must_use]
    pub fn de(vp: Rect, rotulo_w: f32) -> Self {
        let margem = Spacing::Xl3.px();
        let pad = Spacing::Lg.px();
        let n = MAX_LAYERS as f32;
        let cabeca = Modal::altura_do_cabecalho();
        let livre_h = vp.h - margem * 2.0 - cabeca - pad * 2.0 - rotulo_w;
        let livre_w = vp.w - margem * 2.0 - pad * 2.0 - rotulo_w;
        // A célula cheia é a de um campo; encolhe para as 32 caberem, nunca abaixo do alvo mínimo.
        let passo = (livre_h / n)
            .min(livre_w / n)
            .min(ph2d_tokens::ROW_H_PX)
            .max(Spacing::Md.px());
        let w = pad * 2.0 + rotulo_w + passo * n;
        let h = cabeca + pad * 2.0 + rotulo_w + passo * n;
        let cartao = Rect::new(
            vp.x + ((vp.w - w) * 0.5).max(0.0),
            vp.y + ((vp.h - h) * 0.5).max(0.0),
            w,
            h,
        );
        Self {
            cartao,
            x: cartao.x + pad,
            y: cartao.y + cabeca + pad,
            rotulo_w,
            passo,
        }
    }

    /// A célula `(linha, coluna)`.
    #[must_use]
    pub fn celula(&self, linha: usize, coluna: usize) -> Rect {
        let vao = StrokeToken::Default.px();
        Rect::new(
            self.x + self.rotulo_w + coluna as f32 * self.passo,
            self.y + self.rotulo_w + linha as f32 * self.passo,
            self.passo - vao,
            self.passo - vao,
        )
    }

    /// O rótulo à esquerda da linha `n`.
    #[must_use]
    pub fn rotulo_da_linha(&self, n: usize) -> Rect {
        Rect::new(
            self.x,
            self.y + self.rotulo_w + n as f32 * self.passo,
            self.rotulo_w,
            self.passo,
        )
    }

    /// O rótulo de cima da coluna `n`.
    #[must_use]
    pub fn rotulo_da_coluna(&self, n: usize) -> Rect {
        Rect::new(
            self.x + self.rotulo_w + n as f32 * self.passo,
            self.y,
            self.passo,
            self.rotulo_w,
        )
    }
}

/// A célula sob o ponteiro (o despacho põe-na em `Hovered` pelo índice de cliques, em ecrã).
fn pairada(ctx: &PaintCtx) -> Option<(usize, usize)> {
    let store = ctx.host.store();
    (0..MAX_LAYERS)
        .flat_map(|a| (0..=a).map(move |b| (a, b)))
        .find(|&(a, b)| {
            let id = i::PHYSICS_LAYER_CELL[i::physics_layer_cell_index(a, b)];
            matches!(
                store.button_state(id),
                Some(ButtonState::Hovered | ButtonState::Pressed)
            )
        })
}

/// Pinta o modal, se aberto. Corre no início do `paint` do painel, aberto ou fechado.
pub(crate) fn paint(ctx: &mut PaintCtx) {
    if !aberto() {
        return;
    }
    let theme = ctx.host.theme();
    let nomes = crate::state::layer_names();
    let rotulos: Vec<String> = (0..MAX_LAYERS).map(|n| rotulo(&nomes, n)).collect();
    let font = TypeToken::Sm.px();
    let rotulo_w = rotulos
        .iter()
        .map(|r| ctx.text_system.prefix_width(r, font))
        .fold(0.0_f32, f32::max)
        + Spacing::Md.px();
    let vp = ctx.viewport;
    let g = Geometria::de(vp, rotulo_w);
    let matriz = LayerMatrix::from_rows(crate::state::current().vivo.layer_matrix);
    let modal = Modal::new(
        i::PHYSICS_MATRIX_CLOSE,
        tr("card.physics.matrix_title"),
        Button::new(i::PHYSICS_MATRIX_CLOSE, ""),
        Button::new(i::PHYSICS_MATRIX_CLOSE, ""),
    )
    .sem_rodape()
    .close_state(
        ctx.host
            .store()
            .button_state(i::PHYSICS_MATRIX_CLOSE)
            .unwrap_or_default(),
    );
    let sob = pairada(ctx);
    let linha_acesa = |n: usize| sob.map(|s| s.0) == Some(n) || linha_em_destaque() == Some(n);
    let coluna_acesa = |n: usize| sob.map(|s| s.1) == Some(n);
    let editando = editando();
    let raio = frame_radius(theme, Radius::Sm.px());

    // Os textos dos nomes e o campo em edição leem o store antes de a cena ser emprestada.
    let campo = editando.map(|n| {
        let id = i::PHYSICS_LAYER_NAME[n];
        let store = ctx.host.store();
        let (texto, caret, anchor, estado) = match store.get(id) {
            Some(InteractiveState::TextInput {
                text,
                caret,
                selection_anchor,
                state,
            }) => (text.clone(), *caret, *selection_anchor, *state),
            _ => (String::new(), 0, None, Default::default()),
        };
        (n, id, texto, caret, (anchor, estado))
    });

    // A dica de cada célula diz os dois nomes por extenso.
    {
        let store = ctx.host.store_mut();
        for a in 0..MAX_LAYERS {
            for b in 0..=a {
                let id = i::PHYSICS_LAYER_CELL[i::physics_layer_cell_index(a, b)];
                let dica = tr_with(
                    "card.physics.layer_cell",
                    &[("a", &rotulos[a]), ("b", &rotulos[b])],
                );
                if store.tooltip_for(id) != Some(dica.as_str()) {
                    store.set_tooltip(id, dica);
                }
            }
            for id in [i::PHYSICS_LAYER_ROW_LABEL[a], i::PHYSICS_LAYER_COL_LABEL[a]] {
                if store.tooltip_for(id).is_none() {
                    store.set_tooltip(id, tr("card.physics.layer_label_tip"));
                }
            }
        }
    }
    // O losango da matriz (canal MUNDO `CAMADAS`): UM, no canto dos rótulos.
    let canto = Rect::new(g.x, g.y, g.rotulo_w, g.rotulo_w);
    let losango = camada_das_listas::por_cima(ctx.scene, |cena| {
        ph2d_editor_core::widget::key_mark::paint_decorator_for(
            cena,
            theme,
            canto,
            &i::PHYSICS_LAYER_CELL,
        )
    });
    let texto = &mut *ctx.text_system;
    camada_das_listas::por_cima(ctx.scene, |cena| {
        paint_modal(&modal, vp, g.cartao, cena, texto, theme);
        // As faixas acesas, por baixo das células: a linha até ao rótulo, a coluna até ao de cima.
        let faixa = resolve(ColorToken::AccentSoft, theme);
        for n in 0..MAX_LAYERS {
            if linha_acesa(n) {
                let r = g.rotulo_da_linha(n);
                let fim = g.celula(n, n);
                fill_rounded_rect(
                    cena,
                    Rect::new(r.x, r.y, fim.x + fim.w - r.x, r.h),
                    raio,
                    faixa,
                );
            }
            if coluna_acesa(n) {
                let r = g.rotulo_da_coluna(n);
                let fim = g.celula(MAX_LAYERS - 1, n);
                fill_rounded_rect(
                    cena,
                    Rect::new(r.x, r.y, r.w, fim.y + fim.h - r.y),
                    raio,
                    faixa,
                );
            }
        }
        for (n, nome) in rotulos.iter().enumerate() {
            let tinta = |aceso: bool| {
                resolve(
                    if aceso {
                        ColorToken::Text1
                    } else {
                        ColorToken::Text2
                    },
                    theme,
                )
            };
            let r = g.rotulo_da_linha(n);
            if editando != Some(n) {
                paint_text(
                    texto,
                    cena,
                    nome,
                    r.x,
                    r.y + (r.h - font) * 0.5,
                    font,
                    r.w,
                    tinta(linha_acesa(n)),
                );
            }
            let c = g.rotulo_da_coluna(n);
            paint_text_rotated_ccw(
                texto,
                cena,
                nome,
                c.x + (c.w - font) * 0.5,
                c.y + c.h - Spacing::Xs.px(),
                font,
                c.h,
                tinta(coluna_acesa(n)),
            );
        }
        for a in 0..MAX_LAYERS {
            for b in 0..=a {
                let cel = g.celula(a, b);
                let cor = if matriz.collides(a, b) {
                    ColorToken::Accent
                } else {
                    ColorToken::Bg3
                };
                fill_rounded_rect(cena, cel, raio, resolve(cor, theme));
                if a == b {
                    // FRAME-RAW-OK: a diagonal da matriz de camadas, contornada para o olho achar o eixo de simetria
                    ph2d_editor_core::paint::stroke_rounded_rect(
                        cena,
                        cel,
                        raio,
                        StrokeToken::Default.px(),
                        resolve(ColorToken::Text2, theme),
                    );
                }
            }
        }
        if let Some((n, id, buf, caret, (anchor, state))) = &campo {
            let input = TextInput::new(*id, "")
                .placeholder(rotulos[*n].clone())
                .visual((*state, 0.0));
            paint_text_input_with_buffer(
                &input,
                Some(buf),
                Some(*caret),
                *anchor,
                g.rotulo_da_linha(*n),
                cena,
                texto,
                theme,
            );
        }
    });

    // Os alvos, por ordem: o último registado ganha o clique.
    let (store, hit) = ctx.host.store_and_hit_index_mut();
    let _ = store;
    camada_das_listas::alvo(hit, i::PHYSICS_MATRIX_SCRIM, vp);
    camada_das_listas::alvo(hit, i::PHYSICS_MATRIX_CARD, g.cartao);
    ph2d_editor_core::interaction::toggle_paint::limpa_grupo(i::PHYSICS_LAYER_PAINT_GROUP);
    for a in 0..MAX_LAYERS {
        for b in 0..=a {
            let id = i::PHYSICS_LAYER_CELL[i::physics_layer_cell_index(a, b)];
            let cel = g.celula(a, b);
            camada_das_listas::alvo(hit, id, cel);
            ph2d_editor_core::interaction::toggle_paint::celula(
                id,
                i::PHYSICS_LAYER_PAINT_GROUP,
                matriz.collides(a, b),
                cel,
            );
        }
        camada_das_listas::alvo(hit, i::PHYSICS_LAYER_ROW_LABEL[a], g.rotulo_da_linha(a));
        camada_das_listas::alvo(hit, i::PHYSICS_LAYER_COL_LABEL[a], g.rotulo_da_coluna(a));
    }
    if let Some(n) = editando {
        camada_das_listas::alvo(hit, i::PHYSICS_LAYER_NAME[n], g.rotulo_da_linha(n));
    }
    camada_das_listas::alvo(hit, i::PHYSICS_MATRIX_CLOSE, modal.close_rect(g.cartao));
    if let Some(alvo) = losango {
        camada_das_listas::alvo(hit, alvo, canto);
    }
    ph2d_editor_core::widget::composto::grupo(i::PHYSICS_LAYER_CELL.iter().copied());
}

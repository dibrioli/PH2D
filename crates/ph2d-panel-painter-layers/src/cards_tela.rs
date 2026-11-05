//! ⭐ **A superfície dos widgets especiais** (curva, rampa, visor, gizmo): o que eles pedem para pintar, sem o
//! `PaintCtx`. No cartão vem do [`CardCtx`], que corre mais de uma vez por quadro; no painel de antes
//! (`PH2D_CARDS=0`), do `PaintCtx`. Escrever no store só por [`Tela::depois`]: corre depois de pintar, e por isso
//! o último passe do corpo é o que fica.

use ph2d_editor_core::interaction::{HitIndex, WidgetStore};
use ph2d_editor_core::paint::resolve;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::CardCtx;
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vector::VectorScene;
use std::cell::RefCell;

type Adiado = Box<dyn FnOnce(&mut WidgetStore)>;

thread_local! {
    static ADIADOS: RefCell<Vec<Adiado>> = const { RefCell::new(Vec::new()) };
}

/// O que um widget especial pede para pintar.
pub(crate) struct Tela<'a> {
    pub(crate) scene: &'a mut VectorScene,
    pub(crate) text_system: &'a mut TextSystem,
    pub(crate) store: &'a WidgetStore,
    pub(crate) hit_index: &'a mut HitIndex,
    pub(crate) theme: Theme,
    ink: Option<CardInk>,
}

impl<'a> Tela<'a> {
    /// A tela do corpo de um cartão.
    pub(crate) fn do_cartao<'b>(c: &'b mut CardCtx<'_>) -> Tela<'b> {
        Tela {
            scene: &mut *c.scene,
            text_system: &mut *c.text_system,
            store: c.store,
            hit_index: &mut *c.hit_index,
            theme: c.ink.theme,
            ink: Some(c.ink),
        }
    }

    /// Escreve no store depois de a lista pintar.
    pub(crate) fn depois(&self, f: impl FnOnce(&mut WidgetStore) + 'static) {
        ADIADOS.with(|a| a.borrow_mut().push(Box::new(f)));
    }

    /// A tinta desta tela: um valor à parte, porque os pintores emprestam a cena ao mesmo tempo.
    pub(crate) fn tinta(&self) -> Tinta {
        Tinta {
            ink: self.ink,
            theme: self.theme,
        }
    }

    /// O fundo de uma tela própria (curva, rampa): o tom dos campos, no cartão.
    pub(crate) fn fundo(&self) -> ph2d_vector::Color {
        self.ink.map_or_else(
            || resolve(ColorToken::Bg2, self.theme),
            |i| i.color(ColorToken::CardField),
        )
    }
}

/// As cores de uma tela: as do cartão, ou as do tema no painel de antes.
#[derive(Clone, Copy)]
pub(crate) struct Tinta {
    ink: Option<CardInk>,
    theme: Theme,
}

impl Tinta {
    /// A tela é a do corpo de um cartão, não a do painel de antes?
    pub(crate) fn no_cartao(self) -> bool {
        self.ink.is_some()
    }

    pub(crate) fn cor(self, token: ColorToken) -> ph2d_vector::Color {
        self.ink
            .map_or_else(|| resolve(token, self.theme), |i| i.color(token))
    }
}

/// Corre `f` sobre o `PaintCtx` do painel de antes, e aplica o que ficou adiado.
pub(crate) fn no_ctx<R>(ctx: &mut PaintCtx, theme: Theme, f: impl FnOnce(&mut Tela<'_>) -> R) -> R {
    let (store, hit_index) = ctx.host.store_and_hit_index_mut();
    let mut t = Tela {
        scene: &mut *ctx.scene,
        text_system: &mut *ctx.text_system,
        store,
        hit_index,
        theme,
        ink: None,
    };
    let r = f(&mut t);
    aplica(ctx.host.store_mut());
    r
}

/// Aplica as escritas adiadas, pela ordem em que foram pedidas.
pub(crate) fn aplica(store: &mut WidgetStore) {
    let lista = ADIADOS.with(|a| std::mem::take(&mut *a.borrow_mut()));
    for f in lista {
        f(store);
    }
}

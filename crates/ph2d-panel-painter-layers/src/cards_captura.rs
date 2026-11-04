//! ⭐ **O corpo antigo dentro de um cartão** (`docs/CartoesUI/13_PAINTER_PLANO.md`). O corpo do cartão só tem
//! `&WidgetStore`, corre mais de uma vez por quadro e pode voar ou estar solto; os pintores deste painel pedem o
//! `PaintCtx` inteiro (`store_mut`, `bus_mut`). Por isso cada secção corre UMA vez antes da lista, com cena e
//! alvos próprios, na posição do quadro anterior; o corpo do cartão só REPETE, transladado.
//!
//! O que os pintores guardam em coordenadas (o chip de um popover, um gizmo) sai certo com a lista parada e
//! atrasa um quadro enquanto ela anda. A cena e os alvos nunca atrasam.

use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::CardCtx;
use ph2d_editor_core::widget::card_ink::{self, CardInk};
use ph2d_editor_core::zones::Rect;
use ph2d_vector::{Affine, VectorScene};
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

/// Onde o corpo de um cartão foi pintado por último: `(x, y)`.
type Onde = BTreeMap<NodeId, (f32, f32)>;

thread_local! {
    static ONDE: RefCell<Onde> = const { RefCell::new(BTreeMap::new()) };
    /// A coluna dos corpos no último quadro: `(x, w)`.
    static COLUNA: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
}

/// A coluna do quadro anterior; sem ele, a do encaixe.
pub(crate) fn coluna(slot: Rect) -> (f32, f32) {
    use ph2d_card_stack::metrics as m;
    COLUNA.with(Cell::get).unwrap_or_else(|| {
        let recuo = m::LIST_MARGIN + m::LIST_PAD_X + m::CARD_PAD_X;
        (slot.x + recuo, (slot.w - 2.0 * recuo).max(0.0))
    })
}

/// O `y` do corpo no quadro anterior.
pub(crate) fn y_de(id: NodeId, de_outro_modo: f32) -> f32 {
    ONDE.with(|o| o.borrow().get(&id).map_or(de_outro_modo, |p| p.1))
}

/// O `x` do corpo no quadro anterior: num cartão solto, o que o pintor guarda em coordenadas tem de ser o do solto.
pub(crate) fn x_de(id: NodeId, de_outro_modo: f32) -> f32 {
    ONDE.with(|o| o.borrow().get(&id).map_or(de_outro_modo, |p| p.0))
}

/// A chave da peça `n` de uma secção, na memória da posição: fora da faixa dos ids, porque nunca é registada.
pub(crate) fn chave_da_peca(secao: NodeId, n: u64) -> NodeId {
    const FORA_DOS_IDS: u64 = 1 << 56;
    NodeId(secao.0.wrapping_add(FORA_DOS_IDS).wrapping_add(n))
}

/// Um corpo pintado à parte, pronto a repetir.
pub(crate) struct Captura {
    /// A chave da memória da posição: a secção, ou uma peça dela.
    id: NodeId,
    /// A secção do cartão onde ela se repete: é ela que voa ou se solta.
    secao: NodeId,
    cena: VectorScene,
    alvos: HitIndex,
    x: f32,
    y: f32,
    fundo: f32,
}

impl Captura {
    /// Corre `pintor` com o `PaintCtx` real.
    pub(crate) fn de(
        ctx: &mut PaintCtx,
        id: NodeId,
        ink: CardInk,
        (x, y): (f32, f32),
        pintor: impl FnOnce(&mut PaintCtx, f32) -> f32,
    ) -> Self {
        let mut cena = VectorScene::new();
        let mut alvos = HitIndex::default();
        std::mem::swap(ctx.scene, &mut cena);
        std::mem::swap(ctx.host.hit_index_mut(), &mut alvos);
        let fundo = card_ink::com(ink, || pintor(ctx, y));
        std::mem::swap(ctx.scene, &mut cena);
        std::mem::swap(ctx.host.hit_index_mut(), &mut alvos);
        Self {
            id,
            secao: id,
            cena,
            alvos,
            x,
            y,
            fundo,
        }
    }

    /// Um pedaço do corpo do cartão `secao` que ainda pede o `PaintCtx`.
    pub(crate) fn da_peca(
        ctx: &mut PaintCtx,
        chave: NodeId,
        secao: NodeId,
        ink: CardInk,
        onde: (f32, f32),
        pintor: impl FnOnce(&mut PaintCtx, f32) -> f32,
    ) -> Self {
        let mut c = Self::de(ctx, chave, ink, onde, pintor);
        c.secao = secao;
        c
    }

    /// Repete a cena e os alvos em `y`, e devolve o fundo.
    pub(crate) fn repete(&self, c: &mut CardCtx<'_>, y: f32) -> f32 {
        let (dx, dy) = (c.x - self.x, y - self.y);
        c.scene.append_transformed(
            &self.cena,
            Affine::translate((f64::from(dx), f64::from(dy))),
        );
        for (id, r) in self.alvos.iter_registrations() {
            c.hit_index
                .register(id, Rect::new(r.x + dx, r.y + dy, r.w, r.h));
        }
        self.lembra(c, y);
        y + (self.fundo - self.y)
    }

    /// O voo pinta em coordenadas locais e não entra; um solto tem o seu sítio, e não leva a coluna dos outros.
    fn lembra(&self, c: &CardCtx<'_>, y: f32) {
        let voa = c
            .store
            .card_drag()
            .is_some_and(|d| d.active && !d.loose && d.section == self.secao);
        if voa {
            return;
        }
        ONDE.with(|o| o.borrow_mut().insert(self.id, (c.x, y)));
        if !c
            .store
            .is_loose_in(self.secao, core_ids::PAINTER_LAYERS_PANEL)
        {
            COLUNA.with(|k| k.set(Some((c.x, c.w))));
        }
    }
}

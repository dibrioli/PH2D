//! ⭐ **O corpo antigo dentro de um cartão** (`docs/CartoesUI/13_PAINTER_PLANO.md`). O corpo do cartão só tem
//! `&WidgetStore`, corre mais de uma vez por quadro e pode voar ou estar solto; os pintores deste painel pedem o
//! `PaintCtx` inteiro (`store_mut`, `bus_mut`). Por isso cada secção corre UMA vez antes da lista, com cena e
//! alvos próprios, na posição do quadro anterior; o corpo do cartão só REPETE, transladado.
//!
//! O que os pintores guardam em coordenadas (o chip de um popover, um gizmo) sai certo com a lista parada e
//! atrasa um quadro enquanto ela anda. A cena e os alvos nunca atrasam.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::HitIndex;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::CardCtx;
use ph2d_editor_core::widget::absorb_next_header;
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
    static RESET: Cell<Option<NodeId>> = const { Cell::new(None) };
}

/// O Reset da secção que está a correr: passa ao cabeçalho do cartão, com o mesmo id.
pub(crate) fn declara_reset(id: NodeId) {
    RESET.with(|r| r.set(Some(id)));
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

/// Um corpo pintado à parte, pronto a repetir.
pub(crate) struct Captura {
    id: NodeId,
    cena: VectorScene,
    alvos: HitIndex,
    x: f32,
    y: f32,
    fundo: f32,
    /// O título que o cabeçalho absorvido deu. `None` numa secção: ela não está à vista.
    pub(crate) titulo: Option<String>,
    pub(crate) reset: Option<NodeId>,
    gravou: Cell<bool>,
}

impl Captura {
    /// Corre `pintor` com o `PaintCtx` real. `absorve`: o 1.º cabeçalho dá o título em vez de se pintar.
    pub(crate) fn de(
        ctx: &mut PaintCtx,
        id: NodeId,
        ink: CardInk,
        (x, y): (f32, f32),
        absorve: bool,
        pintor: impl FnOnce(&mut PaintCtx, f32) -> f32,
    ) -> Self {
        let mut cena = VectorScene::new();
        let mut alvos = HitIndex::default();
        std::mem::swap(ctx.scene, &mut cena);
        std::mem::swap(ctx.host.hit_index_mut(), &mut alvos);
        RESET.with(|r| r.set(None));
        let corre = |ctx: &mut PaintCtx| card_ink::com(ink, || pintor(ctx, y));
        let (fundo, cabecalho) = if absorve {
            absorb_next_header(|| corre(ctx))
        } else {
            (corre(ctx), None)
        };
        std::mem::swap(ctx.scene, &mut cena);
        std::mem::swap(ctx.host.hit_index_mut(), &mut alvos);
        Self {
            id,
            cena,
            alvos,
            x,
            y,
            fundo,
            titulo: cabecalho.map(|c| c.0),
            reset: RESET.with(Cell::get),
            gravou: Cell::new(false),
        }
    }

    /// A altura do que pintou.
    pub(crate) fn altura(&self) -> f32 {
        self.fundo - self.y
    }

    /// O bloco pintou alguma coisa?
    pub(crate) fn pintou(&self) -> bool {
        self.fundo > self.y
    }

    /// Repete a cena e os alvos em `y`, e devolve o fundo.
    pub(crate) fn repete(&self, c: &mut CardCtx<'_>, y: f32) -> f32 {
        let (dx, dy) = (c.x - self.x, y - self.y);
        c.scene.append_transformed(
            &self.cena,
            Affine::translate((f64::from(dx), f64::from(dy))),
        );
        for (id, r) in self.alvos.iter_registrations() {
            let r = Rect::new(r.x + dx, r.y + dy, r.w, r.h);
            match self.alvos.secondary_of(id) {
                Some(sec) => c.hit_index.register_with_secondary(id, r, sec),
                None => c.hit_index.register(id, r),
            }
        }
        self.lembra(c, y);
        y + (self.fundo - self.y)
    }

    /// O 1.º passe do quadro é o que conta; o voo pinta em coordenadas locais e não entra.
    fn lembra(&self, c: &CardCtx<'_>, y: f32) {
        let voa = c
            .store
            .card_drag()
            .is_some_and(|d| d.active && !d.loose && d.section == self.id);
        if voa || self.gravou.replace(true) {
            return;
        }
        ONDE.with(|o| o.borrow_mut().insert(self.id, (c.x, y)));
        COLUNA.with(|k| k.set(Some((c.x, c.w))));
    }
}

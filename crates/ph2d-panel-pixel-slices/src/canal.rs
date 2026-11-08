//! O canal entre a família e o painel: o que ela sabe, o que ele pede.

use std::cell::RefCell;

use ph2d_pixel_core::SliceId;
pub use ph2d_pixel_slices::campos::{Campo, Vista};

/// Uma linha da lista, já na ordem por nome (T1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Linha {
    pub id: SliceId,
    pub nome: String,
    pub selecionada: bool,
}

/// O que o painel mostra.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Contexto {
    /// A Slice Tool está na mão, com um sprite em edição.
    pub na_mao: bool,
    pub linhas: Vec<Linha>,
    /// `None`: nada selecionado.
    pub vista: Option<Vista>,
    pub transform: bool,
}

/// O menu do botão direito: onde abre, em píxeis da interface, e se há mudança para remover aqui.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Menu {
    pub x: f32,
    pub y: f32,
    pub remover: bool,
}

/// Um gesto do painel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pedido {
    Seleciona(SliceId),
    Todas,
    Nenhuma,
    Transform,
    Nome(String),
    Dados(String),
    /// O seletor de cor está nesta cor: reescreve o mesmo passo do desfazer enquanto estiver aberto.
    Cor([u8; 4]),
    /// O seletor fechou ou o arrasto da letra soltou: a próxima mudança é outro passo.
    PassoFechou,
    /// O número escrito no campo, como texto: a lei G6 é que diz o que ele vale.
    Campo(Campo, String),
    /// O número sob o arrasto da letra: reescreve o mesmo passo até soltar, como a cor.
    Arrasta(Campo, String),
    Nove(bool),
    Pivot(bool),
    Apaga,
    Duplica,
    RemoveMudanca,
}

impl Pedido {
    /// Reescreve o passo do desfazer do pedido anterior do mesmo gesto (seletor aberto, letra arrastada).
    #[must_use]
    pub const fn reescreve_o_passo(&self) -> bool {
        matches!(self, Self::Cor(_) | Self::Arrasta(..))
    }
}

#[derive(Default)]
struct Canal {
    contexto: Contexto,
    menu: Option<Menu>,
    menu_aberto: bool,
    fechar_menu: bool,
    mostrar: bool,
    pedidos: Vec<Pedido>,
}

thread_local! {
    static CANAL: RefCell<Canal> = RefCell::new(Canal::default());
}

fn com<R>(f: impl FnOnce(&mut Canal) -> R) -> R {
    CANAL.with(|c| f(&mut c.borrow_mut()))
}

pub fn publish_contexto(x: Contexto) {
    com(|c| c.contexto = x);
}

#[must_use]
pub fn contexto() -> Contexto {
    com(|c| c.contexto.clone())
}

/// A família pede o menu do botão direito.
pub fn abrir_menu(m: Menu) {
    com(|c| c.menu = Some(m));
}

#[must_use]
pub fn take_menu() -> Option<Menu> {
    com(|c| c.menu.take())
}

/// O painel diz se o menu está à vista.
pub fn publish_menu_aberto(aberto: bool) {
    com(|c| c.menu_aberto = aberto);
}

/// `Esc` com o menu à vista: fecha só o menu; `true` = consumiu.
#[must_use]
pub fn esc_fecha_menu() -> bool {
    com(|c| {
        c.fechar_menu = c.menu_aberto;
        c.menu_aberto
    })
}

#[must_use]
pub fn take_fechar_menu() -> bool {
    com(|c| std::mem::take(&mut c.fechar_menu))
}

/// `Slice Properties...` e o duplo clique: o cartão à vista, aberto.
pub fn mostrar_propriedades() {
    com(|c| c.mostrar = true);
}

#[must_use]
pub fn take_mostrar() -> bool {
    com(|c| std::mem::take(&mut c.mostrar))
}

pub fn pede(p: Pedido) {
    com(|c| c.pedidos.push(p));
}

/// Toma (e limpa) os gestos deste quadro.
#[must_use]
pub fn take_pedidos() -> Vec<Pedido> {
    com(|c| std::mem::take(&mut c.pedidos))
}

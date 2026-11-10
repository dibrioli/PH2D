// ph2d-chrome-sync:z=181 (dispatch priority, ADR-0107; lower = earlier)
//! **O INPUT MAP no chrome** — o abridor da aba (Settings ▸ Input Map…), a ESCUTA partilhada (a aba
//! central e o cartão do Trigger) e os pedidos do Trigger ao mapa (`docs/InputMap/01_plano.md` §3).
//! A janela flutuante que aqui vivia saiu na R2a (10/11): a aba central tomou-lhe o lugar.
//!
//! O mapa é dado de projecto e o dono dele é o [`HeroScreen`](crate::screens::hero::HeroScreen): o
//! handler muta `hero.input_map` directamente, sem um `EditorAction` por gesto.

/// **A metade de DESPACHO**; ver [`apply`].
mod apply;
pub use apply::apply;

/// Os pedidos que o cartão do Trigger faz ao mapa; ver [`pedidos`].
mod pedidos;
pub use pedidos::{PedidoDoMapa, aplica_pedidos_do_mapa};

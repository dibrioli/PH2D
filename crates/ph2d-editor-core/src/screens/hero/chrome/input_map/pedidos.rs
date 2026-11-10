//! **O que o cartão do Trigger pede ao Input Map** — criar a acção se faltar e esperar a tecla dela
//! (onda 4 do F3, dono 09/11; o botão que só criava saiu a 10/11: era o mesmo gesto, duas vezes).
//!
//! O mapa é estado do EDITOR (vive no [`HeroScreen`]): não viaja num `ComponentBlob` nem passa pelo
//! `Ctrl+Z` do documento. O painel pede pelo barramento, a shell drena os pedidos do quadro e chama
//! [`aplica_pedidos_do_mapa`] — a lei mora aqui, ao lado do mapa, e não na shell.

use crate::screens::hero::HeroScreen;

/// Um pedido da fileira do gatilho. O nome vem cru do campo; aqui ele é aparado.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PedidoDoMapa {
    /// Criar se faltar e armar a escuta que TROCA as teclas de teclado (o botão da linha Key).
    /// `Esc` depois deixa a acção criada, sem tecla.
    CriarEEscutar(String),
}

/// Aplica os pedidos do quadro, pela ordem.
///
/// ⚠️ `InputMap::create` é idempotente por NOME, logo dois pedidos iguais não fazem duas linhas.
/// ⚠️ Um nome vazio é recusado: uma acção sem nome é inalcançável por código.
pub fn aplica_pedidos_do_mapa(hero: &mut HeroScreen, pedidos: Vec<PedidoDoMapa>) {
    if pedidos.is_empty() {
        return;
    }
    for PedidoDoMapa::CriarEEscutar(nome) in pedidos {
        let nome = nome.trim();
        if nome.is_empty() {
            continue;
        }
        let id = hero.input_map.create(nome);
        hero.store.listen_for_binding_replacing(id);
    }
}

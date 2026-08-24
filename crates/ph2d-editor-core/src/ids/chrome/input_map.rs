//! **A JANELA DO INPUT MAP** — os `NodeId` da janela flutuante que abre sobre o canvas
//! (plano `docs/Vector Module/30_plano_input_map.md` §0.2, pedido do Enio em 2026-08-24:
//! *"equivalente ao da godot com janela flutuante abrindo sobre o canvas"*).
//!
//! ⚠️ **Ids FIXOS para o quadro, DERIVADOS para as linhas.** A janela tem uma parte que existe
//! sempre (a faixa do título, o fechar, o campo de nome, o Add) e uma lista que muda de tamanho a
//! cada acção criada — o mesmo par que o painel de tokens já usa, e pela mesma razão: um id fixo
//! por linha limitaria a lista ao número de constantes que alguém escreveu.

use super::painter::fnv_node_id_runtime;
use super::{NodeId, hash_node_id};

/// A faixa do título — **a alça de arrasto**. Um Down aqui começa a mover a janela, e ela **não**
/// pode fechar durante o movimento (a cicatriz que o `fill_modal` já registou).
pub const INPUT_MAP_HANDLE: NodeId = hash_node_id("input_map.handle");
/// O **X** que fecha a janela.
pub const INPUT_MAP_CLOSE: NodeId = hash_node_id("input_map.close");
/// O campo de texto onde se escreve o nome da acção nova.
pub const INPUT_MAP_NEW_NAME: NodeId = hash_node_id("input_map.new_name");
/// **+ Add** — cria a acção com o nome do campo acima.
pub const INPUT_MAP_ADD: NodeId = hash_node_id("input_map.add");
/// **A TECLA FOI CAPTURADA** — o `Click` sintético que o despacho de teclado emite quando a escuta
/// estava armada.
///
/// ⚠️ **Não é um botão**: nenhum pixel o desenha e nenhum hit rect o regista. Ele existe porque o
/// despacho de teclado só alcança o `WidgetStore` e a ligação precisa do `HeroScreen` — o despacho
/// **guarda** a tecla e emite isto, e o handler de chrome, que tem o hero, **liga**. *O seed é dono
/// do valor; o dispatch é dono do estado.*
pub const INPUT_MAP_BIND_CAPTURED: NodeId = hash_node_id("input_map.bind_captured");

/// O **X** da linha da acção `row` — apaga a acção inteira.
#[must_use]
pub fn input_map_delete_action_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("input_map.del_action.{row}"))
}

/// **+ Bind** da acção `row` — arma a escuta: a **próxima tecla** vira uma ligação dela.
///
/// ⚠️ **Um id por linha, e ele arma um MODO** — enquanto a escuta dura, a tecla capturada não pode
/// executar o atalho do editor. Sem isso, ligar `S` a uma acção **salva o projecto**.
#[must_use]
pub fn input_map_listen_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("input_map.listen.{row}"))
}

/// **A ZONA MORTA** da acção `row` — abaixo dela a força é `0` (o ruído do analógico).
///
/// ⚠️ Um dos **DOIS** números que substituem o de duplo propósito do Godot. Ver [`crate`].
#[must_use]
pub fn input_map_deadzone_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("input_map.deadzone.{row}"))
}

/// **O PONTO DE DISPARO** da acção `row` — acima dele `pressed` é `true`.
///
/// ⚠️ O segundo dos dois. A porta da acção impõe `press_point >= dead_zone`, então arrastar um
/// **empurra** o outro em vez de deixar nascer um estado incoerente.
#[must_use]
pub fn input_map_press_point_id(row: usize) -> NodeId {
    fnv_node_id_runtime(&format!("input_map.press_point.{row}"))
}

/// O **X** da ligação `bind` da acção `row` — apaga uma ligação só.
#[must_use]
pub fn input_map_delete_binding_id(row: usize, bind: usize) -> NodeId {
    fnv_node_id_runtime(&format!("input_map.del_bind.{row}.{bind}"))
}

//! O que a aba lê, publicado pela família a cada `frame` (plano §3).

use ph2d_input::InputMap;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Snapshot {
    /// Cópia do `hero.input_map` deste quadro (só leitura: editar é `InputMapEdit`).
    pub map: InputMap,
    /// Há um comando ligado: muda o texto da escuta e acende a parte do comando.
    pub gamepad_connected: bool,
    /// O Play corre: o Try it lê o estado do jogo e não toma teclas.
    pub play_running: bool,
}

//! ⭐ **O PISCAR da invencibilidade, como marca DERIVADA** (plano 28, W5).
//!
//! Quem a põe e a tira é a ponte da vida (`ph2d-physics-ecs`), no fim de todo `dispatch` — no tique
//! vivo **e** no replay, logo num scrub o objecto aparece ou some exactamente como naquele tique. Quem
//! a lê é a porta única do desenho (`ph2d_entity_visibility::off_canvas::draws_this_frame`), que
//! assim a aplica às imagens E às formas vectoriais sem saber o que é uma vida.
//!
//! ⚠️ **DERIVADA, e por isso NÃO registada** — o molde do [`crate::MasterPiece`]: ela é função do
//! relógio da vida, e registá-la poria cada metade do piscar na pilha de `Ctrl+Z` e no ficheiro.
//!
//! ⛔ **Não a insira à mão**: o passe seguinte da ponte re-escreve-a pela vida, e uma marca escrita
//! fora dele sobrevive até ao próximo tique — o que é pior que não existir, porque funciona uma vez.

use bevy_ecs::component::Component;

/// **Nesta metade do piscar, o objecto NÃO se desenha.** Ver o cabeçalho do módulo.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct BlinkOff;

//! **O Offset VIVO** — a forma engorda/emagrece sem que a curva autorada mude.
//!
//! Irmão do [`crate::VecBlend`] / [`crate::VecConnector`] / [`crate::VecEnvelope`] no padrão que
//! esta linha já usou três vezes: o componente guarda a **relação** (quanto, com que quina, de
//! que lado) e a aparência é uma **função pura** dela, re-cozida pela shell. Consequência de
//! graça: **undo e save cobrem o offset sem uma linha a mais** — os dois capturam o mundo ECS, e
//! este componente está registrado no `ComponentRegistry`.
//!
//! # Por que não é um `PathEffect` da pilha (ADR-0132) — e a razão é do COMPILADOR
//!
//! Um efeito da pilha é avaliado DENTRO da `ph2d-vec-scene` ([`effect::run_stack`]), e a
//! `ph2d-vec-scene` é o modelo puro de documento: sem kurbo, sem linesweeper. O offset correto
//! exige remover auto-interseções, e quem sabe fazê-lo é a `ph2d-vec-boolean` — que **depende**
//! da `ph2d-vec-scene`. Pôr a seta de volta é um ciclo, e o cargo o recusa por nome:
//!
//! ```text
//! error: cyclic package dependency: package `ph2d-vec-scene` depends on itself
//! ```
//!
//! A alternativa seria um motor REGISTRADO em runtime (um ponteiro de função global instalado
//! pela shell), e ela foi recusada por um motivo que este repo já pagou várias vezes: o
//! `cooked()` é a resposta à pergunta *"o que este documento desenha?"*, e fazê-la depender de
//! alguém ter chamado um instalador significa que o MESMO documento desenha coisas diferentes em
//! processos diferentes — uma porta que pode não ter sido aberta, falhando em silêncio.
//!
//! O segundo fato, medido: `offset_path` devolve `Vec<VecPath>`, e mais de um é o caso NORMAL —
//! o donut do smoke com `Side=Inner` a `d=+0.6` devolve **oito** caminhos. Um efeito é
//! `VecPath -> VecPath`.
//!
//! # A distância é de MUNDO
//!
//! O slider fala fração (`fração × maxdim/2` da bbox de mundo da seleção — a lei aprovada em
//! `ada45fac`), e o que fica guardado aqui é o `d` que essa lei produziu. Guardar a fração
//! obrigaria a re-derivar a escala a cada cozimento, e a escala é da SELEÇÃO — mudaria de sentido
//! ao selecionar outra coisa.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **O Offset vivo de uma forma.** A entidade que o carrega também tem um [`crate::VecPathRef`]:
/// o `VecPath` dela continua sendo a curva **AUTORADA** (é ela que o modo Node edita), e o
/// resultado offsetado é DESENHO — a shell o coze por frame e o passe de render o desenha no
/// lugar da fonte, no z dela.
///
/// ⚠️ **`join`/`side` são os CÓDIGOS do painel** (`ph2d_panel_vector::expand_join/side`), não
/// enums próprios: a tradução código→`LineJoin`/`OffsetSide` já existe numa porta única
/// (`vec_expand::offset_join`/`offset_side`), e uma segunda tabela aqui divergiria dela no dia
/// em que aparecesse um 4º estilo de quina.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecOffset {
    /// A distância em unidades de MUNDO. Negativo encolhe. `0` não acontece: a shell REMOVE o
    /// componente em vez de guardar um offset inerte (senão um documento acumularia relações
    /// invisíveis que não desenham nada).
    pub d: f64,
    /// O estilo de quina, no código do painel: `0` Miter · `1` Round · `2` Bevel.
    pub join: u8,
    /// Que contorno anda, no código do painel: `0` Outer · `1` Inner · `2` Both.
    pub side: u8,
}

impl SimComponent for VecOffset {}

impl VecOffset {
    /// Um offset vivo novo.
    #[must_use]
    pub fn new(d: f64, join: u8, side: u8) -> Self {
        Self { d, join, side }
    }
}

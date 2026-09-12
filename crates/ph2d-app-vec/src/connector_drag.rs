//! **O estado dos DOIS gestos do conector** — construir a linha (`ConnectorDrag`) e arrastar uma
//! alça dela (`HandleDrag`, com o `Grab` e o `EndSide` que ele carrega). Só os TIPOS: os gestos
//! (`impl App`) e a regra pura do largar ficam na shell (`connector_gesture.rs`,
//! `connector_handles.rs`).
//!
//! ⚠️ **Desceram da shell em 2026-09-12** (`line/render-loop`, A9 da auditoria de arquitectura): a
//! `App` guardava os dois gestos em campos soltos (`vec_connect`, `vec_conn_handle`) cujos TIPOS
//! moravam na shell, e um campo assim não pode juntar-se ao [`crate::state::VecState`]. Os campos
//! que eram privados ao módulo da shell passaram a `pub`: quem os lê e escreve são os mesmos dois
//! gestos, agora do outro lado da fronteira.

use ph2d_ecs::{ConnectorEnd, VecConnector};
use ph2d_vec_scene::VecPathId;

/// O conector em construção (Down..Up). O `path` já existe na cena; o `conn` é a relação que
/// o `connector_live` re-cozinha.
#[derive(Clone, Debug)]
pub struct ConnectorDrag {
    pub path: VecPathId,
    pub conn: VecConnector,
    /// Onde o gesto começou (mundo) — para descartar o clique-sem-arrasto que não prendeu
    /// nada (uma linha de comprimento zero solta no vazio não é um objeto, é um acidente).
    pub start_world: [f64; 2],
}

/// Qual das duas pontas.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum EndSide {
    Start,
    End,
}

/// O que a pressão agarrou.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Grab {
    /// Uma das duas pontas (o círculo).
    End(EndSide),
    /// Um ponto de passagem (o quadradinho), pelo índice.
    Waypoint(usize),
    /// **O CORPO da linha, ainda sem nada criado** — o waypoint só nasce se o gesto virar um
    /// ARRASTO. `usize` = o índice em que ele entrará.
    ///
    /// Isto existe por causa de um conflito real: o **duplo-clique** num conector abre o rótulo
    /// dele, e o primeiro clique do par caía no corpo da linha. Criando o waypoint já no `Down`,
    /// todo duplo-clique fincava um ponto de passagem antes de abrir o texto. Um clique e um
    /// arrasto **não são o mesmo gesto**, e a fronteira entre eles é o movimento — não a
    /// pressão.
    Body(usize),
}

/// O arrasto de uma alça (Down..Up).
#[derive(Clone, Debug)]
pub struct HandleDrag {
    pub path: VecPathId,
    pub grab: Grab,
    /// A ponta como ela **estava** (só para [`Grab::End`]). Durante o arrasto a ponta vira
    /// `Free` (para a linha seguir o cursor ao vivo, que é o preview), e o largar resolve a
    /// partir DAQUI — senão o vínculo original se perderia no meio do caminho e "afastar sem
    /// soltar" viraria "soltar".
    pub original: ConnectorEnd,
    /// O waypoint NASCEU neste gesto — se o gesto for cancelado, ele é desfeito.
    pub born_now: bool,
    /// Onde a pressão começou (mundo) — a régua do limiar que separa o clique do arrasto.
    pub start: [f64; 2],
}

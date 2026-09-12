//! **O que a escultura guarda na `App` e SÓ existe com o módulo ligado** (W2/L3-A2, ADR-0075).
//!
//! Eram quatro campos soltos no `app_state.rs`. ⛔ **Eles não puderam juntar-se ao
//! [`ph2d_app_sculpt3d::Sculpt3dRequests`]**, e a fronteira não foi escolhida — foi *imposta*:
//! os quatro carregam **tipos do módulo** ([`super::LoadedPiece`],
//! [`super::entities::SculptRowsSeen`]), logo têm de viver atrás de `#[cfg(feature =
//! "sculpt3d")]`; e uma struct gateada não pode guardar o `doc`, cujo valor inteiro é
//! **sobreviver a um binário que não tem a feature**.
//!
//! ⇒ *dois estados e não um, com a `cfg` a decidir o corte.* Quando `LoadedPiece` e
//! `SculptRowsSeen` saírem para a crate (Fase B), os dois fundem-se e este ficheiro morre.

/// O estado da escultura que a `App` guarda e que só faz sentido com o módulo compilado.
#[derive(Default)]
pub(crate) struct Sculpt3dShellState {
    /// A escultura **já decodificada**, esperando o device (ADR-0150 W8.3).
    ///
    /// ⚠️ Ela é decodificada no LOAD, e não aqui: a recusa de um documento ilegível tem de
    /// acontecer **antes** de qualquer mutação da sessão, e isso exige lê-lo. Guardar o
    /// resultado evita a segunda decodificação — que é `O(vértices)` com octree e adjacência
    /// POR NÍVEL.
    pub(crate) pending: Option<(Vec<super::LoadedPiece>, usize)>,
    /// **As peças da escultura que já tiveram linha na Hierarquia** (ADR-0150).
    ///
    /// ⚠️ Um CONJUNTO de ids, e não um mapa de bits: o mapa é o próprio mundo, lido a cada
    /// quadro — ver o doc de [`super::entities`], onde está por que um mapa guardado apagaria
    /// a escultura no primeiro Ctrl+Z.
    pub(crate) rows: super::entities::SculptRowsSeen,
    /// **O *Duplicate* de uma linha de escultura, pedido e drenado no quadro seguinte**
    /// (`(id da peça de origem, bits da entidade cópia)`).
    ///
    /// ⚠️ **Um PEDIDO**, como os quatro do [`ph2d_app_sculpt3d::Sculpt3dRequests`], e pela
    /// mesma razão: duplicar uma peça precisa da cena, e no ponto em que a Hierarquia responde
    /// ao clique ela está emprestada pelo laço do quadro. ⛔ Ele fica **deste** lado da
    /// fronteira só porque o par `(u32, u64)` viaja junto do `pending` — se algum dia se
    /// mudar sozinho, o sítio dele é lá.
    pub(crate) dup: Option<(u32, u64)>,
    /// **A última selecção da Hierarquia que a escultura já leu** — o detector de MUDANÇA que
    /// impede os dois escritores de `active` (a linha escolhida e o `aim` do pen-down) de
    /// brigarem a cada quadro. Ver [`super::entities`].
    pub(crate) sel: Option<u64>,
}

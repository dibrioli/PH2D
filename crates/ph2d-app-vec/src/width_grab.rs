//! **A parada de largura agarrada pelo Width Tool** (plano 25 §5, ADR-0148) — o TIPO. O gesto e as
//! alças ficam na shell (`width_handles.rs`); o perfil mora no componente, e toda escrita passa
//! pelo [`crate::profile_live::arm`].
//!
//! ⚠️ **Desceu da shell em 2026-09-12** (`line/render-loop`, A9 da auditoria de arquitectura): o
//! campo `vec_width_grab` da `App` tinha o TIPO na shell, e um campo assim não pode juntar-se ao
//! [`crate::state::VecState`].

use ph2d_vec_scene::VecPathId;

/// A parada agarrada por um arrasto.
// Sem `Eq`: o `pos` é `f64`. `PartialEq` basta — ninguém usa isto como chave.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Grab {
    /// O caminho cuja largura está a ser editada.
    pub path: VecPathId,
    /// O índice da parada na lista.
    pub stop: usize,
    /// Esta parada NASCEU neste gesto (o press foi sobre a curva, não sobre uma alça) e o dedo
    /// **ainda não a moveu**. O 1º arrasto o derruba.
    ///
    /// ⚠️ **Existe por causa de um número medido, e é o que torna a inserção invisível.** O
    /// `smoothstep` liga paradas CONSECUTIVAS, então inserir uma re-parametriza os dois vãos que
    /// ela divide: o desvio máximo é **13,1% da faixa do perfil** — e é estrutural, o MESMO em
    /// todo perfil (é o máximo entre um smoothstep e dois meio-smoothsteps), não um acidente de
    /// dados. Trocar por interpolação LINEAR tornaria a inserção exata e poria um VINCO em cada
    /// parada, que é o que o `WidthProfile` recusa desde o 1º dia.
    ///
    /// A cura não é a interpolação, é o GESTO: com o Width Tool cria-se um ponto de largura
    /// **arrastando** a partir da curva (é o que o Illustrator faz), e um clique que não moveu
    /// nada não pediu nada — então ele é desfeito no release e o desenho não muda. Quem arrasta
    /// nunca vê os 13,1%, porque a espessura já está a mudar sob o dedo.
    pub created: bool,
    /// **Onde a parada estava quando o gesto começou**, em fração de arco.
    ///
    /// ⚠️ **Existe porque o índice sozinho MENTE numa forma virgem**, e o defeito era visível:
    /// a parada criada nasce com o multiplicador que o perfil já tem ali (para o desenho não
    /// saltar), então num perfil neutro a lista continua UNIFORME — e o `arm` remove um perfil
    /// uniforme (o neutro-é-ausência, a lei deste módulo). O `press` devolvia então um índice
    /// para uma lista que nunca foi guardada, e o `drag` seguinte relia o NEUTRO (duas paradas)
    /// e editava a de índice 1: **a ponta do traço**. MEDIDO: o 1º gesto do Width numa forma
    /// virgem levava `[(0, 1), (1, 1)]` a `[(0, 1), (0.241, 5)]` — o artista puxava no meio e o
    /// FIM do traço dele mudava de sítio, com a metade final a engrossar toda.
    pub pos: f64,
}

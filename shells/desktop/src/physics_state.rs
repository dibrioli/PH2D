//! `PhysicsState` — o estado de runtime que **só a família `physics` lê**, num
//! campo só de [`crate::App`] (W2/L2, ADR-0075).
//!
//! Antes desta wave os sete campos abaixo viviam soltos em `App`, espalhados por
//! quatro regiões do `app_state.rs` (linhas 861, 902, 1488–1515) e por três
//! sítios do `App::new()`. Sete campos de uma família num `struct` de ~401 é a
//! forma de que a auditoria de velocidade fala em §4-C2: *a metade da feature que
//! fala com a `App` foi ficando na shell*. Juntá-los num agregado com dono dá
//! três coisas de uma vez: a família passa a ter **um** endereço, a `App` perde
//! seis campos, e o corte da W2-Fase B passa a mover um `struct` em vez de
//! reconciliar sete linhas dispersas com o que outras cinco linhas escreveram no
//! mesmo ficheiro.
//!
//! ⚠️ **O molde é o [`crate::motion_state::MotionState`]**, que já faz isto para o
//! Motion (e cujo doc diz, por sua vez, que espelha o `AppGfx.vec_scene`) — é o
//! padrão da casa, não um desenho novo. A diferença é **onde** ele é segurado: o
//! `MotionState` mora no `AppGfx`, que só existe depois do `resumed`; estes sete
//! têm de existir **sem janela** (o `joint_draw_armed` é armado por um botão do
//! §11 e os gates headless lêem-no), então o dono é a `App`.
//!
//! ⛔ **Isto NÃO é o estado do solver.** O mundo rígido vive no `rapier` através
//! da ponte (`render_loop::physics_bridge`) e a autoria vive nos componentes do
//! `ph2d-physics-ecs` — os dois são documento, viajam no arquivo e passam pelo
//! undo. O que está aqui é o **transiente do ponteiro e da sessão**: o gesto em
//! curso, o que o próximo clique vai fazer, e o que a cena de smoke já montou.
//! Nada aqui bumpa schema, e é por isso que a wave inteira não move um único
//! contador partilhado.

/// O transiente da família `physics`, com um dono só.
///
/// `Default` é o que o `App::new()` escrevia campo a campo — e a equivalência é
/// verificável linha a linha contra o git: `interaction` pelo `Default` do
/// wrapper (que é onde as constantes medidas do W-Grab vivem), `join_kind = 0`
/// (Pin, o tipo com que o *Join Selected Bodies* nasce), e os cinco restantes no
/// zero/`None` que já tinham.
#[derive(Default)]
pub(crate) struct PhysicsState {
    /// Como o ponteiro interage com os corpos (ferramenta, modo de segurar, tipo
    /// de junta da mão).
    ///
    /// Transiente, e é decisão (ver `ph2d_physics_ecs::interaction`): descreve o
    /// PONTEIRO, não a cena, então nada disto viaja no arquivo de projeto e
    /// nenhum schema bumpa. O painel de física a EXIBE por uma porta só, sem
    /// guardar cópia.
    pub(crate) interaction: ph2d_physics_ecs::InteractionSettings,

    /// `JointKind` tag (`0` Pin · `1` Spring · `2` Rope · `3` Weld). O selector
    /// de tipo do §11 escreve-o; o `create_joint` lê-o, então o artista cria a
    /// junta do TIPO que quer num gesto só. Runtime-only: é uma escolha de UI
    /// pendente, não o documento. Nasce em Pin.
    pub(crate) join_kind: u8,

    /// **O arrasto de âncora de junta em curso** (W-J2), ou `None`. As duas alças
    /// de canvas — o ponto A cheio e o anel B vazado — abrem este mesmo gesto,
    /// que escreve pela porta de âncora da ponte; ver [`crate::joint_anchor_drag`].
    /// Runtime-only: o arrasto não é o documento, a âncora que ele escreve é.
    pub(crate) joint_anchor_drag: Option<crate::joint_anchor_drag::JointAnchorDrag>,

    /// Armado pelo botão *Draw Joint* do §11, desarmado por uma criação
    /// completa — e deliberadamente NÃO por uma recusa, para que um release no
    /// vazio deixe o gesto pronto para outra tentativa (o precedente do
    /// eyedropper). Runtime-only: é o que o ponteiro está prestes a fazer, não o
    /// documento.
    pub(crate) joint_draw_armed: bool,

    /// O gesto em curso, ou `None`. Ver [`crate::joint_draw`].
    pub(crate) joint_draw: Option<crate::joint_draw::JointDraw>,

    /// A entidade cujo readout de player o laço imprime, ou `None`.
    ///
    /// ⚠️ **Um campo e não uma env var lida por quadro:** quem sabe QUAL entidade
    /// é o sujeito é a cena que a montou, e re-perguntar ao ambiente obrigaria o
    /// laço a adivinhar (o primeiro player? o selecionado?) — uma segunda
    /// resposta para algo que a cena já sabe.
    pub(crate) player_readout_log: Option<u64>,

    /// O prólogo da cena de smoke já correu neste processo.
    ///
    /// ⚠️ **Uma vez por processo, não por quadro:** o prólogo só pode montar a
    /// cena depois de o `gfx` existir, então ele é tentado a cada quadro até
    /// conseguir — e sem esta trava a segunda tentativa montaria a cena outra vez
    /// por cima da primeira.
    pub(crate) smoke_done: bool,
}

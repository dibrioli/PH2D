//! ⭐ **O VOCABULÁRIO DO ARRASTO** — o que um botão em baixo significa.
//!
//! Irmão (`#[path]`) de [`super`] pelo tecto de LOC de 600 linhas, e o corte é
//! por **responsabilidade**: o pai responde *o que a cena É* (os objectos, a
//! câmera, os viewports, o pincel, o histórico) e este responde *como se nomeia
//! um gesto que está a correr*.
//!
//! ⚠️ **Os dois tipos viajam juntos porque um é o estado do outro**: a
//! [`TwistSweep`] só existe enquanto um [`Drag`] que gira estiver vivo, e
//! separá-los poria metade de um gesto em cada ficheiro.

/// O que o arrasto está fazendo.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Drag {
    Orbit,
    Pan,
    Sculpt,
    /// **O TRANSFORM ARMADO** — o botão esquerdo move/gira/escala a parte livre
    /// em vez de esculpir. Ver `sculpt3d_transform`.
    Transform,
    /// **O FILTRO ARMADO** — o arrasto horizontal dá a força com que o verbo
    /// corrente roda na malha INTEIRA. Ver `sculpt3d_filter`.
    Filter,
}

/// **O ângulo VARRIDO desde o pen-down**, acumulado evento a evento.
///
/// ⚠️ **Acumulado, e não um `atan2` da direção inicial à atual** — este é o
/// único jeito de uma varredura passar de meia volta. Um ângulo com sinal
/// satura em `±π`, então a 181° ele voltaria a `−179°` e a torção **inverteria**
/// no meio do gesto. Somando os deltas (que são pequenos) o total cresce sem
/// teto, e a soma é EXATA: ângulos se somam, então subdividir o caminho não
/// muda o resultado — que é o que o [`Grip::Turn`] exige do gesto que o
/// alimenta.
pub(super) struct TwistSweep {
    /// A última direção unitária *âncora → cursor*, em componentes de CÂMERA.
    /// `None` enquanto o cursor está dentro da zona morta: sem direção não há
    /// delta a somar, e a próxima saída re-semeia sem inventar um salto.
    pub(super) last: Option<[f32; 2]>,
    pub(super) total: f32,
}

//! ⭐ **Tocar só um trecho** entre dois marcadores, em loop (doc 17, item 2; maquete B6). TRANSITÓRIO:
//! não grava, não desfaz, não reescreve o `loop_range` do clipe. Sobrepõe o loop vivo do relógio do
//! editor enquanto vale; ✕, outro clipe, outra aba, um contêiner ou um marcador que mudou devolvem o
//! loop do clipe (uma porta: [`arma`]).

use crate::doc::TimelineDoc;
use crate::state::TimelineState;
use ph2d_core::Playhead;

/// O trecho escolhido: os dois marcadores (índice + o segundo em que estavam) e a vista onde nasceu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Trecho {
    /// Marcador do início (índice de armazenamento).
    pub a: usize,
    /// Marcador do fim.
    pub b: usize,
    /// O segundo de `a` quando foi escolhido.
    pub ta: f64,
    /// O segundo de `b`.
    pub tb: f64,
    /// O clipe activo de então.
    pub clip: usize,
    /// A aba Keys (`true`) ou Arrange.
    pub keys: bool,
}

/// Onde o trecho se escolhe e vive: a régua que mostra os marcadores e TOCA (o relógio do editor).
#[must_use]
pub fn vista_com_trecho(keys: bool, stacked: bool, container: Option<usize>, list: bool) -> bool {
    !(keys && stacked) && container.is_none() && !list
}

impl Trecho {
    /// O trecho de `a` a `b`, se os dois existem e `a` vem antes de `b`.
    #[must_use]
    pub fn novo(
        t_de: impl Fn(usize) -> Option<f64>,
        a: usize,
        b: usize,
        clip: usize,
        keys: bool,
    ) -> Option<Self> {
        let (ta, tb) = (t_de(a)?, t_de(b)?);
        (tb > ta).then_some(Self {
            a,
            b,
            ta,
            tb,
            clip,
            keys,
        })
    }

    /// O intervalo `[ta, tb)`, se o trecho ainda vale na vista dada (mesma regra para o motor e o painel).
    #[must_use]
    pub fn intervalo(
        &self,
        t_de: impl Fn(usize) -> Option<f64>,
        clip: usize,
        keys: bool,
        vista: bool,
    ) -> Option<(f64, f64)> {
        let mesmo = vista && clip == self.clip && keys == self.keys;
        (mesmo && t_de(self.a) == Some(self.ta) && t_de(self.b) == Some(self.tb))
            .then_some((self.ta, self.tb))
    }
}

fn t_do_doc(doc: &TimelineDoc) -> impl Fn(usize) -> Option<f64> + '_ {
    |i| doc.markers().get(i).map(|m| m.t.to_seconds())
}

/// O intent: escolher (`Some`) ou largar (`None`) o trecho. Só estado de sessão.
pub(crate) fn escolhe(state: &mut TimelineState, par: Option<(usize, usize)>) {
    state.trecho = par.and_then(|(a, b)| {
        Trecho::novo(
            t_do_doc(&state.doc),
            a,
            b,
            state.doc.active_index(),
            state.keys_mode,
        )
    });
}

/// **A porta única**, a cada quadro (de [`crate::arm_view_loop`]): arma o trecho no relógio do editor
/// (no início, a tocar, em Wrap) e, quando deixa de valer, devolve o loop do clipe.
pub(crate) fn arma(state: &mut TimelineState, editor: &mut Playhead) {
    let vista = vista_com_trecho(
        state.keys_mode,
        !state.doc.stack().is_empty(),
        state.container_open,
        state.containers_list,
    );
    let agora = state.trecho.and_then(|t| {
        t.intervalo(
            t_do_doc(&state.doc),
            state.doc.active_index(),
            state.keys_mode,
            vista,
        )
    });
    if agora.is_none() {
        state.trecho = None;
    }
    let antes = std::mem::replace(&mut state.trecho_armado, agora);
    match agora {
        Some((a, b)) => {
            if antes != agora {
                editor.set_loop_mode(ph2d_core::LoopMode::Wrap);
                editor.seek(a);
                editor.play();
            }
            editor.set_loop(a, b);
        }
        None if antes.is_some() => crate::sync_transport_loop(&state.doc, editor, state.keys_mode),
        None => {}
    }
}

#[cfg(test)]
#[path = "trecho_tests.rs"]
mod tests;

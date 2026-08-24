//! **A RESOLUÇÃO** — do retrato dos dispositivos às acções nomeadas.
//!
//! ⭐ **Não há um retrato próprio aqui.** O retrato é o [`InputState`] que esta crate tem desde a
//! M8: a shell já lhe entrega eventos, o `ph2d.input` do Luau já o lê, e ele já sabe a diferença
//! entre *premida agora* e *a borda deste quadro*. Uma segunda fotografia dos mesmos botões seria
//! duas memórias do mesmo facto, e elas divergiriam no primeiro quadro em que uma se perdesse.

use crate::action::{ActionId, Binding, InputAction};
use crate::map::InputMap;
use crate::state::InputState;

/// **A amostra resolvida de uma acção num tique.**
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Sample {
    /// `0..1` — já normalizada acima da `dead_zone`.
    pub strength: f32,
    /// O valor **cru** cruzou o `press_point`.
    pub pressed: bool,
}

/// **O estado das AÇÕES**: este tique e o anterior.
///
/// ⚠️ **Não confundir com o [`InputState`]**, que é o retrato dos **dispositivos**. Este guarda o
/// que as acções *valem* depois de o mapa ser aplicado, e guarda **um** tique atrás — é o que paga
/// [`Input::just_pressed`].
///
/// ⚠️ **Ele não tem memória do que está segurado.** Isso é do [`InputState`], que é quem sabe largar
/// tudo quando a janela perde o foco.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActionState {
    now: Vec<(ActionId, Sample)>,
    prev: Vec<(ActionId, Sample)>,
}

impl ActionState {
    /// Nenhuma acção resolvida ainda.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// **Resolve um tique**: o que era `now` passa a `prev`, e `now` é recalculado do retrato.
    ///
    /// ⚠️ **Percorre o MAPA, não os dispositivos.** Uma acção sem ligação nenhuma tem de aparecer
    /// com [`Sample::default`] — *declarada e por atribuir* não é *inexistente*, e é a diferença que
    /// deixa o painel oferecer o passo seguinte.
    pub fn tick(&mut self, map: &InputMap, dev: &InputState) {
        core::mem::swap(&mut self.now, &mut self.prev);
        self.now.clear();
        self.now
            .extend(map.actions().iter().map(|a| (a.id, resolve_action(a, dev))));
    }

    /// A amostra de `id` neste tique.
    #[must_use]
    pub fn sample(&self, id: ActionId) -> Sample {
        lookup(&self.now, id)
    }

    /// A amostra de `id` no tique anterior.
    #[must_use]
    pub fn previous(&self, id: ActionId) -> Sample {
        lookup(&self.prev, id)
    }
}

/// A amostra de `id` numa lista, ou o neutro se ela não estiver lá.
///
/// ⚠️ **O neutro, e não um `panic`**: um id de uma acção apagada tem de ler como *silêncio*, que é
/// exactamente o que uma fita gravada antes da remoção precisa que aconteça.
fn lookup(list: &[(ActionId, Sample)], id: ActionId) -> Sample {
    list.iter()
        .find(|(k, _)| *k == id)
        .map_or(Sample::default(), |(_, s)| *s)
}

/// O valor cru de uma ligação, em `0..1`.
fn raw(b: Binding, dev: &InputState) -> f32 {
    match b {
        Binding::Key(k) => f32::from(u8::from(dev.keyboard.held(k))),
        Binding::PadButton(p) => f32::from(u8::from(dev.gamepad.held(p))),
        Binding::PadAxis { axis, positive } => {
            let v = dev.gamepad.axis(axis);
            let v = if v.is_finite() { v } else { 0.0 };
            // ⚠️ Meia haste: a outra metade e' OUTRA accao. Um eixo empurrado para a esquerda NAO
            // da' forca a `move_right` -- da' ZERO, e e' o que faz a subtraccao do `Input::axis`
            // dizer a verdade em vez de somar duas leituras do mesmo movimento.
            if positive { v.max(0.0) } else { (-v).max(0.0) }
        }
    }
}

/// **A LEI dos dois números**, num sítio só.
///
/// - o cru é o **MÁXIMO** sobre as ligações (o Godot faz o mesmo): teclado *ou* comando *ou* a
///   segunda tecla — qualquer um deles serve, e o mais forte manda;
/// - **força**: abaixo da `dead_zone` é `0`; acima, é o que sobra **renormalizado** para `0..1`, de
///   modo a que o primeiro grau útil do analógico seja um `0` verdadeiro e o fundo do curso seja
///   `1` — sem isso, uma `dead_zone` de `0,2` cortaria 20% do curso **no topo** também;
/// - **premida**: o **cru** cruzou o `press_point`.
///
/// ⚠️ **As duas leem o CRU**, e é o que as torna comparáveis — e o que dá sentido ao invariante
/// `press_point >= dead_zone` que a porta da acção impõe.
fn resolve_action(a: &InputAction, dev: &InputState) -> Sample {
    let cru = a
        .bindings
        .iter()
        .map(|b| raw(*b, dev))
        .fold(0.0_f32, f32::max);
    let strength = if cru < a.dead_zone {
        0.0
    } else {
        ((cru - a.dead_zone) / (1.0 - a.dead_zone)).clamp(0.0, 1.0)
    };
    Sample {
        strength,
        pressed: cru >= a.press_point,
    }
}

/// **A VISTA DE LEITURA** — o mapa e o estado juntos, que é o que faz a leitura caber numa linha.
///
/// ⚠️ Ela existe porque a alternativa era passar o mapa a cada chamada
/// (`state.strength(&map, "jump")`), e a lição medida do sistema de input da Unity é que **o custo
/// de montagem é o que mata a adopção**, não a falta de poder. O caminho comum tem de ser **uma
/// linha**.
#[derive(Copy, Clone, Debug)]
pub struct Input<'a> {
    map: &'a InputMap,
    state: &'a ActionState,
}

impl<'a> Input<'a> {
    /// Junta o mapa e o estado numa vista.
    #[must_use]
    pub fn new(map: &'a InputMap, state: &'a ActionState) -> Self {
        Self { map, state }
    }

    /// **Está segurada AGORA?**
    #[must_use]
    pub fn pressed(&self, name: &str) -> bool {
        self.id(name).is_some_and(|i| self.state.sample(i).pressed)
    }

    /// **A BORDA deste tique** — passou de solta a premida.
    #[must_use]
    pub fn just_pressed(&self, name: &str) -> bool {
        self.id(name)
            .is_some_and(|i| self.state.sample(i).pressed && !self.state.previous(i).pressed)
    }

    /// A borda contrária — passou de premida a solta.
    #[must_use]
    pub fn just_released(&self, name: &str) -> bool {
        self.id(name)
            .is_some_and(|i| !self.state.sample(i).pressed && self.state.previous(i).pressed)
    }

    /// **A força, `0..1`.** Uma tecla dá `0` ou `1`; um analógico dá o intermédio.
    #[must_use]
    pub fn strength(&self, name: &str) -> f32 {
        self.id(name).map_or(0.0, |i| self.state.sample(i).strength)
    }

    /// **O eixo, `-1..1`** — a subtracção do Godot.
    ///
    /// ⭐ **As duas seguradas dão ZERO**, que é a resposta que o jogador espera e a que um
    /// acumulador `+1`/`−1` não daria. É a mesma lei que o `PlayerKeys::drive` da shell implementa
    /// à mão hoje, e que passa a sair de graça.
    #[must_use]
    pub fn axis(&self, negative: &str, positive: &str) -> f32 {
        self.strength(positive) - self.strength(negative)
    }

    /// O id de uma acção pelo nome — o caminho quente, para quem lê todos os tiques.
    #[must_use]
    pub fn id(&self, name: &str) -> Option<ActionId> {
        self.map.id(name)
    }

    /// A amostra crua de um id já resolvido.
    #[must_use]
    pub fn sample(&self, id: ActionId) -> Sample {
        self.state.sample(id)
    }
}

#[cfg(test)]
#[path = "resolve_tests.rs"]
mod tests;

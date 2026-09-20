//! ⭐⭐⭐ **A VIGIA DE UM CONTADOR** — o elo que faltava para um NÚMERO fazer acontecer alguma coisa.
//!
//! # A medição que abriu a wave (2026-09-17)
//!
//! Fechado o TOP-20, a §5.0 do `CLAUDE.md` mandou medir a composição antes de construir. O
//! resultado foi **um** buraco, e ele é exacto: o [`crate::Counter`] é **escrito**
//! (`SignalVerb::AddToCounter`) e é **mostrado** (`LabelSource::Counter`), e **ninguém reage a
//! ele**. Nem a válvula de escape do TOP-20 #16: a superfície do Luau tem `emit`, `spawn`,
//! `despawn`, `get`/`set`, `state_*`, `find_by_name` e `input`, e **nenhuma porta para contadores**.
//!
//! ⇒ o artista conseguia contar e VER a conta, e não conseguia fazer acontecer nada a um número.
//! Três vidas chegavam a zero e o jogo não sabia.
//!
//! # ⭐⭐⭐ A lei é a ARESTA, e essa decisão carrega a wave
//!
//! Uma comparação avaliada por quadro é um **NÍVEL**, e um nível dispara **por quadro**: com as
//! vidas a zero, um verbo *Hide* correria 60 vezes por segundo e encheria o orçamento de
//! profundidade que o TOP-20 #15 mediu. As referências sabem-no e **separam as duas coisas** — o
//! Construct 3 tem *«Compare variable»* **e** a condição distinta *«Trigger once while true»*; o
//! Unreal escreve *comparação + Do Once*.
//!
//! ⇒ aqui a aresta é a **lei**, não um knob: emite-se na **subida** (`tem && !held`), e o
//! [`CounterWatchRow::once`] é um segundo knob **por cima** dela (*só da primeira vez*).
//!
//! ⚠️ **Esta casa já pagou esta distinção na wave anterior:** o #15 portou um oráculo cuja entrada
//! é um nível e, escrita à letra, uma porta ia a `Fechada` **e logo a `A abrir`** com um toque.
//!
//! # ⚠️ O que «nascer» é aqui, e a consequência visível
//!
//! `held = false`. ⇒ **uma condição já satisfeita no tique 0 dispara uma vez no tique 0** — que é
//! o que o Construct faz, e o que impede o caso mudo *«a porta nasce aberta e ninguém o diz»*.
//! Rebobinar repõe-o pela porta do [`crate::rewind_runtime`], que é a **sétima** espécie a passar
//! por ela.
//!
//! # ⛔ O que esta wave NÃO constrói, e porquê
//!
//! O suplente **#24 `Health`** do levantamento. Com a vigia, ele é **composição**:
//! `Counter{name:"vidas", start:3}` + `SignalActions[golpe → AddToCounter(-1)]` +
//! `CounterWatch[vidas AtMost 0 → "morri"]`, com o placar a ser o `UiLabel` que já existe.
//! Construí-lo como componente seria uma segunda resposta a *«quanto vale este número?»*.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// Quantas vigias uma entidade pode ter.
///
/// ⚠️ **O número sai do PAINEL** — a lei do `ANIM_TAGS_MAX`: *um modelo que aceita o que o painel
/// não mostra produz estado inalcançável*. A secção usa o molde do `Timers` (lista + selector),
/// logo o tecto é o dele, e há gate na shell a atar os dois.
pub const WATCHES_MAX: usize = 16;

/// **Como o valor se compara com o limiar.**
///
/// ⚠️ **`#[repr(u8)]` e APPEND-ONLY:** ele viaja no documento pelo postcard, que é **posicional** —
/// uma variante no meio reescreveria o sentido de todas as vigias já gravadas, em silêncio. É a
/// mesma nota que o `SignalVerb` carrega.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum Compare {
    /// `valor <= limiar` — *«as vidas chegaram a zero»*. O default, porque é o caso que abriu a wave.
    #[default]
    AtMost,
    /// `valor >= limiar` — *«dez moedas»*.
    AtLeast,
    /// `valor == limiar` — o caso exacto.
    Exactly,
}

impl Compare {
    /// A pergunta, num sítio só — os gates e a lei leem a MESMA função.
    #[must_use]
    pub fn holds(self, valor: i64, limiar: i64) -> bool {
        match self {
            Self::AtMost => valor <= limiar,
            Self::AtLeast => valor >= limiar,
            Self::Exactly => valor == limiar,
        }
    }
}

/// **Uma regra:** *quando o contador `counter` `compare` `value`, diz `signal`.*
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterWatchRow {
    /// **Qual contador — pelo NOME.** ⚠️ A lei da casa: o nome viaja, nunca o índice nem os bits
    /// (apagar um objecto renumera; o undo respawna com bits novos).
    pub counter: String,
    /// A comparação.
    pub compare: Compare,
    /// O limiar.
    pub value: i64,
    /// **O que emitir na travessia.** Vazio = a regra segue o valor e fica **calada** — uma
    /// configuração a meio, não um erro (a lei do `inert` da tabela de acções).
    pub signal: String,
    /// **Só da primeira vez.** Desligado, ela fala a **cada** travessia.
    pub once: bool,
    /// ⭐⭐⭐ **QUAL contador com esse nome** — a cena inteira, ou só o desta entidade.
    ///
    /// É o que faz *«uma vida POR inimigo»* existir: dez inimigos com um `Counter{"vida"}` cada e
    /// a MESMA vigia, e cada um morre com a própria vida a zero. Com [`CounterScope::World`] (o de
    /// fábrica, e o comportamento de sempre) os dez lêem a soma dos dez e **morrem todos juntos**.
    pub scope: CounterScope,
}

/// **Onde a vigia procura o contador que ela julga.**
///
/// ⚠️ **`#[repr(u8)]` e APPEND-ONLY**, como a [`Compare`] ao lado: ele viaja no documento pelo
/// postcard, que é posicional.
///
/// ⛔ **Não há `Tagged` ao lado destes dois, e a ausência é medida:** o [`crate::tags`] resolve uma
/// tag para uma LISTA de entidades, e *somar a vida de um grupo* é a pergunta que o `World` já
/// responde quando o nome é só daquele grupo. Um modo sem consumidor é um controlo morto com cara
/// de feature — a mesma recusa que o `SignalTarget::Speaker` levou.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CounterScope {
    /// **A soma de toda a cena** — o placar. O de fábrica, e o de todo ficheiro já gravado.
    #[default]
    World,
    /// **Só o contador que vive nesta entidade** — a vida de UM inimigo.
    Own,
}

impl CounterScope {
    /// O âmbito que a porta [`crate::counter::soma`] lê, dado QUEM é o dono desta vigia.
    ///
    /// ⚠️ Esta é a única ponte entre o que o ficheiro guarda (um modo) e o que a porta pede (um
    /// modo **com a entidade dentro**) — e é por ela ser uma função com gate que o dia em que
    /// houver um terceiro modo não tem onde divergir.
    #[must_use]
    pub const fn ambito(self, dono: crate::Entity) -> crate::counter::Ambito {
        match self {
            Self::World => crate::counter::Ambito::Mundo,
            Self::Own => crate::counter::Ambito::Objecto(dono),
        }
    }
}

/// **As vigias de uma entidade** — o componente registado. ⚠️ CONFIG, como o `Timers`.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CounterWatch(pub Vec<CounterWatchRow>);

impl SimComponent for CounterWatch {}

/// O estado vivo de **uma** regra.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WatchState {
    /// A condição estava satisfeita da última vez que se olhou. **É isto que faz a aresta.**
    pub held: bool,
    /// Ela já falou alguma vez — lido pelo [`CounterWatchRow::once`].
    pub fired: bool,
}

/// **O estado vivo das vigias** — ⛔ **não registado**, e a ausência é a lei.
///
/// Sem ela, cada quadro em que uma condição continua satisfeita escreveria um componente registado
/// e **viraria um passo de `Ctrl+Z`**. É o precedente exacto do `TimerRuntime`, do `CounterRuntime`,
/// do `FactoryRuntime` e do `StateMachineRuntime` — e, como eles, **não deriva `Serialize`**, logo
/// registá-lo nem compila.
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct CounterWatchRuntime(pub Vec<WatchState>);

/// **O que um slot acabado de nascer recebe** — a porta, com dois leitores ([`reconcile`] e o
/// [`crate::rewind_runtime`]).
///
/// ⚠️ `held = false` **de propósito**: ver o cabeçalho. Uma condição já verdadeira no tique 0 é uma
/// travessia, e não um facto que já estava lá.
#[must_use]
pub const fn born() -> WatchState {
    WatchState {
        held: false,
        fired: false,
    }
}

/// Cria/apaga slots para casar com a config. `false` = nada mudou.
///
/// ⚠️ **Encolher também é nascimento ao contrário** — truncar deita fora o estado dos slots que já
/// não existem, senão uma regra removida e reposta voltaria já disparada (a lei que o `Timers`
/// escreveu).
pub fn reconcile(cfg: &CounterWatch, rt: &mut CounterWatchRuntime) -> bool {
    if rt.0.len() == cfg.0.len() {
        return false;
    }
    rt.0.resize(cfg.0.len(), born());
    true
}

/// ⭐⭐⭐ **A LEI.** Devolve `true` sse esta regra fala **agora**.
///
/// O `soma` é o que a porta [`crate::counter::soma`] devolveu — `None` = **não existe contador com
/// aquele nome**.
///
/// ⚠️⚠️ **`None` conta como NÃO satisfeita**, e essa é a linha que impede o defeito mudo: sem ela,
/// escrever `vidaas` num campo faria uma regra `AtMost 0` anunciar *«morreste»* no arranque, porque
/// um contador ausente leria zero. ⇒ **uma vigia sobre um contador que não existe nunca dispara.**
///
/// ⚠️ **O `held` é escrito mesmo quando a regra está calada** (sem nome de sinal, ou já disparada
/// com `once`): ele descreve o MUNDO, não o que esta regra fez. Sem isso, escrever o nome do sinal
/// mais tarde faria a regra disparar sobre uma condição que já estava satisfeita há minutos.
pub fn advance(row: &CounterWatchRow, state: &mut WatchState, soma: Option<i64>) -> bool {
    let tem = soma.is_some_and(|v| row.compare.holds(v, row.value));
    let sobe = tem && !state.held;
    state.held = tem;
    if !sobe || row.signal.trim().is_empty() || (row.once && state.fired) {
        return false;
    }
    state.fired = true;
    true
}

#[cfg(test)]
#[path = "counter_watch_tests.rs"]
mod tests;

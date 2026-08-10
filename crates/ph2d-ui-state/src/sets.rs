//! **ONDE OS ESTADOS MORAM** — a tabela que viaja no documento.

use crate::binding::SignalBinding;
use crate::pose::UiState;
use crate::role::StateRole;
use crate::spring::Spring;
use ph2d_anim::{Easing, EasingFamily, EasingMode};
use ph2d_vec_scene::VecPathId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Quanto tempo uma transição de UI leva, por omissão.
///
/// ⚠️ **Este número não é de recurso, e a diferença importa** (§0): a [`crate::Machine`] aceita
/// qualquer duração, e nada aqui fica mais barato por ele ser pequeno. O que ele descreve é
/// PERCEPÇÃO — a faixa em que uma troca de estado ainda lê como *resposta ao gesto* e não como
/// uma animação com vida própria. Os 150 ms são o meio da faixa que a indústria converge (o
/// *Smart Animate* do Figma abre em 300, o blend do Rive em 125, o Material chama 200-300 de
/// "medium"), e um botão responde mais rápido que um card que abre.
///
/// ⚠️ **É o `ph2d_tokens::Duration::Fast`**, cujo doc diz *"button press, icon swap"* — a
/// pergunta já tinha dono no design system. Ele não é lido daqui porque uma crate-folha de dados
/// não depende do sistema de tokens; o número é o mesmo, e há gate a compará-los.
pub const DEFAULT_DURATION_S: f64 = 0.15;

/// O teto que o SLIDER oferece.
///
/// ⚠️ Do mesmo tipo que o de cima, e por isso está escrito aqui em vez de num `clamp` escondido:
/// a máquina anda dois segundos ou dez sem se importar. O que se esgota aos 2 s é a ATENÇÃO —
/// além disso a transição deixa de ser a resposta a um gesto e passa a ser algo que o artista
/// autora numa timeline, que é outra ferramenta deste app.
pub const MAX_DURATION_S: f64 = 2.0;

/// A curva por omissão: desacelerar até parar.
///
/// ⚠️ `Out` e não `InOut`: o gesto do artista **já aconteceu** quando a transição começa, então
/// uma entrada lenta é atraso puro — a mesma razão pela qual toda biblioteca de UI abre em
/// *ease-out* e reserva o *ease-in-out* para o que se move sozinho.
pub const DEFAULT_EASING: Easing = Easing {
    family: EasingFamily::Cubic,
    mode: EasingMode::Out,
};

/// Os estados de UM hospedeiro, mais como ele transita.
///
/// ⚠️ **O tempo é do HOSPEDEIRO e não do estado**, e a assimetria é deliberada: *"ir para hover"*
/// e *"voltar de hover"* usam o mesmo número, porque um par de durações diferentes é a primeira
/// coisa que um artista afina e a última que ele consegue explicar. Quem quiser assimetria tem a
/// timeline.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HostStates {
    /// Ordenados por [`StateRole`] — no máximo um de cada.
    states: Vec<UiState>,
    pub duration_s: f64,
    pub easing: Easing,
    /// **A MOLA, quando o artista a escolhe.** `None` = o par duração+curva, e é o caminho que
    /// já shipava — byte-idêntico.
    ///
    /// ⚠️ **`Option` e não um par de números com um "modo" ao lado:** *ter mola* e *que mola* são
    /// a mesma decisão, e um bool separado dos parâmetros seria um estado a mais para manter de
    /// acordo. É o mesmo desenho do `wrap_width` do texto.
    ///
    /// ⚠️ **E ela EXCLUI a duração e a curva**, que continuam no struct porque o artista volta a
    /// elas ao desmarcar — desligar a mola não pode apagar o que ele já tinha afinado.
    pub spring: Option<Spring>,
    /// **A que sinais este hospedeiro responde** ([`SignalBinding`]).
    ///
    /// ⚠️ **Aqui e não numa tabela própria**, e a razão é ciclo de vida: [`StateSets::retain_hosts`]
    /// já corre por frame, então uma forma apagada leva as ligações dela sem uma linha a mais. O
    /// doc do módulo [`crate::binding`] mede as duas razões.
    ///
    /// ⚠️ **Sem teto, e é deliberado:** não há recurso — cada linha são uma `String` e um enum. O
    /// número que existe (`MAX_SIGNAL_BINDINGS`, no painel) é o tamanho do **pool de ids que o
    /// `populate` regista de antemão**, e é um fato da UI, não do documento: um arquivo com mais
    /// ligações do que ele **funciona** (o consumidor lê todas), só não as mostra todas.
    ///
    /// ⚠️ **Sem `#[serde(default)]`, de propósito:** o postcard é POSICIONAL e não sinaliza
    /// ausência — um atributo aqui prometeria uma compatibilidade que o formato não tem. Quem
    /// separa as duas formas de arquivo é o `PROJECT_SCHEMA`, e ele sobe com este campo.
    pub on_signal: Vec<SignalBinding>,
}

impl Default for HostStates {
    fn default() -> Self {
        Self {
            states: Vec::new(),
            duration_s: DEFAULT_DURATION_S,
            easing: DEFAULT_EASING,
            spring: None,
            on_signal: Vec::new(),
        }
    }
}

impl HostStates {
    #[must_use]
    pub fn states(&self) -> &[UiState] {
        &self.states
    }
}

/// Os estados de cada HOSPEDEIRO, indexados pelo [`VecPathId`] dele.
///
/// ⚠️ **A chave é o `VecPathId`, nunca a entidade.** Bits de entidade são id de ALOCAÇÃO e o undo
/// global respawna tudo com bits novos — uma tabela chaveada por eles perderia os estados no
/// primeiro Ctrl+Z. É a mesma lição que a timeline pagou nas bindings e a física nos joints.
///
/// ⚠️ **`BTreeMap` e não `HashMap`:** esta tabela é serializada, e a ordem de iteração de um
/// `HashMap` faria dois saves do mesmo documento diferirem — e, pior, faria o **diff do undo**
/// registrar um passo espúrio sobre um estado que ninguém tocou.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StateSets {
    by_host: BTreeMap<VecPathId, HostStates>,
}

impl StateSets {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_host.is_empty()
    }

    /// Os estados de `host`, ou uma fatia vazia. Ordenados por papel.
    #[must_use]
    pub fn get(&self, host: VecPathId) -> &[UiState] {
        self.by_host.get(&host).map_or(&[], HostStates::states)
    }

    /// O estado de `host` no papel `role`, se ele o autora.
    #[must_use]
    pub fn role(&self, host: VecPathId, role: StateRole) -> Option<&UiState> {
        self.get(host).iter().find(|s| s.role == role)
    }

    /// Quanto tempo e por que curva `host` transita — os defaults se ele nunca os autorou.
    #[must_use]
    pub fn timing(&self, host: VecPathId) -> (f64, Easing) {
        self.by_host
            .get(&host)
            .map_or((DEFAULT_DURATION_S, DEFAULT_EASING), |h| {
                (h.duration_s, h.easing)
            })
    }

    /// **A mola deste hospedeiro, se ele a escolheu.**
    ///
    /// ⚠️ Porta ÚNICA, ao lado do [`Self::timing`]: quem anima pergunta a UMA das duas, e a
    /// resposta desta decide qual. Duas cópias da pergunta *"este hospedeiro usa mola?"* seriam
    /// duas oportunidades de o painel pintar uma coisa e o motor correr outra.
    #[must_use]
    pub fn spring(&self, host: VecPathId) -> Option<Spring> {
        self.by_host.get(&host).and_then(|h| h.spring)
    }

    /// Quem tem estados. Ordem determinista (é um `BTreeMap`).
    pub fn hosts(&self) -> impl Iterator<Item = VecPathId> + '_ {
        self.by_host.keys().copied()
    }

    /// **Grava a pose de um papel** — substitui a que lá estava, ou acrescenta.
    ///
    /// ⚠️ A lista fica **ordenada por papel**, e não por ordem de gravação: a ordem em que o
    /// artista gravou é acidente, e uma lista que se reordenasse debaixo do dedo dele seria a
    /// mesma falha que o `RootOrder` curou na hierarquia.
    pub fn set(&mut self, host: VecPathId, state: UiState) {
        let h = self.by_host.entry(host).or_default();
        match h.states.iter_mut().find(|s| s.role == state.role) {
            Some(slot) => *slot = state,
            None => {
                h.states.push(state);
                h.states.sort_by_key(|s| s.role);
            }
        }
    }

    /// **Apaga o estado de um papel.** O hospedeiro sem estado nenhum **sai da tabela** — um
    /// documento não carrega uma entrada vazia, e é isso que mantém o `is_empty` honesto e o save
    /// enxuto.
    ///
    /// ⚠️ **A entrada só sai quando o TEMPO também é o de fábrica.** Um hospedeiro que ficou sem
    /// estados mas cuja duração o artista afinou ainda carrega uma decisão dele, e despejá-la
    /// junto seria perder trabalho em silêncio.
    pub fn clear(&mut self, host: VecPathId, role: StateRole) -> bool {
        let Some(h) = self.by_host.get_mut(&host) else {
            return false;
        };
        let before = h.states.len();
        h.states.retain(|s| s.role != role);
        if h.states.len() == before {
            return false;
        }
        if h.states.is_empty() && *h == HostStates::default() {
            self.by_host.remove(&host);
        }
        true
    }

    pub fn set_duration(&mut self, host: VecPathId, seconds: f64) {
        self.by_host.entry(host).or_default().duration_s = seconds.clamp(0.0, MAX_DURATION_S);
    }

    pub fn set_easing(&mut self, host: VecPathId, easing: Easing) {
        self.by_host.entry(host).or_default().easing = easing;
    }

    /// **Liga ou desliga a mola** deste hospedeiro, e afina-a.
    ///
    /// ⚠️ Desligar **não apaga** a duração nem a curva: o artista volta a elas com o mesmo clique,
    /// e um desmarcar que jogasse fora o que ele afinou seria trabalho destruído por um gesto que
    /// não promete nada disso.
    pub fn set_spring(&mut self, host: VecPathId, spring: Option<Spring>) {
        self.by_host.entry(host).or_default().spring = spring.map(Spring::clamped);
    }

    /// **A que sinais `host` responde**, ou uma fatia vazia.
    #[must_use]
    pub fn bindings(&self, host: VecPathId) -> &[SignalBinding] {
        self.by_host.get(&host).map_or(&[], |h| &h.on_signal[..])
    }

    /// **Acrescenta uma ligação vazia** e devolve o índice dela.
    pub fn push_binding(&mut self, host: VecPathId) -> usize {
        let h = self.by_host.entry(host).or_default();
        h.on_signal.push(SignalBinding::empty());
        h.on_signal.len() - 1
    }

    /// Renomeia a ligação `index`. Índice fora da lista é ignorado — o painel publica um snapshot
    /// e o clique chega um frame depois, então a lista pode ter encolhido no meio.
    pub fn set_binding_name(&mut self, host: VecPathId, index: usize, name: String) {
        if let Some(b) = self
            .by_host
            .get_mut(&host)
            .and_then(|h| h.on_signal.get_mut(index))
        {
            b.name = name;
        }
    }

    /// Re-aponta a ligação `index` para outro papel.
    pub fn set_binding_role(&mut self, host: VecPathId, index: usize, role: StateRole) {
        if let Some(b) = self
            .by_host
            .get_mut(&host)
            .and_then(|h| h.on_signal.get_mut(index))
        {
            b.role = role;
        }
    }

    /// **Apaga a ligação `index`.** O hospedeiro que fica sem nada — nem estados, nem ligações,
    /// nem tempo afinado — sai da tabela, pela MESMA regra do [`Self::clear`].
    pub fn remove_binding(&mut self, host: VecPathId, index: usize) {
        let Some(h) = self.by_host.get_mut(&host) else {
            return;
        };
        if index >= h.on_signal.len() {
            return;
        }
        h.on_signal.remove(index);
        if h.states.is_empty() && *h == HostStates::default() {
            self.by_host.remove(&host);
        }
    }

    /// **Quem responde a `signal`, e para que papel** — a porta que o consumidor de sinais usa.
    ///
    /// ⚠️ **A busca é por NOME e ignora quem gritou**, que é o contrato do ADR-0143: um contato de
    /// física e um botão autorado com o mesmo nome movem a mesma cena, e é isso que torna a
    /// ligação reusável em vez de um campo escondido dentro do botão.
    ///
    /// ⚠️ **Um hospedeiro pode aparecer DUAS vezes** se o artista o ligou duas vezes ao mesmo
    /// nome — e a porta não deduplica de propósito: escolher uma das duas seria inventar uma
    /// precedência que ele não autorou. Quem consome pede um papel por vez, e o último ganha; a
    /// resposta certa para *"liguei o mesmo nome a dois papéis"* é apagar uma das linhas, que ele
    /// vê na tela.
    pub fn targets<'a>(
        &'a self,
        signal: &'a str,
    ) -> impl Iterator<Item = (VecPathId, StateRole)> + 'a {
        self.by_host.iter().flat_map(move |(&host, h)| {
            h.on_signal
                .iter()
                .filter(move |b| b.matches(signal))
                .map(move |b| (host, b.role))
        })
    }

    /// **Esquece um hospedeiro que já não existe.** Chamado quando uma forma é apagada: sem isto a
    /// tabela acumularia estados de objetos que ninguém vê, e eles viajariam no arquivo para
    /// sempre.
    pub fn retain_hosts(&mut self, alive: impl Fn(VecPathId) -> bool) {
        self.by_host.retain(|id, _| alive(*id));
    }
}

//! Os nomes de sinal autorados que moram FORA do `World`, nos documentos da composição: o grafo do
//! Motion (`pulse.signal` emite, `value.input` ouve — o sinal do runtime dispara a entrada do mesmo
//! nome), os marcadores da timeline (emitem ao cruzar) e as ligações «On signal» dos estados de UI
//! (ouvem). A tabela [`FORA`] é a lista, como a `CAMPOS` do mundo.

use ph2d_nodegraph::graph::Graph;
use ph2d_signal_ends::Role;
use ph2d_timeline::TimelineDoc;
use ph2d_ui_state::StateSets;

/// O tipo do nó do Motion que grita um nome (`ph2d-node-pulse-signal`).
pub const PULSE_SIGNAL: &str = "pulse.signal";
/// O tipo do nó do Motion que lê a entrada publicada (`ph2d-node-value-input`).
pub const VALUE_INPUT: &str = "value.input";
/// O text param com o nome, nos dois nós.
pub const NAME_KEY: &str = "name";

/// Os documentos que a composição passa ao censo; `None` = ausente neste quadro.
#[derive(Clone, Copy, Default)]
pub struct Fora<'a> {
    pub motion: Option<&'a Graph>,
    pub timeline: Option<&'a TimelineDoc>,
    pub ui_states: Option<&'a StateSets>,
    /// Os nomes que os botões e as listas do painel do artista podem gritar (`Signal::from_control`).
    pub painel: Option<&'a [String]>,
}

impl<'a> Fora<'a> {
    /// Os três documentos de um quadro do editor.
    #[must_use]
    pub fn new(motion: &'a Graph, timeline: &'a TimelineDoc, ui_states: &'a StateSets) -> Self {
        Self {
            motion: Some(motion),
            timeline: Some(timeline),
            ui_states: Some(ui_states),
            painel: None,
        }
    }

    /// Com os nomes do painel do artista (a shell tira-os das rows que o painel pinta).
    #[must_use]
    pub fn com_painel(self, nomes: &'a [String]) -> Self {
        Self {
            painel: Some(nomes),
            ..self
        }
    }

    /// [`crate::when_changed`] sobre o mundo e estes documentos.
    pub fn when_changed(&self, world: &ph2d_ecs::World, publish: impl FnOnce(crate::SignalCensus)) {
        crate::when_changed(world, self, publish);
    }
}

/// Uma linha da tabela de fora.
pub struct Origem {
    pub doc: &'static str,
    pub field: &'static str,
    pub role: Role,
    pub read: LeitorFora,
}

/// Como uma linha de [`FORA`] lê os nomes do documento dela.
pub type LeitorFora = fn(&Fora<'_>, &mut dyn FnMut(&str));

/// Os nós `ty` do grafo e o nome de cada um.
fn nomes_de_nos(g: &Graph, ty: &str, put: &mut dyn FnMut(&str)) {
    let texto = g.node_text_params();
    for n in g.nodes().iter().filter(|n| n.type_name == ty) {
        if let Some(nome) = texto.get(&n.id).and_then(|m| m.get(NAME_KEY)) {
            put(nome);
        }
    }
}

/// ⭐ **A tabela** das origens de fora do mundo.
pub static FORA: &[Origem] = &[
    // O `collect_signals` publica o nome cru (salta o só-espaços).
    Origem {
        doc: "MotionGraph",
        field: "pulse.signal.name",
        role: Role::Emit,
        read: |f, put| {
            if let Some(g) = f.motion {
                nomes_de_nos(g, PULSE_SIGNAL, &mut |n| {
                    if !n.trim().is_empty() {
                        put(n);
                    }
                });
            }
        },
    },
    // A entrada lê `input_of(nome)`, que apara o nome.
    Origem {
        doc: "MotionGraph",
        field: "value.input.name",
        role: Role::Listen,
        read: |f, put| {
            if let Some(g) = f.motion {
                nomes_de_nos(g, VALUE_INPUT, &mut |n| put(n.trim()));
            }
        },
    },
    Origem {
        doc: "TimelineDoc",
        field: "markers.signal",
        role: Role::Emit,
        read: |f, put| {
            if let Some(t) = f.timeline {
                t.markers()
                    .iter()
                    .filter_map(|m| m.signal.as_deref())
                    .for_each(put);
            }
        },
    },
    // Os botões (a chave) e as listas (`chave/opção`) do painel do artista gritam o nome cru.
    Origem {
        doc: "AuthoredPanel",
        field: "rows.signal",
        role: Role::Emit,
        read: |f, put| f.painel.into_iter().flatten().for_each(|n| put(n)),
    },
    Origem {
        doc: "StateSets",
        field: "bindings.name",
        role: Role::Listen,
        read: |f, put| {
            if let Some(s) = f.ui_states {
                for h in s.hosts() {
                    s.bindings(h).iter().for_each(|b| put(&b.name));
                }
            }
        },
    },
];

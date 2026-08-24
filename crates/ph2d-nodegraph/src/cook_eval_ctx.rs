//! **O que um NÓ vê** — o [`EvalCtx`], a caixa preta FBP do ADR-0031.
//!
//! Ele é o outro lado da fronteira que o `cook.rs` guarda: aquele arquivo é o
//! MOTOR (a recursão, o memo, o fingerprint, os escopos de tempo), este é a
//! superfície que um nó de facto toca — as suas entradas, o seu relógio, os
//! seus params, a sua identidade. Um nó nunca vê o grafo, e é aqui que essa
//! promessa é feita em código em vez de em prosa.
//!
//! ⚠️ Os campos são `pub(super)` e não públicos: quem CONSTRÓI um `EvalCtx` é o
//! motor, e mais ninguém — um construtor público deixaria um chamador de fora
//! montar um contexto que não corresponde a nó nenhum do grafo.

use super::{CookValue, NodeManifest, Stream};
use std::any::Any;
use std::collections::BTreeMap;
use std::sync::Arc;

/// Per-eval context handed to a node. A node sees **only** this — its typed
/// inputs, the playhead, and its own resolved parameters — never the graph.
/// FBP black box (ADR-0031).
pub struct EvalCtx<'a> {
    pub(super) inputs: &'a [CookValue],
    /// **O LEQUE** ([`crate::cook::TimeFans`]) — a porta 0 deste nó cozida em N
    /// instantes, em vez de uma vez. Vazio para todo nó que não pediu um, que é
    /// todo nó menos um.
    pub(super) fan: &'a [CookValue],
    /// **Os params DIRIGIDOS em cada instante do leque** — a mesma pergunta que o
    /// [`Self::fan`], para um nó cuja história vive num param e não numa porta.
    pub(super) fan_driven: &'a [BTreeMap<&'a str, f32>],
    pub(super) playhead: f64,
    pub(super) manifest: &'static NodeManifest,
    pub(super) overrides: Option<&'a BTreeMap<String, f32>>,
    pub(super) text_overrides: Option<&'a BTreeMap<String, String>>,
    /// **Params driven by a wire** (doc 58) — already cooked, already reduced to one
    /// number, resolved by [`Cook::cook_node`] in the same recursion that resolved the
    /// input ports. Read by [`EvalCtx::param`] BEFORE the override and the default, which
    /// is what makes all 86 node types drivable without touching one of them: they all
    /// read their params through that one funnel.
    pub(super) driven: BTreeMap<&'a str, f32>,
    /// **What the APP published** (doc 65) — a drawn curve, and anything else the graph cannot
    /// reach on its own. Read by [`EvalCtx::external`].
    pub(super) externals: &'a crate::external::All,
    /// The names this node actually READ this eval. The cook keeps them beside the memo, because
    /// the reuse decision is made BEFORE the eval and can only know what the node read LAST time
    /// (`external.rs`: the chicken and the egg).
    pub(super) read_externals: Vec<String>,
    /// Did THIS node emit anything on the previous tick? (`prev_outputs` holds it.)
    pub(super) started: bool,
    /// **A identidade DESTE nó dentro do grafo** — o `NodeId` cru.
    ///
    /// Um nó estocástico precisa de um número que difira do do vizinho para dois
    /// deles com a mesma semente pararem de ser GÊMEOS, e o `NodeId` é
    /// exactamente isso: *"stable instance id within a graph, assigned
    /// monotonically, never reused, survives serialization"* (`graph.rs`).
    ///
    /// ⚠️ **Isto NÃO abre o grafo ao nó** (ADR-0031, a caixa preta FBP): ele
    /// continua sem ver arestas, vizinhos ou tipos — vê o próprio nome, do mesmo
    /// jeito que já vê o próprio manifesto e os próprios params.
    pub(super) node_key: u32,
    /// Seconds since the previous tick, on the ROOT clock (0 on the first tick).
    pub(super) dt: f64,
    pub(super) outputs: Vec<CookValue>,
}

impl<'a> EvalCtx<'a> {
    /// **What the app published under this name** (doc 65) — a drawn curve, most of the time.
    /// Empty if nobody published one, exactly like an unconnected input: a node asking for a shape
    /// that is not there emits nothing, it does not fail.
    ///
    /// Reading one is what puts it in this node's fingerprint (the cook keeps the names beside the
    /// memo), so editing the curve recomputes the node — which nothing else here would notice.
    pub fn external(&mut self, name: &str) -> &'a Stream {
        static EMPTY: Stream = Stream::empty();
        self.read_externals.push(name.to_string());
        self.externals.get(name).map_or(&EMPTY, |e| &e.value)
    }
    /// The cooked **instance stream** on input `port` (empty if unconnected, or
    /// if the upstream emitted a non-stream value; for a `pre` port, the
    /// previous tick's value). The value's domain is guaranteed by `PortType`
    /// checking at connect time, so a motion node reads its columns directly.
    pub fn input(&self, port: usize) -> &Stream {
        self.inputs[port].as_stream()
    }

    /// The cooked **opaque value** on input `port` (e.g. a geometry
    /// `VectorNetwork`), type-erased; the domain layer downcasts it. `None` if
    /// the input is unconnected or carries an instance stream rather than an
    /// opaque value (ADR-0058-amendment-1).
    pub fn input_any(&self, port: usize) -> Option<&(dyn Any + Send + Sync)> {
        self.inputs.get(port).and_then(CookValue::as_any)
    }

    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    /// **Quantas fatias o leque trouxe** ([`crate::cook::TimeFans`]) — `0` quando
    /// este nó não pediu um.
    ///
    /// ⚠️ **`0` não é «a porta está desligada»**: é *"ninguém montou um leque
    /// para mim"*. Um nó com modo de re-cozedura ligado e leque vazio tem de
    /// cair no comportamento sem leque, nunca emitir vazio — o leque é montado
    /// pela camada que conhece os tipos, e uma janela que ainda não o montou
    /// não é uma cena sem conteúdo.
    pub fn fan_len(&self) -> usize {
        self.fan.len()
    }

    /// A fatia `k` do leque, do instante mais próximo do agora para o mais
    /// distante — a ordem em que a camada de domínio montou os mapas. Vazia fora
    /// de alcance, como uma porta desligada.
    /// O valor que o param DIRIGIDO `name` tinha na fatia `k` do leque.
    ///
    /// `None` quando o param não é dirigido por fio nenhum naquele instante — e
    /// aí o valor é o mesmo de sempre ([`Self::param`]), porque um param estático
    /// não tem história.
    pub fn fan_param(&self, name: &str, k: usize) -> Option<f32> {
        self.fan_driven.get(k)?.get(name).copied()
    }

    pub fn fan(&self, k: usize) -> &Stream {
        static EMPTY: Stream = Stream::empty();
        self.fan.get(k).map_or(&EMPTY, CookValue::as_stream)
    }

    /// **Did this node emit anything on the previous tick?** — the node's own memory of the
    /// sequential circuit it sits in. `false` on the very first tick of a sim, and after any
    /// reset that cleared the `pre` state.
    ///
    /// This exists for the **simulation zone** (doc 48), and it is the ONLY question that
    /// answers *"has the sim started?"* correctly. The two obvious cheaper tests both lie:
    ///
    /// - *"is my `state` input empty?"* — a sim that killed its last element hands back an
    ///   EMPTY STREAM, which is a real answer that happens to carry nothing. Read it as "not
    ///   started" and the zone re-seeds from `init`: kill every particle and the scene
    ///   **resurrects**, one frame later, forever.
    /// - *"did an edge deliver a value on `state`?"* — it always did. The interior is wired into
    ///   `state` by a FORWARD edge, so the cook evaluates it first and it hands back an empty
    ///   stream on tick 1 (its own input, the zone's previous output, was the absent one).
    ///
    /// The state lives on the node's OWN previous output, so that is where the question belongs.
    pub fn started(&self) -> bool {
        self.started
    }

    /// **A identidade deste nó** — estável no grafo, sobrevive ao save, e nunca
    /// reusada.
    ///
    /// Existe para um nó estocástico se DECORRELACIONAR de um irmão de mesma
    /// semente (o *Use Layer as Seed* do Cavalry): dois `value.instance_field`
    /// arrastados para a tela com os defaults produziam campos **idênticos**, e a
    /// única saída era o artista lembrar de digitar sementes diferentes.
    ///
    /// ⚠️ **Quem a usa tem de fazê-lo por OPT-IN.** Dobrá-la na semente por
    /// default mudaria a aleatoriedade de **todo grafo já salvo** — a mesma lei 6
    /// que manda um variant novo ser apendado.
    pub fn node_key(&self) -> u32 {
        self.node_key
    }

    /// Current clock time; meaningful for `Temporal` nodes.
    pub fn playhead(&self) -> f64 {
        self.playhead
    }

    /// **Seconds since the previous tick** — `0.0` on the first tick of a cook (there is no
    /// previous), and after any reset.
    ///
    /// The engine has always known this and never said it, so the nodes that needed it invented
    /// it: `motion.integrate` carries a `sim_t` CLOCK COLUMN on its own state and subtracts. That
    /// works (and stays, because a per-element clock is exactly right for elements born at
    /// different times), but a node with no state of its own — a birth rate, a counter — had no
    /// way to ask at all.
    ///
    /// It is the ROOT clock's step. Inside a **time scope** it is `0.0`: the lane's clock is
    /// rewritten, so a delta across ticks is not a thing that exists there — and a node that
    /// needs `dt` to hold state is sequential, which a time scope already refuses
    /// (`CookError::SequentialInTimeScope`).
    pub fn dt(&self) -> f64 {
        self.dt
    }

    /// The current value of parameter `name`, resolved **wire > override > default** — the
    /// hierarchy the plan reserved from day one (*"socket conectado > literal"*): the node
    /// that drives it if one is wired ([`crate::graph::Graph::drive_param`], doc 58), else
    /// the graph's per-instance override ([`crate::graph::Graph::set_param`]), else the node
    /// type's manifest default. Panics if `name` is not a declared param of this node
    /// — a programmer error (the name is a literal of the node's own crate),
    /// caught by its golden test rather than silently reading `0.0`, the same
    /// no-silent-failure discipline as [`NodeManifest::param_default`].
    pub fn param(&self, name: &str) -> f32 {
        self.driven
            .get(name)
            .copied()
            .or_else(|| self.overrides.and_then(|o| o.get(name).copied()))
            .or_else(|| self.manifest.param_default(name))
            .unwrap_or_else(|| {
                panic!(
                    "node `{}` read undeclared param `{name}`",
                    self.manifest.name
                )
            })
    }

    /// The current value of a per-node **text** param `name` (e.g. an expression
    /// node's formula), set via [`crate::graph::Graph::set_text_param`]; `None` if
    /// unset. Unlike [`param`](Self::param) text params are **not** declared in the
    /// frozen `NodeManifest` (which is f32-only, ADR-0039) — they are the additive
    /// string channel (doc 32), so a node reads its own key with its own default.
    pub fn text_param(&self, name: &str) -> Option<&str> {
        self.text_overrides
            .and_then(|m| m.get(name))
            .map(String::as_str)
    }

    /// Emit the next output port's **instance stream**. Call once per output
    /// port, in order.
    pub fn emit(&mut self, stream: Stream) {
        self.outputs.push(CookValue::Instances(stream));
    }

    /// Emit the next output port's **opaque value** — a domain-specific rich
    /// value (e.g. a geometry `VectorNetwork`) carried type-erased behind
    /// `Arc<dyn Any>` (ADR-0058-amendment-1). Call once per output port, in
    /// order, just like [`Self::emit`]. The domain layer
    /// (`ph2d-vector-graph::VectorEvalExt::emit_network`) wraps this.
    pub fn emit_any(&mut self, value: Arc<dyn Any + Send + Sync>) {
        self.outputs.push(CookValue::Opaque(value));
    }
}

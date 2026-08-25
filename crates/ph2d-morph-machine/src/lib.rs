//! **A MÁQUINA DE ESTADOS DO MORPH** — quais formas, por que setas, e onde a forma está agora.
//!
//! Enio, 2026-08-24: *"um tipo de state machine específico para o tool Morph (…) possibilita que o
//! morph seja criado entre múltiplas formas de forma não destrutiva e funcional no runtime do
//! game"*, e em 2026-08-25: *"as setas devem ser desenhadas no canvas onde as formas foram
//! desenhadas"*.
//!
//! # ⭐ Um ESTADO é uma FORMA DESENHADA — não há objecto "estado"
//!
//! É a decisão que apaga o caso especial. O artista desenha A, B e C no canvas e liga-as com
//! setas; o estado da máquina é *"em qual das formas ela está"*, e a seta é *"como se vai de uma
//! para a outra"*. Nada de novo a nomear, nada de novo a gravar, e o que ele vê **é** o modelo.
//!
//! ⚠️ **Isto NÃO é o [`ph2d_ui_state`].** Aquela máquina interpola **poses de N objectos**
//! (translação, tinta, traço, geometria) entre papéis FIXOS de UI — hover, pressed. Aqui o assunto
//! é outro: **duas formas e um `t`**. A confusão custou meia hora na reabertura desta linha, e
//! fica escrita: *quem decide qual subsistema serve é o que o ARTISTA desenha, não o código que
//! está por perto*.
//!
//! # ⛔ Ela não sabe o que é uma tecla
//!
//! Uma condição é o **nome** de uma acção do Input Map, e esta crate nunca a resolve — [`fire`]
//! recebe o nome de quem o apurou. Quem tem o mapa é a shell (no editor) ou o jogo (no jogo), e é
//! por isso que a mesma lei corre nos dois sem uma linha de diferença. Mesma escolha, palavra por
//! palavra, do [`ph2d_ui_state::Machine`]: *"ela não sabe o que é um mouse"*.
//!
//! [`fire`]: MorphMachine::fire
//!
//! # ⚠️ E ela não tem relógio
//!
//! [`MorphMachine::advance`] recebe o `dt` de quem a chama. É a lição W4.T7 do Motion, onde o
//! `MotionTransport` **morreu**: dois relógios divergem, e o modo de falha é a forma a andar noutra
//! velocidade que a cena.

use ph2d_anim::{Easing, EasingFamily, EasingMode};
use ph2d_spring::{Spring, SpringState};
use serde::{Deserialize, Serialize};

/// **O id de uma forma do documento** — um `VecPathId` cru.
///
/// ⚠️ **Um `u64` e não o tipo**: trazer o `ph2d-vec-scene` para cá obrigaria o runtime do jogo a
/// arrastar o documento vectorial inteiro para saber que a máquina está no estado B. Quem casa o
/// id com a forma é quem tem a cena — a mesma fronteira que o [`ph2d_ecs::VecMorph`] já usa
/// (`sources: [u64; 2]`).
pub type ShapeId = u64;

/// **UMA SETA** — de que forma, para que forma, sob que condição, e a que ritmo.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MorphEdge {
    pub from: ShapeId,
    pub to: ShapeId,
    /// **O NOME da acção que a dispara.** Vazio = a seta **nunca** dispara sozinha; ela só é
    /// percorrida por quem a pedir pelo índice (a pré-visualização do painel).
    ///
    /// ⚠️ **Vazio-nunca-casa é load-bearing**, e a lição é emprestada da
    /// [`ph2d_ui_state::SignalBinding`]: a seta nasce sem condição quando o artista a desenha, e
    /// sem esta guarda **toda seta recém-desenhada dispararia** no dia em que alguém publicasse
    /// uma acção de nome vazio. O modo de falha não é um erro — é a forma a saltar sem ninguém ter
    /// pedido.
    pub when: String,
    /// Segundos, quando o motor é a curva. Ignorado se [`Self::spring`] estiver presente.
    pub duration_s: f64,
    pub easing: Easing,
    /// **A MOLA, quando o artista a escolhe.** `None` = o par duração+curva.
    ///
    /// ⚠️ `Option` e não um par de números com um *bool* ao lado: *ter mola* e *que mola* são a
    /// mesma decisão. Mesmo desenho do `HostStates::spring`.
    pub spring: Option<Spring>,
}

impl MorphEdge {
    /// Uma seta nova entre duas formas — **sem condição**, que é como ela nasce quando o artista a
    /// desenha no canvas. Ele nomeia a acção depois, no painel.
    #[must_use]
    pub fn new(from: ShapeId, to: ShapeId) -> Self {
        Self {
            from,
            to,
            when: String::new(),
            duration_s: DEFAULT_DURATION_S,
            easing: DEFAULT_EASING,
            spring: None,
        }
    }

    /// **Esta seta responde a `action`?** ⛔ Uma seta sem condição não responde a nada.
    #[must_use]
    pub fn matches(&self, action: &str) -> bool {
        !self.when.is_empty() && self.when == action
    }
}

/// ⚠️ **O valor de PARTIDA é o do irmão que já shipava** (`ph2d_ui_state::DEFAULT_DURATION_S`), e
/// não um palpite novo: um app que abre com dois ritmos por omissão ensina duas coisas ao artista
/// sem que nada na tela explique a diferença.
///
/// ⚠️ **Mas a constante é PRÓPRIA, e a divergência é permitida de propósito:** *quanto tempo demora
/// um hover* e *quanto tempo demora uma forma a virar outra* são perguntas de produto diferentes —
/// a primeira é um retorno visual de UI, a segunda é uma acção de personagem. ⛔ Nenhuma das duas
/// foi MEDIDA contra um smoke ainda; este número é o ponto de partida do artista, que ele muda por
/// SETA, e não um teto (`CLAUDE.md` §0.0).
pub const DEFAULT_DURATION_S: f64 = 0.15;

/// A curva de partida, pelo mesmo argumento — o `Cubic Out` que a transição de estados de UI usa.
pub const DEFAULT_EASING: Easing = Easing {
    family: EasingFamily::Cubic,
    mode: EasingMode::Out,
};

/// **O GRAFO AUTORADO** — o que o artista desenhou no canvas.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MorphGraph {
    /// A forma em que a máquina nasce.
    pub start: ShapeId,
    pub edges: Vec<MorphEdge>,
}

impl MorphGraph {
    /// **As setas que partem de `from`** — e é a ÚNICA porta pela qual se pergunta isso.
    ///
    /// ⭐ **A correcção nº 1 que a pesquisa trouxe** ([doc 31](../../../docs/Vector%20Module/31_pesquisa_maquinas_de_estado.md)):
    /// o *State Tree* do Unreal só considera as transições do estado **corrente**, e é isso que
    /// impede o grafo de virar a teia que os utilizadores do Animator do Unity descrevem. Um
    /// varrimento global de `edges` seria a versão errada — e é a que se escreve por acidente.
    pub fn from(&self, from: ShapeId) -> impl Iterator<Item = (usize, &MorphEdge)> {
        self.edges
            .iter()
            .enumerate()
            .filter(move |(_, e)| e.from == from)
    }

    /// **Toda forma que o grafo nomeia**, em ordem de primeira aparição, com o `start` à frente.
    ///
    /// ⚠️ É o que o canvas percorre para desenhar as setas, e o que o painel lista. Derivar em vez
    /// de guardar uma lista de estados é o que impede o grafo de ter um estado que nenhuma seta
    /// alcança **e** uma seta para um estado que a lista não tem.
    pub fn shapes(&self) -> Vec<ShapeId> {
        let mut out = vec![self.start];
        for e in &self.edges {
            for id in [e.from, e.to] {
                if !out.contains(&id) {
                    out.push(id);
                }
            }
        }
        out
    }
}

/// O que faz o `t` andar. Ou o relógio deformado por uma curva, ou uma mola.
///
/// ⚠️ Exclusivos porque respondem à MESMA pergunta — *quanto do caminho já andou?* Espelho do
/// `Drive` do [`ph2d_ui_state`], e a duplicação é deliberada: aquele interpola poses, este um
/// escalar, e uni-los obrigaria um dos dois a fingir ser o outro.
#[derive(Clone, Debug)]
enum Drive {
    Curve { duration: f64, easing: Easing },
    Spring { spring: Spring, state: SpringState },
}

/// Uma transição em voo.
#[derive(Clone, Debug)]
struct Flight {
    pair: (ShapeId, ShapeId),
    elapsed: f64,
    drive: Drive,
}

/// **A MÁQUINA** — onde ela está, para onde vai, e que par de formas a cena mostra agora.
///
/// ⚠️ **Ela não é documento e não pode ser serializada:** é *onde a forma está agora*, e o ficheiro
/// guarda *quais são as setas*. Gravá-la faria um projecto reabrir a meio de uma transição. Mesma
/// lei, palavra por palavra, das `UiMachines` da ponte de estados de UI.
#[derive(Clone, Debug)]
pub struct MorphMachine {
    /// **Onde o VOO EM CURSO aterra** — e, em repouso, onde a máquina está.
    ///
    /// ⚠️ **Ele salta no lançamento, e não na chegada.** A meio de `A→B` as setas que se oferecem
    /// são as de **B**: se ele só mudasse na chegada, carregar no botão de `B→C` durante o voo não
    /// casaria com seta nenhuma e o input **desaparecia**. É a mesma pergunta que o `ph2d-ui-state`
    /// resolve pela POSE viva: *"target == current" é um proxy que expira*.
    ///
    /// ⛔⛔ **E ele NÃO salta num pedido em fila — o primeiro gate desta crate provou porquê.**
    /// Com o salto na fila, um segundo pedido era lido a partir de um estado onde a máquina ainda
    /// não está: ou não casava com seta nenhuma (*o input do jogador desaparecia*), ou casava com
    /// uma seta cujo `from` não é onde ela vai aterrar — e o `launch` punha o par `(from, to)`
    /// dessa seta, **saltando um estado inteiro**. Mantendo-o no destino do voo, todo candidato à
    /// fila parte do MESMO sítio, e "o mais novo ganha" passa a ser seguro por construção.
    current: ShapeId,
    /// O par que a cena mostra, e o `t` nele. Fora de um voo, ele é o par do ÚLTIMO voo com `t`
    /// saturado — ⭐ e não `(current, current)`.
    ///
    /// ⚠️ **Isto poupa uma reconstrução de `Plan` por chegada.** O cache do `morph_live` é chaveado
    /// pela geometria em MUNDO das duas fontes, e a busca de fase custa os **5,9 ms** que o `Plan`
    /// foi inventado para matar. Trocar o par ao chegar rebuildaria por nada — `t = 1` no par
    /// `(A, B)` já **é** a forma B, ao bit.
    pair: (ShapeId, ShapeId),
    t: f64,
    flight: Option<Flight>,
    /// **A seta pedida durante um voo**, no máximo UMA.
    ///
    /// ⛔ **Ignorar o pedido perde o input** (o jogador carrega e não acontece nada) e **snapar
    /// para o par novo** não é exprimível: o `VecMorph` guarda **um par**, e sair do meio de
    /// `(A,B)` para `(B,C)` precisaria de uma mistura de três. ⇒ o pedido **espera a chegada**, que
    /// é o *input buffer* que todo jogo de acção tem.
    ///
    /// ⚠️ **O mais NOVO ganha**, e a fila não cresce: uma fila funda reproduziria, um segundo
    /// depois, teclas que o jogador já esqueceu.
    pending: Option<usize>,
}

impl MorphMachine {
    /// Uma máquina parada na forma inicial de `graph`.
    #[must_use]
    pub fn new(graph: &MorphGraph) -> Self {
        Self {
            current: graph.start,
            pair: (graph.start, graph.start),
            t: 0.0,
            flight: None,
            pending: None,
        }
    }

    /// Onde a máquina está comprometida a estar.
    #[must_use]
    pub fn current(&self) -> ShapeId {
        self.current
    }

    /// **O par de formas que a cena tem de mostrar** — o que vai para `VecMorph::sources`.
    #[must_use]
    pub fn pair(&self) -> (ShapeId, ShapeId) {
        self.pair
    }

    /// **Onde no caminho** — o que vai para `VecMorph::t`.
    #[must_use]
    pub fn t(&self) -> f32 {
        // O `VecMorph` guarda `f32`; a integração corre em `f64` para a mola não deriva.
        #[allow(clippy::cast_possible_truncation)]
        {
            self.t as f32
        }
    }

    /// Há uma transição a correr?
    #[must_use]
    pub fn is_flying(&self) -> bool {
        self.flight.is_some()
    }

    /// **As acções que fazem alguma coisa DAQUI** — sem repetições, na ordem das setas.
    ///
    /// ⭐ **A correcção nº 2 da pesquisa** ([doc 31](../../../docs/Vector%20Module/31_pesquisa_maquinas_de_estado.md)):
    /// o medo que os utilizadores descrevem do Animator não é o grafo, é *não saber quem lê o meu
    /// input*. Esta função existe para a UI poder responder isso na tela — sem ela, a mesma
    /// pergunta seria respondida por um `for` escrito no painel, que é a segunda implementação.
    pub fn live_actions<'a>(&self, graph: &'a MorphGraph) -> Vec<&'a str> {
        let mut out: Vec<&str> = Vec::new();
        for (_, e) in graph.from(self.current) {
            if !e.when.is_empty() && !out.contains(&e.when.as_str()) {
                out.push(&e.when);
            }
        }
        out
    }

    /// **A acção `action` aconteceu.** Devolve `true` se alguma seta a consumiu.
    ///
    /// ⚠️ **Só as setas do estado CORRENTE** — ver [`MorphGraph::from`]. E a **primeira** que casa
    /// ganha: a ordem das setas é a ordem em que o artista as desenhou, que é a única ordem que ele
    /// pode ver e mudar.
    pub fn fire(&mut self, graph: &MorphGraph, action: &str) -> bool {
        let Some((ix, _)) = graph.from(self.current).find(|(_, e)| e.matches(action)) else {
            return false;
        };
        self.take_edge(graph, ix);
        true
    }

    /// **Percorre a seta `ix` sem perguntar pela condição** — a porta da pré-visualização do
    /// painel e do botão *Show*.
    ///
    /// ⚠️ Ela existe porque o artista tem de poder **ver** a seta que acabou de desenhar antes de
    /// lhe dar nome; sem isto, uma seta sem condição seria indemonstrável.
    pub fn travel(&mut self, graph: &MorphGraph, ix: usize) -> bool {
        let Some(e) = graph.edges.get(ix) else {
            return false;
        };
        if e.from != self.current {
            return false;
        }
        self.take_edge(graph, ix);
        true
    }

    fn take_edge(&mut self, graph: &MorphGraph, ix: usize) {
        if self.flight.is_some() {
            // ⚠️ **A fila, e o `current` NÃO se mexe** — ver o doc dele.
            self.pending = Some(ix);
            return;
        }
        self.launch(&graph.edges[ix]);
    }

    /// ⚠️ **O `current` salta AQUI**, no lançamento — é o único sítio.
    fn launch(&mut self, e: &MorphEdge) {
        self.current = e.to;
        self.pair = (e.from, e.to);
        self.t = 0.0;
        self.flight = Some(Flight {
            pair: (e.from, e.to),
            elapsed: 0.0,
            drive: match e.spring {
                Some(spring) => Drive::Spring {
                    spring,
                    state: SpringState::at_rest(),
                },
                None => Drive::Curve {
                    duration: e.duration_s.max(0.0),
                    easing: e.easing,
                },
            },
        });
    }

    /// **Faz o tempo andar.** `dt` em segundos, do relógio de quem chama.
    pub fn advance(&mut self, graph: &MorphGraph, dt: f64) {
        let Some(mut f) = self.flight.take() else {
            return;
        };
        f.elapsed += dt;
        let arrived = match &mut f.drive {
            Drive::Curve { duration, easing } => {
                if *duration <= 0.0 {
                    // ⚠️ Duração zero **chega**, não divide por zero. Uma seta instantânea é uma
                    // escolha legítima do artista (um corte), e a alternativa — recusar — faria o
                    // slider ter um valor proibido no meio da faixa.
                    self.t = 1.0;
                    true
                } else {
                    let raw = (f.elapsed / *duration).clamp(0.0, 1.0); // CLAMP-OK: bounds ordered
                    self.t = easing.eval(raw);
                    f.elapsed >= *duration
                }
            }
            Drive::Spring { spring, state } => {
                // ⚠️ O `advance` devolve **se assentou**, e é ele quem integra: a mola vive no
                // `ph2d-spring` e reimplementá-la aqui seria a segunda mola do app.
                let settled = state.advance(dt, *spring);
                self.t = state.x;
                settled
            }
        };
        self.pair = f.pair;
        if arrived {
            self.t = 1.0;
            self.flight = None;
            // O pedido que esperava a chegada — ver o doc de `pending`.
            //
            // ⚠️ **E ele é RECONFERIDO contra onde a máquina de facto chegou.** O artista pode ter
            // apagado ou repontado a seta durante o voo (a pré-visualização corre enquanto ele
            // edita), e uma seta cujo `from` já não é o estado corrente poria o par a saltar um
            // estado. *Um índice guardado é uma afirmação sobre uma lista que pode ter mudado.*
            if let Some(ix) = self.pending.take()
                && let Some(e) = graph.edges.get(ix)
                && e.from == self.current
            {
                self.launch(e);
            }
        } else {
            self.flight = Some(f);
        }
    }
}

#[cfg(test)]
mod tests;

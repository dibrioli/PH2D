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

/// ⭐⭐⭐ **A CHAVE de uma forma** — o que o artista autora *sobre* ela: a tecla que a alcança e o
/// ritmo com que se chega.
///
/// # ⛔⛔ Porque ela NÃO tem a forma dentro (W11)
///
/// Enio, 2026-08-26: *"sendo uma forma que previamente não participava do Morph states, se for
/// arrastada na hierarquia e se tornar filha de um objeto Morph State, automaticamente passa a
/// fazer parte do sistema."*
///
/// ⇒ **a LISTA de formas deixou de ser autorada: ela são os FILHOS.** Guardar as duas coisas
/// (uma lista própria *e* a hierarquia) seria ter duas respostas para *«que formas estão neste
/// conjunto»* — e o arrastar-para-dentro torna a discordância um **gesto do artista**, portanto
/// obrigatória de resolver. É a lei que o módulo 3D Modeling já paga: *a hierarquia da cena É o
/// documento.*
///
/// ⇒ o que sobra de autorado é **isto**, indexado por `ShapeId`. Uma forma sem chave usa os
/// valores de partida — e é assim que um filho arrastado para dentro **já funciona** sem que
/// ninguém escreva nada por ele.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MorphKey {
    /// **O NOME da acção que LEVA a esta forma**, de onde quer que a máquina esteja. Vazio = a
    /// forma é **inalcançável por tecla**.
    pub when: String,
    /// Segundos para CHEGAR aqui, quando o motor é a curva. Ignorado se [`Self::spring`] existir.
    pub duration_s: f64,
    pub easing: Easing,
    /// **A MOLA, quando o artista a escolhe.** `None` = o par duração+curva.
    pub spring: Option<Spring>,
}

impl Default for MorphKey {
    fn default() -> Self {
        Self {
            when: String::new(),
            duration_s: DEFAULT_DURATION_S,
            easing: DEFAULT_EASING,
            spring: None,
        }
    }
}

/// ⭐⭐ **UM ESTADO** — que forma, o que leva ATÉ ela, e a que ritmo.
///
/// # ⛔⛔ A acção pertence ao DESTINO, não à passagem (W10)
///
/// Enio, 2026-08-25: *"em vez de um evento para cada transição, melhor seria um evento por shape.
/// Ou seja: se a seta para cima leva ao retângulo azul, independente de que forma estiver ativa no
/// momento, a seta para cima vai levar ao retângulo azul."*
///
/// ⚠️ **É uma mudança de MODELO, não de painel.** Até aqui a máquina guardava `n(n-1)` arestas, uma
/// por par ordenado, cada uma com a sua condição — e o artista tinha de escrever a mesma tecla em
/// `n-1` sítios para que ela significasse sempre a mesma coisa. Aqui a lista tem **`n`** entradas,
/// uma por forma, e a tecla **é** o nome daquela forma.
///
/// ⭐ O que isso apaga: a possibilidade de a mesma tecla levar a sítios diferentes conforme o
/// estado. ⚠️ **É deliberado, e é o pedido:** *"independente de que forma estiver ativa"*. Uma
/// máquina em que a mesma tecla faz coisas diferentes conforme onde se está é exactamente a teia
/// que a pesquisa (doc 31) descreve como o medo do Animator do Unity.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MorphState {
    /// A forma deste estado.
    pub shape: ShapeId,
    /// **O NOME da acção que LEVA a esta forma**, de onde quer que a máquina esteja. Vazio = a
    /// forma é **inalcançável por tecla**; ela só é visitada por quem a pedir pelo índice (a
    /// pré-visualização do painel).
    ///
    /// ⚠️ **Vazio-nunca-casa é load-bearing**, e a lição é emprestada da
    /// [`ph2d_ui_state::SignalBinding`]: o estado nasce sem condição, e sem esta guarda **toda
    /// forma sem tecla seria alcançada** no dia em que alguém publicasse uma acção de nome vazio.
    /// O modo de falha não é um erro — é a forma a saltar sem ninguém ter pedido.
    pub when: String,
    /// Segundos para CHEGAR aqui, quando o motor é a curva. Ignorado se [`Self::spring`] existir.
    ///
    /// ⚠️ **O ritmo também é do DESTINO**, e pela mesma razão que a tecla: *quanto tempo demora a
    /// virar isto* é uma propriedade da forma, não do sítio de onde se vem.
    pub duration_s: f64,
    pub easing: Easing,
    /// **A MOLA, quando o artista a escolhe.** `None` = o par duração+curva.
    ///
    /// ⚠️ `Option` e não um par de números com um *bool* ao lado: *ter mola* e *que mola* são a
    /// mesma decisão. Mesmo desenho do `HostStates::spring`.
    pub spring: Option<Spring>,
}

impl MorphState {
    /// Um estado novo sobre `shape` — **sem condição**, que é como ele nasce quando o conjunto é
    /// criado. O artista nomeia a acção depois, no painel.
    #[must_use]
    pub fn new(shape: ShapeId) -> Self {
        Self {
            shape,
            when: String::new(),
            duration_s: DEFAULT_DURATION_S,
            easing: DEFAULT_EASING,
            spring: None,
        }
    }

    /// **Esta forma responde a `action`?** ⛔ Um estado sem condição não responde a nada.
    #[must_use]
    pub fn matches(&self, action: &str) -> bool {
        !self.when.is_empty() && self.when == action
    }

    /// ⭐ **O estado DERIVADO** de uma forma mais a chave que o artista lhe deu (W11).
    ///
    /// ⚠️ É a única porta pela qual um `MorphState` nasce de dados autorados — construí-lo à mão
    /// noutro sítio seria a segunda lei de *«o que é uma forma sem chave»*.
    #[must_use]
    pub fn with_key(shape: ShapeId, key: &MorphKey) -> Self {
        Self {
            shape,
            when: key.when.clone(),
            duration_s: key.duration_s,
            easing: key.easing,
            spring: key.spring,
        }
    }

    /// A chave deste estado — o que dele é **autorado**, sem a forma.
    #[must_use]
    pub fn key(&self) -> MorphKey {
        MorphKey {
            when: self.when.clone(),
            duration_s: self.duration_s,
            easing: self.easing,
            spring: self.spring,
        }
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

/// **A MÁQUINA AUTORADA** — que formas o objecto pode vestir, e o que leva a cada uma.
///
/// ⚠️ **Uma LISTA de estados, e não um grafo de arestas** (W10). As passagens continuam a existir
/// — de qualquer estado para qualquer outro —, mas deixam de ser **guardadas**: elas são a
/// consequência de haver `n` formas, e guardá-las obrigava o artista a escrever a mesma tecla
/// `n-1` vezes para ela significar sempre a mesma coisa.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MorphGraph {
    /// As formas, na ordem em que o artista as escolheu. **A primeira é onde a máquina nasce.**
    pub states: Vec<MorphState>,
}

impl MorphGraph {
    /// **A forma em que a máquina nasce** — a primeira da lista.
    ///
    /// ⚠️ **DERIVADA, e não um campo ao lado.** Um `start: ShapeId` guardado podia apontar para uma
    /// forma que a lista não tem — e é o tipo de discordância que passa MUDA por uma fusão, porque
    /// nada no git sabe o que o número significa (`CLAUDE.md` §5.0).
    #[must_use]
    pub fn start(&self) -> Option<ShapeId> {
        self.states.first().map(|s| s.shape)
    }

    /// **O estado que `action` alcança**, com o índice dele. ⛔ **É a ÚNICA porta** pela qual se
    /// pergunta isso — uma segunda varredura noutro sítio daria uma resposta diferente no dia em
    /// que duas formas partilhassem a tecla.
    ///
    /// ⚠️ **A PRIMEIRA que casa ganha**, e a ordem é a da lista, que é a ordem em que o artista
    /// escolheu as formas — a única ordem que ele pode ver e mudar.
    #[must_use]
    pub fn reached_by(&self, action: &str) -> Option<(usize, &MorphState)> {
        self.states
            .iter()
            .enumerate()
            .find(|(_, s)| s.matches(action))
    }

    /// **Toda forma que a máquina nomeia**, na ordem da lista.
    #[must_use]
    pub fn shapes(&self) -> Vec<ShapeId> {
        self.states.iter().map(|s| s.shape).collect()
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
/// guarda *que formas existem e o que leva a cada uma*. Gravá-la faria um projecto reabrir a meio
/// de uma transição. Mesma lei, palavra por palavra, das `UiMachines` da ponte de estados de UI.
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
        Self::seeded(graph, None)
    }

    /// ⭐⭐⭐ **Uma máquina SEMEADA pela forma que a cena já mostra** (plano 32 W11d).
    ///
    /// ⚠️ **Uma máquina que dirige o mundo tem de ser semeada por ele.** Esta não é serializada —
    /// ela morre sempre que a pré-visualização se desliga — mas o `VecMorph` que ela escreveu
    /// **fica** (o ledger larga a condução e a `settle` promove o vivo a documento). Nascer em
    /// `graph.start()` com a cena noutra forma punha as duas em desacordo, e o sintoma era o botão
    /// ▶ **recusado** pela regra *«chegar onde já se está não é chegar»* — sobre um «onde» que só a
    /// máquina acreditava.
    ///
    /// ⛔ **Uma semente FORA do grafo cai no início**, e não é tolerância: a forma pode ter sido
    /// desconectada do conjunto entre uma pré-visualização e a seguinte, e uma máquina parada numa
    /// forma que já não é estado nenhum não teria de onde sair.
    ///
    /// ⚠️ **`unwrap_or_default` e não um pânico:** uma lista vazia é uma máquina INERTE (o
    /// `reached_by` não acha nada e o `travel` recusa), e recusar aqui obrigaria todo chamador a
    /// tratar um caso que o produto não consegue produzir — o `morph_set` nunca cria menos de 2.
    #[must_use]
    pub fn seeded(graph: &MorphGraph, showing: Option<ShapeId>) -> Self {
        let start = showing
            .filter(|s| graph.states.iter().any(|st| st.shape == *s))
            .or_else(|| graph.start())
            .unwrap_or_default();
        Self {
            current: start,
            pair: (start, start),
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

    /// **As acções que fazem alguma coisa DAQUI** — sem repetições, na ordem da lista.
    ///
    /// ⭐ **A correcção nº 2 da pesquisa** ([doc 31](../../../docs/Vector%20Module/31_pesquisa_maquinas_de_estado.md)):
    /// o medo que os utilizadores descrevem do Animator não é o grafo, é *não saber quem lê o meu
    /// input*. Esta função existe para a UI poder responder isso na tela — sem ela, a mesma
    /// pergunta seria respondida por um `for` escrito no painel, que é a segunda implementação.
    ///
    /// ⚠️ **Sob o modelo por-forma (W10) são TODAS menos a de onde já se está** — e é essa a
    /// resposta certa: a tecla que leva à forma corrente não faz nada, e listá-la seria prometer um
    /// efeito que não acontece.
    pub fn live_actions<'a>(&self, graph: &'a MorphGraph) -> Vec<&'a str> {
        let mut out: Vec<&str> = Vec::new();
        for s in &graph.states {
            if !s.when.is_empty() && s.shape != self.current && !out.contains(&s.when.as_str()) {
                out.push(&s.when);
            }
        }
        out
    }

    /// **A acção `action` aconteceu.** Devolve `true` se algum estado a consumiu.
    ///
    /// ⭐ **Sob o modelo por-forma (W10) a acção vale de QUALQUER estado** — é o pedido do Enio,
    /// palavra por palavra: *"independente de que forma estiver ativa no momento"*.
    ///
    /// ⛔ **Menos uma: a tecla da forma em que já se está NÃO faz nada.** Ela seria uma transição
    /// de uma forma para ela própria — nem sequer é exprimível (o `VecMorph` guardaria `(X, X)` e o
    /// `t` andaria sobre um caminho de comprimento zero), e o artista leria um estremecimento sem
    /// causa. *Chegar onde já se está não é chegar.*
    pub fn fire(&mut self, graph: &MorphGraph, action: &str) -> bool {
        let Some((ix, st)) = graph.reached_by(action) else {
            return false;
        };
        if st.shape == self.current {
            return false;
        }
        self.take(graph, ix);
        true
    }

    /// **Vai para o estado `ix` sem perguntar pela condição** — a porta da pré-visualização do
    /// painel e do botão *Show*.
    ///
    /// ⚠️ Ela existe porque o artista tem de poder **ver** a forma antes de lhe dar tecla; sem
    /// isto, um estado sem condição seria indemonstrável.
    pub fn travel(&mut self, graph: &MorphGraph, ix: usize) -> bool {
        let Some(st) = graph.states.get(ix) else {
            return false;
        };
        if st.shape == self.current {
            return false;
        }
        self.take(graph, ix);
        true
    }

    fn take(&mut self, graph: &MorphGraph, ix: usize) {
        if self.flight.is_some() {
            // ⚠️ **A fila, e o `current` NÃO se mexe** — ver o doc dele.
            self.pending = Some(ix);
            return;
        }
        self.launch(&graph.states[ix]);
    }

    /// ⚠️ **O `current` salta AQUI**, no lançamento — é o único sítio.
    ///
    /// ⚠️ **O `from` é o estado CORRENTE**, e não um campo do destino: sob o modelo por-forma a
    /// passagem não é guardada — ela é *de onde estou* para *onde a tecla leva*.
    fn launch(&mut self, st: &MorphState) {
        let from = self.current;
        self.current = st.shape;
        self.pair = (from, st.shape);
        self.t = 0.0;
        self.flight = Some(Flight {
            pair: (from, st.shape),
            elapsed: 0.0,
            drive: match st.spring {
                Some(spring) => Drive::Spring {
                    spring,
                    state: SpringState::at_rest(),
                },
                None => Drive::Curve {
                    duration: st.duration_s.max(0.0),
                    easing: st.easing,
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
            // apagado ou repontado um estado durante o voo (a pré-visualização corre enquanto ele
            // edita), e um índice que já não existe — ou que aponta para a forma em que se acabou
            // de chegar — faria o par saltar um estado ou estremecer sobre si próprio.
            // *Um índice guardado é uma afirmação sobre uma lista que pode ter mudado.*
            if let Some(ix) = self.pending.take()
                && let Some(st) = graph.states.get(ix)
                && st.shape != self.current
            {
                self.launch(st);
            }
        } else {
            self.flight = Some(f);
        }
    }
}

#[cfg(test)]
mod tests;

//! **A saída de sinais do runtime — uma outbox, N consumidores, e o produtor não chama ninguém.**
//!
//! Duas partes do app já GRITAM um nome: a timeline, quando o play cruza um marker
//! ([ADR-0143]), e a física, quando duas coisas se tocam (`SignalOnHit`/`SignalOnLeave`). As
//! duas produzem, ninguém escuta, e o único consumidor que existe é um toast escrito à mão
//! **duas vezes**, uma ao lado de cada produtor. Esta crate é o lugar onde os dois se
//! encontram — e o §3 do doc do produtor da física já dizia que esse lugar seria o SHELL:
//!
//! > *"Esta porta devolve o **nome** e as duas entidades; quem funde as duas fontes numa saída
//! > é o SHELL, que já é o dono do consumidor e já drena a outra."*
//!
//! # Por que isto não é um `MessageBus` de handlers
//!
//! O `ph2d-script::messaging` guarda os consumidores DENTRO de si
//! (`Handler = Box<dyn FnMut(&Message)>`) e despacha com `&mut self`. Esse é o modelo do
//! Defold e está certo para **entidades scriptadas se endereçando** — todas vivem dentro da
//! mesma VM. Ele é o modelo errado para **subsistemas do host**: um consumidor de áudio
//! precisa de `&mut` no ring de comando, um de UI no estado da tela, e nenhum dos dois pode
//! ser emprestado para dentro de uma closure que o barramento possui. Guardar handlers ali é,
//! muito provavelmente, por que aquele barramento tem zero consumidores desde que nasceu.
//!
//! ⚠️ **A [`SignalOutbox::read`] toma `&self`.** É essa assinatura, e nada mais, que deixa um
//! consumidor segurar `&mut` no PRÓPRIO estado enquanto lê — e é ela que torna a ordem dos
//! consumidores **visível no sítio de chamada** em vez de escondida na ordem de registro.
//!
//! # O armazenamento é duplo-buffer com cursor por leitor
//!
//! O desenho é o `Events<E>`/`EventReader` do Bevy, pelo motivo dele: um sinal publicado no
//! quadro `N` fica visível em `N` **e** em `N+1`, e só então é descartado. Um consumidor que
//! corra ANTES do produtor recebe o sinal um quadro depois — nunca *nunca*. O modo de falha do
//! buffer único é **perda silenciosa**, que é a classe de defeito que esta casa mais paga.
//!
//! ⚠️ Isso **não** afrouxa a ordem do quadro: ela segue load-bearing para a LATÊNCIA (mesmo
//! quadro contra o seguinte), que é o que decide se um som sai junto com a colisão. O
//! duplo-buffer é a rede embaixo, não a licença para tirar a ordem.
//!
//! Em regime permanente não há alocação: os dois `Vec` são limpos e trocados, nunca soltos.
//!
//! [ADR-0143]: ../../../docs/architecture/decisions/0143-timeline-signals-a-marker-emits-a-decoupled-event-not-a-call.md

#![forbid(unsafe_code)]

use std::sync::Arc;

/// A entidade a que uma origem se refere, nos bits crus do ECS.
///
/// ⚠️ **Só é válida DENTRO do quadro que publicou o sinal.** Bits de entidade são ids de
/// ALOCAÇÃO — o undo global respawna tudo e recicla os bits —, então um consumidor os resolve
/// durante a leitura e **nunca os guarda**. É a mesma exposição que fez a timeline apontar para
/// objetos por `wire_id` (o hash do `Name`) em vez de por bits, e que o `PhysicsJoint` pagou
/// nomeando os corpos.
///
/// O tipo é `u64` cru, e não `ph2d_ecs::Entity`, para esta crate seguir sendo uma FOLHA: um
/// consumidor de áudio não linka um ECS para saber que uma porta abriu. Quem tem o mundo em mãos
/// converte de volta com `Entity::from_bits`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityBits(pub u64);

/// De onde um sinal veio, com o que só aquela fonte sabe.
///
/// ⚠️ **O CONTRATO é o nome** (ADR-0143) — um consumidor casa numa string e nunca precisa
/// perguntar a origem. Isto existe porque as duas fontes carregam detalhe que se perderia num
/// tipo comum estreito demais (o instante em que a régua cruzou o marker; QUEM bateu em quem), e
/// jogar esse detalhe fora seria o mesmo que codificá-lo num inteiro sem tipo mais tarde.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalOrigin {
    /// O play para a frente cruzou um marker que carrega este nome, no instante `t` da cena.
    Timeline {
        /// O tempo de cena (segundos) em que a régua cruzou o marker.
        t: f64,
    },
    /// Um contato ou um sensor da física emitiu este nome.
    ///
    /// ⚠️ **Não carrega FASE, e é deliberado** (a lei do produtor): chegada e saída não se
    /// distinguem por um campo, distinguem-se por serem NOMES diferentes, autorados em duas rows.
    Contact {
        /// Quem GRITOU — a entidade que carrega o componente de sinal.
        source: EntityBits,
        /// Quem chegou, ou quem saiu.
        other: EntityBits,
    },
    /// Um CONTROLE autorado foi apertado — um botão que o artista desenhou no painel.
    ///
    /// ⚠️ **Sem carga, e isso não é um variant pobre — é a origem em que o nome É tudo.** Um
    /// marker traz o instante e um contato traz quem bateu em quem porque esse detalhe se
    /// perderia; um aperto de botão não tem detalhe nenhum além de *aconteceu, e chama-se assim*.
    /// A origem fica porque um consumidor pode querer ROTEAR pela fonte (e o log imprime-a), não
    /// porque ela carregue dado.
    Control,
    /// Um `pulse.signal` do grafo de Motion disparou neste TIQUE do cook.
    ///
    /// ⚠️ **`rows` existe porque o colapso é LOSSY e o número é o que ele descarta.** Um pulso é
    /// por LINHA e um sinal é por QUADRO, então uma grade de 576 pontos que dispara junto vira UM
    /// evento — 576 sons no mesmo quadro é ruído, não um efeito. Guardar quantas linhas
    /// dispararam devolve a informação sem multiplicar o sinal, e é o que deixa um consumidor
    /// futuro escalar um volume pela intensidade em vez de adivinhar.
    Motion {
        /// O tique fixo do cook em que o pulso saiu.
        tick: u64,
        /// Quantas LINHAS dispararam nesse tique (sempre ≥ 1 — zero não publica).
        rows: usize,
    },
    /// Uma **animação de sprite** (§11) fechou um ciclo ou chegou ao fim.
    ///
    /// ⚠️ **`cycles` existe pela MESMA razão que o `rows` do `Motion`: o colapso é lossy e o
    /// número é o que ele descarta.** Um tique que apanha atraso (a janela esteve parada, a
    /// máquina engasgou) fecha vários ciclos de uma vez, e publicar um sinal por ciclo daria uma
    /// rajada de passos que ninguém deu. Sai **um** evento, com quantos ciclos ele representa.
    ///
    /// ⚠️ **Não carrega o nome da animação, e é deliberado** — o nome do SINAL é autorado na tag
    /// (`AnimationTag::signal_on_loop` / `_on_finish`), então ele já é o contrato. É a mesma lei do
    /// `Contact`: fim e volta não se distinguem por um campo, distinguem-se por serem nomes
    /// diferentes, autorados em dois sítios.
    Animation {
        /// A entidade cuja animação falou.
        source: EntityBits,
        /// Quantos ciclos fecharam neste tique (sempre ≥ 1 — zero não publica).
        cycles: u32,
    },
    /// ⭐⭐⭐ **Um `Timer` autorado chegou ao fim de um período** (TOP-20 #2, 2026-09-08).
    ///
    /// ⚠️ **É o primeiro produtor que não precisa de dois corpos a tocarem-se.** Até aqui o único
    /// produtor autorável da cena era o contacto da física — logo nada podia acontecer *por si*.
    ///
    /// ⚠️ **`fires` existe pela MESMA razão que o `cycles` do `Animation` e o `rows` do `Motion`:
    /// o colapso é lossy, e o número é o que ele descarta.** Um tique que apanha atraso (a janela
    /// esteve parada) fecha vários períodos de um timer que repete, e publicar um sinal por
    /// período daria uma rajada que ninguém pediu. Sai **um** evento, com quantos ele representa.
    ///
    /// ⚠️ **Não carrega o nome do timer, e é deliberado** — o nome do SINAL é autorado no
    /// `Timer::signal`, então ele já é o contrato. É a mesma lei do `Contact` e do `Animation`.
    Timer {
        /// A entidade cujo timer falou.
        source: EntityBits,
        /// Quantos períodos fecharam neste tique (sempre ≥ 1 — zero não publica).
        fires: u32,
    },
    /// ⭐⭐⭐ **Uma FÁBRICA pôs cópias na cena** (TOP-20 #11, 2026-09-14).
    ///
    /// ⚠️ **`count` pela MESMA razão que o `fires` do `Timer`**: uma rajada é um evento com o
    /// número dentro, nunca `n` eventos. Um `burst = 64` que publicasse 64 sinais daria a um
    /// consumidor de SOM sessenta e quatro disparos no mesmo quadro.
    Spawned {
        /// A fábrica que falou.
        source: EntityBits,
        /// Quantas cópias nasceram neste tique (sempre ≥ 1 — zero não publica).
        count: u32,
    },
    /// ⭐⭐⭐ **Uma cópia MORREU** (TOP-20 #12) — de velha ou por sair do ecrã do jogo.
    ///
    /// ⚠️ **Não carrega a CAUSA, e é deliberado** — a lei do `Contact`: *morrer de velho* e *sair
    /// do ecrã* distinguem-se por serem **nomes** diferentes, autorados em campos diferentes
    /// (`Lifetime::on_death`; o fora-do-ecrã é calado hoje), nunca por um campo de fase.
    ///
    /// ⚠️ **`source` é quem MORREU, e ela já não existe quando o consumidor lê** — o dreno corre
    /// a seguir. Os bits servem para o log e para rotear, ⛔ nunca para ir buscar a entidade.
    Death {
        /// A cópia que morreu.
        source: EntityBits,
    },
    /// ⭐⭐⭐ **Um objecto ENTROU ou SAIU de um estado** (TOP-20 #15).
    ///
    /// ⚠️ **Não carrega se foi entrada ou saída, e é a lei do produtor desta casa:** entrar e sair
    /// distinguem-se por serem **nomes diferentes**, autorados em dois campos do estado — nunca por
    /// um campo de fase. É a mesma decisão do `Contact` (chegada/saída) e das tags de animação.
    ///
    /// ⚠️ **Nem carrega o nome do ESTADO**: o nome do sinal é o que o artista autorou, logo ele já é
    /// o contrato. Pôr os dois daria duas respostas a *«o que é este evento?»*.
    StateMachine {
        /// Quem pensou — a entidade que carrega a máquina.
        source: EntityBits,
    },
    /// ⭐⭐⭐ **Um SCRIPT do artista emitiu este nome** (`ph2d.emit`, TOP-20 #16).
    ///
    /// ⚠️ **É a regra de produto do levantamento, e esta variante é o que a torna possível:** um
    /// script liga-se ao mundo pelos MESMOS sinais da tabela de acções, nunca por referência directa
    /// a outro objecto — então o que ele diz tem de entrar no mesmo outbox que o resto da casa.
    /// ⚠️ Sem contagem: duas chamadas a `ph2d.emit` num gancho são dois factos.
    Script {
        /// Quem emitiu — a entidade que carrega o script.
        source: EntityBits,
    },
}

/// Um sinal publicado neste quadro.
#[derive(Debug, Clone, PartialEq)]
pub struct Signal {
    /// O nome autorado — o contrato inteiro.
    ///
    /// `Arc<str>` e não `String`: clonar um sinal para um segundo consumidor é um refcount, não
    /// uma cópia. Sem tabela de interning, que seria estado global a manter vivo para responder
    /// uma pergunta que uma comparação de string já responde neste volume.
    pub name: Arc<str>,
    /// De onde ele veio.
    pub origin: SignalOrigin,
}

impl Signal {
    /// Um sinal que a timeline emitiu ao cruzar um marker.
    #[must_use]
    pub fn from_timeline(name: &str, t: f64) -> Self {
        Self {
            name: Arc::from(name),
            origin: SignalOrigin::Timeline { t },
        }
    }

    /// Um sinal que a física emitiu num contato ou sensor.
    #[must_use]
    pub fn from_contact(name: &str, source: u64, other: u64) -> Self {
        Self {
            name: Arc::from(name),
            origin: SignalOrigin::Contact {
                source: EntityBits(source),
                other: EntityBits(other),
            },
        }
    }

    /// Um sinal que um controle autorado emitiu ao ser apertado.
    ///
    /// ⚠️ **Só o que NÃO tem outro canal vem por aqui.** Um slider e um toggle já dizem o que
    /// valem pelo `WidgetStore`, que é quem dirige a arte; publicá-los TAMBÉM como sinal poria o
    /// mesmo facto em dois fios. Um APERTO não tem estado — ele não é um valor que se leia depois
    /// —, então este é o único canal que o carrega.
    #[must_use]
    pub fn from_control(name: &str) -> Self {
        Self {
            name: Arc::from(name),
            origin: SignalOrigin::Control,
        }
    }

    /// Um sinal que um `pulse.signal` do grafo de Motion emitiu.
    ///
    /// ⚠️ **Quem decide se ISTO chega a ser chamado é o relógio, não o grafo.** O cook re-roda
    /// ao arrastar a régua, então o shell só publica quando o playhead está TOCANDO para a
    /// frente — a mesma lei, pela mesma porta, que o emissor de markers da timeline usa.
    #[must_use]
    pub fn from_motion(name: &str, tick: u64, rows: usize) -> Self {
        Self {
            name: Arc::from(name),
            origin: SignalOrigin::Motion { tick, rows },
        }
    }

    /// Um sinal que a §11 Animation emitiu ao fechar um ciclo (ou ao acabar).
    #[must_use]
    pub fn from_animation(name: &str, source: u64, cycles: u32) -> Self {
        Self {
            name: Arc::from(name),
            origin: SignalOrigin::Animation {
                source: EntityBits(source),
                cycles,
            },
        }
    }

    /// **Um período de um [`SignalOrigin::Timer`] fechou.**
    ///
    /// ⚠️ **`fires` é sempre ≥ 1**: zero não publica, e é o chamador que o garante — publicar um
    /// evento que diz *«aconteceu zero vezes»* é ruído com cara de facto.
    #[must_use]
    pub fn from_timer(name: &str, source: u64, fires: u32) -> Self {
        Self {
            name: Arc::from(name),
            origin: SignalOrigin::Timer {
                source: EntityBits(source),
                fires,
            },
        }
    }

    /// **Um objecto entrou ou saiu de um estado** ([`SignalOrigin::StateMachine`]).
    ///
    /// ⚠️ Sem contagem, ao contrário do timer e da fábrica: uma transição **não colapsa** — cada uma
    /// é um facto com o seu nome, e duas transições num tique são dois nomes diferentes.
    #[must_use]
    pub fn from_state_machine(name: &str, source: u64) -> Self {
        Self {
            name: Arc::from(name),
            origin: SignalOrigin::StateMachine {
                source: EntityBits(source),
            },
        }
    }

    /// **Um script emitiu este nome** ([`SignalOrigin::Script`]).
    #[must_use]
    pub fn from_script(name: &str, source: u64) -> Self {
        Self {
            name: Arc::from(name),
            origin: SignalOrigin::Script {
                source: EntityBits(source),
            },
        }
    }

    /// **Uma fábrica pôs `count` cópias na cena** ([`SignalOrigin::Spawned`]).
    ///
    /// ⚠️ **`count` é sempre ≥ 1** — a mesma lei do `from_timer`.
    #[must_use]
    pub fn from_spawn(name: &str, source: u64, count: u32) -> Self {
        Self {
            name: Arc::from(name),
            origin: SignalOrigin::Spawned {
                source: EntityBits(source),
                count,
            },
        }
    }

    /// **Uma cópia morreu** ([`SignalOrigin::Death`]).
    #[must_use]
    pub fn from_death(name: &str, source: u64) -> Self {
        Self {
            name: Arc::from(name),
            origin: SignalOrigin::Death {
                source: EntityBits(source),
            },
        }
    }

    /// Este sinal se chama `name`? A porta que um consumidor usa para casar no contrato.
    #[must_use]
    pub fn is(&self, name: &str) -> bool {
        &*self.name == name
    }
}

/// **A saída.** Os produtores publicam aqui; cada consumidor lê com o próprio [`SignalReader`].
///
/// Ver o doc do módulo para o porquê do duplo-buffer e do `&self` na leitura.
#[derive(Debug, Default)]
pub struct SignalOutbox {
    /// O que foi publicado no quadro ANTERIOR — ainda legível, descartado no próximo `advance_frame`.
    older: Vec<Signal>,
    /// O que foi publicado NESTE quadro.
    newer: Vec<Signal>,
    /// Quantos sinais já passaram por aqui, desde sempre. É contra ele que um cursor se mede.
    published: u64,
}

impl SignalOutbox {
    /// Uma saída vazia.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Publica um sinal. O produtor não sabe — e não pode saber — quem vai lê-lo.
    pub fn publish(&mut self, signal: Signal) {
        self.newer.push(signal);
        self.published += 1;
    }

    /// Vira o quadro: o que era deste quadro passa a ser do anterior, e o do anterior sai.
    ///
    /// ⚠️ **Chamada UMA vez por quadro, ANTES do primeiro produtor.** Chamada no meio, ela
    /// aposentaria sinais que consumidores daquele mesmo quadro ainda não leram; chamada duas
    /// vezes, cortaria a janela de graça de um quadro pela metade. O sítio no shell tem gate.
    ///
    /// Não solta memória: limpa e troca, então o regime permanente é zero-alloc.
    pub fn advance_frame(&mut self) {
        self.older.clear();
        std::mem::swap(&mut self.older, &mut self.newer);
    }

    /// Os sinais que este leitor ainda não viu, na ordem em que foram publicados.
    ///
    /// ⚠️ **`&self`.** Um consumidor lê a saída enquanto segura `&mut` no próprio estado — é
    /// disso que o modelo de handlers boxeados é incapaz, e é a razão de esta crate existir.
    ///
    /// Um leitor que pule mais de um quadro perde o que caiu fora da janela; o quanto ele perdeu
    /// vai para [`SignalReader::missed`], porque perda silenciosa não é uma opção.
    pub fn read<'a>(&'a self, reader: &mut SignalReader) -> impl Iterator<Item = &'a Signal> + 'a {
        let buffered = (self.older.len() + self.newer.len()) as u64;
        let first = self.published - buffered;
        if reader.seen < first {
            reader.missed += first - reader.seen;
            reader.seen = first;
        }
        let skip = usize::try_from(reader.seen - first).unwrap_or(usize::MAX);
        reader.seen = self.published;
        self.older.iter().chain(self.newer.iter()).skip(skip)
    }

    /// Quantos sinais já foram publicados desde sempre. Um leitor novo começa daqui — ele não
    /// recebe a história ([`SignalReader::at`]).
    #[must_use]
    pub fn published(&self) -> u64 {
        self.published
    }

    /// Quantos sinais estão na janela viva (este quadro mais o anterior).
    #[must_use]
    pub fn buffered(&self) -> usize {
        self.older.len() + self.newer.len()
    }
}

/// **O cursor de UM consumidor.** Dois consumidores têm dois destes, e andam sozinhos.
///
/// Um leitor construído com [`SignalReader::new`] começa em zero e recebe tudo o que estiver na
/// janela; um construído com [`SignalReader::at`] começa no presente, que é o que um consumidor
/// ligado no meio da sessão quer (senão ele acorda com dois quadros de história alheia).
#[derive(Debug, Default, Clone, Copy)]
pub struct SignalReader {
    /// O `published` da saída na última leitura deste consumidor.
    seen: u64,
    /// Quantos sinais caíram fora da janela antes de este consumidor os ler.
    missed: u64,
}

impl SignalReader {
    /// Um leitor que começa do início da janela viva.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Um leitor que começa no PRESENTE de `outbox` — nada do que já foi publicado o alcança.
    #[must_use]
    pub fn at(outbox: &SignalOutbox) -> Self {
        Self {
            seen: outbox.published(),
            missed: 0,
        }
    }

    /// Quantos sinais este consumidor perdeu por não ter lido a tempo. Em regime é zero, e um
    /// número diferente de zero é um defeito de fiação — não um detalhe de performance.
    #[must_use]
    pub fn missed(&self) -> u64 {
        self.missed
    }
}

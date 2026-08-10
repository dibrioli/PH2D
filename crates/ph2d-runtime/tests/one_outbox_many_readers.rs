//! **Os gates do modelo:** uma saída, N consumidores, entrega no MESMO quadro, e nada se perde
//! em silêncio.
//!
//! O gate que carrega a wave é o [`a_consumer_reads_the_outbox_while_mutating_its_own_state`]:
//! ele não afirma um número, ele **compila** — e é a coisa exata que o modelo de handlers
//! boxeados do `ph2d-script::messaging` não consegue fazer.

use ph2d_runtime::{Signal, SignalOrigin, SignalOutbox, SignalReader};

/// O quadro do shell, no mínimo: a saída, dois consumidores com estado próprio, e os cursores
/// deles. É a MESMA forma do `App` — os campos vivem lado a lado num struct só.
#[derive(Default)]
struct Host {
    outbox: SignalOutbox,
    toasts: Vec<String>,
    toast_reader: SignalReader,
    log: Vec<String>,
    log_reader: SignalReader,
}

impl Host {
    /// O consumidor 1. ⚠️ **Ele escreve em `self.toasts` DENTRO do laço que lê `self.outbox`.**
    fn drain_toasts(&mut self) {
        for sig in self.outbox.read(&mut self.toast_reader) {
            self.toasts.push(format!("Signal: {}", sig.name));
        }
    }

    /// O consumidor 2, com cursor próprio e estado próprio.
    fn drain_log(&mut self) {
        for sig in self.outbox.read(&mut self.log_reader) {
            self.log.push(match sig.origin {
                SignalOrigin::Timeline { t } => format!("{} @ {t}", sig.name),
                SignalOrigin::Contact { source, other } => {
                    format!("{} {}->{}", sig.name, source.0, other.0)
                }
                SignalOrigin::Control => format!("{} <controle>", sig.name),
                SignalOrigin::Motion { tick, rows } => {
                    format!("{} @tick {tick} x{rows}", sig.name)
                }
            });
        }
    }
}

/// **O gate da wave.** Se isto compila, um subsistema do host pode consumir sinais segurando
/// `&mut` no próprio estado — que é o que a `read(&self, …)` existe para permitir, e o que um
/// `Box<dyn FnMut>` guardado dentro do barramento torna impossível (o consumidor teria de ser
/// emprestado PARA DENTRO de quem despacha).
///
/// ⚠️ Não é um teste de valor: é um teste de EMPRÉSTIMO. Trocar a assinatura para `&mut self`
/// não falha uma asserção — falha o `cargo check`, aqui.
#[test]
fn a_consumer_reads_the_outbox_while_mutating_its_own_state() {
    let mut host = Host::default();
    host.outbox
        .publish(Signal::from_timeline("porta_abriu", 1.5));
    host.drain_toasts();
    assert_eq!(host.toasts, vec!["Signal: porta_abriu".to_owned()]);
}

/// A entrega é no MESMO quadro: publicar e ler, sem virar quadro nenhum, entrega.
#[test]
fn a_signal_published_this_frame_is_read_this_frame() {
    let mut host = Host::default();
    host.outbox.advance_frame();
    host.outbox.publish(Signal::from_timeline("cue", 0.25));
    host.outbox.publish(Signal::from_contact("bateu", 7, 9));
    host.drain_toasts();
    assert_eq!(host.toasts.len(), 2, "os dois sinais do quadro chegaram");
}

/// **AS DUAS FONTES, UM DRENO.** É a wave inteira numa asserção: a timeline e a física publicam
/// na mesma saída, no mesmo quadro, e o consumidor as recebe **na ordem em que foram
/// publicadas** — sem saber que existem duas fontes.
///
/// ⚠️ A ordem importa e não é decorativa: ela é a ordem do QUADRO (a timeline resolve antes de o
/// mundo andar), e é ela que faz um sinal de contato descrever a colisão DESTE quadro em vez da
/// do anterior.
#[test]
fn both_producers_land_in_one_outbox_in_one_frame() {
    let mut host = Host::default();
    host.outbox.advance_frame();
    host.outbox.publish(Signal::from_timeline("footstep", 1.0));
    host.outbox.publish(Signal::from_contact("door", 11, 42));
    host.outbox.publish(Signal::from_timeline("beat", 2.5));
    host.drain_log();
    assert_eq!(
        host.log,
        vec!["footstep @ 1", "door 11->42", "beat @ 2.5"],
        "um consumidor só, as duas fontes, na ordem de publicação"
    );
}

/// **Dois consumidores, dois cursores, e cada um vê tudo UMA vez.** A independência é o produto:
/// o toast não consome o sinal do log, e vice-versa.
#[test]
fn two_consumers_each_see_every_signal_exactly_once() {
    let mut host = Host::default();
    for frame in 0..3u32 {
        host.outbox.advance_frame();
        host.outbox
            .publish(Signal::from_timeline("tick", f64::from(frame)));
        host.drain_toasts();
        host.drain_log();
    }
    assert_eq!(host.toasts.len(), 3, "o toast viu os 3, cada um uma vez");
    assert_eq!(host.log.len(), 3, "o log viu os 3, cada um uma vez");
    assert_eq!(host.log, vec!["tick @ 0", "tick @ 1", "tick @ 2"]);
    assert_eq!(host.toast_reader.missed(), 0);
    assert_eq!(host.log_reader.missed(), 0);
}

/// **A rede do duplo-buffer:** um consumidor que rodou ANTES do produtor recebe no quadro
/// SEGUINTE — nunca *nunca*. É a diferença entre latência e perda silenciosa.
#[test]
fn a_consumer_that_ran_before_the_producer_gets_it_next_frame() {
    let mut host = Host::default();

    // Quadro 1: o consumidor corre PRIMEIRO (nada a ler), e só então o produtor publica.
    host.outbox.advance_frame();
    host.drain_toasts();
    host.outbox.publish(Signal::from_timeline("tarde", 2.0));
    assert!(host.toasts.is_empty(), "ele leu antes: não havia nada");

    // Quadro 2: mesma ordem errada — e o sinal do quadro anterior chega.
    host.outbox.advance_frame();
    host.drain_toasts();
    assert_eq!(
        host.toasts,
        vec!["Signal: tarde".to_owned()],
        "o sinal sobreviveu um quadro: a ordem custa LATÊNCIA, não o evento"
    );
    assert_eq!(host.toast_reader.missed(), 0, "nada foi perdido");
}

/// **Nada se perde em SILÊNCIO.** Um consumidor que fica dois quadros sem ler perde o que caiu
/// fora da janela — e o número aparece, em vez de o sinal simplesmente não acontecer.
#[test]
fn a_consumer_that_sleeps_too_long_reports_what_it_missed() {
    let mut host = Host::default();
    for frame in 0..4u32 {
        host.outbox.advance_frame();
        host.outbox
            .publish(Signal::from_timeline("tick", f64::from(frame)));
        host.drain_log(); // o log acompanha; o toast dorme.
    }
    host.drain_toasts();
    assert_eq!(
        host.toast_reader.missed(),
        2,
        "dos 4, os 2 mais velhos saíram da janela — e ele SABE"
    );
    assert_eq!(host.toasts.len(), 2, "os 2 que ainda estavam na janela");
    assert_eq!(
        host.log_reader.missed(),
        0,
        "quem leu todo quadro não perdeu"
    );
}

/// A janela viva nunca passa de dois quadros — é isso que faz a memória ser função da TAXA de
/// sinais e não do tempo de sessão. (O `clear` + `swap` do `advance_frame` reusa as alocações;
/// o que este gate afirma é a propriedade observável de que ele depende.)
#[test]
fn the_live_window_is_two_frames_and_never_grows() {
    let mut outbox = SignalOutbox::new();
    let mut reader = SignalReader::new();
    for frame in 0..200u32 {
        outbox.advance_frame();
        for _ in 0..3 {
            outbox.publish(Signal::from_timeline("tick", f64::from(frame)));
        }
        let _ = outbox.read(&mut reader).count();
        assert!(
            outbox.buffered() <= 6,
            "quadro {frame}: a janela tem {} sinais — ela deveria ser ESTE quadro mais o anterior",
            outbox.buffered()
        );
    }
    assert_eq!(outbox.published(), 600);
}

/// Um consumidor ligado no meio da sessão começa no PRESENTE — ele não acorda com dois quadros
/// de história alheia que ninguém pediu.
#[test]
fn a_reader_wired_mid_session_starts_at_the_present() {
    let mut outbox = SignalOutbox::new();
    outbox.publish(Signal::from_timeline("antes", 0.0));

    let mut late = SignalReader::at(&outbox);
    assert_eq!(
        outbox.read(&mut late).count(),
        0,
        "a história não o alcança"
    );

    outbox.publish(Signal::from_timeline("depois", 1.0));
    let seen: Vec<_> = outbox.read(&mut late).map(|s| s.name.to_string()).collect();
    assert_eq!(seen, vec!["depois".to_owned()]);
    assert_eq!(late.missed(), 0, "pular a história não é PERDER a história");
}

/// A origem sobrevive inteira: o detalhe que só a fonte sabe é o que um tipo comum estreito
/// demais jogaria fora.
#[test]
fn each_origin_carries_what_only_its_source_knows() {
    let t = Signal::from_timeline("cue", 4.25);
    assert!(t.is("cue"));
    assert!(!t.is("cu"), "casa no nome INTEIRO");
    assert_eq!(t.origin, SignalOrigin::Timeline { t: 4.25 });

    let c = Signal::from_contact("porta_abriu", 11, 42);
    let SignalOrigin::Contact { source, other } = c.origin else {
        panic!("um sinal de contato tem origem de contato");
    };
    assert_eq!((source.0, other.0), (11, 42), "QUEM gritou e para QUEM");
}

/// Clonar um sinal para um segundo consumidor é um refcount, não uma cópia do nome.
#[test]
fn handing_a_signal_to_a_second_consumer_is_a_refcount() {
    let a = Signal::from_timeline("nome_bem_comprido_de_proposito", 0.0);
    let b = a.clone();
    assert!(
        std::sync::Arc::ptr_eq(&a.name, &b.name),
        "o nome é COMPARTILHADO — um segundo consumidor não paga uma alocação por sinal"
    );
}

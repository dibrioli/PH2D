//! Os gates da bancada que tira a exportação da thread que desenha.

use std::sync::atomic::{AtomicBool, Ordering};

/// ⛔⛔ **A BANCADA É ÚNICA NO PROCESSO, E O `cargo test` CORRE EM PARALELO.**
///
/// Sem este cadeado os quatro gates abaixo **roubam a resposta uns aos outros**: o primeiro
/// `take_finished` a chegar leva a mensagem de quem calhar, e as falhas leem-se como defeitos do
/// produto (`left: Some("depois do estouro"), right: Some("uma vez")`). ⚠️ *Uma fixtura que
/// partilha um recurso global com a suíte não mede o produto — mede a ordem em que o escalonador
/// correu os testes.*
///
/// ⚠️ **Envenenado ainda serve**: um gate que falhe a segurar o cadeado não pode trancar os outros
/// três, senão uma falha vira quatro e nenhuma delas aponta para a causa.
fn bench_lock() -> std::sync::MutexGuard<'static, ()> {
    static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());
    ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Espera até a bancada entregar a resposta, ou desiste. ⚠️ **Um `sleep` fixo seria um gate de
/// relógio** — a família de flake que este repo já paga cinco vezes; sondar até um teto grande é a
/// forma que não mede a máquina.
fn wait_for_answer() -> Option<String> {
    for _ in 0..2000 {
        if let Some(m) = super::take_finished() {
            return Some(m);
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    None
}

/// Espera a bancada ficar livre — o `running` cai no `Drop` do sentinela, e o `Drop` corre **depois**
/// de a resposta ser pousada.
fn wait_idle() {
    for _ in 0..2000 {
        if !super::is_running() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// ⭐⭐⭐ **O TRABALHO NÃO CORRE NA THREAD QUE DESENHA** — o report do Enio de 2026-08-25
/// (*"o linux fica cinza"*) escrito como gate.
///
/// ⚠️ **A régua é a IDENTIDADE da thread, não o relógio.** Um gate que medisse *"a chamada volta
/// depressa"* seria um gate de relógio, e passaria verde numa máquina rápida com o trabalho ainda
/// na thread errada. *A pergunta não é quanto demorou: é quem o fez.*
#[test]
fn the_export_does_not_run_on_the_thread_that_draws() {
    let _one = bench_lock();
    wait_idle();
    let _ = super::take_finished();
    let drawing = std::thread::current().id();
    let accepted = super::spawn(move || {
        assert_ne!(
            std::thread::current().id(),
            drawing,
            "a exportação correu na thread que desenha — a janela volta a ficar cinza"
        );
        "feito".to_string()
    });
    assert!(accepted, "a bancada tem de aceitar o trabalho");
    assert_eq!(wait_for_answer().as_deref(), Some("feito"));
    wait_idle();
}

/// ⭐ **A resposta é tirada UMA vez** — a mesma lei das caixas de correio do painel. Uma resposta
/// pousada repetiria o toast em todo quadro seguinte.
#[test]
fn the_answer_is_taken_once() {
    let _one = bench_lock();
    wait_idle();
    let _ = super::take_finished();
    assert!(super::spawn(|| "uma vez".to_string()));
    assert_eq!(wait_for_answer().as_deref(), Some("uma vez"));
    assert_eq!(
        super::take_finished(),
        None,
        "o segundo `take` do mesmo resultado tem de vir vazio"
    );
    wait_idle();
}

/// ⭐⭐ **DUAS EXPORTAÇÕES NÃO CORREM JUNTAS, e a segunda é recusada em ALTO.**
///
/// ⚠️ Duas a escrever o mesmo caminho dariam um arquivo com metade de cada, e a segunda mensagem
/// apagaria a primeira. ⛔ E recusar **em silêncio** seria o defeito que este módulo já pagou: o
/// artista clica, nada acontece, e ele conclui que o botão está partido.
#[test]
fn a_second_export_is_refused_while_the_first_runs() {
    let _one = bench_lock();
    wait_idle();
    let _ = super::take_finished();
    let release = std::sync::Arc::new(AtomicBool::new(false));
    let held = std::sync::Arc::clone(&release);
    assert!(super::spawn(move || {
        while !held.load(Ordering::Acquire) {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        "o primeiro".to_string()
    }));
    // Com um a correr, o segundo é recusado — e a recusa é o valor de retorno, que é o que deixa
    // quem chama dizer alguma coisa ao artista.
    assert!(super::is_running());
    assert!(
        !super::spawn(|| "o segundo".to_string()),
        "a bancada aceitou dois trabalhos ao mesmo tempo"
    );
    release.store(true, Ordering::Release);
    assert_eq!(wait_for_answer().as_deref(), Some("o primeiro"));
    wait_idle();
}

/// ⭐ **UM ESTOURO NÃO DEIXA A BANCADA OCUPADA PARA SEMPRE.**
///
/// ⚠️ Sem o sentinela, o primeiro `panic!` do trabalhador trancava a exportação até ao fim da
/// sessão — e o sintoma que o artista vê é *"o botão parou de funcionar"*, que não aponta para
/// nada. ⚠️ E o `Mutex` fica **envenenado**: a bancada tem de continuar a responder, senão um
/// defeito da exportação vira uma janela morta.
#[test]
fn a_panic_in_the_worker_frees_the_bench() {
    let _one = bench_lock();
    wait_idle();
    let _ = super::take_finished();
    let hush = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    assert!(super::spawn(|| panic!("o trabalhador estourou")));
    wait_idle();
    std::panic::set_hook(hush);
    assert!(
        !super::is_running(),
        "a bancada ficou ocupada depois de um estouro — o botão nunca mais funciona"
    );
    assert!(
        super::spawn(|| "depois do estouro".to_string()),
        "e ela tem de voltar a aceitar trabalho"
    );
    assert_eq!(wait_for_answer().as_deref(), Some("depois do estouro"));
    wait_idle();
}

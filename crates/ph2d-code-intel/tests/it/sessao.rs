//! I2 (`docs/CodeEditor/03_onda2.md`): o quadro nunca espera o servidor; um servidor que morre
//! volta com os documentos; um arranque partido pára e espera um «tentar de novo».

use crate::falso::{Gravado, Plano, ligar};
use ph2d_code_buffer::Text;
use ph2d_code_intel::{Answer, Event, LspSession, Req, ServerState};
use std::time::{Duration, Instant};

const ESPERA: Duration = Duration::from_secs(10);

fn ate(s: &LspSession, mut quer: impl FnMut(&Event) -> bool) -> Event {
    let fim = Instant::now() + ESPERA;
    while Instant::now() < fim {
        if let Some(e) = s.wait(Duration::from_millis(50))
            && quer(&e)
        {
            return e;
        }
    }
    panic!("o evento esperado não chegou em {ESPERA:?}");
}

fn estado(s: &LspSession, quer: ServerState) {
    ate(s, |e| *e == Event::State(quer.clone()));
}

#[test]
fn um_servidor_lento_nunca_faz_o_quadro_esperar() {
    let atraso = Duration::from_secs(1);
    let mut s = LspSession::with_connect(ligar(Plano::lento(atraso), Gravado::default()));
    estado(&s, ServerState::Ready);
    let u = "file:///ph2d-project/a.luau";
    let mut texto = Text::new("local a = 1\n");
    s.open(u, 1, &texto);
    let pedido = s.ask(u, Req::Completion(5));
    // O quadro: 100 teclas (cada uma um `didChange`) e 100 drenos enquanto o servidor dorme.
    let t0 = Instant::now();
    for v in 2..102u64 {
        let antes = texto.len();
        texto = Text::new(&format!("{}x", texto));
        let d = ph2d_code_intel::Dirty {
            old: antes..antes,
            new: antes..antes + 1,
        };
        s.change(u, v, &texto, d);
        assert!(
            !s.drain().iter().any(|e| matches!(e, Event::Answer { .. })),
            "a resposta chegou antes do atraso do servidor — o falso não está lento"
        );
    }
    let gasto = t0.elapsed();
    assert!(
        gasto < atraso / 10,
        "100 teclas + drenos custaram {gasto:?} com o servidor a dormir {atraso:?}: o quadro esperou"
    );
    let Event::Answer {
        ticket,
        answer,
        version,
        ..
    } = ate(&s, |e| matches!(e, Event::Answer { .. }))
    else {
        unreachable!()
    };
    assert_eq!(ticket, pedido);
    assert_eq!(
        version, 1,
        "a resposta tem de dizer a versão a que responde"
    );
    assert!(matches!(answer, Answer::Completion(ref v) if v[0].label == "amplitude"));
}

#[test]
fn um_pedido_novo_do_mesmo_genero_cancela_o_velho() {
    let atraso = Duration::from_millis(300);
    let mut s = LspSession::with_connect(ligar(Plano::lento(atraso), Gravado::default()));
    estado(&s, ServerState::Ready);
    let u = "file:///ph2d-project/a.luau";
    s.open(u, 1, &Text::new("local a = 1\n"));
    let velho = s.ask(u, Req::Completion(1));
    let novo = s.ask(u, Req::Completion(2));
    let Event::Answer { ticket, .. } = ate(&s, |e| matches!(e, Event::Answer { .. })) else {
        unreachable!()
    };
    assert_eq!(ticket, novo, "o pedido velho ({velho:?}) respondeu");
    std::thread::sleep(atraso * 2);
    assert!(
        !s.drain().iter().any(|e| matches!(e, Event::Answer { .. })),
        "o pedido cancelado respondeu mais tarde"
    );
}

#[test]
fn um_servidor_que_morre_depois_de_responder_volta_com_os_documentos() {
    let g = Gravado::default();
    let plano = Plano {
        atraso: Duration::ZERO,
        morre_depois_de: Some(1),
        morre_no_arranque: false,
    };
    let mut s = LspSession::with_connect(ligar(plano, g.clone()));
    estado(&s, ServerState::Ready);
    let u = "file:///ph2d-project/vivo.luau";
    s.open(u, 3, &Text::new("print(1)\n"));
    s.ask(u, Req::Hover(0));
    estado(&s, ServerState::Restarting);
    estado(&s, ServerState::Ready);
    assert_eq!(g.arranques(), 2);
    let fim = Instant::now() + ESPERA;
    while Instant::now() < fim && !g.metodos(1).iter().any(|m| m == "textDocument/didOpen") {
        std::thread::sleep(Duration::from_millis(10));
    }
    let segundo = g.metodos(1);
    assert!(
        segundo.iter().any(|m| m == "textDocument/didOpen"),
        "o documento não foi reaberto no servidor novo: {segundo:?}"
    );
}

#[test]
fn um_arranque_partido_para_e_so_o_retry_tenta_outra_vez() {
    let g = Gravado::default();
    let plano = Plano {
        atraso: Duration::ZERO,
        morre_depois_de: None,
        morre_no_arranque: true,
    };
    let s = LspSession::with_connect(ligar(plano, g.clone()));
    ate(&s, |e| matches!(e, Event::State(ServerState::Failed(_))));
    // Nenhum laço de re-arranques sozinho.
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(
        g.arranques(),
        1,
        "a sessão re-arrancou sozinha um servidor que nunca respondeu"
    );
    s.retry();
    estado(&s, ServerState::Starting);
    ate(&s, |e| matches!(e, Event::State(ServerState::Failed(_))));
    assert_eq!(g.arranques(), 2);
}

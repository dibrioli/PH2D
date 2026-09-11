//! **O puxão do Grab é carimbado UMA vez por QUADRO, não uma por evento.**
//!
//! A lei que autoriza isto é do motor e tem gate próprio lá
//! (`a_hold_gesture_is_the_same_clay_however_finely_the_pointer_was_sampled`):
//! um `Grip::Hold` é **frozen**, então o puxão TOTAL é a única entrada, e
//! dezesseis eventos pousam onde um pousa, ao bit. O que ela COMPRA — 17,9 ms
//! por quadro virarem 1,2 — mora inteiro na FIAÇÃO, e nenhum teste de unidade a
//! alcança: quem decide é o arquivo de input do shell.
//!
//! ⚠️ **São TRÊS afirmações, e cada uma tem um modo de falha próprio:**
//!
//! 1. o evento de ponteiro **regista** em vez de carimbar — se alguém devolver
//!    o `grab_at` para lá, a coalescência evapora e nada fica vermelho;
//! 2. o pen-up **drena ANTES** de fechar o traço — sem isso a última fatia do
//!    gesto evapora, um erro que **cresce com a velocidade da mão** e some
//!    quando ela é lenta (a forma mais cara de um bug se esconder);
//! 3. o quadro **drena** — sem isso o barro só anda quando o dedo levanta.
//!
//! ⚠️ E os três leem o FONTE com **controle positivo**: uma âncora que deixa de
//! existir vira falha alta, nunca varredura vazia que passa por vácuo.

use std::fs;

fn read(rel: &str) -> String {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/src/");
    fs::read_to_string(format!("{p}{rel}"))
        .unwrap_or_else(|e| panic!("o dono se mudou? {rel}: {e}"))
}

#[test]
fn the_pointer_event_registers_the_pull_instead_of_stamping_it() {
    let src = read("sculpt3d_input.rs");
    let arm = src
        .find("Grip::Hold =>")
        .expect("o braço do Grip::Hold saiu do despachante de arrasto");
    // A janela do braço: até o próximo braço do `match`.
    let rest = &src[arm..];
    let end = rest[1..].find("Grip::").map_or(rest.len(), |i| i + 1);
    let body = &rest[..end];

    assert!(
        body.contains("pending_grab = Some"),
        "o braço do Hold tem de REGISTAR o puxão; ele diz: {body}"
    );
    assert!(
        !body.contains("grab_at("),
        "o braço do Hold voltou a CARIMBAR por evento — a coalescência morreu \
         em silêncio, e o custo é ~16 dabs por quadro: {body}"
    );
}

#[test]
fn the_pen_up_drains_the_pending_pull_before_it_closes_the_stroke() {
    let src = read("sculpt3d_input.rs");
    // ⚠️ **A janela é o CORPO do pen-up, e a 1ª versão deste gate anchorava no
    // ARQUIVO — a mutação sobreviveu por isso.** O dreno de QUADRO
    // (`sculpt3d_flush_grab`) chama a mesma função ~170 linhas acima, então
    // `find` sobre o arquivo devolvia sempre aquela e a comparação era
    // verdadeira por construção: *um gate que não pode falhar pelo motivo que
    // alega*.
    let start = src
        .find("fn sculpt3d_pointer_up")
        .expect("o dono do pen-up se mudou");
    let body = &src[start..];
    let flush = body
        .find("flush_pending_grab()")
        .expect("o pen-up não drena o puxão pendente — a ponta do gesto evapora");
    let close = body
        .find("close_stroke()")
        .expect("o dono do fecho do traço se mudou");
    assert!(
        flush < close,
        "o dreno tem de vir ANTES do fecho: depois dele o último movimento do \
         dedo é carimbado num traço que já acabou (ou perdido)"
    );
}

#[test]
fn the_frame_drains_the_pending_pull() {
    let src = read("render_loop/mod.rs");
    assert!(
        src.contains("sculpt3d_flush_grab()"),
        "o laço de quadro não drena o puxão — com o evento apenas registando, o \
         barro só andaria no pen-up"
    );
}

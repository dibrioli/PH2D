//! ⭐ **A PORTA DOS DIÁLOGOS MODAIS — e o relógio que eles congelam.**
//!
//! # O defeito, com as palavras do Enio
//!
//! *"não vejo em nenhum lugar a mensagem"* (2026-08-22), sobre o toast que a exportação escreve
//! logo depois de o diálogo de arquivo fechar. Ele estava a ser escrito, e estava a ser pintado —
//! **num quadro só**, e depois morria.
//!
//! # O mecanismo, e ele é de RELÓGIO
//!
//! ```text
//! quadro N:  wall_dt = agora − início do quadro anterior      (a linha 1088 do render_loop)
//!            toasts.tick(wall_dt)                              ← envelhece o que já existia
//!            …
//!            [ DIÁLOGO MODAL ABERTO — o loop CONGELA 20 s ]
//!            toast criado, idade 0                             ← pintado 1× no fim deste quadro
//! quadro N+1: wall_dt ≈ 20 s   →   toasts.tick(20 s)          ⛔ idade 20 s > TTL 3 s → MORRE
//! ```
//!
//! ⚠️ **O `wall_dt` mede o quadro INTEIRO**, e o diálogo aconteceu *dentro* dele. A mensagem que
//! devia durar 3 segundos vive **16 ms** — um quadro. Da cadeira, isso é *não aparecer*.
//!
//! # ⭐ Um número a responder DUAS perguntas
//!
//! O `render_loop` unificou o relógio de propósito, e a nota dele (linha 1082) está certa sobre o
//! caso que curou: o `ToastQueue` contava **quadros**, e um toast de "3 s" durava 6 s a 30 fps.
//! Mas o número unificado responde a duas perguntas que **divergem quando o loop congela**:
//!
//! | pergunta | quem lê | o congelamento conta? |
//! |---|---|---|
//! | *quanto durou o último quadro?* | o medidor de fps, o acumulador da sim | **sim** — foi mesmo esse tempo |
//! | *quanto a UI ANIMOU?* | os toasts, o `hero.tick_motion` | ⛔ **não** — nada se moveu, a tela estava parada |
//!
//! A cura não é um segundo relógio nem um teto mágico: é **nomear a parte congelada**. Quem congela
//! o loop declara quanto ([`note_stall`]), e o relógio do chrome desconta isso ([`chrome_dt`]) —
//! uma medição, com a parte parada marcada.
//!
//! # ⚠️ A porta, e por que ela é uma porta
//!
//! Um `rfd::FileDialog` que alguém abra à mão volta a congelar sem declarar. Por isso o diálogo
//! passa por [`save_file`] / [`pick_file`], que **medem a própria duração**. Gate:
//! `every_field3d_modal_goes_through_the_door`.
//!
//! ⛔ **MEDIDO 2026-08-22: há 25 chamadas de `rfd::FileDialog` em 12 arquivos do shell**, e as
//! outras 23 continuam a perder a mensagem que escrevem a seguir. Elas são de outras linhas
//! (`sculpt3d`, image tools, tokens, sheet, texto vetorial) e a lista está no doc §38 — *o defeito
//! é da casa, e nomeá-lo com o endereço é o que impede a próxima linha de o redescobrir.*

use std::path::PathBuf;
use std::time::{Duration, Instant};

thread_local! {
    /// Quanto tempo o loop passou **congelado** neste quadro, em segundos.
    ///
    /// ⚠️ Acumula (um quadro pode abrir mais de um diálogo) e é **tirado** uma vez por quadro.
    static STALLED_S: std::cell::Cell<f64> = const { std::cell::Cell::new(0.0) };
}

/// **Declara que o loop ficou congelado por `d`** — o tempo em que nada foi desenhado.
pub(crate) fn note_stall(d: Duration) {
    STALLED_S.with(|s| s.set(s.get() + d.as_secs_f64()));
}

/// **Tira o congelamento acumulado**, zerando-o. Chamada uma vez por quadro, pelo dono do relógio.
pub(crate) fn take_stall() -> f64 {
    STALLED_S.with(|s| s.replace(0.0))
}

/// ⭐ **Quanto a UI de facto ANIMOU neste quadro.**
///
/// ⚠️ Função pura, e é ela que carrega a decisão inteira — por isso é gateável sem janela. Nunca
/// negativa: um congelamento maior do que o quadro é impossível, mas medir com dois relógios
/// diferentes pode produzi-lo, e um `dt` negativo faria o toast **rejuvenescer**.
pub(crate) fn chrome_dt(wall_dt: f64, stalled_s: f64) -> f64 {
    (wall_dt - stalled_s).max(0.0)
}

/// ⭐ **Corre `f` e DECLARA o que ela congelou** — a lei da porta, sem o `rfd` no caminho.
///
/// ⚠️ **Ela é separada por causa de uma prova de mutação.** Enquanto o cronómetro vivia dentro de
/// [`save_file`], tirá-lo de lá deixava tudo **verde**: os gates chamavam `note_stall` à mão, e a
/// porta — a única coisa que de facto liga o diálogo ao relógio — não era exercida por nenhum. Um
/// `rfd::FileDialog` não abre num teste, mas *"o que passa por aqui é cronometrado"* abre. ⭐ **O
/// que sobra sem gate é uma linha por porta**, e ela não tem lógica nenhuma.
///
/// ⭐⭐ **E o congelamento não é privilégio do DIÁLOGO** (2026-08-25, report do Enio: *"a mensagem
/// não aparece"* sobre uma exportação de um milhão de faces). A lei desta porta sempre foi *quem
/// congela o loop declara quanto* — e um diálogo nativo e uma conta de minutos congelam do mesmo
/// jeito. *O relógio do chrome não distingue «parado por um diálogo» de «parado por uma conta»:
/// para a tela, os dois são o mesmo nada.*
///
/// ⛔⛔ **E DECLARAR CURA A MENSAGEM, NÃO CURA O CONGELAMENTO.** O report seguinte do Enio, no mesmo
/// dia, foi *"o linux fica cinza"*: com a conta a 12 s o loop não responde ao *ping* do compositor,
/// o KDE pinta a janela de cinza e oferece *"forçar o encerramento"* — o sistema operativo a dizer
/// ao artista que o programa morreu, com o trabalho não gravado dentro.
///
/// ⇒ ⭐ **Para um CÁLCULO a resposta certa não é declarar: é não bloquear.** O módulo 3D tirou a
/// exportação da thread que desenha ([`crate::field3d_export_job`]), e com isso não há
/// congelamento nenhum a declarar. *Um diálogo é uma janela que o compositor sabe que abriu; uma
/// conta é o programa a não responder.* Esta porta continua a ser a resposta certa **para o
/// diálogo**, que é o que ela sempre foi.
fn timed<T>(f: impl FnOnce() -> T) -> T {
    let t0 = Instant::now();
    let out = f();
    note_stall(t0.elapsed());
    out
}

/// A porta de **gravar**: abre o diálogo e declara o tempo em que ele congelou o loop.
pub(crate) fn save_file(dialog: rfd::FileDialog) -> Option<PathBuf> {
    timed(|| dialog.save_file())
}

/// A porta de **abrir**, pela mesma razão.
pub(crate) fn pick_file(dialog: rfd::FileDialog) -> Option<PathBuf> {
    timed(|| dialog.pick_file())
}

#[cfg(test)]
#[path = "modal_tests.rs"]
mod tests;

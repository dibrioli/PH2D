//! **A LEGENDA DE UMA CENA DE SMOKE, NO CANVAS** — o rótulo pousa em cima da coisa que ele
//! explica, e não num terminal atrás da janela.
//!
//! Pedido do Enio (2026-08-23): *"melhore as explicações do smoke"*.
//!
//! ## Por que a explicação estava no sítio errado
//!
//! Uma cena de conferência é uma GRELHA de casos — três, seis, oito linhas, metade esquerda
//! contra metade direita — e até aqui o que dizia qual era qual era um bloco de `eprintln!` que
//! sai **antes de a janela abrir**. O Enio corre o comando, a janela cobre o terminal, e a partir
//! daí ele tem de contar linhas de cima para baixo e casá-las de memória com um texto que já não
//! está à vista. *A explicação existia; o que faltava era ela estar onde os olhos estão.*
//!
//! ⚠️ **O anúncio no terminal FICA**, e não é redundância: ele é o único sítio onde cabe o *"deu
//! errado se…"*, que é uma frase e não um rótulo. A ficha diz **o que é aquilo**; o terminal diz
//! **como saber que falhou**. Duas perguntas, dois sítios.
//!
//! ## A costura: uma função PURA por cena, e um só salto global
//!
//! Cada cena expõe uma `captions()` **pura** — âncora de mundo + texto —, e é essa função que os
//! gates medem. O roteador publica-a aqui, e o passe de pintura lê-a.
//!
//! ⚠️ **O global é um SALTO, não um estado.** Ele existe porque `build_level` é uma função livre
//! (documento + registry, sem `App`) chamada de dentro do construtor do `MotionState`: devolver
//! as legendas obrigaria a mudar a assinatura dos ~80 braços do `match`. Ele é escrito uma vez, na
//! construção da cena, e nunca mais. ⛔ **Nenhum gate lê daqui** — eles chamam a `captions()` da
//! cena, senão dois testes em paralelo disputariam a mesma célula.

use std::sync::Mutex;

/// Um rótulo e o ponto de MUNDO sobre o qual ele pousa.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Caption {
    /// Onde, em coordenadas de mundo — o passe de pintura leva-o à tela pelo afim da câmara, e a
    /// ficha é desenhada em pixels de TELA (ela é uma legenda, não uma medida: tem de continuar
    /// legível com o canvas afastado).
    pub(crate) world: [f32; 2],
    /// O que se lê. **Curto** — é uma ficha, não um parágrafo; a frase longa vive no terminal.
    pub(crate) text: String,
}

impl Caption {
    pub(crate) fn new(world: [f32; 2], text: impl Into<String>) -> Self {
        Self {
            world,
            text: text.into(),
        }
    }
}

static LEGEND: Mutex<Vec<Caption>> = Mutex::new(Vec::new());

/// ⭐⭐ **A TRAVA DE QUEM VARRE CENAS** — o [`LEGEND`] é **global ao processo**, e uma cena só o
/// publica ao ser montada.
///
/// ⛔⛔ **Dois testes que montem cenas em paralelo leem a legenda um do outro**, e o sintoma é
/// uma reprova que **passa sozinha** — indistinguível de uma flake de carga, e arquivada como
/// tal ([memória](../../project-memory/feedback_a_shared_tmp_fixture_race_is_misfiled_as_a_load_flake.md)).
/// Medido em 2026-09-09: o gate do tecto do roteador passou a montar nove cenas para perguntar
/// ao COMPORTAMENTO, e o censo das legendas — que monta ~30 — começou a reprovar na suíte
/// inteira e a passar isolado.
///
/// ⇒ **todo teste que monte uma cena toma esta trava.** Envenenada não interessa: o estado que
/// ela protege é reescrito por inteiro na montagem seguinte.
#[cfg(test)]
fn trava() -> std::sync::MutexGuard<'static, ()> {
    static M: Mutex<()> = Mutex::new(());
    M.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// ⭐⭐⭐ **A PORTA ÚNICA por onde um TESTE monta uma cena** — devolve os sinks **e** a legenda
/// que aquela montagem publicou.
///
/// ⛔⛔ **Uma trava só exclui quem a TOMA**, e foi isso que a 1.ª tentativa não viu: pôr o
/// cadeado no leitor deixava os outros ~13 sítios que chamam o `build_level` a publicar por cima
/// dele. ⇒ **a trava mora aqui, e quem monta uma cena num teste passa por aqui.**
///
/// ⚠️ **Ela é segurada só durante a MONTAGEM**, nunca durante o cozimento: a versão que a tomava
/// à volta de uma varredura de 112 níveis levou a suíte do shell de **72 s para 1 796 s** —
/// *uma trava que protege mais do que o estado partilhado paga o preço de toda a gente*.
///
/// ⚠️ **E ela LIMPA antes de montar:** o global só é reescrito por uma cena que **publique**, e
/// sem a limpeza uma cena muda herdava a legenda da anterior — um falso positivo que já vivia no
/// censo antes desta wave.
#[cfg(test)]
pub(crate) fn monta(
    level: &str,
    doc: &mut ph2d_motion_doc::MotionDoc,
    reg: &ph2d_node_registry::NodeRegistry,
) -> (Vec<ph2d_nodegraph::graph::NodeId>, Vec<Caption>) {
    let _t = trava();
    publish(Vec::new());
    let sinks = crate::motion::motion_state::demo_router::build_level(Some(level), doc, reg);
    (sinks, captions())
}

/// A cena publica a legenda dela. Substitui, nunca acumula.
pub(crate) fn publish(captions: Vec<Caption>) {
    if let Ok(mut slot) = LEGEND.lock() {
        *slot = captions;
    }
}

/// O que o passe de pintura desenha neste quadro (vazio quando nenhuma cena publicou).
pub(crate) fn captions() -> Vec<Caption> {
    LEGEND.lock().map(|s| s.clone()).unwrap_or_default()
}

#[cfg(test)]
#[path = "motion_demo_legend_tests.rs"]
mod tests;

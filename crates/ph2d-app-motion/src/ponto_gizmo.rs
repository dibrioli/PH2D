//! ⭐⭐⭐ **O GIZMO DE UMA CORRENTE DE POSIÇÕES** — a segunda metade da ordem do dono
//! (2026-09-17, reaberta em 19/09 como report): *«nós como Grid, rope, etc, não passam de posições
//! do espaço, sem nenhuma capacidade de gerar pixels na tela»*, e ***«sem o duplicator só aparece
//! um gizmo de osso ou segmento de corda (ou outro tipo de segmento) que não renderiza em
//! runtime»***.
//!
//! A [`ph2d_eval_motion::tem_aparencia`] (a W1) cala o que não veio de uma forma. Este
//! ficheiro é o que aparece no lugar: um **manipulador de editor**, desenhado só com a ferramenta
//! Motion na mão e **nunca** no quadro que o produto entrega.
//!
//! ## A FEIÇÃO sai das COLUNAS, nunca de uma lista de nomes de nó
//!
//! | a corrente traz | é | porque |
//! |---|---|---|
//! | `parent` | um **OSSO** | cada elemento declara de quem PENDE — é a cadeia do `rig.*` |
//! | `rope_prev` | uma **CORDA** | o estado de Verlet; os pontos são consecutivos por construção |
//! | nada disso | **PONTOS** | uma nuvem sem ordem (grelha, dispersão, distribuições) |
//!
//! ⚠️ **Derivada, e é a diferença entre isto e uma tabela que envelhece:** um `rig.fabrik` novo, um
//! `motion.verlet_rope` com outro nome, um nó de terceiros — todos caem na feição certa sem
//! ninguém se lembrar de os inscrever. Uma lista de `type_name` ficaria muda no primeiro nó novo,
//! que é como o censo por prefixo desta casa já falhou (CLAUDE.md §5.0).
//!
//! ⚠️ **O `parent` ganha do `rope_prev`** quando os dois existem: pender de alguém é uma afirmação
//! mais forte do que ser consecutivo, e uma cadeia com ramos desenhada como corda ligaria pontos
//! que não se tocam.
//!
//! ## O que ele NÃO faz
//!
//! ⛔ Não tem alças e não edita nada. O gizmo do colisor e o do warp arrastam params; este só diz
//! *onde as posições estão*. Dar-lhe alças seria autorar a saída de um nó pela ponta, que é o que
//! o cartão faz.

use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::graph::NodeId;

/// **Quantos elementos o gizmo desenha, no máximo, por corrente.**
///
/// ⚠️ **O RECURSO é o tempo de CODIFICAR os caminhos no quadro** (o mesmo do `MAX_CONTORNOS` do
/// gizmo do colisor), contra o orçamento de **`1,67 ms`** = 1/10 de um quadro de 60 Hz — a régua
/// que o indicador da pose já usa: *um gizmo é passageiro do quadro, não o assunto dele*.
///
/// **MEDIDO** (`mede_o_custo_do_gizmo_de_pontos`, `--release`, mediana de 9, `load 3,09`):
///
/// | elementos | Ponto | Corda | Osso | pior, em % do orçamento |
/// |---|---|---|---|---|
/// | 1 024 | 0,057 ms | 0,098 | 0,099 | **5,9 %** |
/// | **4 096** | 0,228 ms | 0,403 | 0,390 | **24,2 %** |
/// | 16 384 | 1,375 ms | 1,604 | 1,562 | **96,1 %** |
/// | 65 536 | 3,571 ms | 6,811 | 6,345 | 407,8 % |
///
/// ⇒ `4 096` é o maior degrau em que a **pior** feição fica abaixo de um quarto do orçamento; a
/// `16 384` ele está gasto. ⛔ **Não é um «razoável»**: a cena `=116` entrega **102 400** posições
/// num sink só, e desenhá-las custaria mais de quatro quadros inteiros de gizmo.
///
/// ⚠️ **O corte não apaga a contagem** — [`Grupo::total`] guarda quantas havia, para quem quiser
/// dizê-lo ao artista.
pub const MAX_PONTOS: usize = 4096;

/// O que uma corrente de posições é, à vista.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feicao {
    /// Uma cadeia: cada elemento pende do `parent` dele.
    Osso,
    /// Uma corda: os elementos são consecutivos.
    Corda,
    /// Uma nuvem sem ordem.
    Ponto,
}

/// A feição de uma corrente, lida das colunas dela. Ver a tabela do cabeçalho.
#[must_use]
pub fn feicao_de(s: &Stream) -> Feicao {
    if s.get("parent").is_some() {
        Feicao::Osso
    } else if s.get("rope_prev").is_some() {
        Feicao::Corda
    } else {
        Feicao::Ponto
    }
}

/// Uma corrente de posições, pronta a desenhar.
#[derive(Clone, Debug, PartialEq)]
pub struct Grupo {
    /// O sink de que esta corrente saiu — o que o artista vê seleccionado no grafo.
    pub node: NodeId,
    pub feicao: Feicao,
    /// As posições de MUNDO, já limitadas pelo [`MAX_PONTOS`].
    pub pontos: Vec<[f32; 2]>,
    /// Os pares `(de, para)` como índices em [`Self::pontos`]. Vazio numa nuvem.
    pub segmentos: Vec<[usize; 2]>,
    /// Quantas posições a corrente tinha ANTES do tecto — o que a legenda diria.
    pub total: usize,
}

/// O retrato deste quadro.
#[derive(Clone, Debug, PartialEq)]
pub struct PontoGizmoView {
    pub grupos: Vec<Grupo>,
}

/// As posições de uma corrente, limitadas pelo tecto.
fn posicoes(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.iter().take(MAX_PONTOS).copied().collect(),
        _ => Vec::new(),
    }
}

/// Os segmentos de uma CADEIA — cada elemento liga-se ao `parent` dele.
///
/// ⚠️ **Um `parent` fora de alcance é SALTADO, não coagido.** A raiz declara-se com `-1` (a
/// convenção do `rig.skeleton`), e coagir um índice inválido para `0` desenharia um osso da raiz
/// até ao elemento — uma linha que o artista não autorou, a partir de uma corrente que o tecto
/// cortou a meio.
fn ossos(s: &Stream, n: usize) -> Vec<[usize; 2]> {
    let Some(Column::Scalar(pais)) = s.get("parent") else {
        return Vec::new();
    };
    (0..n)
        .filter_map(|i| {
            let p = *pais.get(i)?;
            // `as usize` sobre um negativo satura em `0` desde a 1.45 do Rust — a comparação
            // com `0.0` vem ANTES de propósito, senão toda raiz viraria um osso até si mesma.
            if p < 0.0 {
                return None;
            }
            let pi = p as usize;
            (pi < n && pi != i).then_some([pi, i])
        })
        .collect()
}

/// Os segmentos de uma CORDA — os consecutivos.
fn corda(n: usize) -> Vec<[usize; 2]> {
    (1..n).map(|i| [i - 1, i]).collect()
}

/// **AS TOMADAS que este gizmo precisa: TODOS os sinks.**
///
/// ⚠️ **Não dá para escolher só os que não têm aparência** — essa pergunta é sobre a CORRENTE, e a
/// corrente só existe depois do cozimento. Pedir todos é a resposta honesta.
///
/// ⭐ **E é barato, com o mecanismo:** desde o doc 115 §21 quem DESENHA publica o que separou, logo
/// uma tomada sobre um sink **não re-coze nada**; o que ela custa é um `Stream::clone`, e um
/// `Stream` guarda `Arc<Column>` ⇒ refcount, nunca uma cópia das colunas. ⛔ Foi exactamente o
/// oposto disto que custou metade de um quadro ao dono quando a tomada do colisor obrigava a uma
/// segunda separação.
#[must_use]
pub fn taps_for(motion: &MotionState) -> Vec<NodeId> {
    motion.sinks.clone()
}

/// A corrente que um nó entregou neste cozimento.
fn tap(motion: &MotionState, node: NodeId) -> Option<&Stream> {
    motion
        .pump
        .tap_streams()
        .iter()
        .find(|(n, _)| *n == node)
        .map(|(_, s)| s)
}

/// ⭐⭐⭐ **O RETRATO**: cada sink cuja corrente **não traz aparência** vira um grupo.
///
/// ⚠️ **A pergunta é a MESMA porta que o lowering usa** ([`ph2d_eval_motion::tem_aparencia`])
/// — um segundo predicado aqui divergiria no dia em que uma origem nova nascesse, e o artista
/// veria o gizmo E os pixels, ou nenhum dos dois.
///
/// `None` sem a ferramenta Motion na mão: um gizmo é de EDITOR, e o quadro que o produto entrega
/// não o tem.
#[must_use]
pub fn resolve(motion: &MotionState, tool_is_motion: bool) -> Option<PontoGizmoView> {
    if !tool_is_motion {
        return None;
    }
    let mut grupos = Vec::new();
    for &node in &motion.sinks {
        let Some(s) = tap(motion, node) else {
            continue;
        };
        if ph2d_eval_motion::tem_aparencia(s) {
            continue;
        }
        let total = s.count();
        if total == 0 {
            continue;
        }
        let pontos = posicoes(s);
        if pontos.is_empty() {
            continue;
        }
        let feicao = feicao_de(s);
        let n = pontos.len();
        let segmentos = match feicao {
            Feicao::Osso => ossos(s, n),
            Feicao::Corda => corda(n),
            Feicao::Ponto => Vec::new(),
        };
        grupos.push(Grupo {
            node,
            feicao,
            pontos,
            segmentos,
            total,
        });
    }
    (!grupos.is_empty()).then_some(PontoGizmoView { grupos })
}

static VIEW: std::sync::Mutex<Option<PontoGizmoView>> = std::sync::Mutex::new(None);

/// Publica (ou limpa) o retrato deste quadro. ⚠️ Publicar de novo SUBSTITUI — largar a ferramenta
/// Motion limpa o gizmo em vez de o deixar a pairar.
pub fn publish(v: Option<PontoGizmoView>) {
    if let Ok(mut slot) = VIEW.lock() {
        *slot = v;
    }
}

/// O retrato deste quadro, se houver.
#[must_use]
pub fn view() -> Option<PontoGizmoView> {
    VIEW.lock().ok().and_then(|s| s.clone())
}

#[cfg(test)]
#[path = "ponto_gizmo_tests.rs"]
mod tests;

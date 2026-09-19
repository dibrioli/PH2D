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
    /// ⭐⭐⭐ **A ROTAÇÃO de cada elemento, em graus** — `None` quando a corrente **não traz** a
    /// coluna (ordem do dono, 2026-09-19: *«no canvas simulam qualquer grafo normalmente»*).
    ///
    /// ⚠️ **`None` e `Some(vec![0; n])` NÃO são a mesma coisa, e a diferença é visível:** sem a
    /// coluna o gizmo não desenha a agulha da direcção — *uma agulha a apontar para a direita em
    /// toda a nuvem seria ruído sobre um grafo que nunca falou de direcção*. Com a coluna toda a
    /// zero, ela aponta para a direita **porque o grafo o disse**.
    pub rot: Option<Vec<f32>>,
    /// ⭐⭐⭐ **A ESCALA de cada elemento** — o MULTIPLICADOR do glifo, nunca uma medida de mundo.
    ///
    /// É isto que faz o `scale` de um `motion.oscillator` PULSAR no canvas sem uma forma ligada.
    /// `None` ⇒ [`ph2d_nodegraph::attr::SIZE_IDENTITY`], que é o glifo nu.
    ///
    /// ⚠️ **Um escalar e não um `[f32; 2]`:** o glifo é chrome e tem de continuar a ler-se como o
    /// mesmo símbolo — uma cruz esmagada num eixo lê-se como uma barra, e o artista deixaria de
    /// saber que aquilo é um ponto. A média dos dois eixos é o que a `SIZE_IDENTITY` torna `1`.
    pub escala: Option<Vec<f32>>,
    /// Quantas posições a corrente tinha ANTES do tecto — o que a legenda diria.
    pub total: usize,
}

impl Grupo {
    /// O multiplicador do glifo no elemento `i` — `1` quando a corrente não autorou escala.
    #[must_use]
    pub fn escala_em(&self, i: usize) -> f32 {
        self.escala
            .as_ref()
            .and_then(|v| v.get(i))
            .copied()
            .unwrap_or(1.0)
    }

    /// A rotação do elemento `i`, em **graus**, ou `None` se a corrente não traz direcção.
    #[must_use]
    pub fn rot_em(&self, i: usize) -> Option<f32> {
        self.rot.as_ref().and_then(|v| v.get(i)).copied()
    }
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

/// **A ESCALA de cada elemento, como MULTIPLICADOR do glifo** — `None` quando a corrente não a
/// autorou (ordem do dono, 2026-09-19).
///
/// ⚠️ **A identidade é [`SIZE_IDENTITY`], que é `1`**, e é por isso que a coluna se lê como um
/// multiplicador directo: um `motion.scale(amount = 0,4)` entrega `0,4`, e o glifo fica a 40 % —
/// *o gizmo mostra o que o grafo fez, e não o que o desenhador dele achou bonito*.
///
/// ⚠️ **Uma coluna `Vec2` colapsa na MÉDIA dos eixos** — ver [`Grupo::escala`] para a razão.
pub(crate) fn escalas(s: &Stream, n: usize) -> Option<Vec<f32>> {
    match s.get("size") {
        Some(Column::Scalar(v)) => Some(v.iter().take(n).copied().collect()),
        Some(Column::Vec2(v)) => Some(v.iter().take(n).map(|e| (e[0] + e[1]) * 0.5).collect()),
        _ => None,
    }
}

/// **A ROTAÇÃO de cada elemento, em graus** — `None` quando a corrente não a traz.
pub(crate) fn rotacoes(s: &Stream, n: usize) -> Option<Vec<f32>> {
    match s.get("rot") {
        Some(Column::Scalar(v)) => Some(v.iter().take(n).copied().collect()),
        _ => None,
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
/// ⛔ **E ELA É VAZIA COM A LEI DESLIGADA, que é a configuração de FÁBRICA:** *um gizmo que não é
/// desenhado não pode cobrar o cozimento de que ele precisaria.* Na rota do DISPOSITIVO a bomba
/// **não marcha**, logo uma tomada obriga o `cook_taps_only` a cozinhar aquele sink **na CPU**.
///
/// ⚠️⚠️ **E o PREÇO disso foi MEDIDO, depois de eu o ter invocado errado.** Eu justifiquei esta
/// cerca com os `195,9 ms` que a auditoria de performance do módulo mede para a CPU a `4,19 M`
/// objectos — *um número de outro regime*. Medido no sítio (`o_que_a_tomada_do_gizmo_custa`,
/// `--release`, mediana de 5, `load 6,03`):
///
/// | cena | linhas | o cozimento do sink | de um quadro |
/// |---|---|---|---|
/// | `=111` | 12 800 | `0,226 ms` | `1,4 %` |
/// | `=116` | **102 400** | **`0,216 ms`** | **`1,3 %`** |
/// | `=120` | 20 | `0,001 ms` | `0,0 %` |
///
/// ⇒ **a cerca fica pela lei e não pelo relógio** (zero custo na configuração de fábrica é a mesma
/// lei do interruptor único), e o gizmo é utilizável mesmo nas cenas grandes. *Invocar um número
/// medido noutro regime para justificar uma decisão é o que o §0.0 proíbe, e eu fi-lo aqui.*
#[must_use]
pub fn taps_for(motion: &MotionState, so_com_forma: bool) -> Vec<NodeId> {
    if !so_com_forma {
        return Vec::new();
    }
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
///
/// ⛔⛔⛔ **E `None` com a LEI DESLIGADA** — report do dono, 2026-09-19: *«os retângulos voltaram e
/// os gizmos estão relativos ao zoom»*. Com a lei desligada as peças **desenham-se**, e o gizmo por
/// cima delas é ruído sobre arte correcta; pior, ele aparecia na configuração de FÁBRICA, que é a
/// lei desta casa violada (*tudo o que é novo shipa desligado*). ⭐ **Os dois lados são o mesmo
/// interruptor: ou se vêem as peças, ou se vê o gizmo.**
///
/// ⚠️ **A lei entra como ARGUMENTO e não é lida do ambiente aqui** — é a lição da auditoria do
/// §31: um gate constrói a resposta à mão e mede a LEI; quem lê o ambiente é a porta do produto,
/// num sítio só.
#[must_use]
pub fn resolve(
    motion: &MotionState,
    tool_is_motion: bool,
    so_com_forma: bool,
) -> Option<PontoGizmoView> {
    if !tool_is_motion || !so_com_forma {
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
            rot: rotacoes(s, n),
            escala: escalas(s, n),
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

/// ⭐⭐⭐ **A ARTE DO DISPOSITIVO DESENHA NESTE QUADRO?** — a metade da lei que o caminho da GPU
/// devia ter e não tinha.
///
/// ⛔⛔⛔ **Report do dono, 2026-09-19: *«os retângulos voltaram»*.** A [W1] escreveu a saída cedo
/// nos **dois lowerings de CPU** e o [doc 115 §32.2] declarou, por escrito, que *«por corrente o
/// device apenas não despacha»* — **uma propriedade que ninguém construiu**. Medido:
/// `grep -c so_com_forma crates/ph2d-gpu-cook/src` devolve **`0`**. E a cena que eu próprio lhe
/// apontei — a `=116`, *«102 400 peças no dispositivo»* — é exactamente uma cena de device.
/// *Escrever a propriedade no doc não a constrói; foi preciso o dono abrir o app para a cobrar.*
///
/// ## Como a pergunta se responde SEM ler o dispositivo de volta
///
/// No caminho da GPU a aparência só pode chegar por uma **FRONTEIRA** — o `source.object` lê um
/// external que a membrana publica na CPU, e é a fronteira que viaja para o device. Logo:
///
/// > *a arte do device tem aparência* ⟺ *alguma corrente de fronteira tem aparência*
///
/// ⚠️ E a pergunta é a MESMA porta que o lowering usa ([`ph2d_eval_motion::tem_aparencia`]) — um
/// segundo predicado aqui divergiria no dia em que uma origem nova nascesse.
///
/// ⛔ **A partição de texturas NÃO serve para isto**, e a razão está escrita no doc dela: ela
/// também fica vazia num *«grafo de objectos cujos ladrilhos vivem todos no atlas partilhado»* —
/// usá-la apagaria uma cena de objectos legítima.
#[must_use]
pub fn a_arte_desenha(motion: &MotionState, so_com_forma: bool) -> bool {
    if !so_com_forma {
        return true;
    }
    motion
        .pump
        .boundary_streams()
        .iter()
        .any(|(_, s)| ph2d_eval_motion::tem_aparencia(s))
}

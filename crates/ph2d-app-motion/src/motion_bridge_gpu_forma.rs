//! ⭐⭐⭐ **AS FORMAS VIVAS NA ROTA DA PLACA** (doc 121 W3) — a cerca que decide se um grafo com
//! formas vivas coze no dispositivo, perguntando pelo CONTEÚDO do que foi publicado.
//!
//! ⛔⛔ **Ela substitui DUAS cercas que recusavam SEMPRE**, e a morte delas é o ponto desta wave:
//! o `graph_has_live_vector_source` (o TIPO: `source.shape`, `source.text`) e o
//! `desenha_forma_condicional` (a INSTÂNCIA: o L-System em `Branches`). As duas existiam porque a
//! placa não tinha rota para um `geometry_id` e desenhava quadrados de átlas em branco — a `=108`
//! mostrou **cinco quadrados onde a CPU desenha cinco plantas** (doc 119 §7). Hoje o cozimento
//! escreve as linhas de forma para o passe de formas, e o que ainda recusa é nomeado abaixo.
//!
//! ⚠️ **Antes de planear, pela razão das irmãs:** a bomba da CPU tem de ser dona do tique desde o
//! início (nada de marchar um prefixo sequencial duas vezes).

use crate::motion_state::MotionState;
use ph2d_nodegraph::cook::OpResolver;
use ph2d_nodegraph::node::{NodeManifest, NodeTypeId};

// ════════════════════════════════════════════════════════════════════════════════════════════
// ⭐⭐⭐ **A CERCA POR CONTEÚDO** (doc 121 W3) — as formas vivas VÃO à placa.
//
// Até esta wave as duas cercas do cabeçalho (o TIPO e a instância condicional) recusavam todo grafo
// com uma forma viva, porque a placa não tinha rota para um `geometry_id` e desenhava quadrados de
// átlas em branco. Agora o cozimento escreve as linhas de forma para o passe de formas
// (`ph2d_gpu_cook::lower_forma`), e a pergunta passa a ser a de CONTEÚDO: *cada forma publicada
// neste quadro, a placa sabe desenhá-la como o Vello?* — a MESMA porta que a rota da CPU pergunta
// cópia a cópia ([`crate::motion_shape_placa::GeometriasDaPlaca::veredito`]).
//
// ⚠️ **Os handles vêm dos EXTERNOS** (a membrana publica-os antes do cozimento): as três fontes
// de forma viva — a `source.shape`, os glifos do `source.text` e a fita do L-System — escrevem-nos
// na coluna `geometry_id`, e a instância condicional (o L-System em `Segments`) não escreve
// nenhum. ⇒ a cerca condicional fica **coberta por construção**: o L-System em `Branches` publica
// a fita, e em `Segments` não publica nada.
// ════════════════════════════════════════════════════════════════════════════════════════════

/// O motivo de uma cena com formas ficar na CPU com a placa desligada por ordem.
pub(crate) const RECUSA_FORMAS_DESLIGADAS: &str =
    "CPU: formas vivas com a placa de formas desligada (PH2D_FORMAS_NA_PLACA=0)";
/// Uma forma que o passe não desenha como o Vello (tinta própria, traço de padrão ou de pincel).
pub(crate) const RECUSA_FORMA_DO_VELLO: &str =
    "CPU: uma forma viva so' o Vello a desenha (tinta propria, traco de padrao ou de pincel)";
/// Uma forma com traço TRACEJADO: no dispositivo a pose de cada cópia não se lê, e sob um afim não
/// conforme o tracejado mede-se no MUNDO (bug #27). ⭐ Um traço CONTÍNUO vai à placa desde a W4 do
/// doc 121 — o shader constrói-o no ecrã a partir do eixo.
pub(crate) const RECUSA_FORMA_TRACEJADA: &str =
    "CPU: uma forma viva tem traco TRACEJADO -- sob afim nao conforme o tracejado mede-se no mundo";
/// O halo do `fx.glow` lê as cópias da CPU (`motion_glow_layer`), que a rota do dispositivo não tem.
pub(crate) const RECUSA_FORMA_COM_BRILHO: &str =
    "CPU: formas vivas com fx.glow -- o halo le' as copias da CPU";
/// Uma forma declara o colisor pelo cartão e alguém o lê — o contacto só existe na CPU (doc 109).
/// ⚠️ Era a cerca do TIPO que o apanhava; sem ela, a pergunta tem de ser feita aqui.
pub(crate) const RECUSA_FORMA_COM_COLISOR: &str =
    "CPU: uma forma viva declara colisor e o grafo le'-o -- o contacto so' existe na CPU (doc 109)";
/// Uma saída com formas traz a mistura POR LINHA — a recusa que o cozimento devolve
/// (`GpuCookError::FormaComMistura`), dita pelo nome.
pub(crate) const RECUSA_FORMA_COM_MISTURA: &str = "CPU: uma saida com formas traz a mistura POR LINHA (coluna blend) -- so' a cena vectorial a sabe";

/// **Alguma `source.shape` declara o colisor NESTE quadro?** — o `Collide` resolvido (um fio
/// conta). ⚠️ A declaração mora no cartão da forma e sai na CORRENTE que o nó emite, que a cerca de
/// externos ([`super::colisor`]) não vê: era a cerca do TIPO que a apanhava.
pub(crate) fn forma_declara_colisor(motion: &mut MotionState, seconds: f64) -> bool {
    let formas: Vec<(ph2d_nodegraph::graph::NodeId, &'static NodeManifest)> = motion
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "source.shape")
        .filter_map(|n| {
            let manifest = motion
                .registry
                .resolve(NodeTypeId::of(n.type_name.as_str()))?
                .manifest();
            Some((n.id, manifest))
        })
        .collect();
    formas.into_iter().any(|(id, manifest)| {
        let resolved = crate::motion_externals::resolved_params(motion, id, seconds, manifest);
        resolved
            .get(ph2d_node_motion_shape::param::COLLIDE)
            .is_some_and(|&v| v >= 0.5)
    })
}

/// **Uma saída leva formas COM a mistura por linha?** — a pergunta que o cozimento responde com
/// [`ph2d_gpu_cook::GpuCookError::FormaComMistura`], feita ANTES de cozinhar.
///
/// ⚠️⚠️ **Duas fontes, porque nenhuma chega sozinha:** a coluna `blend` pode nascer num kernel do
/// dispositivo (o rastro, o estroboscópio, a sombra — com variante escolhida por param) ou num nó
/// só da CPU, e nenhuma tabela escrita à mão acompanharia as duas. ⇒
/// - **a memória da CPU** (`Cook::peek` da saída): a corrente que a bomba cozeu da última vez que a
///   CPU desenhou — a MESMA pergunta do dispositivo (a PRESENÇA das duas colunas), senão as duas
///   rotas alternariam quadro a quadro;
/// - **a recusa do dispositivo** ([`MotionState::formas_pedem_o_vello`]), CONSUMIDA aqui: ela cobre
///   o primeiro quadro, em que a CPU ainda não cozeu a saída. Depois dele a memória da CPU manda,
///   e é ela que devolve o documento à placa quando o artista tira a mistura.
pub(crate) fn saida_com_mistura_em_formas(motion: &mut MotionState) -> bool {
    use ph2d_nodegraph::attr::Column;
    let recusou = std::mem::take(&mut motion.formas_pedem_o_vello);
    recusou
        || motion.sinks.iter().any(|&s| {
            let Some([ph2d_nodegraph::value::CookValue::Instances(st), ..]) =
                motion.pump.cook.peek(s)
            else {
                return false;
            };
            matches!(st.get("geometry_id"), Some(Column::Scalar(_))) && st.get("blend").is_some()
        })
}

/// Anota a recusa da mistura (o quadro seguinte recusa pelo nome) e devolve se o cozimento correu.
pub(crate) fn anota_a_mistura(
    pedem: &mut bool,
    feito: Result<u32, ph2d_gpu_cook::GpuCookError>,
) -> bool {
    if matches!(feito, Err(ph2d_gpu_cook::GpuCookError::FormaComMistura)) {
        *pedem = true;
    }
    feito.is_ok()
}

/// O dispositivo desenhou o quadro: as formas dele são as publicadas, e as cópias da CPU — de um
/// quadro antigo, porque a bomba não correu — não podem chegar a desenho nenhum.
pub(crate) fn anota_as_formas(motion: &mut MotionState, vivas: Vec<u32>) {
    motion.pump.vector_instances.clear();
    motion.formas_no_dispositivo = if motion.gpu_cook.formas().is_some() {
        vivas
    } else {
        Vec::new()
    };
}

/// **Os handles de forma publicados neste quadro** — a coluna `geometry_id` de cada externo,
/// ordenados e sem repetidos. Vazio num documento sem forma viva.
pub(crate) fn handles_publicados(cook: &ph2d_nodegraph::cook::Cook) -> Vec<u32> {
    use ph2d_nodegraph::attr::Column;
    let mut hs: Vec<u32> = cook
        .externals()
        .values()
        .filter_map(|e| match e.value.get("geometry_id") {
            Some(Column::Scalar(v)) => Some(v),
            _ => None,
        })
        .flatten()
        .filter(|&&g| g > 0.5)
        // ⚠️ `as u32` como a CPU (`geometry_at(i) as u32`) e o dispositivo (`u32(f32)`).
        .map(|&g| g as u32)
        .collect();
    hs.sort_unstable();
    hs.dedup();
    hs
}

/// **As formas publicadas podem ir à placa?** — PURA sobre o que recebe, para um gate medir a lei
/// e não o ambiente. `ligada` é a porta do produto ([`crate::motion_shape_placa::por_ordem`]).
///
/// ⚠️ Um handle AUSENTE do store não recusa: não há o que desenhar, e a placa não o acha (o
/// passe desenha-o como nada, como o Vello desenha uma cópia sem geometria).
pub(crate) fn formas_para_a_placa(
    ligada: bool,
    vivas: &[u32],
    com_brilho: bool,
    com_colisor_lido: bool,
    store: &crate::motion_shape_gen::VecPathStore,
    geometrias: &mut crate::motion_shape_placa::GeometriasDaPlaca,
) -> Result<(), &'static str> {
    use crate::motion_shape_placa::Veredito;
    if vivas.is_empty() {
        return Ok(());
    }
    if !ligada {
        return Err(RECUSA_FORMAS_DESLIGADAS);
    }
    if com_brilho {
        return Err(RECUSA_FORMA_COM_BRILHO);
    }
    if com_colisor_lido {
        return Err(RECUSA_FORMA_COM_COLISOR);
    }
    for &h in vivas {
        match geometrias.veredito(h, store) {
            Veredito::Recusada => return Err(RECUSA_FORMA_DO_VELLO),
            Veredito::Pronta { so_conforme: true } => return Err(RECUSA_FORMA_TRACEJADA),
            Veredito::Pronta { so_conforme: false } | Veredito::Vazia | Veredito::Ausente => {}
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "motion_bridge_gpu_forma_tests.rs"]
mod tests;

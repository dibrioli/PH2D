//! ⭐⭐⭐ **O CENSO DO ALCANCE — um param que NENHUMA combinação de gates revela é um controlo
//! INALCANÇÁVEL** (ciclo 6, W3 — doc 110 §9).
//!
//! ⛔⛔ **Nenhuma sonda deste repo fazia esta pergunta.** O retrato do ciclo conta *quantos params
//! o cartão pinta AGORA* — e um `1 de 4` lê-se igual nas duas leituras opostas: o nó está
//! correctamente gateado (os outros três pertencem a outro modo) **ou** três controlos estão
//! enterrados onde a mão não chega. *Uma coluna que não separa «inerte agora» de «inalcançável
//! sempre» manda auditar nó a nó à mão, que é exactamente o que envelhece.*
//!
//! ⚠️ **A pergunta é DERIVADA, nunca enumerada:** para cada param escondido, existe algum valor dos
//! params que o gateiam — dentro da faixa que o próprio hint deles declara, e só se ESSES forem eles
//! próprios alcançáveis — que o faça aparecer? É um ponto fixo sobre o grafo dos gates, e a resposta
//! sai do MESMO predicado que o cartão usa ([`Visibility::shows`]), nunca de uma segunda cópia da
//! lei.
//!
//! ⚠️ **Ela erra de propósito para o lado PERMISSIVO** nos dois sítios em que não sabe: um gate de
//! TEXTO (a presença de um nome de forma) conta como satisfazível nos dois sentidos, e um param sem
//! hint nenhum conta como tendo a faixa toda. *Um censo que acusa o legítimo é apagado na primeira
//! semana; um que só acusa o certo é lido.*

use crate::motion_bridge::params::params_visible::Visibility;
use crate::motion_state::MotionState;
use ph2d_node_registry::{NodeRegistry, ParamWidget};
use ph2d_nodegraph::node::{NodeManifest, NodeTypeId};
use std::collections::{BTreeMap, BTreeSet};

/// Os valores que o ARTISTA consegue pôr num param, se ele for alcançável.
///
/// ⚠️ **Inteiros para um `Enum`/`Toggle` e as PONTAS para um contínuo** — um `ParamGate` arredonda a
/// inteiro (ele lê um índice), e um [`ParamGateAbove`](ph2d_node_registry::ParamGateAbove) só
/// pergunta se a grandeza passa de um limiar, para o que o **máximo** basta. ⛔ Varrer um contínuo
/// seria medir o tamanho da amostragem, não o alcance.
fn dominio(reg: &NodeRegistry, id: NodeTypeId, param: &str, default: f32) -> Vec<f32> {
    let hint = reg
        .param_ui(id)
        .unwrap_or(&[])
        .iter()
        .find(|h| h.param == param);
    let Some(h) = hint else {
        // Sem hint o param não é pintado — mas ele pode ser o SUJEITO de um gate, e um sujeito
        // sem faixa declarada não é uma acusação: é uma pergunta que esta sonda não sabe fazer.
        return vec![default];
    };
    // ⛔⛔ **A faixa de um `Enum` são as LABELS, nunca o `min`/`max`** — e a 1.ª redacção desta
    // sonda leu o `max` e acusou **onze** params do `source.shape` de inalcançáveis. O hint do
    // `Shape` dele declara `min: 0, max: 0` e deixa a faixa ao widget (o `value.attribute` faz o
    // mesmo com o `Channels`), logo o censo via um selector de **uma** opção e dava por enterrado
    // tudo o que pende das outras. *Uma sonda que lê o campo errado fabrica a dívida que vai
    // mandar consertar.*
    if let ParamWidget::Enum { labels } = h.widget {
        return (0..labels.len()).map(|k| k as f32).collect();
    }
    // O teto DIGITÁVEL manda sobre o teto do slider (o artista escreve o número).
    let teto = reg.param_hard_max(id, param).unwrap_or(h.max);
    let mut v: Vec<f32> = vec![h.min, teto, default];
    if matches!(h.widget, ParamWidget::Toggle) || h.step >= 1.0 {
        let (lo, hi) = (h.min.round() as i64, teto.round() as i64);
        // Um selector tem dezenas de entradas, nunca milhares; o tecto protege de um `max` absurdo.
        for k in lo..=hi.min(lo + 512) {
            v.push(k as f32);
        }
    }
    v
}

/// O relatório de um nó: os params que **nenhuma** combinação revela.
fn inalcancaveis(m: &MotionState, man: &NodeManifest, id: NodeTypeId) -> Vec<&'static str> {
    let vis = Visibility::of(&m.registry, id);
    let defaults: BTreeMap<&str, f32> = man.params.iter().map(|p| (p.name, p.default)).collect();
    // O ponto fixo: começa vazio e cresce enquanto alguém entrar.
    let mut alcancaveis: BTreeSet<&str> = BTreeSet::new();
    loop {
        let antes = alcancaveis.len();
        for p in man.params {
            if alcancaveis.contains(p.name) {
                continue;
            }
            // Os valores que cada OUTRO param pode tomar: a faixa dele se ele já for alcançável
            // (a mão chega lá), senão só o default (ninguém consegue mexer no que não é pintado).
            let candidatos = |q: &str| -> Vec<f32> {
                let d = defaults.get(q).copied().unwrap_or(0.0);
                if alcancaveis.contains(q) {
                    dominio(&m.registry, id, q, d)
                } else {
                    vec![d]
                }
            };
            // ⚠️⚠️ **A busca tem de ser CONJUNTA, e a 1.ª redacção desta sonda era GULOSA** —
            // ela escolhia um valor para cada `when` de cada vez, e o `shows` é uma CONJUNÇÃO:
            // com os outros `when` ainda no default, nenhum valor do primeiro satisfaz. Ela
            // acusou o `source.shape::collider_radius`, que precisa de `Collide` ligado **E** de
            // `Collider Shape = Circle` ao mesmo tempo. *Uma condição com duas metades não se
            // verifica uma metade de cada vez.*
            let quem_gateia: Vec<&str> = m
                .registry
                .param_gates(id)
                .unwrap_or(&[])
                .iter()
                .filter(|g| g.param == p.name)
                .map(|g| g.when)
                .chain(
                    m.registry
                        .param_gates_above(id)
                        .unwrap_or(&[])
                        .iter()
                        .filter(|g| g.param == p.name)
                        .map(|g| g.when),
                )
                .collect::<BTreeSet<&str>>()
                .into_iter()
                .collect();
            // O gate de TEXTO conta como satisfazível: esta sonda não sabe se o artista consegue
            // pôr um nome ali, e o lado permissivo é o que não fabrica dívida.
            let tem_texto = |t: &str| {
                m.registry
                    .param_gates_text(id)
                    .unwrap_or(&[])
                    .iter()
                    .find(|g| g.param == p.name && g.when_text == t)
                    .is_some_and(|g| g.when_present)
            };
            let dominios: Vec<Vec<f32>> = quem_gateia.iter().map(|w| candidatos(w)).collect();
            // O produto cartesiano, com tecto: dois selectores de 16 são 256 combinações, e
            // nenhum nó do catálogo tem mais do que isso a gatear um param só.
            let total: usize = dominios.iter().map(|d| d.len().max(1)).product();
            let mut possivel = total == 1 && quem_gateia.is_empty();
            for k in 0..total.min(100_000) {
                let mut tent: BTreeMap<&str, f32> = BTreeMap::new();
                let mut resto = k;
                for (w, d) in quem_gateia.iter().zip(&dominios) {
                    if d.is_empty() {
                        continue;
                    }
                    tent.insert(w, d[resto % d.len()]);
                    resto /= d.len();
                }
                let valor = |q: &str| -> f32 {
                    tent.get(q)
                        .copied()
                        .unwrap_or_else(|| defaults.get(q).copied().unwrap_or(0.0))
                };
                if vis.shows(p.name, &valor, &tem_texto) {
                    possivel = true;
                    break;
                }
            }
            // Sem gate nenhum ⇒ é sempre pintado (a condição vazia é verdadeira); o laço acima
            // corre uma vez com a atribuição vazia e o `shows` responde `true`.
            if possivel {
                alcancaveis.insert(p.name);
            }
        }
        if alcancaveis.len() == antes {
            break;
        }
    }
    man.params
        .iter()
        .map(|p| p.name)
        .filter(|n| !alcancaveis.contains(n))
        .collect()
}

/// O que o censo varreu, e o que ele achou.
///
/// ⚠️⚠️ **As três contagens NÃO são decoração — elas são o que impede este censo de virar um verde
/// vazio.** Ele lê `0 acusados` no catálogo são, e essa é exactamente a leitura de um censo partido
/// (um prefixo mal escrito, um registry que mudou de nome, uma varredura que devolve zero nós). *A
/// lista vazia só afirma alguma coisa ao lado da população que a produziu.*
pub struct Censo {
    /// Nós varridos.
    pub nos: usize,
    /// Params declarados que ele olhou.
    pub params: usize,
    /// Nós que DECLARAM algum gate de visibilidade — sem estes o censo não teria o que testar.
    pub com_gate: usize,
    /// `(nó, params que nenhuma combinação revela)` — vazio é o estado são.
    pub presos: Vec<(&'static str, Vec<&'static str>)>,
}

/// Os nós do catálogo INTEIRO, e os params que ninguém alcança.
pub fn censo() -> Censo {
    let m = MotionState::new();
    let mut c = Censo {
        nos: 0,
        params: 0,
        com_gate: 0,
        presos: Vec::new(),
    };
    for man in m.registry.manifests() {
        c.nos += 1;
        c.params += man.params.len();
        let tem_gate = !m.registry.param_gates(man.id).unwrap_or(&[]).is_empty()
            || !m
                .registry
                .param_gates_above(man.id)
                .unwrap_or(&[])
                .is_empty()
            || !m.registry.param_gates_text(man.id).unwrap_or(&[]).is_empty();
        if tem_gate {
            c.com_gate += 1;
        }
        let presos = inalcancaveis(&m, man, man.id);
        if !presos.is_empty() {
            c.presos.push((man.name, presos));
        }
    }
    c
}

#[cfg(test)]
#[path = "motion_param_reach_tests.rs"]
mod tests;

// ---------------------------------------------------------------------------------------------
// O VOCABULÁRIO que É gateável — dentro de UM nó, dentro de UMA secção.
// ---------------------------------------------------------------------------------------------

/// **Dois params do MESMO nó, na MESMA secção, com o MESMO rótulo** — o único pedaço da lei do
/// vocabulário que uma máquina sabe julgar.
///
/// ⛔⛔ **E é de propósito que ela não é mais larga.** A sonda
/// [`vocabulario_do_artista`](crate::motion_ciclo_probe::vocabulario_do_artista) mede o catálogo
/// inteiro e acusa **~40** grupos de *«um rótulo, várias chaves»* — quase todos legítimos: o
/// `Count` de nove geradores e o `Count` de um contador são perguntas diferentes que a mesma
/// palavra serve bem. Uma catraca de 40 entradas não é um gate, é uma **licença**
/// (`CLAUDE.md` §5.0), e decidir se dois nós fazem a *mesma pergunta* é semântica, não sintaxe.
///
/// ⭐ **Dentro de uma secção, porém, a resposta é mecânica:** ali os dois controlos estão lado a
/// lado sob o mesmo título, e o artista não tem como os distinguir. O `motion.bezier_warp` tem
/// **quatro** *In X* e está certo — cada um vive na secção do seu canto.
///
/// Devolve `(nó, secção, rótulo, quantos)`.
pub fn rotulos_colididos() -> Vec<(&'static str, String, String, usize)> {
    let m = MotionState::new();
    let mut fora = Vec::new();
    for man in m.registry.manifests() {
        let grupos: BTreeMap<&str, &str> = m
            .registry
            .param_groups(man.id)
            .iter()
            .map(|g| (g.param, g.group))
            .collect();
        // (secção, rótulo) → quantos params
        let mut contagem: BTreeMap<(&str, &str), usize> = BTreeMap::new();
        for h in m.registry.param_ui(man.id).unwrap_or(&[]) {
            let sec = grupos.get(h.param).copied().unwrap_or("");
            *contagem.entry((sec, h.label)).or_default() += 1;
        }
        for ((sec, rotulo), n) in contagem {
            if n > 1 {
                fora.push((man.name, sec.to_string(), rotulo.to_string(), n));
            }
        }
    }
    fora
}

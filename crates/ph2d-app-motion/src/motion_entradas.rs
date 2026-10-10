//! **As entradas do jogo, publicadas para o grafo** (ciclo 19, doc 128 §3.6).
//!
//! Duas metades, pela mesma razão que separa `publish_objects` de `publish_all`: [`le_do_jogo`]
//! copia o recurso `MotionInputs` do mundo (onde o componente, o Luau e os sinais escreveram neste
//! quadro), e [`publica`] — no início do cozimento — junta-lhe os disparos de TESTE dos cartões e
//! publica cada nome como `$input:<nome>`. As cenas de teste, que não têm mundo, enchem
//! [`EntradasDoJogo::jogo`] à mão e passam pelo mesmo [`publica`].

use crate::motion_state::MotionState;
use ph2d_ecs::motion_inputs::{Escritor, Valor};
use ph2d_ecs::{MotionInputs, SimWorld};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::external::{INPUT_COLOR, INPUT_COUNT, INPUT_VALUE, input_of};
use ph2d_nodegraph::graph::NodeId;
use std::collections::{BTreeMap, BTreeSet};

/// O estado do publicador.
#[derive(Default)]
pub struct EntradasDoJogo {
    /// O que o jogo publicou neste quadro.
    pub jogo: MotionInputs,
    /// O `fire` de cada `value.input` Trigger, como visto no quadro anterior.
    fire_visto: BTreeMap<NodeId, f32>,
    /// Os disparos de teste dos cartões, por nome.
    disparos_do_editor: BTreeMap<String, u64>,
    /// Os nomes publicados no quadro anterior (para esquecer os que sumiram).
    publicados: BTreeSet<String>,
    /// O que a linha de comando publica ([`VAR_DA_LINHA`]), lido uma vez.
    da_linha: Vec<(String, Valor)>,
}

/// ⭐ **As entradas pela linha de comando** — `PH2D_MOTION_INPUTS="vida=0.25;estado=1;time=0.2,0.5,1"`:
/// um número (ou `true`/`false`) ou uma cor `r,g,b[,a]` por nome. O jogo lançado já com valores, e a
/// porta das FOTOS de uma cena com outras entradas (a ferramenta não clica).
pub const VAR_DA_LINHA: &str = "PH2D_MOTION_INPUTS";

/// Lê o texto da [`VAR_DA_LINHA`]; o que não se entende é dito no terminal e saltado.
#[must_use]
pub fn le_a_linha(texto: &str) -> Vec<(String, Valor)> {
    let mut out = Vec::new();
    for par in texto.split(';').map(str::trim).filter(|p| !p.is_empty()) {
        let Some((nome, valor)) = par.split_once('=') else {
            eprintln!("[{VAR_DA_LINHA}] sem `=`: `{par}`");
            continue;
        };
        let nums: Vec<Option<f32>> = valor
            .split(',')
            .map(|x| match x.trim() {
                "true" => Some(1.0),
                "false" => Some(0.0),
                x => x.parse().ok().filter(|v: &f32| v.is_finite()),
            })
            .collect();
        let v = match nums.as_slice() {
            [Some(x)] => Valor::Numero(*x),
            [Some(r), Some(g), Some(b)] => Valor::Cor([*r, *g, *b, 1.0]),
            [Some(r), Some(g), Some(b), Some(a)] => Valor::Cor([*r, *g, *b, *a]),
            _ => {
                eprintln!(
                    "[{VAR_DA_LINHA}] `{nome}`: um número ou uma cor r,g,b[,a] — veio `{valor}`"
                );
                continue;
            }
        };
        out.push((nome.trim().to_string(), v));
    }
    out
}

impl EntradasDoJogo {
    /// O publicador do app: lê a [`VAR_DA_LINHA`] uma vez.
    #[must_use]
    pub fn do_ambiente() -> Self {
        Self {
            da_linha: std::env::var(VAR_DA_LINHA)
                .map(|t| le_a_linha(&t))
                .unwrap_or_default(),
            ..Self::default()
        }
    }
}

/// Copia o recurso do mundo (ausente = nada publicado).
pub fn le_do_jogo(motion: &mut MotionState, sim: &SimWorld) {
    motion.entradas.jogo = sim
        .world()
        .get_resource::<MotionInputs>()
        .cloned()
        .unwrap_or_default();
}

/// **Os nomes que o grafo LÊ** (cada `value.input`, sem repetir, em ordem) — o que o cartão do
/// `Motion Input` no Inspector oferece, para o artista não ter de adivinhar o nome.
#[must_use]
pub fn nomes_que_o_grafo_le(motion: &MotionState) -> Vec<String> {
    let g = &motion.doc.graph;
    let nomes: BTreeSet<String> = g
        .nodes()
        .iter()
        .filter(|n| n.type_name == ph2d_node_value_input::MANIFEST.name)
        .map(|n| {
            g.node_text_param_overrides(n.id)
                .and_then(|t| t.get(ph2d_node_value_input::NAME_KEY))
                .map_or(ph2d_node_value_input::FACTORY_NAME, String::as_str)
                .trim()
                .to_string()
        })
        .filter(|n| !n.is_empty())
        .collect();
    nomes.into_iter().collect()
}

/// ⭐ O disparo de TESTE: cada mudança do `fire` de um `value.input` Trigger é um disparo no
/// nome dele (o cartão é o jogo de quem ainda não tem jogo).
fn disparos_dos_cartoes(motion: &mut MotionState) {
    let g = &motion.doc.graph;
    let e = &mut motion.entradas;
    let mut vistos = BTreeMap::new();
    for n in g.nodes() {
        if n.type_name != ph2d_node_value_input::MANIFEST.name {
            continue;
        }
        let params = g.node_param_overrides(n.id);
        let p = |k: &str| params.and_then(|o| o.get(k).copied()).unwrap_or(0.0);
        if p(ph2d_node_value_input::KIND).round() as i32 != ph2d_node_value_input::KIND_TRIGGER {
            continue;
        }
        let fire = p(ph2d_node_value_input::FIRE);
        if e.fire_visto.get(&n.id).is_some_and(|&antes| antes != fire) {
            let nome = g
                .node_text_param_overrides(n.id)
                .and_then(|t| t.get(ph2d_node_value_input::NAME_KEY))
                .map_or(ph2d_node_value_input::FACTORY_NAME, String::as_str)
                .trim()
                .to_string();
            if !nome.is_empty() {
                *e.disparos_do_editor.entry(nome).or_insert(0) += 1;
            }
        }
        vistos.insert(n.id, fire);
    }
    e.fire_visto = vistos;
}

fn coluna_de_valores(linhas: &[Valor]) -> Stream {
    let n = linhas.len();
    let v: Vec<f32> = linhas
        .iter()
        .map(|l| match *l {
            Valor::Numero(x) => x,
            // Um número lido de uma cor: a média dos canais (a luminosidade fica para o nó).
            Valor::Cor(c) => (c[0] + c[1] + c[2]) / 3.0,
        })
        .collect();
    let mut s = Stream::new(n).with(INPUT_VALUE, Column::Scalar(v));
    if linhas.iter().any(|l| matches!(l, Valor::Cor(_))) {
        let c = linhas
            .iter()
            .map(|l| match *l {
                Valor::Numero(x) => [x, x, x, 1.0],
                Valor::Cor(c) => c,
            })
            .collect();
        s = s.with(INPUT_COLOR, Column::Vec4(c));
    }
    s
}

/// As colunas de `novo` estão todas em `velho`, iguais (e a contagem é a mesma)?
fn mesmas_colunas(velho: &Stream, novo: &Stream) -> bool {
    velho.count() == novo.count() && novo.columns().all(|(n, c)| velho.get(n) == Some(c))
}

/// Publica cada entrada como `$input:<nome>`; um nome com disparos publica a CONTAGEM (a bomba
/// deriva o pulso de cada tique), os outros os valores. Um nome que deixou de ser escrito sai.
pub fn publica(motion: &mut MotionState) {
    disparos_dos_cartoes(motion);
    let e = &mut motion.entradas;
    for (nome, v) in &e.da_linha {
        e.jogo.set(nome, Escritor::Linha, *v);
    }
    let mut streams: BTreeMap<String, Stream> = motion
        .entradas
        .jogo
        .valores()
        .map(|(nome, linhas)| (nome.to_string(), coluna_de_valores(&linhas)))
        .collect();
    let mut contagens: BTreeMap<String, Vec<f32>> = motion
        .entradas
        .jogo
        .disparos()
        .map(|(nome, n)| (nome.to_string(), n.iter().map(|&c| c as f32).collect()))
        .collect();
    for (nome, &n) in &motion.entradas.disparos_do_editor {
        contagens.entry(nome.clone()).or_default().push(n as f32);
    }
    for (nome, n) in contagens {
        let s = Stream::new(n.len()).with(INPUT_COUNT, Column::Scalar(n));
        streams.insert(nome, s);
    }
    let agora: BTreeSet<String> = streams.keys().map(|n| input_of(n)).collect();
    let mut mudou = false;
    for velho in motion.entradas.publicados.difference(&agora) {
        motion.pump.cook.remove_external(velho);
        mudou = true;
    }
    for (nome, s) in streams {
        let chave = input_of(&nome);
        // ⚠️ Pelas COLUNAS que se publicam: num gatilho a bomba acrescentou o pulso (`v`) à
        // contagem, e republicar só a contagem a cada quadro seria uma «mudança» por quadro.
        let igual = motion
            .pump
            .cook
            .externals()
            .get(&chave)
            .is_some_and(|e| mesmas_colunas(&e.value, &s));
        if !igual {
            motion.pump.cook.set_external(chave, s);
            mudou = true;
        }
    }
    // Uma entrada que mudou com o relógio parado (um valor no Inspector, um disparo de teste)
    // tem de se ver: o anel fica, porque o passado não mudou.
    if mudou {
        motion.pump.mark_dirty_keeping_ring();
    }
    motion.entradas.publicados = agora;
}

#[cfg(test)]
#[path = "motion_entradas_tests.rs"]
mod tests;

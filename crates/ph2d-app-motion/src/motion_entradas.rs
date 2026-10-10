//! **As entradas do jogo, publicadas para o grafo** (ciclo 19, doc 128 §3.6).
//!
//! Duas metades, pela mesma razão que separa `publish_objects` de `publish_all`: [`le_do_jogo`]
//! copia o recurso `MotionInputs` do mundo (onde o componente, o Luau e os sinais escreveram neste
//! quadro), e [`publica`] — no início do cozimento — junta-lhe os disparos de TESTE dos cartões e
//! publica cada nome como `$input:<nome>`. As cenas de teste, que não têm mundo, enchem
//! [`EntradasDoJogo::jogo`] à mão e passam pelo mesmo [`publica`].

use crate::motion_state::MotionState;
use ph2d_ecs::motion_inputs::Valor;
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
}

/// Copia o recurso do mundo (ausente = nada publicado).
pub fn le_do_jogo(motion: &mut MotionState, sim: &SimWorld) {
    motion.entradas.jogo = sim
        .world()
        .get_resource::<MotionInputs>()
        .cloned()
        .unwrap_or_default();
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

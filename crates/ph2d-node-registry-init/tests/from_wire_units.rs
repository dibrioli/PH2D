//! **O QUE UM NÓ PODE DECLARAR `ParamUnit::FromWire`** — e a prova de que ele não mentiu.
//!
//! # A lei, e por que ela precisa de um gate
//!
//! `FromWire` diz *"a unidade deste param é a do param que o meu fio conduz"*. Ela dissolve a
//! cerca do `ParamUnit::None` exactamente onde a cerca não se aplica: um fio que cai numa
//! **coluna** pode significar qualquer coisa (metros em `P`, graus em `rot`, nada em `tint`),
//! mas um fio que cai num **param** termina em UM param declarado com UMA unidade declarada.
//!
//! ⚠️ **Uma declaração errada é pior que nenhuma**, e é o modo de falha que o `FromChannel`
//! já registou por escrito: uma conversão de comprimento aplicada a graus transforma um `±90`
//! num `±9000`. Declarar `frequency` como `FromWire` faria o painel mostrar um hertz em
//! pixels. ⇒ a declaração tem de ser **falsificável**, e este arquivo é a falsificação.
//!
//! # As três condições, todas MEDIDAS
//!
//! 1. **GERADOR.** Nenhuma porta de entrada do nó tem o tipo da saída. Um nó de
//!    TRANSFORMAÇÃO (`value.wave`, cujo `in` é um `VALUE`) recebe a unidade do fio de
//!    ENTRADA, não dos params — declarar `FromWire` nele responderia a pergunta errada.
//! 2. **O CONJUNTO É A ESCALA DA SAÍDA.** Escalar **todos** os params declarados por `k`
//!    escala a saída por `k` (homogeneidade de grau 1). É isto que apanha um param que não
//!    pertence (`frequency` não escala a saída) **e** um conjunto INCOMPLETO (declarar
//!    `amplitude` sem `offset` deixa `w·k·amp + offset ≠ k·out`).
//! 3. **O CONTROLE.** A saída não pode ser nula nem constante — uma saída de zeros é
//!    homogénea em tudo, e um gate satisfeito por zeros não afirma nada.
//!
//! ⛔ **O que NÃO se declara, medido em 2026-08-28.** O `value.wave` reprova a condição 1 (é
//! uma transformação: o `in` dele é um `VALUE`). O `value.pattern` fica fora porque os slots
//! dele são **declarados normalizados** `0..1` pela convenção do próprio nó — *"compose a
//! `value.map_range` for another range"*. E o `value.time` e o `value.cursor` ficam fora porque
//! a saída deles tem unidade PRÓPRIA: segundos e distância de mundo.

mod common;

use common::*;
use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamWidget};
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::NodeManifest;

/// Os params que `m` declara `FromWire`.
fn from_wire(reg: &NodeRegistry, m: &'static NodeManifest) -> Vec<&'static str> {
    m.params
        .iter()
        .filter(|p| reg.param_unit_declared(m.id, p.name) == Some(ParamUnit::FromWire))
        .map(|p| p.name)
        .collect()
}

/// Os nós que declaram alguma coisa `FromWire`.
fn declaring(reg: &NodeRegistry) -> Vec<(&'static NodeManifest, Vec<&'static str>)> {
    let mut out: Vec<_> = reg
        .manifests()
        .filter_map(|m| {
            let s = from_wire(reg, m);
            (!s.is_empty()).then_some((m, s))
        })
        .collect();
    out.sort_by_key(|(m, _)| m.name);
    out
}

/// **CONDIÇÃO 1 — só um GERADOR pode declarar `FromWire`.**
///
/// ⚠️ Um nó cujo `in` tem o tipo do `out` recebe a unidade do fio de entrada. O `value.wave`
/// é exactamente isso (`in: VALUE`), e por isso está fora — apesar de ter `amplitude` e
/// `offset` com os mesmos nomes dos dois que estão dentro. *Nomes iguais, perguntas
/// diferentes.*
#[test]
fn only_a_generator_may_declare_from_wire() {
    let reg = registry();
    for (m, set) in declaring(&reg) {
        let out_ty = m.outputs.first().map(|o| o.ty).expect("emite alguma coisa");
        for p in m.inputs {
            assert!(
                !p.ty.connects_directly(out_ty),
                "`{}` declara {set:?} como `FromWire` mas tem a porta `{}` do tipo da SAIDA — \
                 a unidade dele vem do fio de ENTRADA, nao dos params",
                m.name,
                p.name
            );
        }
    }
}

/// **CONDIÇÃO 2 (a que faz o trabalho) — o conjunto declarado É a escala da saída.**
///
/// Escalar todos os declarados por `k` tem de escalar a saída por `k`. É o teste que separa
/// *"este param vive na unidade do que eu emito"* de *"este param mexe no que eu emito"* —
/// a `frequency` mexe e **não** escala.
///
/// ⚠️ **E apanha o conjunto INCOMPLETO, que é o erro fácil:** com `out = w·amp + offset`,
/// declarar só o `amplitude` dá `w·k·amp + offset`, que não é `k·out` — **desde que o `offset`
/// esteja fora do neutro**, e essa condição é metade do gate.
///
/// ⚠️⚠️ **A 1.ª versão deste teste NÃO a tinha, e a mutação que declara `amplitude` sozinho
/// SOBREVIVEU.** Com o `offset` no default (`0`), `w·amp` é homogéneo em `amp` sozinho e o
/// gate ficava verde sobre uma declaração pela metade — enquanto este doc-comment afirmava o
/// contrário. *Uma afirmação que mutação nenhuma mata é uma afirmação sobre nada.* A cura é
/// pôr **todo param fora do conjunto** no ponto a 3/4 da faixa antes de escalar.
#[test]
fn the_from_wire_set_is_the_output_scale() {
    let reg = registry();
    let all = catalogue(&reg);
    let mut checked = 0usize;
    for (m, set) in declaring(&reg) {
        let benches = all_benches(&reg, &all, m);
        let (g0, n, _) = benches
            .into_iter()
            .find(|(g, n, _)| snapshot(g, &reg, *n).is_some())
            .unwrap_or_else(|| panic!("`{}` tem de montar numa bancada", m.name));

        // Um ponto de partida em que TODO declarado é não-nulo — senão escalar não move nada.
        let base: Vec<(&'static str, f32)> = set
            .iter()
            .enumerate()
            .map(|(i, p)| (*p, 1.0 + i as f32))
            .collect();
        // ⚠️⚠️ **E TODO O RESTO FORA DO NEUTRO — foi isto que a 1.ª versão deste gate não
        // fazia, e a mutação MB SOBREVIVEU por causa disso.** Declarar `amplitude` sem
        // `offset` passa **enquanto o `offset` valer o default `0`**: `w·amp` é homogéneo em
        // `amp` sozinho. O sibling em falta só se vê quando ele CONTRIBUI. É a mesma lei que a
        // `contexts()` da caça aos knobs mortos já pagou — *um knob que só age somando-se a
        // outro nunca se vê com o outro no neutro* — e ela vale para a sonda tanto como para o
        // knob.
        let hints = reg.param_ui(m.id).unwrap_or(&[]);
        let hot: Vec<(&'static str, f32)> = m
            .params
            .iter()
            .filter(|p| !set.contains(&p.name))
            .filter_map(|p| {
                let h = hints.iter().find(|h| h.param == p.name)?;
                // ⚠️ **MAGNITUDES sim, SELETORES não** — o mesmo corte que a `contexts()` da
                // bancada faz, e ele também custou uma corrida: pôr o `kind` do `value.number`
                // a 3/4 da faixa arredonda para `1`, que é o modo **Booleano**, onde a saída é
                // `0`/`1` e não é homogénea em `value` nenhum. *Um seletor escolhe uma LEI; a
                // declaração é sobre a lei que o nó shipa, não sobre todas as que ele tem.*
                if !matches!(h.widget, ParamWidget::Slider | ParamWidget::Angle) {
                    return None;
                }
                // O ponto a 3/4 da faixa, e nunca o default — o mesmo critério da bancada.
                let v = h.min + (h.max - h.min) * 0.75;
                ((v - p.default).abs() > 1e-6).then_some((p.name, v))
            })
            .collect();
        let cook_at = |scale: f32| -> Vec<f32> {
            let mut g: Graph = g0.clone();
            for (p, v) in &hot {
                g.set_param(n, *p, *v);
            }
            for (p, v) in &base {
                g.set_param(n, *p, v * scale);
            }
            trace_values(&g, &reg, n)
        };
        let one = cook_at(1.0);
        let k = 3.0_f32;
        let scaled = cook_at(k);

        // CONTROLE (condição 3): a saída MOVE-SE. Zeros são homogéneos em tudo.
        assert!(
            one.iter().any(|v| v.abs() > 1e-6),
            "`{}`: a bancada devolve tudo zero — um gate satisfeito por zeros nao afirma nada",
            m.name
        );
        assert_eq!(one.len(), scaled.len(), "`{}`: a forma mudou", m.name);
        for (a, b) in one.iter().zip(&scaled) {
            let want = a * k;
            let tol = 1e-4 * want.abs().max(1.0);
            assert!(
                (b - want).abs() <= tol,
                "`{}` declara {set:?} como `FromWire`, entao escalar o CONJUNTO por {k} tem de \
                 escalar a saida por {k}: {a} -> {b}, esperava {want}. Ou sobra um param que \
                 nao vive na unidade da saida, ou FALTA um que vive.",
                m.name
            );
        }
        checked += 1;
    }
    // ⚠️ O controle da própria varredura: um censo que não varreu nada fica verde.
    assert!(
        checked >= 2,
        "o censo tem de alcancar os nos que declaram FromWire, alcancou {checked}"
    );
}

/// A coluna de valor do nó, ao longo do traço — a mesma bancada da caça aos knobs mortos,
/// lida como números em vez de bits (aqui a pergunta é aritmética, não de identidade).
fn trace_values(g: &Graph, reg: &NodeRegistry, n: NodeId) -> Vec<f32> {
    use ph2d_nodegraph::attr::Column;
    use ph2d_nodegraph::cook::Cook;
    use ph2d_nodegraph::value::CookValue;
    let mut cook = Cook::new();
    let mut out = Vec::new();
    for k in 0..8 {
        let t = k as f64 * DT;
        let Ok(vals) = cook.cook(g, reg, n, t) else {
            break;
        };
        if let Some(CookValue::Instances(s)) = vals.first()
            && let Some(Column::Scalar(v)) = s.get(ph2d_nodegraph::attr::VALUE_COLUMN)
        {
            out.extend_from_slice(v);
        }
        if cook.advance_tick(g, reg, t).is_err() {
            break;
        }
    }
    out
}

/// **E a lista de quem declara é IMPRESSA** — um censo mudo envelhece sem ninguém saber.
#[test]
fn the_from_wire_census_is_named() {
    let reg = registry();
    let named = declaring(&reg);
    for (m, set) in &named {
        println!("{}\t{set:?}", m.name);
    }
    // O `value.number` é o nó do report que abriu a lei; se ele sair da lista, a lei perdeu o
    // caso que a motivou.
    assert!(
        named.iter().any(|(m, _)| m.name == "value.number"),
        "o `value.number` e' o caso que abriu a lei"
    );
}

//! Gates do **nó inteiro** — os defaults, o crescimento fraccionário, e a costura entre a
//! gramática e o painel.

use super::*;
use ph2d_nodegraph::attr::Column;

fn scal(s: &ph2d_nodegraph::attr::Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("coluna {name}"),
    }
}

fn height(s: &ph2d_nodegraph::attr::Stream) -> f32 {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).fold(f32::MIN, f32::max),
        _ => 0.0,
    }
}

fn default_tree(generations: f32) -> ph2d_nodegraph::attr::Stream {
    probe_build(DEFAULT_AXIOM, DEFAULT_RULES, generations, &[])
}

/// **O nó dropado da paleta desenha uma árvore** — não um ponto, não nada.
///
/// ⚠️ Um gerador cujo default é vazio treina o artista a achar que o nó está partido, e ele
/// nunca chega a escrever a primeira regra.
#[test]
fn the_factory_defaults_draw_a_tree_that_branches() {
    let s = default_tree(5.0);
    assert!(s.count() > 30, "so {} elementos", s.count());
    assert!(height(&s) > 1.0, "altura {}", height(&s));
    // Ramifica: há mais do que um elemento a pendurar-se no mesmo pai.
    let parent = scal(&s, "parent");
    let mut sorted = parent.clone();
    sorted.sort_by(f32::total_cmp);
    sorted.dedup();
    assert!(
        sorted.len() < parent.len(),
        "sem pais repetidos nao ha ramo nenhum"
    );
    // E as três colunas que só um L-System sabe estão lá.
    for c in ["depth", "gen", "sym"] {
        assert!(s.get(c).is_some(), "falta a coluna {c}");
    }
}

/// ⚠️ **O slider `Step` do painel está VIVO no axioma de fábrica.**
///
/// O default é `A(step)`, não `A(0.5)`: uma expressão vê os params do nó pelo nome. Um
/// literal ali deixaria o slider inerte no estado em que o artista o encontra — o knob morto
/// que o doc 90 caça, no primeiro segundo de vida do nó.
#[test]
fn the_step_slider_is_alive_in_the_factory_axiom() {
    let a = height(&default_tree(5.0));
    let b = height(&probe_build(
        DEFAULT_AXIOM,
        DEFAULT_RULES,
        5.0,
        &[(param::STEP, 1.0)],
    ));
    assert!(
        b > a * 1.8,
        "dobrar o Step tem de dobrar a planta: {a} contra {b}"
    );
}

/// ⭐⭐ **GERAÇÕES FRACCIONÁRIAS: a planta cresce CONTINUAMENTE, e não aos saltos.**
///
/// ⚠️⚠️ **A 1.ª redacção deste gate era VAZIA, e uma mutação provou-o.** Ela media que a
/// altura sobe monotonamente entre a geração 4 e a 5 e que a ponta bate com os inteiros —
/// e **um salto satisfaz as três coisas**: com o crescimento desligado, a planta muda de
/// tamanho de uma vez em `4,1` e fica parada até `5,0`. Monótono, sim; crescimento, não.
/// *A feature é a CONTINUIDADE, então é a continuidade que se mede.*
///
/// A régua é o maior degrau de um passo contra a subida TOTAL do intervalo: com dez passos,
/// um crescimento uniforme dá `0,10` a cada um, e um salto dá `1,00` num só.
#[test]
fn a_fractional_generation_grows_continuously_instead_of_jumping() {
    const STEPS: usize = 10;
    let hs: Vec<f32> = (0..=STEPS)
        .map(|k| height(&default_tree(4.0 + k as f32 / STEPS as f32)))
        .collect();
    for w in hs.windows(2) {
        assert!(w[1] >= w[0] - 1e-5, "a altura recuou: {hs:?}");
    }
    let rise = hs[STEPS] - hs[0];
    assert!(
        rise > 1e-3,
        "a geracao inteira tem de ser mais alta: {hs:?}"
    );
    let worst = hs.windows(2).map(|w| w[1] - w[0]).fold(f32::MIN, f32::max);
    // Uniforme seria `1/STEPS`; um salto seria `1`. A barra fica a meio caminho do
    // desastre, longe do ruído de qual ramo por acaso é o mais alto.
    assert!(
        worst < rise * 0.35,
        "um passo sozinho levou {:.0}% da subida — isto e' um SALTO, nao um crescimento: {hs:?}",
        worst / rise * 100.0
    );
    // ⚠️ E o fecho nas PONTAS: `4.0` é a geração 4 inteira, e `5.0` é a 5 inteira.
    assert!((hs[0] - height(&default_tree(4.0))).abs() < 1e-6);
    assert!((hs[STEPS] - height(&default_tree(5.0))).abs() < 1e-4);
}

/// ⚠️ **Só a geração MAIS NOVA cresce.** O resto da planta fica exactamente onde estava —
/// senão o slider encolheria a árvore inteira em vez de fazer o rebento aparecer.
#[test]
fn only_the_youngest_generation_grows_and_the_rest_stands_still() {
    let a = default_tree(4.0);
    let b = default_tree(4.5);
    let (pa, pb) = (
        match a.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => panic!(),
        },
        match b.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => panic!(),
        },
    );
    let gb = scal(&b, "gen");
    let old = gb.iter().filter(|g| **g < 5.0).count();
    assert_eq!(
        old,
        pa.len(),
        "a parte velha tem de ser a arvore de 4 geracoes inteira"
    );
    // ⚠️ **Não se compara por ÍNDICE.** A travessia é em profundidade, então um elemento novo
    // nasce NO MEIO da lista e empurra os seguintes — a 1.ª redacção deste gate emparelhava
    // `i` com `i` e acusava a árvore inteira de se mexer. O que se preserva é a ORDEM dos
    // velhos, que é o que a filtragem por geração recupera.
    let old_now: Vec<[f32; 2]> = pb
        .iter()
        .zip(gb.iter())
        .filter(|(_, g)| **g < 5.0)
        .map(|(p, _)| *p)
        .collect();
    for (i, (p, q)) in pa.iter().zip(old_now.iter()).enumerate() {
        assert!(
            (p[0] - q[0]).abs() < 1e-5 && (p[1] - q[1]).abs() < 1e-5,
            "o elemento velho {i} mexeu-se: {p:?} -> {q:?}"
        );
    }
}

/// O plano de gerações, incluindo o lixo que um documento carregado pode trazer.
#[test]
fn the_generation_plan_is_total() {
    assert_eq!(generation_plan(0.0), (0, (0, 1.0)));
    assert_eq!(generation_plan(3.0), (3, (3, 1.0)));
    assert_eq!(generation_plan(f32::NAN), (0, (0, 1.0)));
    assert_eq!(generation_plan(-2.0), (0, (0, 1.0)));
    let (g, (y, f)) = generation_plan(4.25);
    assert_eq!((g, y), (5, 5));
    assert!((f - 0.25).abs() < 1e-6);
}

/// ⚠️ **Quando o orçamento satura, a geração que sobrou é INTEIRA** — não uma fracção de uma
/// geração que nunca chegou a existir.
#[test]
fn a_saturated_derivation_does_not_shrink_the_generation_it_did_finish() {
    // `F -> FF` duplica: 32 gerações não cabem em `MAX_MODULES` de maneira nenhuma.
    //
    // ⚠️ **A geração em que ela pára é DERIVADA do tecto, nunca escrita à mão** — a 1.ª
    // redacção cravou `17` e ficou vermelha no dia em que a medição moveu o tecto para
    // `262 144`. Um número copiado de outra constante é a mesma constante escrita duas vezes.
    let last_whole = crate::MAX_MODULES.ilog2();
    let saturated = probe_build("F", "F -> FF", 31.5, &[]);
    let whole = probe_build("F", "F -> FF", last_whole as f32, &[]);
    assert_eq!(
        saturated.count(),
        whole.count(),
        "a derivacao para em 2^{last_whole} modulos (mais a raiz que a tartaruga planta)"
    );
    assert_eq!(saturated.count(), crate::MAX_MODULES + 1);
}

/// Todo param declarado tem um hint, e todo hint nomeia um param declarado **ou** um dos
/// dois text params — o censo local que impede um knob inalcançável no painel.
#[test]
fn every_declared_param_has_a_hint_and_every_hint_has_a_home() {
    for p in MANIFEST.params {
        assert!(
            PARAM_HINTS.iter().any(|h| h.param == p.name),
            "o param {} nao tem hint — ele existe no modelo e o painel nao o mostra",
            p.name
        );
    }
    for h in PARAM_HINTS {
        let declared = MANIFEST.params.iter().any(|p| p.name == h.param);
        let text = h.param == AXIOM_PARAM || h.param == RULES_PARAM;
        assert!(declared || text, "o hint {} nao tem param nenhum", h.param);
    }
    assert!(
        MANIFEST.params.len() + 2 <= ph2d_panel_motion_params_row_cap(),
        "o no declara mais linhas do que o painel pinta"
    );
}

/// O tecto de linhas do painel de params, repetido aqui como NÚMERO e não como dependência.
///
/// ⚠️ Este crate é uma folha e não pode depender do painel; o censo de verdade
/// (`the_panel_shows_every_param_of_every_node`) corre na shell e mede TODOS os nós. Este é
/// só a rede local — se ele e o painel discordarem, é o da shell que manda.
const fn ph2d_panel_motion_params_row_cap() -> usize {
    24
}

/// Um `Generations` negativo, `NaN` ou enorme não pode custar nada nem emitir lixo.
#[test]
fn untrusted_generations_never_costs_and_never_empties() {
    for g in [-3.0, f32::NAN, f32::INFINITY, 0.0] {
        let s = default_tree(g);
        assert!(s.count() >= 1, "sempre pelo menos a raiz (g = {g})");
        assert!(
            s.count() < 4,
            "e nada de derivar com g = {g}: {}",
            s.count()
        );
    }
}

/// Um axioma ou umas regras VAZIAS caem no default — um text param apagado não pode apagar
/// a planta.
#[test]
fn an_emptied_text_param_falls_back_to_the_factory_grammar() {
    let a = probe_build("", "", 5.0, &[]);
    let b = default_tree(5.0);
    assert_eq!(a.count(), b.count());
}

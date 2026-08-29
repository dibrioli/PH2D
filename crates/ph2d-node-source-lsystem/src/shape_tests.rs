//! Gates do **modo guiado** — a gramática derivada dos números de forma.
//!
//! ⚠️ **A régua é o TEXTO gerado, e é a régua certa aqui**: o que este módulo faz é escrever
//! uma gramática, e afirmá-la sobre a planta desenhada mediria o derivador e a tartaruga por
//! cima. A metade que mede a PLANTA vive no `lib_tests.rs`, onde os dois lados se juntam.

use super::*;

fn sh(branches: f32, segments: f32, variation: f32, bend: f32) -> Shape {
    Shape {
        branches,
        segments,
        variation,
        bend,
    }
}

/// O default do manifesto, escrito uma vez.
fn factory() -> Shape {
    sh(2.0, 1.0, 0.0, 0.0)
}

/// ⭐ **O DEFAULT GUIADO é a árvore binária** — a forma que o nó sempre teve, agora
/// alcançável por sliders.
#[test]
fn the_guided_default_is_the_binary_tree_written_with_the_house_symbols() {
    assert_eq!(
        rules(&factory()),
        "A(s) -> F(s)![+A(s*length_scale)][-A(s*length_scale)]"
    );
}

/// ⭐⭐ **A GRAMÁTICA GERADA REFERENCIA OS PARAMS PELO NOME** — a propriedade que faz os
/// sliders sobreviverem à conversão para `Grammar`.
///
/// ⚠️ Se o gerador assasse `s*0.9` em vez de `s*length_scale`, mudar para `Grammar` mataria
/// o *Length Scale* em silêncio: a planta continuaria igual e o slider deixaria de fazer
/// alguma coisa. É o knob morto do doc 90, criado por uma conversão.
///
/// ⚠️⚠️ **A 1.ª redacção deste gate mediu a GRAFIA e não a propriedade, e reprovou sobre
/// produto correto**: ela exigia a palavra `angle` no texto, e num leque de 3 os três ramos
/// saem `+`, nada, `-` — símbolos NUS, que *são* `angle` por definição da tartaruga
/// (`m.arg(0).unwrap_or(set.angle)`). O slider estava vivo e o gate dizia que não.
/// ⇒ Ele afirma agora duas coisas separadas: **nenhum VALOR é assado** (o defeito), e o nome
/// aparece onde de facto **precisa** de aparecer (a viragem fraccionária).
#[test]
fn the_generated_grammar_names_the_params_it_uses_and_never_bakes_their_values() {
    let wide = rules(&sh(5.0, 2.0, 0.0, 7.0));
    for name in ["length_scale", "angle", "bend"] {
        assert!(
            wide.contains(name),
            "a gramatica gerada tem de citar `{name}` pelo nome: {wide}"
        );
    }
    // ⚠️ **O CONTROLE, e é ele o gate**: nenhum VALOR de param entra como literal, em leque
    // nenhum — incluindo os que saem com os símbolos nus.
    for n in 1..=(MAX_BRANCHES as u32) {
        let r = rules(&sh(n as f32, 2.0, 0.0, 7.0));
        for baked in ["0.9", "25.0", "7.0", "0.700"] {
            assert!(
                !r.contains(baked),
                "n = {n}: `{baked}` foi assado como literal — o slider morre na conversao: {r}"
            );
        }
    }
}

/// **`Branches = n` põe exactamente `n` rebentos**, e cada um no seu par de parênteses.
#[test]
fn the_branch_count_is_the_number_of_shoots_in_the_successor() {
    for n in 2..=(MAX_BRANCHES as u32) {
        let r = rules(&sh(n as f32, 1.0, 0.0, 0.0));
        assert_eq!(
            r.matches("A(s*length_scale)").count(),
            n as usize,
            "n = {n}: {r}"
        );
        assert_eq!(r.matches('[').count(), n as usize, "n = {n}: {r}");
        assert_eq!(r.matches(']').count(), n as usize, "n = {n}: {r}");
    }
}

/// ⚠️ **UM ramo não leva parênteses rectos** — e não é estética.
///
/// Um `[A(...)]` com um só filho empurra e tira o mesmo estado a cada geração e desenha o
/// mesmo caule; o custo é um par de módulos por nível, por nada. E com `Branches = 1` o
/// artista pediu um CAULE: a forma que ele espera é a que não ramifica.
#[test]
fn a_single_shoot_carries_no_brackets_at_all() {
    let r = rules(&sh(1.0, 1.0, 0.0, 0.0));
    assert_eq!(r, "A(s) -> F(s)!A(s*length_scale)");
}

/// **Os dois ramos de fora usam o símbolo NU** — que é `+(angle)` por definição da tartaruga.
///
/// ⚠️ É o que faz o slider *Angle* dizer a verdade: o leque abre exactamente `2·angle`, e não
/// uma fracção dele. A escrita nua é também a do ABOP, então é a que o artista reconhece nos
/// exemplos que encontrar.
#[test]
fn the_outermost_shoots_use_the_bare_symbol_so_the_fan_spans_twice_the_angle() {
    for n in 2..=(MAX_BRANCHES as u32) {
        let r = rules(&sh(n as f32, 1.0, 0.0, 0.0));
        assert!(r.contains("[+A("), "n = {n} sem o `+` nu: {r}");
        assert!(r.contains("[-A("), "n = {n} sem o `-` nu: {r}");
    }
    // E os do MEIO saem com a fracção explícita — é ela que ensina de onde o número vem.
    let five = rules(&sh(5.0, 1.0, 0.0, 0.0));
    assert!(five.contains("+(angle*0.500)"), "{five}");
    assert!(five.contains("-(angle*0.500)"), "{five}");
    // ⚠️ E o do CENTRO de um leque ímpar não leva viragem nenhuma — um `+(angle*0.000)`
    // seria um módulo por geração a somar zero.
    assert!(five.contains("[A(s*length_scale)]"), "{five}");
}

/// **`Trunk Segments = k` põe `k` passos antes de bifurcar.**
#[test]
fn the_trunk_segments_are_the_straight_run_before_the_split() {
    for k in 1..=(MAX_SEGMENTS as u32) {
        let r = rules(&sh(2.0, k as f32, 0.0, 0.0));
        assert_eq!(r.matches("F(s)").count(), k as usize, "k = {k}: {r}");
        // E eles vêm ANTES do primeiro ramo — um tronco depois da bifurcação é outra planta.
        let first_bracket = r.find('[').expect("ha ramo");
        assert_eq!(
            r[..first_bracket].matches("F(s)").count(),
            k as usize,
            "os passos tem de vir antes do ramo: {r}"
        );
    }
}

/// **O `Bend` só aparece quando existe, e aparece uma vez por segmento.**
///
/// ⚠️ A ausência a zero não é economia de texto: é o que mantém a gramática assada LEGÍVEL
/// para quem a vai ler pela primeira vez. Um `+(bend)` a somar zero em cada `F` é ruído com o
/// aspecto de intenção.
#[test]
fn the_bend_shows_up_once_per_segment_and_only_when_it_is_not_zero() {
    assert!(!rules(&sh(2.0, 3.0, 0.0, 0.0)).contains("bend"));
    let curved = rules(&sh(2.0, 3.0, 0.0, 12.0));
    assert_eq!(curved.matches("+(bend)").count(), 3, "{curved}");
    // O sinal vive no PARAM, não no símbolo: `+(-12)` verga para o outro lado sozinho.
    let other = rules(&sh(2.0, 3.0, 0.0, -12.0));
    assert_eq!(other.matches("+(bend)").count(), 3, "{other}");
}

/// ⭐ **A VARIAÇÃO é estocástica: três regras, e os pesos somam UM.**
///
/// ⚠️ **Três e não duas** — ver [`super::rules`]. E a soma tem de fechar em `1`, senão a
/// escolha de regra fica enviesada de uma maneira que ninguém autorou: o `pick` normaliza
/// pelo total, então um total de `1,3` faria o `Variation` mexer também na PROPORÇÃO entre
/// as duas alternativas.
/// ⚠️⚠️ **A 1.ª redacção deste gate ESCREVIA O PRÓPRIO ORÁCULO, e ficava verde exactamente
/// no caso partido** (auditoria de 2026-08-29). Ela lia os pesos do texto com um
/// `str::parse::<f32>()` seu, e em `v = 1,0` o texto somava `1,0`
/// (`0.000 + 0.500 + 0.500`) — enquanto **o motor** somava `2,0`, porque o `(0.000)` não
/// passava a guarda `v > 0.0` e virava o neutro. *Dois leitores do mesmo texto, e o gate
/// escolheu o que não está no produto.*
///
/// ⇒ O oráculo passou a ser [`crate::probe_rule_weights`], que chama o `parse_rules` do
/// produto. E a contagem deixou de ser `3` fixo: em `v = 1,0` a regra nominal **não é
/// escrita**, porque um peso zero é a ausência da regra.
#[test]
fn the_weights_the_engine_reads_always_close_at_one() {
    assert_eq!(rules(&factory()).matches("A(s) ->").count(), 1);
    for v in [0.1f32, 0.5, 0.999, 1.0] {
        let r = rules(&sh(2.0, 1.0, v, 0.0));
        let w = crate::probe_rule_weights(&r);
        assert!(
            w.len() >= 2,
            "v = {v}: o motor le' {} regras em {r}",
            w.len()
        );
        let total: f32 = w.iter().sum();
        assert!(
            (total - 1.0).abs() < 2e-3,
            "v = {v}: o MOTOR soma {total} (o texto e' {r})"
        );
        // ⚠️ E nenhuma das que o motor lê pode ter peso zero — um intervalo vazio no sorteio
        // é uma regra que o texto anuncia e que nunca sai.
        assert!(w.iter().all(|x| *x > 0.0), "v = {v}: peso zero em {r}");
        // ⭐⭐ **E O TEXTO NÃO ANUNCIA UMA REGRA QUE O MOTOR DESCARTA.**
        //
        // ⚠️ Sem esta metade o gate media só o motor, e uma mutação que devolvesse o
        // produtor a escrever `(0.000)` **SOBREVIVIA**: o parser (que hoje falha fechado)
        // limpava a sujidade e a soma continuava a fechar. Mas o texto assado é o que o
        // artista vai LER em `Grammar` para aprender a notação — uma regra lá dentro que
        // nunca sai ensina-lhe uma coisa falsa. *Contar o que foi feito não é contar o que
        // foi entregue.*
        assert_eq!(
            r.matches("A(s) ->").count(),
            w.len(),
            "v = {v}: o texto anuncia {} regras e o motor le' {} — {r}",
            r.matches("A(s) ->").count(),
            w.len()
        );
    }
}

/// **As duas alternativas ABREM e FECHAM o leque** — nunca as duas para o mesmo lado.
///
/// ⚠️ Sem isto o `Variation` seria uma segunda resposta a *«quão aberto»*, que é a pergunta
/// que o slider *Angle* já responde — e o artista veria a planta abrir ao mexer nos dois.
#[test]
fn the_two_alternatives_open_and_close_the_fan_around_the_nominal_angle() {
    let r = rules(&sh(2.0, 1.0, 0.4, 0.0));
    let coeffs: Vec<f32> = r
        .split("+(angle*")
        .skip(1)
        .map(|t| {
            t.split(')')
                .next()
                .expect("fecha")
                .parse::<f32>()
                .expect("numero")
        })
        .collect();
    assert_eq!(coeffs.len(), 2, "duas alternativas: {r}");
    assert!(
        coeffs.iter().any(|c| *c > 1.0) && coeffs.iter().any(|c| *c < 1.0),
        "uma tem de abrir e a outra fechar: {coeffs:?}"
    );
}

/// **Fora de faixa é COAGIDO, nunca uma gramática partida.**
///
/// ⚠️ Um param pode chegar aqui conduzido por um FIO, e um fio não conhece a faixa do
/// slider. `Branches = 1e9` tem de dar o leque máximo — não um laço de mil milhões de voltas
/// a construir uma `String`.
#[test]
fn a_wire_driven_param_out_of_range_is_coerced_and_never_explodes() {
    for (b, s) in [
        (1e9f32, 1e9f32),
        (-5.0, -5.0),
        (f32::NAN, f32::NAN),
        (f32::INFINITY, f32::NEG_INFINITY),
    ] {
        let r = rules(&sh(b, s, 0.0, 0.0));
        let shoots = r.matches("A(s*length_scale)").count();
        assert!(
            (1..=MAX_BRANCHES as usize).contains(&shoots),
            "b = {b}: {shoots} rebentos ({r})"
        );
        let steps = r.matches("F(s)").count();
        assert!((1..=MAX_SEGMENTS as usize).contains(&steps), "s = {s}: {r}");
    }
}

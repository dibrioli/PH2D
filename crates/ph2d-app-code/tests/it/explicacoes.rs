//! I8 (Onda 2, `docs/CodeEditor/03_onda2.md`): as explicações escritas à mão. O rato parado no ponto
//! da calha mostra a explicação de cada problema da linha; parado sobre o culpado sublinhado, a
//! mensagem CRUA do Luau. Cada explicação que um balão abre conta no registo local.

use crate::Mesa;
use crate::assistente::{Plano, com_falso, converge, posicao};
use crate::completar::{T0, quadro_em};
use crate::dicas::{aponta, ate, nunca};
use ph2d_script::ScriptId;
use serde_json::{Value, json};

const PAUSA: u128 = 500_000_000;

/// O `=` do smoke 2 (`ferramentas/smoke/script_quebrado.luau`).
const QUEBRADO: &str = "local x = = 1\nprint(x)\n";
const CRU: &str = "SyntaxError: Expected identifier when parsing expression, got '='";

/// O servidor acusa o segundo `=` como o `luau-lsp` 1.70.1 (I0c).
fn acusa_igual(s: &str) -> Vec<Value> {
    s.find("= =")
        .map(|b| b + 2)
        .map(|b| {
            json!({ "range": { "start": posicao(s, b), "end": posicao(s, b + 1) },
                    "severity": 1, "code": 1014, "message": CRU })
        })
        .into_iter()
        .collect()
}

/// As linhas do balão pintado no último quadro (vazio = nenhum).
fn balao(m: &mut Mesa, id: ScriptId) -> Vec<String> {
    m.docs()
        .hints_painted(id)
        .0
        .map(|p| p.lines)
        .unwrap_or_default()
}

fn marcado(m: &mut Mesa, id: ScriptId) -> Vec<String> {
    m.docs()
        .hints_painted(id)
        .0
        .map(|p| p.marked)
        .unwrap_or_default()
}

/// ⭐ A prova do I8 sobre o `=` do smoke 2: a calha explica (título marcado e o porquê), o culpado
/// mostra o cru; nenhum dos dois antes da pausa; só a explicação conta como vista.
#[test]
fn a_calha_explica_e_o_sublinhado_mostra_o_cru() {
    let (mut m, ids, visto) = com_falso(
        &[("a.luau", QUEBRADO)],
        Plano {
            regra: Some(acusa_igual),
            ..Plano::default()
        },
    );
    let id = ids[0];
    m.pede(id);
    converge(&mut m, &visto, "abrir");
    ate(&mut m, id, T0, "o servidor não acusou o =", |m| {
        m.docs().problems_of(id).iter().any(|p| p.message == CRU)
    });
    let p = m
        .docs()
        .problems_of(id)
        .into_iter()
        .find(|p| p.message == CRU)
        .unwrap();
    let e = p.explanation.expect("a classe do = tem explicação");
    assert_eq!(e.title, "A value is missing before \u{201c}=\u{201d}");

    // O ponto da calha.
    let t1 = T0 + 10 * PAUSA;
    quadro_em(&mut m, t1);
    m.pinta(id);
    let (x, y) = m.docs().problem_dot(id, 0).expect("o ponto da linha 1");
    m.rato(id, x, y);
    nunca(&mut m, id, t1 + PAUSA - 1, "abriu antes da pausa", |m| {
        balao(m, id).is_empty()
    });
    ate(&mut m, id, t1 + PAUSA, "a calha não explicou", |m| {
        !balao(m, id).is_empty()
    });
    let linhas = balao(&mut m, id);
    let titulo = marcado(&mut m, id);
    assert_eq!(titulo.join(" "), e.title, "só o título marcado");
    assert_eq!(
        linhas[..titulo.len()],
        titulo[..],
        "o título primeiro: {linhas:?}"
    );
    assert!(
        linhas.join(" ").contains("Luau wanted a value here"),
        "o porquê: {linhas:?}"
    );
    assert!(
        !linhas.join(" ").contains("Expected identifier"),
        "o cru na calha: {linhas:?}"
    );
    assert_eq!(m.docs().seen_kinds(), ["expected_expression"]);

    // O culpado sublinhado (o segundo `=`).
    let t2 = t1 + 10 * PAUSA;
    quadro_em(&mut m, t2);
    aponta(&mut m, id, QUEBRADO.find("= =").unwrap() + 2);
    nunca(&mut m, id, t2 + PAUSA - 1, "o cru antes da pausa", |m| {
        balao(m, id).is_empty()
    });
    ate(
        &mut m,
        id,
        t2 + PAUSA,
        "o sublinhado não mostrou o cru",
        |m| !balao(m, id).is_empty(),
    );
    let linhas = balao(&mut m, id).join(" ");
    assert!(linhas.contains(CRU), "o cru: {linhas}");
    assert!(
        !linhas.contains(&e.title),
        "a explicação no balão do cru: {linhas}"
    );
    assert!(marcado(&mut m, id).is_empty(), "o cru não tem título");
    assert_eq!(
        m.docs().seen_kinds(),
        ["expected_expression"],
        "o cru não conta como explicação vista"
    );
}

/// Sem assistente, o aviso NOSSO (o campo do `self`, I7) explica-se na calha: o balão dos problemas
/// não depende do servidor. Controlo: uma linha sem problema não abre balão nenhum.
#[test]
fn sem_assistente_a_calha_explica_o_aviso_nosso() {
    let texto = "ph2d.property(\"amplitude\", 1.0)\n\
                 function update(self, dt)\n  \
                 local d = self.ampiltude * dt\n\
                 end\n";
    let (mut m, ids) = Mesa::new(&[("a.luau", texto)]);
    let id = ids[0];
    m.pede(id);
    quadro_em(&mut m, T0);
    m.pinta(id);
    assert!(
        m.docs().problems_of(id).iter().any(|p| p.line == 2),
        "o aviso do ampiltude (linha 3)"
    );
    // Controlo: a linha 2 não tem problema.
    let t1 = T0 + 10 * PAUSA;
    quadro_em(&mut m, t1);
    m.pinta(id);
    let livre = m.docs().problem_dot(id, 1).expect("a linha 2 à vista");
    m.rato(id, livre.0, livre.1);
    nunca(
        &mut m,
        id,
        t1 + 4 * PAUSA,
        "balão numa linha sem problema",
        |m| balao(m, id).is_empty(),
    );
    // A linha do aviso.
    let t2 = t1 + 10 * PAUSA;
    quadro_em(&mut m, t2);
    let (x, y) = m.docs().problem_dot(id, 2).expect("a linha 3 à vista");
    m.rato(id, x, y);
    ate(
        &mut m,
        id,
        t2 + PAUSA,
        "a calha não explicou sem assistente",
        |m| !balao(m, id).is_empty(),
    );
    let linhas = balao(&mut m, id);
    // O título partido nos espaços conta como UM título marcado.
    assert_eq!(
        marcado(&mut m, id).join(" "),
        "self.ampiltude is never set \u{2014} did you mean amplitude?"
    );
    assert!(
        linhas
            .join(" ")
            .contains("Alt+Enter changes it to amplitude"),
        "{linhas:?}"
    );
    assert_eq!(m.docs().seen_kinds(), ["unknown_field_near"]);
}

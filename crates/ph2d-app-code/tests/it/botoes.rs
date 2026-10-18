//! I8b (Onda 2, `docs/CodeEditor/03_onda2.md`): as correcções como BOTÕES no balão de um problema.
//! Cada botão é uma acção que o Alt+Enter daria naquele sítio (a mesma porta, `actions_at`); clicar
//! aplica-a num só passo de undo e o balão fecha. Sem servidor ficam as nossas; nada a corrigir, nenhum
//! botão. O botão nasce neutro: o realce só com o rato por cima.
//!
//! ⛔ Toda asserção de «NÃO aparece» dá tempo REAL a uma resposta chegar ([`nunca`]).

use crate::Mesa;
use crate::acoes::{TEXTO, alt_enter, com_servidor, sem_assistente, texto};
use crate::assistente::{Plano, com_falso, converge, tecla};
use crate::completar::{T0, quadro_em};
use crate::definicao::clica_em;
use crate::dicas::{aponta, ate, balao as pairar, nunca};
use crate::explicacoes::{QUEBRADO, acusa_igual};
use ph2d_a11y::Role;
use ph2d_editor_core::doc_surface::{DocumentSurface, Key};
use ph2d_host::Modifiers;
use ph2d_script::ScriptId;
use std::time::Duration;

const PAUSA: u128 = 500_000_000;

/// Os botões do balão pintado no último quadro: (centro, rótulo).
fn botoes(m: &mut Mesa, id: ScriptId) -> Vec<((f32, f32), String)> {
    m.docs()
        .hints_painted(id)
        .0
        .map(|p| {
            p.buttons
                .into_iter()
                .map(|(r, l)| (crate::centro(r), l))
                .collect()
        })
        .unwrap_or_default()
}

fn rotulos(m: &mut Mesa, id: ScriptId) -> Vec<String> {
    botoes(m, id).into_iter().map(|b| b.1).collect()
}

fn quente(m: &mut Mesa, id: ScriptId) -> Option<usize> {
    m.docs().hints_painted(id).0.and_then(|p| p.hot)
}

fn aberto(m: &mut Mesa, id: ScriptId) -> bool {
    m.docs().hints_painted(id).0.is_some()
}

fn ctrl_z(m: &mut Mesa, id: ScriptId) {
    tecla(m, id, Key::Char('z'), crate::ctrl(), None, "");
}

/// O texto que o Alt+Enter + Enter deixa com o cursor no byte `em`.
fn pelo_alt_enter(m: &mut Mesa, id: ScriptId, em: usize) -> String {
    clica_em(m, id, em, Modifiers::default());
    alt_enter(m, id);
    ate(m, id, T0, "as acções não abriram", |m| {
        m.docs().actions_of(id).is_some()
    });
    tecla(m, id, Key::Enter, Modifiers::default(), None, "");
    texto(m, id)
}

/// ⭐ A prova do I8b, sem servidor: o balão do `self.ampiltude` sublinhado traz o botão «Did you mean
/// amplitude?»; ele nasce neutro, acende com o rato e o balão fica aberto; o clique dá o MESMO texto
/// que o Alt+Enter, o balão fecha, e UM Ctrl+Z volta ao de antes.
#[test]
fn o_botao_do_balao_corrige_como_o_alt_enter_num_passo() {
    let errado = TEXTO.find("ampiltude").unwrap();
    let (mut a, ida) = sem_assistente(TEXTO);
    let esperado = pelo_alt_enter(&mut a, ida, errado + 3);
    assert_eq!(esperado, TEXTO.replace("ampiltude", "amplitude"));

    let (mut m, id) = sem_assistente(TEXTO);
    let t1 = T0 + 10 * PAUSA;
    quadro_em(&mut m, t1);
    m.pinta(id);
    aponta(&mut m, id, errado + 3);
    nunca(&mut m, id, t1 + PAUSA - 1, "o botão antes da pausa", |m| {
        botoes(m, id).is_empty()
    });
    ate(
        &mut m,
        id,
        t1 + PAUSA,
        "o balão não trouxe o botão",
        |m| !botoes(m, id).is_empty(),
    );
    assert_eq!(rotulos(&mut m, id), ["Did you mean amplitude?"]);
    assert_eq!(quente(&mut m, id), None, "nasce neutro");

    // O realce anda no relógio de animação do app (o do hero), como o dos botões dos painéis: o
    // quadro da família anima-o, o tique do hero avança-o.
    let t2 = t1 + 2 * PAUSA;
    let vivo = |m: &mut Mesa| {
        quadro_em(m, t2);
        m.pinta(id);
        m.docs()
            .hints_painted(id)
            .0
            .map(|p| p.live)
            .unwrap_or_default()
    };
    assert_eq!(vivo(&mut m), [0.0], "em repouso");
    let (x, y) = botoes(&mut m, id)[0].0;
    m.rato(id, x, y);
    quadro_em(&mut m, t2);
    m.pinta(id);
    assert_eq!(quente(&mut m, id), Some(0), "o rato por cima acende-o");
    assert!(aberto(&mut m, id), "o rato no balão não o fecha");
    let _ = vivo(&mut m);
    m.hero.motion.advance(0.016);
    let meio = vivo(&mut m)[0];
    assert!(0.0 < meio && meio < 1.0, "acende aos poucos: {meio}");
    m.hero.motion.advance(2.0);
    assert_eq!(vivo(&mut m), [1.0], "e assenta aceso");

    // O leitor de ecrã: o botão é filho do balão.
    let nodes = m.docs().a11y(id.0);
    let balao = nodes
        .iter()
        .find(|n| n.1.role() == Role::Tooltip)
        .expect("o balão");
    let botao = nodes
        .iter()
        .find(|n| n.1.role() == Role::Button && n.1.label() == Some("Did you mean amplitude?"))
        .expect("o botão no leitor de ecrã");
    assert!(
        balao.1.children().contains(&botao.0.into()),
        "filho do balão"
    );

    m.clica(id, x, y, Modifiers::default());
    assert_eq!(texto(&mut m, id), esperado, "o mesmo texto que o Alt+Enter");
    m.pinta(id);
    assert!(!aberto(&mut m, id), "aplicada, o balão fecha");
    ctrl_z(&mut m, id);
    assert_eq!(
        texto(&mut m, id),
        TEXTO,
        "um Ctrl+Z desfaz a correcção inteira"
    );
}

/// O ponto da calha da linha leva os mesmos botões que o sublinhado (os do culpado da linha).
#[test]
fn o_ponto_da_calha_leva_os_mesmos_botoes() {
    let (mut m, id) = sem_assistente(TEXTO);
    let linha = TEXTO[..TEXTO.find("ampiltude").unwrap()]
        .matches('\n')
        .count();
    let t1 = T0 + 10 * PAUSA;
    quadro_em(&mut m, t1);
    m.pinta(id);
    let (x, y) = m.docs().problem_dot(id, linha).expect("o ponto da linha");
    m.rato(id, x, y);
    ate(
        &mut m,
        id,
        t1 + PAUSA,
        "a calha não trouxe o botão",
        |m| !botoes(m, id).is_empty(),
    );
    assert_eq!(rotulos(&mut m, id), ["Did you mean amplitude?"]);
    let (bx, by) = botoes(&mut m, id)[0].0;
    m.clica(id, bx, by, Modifiers::default());
    assert_eq!(texto(&mut m, id), TEXTO.replace("ampiltude", "amplitude"));
}

/// Com o servidor: o global errado traz as mesmas acções do Alt+Enter, pela mesma ordem (a
/// candidata da completion dele e a quickfix que ecoa o diagnóstico), e o 1.º botão corrige.
#[test]
fn com_o_servidor_os_botoes_sao_os_do_alt_enter() {
    let errado = TEXTO.find("pirnt").unwrap();
    let (mut a, ida, _) = com_servidor(Duration::ZERO);
    clica_em(&mut a, ida, errado + 2, Modifiers::default());
    alt_enter(&mut a, ida);
    ate(&mut a, ida, T0, "as acções não chegaram", |m| {
        m.docs().actions_of(ida).is_some()
    });
    let titulos = a.docs().actions_of(ida).unwrap().0;
    assert_eq!(titulos, ["Did you mean print?", "Do server"]);

    let (mut m, id, _) = com_servidor(Duration::ZERO);
    let t1 = T0 + 10 * PAUSA;
    quadro_em(&mut m, t1);
    m.pinta(id);
    aponta(&mut m, id, errado + 2);
    ate(
        &mut m,
        id,
        t1 + PAUSA,
        "os botões do servidor não chegaram",
        |m| botoes(m, id).len() == 2,
    );
    assert_eq!(rotulos(&mut m, id), titulos);
    let (x, y) = botoes(&mut m, id)[0].0;
    m.clica(id, x, y, Modifiers::default());
    assert_eq!(texto(&mut m, id), TEXTO.replace("pirnt", "print"));
    ctrl_z(&mut m, id);
    assert_eq!(texto(&mut m, id), TEXTO);
}

/// Nada a corrigir ⇒ nenhum botão: o `=` a mais tem explicação mas nenhuma acção (o servidor não dá
/// quickfix), e uma palavra sem problema mostra o que o assistente diz dela, sem botões.
#[test]
fn nada_a_corrigir_nenhum_botao() {
    let (mut m, ids, visto) = com_falso(
        &[("a.luau", QUEBRADO)],
        Plano {
            regra: Some(acusa_igual),
            pairar: Some(pairar),
            ..Plano::default()
        },
    );
    let id = ids[0];
    m.pede(id);
    converge(&mut m, &visto, "abrir");
    ate(&mut m, id, T0, "o servidor não acusou o =", |m| {
        !m.docs().problems_of(id).is_empty()
    });
    let t1 = T0 + 10 * PAUSA;
    quadro_em(&mut m, t1);
    m.pinta(id);
    let (x, y) = m.docs().problem_dot(id, 0).expect("o ponto da linha 1");
    m.rato(id, x, y);
    ate(&mut m, id, t1 + PAUSA, "a calha não explicou", |m| {
        aberto(m, id)
    });
    nunca(
        &mut m,
        id,
        t1 + 4 * PAUSA,
        "um botão sem correcção",
        |m| botoes(m, id).is_empty(),
    );

    let t2 = t1 + 10 * PAUSA;
    quadro_em(&mut m, t2);
    aponta(&mut m, id, QUEBRADO.find("print").unwrap() + 1);
    ate(&mut m, id, t2 + PAUSA, "a palavra não teve balão", |m| {
        aberto(m, id)
    });
    nunca(
        &mut m,
        id,
        t2 + 4 * PAUSA,
        "um botão numa palavra sem problema",
        |m| botoes(m, id).is_empty(),
    );
}

/// Clicar no texto do balão (fora dos botões) não põe o cursor por baixo dele nem o fecha.
#[test]
fn o_clique_no_texto_do_balao_fica_nele() {
    let errado = TEXTO.find("ampiltude").unwrap();
    let (mut m, id) = sem_assistente(TEXTO);
    let t1 = T0 + 10 * PAUSA;
    quadro_em(&mut m, t1);
    m.pinta(id);
    aponta(&mut m, id, errado + 3);
    ate(&mut m, id, t1 + PAUSA, "o balão não abriu", |m| {
        !botoes(m, id).is_empty()
    });
    let r = m.docs().hints_painted(id).0.unwrap().rect;
    let antes = crate::definicao::cursor(&mut m, id);
    let (x, y) = (r.x + r.w - 2.0, r.y + 2.0);
    m.clica(id, x, y, Modifiers::default());
    m.pinta(id);
    assert!(aberto(&mut m, id), "o balão fica");
    assert_eq!(
        crate::definicao::cursor(&mut m, id),
        antes,
        "o cursor não mexe"
    );
    assert_eq!(texto(&mut m, id), TEXTO);
}

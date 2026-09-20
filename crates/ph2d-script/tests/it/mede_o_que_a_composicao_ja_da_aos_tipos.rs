//! ⭐⭐⭐ **A PERGUNTA DE ANTES DA 1.ª LINHA dos TIPOS DE PROPRIEDADE** (item aberto do handoff do
//! `#16`): *a composição de hoje já os exprime?*
//!
//! O `ScriptValue` tem **três** variantes (número · sim/não · texto), e a lista aberta pede
//! `Vector2`, `Color` e **enum**. ⚠️ **Medir antes de construir**, porque as três respostas são
//! DIFERENTES e só uma delas é uma capacidade em falta.
//!
//! ⚠️ **Sonda, não gate.** Ela corre com `--ignored` e IMPRIME — o que fica no repo é a TABELA.

use ph2d_script::props::{PropDecl, PropHint, ScriptValue, resolve};
use std::collections::BTreeMap;

fn decl(nome: &str, v: ScriptValue) -> PropDecl {
    PropDecl {
        name: nome.into(),
        default: v,
        hint: PropHint::default(),
    }
}

fn proprio(pares: &[(&str, ScriptValue)]) -> BTreeMap<String, ScriptValue> {
    pares
        .iter()
        .map(|(k, v)| ((*k).to_owned(), v.clone()))
        .collect()
}

#[test]
#[ignore = "sonda: imprime a medicao que decide o ambito da wave"]
fn mede_o_que_a_composicao_ja_da_aos_tipos() {
    println!("== A) VECTOR2: dois numeros ==");
    let decls = vec![
        decl("pos_x", ScriptValue::Number(0.0)),
        decl("pos_y", ScriptValue::Number(0.0)),
    ];
    let r = resolve(
        Some(&decls),
        &proprio(&[("pos_x", ScriptValue::Number(3.0))]),
    );
    println!(
        "   declaracoes: {} · fileiras no painel: {} · o artista PODE exprimi-lo: SIM",
        decls.len(),
        r.values.len()
    );
    println!("   => CAPACIDADE existe; falta a AFICAO (uma fileira em vez de duas).");

    println!("\n== B) COLOR: tres/quatro numeros ==");
    let decls: Vec<PropDecl> = ["r", "g", "b", "a"]
        .iter()
        .map(|n| decl(n, ScriptValue::Number(1.0)))
        .collect();
    let r = resolve(Some(&decls), &proprio(&[]));
    println!(
        "   declaracoes: {} · fileiras: {} · o artista PODE exprimi-lo: SIM",
        decls.len(),
        r.values.len()
    );
    println!("   => CAPACIDADE existe; falta a AMOSTRA de cor (quatro campos nao sao uma cor).");

    println!("\n== C) ENUM: um texto ==");
    let decls = vec![decl("mode", ScriptValue::Text("fast".into()))];
    // ⭐ O artista escreve o valor A' MAO. Um erro de escrita e' um valor VALIDO para o modelo.
    let r = resolve(
        Some(&decls),
        &proprio(&[("mode", ScriptValue::Text("fst".into()))]),
    );
    let linha = &r.values[0];
    println!(
        "   o objecto guarda {:?} · o painel mostra {:?} · e' orfao? {}",
        "fst",
        linha.value,
        r.orphans.len()
    );
    println!(
        "   => CAPACIDADE existe e a AFICAO MENTE: `fst` nao e' recusado nem nomeado — ele CHEGA\n   \
         ao script, que compara com `fast` e cai no ramo errado EM SILENCIO."
    );

    println!("\n== veredito ==");
    println!(
        "   as tre^s tem a capacidade pela composicao, e so' UMA tem um defeito de CORRECCAO:\n   \
         o enum. Vector2 e Color custam fileiras feias; o enum custa um valor ERRADO que ninguem\n   \
         acusa — a familia do «aceita e mente» que esta casa ja' pagou no `lattice`, no\n   \
         `kaleidoscope` e no `iterations` do colisor.\n   \
         => a wave comeca pelo ENUM, e ele NAO precisa de variante nova (e' um `Text` com a lista\n   \
         na PISTA), logo custa ZERO no fio e ZERO no schema."
    );
}

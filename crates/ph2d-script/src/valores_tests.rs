//! Gates da FORMA de um `vec2`/`color` do lado do Luau.

use super::{COR, MARCA, VEC2, tabela_de, tabela_tipada};
use crate::props::ScriptValue;
use mlua::{Lua, Value};

fn tabela(lua: &Lua, v: &ScriptValue) -> mlua::Table {
    match tabela_de(lua, v).expect("construir") {
        Some(Value::Table(t)) => t,
        outro => panic!("esperava uma tabela, veio {outro:?}"),
    }
}

/// ⭐ **Ida e volta pela mesma porta** — o que a casa escreve, a casa lê.
///
/// **Mutação que deve sangrar:** trocar `"y"` por `"x"` num dos dois lados.
#[test]
fn o_que_a_casa_escreve_a_casa_le() {
    let lua = Lua::new();
    for v in [
        ScriptValue::Vec2([1.5, -2.0]),
        ScriptValue::Vec2([0.0, 0.0]),
        ScriptValue::Color([1.0, 0.25, 0.0, 0.5]),
        ScriptValue::Color([0.0, 0.0, 0.0, 1.0]),
    ] {
        let t = tabela(&lua, &v);
        assert_eq!(tabela_tipada(&t), Some(v.clone()), "ida e volta de {v:?}");
    }
}

/// ⛔ **Os três tipos simples NÃO são tabela** — o caminho de omissão fica ao bit.
#[test]
fn os_tres_tipos_simples_nao_viram_tabela() {
    let lua = Lua::new();
    for v in [
        ScriptValue::Number(2.0),
        ScriptValue::Bool(true),
        ScriptValue::Text("oi".into()),
    ] {
        assert!(
            tabela_de(&lua, &v).expect("construir").is_none(),
            "{v:?} nao pode virar tabela: o caminho de omissao mudaria"
        );
    }
}

/// ⛔⛔ **Uma tabela que NÃO se declara não é um valor** — a desambiguação é explícita.
///
/// ⚠️ A metade que importa é a LISTA: `{1, 0, 0}` lê-se igual a uma cor vermelha e a uma posição
/// com lixo no fim, e adivinhar pelo comprimento é o palpite que o handoff proibiu.
#[test]
fn uma_tabela_sem_marca_nao_e_um_valor() {
    let lua = Lua::new();
    let lista = lua.create_table().expect("tabela");
    lista.set(1, 1.0).expect("set");
    lista.set(2, 0.0).expect("set");
    lista.set(3, 0.0).expect("set");
    assert_eq!(tabela_tipada(&lista), None, "uma lista nao se declara");

    // ⭐⭐⭐ **O caso que DISCRIMINA, e que faltava:** um registo com os campos TODOS certos e sem
    // a marca. ⛔⛔ Sem ele este gate era verde sobre uma marca **assumida**: uma prova de mutação
    // trocou o `ok()?` por *«se faltar, é um vec2»* e ele **SOBREVIVEU** — porque cada uma das
    // outras fixturas era recusada por um SEGUNDO motivo (faltavam-lhe os campos).
    // *Uma fixtura recusada duas vezes não afirma nenhuma das duas recusas.*
    let registo_sem_marca = lua.create_table().expect("tabela");
    registo_sem_marca.set("x", 1.0).expect("set");
    registo_sem_marca.set("y", 2.0).expect("set");
    assert_eq!(
        tabela_tipada(&registo_sem_marca),
        None,
        "um registo com os campos certos e SEM marca foi lido como um valor"
    );

    let marca_errada = lua.create_table().expect("tabela");
    marca_errada.set(MARCA, "quaternion").expect("set");
    assert_eq!(tabela_tipada(&marca_errada), None);

    // Com a marca certa e um campo a menos, também não: metade de um `vec2` não é um `vec2`.
    let meio = lua.create_table().expect("tabela");
    meio.set(MARCA, VEC2).expect("set");
    meio.set("x", 1.0).expect("set");
    assert_eq!(tabela_tipada(&meio), None, "falta o `y`");
}

/// ⛔ **Uma string numérica NÃO é um número** — a cerca contra a coerção do `FromLua`.
///
/// ⚠️ **Sem ela o `get::<f64>` aceitaria `"1"`**, e um default que veio de uma string entraria no
/// documento com a aritmética do Luau a decidir o valor. É a mesma cerca que o `read_decl` já põe
/// no `min`/`max`/`step`.
///
/// **Mutação que deve sangrar:** trocar o `match` do [`super::numero`] por `t.get::<f64>(chave)`.
#[test]
fn uma_string_numerica_nao_e_um_numero() {
    let lua = Lua::new();
    let t = lua.create_table().expect("tabela");
    t.set(MARCA, COR).expect("set");
    t.set("r", "1").expect("set");
    t.set("g", 0.0).expect("set");
    t.set("b", 0.0).expect("set");
    t.set("a", 1.0).expect("set");
    assert_eq!(
        tabela_tipada(&t),
        None,
        "uma string numerica passou por um canal de cor"
    );
}

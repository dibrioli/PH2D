//! A colheita das declarações e o isolamento entre scripts — sobre a VM REAL do `ScriptHost`
//! (sandbox ligada), nunca sobre uma VM de teste sem ela: a sandbox é precisamente o que muda o
//! sítio onde um `function update` escreve.

use super::*;
use crate::ScriptHost;
use crate::props::{DeclError, ScriptValue};

fn host() -> ScriptHost {
    ScriptHost::new().expect("a VM da casa arranca")
}

#[test]
fn as_declaracoes_saem_na_ordem_das_chamadas_com_tipo_e_pistas() {
    let h = host();
    let m = load_module(
        h.runtime().lua(),
        "bob.luau",
        r#"
ph2d.property("zeta", 1.5, { min = 0, max = 10, step = 0.5 })
ph2d.property("alpha", true)
ph2d.property("label", "oi")
ph2d.property("inteiro", 3)
"#,
    )
    .expect("carrega");
    let nomes: Vec<&str> = m.decls.iter().map(|d| d.name.as_str()).collect();
    assert_eq!(nomes, ["zeta", "alpha", "label", "inteiro"], "Q7");
    assert_eq!(m.decls[0].default, ScriptValue::Number(1.5));
    assert_eq!(m.decls[0].hint.min, Some(0.0));
    assert_eq!(m.decls[0].hint.max, Some(10.0));
    assert_eq!(m.decls[0].hint.step, Some(0.5));
    assert_eq!(m.decls[1].default, ScriptValue::Bool(true));
    assert_eq!(m.decls[2].default, ScriptValue::Text("oi".into()));
    assert_eq!(m.decls[3].default, ScriptValue::Number(3.0));
}

#[test]
fn um_script_sem_declaracoes_carrega_com_a_lista_vazia() {
    let h = host();
    let m = load_module(h.runtime().lua(), "vazio.luau", "local x = 1").expect("carrega");
    assert!(m.decls.is_empty());
    assert!(m.hook("update").is_none());
}

#[test]
fn um_erro_de_sintaxe_e_nomeado_como_tal() {
    let h = host();
    let e = load_module(h.runtime().lua(), "partido.luau", "function update(")
        .err()
        .expect("recusa");
    assert!(matches!(e, ModuleError::Syntax(_)), "{e:?}");
}

#[test]
fn uma_declaracao_recusada_para_o_topo_e_diz_qual() {
    let h = host();
    let lua = h.runtime().lua();
    for (src, esperado) in [
        (
            r#"ph2d.property("a", 1) ph2d.property("a", 2)"#,
            DeclError::Twice("a".into()),
        ),
        (
            r#"ph2d.property("id", 1)"#,
            DeclError::Reserved("id".into()),
        ),
        (r#"ph2d.property("2x", 1)"#, DeclError::BadName("2x".into())),
        (
            r#"ph2d.property("t", {})"#,
            DeclError::BadDefault("t".into()),
        ),
        (
            r#"ph2d.property("n", nil)"#,
            DeclError::BadDefault("n".into()),
        ),
        (
            r#"ph2d.property("s", 1, { mni = 0 })"#,
            DeclError::UnknownOption {
                name: "s".into(),
                option: "mni".into(),
            },
        ),
        (
            r#"ph2d.property("b", true, { max = 1 })"#,
            DeclError::BadHint("b".into()),
        ),
        (
            r#"ph2d.property("s", 1, { min = "0" })"#,
            DeclError::BadHint("s".into()),
        ),
    ] {
        let e = load_module(lua, "mau.luau", src).err().expect(src);
        assert_eq!(e, ModuleError::Decl(esperado), "{src}");
    }
}

#[test]
fn o_resto_do_topo_nao_corre_depois_de_uma_recusa() {
    let h = host();
    let e = load_module(
        h.runtime().lua(),
        "meio.luau",
        r#"
ph2d.property("a", 1)
ph2d.property("a", 2)
function update() end
"#,
    );
    assert!(matches!(e, Err(ModuleError::Decl(DeclError::Twice(_)))));
}

/// ⛔ **Fora do topo é ERRO** — e o balde de um carregamento anterior (mesmo um que falhou) não
/// fica para trás a aceitar declarações.
#[test]
fn declarar_dentro_de_um_gancho_e_recusado_mesmo_depois_de_uma_carga_falhada() {
    let h = host();
    let lua = h.runtime().lua();
    let _ = load_module(
        lua,
        "falha.luau",
        r#"ph2d.property("a", 1) ph2d.property("a", 1)"#,
    );
    let m = load_module(
        lua,
        "tarde.luau",
        r#"
function update()
  ph2d.property("tarde", 1)
end
"#,
    )
    .expect("o topo carrega");
    let err = m
        .hook("update")
        .expect("o gancho existe")
        .call::<()>(())
        .expect_err("declarar num gancho é recusado");
    assert!(
        runtime_message(&err).contains("must be called at the top"),
        "{err}"
    );
    assert!(m.decls.is_empty(), "e não entrou na lista");
}

#[test]
fn dois_scripts_nao_se_pisam() {
    let h = host();
    let lua = h.runtime().lua();
    let a = load_module(
        lua,
        "a.luau",
        "contador = 10\nfunction update() contador = contador + 1 return contador end",
    )
    .expect("a");
    let b = load_module(
        lua,
        "b.luau",
        "contador = 100\nfunction update() contador = contador + 1 return contador end",
    )
    .expect("b");
    let ua = a.hook("update").expect("a.update");
    let ub = b.hook("update").expect("b.update");
    assert_eq!(ua.call::<f64>(()).expect("a"), 11.0);
    assert_eq!(ub.call::<f64>(()).expect("b"), 101.0);
    assert_eq!(ua.call::<f64>(()).expect("a"), 12.0, "o global de A é de A");
    // E nenhum dos dois vazou para o ambiente partilhado.
    assert_eq!(
        lua.globals().get::<Value>("contador").expect("lê"),
        Value::Nil
    );
    assert_eq!(
        lua.globals().get::<Value>("update").expect("lê"),
        Value::Nil
    );
}

#[test]
fn um_modulo_le_a_biblioteca_mas_nao_reescreve_a_casa() {
    let h = host();
    let lua = h.runtime().lua();
    let m = load_module(
        lua,
        "mat.luau",
        "function update() return math.floor(2.7) end",
    )
    .expect("carrega");
    assert_eq!(
        m.hook("update").expect("u").call::<f64>(()).expect("corre"),
        2.0
    );
    let e = load_module(lua, "vandalo.luau", "ph2d.set = nil")
        .err()
        .expect("a casa é só de leitura");
    assert!(matches!(e, ModuleError::Runtime(_)), "{e:?}");
}

// ─── vec2 e color: os construtores ──────────────────────────────────────────────────────────────

/// ⭐⭐ **Os dois construtores atravessam a sandbox e dão o tipo certo** — com a `color` a assumir
/// opaco quando o quarto argumento falta.
#[test]
fn os_construtores_declaram_o_tipo_do_valor() {
    let h = host();
    let m = load_module(
        h.runtime().lua(),
        "tipos.luau",
        r#"
ph2d.property("offset", ph2d.vec2(1.5, -2))
ph2d.property("tint", ph2d.color(1, 0.5, 0))
ph2d.property("fade", ph2d.color(0, 0, 0, 0.25))
"#,
    )
    .expect("carrega");
    assert_eq!(m.decls[0].default, ScriptValue::Vec2([1.5, -2.0]));
    assert_eq!(m.decls[1].default, ScriptValue::Color([1.0, 0.5, 0.0, 1.0]));
    assert_eq!(
        m.decls[2].default,
        ScriptValue::Color([0.0, 0.0, 0.0, 0.25])
    );
}

/// ⭐⭐⭐ **O PORTÃO-COROA: a forma que o artista ESCREVE é a forma que ele LÊ em `self`.**
///
/// ⚠️⚠️ **Sem isto o artista tem duas formas para a mesma coisa** — escreveria a declaração como
/// uma lista e leria `self.offset.x` como um registo, e copiar uma para a outra produziria um
/// default que o painel não sabe pintar. A propriedade é comprada por [`crate::valores::tabela_de`]
/// ter **dois** chamadores (o construtor e o `to_lua` da cena), e é isso que este gate mede: a
/// tabela que o construtor devolveu ao SCRIPT e a que a casa escreve para o MESMO valor têm de ter
/// as mesmas chaves e os mesmos números.
///
/// **Mutações que devem sangrar:** trocar uma chave num dos lados · acrescentar um campo só num.
#[test]
fn a_forma_que_o_artista_escreve_e_a_forma_que_ele_le() {
    let h = host();
    let lua = h.runtime().lua();
    let m = load_module(
        lua,
        "forma.luau",
        r#"
ph2d.property("offset", ph2d.vec2(1.5, -2))
ph2d.property("tint", ph2d.color(1, 0.5, 0, 0.25))
"#,
    )
    .expect("carrega");
    for d in &m.decls {
        // O que o SCRIPT vê quando escreve o construtor.
        let do_artista: mlua::Table = match &d.default {
            ScriptValue::Vec2([x, y]) => lua
                .load(format!("return ph2d.vec2({x}, {y})"))
                .eval()
                .expect("o construtor corre"),
            ScriptValue::Color([r, g, b, a]) => lua
                .load(format!("return ph2d.color({r}, {g}, {b}, {a})"))
                .eval()
                .expect("o construtor corre"),
            outro => panic!("esperava um tipo novo, veio {outro:?}"),
        };
        // O que a CASA escreve em `self` para o mesmo valor — a mesma porta do `to_lua`.
        let mlua::Value::Table(da_casa) = crate::valores::tabela_de(lua, &d.default)
            .expect("constroi")
            .expect("os dois tipos novos SAO tabela")
        else {
            panic!("a porta devolveu algo que nao e' uma tabela");
        };
        let chaves = |t: &mlua::Table| {
            let mut ks: Vec<String> = t
                .clone()
                .pairs::<String, mlua::Value>()
                .map(|par| par.expect("par").0)
                .collect();
            ks.sort();
            ks
        };
        assert_eq!(
            chaves(&do_artista),
            chaves(&da_casa),
            "`{}`: o script e a casa escrevem CHAVES diferentes",
            d.name
        );
        for k in chaves(&do_artista) {
            let a: mlua::Value = do_artista.get(k.as_str()).expect("do artista");
            let b: mlua::Value = da_casa.get(k.as_str()).expect("da casa");
            assert_eq!(
                format!("{a:?}"),
                format!("{b:?}"),
                "`{}`.{k}: o script le' um valor e a casa escreve outro",
                d.name
            );
        }
    }
}

/// ⛔ **Uma tabela que não se declara continua a ser recusada** — e a lista `{1, 0, 0}`, que é a
/// forma que alguém escreveria a adivinhar, é o caso que importa: *adivinhar pelo comprimento é um
/// palpite, e um `{1, 0, 0}` lê-se igual a uma cor vermelha e a uma posição com lixo no fim.*
#[test]
fn uma_tabela_crua_nao_e_um_default() {
    let h = host();
    let lua = h.runtime().lua();
    for src in [
        r#"ph2d.property("t", {1, 0, 0})"#,
        r#"ph2d.property("t", { x = 1, y = 2 })"#,
        r#"ph2d.property("t", { __kind = "vec3", x = 1, y = 2, z = 3 })"#,
    ] {
        let e = load_module(lua, "mau.luau", src).err().expect(src);
        assert_eq!(
            e,
            ModuleError::Decl(DeclError::BadDefault("t".into())),
            "{src}"
        );
    }
}

//! ⭐⭐ **O formato REAL das mensagens do Luau, congelado** (CodeEditor Onda 0, T1).
//!
//! Medido em 2026-10-15 com **mlua 0.12.1** / **luau0-src 0.21.0+luau736** (`Cargo.lock`), sobre a
//! VM da casa (`ScriptHost`, sandbox ligada) e o nome de chunk que a cena usa (`set_name("=<nome>")`).
//! Uma subida do mlua/Luau que mude o formato fica VERMELHA aqui antes de o cartão perder a linha.

use ph2d_ecs::{SimWorld, StableId, Transform};
use ph2d_script::{
    DiagnosticKind, LuauScript, ScriptDiagnostic, ScriptHost, ScriptInfo, load_module,
};

fn host() -> ScriptHost {
    ScriptHost::new().expect("vm")
}

/// O erro CRU de `update(self, dt)` num módulo carregado com o nome `chunk`.
fn update_error(h: &ScriptHost, chunk: &str, src: &str) -> mlua::Error {
    let lua = h.runtime().lua();
    let m = load_module(lua, chunk, src).unwrap_or_else(|d| panic!("{d:?}"));
    m.hook("update")
        .expect("update")
        .call::<()>((lua.create_table().expect("self"), 0.1))
        .expect_err("falha")
}

const NESTED: &str = "local function inner(v)\n  return v.x + 1\nend\nlocal function middle(v)\n  return inner(v)\nend\nfunction update(self, dt)\n  middle(nil)\nend\n";

/// ⭐ As cadeias CRUAS, como o mlua as entrega.
#[test]
fn o_formato_cru_do_luau_esta_congelado() {
    let h = host();
    let lua = h.runtime().lua();
    let syntax = lua
        .load("local a = 1\nlocal b = 2\nlocal c = = 3\n")
        .set_name("=bob.luau")
        .into_function()
        .expect_err("sintaxe");
    assert!(
        matches!(&syntax, mlua::Error::SyntaxError { message, .. }
            if message == "bob.luau:3: Expected identifier when parsing expression, got '='"),
        "{syntax:?}"
    );
    let rt = update_error(
        &h,
        "bob.luau",
        "-- 1\n-- 2\n-- 3\n-- 4\nfunction update(self, dt)\n  local t = nil\n  local z = t.x\nend\n",
    );
    assert!(
        matches!(&rt, mlua::Error::RuntimeError(s) if s == "bob.luau:7: attempt to index nil with 'x'\nstack traceback:\n\t[C]: in ?\n\tbob.luau:7: in function 'update'"),
        "{rt:?}"
    );
    let nested = update_error(&h, "bob.luau", NESTED);
    assert!(
        matches!(&nested, mlua::Error::RuntimeError(s) if s == "bob.luau:2: attempt to index nil with 'x'\nstack traceback:\n\t[C]: in ?\n\tbob.luau:2: in function 'inner'\n\tbob.luau:5: in function 'middle'\n\tbob.luau:8: in function 'update'"),
        "{nested:?}"
    );
    let cb = update_error(
        &h,
        "bob.luau",
        "function update(self, dt)\n  local q = 1\n  ph2d.emit(\"\")\nend\n",
    );
    assert!(
        matches!(&cb, mlua::Error::CallbackError { traceback, cause }
            if traceback == "stack traceback:\n\t[C]: in ?\n\tbob.luau:3: in function 'update'"
            && matches!(&**cause, mlua::Error::RuntimeError(m) if m == "ph2d.emit: the signal needs a name")),
        "{cb:?}"
    );
}

/// ⭐⭐⭐ **A linha sai certa nos casos medidos** — sintaxe, `update`, função aninhada, chamada da
/// casa, topo, declaração.
#[test]
fn a_linha_e_a_deste_script() {
    let h = host();
    let lua = h.runtime().lua();
    let d = load_module(lua, "bob.luau", "local a = 1\nlocal b = 2\nlocal c = = 3\n")
        .err()
        .expect("sintaxe");
    assert_eq!(
        d,
        ScriptDiagnostic {
            kind: DiagnosticKind::Syntax,
            line: Some(3),
            message: "Expected identifier when parsing expression, got '='".into(),
            traceback: None,
        }
    );
    let e = update_error(
        &h,
        "bob.luau",
        "-- 1\n-- 2\n-- 3\n-- 4\nfunction update(self, dt)\n  local t = nil\n  local z = t.x\nend\n",
    );
    let d = ScriptDiagnostic::from_error(&e, "bob.luau");
    assert_eq!(d.kind, DiagnosticKind::Runtime);
    assert_eq!(d.line, Some(7));
    assert_eq!(d.message, "attempt to index nil with 'x'");
    assert_eq!(
        d.traceback.as_deref(),
        Some("stack traceback:\n\t[C]: in ?\n\tbob.luau:7: in function 'update'")
    );
    let d = ScriptDiagnostic::from_error(&update_error(&h, "bob.luau", NESTED), "bob.luau");
    assert_eq!(
        (d.line, d.message.as_str()),
        (Some(2), "attempt to index nil with 'x'")
    );
    // Uma chamada da casa não tem posição na mensagem: a linha vem da pilha.
    let e = update_error(
        &h,
        "bob.luau",
        "function update(self, dt)\n  local q = 1\n  ph2d.emit(\"\")\nend\n",
    );
    let d = ScriptDiagnostic::from_error(&e, "bob.luau");
    assert_eq!(
        (d.line, d.message.as_str()),
        (Some(3), "ph2d.emit: the signal needs a name")
    );
    assert!(d.traceback.is_some());
    // O topo do script.
    let d = load_module(lua, "bob.luau", "local a = 1\nlocal b = nil\nprint(b.c)\n")
        .err()
        .expect("topo");
    assert_eq!((d.kind.clone(), d.line), (DiagnosticKind::Runtime, Some(3)));
    assert_eq!(d.message, "attempt to index nil with 'c'");
    // Uma declaração recusada aponta a linha do `ph2d.property` que a recusou.
    let d = load_module(
        lua,
        "bob.luau",
        "ph2d.property(\"a\", 1)\n\nph2d.property(\"a\", 2)\n",
    )
    .err()
    .expect("decl");
    assert!(matches!(d.kind, DiagnosticKind::Decl(_)), "{d:?}");
    assert_eq!(d.line, Some(3));
}

/// ⛔ **Uma posição de OUTRO chunk nunca é deste script**, e um `:` dentro da mensagem não engana.
#[test]
fn uma_posicao_alheia_nao_e_atribuida_a_este_script() {
    let h = host();
    let lua = h.runtime().lua();
    // `helper` mora noutro chunk; a falha é lá, e a moldura DESTE script é a linha 2.
    let lib =
        load_module(lua, "lib.luau", "function helper(v)\n\n  return v.x\nend\n").expect("lib");
    lua.globals()
        .raw_set("zz_helper", lib.hook("helper").expect("helper"))
        .expect("o global de teste");
    let e = update_error(
        &h,
        "bob.luau",
        "function update(self, dt)\n  return zz_helper(nil)\nend\n",
    );
    assert!(
        matches!(&e, mlua::Error::RuntimeError(s) if s == "lib.luau:3: attempt to index nil with 'x'\nstack traceback:\n\t[C]: in ?\n\tlib.luau:3: in function 'helper'\n\tbob.luau:2: in function 'update'"),
        "{e:?}"
    );
    let d = ScriptDiagnostic::from_error(&e, "bob.luau");
    assert_eq!(d.line, Some(2), "a moldura deste script, nunca a 3 do lib");
    assert_eq!(
        d.message, "lib.luau:3: attempt to index nil with 'x'",
        "a posição alheia fica"
    );
    let d = ScriptDiagnostic::from_error(&e, "lib.luau");
    assert_eq!(
        (d.line, d.message.as_str()),
        (Some(3), "attempt to index nil with 'x'")
    );
    // Uma posição alheia DENTRO da mensagem não é a deste script.
    let e = mlua::Error::RuntimeError(
        "bob.luau:2: other.luau:99: fake\nstack traceback:\n\t[C]: in ?\n\t[C]: in function 'error'\n\tbob.luau:2: in function 'update'".into(),
    );
    let d = ScriptDiagnostic::from_error(&e, "bob.luau");
    assert_eq!(
        (d.line, d.message.as_str()),
        (Some(2), "other.luau:99: fake")
    );
    // Um nome com `:` lê-se inteiro; um prefixo do nome não é o nome.
    let e = mlua::Error::RuntimeError("a:b.luau:4: x: y".into());
    assert_eq!(ScriptDiagnostic::from_error(&e, "a:b.luau").line, Some(4));
    assert_eq!(ScriptDiagnostic::from_error(&e, "a").line, None);
    let e = mlua::Error::RuntimeError("bob.luau2:4: x".into());
    assert_eq!(ScriptDiagnostic::from_error(&e, "bob.luau").line, None);
    // Sem posição em lado nenhum: `None`, e a mensagem inteira fica.
    let e = mlua::Error::RuntimeError("nopos\nsegunda".into());
    let d = ScriptDiagnostic::from_error(&e, "bob.luau");
    assert_eq!((d.line, d.message.as_str()), (None, "nopos\nsegunda"));
}

/// Põe o script na biblioteca do projeto (o recurso do mundo) e devolve o componente que o aponta.
fn script(sim: &mut SimWorld, name: &str, source: &str) -> (LuauScript, ph2d_script::ScriptId) {
    let mut lib = sim
        .world_mut()
        .get_resource_or_init::<ph2d_script::ProjectScripts>();
    let id = lib.insert(name, source);
    (LuauScript::library(id), id)
}

/// ⭐⭐ **O prazo de um quadro diz ONDE o laço está** — a pilha é colhida dentro da interrupção.
#[test]
fn o_prazo_estourado_diz_a_linha_do_laco() {
    let mut sim = SimWorld::new();
    let (cfg, _) = script(
        &mut sim,
        "loop.luau",
        "local function spin()\n  local i = 0\n  while true do\n    i = i + 1\n  end\nend\nfunction update(self, dt)\n  spin()\nend\n",
    );
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, StableId(1), cfg))
        .id();
    let mut h = host();
    h.scene_sync(sim.world_mut());
    let r = h.scene_tick(sim.world_mut(), 0.1);
    assert_eq!(r.failed.len(), 1);
    let d = h.scene_failure(e).expect("parou").clone();
    assert_eq!(d.kind, DiagnosticKind::Budget);
    assert_eq!(d.line, Some(3), "{d:#}");
    assert_eq!(
        d.message,
        "the script ran for longer than a frame (16.667 ms) — is there a loop that never ends?"
    );
    assert_eq!(
        d.traceback.as_deref(),
        Some(
            "stack traceback:\n\tloop.luau:3: in function 'spin'\n\tloop.luau:8: in function 'update'"
        )
    );
    // No TOPO também.
    let (cfg, top) = script(
        &mut sim,
        "top.luau",
        "local i = 0\n\nwhile true do\n  i = i + 1\nend\n",
    );
    sim.world_mut()
        .spawn((Transform::IDENTITY, StableId(2), cfg));
    h.scene_sync(sim.world_mut());
    let Some(ScriptInfo::Broken(d)) = h.scene_info(top) else {
        panic!("{:?}", h.scene_info(top));
    };
    assert_eq!(
        (d.kind.clone(), d.line),
        (DiagnosticKind::Budget, Some(3)),
        "{d:#}"
    );
}

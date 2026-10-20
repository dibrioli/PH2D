//! As curas da biblioteca padrão (CodeEditor item 6 (g), `docs/CodeEditor/05_api_que_funciona.md`
//! §3.7 e §6): o `loadstring` de cada script e a família do inteiro, medidos como SCRIPTS.

use ph2d_ecs::{Entity, SimWorld, StableId, Transform, World};

use crate::ScriptHost;
use crate::component::{LuauScript, ProjectScripts};

fn actor(world: &mut World, sid: u64, name: &str, text: &str) -> Entity {
    let id = {
        let mut lib = world.get_resource_or_init::<ProjectScripts>();
        lib.by_name(name)
            .unwrap_or_else(|| lib.insert(name, text.to_owned()))
    };
    world
        .spawn((Transform::IDENTITY, StableId(sid), LuauScript::library(id)))
        .id()
}

fn pose(world: &World, e: Entity) -> Transform {
    *world.get::<Transform>(e).expect("pose")
}

fn tick(host: &mut ScriptHost, sim: &mut SimWorld) -> crate::scene::SceneReport {
    host.scene_sync(sim.world_mut());
    host.scene_tick(sim, 1.0 / 60.0)
}

// ── §3.7 (g) loadstring, _G e a família do inteiro ───────────────────────────────────────────

#[test]
fn loadstring_nao_passa_globais_entre_scripts() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let a = actor(
        w,
        1,
        "a.luau",
        r#"
function update(self)
  meu = 5
  loadstring("vazou = 42")()
  local trancado = not pcall(function() _G.porG = 7 end)
  ph2d.set(self.id, "x", vazou or -1)
  ph2d.set(self.id, "y", loadstring("return meu")() or -1)
  ph2d.set(self.id, "rotation", trancado and 1 or 0)
  local f, err = loadstring("return +")
  ph2d.set(self.id, "scale_x", (f == nil and err ~= nil) and 1 or 0)
end
"#,
    );
    let b = actor(
        w,
        2,
        "b.luau",
        r#"
function update(self)
  ph2d.set(self.id, "x", vazou or -1)
  ph2d.set(self.id, "y", meu or -1)
  ph2d.set(self.id, "rotation", porG or -1)
end
"#,
    );
    let mut host = ScriptHost::new().expect("vm");
    for _ in 0..2 {
        assert!(tick(&mut host, &mut sim).failed.is_empty());
    }
    let (pa, pb) = (pose(sim.world(), a), pose(sim.world(), b));
    assert_eq!(
        (pa.translation.x, pa.translation.y, pa.rotation),
        (42.0, 5.0, 1.0),
        "o script vê o seu; o _G partilhado continua trancado"
    );
    assert_eq!(
        pa.scale.x, 1.0,
        "um erro de sintaxe devolve nil e a mensagem"
    );
    assert_eq!(
        (pb.translation.x, pb.translation.y, pb.rotation),
        (-1.0, -1.0, -1.0),
        "e nunca o do outro"
    );
}

/// O `(int)` do Luau mente fora de ±2³¹ no x86: cada função da lista satura (a resposta do ARM).
#[test]
fn a_familia_do_inteiro_satura() {
    let host = ScriptHost::new().expect("vm");
    let casos: &[(&str, &str)] = &[
        ("string.sub('abcdef', 2, 1e10)", "bcdef"),
        ("string.sub('abcdef', 1e10)", ""),
        ("string.sub('abcdef', -1e10, 2)", "ab"),
        ("select('#', string.byte('abc', 1, 1e10))", "3"),
        ("pcall(string.rep, 'x', 1e10)", "false"),
        ("#string.rep('x', -1e10)", "0"),
        ("string.find('abc', 'a', 1e10)", "nil"),
        ("string.match('abc', 'a', 1e10)", "nil"),
        ("select(2, string.gsub('aaa', 'a', 'b', 1e10))", "3"),
        ("pcall(table.unpack, {1, 2, 3}, 1, 2^31)", "false"),
        ("pcall(table.concat, {1, 2}, '', 1, 1e10)", "false"),
        ("pcall(table.move, {1, 2}, 1, 1e10, 1)", "false"),
        ("pcall(utf8.len, 'abc', 1, 1e10)", "false"),
        ("pcall(utf8.codepoint, 'abc', 1, 1e10)", "false"),
        ("utf8.offset('abc', 1e10)", "nil"),
        ("math.ldexp(1, 1e10)", "inf"),
        ("bit32.lrotate(1, 32 + 1)", "2"),
        ("bit32.lrotate(1, 2^32 + 1)", "2"),
        ("bit32.rrotate(2, 2^32 + 1)", "1"),
        ("pcall(string.sub, 'abc', 0/0)", "false"),
        ("pcall(bit32.lrotate, 1, 0/0)", "false"),
        // ⛔ Só texto: um texto que começa por um byte de controlo nunca é tomado por bytecode.
        ("loadstring('\\treturn 1')()", "1"),
        ("#string.rep('ab', 3)", "6"),
        ("table.remove({1, 2, 3})", "3"),
        // Os que já respondem com erro honesto (a classificação do 05 §6, medida e não lida).
        ("pcall(table.insert, {}, 2^32 + 1, 'x')", "false"),
        ("pcall(table.insert, {}, 1e10, 'x')", "false"),
        (
            "#(function() local t = {} table.insert(t, 2^40) return t end)()",
            "1",
        ),
        // Além da lista o Luau devolve nil e não tira nada — dentro do alcance (10) e fora dele igual.
        (
            "(function() local t = {1, 2, 3} table.remove(t, 1e10) return #t end)()",
            "3",
        ),
        ("pcall(table.find, {1}, 1, 1e10)", "false"),
        ("pcall(bit32.extract, 1, 2^32 + 1)", "false"),
        ("pcall(bit32.replace, 1, 1, 2^32 + 1)", "false"),
        ("pcall(table.create, 2^32 + 3)", "false"),
        ("pcall(buffer.create, 2^32 + 3)", "false"),
        ("pcall(select, 2^32 + 1, 'a')", "false"),
        ("pcall(tonumber, '10', 2^32 + 10)", "false"),
        ("pcall(utf8.char, 2^32 + 65)", "false"),
        ("pcall(string.char, 2^32 + 65)", "false"),
    ];
    let lua = host.runtime().lua();
    let mut erros = Vec::new();
    for (expr, want) in casos {
        // Como um SCRIPT (o ambiente dele): no global o `FASTCALL` salta a cura.
        let got = match crate::load_module(lua, "familia", &format!("print(tostring(({expr})))")) {
            Ok(_) => host
                .drain_console()
                .into_iter()
                .map(|l| l.text)
                .collect::<String>(),
            Err(d) => format!("ERRO {}", d.message),
        };
        if got != *want {
            erros.push(format!("{expr} → {got} (devia {want})"));
        }
    }
    assert!(erros.is_empty(), "{}", erros.join("\n"));
}

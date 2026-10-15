//! ADR-0188: o texto vem da BIBLIOTECA do projeto — um script partilhado recarrega em todos os
//! objectos, e o chunk chama-se pelo nome único da biblioteca. Filho do `scene_tests` (os arnêses
//! são dele).

use super::*;

/// ⭐⭐ **Dois objectos no MESMO script, o texto editado na biblioteca: os DOIS recarregam** — e
/// cada um guarda o seu `self` (recarregar não renasce).
///
/// Mutação que deve sangrar: o `refresh` não comparar o hash (nunca recarrega) · o `sync` não reler
/// a biblioteca a cada quadro.
#[test]
fn editing_a_shared_library_script_reloads_every_object_keeping_self() {
    let path = script_file(
        "partilhado",
        r#"
ph2d.property("passo", 1)
function init(self) self.n = 0 end
function update(self, dt) self.n = self.n + self.passo ; ph2d.set(self.id, "x", self.n) end
"#,
    );
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let a = actor(w, 1, &path, &[]);
    let b = actor(w, 2, &path, &[("passo", ScriptValue::Number(2.0))]);
    let mut host = ScriptHost::new().expect("vm");
    for _ in 0..2 {
        frame(&mut host, &mut sim, 0.1);
    }
    assert_eq!((x(sim.world(), a), x(sim.world(), b)), (2.0, 4.0));
    rewrite(
        sim.world_mut(),
        &path,
        r#"
ph2d.property("passo", 1)
function init(self) self.n = 1000 end
function update(self, dt) self.n = self.n + 10 * self.passo ; ph2d.set(self.id, "x", self.n) end
"#,
    );
    frame(&mut host, &mut sim, 0.1);
    assert_eq!(
        (x(sim.world(), a), x(sim.world(), b)),
        (12.0, 24.0),
        "os dois correm a lei nova sobre o self velho, e o init NÃO correu outra vez"
    );
    assert_eq!(host.scene().live_count(), 2);
}

/// O nome do chunk é o nome ÚNICO na biblioteca — é ele que a mensagem do Luau cita.
#[test]
fn the_chunk_name_is_the_unique_library_name() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let primeiro = script_file("bob", "-- outro bob");
    id_of(w, &primeiro);
    let id = w.resource_mut::<ProjectScripts>().insert(
        "bob.luau",
        "function update(self, dt)\n  error('ai')\nend\n",
    );
    let e = w
        .spawn((Transform::IDENTITY, StableId(1), LuauScript::library(id)))
        .id();
    let mut host = ScriptHost::new().expect("vm");
    frame(&mut host, &mut sim, 0.1);
    let d = host.scene_failure(e).expect("parou");
    assert_eq!(d.line, Some(2), "{d:?}");
    assert!(d.message.contains("ai"), "{d:?}");
    assert_eq!(
        sim.world()
            .resource::<ProjectScripts>()
            .get(id)
            .map(|s| s.name.as_str()),
        Some("bob 2.luau")
    );
}

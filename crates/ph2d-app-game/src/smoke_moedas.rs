//! **J5 — as moedas montadas à mão com as peças** (GEF 21, [plano](../../../docs/GameEngineFuncional/21_moedas_plano.md)):
//! nada aqui é da moeda — cada peça é o `+` do Inspector com os valores que o artista escreveria (dono,
//! 02/11: *«revisar e melhorar a engine testando em um game real»*). O texto do placar é o dono que o
//! escreve no smoke (Text dentro do HUD + *Label*): é o caminho do artista, e a cena não o finge.

use ph2d_ecs::{
    AudioSource2D, Counter, Entity, SignalAction, SignalActions, SignalFrom, SignalTarget,
    SignalVerb, SimWorld,
};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, RigidBody, SignalOnHit};

use crate::smoke::Portas;

/// Os nomes que ligam as peças: o sinal que a moeda grita, o contador que ela soma e o HUD que o leva.
pub const SINAL: &str = "coin";
pub const CONTADOR: &str = "coins";
pub const PLACAR: &str = "HUD";
/// O som da moeda (Kenney *Digital Audio* `powerUp9`, CC0): a cena IMPORTA-O e a moeda nomeia-o.
pub const SOM: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/GameEngineFuncional/arte/sons/moeda.ogg"
);
/// O nome com que o som fica no projeto (o do ficheiro).
pub const NOME_DO_SOM: &str = "moeda.ogg";
/// O raio do círculo, no espaço da moeda (escala `0,5` ⇒ `0,25 m` no mundo, o desenho dela).
pub const RAIO: f32 = 0.5;

/// O `+` do Inspector, que falha alto.
fn mais(sim: &mut SimWorld, portas: Portas<'_>, e: Entity, componente: &str) {
    (portas.menu.anexa)(sim, e.to_bits(), componente)
        .unwrap_or_else(|m| panic!("{componente}: {m}"));
}

/// Uma linha da tabela: em [`SINAL`], só se fui eu que gritei.
fn linha(verb: SignalVerb, target: &str, arg: &str) -> SignalAction {
    SignalAction {
        on: SINAL.into(),
        target: target.into(),
        verb,
        arg: arg.into(),
        target_by: SignalTarget::Named,
        from: SignalFrom::Myself,
        item: Default::default(),
    }
}

/// Veste a moeda `e`: Physics Body *Static* de *Trigger* · Signal On Hit · Audio Source 2D · Signal Actions.
pub fn veste_moeda(sim: &mut SimWorld, portas: Portas<'_>, e: Entity) {
    for c in [
        "ph2d::physics::RigidBody",
        "ph2d::physics::SignalOnHit",
        "ph2d::ecs::AudioSource2D",
        "ph2d::ecs::SignalActions",
    ] {
        mais(sim, portas, e, c);
    }
    let mut m = sim.world_mut().entity_mut(e);
    m.get_mut::<RigidBody>().expect("Physics Body").kind = BodyKind::Static;
    let mut col = m.get_mut::<Collider>().expect("Collision Shape");
    col.shape = ColliderShape::Ball { radius: RAIO };
    col.is_sensor = true;
    m.get_mut::<SignalOnHit>().expect("Signal On Hit").0 = SINAL.into();
    m.get_mut::<AudioSource2D>().expect("Audio Source 2D").sound = NOME_DO_SOM.into();
    m.get_mut::<SignalActions>().expect("Signal Actions").0 = vec![
        linha(SignalVerb::PlaySound, "", ""),
        linha(SignalVerb::AddToCounter, PLACAR, "1"),
        linha(SignalVerb::Destroy, "", ""),
    ];
}

/// O HUD (Add ▸ HUD) com o **Counter** das moedas — e o Import… do som da moeda.
pub fn placar(sim: &mut SimWorld, portas: Portas<'_>) -> Entity {
    let nome = (portas.importa_som)(sim, std::path::Path::new(SOM))
        .unwrap_or_else(|m| panic!("o som da moeda: {m}"));
    assert_eq!(nome, NOME_DO_SOM, "o som entra com o nome do ficheiro");
    let hud = Entity::from_bits(
        (portas.componentes)(sim, "ph2d::ecs::UiCanvas").unwrap_or_else(|m| panic!("HUD: {m}")),
    );
    let nome = ph2d_ecs::Name::new(PLACAR);
    sim.world_mut().entity_mut(hud).insert(nome);
    mais(sim, portas, hud, "ph2d::ecs::Counter");
    *sim.world_mut().get_mut::<Counter>(hud).expect("Counter") = Counter {
        name: CONTADOR.into(),
        start: 0,
        keep_on_restart: false,
    };
    hud
}

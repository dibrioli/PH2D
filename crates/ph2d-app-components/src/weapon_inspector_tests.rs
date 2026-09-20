//! Os gates do instantâneo e do dreno da secção WEAPON.

use super::{apply_all, build_info};
use ph2d_ecs::{Counter, CounterRuntime, Entity, Name, SimWorld, StableId, WeaponFire};
use ph2d_editor_core::weapon_edits::{WeaponFieldEdit as E, WeaponQueixa};

fn arma(sim: &mut SimWorld, pente: Option<(&str, i64)>) -> Entity {
    let mut ent = sim.world_mut().spawn((
        Name::new("Arma"),
        StableId(1),
        WeaponFire {
            on_signal: "fire".to_owned(),
            cooldown_ms: 250,
            ammo_counter: pente.map(|p| p.0.to_owned()).unwrap_or_default(),
            reload_ms: 800,
            on_fire: "shot".to_owned(),
            ..WeaponFire::default()
        },
    ));
    if let Some((nome, n)) = pente {
        ent.insert((
            Counter {
                name: nome.to_owned(),
                start: n,
                keep_on_restart: false,
            },
            CounterRuntime { value: n },
        ));
    }
    ent.id()
}

/// **O instantâneo traz a munição VIVA**, e não o `start` do contador.
#[test]
fn o_instantaneo_traz_a_municao_viva() {
    let mut sim = SimWorld::new();
    let e = arma(&mut sim, Some(("ammo", 6)));
    if let Some(mut rt) = sim.world_mut().get_mut::<CounterRuntime>(e) {
        rt.value = 2;
    }
    let info = build_info(&sim, e.to_bits(), true, 1).expect("a arma tem o componente");
    assert_eq!(info.municao, Some(2), "o que ela tem AGORA");
    assert_eq!(info.pente, 6, "e o cheio vem do `start`");
    assert_eq!(info.queixa(), None);
}

/// ⭐⭐ **O pente NOUTRA entidade lê-se como ausente, e a queixa di-lo.**
///
/// É a lente da ponte: ela também só lê o contador desta entidade. *Se o painel somasse os do nome,
/// mostraria um número e a arma gastaria outro.*
#[test]
fn um_pente_noutra_entidade_e_acusado() {
    let mut sim = SimWorld::new();
    let e = arma(&mut sim, None);
    // a arma NOMEIA um pente…
    if let Some(mut w) = sim.world_mut().get_mut::<WeaponFire>(e) {
        w.ammo_counter = "ammo".to_owned();
    }
    // …que existe noutro objecto
    sim.world_mut().spawn((
        Name::new("Placar"),
        StableId(2),
        Counter {
            name: "ammo".to_owned(),
            start: 9,
            keep_on_restart: false,
        },
        CounterRuntime { value: 9 },
    ));
    let info = build_info(&sim, e.to_bits(), true, 1).expect("componente");
    assert_eq!(info.municao, None);
    assert_eq!(info.queixa(), Some(WeaponQueixa::PenteAusente));
}

/// **Um objecto sem o componente não tem secção.**
#[test]
fn sem_o_componente_nao_ha_seccao() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((Name::new("Caixa"), StableId(1)))
        .id();
    assert!(build_info(&sim, e.to_bits(), true, 1).is_none());
}

/// **Cada edição escreve o SEU campo, e escrever o mesmo valor não é uma mudança.**
///
/// ⚠️ A segunda metade é o que impede um clique numa caixa já certa de entrar no `Ctrl+Z`.
#[test]
fn cada_edicao_escreve_o_seu_campo_e_o_igual_nao_conta() {
    let mut sim = SimWorld::new();
    let e = arma(&mut sim, Some(("ammo", 6)));
    let b = e.to_bits();

    assert!(apply_all(&mut sim, &[(b, E::CooldownMs(100))]));
    assert!(
        !apply_all(&mut sim, &[(b, E::CooldownMs(100))]),
        "o mesmo valor outra vez nao e' uma mudanca"
    );
    assert!(apply_all(&mut sim, &[(b, E::OnEmpty("click".to_owned()))]));

    let w = sim.world().get::<WeaponFire>(e).expect("componente");
    assert_eq!(w.cooldown_ms, 100);
    assert_eq!(w.on_empty, "click");
    assert_eq!(w.on_signal, "fire", "e nenhuma delas tocou nas vizinhas");
    assert_eq!(w.on_fire, "shot");
}

/// **As oito edições chegam ao componente** — a metade que prova que nenhuma variante é decorativa.
#[test]
fn as_oito_edicoes_chegam() {
    let mut sim = SimWorld::new();
    let e = arma(&mut sim, Some(("ammo", 6)));
    let b = e.to_bits();
    let todas = [
        E::OnSignal("a".to_owned()),
        E::CooldownMs(1),
        E::AmmoCounter("b".to_owned()),
        E::ReloadMs(2),
        E::ReloadOn("c".to_owned()),
        E::OnFire("d".to_owned()),
        E::OnEmpty("e".to_owned()),
        E::OnReloaded("f".to_owned()),
    ];
    for edit in &todas {
        assert!(
            apply_all(&mut sim, &[(b, edit.clone())]),
            "a edicao {edit:?} tem de chegar ao componente"
        );
    }
    let w = sim
        .world()
        .get::<WeaponFire>(e)
        .expect("componente")
        .clone();
    assert_eq!(
        w,
        WeaponFire {
            on_signal: "a".to_owned(),
            cooldown_ms: 1,
            ammo_counter: "b".to_owned(),
            reload_ms: 2,
            reload_on: "c".to_owned(),
            on_fire: "d".to_owned(),
            on_empty: "e".to_owned(),
            on_reloaded: "f".to_owned(),
        },
        "os OITO campos, e nenhum ficou por escrever"
    );
}

/// ⭐⭐ **O ESPELHO do tecto é IGUAL ao original.**
///
/// ⛔ A `ph2d-editor-core` **não vê o `ph2d-ecs`** (ela é o vocabulário do editor, não do mundo),
/// logo o tecto das duas caixas de tempo tem de ser re-declarado lá. *Uma cópia sem quem a compare é
/// a segunda resposta à mesma pergunta*, e esta crate é a única que vê os dois lados.
///
/// ⚠️ **O gate mora AQUI por isso**, e não junto de nenhum dos dois: quem declara um espelho não
/// pode ser quem o verifica.
#[test]
fn o_tecto_do_painel_e_o_tecto_da_lei() {
    #[allow(clippy::cast_precision_loss)]
    let lei = ph2d_ecs::WEAPON_MAX_MS as f64;
    assert!(
        (ph2d_editor_core::weapon_edits::WEAPON_MAX_MS_UI - lei).abs() < f64::EPSILON,
        "o tecto do painel ({}) tem de ser o da lei ({lei})",
        ph2d_editor_core::weapon_edits::WEAPON_MAX_MS_UI
    );
}

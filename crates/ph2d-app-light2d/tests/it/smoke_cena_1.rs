//! A cena de smoke 1 (`PH2D_LIGHT2D_SMOKE=1`) ensina o que a tabela dela diz — medido pelo
//! gabarito, não pelo olho: o Verde (atrás da parede) na sombra, o Vermelho e o Azul acesos; e a
//! família acende a cena (a guarda vê as duas `Luz`).

use bevy_ecs::world::World;
use ph2d_app_light2d::{LuzDoQuadro, smoke};
use ph2d_ecs::{GlobalTransform, Name};

fn centro(w: &mut World, nome: &str) -> [f64; 2] {
    let mut q = w.query::<(&Name, &GlobalTransform)>();
    let t = q
        .iter(w)
        .find(|(n, _)| n.0 == nome)
        .map(|(_, g)| g.translation())
        .expect("o quadrado existe");
    [f64::from(t.x), f64::from(t.y)]
}

#[test]
fn a_cena_1_ensina_a_sombra_atras_da_parede() {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 1));
    assert!(!smoke::encenar(&mut World::new(), 2), "só há a cena 1");
    assert_eq!(smoke::ROUTERS[0].max_level, 1);

    let mut luz = LuzDoQuadro::new();
    assert!(luz.preparar(&w, crate::comum::afim(), (crate::comum::W, crate::comum::H)));
    let cena = luz.cena().clone();
    assert_eq!(cena.luzes, 2, "a lâmpada e a ambiente");
    assert_eq!(cena.oclusores.len(), 1, "a parede");

    let em = |p: [f64; 2]| ph2d_light2d_ref::luz_em(&cena, p)[1];
    let (verde, vermelho, azul) = (
        em(centro(&mut w, "Verde")),
        em(centro(&mut w, "Vermelho")),
        em(centro(&mut w, "Azul")),
    );
    let ambiente = cena.ceu[1] as f64;
    eprintln!(
        "cena 1: luz no Verde {verde:.3} · Vermelho {vermelho:.3} · Azul {azul:.3} (ambiente {ambiente:.3})"
    );
    assert!(
        (verde - ambiente).abs() < 0.02,
        "o Verde está atrás da parede: só a ambiente o alcança ({verde:.3})"
    );
    assert!(
        vermelho > 5.0 * verde && azul > 3.0 * verde,
        "o Vermelho e o Azul acendem"
    );
}

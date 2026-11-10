//! ⭐ **A caixa de cada habilidade** (ficha `docs/GameEngineFuncional/componentes/03_character.md`): desmarcada,
//! a lei recebe a habilidade inerte; marcada, os números com que ela nasce já a ligam.

use ph2d_physics_ecs::PlatformPlayer;

/// Quem liga cada caixa e se a lei a vê armada.
fn armadas(p: &PlatformPlayer) -> [bool; 7] {
    let c = p.config();
    [
        c.wall.armed(),
        c.dash.armed(),
        c.crouch.armed(&c.ride),
        c.swim.armed(),
        c.ledge.armed(),
        c.glide.armed(),
        c.fall.armed(),
    ]
}

fn liga(p: &mut PlatformPlayer, i: usize, on: bool) {
    *[
        &mut p.walls_on,
        &mut p.dash_on,
        &mut p.crouch_on,
        &mut p.swim_on,
        &mut p.ledge_on,
        &mut p.glide_on,
        &mut p.fall_limit_on,
    ][i] = on;
}

#[test]
fn a_caixa_desmarcada_desliga_e_a_marcada_liga_com_os_numeros_de_nascenca() {
    let base = PlatformPlayer::default();
    assert_eq!(
        armadas(&base),
        [false; 7],
        "o Platform Player nasce sem habilidades"
    );
    for i in 0..7 {
        let mut p = base;
        liga(&mut p, i, true);
        let mut esperado = [false; 7];
        esperado[i] = true;
        assert_eq!(
            armadas(&p),
            esperado,
            "marcar a caixa {i} liga só ela, já no primeiro Play"
        );
        liga(&mut p, i, false);
        assert_eq!(armadas(&p), [false; 7], "desmarcar a caixa {i} desliga");
        assert_eq!(p, base, "desmarcar devolve os números como estavam");
    }
}

/// As três partes da parede (deslizar, pular, agarrar) caem juntas com a caixa.
#[test]
fn a_parede_desmarcada_zera_as_tres_partes() {
    let mut p = PlatformPlayer {
        wall_grab_stamina: 1.0,
        ..PlatformPlayer::default()
    };
    let w = p.config().wall;
    assert_eq!(
        (w.slide_speed, w.jump_height, w.grab_stamina),
        (0.0, 0.0, 0.0)
    );
    p.walls_on = true;
    assert!(p.config().wall.grab_stamina > 0.0);
}

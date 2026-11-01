//! Os gates da LEI da moldura da câmera (plano GEF 17 A3–A4, 17b, onda 18) — sem mundo: a moldura escrita à mão.
//! Vieram de `ph2d-app-game/src/moldura_tests.rs` (onda 18, pela catraca das crates `ph2d-app-*`). Os da cena 4
//! montada moram nos irmãos `jogo_moldura.rs` e `jogo_moldura_ponteiro.rs`.

use ph2d_camera_frame::*;
use ph2d_editor_core::CameraFieldEdit;

fn selecionada() -> Moldura {
    Moldura {
        camera: 7,
        nome: "Camera".into(),
        pose: [2.0, 1.0],
        centro: [2.0, 1.0],
        meia: [8.0, 4.5],
        ativa: true,
        selecionada: true,
        limites: Some(([-20.0, -5.0], [30.0, 15.0])),
        mira: [2.0, 1.0],
        offset: [0.0, 0.0],
        segue: None,
    }
}

fn altura(e: Escrita) -> f32 {
    let Escrita::Altura(h) = e else {
        panic!("o canto escreve a altura");
    };
    h
}

/// ⭐ **Sem cerca, o canto só muda a altura e o centro fica** — o canto vai para onde o ponteiro o leva no eixo
/// que manda.
#[test]
fn o_canto_livre_so_muda_a_altura_e_mantem_o_centro() {
    let m = Moldura {
        limites: None,
        ..selecionada()
    };
    let aspecto = 16.0 / 9.0;
    let (lo, hi) = caixa(&m);
    let alca = alca_em(&m, hi, 0.1).expect("o canto de cima-direita");
    assert_eq!(alca, Alca::Canto(2));
    let a = Arrasto::novo(&m, alca, hi, aspecto, m.pose, (0.0, 0.0));
    // Para fora em y: a altura dobra a distância ao centro.
    assert!((altura(arrastar(&a, [hi[0], 9.0])) - 16.0).abs() < 1e-3);
    // Para dentro em x (y igual): a largura manda, a proporção fica.
    assert!((altura(arrastar(&a, [6.0, 1.0])) - 8.0 / aspecto).abs() < 1e-3);
    // O outro canto pelo outro lado dá o mesmo: a conta é a partir do centro.
    let b = Arrasto::novo(&m, Alca::Canto(0), lo, aspecto, m.pose, (0.0, 0.0));
    assert!((altura(arrastar(&b, [lo[0], -7.0])) - 16.0).abs() < 1e-3);
}

/// ⭐ **Com cerca, o canto cai no ponteiro com o centro que a cerca dá** (17b A7): a câmera em (2; 1) com 9 m de
/// altura sobe a borda de baixo ao crescer (a cerca de baixo está em −5), e o canto de cima-direita fica sob o
/// ponteiro no eixo que manda.
#[test]
fn o_canto_com_cerca_cai_no_ponteiro() {
    let m = selecionada();
    let aspecto = 16.0 / 9.0;
    let (_, hi) = caixa(&m);
    let a = Arrasto::novo(&m, Alca::Canto(2), hi, aspecto, m.pose, (0.0, 0.0));
    let h = altura(arrastar(&a, [hi[0], 13.0]));
    let meia = ph2d_ecs::half_extent(h, aspecto);
    let y = ph2d_ecs::clamp_axis_to_limits(1.0, meia[1], -5.0, 15.0) + meia[1];
    assert!((y - 13.0).abs() < 1e-3, "o canto ficou em y {y}");
}

/// ⭐ **O miolo é um clique PENDENTE** (17b A1): agarra mesmo com outro objecto por baixo, mas só escreve depois
/// de o ponteiro andar o limiar — soltar antes é um clique, e o clique é da seleção. Uma câmera não selecionada
/// não agarra nada.
#[test]
fn o_miolo_so_move_depois_do_limiar() {
    let m = selecionada();
    assert_eq!(alca_em(&m, [3.0, 2.0], 0.1), Some(Alca::Dentro));
    let livre = Moldura {
        selecionada: false,
        ..selecionada()
    };
    assert_eq!(alca_em(&livre, [3.0, 2.0], 0.1), None);
    assert_eq!(alca_em(&livre, caixa(&livre).1, 0.1), None);
    let mut a = Arrasto::novo(
        &m,
        Alca::Dentro,
        [3.0, 2.0],
        1.0,
        [0.5, 0.5],
        (100.0, 100.0),
    );
    assert!(
        !a.arma((102.0, 101.0)),
        "dentro do limiar ainda é um clique"
    );
    assert!(a.era_um_clique());
    assert!(a.arma((110.0, 100.0)));
    assert!(a.arma((100.0, 100.0)), "armado fica armado");
    assert!(!a.era_um_clique());
    // O miolo move a câmera pelo que o ponteiro andou.
    assert_eq!(arrastar(&a, [4.0, 0.0]), Escrita::Pose([1.5, -1.5]));
    // As outras alças nascem armadas.
    let c = Arrasto::novo(&m, Alca::Canto(0), [0.0, 0.0], 1.0, m.pose, (0.0, 0.0));
    assert!(!c.era_um_clique());
}

/// ⭐ **As alças dos limites escrevem `CameraLimits` e nunca os invertem** — cada canto e cada borda arrastados
/// para lá do lado oposto param a um centímetro dele; o centro move a cerca inteira.
#[test]
fn as_alcas_dos_limites_nunca_invertem() {
    let m = selecionada();
    let (min, max) = m.limites.expect("limites");
    let longe = 1000.0;
    for (i, c) in cantos(min, max).into_iter().enumerate() {
        assert_eq!(alca_em(&m, c, 0.1), Some(Alca::LimiteCanto(i as u8)));
        let a = Arrasto::novo(&m, Alca::LimiteCanto(i as u8), c, 1.0, m.pose, (0.0, 0.0));
        let alvo = [
            if c[0] == min[0] { longe } else { -longe },
            if c[1] == min[1] { longe } else { -longe },
        ];
        let Escrita::Limites(lo, hi) = arrastar(&a, alvo) else {
            panic!("o canto dos limites escreve os limites");
        };
        for e in 0..2 {
            assert!(
                hi[e] - lo[e] >= LIMITE_VAO_MIN - 1e-4,
                "canto {i}: {lo:?} {hi:?}"
            );
        }
    }
    let bordas = [
        ([min[0], 5.0], [longe, 0.0]),
        ([max[0], 5.0], [-longe, 0.0]),
        ([0.0, min[1]], [0.0, longe]),
        ([0.0, max[1]], [0.0, -longe]),
    ];
    for (i, (p, d)) in bordas.into_iter().enumerate() {
        assert_eq!(alca_em(&m, p, 0.1), Some(Alca::LimiteBorda(i as u8)));
        let a = Arrasto::novo(&m, Alca::LimiteBorda(i as u8), p, 1.0, m.pose, (0.0, 0.0));
        let Escrita::Limites(lo, hi) = arrastar(&a, [p[0] + d[0], p[1] + d[1]]) else {
            panic!("a borda escreve os limites");
        };
        for e in 0..2 {
            assert!(
                hi[e] - lo[e] >= LIMITE_VAO_MIN - 1e-4,
                "borda {i}: {lo:?} {hi:?}"
            );
        }
    }
    let meio = [5.0, 5.0];
    assert_eq!(alca_em(&m, meio, 0.1), Some(Alca::LimiteCentro));
    let a = Arrasto::novo(&m, Alca::LimiteCentro, meio, 1.0, m.pose, (0.0, 0.0));
    assert_eq!(
        arrastar(&a, [6.0, 3.0]),
        Escrita::Limites([-19.0, -7.0], [31.0, 13.0])
    );
}

/// ⭐ **Com um pai rodado e escalado, o miolo leva a câmera com o ponteiro no MUNDO** (onda 18): a translação local
/// sai da inversa do pai, e o pai a compor devolve a pose de mundo do `Down` mais o delta. Somar o delta à local
/// (o que era) poria a câmera a 90° do ponteiro e a meia distância.
#[test]
fn com_pai_rodado_e_escalado_o_miolo_segue_o_ponteiro_no_mundo() {
    use ph2d_core::Vec2;
    use ph2d_ecs::Transform;
    let pai = Transform {
        translation: Vec2::new(3.0, -1.0),
        rotation: std::f32::consts::FRAC_PI_2,
        scale: Vec2::new(2.0, 2.0),
        ..Transform::default()
    };
    let local = Transform {
        translation: Vec2::new(1.5, 0.5),
        ..Transform::default()
    };
    let mundo = Transform::compose(pai, local);
    let m = Moldura {
        pose: [mundo.translation.x, mundo.translation.y],
        ..selecionada()
    };
    let p0 = m.centro;
    let mut a =
        Arrasto::novo(&m, Alca::Dentro, p0, 16.0 / 9.0, [1.5, 0.5], (0.0, 0.0)).com_pai(pai);
    assert!(a.arma((40.0, 0.0)));
    let Escrita::Pose(l) = arrastar(&a, [p0[0] + 4.0, p0[1] + 2.0]) else {
        panic!("sem follow o miolo escreve a pose");
    };
    let volta = Transform::compose(
        pai,
        Transform {
            translation: Vec2::new(l[0], l[1]),
            ..Transform::default()
        },
    );
    let quer = [mundo.translation.x + 4.0, mundo.translation.y + 2.0];
    assert!(
        (volta.translation.x - quer[0]).abs() < 1e-4
            && (volta.translation.y - quer[1]).abs() < 1e-4,
        "a câmera foi para ({}, {}) com o ponteiro a pedir {quer:?}",
        volta.translation.x,
        volta.translation.y
    );
}

fn que_segue(zona: [f32; 2]) -> Moldura {
    Moldura {
        limites: None,
        offset: [1.0, 0.5],
        centro: [5.0, 4.0],
        segue: Some(ph2d_camera_frame::Segue {
            nome: "Player".into(),
            alvo: Some([2.0, 1.0]),
            offset: [0.0, 0.0],
            zona,
        }),
        ..selecionada()
    }
}

/// ⭐ **A zona morta à vista** (onda 18, A5): a caixa é `±dz · meia` à volta da MIRA (a âncora da lei
/// `dead_zone_goal` com o relógio parado), nunca do centro da moldura; a borda arrastada escreve a fracção do seu
/// eixo, simétrica, cravada a `0..=1`.
///
/// **Mutações que devem sangrar:** centrar a zona na moldura · escrever o delta em vez da distância ao centro · não
/// cravar a fracção.
#[test]
fn a_zona_morta_pinta_se_a_volta_da_ancora_e_a_borda_escreve_a_fraccao() {
    let m = que_segue([0.25, 0.5]);
    let (lo, hi) = ph2d_camera_frame::caixa_da_zona(&m).expect("quem segue tem zona");
    assert_eq!((lo, hi), ([0.0, -1.25], [4.0, 3.25]));
    let pegas = ph2d_camera_frame::pegas_da_zona(&m, 0.1);
    let direita = pegas[1].expect("a pega da direita");
    assert_eq!(direita, [4.0, 1.0]);
    assert_eq!(alca_em(&m, direita, 0.1), Some(Alca::ZonaMorta(1)));
    let mut a = Arrasto::novo(
        &m,
        Alca::ZonaMorta(1),
        direita,
        16.0 / 9.0,
        m.pose,
        (0.0, 0.0),
    );
    assert!(a.arma((0.0, 0.0)), "a borda arma logo");
    assert_eq!(arrastar(&a, [6.0, 1.0]), Escrita::ZonaMorta([0.5, 0.5]));
    assert_eq!(arrastar(&a, [-6.0, 7.0]), Escrita::ZonaMorta([1.0, 0.5]));
    assert_eq!(
        edicoes(Escrita::ZonaMorta([0.5, 0.5])),
        Some(CameraFieldEdit::DeadZone([0.5, 0.5]))
    );
}

/// ⭐ Com a zona a ZERO as pegas abrem a três agarres do centro — fora do ícone, que fica no centro — e agarram-se.
#[test]
fn a_zona_zero_abre_as_pegas_fora_do_icone() {
    let m = que_segue([0.0, 0.0]);
    let tol = 0.1;
    let pegas = ph2d_camera_frame::pegas_da_zona(&m, tol);
    let c = ph2d_camera_frame::centro_da_zona(&m);
    for (i, p) in pegas.iter().enumerate() {
        let p = p.expect("quem segue tem as quatro pegas");
        let d = (p[0] - c[0]).abs().max((p[1] - c[1]).abs());
        assert!(
            (d - tol * ph2d_camera_frame::ZONA_PEGA_MIN).abs() < 1e-5,
            "{i}: {d}"
        );
        assert_eq!(alca_em(&m, p, tol), Some(Alca::ZonaMorta(i as u8)));
    }
    let sem = Moldura {
        segue: None,
        ..que_segue([0.3, 0.3])
    };
    assert_eq!(ph2d_camera_frame::pegas_da_zona(&sem, tol), [None; 4]);
}

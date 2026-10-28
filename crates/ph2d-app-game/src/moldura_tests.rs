//! Os gates da moldura da câmera (plano GEF 17 A2–A4).

use super::*;
use ph2d_ecs::SimWorld;
use ph2d_editor_core::{GameFormat, GamePreset};

const DT: f64 = 1.0 / 60.0;

fn nivel_2() -> SimWorld {
    let mut sim = SimWorld::new();
    let _ = crate::smoke::montar(&mut sim, &crate::registo_de_teste(), 2);
    sim
}

fn formato(p: GamePreset) -> GameFormat {
    GameFormat {
        preset: p,
        ..Default::default()
    }
}

/// O mapa mundo→pixel de uma câmara que cobre a janela (a mesma conta do `Camera2d`).
fn px(c: ([f32; 2], f32), janela: [f32; 2], p: [f32; 2]) -> [f32; 2] {
    let s = c.1 / janela[1];
    [
        janela[0] * 0.5 + (p[0] - c.0[0]) / s,
        janela[1] * 0.5 - (p[1] - c.0[1]) / s,
    ]
}

/// ⭐⭐ **A moldura é o rectângulo que o jogo mostra**, em dois formatos: a da câmera activa tem o centro da vista
/// do quadro, a proporção do formato, e com o «Look through» os cantos dela caem nos cantos da caixa do jogo.
#[test]
fn a_moldura_e_o_rectangulo_que_o_jogo_mostra() {
    let (janela, canvas) = ([1600.0, 900.0], [300.0, 60.0, 960.0, 700.0]);
    for p in [GamePreset::Wide16x9, GamePreset::GameBoy] {
        let f = formato(p);
        let mut sim = nivel_2();
        let (vista, _) =
            ph2d_app_components::camera_2d::update(&mut sim, f.aspect(), 1, DT, [0.0, 0.0]);
        let vista = vista.expect("o nível 2 tem câmera");
        let ms = molduras(sim.world_mut(), Some(vista.center), f.aspect(), None);
        let m = ms.iter().find(|m| m.ativa).expect("a activa");
        assert_eq!(m.centro, vista.center, "{p:?}");
        assert!((m.meia[0] / m.meia[1] - f.aspect()).abs() < 1e-4, "{p:?}");
        assert!((m.meia[1] * 2.0 - vista.height_world).abs() < 1e-4, "{p:?}");

        let caixa_px = crate::encaixe::caixa_do_jogo(canvas, f);
        let cam = crate::encaixe::na_janela(vista.center, vista.height_world, janela, caixa_px);
        let (lo, hi) = caixa(m);
        let a = px(cam, janela, [lo[0], hi[1]]);
        let b = px(cam, janela, [hi[0], lo[1]]);
        for (v, e) in [
            (a, [caixa_px[0], caixa_px[1]]),
            (b, [caixa_px[0] + caixa_px[2], caixa_px[1] + caixa_px[3]]),
        ] {
            assert!(
                (v[0] - e[0]).abs() < 0.05 && (v[1] - e[1]).abs() < 0.05,
                "{p:?}: canto {v:?}, a caixa do jogo diz {e:?}"
            );
        }
    }
}

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

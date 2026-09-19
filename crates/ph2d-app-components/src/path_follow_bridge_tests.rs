//! Os gates da ponte do SEGUIDOR DE CAMINHO (suplente #23) — onde o nome vira curva, e a curva
//! vira pose.
//!
//! ⚠️ **A fixtura é a MESMA meia circunferência da sonda do §5.0** (raio `2`, de `(−2, 0)` a
//! `(2, 0)`, a passar por `(0, 2)`): ela tem a resposta **fechada** em todos os pontos que estes
//! gates medem, logo nenhum número aqui é um valor observado a ser carimbado.

use super::*;
use ph2d_core::Vec2;
use ph2d_ecs::{Name, PathFollow, Timer, TimerRuntime, Timers, Transform, VecPathRef};
use ph2d_vec_scene::{VecPath, VecVertex, VertexKind};

/// A constante clássica do quarto de círculo por cúbica, já escalada ao raio `2`.
const K: f64 = 0.552_284_749_830_793_4 * 2.0;

fn arco() -> Vec<VecVertex> {
    vec![
        VecVertex {
            anchor: [-2.0, 0.0],
            in_handle: [-2.0, 0.0],
            out_handle: [-2.0, K],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [0.0, 2.0],
            in_handle: [-K, 2.0],
            out_handle: [K, 2.0],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [2.0, 0.0],
            in_handle: [2.0, K],
            out_handle: [2.0, 0.0],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
    ]
}

/// A cena: uma forma chamada `Trilho` com a pose dada, e um seguidor a meio do relógio.
fn cena(pose_do_trilho: Transform, pf: PathFollow) -> (SimWorld, VecScene, Entity) {
    let mut sim = SimWorld::new();
    let mut cena = VecScene::new();
    let id = cena.push_path(VecPath {
        verts: arco(),
        closed: false,
        ..VecPath::default()
    });
    sim.world_mut()
        .spawn((Name::new("Trilho"), pose_do_trilho, VecPathRef(id)));
    let cfg = Timers(vec![Timer {
        duration_us: 1_000_000,
        autostart: true,
        ..Timer::default()
    }]);
    let mut rt = TimerRuntime(cfg.0.iter().map(ph2d_ecs::timer::born).collect());
    rt.0[0].elapsed_us = 500_000; // meio percurso ⇒ o topo do arco
    let seguidor = sim
        .world_mut()
        .spawn((Name::new("Carro"), Transform::IDENTITY, pf, cfg, rt))
        .id();
    (sim, cena, seguidor)
}

fn segue(caminho: &str) -> PathFollow {
    PathFollow {
        caminho: caminho.into(),
        ..PathFollow::default()
    }
}

/// ⭐⭐⭐ **A meio do relógio, o seguidor está no TOPO do arco** — e o topo de uma meia
/// circunferência de raio `2` é `(0, 2)`, sem nenhum número observado.
#[test]
fn um_seguidor_pousa_na_curva() {
    let (mut sim, c, e) = cena(Transform::IDENTITY, segue("Trilho"));
    let pedidos = a_escrever(&mut sim, &c);
    assert_eq!(pedidos.len(), 1);
    assert_eq!(pedidos[0].entity, e);
    assert!(pedidos[0].mundo[0].abs() < 1e-4, "{:?}", pedidos[0]);
    assert!((pedidos[0].mundo[1] - 2.0).abs() < 1e-4, "{:?}", pedidos[0]);
}

/// ⭐ **Ele aponta para onde vai** — no topo do arco a tangente é horizontal, logo o ângulo é `0`.
///
/// ⚠️ E o CONTROLO está no gate irmão [`sem_alinhar_a_rotacao_e_de_quem_a_autorou`]: sem ele,
/// *«aponta para `0`»* e *«não escreve rotação nenhuma»* leem-se exactamente igual nesta fixtura.
#[test]
fn o_seguidor_aponta_para_onde_vai() {
    let (mut sim, c, _) = cena(Transform::IDENTITY, segue("Trilho"));
    let a = a_escrever(&mut sim, &c)[0]
        .angulo
        .expect("alinha por omissão");
    assert!(a.abs() < 1e-4, "apontou {a}");
}

#[test]
fn sem_alinhar_a_rotacao_e_de_quem_a_autorou() {
    let (mut sim, c, _) = cena(
        Transform::IDENTITY,
        PathFollow {
            alinha: false,
            ..segue("Trilho")
        },
    );
    assert!(a_escrever(&mut sim, &c)[0].angulo.is_none());
}

/// ⭐⭐ **O ÂNGULO soma-se à tangente, e ele está em GRAUS** — a arte que aponta para cima.
#[test]
fn o_angulo_e_em_graus_e_soma_se_a_tangente() {
    let (mut sim, c, _) = cena(
        Transform::IDENTITY,
        PathFollow {
            angulo: 90.0,
            ..segue("Trilho")
        },
    );
    let a = a_escrever(&mut sim, &c)[0].angulo.unwrap();
    assert!(
        (a - std::f32::consts::FRAC_PI_2).abs() < 1e-4,
        "leu {a} rad — se for ~90 ele guardou RADIANOS onde o descritor promete graus"
    );
}

/// ⭐⭐ **O LADO é PERPENDICULAR à curva** — no topo do arco a tangente aponta para `+X`, logo a
/// normal é `+Y` e meio metro de lado põe o objecto em `y = 2,5`.
///
/// ⚠️ **O `x` tem de ficar onde estava**: um deslocamento que mexesse nos dois eixos seria uma
/// translação e não um lado.
#[test]
fn o_lado_desloca_perpendicularmente_a_curva() {
    let (mut sim, c, _) = cena(
        Transform::IDENTITY,
        PathFollow {
            lado: 0.5,
            ..segue("Trilho")
        },
    );
    let p = a_escrever(&mut sim, &c)[0];
    assert!(p.mundo[0].abs() < 1e-4, "{p:?}");
    assert!((p.mundo[1] - 2.5).abs() < 1e-4, "{p:?}");
}

/// ⭐⭐⭐ **A CURVA VIVE EM MUNDO, e o gate mede-o com a forma DESLOCADA, RODADA e ESCALADA.**
///
/// ⚠️ É o bloco (E) da sonda do §5.0 a virar lei: o documento guarda LOCAL, e um seguidor que
/// ignorasse a pose da forma andaria a `5,099` de onde o artista a vê.
///
/// Meia volta (`π`) e escala `2` sobre o topo `(0, 2)` dá `(0, −4)`, mais a translação `(5, −3)`
/// ⇒ `(5, −7)`. E a tangente `+X` rodada meia volta aponta para `−X` ⇒ `π`.
#[test]
fn a_pose_da_forma_leva_o_seguidor_com_ela() {
    let (mut sim, c, _) = cena(
        Transform {
            translation: Vec2::new(5.0, -3.0),
            rotation: std::f32::consts::PI,
            scale: Vec2::new(2.0, 2.0),
            ..Transform::IDENTITY
        },
        segue("Trilho"),
    );
    let p = a_escrever(&mut sim, &c)[0];
    assert!((p.mundo[0] - 5.0).abs() < 1e-3, "{p:?}");
    assert!((p.mundo[1] + 7.0).abs() < 1e-3, "{p:?}");
    let a = p.angulo.unwrap().abs();
    assert!(
        (a - std::f32::consts::PI).abs() < 1e-3,
        "apontou {a} em vez de meia volta"
    );
}

/// ⛔ **Um nome que não casa é INERTE** — e não um estouro nem uma pose na origem.
#[test]
fn um_nome_que_nao_existe_e_inerte() {
    let (mut sim, c, _) = cena(Transform::IDENTITY, segue("Pista Que Nao Existe"));
    assert!(a_escrever(&mut sim, &c).is_empty());
}

/// ⛔ **Um objecto com aquele nome mas SEM geometria também é inerte** — a forma é a que carrega um
/// `VecPathRef`, e apontar a um sprite não é um erro do artista, é um caminho ainda por desenhar.
#[test]
fn um_nome_sem_geometria_e_inerte() {
    let (mut sim, c, _) = cena(Transform::IDENTITY, segue("Carro"));
    assert!(a_escrever(&mut sim, &c).is_empty());
}

/// ⭐⭐⭐ **A curva percorrida é a COZIDA, e não a autorada** — o gate usa uma QUINA VIVA, que é o
/// único sítio onde as duas diferem sem mudar os vértices.
///
/// ⚠️ **A régua é o COMPRIMENTO**: arredondar uma quina corta o canto, logo o percurso encolhe. Um
/// seguidor que andasse na fonte andaria ao lado do que está na tela.
#[test]
fn o_seguidor_anda_na_curva_cozida() {
    let mut sim = SimWorld::new();
    let mut c = VecScene::new();
    // Um «L» de quina viva: dois segmentos rectos que se encontram num ângulo recto.
    let quina = |r: f64| {
        vec![
            VecVertex::corner([-1.0, 0.0]),
            VecVertex {
                corner_radius: r,
                ..VecVertex::corner([0.0, 0.0])
            },
            VecVertex::corner([0.0, 1.0]),
        ]
    };
    let id = c.push_path(VecPath {
        verts: quina(0.5),
        closed: false,
        ..VecPath::default()
    });
    sim.world_mut()
        .spawn((Name::new("Trilho"), Transform::IDENTITY, VecPathRef(id)));
    let cfg = Timers(vec![Timer {
        duration_us: 1_000_000,
        autostart: true,
        ..Timer::default()
    }]);
    let mut rt = TimerRuntime(cfg.0.iter().map(ph2d_ecs::timer::born).collect());
    rt.0[0].elapsed_us = 500_000;
    sim.world_mut()
        .spawn((Transform::IDENTITY, segue("Trilho"), cfg, rt));

    let com_quina = a_escrever(&mut sim, &c)[0].mundo;
    // O CONTROLO: a mesma cena com a quina afiada.
    c.path_mut(id).unwrap().verts = quina(0.0);
    let afiada = a_escrever(&mut sim, &c)[0].mundo;
    let d = (com_quina[0] - afiada[0]).hypot(com_quina[1] - afiada[1]);
    assert!(
        d > 1e-3,
        "a quina viva não mudou o percurso ({com_quina:?} contra {afiada:?}) — o seguidor está a \
         andar na fonte"
    );
}

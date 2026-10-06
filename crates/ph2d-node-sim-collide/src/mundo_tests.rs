//! Os gates do aperto de mão com o MUNDO DE CONTACTO (doc 121 §9.20) — pela porta do nó, com a chave.

use super::{RADIUS_AUTO, SHAPE_BOWL, SHAPE_PLANE, collide_com_chave};
use ph2d_contact::obstaculo::{self, FormaFixa};
use ph2d_nodegraph::attr::{COLLIDER_BOX_COLUMN, Column, Stream};

/// Uma caixa a meio do chão do plano (`y = −2`), a cair.
fn afundada() -> Stream {
    Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, -2.05]]))
        .with("vel", Column::Vec2(vec![[0.0, -1.0]]))
        .with(COLLIDER_BOX_COLUMN, Column::Vec2(vec![[0.11, 0.11]]))
}

fn plano(s: &Stream, chave: u32) -> Stream {
    collide_com_chave(
        s,
        SHAPE_PLANE,
        -2.0,
        [0.0, 0.0],
        2.0,
        0.3,
        0.6,
        (RADIUS_AUTO, 0.25, 1.0),
        [0.0, 1.0],
        (0.0, 0),
        [1.0, 0.5],
        chave,
    )
}

fn p(s: &Stream) -> [f32; 2] {
    match s.get("P") {
        Some(Column::Vec2(v)) => v[0],
        _ => panic!("sem P"),
    }
}

/// ⭐⭐ **Sem recibo o nó projecta como sempre; COM o recibo do mundo, não mexe na peça** — só lhe
/// escreve o `hit`. Projectá-la por cima do solver desfaria o contacto que ele guarda.
#[test]
fn with_the_receipt_the_world_owns_the_piece_and_the_node_only_detects() {
    let sem = plano(&afundada(), 5);
    assert!(
        p(&sem)[1] > -2.0,
        "sem recibo, a peca sai do chao: {:?}",
        p(&sem)
    );
    let mut com = afundada();
    obstaculo::passa_recibo(&mut com, 5);
    let out = plano(&com, 5);
    assert_eq!(
        p(&out),
        [0.0, -2.05],
        "com recibo, a peca fica onde o mundo a pos"
    );
    assert!(
        matches!(out.get(super::HIT_COL), Some(Column::Scalar(h)) if h[0] > 0.0),
        "e o toque continua a ser detectado"
    );
    // O recibo de OUTRO nó não conta.
    let mut alheio = afundada();
    obstaculo::passa_recibo(&mut alheio, 6);
    assert!(p(&plano(&alheio, 5))[1] > -2.0);
}

/// ⭐ **O nó DECLARA-SE a cada passagem e consome o próprio recibo** — e só quando alguém declara
/// colisor (uma cena sem colisores sai como sempre saiu, sem colunas novas).
#[test]
fn the_node_declares_itself_consumes_its_receipt_and_is_silent_without_colliders() {
    let mut com = afundada();
    obstaculo::passa_recibo(&mut com, 5);
    let out = plano(&com, 5);
    let d = obstaculo::declarados(&out);
    assert_eq!(d.len(), 1);
    assert_eq!(
        d[0].1.forma,
        FormaFixa::Plano {
            normal: [0.0, 1.0],
            altura: -2.0
        }
    );
    assert!(!obstaculo::tem_recibo(&out, 5), "o recibo e' consumido");
    let ponto = Stream::new(1)
        .with("P", Column::Vec2(vec![[0.0, 0.0]]))
        .with("vel", Column::Vec2(vec![[0.0, 0.0]]));
    let taca = collide_com_chave(
        &ponto,
        SHAPE_BOWL,
        -2.0,
        [0.0, 0.0],
        2.0,
        0.3,
        0.6,
        (RADIUS_AUTO, 0.25, 1.0),
        [0.0, 1.0],
        (0.0, 0),
        [1.0, 0.5],
        5,
    );
    assert!(
        taca.columns().all(|(k, _)| !obstaculo::e_da_porta(k)),
        "sem colisor declarado nao ha' declaracao nenhuma"
    );
}

//! Os gates do mundo no espaço do objecto Motion (doc ObjectsAndComponents 01 §14, item 6).

use super::*;
use crate::motion_objeto::Dono;
use ph2d_nodegraph::external::{curve_of, in_space, pose_of, position_of};

/// Dono em (400, 150), rodado 20°, escala (2, 0.5) — não uniforme de propósito.
fn afim_do_dono() -> Afim {
    let t = ph2d_ecs::Transform {
        translation: ph2d_core::Vec2::new(400.0, 150.0),
        rotation: 20f32.to_radians(),
        scale: ph2d_core::Vec2::new(2.0, 0.5),
        ..ph2d_ecs::Transform::IDENTITY
    };
    ph2d_ecs::GlobalTransform::from_transform(t).affine()
}

fn perto(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3
}

fn um_ponto(p: [f32; 2]) -> Stream {
    Stream::new(1).with("P", Column::Vec2(vec![p]))
}

fn p_de(s: &Stream) -> [f32; 2] {
    match s.get("P") {
        Some(Column::Vec2(v)) => v[0],
        other => panic!("sem P: {other:?}"),
    }
}

/// O que depende do espaço tem espécie; a aparência, as tabelas e o deslocado no tempo, não.
#[test]
fn so_o_que_vem_do_mundo_muda_de_espaco() {
    assert_eq!(especie(CURSOR), Some(Especie::Pontos));
    assert_eq!(especie(&position_of("Alvo")), Some(Especie::Pontos));
    assert_eq!(especie(&curve_of("Rio")), Some(Especie::Pontos));
    assert_eq!(especie(CAMERA), Some(Especie::Vista));
    assert_eq!(especie(&pose_of("Alvo")), Some(Especie::Pose));
    assert_eq!(especie("Alvo"), None, "a aparência mora na origem");
    assert_eq!(especie(&external::appearance_of("Alvo", 0.5)), None);
    assert_eq!(
        especie(&in_space(CURSOR, 7)),
        None,
        "uma cópia nunca é copiada"
    );
}

#[test]
fn a_inversa_desfaz_a_afim() {
    let a = afim_do_dono();
    let i = inversa(&a).expect("invertível");
    for p in [[0.0, 0.0], [10.0, -3.0], [-250.0, 80.0]] {
        assert!(perto(ponto(&i, ponto(&a, p)), p));
    }
    assert_eq!(
        inversa(&[0.0, 0.0, 0.0, 0.0, 5.0, 5.0]),
        None,
        "escala zero"
    );
}

/// Um ponto do mundo que o dono leva para `p` volta a `p`; a vista leva o comprimento.
#[test]
fn pontos_e_vista_voltam_ao_espaco_do_objecto() {
    let a = afim_do_dono();
    let i = inversa(&a).unwrap();
    let local = [12.0, -7.0];
    let mundo = um_ponto(ponto(&a, local));
    assert!(perto(p_de(&converte(Especie::Pontos, &mundo, &i)), local));

    let vista = um_ponto(ponto(&a, local))
        .with("size", Column::Vec2(vec![[800.0, 450.0]]))
        .with(
            ph2d_node_source_camera::ZOOM_COLUMN,
            Column::Scalar(vec![2.0]),
        );
    let v = converte(Especie::Vista, &vista, &i);
    let s = 1.0f32; // √|det| de diag(2, 0.5) rodada
    assert!(perto(p_de(&v), local));
    let Some(Column::Vec2(size)) = v.get("size") else {
        panic!()
    };
    assert!(perto(size[0], [800.0 / s, 450.0 / s]));
    let a2 = [2.0, 0.0, 0.0, 2.0, 0.0, 0.0];
    let v2 = converte(Especie::Vista, &vista, &inversa(&a2).unwrap());
    let (Some(Column::Vec2(size)), Some(Column::Scalar(zoom))) =
        (v2.get("size"), v2.get(ph2d_node_source_camera::ZOOM_COLUMN))
    else {
        panic!()
    };
    assert!(
        perto(size[0], [400.0, 225.0]),
        "o dono 2× vê metade das unidades"
    );
    assert!(
        (zoom[0] - 4.0).abs() < 1e-4,
        "e cada unidade dele tem o dobro dos pixels"
    );
}

/// ⭐ A pose de um objecto filho do dono, convertida, é a pose LOCAL dele.
#[test]
fn a_pose_de_um_filho_volta_a_ser_a_local() {
    let pai = ph2d_ecs::Transform {
        translation: ph2d_core::Vec2::new(400.0, 150.0),
        rotation: 20f32.to_radians(),
        scale: ph2d_core::Vec2::new(2.0, 2.0),
        ..ph2d_ecs::Transform::IDENTITY
    };
    let filho = ph2d_ecs::Transform {
        translation: ph2d_core::Vec2::new(5.0, 1.0),
        rotation: 0.3,
        scale: ph2d_core::Vec2::new(1.5, 0.75),
        ..ph2d_ecs::Transform::IDENTITY
    };
    let mundo = ph2d_ecs::Transform::compose(pai, filho);
    let s = Stream::new(1)
        .with("rotation", Column::Scalar(vec![mundo.rotation]))
        .with("size", Column::Vec2(vec![[mundo.scale.x, mundo.scale.y]]));
    let i = inversa(&ph2d_ecs::GlobalTransform::from_transform(pai).affine()).unwrap();
    let p = converte(Especie::Pose, &s, &i);
    let (Some(Column::Scalar(r)), Some(Column::Vec2(sz))) = (p.get("rotation"), p.get("size"))
    else {
        panic!()
    };
    assert!((r[0] - 0.3).abs() < 1e-4, "rotação {}", r[0]);
    assert!(perto(sz[0], [1.5, 0.75]), "escala {:?}", sz[0]);
}

fn dono(afim: Afim) -> Dono {
    Dono {
        entity: 0,
        visible: true,
        afim,
    }
}

fn x_do_cursor(m: &mut MotionState, no: NodeId) -> [f32; 2] {
    let v = m
        .pump
        .cook
        .cook(&m.doc.graph, &m.registry, no, 0.0)
        .expect("coze");
    let x = |k: usize| match v[k].as_stream().columns().next() {
        Some((_, Column::Scalar(s))) => s[0],
        other => panic!("{other:?}"),
    };
    [x(0), x(1)]
}

use ph2d_nodegraph::graph::NodeId;

/// ⭐⭐ **O caminho inteiro, pela porta do PRODUTO** (`publish_all`): o mesmo rato lido por um nó
/// do dono deslocado chega no espaço dele; por um nó sem dono, no mundo. O dono volta à origem ⇒
/// o nó recozinha e lê o mundo outra vez (a cópia sumiu e o fingerprint viu).
#[test]
fn o_rato_chega_ao_no_no_espaco_do_dono() {
    let mut m = MotionState::new();
    let do_dono = m.doc.graph.add_node("value.cursor");
    let solto = m.doc.graph.add_node("value.cursor");
    let sid = 77;
    m.doc.owners.insert(do_dono, sid);
    let a = afim_do_dono();
    m.objeto.donos.insert(sid, dono(a));
    let local = [30.0, -12.0];
    let mundo = ponto(&a, local);

    let publica_rato = |m: &mut MotionState| {
        m.pump
            .cook
            .set_external(CURSOR.to_string(), um_ponto(mundo));
        crate::motion_externals::publish_all(m, 0.0);
    };
    publica_rato(&mut m);
    assert!(perto(x_do_cursor(&mut m, do_dono), local));
    assert!(perto(x_do_cursor(&mut m, solto), mundo));

    m.objeto.donos.insert(sid, dono(IDENTIDADE));
    publica_rato(&mut m);
    assert!(
        m.pump
            .cook
            .externals()
            .keys()
            .all(|k| !external::is_spaced(k)),
        "na origem não há cópia"
    );
    assert!(perto(x_do_cursor(&mut m, do_dono), mundo));
}

/// ⭐ Sem dono fora da origem, nada é copiado: o caminho de sempre.
#[test]
fn na_origem_nada_e_copiado() {
    let mut m = MotionState::new();
    let n = m.doc.graph.add_node("value.cursor");
    m.doc.owners.insert(n, 5);
    m.objeto.donos.insert(5, dono(IDENTIDADE));
    m.pump
        .cook
        .set_external(CURSOR.to_string(), um_ponto([1.0, 2.0]));
    let antes = m.pump.cook.externals().clone();
    publica(&mut m);
    assert_eq!(m.pump.cook.externals(), &antes);
}

/// ⭐ **Uma conversão serve os dois caminhos** porque a placa não tem porta própria para o mundo: o
/// único leitor de externo é `ctx.external` (um nó da CPU), e o `ph2d-gpu-cook` não lê a tabela. Se
/// um dia a ler, este censo reprova e a conversão tem de ir lá também.
#[test]
fn a_placa_nao_le_o_mundo_por_outra_porta() {
    fn varre(dir: &std::path::Path, achados: &mut Vec<String>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                varre(&p, achados);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let s = std::fs::read_to_string(&p).unwrap();
                for (i, l) in s.lines().enumerate() {
                    let codigo = l.split("//").next().unwrap_or("");
                    if codigo.contains(".externals()") || codigo.contains(".external(") {
                        achados.push(format!("{}:{}", p.display(), i + 1));
                    }
                }
            }
        }
    }
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../ph2d-gpu-cook/src");
    assert!(
        raiz.is_dir(),
        "a régua tem de varrer alguma coisa: {}",
        raiz.display()
    );
    let mut achados = Vec::new();
    varre(&raiz, &mut achados);
    assert!(achados.is_empty(), "a placa lê externos: {achados:?}");
}

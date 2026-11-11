use super::*;
use ph2d_ecs::SimWorld;

#[test]
fn sem_look_a_forma_resolvida_e_a_mesma_bit_a_bit() {
    let mut sim = SimWorld::new();
    let col = Collider {
        offset: [0.3, -0.7],
        ..Collider::default()
    };
    let e = sim.world_mut().spawn(col).id();
    assert_eq!(forma_resolvida(sim.world(), e, &col), col);
    sim.world_mut().entity_mut(e).insert(ShapeLook::default());
    assert_eq!(forma_resolvida(sim.world(), e, &col), col);
}

#[test]
fn o_offset_para_centro_desfaz_a_porta() {
    let mut sim = SimWorld::new();
    let col = Collider {
        shape: ColliderShape::Capsule {
            half_height: 0.4,
            radius: 0.2,
        },
        offset: [0.1, 0.2],
        ..Collider::default()
    };
    let e = sim
        .world_mut()
        .spawn((
            col,
            ShapeLook {
                pivot: [0.5, -1.0],
                ..ShapeLook::default()
            },
        ))
        .id();
    let r = forma_resolvida(sim.world(), e, &col);
    assert_eq!(r.offset, [0.1 - 0.5 * 0.2, 0.2 + 0.6]);
    let volta = offset_para_centro(sim.world(), e, col.shape, r.offset);
    assert!((volta[0] - col.offset[0]).abs() < 1e-6 && (volta[1] - col.offset[1]).abs() < 1e-6);
}

#[test]
fn o_sanitize_poe_cor_e_opacidade_na_faixa() {
    let mut l = ShapeLook {
        pivot: [f32::NAN, 3.0],
        tint: [2.0, -1.0, 0.5, f32::INFINITY],
        opacity: 7.0,
        cor_propria: true,
        largura: 0.0,
        tracejado: true,
        traco: [f32::NAN, -3.0],
        traco_proprio: true,
    };
    l.sanitize();
    assert_eq!(l.pivot, [0.0, 3.0]);
    assert_eq!(l.tint, [1.0, 0.0, 0.5, 1.0]);
    assert_eq!(l.opacity, 1.0);
    assert_eq!(
        (l.largura, l.traco),
        (TRACO_MINIMO, [6.0, TRACO_MINIMO]),
        "o traço nunca some"
    );
    l.opacity = 0.0;
    l.sanitize();
    assert_eq!(l.opacity, OPACIDADE_MINIMA, "a forma nunca some de todo");
}

/// O Look que o artista deu à forma solta: cor própria, meia opacidade e o pivô no canto.
fn look_do_artista() -> ShapeLook {
    ShapeLook {
        pivot: [-1.0, -1.0],
        tint: [0.2, 0.9, 0.1, 1.0],
        opacity: 0.5,
        cor_propria: true,
        largura: 5.0,
        tracejado: true,
        traco: [2.0, 9.0],
        traco_proprio: true,
    }
}

fn padrao_com_pivo() -> ShapeLook {
    ShapeLook {
        pivot: [-1.0, -1.0],
        ..ShapeLook::default()
    }
}

#[test]
fn ao_virar_corpo_a_forma_solta_veste_o_padrao_do_corpo_e_guarda_o_pivo() {
    // As três entradas do `torna_corpo`: forma solta sem imagem, com imagem, e um grupo com a forma dentro.
    for caso in 0..3 {
        let mut sim = SimWorld::new();
        let w = sim.world_mut();
        let forma = w.spawn((Collider::default(), look_do_artista())).id();
        let alvo = if caso == 2 {
            let g = w.spawn(ph2d_ecs::Transform::IDENTITY).id();
            crate::migra_formas::pendura(w, forma, g, None);
            g
        } else {
            forma
        };
        crate::torna_corpo(w, alvo, crate::BodyKind::Dynamic, [0.5, 0.5], caso == 1);
        let mut q = w.query::<&ShapeLook>();
        let looks: Vec<ShapeLook> = q.iter(w).copied().collect();
        assert_eq!(looks, vec![padrao_com_pivo()], "caso {caso}");
    }
}

#[test]
fn o_que_o_artista_muda_depois_de_associar_fica_e_o_mesmo_dono_nao_repoe() {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let forma = w.spawn((Collider::default(), ShapeLook::default())).id();
    let corpo = crate::torna_corpo(w, forma, crate::BodyKind::Static, [0.5, 0.5], false);
    *w.get_mut::<ShapeLook>(forma).expect("look") = look_do_artista();
    let antes = donos_dos_looks(w);
    assert_eq!(antes, vec![(forma, Some(corpo))]);
    look_ao_associar(w, &antes);
    assert_eq!(w.get::<ShapeLook>(forma), Some(&look_do_artista()));
}

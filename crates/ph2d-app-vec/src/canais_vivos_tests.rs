//! Gates do [`super`] (`canais_vivos.rs`) — perfil, padrão e texto no caminho, Blend e Envelope.

use super::*;
use ph2d_ecs::VecEnvelopeChild;

fn k(campo: u16, j: u16) -> FieldKey {
    FieldKey::new(VETOR, campo, 0, j)
}

fn mundo() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(()).id();
    (sim, e)
}

/// Escrever o que se leu, em TODO campo publicado, não muda o mundo (a régua do eco).
fn identidade(sim: &mut SimWorld, e: Entity, caminho: bool) {
    let mut cs = Vec::new();
    campos(sim, e, caminho, &mut cs);
    assert!(!cs.is_empty());
    for kk in cs {
        assert!(e_vivo(kk), "{kk:?}");
        let v = ler(sim, e, kk).expect("publicado ⇒ legível");
        escrever(sim, e, kk, v);
        assert_eq!(ler(sim, e, kk), Some(v), "{kk:?}");
    }
}

/// ⭐ O perfil: a ausência é o neutro, então um caminho sem perfil publica o «existe» = 0;
/// ligá-lo arma o neutro de duas paragens, e as paragens da curva vêm a seguir; desligar remove.
#[test]
fn o_perfil_arma_publica_as_paragens_e_desarma() {
    let (mut sim, e) = mundo();
    let mut cs = Vec::new();
    campos(&sim, e, false, &mut cs);
    assert!(cs.is_empty(), "sem caminho, sem perfil, nada");
    campos(&sim, e, true, &mut cs);
    assert_eq!(cs, vec![k(c::PERFIL_EXISTE, 0)]);
    assert_eq!(ler(&sim, e, k(c::PERFIL_EXISTE, 0)), Some(0.0));
    assert_eq!(ler(&sim, e, k(c::PERFIL_N, 0)), None);
    escrever(&mut sim, e, k(c::PERFIL_POS, 0), 0.3);
    assert!(
        sim.world().get::<VecStrokeProfile>(e).is_none(),
        "ausente fica ausente"
    );

    escrever(&mut sim, e, k(c::PERFIL_EXISTE, 0), 1.0);
    escrever(&mut sim, e, k(c::PERFIL_N, 0), 3.0);
    for (j, (p, m)) in [(0.0, 0.5), (0.4, 2.0), (1.0, 0.25)]
        .into_iter()
        .enumerate()
    {
        let j = u16::try_from(j).unwrap();
        escrever(&mut sim, e, k(c::PERFIL_POS, j), p);
        escrever(&mut sim, e, k(c::PERFIL_MULT, j), m);
    }
    let s = sim
        .world()
        .get::<VecStrokeProfile>(e)
        .unwrap()
        .stops
        .clone();
    let lidos: Vec<(f32, f32)> = s.as_slice().iter().map(|x| (f(x.pos), f(x.mult))).collect();
    assert_eq!(lidos, vec![(0.0, 0.5), (0.4, 2.0), (1.0, 0.25)]);
    identidade(&mut sim, e, true);
    escrever(&mut sim, e, k(c::PERFIL_EXISTE, 0), 0.0);
    assert!(sim.world().get::<VecStrokeProfile>(e).is_none());
}

/// O padrão no caminho: os seis campos vão e voltam; a rotação `0` remove o componente (a lei
/// do `pattern_live::set_rotation`); sem vínculo, nada.
#[test]
fn o_padrao_no_caminho_vai_e_volta_e_a_rotacao_zero_sai() {
    let (mut sim, e) = mundo();
    escrever(&mut sim, e, k(c::PADRAO_CAMINHO_ROTACAO, 0), 30.0);
    assert!(
        sim.world().get::<VecPatternRotation>(e).is_none(),
        "sem vínculo, sem rotação"
    );
    sim.world_mut()
        .entity_mut(e)
        .insert(VecPatternPath::default());
    for (campo, v) in [
        (c::PADRAO_CAMINHO_INICIO, 0.1),
        (c::PADRAO_CAMINHO_FIM, 0.8),
        (c::PADRAO_CAMINHO_ESPACO, 1.5),
        (c::PADRAO_CAMINHO_DESLOCA, -2.0),
        (c::PADRAO_CAMINHO_ESPELHO, 1.0),
        (c::PADRAO_CAMINHO_ROTACAO, 45.0),
    ] {
        escrever(&mut sim, e, k(campo, 0), v);
        assert_eq!(ler(&sim, e, k(campo, 0)), Some(v), "campo {campo}");
    }
    identidade(&mut sim, e, false);
    escrever(&mut sim, e, k(c::PADRAO_CAMINHO_ROTACAO, 0), 0.0);
    assert!(sim.world().get::<VecPatternRotation>(e).is_none());
    assert_eq!(ler(&sim, e, k(c::PADRAO_CAMINHO_ROTACAO, 0)), Some(0.0));
}

/// O texto no caminho e o Blend vivo: vão e voltam; os passos prendem a `1..=MAX_BLEND_STEPS`.
#[test]
fn o_texto_no_caminho_e_o_blend_vao_e_voltam() {
    let (mut sim, e) = mundo();
    sim.world_mut().entity_mut(e).insert((
        VecTextPath {
            path: 3,
            start_offset: 0.0,
            flip: false,
        },
        VecBlend::new(vec![1, 2], 5),
    ));
    escrever(&mut sim, e, k(c::TEXTO_CAMINHO_INICIO, 0), 0.6);
    escrever(&mut sim, e, k(c::TEXTO_CAMINHO_ESPELHO, 0), 1.0);
    let t = *sim.world().get::<VecTextPath>(e).unwrap();
    assert_eq!((t.path, t.start_offset, t.flip), (3, 0.6, true));
    escrever(&mut sim, e, k(c::BLEND_PASSOS, 0), 7.4);
    assert_eq!(sim.world().get::<VecBlend>(e).unwrap().steps, 7);
    escrever(&mut sim, e, k(c::BLEND_PASSOS, 0), 0.0);
    assert_eq!(sim.world().get::<VecBlend>(e).unwrap().steps, 1);
    escrever(&mut sim, e, k(c::BLEND_PASSOS, 0), 1e9);
    assert_eq!(
        sim.world().get::<VecBlend>(e).unwrap().steps,
        ph2d_tool_vector::params::MAX_BLEND_STEPS
    );
    assert_eq!(sim.world().get::<VecBlend>(e).unwrap().sources, vec![1, 2]);
    identidade(&mut sim, e, false);
}

/// ⭐ O Envelope: a geometria inteira é UM grupo (cantos, alças, tipo, warp, pinos, curvatura);
/// os pinos têm contador; o warp `0` é nenhum e `k+1` o estilo `k`; os filhos não são canal.
#[test]
fn o_envelope_e_um_grupo_e_vai_e_volta() {
    let (mut sim, e) = mundo();
    let corners = [[0.0, 0.0], [10.0, 0.0], [10.0, 5.0], [0.0, 5.0]];
    let edges = [[[3.0, 0.0], [7.0, 0.0]]; 4];
    let filho = VecEnvelopeChild {
        path: 4,
        source: vec![1, 2, 3],
    };
    sim.world_mut()
        .entity_mut(e)
        .insert(VecEnvelope::at_rest(vec![filho.clone()], corners, edges));
    let mut cs = Vec::new();
    campos(&sim, e, false, &mut cs);
    assert_eq!(cs.len(), 4 * 2 + 8 * 2 + 4, "sem pinos ainda");
    assert!(cs.iter().all(|kk| kk.mesmo_grupo(cs[0])), "um grupo só");
    escrever(&mut sim, e, k(c::ENVELOPE_CANTO_Y, 2), 7.5);
    escrever(&mut sim, e, k(c::ENVELOPE_ALCA_X, 5), 8.0);
    escrever(&mut sim, e, k(c::ENVELOPE_TIPO, 0), 1.0);
    escrever(&mut sim, e, k(c::ENVELOPE_WARP, 0), 3.0);
    escrever(&mut sim, e, k(c::ENVELOPE_CURVA, 0), -0.25);
    escrever(&mut sim, e, k(c::ENVELOPE_N_PINOS, 0), 2.0);
    escrever(&mut sim, e, k(c::ENVELOPE_PINO + 3, 1), 9.0);
    let env = sim.world().get::<VecEnvelope>(e).unwrap().clone();
    assert_eq!(env.corners[2], [10.0, 7.5]);
    assert_eq!(env.edges[2][1], [8.0, 0.0], "alça 5 = aresta 2, lado 1");
    assert_eq!(env.kind, EnvelopeKind::Mesh);
    assert_eq!(env.warp, Some(EnvelopeWarp::ALL[2]));
    assert_eq!(env.bend, -0.25);
    assert_eq!(env.pins.len(), 2);
    assert_eq!(env.pins[1][1][1], 9.0, "pino 1, destino y");
    assert_eq!(env.children, vec![filho], "os filhos são estrutura");
    identidade(&mut sim, e, false);
    escrever(&mut sim, e, k(c::ENVELOPE_WARP, 0), 0.0);
    assert_eq!(sim.world().get::<VecEnvelope>(e).unwrap().warp, None);
    escrever(&mut sim, e, k(c::ENVELOPE_TIPO, 0), 9.0);
    assert_eq!(
        sim.world().get::<VecEnvelope>(e).unwrap().kind,
        EnvelopeKind::Pins
    );
}

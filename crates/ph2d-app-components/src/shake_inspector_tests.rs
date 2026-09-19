//! Os gates da ponte do Inspector do abanão. ⚠️ Eles medem as **duas varreduras do mundo** — as
//! colunas que o painel não consegue derivar sozinho — e a **ida e volta** de cada edição.

use super::*;
use ph2d_ecs::{GameCamera, Name, StableId, Transform};

fn cena() -> (SimWorld, Entity, Entity) {
    let mut sim = SimWorld::new();
    let cam = sim
        .world_mut()
        .spawn((
            Name::new("Camera"),
            Transform::IDENTITY,
            StableId(1),
            GameCamera::default(),
            CameraShake::default(),
        ))
        .id();
    let bomba = sim
        .world_mut()
        .spawn((
            Name::new("Bomba"),
            Transform::IDENTITY,
            StableId(2),
            ShakeEmitter(vec![ShakeSource::default()]),
        ))
        .id();
    (sim, cam, bomba)
}

/// ⭐⭐⭐ **A coluna que aponta para OUTRO objecto** — sem ela, *«o emissor não funciona»* e *«não há
/// quem abane»* leem-se exactamente igual.
#[test]
fn o_emissor_sabe_se_existe_quem_abane() {
    let (mut sim, cam, bomba) = cena();
    let com = build_info_emitter(&mut sim, bomba.to_bits(), true, 1).unwrap();
    assert!(com.ha_camera_que_treme);

    // ⭐ O CONTROLO: tirar o componente da câmera tem de mudar a resposta.
    sim.world_mut().entity_mut(cam).remove::<CameraShake>();
    let sem = build_info_emitter(&mut sim, bomba.to_bits(), true, 1).unwrap();
    assert!(!sem.ha_camera_que_treme);
}

/// ⚠️ **Ela pergunta pelo COMPONENTE e não pela câmera ACTIVA** — uma segunda câmera que treme e
/// que o artista vai activar a seguir não é um defeito.
#[test]
fn uma_camera_desligada_que_treme_ainda_conta() {
    let (mut sim, cam, bomba) = cena();
    sim.world_mut().entity_mut(cam).insert(GameCamera {
        active: false,
        ..GameCamera::default()
    });
    let info = build_info_emitter(&mut sim, bomba.to_bits(), true, 1).unwrap();
    assert!(
        info.ha_camera_que_treme,
        "uma câmera desligada que treme ainda é quem vai abanar"
    );
}

/// ⭐ **«esta é a câmera que MANDA?»** — a outra varredura.
#[test]
fn a_seccao_da_camera_diz_se_ela_manda() {
    let (mut sim, cam, _) = cena();
    assert!(
        build_info_camera(&mut sim, cam.to_bits(), true, 1)
            .unwrap()
            .activa
    );

    // Uma segunda câmera com prioridade maior rouba o comando.
    sim.world_mut().spawn((
        Name::new("Outra"),
        Transform::IDENTITY,
        StableId(9),
        GameCamera {
            priority: 5,
            ..GameCamera::default()
        },
    ));
    assert!(
        !build_info_camera(&mut sim, cam.to_bits(), true, 1)
            .unwrap()
            .activa,
        "com outra a mandar, esta secção tem de dizer que não é ela"
    );
}

/// ⚠️ **O TRAUMA vem do VIVO**, e é a única coluna que muda sem ninguém editar nada.
#[test]
fn o_trauma_vem_do_vivo_e_zero_sem_ele() {
    let (mut sim, cam, _) = cena();
    assert_eq!(
        build_info_camera(&mut sim, cam.to_bits(), true, 1)
            .unwrap()
            .trauma,
        0.0,
        "sem o componente vivo o trauma é zero e não `None`"
    );
    sim.world_mut().entity_mut(cam).insert(CameraShakeRuntime {
        trauma: 0.42,
        t: 1.0,
    });
    assert_eq!(
        build_info_camera(&mut sim, cam.to_bits(), true, 1)
            .unwrap()
            .trauma,
        0.42
    );
}

/// ⚠️ **A secção só existe se o objecto TIVER o componente** — ADR-0166, e as duas metades.
#[test]
fn sem_componente_nao_ha_seccao() {
    let (mut sim, cam, bomba) = cena();
    assert!(build_info_camera(&mut sim, bomba.to_bits(), true, 1).is_none());
    assert!(build_info_emitter(&mut sim, cam.to_bits(), true, 1).is_none());
}

/// ⭐⭐ **A IDA E VOLTA dos cinco números da câmera** — cada edição escreve o SEU campo e **não toca
/// nos vizinhos**, que é a metade que um `if` por campo mal escrito quebra em silêncio.
#[test]
fn cada_edicao_da_camera_escreve_o_seu_campo() {
    let (mut sim, cam, _) = cena();
    let b = cam.to_bits();
    let antes = build_info_camera(&mut sim, b, true, 1).unwrap();
    assert!(apply_shake(&mut sim, &[(b, SE::Amplitude(1.5))]));
    let d = build_info_camera(&mut sim, b, true, 1).unwrap();
    assert_eq!(d.amplitude, 1.5);
    assert_eq!(
        (d.frequencia, d.decaimento, d.expoente, d.semente),
        (
            antes.frequencia,
            antes.decaimento,
            antes.expoente,
            antes.semente
        ),
        "a amplitude não pode tocar nos vizinhos"
    );
    for (e, lê) in [(SE::Frequencia(7.0), 7.0), (SE::Decaimento(4.0), 4.0)] {
        apply_shake(&mut sim, &[(b, e.clone())]);
        let d = build_info_camera(&mut sim, b, true, 1).unwrap();
        let v = match e {
            SE::Frequencia(_) => d.frequencia,
            _ => d.decaimento,
        };
        assert_eq!(v, lê);
    }
    apply_shake(&mut sim, &[(b, SE::Semente(777))]);
    assert_eq!(
        build_info_camera(&mut sim, b, true, 1).unwrap().semente,
        777
    );
}

/// ⛔⛔ **O expoente é COAGIDO à faixa da lei, e as duas pontas** — um `0` apagaria o trauma.
#[test]
fn o_expoente_e_coagido_a_faixa_da_lei() {
    let (mut sim, cam, _) = cena();
    let b = cam.to_bits();
    apply_shake(&mut sim, &[(b, SE::Expoente(0))]);
    assert_eq!(
        build_info_camera(&mut sim, b, true, 1).unwrap().expoente,
        ph2d_shake::EXPOENTE_MIN
    );
    apply_shake(&mut sim, &[(b, SE::Expoente(99))]);
    assert_eq!(
        build_info_camera(&mut sim, b, true, 1).unwrap().expoente,
        ph2d_shake::EXPOENTE_MAX
    );
    // ⭐ E um valor DENTRO da faixa passa intacto — senão a cerca leria-se como um valor fixo.
    apply_shake(&mut sim, &[(b, SE::Expoente(2))]);
    assert_eq!(build_info_camera(&mut sim, b, true, 1).unwrap().expoente, 2);
}

/// ⭐ **A lista cresce, encolhe e respeita o TECTO** — e o tecto é o do modelo.
#[test]
fn a_lista_de_fontes_cresce_encolhe_e_para_no_tecto() {
    let (mut sim, _, bomba) = cena();
    let b = bomba.to_bits();
    for _ in 0..(ph2d_ecs::SHAKE_EMITTERS_MAX + 5) {
        apply_emitter(&mut sim, &[(b, EE::Add)]);
    }
    assert_eq!(
        build_info_emitter(&mut sim, b, true, 1).unwrap().rows.len(),
        ph2d_ecs::SHAKE_EMITTERS_MAX
    );
    assert!(
        !apply_emitter(&mut sim, &[(b, EE::Add)]),
        "no tecto o `Add` não pode dizer que mudou alguma coisa"
    );
    apply_emitter(&mut sim, &[(b, EE::Remove(0))]);
    assert_eq!(
        build_info_emitter(&mut sim, b, true, 1).unwrap().rows.len(),
        ph2d_ecs::SHAKE_EMITTERS_MAX - 1
    );
}

/// ⚠️ **Um índice fora da lista é IGNORADO, nunca um `panic`** — o painel pode publicar uma edição
/// sobre a linha que o quadro anterior apagou.
#[test]
fn um_indice_fora_da_lista_nao_estoura() {
    let (mut sim, _, bomba) = cena();
    let b = bomba.to_bits();
    assert!(!apply_emitter(&mut sim, &[(b, EE::Forca(99, 1.0))]));
    assert!(!apply_emitter(&mut sim, &[(b, EE::Remove(99))]));
}

/// ⭐⭐ **A IDA E VOLTA de cada campo de uma fonte**, com a mesma metade dos vizinhos.
#[test]
fn cada_edicao_de_uma_fonte_escreve_o_seu_campo() {
    let (mut sim, _, bomba) = cena();
    let b = bomba.to_bits();
    apply_emitter(&mut sim, &[(b, EE::On(0, "boom".into()))]);
    apply_emitter(&mut sim, &[(b, EE::De(0, 1))]);
    apply_emitter(&mut sim, &[(b, EE::Forca(0, 0.25))]);
    apply_emitter(&mut sim, &[(b, EE::Dentro(0, 1.0))]);
    apply_emitter(&mut sim, &[(b, EE::Fora(0, 9.0))]);
    let r = &build_info_emitter(&mut sim, b, true, 1).unwrap().rows[0];
    assert_eq!(r.on, "boom");
    assert_eq!(r.de, SignalFrom::Myself.tag());
    assert_eq!((r.forca, r.dentro, r.fora), (0.25, 1.0, 9.0));
}

/// ⛔⛔ **A tag da cerca é a POSIÇÃO no `ALL`** — e o gate afirma a volta, que é o que prova que
/// toda cerca que a lei resolve é alcançável pelo painel.
#[test]
fn a_tag_da_cerca_faz_a_volta_inteira() {
    let (mut sim, _, bomba) = cena();
    let b = bomba.to_bits();
    for (i, esperado) in SignalFrom::ALL.iter().enumerate() {
        let t = u8::try_from(i).unwrap();
        apply_emitter(&mut sim, &[(b, EE::De(0, t))]);
        let lido = build_info_emitter(&mut sim, b, true, 1).unwrap().rows[0].de;
        assert_eq!(lido, t, "a tag {t} tem de voltar como {t}");
        assert_eq!(SignalFrom::from_tag(lido), *esperado);
    }
}

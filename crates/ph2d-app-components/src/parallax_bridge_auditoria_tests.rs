//! Os gates da ponte da paralaxe, parte 4: os achados da [auditoria 26](../../../docs/Components/26_auditoria_paralaxe_2026-09-23.md).
//!
//! ⚠️ **Todos passam pelo `settle` do ledger entre quadros** — é a metade que os gates das W1–W7
//! não corriam, e foi por isso que eles ficaram verdes sobre a pose cozida no documento: *um gate
//! que parte de um ledger virgem e nunca o assenta mede o primeiro quadro, e o defeito estava no
//! segundo.*

#![allow(clippy::wildcard_imports)]
use super::*;

/// Um quadro inteiro como a shell o corre: a ponte e, no fim, o `settle` da captura.
fn quadro(
    sim: &mut SimWorld,
    drive: &mut PreviewDrive,
    vista: Option<([f32; 2], [f32; 2])>,
) -> usize {
    let n = drive_parallax(sim, vista, PARADO, drive);
    drive.settle();
    n
}

fn pose(sim: &SimWorld, e: ph2d_ecs::Entity) -> Transform {
    *sim.world().get::<Transform>(e).expect("pose")
}

fn com_camera(sim: &mut SimWorld, dolly: f32) -> ph2d_ecs::Entity {
    sim.world_mut()
        .spawn((
            ph2d_ecs::GameCamera {
                dolly,
                ..ph2d_ecs::GameCamera::default()
            },
            ph2d_ecs::StableId(1),
            Transform::default(),
        ))
        .id()
}

/// ⛔⛔⛔ **Quem deixa de ser conduzido VOLTA à pose autorada, e a próxima condução desloca UMA vez**
/// (auditoria 26, §1.2). As cinco saídas da ponte, cada uma com o mesmo roteiro: conduzir → sair →
/// assentar → a pose é a autorada e o ledger está vazio → voltar a conduzir → o deslocamento
/// simples.
#[test]
fn quem_deixa_de_ser_conduzido_volta_a_pose_autorada() {
    let autorada = pose_em(3.0, -2.0);
    let vista = Some(([400.0, 0.0], SEM_LIMITE));
    let esperado = 3.0 + 400.0 * 0.5;
    type Saida = fn(&mut SimWorld, ph2d_ecs::Entity) -> Option<([f32; 2], [f32; 2])>;
    type Volta = fn(&mut SimWorld, ph2d_ecs::Entity);
    let saidas: [(&str, Saida, Volta); 4] = [
        ("sem camera", |_, _| None, |_, _| {}),
        (
            "k volta ao neutro",
            |sim, e| {
                sim.world_mut()
                    .entity_mut(e)
                    .insert(ScrollFactor { k: [1.0, 1.0] });
                Some(([400.0, 0.0], SEM_LIMITE))
            },
            |sim, e| {
                sim.world_mut()
                    .entity_mut(e)
                    .insert(ScrollFactor { k: [0.5, 0.5] });
            },
        ),
        (
            "o componente sai",
            |sim, e| {
                sim.world_mut().entity_mut(e).remove::<ScrollFactor>();
                Some(([400.0, 0.0], SEM_LIMITE))
            },
            |sim, e| {
                sim.world_mut()
                    .entity_mut(e)
                    .insert(ScrollFactor { k: [0.5, 0.5] });
            },
        ),
        (
            "ganha um canvas",
            |sim, e| {
                sim.world_mut().entity_mut(e).insert(UiCanvas::default());
                Some(([400.0, 0.0], SEM_LIMITE))
            },
            |sim, e| {
                sim.world_mut().entity_mut(e).remove::<UiCanvas>();
            },
        ),
    ];
    for (nome, sair, voltar) in saidas {
        let (mut sim, e) = cena([0.5, 0.5], autorada);
        let mut drive = PreviewDrive::default();
        quadro(&mut sim, &mut drive, vista);
        assert!(
            (pose(&sim, e).translation.x - esperado).abs() < 1e-3,
            "{nome}: o controlo não conduziu"
        );
        let v = sair(&mut sim, e);
        quadro(&mut sim, &mut drive, v);
        assert_eq!(
            pose(&sim, e),
            autorada,
            "{nome}: a pose deslocada ficou no mundo — a captura grava-a como documento"
        );
        assert!(
            drive.is_empty(),
            "{nome}: o ledger ainda conduz um objecto que a ponte largou"
        );
        voltar(&mut sim, e);
        quadro(&mut sim, &mut drive, vista);
        assert!(
            (pose(&sim, e).translation.x - esperado).abs() < 1e-3,
            "{nome}: ao voltar a conduzir o deslocamento não é o simples: x = {} contra {esperado}",
            pose(&sim, e).translation.x
        );
    }
}

/// ⛔⛔ **A câmera a atravessar a camada também a LARGA** — a 5.ª saída, com câmera e dolly reais.
#[test]
fn o_dolly_que_atravessa_a_camada_larga_a() {
    let autorada = pose_em(0.0, 0.0);
    let mut sim = SimWorld::default();
    let cam = com_camera(&mut sim, 0.4);
    let e = sim
        .world_mut()
        .spawn((ScrollFactor { k: [2.0, 2.0] }, autorada))
        .id();
    let mut drive = PreviewDrive::default();
    let vista = Some(([25.0, 0.0], SEM_LIMITE));
    assert_eq!(quadro(&mut sim, &mut drive, vista), 1, "o controlo conduz");
    assert_ne!(pose(&sim, e), autorada, "o controlo desloca");
    // `k = 2` ⇒ `z = z₀/2`; um dolly de `0,6` passa para lá dela.
    sim.world_mut()
        .get_mut::<ph2d_ecs::GameCamera>(cam)
        .expect("camera")
        .dolly = 0.6;
    quadro(&mut sim, &mut drive, vista);
    assert_eq!(
        pose(&sim, e),
        autorada,
        "a camada atravessada ficou deslocada"
    );
    assert!(drive.is_empty());
}

/// ⛔⛔ **O dolly de volta a ZERO devolve a escala autorada** (auditoria 26, §1.3).
#[test]
fn o_dolly_de_volta_a_zero_devolve_a_escala() {
    let autorada = Transform {
        scale: Vec2::new(2.0, 2.0),
        ..pose_em(0.0, 0.0)
    };
    let mut sim = SimWorld::default();
    let cam = com_camera(&mut sim, 0.5);
    let e = sim
        .world_mut()
        .spawn((ScrollFactor { k: [0.25, 0.25] }, autorada))
        .id();
    let mut drive = PreviewDrive::default();
    let vista = Some(([10.0, 0.0], SEM_LIMITE));
    quadro(&mut sim, &mut drive, vista);
    assert!(
        (pose(&sim, e).scale.x - 2.0).abs() > 0.1,
        "o controlo: o dolly escalou"
    );
    sim.world_mut()
        .get_mut::<ph2d_ecs::GameCamera>(cam)
        .expect("camera")
        .dolly = 0.0;
    for _ in 0..3 {
        quadro(&mut sim, &mut drive, vista);
    }
    assert_eq!(
        pose(&sim, e).scale,
        autorada.scale,
        "o dolly voltou a zero e a escala ficou presa"
    );
}

/// ⛔⛔ **O dolly escala à volta do CENTRO DA VISTA, pela ponte** (auditoria 26, §1.4). O céu da
/// cena `=2`, com o pivô longe da origem, que é onde as duas leis discordam.
#[test]
fn o_dolly_leva_o_fundo_para_o_centro_da_vista() {
    let autorada = pose_em(0.0, 3.2);
    let mut sim = SimWorld::default();
    com_camera(&mut sim, 0.9);
    let e = sim
        .world_mut()
        .spawn((ScrollFactor { k: [0.12, 0.12] }, autorada))
        .id();
    let mut drive = PreviewDrive::default();
    quadro(&mut sim, &mut drive, Some(([0.0, 0.0], SEM_LIMITE)));
    let esc = ScrollFactor::escala_do_dolly(0.12, 0.9).expect("dentro");
    let y = pose(&sim, e).translation.y;
    assert!(
        (y - 3.2 * esc).abs() < 1e-4,
        "y = {y}: o pivô ficou onde estava em vez de convergir ({} esperado)",
        3.2 * esc
    );
    // O CONTROLO: a escala é a do dolly — sem ela a posição podia estar certa por acaso.
    assert!((pose(&sim, e).scale.y - esc).abs() < 1e-6);
}

/// ⛔ **Um `k` NEGATIVO também sente o dolly** — a mutação que o ignorava sobrevivia à bancada
/// inteira (auditoria 26, §3), porque todo gate com `k < 0` corria com o dolly a zero.
#[test]
fn um_k_negativo_tambem_sente_o_dolly() {
    let autorada = pose_em(1.0, 0.0);
    let corre = |dolly: f32| {
        let mut sim = SimWorld::default();
        com_camera(&mut sim, dolly);
        let e = sim
            .world_mut()
            .spawn((ScrollFactor { k: [-0.5, -0.5] }, autorada))
            .id();
        let mut drive = PreviewDrive::default();
        quadro(&mut sim, &mut drive, Some(([6.0, 0.0], SEM_LIMITE)));
        pose(&sim, e)
    };
    let (sem, com) = (corre(0.0), corre(0.5));
    let esc = ScrollFactor::escala_do_dolly(-0.5, 0.5).expect("dentro");
    let x = ph2d_ecs::scroll_factor::saida_eixo(-0.5, 1.0, 6.0, 6.0, 0.0, esc, None);
    assert!(
        (com.translation.x - x).abs() < 1e-5,
        "{} contra {x}",
        com.translation.x
    );
    assert!(
        (com.translation.x - sem.translation.x).abs() > 0.1,
        "o dolly foi ignorado com k < 0"
    );
}

/// ⛔⛔ **O autorado NÃO deriva em `f32` enquanto a câmera anda** (auditoria 26, §2.1). A régua é a
/// igualdade AO BIT — a tolerância `1e-4` do gate irmão do arrasto é exactamente onde a deriva se
/// escondia.
#[test]
fn o_autorado_nao_deriva_com_a_camera_a_andar() {
    let autorada = pose_em(0.1, 0.3);
    let (mut sim, e) = cena([0.5, 0.35], autorada);
    let mut drive = PreviewDrive::default();
    let mut c = 0.0_f32;
    for _ in 0..3000 {
        c += 0.1;
        quadro(&mut sim, &mut drive, Some(([c, c * 0.5], SEM_LIMITE)));
    }
    let Some(Driven::ParallaxPose(memo)) = drive.authored(e.to_bits(), Driver::ParallaxPose) else {
        panic!("tinha de continuar a ser conduzido");
    };
    assert_eq!(memo, autorada, "o autorado derivou");
    // O CONTROLO: a vista andou mesmo.
    assert!(pose(&sim, e).translation.x > 100.0);
}

/// ⭐ **A repetição em Y** — todas as fixturas de repetição da bancada eram `[tile, 0]`, e a
/// mutação `floor` só em Y sobrevivia (auditoria 26, §3).
#[test]
fn a_repeticao_em_y_fica_presa_a_vista() {
    let tile = 3.0_f32;
    let mut sim = SimWorld::default();
    let e = sim
        .world_mut()
        .spawn((
            ScrollFactor { k: [1.0, 0.4] },
            ScrollRepeat { tile: [0.0, tile] },
            pose_em(0.0, 0.5),
        ))
        .id();
    let mut drive = PreviewDrive::default();
    let mut c = 0.0_f32;
    let mut viu_corrigir = false;
    while c < 60.0 {
        quadro(&mut sim, &mut drive, Some(([0.0, c], SEM_LIMITE)));
        let rel = pose(&sim, e).translation.y - c - 0.5;
        assert!(
            rel.abs() <= tile / 2.0 + 1e-3,
            "c = {c}: a camada saiu da vista em Y ({rel})"
        );
        // A janela é CENTRADA: os dois lados do intervalo são visitados.
        viu_corrigir |= rel < -tile * 0.25;
        c += 0.7;
    }
    assert!(viu_corrigir, "a varredura não chegou a corrigir");
}

/// ⭐ **A cerca com dolly usa a meia-vista que a camada VÊ** (auditoria 26, §2.3), pela ponte. A
/// régua é a lei de ponta a ponta (a `meia_da_camada` + o `confina_eixo` + a `saida_eixo`), e o
/// CONTROLO é que a meia-vista crua daria outro número — sem ele o gate passaria com a fiação a
/// ignorar o dolly.
#[test]
fn a_cerca_com_dolly_usa_a_meia_vista() {
    let (k, delta, c, h, min, max) = (0.5_f32, 0.6_f32, 2000.0_f32, 200.0_f32, -600.0, 600.0);
    let mut sim = SimWorld::default();
    com_camera(&mut sim, delta);
    let e = sim
        .world_mut()
        .spawn((
            ScrollFactor { k: [k, k] },
            ScrollLimits {
                min: [min, 0.0],
                max: [max, 0.0],
            },
            pose_em(0.0, 0.0),
        ))
        .id();
    let mut drive = PreviewDrive::default();
    quadro(&mut sim, &mut drive, Some(([c, 0.0], [h, 0.0])));
    let esc = ScrollFactor::escala_do_dolly(k, delta).expect("dentro");
    let conf = ph2d_ecs::confina_eixo(c, ph2d_ecs::meia_da_camada(h, k, esc), min, max);
    let esperado = ph2d_ecs::scroll_factor::saida_eixo(k, 0.0, c, conf, 0.0, esc, None);
    let x = pose(&sim, e).translation.x;
    assert!((x - esperado).abs() < 1e-3, "x = {x} contra {esperado}");
    let cru = ph2d_ecs::scroll_factor::saida_eixo(
        k,
        0.0,
        c,
        ph2d_ecs::confina_eixo(c, h, min, max),
        0.0,
        esc,
        None,
    );
    assert!(
        (cru - esperado).abs() > 1.0,
        "o controlo não contém o fenómeno"
    );
}

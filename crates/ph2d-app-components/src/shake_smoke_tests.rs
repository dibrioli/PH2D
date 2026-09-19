//! Os gates da cena do abanão. ⚠️ Eles medem o que a cena **MONTA**, não o que ela imprime — e a
//! régua mais importante é a que nenhuma das irmãs tem: *o abanão chega a ser VISTO?*

use super::*;
use ph2d_ecs::{CameraRuntime, SimWorld, StableId};

fn montada() -> (SimWorld, Montada) {
    let mut sim = SimWorld::new();
    let m = montar(sim.world_mut(), 1);
    (sim, m)
}

/// ⭐⭐⭐ **O PÁTIO existe, e a régua é a POPULAÇÃO** — sobre um chão de cor chapada um abanão é
/// rigorosamente invisível, e a cena ensinaria *«o abanão não funciona»*.
#[test]
fn a_cena_tem_com_que_se_ver_o_abanao() {
    let (mut sim, _) = montada();
    let mundo = sim.world_mut();
    let postes = mundo
        .query::<&Name>()
        .iter(mundo)
        .filter(|n| n.0.starts_with("Post "))
        .count();
    let esperado = usize::try_from(POSTES_X * POSTES_Y).unwrap();
    assert_eq!(postes, esperado, "o pátio é o que torna o abanão visível");
    assert!(
        esperado >= 20,
        "com poucos postes o dono pode parar num canto vazio e não ver nada: {esperado}"
    );
}

/// ⚠️ **E eles têm de COBRIR a vista** — a régua é a do doc: a grelha tem de ser mais larga que o
/// enquadramento de fábrica, senão há sítios onde o abanão fica invisível.
#[test]
fn o_patio_cobre_mais_do_que_a_vista_enquadra() {
    #[allow(clippy::cast_precision_loss)]
    let largura = (POSTES_X - 1) as f32 * POSTE_PASSO;
    #[allow(clippy::cast_precision_loss)]
    let altura = (POSTES_Y - 1) as f32 * POSTE_PASSO;
    let vista_h = ph2d_ecs::GameCamera::default().height_world;
    assert!(
        altura >= vista_h,
        "a grelha tem {altura} m de alto e a vista enquadra {vista_h}"
    );
    assert!(largura > altura, "e o ecrã é mais largo que alto");
    // ⭐ **E o PASSO é menor que a vista** — senão o herói pode parar entre duas fileiras e não ter
    // um único poste na tela, que é o defeito de volta num sítio só.
    assert!(
        POSTE_PASSO < vista_h,
        "com passo {POSTE_PASSO} numa vista de {vista_h} há sítios sem poste nenhum"
    );
}

/// ⭐⭐⭐ **A cena tem as TRÊS peças do abanão, e nenhuma sozinha faz alguma coisa.**
#[test]
fn a_cena_tem_quem_grite_quem_ouca_e_quem_trema() {
    let (mut sim, _) = montada();
    let mundo = sim.world_mut();
    assert_eq!(
        mundo.query::<&SignalOnAction>().iter(mundo).count(),
        1,
        "quem GRITA"
    );
    assert_eq!(
        mundo.query::<&ShakeEmitter>().iter(mundo).count(),
        1,
        "quem OUVE"
    );
    assert_eq!(
        mundo.query::<&CameraShake>().iter(mundo).count(),
        1,
        "quem TREME"
    );
}

/// ⚠️ **O gatilho e a fonte falam o MESMO nome** — duas consts diferentes seriam uma cena que monta
/// bem e não faz nada, e nenhum gate de presença o veria.
#[test]
fn o_nome_do_sinal_casa_dos_dois_lados() {
    let (mut sim, _) = montada();
    let mundo = sim.world_mut();
    let publicado: Vec<String> = mundo
        .query::<&SignalOnAction>()
        .iter(mundo)
        .flat_map(|t| t.0.iter().map(|r| r.signal.clone()))
        .collect();
    let ouvido: Vec<String> = mundo
        .query::<&ShakeEmitter>()
        .iter(mundo)
        .flat_map(|e| e.0.iter().map(|f| f.on.clone()))
        .collect();
    assert_eq!(
        publicado, ouvido,
        "quem grita e quem ouve têm de dizer o mesmo"
    );
    assert!(!publicado[0].is_empty());
}

/// ⭐⭐ **A cerca é `Myself`, e isso é LEI da cena:** com `Anyone` uma segunda bomba ouviria o
/// estrondo da primeira e a distância deixaria de ser a variável.
#[test]
fn a_fonte_ouve_so_o_proprio_estrondo() {
    let (mut sim, _) = montada();
    let mundo = sim.world_mut();
    let de = mundo.query::<&ShakeEmitter>().iter(mundo).next().unwrap().0[0].de;
    assert_eq!(de, SignalFrom::Myself);
}

/// ⭐⭐⭐ **O alcance da fonte tem de COBRIR a vista e ACABAR dentro do pátio** — as duas metades.
///
/// ⚠️ Sem a primeira, um estrondo ao pé não abana nada e o passo (1) mente; sem a segunda, o dono
/// não consegue **andar para fora** do alcance e o passo (2) é inalcançável.
#[test]
fn o_alcance_deixa_a_cena_ensinar_as_duas_coisas() {
    let (mut sim, _) = montada();
    let mundo = sim.world_mut();
    let f = mundo.query::<&ShakeEmitter>().iter(mundo).next().unwrap().0[0].clone();
    assert!(f.fora > f.dentro, "{:?}", (f.dentro, f.fora));
    assert!(
        ph2d_shake::atenuacao(0.0, f.dentro, f.fora) > 0.9,
        "ao pé tem de chegar inteiro"
    );
    #[allow(clippy::cast_precision_loss)]
    let meia_grelha = (POSTES_X / 2) as f32 * POSTE_PASSO;
    assert!(
        f.fora < meia_grelha * 2.0,
        "o dono tem de conseguir ANDAR para fora do alcance sem sair do pátio: \
         fora = {}, meia-grelha = {meia_grelha}",
        f.fora
    );
}

/// ⭐ **A câmera SEGUE o herói** — sem isso andar não afasta a vista, e o passo (2) da cena não
/// ensina nada.
#[test]
fn a_camera_segue_o_heroi_que_o_dono_conduz() {
    let (mut sim, _) = montada();
    let mundo = sim.world_mut();
    let (_, follow) = mundo
        .query::<(&GameCamera, &CameraFollow)>()
        .iter(mundo)
        .next()
        .expect("a câmera tem de seguir alguém");
    let alvo = follow.target.clone();
    let existe = mundo.query::<&Name>().iter(mundo).any(|n| n.0 == alvo);
    assert!(existe, "a câmera segue «{alvo}», que ninguém na cena tem");
    // ⭐ E o alvo é o objecto que o dono CONDUZ.
    let mundo = sim.world_mut();
    let conduzido = mundo
        .query::<(&Name, &ph2d_physics_ecs::TopDownPlayer)>()
        .iter(mundo)
        .next()
        .map(|(n, _)| n.0.clone());
    assert_eq!(conduzido.as_deref(), Some(alvo.as_str()));
}

/// ⚠️ **Quem nasce escolhido tem a secção que o roteiro nomeia** — a lição do #15.
#[test]
fn quem_nasce_escolhido_tem_a_seccao_do_roteiro() {
    let (sim, m) = montada();
    let e = Entity::from_bits(m.escolhido);
    assert!(
        sim.world().get::<ShakeEmitter>(e).is_some(),
        "o passo (3) manda ver SHAKE EMITTER no escolhido"
    );
    assert!(
        sim.world().get::<Sprite>(e).is_some(),
        "e ele tem de ter CORPO, senão o dedo do dono não lhe chega no canvas"
    );
}

/// ⭐⭐⭐ **A CENA INTEIRA, pelo caminho do PRODUTO:** um estrondo tem de abanar, e o mesmo
/// estrondo a `20` m **não**. ⛔ Este é o único gate que percorre a cena montada com a ponte.
#[test]
fn na_cena_montada_o_estrondo_abana_e_a_distancia_conta() {
    use ph2d_runtime::{Signal, SignalOutbox, SignalReader};
    const DT: f32 = 1.0 / 60.0;

    fn treme_com_a_camera_em(x: f32) -> f32 {
        let mut sim = SimWorld::new();
        montar(sim.world_mut(), 1);
        // A câmera vive onde o `follow` a puser; aqui pousa-se o `CameraRuntime` à mão, que é o que
        // a fase da shell faz no primeiro quadro.
        let cam = ph2d_ecs::active_camera_of(sim.world_mut()).unwrap();
        sim.world_mut().entity_mut(cam).insert(CameraRuntime {
            center: [x, 0.0],
            anchor: [x, 0.0],
            last_target: None,
            velocity: [0.0, 0.0],
            settled: true,
        });
        let bomba = {
            let mundo = sim.world_mut();
            mundo
                .query::<(Entity, &ShakeEmitter)>()
                .iter(mundo)
                .next()
                .unwrap()
                .0
        };
        let mut out = SignalOutbox::new();
        let mut r = SignalReader::at(&out);
        out.publish(Signal::from_action(SINAL, bomba.to_bits(), 0));
        crate::shake_bridge::drive_camera_shake(&mut sim, &out, &mut r, DT).trauma
    }

    let perto = treme_com_a_camera_em(0.0);
    let longe = treme_com_a_camera_em(30.0);
    assert!(
        perto > 0.8,
        "ao pé da bomba tem de abanar forte: {perto:.3}"
    );
    assert_eq!(longe, 0.0, "a 30 m o mesmo estrondo não pode chegar");
    // ⭐ E o MEIO existe — senão a cena é um interruptor e não uma lei.
    let meio = treme_com_a_camera_em(11.0);
    assert!(
        meio > 0.0 && meio < perto,
        "a meio caminho tem de abanar MENOS e não nada: {meio:.3}"
    );
}

/// ⚠️ **Os `StableId` existem** — sem eles a `active_camera_of` devolve `None` e a cena monta sem
/// câmera activa, que é um defeito mudo (a cena parece certa e nada treme).
#[test]
fn a_cena_atribui_identidade() {
    let (mut sim, _) = montada();
    let cam = ph2d_ecs::active_camera_of(sim.world_mut());
    assert!(cam.is_some(), "a cena tem de ter uma câmera ACTIVA");
    assert!(sim.world().get::<StableId>(cam.unwrap()).is_some());
}

//! Os gates da PONTE da câmera de jogo — o que ela DECIDE, medido sem janela.
//!
//! ⚠️ **Eles correm o caminho do PRODUTO** ([`super::update`]), e não uma cópia dele: um gate que
//! reimplementasse o passe mediria o teste. A lei em si já é gateada contra o oráculo do Godot em
//! `ph2d-ecs`; o que se prova aqui é a **costura** — quem manda, para onde ela olha, e o que ela
//! devolve à vista.

use ph2d_core::Vec2;
use ph2d_ecs::{
    CameraFollow, CameraLimits, GameCamera, Name, SimWorld, Transform, assign_missing_stable_ids,
};

use super::update;

const ASPECT: f32 = 16.0 / 9.0;
const DT: f64 = 1.0 / 60.0;
/// ⚠️ **O abanão de uma vista que não treme** (suplente #25). Estes gates medem a LEI DA CÂMERA, e
/// pô-lo aqui por nome é o que faz cada um deles dizer, na assinatura, que não está a medir o
/// abanão — *um `[0.0, 0.0]` cru em quinze chamadas leria-se como ruído de argumento*.
const SEM_ABANAO: [f32; 2] = [0.0, 0.0];

fn mundo() -> SimWorld {
    SimWorld::default()
}

/// Uma câmera na posição `p`, com a config dada.
fn camera(sim: &mut SimWorld, nome: &str, p: [f32; 2], cam: GameCamera) -> ph2d_ecs::Entity {
    let e = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(p[0], p[1])),
            Name::new(nome),
            cam,
        ))
        .id();
    assign_missing_stable_ids(sim.world_mut());
    e
}

/// Um objecto qualquer com nome — o que uma câmera segue.
fn objecto(sim: &mut SimWorld, nome: &str, p: [f32; 2]) -> ph2d_ecs::Entity {
    let e = sim
        .world_mut()
        .spawn((
            Transform::from_translation(Vec2::new(p[0], p[1])),
            Name::new(nome),
        ))
        .id();
    assign_missing_stable_ids(sim.world_mut());
    e
}

fn mover(sim: &mut SimWorld, e: ph2d_ecs::Entity, p: [f32; 2]) {
    if let Some(mut t) = sim.world_mut().get_mut::<Transform>(e) {
        t.translation = Vec2::new(p[0], p[1]);
    }
}

/// ⚠️ **Uma cena sem câmera devolve `None`, e isso é o comportamento de HOJE** — o editor fica com
/// o enquadramento dele. ⛔ Devolver uma vista de omissão faria toda cena existente saltar para a
/// origem no primeiro quadro depois desta wave.
///
/// **Mutação que deve sangrar:** trocar o `else` do `active_camera_of` por uma vista de omissão.
#[test]
fn a_scene_with_no_camera_leaves_the_view_to_the_editor() {
    let mut sim = mundo();
    let (vista, r) = update(&mut sim, ASPECT, 1, DT, SEM_ABANAO);
    assert!(
        vista.is_none(),
        "sem camera a vista tem de ficar com o editor"
    );
    assert_eq!(r.cameras, 0);
    assert!(!r.following);
}

/// ⭐ **Uma câmera SEM `follow` é uma câmera FIXA** — a vista é a pose dela, sem componente nenhum
/// a mais. É a metade do desenho que justifica os três componentes separados.
#[test]
fn a_camera_without_follow_frames_its_own_pose() {
    let mut sim = mundo();
    camera(&mut sim, "Cam", [3.0, -2.0], GameCamera::default());
    let (vista, r) = update(&mut sim, ASPECT, 1, DT, SEM_ABANAO).clone();
    let v = vista.expect("uma camera activa tem de dar vista");
    assert_eq!(v.center, [3.0, -2.0]);
    assert_eq!(r.cameras, 1);
    assert!(!r.following, "ela nao tem follow");
}

/// ⭐⭐⭐ **O NASCIMENTO não amortece** — senão a câmera viaja da origem até ao alvo no primeiro
/// segundo de jogo, que é o report que o `Timer` e o som já pagaram.
///
/// **Mutação que deve sangrar:** apagar o ramo do `settled` (a câmera passaria a arrancar em `(0,0)`
/// e a amortecer até `[40, 0]`).
#[test]
fn the_first_frame_settles_instead_of_travelling() {
    let mut sim = mundo();
    objecto(&mut sim, "Heroi", [40.0, 0.0]);
    camera(&mut sim, "Cam", [0.0, 0.0], GameCamera::default());
    let cam_e = ph2d_ecs::active_camera_of(sim.world_mut()).unwrap();
    sim.world_mut().entity_mut(cam_e).insert(CameraFollow {
        target: "Heroi".into(),
        ..CameraFollow::default()
    });
    let (vista, r) = update(&mut sim, ASPECT, 1, DT, SEM_ABANAO);
    assert!(r.target_found, "o alvo chama-se `Heroi` e existe");
    assert_eq!(
        vista.unwrap().center,
        [40.0, 0.0],
        "a camera tem de NASCER no alvo, nunca viajar ate' ele"
    );
}

/// ⭐⭐ **Depois de assentar, ela AMORTECE** — e o número sai da lei, que é a do oráculo.
#[test]
fn once_settled_it_damps_towards_the_target() {
    let mut sim = mundo();
    let heroi = objecto(&mut sim, "Heroi", [0.0, 0.0]);
    camera(&mut sim, "Cam", [0.0, 0.0], GameCamera::default());
    let cam_e = ph2d_ecs::active_camera_of(sim.world_mut()).unwrap();
    sim.world_mut().entity_mut(cam_e).insert(CameraFollow {
        target: "Heroi".into(),
        damping: [5.0, 5.0],
        ..CameraFollow::default()
    });
    // Quadro 1: assenta em (0,0).
    let _ = update(&mut sim, ASPECT, 1, DT, SEM_ABANAO);
    // O herói salta para 300; um tique tem de dar `300 · 5/60 = 25`.
    mover(&mut sim, heroi, [300.0, 0.0]);
    let (vista, _) = update(&mut sim, ASPECT, 1, DT, SEM_ABANAO);
    let c = vista.unwrap().center;
    assert!(
        (c[0] - 25.0).abs() < 1e-3,
        "um tique a speed 5 devia dar 25, deu {c:?}"
    );
}

/// ⭐⭐⭐ **Um alvo que não existe é RELATADO, nunca engolido.**
///
/// ⚠️ *«segue ninguém»* e *«segue alguém parado»* dão exactamente a mesma câmera imóvel, e só uma
/// delas é um defeito da cena. Sem esta coluna o dono lê as duas como *«a câmera não funciona»*.
///
/// **Mutação que deve sangrar:** pôr `target_found = true` incondicionalmente.
#[test]
fn a_target_that_does_not_exist_is_reported_not_swallowed() {
    let mut sim = mundo();
    camera(&mut sim, "Cam", [7.0, 1.0], GameCamera::default());
    let cam_e = ph2d_ecs::active_camera_of(sim.world_mut()).unwrap();
    sim.world_mut().entity_mut(cam_e).insert(CameraFollow {
        target: "NaoExiste".into(),
        ..CameraFollow::default()
    });
    let (vista, r) = update(&mut sim, ASPECT, 1, DT, SEM_ABANAO);
    assert!(r.following, "ela TEM follow");
    assert!(
        !r.target_found,
        "e o alvo NAO foi encontrado — a coluna tem de o dizer"
    );
    assert_eq!(r.target_name, "NaoExiste");
    assert_eq!(
        vista.unwrap().center,
        [7.0, 1.0],
        "sem alvo ela fica na pose dela, e nao na origem"
    );
}

/// ⭐⭐ **A cerca prende a JANELA, não o centro** — e o relatório diz quando ela mordeu.
#[test]
fn the_limits_hold_the_window_and_the_report_says_so() {
    let mut sim = mundo();
    let heroi = objecto(&mut sim, "Heroi", [0.0, 0.0]);
    camera(
        &mut sim,
        "Cam",
        [0.0, 0.0],
        GameCamera {
            height_world: 10.0,
            ..GameCamera::default()
        },
    );
    let cam_e = ph2d_ecs::active_camera_of(sim.world_mut()).unwrap();
    sim.world_mut().entity_mut(cam_e).insert(CameraFollow {
        target: "Heroi".into(),
        damping: [0.0, 0.0], // instantâneo: isola a cerca do amortecimento
        ..CameraFollow::default()
    });
    sim.world_mut().entity_mut(cam_e).insert(CameraLimits {
        min: [-100.0, -100.0],
        max: [100.0, 100.0],
    });
    let _ = update(&mut sim, ASPECT, 1, DT, SEM_ABANAO);
    mover(&mut sim, heroi, [1000.0, 0.0]);
    let (vista, r) = update(&mut sim, ASPECT, 1, DT, SEM_ABANAO);
    // meia-largura = 5 · (16/9) = 8,888…  ⇒  o centro pára em 100 − 8,888… = 91,111…
    let esperado = 100.0 - 5.0 * ASPECT;
    let c = vista.unwrap().center;
    assert!(
        (c[0] - esperado).abs() < 1e-3,
        "a JANELA e' que tem de parar na cerca: esperado {esperado}, deu {c:?}"
    );
    assert!(r.limited, "a cerca mordeu e o relatorio tem de o dizer");
}

/// ⭐ **A prioridade escolhe quem manda, e a desligada não concorre** — pelo caminho do produto.
#[test]
fn the_bridge_uses_the_camera_that_commands() {
    let mut sim = mundo();
    camera(
        &mut sim,
        "Baixa",
        [1.0, 0.0],
        GameCamera {
            priority: 1,
            ..GameCamera::default()
        },
    );
    camera(
        &mut sim,
        "Alta",
        [9.0, 0.0],
        GameCamera {
            priority: 5,
            ..GameCamera::default()
        },
    );
    let (vista, r) = update(&mut sim, ASPECT, 1, DT, SEM_ABANAO);
    assert_eq!(r.cameras, 2);
    assert_eq!(vista.unwrap().center[0], 9.0, "a de maior prioridade manda");
}

/// ⭐⭐⭐ **O `offset` NÃO realimenta o estado vivo.**
///
/// ⚠️ Se ele fosse somado ao centro guardado, a zona morta passaria a medir a distância ao ALVO
/// DESLOCADO e a câmera afastar-se-ia um pouco mais a cada quadro — uma deriva lenta que só se vê
/// depois de minutos, e que nenhum gate de um quadro apanharia.
///
/// **Mutação que deve sangrar:** somar o `offset` a `rt.center` em vez de à vista.
#[test]
fn the_offset_frames_the_view_without_feeding_back() {
    let mut sim = mundo();
    let heroi = objecto(&mut sim, "Heroi", [0.0, 0.0]);
    camera(
        &mut sim,
        "Cam",
        [0.0, 0.0],
        GameCamera {
            offset: [0.0, 3.0],
            ..GameCamera::default()
        },
    );
    let cam_e = ph2d_ecs::active_camera_of(sim.world_mut()).unwrap();
    sim.world_mut().entity_mut(cam_e).insert(CameraFollow {
        target: "Heroi".into(),
        damping: [0.0, 0.0],
        ..CameraFollow::default()
    });
    let mut ultimo = [0.0_f32, 0.0];
    for _ in 0..240 {
        mover(&mut sim, heroi, [0.0, 0.0]);
        let (v, _) = update(&mut sim, ASPECT, 1, DT, SEM_ABANAO);
        ultimo = v.unwrap().center;
    }
    assert!(
        (ultimo[1] - 3.0).abs() < 1e-3,
        "depois de 240 quadros parados a vista tem de continuar a 3 m acima do alvo, e esta' em \
         {ultimo:?} — o offset esta' a realimentar"
    );
}

/// ⚠️ **Um quadro SEM passo fixo ainda arma a câmera** — senão uma câmera anexada pela paleta ficava
/// por nascer até calhar um tique, e o sintoma é *«às vezes a câmera não pega»*.
#[test]
fn a_frame_with_no_fixed_tick_still_gives_the_camera_a_life() {
    let mut sim = mundo();
    camera(&mut sim, "Cam", [4.0, 4.0], GameCamera::default());
    let (vista, _) = update(&mut sim, ASPECT, 0, DT, SEM_ABANAO);
    assert_eq!(
        vista
            .expect("mesmo com ticks=0 a camera activa da' vista")
            .center,
        [4.0, 4.0]
    );
}

/// ⛔⛔⛔ **A ANTECIPAÇÃO NUNCA LIDERA MAIS DO QUE A PRÓPRIA DEFINIÇÃO** — o report de 2026-09-10
/// (*«lookahead parece completamente bugado»*), medido.
///
/// # O defeito, e o número dele
///
/// A ponte vê o alvo **uma vez por quadro**, e um quadro leva `ticks` passos — mas a primeira
/// redacção dividia a diferença pelo **passo** em vez de pelo tempo entre as duas amostras. Numa
/// moldura de dois tiques isso lê exactamente o **DOBRO** da velocidade. Medido, com o herói a
/// `8 m/s` e antecipação de `0,5 s`:
///
/// ```text
///   quadro  tiques   herói      mira        (a mira devia ser herói + 4,00)
///        3       1   0,4000    4,4000  ✓
///        4       2   0,6667    8,6667  ⛔ +8,00 — o DOBRO
///        5       1   0,8000    4,8000  ✓  (e volta)
/// ```
///
/// `4 m` de ida e volta numa vista de `17,8 m` de largura são **22 % do ecrã**, várias vezes por
/// segundo. *Nenhum outro gate desta linha o via: todos passam `ticks = 1`.*
///
/// # A régua é a DEFINIÇÃO da antecipação, não um número escolhido
///
/// Ela diz *«olha onde o alvo estará daqui a `L` segundos»* ⇒ a dianteira não pode passar de
/// `v · L`. ⛔ Uma barra afinada à mão mediria o nosso próprio defeito.
///
/// **Mutação que deve sangrar:** trocar `sample_dt` por `dt` na ponte.
#[test]
fn the_lookahead_never_leads_by_more_than_its_own_definition() {
    let mut sim = mundo();
    let heroi = objecto(&mut sim, "Heroi", [0.0, 0.0]);
    camera(&mut sim, "Cam", [0.0, 0.0], GameCamera::default());
    let cam_e = ph2d_ecs::active_camera_of(sim.world_mut()).unwrap();
    const L: f32 = 0.5;
    const VEL: f32 = 8.0;
    sim.world_mut().entity_mut(cam_e).insert(CameraFollow {
        target: "Heroi".into(),
        damping: [5.0, 5.0],
        dead_zone: [0.0, 0.0],
        lookahead: [L, 0.0],
        offset: [0.0, 0.0],
    });
    // ⚠️ **O padrão que um vsync de ~60 Hz de facto entrega** — e é ele que produz o fenómeno.
    // Um padrão de `1` puro deixa este gate VERDE sobre o defeito, que é o que aconteceu.
    let padrao = [
        1u32, 1, 0, 1, 2, 1, 1, 0, 2, 1, 1, 1, 2, 1, 0, 1, 1, 2, 1, 1,
    ];
    let mut x = 0.0_f32;
    let mut pior = 0.0_f32;
    for &t in &padrao {
        x += VEL * t as f32 * DT as f32;
        mover(&mut sim, heroi, [x, 0.0]);
        let _ = update(&mut sim, ASPECT, t, DT, SEM_ABANAO);
        let ancora = sim
            .world()
            .get::<ph2d_ecs::CameraRuntime>(cam_e)
            .unwrap()
            .anchor[0];
        pior = pior.max(ancora - x);
    }
    assert!(
        pior <= VEL * L + 1e-3,
        "a mira liderou {pior:.4} m, e a antecipacao de {L} s a {VEL} m/s vale {} m — acima disso \
         ela deixou de ser uma antecipacao e passou a ser um salto",
        VEL * L
    );
}

/// ⛔⛔ **PARAR não colapsa a mira num quadro** — a segunda metade do mesmo report.
///
/// # O defeito, e o número dele
///
/// Uma velocidade tirada de duas amostras vai a **zero** no instante em que o alvo pára, e a mira
/// cai de `v · L` metros de uma vez. Medido: `+5,6000 → +1,6000`, **quatro metros num quadro** — a
/// câmera dava um recuo que ninguém pediu. *Antecipar sem suavizar é trocar um atraso por um
/// solavanco.*
///
/// # A régua
///
/// A dianteira tem de decair pelo **horizonte da própria antecipação**, e não de repente. A barra é
/// o passo que a lei dá num tique (`k = min(dt/L, 1)` da [`ph2d_ecs::smooth_velocity`]), com folga:
/// ⛔ um número escolhido mediria o nosso defeito.
///
/// **Mutação que deve sangrar:** trocar o [`ph2d_ecs::smooth_velocity`] pela velocidade CRUA.
#[test]
fn stopping_does_not_collapse_the_aim_in_one_frame() {
    let mut sim = mundo();
    let heroi = objecto(&mut sim, "Heroi", [0.0, 0.0]);
    camera(&mut sim, "Cam", [0.0, 0.0], GameCamera::default());
    let cam_e = ph2d_ecs::active_camera_of(sim.world_mut()).unwrap();
    const L: f32 = 0.5;
    const VEL: f32 = 8.0;
    sim.world_mut().entity_mut(cam_e).insert(CameraFollow {
        target: "Heroi".into(),
        damping: [5.0, 5.0],
        dead_zone: [0.0, 0.0],
        lookahead: [L, 0.0],
        offset: [0.0, 0.0],
    });
    let mut x = 0.0_f32;
    // Anda o suficiente para a antecipação estar montada.
    for _ in 0..120 {
        x += VEL * DT as f32;
        mover(&mut sim, heroi, [x, 0.0]);
        let _ = update(&mut sim, ASPECT, 1, DT, SEM_ABANAO);
    }
    let lead = |sim: &SimWorld| {
        sim.world()
            .get::<ph2d_ecs::CameraRuntime>(cam_e)
            .unwrap()
            .anchor[0]
            - x
    };
    let montada = lead(&sim);
    assert!(
        montada > 1.0,
        "a fixtura NAO produz o fenomeno: a antecipacao so' esta' em {montada:.4} m depois de 2 s"
    );
    // E agora PÁRA. ⚠️ O passo por tique da lei é `dt/L`; a folga é `3×` isso.
    let passo_da_lei = (DT as f32) / L;
    let mut anterior = montada;
    for n in 0..12 {
        let _ = update(&mut sim, ASPECT, 1, DT, SEM_ABANAO);
        let agora = lead(&sim);
        let queda = (anterior - agora) / montada;
        assert!(
            queda <= passo_da_lei * 3.0,
            "quadro {n}: a mira caiu {:.1} % da dianteira num quadro so' (a lei da' {:.1} % por \
             tique) — e' o solavanco que a suavizacao existe para nao ter",
            queda * 100.0,
            passo_da_lei * 100.0
        );
        anterior = agora;
    }
    assert!(
        anterior < montada,
        "e ela tem de DECAIR: ficou em {anterior:.4} contra {montada:.4}"
    );
}

/// ⭐⭐⭐ **EM REGIME, a dianteira é EXACTAMENTE `v · L`** — o gate que os outros dois não podiam ser.
///
/// # Porque ele existe, e o que ele apanhou
///
/// Os dois irmãos acima medem **transientes** (o pico numa moldura de dois tiques, o colapso ao
/// parar). Este mede o **valor**, e foi ele que expôs o defeito MAIOR do report de 2026-09-10:
///
/// ```text
///   antes das curas   velocidade suavizada = 0,7822 m/s   sobre um herói a 8,0000  ⛔ 10× fraca
///   depois            velocidade suavizada = 8,0000 m/s                            ✓
/// ```
///
/// ⛔ A causa não era a lei: num quadro **sem tique** o `sample_dt` é zero, a amostra não tem de
/// onde tirar velocidade e o `damp_axis` — cujo braço de `dt <= 0` significa *instantâneo* —
/// adoptava esse zero. **Cada quadro perdido zerava a velocidade**, e com dois em dez ela nunca
/// subia.
///
/// # ⚠️⚠️ E esse defeito MASCARAVA o do `sample_dt`
///
/// Enquanto nada convergia, o viés do estimador errado era invisível: a mutação `sample_dt → dt`
/// **sobreviveu** a este ficheiro inteiro. Curado o terceiro, ela passa a ler `11,3373 m/s` sobre
/// `8,0000` — **+42 %** — e sangra aqui. *Um segundo erro pode ser load-bearing para o primeiro, e
/// o sinal é a cura não melhorar com o knob que devia curá-la.*
///
/// **Mutações que devem sangrar:** `sample_dt → dt` · tirar o `if ticks > 0` da velocidade.
#[test]
fn in_steady_state_the_lead_is_exactly_velocity_times_lookahead() {
    let mut sim = mundo();
    let heroi = objecto(&mut sim, "Heroi", [0.0, 0.0]);
    camera(&mut sim, "Cam", [0.0, 0.0], GameCamera::default());
    let cam_e = ph2d_ecs::active_camera_of(sim.world_mut()).unwrap();
    const L: f32 = 0.5;
    const VEL: f32 = 8.0;
    sim.world_mut().entity_mut(cam_e).insert(CameraFollow {
        target: "Heroi".into(),
        damping: [5.0, 5.0],
        dead_zone: [0.0, 0.0],
        lookahead: [L, 0.0],
        offset: [0.0, 0.0],
    });
    // ⚠️ **O padrão TEM de ter quadros de 0 e de 2 tiques**: é isso que produz o fenómeno. Um
    // padrão de `1` puro deixa este gate verde sobre os três defeitos.
    let padrao = [1u32, 1, 0, 1, 2, 1, 1, 0, 2, 1];
    assert!(
        padrao.contains(&0) && padrao.contains(&2),
        "a fixtura nao produz o fenomeno: sem quadros de 0 e de 2 tiques os tres defeitos sao \
         invisiveis"
    );
    let mut x = 0.0_f32;
    for i in 0..400 {
        let t = padrao[i % padrao.len()];
        x += VEL * t as f32 * DT as f32;
        mover(&mut sim, heroi, [x, 0.0]);
        let _ = update(&mut sim, ASPECT, t, DT, SEM_ABANAO);
    }
    let rt = sim.world().get::<ph2d_ecs::CameraRuntime>(cam_e).unwrap();
    let v = rt.velocity[0];
    let dianteira = rt.anchor[0] - x;
    // ⚠️ **A barra é `1 %` porque o estimador é NÃO-ENVIESADO**, e não porque `1 %` seja bonito: em
    // regime a média exponencial de uma velocidade constante converge para ela. ⛔ Uma barra larga
    // aceitaria de volta os `+42 %` do `dt` errado.
    assert!(
        (v - VEL).abs() < VEL * 0.01,
        "a velocidade suavizada leu {v:.4} m/s sobre um alvo a {VEL:.4} — em regime ela tem de \
         convergir para a verdade, e um desvio aqui e' viés do estimador, nunca ruído"
    );
    assert!(
        (dianteira - VEL * L).abs() < VEL * L * 0.01,
        "a dianteira leu {dianteira:.4} m e a antecipacao de {L} s a {VEL} m/s vale {:.4}",
        VEL * L
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// ⭐⭐⭐ O ABANÃO DA VISTA (suplente #25) — ele CHEGA ao consumidor?
//
// ⚠️ Esta é a pergunta que o `CLAUDE.md` §5.0 nomeia sobre si mesmo: *nenhum instrumento do repo
// pergunta se o VALOR chega a um consumidor*. Os gates da ponte medem o trauma e o offset; estes
// medem o único sítio onde isso vira PIXEL.
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// ⭐⭐⭐ **O offset do abanão CHEGA à vista** — a mesma cena, duas chamadas, e a única diferença é
/// o argumento. ⛔ Sem este gate, apagar a soma no `CameraView` deixaria a suíte inteira verde: os
/// doze gates da ponte entram pelo canal de DENTRO dela, que fica abaixo desta costura.
#[test]
fn o_abanao_chega_ao_centro_da_vista() {
    let mut sim = mundo();
    camera(&mut sim, "Cam", [7.0, -3.0], GameCamera::default());
    let parado = update(&mut sim, ASPECT, 1, DT, SEM_ABANAO)
        .0
        .unwrap()
        .center;
    let tremido = update(&mut sim, ASPECT, 1, DT, [0.25, -0.5])
        .0
        .unwrap()
        .center;
    assert_eq!(parado, [7.0, -3.0]);
    assert_eq!(
        tremido,
        [7.25, -3.5],
        "o offset tem de SOMAR ao centro, e exactamente"
    );
}

/// ⭐⭐ **E ele SOMA ao `offset` autorado da câmera, sem o substituir** — os dois são deslocamentos
/// da vista e nenhum é dono do campo.
#[test]
fn o_abanao_soma_ao_offset_autorado_em_vez_de_o_substituir() {
    let mut sim = mundo();
    camera(
        &mut sim,
        "Cam",
        [0.0, 0.0],
        GameCamera {
            offset: [2.0, 0.0],
            ..GameCamera::default()
        },
    );
    let c = update(&mut sim, ASPECT, 1, DT, [0.5, 0.0])
        .0
        .unwrap()
        .center;
    assert_eq!(c, [2.5, 0.0]);
}

/// ⭐⭐⭐ **O abanão entra DEPOIS dos limites, e é uma decisão DECLARADA** (o corpo do `update`
/// escreve-a): antes deles a cerca COMERIA o abanão exactamente na borda do nível, e o artista leria
/// *«o abanão parou de funcionar aqui»*.
///
/// ⚠️ **A régua tem de conter o fenómeno:** a câmera é levada CONTRA a cerca primeiro (o relatório
/// tem de dizer `limited`), senão o gate mede uma câmera livre e passa por vácuo.
#[test]
fn na_borda_do_nivel_a_vista_ainda_treme() {
    let mut sim = mundo();
    let heroi = objecto(&mut sim, "Heroi", [0.0, 0.0]);
    camera(
        &mut sim,
        "Cam",
        [0.0, 0.0],
        GameCamera {
            height_world: 10.0,
            ..GameCamera::default()
        },
    );
    let cam_e = ph2d_ecs::active_camera_of(sim.world_mut()).unwrap();
    sim.world_mut().entity_mut(cam_e).insert(CameraFollow {
        target: "Heroi".into(),
        damping: [0.0, 0.0],
        ..CameraFollow::default()
    });
    sim.world_mut().entity_mut(cam_e).insert(CameraLimits {
        min: [-100.0, -100.0],
        max: [100.0, 100.0],
    });
    let _ = update(&mut sim, ASPECT, 1, DT, SEM_ABANAO);
    mover(&mut sim, heroi, [1000.0, 0.0]);

    let (parada, r) = update(&mut sim, ASPECT, 1, DT, SEM_ABANAO);
    assert!(
        r.limited,
        "o arranjo tem de CONTER o fenomeno: a cerca tem de morder"
    );
    let na_cerca = parada.unwrap().center[0];

    let tremida = update(&mut sim, ASPECT, 1, DT, [0.75, 0.0])
        .0
        .unwrap()
        .center[0];
    assert!(
        (tremida - (na_cerca + 0.75)).abs() < 1e-4,
        "presa na cerca a vista ainda tem de tremer: {na_cerca} -> {tremida}"
    );
}

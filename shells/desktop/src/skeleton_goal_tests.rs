//! Os gates da **ÂNCORA** — a restrição de cinemática inversa que fica.
//!
//! A LEI (o alcance, a mistura, a suavidade) é do módulo (`ph2d-skeleton`) e está gateada lá. Aqui
//! mede-se o que só existe com um mundo ECS: a âncora nasce sem mover nada · ela persiste ao longo
//! dos quadros · a mistura desliga-a · o laço é recusado · o alvo apagado deixa a corrente em paz ·
//! e o que ela escreve é **pré-visualização**, não documento.

use super::*;
use ph2d_ecs::{ChildOf, Name, RootOrder};

/// Um braço de dois ossos deitado no `+X`, com a raiz na origem: ombro `(0,0)→(10,0)`, cotovelo
/// `(10,0)→(20,0)`. Devolve `(sim, [ombro, cotovelo])`.
fn braco() -> (SimWorld, [Entity; 2]) {
    let mut sim = SimWorld::default();
    let ombro = osso(&mut sim, "Shoulder", [0.0, 0.0], 10.0, None);
    let cotovelo = osso(&mut sim, "Elbow", [10.0, 0.0], 10.0, Some(ombro));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    (sim, [ombro, cotovelo])
}

fn osso(sim: &mut SimWorld, nome: &str, pos: [f32; 2], len: f64, pai: Option<Entity>) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            Transform {
                translation: ph2d_core::Vec2::new(pos[0], pos[1]),
                ..Transform::IDENTITY
            },
            Name::new(nome),
            RootOrder(0),
            Bone {
                length: len,
                strength: 1.0,
            },
        ))
        .id();
    if let Some(p) = pai {
        sim.world_mut().entity_mut(e).insert(ChildOf(p));
    }
    e
}

/// Onde a ponta da corrente está, em mundo.
fn ponta(sim: &SimWorld, e: Entity) -> [f64; 2] {
    crate::bone_gesture::tip_of(sim, e.to_bits()).expect("o osso existe")
}

/// Põe a âncora em `p` e resolve um quadro.
fn quadro(sim: &mut SimWorld, tip: Entity, p: [f64; 2]) -> usize {
    assert!(drag_anchor(sim, tip, p), "a ancora tem de existir");
    let mut pv = PreviewDrive::default();
    solve(sim, &mut pv)
}

/// ⭐⭐⭐ **CRIAR A ÂNCORA NÃO MOVE UM PIXEL** — a lei da casa (*todo motor novo é no-op no ponto
/// neutro*).
///
/// Se carregar em *Add IK* deslocasse o braço, o artista perderia a pose que acabou de fazer, e a
/// feature seria uma armadilha em vez de uma ferramenta.
#[test]
fn adding_an_anchor_moves_nothing() {
    let (mut sim, [_, cotovelo]) = braco();
    let antes = crate::skeleton_live::bone_segments(&sim);
    assert!(add(&mut sim, cotovelo).is_some(), "a ancora nasce");
    let mut pv = PreviewDrive::default();
    solve(&mut sim, &mut pv);
    let depois = crate::skeleton_live::bone_segments(&sim);
    assert_eq!(antes, depois, "criar a ancora moveu o esqueleto");
    assert!(
        pv.is_empty(),
        "nada foi conduzido - a corrente nao se mexeu"
    );
}

/// ⭐⭐⭐ **A ÂNCORA PERSISTE** — a razão de ela existir. O arrasto da ponta acaba no `Up`; isto
/// continua a valer quadro após quadro, e é o que a timeline vai animar.
#[test]
fn the_chain_keeps_following_the_anchor_frame_after_frame() {
    let (mut sim, [_, cotovelo]) = braco();
    add(&mut sim, cotovelo).expect("a ancora");
    // Um alvo acima e à esquerda, bem dentro do alcance (20 unidades).
    let alvo = [4.0, 9.0];
    quadro(&mut sim, cotovelo, alvo);
    let p1 = ponta(&sim, cotovelo);
    let erro1 = (p1[0] - alvo[0]).hypot(p1[1] - alvo[1]);
    // ⚠️ **A barra é DERIVADA do recurso, não escolhida** (§0.0). A lei do alcance é exacta em
    // `f64`, mas quem guarda a pose é o `Transform` da casa, cuja rotação é **`f32`** — e um ângulo
    // arredondado a `f32` desloca a ponta em `alcance × ε`. Com dois ossos de 10 isso é
    // `20 × 1,19e-7 = 2,4e-6`; **medido: `1,19e-6`**, metade do orçamento, porque os dois erros de
    // arredondamento não se somam no pior caso.
    let orcamento = 20.0 * f64::from(f32::EPSILON);
    assert!(
        erro1 < orcamento,
        "a ponta ficou a {erro1} do alvo no 1.o quadro (orcamento do f32: {orcamento})"
    );
    // ⭐⭐ **A DERIVA É LIMITADA, e não zero** — e a diferença entre as duas afirmações é o que este
    // gate mede. Um quadro escreve a rotação em **`f32`**; o quadro seguinte lê a pose arredondada e
    // resolve de novo, então o segundo osso ajusta-se um bit para compensar o primeiro. É uma
    // cascata que **converge**, não um rastejo.
    //
    // ⚠️ *Convergir e ficar parado são coisas diferentes*, e uma barra em `0` só diria que a
    // primeira medição calhou. A régua é a razão: o que se anda em duzentos quadros tem de ser
    // MENOR do que o que se andou nos cinco primeiros.
    let mut pv = PreviewDrive::default();
    for _ in 0..5 {
        solve(&mut sim, &mut pv);
    }
    let p5 = ponta(&sim, cotovelo);
    let cedo = (p5[0] - p1[0]).hypot(p5[1] - p1[1]);
    for _ in 0..200 {
        solve(&mut sim, &mut pv);
    }
    let p205 = ponta(&sim, cotovelo);
    let tarde = (p205[0] - p5[0]).hypot(p205[1] - p5[1]);
    eprintln!("[probe] deriva: 5 quadros {cedo:.3e} · 200 quadros seguintes {tarde:.3e}");
    assert!(
        tarde <= cedo,
        "a corrente andou {tarde} em 200 quadros contra {cedo} nos 5 primeiros - isto e' RASTEJO, \
         nao convergencia, e ao fim de um minuto de reproducao o braco esta' noutro sitio"
    );
    // E o total fica dentro do orçamento do `f32`, que é o recurso.
    let total = (p205[0] - alvo[0]).hypot(p205[1] - alvo[1]);
    assert!(
        total < orcamento,
        "depois de 205 quadros a ponta esta' a {total} do alvo (orcamento do f32: {orcamento})"
    );
}

/// ⭐⭐⭐ **MIX = 0 É O NO-OP** — a restrição desligada devolve a pose que o artista autorou.
///
/// ⚠️ E o gate mede a POSE, não o número: um `mix` que chegasse ao `blend_angle` mas cujo resultado
/// não fosse escrito daria o mesmo verde numa asserção sobre o campo.
#[test]
fn a_mix_of_zero_leaves_the_authored_pose_alone() {
    let (mut sim, [_, cotovelo]) = braco();
    add(&mut sim, cotovelo).expect("a ancora");
    let antes = crate::skeleton_live::bone_segments(&sim);
    sim.world_mut()
        .get_mut::<IkGoal>(cotovelo)
        .expect("a ancora")
        .mix = 0.0;
    quadro(&mut sim, cotovelo, [4.0, 9.0]);
    assert_eq!(
        antes,
        crate::skeleton_live::bone_segments(&sim),
        "com mix=0 a corrente mexeu-se - a restricao desligada tem de ser inerte"
    );
}

/// **Meia mistura fica a MEIO CAMINHO** — e é o que faz o número valer a pena animar.
///
/// ⚠️ A régua é o ÂNGULO do ombro, não a posição da ponta: a mistura é sobre ângulos, e medir a
/// ponta mediria a composição de duas misturas.
#[test]
fn half_a_mix_lands_between_the_two_poses() {
    let alvo = [4.0, 9.0];
    let angulo = |mix: f64| -> f64 {
        let (mut sim, [ombro, cotovelo]) = braco();
        add(&mut sim, cotovelo).expect("a ancora");
        sim.world_mut()
            .get_mut::<IkGoal>(cotovelo)
            .expect("a ancora")
            .mix = mix;
        quadro(&mut sim, cotovelo, alvo);
        f64::from(sim.world().get::<Transform>(ombro).expect("t").rotation)
    };
    let (zero, meio, cheio) = (angulo(0.0), angulo(0.5), angulo(1.0));
    let esperado = ph2d_skeleton::blend_angle(zero, cheio, 0.5);
    assert!(
        (meio - esperado).abs() < 1e-6,
        "meia mistura deu {meio:.6} e a lei diz {esperado:.6} (extremos {zero:.6} e {cheio:.6})"
    );
    assert!(
        (meio - zero).abs() > 1e-3 && (meio - cheio).abs() > 1e-3,
        "a fixtura nao produz o fenomeno: os dois extremos coincidem"
    );
}

/// ⛔⛔ **O LAÇO É RECUSADO** — um alvo pendurado na própria corrente realimenta-se: mover o osso
/// move o alvo, que move o osso.
///
/// ⚠️ **O gate mede a RECUSA e não a estabilidade**: com o alvo dentro da corrente a resposta certa
/// não é «convergir», é «não fazer nada» — não há resposta certa para um laço, e resolvê-lo
/// devolveria uma pose que depende do número de quadros que passaram.
#[test]
fn a_target_inside_the_chain_is_refused_instead_of_solved() {
    let (mut sim, [ombro, cotovelo]) = braco();
    let alvo = add(&mut sim, cotovelo).expect("a ancora");
    // O artista pendura a âncora no ombro — que É governado por ela.
    sim.world_mut().entity_mut(alvo).insert(ChildOf(ombro));
    let antes = crate::skeleton_live::bone_segments(&sim);
    let mut pv = PreviewDrive::default();
    assert_eq!(
        solve(&mut sim, &mut pv),
        0,
        "o passe resolveu um laco em vez de o recusar"
    );
    assert_eq!(
        antes,
        crate::skeleton_live::bone_segments(&sim),
        "o laco moveu a corrente"
    );
}

/// **O ALVO APAGADO deixa a corrente em paz** — a mesma leitura do osso apagado numa pele: uma
/// restrição que perdeu o sujeito não pode desfazer a pose.
#[test]
fn deleting_the_target_leaves_the_chain_where_it_was() {
    let (mut sim, [_, cotovelo]) = braco();
    let alvo = add(&mut sim, cotovelo).expect("a ancora");
    quadro(&mut sim, cotovelo, [4.0, 9.0]);
    let dobrado = crate::skeleton_live::bone_segments(&sim);
    sim.world_mut().despawn(alvo);
    let mut pv = PreviewDrive::default();
    assert_eq!(
        solve(&mut sim, &mut pv),
        0,
        "sem alvo nao ha' o que resolver"
    );
    assert_eq!(
        dobrado,
        crate::skeleton_live::bone_segments(&sim),
        "apagar o alvo endireitou o braco"
    );
}

/// ⭐⭐ **O QUE A RESTRIÇÃO ESCREVE É PRÉ-VISUALIZAÇÃO** — o documento é a pose da ÂNCORA.
///
/// ⚠️ Sem isto, cada clique com a âncora fora do sítio empilharia um passo de undo cujo conteúdo é
/// *«o solver mexeu»* — vinte cliques, vinte Ctrl+Z mudos, que é o defeito que a auditoria da §11
/// mediu e que o `preview_drive` existe para curar.
#[test]
fn the_pose_the_constraint_writes_is_preview_not_document() {
    let (mut sim, [ombro, cotovelo]) = braco();
    add(&mut sim, cotovelo).expect("a ancora");
    let autorada = *sim.world().get::<Transform>(ombro).expect("t");
    let mut pv = PreviewDrive::default();
    drag_anchor(&mut sim, cotovelo, [4.0, 9.0]);
    solve(&mut sim, &mut pv);
    assert!(
        sim.world().get::<Transform>(ombro).expect("t").rotation != autorada.rotation,
        "a fixtura nao produz o fenomeno: a restricao nao mexeu no ombro"
    );
    // A fotografia repõe o autorado…
    let vivo = pv.substitute_authored(&mut sim);
    assert_eq!(
        sim.world().get::<Transform>(ombro).expect("t").rotation,
        autorada.rotation,
        "a captura viu a pose CONDUZIDA - ela iria para o undo e para o save"
    );
    // …e devolve o vivo logo a seguir, senão o artista veria o braco saltar para tras.
    crate::preview_drive::PreviewDrive::restore_live(&mut sim, &vivo);
    assert!(
        sim.world().get::<Transform>(ombro).expect("t").rotation != autorada.rotation,
        "o vivo nao voltou"
    );
}

/// **A CORRENTE tem o comprimento que a âncora pede** — o *Chain Length* do Blender, e a razão de
/// o valor de nascimento ser `2` e não `0`.
#[test]
fn the_chain_length_decides_how_many_bones_bend() {
    let (mut sim, [ombro, cotovelo]) = braco();
    let punho = osso(&mut sim, "Wrist", [10.0, 0.0], 10.0, Some(cotovelo));
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    assert_eq!(governed(&sim, punho, 0).len(), 3, "0 = ate' a raiz");
    assert_eq!(
        governed(&sim, punho, 2),
        vec![cotovelo, punho],
        "os DOIS de baixo"
    );
    assert_eq!(governed(&sim, punho, 1), vec![punho], "so' a ponta");
    assert_eq!(
        governed(&sim, punho, 99).len(),
        3,
        "um numero absurdo e' aparado pela ARVORE, nao por uma constante"
    );
    let _ = ombro;
}

/// ⭐⭐ **O LOSANGO SUBSTITUI O ANEL, e não se soma a ele** — um ponto, um verbo.
///
/// ⚠️ Duas alças desenhadas por cima uma da outra prometeriam dois verbos onde há um, e o dedo
/// escolheria pela ordem em que as perguntas correm — que é a forma de defeito que este módulo já
/// pagou com o anel do objecto vazio por cima da bolinha da junta.
#[test]
fn an_anchored_end_stops_showing_the_plain_tip_ring() {
    let (mut sim, [_, cotovelo]) = braco();
    assert_eq!(
        unanchored_ends(&sim),
        vec![cotovelo.to_bits()],
        "sem ancora, a ponta mostra o anel"
    );
    add(&mut sim, cotovelo).expect("a ancora");
    assert!(
        unanchored_ends(&sim).is_empty(),
        "a ponta ancorada continua a mostrar o anel - dois verbos no mesmo ponto"
    );
    assert_eq!(anchors(&sim).len(), 1, "e o losango passa a existir");
}

/// ⛔⛔ **O ALVO NÃO GANHA O ANEL DE OBJECTO VAZIO** — ele tem alça própria.
///
/// ⚠️ Um alvo é um objecto com `Transform` e sem pixels, então ele responde *sim* à pergunta do
/// objecto vazio **por construção**. Sem a marca, ele nasceria com um segundo anel concêntrico com
/// o losango — e o disco desse anel **disputaria o clique** com a alça que arrasta a corrente
/// inteira. É o report de 06/09 (*«alguns bones têm círculos grandes e pequenos»*) outra vez, e o
/// doc daquela cura já tinha escrito a lei.
#[test]
fn the_anchor_object_does_not_get_a_second_ring() {
    let (mut sim, [_, cotovelo]) = braco();
    let alvo = add(&mut sim, cotovelo).expect("a ancora");
    assert!(
        !crate::group_gizmo_view::is_empty_object(&sim, alvo),
        "o alvo respondeu 'sou um objecto vazio' - ele ganha um 2.o anel E um disco a disputar o \
         clique com a propria alca"
    );
    // A metade que prova que a fixtura produz o fenómeno: sem a marca, ele responderia sim.
    sim.world_mut()
        .entity_mut(alvo)
        .remove::<ph2d_skeleton_ecs::IkTarget>();
    assert!(
        crate::group_gizmo_view::is_empty_object(&sim, alvo),
        "a fixtura nao produz o fenomeno: sem a marca o alvo ja' nao era um objecto vazio, entao o \
         gate acima estava verde por outro motivo"
    );
}

/// ⭐⭐⭐ **A ÂNCORA É AGARRÁVEL LONGE DE TODO OSSO** — a razão de o dedo a procurar ANTES do osso.
///
/// ⚠️ O `hit` só devolve um osso quando o ponteiro está a `BONE_HIT_PX` do **segmento**. Uma âncora
/// arrastada para fora de alcance fica longe de tudo, e testá-la depois do osso tornaria a alça
/// inalcançável exactamente no estado em que ela mais se distingue da ponta.
#[test]
fn the_anchor_is_grabbable_far_away_from_every_bone() {
    let (mut sim, [_, cotovelo]) = braco();
    add(&mut sim, cotovelo).expect("a ancora");
    // Bem para fora do alcance (a corrente mede 20) e longe de todo segmento.
    let longe = [80.0, 60.0];
    drag_anchor(&mut sim, cotovelo, longe);
    let mut pv = PreviewDrive::default();
    solve(&mut sim, &mut pv);
    assert!(
        crate::bone_gesture::hit(&sim, longe, 1.0).is_none(),
        "a fixtura nao produz o fenomeno: ha' um osso debaixo do ponteiro"
    );
    let h = crate::bone_gesture::hover(&sim, longe, 1.0, None).expect("a ancora tem de ser achada");
    assert_eq!(h.bone, cotovelo.to_bits());
    assert_eq!(h.part, ph2d_skeleton_render::BonePart::Tip);
}

/// ⭐⭐ **FORA DE ALCANCE, A CORRENTE ESTICA e NÃO se rasga** — e a âncora fica onde o artista a pôs,
/// que é o único estado em que o tracejado tem o que dizer.
#[test]
fn out_of_reach_the_chain_straightens_and_the_anchor_stays_put() {
    let (mut sim, [ombro, cotovelo]) = braco();
    add(&mut sim, cotovelo).expect("a ancora");
    let longe = [0.0, 100.0];
    quadro(&mut sim, cotovelo, longe);
    let p = ponta(&sim, cotovelo);
    // A ponta fica no alcance máximo (20) na direcção do alvo, não em cima dele.
    let dist = p[0].hypot(p[1]);
    assert!(
        (dist - 20.0).abs() < 1e-4,
        "a corrente mede {dist} e o alcance e' 20 - ela esticou ou encolheu"
    );
    // E os dois ossos ficam ALINHADOS: uma corrente esticada não tem barriga.
    let (a1, b1) = crate::bone_gesture::test_segment(&sim, ombro.to_bits());
    let cruz = (b1[0] - a1[0]) * (p[1] - b1[1]) - (b1[1] - a1[1]) * (p[0] - b1[0]);
    assert!(
        cruz.abs() < 1e-3,
        "a corrente esticada saiu com barriga (cruzado {cruz})"
    );
    let (_, ancora, ..) = anchors(&sim)[0];
    assert!(
        (ancora[1] - 100.0).abs() < 1e-6,
        "a ancora moveu-se sozinha para {ancora:?} - ela e' o valor AUTORADO"
    );
}

/// ⭐⭐⭐ **UMA CORRENTE PARADA NÃO ESCREVE** — o defeito que esta sonda achou, virado gate.
///
/// ⛔ **Medido no app a correr** (2026-09-07, `PH2D_BONE_LOG=1`): o braço da cena de smoke,
/// **parado**, resolvia em **1485 de 1485** quadros em 25 s, e o punho oscilava `±5e-4 rad` com a
/// amplitude a crescer. Uma corrente esticada que treme é a queixa que o artista faz.
///
/// ⚠️⚠️ **A fixtura `braco()` NÃO produz o fenómeno** — ali as coordenadas são redondas e exactas em
/// `f32`, e o sistema fica parado por acaso. *Onde os objectos NASCEM é a fixtura que os gates estão
/// a perder*: esta constrói a cadeia pela **porta do gesto**, nas coordenadas da cena real.
///
/// As duas causas, as duas curadas em `ph2d-skeleton`: a resposta exacta da recta faltava no ramo
/// de 2 ossos (o de 3+ já a tinha), e o sinal da dobra estava **invertido** contra a álgebra.
#[test]
fn a_still_chain_writes_nothing() {
    // ⚠️ **AS COORDENADAS DA CENA DO SMOKE**, e não as redondas do `braco()`: ali tudo é exacto em
    // `f32` e o sistema fica parado por acaso. *Onde os objectos NASCEM é a fixtura que os gates
    // estão a perder.*
    let mut sim = SimWorld::default();
    let mut pai: Option<Entity> = None;
    let mut punho = None;
    for i in 0..3 {
        let x = -8.2 + f64::from(i) * (6.4 / 3.0);
        let bits = crate::bone_gesture::create(&mut sim, pai, [x, 2.5], [x + 6.4 / 3.0, 2.5])
            .expect("osso");
        pai = Some(Entity::from_bits(bits));
        punho = pai;
    }
    let punho = punho.expect("punho");
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    add(&mut sim, punho).expect("a ancora");
    let mut pv = PreviewDrive::default();
    let mut escreveu = 0;
    let mut angulos: Vec<f32> = Vec::new();
    for _ in 0..300 {
        escreveu += solve(&mut sim, &mut pv);
        angulos.push(sim.world().get::<Transform>(punho).expect("t").rotation);
    }
    let ultimos: Vec<f32> = angulos[290..].to_vec();
    let min = ultimos.iter().copied().fold(f32::MAX, f32::min);
    let max = ultimos.iter().copied().fold(f32::MIN, f32::max);
    let amplitude = f64::from(max - min);
    eprintln!("[probe] escreveu em {escreveu} de 300 quadros; amplitude {amplitude:.3e} rad");
    assert_eq!(
        escreveu, 0,
        "a corrente PARADA reescreveu a pose em {escreveu} de 300 quadros (amplitude \
         {amplitude:.3e} rad) - ela esta' a vibrar, e no app isso e' 60 vezes por segundo"
    );
}

/// ⚠️ **SONDA de relógio:** o que a âncora custa por quadro, com e sem restrição na cena.
#[test]
#[ignore = "sonda de relógio: imprime, não julga"]
fn measure_the_price_of_one_frame_of_anchors() {
    use std::time::Instant;
    // Uma cena com 2000 objectos, que é a ordem de um documento cheio.
    let povoar = |sim: &mut SimWorld| {
        for i in 0..2000 {
            sim.world_mut().spawn((
                Transform::IDENTITY,
                Name::new(format!("Obj {i}")),
                RootOrder(0),
            ));
        }
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    };
    let medir = |sim: &mut SimWorld, n: usize| {
        let mut pv = PreviewDrive::default();
        let t = Instant::now();
        for _ in 0..n {
            let _ = anchors(sim);
            let _ = solve(sim, &mut pv);
        }
        t.elapsed().as_secs_f64() / n as f64 * 1e6
    };
    let (mut vazia, _) = braco();
    povoar(&mut vazia);
    let sem = medir(&mut vazia, 200);
    let (mut cheia, [_, cotovelo]) = braco();
    povoar(&mut cheia);
    add(&mut cheia, cotovelo).expect("a ancora");
    drag_anchor(&mut cheia, cotovelo, [4.0, 9.0]);
    let com = medir(&mut cheia, 200);
    eprintln!(
        "[probe] 2000 objectos: SEM ancora {sem:.1} us/quadro ({:.3} % de um quadro de 16,7 ms) \
         · COM uma {com:.1} us ({:.3} %)",
        sem / 16_700.0 * 100.0,
        com / 16_700.0 * 100.0
    );
}

/// ⚠️ **SONDA:** a âncora atravessa a captura do undo? (Report do dono, 2026-09-07: *«Undo não
/// funciona para add IK»*.)
///
/// Ela separa as DUAS metades que o sintoma não distingue: *a fotografia não vê a âncora* (e aí o
/// passo nasce vazio) contra *a fotografia vê e o passo não é registado* (e aí a causa é um dos
/// cinco motivos de supressão do `post_frame_undo`).
#[test]
#[ignore = "sonda de medição: imprime, não julga"]
fn probe_does_the_anchor_cross_the_undo_capture() {
    use ph2d_ecs::scene::{ComponentRegistry, register_ecs_components};
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    ph2d_render::register_render_components(&mut reg);
    ph2d_skeleton_ecs::register_skeleton_components(&mut reg);

    let (mut sim, [_, cotovelo]) = braco();
    let vec = ph2d_vec_scene::VecScene::new();
    let mut cache = ph2d_ecs::scene::incremental::CaptureCache::new();
    let tirar = |sim: &mut SimWorld, cache: &mut ph2d_ecs::scene::incremental::CaptureCache| {
        crate::undo::ProjectState::capture(
            &PreviewDrive::default(),
            sim,
            &vec,
            &ph2d_flip::FlipDoc::new(),
            &ph2d_guides::GuideSet::default(),
            &ph2d_ui_state::StateSets::default(),
            &crate::project_library::LibraryDoc::default(),
            &reg,
            cache,
            None,
        )
    };
    let antes = tirar(&mut sim, &mut cache);
    let alvo = add(&mut sim, cotovelo).expect("a ancora");
    let depois = tirar(&mut sim, &mut cache);
    eprintln!(
        "[probe] a captura VE' a ancora? {} (partes que diferem: {:?})",
        antes != depois,
        depois.parts_that_differ(&antes)
    );
    // E o restauro leva-a embora?
    let _ = antes.restore(&mut sim, &reg);
    eprintln!(
        "[probe] depois do restore: alvo vivo? {} · o osso ainda tem ancora? {}",
        sim.world().get_entity(alvo).is_ok(),
        sim.world()
            .iter_entities()
            .any(|er| er.contains::<ph2d_skeleton_ecs::IkGoal>())
    );
}

#[path = "skeleton_agenda_tests.rs"]
mod agenda;

/// ⭐⭐⭐ **APAGAR A ÂNCORA DEVOLVE A POSE QUE O ARTISTA AUTOROU** (report do dono, 2026-09-07:
/// *«Remove IK … não funciona plenamente»*).
///
/// ⛔ **Sem isto o verbo ASSAVA a pose da restrição no documento, em silêncio**: a âncora saía e a
/// corrente ficava dobrada onde ela a tinha posto, sem caminho de volta. E isso contradizia a lei
/// que este módulo escreveu — *o que a restrição escreve é pré-visualização*.
///
/// ⚠️ **O gate mede a POSE, e não a ausência do componente.** Apagar o `IkGoal` já funcionava; o
/// que não funcionava era *plenamente*, e a diferença entre as duas é exactamente a rotação dos
/// ossos.
#[test]
fn removing_the_anchor_gives_the_authored_pose_back() {
    let (mut sim, [ombro, cotovelo]) = braco();
    add(&mut sim, cotovelo).expect("a ancora");
    let autorada = crate::skeleton_live::bone_segments(&sim);
    let mut pv = PreviewDrive::default();
    drag_anchor(&mut sim, cotovelo, [4.0, 9.0]);
    solve(&mut sim, &mut pv);
    assert!(
        crate::skeleton_live::bone_segments(&sim) != autorada,
        "a fixtura nao produz o fenomeno: a restricao nao dobrou a corrente"
    );
    assert!(remove(&mut sim, cotovelo, &mut pv), "havia ancora");
    assert_eq!(
        crate::skeleton_live::bone_segments(&sim),
        autorada,
        "a corrente ficou dobrada onde a ancora a pos - o verbo ASSOU a pre-visualizacao no \
         documento, e nao ha' caminho de volta"
    );
    let _ = ombro;
}

/// ⚠️ **E o que ele devolve é o AUTORADO, não a pose de repouso** — se o artista girou o ombro à
/// mão antes de criar a âncora, é ESSA pose que volta.
///
/// ⛔ Um gate que só medisse *«voltou ao que estava antes de arrastar»* ficaria verde sobre uma
/// implementação que endireitasse a corrente, e o artista perderia trabalho.
#[test]
fn what_comes_back_is_the_authored_pose_not_the_rest_pose() {
    let (mut sim, [ombro, cotovelo]) = braco();
    // O artista posa o ombro À MÃO, e só então cria a âncora.
    sim.world_mut()
        .get_mut::<Transform>(ombro)
        .expect("t")
        .rotation = 0.4;
    add(&mut sim, cotovelo).expect("a ancora");
    let autorada = crate::skeleton_live::bone_segments(&sim);
    let mut pv = PreviewDrive::default();
    drag_anchor(&mut sim, cotovelo, [2.0, 12.0]);
    solve(&mut sim, &mut pv);
    remove(&mut sim, cotovelo, &mut pv);
    // ⚠️ A comparação é em `f32`, que é o tipo em que a pose VIVE: alargá-la para `f64` e comparar
    // com o literal `0.4` mede o arredondamento do `f32`, não o produto.
    assert_eq!(
        sim.world().get::<Transform>(ombro).expect("t").rotation,
        0.4_f32,
        "o ombro voltou ao repouso em vez da pose que o artista tinha feito"
    );
    assert_eq!(crate::skeleton_live::bone_segments(&sim), autorada);
}

//! ⭐ **A ÂNCORA DE IK** — criar o alvo de uma corrente, e as perguntas que ela faz ao mundo.
//!
//! Lei pura sobre o `SimWorld`, saída de `shells/desktop/src/skeleton_goal.rs` pelo mesmo motivo
//! do irmão [`crate::bone`]: a cena `PH2D_VEC_BONE_SMOKE` chama o [`add`].

use ph2d_ecs::{Entity, Name, RootOrder, SimWorld, Transform};

/// O nome que a âncora recebe ao nascer.
///
/// ⚠️⚠️ **O valor é `"IK Goal"` e foi LIDO do ficheiro da shell, não escrito de memória.** A
/// primeira redacção desta extracção pôs `"IK Target"` — plausível, e uma renomeação SILENCIOSA
/// de todo objecto que o gesto cria. O compilador só a apanhou porque a const ficou ausente; se
/// eu a tivesse trazido com o nome certo e o valor errado, nada teria falhado.
///
/// ⛔ A shell RE-EXPORTA esta — duas cópias de um literal são duas respostas à mesma pergunta.
pub const ANCHOR_NAME: &str = "IK Goal";
use ph2d_skeleton_ecs::{Bone, IkGoal};

/// **A corrente que esta âncora governa** — os `n` ossos que acabam em `tip`, da raiz para a ponta.
///
/// ⚠️ `chain == 0` significa *até à raiz do esqueleto*, que é a leitura do Blender. E o tecto real
/// é a corrente que EXISTE: um número absurdo vindo de um ficheiro é aparado pela árvore, não por
/// uma constante escolhida.
pub fn governed(sim: &SimWorld, tip: Entity, chain: u32) -> Vec<Entity> {
    let toda = crate::skin_live::chain_to(sim, tip.to_bits());
    if chain == 0 {
        return toda;
    }
    let n = (chain as usize).min(toda.len());
    toda[toda.len() - n..].to_vec()
}

/// **AS JUNTAS DE UMA CORRENTE, em MUNDO** — `corrente.len() + 1` posições e os comprimentos entre
/// elas. `None` quando um osso da corrente não tem segmento (a forma nasceu neste quadro).
///
/// ⭐ **Uma porta, dois consumidores:** quem RESOLVE ([`solve_one`]) e quem CAPTURA o lado da dobra
/// ([`add`]). ⚠️ Escrita duas vezes, ela divergia no dia em que um dos dois passasse a saltar um
/// osso — e o sintoma seria a âncora nascer com o lado do vizinho.
pub fn joints_of(sim: &SimWorld, corrente: &[Entity]) -> Option<(Vec<[f64; 2]>, Vec<f64>)> {
    let segs = crate::skin_live::bone_segments(sim);
    let mut juntas: Vec<[f64; 2]> = Vec::with_capacity(corrente.len() + 1);
    let mut comps: Vec<f64> = Vec::with_capacity(corrente.len());
    for (i, &e) in corrente.iter().enumerate() {
        let (_, a, b) = segs.iter().copied().find(|(x, _, _)| *x == e.to_bits())?;
        juntas.push(a);
        comps.push((b[0] - a[0]).hypot(b[1] - a[1]));
        if i + 1 == corrente.len() {
            juntas.push(b);
        }
    }
    Some((juntas, comps))
}

/// ⭐⭐⭐ **DE QUE LADO A CORRENTE JÁ ESTÁ**, no instante em que a âncora nasce.
///
/// A recta de referência é `raiz → ponta`, e ela é exactamente a certa aqui: o alvo nasce **na
/// ponta** (é o que faz criar a âncora ser um no-op visual), logo `raiz → alvo` e `raiz → ponta`
/// são a mesma recta.
///
/// ⚠️ Uma corrente que nasce **recta** não tem lado, e aí devolve-se [`BendSide::Keep`] — que é o
/// desempate determinístico de sempre. ⛔ Escolher um lado ali seria inventar uma decisão do artista
/// a partir de ruído de `f32`.
pub fn captured_side(sim: &SimWorld, corrente: &[Entity]) -> ph2d_skeleton::BendSide {
    let Some((juntas, comps)) = joints_of(sim, corrente) else {
        return ph2d_skeleton::BendSide::Keep;
    };
    let alcance: f64 = comps.iter().sum();
    let Some(&ponta) = juntas.last() else {
        return ph2d_skeleton::BendSide::Keep;
    };
    // ⚠️ **Quem decide o que «recta» significa é a LEI**, não esta função: a barra é uma fracção do
    // alcance e vive lá dentro, ao lado do arqueamento que a usa.
    ph2d_skeleton::bend_side_of(&juntas, ponta, alcance)
}

/// ⭐⭐⭐ **O LADO QUE ESTA CORRENTE JÁ TEM, para um `chain` dado** — a porta que o *Add IK* e a
/// mudança do número **Chain** partilham.
///
/// ⛔⛔ **ORDEM DO DONO** (2026-09-14): *«o lado da dobra é capturado no momento em que carrega Add
/// IK e sempre que IK Chain for mudado»*. A razão é geométrica: o lado descreve **uma corrente**, e
/// subir o `Chain` de `2` para `4` troca a corrente por outra — o bit guardado passaria a falar de
/// uma geometria que já não é a que está debaixo do artista.
///
/// ⚠️ **Uma corrente RECTA continua a devolver [`ph2d_skeleton::BendSide::Keep`]** (a lei do
/// [`captured_side`]): ali não há lado para capturar, e inventar um seria fabricar uma decisão do
/// artista a partir de ruído de `f32`.
#[must_use]
pub fn side_for_chain(sim: &SimWorld, bone: Entity, chain: u32) -> ph2d_skeleton::BendSide {
    captured_side(sim, &governed(sim, bone, chain))
}

/// ⭐⭐⭐ **CRIA a âncora** deste osso, com o alvo pousado na ponta dele. Devolve o alvo.
///
/// ⚠️ **Nasce COINCIDENTE com a ponta**, e isso é a lei da casa aplicada: *todo motor novo é no-op
/// no ponto neutro*. Carregar em *Add IK* não pode mover o desenho — se movesse, o artista perderia
/// a pose que acabou de fazer e a feature seria uma armadilha.
///
/// ⚠️ **O alvo nasce RAIZ**, e não filho do osso: filho da corrente é exactamente o laço que o
/// [`feeds_back`] recusa.
///
/// `None` se `bone` não é um osso, ou se ele já tem âncora (o painel não oferece o botão nesse
/// caso — e recusar aqui também é o que impede duas âncoras a puxar a mesma corrente).
pub fn add(sim: &mut SimWorld, bone: Entity) -> Option<Entity> {
    if sim.world().get::<Bone>(bone).is_none() || sim.world().get::<IkGoal>(bone).is_some() {
        return None;
    }
    let ponta = crate::bone::tip_of(sim, bone.to_bits())?;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "o `Transform` da casa é f32; a geometria do documento é f64"
    )]
    let alvo = sim
        .world_mut()
        .spawn((
            Transform {
                translation: ph2d_core::Vec2::new(ponta[0] as f32, ponta[1] as f32),
                ..Transform::IDENTITY
            },
            Name::new(ANCHOR_NAME),
            RootOrder(0),
            // ⚠️ **A marca vem no spawn**, e não depois: entre o spawn e um `insert` seguinte corre
            // pelo menos um `empty_objects`, e o alvo apareceria com o anel do objecto vazio por um
            // quadro. *Um piscar de um quadro é indistinguível de um defeito intermitente.*
            ph2d_skeleton_ecs::IkTarget,
        ))
        .id();
    // ⚠️ **Semear o id ANTES de o guardar** — um objecto criado neste quadro ainda não tem
    // `StableId` (a varredura corre uma vez por quadro), e sem isto a âncora nomearia `NONE`. É o
    // mesmo que o `skeleton_live::bind` faz, e pela mesma razão.
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let id = ph2d_ecs::stable_id_of(sim.world(), alvo)?;
    // ⭐⭐⭐ **O LADO É CAPTURADO, não escolhido** — a âncora nasce a defender a dobra que o artista
    // já posou à mão. ⚠️ Sem isto ela nasceria em `Keep`, e a primeira vez que ele esticasse o
    // membro o joelho inverteria sozinho: é o defeito medido em
    // `the_elbow_flips_when_the_chain_passes_through_straight`.
    let bend = side_for_chain(sim, bone, ph2d_skeleton_ecs::DEFAULT_CHAIN);
    sim.world_mut().entity_mut(bone).insert(IkGoal {
        target: id,
        bend,
        ..IkGoal::default()
    });
    Some(alvo)
}

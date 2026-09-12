//! ⭐⭐⭐ **O LIMITE DE UMA JUNTA** — irmão do [`super::bone_gesture`] pelo teto de 600 LOC do
//! HR-18, com o corte por RESPONSABILIDADE: ali mora *o que a mão faz a um osso*, aqui *até onde
//! ele pode ir*.
//!
//! ⚠️ **A [`limited`] é a porta que as DUAS mãos atravessam** — o gesto que gira o osso e o solver
//! que o gira a cada quadro. Um limite que só uma delas honrasse é pior que limite nenhum: a junta
//! obedeceria ou não conforme quem a moveu, e o artista não conseguiria formar um modelo do que a
//! ferramenta faz.
//!
//! ⛔ **O Godot põe isto na RESTRIÇÃO de IK** (`ccdik_joint_constraint_angle_min`/`_max`), e nós
//! pomos no OSSO — o modelo do Blender e do Moho. A razão está no doc do
//! [`ph2d_skeleton_ecs::BoneLimit`]: *«este cotovelo não dobra para trás»* é uma afirmação sobre a
//! ANATOMIA, logo vale sem IK nenhuma e não pode evaporar num *Remove IK*.

use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_vec_scene::Xform;

/// ⭐⭐⭐ **DÁ A ESTA JUNTA UM LIMITE, em volta da pose que ela TEM.**
///
/// ⚠️ **A faixa é capturada, como o lado da dobra da âncora** — e é a mesma razão: uma faixa
/// escrita de fora (o `Default` do componente, a volta inteira) não move a pose mas também não diz
/// nada ao artista; uma faixa em volta da pose ACTUAL nasce com o osso no meio dela, então ele vê
/// para onde pode ir e aperta a partir daí.
///
/// ⭐ **É um no-op exacto na mesma**, e não por sorte: a pose está no CENTRO da faixa, logo dentro
/// dela — e a lei devolve `rot` ao bit quando ele está dentro.
///
/// ⚠️ O quarto de volta é o valor de **nascimento** e não um tecto de recurso — e ele **não** é um
/// número medido: é o ponto de partida de onde o artista aperta ou alarga, e a `drag_edge` alcança
/// até meia volta.
///
/// ⛔⛔ **A nota anterior justificava-o com uma tabela que o desmente** (auditoria de 2026-09-08):
/// ela dizia *«cobre com folga a amplitude das juntas que um rig tem (cotovelo ~150°, joelho ~140°,
/// dedo ~90°)»* — e 90° é **menor** que os dois primeiros. *Uma justificação que contradiz a própria
/// tabela é pior que nenhuma: ela convence.*
///
/// ⚠️ E ele **não é o mesmo** do `Default` do [`ph2d_skeleton_ecs::BoneLimit`], que é a **volta
/// inteira** — a nota do `SmartBone` chegou a afirmar a igualdade, e as duas metades eram falsas.
pub(crate) fn add_limit(sim: &mut SimWorld, bone: Entity) -> bool {
    if sim.world().get::<ph2d_skeleton_ecs::Bone>(bone).is_none()
        || sim
            .world()
            .get::<ph2d_skeleton_ecs::BoneLimit>(bone)
            .is_some()
    {
        return false;
    }
    let Some(t) = sim.world().get::<Transform>(bone).copied() else {
        return false;
    };
    let centro = f64::from(t.rotation);
    let meia = ph2d_skeleton::FULL_TURN / 8.0; // o quarto de volta, metade de cada lado
    sim.world_mut()
        .entity_mut(bone)
        .insert(ph2d_skeleton_ecs::BoneLimit {
            min: centro - meia,
            max: centro + meia,
        });
    true
}

/// Tira o limite: a junta volta a girar livremente. ⛔ Ela **não** é reposta na pose de repouso —
/// tirar uma cerca não é desfazer o que se fez dentro dela.
pub(crate) fn remove_limit(sim: &mut SimWorld, bone: Entity) -> bool {
    sim.world_mut()
        .entity_mut(bone)
        .take::<ph2d_skeleton_ecs::BoneLimit>()
        .is_some()
}

/// ⭐⭐⭐ **A ROTAÇÃO QUE ESTE OSSO PODE DE FACTO TER** — a pedida, aparada pelo limite dele.
///
/// ⚠️ **Porta única de dois consumidores, exactamente como a [`aim_rotation`] ao lado**: o gesto que
/// gira um osso à mão e a restrição que o gira a cada quadro. É isso que faz *«este cotovelo não
/// dobra para trás»* valer nas duas — um limite que só o solver honrasse seria um limite que o dedo
/// atravessa, e o artista veria a junta obedecer ou não conforme quem a moveu.
///
/// ⚠️ **Sem [`ph2d_skeleton_ecs::BoneLimit`] ela devolve `rot` AO BIT** — a junta sem limite é a
/// esmagadora maioria, e ela não paga nada por esta lei existir.
pub(crate) fn limited(sim: &SimWorld, bone: Entity, rot: f64) -> f64 {
    sim.world()
        .get::<ph2d_skeleton_ecs::BoneLimit>(bone)
        .map_or(rot, |l| ph2d_skeleton::clamp_to_limit(rot, l.min, l.max))
}

/// Quantos segmentos o leque do arco tem por volta inteira.
///
/// ⚠️ **Por VOLTA e não por arco**, para a densidade do traço não depender de quão apertado o
/// limite está: com um número fixo por arco, uma faixa de 5° teria a mesma contagem que uma de 300°
/// e gastaria trinta pontos onde dois bastam.
const FAN_PER_TURN: usize = 48;
// ⚠️ **De que recurso é o `48`** (auditoria de 2026-09-08, que o apanhou a não o dizer): é da
// **resolução do ECRÃ**, não do relógio. A 48 por volta cada segmento cobre `7,5°`, e a corda de um
// arco de `7,5°` afasta-se do arco em `r · (1 − cos 3,75°) ≈ r/1170` — abaixo de meio pixel para
// qualquer osso com menos de 585 px no ecrã, que é maior que o canvas inteiro no zoom em que um rig
// se posa. ⛔ Não é um tecto de custo: o leque de uma faixa de 90° tem **12** triângulos.

/// ⭐⭐⭐ **O ARCO DE LIMITE de uma junta, em MUNDO** — o que se desenha e o que o dedo apanha.
///
/// ⭐⭐ **O raio é o comprimento do osso**, então o arco É o caminho que a ponta percorre. ⛔ Um raio
/// escolhido seria um número sem dono.
///
/// ⚠️ **Tudo se constrói no espaço do PAI e só depois se leva a mundo**, ponto a ponto. Compor
/// ângulos (`local + rotação do pai`) só está certo com um pai conforme; transformar pontos está
/// certo com qualquer afim — é a mesma lei que o [`crate::bone_gesture::aim_rotation`] segue, e sob
/// um pai escalado só num eixo o arco é uma ELIPSE, que é o que o artista tem de ver.
pub(crate) fn arc(
    sim: &SimWorld,
    bone: Entity,
    px_to_world: f64,
) -> Option<ph2d_skeleton_render::LimitArc> {
    let l = *sim.world().get::<ph2d_skeleton_ecs::BoneLimit>(bone)?;
    let comp = sim.world().get::<ph2d_skeleton_ecs::Bone>(bone)?.length;
    if !(comp.is_finite() && comp > 0.0) {
        return None;
    }
    let t = sim.world().get::<Transform>(bone)?;
    let o = [f64::from(t.translation.x), f64::from(t.translation.y)];
    let pai = sim
        .world()
        .get::<ph2d_ecs::ChildOf>(bone)
        .map(ph2d_ecs::ChildOf::parent);
    let pai_mundo = pai.map_or(Xform::IDENTITY, |p| {
        ph2d_vec_entities::transform::xform_of_transform(
            ph2d_vec_entities::transform::world_transform(sim, p),
        )
    });
    let ponto = |a: f64| pai_mundo.apply([o[0] + comp * a.cos(), o[1] + comp * a.sin()]);
    let (lo, hi) = (l.min.min(l.max), l.min.max(l.max));
    let faixa = (hi - lo).min(ph2d_skeleton::FULL_TURN);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a contagem sai de uma fracção de volta, sempre pequena e positiva"
    )]
    let n = ((faixa / ph2d_skeleton::FULL_TURN) * FAN_PER_TURN as f64).ceil() as usize;
    let n = n.max(1);
    let fan = (0..=n)
        .map(|i| {
            #[expect(clippy::cast_precision_loss, reason = "n é um punhado de segmentos")]
            let t = i as f64 / n as f64;
            ponto(lo + faixa * t)
        })
        .collect();
    // ⭐⭐⭐ **AS ALÇAS SAEM PARA FORA DO ALCANCE DO OSSO.**
    //
    // ⛔ A folga é **derivada, nunca escolhida**: um dedo da casa (`BONE_HIT_PX`, o que separa os
    // dois alvos) mais o raio da própria alça (`LIMIT_HANDLE_R_PX`, para o triângulo INTEIRO ficar
    // fora, e não só o centro dele). É o mesmo mecanismo do piso do anel da âncora, que também sai
    // do dedo em vez de um literal.
    //
    // ⚠️ E é por isto que este passe precisa do ZOOM: a folga é uma grandeza de TELA (o dedo mede
    // píxeis) sobre uma geometria de MUNDO. Num osso curto uma folga fixa em mundo cairia dentro do
    // osso, e num longo ficaria a meio metro dele.
    let folga = (crate::bone_pick::BONE_HIT_PX + ph2d_skeleton_render::LIMIT_HANDLE_R_PX)
        * px_to_world.max(0.0);
    let fora = |a: f64| {
        pai_mundo.apply([
            o[0] + (comp + folga) * a.cos(),
            o[1] + (comp + folga) * a.sin(),
        ])
    };
    Some(ph2d_skeleton_render::LimitArc {
        apex: pai_mundo.apply(o),
        edge_min: ponto(lo),
        edge_max: ponto(hi),
        handle_min: fora(lo),
        handle_max: fora(hi),
        fan,
    })
}

/// ⭐⭐⭐ **ARRASTAR UMA PAREDE** — o ponteiro dá o ângulo, e ele vira a borda pedida.
///
/// ⚠️ **A porta do ângulo é a MESMA que o gesto de girar usa** ([`crate::bone_gesture::aim_rotation`]):
/// a alça está onde a ponta do osso estaria naquele ângulo, então *«arrastar a parede»* e *«girar o
/// osso»* têm de ler o ponteiro pela mesma lei. Duas leituras divergiriam sob um pai escalado, e o
/// artista veria a parede parar num sítio diferente de onde a ponta pára.
///
/// ⛔ **A borda arrastada nunca ATRAVESSA a outra.** Sem isso um puxão a mais inverteria a faixa, e
/// a lei responde a isso travando a junta no centro — o artista veria o osso saltar para o meio e
/// deixar de rodar, sem nada que explicasse porquê. Ela pára **colada** à outra, que é a faixa
/// nula: apertar até não sobrar nada é uma coisa que ele pode querer, inverter não.
pub(crate) fn drag_edge(sim: &mut SimWorld, bone: Entity, world: [f64; 2], is_max: bool) -> bool {
    let Some(a) = crate::bone_gesture::aim_rotation(sim, bone, world) else {
        return false;
    };
    let Some(l) = sim
        .world()
        .get::<ph2d_skeleton_ecs::BoneLimit>(bone)
        .copied()
    else {
        return false;
    };
    // O ângulo vem em `(-π, π]` e a faixa vive em torno do centro — trazer a leitura para a volta
    // do OUTRO extremo é o que faz a parede seguir o dedo em vez de saltar meia volta.
    let outro = if is_max { l.min } else { l.max };
    let pedido = outro + ph2d_skeleton::wrap_pi(a - outro);
    let Some(mut m) = sim
        .world_mut()
        .get_mut::<ph2d_skeleton_ecs::BoneLimit>(bone)
    else {
        return false;
    };
    set_edge(&mut m, is_max, pedido);
    true
}

/// ⭐⭐⭐ **A PORTA ÚNICA DE MOVER UMA PAREDE** — e ela impede que uma paredes passe a outra.
///
/// ⛔⛔ **Ela existe porque o gesto tinha o guarda e os CAMPOS não** (auditoria de 2026-09-08): o
/// arrasto travava a parede no vizinho, e digitar `Limit Min = 90` com `Limit Max = 45` escrevia
/// cru. Com `min > max` a lei devolve `meia = 0` e a junta **congela** no ponto médio — e o desenho
/// **normaliza** (`arc()` ordena os dois), então o canvas pinta um sector perfeitamente normal
/// enquanto o osso não roda um grau. *A cura de um guarda escrito num sítio não é escrevê-lo no
/// outro: é as duas superfícies passarem pela mesma porta.*
///
/// ⚠️ A parede empurrada **para além** da vizinha para NELA, ⛔ nunca a atravessa: uma faixa
/// invertida é estado que a lei aceita sem entrar em pânico (há gate) e que o artista não consegue
/// explicar — o osso salta para o meio e deixa de rodar, sem nada que o diga.
pub(crate) fn set_edge(l: &mut ph2d_skeleton_ecs::BoneLimit, is_max: bool, pedido: f64) {
    if is_max {
        l.max = pedido.max(l.min);
    } else {
        l.min = pedido.min(l.max);
    }
}

//! ⭐⭐⭐ **O LADO DA DOBRA ANIMA** — pedido do dono, 2026-09-18.
//!
//! Ele escolheu *«o lado da dobra é escolhido e é consistente, mas **não é animável**»*, e a
//! resposta **não** é o *Pole Target* do Blender: há recusa MEDIDA contra ele, e ela fica de pé —
//! em 3D o triângulo raiz–cotovelo–ponta roda em torno do eixo raiz→ponta (um grau de liberdade
//! **contínuo**, que um objecto no espaço fixa) e no plano isso não existe: sobra **um bit**. Um
//! alvo arrastável que codifica um bit dá a ilusão de um controlo contínuo e **salta** ao cruzar a
//! recta, e é por isso que Godot e Spine, independentes, escolheram o interruptor.
//!
//! ⚠️⚠️ **ESTE MÓDULO VIVE ATRÁS DA FEATURE `skeleton`, e isso tem uma consequência medida:** um
//! `cargo test -p ph2d-timeline` SOZINHO compila-o fora e imprime `0 passed` — *um teste que não
//! corre lê-se exactamente como um teste verde*. O portão do workspace corre-o porque a shell liga
//! a feature (conferido: `nextest list --workspace` lista os dois), e quem o correr isolado tem de
//! passar `--features skeleton`.
//!
//! ⇒ *o que faltava não era o alvo: era a ANIMABILIDADE do bit*, que o Spine tem (`bendDirection`,
//! animável) e nós não tínhamos. O mecanismo inteiro vive no doc do [`ph2d_skeleton::BendSide`].

use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_timeline::{PropKind, TimelineState, apply_from_doc};

/// ⭐⭐⭐ **DUAS CHAVES DE SINAIS OPOSTOS TROCAM O LADO NO TEMPO** — a capacidade inteira num teste.
///
/// ⚠️ **As TRÊS metades são três defeitos:** o canal não escrever (o joelho fica onde estava), o
/// canal escrever sempre o mesmo lado (a animação não anima), e o meio da interpolação não trocar
/// (o artista vê o salto no sítio errado).
#[test]
fn duas_chaves_opostas_trocam_o_lado_do_joelho_no_tempo() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("Cotovelo"),
            ph2d_skeleton_ecs::Bone::default(),
            ph2d_skeleton_ecs::IkGoal::default(),
        ))
        .id()
        .to_bits();

    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    doc.bind(e, PropKind::IkBendSide);
    for (t, v) in [(0.0, 1.0_f32), (2.0, -1.0_f32)] {
        doc.upsert_key(
            e,
            PropKind::IkBendSide,
            RationalTime::from_seconds(t),
            AnimValue::Float(v),
            Interp::Linear,
        );
    }

    let lado = |sim: &SimWorld| {
        sim.world()
            .get::<ph2d_skeleton_ecs::IkGoal>(Entity::from_bits(e))
            .expect("a restricao continua la'")
            .bend
    };

    apply_from_doc(sim.world_mut(), doc, 0.0);
    assert_eq!(
        lado(&sim),
        ph2d_skeleton_ecs::BendSide::Ccw,
        "no instante 0 a chave vale +1 e o joelho tinha de estar do lado anti-horario: o canal \
         nao chegou ao `IkGoal`, e uma track que nao escreve le-se como um joelho que nao obedece"
    );

    apply_from_doc(sim.world_mut(), doc, 2.0);
    assert_eq!(
        lado(&sim),
        ph2d_skeleton_ecs::BendSide::Cw,
        "no instante 2 a chave vale −1 e o lado NAO trocou: o canal escreve sempre o mesmo, e \
         entao ele nao anima coisa nenhuma"
    );

    // ⭐ E o MEIO: a interpolação é linear, logo o sinal vira exactamente onde a curva cruza o
    // zero. ⚠️ O empate cai para o anti-horário, que é a convenção DECLARADA (`>= 0`) — sem ela o
    // instante do salto dependeria do último bit de um `f32`.
    apply_from_doc(sim.world_mut(), doc, 1.0);
    assert_eq!(
        lado(&sim),
        ph2d_skeleton_ecs::BendSide::Ccw,
        "no zero exacto o lado nao caiu para o vencedor declarado: um bit sem regra de empate \
         troca de lado conforme o arredondamento"
    );
    apply_from_doc(sim.world_mut(), doc, 1.5);
    assert_eq!(
        lado(&sim),
        ph2d_skeleton_ecs::BendSide::Cw,
        "passado o zero o lado nao mudou: a troca nao acompanha a curva"
    );
}

/// ⭐⭐⭐ **O CANAL RESOLVE-SE DO ID QUE O DOCUMENTO GRAVA** — e esta linha faltou, em silêncio.
///
/// ⛔⛔ O `AnimTarget` é o id **opaco** que a track guarda; o `from_target` que o traduz de volta tem
/// um `_ => None`, e a variante nova caiu nele **sem um aviso** — os quatro `match` exaustivos desta
/// crate obrigaram-me a responder, este não. Sem a linha, uma track gravada não se resolve ao
/// carregar, e o gate genérico de ida-e-volta **saltava o canal por vacuidade**.
///
/// ⚠️ *Um `match` com wildcard é onde uma variante nova desaparece*, e o preço aqui é a
/// PERSISTÊNCIA e não a compilação. Foi uma MUTAÇÃO que o mostrou.
#[test]
fn o_canal_resolve_se_do_id_que_o_documento_grava() {
    assert_eq!(
        PropKind::from_target(ph2d_anim::AnimTarget::new(17)),
        Some(PropKind::IkBendSide),
        "o id `17` nao volta a ser o lado da dobra: uma track gravada perde-se ao carregar, e o \
         canal existe so' enquanto a sessao dura"
    );
    // ⚠️ E a volta: o id que o canal declara tem de ser o mesmo. *Sem esta metade, um `17` escrito
    // à mão no gate passaria a concordar com um canal que mudou de discriminante* — que é a coisa
    // que um valor de fio congelado nunca pode fazer.
    assert_eq!(
        PropKind::IkBendSide.target().get(),
        17,
        "o discriminante do canal mudou: ele e' um valor de FIO congelado, e mexer nele torna \
         ilegivel todo documento ja' gravado"
    );
}

/// ⛔⛔ **SEM `IkGoal` O CANAL NÃO INVENTA UMA RESTRIÇÃO** — a metade negativa, e ela é metade do
/// valor.
///
/// ⚠️ Uma track sobre um osso que perdeu a âncora tem de ser **inerte**, não criar uma: *o
/// documento do artista não pode ganhar uma restrição porque uma track ficou para trás*.
#[test]
fn sem_restricao_o_canal_e_inerte() {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::default(),
            Name::new("Osso solto"),
            ph2d_skeleton_ecs::Bone::default(),
        ))
        .id()
        .to_bits();

    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    doc.bind(e, PropKind::IkBendSide);
    doc.upsert_key(
        e,
        PropKind::IkBendSide,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(-1.0),
        Interp::Linear,
    );
    apply_from_doc(sim.world_mut(), doc, 0.0);
    assert!(
        sim.world()
            .get::<ph2d_skeleton_ecs::IkGoal>(Entity::from_bits(e))
            .is_none(),
        "o canal CRIOU uma restricao de IK num osso que nao tinha nenhuma — o documento do \
         artista ganhou uma coisa que ele nao autorou"
    );
}

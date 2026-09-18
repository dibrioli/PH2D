//! ⭐⭐⭐ **O CENSO DOS NÚMEROS DO OSSO: o valor CHEGA a um consumidor?**
//!
//! ⛔⛔⛔ **É a pergunta que o `§5.0` nomeia sobre este repo inteiro** — *«nenhum instrumento
//! pergunta se o VALOR chega a um consumidor»* — e é a família dos dois reports do dono desta
//! semana (o botão que não era pintado; a recusa que não se via). Os censos que já existem provam
//! que o clique e o valor **chegam ao barramento**; nenhum prova que alguma coisa acontece a seguir.
//!
//! ⚠️⚠️ **E uma varredura por NOME não serve, medido:** a shell despacha estes ids **por tabela**
//! (`VECTOR_BONE_BEND_IDS.iter().position(…)`, `BoneKnob::of_id`), logo um `grep` pelo nome de cada
//! id na shell acusa **12 controlos VIVOS**. *Um id que a régua não vê e um id morto leem-se igual.*
//!
//! ⇒ a régua é o **PRODUTO**: aplicar o knob por [`super::apply`] com dois valores e medir a
//! **pose que a lei deriva** ([`ph2d_skeleton::bend::frames`]). Um knob que não move um único
//! quadro da pose não tem consumidor nesta grandeza — e então tem de **declarar qual é o dele**.

use super::{BoneKnob, apply};
use ph2d_skeleton_ecs::Bone;

/// A pose que a lei deriva deste osso — a grandeza em que um número do osso se vê.
fn pose(b: &Bone) -> Vec<[f64; 6]> {
    ph2d_skeleton::bend::frames(b.spec())
        .into_iter()
        .map(|x| x.0)
        .collect()
}

/// O osso do censo: **três segmentos**, porque com um só as quatro alças são *provadamente* inertes
/// (gate `one_segment_never_bends_whatever_the_handles_say`, em `ph2d-skeleton`).
///
/// ⚠️ **Sem isto o censo acusaria quatro knobs vivos de uma vez** — a mesma armadilha que o censo
/// irmão da escultura pagou, onde metade dos verbos era inerte por lei no arranjo de fábrica.
fn osso_do_censo() -> Bone {
    let mut b = Bone::default();
    apply(&mut b, BoneKnob::Length, 2.0);
    apply(&mut b, BoneKnob::Segments, 3.0);
    // ⛔⛔ **E uma CURVATURA, senão o `Length` lê-se MORTO sobre produto certo.** Num osso RECTO o
    // afim de flexão de cada sub-osso é a **identidade**, qualquer que seja o comprimento — o
    // comprimento aparece na geometria do osso, não naquele afim. *Uma régua medida no ponto neutro
    // de outro knob acusa este* (a mesma família do corpus no neutro que este repo já pagou duas
    // vezes), e foi o próprio censo que a apanhou na 1.ª corrida.
    apply(&mut b, BoneKnob::CurveInY, 0.3);
    b
}

/// Os dois valores em que cada knob é varrido, e **porquê** este par.
fn faixa(k: BoneKnob) -> (f64, f64) {
    match k {
        // ⚠️ Nunca `0` de um lado: um comprimento nulo degenera a pose e o censo leria «mexeu»
        // por a lei ter colapsado, não por o knob ter chegado.
        BoneKnob::Length => (1.0, 3.0),
        BoneKnob::Strength => (0.5, 2.0),
        BoneKnob::Segments => (2.0, 5.0),
        BoneKnob::CurveInX | BoneKnob::CurveOutX => (0.2, 0.8),
        BoneKnob::CurveInY | BoneKnob::CurveOutY => (-0.4, 0.4),
    }
}

/// ⭐⭐⭐ **TODO NÚMERO DO OSSO CHEGA À POSE — menos UM, que declara o consumidor dele.**
///
/// ⛔ **O `Strength` é a excepção NOMEADA e não uma folga:** ele é o *alcance*, e o consumidor dele
/// são os **pesos da pele** (`SkinBone::new`), não a pose do osso. *Uma régua que o medisse aqui
/// acusaria um controlo vivo* — e é por isso que ele tem a metade própria, abaixo.
#[test]
fn todo_numero_do_osso_chega_a_pose() {
    let mut mortos = Vec::new();
    let mut medidos = 0;
    for k in BoneKnob::TODOS {
        if k == BoneKnob::Strength {
            continue;
        }
        let (a, b) = faixa(k);
        let (mut x, mut y) = (osso_do_censo(), osso_do_censo());
        apply(&mut x, k, a);
        apply(&mut y, k, b);
        medidos += 1;
        if pose(&x) == pose(&y) {
            mortos.push(k);
        }
    }
    assert_eq!(
        medidos,
        BoneKnob::TODOS.len() - 1,
        "o censo mediu {medidos} knobs de {} — um knob novo entrou sem faixa, ou a lista TODOS \
         deixou de descrever a populacao",
        BoneKnob::TODOS.len() - 1
    );
    assert!(
        mortos.is_empty(),
        "estes numeros do osso nao movem um unico quadro da pose: {mortos:?} — o painel promete \
         um controlo que o barro nao sente, e o artista le' isso como «a ferramenta nao funciona»"
    );
}

/// ⚠️ **O CONTROLO POSITIVO da régua acima:** um knob que a lei IGNORA tem de ser acusado.
///
/// ⛔ Sem esta metade, uma `pose()` que devolvesse sempre a mesma coisa deixaria o gate irmão verde
/// sobre **todos** os knobs mortos ao mesmo tempo. *Uma régua que não vê o fenómeno acontecer não
/// prova que ele não aconteceu.*
#[test]
fn a_regua_do_censo_acusa_um_knob_que_a_lei_ignora() {
    let (mut x, mut y) = (osso_do_censo(), osso_do_censo());
    // O `strength` é, por lei, invisível na pose — é ele o controlo.
    apply(&mut x, BoneKnob::Strength, 0.5);
    apply(&mut y, BoneKnob::Strength, 4.0);
    assert_eq!(
        pose(&x),
        pose(&y),
        "o alcance passou a mover a pose: ou a lei mudou, ou esta regua deixou de medir a pose — \
         nos dois casos a excepcao NOMEADA do gate irmao deixou de ser verdade"
    );
}

/// ⭐⭐ **E o alcance CHEGA ao consumidor DELE** — os pesos da pele.
///
/// ⚠️ Sem esta metade o `Strength` ficaria fora do censo **sem prova nenhuma**, que é a diferença
/// entre uma excepção medida e uma folga. *Uma célula sem proveniência e uma com proveniência têm o
/// mesmo aspecto numa tabela.*
#[test]
fn o_alcance_chega_aos_pesos_da_pele() {
    let peso = |s: f64| {
        let mut b = osso_do_censo();
        apply(&mut b, BoneKnob::Strength, s);
        let spec = b.spec();
        let id = ph2d_skeleton::Xform::IDENTITY;
        let perto = ph2d_skeleton::SkinBone::new(id, spec.length, spec.strength, id, id)
            .expect("o osso do censo nao e' singular");
        // ⛔⛔ **DOIS ossos AO ALCANCE, e as duas correcções foram da RÉGUA.** Com **um** osso a
        // normalização dá-lhe sempre a fatia inteira (`1` contra `1`); com um segundo **fora** do
        // alcance, o `w[0]` continua `1` nos dois lados — ali quem responde é o caminho de recurso
        // *«o mais próximo leva tudo»*. ⇒ o vizinho tem de estar **dentro** do alcance do ponto,
        // para que a razão entre os dois seja o que se mede.
        let vizinho = ph2d_skeleton::SkinBone::new(
            ph2d_skeleton::Xform([1.0, 0.0, 0.0, 1.0, 0.0, spec.length * 1.5]),
            spec.length,
            2.0,
            id,
            id,
        )
        .expect("o osso vizinho");
        let pele = ph2d_skeleton::Skin::new(vec![perto, vizinho]).expect("uma pele de dois ossos");
        let mut w = pele.scratch();
        // O ponto fica FORA do eixo, a uma distância que o alcance curto não cobre e o longo sim.
        pele.weights_at([spec.length * 0.5, spec.length * 1.2], &mut w);
        w[0]
    };
    let (curto, longo) = (peso(0.3), peso(3.0));
    assert!(
        (curto - longo).abs() > 1e-9,
        "o alcance nao muda o peso da pele ({curto} contra {longo}): ele nao chega a consumidor \
         nenhum, e a excepcao do censo passa a ser uma folga"
    );
}

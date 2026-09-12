//! ⭐⭐⭐ **O QUE A ÂNCORA ESCREVE É PRÉ-VISUALIZAÇÃO** — irmão de [`super::tests`] pelo teto de 600
//! LOC, e o corte é por RESPONSABILIDADE: ali mede-se a **LEI** que a restrição resolve por quadro
//! (o alcance, o *Mix*, o lado da dobra); aqui, o que acontece ao **DOCUMENTO** — quem larga, quem
//! escreve por cima, e o que sobra quando o motor é desligado.
//!
//! ⚠️ A fixtura é a **mesma** (`super::tests::braco`) de propósito: dois braços para o mesmo módulo
//! divergiriam, e o gate que aqui mede o ledger tem de correr sobre a corrente que ali resolve.

use super::*;
use crate::goal::braco;

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
    let autorada = ph2d_skeleton_live::skin_live::bone_segments(&sim);
    let mut pv = PreviewDrive::default();
    drag_anchor(&mut sim, cotovelo, [4.0, 9.0]);
    solve(&mut sim, &mut pv);
    assert!(
        ph2d_skeleton_live::skin_live::bone_segments(&sim) != autorada,
        "a fixtura nao produz o fenomeno: a restricao nao dobrou a corrente"
    );
    assert!(remove(&mut sim, cotovelo, &mut pv), "havia ancora");
    assert_eq!(
        ph2d_skeleton_live::skin_live::bone_segments(&sim),
        autorada,
        "a corrente ficou dobrada onde a ancora a pos - o verbo ASSOU a pre-visualizacao no \
         documento, e nao ha' caminho de volta"
    );
    let _ = ombro;
}

/// ⭐⭐ **POSAR À MÃO UM OSSO SOB RESTRIÇÃO NÃO É ENGOLIDO** — e este gate DEFENDE a propriedade,
/// não cura um defeito.
///
/// ⛔⛔ **Ele nasceu de um diagnóstico MEU que a medição derrubou** (2026-09-07, a caçar o report
/// *«undo tem poucos passos»*). O raciocínio era: com a corrente assente a restrição não escreve,
/// logo não declara condução, logo o memo fica com o autorado velho e a fotografia repõe-o por cima
/// da pose que o artista acabou de fazer ⇒ nenhum passo nasce. Construí a cura (declarar a condução
/// **todo quadro**) e ela ficou verde… e o desenho ORIGINAL também. ⇒ **a cura era redundante e foi
/// revertida.**
///
/// ⭐ A razão está na regra da **outra mão** que o [`ph2d_preview_drive::PreviewDrive::driven`] já
/// tinha: perturbar um osso governado muda a solução, então a restrição **volta a escrever** no
/// quadro seguinte — e nessa escrita o `before` é a pose do artista, que passa a ser o autorado.
/// *O buraco fechava-se sozinho porque o motor reage ao que a outra mão fez.*
///
/// ⚠️ **E TRÊS fixturas não produziram o fenómeno antes desta**, cada uma por uma metade diferente:
/// a régua era `solve() == 0` (que conta escritas de 1 ULP, não movimento) · a cadeia do `braco()`
/// nunca assenta · e sem um arrasto ANTES não há entrada no memo para ficar velha. *A primeira
/// vermelha que vi era a régua errada, não o produto.*
///
/// ⚠️ **O gate mede o que a CAPTURA vê**, não o mundo: no mundo a pose do artista está lá — é na
/// fotografia que ela poderia desaparecer, e é a fotografia que vira o passo de undo.
#[test]
fn posing_a_governed_bone_by_hand_is_not_swallowed_by_the_ledger() {
    // ⚠️⚠️ **A CADEIA DA CENA REAL, e não o `braco()`.** A do `braco()` continua a escrever no
    // ÚLTIMO BIT do `f32` para sempre — e uma restrição que escreve todo quadro **declara** todo
    // quadro, o que faz o memo estar sempre fresco **por acidente**. *Uma fixtura que nunca assenta
    // não testa o que acontece quando ela assenta*, e foi a segunda a não produzir o fenómeno.
    let mut sim = SimWorld::default();
    let mut pai: Option<Entity> = None;
    let (mut ombro, mut punho) = (None, None);
    for i in 0..3 {
        let x = -8.2 + f64::from(i) * (6.4 / 3.0);
        let bits = crate::bone_gesture::create(&mut sim, pai, [x, 2.5], [x + 6.4 / 3.0, 2.5])
            .expect("osso");
        pai = Some(Entity::from_bits(bits));
        ombro = ombro.or(pai);
        punho = pai;
    }
    let punho = punho.expect("punho");
    let _ = ombro;
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    add(&mut sim, punho).expect("a ancora");
    let mut pv = PreviewDrive::default();
    // ⚠️⚠️ **A âncora tem de ESCREVER antes de assentar** — é a escrita que cria a entrada no memo,
    // e é uma entrada VELHA que engole a pose do artista. Sem este arrasto o memo está vazio, a
    // fotografia não tem o que repor e o gate fica verde sobre o defeito. *Terceira fixtura desta
    // sessão a não produzir o fenómeno, e as três falhavam por metades diferentes.*
    drag_anchor(&mut sim, punho, [-3.0, 6.0]);
    let mut anterior = ph2d_skeleton_live::skin_live::bone_segments(&sim);
    for _ in 0..200 {
        solve(&mut sim, &mut pv);
        let agora = ph2d_skeleton_live::skin_live::bone_segments(&sim);
        if agora == anterior {
            break;
        }
        anterior = agora;
    }
    assert_eq!(
        solve(&mut sim, &mut pv),
        0,
        "a fixtura nao produz o fenomeno: a corrente ainda escreve, entao ela declara todo quadro \
         por acidente e o memo nunca fica velho"
    );
    // O artista posa o ombro À MÃO.
    // ⚠️ O ombro do meio da cadeia governada — a âncora nasce em `chain = 2`, então ela manda nos
    // DOIS de baixo; o `ombro` aqui é a raiz, e o que se mede é o osso que ela de facto governa.
    let governado = *governed(&sim, punho, 2).first().expect("a corrente");
    sim.world_mut()
        .get_mut::<Transform>(governado)
        .expect("t")
        .rotation = 1.1;
    solve(&mut sim, &mut pv);
    // A fotografia: o autorado tem de ser o que o artista acabou de fazer.
    let vivo = pv.substitute_authored(&mut sim);
    let fotografado = sim.world().get::<Transform>(governado).expect("t").rotation;
    ph2d_preview_drive::PreviewDrive::restore_live(&mut sim, &vivo);
    assert!(
        (f64::from(fotografado) - 1.1).abs() < 1e-3,
        "a fotografia viu {fotografado} e o artista pos 1.1 - a pose dele foi ENGOLIDA pelo memo, \
         e nenhum passo de undo nasce dela"
    );
}

/// ⭐⭐⭐ **UMA CORRENTE ASSENTE CONTINUA CONDUZIDA — e o *Remove IK* ainda tem o autorado.**
///
/// ⛔⛔ **O irmão do report do dono de 2026-09-09** (*«Remove Smart Bone não devolve o objeto
/// animado à posição inicial»*), achado ao medir aquele: uma restrição é um condutor
/// **PERSISTENTE** — ela escreve todo quadro, e o output dela é constante assim que a corrente
/// assenta. O `solve_one` só declarava condução quando a rotação **mudava**, então a
/// [`ph2d_preview_drive::PreviewDrive::settle`] lia a constância como *«o motor largou»* e promovia
/// a pose da restrição a **documento**.
///
/// ⚠️ **O gate anterior (`removing_the_anchor_gives_the_authored_pose_back`) não o via**, e a razão
/// é a mesma dos outros quatro desta linha: ele chama `solve` **uma vez** e nunca a `settle`, que a
/// shell corre em todo quadro. *Uma fixtura que não corre o quadro do artista mede outro programa.*
///
/// ⚠️ E este defeito é MAIOR que o verbo: com a pose promovida a documento, ela entra no undo e no
/// save — que é exactamente o que o `preview_drive` existe para impedir.
#[test]
fn a_settled_chain_is_still_driven_and_remove_still_has_the_authored_pose() {
    let (mut sim, [ombro, cotovelo]) = braco();
    add(&mut sim, cotovelo).expect("a ancora");
    let autorada = ph2d_skeleton_live::skin_live::bone_segments(&sim);
    let mut pv = PreviewDrive::default();
    drag_anchor(&mut sim, cotovelo, [4.0, 9.0]);
    // ⚠️ Quadros a sério, com a `settle` de cada um — a corrente assenta ao fim de alguns.
    for _ in 0..30 {
        solve(&mut sim, &mut pv);
        pv.settle();
    }
    let assente = ph2d_skeleton_live::skin_live::bone_segments(&sim);
    assert!(
        assente != autorada,
        "a fixtura nao produz o fenomeno: a restricao nao dobrou a corrente"
    );
    assert!(
        pv.drives(ombro.to_bits()) || pv.drives(cotovelo.to_bits()),
        "o ledger largou a corrente ASSENTE -- a pose da restricao virou documento, entra no undo \
         e no save, e o Remove IK ja' nao tem o autorado para devolver"
    );

    assert!(remove(&mut sim, cotovelo, &mut pv), "havia ancora");
    assert_eq!(
        ph2d_skeleton_live::skin_live::bone_segments(&sim),
        autorada,
        "a corrente ficou dobrada onde a ancora a pos"
    );
}

//! Os gates da fase das CUTSCENES (TOP-20 #19).
//!
//! ⚠️ Eles correm a função do PRODUTO (`toca_as_cutscenes`) sobre um mundo e um `TimelineDoc`
//! construídos à mão — nenhum precisa de janela nem de GPU, e é por isso que eles são possíveis
//! aqui e não em `tests/it/` (a função é `pub(crate)`).

use super::toca_as_cutscenes;
use ph2d_anim::{AnimValue, Interp, RationalTime};
use ph2d_ecs::timer::{TimerRuntime, TimerState};
use ph2d_ecs::{Entity, SequencePlayer, Transform, World};
use ph2d_preview_drive::{Driven, Driver, PreviewDrive};
use ph2d_timeline::{PropKind, StackHost, StripSource, TimelineDoc};

/// Um mundo com UM objecto que toca a cutscene `nome`, e um documento cujo container `nome` leva
/// esse objecto de `y = 0` a `y = 10` em 2 s.
///
/// ⚠️ **A construção é a do `nesting_leads`**, palavra por palavra na parte que interessa: um
/// container é uma pilha com uma lane e uma strip que aponta a um clip keyado.
fn cena(nome: &str, us: u64, a_correr: bool) -> (World, TimelineDoc, Entity) {
    let mut world = World::new();
    let e = world
        .spawn((
            Transform::default(),
            SequencePlayer {
                container: nome.to_owned(),
            },
            TimerRuntime(vec![TimerState {
                elapsed_us: us,
                running: a_correr,
            }]),
        ))
        .id();
    let bits = e.to_bits();

    let mut doc = TimelineDoc::new();
    doc.upsert_key(
        bits,
        PropKind::TranslationY,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(0.0),
        Interp::Linear,
    );
    doc.upsert_key(
        bits,
        PropKind::TranslationY,
        RationalTime::from_seconds(2.0),
        AnimValue::Float(10.0),
        Interp::Linear,
    );
    // ⚠️⚠️ **Um container ISCA antes do verdadeiro, e ele é a diferença entre um gate e um
    // teatro:** com um só container o alvo é sempre o índice `0`, e a mutação *«toca sempre o
    // container 0»* fica **inobservável** — ela SOBREVIVEU à 1.ª redacção destes gates. A isca é
    // vazia, logo tocá-la não move nada, e é isso que faz a troca de índice sangrar.
    doc.add_container("Isca".to_owned());
    let c = doc.add_container(nome.to_owned());
    let host = StackHost::Container(c);
    let lane = doc.add_lane_in(host, "Body".to_owned()).expect("1.ª lane");
    doc.add_strip_to(host, lane, StripSource::Clip(0), 0.0, 2.0)
        .expect("a strip cabe");
    (world, doc, e)
}

fn y(world: &World, e: Entity) -> f32 {
    world.get::<Transform>(e).expect("tem pose").translation.y
}

/// ⭐⭐⭐ **Uma cutscene a correr MOVE o objecto, no instante do relógio dele.**
///
/// ⚠️ Com o CONTROLO ao lado — o mesmo mundo com o relógio PARADO não se mexe. Sem ele, a metade
/// positiva ficaria verde com uma fase que escrevesse sempre.
#[test]
fn uma_cutscene_a_correr_move_o_objecto_e_uma_parada_nao() {
    let mut drive = PreviewDrive::default();
    let (mut w, mut doc, e) = cena("Porta", 1_000_000, true);
    assert_eq!(
        toca_as_cutscenes(&mut w, &mut doc, &mut drive, |_| false),
        1
    );
    let meio = y(&w, e);
    assert!(
        (meio - 5.0).abs() < 1e-3,
        "a meio de 2 s a curva vale 5, e leu {meio}"
    );

    let mut drive = PreviewDrive::default();
    let (mut w, mut doc, e) = cena("Porta", 1_000_000, false);
    assert_eq!(
        toca_as_cutscenes(&mut w, &mut doc, &mut drive, |_| false),
        0
    );
    assert!(
        y(&w, e).abs() < 1e-6,
        "com o relógio parado a cutscene não escreve nada"
    );
}

/// ⭐⭐⭐ **O que a cutscene escreve NÃO é documento** — o ledger guarda o autorado, e é ele que o
/// `Ctrl+Z` devolve.
///
/// ⛔ Sem esta entrada, *«a cutscene moveu o herói»* seria um passo de undo **por quadro**.
#[test]
fn o_que_a_cutscene_escreve_passa_pelo_ledger() {
    let mut drive = PreviewDrive::default();
    let (mut w, mut doc, e) = cena("Porta", 1_000_000, true);
    toca_as_cutscenes(&mut w, &mut doc, &mut drive, |_| false);
    let Some(Driven::SolverPose(autorado)) = drive.authored(e.to_bits(), Driver::SolverPose) else {
        panic!("o ledger não guardou a pose autorada deste objecto");
    };
    assert!(
        autorado.translation.y.abs() < 1e-6,
        "o autorado é a pose de ANTES da cutscene (y = 0), e leu {}",
        autorado.translation.y
    );
}

/// ⛔ **Um nome que não resolve não toca nada** — e o gate é sobre a FASE, não só sobre a lei pura:
/// aqui prova-se que ela não cai no container `0`, que existe e moveria o objecto.
#[test]
fn um_nome_errado_nao_toca_o_primeiro_container() {
    let mut drive = PreviewDrive::default();
    let (mut w, mut doc, e) = cena("Porta", 1_000_000, true);
    // O objecto passa a pedir uma cutscene que não existe; o container «Porta» continua lá.
    w.entity_mut(e).insert(SequencePlayer {
        container: "Ausente".to_owned(),
    });
    assert_eq!(
        toca_as_cutscenes(&mut w, &mut doc, &mut drive, |_| false),
        0
    );
    assert!(y(&w, e).abs() < 1e-6, "nada foi tocado");
}

/// ⚠️ **Sem cutscene nenhuma a fase é um no-op barato** — nem censo tira. É o caso de toda cena
/// que já existe, e o gate prende-o para que ninguém o troque por um censo por quadro.
#[test]
fn sem_cutscenes_a_fase_nao_mexe_no_ledger() {
    let mut drive = PreviewDrive::default();
    let (mut w, mut doc, _) = cena("Porta", 1_000_000, false);
    assert_eq!(
        toca_as_cutscenes(&mut w, &mut doc, &mut drive, |_| false),
        0
    );
    assert!(
        drive.is_empty(),
        "sem cutscenes o ledger não recebe uma única entrada"
    );
}

/// ⭐⭐⭐ **A vista de EDIÇÃO não deixa uma cutscene correr, e a de CENA deixa.**
///
/// ⚠️ **A tabela inteira, e não um caso:** as duas vistas de edição dizem NÃO por razões
/// diferentes (uma sola um clip, a outra o interior de um container), e um gate que medisse só uma
/// ficaria verde no dia em que a outra deixasse de ser lida.
///
/// **Mutação que deve sangrar:** trocar o `&&` por `||`, ou negar qualquer um dos dois.
#[test]
fn so_a_vista_da_cena_deixa_uma_cutscene_correr() {
    use super::a_vista_deixa_correr as porta;
    assert!(porta(false, None), "Arrange: a vista da CENA deixa correr");
    assert!(!porta(true, None), "Keys sola o clip que o animador edita");
    assert!(
        !porta(false, Some(0)),
        "dentro de um container sola-se o interior dele"
    );
    assert!(
        !porta(true, Some(0)),
        "as duas ao mesmo tempo continuam a dizer não"
    );
}

/// ⛔⛔ **E o `timeline_bridge` decide PELA PORTA** — a metade que faz dela uma porta e não uma
/// função solta.
///
/// ⚠️ **Sem isto, o painel e o motor podiam divergir em silêncio:** o instantâneo do Inspector lê a
/// mesma porta para DIZER ao artista porque é que a cutscene dele está parada, e uma condição
/// escrita à mão no `bridge` deixaria o painel a prometer uma coisa e o motor a fazer outra —
/// *duas portas paradas com todos os campos certos*, que é o modo de falha que esta wave já pagou
/// uma vez.
///
/// **Mutação que deve sangrar:** reescrever a condição do `bridge` à mão.
#[test]
fn o_bridge_pergunta_a_porta_e_nao_reescreve_a_condicao() {
    let fonte = include_str!("timeline_bridge.rs");
    assert!(
        fonte.contains("fase_sequences::a_vista_deixa_correr("),
        "o `timeline_bridge` deixou de perguntar à porta — o painel e o motor podem divergir"
    );
    assert!(
        !fonte.contains("container.is_none() && !solo"),
        "a condição voltou a estar escrita à mão ao lado da porta que a responde"
    );
}

/// ⭐⭐ **E o instantâneo do Inspector lê a MESMA porta** — o segundo leitor, que é o que a torna
/// uma porta.
///
/// **Mutação que deve sangrar:** o publicador passar um `true` cravado em vez de perguntar.
#[test]
fn o_instantaneo_do_inspector_le_a_mesma_porta() {
    let fonte = include_str!("fase_snapshots_publish.rs");
    assert!(
        fonte.contains("fase_sequences::a_vista_deixa_correr("),
        "o publicador deixou de ler a porta — o aviso do painel passa a ser um palpite"
    );
}

//! Gates da cena `=44` — **a TABELA e a SEMENTE**.
//!
//! Eles medem a GEOMETRIA que a cena cozinha, não a intenção com que ela foi
//! escrita: uma fileira que anuncia doze passos tem de desenhar um período de
//! doze, e as duas fileiras que a mensagem chama de gêmeas têm de sair com a
//! **mesma silhueta** — senão a cena afirma um defeito que ela própria não
//! reproduz, e o smoke julga outra coisa.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

/// Cozinha a cena e devolve o `Y` de cada fileira, na ordem das bandas.
fn rows() -> Vec<Vec<f32>> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).unwrap();
    let mut doc = MotionDoc::default();
    let sinks = build_table_seed_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), BANDS, "uma fileira por banda");
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(|&s| {
            let out = cook.cook(&doc.graph, &reg, s, 0.0).expect("cook");
            match out[0].as_stream().get("P") {
                Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
                _ => panic!("sem coluna P"),
            }
        })
        .collect()
}

/// A SILHUETA de uma fileira — ela menos a própria linha de base.
///
/// ⚠️ **Comparar Y CRU entre fileiras mede o LAYOUT, não o campo**, e a primeira
/// versão do gate dos gêmeos fez exactamente isso: as duas saíam a diferir por
/// `1,05` em todo elemento, que é o `BAND_GAP`. É a terceira vez que este repo
/// paga a lição (o `1,15` do grupo B, o `offset_y` do `Range` no grupo C), e é a
/// mesma cura — o que a mensagem manda o olho comparar é a **forma**, então é a
/// forma que o oráculo tem de ler.
fn shape(v: &[f32]) -> Vec<f32> {
    let lo = v.iter().fold(f32::INFINITY, |m, x| m.min(*x));
    v.iter().map(|x| x - lo).collect()
}

/// Quanto duas silhuetas divergem no pior elemento.
fn worst(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f32::max)
}

/// A barra de *"a MESMA silhueta"*, e ela **não é zero de propósito**.
///
/// ⚠️ Os dois campos são idênticos ao bit no `v`, mas a normalização os subtrai
/// de bases a alturas diferentes (1,05 de distância), e `x − lo` em `f32`
/// arredonda diferente conforme a magnitude — medido, o resíduo é da ordem de
/// **1e-7**, o ULP daquela faixa. A barra fica **cem vezes acima do ruído e
/// cinco ordens abaixo da amplitude** (~0,84) que as fileiras percorrem, então
/// ela não pode ser satisfeita por dois campos que de facto diferem: os que
/// diferem, diferem por ~0,5.
const SAME_SHAPE: f32 = 1e-5;

/// O período de uma fileira: o menor `p` com que ela se repete exatamente.
///
/// ⚠️ Medido dos PIXELS cozidos, não do param — é isso que o torna oráculo em
/// vez de espelho do que a cena pediu.
fn period(v: &[f32]) -> usize {
    let tol = 1e-4;
    (1..=v.len() / 2)
        .find(|p| {
            v.iter()
                .zip(v.iter().skip(*p))
                .all(|(a, b)| (a - b).abs() < tol)
        })
        .unwrap_or(v.len())
}

/// **A tabela desenha o período que a mensagem anuncia**, e o legado o dele.
///
/// ⚠️ Se a tabela não chegasse ao cozido, as DUAS fileiras teriam período três e
/// a cena anunciaria uma largura que não desenha.
#[test]
fn the_table_row_draws_a_longer_period_than_the_eight_slots() {
    let r = rows();
    let legacy = period(&r[0]);
    let table = period(&r[1]);
    assert_eq!(
        legacy, LEGACY_STEPS as usize,
        "a fileira 1 cicla nos oito slots"
    );
    assert_eq!(
        table, TABLE_STEPS,
        "a fileira 2 cicla na TABELA, acima do teto de oito"
    );
    assert!(table > 8, "e o numero que prova a wave e' maior que OITO");
}

/// **As duas fileiras que a mensagem chama de GÊMEAS são idênticas** — o defeito
/// tem de estar na tela, senão a metade seguinte não significa nada.
#[test]
fn the_two_off_rows_really_are_twins() {
    let r = rows();
    let d = worst(&shape(&r[2]), &shape(&r[3]));
    assert!(
        d < SAME_SHAPE,
        "bandas 3 e 4 sao o MESMO campo (pior |d| = {d:e})"
    );
}

/// **E as duas com o toggle NÃO são** — nem por acaso em alguns elementos: quase
/// todo elemento tem de diferir.
#[test]
fn the_two_on_rows_are_not_twins() {
    let r = rows();
    let (a, b) = (shape(&r[4]), shape(&r[5]));
    let d = worst(&a, &b);
    assert!(
        d > 0.1,
        "bandas 5 e 6 diferem, e por MUITO mais que a barra (pior |d| = {d:e})"
    );
    let same = a
        .iter()
        .zip(&b)
        .filter(|(x, y)| (*x - *y).abs() < SAME_SHAPE)
        .count();
    assert!(
        same < 3,
        "quase todo elemento difere (iguais: {same} de {})",
        a.len()
    );
}

/// **Toda fileira tem FAIXA** — uma fileira chata seria um gráfico de nada, e as
/// comparações de silhueta acima passariam por vácuo sobre duas linhas retas.
#[test]
fn every_row_has_swing() {
    for (i, v) in rows().iter().enumerate() {
        let lo = v.iter().fold(f32::INFINITY, |m, x| m.min(*x));
        let hi = v.iter().fold(f32::NEG_INFINITY, |m, x| m.max(*x));
        assert!(hi - lo > 0.3, "fileira {} chata ({lo:.3}..{hi:.3})", i + 1);
        assert_eq!(v.len(), COLS as usize, "fileira {}", i + 1);
    }
}

/// **A mensagem tem uma linha por fileira** — o `PARE` do anúncio conta esta
/// lista, então ela não pode divergir do número de bandas.
#[test]
fn the_announcement_names_every_row() {
    assert_eq!(BAND_LABELS.len(), BANDS);
    assert_eq!(LANES.len(), BANDS);
}

/// **A tabela que a cena escreve tem os passos que ela anuncia** — o texto é
/// gerado, então o gate confere que a gramática do nó o lê de volta com o mesmo
/// comprimento.
#[test]
fn the_generated_table_parses_back_to_the_announced_length() {
    let parsed = ph2d_node_value_pattern::table::parse(&table_text());
    assert_eq!(parsed.len(), TABLE_STEPS);
    // E é uma rampa: o primeiro e o último são os extremos.
    assert!((parsed[0] - 0.0).abs() < 1e-6);
    assert!((parsed[TABLE_STEPS - 1] - 1.0).abs() < 1e-6);
}

//! Os gates da cena `=62` — os dois defeitos de junção.
//!
//! ⚠️ **Cada par tem de SEPARAR**, e é isso que se prova: uma cena cujas duas bandas saem
//! iguais diz *"o knob funciona"* sobre um número que o cozimento ignorou, e é a única coisa
//! pior que não ter cena.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// As quatro bandas, cada uma pelas duas colunas que a cena julga: a ESCALA (o par do
/// carimbo) e a TINTA (o par da junção). Nomeado porque a tupla crua dispara o
/// `type_complexity` do clippy.
type Bands = (Vec<Vec<[f32; 2]>>, Vec<Vec<[f32; 4]>>);

/// Coze a cena e devolve as quatro bandas.
fn bands() -> Bands {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_join_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 4, "dois pares");
    doc.graph.validate(&reg).expect("bem-tipado");
    let mut cook = Cook::new();
    let (mut sizes, mut tints) = (Vec::new(), Vec::new());
    for s in &sinks {
        let v = cook.cook(&doc.graph, &reg, *s, 0.0).expect("a banda coze");
        let st = v[0].as_stream();
        sizes.push(match st.get("size") {
            Some(Column::Vec2(p)) => p.clone(),
            _ => Vec::new(),
        });
        tints.push(match st.get("tint") {
            Some(Column::Vec4(p)) => p.clone(),
            _ => Vec::new(),
        });
    }
    (sizes, tints)
}

/// **O PAR DO CARIMBO SEPARA** — a banda 1 sai com todos do mesmo tamanho, a 2 com o tamanho
/// de cada ponto.
#[test]
fn the_stamp_pair_separates_on_the_point_scale() {
    let (sizes, _) = bands();
    assert!(
        sizes[0].is_empty(),
        "com Point Scale 0 o carimbo não emite `size` — é a forma que manda: {:?}",
        sizes[0]
    );
    let on = &sizes[1];
    assert_eq!(on.len(), STAMPS as usize, "um carimbo por ponto");
    let (lo, hi) = (
        on.first().expect("primeiro")[0],
        on.last().expect("último")[0],
    );
    assert!(
        hi > lo + 1.0,
        "a escala tem de CRESCER ao longo da fileira: {lo:.2} -> {hi:.2}"
    );
}

/// **O PAR DA JUNÇÃO SEPARA** — e o oráculo é onde a cor REINICIA.
///
/// ⚠️ **A régua não é "as duas listas diferem"**, que passaria por qualquer motivo: é que a
/// banda mentirosa tem um SALTO PARA TRÁS no meio (a 10ª peça volta ao começo do degradê) e a
/// honesta não tem nenhum. É a assinatura exacta do `Index` que reinicia.
#[test]
fn the_join_pair_separates_on_where_the_gradient_restarts() {
    let (_, tints) = bands();
    let backsteps = |t: &Vec<[f32; 4]>| t.windows(2).filter(|w| w[1][0] < w[0][0] - 0.05).count();
    let total = (LEFT * LEFT + RIGHT * RIGHT) as usize;
    assert_eq!(tints[2].len(), total, "9 + 4 = 13 peças");
    assert_eq!(tints[3].len(), total);
    assert_eq!(
        backsteps(&tints[2]),
        1,
        "com Reindex off a cor volta atrás UMA vez — no ponto em que a 2ª grelha começa"
    );
    assert_eq!(
        backsteps(&tints[3]),
        0,
        "com Reindex on o degradê corre uma vez só"
    );
}

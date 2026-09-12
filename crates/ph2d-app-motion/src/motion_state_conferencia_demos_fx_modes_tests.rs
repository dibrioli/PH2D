//! Gates da cena `=84` — **o que o efeito não sabia fazer** (doc 89, folha 11).
//!
//! ⚠️ **O oráculo de cada linha é o MECANISMO, não «a figura mudou».** A sombra prova-se pela
//! COLUNA `blend` (é ela que o lowering lê para escolher o pipeline), e a lente pelo elemento que
//! fica LIMPO — uma régua que medisse só a excursão passaria numa implementação que apenas
//! escalasse tudo.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// O stream de uma célula, já cozido.
fn cell(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> Stream {
    let mut c = Cook::new();
    let out = c.cook(&doc.graph, reg, sink, 0.0).expect("coze");
    match out.first() {
        Some(CookValue::Instances(s)) => s.clone(),
        _ => panic!("a celula emite instancias"),
    }
}

/// **A CENA CONSTRÓI TODAS AS CÉLULAS** — duas por linha da [`ROWS_TABLE`].
#[test]
fn the_fx_scene_builds_every_cell() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_fx_demo_document(&mut doc, &reg).expect("a cena constroi");
    assert_eq!(sinks.len(), ROWS_TABLE.len() * 2, "duas celulas por linha");
    let (n, count) = authored();
    assert_eq!(n, ROWS_TABLE.len(), "o anuncio conta a mesma tabela");
    assert!(count >= 8.0, "uma fileira precisa de pecas que cheguem");
    for sink in &sinks {
        assert!(cell(&doc, &reg, *sink).count() > 0, "toda celula desenha");
    }
}

/// **SÓ A METADE CURADA DA SOMBRA PEDE UM MODO** — e ela pede-o só nos FANTASMAS.
///
/// ⚠️ **A coluna é o oráculo, e não os pixels**: `Multiply` e `Over` desenham igual sobre o
/// vazio, então uma régua de imagem precisaria do fundo e mediria as duas coisas ao mesmo tempo.
/// A coluna `blend` é o que o lowering lê para escolher o pipeline — é ali que a cura vive.
#[test]
fn only_the_cured_shadow_asks_for_a_blend_and_only_on_its_ghosts() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_fx_demo_document(&mut doc, &reg).expect("constroi");
    let k = ROWS_TABLE
        .iter()
        .position(|r| r.case == Case::Shadow)
        .expect("a linha da sombra existe");
    let blend_of = |s: &Stream| match s.get("blend") {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    };
    let plain = cell(&doc, &reg, sinks[k * 2]);
    assert!(
        blend_of(&plain).is_none_or(|v| v.iter().all(|x| *x == 0.0)),
        "a metade de sempre nao escolhe modo nenhum"
    );
    let cured = cell(&doc, &reg, sinks[k * 2 + 1]);
    let col = blend_of(&cured).expect("a metade curada escreve a coluna");
    let ghosts = col.iter().filter(|v| **v > 0.5).count();
    assert!(ghosts > 0, "ha' fantasmas a multiplicar");
    assert!(
        ghosts < col.len(),
        "e nem toda linha o faz -- a barra e as pecas ficam no modo do sink ({ghosts} de {})",
        col.len()
    );
    // ⚠️ E a BARRA continua no sink: ela é o fundo, e um fundo que multiplicasse não teria o que
    // escurecer. Ela é a PRIMEIRA linha do stream (a ordem de desenho).
    assert_eq!(col[0], 0.0, "a barra do fundo nao leva o modo da sombra");
}

/// **A LENTE MOVE O PONTO LIMPO, E ALARGA-O.**
///
/// ⚠️ Duas afirmações, e as duas são precisas: o eixo deslocado muda **qual** peça não tem
/// franja, e o raio limpo faz com que **mais de uma** deixe de a ter. Medir só a primeira
/// passaria com o `start` morto; medir só a segunda passaria com o eixo morto.
#[test]
fn the_lens_moves_the_clean_spot_and_widens_it() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_fx_demo_document(&mut doc, &reg).expect("constroi");
    let k = ROWS_TABLE
        .iter()
        .position(|r| r.case == Case::Lens)
        .expect("a linha da lente existe");
    // A franja de cada peça: quanto o fantasma R se afastou dela.
    let spread = |s: &Stream| -> Vec<f32> {
        let Some(Column::Vec2(p)) = s.get("P") else {
            panic!("P")
        };
        let n = p.len() / 3;
        (0..n)
            .map(|i| {
                let (g, e) = (p[i], p[2 * n + i]);
                (g[0] - e[0]).hypot(g[1] - e[1])
            })
            .collect()
    };
    let cleanest = |v: &[f32]| {
        v.iter()
            .enumerate()
            .min_by(|a, b| a.1.partial_cmp(b.1).expect("finito"))
            .expect("nao vazio")
            .0
    };
    let plain = spread(&cell(&doc, &reg, sinks[k * 2]));
    let cured = spread(&cell(&doc, &reg, sinks[k * 2 + 1]));
    assert_ne!(
        cleanest(&plain),
        cleanest(&cured),
        "o eixo deslocado tem de mudar QUAL peca fica limpa ({plain:?} vs {cured:?})"
    );
    let untouched = |v: &[f32]| v.iter().filter(|x| **x == 0.0).count();
    assert!(
        untouched(&cured) > untouched(&plain),
        "e o raio limpo tem de deixar MAIS pecas sem franja ({} vs {})",
        untouched(&plain),
        untouched(&cured)
    );
}

/// **A LINHA DA ALFA VARIA A ALFA E MAIS NADA.**
///
/// ⚠️ Ela existe para tornar julgável a resposta do `Multiply` à alfa, então o que ela tem de
/// provar é precisamente que as duas metades **são iguais em tudo o resto**: mesmo modo nos
/// fantasmas, mesma contagem de linhas, e só a alfa do fantasma diferente. Uma régua que só
/// medisse "as alfas diferem" passaria numa cena em que uma metade também tivesse mudado de
/// modo — e aí o par deixaria de dizer o que anuncia.
#[test]
fn the_alpha_row_varies_the_alpha_and_nothing_else() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_fx_demo_document(&mut doc, &reg).expect("constroi");
    let k = ROWS_TABLE
        .iter()
        .position(|r| r.case == Case::ShadowAlpha)
        .expect("a linha da alfa existe");
    let halves = [
        cell(&doc, &reg, sinks[k * 2]),
        cell(&doc, &reg, sinks[k * 2 + 1]),
    ];
    assert_eq!(
        halves[0].count(),
        halves[1].count(),
        "as duas metades desenham o mesmo numero de linhas"
    );
    // O modo, por linha: as duas metades TÊM de pedir o mesmo.
    let modes = |s: &Stream| match s.get("blend") {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("a linha da alfa escreve sempre a coluna blend"),
    };
    assert_eq!(
        modes(&halves[0]),
        modes(&halves[1]),
        "so' a alfa muda -- o modo e' o mesmo nas duas metades"
    );
    assert!(
        modes(&halves[0]).contains(&MULTIPLY),
        "e o modo pedido e' o Multiply"
    );
    // A alfa do FANTASMA: a barra é a linha 0, os fantasmas vêm a seguir.
    let ghost_alpha = |s: &Stream| match s.get("tint") {
        Some(Column::Vec4(t)) => t[1][3],
        _ => panic!("tint"),
    };
    let (faint, strong) = (ghost_alpha(&halves[0]), ghost_alpha(&halves[1]));
    assert!(
        strong > faint + 0.3,
        "a metade forte tem de ser MUITO mais opaca ({faint} vs {strong})"
    );
}

/// **NENHUMA CÉLULA INVADE A VIZINHA** — a lei de layout das cenas irmãs.
#[test]
fn no_cell_climbs_into_its_neighbour() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_fx_demo_document(&mut doc, &reg).expect("constroi");
    for (k, sink) in sinks.iter().enumerate() {
        let s = cell(&doc, &reg, *sink);
        let Some(Column::Vec2(p)) = s.get("P") else {
            panic!("P")
        };
        let row = k / 2;
        let half = k % 2;
        let cx = if half == 0 { -COL_X } else { COL_X };
        let cy = (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - row as f32 * ROW_GAP;
        let (mut mx, mut my) = (0.0f32, 0.0f32);
        for q in p {
            mx = mx.max((q[0] - cx).abs());
            my = my.max((q[1] - cy).abs());
        }
        assert!(
            my < ROW_GAP * 0.5,
            "celula {k} sobe {my}, meia linha e' {}",
            ROW_GAP * 0.5
        );
        assert!(
            mx < COL_X,
            "celula {k} alarga {mx}, a coluna vive a {COL_X}"
        );
    }
}

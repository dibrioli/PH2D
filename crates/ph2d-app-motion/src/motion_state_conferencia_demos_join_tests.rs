//! Os gates da cena `=62` — os dois defeitos de junção.
//!
//! ⚠️ **Cada par tem de SEPARAR**, e é isso que se prova: uma cena cujas duas bandas saem
//! iguais diz *"o knob funciona"* sobre um número que o cozimento ignorou, e é a única coisa
//! pior que não ter cena.

//! ⚠️ **O cook é o da SHELL, não um `Cook` nu** (desde 2026-09-06): a forma da banda do carimbo
//! é um `source.shape`, que lê um EXTERNAL que a shell publica — num cook virgem ele emite
//! **zero**, e as bandas sairiam vazias com os gates a dizer que a cena não monta.

use super::*;
use crate::motion_state::MotionState;
use ph2d_nodegraph::attr::Column;

/// As quatro bandas, cada uma pelas duas colunas que a cena julga: a ESCALA (o par do
/// carimbo) e a TINTA (o par da junção). Nomeado porque a tupla crua dispara o
/// `type_complexity` do clippy.
type Bands = (Vec<Vec<[f32; 2]>>, Vec<Vec<[f32; 4]>>);

/// Coze a cena e devolve as quatro bandas.
fn bands() -> Bands {
    let mut state = MotionState::new();
    assert!(
        state.doc.graph.nodes().is_empty(),
        "desligue o PH2D_GPU_COOK_DEMO: o `MotionState::new` semearia outra cena"
    );
    let sinks = build_join_demo_document(&mut state.doc, &state.registry).expect("a cena monta");
    assert_eq!(sinks.len(), 4, "dois pares");
    state
        .doc
        .graph
        .validate(&state.registry)
        .expect("bem-tipado");
    crate::motion_shape_gen::publish(&mut state, 0.0);
    let (mut sizes, mut tints) = (Vec::new(), Vec::new());
    for s in &sinks {
        let v = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, *s, 0.0)
            .expect("a banda coze");
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
    // ⚠️ **Esta afirmação MUDOU em 2026-09-06, e o produto é que mudou por baixo dela.** Ela
    // dizia *«com Point Scale 0 o carimbo não emite `size`»* — verdade enquanto a forma era uma
    // grelha nua, que não tem `size` nenhum. Uma FORMA a sério tem, então o que se afirma agora
    // é o que a cena de facto ensina: **todos do MESMO tamanho, o da forma**.
    let off = &sizes[0];
    assert_eq!(off.len(), STAMPS as usize, "um carimbo por ponto");
    assert!(
        off.iter().all(|s| (s[0] - off[0][0]).abs() < 1e-4),
        "com Point Scale 0 as copias saem TODAS do tamanho da forma: {off:?}"
    );
    let on = &sizes[1];
    assert_eq!(on.len(), STAMPS as usize, "um carimbo por ponto");
    assert!(
        on.iter().any(|s| (s[0] - off[0][0]).abs() > 1e-3),
        "com Point Scale 1 elas deixam de ser todas iguais"
    );
    let (lo, hi) = (
        on.first().expect("primeiro")[0],
        on.last().expect("último")[0],
    );
    assert!(
        hi > lo + SHAPE_SIZE,
        "a escala tem de CRESCER ao longo da fileira: {lo:.2} -> {hi:.2}"
    );
    // ⚠️ **E NENHUMA cópia sai invisível** — com o drive em `Set` a primeira recebia `size = 0`
    // (a rampa começa em zero) e a fileira mostrava seis peças de sete.
    assert!(
        on.iter().all(|s| s[0] > 1e-3),
        "toda copia tem de ter tamanho: {on:?}"
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

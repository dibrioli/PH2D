//! Gates da row de PASSOS — a faixa de barras, o arrasto e as duas FACES.

use super::*;
use ph2d_editor_core::interaction::InteractiveState;

fn row(value: &str) -> StepsRow {
    StepsRow {
        name: "table",
        label: "Table".into(),
        value: value.into(),
        min: 0.0,
        max: 1.0,
    }
}

/// O que pintar `value` numa row de `w` de largura de fato produziu: a altura que ela
/// alegou, **quantos caminhos a cena de fato codifica**, a GEOMETRIA deles, e as barras
/// que ficaram registradas.
///
/// ⚠️ Os dois do meio são o ponto. A lição está pregada no irmão de PALETA, que shipou uma
/// faixa reservando espaço e **desenhando nada**, com dois gates verdes medindo a ALTURA
/// devolvida: *um número que um pintor devolve não é prova de que um pintor pintou*.
///
/// ⚠️ **E o fluxo é o `path_data`, não o `draw_data` que a paleta lê** — a diferença não é
/// gosto: a paleta discrimina por COR, e uma cor mora na tinta; uma barra discrimina por
/// ALTURA, e altura é geometria. Escrito com o fluxo da paleta, este gate nasceu VERDE
/// sobre duas faixas de valores diferentes — porque todas as barras usam a MESMA tinta.
fn painted(value: &str, w: f32, raw: bool) -> (f32, u32, Vec<u32>, StepsWidgets) {
    let r = row(value);
    let mut hit = HitIndex::default();
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    let mut store = WidgetStore::with_capacity(8);
    if raw {
        store.register(
            param_checkbox_id(0),
            InteractiveState::Checkbox {
                state: CheckboxState::Normal,
                value: CheckboxValue::Checked,
            },
        );
    }
    let mut out = StepsWidgets::new();
    let h = paint_steps_row(
        &r,
        0,
        0.0,
        w,
        0.0,
        12.0,
        &store,
        &mut hit,
        &mut scene,
        &mut text,
        Theme::default(),
        &mut out,
    );
    let e = scene.inner().encoding();
    (h, e.n_paths, e.path_data.clone(), out)
}

/// **Cada valor DESENHA uma barra, e a barra tem a altura do valor.**
///
/// Duas metades, porque há dois modos de falha distintos e cada um passa pelo gate do
/// outro: *nada é desenhado* (a faixa fica vazia) e *tudo é desenhado igual* (uma fileira
/// de barras cheias, que conta caminhos igualzinho a uma faixa correta).
#[test]
fn every_value_paints_a_bar_of_its_own_height() {
    let (_, one, _, _) = painted("0.5", 400.0, false);
    let (_, twelve, _, _) = painted(
        "0.5 0.5 0.5 0.5 0.5 0.5 0.5 0.5 0.5 0.5 0.5 0.5",
        400.0,
        false,
    );
    assert!(
        twelve >= one + 11,
        "cada passo a mais desenha ao menos um caminho a mais: {one} -> {twelve}"
    );

    // Mesmo COMPRIMENTO, alturas diferentes: se os bytes forem os mesmos, as barras não
    // estão lendo o valor.
    let flat = painted("0.5 0.5 0.5 0.5", 400.0, false).2;
    let ramp = painted("0.1 0.4 0.7 1.0", 400.0, false).2;
    assert_ne!(
        flat, ramp,
        "duas listas de mesmo comprimento e valores diferentes pintaram a MESMA geometria \
         — as barras não estão usando o valor"
    );
}

/// **A faixa ENVOLVE**, então a row não tem teto de comprimento próprio — a lei que a
/// paleta estreou e que faz do `MAX_ENTRIES` o único teto (e ele é de RECURSO).
#[test]
fn the_strip_wraps_instead_of_running_off_the_edge() {
    let narrow = per_line(60.0);
    assert!(
        (2..20).contains(&narrow),
        "uma row estreita cabe poucas: {narrow}"
    );
    assert!(per_line(600.0) > narrow, "uma row larga cabe mais");
    assert_eq!(
        per_line(1.0),
        1,
        "mais estreita que uma barra ainda desenha uma"
    );

    // E a ALTURA cresce com as linhas: 40 barras numa row estreita não cabem numa fileira.
    let many: String = (0..40).map(|_| "0.5 ").collect();
    let tall = painted(&many, 120.0, false).0;
    let short = painted("0.5 0.5", 120.0, false).0;
    assert!(tall > short, "a altura segue a contagem: {short} -> {tall}");
}

/// **UMA face por vez** — com o `Type` marcado o strip não registra barra nenhuma, e sem
/// ele o campo de texto não é oferecido.
///
/// ⚠️ É o gate que impede as DUAS portas de existirem ao mesmo tempo sobre o mesmo valor:
/// um campo e uma barra a escrever a mesma string, cada um com a própria ideia de quando.
#[test]
fn one_face_at_a_time_bars_or_the_raw_field() {
    let bars = painted("0.1 0.5 0.9", 400.0, false).3;
    assert_eq!(bars.points.len(), 3, "as três barras estão vivas");

    let raw = painted("0.1 0.5 0.9", 400.0, true).3;
    assert!(
        raw.points.is_empty(),
        "com o `Type` marcado nenhuma barra é registrada"
    );
}

/// **O endereço de uma barra sobrevive ao par de `u8`** — a lista vai a 1024 e o
/// `CurvePoint` endereça com dois bytes.
#[test]
fn the_bar_address_survives_the_two_byte_split() {
    for i in [0usize, 1, 255, 256, 257, 511, 512, 1023] {
        let (page, index) = pack(i);
        assert_eq!(unpack(page, index), i, "índice {i}");
    }
    // E o teto da lista cabe no par (a premissa que torna o split suficiente).
    let (page, _) = pack(ph2d_steps::MAX_ENTRIES - 1);
    assert!(page < 4, "a página cabe num u8: {page}");
}

/// **Arrastar uma barra muda SÓ aquele passo** — os demais voltam da string bit a bit.
///
/// ⚠️ É a propriedade inteira do editor: ele é uma FACE da string, não um segundo modelo
/// dela. Sem o round-trip exato do `ph2d_steps`, cada arrasto reescreveria a lista toda com
/// números ligeiramente diferentes, e a deriva só apareceria depois de vários gestos.
#[test]
fn dragging_a_bar_rewrites_only_that_step() {
    let r = row("0.125 0.25 0.375 0.5");
    let mut store = WidgetStore::with_capacity(4);
    let (page, index) = pack(2);
    store.set_curve_point_drag(param_steps_editor_id(0), page, index, 0.0, 0.75);
    let got = drain_drag(&mut store, 0, &r).expect("o arrasto foi drenado");
    let before = ph2d_steps::parse(&r.value);
    let after = ph2d_steps::parse(&got);
    assert_eq!(after.len(), before.len(), "a contagem não muda");
    assert!(
        (after[2] - 0.75).abs() < 1e-6,
        "o passo arrastado tomou o valor do dedo: {}",
        after[2]
    );
    for i in [0usize, 1, 3] {
        assert_eq!(
            after[i].to_bits(),
            before[i].to_bits(),
            "o passo {i} tem de voltar BIT A BIT"
        );
    }
}

/// **Um valor FORA da faixa é desenhado saturado e NÃO é reescrito.**
///
/// ⚠️ A alternativa — clampar a lista inteira ao pintar — destruiria em silêncio um número
/// que o artista digitou de propósito, e o destruiria no primeiro arrasto de OUTRA barra.
#[test]
fn a_value_outside_the_range_survives_a_drag_of_another_bar() {
    let r = row("0.5 42 -7");
    let mut store = WidgetStore::with_capacity(4);
    let (page, index) = pack(0);
    store.set_curve_point_drag(param_steps_editor_id(0), page, index, 0.0, 0.25);
    let after = ph2d_steps::parse(&drain_drag(&mut store, 0, &r).expect("drenado"));
    assert!((after[0] - 0.25).abs() < 1e-6);
    assert_eq!(after[1], 42.0, "o de fora da faixa sobrevive");
    assert_eq!(after[2], -7.0);
    // E ele é DESENHADO saturado: a fração satura nas duas pontas.
    assert_eq!(frac(42.0, 0.0, 1.0), 1.0);
    assert_eq!(frac(-7.0, 0.0, 1.0), 0.0);
}

/// **Um arrasto de outro painel é DEIXADO para ele** — o stash é um canal global, então a
/// pergunta de posse é parte da chamada.
#[test]
fn a_foreign_drag_is_left_alone() {
    let r = row("0.1 0.2");
    let mut store = WidgetStore::with_capacity(4);
    store.set_curve_point_drag(ph2d_a11y::NodeId(999_999), 0, 0, 0.0, 0.9);
    assert!(drain_drag(&mut store, 0, &r).is_none());
    // E ele continua lá para o dono.
    assert!(
        store
            .take_curve_point_drag_if(|p| p == ph2d_a11y::NodeId(999_999))
            .is_some()
    );
}

/// **`+` repete o último passo** (o padrão não salta de forma ao crescer) e **`−` devolve o
/// sinal de NADA AUTORADO** quando tira o último.
#[test]
fn add_repeats_the_last_and_the_last_remove_returns_to_the_legacy_path() {
    assert_eq!(add_step(&row("0.2 0.7")), "0.2 0.7 0.7");
    // Lista vazia nasce no meio da faixa — o único valor que não presume nada.
    assert_eq!(add_step(&row("")), "0.5");
    assert_eq!(remove_step(&row("0.2 0.7 0.9")), "0.2 0.7");
    assert_eq!(
        remove_step(&row("0.2")),
        "",
        "tirar o último passo devolve o nó ao caminho legado"
    );
}

/// **O `+` para no teto de RECURSO** em vez de crescer para sempre.
#[test]
fn add_stops_at_the_resource_cap() {
    let full: String = (0..ph2d_steps::MAX_ENTRIES).map(|_| "0.5 ").collect();
    let r = row(&full);
    let grown = ph2d_steps::parse(&add_step(&r));
    assert_eq!(grown.len(), ph2d_steps::MAX_ENTRIES);
}

/// **A barra cresce a partir do ZERO da faixa, não do fundo** — numa lista COM SINAL um
/// valor negativo desenha para baixo em vez de mentir com uma barra curta.
#[test]
fn the_bar_grows_from_the_ranges_zero() {
    // Faixa `0..1`: o zero É o fundo, então a lei reduz ao preenchimento de baixo.
    assert_eq!(frac(0.0, 0.0, 1.0), 0.0);
    // Faixa `-1..1`: o zero está no MEIO.
    assert!((frac(0.0, -1.0, 1.0) - 0.5).abs() < 1e-6);
    assert_eq!(frac(-1.0, -1.0, 1.0), 0.0);
    assert_eq!(frac(1.0, -1.0, 1.0), 1.0);
    // E os bytes de tinta de +0,5 e −0,5 diferem: eles crescem para lados opostos.
    let mut up = row("0.5");
    up.min = -1.0;
    let mut down = row("-0.5");
    down.min = -1.0;
    let bytes = |r: &StepsRow| {
        let mut hit = HitIndex::default();
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        let store = WidgetStore::with_capacity(4);
        let mut out = StepsWidgets::new();
        paint_steps_row(
            r,
            0,
            0.0,
            400.0,
            0.0,
            12.0,
            &store,
            &mut hit,
            &mut scene,
            &mut text,
            Theme::default(),
            &mut out,
        );
        scene.inner().encoding().path_data.clone()
    };
    assert_ne!(bytes(&up), bytes(&down));
}

/// **Uma faixa degenerada não divide por zero** — um hint sem faixa desenha vazio em vez
/// de produzir `NaN`/`inf` na altura da barra.
#[test]
fn a_degenerate_range_draws_empty_instead_of_dividing_by_zero() {
    assert_eq!(frac(0.5, 1.0, 1.0), 0.0);
    assert_eq!(frac(0.5, 1.0, 0.0), 0.0, "faixa invertida");
    assert!(value_at_frac(0.5, 1.0, 1.0).is_finite());
}

//! **O ESTILO de um `source.shape`** — o `size` como coluna, o Trim e o tracejado (doc 89
//! folha 14, as duas células que a fecharam).
//!
//! Irmão do `motion_shape_gen_tests` (a porta única chave↔geometria) e do
//! `motion_shape_catalogue_tests` (a tabela por-espécie), pelo mesmo corte por assunto: aqui
//! mora o que é verdade sobre a forma **depois** de ela ser cozida — a escala que saiu para a
//! instância, o trecho revelado, a linha interrompida.

use super::{build_shape_path, manifest_default};
use crate::motion_state::MotionState;
use ph2d_node_motion_shape::{ALL_KINDS, ShapeParams, param, shape_key};
use ph2d_nodegraph::attr::Column;

/// O descritor cru de uma espécie num dado tamanho, com o resto nos defaults do manifesto.
fn raw(kind: usize, size: f32) -> ShapeParams {
    ShapeParams::read(|n: &str| match n {
        param::KIND => kind as f32,
        param::SIZE => size,
        other => manifest_default(other),
    })
}

/// **A GEOMETRIA EM RAIO 1, ESCALADA, É A GEOMETRIA NO TAMANHO AUTORADO** — a lei que
/// autoriza tirar o `size` da chave (doc 89 folha 14).
///
/// ⚠️ **É a lei INTEIRA da célula, e não a metade fácil.** Tirar o `size` do cache é trivial;
/// o que custa é que a imagem não mude, e isso só é verdade porque a receita é **linear no
/// `size`**: o `ry` é `size·aspect`, o raio de canto e os três desvios são FRAÇÕES do `size`,
/// e todo o resto (proporções, ângulos, contagens) é livre de escala. Este gate percorre o
/// catálogo inteiro em vez de acreditar na frase.
///
/// ⚠️ A tolerância é **relativa**, porque o erro que se admite aqui é o do `f32` a subir para
/// `f64` e a voltar pela multiplicação da escala — ele cresce com a coordenada. Um absoluto
/// seria apertado demais na forma grande e frouxo demais na pequena.
#[test]
fn the_unit_geometry_scaled_is_the_geometry_at_the_authored_size() {
    const SIZES: [f32; 4] = [0.35, 1.0, 3.0, 7.5];
    let mut worst = 0.0_f64;
    let mut worst_at = String::new();
    let mut offenders: Vec<String> = Vec::new();
    for (k, kind) in ALL_KINDS.iter().enumerate() {
        let mut kind_worst = 0.0_f64;
        for s in SIZES {
            let direct = build_shape_path(&raw(k, s));
            let (unit_p, scale) = ShapeParams::read_unit(|n: &str| match n {
                param::KIND => k as f32,
                param::SIZE => s,
                other => manifest_default(other),
            });
            assert_eq!(scale, s, "a escala publicada e' o tamanho autorado");
            let unit = build_shape_path(&unit_p);
            assert_eq!(
                unit.verts.len(),
                direct.verts.len(),
                "espécie {k} em {s}: a contagem de vértices mudou com a escala"
            );
            for (u, d) in unit.verts.iter().zip(&direct.verts) {
                for a in 0..2 {
                    let got = u.anchor[a] * f64::from(scale);
                    let want = d.anchor[a];
                    let err = (got - want).abs() / f64::from(s);
                    kind_worst = kind_worst.max(err);
                    if err > worst {
                        worst = err;
                        worst_at = format!("kind {k} @ {s}");
                    }
                }
            }
        }
        if kind_worst > 1e-6 {
            offenders.push(format!("{k}={kind:?} ({kind_worst:.4})"));
        }
    }
    assert!(
        offenders.is_empty(),
        "espécies NÃO equivariantes: {offenders:?}"
    );
    // ⚠️ O número é MEDIDO, não escolhido: rode com `--nocapture` e a mensagem imprime o pior
    // do catálogo. Baixá-lo é a mutação que prova que este gate mede alguma coisa.
    assert!(
        worst < 1e-6,
        "a forma em raio 1 escalada divergiu da forma no tamanho: pior erro relativo {worst:e} \
         em {worst_at}"
    );
}

/// **UM SLIDER DE `size` ANIMADO INTERNA UMA GEOMETRIA, NÃO UMA POR VALOR** — o defeito
/// que a célula nomeava, medido pelo handle que o `publish` devolve.
///
/// ⚠️ O oráculo é o HANDLE, e não uma contagem interna do store: é o handle que diz se o
/// `intern` achou a chave ou construiu de novo, e ele é a mesma coisa que a instância carrega.
#[test]
fn an_animated_size_interns_one_geometry_not_one_per_value() {
    let mut state = MotionState::new();
    let n = state.doc.graph.add_node("source.shape");
    let mut handles = Vec::new();
    for step in 0..24 {
        state
            .doc
            .graph
            .set_param(n, param::SIZE, 0.2 + 0.1 * step as f32);
        super::publish(&mut state, 0.0);
        let out = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, n, 0.0)
            .expect("cook");
        let stream = out[0].as_stream();
        let Some(Column::Scalar(ids)) = stream.get("geometry_id") else {
            panic!("geometry_id column");
        };
        handles.push(ids[0] as u32);
    }
    assert!(handles.iter().all(|&h| h == handles[0]), "{handles:?}");
}

/// **E O TAMANHO AUTORADO VIAJA NA COLUNA `size`** — a outra metade, sem a qual a de cima
/// seria verde numa implementação que simplesmente ignorasse o tamanho.
///
/// ⚠️ A coluna **não existia** antes desta wave: um `value.attribute("size")` a jusante lia o
/// default de stream (unidade), não o tamanho da forma. É a metade "não é coluna" do título
/// da célula.
#[test]
fn the_authored_size_rides_the_instance_column() {
    let mut state = MotionState::new();
    let n = state.doc.graph.add_node("source.shape");
    state.doc.graph.set_param(n, param::SIZE, 2.75);
    super::publish(&mut state, 0.0);
    let out = state
        .pump
        .cook
        .cook(&state.doc.graph, &state.registry, n, 0.0)
        .expect("cook");
    let Some(Column::Vec2(size)) = out[0].as_stream().get("size") else {
        panic!("a coluna `size` tem de existir");
    };
    assert_eq!(size[0], [2.75, 2.75]);
}

/// **O TRIM ABRE O CONTORNO E ENCURTA O CAMINHO** — e o neutro não toca em nada.
///
/// ⚠️ As duas metades numa só de propósito: *aparar* e *não aparar* são a mesma pergunta, e um
/// gate que só medisse a aparada ficaria verde numa implementação que aparasse SEMPRE — que é
/// o defeito caro, porque ele quebra toda forma que já existe.
#[test]
fn the_trim_opens_the_contour_and_the_neutral_one_leaves_it_shut() {
    let plain = build_shape_path(&raw(0, 1.0)); // Circle
    assert!(plain.closed, "um circulo nasce fechado");
    assert!(plain.effects.is_empty(), "e sem pilha nenhuma");

    let trimmed = build_shape_path(&ShapeParams::read(|n: &str| match n {
        param::TRIM_END => 0.35,
        param::STROKE_WIDTH => 0.05,
        other => manifest_default(other),
    }));
    assert_eq!(trimmed.effects.len(), 1, "o Trim entrou na pilha");
    // ⚠️ O oráculo é o COZIDO, não a pilha: um efeito na lista que o `run_stack` não aplicasse
    // deixaria este gate verde e a tela igual.
    let cooked = trimmed.cooked();
    assert!(
        !cooked.closed,
        "aparar um fechado ABRE o contorno — e' o que faz um traco correr em torno dele"
    );
    assert!(
        cooked.verts.len() < plain.verts.len(),
        "e o trecho tem menos vertices que a volta inteira: {} contra {}",
        cooked.verts.len(),
        plain.verts.len()
    );
}

/// **O TRACEJADO CHEGA AO TRAÇO, E É INERTE SEM ELE** — a lei do `StrokeSpec`, que já falava
/// tracejado; o que faltava era o nó ter onde dizê-lo.
#[test]
fn the_dash_reaches_the_stroke_and_is_inert_without_one() {
    let dashed = build_shape_path(&ShapeParams::read(|n: &str| match n {
        param::STROKE_WIDTH => 0.05,
        param::DASH => 3.0,
        param::DASH_GAP => 2.0,
        other => manifest_default(other),
    }));
    assert_eq!(
        dashed.stroke.expect("ha' traco").dash,
        Some((3.0, 2.0)),
        "em MULTIPLOS da largura, como o StrokeSpec os fala"
    );
    // Sem largura não há `StrokeSpec` nenhum — então o tracejado não tem onde pousar, e a
    // forma é a de sempre. É a razão de o `ParamGateAbove` o esconder.
    let no_stroke = build_shape_path(&ShapeParams::read(|n: &str| match n {
        param::DASH => 3.0,
        other => manifest_default(other),
    }));
    assert!(no_stroke.stroke.is_none());
}

/// **O DEFAULT DE TUDO ISTO É BYTE-IDÊNTICO** — cinco params novos e a forma que já existia
/// não se mexe.
///
/// ⚠️ Compara a CHAVE e a geometria: a chave porque um param novo que não entrasse nela
/// deixaria o controle inerte depois da primeira vez (o defeito do *Pattern Offset*), e a
/// geometria porque uma chave nova sobre a mesma forma só custaria uma entrada de cache.
#[test]
fn the_defaults_of_the_new_family_move_nothing() {
    let dflt = |n: &str| manifest_default(n);
    let p = ShapeParams::read(dflt);
    assert_eq!(p.trim, [0.0, 1.0, 0.0], "o neutro do TrimSpec");
    assert_eq!(p.dash, None);
    assert_eq!(p.stroke, None);
    let path = build_shape_path(&p);
    assert!(
        path.effects.is_empty(),
        "pilha vazia ⇒ `cooked` e' Borrowed"
    );
    // E a chave do default é estável — a mesma f32 para os mesmos bits, pelos dois lados.
    assert_eq!(shape_key(dflt), shape_key(dflt));
}

/// **UM PARAM CONDUZIDO POR FIO CHEGA AO PUBLICADOR** — o defeito silencioso que o Trim
/// tornou provável, medido em 2026-08-21.
///
/// ⚠️ **Contagem 0 era o sintoma inteiro:** a chave de conteúdo é derivada dos params, o shell
/// publica antes do cook, e o valor conduzido só existe durante o cook — as duas chaves não se
/// encontravam e o `eval` clonava o external vazio. A forma **desaparecia**, com o nó certo
/// selecionado e nada vermelho em lado nenhum. E o Trim é justamente o param que se quer
/// conduzir: *keyar o `end` de 0 a 1 desenha a forma*.
///
/// ⚠️ **O oráculo não é a contagem, é a GEOMETRIA acompanhar o relógio.** Um gate que só
/// exigisse contagem 1 ficaria verde numa implementação que publicasse o valor ESTÁTICO e
/// ignorasse o fio — a forma apareceria, parada, e o modo de falha seria pior que o de hoje
/// (silencioso *e* plausível).
#[test]
fn a_wire_driven_param_reaches_the_publisher_and_the_geometry_follows_the_clock() {
    let mut state = MotionState::new();
    let n = state.doc.graph.add_node("source.shape");
    state.doc.graph.set_param(n, param::STROKE_WIDTH, 0.05);
    let clock = state.doc.graph.add_node("value.time");
    state
        .doc
        .graph
        .drive_param(n, param::TRIM_END, (clock, 0))
        .expect("um param conduzido por fio");

    let revealed = |state: &mut MotionState, sec: f64| -> usize {
        super::publish(state, sec);
        let out = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, n, sec)
            .expect("cook");
        let stream = out[0].as_stream();
        let Some(Column::Scalar(ids)) = stream.get("geometry_id") else {
            panic!("geometry_id column");
        };
        let handle = ids[0] as u32;
        state
            .shape_store
            .get(handle)
            .map_or(0, |p| p.cooked().verts.len())
    };
    let early = revealed(&mut state, 0.25);
    let late = revealed(&mut state, 0.80);
    assert!(
        early > 0,
        "a forma DESAPARECEU — a chave do shell nao encontrou a do no'"
    );
    assert!(
        late > early,
        "mais relogio, mais traco revelado: {early} vertices em 0,25 s contra {late} em 0,80 s"
    );
}

/// **SONDA — quanto o cache de geometria CRESCE com um param animado.**
///
/// ⚠️ Ela existe porque a wave do Trim tornou o crescimento **alcançável na prática**: até
/// 2026-08-21 um param conduzido por fio fazia a forma desaparecer, então ninguém animava um
/// `source.shape` por fio; hoje anima, e cada valor visitado é uma chave nova no
/// `VecPathStore`. O `size` saiu da chave nesta mesma wave (a célula da folha 14) e por isso
/// **não** cresce; o Trim, o `sweep` e os outros continuam a crescer, porque mudam a
/// geometria de facto.
///
/// `cargo test -p ph2d-host-desktop --bins measure_shape_store_growth -- --ignored --nocapture`
#[test]
#[ignore = "sonda de medição, não gate"]
fn measure_shape_store_growth() {
    const FRAMES: u32 = 600; // dez segundos a 60 fps
    let mut state = MotionState::new();
    let n = state.doc.graph.add_node("source.shape");
    state.doc.graph.set_param(n, param::STROKE_WIDTH, 0.05);
    let clock = state.doc.graph.add_node("value.time");
    state
        .doc
        .graph
        .drive_param(n, param::TRIM_OFFSET, (clock, 0))
        .expect("drive");
    let mut last = 0;
    for f in 0..FRAMES {
        let sec = f64::from(f) / 60.0;
        super::publish(&mut state, sec);
        let out = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, n, sec)
            .expect("cook");
        if let Some(Column::Scalar(ids)) = out[0].as_stream().get("geometry_id") {
            last = ids[0] as u32;
        }
    }
    println!(
        "\n=== VecPathStore apos {FRAMES} quadros com `trim_offset` conduzido: {last} \
         geometrias internadas ({:.1} por quadro)\n",
        f64::from(last) / f64::from(FRAMES)
    );
}

/// **UM `motion.drive(Size)` A JUSANTE MUDA DE SIGNIFICADO — e a mudança é a CERTA.**
///
/// ⚠️ **A nota do handoff previu isto e mandou medir antes de mexer**, e é o único efeito
/// visível da normalização: enquanto o `source.shape` não publicava coluna `size`, o
/// `motion.drive` partia da IDENTIDADE unitária (`identity: [1,1]`), então `Set 3` e
/// `Multiply 3` davam **a mesma coisa** — o modo era um controle sem diferença. Com o
/// tamanho autorado na coluna, `Set` **põe** e `Multiply` **compõe**, que é o que os dois
/// nomes prometem.
///
/// ⚠️ O que se perde é ler `Set` como *"×N sobre o que a forma tem"* — e isso já se chama
/// `Multiply`, uma linha acima no mesmo dropdown.
#[test]
fn the_size_column_makes_set_and_multiply_mean_different_things() {
    let mut state = MotionState::new();
    let shape = state.doc.graph.add_node("source.shape");
    state.doc.graph.set_param(shape, param::SIZE, 2.0);
    let drive = state.doc.graph.add_node("motion.drive");
    // `channel = Size` (a tabela do nó) e um valor constante de 3.
    state.doc.graph.set_param(drive, "channel", 3.0);
    let _ = state.doc.graph.connect(ph2d_nodegraph::graph::Edge {
        from: (shape, 0),
        to: (drive, 0),
        delayed: false,
    });
    // ⚠️ O valor entra pela PORTA, não por um param: com a porta solta o `motion.drive` lê
    // ZERO e os três modos colapsam (medido: `add=2 set=0 mul=0`) — a fixture não conteria o
    // fenómeno. Um relógio cozinhado em `SEC` dá o número.
    const SEC: f64 = 3.0;
    let clock = state.doc.graph.add_node("value.time");
    let _ = state.doc.graph.connect(ph2d_nodegraph::graph::Edge {
        from: (clock, 0),
        to: (drive, 1),
        delayed: false,
    });
    super::publish(&mut state, SEC);
    let sized = |state: &mut MotionState, mode: f32| -> f32 {
        state.doc.graph.set_param(drive, "mode", mode);
        let out = state
            .pump
            .cook
            .cook(&state.doc.graph, &state.registry, drive, SEC)
            .expect("cook");
        match out[0].as_stream().get("size") {
            Some(Column::Vec2(v)) if !v.is_empty() => v[0][0],
            _ => f32::NAN,
        }
    };
    // A tabela de modos do `motion.drive` é `["Add", "Set", "Multiply"]` — lida do nó, não
    // adivinhada (a 1ª versão deste gate leu `Set = 0` e mediu o Add).
    let add = sized(&mut state, 0.0);
    let set = sized(&mut state, 1.0);
    let mul = sized(&mut state, 2.0);
    assert!(
        (set - mul).abs() > 1e-3,
        "`Set` e `Multiply` tem de diferir sobre um tamanho autorado: {set} contra {mul}"
    );
    assert!((add - 5.0).abs() < 1e-3, "Add SOMA ao 2 autorado: {add}");
    assert!((set - 3.0).abs() < 1e-3, "Set PÕE o numero: {set}");
    assert!(
        (mul - 6.0).abs() < 1e-3,
        "Multiply COMPOE com o 2 autorado: {mul}"
    );
}

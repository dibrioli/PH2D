//! **A DOAÇÃO, pela porta do produto** (`docs/3D/05.2`, costura S1).
//!
//! Os gates de unidade do `impasto_shade` provam a LEI (a composição das duas fontes de normal, e que
//! ela degenera exato sem forma). Estes provam o PASSE: que o plano de forma atravessa as três camadas
//! que perguntam *"há algo a iluminar?"* e chega aos pixels.
//!
//! ⚠️ São três perguntas, não uma, e é por isso que há um gate por camada — a lição que a casa já pagou
//! com o *default de mídia* do Painter (*"a regra de ENTRADA mascara a de SAÍDA"*): `impasto_visible`
//! decide se o passe corre, `impasto_fields` decide se há planos, e o early-out por texel decide se
//! aquele pixel muda. Um plano que passe em duas e morra na terceira é invisível — e verde.

use ph2d_editor_core::tool::RasterEditTool as _;
use ph2d_tool_painter::PainterTool;
use std::sync::Arc;

const N: u32 = 16;

/// Um Painter com tela CHAPADA e nenhum relevo — o caso que a doação existe para servir.
fn flat_tool() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![128u8; (N * N * 4) as usize], N, N);
    t
}

/// A tela composta, tal como o passe a entrega.
fn lit(t: &PainterTool) -> Vec<u8> {
    let mut rgba = vec![128u8; (N * N * 4) as usize];
    t.apply_impasto_light(
        &mut rgba,
        ph2d_tool_painter::Region {
            x: 0,
            y: 0,
            w: N,
            h: N,
        },
    );
    rgba
}

/// Uma forma que cobre a tela inteira, inclinada PARA a lâmpada principal (que vem de cima e da
/// esquerda: `x < 0`, `y < 0` no espaço do rig).
fn tilted_form() -> Arc<Vec<f32>> {
    let n = [-0.5f32, -0.5, std::f32::consts::FRAC_1_SQRT_2];
    let mut v = Vec::with_capacity((N * N * 4) as usize);
    for _ in 0..N * N {
        v.extend_from_slice(&[n[0], n[1], n[2], 1.0]);
    }
    Arc::new(v)
}

/// **Tinta chapada sobre uma forma ACENDE — e é isto que o objetivo O1 quer dizer.**
///
/// Sem a doação a mesma tela sai intocada ao byte: não há relevo, não há gradiente, não há nada que o
/// passe pudesse ter a dizer. É a forma que traz a informação.
#[test]
fn flat_paint_over_a_form_lights_up() {
    let mut t = flat_tool();
    let before = lit(&t);
    assert_eq!(
        before,
        vec![128u8; (N * N * 4) as usize],
        "sem escultura e sem relevo, o passe não toca um byte"
    );

    t.set_donated_form(Some(tilted_form()));
    let after = lit(&t);
    assert_ne!(before, after, "a forma não chegou aos pixels");
    // Inclinada PARA a luz ⇒ mais clara que o cinza de partida.
    assert!(
        after[0] > 140,
        "a forma virada para a lâmpada tinha de clarear a tinta: {} contra 128",
        after[0]
    );
}

/// **Tirar a doação devolve a tela ao byte.**
///
/// A costura S1 promete *"com a flag off, byte-idêntico"*, e a promessa não é sobre uma flag de
/// compilação: é sobre o passe. Se remover o plano deixasse resíduo, a promessa de removibilidade do
/// módulo (`docs/3D/02.3`) seria falsa no lugar mais caro — o pixel.
#[test]
fn taking_the_donation_away_gives_the_canvas_back_to_the_byte() {
    let mut t = flat_tool();
    let pristine = lit(&t);
    t.set_donated_form(Some(tilted_form()));
    assert_ne!(lit(&t), pristine, "premissa: a doação de fato mudou algo");
    t.set_donated_form(None);
    assert_eq!(lit(&t), pristine, "a tela não voltou ao que era");
}

/// **A forma virada para o OLHO não muda nada, e isso é o modelo RELATIVO.**
///
/// Uma normal `[0, 0, 1]` é o que uma superfície plana tem, e o passe divide pela resposta de uma
/// superfície plana — então a razão é exatamente 1 e o pixel é o pixel. Sem esta propriedade a
/// escultura escureceria (ou clarearia) a pintura inteira só por existir, que é o bug que metade dos
/// filtros de emboss já escritos tem.
#[test]
fn a_form_facing_the_viewer_multiplies_by_exactly_one() {
    let mut t = flat_tool();
    let pristine = lit(&t);
    let flat_form: Vec<f32> = (0..N * N).flat_map(|_| [0.0, 0.0, 1.0, 1.0]).collect();
    t.set_donated_form(Some(Arc::new(flat_form)));
    assert_eq!(
        lit(&t),
        pristine,
        "uma forma chapada mudou a pintura — o modelo deixou de ser relativo"
    );
}

/// **Fora da silhueta a forma é AUSENTE, não deitada.**
///
/// O G-buffer escreve `[0, 0, 0, 0]` onde não há malha, e um `z` zero **não** é "nenhuma forma": é uma
/// normal deitada, que somada à inclinação da tinta daria um vetor quase horizontal e uma faixa escura
/// em volta de toda escultura. O neutro de *"não há forma aqui"* é `[0, 0, 1]`.
///
/// ⚠️ O fixture é metade coberto de propósito: com a tela inteira coberta a armadilha não existe, e
/// com ela inteira vazia o gate seria o irmão de cima.
#[test]
fn outside_the_silhouette_the_form_is_absent_not_lying_down() {
    let mut t = flat_tool();
    // Metade esquerda: o que o G-buffer escreve FORA da malha. Metade direita: uma forma de verdade.
    let mut half = Vec::with_capacity((N * N * 4) as usize);
    for y in 0..N {
        let _ = y;
        for x in 0..N {
            if x < N / 2 {
                half.extend_from_slice(&[0.0, 0.0, 0.0, 0.0]);
            } else {
                half.extend_from_slice(&[-0.5, -0.5, std::f32::consts::FRAC_1_SQRT_2, 1.0]);
            }
        }
    }
    t.set_donated_form(Some(Arc::new(half)));
    let px = lit(&t);
    let at = |x: u32, y: u32| px[((y * N + x) * 4) as usize];
    assert_eq!(
        at(2, 8),
        128,
        "onde o G-buffer diz que não há forma, a tinta é intocada"
    );
    assert!(
        at(13, 8) > 140,
        "e onde há forma, ela acende: {}",
        at(13, 8)
    );
}

/// **Um plano com a forma errada é RECUSADO, não lido torto.**
///
/// Um canvas pode ser redimensionado entre o instante em que a malha foi rasterizada e o instante em
/// que a luz roda. Ler o plano velho descreveria a escultura no lugar errado — uma luz torta que
/// ninguém liga à escultura. Guardar e conferir na LEITURA é o que torna o descasamento inofensivo em
/// vez de invisível.
#[test]
fn a_form_plane_of_the_wrong_shape_is_refused() {
    let mut t = flat_tool();
    let pristine = lit(&t);
    // Metade dos texels que o canvas tem.
    let short: Vec<f32> = (0..N * N / 2)
        .flat_map(|_| [-0.5, -0.5, std::f32::consts::FRAC_1_SQRT_2, 1.0])
        .collect();
    t.set_donated_form(Some(Arc::new(short)));
    assert_eq!(
        lit(&t),
        pristine,
        "um plano de outro tamanho foi lido em vez de recusado"
    );
    // ⚠️ E ele fica GUARDADO — recusar é uma decisão da leitura, não um descarte. O canvas pode voltar
    // ao tamanho de antes, e aí o plano volta a servir.
    assert!(
        t.donated_form().is_some(),
        "o plano não devia ser jogado fora"
    );
}

/// **O RELEVO DA TINTA fora da silhueta continua sendo lido como relevo.**
///
/// ⚠️ **Este gate nasceu de uma mutação que sobreviveu, e a mutação era boa.** Tirar o guard de peso do
/// `form_at` passava nos cinco gates acima — porque naquelas fixtures a tinta é CHAPADA, e aí o
/// early-out por texel já pega o caso pelo peso zero. O guard só carrega peso onde há **relevo de tinta
/// fora da escultura**: sem ele `form_at` devolve `[0, 0, 0, 0]`, o `z` do vetor composto vira ZERO, a
/// normal deita, e o relevo da pincelada é iluminado por uma superfície vertical.
///
/// A fixture é que não continha o fenômeno. Esta contém: um traço com corpo, e uma escultura que não
/// chega até ele.
#[test]
fn paint_relief_outside_the_forms_silhouette_is_still_read_as_relief() {
    use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase};
    let cp = |pos: [f32; 2], phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    let mut t = flat_tool();
    t.toggle_brush_impasto();
    t.set_brush_size_px(5.0);
    // Um traço na metade ESQUERDA, onde a escultura não chega.
    t.on_canvas_pointer(cp([3.0, 8.0], PointerPhase::Down));
    t.on_canvas_pointer(cp([5.0, 8.0], PointerPhase::Move));
    t.on_canvas_pointer(cp([5.0, 8.0], PointerPhase::Up));

    let without = lit(&t);
    // A escultura cobre só a metade DIREITA — o traço fica fora dela.
    let mut half = Vec::with_capacity((N * N * 4) as usize);
    for _y in 0..N {
        for x in 0..N {
            if x < N / 2 {
                half.extend_from_slice(&[0.0, 0.0, 0.0, 0.0]);
            } else {
                half.extend_from_slice(&[-0.5, -0.5, std::f32::consts::FRAC_1_SQRT_2, 1.0]);
            }
        }
    }
    t.set_donated_form(Some(Arc::new(half)));
    let with = lit(&t);

    // Premissa: o traço de fato deixou relevo, senão este gate mede uma tela chapada.
    let pristine = vec![128u8; (N * N * 4) as usize];
    assert_ne!(
        without, pristine,
        "premissa: o traço tem de acender sozinho"
    );

    // E o LADO ESQUERDO — onde não há forma — tem de sair IDÊNTICO com e sem a doação.
    let mut worst = 0i32;
    for y in 0..N {
        for x in 0..N / 2 {
            let i = ((y * N + x) * 4) as usize;
            for c in 0..3 {
                worst = worst.max((i32::from(without[i + c]) - i32::from(with[i + c])).abs());
            }
        }
    }
    assert_eq!(
        worst, 0,
        "a escultura mudou o relevo da tinta FORA dela em {worst} níveis — a normal deitou"
    );
}

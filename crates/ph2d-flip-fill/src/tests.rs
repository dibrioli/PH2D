//! Os testes do balde — módulo-irmão pelo cap de LOC do workspace (700; o gate
//! `architecture_workspace_file_loc_cap` exclui `src/tests.rs`, a convenção de
//! test-code). Declarado pelo pai com `mod tests;`, então `super` é a crate.

use super::*;
/// Um quadrado de linha, com meia-espessura `w`.
fn square(a: f32, b: f32, w: f32) -> (Vec<Vec2>, Vec<f32>, bool) {
    (
        vec![
            Vec2::new(a, a),
            Vec2::new(b, a),
            Vec2::new(b, b),
            Vec2::new(a, b),
        ],
        vec![w; 4],
        true,
    )
}

/// O caminho feliz: clicar dentro de uma forma fechada devolve o contorno dela.
#[test]
fn clicking_inside_a_closed_shape_returns_its_contour() {
    let strokes = [square(0.0, 20.0, 0.3)];
    let r = fill_at(&strokes, Vec2::new(10.0, 10.0), FillParams::default())
        .expect("um quadrado fechado tem de preencher");
    assert!(r.holes.is_empty());
    let area = signed_area(&r.outer).abs();
    // ~20×20 = 400, menos a linha, mais o grow. A ordem de grandeza é o que importa.
    assert!(
        (300.0..=450.0).contains(&area),
        "a área bate com a forma: {area}"
    );
}

/// **A promessa dos buracos:** a letra "O". Clicar entre os dois quadrados devolve
/// o externo E o furo — e é por isso que o `FlipStroke` tem `holes`.
#[test]
fn a_donut_returns_the_outer_ring_and_the_hole() {
    let strokes = [square(0.0, 30.0, 0.3), square(10.0, 20.0, 0.3)];
    let r = fill_at(&strokes, Vec2::new(3.0, 3.0), FillParams::default())
        .expect("a rosquinha tem de preencher");
    assert_eq!(r.holes.len(), 1, "um furo");
    let outer = signed_area(&r.outer).abs();
    let hole = signed_area(&r.holes[0]).abs();
    assert!(outer > 700.0, "o externo é o quadrado de 30: {outer}");
    assert!(
        (60.0..=140.0).contains(&hole),
        "o furo é o quadrado de 10: {hole}"
    );
}

/// **Uma forma ABERTA é recusada** — em vez de pintar o documento inteiro. É o
/// momento em que a UI deve sugerir o Gap Closure.
#[test]
fn an_open_shape_is_refused_not_flooded() {
    // Um "C": três lados de um quadrado.
    let c = (
        vec![
            Vec2::new(20.0, 0.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 20.0),
            Vec2::new(20.0, 20.0),
        ],
        vec![0.3; 4],
        false,
    );
    let err = fill_at(&[c], Vec2::new(10.0, 10.0), FillParams::default()).unwrap_err();
    assert_eq!(err, FillError::Leaked);
}

/// **E o Gap Closure fecha o "C"**: com alcance suficiente, as duas pontas se
/// estendem, colidem, e a região passa a existir. Os fechamentos voltam no
/// resultado — o chamador os materializa como traços invisíveis.
#[test]
fn gap_closure_makes_the_open_shape_fillable() {
    let c = (
        vec![
            Vec2::new(20.0, 0.0),
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 20.0),
            Vec2::new(20.0, 20.0),
        ],
        vec![0.3; 4],
        false,
    );
    // As duas pontas apontam para +x; elas não se veem. Mas a quina de cada uma…
    // Não: aqui o que fecha é uma parede à direita. Põe uma linha vertical em x=22.
    let wall = (
        vec![Vec2::new(22.0, -2.0), Vec2::new(22.0, 22.0)],
        vec![0.3; 2],
        false,
    );
    let params = FillParams {
        gap_reach: 5.0,
        ..Default::default()
    };
    let r = fill_at(&[c, wall], Vec2::new(10.0, 10.0), params)
        .expect("com Gap Closure, o C fecha contra a parede");
    assert!(
        !r.closures.is_empty(),
        "os fechamentos têm de voltar (viram traços invisíveis)"
    );
    assert!(signed_area(&r.outer).abs() > 300.0);
}

/// Clicar em cima da linha não preenche (e diz por quê).
#[test]
fn clicking_on_the_line_is_refused() {
    let strokes = [square(0.0, 20.0, 0.5)];
    let err = fill_at(&strokes, Vec2::new(0.0, 10.0), FillParams::default()).unwrap_err();
    assert_eq!(err, FillError::OnBoundary);
}

/// **Determinismo (HR-5):** a mesma entrada dá o mesmo contorno, bit a bit.
#[test]
fn the_same_click_gives_the_same_geometry() {
    let strokes = [square(0.0, 30.0, 0.3), square(10.0, 20.0, 0.3)];
    let a = fill_at(&strokes, Vec2::new(3.0, 3.0), FillParams::default()).unwrap();
    let b = fill_at(&strokes, Vec2::new(3.0, 3.0), FillParams::default()).unwrap();
    assert_eq!(a, b, "o balde tem de ser determinístico");
}

/// Sem linha nenhuma, não há o que preencher.
#[test]
fn nothing_to_fill_is_an_error_not_a_panic() {
    assert_eq!(
        fill_at(&[], Vec2::new(0.0, 0.0), FillParams::default()).unwrap_err(),
        FillError::Empty
    );
}

/// **O resultado do balde NÃO pode depender do ZOOM da câmera.**
///
/// O desenho vive em unidades de mundo; aproximar a câmera faz a mesma forma ocupar
/// MENOS unidades. Um teto de resolução em "px de buffer por unidade de documento"
/// (o `clamp(0.5, 64.0)` do 1º corte) é um teto na unidade ERRADA: com zoom, ele
/// cortava a resolução em pedaços — 1 px de buffer chegou a valer 5 px de tela — e
/// como o `grow` e a tolerância do RDP vivem em px de buffer, o preenchimento inchava
/// 12 px para FORA da linha e o contorno virava um polígono de 17 lados.
///
/// Aqui: o MESMO círculo, visto com quatro zooms. A geometria de saída, medida em
/// px de TELA, tem de ser a mesma nos quatro.
#[test]
fn the_fill_is_invariant_under_camera_zoom() {
    let mut results = Vec::new();
    for height_world in [10.0f32, 5.0, 2.0, 1.0] {
        let px_to_world = height_world / 800.0;
        // Um círculo de 250 px de RAIO na tela, com um traço de 6 px — em unidades
        // de mundo, que encolhem quando a câmera aproxima.
        let r = 250.0 * px_to_world;
        let n = 64;
        let pts: Vec<Vec2> = (0..n)
            .map(|i| {
                // Polígono regular: sem transcendental no teste (HR-5), a forma exata
                // não importa — importa ela ser a MESMA em px de tela.
                let t = i as f32 / n as f32;
                let (x, y) = unit_circle(t);
                Vec2::new(r * x, r * y)
            })
            .collect();
        let half = 6.0 * 0.5 * px_to_world; // meia-espessura de um traço de 6 px
        let strokes = vec![(pts, vec![half; n], true)];

        let res = fill_at(
            &strokes,
            Vec2::new(0.0, 0.0),
            FillParams {
                precision: 1.0 / px_to_world, // 1 px de buffer por px de tela
                gap_reach: 0.0,
                grow: 0,
                trap_px: 0.0,
                mode: FillMode::Paint,
            },
        )
        .expect("o circulo fechado preenche em qualquer zoom");

        // O maior raio do contorno, em px de TELA.
        let max_r_px = res
            .outer
            .iter()
            .map(|p| (p.x * p.x + p.y * p.y).sqrt() / px_to_world)
            .fold(0.0f32, f32::max);
        results.push((height_world, res.outer.len(), max_r_px));
    }
    let (_, n0, r0) = results[0];
    for &(h, n, r) in &results[1..] {
        assert!(
            (r - r0).abs() < 2.0,
            "zoom h={h}: o contorno saiu a {r:.1} px do centro, mas com h=10 saiu a \
             {r0:.1} px — o balde depende do zoom"
        );
        // A contagem de vértices mede o FACETAMENTO: cair pela metade = polígono.
        assert!(
            n * 2 >= n0,
            "zoom h={h}: o contorno desabou de {n0} para {n} vertices — o buffer \
             ficou grosseiro e o fill virou um poligono"
        );
    }
    // **A regra, em números.** O EIXO da linha está a 250 px do centro. A borda do
    // preenchimento tem de cravar NELE — é a única âncora que não depende nem do
    // zoom nem da espessura (BUGS #14): a linha renderizada sempre cavalga o eixo,
    // metade para cada lado, então a cor que termina ali nunca aparece por fora
    // nem descola da linha.
    for &(h, _, r) in &results {
        assert!(
            (r - 250.0).abs() <= 1.5,
            "zoom h={h}: o contorno saiu a {r:.1} px do centro — a ancora nao e o \
             eixo (250)"
        );
    }
}

/// Um ponto do círculo unitário em `t ∈ [0,1)`, sem `sin`/`cos` (HR-5).
///
/// `u = tan(θ/2)` parametriza racionalmente o arco: com `u ∈ [0,1]` sai EXATAMENTE o
/// 1º quadrante (θ de 0° a 90°), e os outros três saem por rotação de 90°. (A 1ª
/// versão usava `u ∈ [-1,1]`, que é um SEMICÍRCULO — girado quatro vezes, ele saltava
/// de (0,1) para (1,0) e o "círculo" tinha uma corda enorme. O solver estava certo; o
/// teste é que descrevia outra forma.)
fn unit_circle(t: f32) -> (f32, f32) {
    let q = (t * 4.0).floor() as i32 % 4;
    let u = (t * 4.0).fract(); // [0,1) → o quarto de arco
    let d = 1.0 + u * u;
    let (x, y) = ((1.0 - u * u) / d, 2.0 * u / d);
    match q {
        0 => (x, y),
        1 => (-y, x),
        2 => (-x, -y),
        _ => (y, -x),
    }
}

/// O mesmo círculo de sempre (raio 250 px de tela), com a espessura e o grow pedidos.
/// Devolve o alcance máximo do contorno, em px de TELA do zoom do clique.
fn circle_fill_edge(px_to_world: f32, width_px: f32, grow: i32) -> f32 {
    let r = 250.0 * px_to_world;
    let n = 96;
    let pts: Vec<Vec2> = (0..n)
        .map(|i| {
            let (x, y) = unit_circle(i as f32 / n as f32);
            Vec2::new(r * x, r * y)
        })
        .collect();
    let half = width_px * 0.5 * px_to_world;
    let res = fill_at(
        &[(pts, vec![half; n], true)],
        Vec2::new(0.0, 0.0),
        FillParams {
            precision: 1.0 / px_to_world,
            gap_reach: 0.0,
            grow,
            trap_px: 0.0,
            mode: FillMode::Paint,
        },
    )
    .expect("o circulo fechado preenche");
    res.outer
        .iter()
        .map(|p| (p.x * p.x + p.y * p.y).sqrt() / px_to_world)
        .fold(0.0f32, f32::max)
}

/// **A cor para no EIXO da linha — em QUALQUER espessura.**
///
/// A âncora do preenchimento é o eixo da polilinha (a intuição do Enio, BUGS #14).
/// Das três âncoras possíveis — borda externa, borda interna, eixo — só o eixo é
/// **geometria pura**: não depende do zoom nem da espessura. E a linha renderizada
/// sempre o cavalga, metade para cada lado: um fill que termina no eixo nunca
/// transborda (a cor não passa do eixo; a linha cobre dali para fora) e nunca abre
/// vão (a metade interna da linha cobre a borda da cor).
#[test]
fn the_colour_stops_at_the_line_axis_at_any_width() {
    let px_to_world = 10.0f32 / 800.0;
    for width_px in [1.0f32, 2.0, 6.0, 16.0, 40.0] {
        let max_r = circle_fill_edge(px_to_world, width_px, 0);
        assert!(
            (max_r - 250.0).abs() <= 1.5,
            "linha de {width_px}px: o contorno saiu a {max_r:.1} px do centro — a \
             ancora nao e o eixo (250)"
        );
    }
}

/// **O gate do bug do produto (BUGS #14): preencher num zoom, OLHAR noutro.**
///
/// A espessura do traço é ABSOLUTA em px de tela (Enio 2026-07-11); a geometria do
/// fill é assada em unidades de documento no instante do clique. A relação entre as
/// duas muda quando a câmera aproxima — e qualquer âncora derivada da espessura
/// (silhueta, borda interna) deixa a borda do fill presa ao zoom do clique:
/// transbordo ≈ (w/2)·(zoom−1) px. Só o eixo sobrevive.
///
/// Aqui: preenche no zoom default e olha com a câmera a 1×, 2× e 4×. Em px de TELA
/// da vista, a cor não pode nem aparecer por fora da linha nem descolar dela — em
/// nenhuma espessura.
#[test]
fn the_baked_fill_stays_under_the_line_at_any_later_zoom() {
    let ptw_fill = 10.0f32 / 800.0; // o zoom do CLIQUE (câmera default)
    for width_px in [3.0f32, 6.0, 16.0] {
        // O contorno é assado UMA vez, no zoom do clique…
        let max_r_fill_px = circle_fill_edge(ptw_fill, width_px, 0);
        let max_r_doc = max_r_fill_px * ptw_fill;
        for zoom in [1.0f32, 2.0, 4.0] {
            // …e a vista o mede com outra câmera. A linha, absoluta em px de tela,
            // cobre eixo ± w/2 EM QUALQUER zoom; o contorno assado escala junto
            // com o documento.
            let ptw_view = ptw_fill / zoom;
            let axis_px = 250.0 * ptw_fill / ptw_view;
            let fill_px = max_r_doc / ptw_view;
            let overflow = fill_px - (axis_px + width_px * 0.5);
            let gap = (axis_px - width_px * 0.5) - fill_px;
            assert!(
                overflow <= 1.0,
                "linha de {width_px}px, zoom {zoom}x apos o clique: a cor aparece \
                 {overflow:.1} px por FORA da linha"
            );
            assert!(
                gap <= 1.0,
                "linha de {width_px}px, zoom {zoom}x apos o clique: a cor descolou \
                 {gap:.1} px da linha"
            );
        }
    }
}

/// **O slider Grow é CONTÍNUO em 0** — a reclamação exata do Enio (BUGS #14):
/// *"linhas finas nem têm valor no slider para ajustar. Aí grow 0 e −1."*
///
/// A âncora dupla (silhueta em 0, borda interna nos negativos) fazia o passo 0 → −1
/// saltar `w + 1` px. Com a âncora única no eixo, cada passo do slider move o
/// contorno ~1 px — inclusive o que cruza o zero.
#[test]
fn the_grow_slider_is_continuous_through_zero() {
    let px_to_world = 10.0f32 / 800.0;
    // Linha GROSSA de propósito: era nela que o salto 0 → −1 valia w+1 px.
    let width_px = 16.0f32;
    let edges: Vec<f32> = [-2i32, -1, 0, 1, 2]
        .iter()
        .map(|&g| circle_fill_edge(px_to_world, width_px, g))
        .collect();
    for pair in edges.windows(2) {
        let step = pair[1] - pair[0];
        assert!(
            (step - 1.0).abs() <= 1.0,
            "um passo do Grow moveu o contorno {step:.1} px (deveria ~1 px): o \
             slider salta — {edges:?}"
        );
    }
}

/// **Grow negativo recua o CONTORNO |grow| px do eixo — igual em qualquer espessura.**
///
/// (A geometria recua do EIXO; o vão *visível* só começa quando o recuo passa da
/// meia-espessura, que cobre o eixo até w/2 px. É o preço da âncora zoom-proof —
/// BUGS #14: as bordas visuais são aparência, e aparência é função da câmera.)
#[test]
fn a_negative_grow_recedes_the_contour_the_same_at_any_line_width() {
    let px_to_world = 10.0f32 / 800.0;
    for grow_px in [-2i32, -4, -8] {
        for width_px in [1.0f32, 6.0, 16.0, 40.0] {
            let fill_edge = circle_fill_edge(px_to_world, width_px, grow_px);
            let want = 250.0 + grow_px as f32;
            assert!(
                (fill_edge - want).abs() <= 1.5,
                "grow {grow_px} px, linha de {width_px} px: o contorno saiu a \
                 {fill_edge:.1} px em vez de {want:.1} — o recuo depende da espessura"
            );
        }
    }
}

/// E o lado positivo é simétrico: `grow = +N` avança o contorno N px além do eixo
/// (por baixo da linha; além de w/2 vira o "off-register" da animação 2D), também
/// igual em qualquer espessura.
#[test]
fn a_positive_grow_advances_the_contour_the_same_at_any_line_width() {
    let px_to_world = 10.0f32 / 800.0;
    for width_px in [1.0f32, 6.0, 16.0, 40.0] {
        let fill_edge = circle_fill_edge(px_to_world, width_px, 4);
        assert!(
            (fill_edge - 254.0).abs() <= 1.5,
            "linha de {width_px} px: o contorno saiu a {fill_edge:.1} px em vez de \
             254 (eixo + 4)"
        );
    }
}

/// Tabela de medição (não é gate): espessura × zoom-depois-do-clique, transbordo e
/// vão em px de TELA da vista. `cargo test -p ph2d-flip-fill sweep_table -- \
/// --ignored --nocapture`.
#[test]
#[ignore = "diagnostico manual: imprime a tabela de C.5.2 do handoff"]
fn sweep_table() {
    let ptw_fill = 10.0f32 / 800.0;
    println!("largura | zoom | transbordo (px tela) | vao (px tela)");
    for width_px in [1.0f32, 3.0, 6.0, 16.0, 40.0] {
        let max_r_doc = circle_fill_edge(ptw_fill, width_px, 0) * ptw_fill;
        for zoom in [1.0f32, 2.0, 4.0] {
            let ptw_view = ptw_fill / zoom;
            let axis_px = 250.0 * ptw_fill / ptw_view;
            let fill_px = max_r_doc / ptw_view;
            let overflow = fill_px - (axis_px + width_px * 0.5);
            let gap = (axis_px - width_px * 0.5) - fill_px;
            println!("{width_px:>5}px |  {zoom}x | {overflow:+20.1} | {gap:+12.1}");
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Trap (a bola) — wave COLORIZE, fatia C1. `docs/Flip/09_colorize.md`.
// ─────────────────────────────────────────────────────────────────────────────

/// Um quadrado com um **vão** de `gap` unidades no meio do lado de baixo: os dois
/// pedaços do lado inferior entram como traços ABERTOS separados.
fn square_with_gap(a: f32, b: f32, gap: f32) -> Vec<(Vec<Vec2>, Vec<f32>, bool)> {
    let mid = (a + b) * 0.5;
    let half = gap * 0.5;
    vec![
        // U invertido: sobe, atravessa, desce — um traço aberto só.
        (
            vec![
                Vec2::new(a, a),
                Vec2::new(a, b),
                Vec2::new(b, b),
                Vec2::new(b, a),
            ],
            vec![0.3; 4],
            false,
        ),
        // Os dois cotocos do lado de baixo, com o vão entre eles.
        (
            vec![Vec2::new(a, a), Vec2::new(mid - half, a)],
            vec![0.3; 2],
            false,
        ),
        (
            vec![Vec2::new(mid + half, a), Vec2::new(b, a)],
            vec![0.3; 2],
            false,
        ),
    ]
}

/// **A promessa da fatia C1, no ponto de entrada do balde** (não só na unidade da
/// bola): uma forma cujo contorno tem vão **vaza** com o balde de sempre e
/// **preenche** com o Trap ligado — sem tocar no Gap Closure.
///
/// O ramo `trap == 0` é o controle POSITIVO: sem ele o gate ficaria verde numa forma
/// que nunca teve vão, provando nada ([[feedback_a_negative_search_needs_a_positive_control]]).
#[test]
fn the_trap_fills_a_shape_whose_outline_has_a_gap() {
    let strokes = square_with_gap(0.0, 20.0, 1.2);
    let click = Vec2::new(10.0, 10.0);

    let plain = fill_at(&strokes, click, FillParams::default());
    assert_eq!(
        plain,
        Err(FillError::Leaked),
        "premissa do gate: com o Trap desligado esta forma TEM de vazar \
         (se ela fecha sozinha, a fixture nao contem o fenomeno)"
    );

    // Precision default = 4 px de buffer por unidade ⇒ o vão de 1,2 unidades tem ~4,8
    // px de buffer. Uma bola de raio 4 (diâmetro 8) não passa por ele.
    let trapped = fill_at(
        &strokes,
        click,
        FillParams {
            trap_px: 4.0,
            ..Default::default()
        },
    )
    .expect("com a bola, a forma com vao TEM de preencher");
    let area = signed_area(&trapped.outer).abs();
    assert!(
        (200.0..=450.0).contains(&area),
        "a regiao tem de ser o miolo do quadrado (~400), veio {area} \
         (uma area minuscula tambem 'nao vaza')"
    );
}

/// **A bola grande demais DIZ isso**, com um erro próprio — e não com o `Leaked`, cuja
/// mensagem manda o artista para o lado errado (subir o Gap Closure, quando o que ele
/// precisa é BAIXAR o Trap).
#[test]
fn a_ball_too_fat_for_the_click_says_so_instead_of_leaking() {
    let strokes = [square(0.0, 20.0, 0.3)];
    let err = fill_at(
        &strokes,
        Vec2::new(10.0, 10.0),
        FillParams {
            // Raio 200 px de buffer num miolo de ~80: não cabe de jeito nenhum.
            trap_px: 200.0,
            ..Default::default()
        },
    )
    .unwrap_err();
    assert_eq!(err, FillError::BallTooFat);
}

/// **O default é ZERO, e isso é o contrato da wave inteira.**
///
/// Todo o resto da suíte roda com `FillParams::default()`; se o default deixasse de ser
/// 0, os 38 gates do W4 passariam a medir o caminho da bola sem ninguém decidir isso —
/// e o comportamento que o Enio aprovou no smoke mudaria em silêncio.
#[test]
fn the_trap_is_off_by_default() {
    assert_eq!(FillParams::default().trap_px, 0.0);
}

/// 🔴 **O balde preenche uma caixa cujas QUINAS são traços separados — no default**
/// (BUGS #23, o item que a wave Colorize deixou nomeado para o balde).
///
/// Uma caixa desenhada à mão são QUATRO traços, e o tremor deixa as pontas sem coincidir:
/// medido nesta fixture, o vão entre os EIXOS das quinas é **0,0045 a 0,0404 doc**, contra
/// **0,26 de tinta** (2 × 0,13). Na tela: quinas fechadas com 6× de folga. No raster (que é
/// o EIXO, raio 0 — BUGS #14): um buraco.
///
/// ⚠️ **O sintoma não era vazar, era RECUSAR**, e piorava com a Precision: a 40 o balde
/// preenchia, e a partir de **80** devolvia `Leaked` — o artista lê *"raise Gap Closure to
/// seal the outline"* sobre uma quina que ele vê fechada, e **subir a Precision para melhorar
/// o contorno quebrava o balde**.
///
/// Os dois remédios existentes RESGATAVAM (medido: `gap_reach` a partir de 0,05, ou
/// `trap_px` 2) — mas os dois nascem **desligados**, então o default era a recusa. E mandar
/// fechar à mão o que a tinta já cobre é a ferramenta pedindo que se conserte o que não está
/// quebrado.
///
/// Este gate roda no **DEFAULT** (`gap_reach: 0`, `trap_px: 0`), que é onde o artista está,
/// e varre a faixa de Precision — porque o defeito é função dela.
///
/// Mutação que sangra: tirar o laço de `weld::welds` do raster (volta o `Leaked` a 80+).
#[test]
fn the_bucket_fills_a_box_whose_corners_are_separate_strokes() {
    let hand = |pts: &[Vec2], seed: usize| -> Vec<Vec2> {
        let h = |k: usize| ((k as u64).wrapping_mul(2_654_435_761) % 1000) as f32 / 1000.0 - 0.5;
        pts.iter()
            .enumerate()
            .map(|(i, p)| Vec2::new(p.x + h(i + seed) * 0.05, p.y + h(i + seed + 91) * 0.05))
            .collect()
    };
    let seg = |a: Vec2, b: Vec2, n: usize| -> Vec<Vec2> {
        (0..n)
            .map(|i| {
                let t = i as f32 / (n - 1) as f32;
                Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
            })
            .collect()
    };
    let strokes: Vec<(Vec<Vec2>, Vec<f32>, bool)> = [
        (Vec2::new(-4.0, -2.5), Vec2::new(4.0, -2.5), 0usize),
        (Vec2::new(4.0, -2.5), Vec2::new(4.0, 2.5), 7),
        (Vec2::new(4.0, 2.5), Vec2::new(-4.0, 2.5), 13),
        (Vec2::new(-4.0, 2.5), Vec2::new(-4.0, -2.5), 29),
    ]
    .into_iter()
    .map(|(a, b, s)| {
        let pts = hand(&seg(a, b, 24), s);
        let n = pts.len();
        (pts, vec![0.13; n], false)
    })
    .collect();

    // ── Controle positivo: a fixture CONTÉM o fenômeno. ──
    // As quinas têm de estar DESCOLADAS (senão não há buraco) e ainda assim cobertas pelo
    // corpo pintado (senão o vão é deliberado e a solda não deve — nem pode — fechá-lo).
    for i in 0..4 {
        let gap = (*strokes[i].0.last().expect("ponta") - strokes[(i + 1) % 4].0[0]).length();
        assert!(
            gap > 0.0 && gap < 0.26,
            "controle positivo: a quina {i} tem de estar descolada ({gap}) e coberta pela \
             tinta (2 x 0,13) — senão a fixture não contém o fenômeno"
        );
    }

    for precision in [40.0f32, 80.0, 160.0, 320.0] {
        let r = fill_at(
            &strokes,
            Vec2::new(0.0, 0.0),
            FillParams {
                precision,
                gap_reach: 0.0, // o DEFAULT: sem Gap Closure
                trap_px: 0.0,   // o DEFAULT: sem Trap
                ..Default::default()
            },
        );
        let f = r.unwrap_or_else(|e| {
            panic!(
                "precisão {precision}: o balde recusou ({e:?}) uma caixa cujas quinas o \
                 artista vê FECHADAS — e mandá-lo subir o Gap Closure é pedir que ele \
                 conserte o que não está quebrado (BUGS #23)"
            )
        });
        // Controle positivo #2: encheu de verdade. "Não recusou" fica verde com área ~0.
        let area = signed_area(&f.outer).abs();
        assert!(
            area > 30.0,
            "precisão {precision}: a caixa tem 40 unidades² e o preenchimento saiu com \
             {area:.1} — recusar e devolver um caco são a mesma falha"
        );
    }
}

/// 🔴 **A solda e o Gap Closure são DISJUNTOS: um vão DELIBERADO segue aberto.**
///
/// A regra da solda é *"a tinta que o artista pintou cobre o vão"*. Um C aberto — o esboço,
/// a forma que se deixa aberta de propósito — tem as pontas **longe** uma da outra, então ela
/// nunca o toca. Sem este gate a solda seria o remédio novo que torna o antigo contagem
/// dupla ([[feedback_a_new_remedy_makes_the_old_one_double_counting]]): o Gap Closure viraria
/// um knob que não muda nada, e ninguém notaria até alguém precisar dele.
///
/// Mutação que sangra: trocar o teste de alcance da solda por uma constante generosa — o C
/// fecha sozinho e o `Leaked` some.
#[test]
fn a_deliberate_gap_still_leaks_and_still_needs_the_gap_closure() {
    // Um quadrado com UM vão de 1,0 doc no lado direito — 4x a tinta que o cobriria. As
    // outras três quinas COINCIDEM (as pontas dos dois traços se tocam), então só há um vão
    // na figura e ele é o que está sob teste.
    let w = 0.13;
    let strokes: Vec<(Vec<Vec2>, Vec<f32>, bool)> = vec![
        (
            vec![
                Vec2::new(-2.0, -2.0),
                Vec2::new(2.0, -2.0),
                Vec2::new(2.0, -0.5),
            ],
            vec![w; 3],
            false,
        ),
        (
            vec![
                Vec2::new(2.0, 0.5),
                Vec2::new(2.0, 2.0),
                Vec2::new(-2.0, 2.0),
                Vec2::new(-2.0, -2.0),
            ],
            vec![w; 4],
            false,
        ),
    ];
    let params = |gap_reach: f32| FillParams {
        precision: 80.0,
        gap_reach,
        ..Default::default()
    };
    assert_eq!(
        fill_at(&strokes, Vec2::new(0.0, 0.0), params(0.0)).unwrap_err(),
        FillError::Leaked,
        "um vão de 1,0 doc contra 0,26 de tinta é DELIBERADO — a solda não pode fechá-lo, \
         senão o Gap Closure vira um knob que não muda nada"
    );
    // **A régua do slider é HONESTA (a dívida do 4× foi PAGA):** o vão é de 1,0 doc e o
    // `reach` que o fecha é **1,0** — o pareamento ponta-a-ponta (`gap.rs`, passe 3)
    // fechou o buraco do colinear (o `ray_hit` trata colinear como paralelo, então as
    // extensões destas duas pontas se atravessavam sem "colidir", e o vão só fechava a
    // reach 4,0, quando o raio alcançava a quina DISTANTE da caixa por acidente). O
    // artista que mede o próprio vão e digita esse número agora recebe o preenchimento.
    assert!(
        fill_at(&strokes, Vec2::new(0.0, 0.0), params(1.0)).is_ok(),
        "controle positivo: reach = o VAO fecha — o rotulo do slider e' verdadeiro"
    );
    // E abaixo do vão segue aberto: o slider não mente para o outro lado.
    assert_eq!(
        fill_at(&strokes, Vec2::new(0.0, 0.0), params(0.9)).unwrap_err(),
        FillError::Leaked,
        "reach menor que o vao nao fecha"
    );
}

/// 🔴 **O Trap sobrevive ao clamp de `MAX_SIDE`** (o aberto do doc 09: *"bola de 21,6 doc
/// a 10× de zoom"*). O raio do Trap é uma promessa na escala PEDIDA; num desenho grande a
/// grade cede resolução (`Grid::new` clampa o `scale`) e o raio CRU inflava a bola na
/// razão do clamp — aqui, 4 px a 80/doc prometem 0,05 doc, e crus na grade efetiva
/// (~2 px/doc) viram ~2 doc: MAIOR que o corredor, e o balde recusava com `BallTooFat`
/// um clique que na tela tem folga de sobra. Mutação que sangra: consumir `trap_px` cru
/// (voltar o `px_from_requested` para a identidade).
#[test]
fn the_trap_ball_survives_the_max_side_clamp() {
    // Corredor 2000 × 2 doc, selado: a precisão pedida (80) estoura o teto (2000·80 ≫
    // 4096) e a resolução cede para ~2 px/doc.
    let w = 0.1;
    let strokes: Vec<(Vec<Vec2>, Vec<f32>, bool)> = vec![
        (
            vec![Vec2::new(0.0, 0.0), Vec2::new(2000.0, 0.0)],
            vec![w; 2],
            false,
        ),
        (
            vec![Vec2::new(0.0, 2.0), Vec2::new(2000.0, 2.0)],
            vec![w; 2],
            false,
        ),
        (
            vec![Vec2::new(0.0, 0.0), Vec2::new(0.0, 2.0)],
            vec![w; 2],
            false,
        ),
        (
            vec![Vec2::new(2000.0, 0.0), Vec2::new(2000.0, 2.0)],
            vec![w; 2],
            false,
        ),
    ];
    let r = fill_at(
        &strokes,
        Vec2::new(1000.0, 1.0),
        FillParams {
            precision: 80.0,
            trap_px: 4.0,
            ..Default::default()
        },
    );
    assert!(
        r.is_ok(),
        "a bola PROMETIDA (4 px a 80/doc = 0,05 doc) cabe folgada no corredor de 2 doc — \
         a recusa e' a bola inflada pelo clamp: {:?}",
        r.err()
    );
}

/// 🔴 **A porta da conversão, contra números computados À MÃO da lei do clamp** (nunca
/// re-derivados da função — o oráculo que usa a função sob teste é sempre-verde).
///
/// Sem clamp: `scale` pedido cabe no budget ⇒ conversão é a IDENTIDADE, bit a bit.
/// Com clamp: bbox 2000 doc, margem 20, teto 4096 ⇒ budget = 4096 − 40 = 4056 px de
/// arte ⇒ escala efetiva = 4056/2000 = **2,028** ⇒ 4 px pedidos a 80/doc valem
/// `4 × 2,028/80 = 0,1014` px efetivos (a MESMA fração do documento: 0,05 doc).
#[test]
fn the_requested_px_door_follows_the_clamp_law() {
    // Sem clamp: 10 doc × 10 px/doc = 100 px ≪ 4096.
    let small = Grid::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0), 10.0, 20, 4096);
    assert_eq!(
        small.px_from_requested(4.0, 10.0),
        4.0,
        "sem clamp a conversao e' a identidade"
    );
    // Com clamp: os números acima, escritos à mão.
    let big = Grid::new(Vec2::new(0.0, 0.0), Vec2::new(2000.0, 2.0), 80.0, 20, 4096);
    assert!(
        (big.scale - 2.028).abs() < 1e-3,
        "a lei do clamp: budget 4056 / 2000 doc = 2,028 px/doc (saiu {})",
        big.scale
    );
    let eff = big.px_from_requested(4.0, 80.0);
    assert!(
        (eff - 0.1014).abs() < 1e-3,
        "4 px a 80/doc sao 0,05 doc = 0,1014 px efetivos (saiu {eff})"
    );
}

/// 🔴 **A porta do overlay É o passo 1 do clique** (doc 06 §8 — os helpers ao vivo).
///
/// O fixture é o vão CANÔNICO do BUGS #23 (duas metades colineares da mesma linha, vão
/// de 1,0): `preview_closures` no alcance exato devolve o par ponta-a-ponta — e a MESMA
/// lista que o `fill_at` materializa num clique dentro da caixa. Se as duas divergirem,
/// o artista vê um helper na tela e o clique fecha outro vão.
///
/// Mutação que sangra: `preview_closures` escalar o `reach` (p.ex. `reach * 0.5`) — o
/// helper some no alcance que o clique honra.
#[test]
fn the_preview_door_is_the_clicks_own_first_step() {
    // A caixa do §2.5: paredes que fecham uma célula, com o vão colinear de 1,0 na
    // parede direita (entre (2,-0.5) e (2,0.5)).
    let strokes = vec![
        (
            vec![Vec2::new(2.0, -2.0), Vec2::new(2.0, -0.5)],
            vec![0.1; 2],
            false,
        ),
        (
            vec![Vec2::new(2.0, 0.5), Vec2::new(2.0, 2.0)],
            vec![0.1; 2],
            false,
        ),
        (
            vec![
                Vec2::new(2.0, 2.0),
                Vec2::new(-2.0, 2.0),
                Vec2::new(-2.0, -2.0),
                Vec2::new(2.0, -2.0),
            ],
            vec![0.1; 4],
            false,
        ),
    ];
    let preview = preview_closures(&strokes, 1.0);
    assert!(
        preview.iter().any(|c| {
            let (lo, hi) = if c.a.y < c.b.y {
                (c.a, c.b)
            } else {
                (c.b, c.a)
            };
            (lo.y - -0.5).abs() < 1e-3 && (hi.y - 0.5).abs() < 1e-3
        }),
        "o helper do vao colinear tinha de existir no alcance exato: {preview:?}"
    );
    // Abaixo do vão, nenhum helper PARA AQUELE vão (o slider não mente).
    assert!(
        !preview_closures(&strokes, 0.9).iter().any(|c| {
            let (lo, hi) = if c.a.y < c.b.y {
                (c.a, c.b)
            } else {
                (c.b, c.a)
            };
            (lo.y - -0.5).abs() < 1e-3 && (hi.y - 0.5).abs() < 1e-3
        }),
        "alcance menor que o vao nao pode mostrar helper nele"
    );
    // E o clique materializa A MESMA lista — a tela e o clique não podem discordar.
    let r = fill_at(
        &strokes,
        Vec2::new(0.0, 0.0),
        FillParams {
            gap_reach: 1.0,
            ..Default::default()
        },
    )
    .expect("com o vao fechado pelo par, a caixa preenche");
    assert_eq!(
        r.closures, preview,
        "o fill_at tem de materializar exatamente o que o overlay mostrou"
    );
}

//! Os gates da **correspondência de um contorno SUAVE** — o segundo smoke do Enio.
//!
//! > *"o porquê da rotação?"* — um quadrado a caminho de um círculo **rodava 45°** e voltava.
//!
//! Estes gates moram num arquivo irmão do [`super::tests`] pelo teto de LOC (600).
//!
//! # A pergunta que eles fazem é *"o que é uma FEATURE?"*
//!
//! Uma âncora de um contorno suave **não é uma feature — é artefato da parametrização.** As 4
//! âncoras do círculo do catálogo existem porque a elipse é cozida em 4 cúbicas: o artista nunca
//! as autorou, e a virada delas é `(0, 1)` (medido). Não há nada ali para casar.
//!
//! Enquanto o motor **obrigava** âncora a casar com âncora, a resposta certa — a quina a 45° do
//! quadrado vai para o ponto a 45° do círculo, que fica **no MEIO de um segmento** — não estava
//! sequer no conjunto de candidatos. O melhor de quatro casamentos ruins é o giro de 45°.

use super::*;
use crate::matching::{SEARCH_COUNT, features, map_forward, search};
use ph2d_vec_scene::{ShapeKind, cook};

fn shape(kind: ShapeKind, c: [f64; 2], half: [f64; 2], params: &[f64]) -> VecPath {
    cook(
        kind,
        [c[0] - half[0], c[1] - half[1]],
        [c[0] + half[0], c[1] + half[1]],
        params,
    )
}

fn square(c: [f64; 2], r: f64) -> VecPath {
    shape(ShapeKind::Rectangle, c, [r, r], &[])
}

fn circle(c: [f64; 2], r: f64) -> VecPath {
    shape(ShapeKind::Ellipse, c, [r, r], &[])
}

/// O centro amostrado por arco (o "onde a forma está").
fn center(o: &Outline) -> Point {
    let (mut x, mut y) = (0.0, 0.0);
    for k in 0..256 {
        let p = o.at(f64::from(k) / 256.0);
        x += p.x;
        y += p.y;
    }
    Point::new(x / 256.0, y / 256.0)
}

/// O quanto duas direções (a partir do centro de cada forma) divergem, em **graus**.
///
/// A comparação é por seno do ângulo entre os unitários — o `atan2` aqui só serve para **relatar**
/// o número na unidade do usuário (é o que o `assert` imprime quando quebra).
fn angle_between(a: kurbo::Vec2, b: kurbo::Vec2) -> f64 {
    let (ua, ub) = (a / a.hypot(), b / b.hypot());
    let cross = ua.x * ub.y - ua.y * ub.x;
    let dot = ua.x * ub.x + ua.y * ub.y;
    cross.atan2(dot).to_degrees()
}

/// **O SMOKE DO ENIO: um quadrado NÃO GIRA a caminho de um círculo.**
///
/// O círculo é rotacionalmente simétrico: não existe informação nele que possa pedir uma rotação.
/// Logo a quina do quadrado a 45° **tem de** caminhar em linha reta para o ponto do círculo a 45°
/// — o mesmo raio, sem girar. Qualquer outra coisa é a correspondência inventando movimento que a
/// geometria não pediu.
///
/// Medido com o motor de âncora↔âncora: **+45,00° em todas as quatro quinas** (giro rígido).
///
/// A tolerância é de **2°** e é generosa de propósito: a fase é buscada num varrimento discreto,
/// e o que o gate protege é a CLASSE do defeito (um giro de 45°), não o último dígito.
#[test]
fn a_square_does_not_spin_on_its_way_to_a_circle() {
    let (a, b) = (square([0.0, 0.0], 1.0), circle([6.0, 0.0], 1.0));
    let (oa, ob) = (Outline::of(&a).unwrap(), Outline::of(&b).unwrap());
    let corr = search(&oa, &ob, BlendOpts::default());
    let target = if corr.reversed { ob.reversed() } else { ob };
    let (ca, cb) = (center(&oa), center(&target));

    for ua in oa.anchors() {
        let v = map_forward(&corr.knots, ua);
        let (pa, pb) = (oa.at(ua), target.at(v));
        let spin = angle_between(pa - ca, pb - cb);
        assert!(
            spin.abs() < 2.0,
            "a quina no arco {ua:.3} (a {:.1}° do centro do quadrado) foi casada com o ponto a \
             {:.1}° do centro do círculo — um giro de {spin:+.2}°. O círculo é rotacionalmente \
             simétrico: ele não pode PEDIR uma rotação. A resposta certa cai no MEIO de um \
             segmento dele, e enquanto os candidatos forem só as âncoras ela não existe.",
            angle_between(kurbo::Vec2::new(1.0, 0.0), pa - ca),
            angle_between(kurbo::Vec2::new(1.0, 0.0), pb - cb),
        );
    }
}

/// **E o giro também não pode aparecer na FORMA do meio** (o que o olho do Enio viu).
///
/// O gate acima mede a correspondência; este mede o **produto**. Um quadrado a meio caminho de um
/// círculo é um "quadrado arredondado": o ponto mais distante do centro ainda é uma quina, e ela
/// tem de estar onde a quina do quadrado estava — não 22° adiante.
///
/// Sem o gate do produto, uma correspondência certa com um corte errado passaria despercebida.
#[test]
fn the_middle_shape_keeps_the_orientation_of_the_square() {
    let (a, b) = (square([0.0, 0.0], 1.0), circle([6.0, 0.0], 1.0));

    for step in 1..=5 {
        let t = f64::from(step) / 10.0; // até t=0,5: além disso a quina já derreteu no círculo
        let m = morph(&a, &b, t, BlendOpts::default()).expect("o passo");
        let o = Outline::of(&m).unwrap();
        let c = center(&o);

        // O ponto mais distante do centro: numa forma com quina, é a quina.
        let far = (0..512)
            .map(|k| o.at(f64::from(k) / 512.0))
            .max_by(|p, q| {
                (*p - c)
                    .hypot()
                    .partial_cmp(&(*q - c).hypot())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .expect("a forma tem pontos");

        // As quinas do quadrado estão a ±45° e ±135°. A quina mais distante da forma do meio tem
        // de estar sobre UMA delas (a simetria de 4 dobras torna as quatro equivalentes).
        let spin = (0..4)
            .map(|q| {
                let ang = -135.0 + 90.0 * f64::from(q);
                let (s, c2) = (ang.to_radians().sin(), ang.to_radians().cos());
                angle_between(far - c, kurbo::Vec2::new(c2, s)).abs()
            })
            .fold(f64::MAX, f64::min);
        assert!(
            spin < 3.0,
            "em t={t} a quina da forma do meio está {spin:.2}° fora do eixo das quinas do \
             quadrado — a forma GIROU no caminho, e o círculo não pediu isso"
        );
    }
}

/// **O que é uma FEATURE: a virada, não a existência da âncora.**
///
/// É a espinha do conserto, e ela merece um gate próprio — porque é uma afirmação sobre o
/// CATÁLOGO, e o catálogo muda.
#[test]
fn only_a_real_corner_is_a_candidate_node() {
    let cases: [(&str, VecPath, usize, usize); 4] = [
        // (nome, forma, âncoras, features esperadas)
        ("quadrado", square([0.0, 0.0], 1.0), 4, 4),
        ("círculo", circle([0.0, 0.0], 1.0), 4, 0),
        (
            "estrela",
            shape(ShapeKind::Star, [0.0, 0.0], [1.0, 1.0], &[5.0, 0.45, 0.0]),
            10,
            10,
        ),
        (
            "coração",
            shape(ShapeKind::Heart, [0.0, 0.0], [1.0, 1.0], &[]),
            2,
            2,
        ),
    ];
    for (name, p, anchors, want) in &cases {
        let o = Outline::of(p).unwrap();
        assert_eq!(
            o.segs.len(),
            *anchors,
            "{name}: mudou a contagem de âncoras"
        );
        let f = features(&o);
        assert_eq!(
            f.len(),
            *want,
            "{name}: {} âncoras, mas {} features (esperava {want}) — uma âncora de contorno SUAVE \
             não é uma feature, é artefato da parametrização; e uma quina de verdade NÃO pode ser \
             perdida",
            anchors,
            f.len()
        );
    }
}

/// **A fase é O NÚMERO — não o ponto de grade mais próximo dele.**
///
/// O varrimento testa 256 fases; a resposta certa quase nunca é uma delas. O refino parabólico (as
/// três amostras em volta do mínimo) é o que transforma "o melhor ponto da grade" na fase de
/// verdade — e o oráculo aqui é **exato e derivável**: um círculo rodado de θ tem de ser casado
/// com a fase `−θ/360`, e nada mais.
///
/// Medido: **erro ≤ 3,2e-8** com o refino, **até 1,8e-3 (0,66° de giro)** sem ele — e o erro sem o
/// refino é **sistemático**, não ruído: ele é a distância até a grade, e vira "a forma treme" no
/// dia em que o `t` for animado ([[feedback_loose_oracle_hides_systematic_bias]]).
///
/// **Os ângulos são FORA DA GRADE, de propósito.** Em θ = 45° a resposta cai exatamente sobre uma
/// amostra (0,875 × 256 = 224), o erro é **zero mesmo sem o refino**, e o gate ficaria verde com o
/// defeito ligado — uma fixture que não pode falhar
/// ([[feedback_zero_valued_fixture_is_a_gate_that_cannot_fail]]).
#[test]
fn the_phase_is_the_number_not_the_nearest_sample_of_the_grid() {
    for deg in [10.0, 33.0, 77.5, 123.4_f64] {
        let a = circle([0.0, 0.0], 1.0);
        let mut b = circle([5.0, 0.0], 1.0);
        let (s, co) = (deg.to_radians().sin(), deg.to_radians().cos());
        let rot = |q: [f64; 2]| {
            let (dx, dy) = (q[0] - 5.0, q[1]);
            [5.0 + dx * co - dy * s, dx * s + dy * co]
        };
        for v in &mut b.verts {
            v.anchor = rot(v.anchor);
            v.in_handle = rot(v.in_handle);
            v.out_handle = rot(v.out_handle);
        }

        let (oa, ob) = (Outline::of(&a).unwrap(), Outline::of(&b).unwrap());
        let corr = search(&oa, &ob, BlendOpts::default());
        assert_eq!(
            corr.knots.len(),
            1,
            "duas formas SEM quina não têm nó nenhum a casar: a correspondência é uma fase só"
        );
        let (got, want) = (corr.knots[0].1, 1.0 - deg / 360.0);
        let d = (got - want).abs();
        let err = d.min(1.0 - d);
        assert!(
            err < 1e-5,
            "o círculo está rodado de {deg}°, então a fase certa é {want:.6} — o motor achou \
             {got:.6} (erro {err:.2e} = {:.3}° de giro). A grade tem passo 1/256: sem o refino, a \
             fase fica presa nela.",
            err * 360.0
        );
    }
}

/// **Um círculo caminhando para um círculo não gira, não encolhe e não treme.**
///
/// O caso **degenerado** é o mais importante (nenhuma das duas formas tem feature): não há nó
/// nenhum a casar, e a correspondência é uma **fase contínua** pura. Se ela for buscada só entre
/// as âncoras, duas elipses com parametrizações desalinhadas giram uma contra a outra.
///
/// Aqui o círculo B nasce **rodado** (as âncoras dele caem em posições de arco que não são as de
/// A). Sob a busca só-nas-âncoras, a melhor fase disponível erra — e o meio do caminho deixa de
/// ser um círculo do mesmo raio.
#[test]
fn a_circle_walking_to_a_rotated_circle_stays_a_circle() {
    let a = circle([0.0, 0.0], 1.0);
    // O MESMO círculo, com a origem do percurso deslocada de 1/8 de volta (45°): mesma geometria,
    // parametrização diferente. É o que acontece quando o artista desenha o segundo à mão.
    let mut b = circle([5.0, 0.0], 1.0);
    let rot = |p: [f64; 2], c: [f64; 2]| {
        let (dx, dy) = (p[0] - c[0], p[1] - c[1]);
        let (s, co) = (45f64.to_radians().sin(), 45f64.to_radians().cos());
        [c[0] + dx * co - dy * s, c[1] + dx * s + dy * co]
    };
    for v in &mut b.verts {
        v.anchor = rot(v.anchor, [5.0, 0.0]);
        v.in_handle = rot(v.in_handle, [5.0, 0.0]);
        v.out_handle = rot(v.out_handle, [5.0, 0.0]);
    }

    let mid = morph(&a, &b, 0.5, BlendOpts::default()).expect("o meio");
    let o = Outline::of(&mid).unwrap();
    let c = center(&o);
    let (mut lo, mut hi) = (f64::MAX, 0.0f64);
    for k in 0..256 {
        let r = (o.at(f64::from(k) / 256.0) - c).hypot();
        lo = lo.min(r);
        hi = hi.max(r);
    }
    // Um círculo interpolado com um círculo do mesmo raio é um círculo do mesmo raio. Se a fase
    // estiver errada, os pontos cruzam o disco e o meio ENCOLHE (no pior caso, colapsa num ponto).
    assert!(
        lo > 0.98 && hi < 1.02,
        "o meio de dois círculos de raio 1 tem raio entre {lo:.4} e {hi:.4} — a fase está errada e \
         os pontos estão atravessando o disco em vez de caminhar radialmente"
    );
}

/// **A correspondência é buscada UMA vez por blend — não uma por passo.**
///
/// Ela é função do par `(A, B, opções)`; o `t` não entra nela. Enquanto a busca morava dentro do
/// `morph`, um blend de 10 passos a repetia **dez vezes** e chegava dez vezes à mesma resposta —
/// e depois que o varrimento de fase entrou (256 fases × 256 amostras) a conta virou **5,9 ms**
/// por blend, contra 0,18 ms do caminho da DP. O artista re-roda o blend a cada frame enquanto
/// arrasta o slider de Steps.
///
/// O gate **conta**, e não cronometra: um cronômetro mede a máquina; o contador mede o código
/// ([[feedback_an_optimization_needs_a_gate_that_proves_it_fires]]).
#[test]
fn the_correspondence_is_searched_once_per_blend_not_once_per_step() {
    let (a, b) = (square([0.0, 0.0], 1.0), circle([6.0, 0.0], 1.0));

    SEARCH_COUNT.with(|c| c.set(0));
    let out = steps(&a, &b, 12, BlendOpts::default());
    let searches = SEARCH_COUNT.with(std::cell::Cell::get);

    assert_eq!(out.len(), 12, "os 12 passos");
    assert_eq!(
        searches, 1,
        "12 passos custaram {searches} buscas de correspondência — ela não depende do `t`, e \
         repeti-la por passo é trabalho jogado fora (o caminho da fase varre 256 fases × 256 \
         amostras). Monte o `Plan` uma vez e avalie cada `t` nele."
    );

    // E o plano avaliado em `t` tem de dar EXATAMENTE o que o `morph` daria — senão o hoist mudou
    // o produto, e não só o custo.
    let plan = Plan::new(&a, &b, BlendOpts::default()).expect("o plano");
    for i in 1..=12 {
        let t = f64::from(i) / 13.0;
        let direct = morph(&a, &b, t, BlendOpts::default()).expect("o morph direto");
        let from_plan = plan.at(t);
        assert_eq!(
            from_plan.verts.len(),
            direct.verts.len(),
            "t={t}: o plano e o morph direto discordam na contagem de vértices"
        );
        for (k, (u, v)) in from_plan.verts.iter().zip(&direct.verts).enumerate() {
            assert!(
                (u.anchor[0] - v.anchor[0]).abs() < 1e-12
                    && (u.anchor[1] - v.anchor[1]).abs() < 1e-12,
                "t={t}: o vértice {k} do plano ({:?}) não é o do morph direto ({:?})",
                u.anchor,
                v.anchor
            );
        }
    }
}

/// **A MESMA forma, com âncoras a mais, blenda IGUAL.**
///
/// Picar uma aresta reta em 20 pedaços não muda **nada** do que se vê — mas mudava a
/// correspondência. Medido, quadrado → estrela: as quinas casavam com os vértices
/// `(0,4 · 0,6 · 0,8 · 0,2)` da estrela; com uma aresta subdividida em 5, o último virava `0,0`;
/// com 20, a correspondência inteira embaralhava (`0,4 · 0,8 · 0,0 · 0,2`).
///
/// A causa era o **referencial**: o centro e a escala que normalizam o custo saíam da **média das
/// âncoras**, e âncora é parametrização — os 20 pontos novos arrastavam a média para o lado da
/// aresta picada. Agora o quadro sai de amostras equiespaçadas em **arco**, que descrevem o
/// contorno e não a lista de pontos.
///
/// Não é um caso de laboratório: todo caminho traçado, importado ou passado por um `Simplify` tem
/// âncoras onde o algoritmo as deixou. **Duas formas que se veem iguais têm de blendar igual.**
#[test]
fn extra_anchors_on_a_straight_edge_do_not_change_the_correspondence() {
    let a = square([0.0, 0.0], 1.0);
    let b = shape(ShapeKind::Star, [6.0, 0.0], [1.0, 1.0], &[5.0, 0.45, 0.0]);
    let base = search(
        &Outline::of(&a).unwrap(),
        &Outline::of(&b).unwrap(),
        BlendOpts::default(),
    );

    for k in [2usize, 5, 20] {
        // O MESMO quadrado, com a 1ª aresta picada em `k` pedaços: geometria idêntica, âncoras a
        // mais. Elas são colineares, então NENHUMA delas é uma quina — e o motor não pode se
        // importar com elas.
        let mut chopped = a.clone();
        let (p, q) = (a.verts[0].anchor, a.verts[1].anchor);
        let mut verts = vec![a.verts[0]];
        for i in 1..k {
            let f = f64::from(u32::try_from(i).unwrap()) / f64::from(u32::try_from(k).unwrap());
            let at = [p[0] + (q[0] - p[0]) * f, p[1] + (q[1] - p[1]) * f];
            verts.push(VecVertex {
                anchor: at,
                in_handle: at,
                out_handle: at,
                kind: VertexKind::Corner,
                corner_radius: 0.0,
            });
        }
        verts.extend_from_slice(&a.verts[1..]);
        chopped.verts = verts;

        let got = search(
            &Outline::of(&chopped).unwrap(),
            &Outline::of(&b).unwrap(),
            BlendOpts::default(),
        );
        assert_eq!(
            got.knots.len(),
            base.knots.len(),
            "com a aresta em {k} pedaços a correspondência mudou de tamanho ({} nós contra {})",
            got.knots.len(),
            base.knots.len()
        );
        for (i, (g, w)) in got.knots.iter().zip(&base.knots).enumerate() {
            assert!(
                (g.0 - w.0).abs() < 1e-9 && (g.1 - w.1).abs() < 1e-9,
                "com a aresta em {k} pedaços, o nó {i} virou {g:?} (era {w:?}) — os pontos novos \
                 são COLINEARES: eles não mudam a forma, e não podem mudar com quem ela casa"
            );
        }
    }
}

/// **O LIMIAR NÃO SENTA EM CIMA DE NENHUM POLÍGONO QUE O CATÁLOGO SAIBA FAZER.**
///
/// A 1ª versão usou `cos(15°)` — e um polígono regular de **24 lados** vira exatamente 15°. A
/// comparação é estrita, o `f64` erra o cosseno no último bit, e cada uma das 24 quinas **idênticas**
/// caía de um lado ou do outro conforme o arredondamento: **68 resultados distintos** sob ruído de
/// `1e-13`, e **5,5% de mudança de área só transladando a cena**.
///
/// Um polígono regular de N lados vira `360/N`. O limiar tem de morar onde **nenhum N inteiro
/// chega** — e este gate **enumera** o catálogo inteiro (`MAX_POLYGON_SIDES = 128`) para dizer isso
/// na cara de quem mexer no número.
///
/// A margem exigida (`1e-6` em cosseno) é ~10 ordens de grandeza acima do ruído do `f64` e ~3 abaixo
/// da distância real (0,0017 até o 22-gon): ela recusa o empate sem fingir precisão que não existe.
#[test]
fn the_corner_threshold_does_not_sit_on_a_polygon_the_catalog_can_make() {
    for n in 3..=128u32 {
        // A virada de um polígono regular de `n` lados é o ângulo externo, `360/n`.
        let turn = (360.0 / f64::from(n)).to_radians();
        let gap = (turn.cos() - crate::matching::FEATURE_TURN_COS).abs();
        assert!(
            gap > 1e-6,
            "o polígono de {n} lados vira {:.4}° e o limiar está a {gap:.2e} dele — as {n} quinas \
             IDÊNTICAS dele vão cair dos dois lados da cerca conforme o último bit do `f64`, e o \
             blend passa a mudar quando o artista só MOVE a arte. Um limiar mora onde o domínio é \
             vazio: escolha um ângulo cujo 360/θ não seja inteiro.",
            turn.to_degrees()
        );
    }
}

/// **MOVER a arte pela tela não pode mudar o blend.**
///
/// É o gate do artista, e ele é o oráculo do defeito acima: uma translação **pura** não deforma
/// nada — se o intermediário muda, alguma decisão do motor está sendo tomada por ruído de `f64`, e
/// não pela forma.
///
/// A fixture é o **24-gon** de propósito: é a forma que sentava exatamente em cima do limiar antigo
/// (virada de 15,00°). Num polígono qualquer o gate ficaria verde com o defeito ligado
/// ([[feedback_a_gate_only_proves_what_its_fixture_contains]]).
///
/// Medido com `cos(15°)`: **5,50%** de mudança de área. Aqui a tolerância é `1e-9` — o intermediário
/// tem de ser a MESMA forma, transladada.
#[test]
fn moving_the_art_across_the_canvas_does_not_change_the_blend() {
    let poly = |c: [f64; 2]| shape(ShapeKind::Polygon, c, [1.0, 1.0], &[24.0, 0.0]);
    let sq = |c: [f64; 2]| square(c, 1.0);

    let reference: Vec<[f64; 2]> = {
        let m = morph(
            &poly([0.0, 0.0]),
            &sq([5.0, 0.0]),
            0.5,
            BlendOpts::default(),
        )
        .expect("o meio");
        let o = Outline::of(&m).unwrap();
        let c = center(&o);
        (0..128)
            .map(|k| {
                let p = o.at(f64::from(k) / 128.0);
                [p.x - c.x, p.y - c.y] // relativo ao próprio centro: a translação sai da conta
            })
            .collect()
    };

    for d in [1.0, 7.5, 33.25, 100.0] {
        let m = morph(&poly([d, d]), &sq([5.0 + d, d]), 0.5, BlendOpts::default())
            .expect("o meio, deslocado");
        let o = Outline::of(&m).unwrap();
        let c = center(&o);
        for (k, want) in reference.iter().enumerate() {
            let p = o.at(f64::from(u32::try_from(k).unwrap()) / 128.0);
            let (dx, dy) = (p.x - c.x - want[0], p.y - c.y - want[1]);
            assert!(
                dx.hypot(dy) < 1e-9,
                "com a cena deslocada de {d}, o ponto {k} do intermediário saiu {:.2e} do lugar — \
                 uma TRANSLAÇÃO não deforma nada: se o blend mudou, quem decidiu foi o ruído do \
                 `f64`, não a forma",
                dx.hypot(dy)
            );
        }
    }
}

/// **Um `offset` gigante não PANICA e não faz lixo** (achado adversarial: overflow em `i32`).
///
/// `BlendOpts.offset` é campo público. `c_auto + offset` estourava o `i32` (pânico em debug, wrap
/// silencioso em release) quando o offset chegava perto do teto. A soma é em `i64` agora; o
/// `rem_euclid` traz de volta para um índice válido.
#[test]
fn a_giant_offset_neither_panics_nor_corrupts() {
    let a = square([0.0, 0.0], 1.0);
    let b = shape(ShapeKind::Star, [6.0, 0.0], [1.0, 1.0], &[5.0, 0.45, 0.0]);
    for offset in [i32::MAX, i32::MIN, i32::MAX - 1, 1_000_000, -1_000_000] {
        let opts = BlendOpts { offset };
        let m = morph(&a, &b, 0.5, opts).expect("o meio");
        for v in &m.verts {
            assert!(
                v.anchor[0].is_finite() && v.anchor[1].is_finite(),
                "offset={offset}: a correspondência produziu um vértice não-finito"
            );
        }
    }
}

/// **Um `t` NaN devolve A — não uma forma de vértices NaN** (achado adversarial: `clamp` propaga
/// NaN).
///
/// Não é alcançável por `steps()` (`i/(n+1)` é sempre são), mas **é a API do morph vivo**: o `t`
/// virá de uma curva animada, e uma singularidade nela não pode sujar a cena e o save com NaN.
#[test]
fn a_nan_t_yields_a_not_a_shape_of_nans() {
    let a = square([0.0, 0.0], 1.0);
    let b = circle([6.0, 0.0], 1.0);
    let plan = Plan::new(&a, &b, BlendOpts::default()).expect("o plano");
    let at_nan = plan.at(f64::NAN);
    let at_zero = plan.at(0.0);
    assert_eq!(
        at_nan.verts.len(),
        at_zero.verts.len(),
        "t=NaN devia devolver A (o mesmo que t=0)"
    );
    for v in &at_nan.verts {
        assert!(
            v.anchor[0].is_finite() && v.anchor[1].is_finite(),
            "t=NaN vazou NaN para a geometria — `clamp` propaga NaN, e o morph vivo chama isto por frame"
        );
    }
}

/// **O quantum do Rotate vem das QUINAS, não das âncoras da forma lisa** (o smoke do Enio:
/// estrela → círculo, Rotate/Reverse "estranhos").
///
/// Quando um lado é suave (o círculo, 0 features), não há nós discretos a ciclar — girar só torce.
/// O quantum era `1/âncoras-do-círculo = 1/4 = 90°`, e o 2º toque (180°) **colapsava** a forma. As
/// quinas da estrela (10) são o que o artista percebe como pontos de ajuste, e dão 36°/toque —
/// passos finos, a torção cresce em vez de colapsar de uma vez.
///
/// O gate mede a fase: um toque de Rotate desloca a correspondência por `1/10`, não por `1/4`.
#[test]
fn rotate_steps_by_the_corners_not_by_the_smooth_shapes_anchors() {
    let star = shape(ShapeKind::Star, [0.0, 0.0], [1.0, 1.0], &[5.0, 0.45, 0.0]);
    let circ = circle([6.0, 0.0], 1.0);
    let (oa, ob) = (Outline::of(&star).unwrap(), Outline::of(&circ).unwrap());
    assert_eq!(features(&oa).len(), 10, "a estrela tem 10 quinas");
    assert_eq!(features(&ob).len(), 0, "o círculo é liso — 0 quinas");
    assert_eq!(ob.segs.len(), 4, "e 4 âncoras (o quantum ERRADO, de antes)");

    let phi = |off: i32| search(&oa, &ob, BlendOpts { offset: off }).knots[0].1;
    let step = (phi(1) - phi(0)).rem_euclid(1.0);
    // 1/10 = 0,1 (as quinas da estrela), NÃO 1/4 = 0,25 (as âncoras do círculo).
    assert!(
        (step - 0.1).abs() < 1e-9,
        "um toque de Rotate deslocou a fase em {step:.4} (= {:.0}°) — devia ser 1/10 = 36° (as 10 \
         quinas da estrela), não 1/4 = 90° (as 4 âncoras arbitrárias do círculo), que colapsava a \
         forma no 2º toque",
        step * 360.0
    );

    // **O OUTRO SENTIDO: círculo → estrela** (agora a forma com quinas é a B). O quantum tem de ser
    // o MESMO 1/10 — o `max(features_a, features_b)` é simétrico, e o fallback nas âncoras da lisa
    // não pode vazar quando o outro lado tem quinas. Sem este par, a fixture só provaria o caso
    // A-tem-quinas ([[feedback_a_gate_only_proves_what_its_fixture_contains]]).
    let (ob2, oa2) = (Outline::of(&star).unwrap(), Outline::of(&circ).unwrap());
    let phi2 = |off: i32| search(&oa2, &ob2, BlendOpts { offset: off }).knots[0].1;
    let step2 = (phi2(1) - phi2(0)).rem_euclid(1.0);
    assert!(
        (step2 - 0.1).abs() < 1e-9,
        "círculo → estrela deu quantum {step2:.4} (= {:.0}°) — o quantum tem de vir das quinas \
         seja qual for o lado que as tem; aqui a estrela é a B",
        step2 * 360.0
    );
}

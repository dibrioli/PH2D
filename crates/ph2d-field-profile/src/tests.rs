//! Os gates da costura.
//!
//! ⚠️ Uma conversão entre dois documentos falha **em silêncio**: o sólido sai *quase* como o
//! desenho, e ninguém sabe dizer qual das duas pontas mentiu. Cada gate aqui afirma uma coisa que,
//! quebrada, produz exatamente esse "quase".

use super::*;
use ph2d_vec_scene::VertexKind;

/// A constante de Bézier que aproxima um quarto de círculo — `4/3·(√2 − 1)`.
const KAPPA: f64 = 0.552_284_749_830_793_4;

fn path_of(verts: Vec<VecVertex>) -> VecPath {
    VecPath {
        verts,
        closed: true,
        ..VecPath::default()
    }
}

fn square(a: f64) -> VecPath {
    path_of(
        [[-a, -a], [a, -a], [a, a], [-a, a]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
    )
}

/// Círculo de raio `r` em quatro arcos cúbicos, anti-horário.
fn circle(r: f64) -> VecPath {
    let k = r * KAPPA;
    path_of(vec![
        VecVertex {
            anchor: [r, 0.0],
            in_handle: [r, -k],
            out_handle: [r, k],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [0.0, r],
            in_handle: [k, r],
            out_handle: [-k, r],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [-r, 0.0],
            in_handle: [-r, k],
            out_handle: [-r, -k],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
        VecVertex {
            anchor: [0.0, -r],
            in_handle: [-k, -r],
            out_handle: [k, -r],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        },
    ])
}

/// ⭐ **Um quadrado coze em QUATRO pontos** — nem cinco, nem dezasseis.
///
/// Cinco seria o primeiro ponto repetido no fim (uma aresta de comprimento zero, que é uma divisão
/// por zero na distância ponto-segmento). Dezasseis seria mandar a reta ao achatador como cúbica
/// degenerada e ele decidir subdividir. Os dois são invisíveis na forma e caríssimos no traçado.
#[test]
fn a_square_cooks_to_exactly_four_points() {
    let p = cook_path(&square(1.0), 1e-3).expect("quadrado é perfil válido");
    assert_eq!(p.contours().len(), 1);
    assert_eq!(
        p.segment_count(),
        4,
        "um quadrado tem quatro arestas; saiu com {} — {:?}",
        p.segment_count(),
        p.contours()[0]
    );
}

/// Distância de um ponto ao segmento `a—b`.
fn point_to_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let e = [b[0] - a[0], b[1] - a[1]];
    let w = [p[0] - a[0], p[1] - a[1]];
    let ee = e[0] * e[0] + e[1] * e[1];
    let t = if ee > 0.0 {
        ((w[0] * e[0] + w[1] * e[1]) / ee).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (w[0] - t * e[0]).hypot(w[1] - t * e[1])
}

/// **O achatamento entrega a tolerância que declara.**
///
/// Não se confia na promessa da biblioteca: mede-se — a distância de cada ponto da **curva** à
/// polilinha que a substitui.
///
/// # ⚠️ O oráculo não é o círculo, e essa distinção custou um vermelho
///
/// A primeira versão deste gate media contra o **círculo verdadeiro** e reprovava a 10⁻⁴ com uma
/// flecha de 1,86·10⁻⁴. Não era o achatador: um círculo feito de quatro cúbicas com `κ = 0,5523`
/// **já é ~2,7·10⁻⁴ diferente do círculo** por construção, e abaixo dessa ordem o erro medido era
/// quase todo da fonte, não da conversão. *Um oráculo que aproxima o que mede deixa de ser oráculo
/// exatamente quando a tolerância desce até ele.*
///
/// O que o achatamento promete — e portanto o que se afirma aqui — é ficar a menos de `ε` **da
/// curva que lhe deram**.
#[test]
fn the_flattening_honours_the_tolerance_it_declares() {
    let r = 1.0_f64;
    // A MESMA curva, montada aqui de forma independente: um gate que pedisse a curva ao código sob
    // teste estaria a comparar a conversão consigo própria.
    let k = r * KAPPA;
    let mut source = BezPath::new();
    source.move_to(Point::new(r, 0.0));
    source.curve_to(Point::new(r, k), Point::new(k, r), Point::new(0.0, r));
    source.curve_to(Point::new(-k, r), Point::new(-r, k), Point::new(-r, 0.0));
    source.curve_to(Point::new(-r, -k), Point::new(-k, -r), Point::new(0.0, -r));
    source.curve_to(Point::new(k, -r), Point::new(r, -k), Point::new(r, 0.0));
    source.close_path();
    let arcs: Vec<kurbo::CubicBez> = source
        .segments()
        .filter_map(|s| match s {
            kurbo::PathSeg::Cubic(c) => Some(c),
            _ => None,
        })
        .collect();
    assert_eq!(arcs.len(), 4, "o círculo de referência tem quatro arcos");

    for tol in [1e-2_f64, 1e-3, 1e-4] {
        let p = cook_path(&circle(r), tol).expect("círculo é perfil válido");
        let c: Vec<[f64; 2]> = p.contours()[0]
            .iter()
            .map(|q| [f64::from(q[0]), f64::from(q[1])])
            .collect();

        let mut worst = 0.0_f64;
        for arc in &arcs {
            for i in 0..=400 {
                let t = f64::from(i) / 400.0;
                let pt = kurbo::ParamCurve::eval(arc, t);
                let d = (0..c.len())
                    .map(|j| point_to_segment([pt.x, pt.y], c[j], c[(j + 1) % c.len()]))
                    .fold(f64::INFINITY, f64::min);
                worst = worst.max(d);
            }
        }
        assert!(
            worst <= tol,
            "tolerância {tol:e}: a curva afasta-se {worst:e} da polilinha, em {} arestas",
            c.len()
        );
        // E não está a subdividir muito mais do que precisa: a conta fechada para um arco de raio
        // `R` é `n ≈ 2,22·√(R/ε)`, e o dobro disso é folga generosa.
        let expected = 2.22 * (r / tol).sqrt();
        assert!(
            (c.len() as f64) < 2.0 * expected,
            "tolerância {tol:e}: {} arestas contra as ~{expected:.0} que a geometria pede",
            c.len()
        );
    }
}

/// ⭐ **O raio vivo de quina do editor vetorial CHEGA ao sólido.**
///
/// É a prova de que o cozimento parte de `cooked()` e não da fonte. Se alguém trocar por
/// `path.verts`, o quadrado volta a ter quatro pontos e a quina arredondada desaparece do sólido —
/// sem erro nenhum, e com o editor a mostrar a quina redonda na tela.
#[test]
fn a_live_corner_radius_reaches_the_profile() {
    let mut p = square(1.0);
    for v in &mut p.verts {
        v.corner_radius = 0.4;
    }
    let cooked = cook_path(&p, 1e-3).expect("quadrado com quina viva");
    assert!(
        cooked.segment_count() > 4,
        "a quina viva tem de virar arco no perfil; saiu com {} pontos",
        cooked.segment_count()
    );
    // E o vértice afiado deixou de existir: nada fica a menos de ~0,1 da quina (1, 1).
    let nearest = cooked.contours()[0]
        .iter()
        .map(|q| (f64::from(q[0]) - 1.0).hypot(f64::from(q[1]) - 1.0))
        .fold(f64::INFINITY, f64::min);
    assert!(
        nearest > 0.1,
        "a quina afiada sobreviveu ao cozimento: há ponto a {nearest:.3} de (1, 1)"
    );
}

/// ⚠️ Um contorno **aberto** é recusado alto.
///
/// Ignorá-lo em silêncio daria um sólido que é quase o desenho — e a diferença apareceria como uma
/// parede que não fechou, três waves depois de alguém desenhar um path aberto sem reparar.
#[test]
fn an_open_contour_is_refused_not_skipped() {
    let mut p = square(1.0);
    p.closed = false;
    assert_eq!(
        cook_path(&p, 1e-3),
        Err(CookError::OpenContour { contour: 0 }),
        "contorno aberto tem de ser recusado"
    );
}

/// ⚠️ **A conversão não espelha o Y.** É um pino, não uma descoberta: se um dia alguém precisar do
/// espelho, ele pertence à ferramenta que escolhe o plano de desenho — e este gate é onde a decisão
/// fica escrita em vez de virar um `-y` perdido numa linha.
#[test]
fn the_cook_does_not_flip_the_y_axis() {
    // Triângulo com a base em y = 0 e o topo em y = 2 — assimétrico de propósito.
    let tri = path_of(
        [[-1.0, 0.0], [1.0, 0.0], [0.0, 2.0]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
    );
    let p = cook_path(&tri, 1e-3).expect("triângulo");
    let (min, max) = p.bounds();
    assert!(
        (min[1] - 0.0).abs() < 1e-6 && (max[1] - 2.0).abs() < 1e-6,
        "o topo tem de continuar em +2 e a base em 0; saiu ({}, {})",
        min[1],
        max[1]
    );
}

/// ⭐ **A tolerância automática é uma FRAÇÃO, e é isso que faz a mesma forma sair igual em qualquer
/// unidade.**
///
/// A mesma peça desenhada em milímetros e em metros tem de dar o mesmo número de arestas. Com uma
/// tolerância absoluta, mudar a unidade do documento mudaria a suavidade da forma **e** o custo do
/// traçado — 1000× em cada direção.
#[test]
fn the_automatic_tolerance_follows_the_size_of_the_drawing() {
    let small = cook_path_auto(&circle(0.01)).expect("círculo pequeno");
    let big = cook_path_auto(&circle(100.0)).expect("círculo grande");
    assert_eq!(
        small.segment_count(),
        big.segment_count(),
        "a mesma forma em escalas 10.000× diferentes tem de dar o mesmo nº de arestas: {} vs {}",
        small.segment_count(),
        big.segment_count()
    );
}

/// ⭐⭐ **A SUAVIDADE É MEDIDA PELA NORMAL, NÃO PELA SILHUETA** (W54).
///
/// # O smoke que reescreveu a régua
///
/// Enio, 2026-08-23, com duas fotos de peças de perfil: *"contudo sem ajustes de resolução"* — a
/// **silhueta lisa** e a **luz em degraus**. A tolerância que shipava (`1e-3`) errava a silhueta em
/// **0,079 % da peça** (invisível) e partia a normal em **6,43°** (muito visível).
///
/// ⚠️ **O gate que estava aqui prendia a régua errada:** ele exigia que o círculo caísse num
/// *orçamento de arestas* (24..=80), que é a grandeza do **custo**, e nada dizia sobre o que se vê.
/// Um gate assim defende o número velho contra a medição que o desmente.
///
/// ⭐ A lei é o **ângulo entre facetas**: num círculo de `N` arestas ele é `360/N`, e é ele que a luz
/// mostra. A barra são **2,5°** — entre o `2,14°` que shipa e o `3°` em que o nosso sombreamento
/// começa a mostrar o degrau (o limiar veio das fotos, e está declarado como oráculo de aparência no
/// doc do [`super::TOLERANCE_RATIO`], com a tabela inteira).
#[test]
fn the_automatic_tolerance_keeps_the_normal_smooth() {
    for r in [0.01_f64, 1.0, 100.0] {
        let n = cook_path_auto(&circle(r)).expect("círculo").segment_count();
        let jump = 360.0 / n as f64;
        assert!(
            jump <= 2.5,
            "um círculo de raio {r} saiu com {n} arestas, isto é {jump:.2}° entre facetas — acima \
             de ~3° o sombreamento mostra o degrau, e foi esse o report do smoke"
        );
        // ⚠️ **E o outro lado**: uma tolerância absurdamente fina não é «melhor», é um traçado que
        // ninguém espera — o custo é LINEAR nas arestas (~1 ms cada a 640×480, medido).
        assert!(
            n <= 400,
            "um círculo saiu com {n} arestas — a ~1 ms cada, é meio segundo de traçado assente"
        );
    }
}

/// A regra de preenchimento atravessa a costura — um compound path com buraco continua com buraco.
#[test]
fn the_fill_rule_crosses_the_seam() {
    let mut p = square(1.0);
    p.fill_rule = ph2d_vec_scene::FillRule::EvenOdd;
    p.subpaths.push(ph2d_vec_scene::Contour::new_closed(
        [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]]
            .into_iter()
            .map(VecVertex::corner)
            .collect(),
    ));
    let cooked = cook_path(&p, 1e-3).expect("compound");
    assert_eq!(cooked.fill(), ph2d_field::FillRule::EvenOdd);
    assert_eq!(
        cooked.contours().len(),
        2,
        "o buraco é um contorno a mais, e tem de chegar"
    );
}

/// A tolerância viaja **dentro** do perfil — é ela que responde "este perfil está bom?" sem
/// adivinhação, e é ela que um re-cozimento tem de igualar para não mudar a forma em silêncio.
#[test]
fn the_profile_remembers_the_tolerance_it_was_cooked_at() {
    let p = cook_path(&circle(1.0), 5e-4).expect("círculo");
    assert!((f64::from(p.tolerance()) - 5e-4).abs() < 1e-9);
}

/// ⭐⭐⭐ **UM DESENHO COM FURO JÁ VIRA UMA PEÇA COM FURO** (W57) — a composição que já existia.
///
/// ⚠️ **Este gate nasceu de uma pergunta que ia virar wave.** O `§54.7` deixou aberto *"um contorno
/// de cada vez: vários perfis numa peça só (ou **furos como contornos interiores**) pede uma decisão
/// de produto"*, e eu ia construir uma fonte de perfil com N desenhos. ⛔ **A `CLAUDE.md` §5.0 manda
/// medir antes:** *"antes de construir um item de lista aberta, MEÇA se a composição já o exprime"*.
///
/// Ela exprime. O [`ph2d_vec_scene::VecPath`] tem `subpaths` + `fill_rule` desde a v6 do formato
/// (compound paths), e a [`cook_path`] percorre `contour_count()` — logo **um contorno interior já
/// é um furo**, e a regra de preenchimento do desenho é que decide. *O que faltava não era código:
/// era um gate a dizer que isto funciona.*
///
/// ⚠️ **E a regra IMPORTA**: com `NonZero` e os dois contornos no **mesmo sentido**, o interior soma
/// e o furo **não aparece** — o que sai é um disco cheio. É a regra do desenho a mandar, e é o que
/// o artista já espera do editor vetorial.
#[test]
fn a_drawing_with_an_inner_contour_becomes_a_piece_with_a_hole() {
    use ph2d_vec_scene::{Contour, FillRule as VecFill};
    // Um anel: círculo de raio 1 por fora, círculo de raio 0,4 por dentro.
    let mut ring = circle(1.0);
    let inner = circle(0.4);
    ring.subpaths.push(Contour::new_closed(inner.verts.clone()));
    ring.fill_rule = VecFill::EvenOdd;
    let profile = cook_path_auto(&ring).expect("o anel");
    assert_eq!(
        profile.contours().len(),
        2,
        "o contorno interior não chegou ao perfil — a peça sairia sem furo"
    );
    let f = ph2d_field_eval::Field::from_tree(&ph2d_field_eval::profile::sd_profile(
        &profile,
        &fidget::context::Tree::x(),
        &fidget::context::Tree::y(),
    ));
    // ⚠️ Os três pontos que separam «anel» de «disco»: o centro é FORA (é o furo), a meia-parede é
    // DENTRO, e o exterior é fora. Um disco cheio falharia só no primeiro.
    assert!(
        f.at(0.0, 0.0, 0.0) > 0.0,
        "o centro do anel está DENTRO da matéria ({:.4}) — o furo não abriu",
        f.at(0.0, 0.0, 0.0)
    );
    assert!(
        f.at(0.7, 0.0, 0.0) < 0.0,
        "a parede do anel está FORA da matéria ({:.4})",
        f.at(0.7, 0.0, 0.0)
    );
    assert!(f.at(1.5, 0.0, 0.0) > 0.0, "o exterior está dentro");
    // …e a distância do centro à parede é o raio do furo: o furo tem o TAMANHO desenhado.
    let d = f.at(0.0, 0.0, 0.0);
    assert!(
        (d - 0.4).abs() < 0.02,
        "o furo mede {d:.4} em vez de 0,4 — ele abriu no sítio errado"
    );

    // ⚠️ **E o controle da REGRA**: com `NonZero` e os dois contornos no mesmo sentido, o furo
    // fecha. Sem isto, o gate acima passaria mesmo que a regra fosse ignorada.
    let mut solid = ring.clone();
    solid.fill_rule = VecFill::NonZero;
    let profile = cook_path_auto(&solid).expect("o disco");
    let g = ph2d_field_eval::Field::from_tree(&ph2d_field_eval::profile::sd_profile(
        &profile,
        &fidget::context::Tree::x(),
        &fidget::context::Tree::y(),
    ));
    assert!(
        g.at(0.0, 0.0, 0.0) < 0.0,
        "com `NonZero` e o mesmo sentido o centro devia ser MATÉRIA ({:.4}) — a regra de \
         preenchimento do desenho não está a chegar ao perfil",
        g.at(0.0, 0.0, 0.0)
    );
}

/// ⭐⭐⭐ **O VASO DO DONO ENCOLHE, E A FIGURA NÃO MUDA** (2026-09-16).
///
/// Os mesmos `12` pontos da cena `5` do smoke. Antes desta wave cada quina arredondada virava `~8`
/// segmentos rectos e o contorno saía com `94` arestas — `68` dos `76 ms` do quadro
/// (`docs/Render3d/06_auditoria_do_vaso.md`). Agora cada uma é **um arco**.
///
/// ⚠️ **A barra da FIGURA é medida contra a polilinha densa, que é a figura**, nos DOIS sentidos.
/// *Um gate que só contasse arcos aprovaria um arco no sítio errado.*
#[test]
fn o_vaso_do_dono_vira_arcos_e_a_figura_fica() {
    let path = vaso_do_dono();
    let p = crate::cook_path_auto(&path).expect("o vaso é um perfil válido");
    let arcos = p.arc_count();
    assert_eq!(
        arcos,
        10,
        "as dez quinas com raio têm de virar dez ARCOS (deu {arcos}); primitivas: {}",
        p.prim_count()
    );
    assert!(
        p.prim_count() <= 24,
        "o vaso tinha 94 arestas tesseladas; com arcos tem de caber em 24 (deu {})",
        p.prim_count()
    );
    // ⚠️ A polilinha densa NÃO encolheu — ela é a figura, e os seus leitores não mudam.
    assert!(
        p.segment_count() >= 80,
        "a polilinha densa é a figura e tem de ficar: deu {}",
        p.segment_count()
    );
    as_duas_vistas_concordam(&p);
}

/// ⭐⭐ **AS DUAS VISTAS DESCREVEM A MESMA CURVA** — o gate que impede a decomposição exacta de
/// derivar da polilinha. Sem ele, um arco no sítio errado passaria por ser barato.
fn as_duas_vistas_concordam(p: &ph2d_field::Profile) {
    let tol = f64::from(p.tolerance());
    for (c, (poli, arcs)) in p.contours().iter().zip(p.arcs()).enumerate() {
        if arcs.is_empty() {
            continue;
        }
        let denso = amostra_da_decomposicao(arcs);
        let ida = hausdorff(&denso, poli);
        let volta = hausdorff(poli, &denso);
        assert!(
            ida <= tol * 3.0 && volta <= tol * 3.0,
            "contorno {c}: as duas vistas discordam — ida {ida:.6}, volta {volta:.6}, tol {tol:.6}"
        );
    }
}

/// ⭐⭐⭐ **O FILETE VIVO É UM ARCO EM TODO ÂNGULO** — o gate que faltava (2026-09-16).
///
/// ⛔⛔ O gate que existia (`the_fillet_agrees_with_the_crates_canonical_corner_rounding`, na
/// `ph2d-vec-scene`) corre sobre um **QUADRADO**: quatro cantos a `90°`, que é exactamente o único
/// ângulo em que o defeito era invisível — o alçapão usava `(4/3)·tan(α/4)·s_in` onde a lei do arco
/// pede `·r`, e `s_in = r·tan(α/2)` vale `r` só a `90°`. Medido antes da cura, raio pedido `0,05`:
/// a `130°` saía **`0,083`** (`+66 %`, `110×` a tolerância).
///
/// *Uma fixtura de um ângulo só não mede uma lei que depende do ângulo.*
#[test]
fn o_filete_vivo_e_um_arco_em_todo_angulo() {
    for graus in [30.0_f64, 50.0, 70.0, 90.0, 110.0, 130.0, 150.0] {
        let (r_pedido, r_medido, desvio, tol) = filete_medido(graus, 0.05);
        // O RAIO é a lei, e ela vale em toda a faixa: era aqui que estava o defeito (a `130°` o
        // artista pedia `0,05` e recebia `0,083`).
        assert!(
            (r_medido - r_pedido).abs() <= r_pedido * 0.01,
            "a {graus}° o raio pedido é {r_pedido} e o medido {r_medido}"
        );
        // ⚠️ **A FIDELIDADE tem um tecto que não é meu: UMA cúbica não representa um arco grande.**
        // Até `130°` ela cabe na tolerância; a `150°` o erro intrínseco da aproximação cúbica passa
        // um pouco dela (`2,97e-4` contra `2,87e-4`, `1,04×`). Isso é uma propriedade da curva que a
        // `ph2d-vec-scene` emite, não desta wave — e a consequência prática é benigna: o
        // reconhecedor de arcos simplesmente **não** aceita uma quina dessas, e ela fica tesselada
        // como sempre esteve. *A barra diz o que é verdade, e nomeia o sítio onde deixa de ser.*
        let teto = if graus <= 130.0 { tol } else { tol * 1.5 };
        assert!(
            desvio <= teto,
            "a {graus}° a curva afasta-se {desvio:.3e} do círculo (barra {teto:.3e})"
        );
    }
}

/// Uma quina em V com o ângulo dado, cozida: devolve `(raio pedido, raio medido, desvio máximo do
/// círculo verdadeiro, tolerância de cozimento)`.
fn filete_medido(graus: f64, r: f64) -> (f64, f64, f64, f64) {
    let alpha = graus.to_radians();
    let theta = std::f64::consts::PI - alpha;
    let half = theta * 0.5;
    let path = ph2d_vec_scene::VecPath {
        verts: vec![
            ph2d_vec_scene::VecVertex::corner([-half.sin(), half.cos()]),
            ph2d_vec_scene::VecVertex {
                corner_radius: r,
                ..ph2d_vec_scene::VecVertex::corner([0.0, 0.0])
            },
            ph2d_vec_scene::VecVertex::corner([half.sin(), half.cos()]),
            ph2d_vec_scene::VecVertex::corner([0.0, 3.0]),
        ],
        closed: true,
        ..ph2d_vec_scene::VecPath::default()
    };
    let cooked = path.cooked();
    let tol = crate::span_of(&cooked) * crate::TOLERANCE_RATIO;
    let (verts, _) = cooked.contour(0).expect("há contorno");
    let n = verts.len();
    // O centro VERDADEIRO do filete: sobre a bissectriz, a `r/sin(θ/2)` do vértice.
    let ctr = [0.0, r / half.sin()];
    for i in 0..n {
        let a = &verts[i];
        let b = &verts[(i + 1) % n];
        if a.out_handle == a.anchor && b.in_handle == b.anchor {
            continue;
        }
        let bez = kurbo::CubicBez::new(
            kurbo::Point::new(a.anchor[0], a.anchor[1]),
            kurbo::Point::new(a.out_handle[0], a.out_handle[1]),
            kurbo::Point::new(b.in_handle[0], b.in_handle[1]),
            kurbo::Point::new(b.anchor[0], b.anchor[1]),
        );
        let mut pior = 0.0f64;
        let mut medio = 0.0f64;
        for k in 0..=20 {
            let t = f64::from(k) / 20.0;
            let z = kurbo::ParamCurve::eval(&bez, t);
            let rr = (z.x - ctr[0]).hypot(z.y - ctr[1]);
            pior = pior.max((rr - r).abs());
            if k == 10 {
                medio = rr;
            }
        }
        return (r, medio, pior, tol);
    }
    panic!("a {graus}° não saiu filete nenhum do cozimento");
}

/// Os `12` pontos da cena `5` do smoke — o vaso oco do dono.
fn vaso_do_dono() -> ph2d_vec_scene::VecPath {
    const VASO: [([f64; 2], f64); 12] = [
        ([0.00, -0.45], 0.0),
        ([0.26, -0.45], 0.05),
        ([0.30, -0.34], 0.05),
        ([0.15, -0.10], 0.06),
        ([0.33, 0.22], 0.06),
        ([0.27, 0.44], 0.04),
        ([0.33, 0.52], 0.02),
        ([0.27, 0.52], 0.02),
        ([0.21, 0.44], 0.04),
        ([0.09, -0.08], 0.05),
        ([0.19, -0.32], 0.04),
        ([0.00, -0.32], 0.0),
    ];
    ph2d_vec_scene::VecPath {
        verts: VASO
            .iter()
            .map(|&(p, r)| ph2d_vec_scene::VecVertex {
                corner_radius: r,
                ..ph2d_vec_scene::VecVertex::corner(p)
            })
            .collect(),
        closed: true,
        ..ph2d_vec_scene::VecPath::default()
    }
}

/// Amostra uma decomposição exacta numa polilinha densa — 32 pontos por arco, muito mais fino que a
/// tolerância. É régua, nunca produto.
fn amostra_da_decomposicao(arcs: &[([f32; 2], f32)]) -> Vec<[f32; 2]> {
    let n = arcs.len();
    let mut out = Vec::new();
    for i in 0..n {
        let (a, bulge) = arcs[i];
        let (b, _) = arcs[(i + 1) % n];
        out.push(a);
        let bulge = f64::from(bulge);
        if bulge == 0.0 {
            continue;
        }
        let (dx, dy) = (f64::from(b[0] - a[0]), f64::from(b[1] - a[1]));
        let l = dx.hypot(dy);
        let s = bulge * l * 0.5;
        let k = (s * s - (l * 0.5) * (l * 0.5)) / (2.0 * s);
        let (mx, my) = (f64::from(a[0] + b[0]) * 0.5, f64::from(a[1] + b[1]) * 0.5);
        let (nx, ny) = (-dy / l, dx / l);
        let (cx, cy) = (mx + nx * k, my + ny * k);
        let r = (s - k).abs();
        // ⚠️ O SENTIDO deriva-se dos ângulos, nunca de uma convenção decorada: com `|bulge| < 1` o
        // arco é o MENOR, logo a diferença dobrada para `(−π, π]` é exactamente ele. A 1.ª versão
        // assumiu `θ = 4·atan(bulge)` com sinal, e a régua leu `0,065` de discordância sobre uma
        // decomposição correcta.
        let a0 = (f64::from(a[1]) - cy).atan2(f64::from(a[0]) - cx);
        let a1 = (f64::from(b[1]) - cy).atan2(f64::from(b[0]) - cx);
        let mut theta = a1 - a0;
        while theta > std::f64::consts::PI {
            theta -= 2.0 * std::f64::consts::PI;
        }
        while theta <= -std::f64::consts::PI {
            theta += 2.0 * std::f64::consts::PI;
        }
        for t in 1..32 {
            let ang = a0 + theta * f64::from(t) / 32.0;
            #[allow(clippy::cast_possible_truncation)]
            out.push([(cx + r * ang.cos()) as f32, (cy + r * ang.sin()) as f32]);
        }
    }
    out
}

/// A maior distância de um ponto de `a` à **polilinha** `b` — ponto→SEGMENTO, nunca ponto→vértice.
///
/// ⛔⛔ **A 1.ª versão media ponto→vértice e acusou uma decomposição CORRECTA.** Numa polilinha
/// achatada a `9,7e-5` sobre um arco de raio `0,05` os vértices ficam a `~0,0062` um do outro, logo
/// um ponto no meio de dois lê `~0,0031` de «erro» — e foi exactamente `0,0028`–`0,0032` que ela
/// leu, uniformemente, em TODOS os dez arcos. *Um desvio igual em todas as amostras é assinatura da
/// régua, não do objecto* — e a lição já estava paga neste repo (a régua da ponta, ponto→FACE).
fn hausdorff(a: &[[f32; 2]], b: &[[f32; 2]]) -> f64 {
    let n = b.len();
    a.iter()
        .map(|p| {
            (0..n)
                .map(|i| dist_ponto_segmento(*p, b[i], b[(i + 1) % n]))
                .fold(f64::INFINITY, f64::min)
        })
        .fold(0.0, f64::max)
}

fn dist_ponto_segmento(p: [f32; 2], a: [f32; 2], b: [f32; 2]) -> f64 {
    let (px, py) = (f64::from(p[0]), f64::from(p[1]));
    let (ax, ay) = (f64::from(a[0]), f64::from(a[1]));
    let (bx, by) = (f64::from(b[0]), f64::from(b[1]));
    let (ex, ey) = (bx - ax, by - ay);
    let ee = ex * ex + ey * ey;
    let t = if ee > 0.0 {
        (((px - ax) * ex + (py - ay) * ey) / ee).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (px - (ax + ex * t)).hypot(py - (ay + ey * t))
}

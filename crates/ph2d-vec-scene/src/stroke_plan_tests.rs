//! Testes de [`crate::stroke_plan`] — arquivo irmão.
//!
//! O que se prova aqui é a RECEITA. Ela morava dentro do renderer (`ph2d-vec-render`, módulo
//! `markers`) enquanto desenhar era a única coisa que se fazia com um traço; mudou-se para cá
//! quando o **Outline Stroke** passou a ser o segundo consumidor, e estes gates vieram junto —
//! um gate que fica onde a lei NÃO está mais é um gate sobre nada.
//!
//! O de baixo que mais morde continua sendo o mesmo: **a linha para exatamente nas costas da
//! cabeça**. Recuo e cabeça são a mesma medida vista dos dois lados, e errar não quebra a
//! compilação — quebra o desenho.

use super::*;
use crate::{Marker, Rgba8, VecVertex};

/// Uma linha horizontal, da esquerda para a direita: a ponta do FIM olha para `+x`.
fn line(len: f64) -> VecPath {
    VecPath {
        verts: vec![VecVertex::corner([0.0, 0.0]), VecVertex::corner([len, 0.0])],
        closed: false,
        ..VecPath::default()
    }
}

fn spec(width: f64) -> StrokeSpec {
    StrokeSpec::new(Rgba8::new(0, 0, 0, 255), width)
}

fn head_spec(end: Marker, scale: f64, round: f64) -> StrokeSpec {
    let mut s = spec(2.0);
    s.marker_end = end;
    s.marker_scale = scale;
    s.marker_round = round;
    s
}

/// A peça da LINHA do plano, se houver.
fn line_piece<'a>(plan: &'a [StrokePiece<'a>]) -> Option<&'a VecPath> {
    plan.iter().find_map(|p| match p {
        StrokePiece::Line { path } => Some(&**path),
        _ => None,
    })
}

/// A geometria da PONTA — cheia ou vazada, o que importa aqui é a forma.
fn head_piece<'a>(plan: &'a [StrokePiece<'a>]) -> Option<&'a VecPath> {
    plan.iter().find_map(|p| match p {
        StrokePiece::Fill { path } | StrokePiece::Symbol { path } => Some(path),
        StrokePiece::Line { .. } => None,
    })
}

/// O quanto a cabeça avança para TRÁS do bico, no contorno real (as cúbicas, não as âncoras).
fn head_depth(geo: &VecPath, tip_x: f64) -> f64 {
    const STEPS: usize = 32;
    let n = geo.verts.len();
    let segs = if geo.closed { n } else { n - 1 };
    let mut deepest = f64::MIN;
    for i in 0..segs {
        let (a, b) = (&geo.verts[i], &geo.verts[(i + 1) % n]);
        let (p0, p1, p2, p3) = (a.anchor, a.out_handle, b.in_handle, b.anchor);
        for k in 0..=STEPS {
            let u = k as f64 / STEPS as f64;
            let v = 1.0 - u;
            let x = v * v * v * p0[0]
                + 3.0 * v * v * u * p1[0]
                + 3.0 * v * u * u * p2[0]
                + u * u * u * p3[0];
            deepest = deepest.max(tip_x - x);
        }
    }
    deepest
}

/// **O caso de 99% dos paths não paga uma cópia.** Sem ponta há UMA peça, e o caminho dela é
/// o próprio path emprestado — não um clone. Isto não é micro-otimização: o renderer chama
/// isto para cada traço de cada frame, e a promessa de custo zero é o que permitiu que a
/// receita saísse de dentro dele. (E a linha chega inteira na extremidade.)
#[test]
fn a_plain_stroke_is_one_borrowed_piece() {
    let p = line(60.0);
    let s = spec(2.0);
    assert!(!s.has_markers());
    let plan = stroke_plan(&p, &s);
    assert_eq!(plan.len(), 1, "sem ponta, só a linha");
    let StrokePiece::Line { path } = &plan[0] else {
        panic!("a linha é traçada, não preenchida");
    };
    assert!(
        matches!(path, Cow::Borrowed(_)),
        "sem ponta nada é reconstruído — o plano empresta o path"
    );
    assert_eq!(
        path.verts.last().expect("tem vertices").anchor,
        [60.0, 0.0],
        "e não foi encurtada"
    );
}

/// **O gate que morde: a linha para EXATAMENTE nas costas da cabeça, em qualquer `scale`.**
///
/// O recuo e a cabeça têm de ler o MESMO `marker_scale`. Se o recuo ignorar o `scale`, uma
/// cabeça 2.5× maior engole o fim do traço e a linha reaparece ATRAVESSANDO a seta; se a
/// cabeça ignorar, sobra um vão entre o fim da linha e a ponta.
#[test]
fn the_line_meets_the_head_at_every_scale() {
    let path = line(60.0);
    for m in [Marker::Triangle, Marker::Diamond, Marker::CircleOpen] {
        for scale in [0.5, 1.0, 2.5] {
            for round in [0.0, 0.5, 1.0] {
                let s = head_spec(m, scale, round);
                let plan = stroke_plan(&path, &s);
                let geo = head_piece(&plan).expect("a ponta existe");
                let trimmed = line_piece(&plan).expect("sobra linha");
                let end_x = trimmed.verts.last().expect("tem vertices").anchor[0];

                let gap = head_depth(geo, 60.0) - (60.0 - end_x);
                assert!(
                    gap.abs() < 1e-3 * s.width * s.marker_scale,
                    "{m:?} (scale {scale}, round {round}): a linha termina em x={end_x} e a \
                     cabeça vai ate x={} — {} de {}",
                    60.0 - head_depth(geo, 60.0),
                    if gap > 0.0 { "VAO" } else { "sobreposicao" },
                    gap.abs()
                );
            }
        }
    }
}

/// O `marker_round` do usuário chega na cabeça: com ele a ponta perde as quinas vivas (mais
/// vértices, handles não-degenerados). Um `0.0` cravado passaria despercebido — a cena
/// renderiza, só que afiada para sempre.
#[test]
fn the_users_round_reaches_the_head() {
    let path = line(60.0);
    let sharp = stroke_plan(&path, &head_spec(Marker::Triangle, 1.0, 0.0));
    let round = stroke_plan(&path, &head_spec(Marker::Triangle, 1.0, 0.6));
    let sharp = head_piece(&sharp).expect("existe");
    let round = head_piece(&round).expect("existe");
    assert_eq!(sharp.verts.len(), 3, "afiada: um vertice por quina");
    assert_eq!(round.verts.len(), 6, "arredondada: dois por quina");
    assert!(
        round
            .verts
            .iter()
            .all(|v| v.in_handle != v.anchor || v.out_handle != v.anchor),
        "sobrou quina viva: o marker_round nao chegou no build"
    );
}

/// Uma ponta CHEIA vira peça **preenchida** e a linha ENCURTA para caber nela; uma ponta
/// VAZADA vira **símbolo** — traçado, mas com caneta própria, porque o tracejado é da LINHA
/// (um losango pontilhado é ruído, não desenho).
#[test]
fn a_filled_head_fills_and_an_open_head_is_a_symbol() {
    let p = line(60.0);
    let mut s = head_spec(Marker::Triangle, 1.0, 0.0);
    s.dash = Some((2.0, 2.0));
    let plan = stroke_plan(&p, &s);
    assert!(
        matches!(plan[1], StrokePiece::Fill { .. }),
        "o triângulo é cheio"
    );

    s.marker_end = Marker::Open;
    let plan = stroke_plan(&p, &s);
    assert!(
        matches!(plan[1], StrokePiece::Symbol { .. }),
        "a ponta aberta é um símbolo, nunca a caneta tracejada da linha"
    );
    assert!(
        matches!(plan[0], StrokePiece::Line { .. }),
        "…e a LINHA continua sendo a linha (é ela que carrega o tracejado)"
    );
}

/// **Uma linha mais curta que os recuos somados não tem linha** — só as pontas. Cair de
/// volta na linha inteira desenharia exatamente o traço que o recuo existe para esconder.
#[test]
fn a_line_shorter_than_its_heads_has_no_line_piece() {
    let p = line(0.5);
    let mut s = head_spec(Marker::Triangle, 1.0, 0.0);
    s.width = 4.0; // caneta gorda: os recuos somam mais que o comprimento
    s.marker_start = Marker::Triangle;
    let plan = stroke_plan(&p, &s);
    assert!(
        !plan.is_empty(),
        "as pontas continuam existindo — some a LINHA, não o desenho"
    );
    assert!(line_piece(&plan).is_none(), "não sobra linha para desenhar");
}

/// Um contorno FECHADO não tem extremo onde pôr uma ponta — nem com o seletor marcado. E
/// por não ter ponta, também não encurta.
#[test]
fn a_closed_contour_has_no_heads() {
    let p = VecPath {
        verts: vec![
            VecVertex::corner([0.0, 0.0]),
            VecVertex::corner([40.0, 0.0]),
            VecVertex::corner([40.0, 40.0]),
        ],
        closed: true,
        ..VecPath::default()
    };
    let mut s = head_spec(Marker::Triangle, 2.0, 0.0);
    s.marker_start = Marker::Triangle;
    let plan = stroke_plan(&p, &s);
    assert_eq!(plan.len(), 1, "só a linha — um anel não tem extremo");
    assert_eq!(
        line_piece(&plan).expect("a linha").verts.len(),
        3,
        "e ela não foi encurtada"
    );
}

// ─── O TRACEJADO QUE FECHA A VOLTA (Enio, 2026-08-22) ────────────────────────────────────

/// Um traço tracejado de largura 1 — assim os MÚLTIPLOS que o `dash` guarda são, em número, os
/// próprios comprimentos, e a aritmética do gate fica legível.
fn dashed(d: f64, g: f64) -> StrokeSpec {
    let mut s = StrokeSpec::new(crate::Rgba8::new(0, 0, 0, 255), 1.0);
    s.dash = Some((d, g));
    s
}

/// **A LEI: um número INTEIRO de períodos cabe na volta.**
///
/// É a única coisa que faz a emenda desaparecer — o padrão acaba exatamente onde começou, e a
/// junta cai numa transição vão→traço indistinguível das outras.
///
/// ⚠️ O controlo positivo é a metade que importa: sem ele o gate passaria com o ajuste desligado,
/// porque `12 / 3,5` sobrar `1,5` só é um defeito se alguém afirmar que sobrava.
#[test]
fn the_dash_closes_the_loop_on_a_shape_path() {
    let rect = crate::rectangle([0.0, 0.0], [4.0, 2.0]); // perímetro 12
    let s = dashed(2.5, 1.0); // período 3,5 — NÃO divide 12
    let raw = s.dash_lengths().expect("ha' tracejado");
    assert!(
        (12.0 % (raw[0] + raw[1])).abs() > 1e-9,
        "controlo positivo: o padrao cru TEM de sobrar, senao o gate nao prova nada"
    );

    let [d, g] = crate::dash_for(&rect, &s).expect("ha' tracejado");
    let period = d + g;
    let n = 12.0 / period;
    assert!(
        (n - n.round()).abs() < 1e-9,
        "o padrao nao fecha a volta: cabem {n} periodos, e a emenda aparece no ponto inicial"
    );
    assert!(n.round() >= 1.0);
}

/// **A RAZÃO traço/vão é preservada** — é ela a assinatura visual do tracejado.
///
/// ⚠️ Esticar só o vão fecharia a volta na mesma e seria a cura errada: o pontilhado mudaria de
/// caráter ao redimensionar a forma, e o artista veria o desenho dele derivar sozinho.
#[test]
fn fitting_the_loop_keeps_the_dash_to_gap_ratio() {
    let rect = crate::rectangle([0.0, 0.0], [4.0, 2.0]);
    let s = dashed(3.0, 1.0);
    let [d, g] = crate::dash_for(&rect, &s).expect("ha' tracejado");
    assert!(
        (d / g - 3.0).abs() < 1e-9,
        "a razao 3:1 virou {:.4}:1 -- o tracejado mudou de carater",
        d / g
    );
}

/// **UM CAMINHO ABERTO COMEÇA E ACABA EM TRAÇO** — as duas juntas dele são as PONTAS.
///
/// O artefato reportado (Enio, 2026-08-22): com marcadores, a seta e o círculo apareciam
/// **descolados** do traço. A causa não era a posição do marcador — era o padrão a acabar no meio
/// de um vão, deixando um espaço morto entre o último traço e a ponta.
///
/// A lei: `n` traços e `n − 1` vãos cobrem o comprimento exato, então há traço nas duas
/// extremidades e o marcador encosta.
///
/// ⚠️ **Este gate substitui um que afirmava o CONTRÁRIO** (*"um caminho aberto mantém o dash
/// autorado"*), escrito quando eu tinha decidido que só um contorno fechado tinha junta. A imagem
/// do Enio mostrou a omissão: um caminho aberto tem DUAS.
#[test]
fn an_open_path_starts_and_ends_on_a_dash() {
    let line = crate::line([0.0, 0.0], [10.0, 0.0]);
    let s = dashed(2.5, 1.0);
    let raw = s.dash_lengths().expect("ha' tracejado");
    let n_raw = (10.0 + raw[1]) / (raw[0] + raw[1]);
    assert!(
        (n_raw - n_raw.round()).abs() > 1e-9,
        "controlo positivo: o padrao cru TEM de nao encaixar, senao o gate passa vazio"
    );

    let [d, g] = crate::dash_for(&line, &s).expect("ha' tracejado");
    let n = ((10.0 + g) / (d + g)).round();
    let span = n * d + (n - 1.0) * g;
    assert!(
        (span - 10.0).abs() < 1e-9,
        "os {n} tracos e {} vaos cobrem {span}, e a linha mede 10 -- sobra espaco morto na ponta",
        n - 1.0
    );
    assert!(
        (d / g - 2.5).abs() < 1e-9,
        "a razao traco/vao mudou: o tracejado trocou de carater"
    );
}

/// **Sem tracejado, a pergunta não chega à geometria.** É o gate do CUSTO: medir o perímetro é um
/// `arclen` por segmento, e 99% dos traços são sólidos — pagá-lo neles seria uma regressão
/// silenciosa em toda cena.
///
/// ⚠️ Ele prova a saída, não o caminho. A garantia real é a ORDEM dentro do `dash_for` (a guarda
/// antes da medição), e é por isso que o doc dela nomeia a ordem explicitamente.
#[test]
fn a_solid_stroke_asks_nothing_of_the_geometry() {
    let rect = crate::rectangle([0.0, 0.0], [4.0, 2.0]);
    let solid = StrokeSpec::new(crate::Rgba8::new(0, 0, 0, 255), 1.0);
    assert_eq!(crate::dash_for(&rect, &solid), None);
}

/// **Um período MAIOR que a volta não é encolhido até caber.** Ele satura em uma repetição: um
/// tracejado grosso autorado não pode virar um pontilhado fino que ninguém escolheu.
#[test]
fn a_period_longer_than_the_loop_saturates_at_one() {
    let small = crate::rectangle([0.0, 0.0], [1.0, 1.0]); // perímetro 4
    let s = dashed(20.0, 5.0); // período 25 > 4
    let [d, g] = crate::dash_for(&small, &s).expect("ha' tracejado");
    assert!(
        ((d + g) - 4.0).abs() < 1e-9,
        "com n=1 o periodo tem de ser a volta inteira, e deu {}",
        d + g
    );
}

/// **FECHA A VOLTA COZIDA, NÃO A AUTORADA** — o artefato reportado (Enio, 2026-08-22: um resíduo
/// de traço na junta de um retângulo ARREDONDADO).
///
/// Quem desenha traça a geometria derivada (`build_contours(&cooked, …)`); a fonte angulosa tem
/// outro perímetro, porque o raio corta os cantos. Ajustar pela fonte encaixa o padrão numa volta
/// que ninguém desenha, e a junta volta a aparecer — com o agravante de parecer corrigida.
///
/// ⚠️ As duas asserções finais são o par: a primeira diz que fecha no cozido, a segunda que NÃO
/// fecha no autorado. Sem a segunda, um dia em que os dois perímetros coincidissem faria o gate
/// passar sobre a versão errada.
#[test]
fn the_dash_closes_the_cooked_loop_not_the_authored_one() {
    let mut r = crate::rectangle([0.0, 0.0], [4.0, 2.0]);
    for v in &mut r.verts {
        v.corner_radius = 0.5;
    }
    let len_of = |verts: &[crate::VecVertex]| {
        crate::arc_path::ArcPath::from_contour(verts, true)
            .expect("contorno")
            .total()
    };
    let src = len_of(&r.verts);
    let cooked = len_of(&r.cooked().verts);
    assert!(
        (src - cooked).abs() > 1e-6,
        "controlo positivo: o raio TEM de encurtar a volta ({src} vs {cooked}), senao o gate \
         nao distingue as duas geometrias"
    );

    let s = dashed(2.5, 1.0);
    let [d, g] = crate::dash_for(&r, &s).expect("ha' tracejado");
    let period = d + g;

    let n_cooked = cooked / period;
    assert!(
        (n_cooked - n_cooked.round()).abs() < 1e-9,
        "o padrao nao fecha a volta DESENHADA: cabem {n_cooked} periodos"
    );
    let n_src = src / period;
    assert!(
        (n_src - n_src.round()).abs() > 1e-9,
        "o padrao fechou a volta AUTORADA -- e' a geometria que ninguem desenha"
    );
}

/// **A LINHA PARA NAS COSTAS DO MARCADOR, seja qual for o ângulo da alça.**
///
/// O defeito (Enio, 2026-08-22): *"a depender do ângulo do handle/alça o stroke não se encaixa no
/// objeto do start ou do end"*. A ponta ocupa uma extensão RETA; encurtar a linha anda em ARCO. Na
/// extremidade encurvada as duas divergem — medido, com a ponta a pedir 4,0 de espaço, uma alça a
/// 80° recuava **0,66** no eixo dela.
///
/// ⚠️ **O laço varre ÂNGULOS, e é isso que o torna um gate e não um exemplo.** Um caso só — a
/// extremidade reta — passa mesmo com o defeito inteiro presente, porque ali arco e reta
/// coincidem. É exatamente o caso em que o bug era invisível.
#[test]
fn the_line_stops_on_the_markers_back_at_any_handle_angle() {
    use crate::{Marker, VertexKind, end_tangent};

    let curve = |hx: f64, hy: f64| crate::VecPath {
        verts: vec![
            crate::VecVertex {
                anchor: [0.0, 0.0],
                in_handle: [0.0, 0.0],
                out_handle: [4.0, 0.0],
                kind: VertexKind::Smooth,
                corner_radius: 0.0,
            },
            crate::VecVertex {
                anchor: [10.0, 0.0],
                in_handle: [10.0 + hx, hy],
                out_handle: [10.0, 0.0],
                kind: VertexKind::Smooth,
                corner_radius: 0.0,
            },
        ],
        closed: false,
        ..crate::VecPath::default()
    };

    let mut s = StrokeSpec::new(crate::Rgba8::new(255, 255, 255, 255), 1.0);
    s.marker_end = Marker::Triangle;
    let want = Marker::Triangle.inset(s.marker_scale) * s.width;

    // ⚠️ Ângulos em que a curva AINDA oferece o espaço que a ponta pede. O caso em que ela não
    // oferece é degenerado e tem gate próprio (abaixo) — misturá-los aqui obrigaria este a
    // aceitar duas respostas, e um gate que aceita duas respostas não separa nada.
    for (hx, hy) in [(-4.0, 0.0), (-4.0, 1.0), (-4.0, 3.0), (-4.0, 6.0)] {
        let p = curve(hx, hy);
        let (tip, dir) = end_tangent(&p, false).expect("tangente");
        let (a, b) = crate::marker_arc_insets(&p, &s);
        let t = crate::trim_path(&p, a, b).expect("trim");
        let last = t.verts.last().expect("fim").anchor;
        let proj = (tip[0] - last[0]) * dir[0] + (tip[1] - last[1]) * dir[1];
        assert!(
            (proj - want).abs() < 1e-6,
            "alca ({hx}, {hy}): a linha parou a {proj:.4} do bico, e a ponta ocupa {want:.4} -- \
             ela nao encaixa nas costas do marcador"
        );
    }
}

/// **Uma extremidade RETA continua a recuar exatamente o `inset`** — o controlo que impede a cura
/// de virar uma segunda deformação.
///
/// ⚠️ Sem ele, uma conversão arco→reta com erro de escala passaria no gate acima (que compara
/// contra a mesma conta) e mudaria em silêncio TODA linha reta com ponta — o caso comum.
#[test]
fn a_straight_end_still_recedes_exactly_the_inset() {
    use crate::Marker;
    let line = crate::line([0.0, 0.0], [10.0, 0.0]);
    let mut s = StrokeSpec::new(crate::Rgba8::new(255, 255, 255, 255), 1.0);
    s.marker_end = Marker::Triangle;
    let want = Marker::Triangle.inset(s.marker_scale) * s.width;
    let (_, b) = crate::marker_arc_insets(&line, &s);
    assert!(
        (b - want).abs() < 1e-9,
        "numa reta o arco E' a reta: esperado {want}, veio {b}"
    );
}

/// **QUANDO A CURVA NÃO TEM O ESPAÇO QUE A PONTA PEDE, a linha recua o máximo e não sobra lixo.**
///
/// Uma alça muito angulada faz a curva dobrar sobre si: a extensão dela NA DIREÇÃO da ponta fica
/// menor que a própria ponta (medido: 1,64 disponíveis para uma ponta de 4,0). Não há recuo que
/// ponha a linha nas costas do marcador, porque não há costas onde pô-la.
///
/// A resposta honesta é saturar — a linha some e fica a ponta, que é o que o `stroke_plan` já
/// documenta para *"uma linha mais curta que os recuos somados"*. O que este gate proíbe é o
/// contrário: um recuo que PARE no meio e deixe um coto de linha atravessado na ponta.
#[test]
fn a_marker_bigger_than_the_curve_can_offer_consumes_the_line() {
    use crate::{Marker, VertexKind};
    let p = crate::VecPath {
        verts: vec![
            crate::VecVertex {
                anchor: [0.0, 0.0],
                in_handle: [0.0, 0.0],
                out_handle: [4.0, 0.0],
                kind: VertexKind::Smooth,
                corner_radius: 0.0,
            },
            crate::VecVertex {
                anchor: [10.0, 0.0],
                in_handle: [9.0, 6.0],
                out_handle: [10.0, 0.0],
                kind: VertexKind::Smooth,
                corner_radius: 0.0,
            },
        ],
        closed: false,
        ..crate::VecPath::default()
    };
    let mut s = StrokeSpec::new(crate::Rgba8::new(255, 255, 255, 255), 1.0);
    s.marker_end = Marker::Triangle;

    let (_, b) = crate::marker_arc_insets(&p, &s);
    let span: f64 = p
        .verts
        .windows(2)
        .map(|w| (w[1].anchor[0] - w[0].anchor[0]).hypot(w[1].anchor[1] - w[0].anchor[1]))
        .sum();
    assert!(
        b >= span - 1e-9,
        "sem espaco para a ponta, o recuo tem de consumir a linha INTEIRA ({b} de {span}) -- \
         parar no meio deixaria um coto atravessado no marcador"
    );
}

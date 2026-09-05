//! Gates da tradução SVG ⇄ documento. **A lei dos eixos é a primeira, e tem duas metades**: o
//! sentido (uma ponta para cima continua para cima) e a ida-e-volta (as duas funções são inversas).

use crate::{Drawing, Options, import, svg_to_world, world_to_svg};
use ph2d_vec_scene::{FillRule, Paint, Xform};

fn ler(svg: &str) -> Drawing {
    import(svg.as_bytes(), &Options::default()).expect("o SVG da fixtura tem de entrar")
}

fn ler_ppm(svg: &str, ppm: f64) -> Drawing {
    import(
        svg.as_bytes(),
        &Options {
            pixels_per_meter: ppm,
        },
    )
    .expect("o SVG da fixtura tem de entrar")
}

/// Um triângulo com a **ponta para cima** no ficheiro (no SVG, para cima é `y` MENOR).
const PONTA_PARA_CIMA: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
  <path id="seta" d="M 50 10 L 90 90 L 10 90 Z" fill="#3366cc"/>
</svg>"##;

/// ⭐⭐⭐ **UMA PONTA PARA CIMA CONTINUA PARA CIMA.**
///
/// É a lei que o exportador de 02/09 não tinha: ele escrevia coordenadas de mundo cruas num
/// ficheiro cujo Y desce, e o resultado saía espelhado — com o comentário a AFIRMAR que os dois
/// eixos concordavam.
///
/// ⚠️ Mutação que tem de sangrar: tirar o sinal do `-k` na [`svg_to_world`]. O desenho continua a
/// entrar, com o tamanho certo e no sítio certo — e de cabeça para baixo.
#[test]
fn a_tip_that_points_up_in_the_file_points_up_in_the_world() {
    let d = ler(PONTA_PARA_CIMA);
    assert_eq!(d.shapes.len(), 1);
    let ys: Vec<f64> = d.shapes[0].path.verts.iter().map(|v| v.anchor[1]).collect();
    let ponta = ys[0];
    assert!(
        ys[1..].iter().all(|y| *y < ponta),
        "a ponta (o 1.o vertice) tem de ficar ACIMA das outras duas no mundo: {ys:?}"
    );
}

/// **As duas direcções são inversas uma da outra** — e é isto que impede o exportador e o
/// importador de divergirem no dia em que alguém mexer numa delas.
#[test]
fn the_two_directions_of_the_axis_law_are_inverses() {
    for ppm in [1.0, 0.5, 100.0] {
        let ida = svg_to_world(ppm);
        let volta = world_to_svg(ppm);
        for p in [[0.0, 0.0], [3.0, -7.5], [-11.25, 2.0]] {
            let r = volta.apply(ida.apply(p));
            assert!(
                (r[0] - p[0]).abs() < 1e-12 && (r[1] - p[1]).abs() < 1e-12,
                "ppm={ppm}: {p:?} -> {r:?}"
            );
        }
    }
}

/// **Um px é um px** — a mesma lei que dimensiona uma sprite importada.
#[test]
fn one_pixel_is_one_pixel() {
    let d = ler_ppm(PONTA_PARA_CIMA, 50.0);
    assert!(
        (d.size[0] - 2.0).abs() < 1e-9 && (d.size[1] - 2.0).abs() < 1e-9,
        "100 px a 50 px/m tem de dar 2 unidades: {:?}",
        d.size
    );
}

/// ⭐⭐⭐ **O `<g transform>` ANINHADO coloca a forma onde o ficheiro diz.**
///
/// ⚠️ Esta é a fixtura da armadilha do usvg: o doc do `Path::data()` diz *"All segments are in
/// absolute coordinates"* e ali *absolute* quer dizer **comandos** absolutos, não **espaço**
/// absoluto. Um ficheiro SEM transform lê igual das duas maneiras — só o aninhamento separa as
/// hipóteses.
#[test]
fn a_nested_transform_places_the_shape_where_the_file_says() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
      <g transform="translate(30 10)"><g transform="scale(2)">
        <rect x="0" y="0" width="5" height="5" fill="#000"/>
      </g></g>
    </svg>"##;
    let d = ler(svg);
    let xs: Vec<f64> = d.shapes[0].path.verts.iter().map(|v| v.anchor[0]).collect();
    let min = xs.iter().copied().fold(f64::INFINITY, f64::min);
    let max = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    // O rect vai de x=30 a x=40 no ficheiro; o viewBox de 100 centra-se em -50.
    assert!(
        (min - (-20.0)).abs() < 1e-6 && (max - (-10.0)).abs() < 1e-6,
        "o rect tem de ocupar x in [-20, -10]: [{min}, {max}]"
    );
}

/// **Uma quadrática vira uma cúbica EXACTA** — a elevação de grau é identidade algébrica, então o
/// ponto do meio das duas curvas coincide ao bit da aritmética.
#[test]
fn a_quadratic_becomes_an_exact_cubic() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 10 10">
      <path d="M 0 0 Q 5 10 10 0" fill="none" stroke="#000" stroke-width="1"/>
    </svg>"##;
    let d = ler(svg);
    let v = &d.shapes[0].path.verts;
    assert_eq!(v.len(), 2);
    // A quadratica em t=0.5 esta' em (5, 5) no ficheiro. Em mundo (centrado, Y virado) e' (0, 0).
    let m = meio(v[0].anchor, v[0].out_handle, v[1].in_handle, v[1].anchor);
    assert!(
        m[0].abs() < 1e-9 && m[1].abs() < 1e-9,
        "o meio da curva tem de bater com o da quadratica: {m:?}"
    );
}

fn meio(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2]) -> [f64; 2] {
    // de Casteljau em t = 0.5
    let mid = |p: [f64; 2], q: [f64; 2]| [(p[0] + q[0]) * 0.5, (p[1] + q[1]) * 0.5];
    let (ab, bc, cd) = (mid(a, b), mid(b, c), mid(c, d));
    mid(mid(ab, bc), mid(bc, cd))
}

/// **O que o ficheiro carrega e o documento não exprime SAI NOMEADO** — a lei do importador
/// `.ase`: *um importador que ignora em silêncio é pior do que um que recusa*.
#[test]
fn what_the_file_carries_and_we_do_not_is_named() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
      <defs><clipPath id="c"><rect x="0" y="0" width="10" height="10"/></clipPath></defs>
      <text x="10" y="20" font-size="10">ola</text>
      <g clip-path="url(#c)"><rect x="0" y="0" width="50" height="50" fill="#111"/></g>
    </svg>"##;
    let d = ler(svg);
    let juntas = d.notes.join(" | ");
    assert!(
        juntas.contains("<text>"),
        "o texto tem de ser nomeado: {juntas}"
    );
    assert!(
        juntas.contains("clip-path"),
        "o clip tem de ser nomeado: {juntas}"
    );
}

/// **Um `<g id>` vira um GRUPO da Hierarquia; os grupos que o usvg FABRICA não.**
///
/// ⚠️⚠️ **A 1.ª redacção deste gate NÃO continha o fenómeno, e a mutação SOBREVIVEU.** A fixtura
/// era um `<g id opacity>` com um filho: ali a opacidade vive no grupo que **já tem id**, então o
/// usvg não fabrica ninguém — aceitar todo grupo dava exactamente o mesmo resultado, e o gate ficava
/// verde sobre a doença que o nome dele promete apanhar.
///
/// ⇒ a fixtura tem agora as **duas** espécies: um `<g id>` que o artista desenhou, e um `<rect
/// opacity>` **solto**, que o usvg embrulha num grupo próprio (sem `id`) só para poder compor a
/// opacidade. Sem a regra, a Hierarquia enche-se de pastas anónimas que ninguém desenhou.
///
/// Mutação que tem de sangrar: tirar a guarda do `id` vazio no [`crate::import`].
#[test]
fn a_named_group_becomes_a_group_and_an_invented_one_does_not() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
      <g id="cabeca">
        <rect x="0" y="0" width="10" height="10" fill="#111"/>
      </g>
      <rect x="20" y="20" width="10" height="10" fill="#222" opacity="0.5"/>
    </svg>"##;
    let d = ler(svg);
    assert_eq!(d.shapes.len(), 2);
    assert_eq!(
        d.groups.len(),
        1,
        "so' o grupo NOMEADO conta — o embrulho que o usvg fabrica para a opacidade nao e' uma \
         pasta que o artista desenhou: {:?}",
        d.groups
    );
    assert_eq!(
        d.shapes[1].group, None,
        "a forma solta fica na raiz, e nao dentro do embrulho"
    );
    assert!(
        (d.shapes[1].path.opacity.get() - 0.5).abs() < 1e-3,
        "e a opacidade do embrulho CHEGA a ela na mesma: {}",
        d.shapes[1].path.opacity.get()
    );
    assert_eq!(d.groups[0].name, "cabeca");
    assert_eq!(d.shapes[0].group, Some(0));
}

/// ⭐ **A opacidade e a mistura do ficheiro chegam à FORMA** — as duas propriedades que a v19 do
/// schema pôs no documento (estudo 42, item 2) são exactamente o que o `opacity` e o
/// `mix-blend-mode` do SVG dizem.
#[test]
fn the_files_opacity_and_blend_reach_the_shape() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
      <rect x="0" y="0" width="10" height="10" fill="#111" opacity="0.25"
            style="mix-blend-mode:multiply"/>
    </svg>"##;
    let d = ler(svg);
    let p = &d.shapes[0].path;
    assert!(
        (p.opacity.get() - 0.25).abs() < 1e-3,
        "opacidade: {}",
        p.opacity.get()
    );
    assert_eq!(p.blend, ph2d_blend_mode::BlendMode::Multiply);
}

/// **A opacidade de um grupo sobre VÁRIAS formas é aproximada, e a nota di-lo.** Sobre UMA forma
/// as duas contas são a mesma — e é esse o caso comum, porque o usvg embrulha em grupo toda forma
/// que traz `opacity` própria.
#[test]
fn a_group_opacity_over_many_shapes_is_named_and_over_one_is_not() {
    let uma = ler(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
      <g opacity="0.5"><rect x="0" y="0" width="9" height="9" fill="#111"/></g></svg>"##,
    );
    assert!(
        !uma.notes.iter().any(|n| n.contains("opacidade de GRUPO")),
        "uma forma so' nao e' aproximacao nenhuma: {:?}",
        uma.notes
    );
    let duas = ler(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
      <g opacity="0.5"><rect x="0" y="0" width="9" height="9" fill="#111"/>
      <rect x="4" y="4" width="9" height="9" fill="#333"/></g></svg>"##,
    );
    assert!(
        duas.notes.iter().any(|n| n.contains("opacidade de GRUPO")),
        "duas formas sobrepostas: a conta muda, e tem de ser dita: {:?}",
        duas.notes
    );
}

/// **O tracejado entra em MÚLTIPLOS da largura** — é assim que ele continua certo quando o traço
/// engrossa, e é a unidade que o documento guarda.
#[test]
fn the_dash_arrives_in_multiples_of_the_width() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
      <path d="M 0 0 L 50 0" stroke="#000" stroke-width="4" stroke-dasharray="8 12"/>
    </svg>"##;
    let d = ler(svg);
    let s = d.shapes[0].path.stroke.as_ref().expect("ha' traco");
    let (a, b) = s.dash.expect("ha' tracejado");
    assert!(
        (a - 2.0).abs() < 1e-9 && (b - 3.0).abs() < 1e-9,
        "dash: {a}, {b}"
    );
}

/// ⭐ **O gradiente viaja com a geometria** — ele entra em coordenadas locais e passa pela MESMA
/// porta que as âncoras, senão a rampa ficaria noutro sítio que a forma.
#[test]
fn a_gradient_travels_with_the_geometry() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
      <defs><linearGradient id="g" gradientUnits="userSpaceOnUse" x1="0" y1="0" x2="100" y2="0">
        <stop offset="0" stop-color="#ff0000"/><stop offset="1" stop-color="#0000ff"/>
      </linearGradient></defs>
      <rect x="0" y="0" width="100" height="100" fill="url(#g)"/>
    </svg>"##;
    let d = ler(svg);
    let Some(Paint::Linear { stops, start, end }) = &d.shapes[0].path.fill else {
        panic!(
            "tem de entrar como gradiente linear: {:?}",
            d.shapes[0].path.fill
        );
    };
    assert_eq!(stops.len(), 2);
    assert!(
        (start[0] - (-50.0)).abs() < 1e-6 && (end[0] - 50.0).abs() < 1e-6,
        "a rampa tem de atravessar a forma centrada: {start:?} -> {end:?}"
    );
}

/// **A regra de preenchimento vem do ficheiro** — um `evenodd` mal traduzido faz um buraco virar
/// tinta, e o desenho parece certo até haver um contorno interior.
#[test]
fn the_fill_rule_comes_from_the_file() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100" viewBox="0 0 100 100">
      <path fill-rule="evenodd" fill="#000"
            d="M 0 0 H 50 V 50 H 0 Z M 10 10 H 40 V 40 H 10 Z"/>
    </svg>"##;
    let d = ler(svg);
    assert_eq!(d.shapes[0].path.fill_rule, FillRule::EvenOdd);
    assert_eq!(
        d.shapes[0].path.subpaths.len(),
        1,
        "o 2.o contorno tem de entrar como sub-caminho (um compound), nao como forma nova"
    );
}

/// **O `Z` não deixa um vértice duplicado** — o ficheiro que volta ao início à mão e fecha teria
/// um segmento de comprimento zero, e ele reaparece em todo gesto que percorre o contorno.
#[test]
fn an_explicit_return_to_the_start_does_not_leave_a_twin() {
    let d = ler(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 10 10">
      <path d="M 0 0 L 8 0 L 8 8 L 0 0 Z" fill="#000"/></svg>"##,
    );
    let v = &d.shapes[0].path.verts;
    assert_eq!(v.len(), 3, "tres cantos, nao quatro: {v:?}");
    assert!(d.shapes[0].path.closed);
}

/// **Um ficheiro que não é SVG é RECUSADO** — nunca um documento vazio que se lê como *"o import
/// funcionou e o desenho estava em branco"*.
#[test]
fn something_that_is_not_an_svg_is_refused_not_silently_empty() {
    assert!(import(b"", &Options::default()).is_err());
    assert!(import(b"<svg this is not closed", &Options::default()).is_err());
    let enorme = vec![b' '; (crate::MAX_SVG_BYTES + 1) as usize];
    assert!(matches!(
        import(&enorme, &Options::default()),
        Err(crate::Error::TooLarge(_))
    ));
}

/// **A composição de afins tem uma ORDEM, e ela é `abs` primeiro** — trocá-la coloca cada forma
/// noutro sítio, e um ficheiro com um único `transform` de translação continuaria a parecer certo.
#[test]
fn the_frame_is_applied_after_the_nodes_own_transform() {
    let escala = Xform([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]);
    let translada = Xform([1.0, 0.0, 0.0, 1.0, 10.0, 0.0]);
    assert_eq!(translada.then(&escala).apply([0.0, 0.0]), [20.0, 0.0]);
    assert_eq!(escala.then(&translada).apply([0.0, 0.0]), [10.0, 0.0]);
}

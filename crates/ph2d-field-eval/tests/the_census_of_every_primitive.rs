//! ⭐⭐⭐ **O CENSO DE TODA PRIMITIVA** (W101) — quatro perguntas, uma lista, **derivada**.
//!
//! | pergunta | gate |
//! |---|---|
//! | o campo ainda é uma distância? | `every_primitive_honours_the_march` |
//! | a caixa do mundo contém a peça? | `the_bounding_radius_contains_the_piece` |
//! | o filete que o documento aceita ainda deixa peça? | `the_biggest_fillet_still_leaves_a_body` |
//! | *(o controle das três)* | `the_probe_can_see_a_field_that_climbs_too_fast` |
//!
//! ⭐ **Uma lista só** ([`representative`]), e é isso que torna o censo barato de estender: uma
//! primitiva nova é **erro de compilação** ali, e as três perguntas passam a valer para ela sem uma
//! linha de mudança.
//!
//! # ⚠️ Três destes gates nasceram de MUTAÇÕES QUE SOBREVIVERAM
//!
//! A W101 gateou o **campo** (a forma sai certa) e não a **API do documento**. Sobreviveram, na
//! primeira ronda: o `bounding_radius` da cápsula trocado por uma hipotenusa, o `round_limit` do
//! cone sem o `√(1+m²)`, e o `set_round` a esquecer as formas novas. *Escrevi a guarda certa e não
//! a gateei* — pela terceira vez neste repo.
//!
//! # A afirmação da marcha
//!
//! A marcha de esferas anda `d · SAFE_STEP` e é segura enquanto `SAFE_STEP · ‖∇f‖ ≤ 1`. As três
//! formas da W101 ([`Primitive::Cone`], [`Primitive::Capsule`], [`Primitive::Prism`]) são
//! construídas por `max` de meias-fatias, e o argumento é geométrico e não empírico: **o máximo de
//! funções 1-Lipschitz é 1-Lipschitz**. Este gate mede-o em vez de o acreditar.
//!
//! # ⚠️ Por que ele existe, e o que ele substitui
//!
//! A sonda que já havia — `the_table_of_who_inflates_the_gradient` — é `#[ignore]` (imprime uma
//! tabela) **e percorre uma lista escrita à mão**. Um `Primitive` novo não aparecia nela, e nada
//! reprovava: a mesma família de defeito que a W53 pagou com uma feature inteira invisível, um
//! nível abaixo.
//!
//! ⭐ Aqui a lista sai de [`PrimitiveKind::ALL`], e o `match` de [`representative`] é **exaustivo**
//! ⇒ uma primitiva nova é **erro de compilação** até alguém dizer com que números ela se mede.
//!
//! # ⚠️ O que a sonda NÃO acusa
//!
//! Numa quina convexa a distância é exacta e `‖∇f‖ = 1`; num vinco côncavo a derivada não existe e
//! a diferença central lê **menos** que 1. O que se caça é o contrário — uma região **lisa** onde o
//! campo sobe mais depressa que a distância, que é o que faz a marcha atravessar a superfície.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, PrimitiveKind, Xform};
use ph2d_field_eval::Field;

/// A folga sobre `1`. ⚠️ Ela é do **instrumento**, não da forma: a diferença central com
/// `eps = 1e-4` sobre uma quina lê um pouco acima de 1 por amostragem, e o `SAFE_STEP = 1/√2` do
/// módulo já concede `1,414`. Um `1,02` aqui é 70× mais apertado do que o que a marcha tolera.
const SLACK: f64 = 1.02;

/// ⭐ **Uma peça representativa de cada família** — e o `match` é o que fecha a corrente.
///
/// ⚠️ Os números não são decorativos: cada um põe a forma dentro da caixa `[-1, 1]³` que a sonda
/// varre, com **inclinação a sério** onde ela existe (um cone quase-cilíndrico não testaria o
/// `√(1+m²)` da parede, que é precisamente o termo que a W101 acrescentou).
fn representative(k: PrimitiveKind) -> Option<Primitive> {
    Some(match k {
        PrimitiveKind::Box => Primitive::Box {
            half: [0.4, 0.3, 0.25],
            round: 0.08,
            chamfer: 0.0,
        },
        PrimitiveKind::Sphere => Primitive::Sphere { radius: 0.5 },
        PrimitiveKind::Cylinder => Primitive::Cylinder {
            radius: 0.4,
            half_height: 0.3,
            round: 0.08,
            chamfer: 0.0,
        },
        PrimitiveKind::Torus => Primitive::Torus {
            major: 0.4,
            minor: 0.15,
        },
        // ⚠️ As duas de PERFIL ficam de fora **desta** sonda, e não é omissão: elas precisam de um
        // contorno desenhado, a `the_table_of_who_inflates_the_gradient` já as mede com um, e o que
        // este gate defende é a lista de primitivas de FÓRMULA estar completa. `None` é uma
        // resposta declarada, não um esquecimento — e o `match` continua exaustivo.
        PrimitiveKind::Extrude | PrimitiveKind::Revolve => return None,
        PrimitiveKind::Cone => Primitive::Cone {
            bottom: 0.45,
            top: 0.12,
            half_height: 0.35,
            round: 0.06,
            chamfer: 0.0,
        },
        PrimitiveKind::Capsule => Primitive::Capsule {
            radius: 0.25,
            half_height: 0.4,
        },
        // ⚠️ **ESTREITADO**, e não o prisma recto: um prisma de paredes verticais não testaria o
        // `√(1+m²)` da parede inclinada, que é o termo que a W102 acrescentou.
        PrimitiveKind::Prism => Primitive::Prism {
            sides: 6,
            bottom: 0.45,
            top: 0.18,
            half_height: 0.3,
            round: 0.05,
            chamfer: 0.0,
        },
        PrimitiveKind::Wedge => Primitive::Wedge {
            half: [0.45, 0.3, 0.35],
            round: 0.05,
            chamfer: 0.0,
        },
        // ⚠️ **Meia volta e um pouco**, de propósito: é o lado do `min` do sector, e um arco de menos
        // de meia volta nunca lá chegaria.
        PrimitiveKind::TorusArc => Primitive::TorusArc {
            major: 0.4,
            minor: 0.15,
            angle: std::f64::consts::PI as f32 * 1.3,
            round: 0.04,
            chamfer: 0.0,
        },
        // ⚠️ **CINCO pontas**, que é o número ímpar: com um par, metade das paredes cai sobre a
        // outra metade por simetria, e uma costura entre pipas vizinhas nunca seria varrida em
        // ângulo genérico.
        PrimitiveKind::Star => Primitive::Star {
            points: 5,
            outer: 0.45,
            inner: 0.18,
            half_height: 0.25,
            round: 0.03,
            chamfer: 0.0,
        },
        PrimitiveKind::BoxFrame => Primitive::BoxFrame {
            half: [0.45, 0.35, 0.4],
            thickness: 0.12,
            round: 0.03,
            chamfer: 0.0,
        },
        // ⚠️ **Os três semi-eixos DIFERENTES**: com dois iguais o campo é o de um esferóide, e a
        // razão `min/max` — que é exatamente o que esta forma subestima — só aparece num par.
        PrimitiveKind::Ellipsoid => Primitive::Ellipsoid {
            radii: [0.5, 0.2, 0.35],
        },
        // ─────────────────────────── W106 ───────────────────────────
        // ⚠️ **Cada representante escolhe o caso que EXERCITA a fórmula**, não o mais bonito: um
        // default cómodo esconde exactamente o termo que a forma acrescentou.
        PrimitiveKind::Octahedron => Primitive::Octahedron {
            radius: 0.45,
            round: 0.06,
            chamfer: 0.0,
        },
        // ⚠️ **Raios DIFERENTES**: com dois iguais ele degenera na cápsula, e o termo tangente —
        // que é tudo o que esta forma acrescenta — nunca seria exercitado.
        PrimitiveKind::RoundCone => Primitive::RoundCone {
            bottom: 0.35,
            top: 0.14,
            half_height: 0.3,
        },
        // ⚠️ Corte ACIMA do equador: em `cut = 0` a tampa é o disco maior e o caso é o mais fácil.
        PrimitiveKind::CutSphere => Primitive::CutSphere {
            radius: 0.45,
            cut: 0.15,
            round: 0.05,
            chamfer: 0.0,
        },
        PrimitiveKind::HollowDome => Primitive::HollowDome {
            radius: 0.45,
            cut: 0.1,
            thickness: 0.1,
            round: 0.03,
            chamfer: 0.0,
        },
        PrimitiveKind::Link => Primitive::Link {
            major: 0.3,
            minor: 0.1,
            length: 0.25,
        },
        PrimitiveKind::SolidAngle => Primitive::SolidAngle {
            radius: 0.45,
            angle: 0.7,
            round: 0.05,
            chamfer: 0.0,
        },
        // ⚠️ **Sete dentes**, que é ímpar pela razão da estrela: com um par, metade dos flancos cai
        // sobre a outra metade por simetria e uma costura entre dentes vizinhos nunca seria varrida
        // em ângulo genérico.
        PrimitiveKind::Gear => Primitive::Gear {
            teeth: 7,
            root: 0.32,
            outer: 0.45,
            tooth: 0.45,
            half_height: 0.15,
            round: 0.02,
            chamfer: 0.0,
        },
        PrimitiveKind::Cross => Primitive::Cross {
            arm: 0.45,
            width: 0.14,
            half_height: 0.12,
            round: 0.03,
            chamfer: 0.0,
        },
        PrimitiveKind::Heart => Primitive::Heart {
            size: 0.3,
            half_height: 0.12,
            round: 0.03,
            chamfer: 0.0,
        },
        PrimitiveKind::Moon => Primitive::Moon {
            radius: 0.45,
            bite: 0.4,
            offset: 0.2,
            half_height: 0.12,
            round: 0.02,
            chamfer: 0.0,
        },
        PrimitiveKind::Drop => Primitive::Drop {
            radius: 0.22,
            height: 0.55,
            half_height: 0.12,
            round: 0.03,
            chamfer: 0.0,
        },
        PrimitiveKind::Pie => Primitive::Pie {
            radius: 0.45,
            angle: 1.0,
            half_height: 0.12,
            round: 0.03,
            chamfer: 0.0,
        },
        // ⚠️ **Bases DIFERENTES**: iguais seria uma caixa, e o flanco inclinado — o `√(1+m²)` que
        // esta forma tem — não seria exercitado.
        PrimitiveKind::Trapezoid => Primitive::Trapezoid {
            bottom: 0.45,
            top: 0.2,
            half_width: 0.3,
            half_height: 0.12,
            round: 0.03,
            chamfer: 0.0,
        },
        PrimitiveKind::Vesica => Primitive::Vesica {
            radius: 0.45,
            offset: 0.25,
            half_height: 0.12,
            round: 0.02,
            chamfer: 0.0,
        },
        // ─────────────────────────── W119 ───────────────────────────
        // ⚠️ **Cada representante escolhe o caso que EXERCITA a fórmula**: uma haste FINA contra uma
        // ponta larga (a farpa a sério), e não uma seta quase-rectangular.
        PrimitiveKind::Arrow => Primitive::Arrow {
            heads: 1,
            half_length: 0.45,
            shaft: 0.09,
            head: 0.24,
            head_length: 0.26,
            half_height: 0.10,
            round: 0.03,
            chamfer: 0.0,
        },
        PrimitiveKind::Chevron => Primitive::Chevron {
            half_length: 0.40,
            half_span: 0.30,
            thickness: 0.09,
            half_height: 0.10,
            round: 0.02,
            chamfer: 0.0,
        },
        // ⚠️ **Braços DESIGUAIS**: com `run == rise` a peça é simétrica na diagonal e metade das
        // costuras cai sobre a outra metade.
        PrimitiveKind::BentArrow => Primitive::BentArrow {
            run: 0.42,
            rise: 0.34,
            shaft: 0.08,
            head: 0.18,
            head_length: 0.20,
            half_height: 0.10,
            round: 0.02,
            chamfer: 0.0,
        },
        // ⚠️ **Diagonais DIFERENTES**: iguais seria um quadrado rodado, e o par de flancos que não é
        // ortogonal ao outro — que é tudo o que esta forma acrescenta — não seria exercitado.
        PrimitiveKind::Rhombus => Primitive::Rhombus {
            half_width: 0.45,
            half_span: 0.26,
            half_height: 0.10,
            round: 0.03,
            chamfer: 0.0,
        },
        // ⚠️ **Com SECTOR**, e não o anel fechado: o anel é o caso em que o sector sai da árvore, e
        // um representante assim não mediria os dois semiplanos nem os aros que eles formam.
        PrimitiveKind::Tube => Primitive::Tube {
            outer: 0.45,
            inner: 0.26,
            angle: 1.1,
            half_height: 0.12,
            round: 0.03,
            chamfer: 0.0,
        },
        // ⚠️ Corte ACIMA do centro: em `cut = 0` sai o semicírculo, que é o caso mais fácil.
        PrimitiveKind::CircleSegment => Primitive::CircleSegment {
            radius: 0.45,
            cut: 0.16,
            half_height: 0.10,
            round: 0.03,
            chamfer: 0.0,
        },
        // ─────────────────────────── W120 ───────────────────────────
        PrimitiveKind::SpeechRect => Primitive::SpeechRect {
            half_width: 0.42,
            half_span: 0.28,
            tail: 0.20,
            half_height: 0.10,
            round: 0.05,
            chamfer: 0.0,
        },
        // ⚠️ **Eixos DIFERENTES**: iguais o oval é um disco, e o termo que ele acrescenta — a
        // subestimação por `min/max` — não seria exercitado.
        PrimitiveKind::SpeechOval => Primitive::SpeechOval {
            half_width: 0.44,
            half_span: 0.26,
            tail: 0.20,
            half_height: 0.10,
            round: 0.04,
            chamfer: 0.0,
        },
        // ⚠️ **Cinco bossas E cauda**: sem cauda ela é a outra porta, e com bossas pares metade das
        // costuras cai sobre a outra por simetria.
        PrimitiveKind::Cloud => Primitive::Cloud {
            lobes: 5,
            half_width: 0.45,
            half_span: 0.22,
            tail: 0.18,
            half_height: 0.10,
            round: 0.03,
            chamfer: 0.0,
        },
        PrimitiveKind::Bolt => Primitive::Bolt {
            half_width: 0.28,
            half_span: 0.45,
            half_height: 0.10,
            round: 0.02,
            chamfer: 0.0,
        },
        PrimitiveKind::Shield => Primitive::Shield {
            half_width: 0.34,
            half_span: 0.44,
            half_height: 0.10,
            round: 0.04,
            chamfer: 0.0,
        },
        PrimitiveKind::Tag => Primitive::Tag {
            half_width: 0.45,
            half_span: 0.26,
            point: 0.24,
            hole: 0.07,
            half_height: 0.10,
            round: 0.03,
            chamfer: 0.0,
        },
        // ⚠️ **Braços DESIGUAIS** — é o que um visto é, e com eles iguais a peça vira um «V».
        PrimitiveKind::Check => Primitive::Check {
            half_width: 0.42,
            half_span: 0.30,
            thickness: 0.11,
            half_height: 0.10,
            round: 0.03,
            chamfer: 0.0,
        },
        PrimitiveKind::Banner => Primitive::Banner {
            half_width: 0.45,
            half_span: 0.22,
            notch: 0.14,
            half_height: 0.10,
            round: 0.03,
            chamfer: 0.0,
        },
        PrimitiveKind::Brace => Primitive::Brace {
            half_span: 0.44,
            thickness: 0.09,
            half_height: 0.10,
            round: 0.02,
            chamfer: 0.0,
        },
    })
}

/// O maior `‖∇f‖` sobre uma grelha densa da caixa `[-e, e]³`.
///
/// ⚠️ **Recebe o [`Field`], e não a primitiva**, e é isso que deixa o controle abaixo medir-se pelo
/// **mesmo** instrumento: uma sonda testada com um campo que ela própria não produz é uma sonda
/// cujo controle não controla nada.
/// ⛔⛔ **A GRELHA GROSSA sozinha mede a forma ONDE ELA É LISA** (2026-08-30, o 2.º report do Enio).
///
/// `30³` sobre `[−1,2; 1,2]` dá células de `0,08`, e um recuo é uma casca de `~0,1` em volta das
/// arestas: a varredura passa por cima dela. A segunda passagem é **fina e só perto da superfície**,
/// que é onde a marcha de facto decide. Medido na caixa com os dois recuos: a grossa lê `0,79`, a
/// fina lê `0,85`.
fn worst_gradient(f: &Field, e: f64, steps: usize) -> f64 {
    let mut worst = 0.0f64;
    let mut varre = |e: f64, steps: usize, banda: Option<f64>| {
        let at = |t: usize| -e + 2.0 * e * (t as f64 + 0.5) / steps as f64;
        for i in 0..steps {
            for j in 0..steps {
                for k in 0..steps {
                    let (x, y, z) = (at(i), at(j), at(k));
                    if banda.is_some_and(|b| f.at(x, y, z).abs() > b) {
                        continue;
                    }
                    let g = f.gradient_norm(x, y, z, 1.0e-4);
                    if g.is_finite() {
                        worst = worst.max(g);
                    }
                }
            }
        }
    };
    varre(e, steps, None);
    // ⭐ A casca: `0,022` de célula, contra os `0,08` da grossa.
    varre(0.85, 78, Some(0.03));
    worst
}

fn doc_of(p: Primitive) -> FieldDoc {
    FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("a peça")
}

fn field_of(p: Primitive) -> Field {
    Field::new(&doc_of(p))
}

/// ⭐⭐⭐ **O produto `passo × ‖∇f‖` é o que fura, e é ele que este gate mede.**
///
/// # ⛔ A barra era `‖∇f‖ ≤ 1,02`, e a W103 mostrou que essa pergunta é a errada
///
/// Enquanto **toda** primitiva era 1-Lipschitz, medir só o gradiente dava no mesmo. Deixou de dar
/// no dia em que o filete do cone passou a existir: ele arredonda por **interseção arredondada** (a
/// única saída com paredes não-ortogonais), e isso infla — `1,1943` medido, que é o `√(1 − cos φ)`
/// do canto. Baixar a barra seria **afrouxar o gate**; a resposta certa é perguntar ao módulo qual
/// o passo que ele vai de facto usar nesta peça ([`ph2d_field_eval::safe_march_step`]) e exigir que
/// o **produto** seja seguro.
///
/// ⭐ Isto é estritamente MAIS forte do que a barra antiga: uma primitiva que inflasse **sem** o
/// declarar ao [`ph2d_field::fillet_inflates`] passaria a barra de `1,02`… não passaria — mas
/// também não passa aqui, e agora com a causa nomeada (o passo continua em `1,0`).
#[test]
fn every_primitive_honours_the_march() {
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else {
            continue;
        };
        let doc = doc_of(p.clone());
        let passo = f64::from(ph2d_field_eval::safe_march_step(&doc));
        let g = worst_gradient(&Field::new(&doc), 1.0, 24);
        assert!(
            passo * g <= SLACK,
            "«{}»: passo {passo:.4} × ‖∇f‖ {g:.4} = {:.4} — acima de 1 a marcha ATRAVESSA a \
             superfície",
            k.key(),
            passo * g
        );
    }
}

/// ⭐⭐⭐ **TODA FORMA OFERECE PELO MENOS UM NÚMERO** — o censo que fecha o braço `_` da tabela das
/// chapas.
///
/// # ⛔ Por que ele nasceu (W119)
///
/// A tabela por-primitiva passou as `700` linhas do gate de LOC e partiu-se em dois arquivos: o
/// [`ph2d_field::dims`] delega a família das chapas a um irmão, e o irmão acaba num braço `_` que
/// devolve `Vec::new()`. Esse braço é **inalcançável pelo caminho do produto** — o único chamador
/// nomeia as catorze chapas uma a uma, e o `match` dele continua exaustivo, logo uma primitiva nova
/// é erro de compilação **lá**.
///
/// ⛔ **O que ele NÃO apanha é o outro erro:** pôr a forma nova na lista do braço que delega e
/// esquecê-la no irmão. Aí ela compila, a paleta cria-a, e o painel dela nasce **VAZIO** — um slider
/// que não existe não deixa rasto nenhum, que é a falha mais cara de diagnosticar (é a mesma lição
/// que o `set_round` da W101 registou, com as palavras dele).
///
/// ⇒ *um braço `_` sem um censo ao lado é uma licença.*
#[test]
fn every_primitive_offers_at_least_one_dimension() {
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else {
            continue;
        };
        let linhas = ph2d_field::dims(&p);
        assert!(
            !linhas.is_empty(),
            "«{}» não oferece número nenhum — o painel dela nasce vazio, e um slider que não existe \
             não deixa rasto",
            k.key()
        );
        // ⭐ E o filete tem de estar lá quando a forma o tem: é a linha que o braço `_` deixaria
        // cair primeiro, e a que o artista mais procura neste módulo.
        if ph2d_field::round_limit(&p).is_some() {
            assert!(
                linhas.iter().any(|d| d.key == "field.dim.round"),
                "«{}» tem filete e não o oferece no painel",
                k.key()
            );
        }
    }
}

/// ⭐⭐⭐ **TODA LINHA DE TODA FORMA MARCHA AO LONGO DA FAIXA DELA** — o gate que o report da nuvem
/// obrigou a escrever (Enio, 05/09: *«cloud completamente bugado»*).
///
/// # ⛔⛔⛔ O buraco, e ele era o maior desta família
///
/// Todo censo desta casa mede a forma **no ponto em que ela nasce**. ⇒ uma primitiva podia estar
/// perfeita no representante e **furar em quase todo o curso dos próprios controlos**, e nenhum
/// gate dizia nada. Medido na nuvem, pela porta do painel: `passo × ‖∇f‖` ia de `0,94` no
/// nascimento para **`1,29`** ao estreitar a largura e **`1,54`** ao subir o `Span` — acima de `1` a
/// marcha atravessa a superfície, e o que se vê **não é a peça**.
///
/// ⚠️ **O `every_counted_shape_marches_safely_at_its_own_ceiling` não o via**: ele varia a
/// CONTAGEM, e os três defeitos da nuvem estavam nas linhas **contínuas**.
///
/// ⭐ A régua é a do painel: arrasta-se cada linha para três pontos da faixa que ela **declara**, e
/// pergunta-se à marcha. ⚠️ Nada de valores inventados — sair da faixa mediria uma peça que o
/// documento recusa.
///
/// ⚠️ **É caro de propósito** (`43` formas × `~6` linhas × `3` pontos): é um gate de FECHO, como os
/// irmãos deste arquivo.
#[test]
fn every_row_of_every_primitive_marches_safely_across_its_range() {
    use ph2d_field::Span;
    let mut maus = Vec::new();
    let mut medidas = 0;
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else { continue };
        for (i, d) in ph2d_field::dims(&p).iter().enumerate() {
            #[allow(clippy::cast_precision_loss)]
            let alvos: Vec<f32> = match d.span {
                Span::Count { min, max } => vec![min as f32, max as f32],
                Span::Wall(w) | Span::WallFromZero(w) => vec![w * 0.15, w * 0.6, w * 0.9],
                Span::Turn(h) | Span::Walls(h) => vec![-h * 0.8, h * 0.8],
                Span::Locked | Span::Choice(_) => continue,
                // ⚠️ Sem parede, a faixa é o alcance da VISTA — e o que se varre é uma década em
                // volta do valor de nascimento, que é o que uma mão alcança.
                Span::FromZero | Span::Positive | Span::Free | Span::Along => {
                    let v = d.value.abs().max(0.05);
                    vec![v * 0.25, v * 2.0, v * 4.0]
                }
            };
            for alvo in alvos {
                let mut q = p.clone();
                if ph2d_field::set_dim(&mut q, 0, i, alvo).is_err() {
                    continue;
                }
                ph2d_field::clamp_round(&mut q);
                let Ok(doc) = FieldDoc::new(
                    vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(q))],
                    NodeId(0),
                ) else {
                    continue;
                };
                medidas += 1;
                let passo = f64::from(ph2d_field_eval::safe_march_step(&doc));
                let g = worst_gradient(&Field::new(&doc), 1.0, 20);
                // ⚠️ Uma forma com tecto DECLARADO responde pela folga dela — ver
                // [`TETO_MEDIDO_E_NAO_CURADO`], que já traz a tabela e o censo de obsolescência.
                let barra = teto_declarado(k.key())
                    .or_else(|| faixa_declarada(k.key()))
                    .unwrap_or(SLACK);
                if passo * g > barra {
                    maus.push(format!(
                        "«{}» com {} = {alvo:.3}: passo {passo:.4} × ‖∇f‖ {g:.4} = {:.4}",
                        k.key(),
                        d.key,
                        passo * g
                    ));
                }
            }
        }
    }
    assert!(
        medidas >= 300,
        "só {medidas} pontos medidos — a lista derivada das faixas partiu-se"
    );
    assert!(
        maus.is_empty(),
        "estas formas FURAM ao arrastar um controlo delas — a peça sai rasgada e o que se vê não é \
         a forma: {maus:#?}"
    );
}

/// ⛔⛔ **A CHAVE no extremo FINO do controlo dela — MEDIDA, DECLARADA e não curada.**
///
/// Com a espessura a `15 %` da parede (`0,033` numa peça de `0,44`) ela mede `1,023` contra a barra
/// de `1,02` — **`0,3 %` acima**, e só ali: em `25 %` e acima ela fica em `0,99`.
///
/// ⚠️ **Duas curas foram medidas e não a fecharam**: recuar a parede do filete de `0,50` para
/// `0,45` da espessura (a marcha nem se mexeu — `1,0234 → 1,0233`), e apertar o raio da união dos
/// arcos. ⇒ o que resta é a **casca** de um arco muito fino, onde a espessura da banda se aproxima
/// do raio da mistura, e curá-lo é desenho novo.
///
/// ⚠️ *Uma chave com a parede a `7,5 %` do tamanho dela é um fio de cabelo*, e o defeito vive nos
/// últimos `15 %` de um controlo. Fica com o número em vez de uma barra afrouxada.
const FAIXA_MEDIDA_E_NAO_CURADA: [(&str, f64); 1] = [("brace", 1.03)];

fn faixa_declarada(key: &str) -> Option<f64> {
    FAIXA_MEDIDA_E_NAO_CURADA
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}

/// ⭐⭐⭐ **TODA FORMA CONTADA MARCHA NO PRÓPRIO TETO** — o gate que torna um teto medido REAL (W120).
///
/// # ⛔ O buraco que ele tapa
///
/// Cinco formas desta casa têm uma **contagem** com teto (lados, pontas, dentes, pontas de seta,
/// bossas), e o [`representative`] de cada uma usa um valor **típico**. ⇒ o censo da marcha media a
/// forma que o artista vê ao criar, e **nunca** a que ele alcança arrastando o slider até ao fim.
///
/// ⚠️ **Um teto que ninguém corre é uma promessa**, e esta wave apanhou-a a falhar: a nuvem passava
/// a `5` bossas e furava a `8` — a primeira medição escreveu `12` a partir do **preço**, e só a
/// marcha disse que o número era outro.
///
/// ⭐ A lista é **derivada** da tabela de linhas: qualquer forma com uma [`ph2d_field::Span::Count`]
/// entra aqui sozinha, no dia em que nascer.
#[test]
fn every_counted_shape_marches_safely_at_its_own_ceiling() {
    use ph2d_field::Span;
    let mut maus = Vec::new();
    let mut medidas = 0;
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else { continue };
        for (i, d) in ph2d_field::dims(&p).iter().enumerate() {
            let Span::Count { max, .. } = d.span else {
                continue;
            };
            let mut no_teto = p.clone();
            #[allow(clippy::cast_precision_loss)]
            if ph2d_field::set_dim(&mut no_teto, 0, i, max as f32).is_err() {
                maus.push(format!(
                    "«{}»: a porta recusou o próprio teto {max}",
                    k.key()
                ));
                continue;
            }
            // ⚠️ **Subir uma contagem ENCOLHE o filete que a forma comporta**, e o produto já
            // sabe disso: quem escreve um número chama o [`ph2d_field::clamp_round`] a seguir. Sem
            // esta linha o gate media uma peça que o documento **recusa**, e a mensagem falava de um
            // filete grande de mais em vez da marcha.
            ph2d_field::clamp_round(&mut no_teto);
            medidas += 1;
            let doc = doc_of(no_teto);
            let passo = f64::from(ph2d_field_eval::safe_march_step(&doc));
            let g = worst_gradient(&Field::new(&doc), 1.0, 24);
            if passo * g > teto_declarado(k.key()).unwrap_or(SLACK) {
                maus.push(format!(
                    "«{}» no teto ({max}): passo {passo:.4} × ‖∇f‖ {g:.4} = {:.4}",
                    k.key(),
                    passo * g
                ));
            }
        }
    }
    assert!(
        medidas >= 4,
        "só {medidas} formas com contagem foram medidas — a lista derivada partiu-se"
    );
    assert!(
        maus.is_empty(),
        "estas formas FURAM no próprio teto — um teto que ninguém corre é uma promessa: {maus:#?}"
    );
}

/// ⛔⛔⛔ **AS FORMAS QUE FURAM NO PRÓPRIO TETO — MEDIDAS, DECLARADAS e ainda NÃO curadas.**
///
/// # A estrela (achado de 2026-09-05, e é PRÉ-EXISTENTE)
///
/// O [`ph2d_field::MAX_STAR_POINTS`] foi escrito a partir do **PREÇO** (*«a estrela chega ao preço
/// do prisma às 16 pontas»*) e ninguém correu a **marcha** lá. Corrida, ela diz outra coisa:
///
/// | pontas | 5 | 6 | 7 | 8 | **9** | 10 | 12 | 14 | 16 |
/// |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
/// | `passo × ‖∇f‖` | `0,71` | `0,71` | `0,82` | `0,88` | **`1,07`** | `1,22` | `1,61` | `2,06` | `3,47` |
///
/// ⇒ **o joelho está entre `8` e `9`**, e o teto shipado é `16` — o dobro. *É o §0 outra vez: um
/// limite escrito a partir de um recurso enquanto outro amarra primeiro.*
///
/// # ⚠️ Por que ela fica DECLARADA e não é curada aqui
///
/// ⛔ **Um gate de gradiente diz «pode furar»; só a IMAGEM diz «fura»** — a lei que o
/// `the_bend_draws_what_an_honest_march_draws` pagou nesta mesma crate, e quando os dois discordam
/// **manda a imagem**. Baixar um teto que já shipa tira produto alcançável a partir de um limite
/// **conservador**, e isso é decisão de quem vê (§0.8), não desta wave.
///
/// ⇒ o número fica aqui, com a tabela, e a wave que o resolver ou traz a imagem ou traz a cura.
///
/// ⚠️ **A folga é uma catraca e SÓ ENCOLHE** — o censo abaixo recusa uma entrada que já cumpra a
/// barra normal, para ela não virar licença para a próxima forma.
/// ⚠️ **A folga cobre as DUAS réguas deste arquivo**: o gate do tecto amostra a `24` e o da faixa a
/// `20`, e a mesma peça lê `3,55` num e `3,65` no outro. *Uma folga calibrada num instrumento
/// descreve outra coisa no instrumento ao lado.*
const TETO_MEDIDO_E_NAO_CURADO: [(&str, f64); 1] = [("star", 3.70)];

fn teto_declarado(key: &str) -> Option<f64> {
    TETO_MEDIDO_E_NAO_CURADO
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}

/// ⛔⛔ **A METADE QUE IMPEDE A CATRACA DE SUBIR** — cada entrada tem de **ainda** furar a barra
/// normal, e ficar **abaixo** da folga que declara.
#[test]
fn the_declared_ceiling_list_has_no_stale_entries() {
    use ph2d_field::Span;
    for (nome, folga) in TETO_MEDIDO_E_NAO_CURADO {
        let k = PrimitiveKind::ALL
            .iter()
            .find(|k| k.key() == nome)
            .unwrap_or_else(|| panic!("«{nome}» já não é uma forma — a entrada ficou órfã"));
        let p = representative(*k).unwrap_or_else(|| panic!("«{nome}» não tem representante"));
        let mut medido = None;
        for (i, d) in ph2d_field::dims(&p).iter().enumerate() {
            let Span::Count { max, .. } = d.span else {
                continue;
            };
            let mut q = p.clone();
            #[allow(clippy::cast_precision_loss)]
            ph2d_field::set_dim(&mut q, 0, i, max as f32).expect("o próprio teto");
            ph2d_field::clamp_round(&mut q);
            let doc = doc_of(q);
            let passo = f64::from(ph2d_field_eval::safe_march_step(&doc));
            medido = Some(passo * worst_gradient(&Field::new(&doc), 1.0, 24));
        }
        let v = medido.unwrap_or_else(|| {
            panic!("«{nome}» já não tem contagem — a entrada não descreve nada")
        });
        println!("  [teto] {nome}: {v:.4} (folga declarada {folga:.2})");
        assert!(
            v > SLACK,
            "«{nome}» já cumpre a barra normal ({v:.4}) — APAGUE a entrada, senão ela vira licença"
        );
        assert!(
            v < folga,
            "«{nome}» piorou para {v:.4}, acima da folga declarada de {folga:.2} — a catraca SÓ ENCOLHE"
        );
    }
}

/// ⭐⭐⭐ **TODA LINHA DE TODA FORMA SABE SER ESCRITA** — o gate que fecha o `_ => Err(bad("dim"))`
/// do [`ph2d_field::set_dim`] (W120).
///
/// # ⛔⛔ O buraco que ele tapa, e por que os outros não o viam
///
/// A tabela de linhas e a porta de escrita são **dois** `match` sobre a mesma forma, ligados por um
/// **índice**. O primeiro é exaustivo por variante; o segundo casa `(forma, índice)` e acaba num
/// braço `_`. ⇒ acrescentar uma primitiva com seis linhas e esquecer os braços de escrita **compila
/// e passa a suíte**: o painel pinta os seis controlos, o artista arrasta, e **nada acontece**.
///
/// ⚠️ *Um slider que se mexe e não faz nada é a falha mais cara de diagnosticar, porque não deixa
/// rasto* — a frase é do `set_round` da W101, e a W106 pagou-a outra vez com **catorze** formas
/// cujos sliders eram inertes. Nenhum dos dois censos existentes a apanha: o
/// `every_primitive_offers_at_least_one_dimension` mede a OFERTA, e a marcha mede o CAMPO.
///
/// ⭐ A régua é o produto: empurra-se cada linha para um valor diferente e pergunta-se à **tabela**
/// se ela mudou. ⚠️ Aceita-se que a porta **COAJA** (uma parede pode segurar o valor pedido) — o que
/// se recusa é ela **não mexer nada**, ou devolver erro sobre um valor que a própria faixa oferece.
#[test]
fn every_row_of_every_primitive_can_be_written() {
    use ph2d_field::Span;
    let mut mudos = Vec::new();
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else { continue };
        let linhas = ph2d_field::dims(&p);
        for (i, d) in linhas.iter().enumerate() {
            // ⚠️ **O alvo sai da FAIXA declarada**, e não de um delta fixo: uma contagem anda de um
            // em um, e um recuo de uma aresta vive num intervalo minúsculo.
            let alvo = match d.span {
                Span::Count { min, max } => {
                    #[allow(clippy::cast_precision_loss)]
                    let (lo, hi) = (min as f32, max as f32);
                    if (d.value - lo).abs() < 0.5 { hi } else { lo }
                }
                Span::Wall(w) | Span::WallFromZero(w) => {
                    if d.value < w * 0.5 {
                        w * 0.75
                    } else {
                        w * 0.25
                    }
                }
                Span::FromZero | Span::Positive | Span::Free | Span::Along => {
                    if d.value > 0.0 {
                        d.value * 0.5
                    } else {
                        0.25
                    }
                }
                Span::Turn(h) | Span::Walls(h) => {
                    if d.value.abs() < h * 0.5 {
                        h * 0.5
                    } else {
                        0.0
                    }
                }
                #[allow(clippy::cast_precision_loss)]
                Span::Choice(opcoes) => {
                    if d.value < 0.5 {
                        1.0
                    } else {
                        (opcoes.len() - 1) as f32
                    }
                }
                // ⛔ **A ÚNICA faixa que declara «não se escreve»** — e a porta que a recusa é a
                // mesma que a pinta. Uma linha assim não é um controlo morto: é um controlo que diz
                // que não é um.
                Span::Locked => continue,
            };
            let mut q = p.clone();
            match ph2d_field::set_dim(&mut q, 0, i, alvo) {
                Err(e) => mudos.push(format!(
                    "{} linha {i} ({}) RECUSOU {alvo}: {e:?}",
                    k.key(),
                    d.key
                )),
                Ok(()) => {
                    let depois = ph2d_field::dims(&q);
                    if (depois[i].value - d.value).abs() < 1.0e-7 {
                        mudos.push(format!(
                            "{} linha {i} ({}) aceitou {alvo} e NÃO MEXEU (ficou em {})",
                            k.key(),
                            d.key,
                            d.value
                        ));
                    }
                }
            }
        }
    }
    assert!(
        mudos.is_empty(),
        "estes controlos são pintados e NÃO ESCREVEM — o artista arrasta e nada acontece: {mudos:#?}"
    );
}

/// ⭐⭐⭐ **UMA FAIXA QUE OFERECE NEGATIVO ACEITA NEGATIVO** — o defeito PRÉ-EXISTENTE que o lote da
/// seta apanhou de passagem (W119).
///
/// # ⛔⛔ O defeito
///
/// A [`ph2d_field::Span::Free`] diz, no próprio doc, que ela é *«simétrica e sem parede nenhuma: uma
/// **posição**. As duas pontas são o alcance da vista, e **a de baixo é negativa**»* — e o painel
/// desenha o slider assim. A porta de escrita ([`ph2d_field::set_dim`]) recusava **tudo** o que
/// fosse `< 0`.
///
/// ⇒ o `Cut` de uma esfera cortada e o de uma cúpula oca desciam até meio do curso e o número
/// **parava lá, sem dizer porquê**. É exactamente a affordance que mente que a
/// [`ph2d_field::Span::WallFromZero`] foi criada para curar, um campo ao lado — e a lição está
/// escrita no doc dela desde a W101. *Uma faixa que oferece o que a porta recusa é uma affordance
/// que mente.*
///
/// ⚠️ **A lista é DERIVADA**: uma faixa `Free` nova entra sem uma linha aqui.
///
/// ⛔ **Prova de mutação:** devolver a guarda do `set_dim` a `value < 0.0` reprova em TRÊS formas —
/// a esfera cortada, a cúpula oca e o segmento de círculo.
#[test]
fn a_span_that_offers_negative_accepts_negative() {
    let mut casos = 0;
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else { continue };
        for (linha, d) in ph2d_field::dims(&p).iter().enumerate() {
            if !matches!(d.span, ph2d_field::Span::Free) {
                continue;
            }
            casos += 1;
            let alvo = -d.value.abs().max(0.05);
            let mut q = p.clone();
            ph2d_field::set_dim(&mut q, 0, linha, alvo).unwrap_or_else(|e| {
                panic!(
                    "«{}» linha {linha}: a faixa é `Free` (o slider desce abaixo de zero) e a porta \
                     RECUSOU {alvo} — {e:?}",
                    k.key()
                )
            });
            assert!(
                (ph2d_field::dims(&q)[linha].value - alvo).abs() < 1.0e-6,
                "«{}» linha {linha}: aceitou {alvo} e guardou {}",
                k.key(),
                ph2d_field::dims(&q)[linha].value
            );
            // ⭐ E o **zero** passa: uma posição em zero é a origem, não um estado inválido.
            let mut z = p.clone();
            ph2d_field::set_dim(&mut z, 0, linha, 0.0)
                .unwrap_or_else(|e| panic!("«{}» linha {linha}: recusou o zero — {e:?}", k.key()));
        }
    }
    assert!(
        casos >= 3,
        "só {casos} faixas `Free` no catálogo — este gate ficou sem sujeito e mede nada"
    );
}

/// ⛔ **O CONTROLE do gate acima**: uma faixa que **não** oferece negativo continua a recusá-lo.
///
/// ⚠️ Sem ele, abrir a guarda para tudo passaria o gate de cima — e um raio negativo é uma peça que
/// não existe, não uma posição.
#[test]
fn a_span_that_does_not_offer_negative_still_refuses_it() {
    let mut caixa = Primitive::Box {
        half: [0.4, 0.3, 0.25],
        round: 0.05,
        chamfer: 0.0,
    };
    assert!(
        ph2d_field::set_dim(&mut caixa, 0, 0, -0.2).is_err(),
        "uma largura negativa passou — a guarda abriu de mais"
    );
    let mut esfera = Primitive::Sphere { radius: 0.4 };
    assert!(
        ph2d_field::set_dim(&mut esfera, 0, 0, -0.1).is_err(),
        "um raio negativo passou"
    );
}

/// ⛔ **O CONTROLE do gate acima, e sem ele o produto passaria por ser pequeno.**
///
/// ⚠️ Se `safe_march_step` devolvesse um número minúsculo para tudo, o produto seria seguro em toda
/// a linha e o gate deixaria de medir. Esta metade exige o contrário: quem **não** infla tem de
/// andar a passo INTEIRO — é o `CLAUDE.md` §0 aplicado ao passo (*o caminho lento não define o teto
/// do rápido*), e é a propriedade que a W56f construiu.
#[test]
fn the_shapes_that_do_not_inflate_still_march_at_full_step() {
    let mut inteiros = 0;
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else {
            continue;
        };
        let passo = ph2d_field_eval::safe_march_step(&doc_of(p.clone()));
        if ph2d_field::fillet_inflates(&p) {
            assert!(
                passo < 1.0,
                "«{}» infla e mesmo assim anda a passo inteiro",
                k.key()
            );
        } else {
            assert!(
                (passo - 1.0).abs() < 1.0e-6,
                "«{}» não infla e anda a {passo:.4} — o caminho lento a definir o teto do rápido",
                k.key()
            );
            inteiros += 1;
        }
    }
    assert!(
        inteiros >= 4,
        "o controle perdeu o sujeito: {inteiros} formas a passo inteiro"
    );
}

/// ⭐⭐⭐ **A CAIXA DO MUNDO CONTÉM A PEÇA** — e uma mutação sobrevivente pediu este gate.
///
/// # ⚠️ O defeito que ele apanha é SILENCIOSO
///
/// O [`ph2d_field::bounding_radius`] é o que dá o alcance ao traçado e à malha. Um valor **pequeno
/// demais** não falha: a peça sai **cortada nas pontas**, e o artista culpa a forma. A cápsula é o
/// caso: a ponta dela está a `h + r` no eixo, e uma hipotenusa `√(h²+r²)` — que é o que as outras
/// primitivas usam — dá **menos**. A mutação que a trocou passou a suíte inteira.
///
/// ⚠️ **A régua é o CAMPO, e não outra fórmula nossa**: amostra-se a esfera de raio `bounding` em
/// muitas direções e exige-se que o campo lá seja **positivo** (fora). Comparar duas contas nossas
/// seria cego a uma mutação que mexesse nas duas.
#[test]
fn the_bounding_radius_contains_the_piece() {
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else {
            continue;
        };
        // ⚠️ **Um cabelo PARA FORA, e não o raio exacto.** Numa esfera o `bounding_radius` **é** a
        // superfície (0,5 para raio 0,5), e o campo ali é zero a menos de um ULP — pedir
        // estritamente positivo reprovaria a forma mais apertada e correcta que existe. A
        // afirmação verdadeira é *«nada da peça fica ALÉM do raio de contenção»*.
        let r = f64::from(ph2d_field::bounding_radius(&p)) * 1.001;
        let f = field_of(p);
        // ⛔⛔⛔ **MEDE-SE O ALCANCE DA PEÇA, e não se espera que uma amostra caia nele.**
        //
        // As duas versões anteriores deste gate falharam a mesma pergunta, cada uma à sua maneira,
        // e as duas ficaram VERDES sobre um defeito que chegou à tela (report do Enio, 30/08:
        // quatro setas para arcos pretos a atravessar uma cruz):
        //
        // | tentativa | por que ficou verde |
        // |---|---|
        // | `24×24` **direções**, campo `≥ 0` na esfera de raio `r` | exige **acertar na direção** do ponto mais afastado; a quina de um braço fino estava a `75,7°/17,3°` e a grelha passa a `7,5°` e `22,5°` — de raspão pelos dois lados |
        // | grelha de **pontos** na caixa | a saliência era de `0,003` e o passo da grelha `0,028` — *nenhuma amostra caiu lá dentro* |
        //
        // ⭐⭐⭐ *Uma fixtura de amostras prova o que amostrou.* ⇒ aqui **bissecta-se a superfície**
        // ao longo de cada direção e toma-se o **máximo** dos raios encontrados. Isso mede a
        // grandeza que a pergunta é — *até onde a peça chega* — em vez de esperar que uma amostra
        // caia por cima dela, e converge com a densidade de direções em vez de depender da sorte.
        const DIRS: usize = 96;
        let mut alcance = 0.0f64;
        let mut viu_peca = false;
        for i in 0..DIRS {
            for j in 0..(DIRS * 2) {
                let theta = std::f64::consts::PI * (f64::from(u32::try_from(i).unwrap()) + 0.5)
                    / DIRS as f64;
                let phi = std::f64::consts::TAU * (f64::from(u32::try_from(j).unwrap()) + 0.5)
                    / (DIRS * 2) as f64;
                let d = [
                    theta.sin() * phi.cos(),
                    theta.sin() * phi.sin(),
                    theta.cos(),
                ];
                let at = |t: f64| f.at(d[0] * t, d[1] * t, d[2] * t);
                // ⛔ O CONTROLE, colhido no mesmo varrimento: a peça tem de EXISTIR nalgum sítio.
                // ⚠️ Não se pergunta pelo centro — o de um toro é **vazio**, e a 1.ª escrita deste
                // controlo reprovou-o por isso.
                for n in 1..8 {
                    if at(r * f64::from(n) / 8.0) < 0.0 {
                        viu_peca = true;
                        break;
                    }
                }
                // Fora, à partida: se já está fora em `r`, esta direção não excede.
                if at(r) >= 0.0 {
                    // Bissecta entre 0 e r para achar onde a superfície está — mas só interessa
                    // o caso em que ela passa de r, então salta.
                    continue;
                }
                // ⚠️ Ainda DENTRO em `r` ⇒ a peça excede nesta direção. Acha até onde.
                let mut lo = r;
                let mut hi = r * 4.0;
                for _ in 0..40 {
                    let mid = 0.5 * (lo + hi);
                    if at(mid) < 0.0 { lo = mid } else { hi = mid }
                }
                alcance = alcance.max(hi);
            }
        }
        assert!(
            viu_peca,
            "«{}»: nenhuma direção encontrou peça — a varredura não tem sujeito",
            k.key()
        );
        assert!(
            alcance == 0.0,
            "«{}»: há peça a {alcance:.4} do centro e o bounding_radius diz {:.4} — a caixa do \
             mundo CORTA a peça, e o corte sai ESFÉRICO (um arco preto a atravessá-la)",
            k.key(),
            r / 1.001
        );
    }
}

/// ⭐⭐⭐ **O MAIOR FILETE QUE O DOCUMENTO ACEITA AINDA DEIXA PEÇA** — o segundo gate que uma mutação
/// sobrevivente pediu.
///
/// # ⚠️ O que ele defende
///
/// O `round_limit` de um cone tem de contar a **inclinação** (`a/√(1+m²)`): a receita do filete
/// recua cada parede na **perpendicular**, e na parede inclinada isso baixa `a` de
/// `round·√(1+m²)`, não de `round`. Sem o `√`, o documento aceita um filete que **inverte a
/// parede** — e o que sai é uma peça **vazia**, aceite pela validação, sem uma palavra.
///
/// ⚠️ **A régua é «há peça», e não um número:** compara-se o campo no centro da forma, que tem de
/// estar dentro. É a pergunta mais fraca que ainda mata a mutação, e por isso a mais estável.
#[test]
fn the_biggest_fillet_still_leaves_a_body() {
    for k in PrimitiveKind::ALL {
        let Some(mut p) = representative(k) else {
            continue;
        };
        let Some(limite) = ph2d_field::round_limit(&p) else {
            continue;
        };
        // Um cabelo abaixo do limite — a validação recusa `>=`.
        let quase = limite * 0.999;
        // ⭐⭐⭐ **AS DUAS PORTAS, e são mesmo duas.** O filete escreve-se por
        // `ph2d_field::set_shape_radius` (o raio de uma forma, que o gizmo usa) **e** por
        // `dims::set_dim` no índice do filete (a linha do painel). ⚠️ Uma mutação que esvaziasse a
        // segunda **sobreviveu** a este gate quando ele só atravessava a primeira: *duas portas
        // para o mesmo número são duas coisas para gatear, e a que se esquece é a que o artista
        // usa.*
        // ⛔⛔ **CADA PORTA PARTE DO ORIGINAL, e a 1.ª versão deste laço não partia.**
        //
        // ⚠️ Ele fazia `p = escrita` no fim de cada volta, então a segunda porta já encontrava o
        // filete escrito pela primeira — e uma porta que **não escrevesse nada** passava, porque o
        // valor certo já lá estava. A mutação que esvazia o `dims::set_round` **sobreviveu a este
        // gate**, que era exactamente o gate escrito para a matar. *Um arnês que acumula estado
        // entre casos testa o primeiro caso duas vezes.*
        let original = p.clone();
        for porta in ["set_shape_radius", "set_dim"] {
            let mut escrita = original.clone();
            if porta == "set_shape_radius" {
                let mut shape = ph2d_field::NodeShape::Leaf(escrita.clone());
                ph2d_field::set_shape_radius(&mut shape, 0, quase).expect("aceite");
                let ph2d_field::NodeShape::Leaf(dentro) = shape else {
                    unreachable!("entrou folha, sai folha")
                };
                escrita = dentro;
            } else {
                let idx = ph2d_field::dims(&escrita)
                    .iter()
                    .position(|d| d.key == "field.dim.round")
                    .expect("uma forma com filete tem a linha dele");
                ph2d_field::set_dim(&mut escrita, 0, idx, quase).expect("aceite");
            }
            assert_eq!(
                ph2d_field::NodeShape::Leaf(escrita.clone()).radius(),
                Some(quase),
                "«{}» pela porta `{porta}`: o filete foi aceite e NÃO foi escrito — o controle \
                 mexe-se e não faz nada, sem deixar rasto",
                k.key()
            );
            p = escrita;
        }
        // ⛔⛔ **A PERGUNTA MUDOU NA W103, e a antiga estava ERRADA — não «apertada demais».**
        //
        // ⚠️ Ela era `f(0,0,0) < 0`: *«o centro da peça está dentro»*. Isso é uma afirmação sobre
        // sólidos **maciços**, e o [`Primitive::BoxFrame`] tem o miolo **vazio de propósito** — o
        // gate reprovou-o a dizer *«a parede inverteu, e a validação deixou passar»*, que é uma
        // causa que não existia. *Uma sonda que amostra UM ponto escolhido a olho carrega, sem o
        // dizer, a forma que o autor tinha em mente.*
        //
        // ⭐ A pergunta derivada faz o mesmo trabalho sem essa premissa: **conta** quantas amostras
        // de uma grelha caem dentro, com filete zero e com o filete máximo. `n1 > 0` é «ainda há
        // peça»; `n1 <= n0` é «o filete comeu, não cresceu» — e é este segundo lado que apanha a
        // parede invertida que a redação antiga NOMEAVA e nunca media.
        let n1 = inside_count(&p);
        let n0 = {
            // ⚠️ Pela porta do RAIO e não pelo `set_dim`: o zero é a aresta viva, e é ela que dá a
            // referência contra a qual o filete tem de tirar material.
            let mut shape = ph2d_field::NodeShape::Leaf(original.clone());
            ph2d_field::set_shape_radius(&mut shape, 0, 0.0).expect("aresta viva é sempre válida");
            let ph2d_field::NodeShape::Leaf(sem) = shape else {
                unreachable!("entrou folha, sai folha")
            };
            inside_count(&sem)
        };
        assert!(
            n1 > 0,
            "«{}»: com o maior filete que o documento aceita ({quase:.4}) NÃO sobrou peça nenhuma \
             — a validação deixou passar um filete que apaga a forma",
            k.key()
        );
        // ⛔⛔⛔ **ESTRITAMENTE MENOR, e é esta desigualdade que apanha o filete INERTE.**
        //
        // ⚠️ A W101 e a W102 shiparam o cone, o prisma, a cunha e a estrela com um `round` que
        // **não fazia nada** (`+0,0 %` de volume, campo bit a bit igual) ou que fazia a peça
        // **crescer** (`+41,0 %` na cunha). Nenhum gate reprovava, porque nenhum comparava a peça
        // COM o filete contra a peça SEM ele — mediam-se propriedades da forma filetada, e um campo
        // que ignora o filete continua a ser uma forma correcta. *Um `<=` teria deixado passar as
        // duas inertes; a inércia é precisamente a igualdade.*
        assert!(
            n1 < n0,
            "«{}»: o filete de {quase:.4} não tirou NADA ({n0} amostras dentro sem ele, {n1} com \
             ele) — ou ele é inerte, ou o recuo da fonte tem o sinal trocado",
            k.key()
        );
    }
}

/// Quantas amostras de uma grelha do bordo da peça caem **dentro** dela.
///
/// ⚠️ **A grelha sai do [`ph2d_field::bounding_radius`]**, e não de um número escrito aqui: uma
/// caixa fixa mediria a fração da caixa em peças de tamanhos diferentes, e a comparação de duas
/// formas deixaria de querer dizer alguma coisa.
fn inside_count(p: &Primitive) -> u32 {
    use fidget::shape::EzShape;
    // ⚠️ **A finura sai do FILETE, não do gosto** — e a primeira versão (`24`) reprovou a gaiola por
    // uma razão que não existia: com a viga a `0,12` e o passo a `0,060`, cabiam **duas** amostras
    // na secção, e um filete que come `21 %` dela não movia nenhuma. *Uma grelha que não resolve a
    // feature mede a grelha.* A `64` o passo é `0,023`, e a diferença aparece.
    const N: i32 = 64;
    // ⚠️ **Em FATIA, e não ponto a ponto**: a mesma grelha por [`Field::at`] custava **18 s** neste
    // gate, e um teste que ficou lento é uma medição de custo que ninguém pediu. A fita é a mesma; o
    // que muda é quantas vezes se atravessa a fronteira do avaliador.
    let r = f64::from(ph2d_field::bounding_radius(p)) * 1.05;
    let c = |v: i32| ((f64::from(v) / f64::from(N)).mul_add(2.0, -1.0) * r) as f32;
    let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
    for i in 0..=N {
        for j in 0..=N {
            for k in 0..=N {
                xs.push(c(i));
                ys.push(c(j));
                zs.push(c(k));
            }
        }
    }
    let shape = ph2d_field_eval::Engine::from(ph2d_field_eval::compile(&doc_of(p.clone())));
    let tape = shape.ez_float_slice_tape();
    let mut eval = ph2d_field_eval::Engine::new_float_slice_eval();
    let out = eval.eval(&tape, &xs, &ys, &zs).expect("avalia a grelha");
    u32::try_from(out.iter().filter(|v| **v < 0.0).count()).unwrap_or(u32::MAX)
}

/// ⛔ **O CONTROLE, e sem ele o gate acima não vale nada.**
///
/// ⚠️ Uma sonda que devolvesse sempre um número pequeno passaria as nove famílias e não mediria
/// coisa nenhuma. Um campo **deliberadamente esticado** (a esfera multiplicada por dois) tem
/// `‖∇f‖ = 2` por construção — se esta sonda não o vir, ela não vê nada.
///
/// ⚠️ E ele é construído **fora** do `Primitive`, de propósito: não há forma de o exprimir com uma
/// primitiva legítima, que é exactamente o que o gate acima afirma.
#[test]
fn the_probe_can_see_a_field_that_climbs_too_fast() {
    use fidget::context::Tree;
    let esticada = (Tree::x().square() + Tree::y().square() + Tree::z().square())
        .max(1.0e-30)
        .sqrt()
        * Tree::constant(2.0)
        - Tree::constant(1.0);
    let g = worst_gradient(&Field::from_tree(&esticada), 1.0, 12);
    assert!(
        g > 1.5,
        "a sonda leu ‖∇f‖ = {g:.4} num campo que sobe ao DOBRO da distância — ela não mede nada"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// ⭐⭐⭐ O CHANFRO — pedido do Enio, 2026-08-30
//
// > *«em todas as peças temos fillet para as bordas arredondadas mas não temos um slider para
// > chamfer. Poderíamos ter os 2, com chamfer antes de fillet para a possibilidade de arredondar as
// > bordas geradas por chamfer»*
//
// ⚠️ As três perguntas do censo valem para ele **sem uma linha de fixtura nova**, porque a lista é a
// mesma [`representative`] — que é a razão de o censo existir.
// ─────────────────────────────────────────────────────────────────────────────

/// Esta forma com o chanfro posto a `fracao` da parede que o documento declara.
fn com_chanfro(mut p: Primitive, fracao: f32) -> Option<Primitive> {
    let limite = ph2d_field::round_limit(&p)?;
    let alvo = limite * fracao;
    let i = ph2d_field::dims(&p)
        .iter()
        .position(|d| d.key == "field.dim.chamfer")?;
    ph2d_field::set_dim(&mut p, 0, i, alvo).ok()?;
    Some(p)
}

/// ⭐⭐⭐ **O CHANFRO MUDA TODA FORMA QUE O OFERECE** — e é a régua que separa um slider de um knob
/// morto.
///
/// ⚠️ **A lista é derivada** de [`PrimitiveKind::ALL`], então uma primitiva nova entra aqui sozinha.
/// ⛔ Uma lista escrita à mão teria deixado de fora exactamente a forma cujo construtor esqueceu de
/// ler o número — que é o defeito que este gate existe para apanhar, e que a W104 já pagou uma vez
/// com o `round` do cone e do prisma **inertes** (`+0,0 %` de volume, campo bit a bit igual).
#[test]
fn every_shape_that_offers_a_chamfer_is_changed_by_it() {
    let mut testadas = 0;
    let mut mudas = Vec::new();
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else { continue };
        let Some(chanfrada) = com_chanfro(p.clone(), 0.5) else {
            continue;
        };
        testadas += 1;
        let (vivo, cortado) = (inside_count(&p), inside_count(&chanfrada));
        // O chanfro **tira** material da quina: a contagem tem de descer, e não só mexer.
        let queda = f64::from(vivo.saturating_sub(cortado)) / f64::from(vivo.max(1)) * 100.0;
        if queda < 0.05 {
            mudas.push(format!("{k:?} ({vivo} -> {cortado}, {queda:.3} %)"));
        }
    }
    assert!(
        testadas >= 20,
        "o censo só chegou a {testadas} formas com aresta — a lista derivada partiu-se"
    );
    assert!(
        mudas.is_empty(),
        "estas formas oferecem o slider do chanfro e não são cortadas por ele: {mudas:?}"
    );
}

/// ⭐⭐⭐ **O CHANFRO HONRA A MARCHA, em toda forma que o oferece** — o produto `passo × ‖∇f‖`.
///
/// # ⛔ A 1.ª redacção deste gate media o gradiente CRU, e a resposta foi um achado
///
/// Ela afirmava que *o chanfro sozinho nunca infla* — «um `max` de funções 1-Lipschitz é
/// 1-Lipschitz» — e o censo refutou-a com o número que este arquivo já tinha escrito: **`1,1943` no
/// cone**, o `√(1 − cos φ)` do canto. *O plano do chanfro herda o ângulo das duas faces que ele
/// corta*, e a demonstração só vale enquanto as normais são ortogonais — que é precisamente o que
/// uma parede inclinada não é.
///
/// ⇒ a pergunta certa é a mesma que o [`every_primitive_honours_the_march`] já faz: **o produto**.
/// O `ph2d_field::fillet_inflates` passou a distinguir as quatro exactas (peças ortogonais, o
/// chanfro sozinho lê `1,0000`) das dezassete de parede inclinada (qualquer recuo infla).
#[test]
fn a_chamfer_honours_the_march_on_every_shape() {
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else { continue };
        let Some(mut chanfrada) = com_chanfro(p, 0.5) else {
            continue;
        };
        // ⭐ Sem filete: é o chanfro sozinho que está sob teste. ⚠️ E o zero **passa** desde a
        // `Span::WallFromZero` — antes dela esta linha era silenciosamente recusada e o gate media
        // os dois recuos juntos.
        let i = ph2d_field::dims(&chanfrada)
            .iter()
            .position(|d| d.key == "field.dim.round")
            .expect("uma forma com chanfro tem filete");
        ph2d_field::set_dim(&mut chanfrada, 0, i, 0.0).expect("o zero é a aresta viva");
        let passo = f64::from(ph2d_field_eval::safe_march_step(&doc_of(chanfrada.clone())));
        let g = worst_gradient(&field_of(chanfrada), 1.0, 24);
        assert!(
            passo * g <= SLACK,
            "{k:?}: passo {passo:.4} x ‖∇f‖ {g:.4} = {:.4} — a marcha atravessa a superfície",
            passo * g
        );
    }
}

/// ⭐⭐⭐ **TODA FORMA MARCHA EM SEGURANÇA COM OS DOIS RECUOS LIGADOS** — o gate que faltava.
///
/// # ⛔⛔ Ele nasceu de um report do Enio (2026-08-30), e o buraco era do CENSO
///
/// *«com um prisma veja que algumas arestas não receberam o fillet e ao rotacionar a aparência da
/// aresta muda»*. A segunda metade é a assinatura de um campo que **sobe mais depressa que a
/// distância**: a marcha atravessa a superfície, e o ponto em que ela pára passa a depender da
/// direcção do raio.
///
/// ⚠️ **O gate que existia media o par numa forma só — a caixa.** Medido depois do report, sobre as
/// **vinte** formas com aresta:
///
/// | forma | `passo × ‖∇f‖` antes | depois |
/// |---|---:|---:|
/// | `Octahedron` | **`3,4572`** | `0,8165` |
/// | `Wedge` | `1,8928` | `0,9981` |
/// | `Prism` | `1,4061` | `0,9860` |
/// | `Box` | `1,2237` | `1,0000` (passo **cheio**) |
/// | *(16 de 20 acima de `1`)* | | *(nenhuma)* |
///
/// ⇒ *um gate escrito para uma forma deixa as outras dezanove por medir*, que é a mesma família do
/// `the_fillet_reaches_every_edge_of_every_shape`.
///
/// # ⭐ A causa era a CONSTRUÇÃO, e não a feature
///
/// A 1.ª versão compunha chanfro-e-filete **misturando duas vezes**
/// (`intersection(intersection(a, plano, r), b, r)`), e cada nível encaixado soma um quadrado na lei
/// de Cauchy–Schwarz do [`ph2d_field_eval::gradient_bound`] — `√3` numa caixa, medido a `1,7306`.
/// Hoje é **encolher, chanfrar, deslocar**: um `max` de 1-Lipschitz é 1-Lipschitz.
#[test]
fn every_shape_marches_safely_with_both_recesses_on() {
    let mut furam = Vec::new();
    let mut testadas = 0;
    for k in PrimitiveKind::ALL {
        let Some(base) = representative(k) else {
            continue;
        };
        let Some(limite) = ph2d_field::round_limit(&base) else {
            continue;
        };
        let meio = limite * 0.5;
        let escreve = |p: &Primitive, chave: &str, v: f32| -> Option<Primitive> {
            let mut p = p.clone();
            let i = ph2d_field::dims(&p).iter().position(|d| d.key == chave)?;
            ph2d_field::set_dim(&mut p, 0, i, v).ok()?;
            Some(p)
        };
        let Some(par) = escreve(&base, "field.dim.round", meio)
            .and_then(|p| escreve(&p, "field.dim.chamfer", meio))
        else {
            continue;
        };
        testadas += 1;
        let d = doc_of(par.clone());
        let passo = f64::from(ph2d_field_eval::safe_march_step(&d));
        let g = worst_gradient(&field_of(par), 1.2, 30);
        if passo * g > SLACK {
            furam.push(format!("{k:?} {:.4}", passo * g));
        }
    }
    assert!(
        testadas >= 20,
        "só {testadas} formas com aresta — a lista derivada partiu-se"
    );
    assert!(
        furam.is_empty(),
        "estas formas marcham por cima da própria superfície com os dois recuos ligados: {furam:?}"
    );
}

/// ⭐⭐ **E as QUATRO de peças ORTOGONAIS andam o passo CHEIO com os dois recuos** — a caixa, o
/// cilindro, a extrusão e a moldura.
///
/// ⚠️ **O passo cheio não quer dizer que o campo delas não infla** — com chanfro ele infla
/// (`1,40`–`1,59` no cru, medido; ver `ph2d_field::fillet_inflates`). Quem o traz de volta abaixo
/// de `1` é o divisor `2` do `edge_shrink`, e é sobre o campo **já dividido** que o
/// [`ph2d_field_eval::safe_march_step`] responde. *O que este gate defende é o par
/// divisor+tecto ficar consistente, e não uma propriedade da forma.*
#[test]
fn the_four_exact_shapes_keep_the_full_step_with_both_recesses() {
    let caixa = |round: f32, chamfer: f32| Primitive::Box {
        half: [0.4, 0.3, 0.25],
        round,
        chamfer,
    };
    for (r, c) in [(0.0, 0.0), (0.05, 0.0), (0.0, 0.05), (0.05, 0.05)] {
        let passo = ph2d_field_eval::safe_march_step(&doc_of(caixa(r, c)));
        assert!(
            (passo - 1.0).abs() < 1e-6,
            "filete {r} + chanfro {c}: o campo dividido da caixa fica abaixo de `1` e ela tem de \
             andar o passo cheio, andou {passo}"
        );
    }
}

/// ⭐⭐⭐ **O RECUO ENTREGUE É O NÚMERO PEDIDO** — a régua que os quatro caracteres desta casa
/// partilham, medida na forma em que ela é inequívoca.
///
/// Uma caixa chanfrada em `c` tem de perder a quina a **exactamente** `c` de distância, ao longo de
/// cada uma das duas faces. ⚠️ Um chanfro que entregasse `0,71×` do pedido mentiria uma fracção
/// fixa, sempre — e trocar entre o chip *Chamfer* e este slider mudaria o tamanho da peça.
#[test]
fn the_chamfer_sets_back_exactly_what_it_promises() {
    const H: f64 = 0.4;
    for c in [0.05f64, 0.10, 0.15] {
        let f = field_of(Primitive::Box {
            half: [H as f32; 3],
            round: 0.0,
            chamfer: c as f32,
        });
        // Caminhamos na face `x = H` (fora dela o campo é positivo) e achamos onde a face acaba.
        let mut fim = 0.0f64;
        for i in 0..=4000 {
            let y = f64::from(i) / 4000.0 * H;
            // Na face plana o campo em `(H, y, 0)` é exactamente zero.
            if f.at(H, y, 0.0).abs() <= 1e-6 {
                fim = y;
            }
        }
        let recuo = H - fim;
        assert!(
            (recuo - c).abs() < 2e-3,
            "chanfro pedido {c:.3}: a face acaba a {recuo:.5} da quina, e tinha de acabar a {c:.3}"
        );
    }
}

/// ⭐⭐⭐ **O FILETE ARREDONDA a aresta do chanfro — ele não a MOVE.**
///
/// # ⛔⛔ Ele nasceu do 2.º report do Enio sobre a mesma feature (2026-08-30)
///
/// *«não funcionou. o fillet só muda a posição do chamfer»* — e ele estava literalmente certo. A
/// construção de então («encolher, chanfrar, deslocar») é a receita do filete da caixa, e ela
/// arredonda o que é **distância exacta**; um plano de chanfro é um **semiespaço**, e deslocar um
/// semiespaço dá outro semiespaço. É a lei que a W104 já tinha medido e escrito neste módulo, e eu
/// quebrei-a.
///
/// Medido, com o chanfro em `0,12`:
///
/// | filete | posição da quina | maior giro da normal |
/// |---:|---:|---:|
/// | `0,00` | `0,48083` | `40,2°` |
/// | `0,02` | `0,47255` | **`45,000°`** |
/// | `0,08` | `0,44770` | **`45,000°`** |
///
/// ⇒ o giro **cravado em `45°`** (uma quina perfeita) enquanto a posição desliza. *O gate que
/// existia media a MORDIDA — o volume que sai — e um chanfro deslocado tira volume na mesma.*
///
/// ⭐ A cura é arredondar as **três** superfícies de uma vez (as duas faces e o plano), com a
/// [`ph2d_field_eval::ops::intersection_round_n`]. Depois dela o giro cai a `1,3°` e `0,5°`.
#[test]
fn the_fillet_rounds_the_chamfer_edge_instead_of_moving_it() {
    let campo = |round: f32, chamfer: f32| {
        field_of(Primitive::Box {
            half: [0.4; 3],
            round,
            chamfer,
        })
    };
    // O maior giro da normal entre dois passos, ao longo da secção `z = 0` de um quadrante.
    let giro = |f: &Field| {
        let normal = |ang: f64| {
            let (c, s) = (ang.cos(), ang.sin());
            let (mut lo, mut hi) = (0.0f64, 2.0);
            for _ in 0..60 {
                let m = 0.5 * (lo + hi);
                if f.at(c * m, s * m, 0.0) <= 0.0 {
                    lo = m;
                } else {
                    hi = m;
                }
            }
            let r = 0.5 * (lo + hi);
            let (x, y, e) = (c * r, s * r, 1.0e-4);
            let gx = f.at(x + e, y, 0.0) - f.at(x - e, y, 0.0);
            let gy = f.at(x, y + e, 0.0) - f.at(x, y - e, 0.0);
            let m = gx.hypot(gy).max(1.0e-12);
            (gx / m, gy / m)
        };
        let mut pior = 0.0f64;
        let mut ant = normal(0.0);
        for i in 1..=900 {
            let a = std::f64::consts::FRAC_PI_2 * f64::from(i) / 900.0;
            let cur = normal(a);
            pior = pior.max(
                (ant.0 * cur.0 + ant.1 * cur.1)
                    .clamp(-1.0, 1.0)
                    .acos()
                    .to_degrees(),
            );
            ant = cur;
        }
        pior
    };
    let vivo = giro(&campo(0.0, 0.12));
    assert!(
        vivo > 20.0,
        "⛔ o CONTROLE falhou: um chanfro sem filete TEM de deixar uma quina, e o giro leu \
         {vivo:.2}° — a sonda não está a ver quinas"
    );
    for r in [0.02f32, 0.05] {
        let g = giro(&campo(r, 0.12));
        assert!(
            g < 5.0,
            "com o chanfro em 0,12 e o filete em {r}, o giro da normal é {g:.3}° — o filete está a \
             MOVER a quina em vez de a arredondar (report do Enio, 2026-08-30)"
        );
    }
}

/// ⭐ **O FILETE POR CIMA morde as arestas que o chanfro criou** — que é literalmente o pedido.
#[test]
fn the_fillet_bites_the_edges_the_chamfer_made() {
    let caixa = |round: f32| Primitive::Box {
        half: [0.4; 3],
        round,
        chamfer: 0.12,
    };
    let (so_chanfro, com_filete) = (inside_count(&caixa(0.0)), inside_count(&caixa(0.05)));
    assert!(
        com_filete < so_chanfro,
        "o filete tem de tirar mais material nas arestas novas do chanfro: {so_chanfro} -> \
         {com_filete}"
    );
}

/// ⭐⭐⭐ **OS DOIS RECUOS DE UMA ARESTA VOLTAM A ZERO** — e este gate cura um defeito
/// **pré-existente** que o chanfro tornou impossível de ignorar.
///
/// # ⛔ O defeito, e a lei que já estava escrita a nomeá-lo
///
/// O filete usava [`ph2d_field::Span::Wall`], e o painel mapeia essa faixa para um slider que **começa em
/// zero**. A porta de escrita, porém, recusava o zero — `Wall` promete «positiva». ⇒ o artista
/// arredondava uma aresta e **não conseguia desarredondá-la**: o controle descia até ao fundo e o
/// número parava logo acima dele, sem dizer porquê.
///
/// ⚠️ O doc da [`ph2d_field::Span::Count`] já descrevia exactamente esta família — *«uma faixa que oferece
/// o que a porta recusa é uma affordance que mente»* — e foi por isso que ela ganhou um `min`. O que
/// faltava era aplicar a mesma lei ao outro lado da faixa.
///
/// ⭐ Para o **chanfro** isto não seria conforto: zero é o estado de nascimento dele, e um knob que
/// só liga é um knob que prende o artista.
///
/// ⚠️ **A lista é derivada** de [`PrimitiveKind::ALL`] — uma forma nova entra sozinha.
#[test]
fn both_recesses_of_an_edge_can_go_back_to_zero() {
    let mut presas = Vec::new();
    let mut testadas = 0;
    for k in PrimitiveKind::ALL {
        let mut p = match representative(k) {
            Some(p) => p,
            None => continue,
        };
        let mut tem_aresta = false;
        for chave in ["field.dim.chamfer", "field.dim.round"] {
            let Some(i) = ph2d_field::dims(&p).iter().position(|d| d.key == chave) else {
                continue;
            };
            tem_aresta = true;
            let limite = ph2d_field::round_limit(&p).expect("tem parede");
            // Primeiro liga-se o recuo, senão o gate mede o estado em que ele já estava.
            ph2d_field::set_dim(&mut p, 0, i, limite * 0.5).expect("ligar o recuo");
            if ph2d_field::set_dim(&mut p, 0, i, 0.0).is_err() {
                presas.push(format!("{k:?}/{chave}"));
            } else {
                assert!(
                    (ph2d_field::dims(&p)[i].value).abs() < 1e-9,
                    "{k:?}/{chave}: a porta disse Ok e o número não foi a zero"
                );
            }
        }
        if tem_aresta {
            testadas += 1;
        }
    }
    assert!(
        testadas >= 20,
        "só {testadas} formas com aresta — a lista derivada partiu-se"
    );
    assert!(
        presas.is_empty(),
        "estes recuos não voltam a zero, e o slider do painel oferece o zero: {presas:?}"
    );
}

/// ⭐⭐ **ESCALAR UMA FORMA ESCALA OS DOIS RECUOS DELA** — o filete **e** o chanfro.
///
/// ⚠️ Os dois são **comprimentos da peça**, e um deles fixo faria a aresta mudar de carácter ao
/// redimensionar: uma caixa de `1` com chanfro de `0,1` tem um corte de 10 % da face; a mesma caixa
/// levada a `4` teria um corte de 2,5 %, e o artista veria a quina «endurecer» sozinha.
///
/// ⛔ **O compilador apontou este defeito** — 17 avisos de `chamfer` não usado no `scale_primitive`,
/// um por forma. *Um aviso de variável não usada num `match` exaustivo é o compilador a dizer que
/// alguém acrescentou um campo e esqueceu metade do trabalho.*
///
/// ⚠️ **A lista é derivada** de [`PrimitiveKind::ALL`].
#[test]
fn scaling_a_shape_scales_both_recesses() {
    const FATOR: f32 = 3.0;
    let mut paradas = Vec::new();
    let mut testadas = 0;
    for k in PrimitiveKind::ALL {
        let Some(base) = representative(k) else {
            continue;
        };
        let Some(mut p) = com_chanfro(base, 0.5) else {
            continue;
        };
        testadas += 1;
        let antes: Vec<(&str, f32)> = ph2d_field::dims(&p)
            .iter()
            .filter(|d| d.key == "field.dim.round" || d.key == "field.dim.chamfer")
            .map(|d| (d.key, d.value))
            .collect();
        assert!(
            ph2d_field::scale_primitive(&mut p, FATOR),
            "{k:?}: a escala foi recusada"
        );
        let depois: Vec<(&str, f32)> = ph2d_field::dims(&p)
            .iter()
            .filter(|d| d.key == "field.dim.round" || d.key == "field.dim.chamfer")
            .map(|d| (d.key, d.value))
            .collect();
        for ((chave, a), (_, b)) in antes.iter().zip(depois.iter()) {
            // ⚠️ Um recuo que já era zero continua zero — a régua é a RAZÃO, e ela só existe
            // sobre um número que estava ligado.
            if *a <= 0.0 {
                continue;
            }
            if ((b / a) - FATOR).abs() > 1e-3 {
                paradas.push(format!("{k:?}/{chave} {a:.5} -> {b:.5} (x{:.3})", b / a));
            }
        }
    }
    assert!(
        testadas >= 20,
        "só {testadas} formas com aresta — a lista derivada partiu-se"
    );
    assert!(
        paradas.is_empty(),
        "estes recuos não acompanharam a escala da forma: {paradas:?}"
    );
}

/// ⭐⭐⭐ **AS DUAS PORTAS QUE BAIXAM UM DOCUMENTO ENTREGAM O MESMO CAMPO** — o gate estrutural que
/// faltava, e o report do Enio de 2026-08-30 pagou-o.
///
/// # ⛔⛔ O defeito: uma cura escrita numa das duas rotas
///
/// Este módulo tem **duas** rotas de um [`FieldDoc`] para uma árvore, e elas duplicam a linha que
/// baixa uma FOLHA:
///
/// - [`ph2d_field_eval::compile_with`] — as sondas, os gates, e **todo este arquivo**;
/// - `hybrid::Builder` (por [`Hybrid::new`]) — **o produto**, porque só ele sabe o que é uma
///   escultura.
///
/// O divisor da aresta (`ph2d_field::edge_shrink`) foi escrito só na primeira. Medido no mesmo raio
/// e na mesma caixa: o campo do traçado vinha **`8×`** o dos gates. A marcha andava o passo cheio
/// sobre o campo cru, atravessava a superfície, e **catorze gates deste arquivo ficavam verdes**
/// porque mediam a rota que a produção não corre. *O que o Enio viu foram facetas escuras que
/// mudavam ao rodar; o que os números diziam era `passo × ‖∇f‖ ≤ 0,80`.*
///
/// ⇒ hoje o divisor mora DENTRO da única função que baixa uma primitiva, e este gate é o que
/// impede a próxima rota de o esquecer: ele não sabe o que é um divisor, **ele pergunta se as duas
/// portas concordam**.
///
/// ⚠️ **A tolerância é relativa e existe porque as duas portas correm em precisões diferentes**
/// (o `Field` é o interpretador `f64`, o `Hybrid` é o JIT `f32`) — ⛔ não porque se espere
/// divergência de FÓRMULA. O pior desvio relativo MEDIDO nas 20 formas é `3,68e-8`, e a barra
/// está `2 700×` acima dele: um factor esquecido é uma divergência de **100 %**, não de um ULP.
#[test]
fn the_two_doors_lower_a_leaf_the_same_way() {
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let mut pior = 0.0f64;
    let mut testadas = 0;
    let mut divergem = Vec::new();
    for k in PrimitiveKind::ALL {
        let Some(base) = representative(k) else {
            continue;
        };
        let Some(limite) = ph2d_field::round_limit(&base) else {
            continue;
        };
        let meio = limite * 0.5;
        let escreve = |p: &Primitive, chave: &str, v: f32| -> Option<Primitive> {
            let mut p = p.clone();
            let i = ph2d_field::dims(&p).iter().position(|d| d.key == chave)?;
            ph2d_field::set_dim(&mut p, 0, i, v).ok()?;
            Some(p)
        };
        let Some(par) = escreve(&base, "field.dim.round", meio)
            .and_then(|p| escreve(&p, "field.dim.chamfer", meio))
        else {
            continue;
        };
        testadas += 1;
        let doc = doc_of(par);
        let sondas = Field::new(&doc);
        let mut produto = ph2d_field_eval::hybrid::Hybrid::new(&doc, &reg);
        // Uma grelha grosseira que cobre a peça e o espaço à volta dela — o defeito que este gate
        // caça é um FACTOR, e um factor aparece em todo ponto onde o campo não é zero.
        let (mut xs, mut ys, mut zs) = (Vec::new(), Vec::new(), Vec::new());
        let at = |t: usize| -1.1 + 2.2 * (t as f32 + 0.5) / 11.0;
        for i in 0..11 {
            for j in 0..11 {
                for l in 0..11 {
                    xs.push(at(i));
                    ys.push(at(j));
                    zs.push(at(l));
                }
            }
        }
        let saida = produto.eval(&xs, &ys, &zs).expect("o produto avalia");
        for n in 0..xs.len() {
            let a = sondas.at(f64::from(xs[n]), f64::from(ys[n]), f64::from(zs[n]));
            let b = f64::from(saida[n]);
            if !a.is_finite() || !b.is_finite() {
                continue;
            }
            let rel = (a - b).abs() / (1.0 + a.abs());
            if rel > pior {
                pior = rel;
            }
            if rel > 1.0e-4 {
                divergem.push(format!("{k:?} rel {rel:.6} ({a:.6} contra {b:.6})"));
                break;
            }
        }
    }
    assert!(
        testadas >= 20,
        "só {testadas} formas com aresta — a lista derivada partiu-se"
    );
    assert!(
        divergem.is_empty(),
        "as duas portas discordam (pior relativo {pior:.3e}): {divergem:?} — uma delas baixa a \
         folha por outra receita, e é a do PRODUTO que o artista vê"
    );
}

/// ⭐⭐⭐ **A CAIXA POR EIXO CONTÉM A PEÇA** — a irmã do [`the_bounding_radius_contains_the_piece`],
/// e ela existe porque três dívidas medidas vinham de a bola não ter lados (Enio, 2026-08-31).
///
/// ⚠️ **A régua é a MESMA** — bissectar a superfície ao longo de muitas direcções —, e o que muda é
/// o que se colhe: de cada superfície encontrada tira-se a **coordenada** em cada eixo, e o máximo
/// delas é o alcance daquele eixo. *Comparar a tabela nova contra o `bounding_radius` seria cego a
/// uma mutação que mexesse nas duas.*
///
/// ⛔⛔ **Prova de mutação (2026-08-31):** trocar a meia-extensão axial da cápsula por `half_height`
/// (isto é, esquecer o `+ radius` que põe a ponta no eixo) reprova aqui — é exactamente o defeito
/// que o gate irmão já tinha apanhado no raio, um nível abaixo.
#[test]
fn the_bounding_half_extents_contain_the_piece() {
    for k in PrimitiveKind::ALL {
        let Some(p) = representative(k) else {
            continue;
        };
        let meias = ph2d_field::bounding_half_extents(&p);
        // ⚠️ **Um cabelo para fora**, pela razão do gate irmão: numa esfera a meia-extensão **é** a
        // superfície, e o campo ali é zero a menos de um ULP.
        let folga = [
            f64::from(meias[0]) * 1.001,
            f64::from(meias[1]) * 1.001,
            f64::from(meias[2]) * 1.001,
        ];
        // ⛔ **E a caixa nunca pode ser MAIOR que a esfera** — se for, uma das duas está errada, e a
        // pergunta *«qual?»* é a que esta linha obriga a fazer.
        let r = f64::from(ph2d_field::bounding_radius(&p)) * 1.001;
        for (e, m) in folga.iter().enumerate() {
            assert!(
                *m <= r,
                "«{}»: a meia-extensão do eixo {e} é {m:.4} e o raio de contenção é {r:.4} — uma \
                 caixa maior do que a esfera que a contém é uma contradição",
                k.key()
            );
        }
        let f = field_of(p);
        const DIRS: usize = 96;
        let (mut alcance, mut viu_peca) = ([0.0f64; 3], false);
        for i in 0..DIRS {
            for j in 0..(DIRS * 2) {
                #[allow(clippy::cast_precision_loss)]
                let theta = std::f64::consts::PI * (i as f64 + 0.5) / DIRS as f64;
                #[allow(clippy::cast_precision_loss)]
                let phi = std::f64::consts::TAU * (j as f64 + 0.5) / (DIRS * 2) as f64;
                let d = [
                    theta.sin() * phi.cos(),
                    theta.sin() * phi.sin(),
                    theta.cos(),
                ];
                let at = |t: f64| f.at(d[0] * t, d[1] * t, d[2] * t);
                // ⭐⭐⭐ **PROCURA A ÚLTIMA ENTRADA, e não bissecta a partir da ORIGEM** (W119).
                //
                // ⛔⛔ **A 1.ª redacção supunha que a origem está DENTRO da peça** — `lo = 0` como
                // extremo «interior» de uma bissecção —, e isso era verdade para as vinte e oito
                // primitivas que existiam. A seta dobrada é a primeira cujo **miolo é vazio**: o
                // canto de dentro do «L» não tem matéria, e ali a bissecção não tem invariante
                // nenhuma. ⇒ ela convergia para uma troca de sinal qualquer e **acusou peça a
                // `0,3459`** num eixo onde uma varredura densa mede `0,3386`. *Uma régua que
                // pressupõe a forma das peças que já existem acusa a primeira que é diferente.*
                //
                // ⭐ A cura não tem pressuposto: amostra-se a semi-recta de fora para dentro e
                // guarda-se o **maior** `t` com matéria; o `t` seguinte está fora por construção, e
                // é entre esses dois que se bissecta. Uma peça oca, um anel, duas ilhas — todas se
                // medem igual.
                const AMOSTRAS: usize = 256;
                let far = r * 4.0;
                let mut dentro: Option<f64> = None;
                for n in (1..=AMOSTRAS).rev() {
                    #[allow(clippy::cast_precision_loss)]
                    let t = far * n as f64 / AMOSTRAS as f64;
                    if at(t) < 0.0 {
                        dentro = Some(t);
                        break;
                    }
                }
                let Some(mut lo) = dentro else { continue };
                viu_peca = true;
                #[allow(clippy::cast_precision_loss)]
                let mut hi = (lo + far / AMOSTRAS as f64).min(far * 1.001);
                for _ in 0..40 {
                    let mid = 0.5 * (lo + hi);
                    if at(mid) < 0.0 { lo = mid } else { hi = mid }
                }
                for e in 0..3 {
                    alcance[e] = alcance[e].max((d[e] * hi).abs());
                }
            }
        }
        assert!(
            viu_peca,
            "«{}»: nenhuma direcção encontrou peça — a varredura não tem sujeito",
            k.key()
        );
        for e in 0..3 {
            assert!(
                alcance[e] <= folga[e],
                "«{}»: há peça a {:.4} no eixo {e} e a meia-extensão diz {:.4} — a caixa por eixo \
                 CORTA a peça, e quem a lê fica com uma cerca pequena demais",
                k.key(),
                alcance[e],
                meias[e]
            );
        }
    }
}

/// ⛔⛔⛔ **A ENGRENAGEM CHATA** — a fixtura que o censo não tinha, e sem a qual a cura da
/// [`ph2d_field::bounding_radius`] não teria quem a defenda.
///
/// O exemplar do censo tem `half_height = 0,15`, e a folga da **altura** dentro do `hyp` escondia o
/// erro do plano: era a única das nove configurações medidas em 2026-08-31 que **não** cortava a
/// peça. ⇒ a fixtura que morde é a **chata**, onde o plano manda sozinho.
///
/// ⛔⛔ **Prova de mutação:** devolver a linha da engrenagem a `hyp(outer, half_height)` reprova
/// aqui em `3` e `5` dentes — a `3`, a peça chega a `0,5050` e o raio diz `0,4504` (`12 %` cortados,
/// e o corte sai como um **arco preto** a atravessá-la).
#[test]
fn a_flat_gear_is_not_cut_by_its_own_bounding_radius() {
    for teeth in [ph2d_field::MIN_GEAR_TEETH, 5, 7, 24] {
        let p = ph2d_field::Primitive::Gear {
            teeth,
            root: 0.32,
            outer: 0.45,
            tooth: 0.45,
            // ⚠️ **Chata de propósito**: com altura, o `hyp` empresta uma folga que não é do plano.
            half_height: 0.02,
            round: 0.0,
            chamfer: 0.0,
        };
        let r = f64::from(ph2d_field::bounding_radius(&p)) * 1.001;
        let f = field_of(p);
        let mut alcance = 0.0f64;
        for i in 0..2_000 {
            #[allow(clippy::cast_precision_loss)]
            let a = std::f64::consts::TAU * (i as f64) / 2_000.0;
            let d = [a.cos(), a.sin()];
            let at = |t: f64| f.at(d[0] * t, d[1] * t, 0.0);
            let (mut lo, mut hi) = (0.0f64, 2.0f64);
            for _ in 0..40 {
                let m = 0.5 * (lo + hi);
                if at(m) < 0.0 { lo = m } else { hi = m }
            }
            alcance = alcance.max(hi);
        }
        assert!(
            alcance <= r,
            "engrenagem de {teeth} dentes e `half_height 0,02`: há peça a {alcance:.5} e o raio de \
             contenção diz {:.5} — a ponta de um dente é uma CORDA, e os cantos dela passam do \
             `outer`",
            r / 1.001
        );
    }
}

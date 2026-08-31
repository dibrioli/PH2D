//! ⭐⭐⭐ **A IMAGEM QUE O PRODUTO DESENHA CONCORDA COM UMA MARCHA HONESTA** — o gate que faltava,
//! e o report do Enio de 2026-08-30 pagou-o (*«piorou os artefatos ao rotacionar»*).
//!
//! # ⛔⛔ Por que os gates que já existiam ficaram TODOS verdes com o produto partido
//!
//! Duas rotas baixam um [`FieldDoc`] a uma árvore: o [`ph2d_field_eval::compile_with`] (as sondas e
//! os gates) e o `hybrid::Builder` (**o produto**). O divisor da aresta tinha sido escrito só na
//! primeira. Medido no mesmo raio, na mesma caixa: o traçado via um campo **`8×`** maior que o dos
//! gates, marchava o passo cheio sobre ele, e atravessava a superfície — enquanto **catorze** gates
//! do censo mediam a rota que a produção não usa e diziam `passo × ‖∇f‖ ≤ 0,80`.
//!
//! ⇒ *Um gate que avalia por outra porta que não a do produto mede um programa que ninguém corre.*
//!
//! # A régua: um ORÁCULO, e não uma segunda opinião
//!
//! A marcha de referência aqui é deliberadamente **burra e lenta** — passo minúsculo, `f64`, sem
//! JIT, sem fita, sem ladrilho, sem fatia, sem anti-serrilhado. Ela não partilha código nenhum com
//! a marcha do produto a não ser o campo e a câmera. Se as duas imagens concordam, tudo o que está
//! entre elas está certo; se divergem, o gate não diz **onde**, e é essa a virtude — ele apanha a
//! família inteira, não o defeito de hoje.
//!
//! ⚠️ **A régua é a NORMAL, e não a máscara.** O defeito que o Enio fotografou não abre buraco: ele
//! pinta facetas escuras no meio da peça, porque o raio aterra fundo dentro e o gradiente ali é
//! outro. Uma régua de silhueta não o vê — a que já existia
//! (`a_shape_with_both_recesses_draws_whole_and_strands_no_ray`) ficou verde a jornada inteira.
//!
//! ⚠️ **A silhueta é EXCLUÍDA de propósito** (`erode`): ali meio pixel de diferença vira a normal ao
//! contrário nas duas marchas, e a peça está correcta. *Uma barra que tem de tolerar o contorno não
//! consegue ser apertada no interior, que é onde o defeito vive.*

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, Xform};
use ph2d_field_eval::{Field, hybrid::Registry};
use ph2d_field_render::{Orbit, Screen, trace_with_threads};

/// Lado da imagem. ⚠️ **Ele paga a marcha `f64` do oráculo**, que é `O(lado² × passos)` no
/// interpretador — medido `1,4 s` em release a `72`, e é por isso que ele não é `420`.
const SIDE: u32 = 72;

/// ⚠️ **A MESMA tolerância de acerto do produto** (`Sharpness::for_frame`, que a este enquadramento
/// dá o tecto `HIT_EPS`). ⛔ Um oráculo mais apertado que o produto mede a TOLERÂNCIA e não a
/// marcha: num vinco, `10×` de diferença de profundidade vira a normal — foram `10` pixels a `28°`
/// numa caixa correcta, e nenhum deles era defeito.
const HIT: f64 = 2.0e-4;

/// ⭐⭐⭐ **O passo do ORÁCULO é minúsculo DE PROPÓSITO** — e isso é o que o torna um oráculo.
///
/// ⛔⛔ A 1.ª versão andava o valor do campo inteiro (`t += v`), que é **a lei do produto**. Com a
/// prova de mutação isso apareceu na hora: com o divisor removido, o oráculo herdava o mesmo campo
/// desonesto, atravessava a superfície pela mesma razão, e as duas imagens **concordavam no
/// errado** — o gate só morria pela cláusula de controle (*«a peça não está a ser desenhada»*).
///
/// ⇒ com `0,1` a marcha de referência continua correcta sobre um campo que exagere a distância até
/// **`10×`**; o pior que este módulo produz é `2×` (o `√4` de uma caixa com os dois recuos, quatro
/// superfícies activas na mesma quina). *Um oráculo que partilha a lei do que ele julga não é um
/// oráculo — é um espelho.*
const ORACLE_STEP: f64 = 0.1;

fn doc_of(p: Primitive) -> FieldDoc {
    FieldDoc::new(
        vec![Node::new(Xform::IDENTITY, NodeKind::Leaf(p))],
        NodeId(0),
    )
    .expect("a peça")
}

/// A mesma peça com uma PILHA de modificadores — ver
/// [`the_deformed_rosette_agrees_with_an_honest_march`].
fn doc_with_mods(p: Primitive, mods: Vec<Unary>) -> FieldDoc {
    let mut n = Node::new(Xform::IDENTITY, NodeKind::Leaf(p));
    n.mods = mods;
    FieldDoc::new(vec![n], NodeId(0)).expect("a peça")
}

/// A marcha do ORÁCULO: `f64`, passo do campo, e a normal por diferença central. `None` = fundo.
fn honest(f: &Field, o: [f64; 3], d: [f64; 3]) -> Option<[f64; 3]> {
    let mut t = 0.0f64;
    for _ in 0..4000 {
        let q = [o[0] + d[0] * t, o[1] + d[1] * t, o[2] + d[2] * t];
        let v = f.at(q[0], q[1], q[2]);
        if v < HIT {
            let e = 1.0e-4;
            let g = [
                f.at(q[0] + e, q[1], q[2]) - f.at(q[0] - e, q[1], q[2]),
                f.at(q[0], q[1] + e, q[2]) - f.at(q[0], q[1] - e, q[2]),
                f.at(q[0], q[1], q[2] + e) - f.at(q[0], q[1], q[2] - e),
            ];
            let n = (g[0] * g[0] + g[1] * g[1] + g[2] * g[2]).sqrt();
            if n <= 0.0 {
                return None;
            }
            return Some([g[0] / n, g[1] / n, g[2] / n]);
        }
        t += v * ORACLE_STEP;
        if t > 8.0 {
            return None;
        }
    }
    None
}

/// Quantos pixels do INTERIOR o produto pinta com uma normal a mais de `limite` graus do oráculo.
fn disagreeing_pixels(p: &Primitive, yaws: &[f32], limite_graus: f64) -> (usize, usize, f64) {
    disagreeing_pixels_of(&doc_of(p.clone()), yaws, limite_graus)
}

fn disagreeing_pixels_of(doc: &FieldDoc, yaws: &[f32], limite_graus: f64) -> (usize, usize, f64) {
    let doc = doc.clone();
    let f = Field::new(&doc);
    let reg = Registry::new();
    let screen = Screen::new(SIDE, SIDE, 0.85);
    let (mut mal, mut medidos, mut pior) = (0usize, 0usize, 0.0f64);
    for &a in yaws {
        let (sy, cy) = (a * 0.5).sin_cos();
        let cam = Orbit {
            half_extent: 0.85,
            rotation: [0.0, sy, 0.0, cy],
            ..Orbit::default()
        };
        // ⚠️ **Sem anti-serrilhado**: a 2.ª passagem re-marcha a silhueta com outra amostragem, e o
        // que este gate mede é o INTERIOR. Ligá-lo acrescentaria ruído exactamente onde a régua já
        // não olha.
        let g = trace_with_threads(&doc, &reg, &cam, SIDE, SIDE, true);
        let dentro = |x: u32, y: u32| g.hit[(y * SIDE + x) as usize];
        let (bx, by, bz) = cam.basis();
        for y in 1..SIDE - 1 {
            for x in 1..SIDE - 1 {
                // ⭐ Só o INTERIOR — ver a nota do módulo.
                if !(dentro(x, y)
                    && dentro(x - 1, y)
                    && dentro(x + 1, y)
                    && dentro(x, y - 1)
                    && dentro(x, y + 1))
                {
                    continue;
                }
                let (sx, sy2) = screen.plane_at(x as f32 + 0.5, y as f32 + 0.5);
                let (o, d) = cam.ray_at_plane(sx, sy2);
                let of = [f64::from(o[0]), f64::from(o[1]), f64::from(o[2])];
                let df = [f64::from(d[0]), f64::from(d[1]), f64::from(d[2])];
                let Some(certo) = honest(&f, of, df) else {
                    continue;
                };
                // O produto entrega a normal em espaço de VISTA; o oráculo em mundo.
                let dot = |b: [f32; 3]| {
                    certo[0] * f64::from(b[0])
                        + certo[1] * f64::from(b[1])
                        + certo[2] * f64::from(b[2])
                };
                let alvo = [dot(bx), dot(by), dot(bz)];
                let n = g.normal[(y * SIDE + x) as usize];
                let c = (alvo[0] * f64::from(n[0])
                    + alvo[1] * f64::from(n[1])
                    + alvo[2] * f64::from(n[2]))
                .clamp(-1.0, 1.0);
                let graus = c.acos().to_degrees();
                medidos += 1;
                pior = pior.max(graus);
                if graus > limite_graus {
                    mal += 1;
                }
            }
        }
    }
    (mal, medidos, pior)
}

/// ⭐⭐⭐ **Quantos pixels o ORÁCULO acerta e o produto deixa VAZIOS** — a outra pergunta, e a que
/// apanhou o report de 2026-08-31.
///
/// ⚠️ **`disagreeing_pixels_of` é cega a isto de propósito:** ela compara NORMAIS, e para isso só
/// olha pixels que **os dois** acertaram. Uma peça que o produto simplesmente **não desenha** não
/// tem normal para comparar — ela some da população em vez de reprovar.
///
/// ⛔ O oráculo não tem caixa de recorte, e o produto tem. Quando um deformador estende a peça para
/// além do que o bordo dela previu, é aqui que se vê: `604` pixels num corte **rectilíneo**, que é
/// a fronteira do recorte a cortar uma cauda que ninguém sabia que existia.
fn pixels_the_product_misses(doc: &FieldDoc, yaws: &[f32]) -> (usize, usize) {
    let doc = doc.clone();
    let f = Field::new(&doc);
    let reg = Registry::new();
    let screen = Screen::new(SIDE, SIDE, 0.85);
    let (mut faltam, mut oraculo) = (0usize, 0usize);
    for &a in yaws {
        let (sy, cy) = (a * 0.5).sin_cos();
        let cam = Orbit {
            half_extent: 0.85,
            rotation: [0.0, sy, 0.0, cy],
            ..Orbit::default()
        };
        let g = trace_with_threads(&doc, &reg, &cam, SIDE, SIDE, true);
        for y in 0..SIDE {
            for x in 0..SIDE {
                let (sx, sy2) = screen.plane_at(x as f32 + 0.5, y as f32 + 0.5);
                let (o, d) = cam.ray_at_plane(sx, sy2);
                let of = [f64::from(o[0]), f64::from(o[1]), f64::from(o[2])];
                let df = [f64::from(d[0]), f64::from(d[1]), f64::from(d[2])];
                if honest(&f, of, df).is_none() {
                    continue;
                }
                oraculo += 1;
                if !g.hit[(y * SIDE + x) as usize] {
                    faltam += 1;
                }
            }
        }
    }
    (faltam, oraculo)
}

/// As vistas: **a girar**, porque um campo desonesto só morde onde o raio raspa a superfície.
fn yaws() -> Vec<f32> {
    (0..4).map(|i| 0.37 + 0.42 * i as f32).collect()
}

/// ⭐⭐⭐ **A CAIXA com os dois recuos** — a forma exacta do report.
///
/// ⛔⛔ **Prova de mutação (2026-08-30):** devolver o `primitive()` do
/// `ph2d_field_eval::primitive_tree` à fórmula crua (isto é, tirar o divisor da porta única) leva
/// este gate de **`0`** pixels em desacordo para **`1 186` de `2 308`** — `51,4 %` do interior —,
/// com o pior desvio em **`77,0°`**. O irmão do prisma vai a `1 810` de `6 217` e `83,3°`. *Era
/// isso que o Enio fotografou.*
#[test]
fn the_traced_box_agrees_with_an_honest_march() {
    let p = Primitive::Box {
        half: [0.42, 0.30, 0.26],
        round: 0.12,
        chamfer: 0.12,
    };
    let (mal, medidos, pior) = disagreeing_pixels(&p, &yaws(), 12.0);
    assert!(
        medidos > 2_000,
        "⛔ o CONTROLE falhou: só {medidos} pixels de interior — a peça não está a ser desenhada"
    );
    assert_eq!(
        mal, 0,
        "{mal} de {medidos} pixels do INTERIOR têm a normal a mais de 12° do oráculo (pior \
         {pior:.1}°) — o traçado está a ler um campo diferente do que os gates medem, ou a marcha \
         está a atravessar a superfície"
    );
}

/// O irmão numa forma de parede **não-ortogonal**, que arredonda por outra receita.
#[test]
fn the_traced_prism_agrees_with_an_honest_march() {
    let p = Primitive::Prism {
        sides: 6,
        bottom: 0.5,
        top: 0.5,
        half_height: 0.55,
        round: 0.10,
        chamfer: 0.10,
    };
    let (mal, medidos, pior) = disagreeing_pixels(&p, &yaws(), 12.0);
    assert!(
        medidos > 2_000,
        "⛔ o CONTROLE falhou: só {medidos} pixels de interior"
    );
    assert_eq!(
        mal, 0,
        "{mal} de {medidos} pixels do INTERIOR divergem do oráculo (pior {pior:.1}°)"
    );
}

/// ⭐⭐ **A ROSETA DEFORMADA** — um `Taper` antes de uma repetição radial, que é a pilha que rasgava.
///
/// ⛔ Com a janela de fatias a `1` o campo lia `‖∇f‖ = 730,5` e a peça saía **estilhaçada**: lascas
/// soltas a flutuar e buracos. ⚠️ **Nenhuma régua de silhueta a apanharia** — uma roseta é côncava,
/// então «fundo rodeado de peça» é o aspecto NORMAL dela. O que a apanha é a imagem contra a marcha
/// honesta.
#[test]
fn the_deformed_rosette_agrees_with_an_honest_march() {
    let doc = doc_with_mods(
        Primitive::Box {
            half: [0.35, 0.35, 0.30],
            round: 0.0,
            chamfer: 0.0,
        },
        vec![
            Unary::Taper { slope: 0.6 },
            Unary::Radial {
                count: 6,
                joint: ph2d_field::Joint::SHARP,
            },
        ],
    );
    let (mal, medidos, pior) = disagreeing_pixels_of(&doc, &yaws(), 12.0);
    assert!(
        medidos > 1_000,
        "⛔ o CONTROLE falhou: só {medidos} pixels de interior — a roseta não está a ser desenhada"
    );
    // ⚠️ **Aqui a barra NÃO é zero, e a razão é a forma:** uma roseta de junta VIVA tem vincos
    // côncavos no interior, e sobre um vinco meio pixel de profundidade vira a normal nas duas
    // marchas — é a mesma razão pela qual a silhueta é excluída, um nível para dentro.
    //
    // Medido: curada **`6` de `6 844`** (`0,09 %`, pior `20,5°`); com a janela de fatias a `1`,
    // **`34` de `6 140`** (`0,55 %`, pior `94,6°`). A barra de `0,2 %` fica entre as duas.
    assert!(
        mal * 500 <= medidos,
        "{mal} de {medidos} pixels do INTERIOR divergem do oráculo (pior {pior:.1}°) — a pilha de \
         modificadores está a devolver um campo que a marcha lê de outra maneira"
    );
}

/// ⭐⭐⭐ **A DOBRA DESENHA O QUE A MARCHA HONESTA DESENHA** — e este gate existe porque uma medição
/// sem consumidor me fez estragar uma feature que funcionava.
///
/// # ⛔⛔ O que aconteceu (2026-08-30)
///
/// O `‖∇f‖` da dobra sozinha media `1,72` dentro da caixa de recorte — acima de `1`, o número que
/// diz *«a marcha pode atravessar a superfície»*. Curei-o apertando a parede da curvatura contra o
/// envelope da pilha, e a peça **deixou de dobrar**: num bloco, `0,3`, `0,6` e `1,0` voltas passaram
/// a dar a mesma coisa. Report do Enio: *«VC danificou o Bend que funcionava antes das últimas
/// mudanças»* — e ele tinha razão.
///
/// ⭐ **A régua que faltava é esta.** Com a lei que ele tinha, a imagem concorda com o oráculo
/// **exactamente**: `0` de `1 678` / `1 672` / `4 274` / `4 274` pixels fora de `12°`, com o pior
/// desvio em `0,0°`–`5,1°`. ⇒ *o `1,72` é real e **não tem consumidor**: onde os raios de facto
/// passam, o campo continua a ser um minorante.*
///
/// ⚠️ **A lição não é «o gradiente não importa»** — é que um gate de gradiente diz *«pode furar»*, e
/// só a imagem diz *«fura»*. Quando os dois discordam, quem manda é a imagem, e a dívida do outro
/// fica escrita ([`ph2d_field_eval`], `every_modifier_alone_keeps_the_field_marchable`).
#[test]
fn the_bend_draws_what_an_honest_march_draws() {
    for (nome, half, turns) in [
        ("barra", [0.10f32, 0.10, 0.80], 0.12f32),
        ("barra forte", [0.10, 0.10, 0.80], 1.0),
        ("bloco", [0.35, 0.35, 0.30], 0.3),
        ("bloco forte", [0.35, 0.35, 0.30], 1.0),
    ] {
        let doc = doc_with_mods(
            Primitive::Box {
                half,
                round: 0.0,
                chamfer: 0.0,
            },
            vec![Unary::Bend {
                turns,
                lower: -2.0,
                upper: 2.0,
                falloff: 0.1,
            }],
        );
        let (mal, medidos, pior) = disagreeing_pixels_of(&doc, &yaws(), 12.0);
        assert!(
            medidos > 1_000,
            "⛔ o CONTROLE falhou em «{nome}»: só {medidos} pixels de interior"
        );
        assert_eq!(
            mal, 0,
            "«{nome}» com {turns} voltas: {mal} de {medidos} pixels divergem do oráculo (pior \
             {pior:.1}°) — a dobra passou a desenhar o que a marcha honesta não desenha"
        );
    }
}

/// ⭐⭐⭐ **A BANDA DA DOBRA, VARRIDA — e o produto desenha o que o oráculo desenha.**
///
/// # ⛔⛔ O parâmetro que nenhuma medição percorria
///
/// Todo gate deste repo — e o próprio nascimento do modificador — dá à banda uma faixa que **cobre
/// a peça inteira** (`[−2, 2]`, `[−9, 9]`). As duas linhas que o Enio arrastou na foto de
/// 2026-08-31 (`From` e `To`) são exactamente esse parâmetro, e fora do ponto testado o mapa
/// **congelava**: com a banda escrita no `z` de entrada, `x` e `z` deixam de depender do eixo e o
/// campo fica constante — a peça ganha uma cauda semi-infinita que a caixa de recorte corta por um
/// plano.
///
/// ⛔⛔ **Prova de mutação (2026-08-31):** devolver a `ph2d_field_eval::stack_bend::bend` à lei
/// anterior leva este gate de **`0`** pixels em falta para **`4 084` de `31 538`** (`13,0 %`), com
/// `4` das `10` configurações a reprovar — até `1 868` pixels numa só. ⚠️ O irmão
/// `a_bent_piece_never_starves_the_march` **SOBREVIVE** à mesma mutação: *a cauda não mata a marcha
/// à fome (ela cabe no orçamento), ela é cortada pela caixa de recorte.*
///
/// ⚠️ **As acusadoras são as bandas ESTREITAS** (`[−0,02, 0,02]` e a degenerada), e não a da foto —
/// e a razão é geométrica: quanto mais estreita a banda, mais matéria fica na secção congelada, e
/// mais gorda é a cauda. A banda larga e assimétrica da foto (`[−0,187, 0,048]`) congela uma lasca,
/// que a `72²` desta grelha dá menos de `1 %`. *Ela fica na lista na mesma — é a reprodução.*
///
/// ⚠️ **A barra não é zero**: meio pixel de silhueta cai de lados diferentes nas duas marchas, e
/// isso é a peça a estar certa. Medido curado: `0`–`3` por configuração.
#[test]
fn the_band_of_the_bend_draws_what_an_honest_march_draws() {
    let vistas = [0.37f32, 1.21];
    let mut maus = Vec::new();
    let (mut soma_falta, mut soma_oraculo) = (0usize, 0usize);
    for (lo, up, fall) in [
        // A banda EXACTA da foto, e as vizinhas.
        (-0.187f32, 0.048f32, 0.072f32),
        (-0.02, 0.02, 0.01),
        // Inteiramente FORA da matéria — o pior caso do congelamento.
        (0.20, 0.40, 0.05),
        // Degenerada (largura zero) e a que COBRE a peça.
        (0.0, 0.0, 0.0),
        (-2.0, 2.0, 0.1),
    ] {
        for turns in [0.05f32, ph2d_field::mods::MAX_BEND_TURNS] {
            let doc = doc_with_mods(
                // ⚠️ **A chapa alta e fina do report**: a dobra age no `Z`, que aqui é a dimensão
                // curta, então toda banda que o artista arraste cai fora da matéria.
                Primitive::Box {
                    half: [0.207, 0.5025, 0.036],
                    round: 0.02,
                    chamfer: 0.0,
                },
                vec![Unary::Bend {
                    turns,
                    lower: lo,
                    upper: up,
                    falloff: fall,
                }],
            );
            let (faltam, oraculo) = pixels_the_product_misses(&doc, &vistas);
            soma_falta += faltam;
            soma_oraculo += oraculo;
            if faltam * 100 > oraculo {
                maus.push(format!(
                    "{turns} voltas, banda [{lo}, {up}] fall {fall}: {faltam} de {oraculo} pixels \
                     que o oráculo acerta saem VAZIOS"
                ));
            }
        }
    }
    // ⛔ **O CONTROLE**: sem peça na tela o gate acima passaria por não haver pixels.
    assert!(
        soma_oraculo > 8_000,
        "só {soma_oraculo} pixels de peça em 10 configurações — a chapa não está a ser desenhada"
    );
    assert!(
        maus.is_empty(),
        "{} configuração(ões) de banda desenham menos peça que a marcha honesta (total {soma_falta} \
         de {soma_oraculo}) — a peça está a ser cortada pela caixa de recorte: {}",
        maus.len(),
        maus.join(" · ")
    );
}

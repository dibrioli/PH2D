//! ⚠️ **O ORÇAMENTO DE PASSOS NÃO VÊ UM ENCOLHIMENTO LOCAL** — a pergunta que a wave da torção
//! deixou nomeada e por medir.
//!
//! O divisor da torção vive no **operador** (e é a escolha certa: no `gradient_bound` ele
//! penalizaria a cena inteira). A consequência é que `safe_march_step(doc)` fica em `1,0` — o
//! documento não infla — enquanto a região torcida devolve `1/σ` da distância e pede `~σ×` mais
//! passos. O orçamento é derivado do **passo**, logo fica no mínimo.
//!
//! ⛔ **Um raio que esgota o orçamento é largado em SILÊNCIO** (`march::EXHAUSTED`), e o pixel dele
//! lê-se como fundo — que é exactamente o defeito de 2026-08-30 a entrar por outra porta.

use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Unary, Xform};
use ph2d_field_eval::{hybrid::Registry, safe_march_step};
use ph2d_field_render::{EXHAUSTED, Gbuffer, Orbit, trace_stepped_for_test};
use std::sync::atomic::Ordering;

fn barra(turns: f32) -> FieldDoc {
    let mut n = Node::new(
        Xform::IDENTITY,
        NodeKind::Leaf(Primitive::Box {
            half: [0.34, 0.11, 0.62],
            round: 0.02,
            chamfer: 0.0,
        }),
    );
    if turns != 0.0 {
        n.mods.push(Unary::Twist {
            turns,
            lower: -9.0,
            upper: 9.0,
            falloff: 0.0,
        });
    }
    FieldDoc::new(vec![n], NodeId(0)).expect("peça")
}

/// Um pixel de fundo com peça dos DOIS lados na linha **e** na coluna.
///
/// ⛔⛔ **ELE NÃO MEDE A MARCHA, e eu li-o como se medisse.** Uma fita torcida tem silhueta **não
/// convexa**: de três-quartos vê-se o fundo entre as voltas, cercado de peça na linha e na coluna.
/// Ele contou `335` numa barra a uma volta por unidade — e o passo a **um quarto** devolve a máscara
/// **pixel a pixel idêntica** (`0` mudados), o que prova que ali não há travessia nenhuma.
///
/// *É a segunda vez que este detector me engana* — a primeira foi com uma fixtura em espiral, e a
/// nota ficou escrita e não foi lida. Ele fica porque é útil numa peça **convexa**; a régua da
/// marcha é a invariância ao passo, logo abaixo.
fn furos(g: &Gbuffer) -> usize {
    let (w, h) = (g.width as usize, g.height as usize);
    let mut n = 0;
    for y in 0..h {
        for x in 0..w {
            if g.hit[y * w + x] {
                continue;
            }
            let linha = (0..x).any(|i| g.hit[y * w + i]) && (x + 1..w).any(|i| g.hit[y * w + i]);
            let coluna = (0..y).any(|j| g.hit[j * w + x]) && (y + 1..h).any(|j| g.hit[j * w + x]);
            if linha && coluna {
                n += 1;
            }
        }
    }
    n
}

#[test]
fn measure_whether_a_twisted_piece_starves_the_march() {
    let reg = Registry::default();
    let cam = Orbit {
        half_extent: 1.0,
        ..Orbit::default()
    };
    println!("\n voltas | passo do doc | esgotados | acertos | furos");
    for turns in [0.0f32, 0.25, 0.5, 1.0, 2.0] {
        let doc = barra(turns);
        let passo = safe_march_step(&doc);
        EXHAUSTED.store(0, Ordering::Relaxed);
        let g = trace_stepped_for_test(&doc, &reg, &cam, 320, 320, passo);
        println!(
            "{turns:7.2} | {passo:12.4} | {:9} | {:7} | {:5}",
            EXHAUSTED.load(Ordering::Relaxed),
            g.hits(),
            furos(&g)
        );
    }
}

/// ⭐ **Os «furos» são a peça, ou a marcha?** — a única pergunta que o contador de furos não responde.
///
/// ⚠️ Uma fita torcida tem silhueta **não convexa**: de três-quartos vê-se o fundo entre as voltas,
/// cercado de peça na linha e na coluna. O detector de furos conta isso, e conta certo — *ele mede a
/// silhueta, não a marcha*.
///
/// A régua que separa as duas: render a MESMA peça com um passo **quatro vezes menor**. Se a máscara
/// não se mexer, não há travessia nenhuma e o que se vê é a forma.
#[test]
fn measure_whether_a_smaller_step_finds_more_piece() {
    let reg = Registry::default();
    let cam = Orbit {
        half_extent: 1.0,
        ..Orbit::default()
    };
    println!("\n voltas | acertos passo cheio | passo/4 | pixels que MUDARAM");
    for turns in [0.5f32, 1.0, 2.0] {
        let doc = barra(turns);
        let passo = safe_march_step(&doc);
        let a = trace_stepped_for_test(&doc, &reg, &cam, 320, 320, passo);
        let b = trace_stepped_for_test(&doc, &reg, &cam, 320, 320, passo * 0.25);
        let mudou = a.hit.iter().zip(&b.hit).filter(|(x, y)| x != y).count();
        println!(
            "{turns:7.2} | {:18} | {:7} | {mudou:18}",
            a.hits(),
            b.hits()
        );
    }
}

/// ⭐⭐⭐ **UM DEFORMADOR NÃO PODE MATAR RAIOS À FOME** — o gate do orçamento.
///
/// O divisor da torção vive no **operador**, então `safe_march_step` fica em `1,0` e a região torcida
/// pede `σ×` mais passos. Sem o `field_shrink` no orçamento, medido a uma volta por unidade:
/// **18 raios esgotados**. *E um raio que acaba os passos é largado em silêncio.*
#[test]
fn a_twisted_piece_never_starves_the_march() {
    let reg = Registry::default();
    let cam = Orbit {
        half_extent: 1.0,
        ..Orbit::default()
    };
    for turns in [0.25f32, 0.5, 1.0, ph2d_field::mods::MAX_TWIST_TURNS] {
        let doc = barra(turns);
        EXHAUSTED.store(0, Ordering::Relaxed);
        let g = trace_stepped_for_test(&doc, &reg, &cam, 320, 320, safe_march_step(&doc));
        let mortos = EXHAUSTED.load(Ordering::Relaxed);
        assert_eq!(
            mortos, 0,
            "{turns} voltas: {mortos} raios acabaram o orçamento — o pixel deles lê-se como FUNDO, e \
             ninguém o diz"
        );
        // ⛔ **O CONTROLE**: sem peça na tela o gate acima passaria por não haver raios.
        assert!(
            g.hits() > 5_000,
            "{turns} voltas: a peça mal aparece ({} pixels) — o gate acima não está a medir nada",
            g.hits()
        );
    }
}

/// ⭐⭐⭐ **A RÉGUA DA MARCHA É A INVARIÂNCIA AO PASSO** — e ela não precisa de oráculo nenhum.
///
/// Um passo seguro nunca atravessa a superfície, logo encurtá-lo **não pode achar mais peça**. Se a
/// máscara mudar ao dividir o passo por quatro, o passo cheio estava a saltar por cima de alguma
/// coisa — e é isso, e não a contagem de furos, que diz se a marcha está certa.
///
/// ⚠️ **É a régua que a silhueta não convexa não confunde**: ela compara a peça consigo própria.
#[test]
fn a_shorter_step_finds_exactly_the_same_piece() {
    let reg = Registry::default();
    let cam = Orbit {
        half_extent: 1.0,
        ..Orbit::default()
    };
    for turns in [0.0f32, 0.5, 1.0, ph2d_field::mods::MAX_TWIST_TURNS] {
        let doc = barra(turns);
        let passo = safe_march_step(&doc);
        let cheio = trace_stepped_for_test(&doc, &reg, &cam, 320, 320, passo);
        let curto = trace_stepped_for_test(&doc, &reg, &cam, 320, 320, passo * 0.25);
        let mudou = cheio
            .hit
            .iter()
            .zip(&curto.hit)
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(
            mudou, 0,
            "{turns} voltas: {mudou} pixels mudam quando o passo é dividido por quatro — o passo \
             cheio está a atravessar a superfície"
        );
        assert!(cheio.hits() > 5_000, "{turns} voltas: a peça mal aparece");
    }
}

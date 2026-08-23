//! ⛔⛔⛔ **OS GATES DAS RÉGUAS QUE MEDIAM NADA** (2026-08-23).
//!
//! # O que aconteceu
//!
//! Duas réguas desta crate — o [`ph2d_quadfill::FillReport::domain_skew`] e o
//! [`ph2d_quadfill::skew_by_fan`] — foram escritas para separar o enviesamento por
//! **valência do patch**, e as duas mediam a população errada:
//!
//! | régua | o que devia medir | o que media |
//! |---|---|---|
//! | `domain_skew.0` | as células de domínio dos patches `n = 4` | ⛔ **nada** — o balde ficava vazio |
//! | `skew_by_fan.0` | as faces dos patches `n = 4` | ⛔ só a cauda depois do último leque |
//! | `skew_by_fan.1` | as faces dos patches `n ≠ 4` | ⛔ leques **e** rectângulos |
//!
//! **A causa é uma só:** a escrituração vivia no fim do laço dos patches e o caminho
//! do rectângulo saía por `continue` **antes dela**. ⇒ o balde do rectângulo nunca era
//! preenchido, e o vector de etiquetas nunca era estendido — então o primeiro leque a
//! seguir rotulava de *leque* as faces de rectângulo que o precediam.
//!
//! # ⚠️ Por que isto custou um dia
//!
//! `domain_skew.0` imprimia **`0,0°`** — a mediana de um vector vazio. Isso leu-se
//! como *«a grade do rectângulo nasce PERFEITA no domínio e chega torta à
//! superfície»*, e sobre essa leitura construiu-se uma conclusão inteira: *o defeito
//! parte-se em dois, um em cada fase*. ⛔ **Um zero de «não medido» e um zero de
//! «perfeito» são o mesmo byte** — e a régua irmã ([`ph2d_quadfill::skew_by_fan`])
//! até trazia esse aviso no doc, para a coluna dela.
//!
//! # O que estes gates prendem
//!
//! ⭐ Não a linha que estava errada — **a classe**. Um balde que ninguém enche e uma
//! etiquetagem que não cobre a malha passam a ser **contagens** e não medianas: um
//! inteiro a zero não se disfarça de resultado bom.

use ph2d_crossfield::{Dual, solve_miq};
use ph2d_mesh::{Mesh, shapes};
use ph2d_quadfill::{SMOOTHING_ROUNDS, fill, skew_by_fan};

fn fixture() -> Mesh {
    let mut m = shapes::uv_sphere(24, 36, 1.0);
    m.triangulate();
    m
}

fn run() -> (Mesh, ph2d_quadfill::FillReport, Vec<usize>) {
    let reference = fixture();
    let mut work = reference.clone();
    ph2d_remesh_iso::remesh_isotropic(&mut work, ph2d_remesh_iso::ALPHA);
    work.triangulate();
    let dual = Dual::build(&work);
    let (field, _) = solve_miq(&dual);
    let layout = ph2d_trace::trace_patches(&work, &dual, &field);
    let spec = layout.to_layout(0.25).expect("o layout fecha");
    let (quant, _) = ph2d_quantize::quantize_within(&spec, ph2d_quantize::Budget::new(256, 512))
        .expect("a quantizacao fecha");
    let valences: Vec<usize> = layout.side_arcs.iter().map(Vec::len).collect();
    let (mesh, report) =
        fill(&work, &reference, &layout, &quant, SMOOTHING_ROUNDS).expect("a montagem fecha");
    (mesh, report, valences)
}

/// ⭐⭐⭐ **AS DUAS COLUNAS DO DOMÍNIO MEDIRAM ALGUMA COISA.**
///
/// ⚠️ **A fixtura contém o fenómeno, e isso é asserido primeiro** — uma esfera cujos
/// patches fossem todos leques deixaria este gate verde por vacuidade, que é a mesma
/// doença que ele existe para apanhar (`reference_topic_fixture_discipline`).
#[test]
fn both_valences_actually_get_measured_in_the_domain() {
    let (_, r, valences) = run();
    let quads = valences.iter().filter(|&&n| n == 4).count();
    let fans = valences.len() - quads;
    assert!(
        quads > 0 && fans > 0,
        "a fixtura deixou de conter as DUAS valencias ({quads} rectangulos, {fans} leques) -- \
         este gate ficaria verde sem medir nada"
    );
    assert_eq!(
        r.quad_patches, quads,
        "o relatorio conta patches de quatro lados diferente do layout"
    );
    assert!(
        r.domain_cells.0 > 0,
        "⛔ o balde do DOMINIO dos rectangulos ficou VAZIO com {quads} patches de quatro \
         lados na peca -- e a mediana de um vector vazio e' 0,0, que se le como «perfeito». \
         Alguem voltou a por um `continue` no laco dos patches do `stitch`?"
    );
    assert!(
        r.domain_cells.1 > 0,
        "⛔ o balde do DOMINIO dos leques ficou VAZIO com {fans} leques na peca"
    );
}

/// ⭐⭐ **A ETIQUETAGEM POR VALÊNCIA COBRE A MALHA INTEIRA.**
///
/// ⛔ Quando ela não cobria, as faces sem etiqueta caíam **na coluna do rectângulo** e
/// as duas medianas descreviam populações que ninguém escolheu. Hoje uma etiquetagem
/// curta devolve `NaN`, e é isso que este gate lê.
#[test]
fn the_valence_labelling_covers_every_face() {
    let (_, r, _) = run();
    assert!(
        r.skew_by_fan.0.is_finite() && r.skew_by_fan.1.is_finite(),
        "⛔ `skew_by_fan` devolveu NaN: a etiquetagem por valencia nao cobre a malha, \
         e as duas colunas estao a medir a populacao errada ({:?})",
        r.skew_by_fan
    );
    assert!(
        r.skew_by_fan.0 > 0.0 && r.skew_by_fan.1 > 0.0,
        "uma das colunas de valencia veio a zero -- ou a peca e' perfeita, ou o balde \
         esta vazio, e as duas leem igual ({:?})",
        r.skew_by_fan
    );
}

/// ⭐ **A RECUSA É DIRECTA, e não depende da cadeia inteira.**
///
/// ⚠️ Ela existe porque os dois gates acima só apanham a regressão **através** do
/// `fill`; quem chamar a régua de fora — uma sonda, um bench — merece a mesma recusa
/// alta em vez de uma média sobre a população errada.
#[test]
fn a_short_labelling_is_refused_out_loud() {
    let (mesh, ..) = run();
    let faces = mesh.faces().len();
    assert!(faces > 2, "a fixtura tem de ter faces");
    let short = vec![false; faces - 1];
    let (a, b) = skew_by_fan(&mesh, &short);
    assert!(
        a.is_nan() && b.is_nan(),
        "uma etiquetagem CURTA devolveu numeros em vez de NaN -- e uma media sobre a \
         populacao errada nao tem como ser notada"
    );
    let exact = vec![false; faces];
    let (a, _) = skew_by_fan(&mesh, &exact);
    assert!(
        a.is_finite(),
        "a etiquetagem EXACTA foi recusada -- o controlo positivo da recusa falhou"
    );
}

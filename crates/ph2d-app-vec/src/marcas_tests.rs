//! Gates do [`super`] (`marcas.rs`) — o losango de cada campo do painel do vetor.

use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};

const E: u64 = 7;
const R: u16 = v::TINTA_PREENCH + v::OFF_SOLIDA;
const SOLO: Instante = Instante {
    t_cut: 0.0,
    solo: true,
};

fn f(c: u16) -> PropKind {
    PropKind::Field(FieldKey::new(VETOR, c, 0, 0))
}

/// O preenchimento gravado inteiro (r, g, b, a) em 0 s e 1 s, a 24 fps — o que o autokey deixa.
fn doc() -> TimelineDoc {
    let mut d = TimelineDoc::new();
    d.fps_display = 24.0;
    for t in [0.0, 1.0] {
        for c in R..R + 4 {
            d.insert_key(
                E,
                f(c),
                RationalTime::from_seconds(t),
                AnimValue::Float(100.0),
                Interp::Linear,
            );
        }
    }
    d
}

fn na_curva(_: u64, _: PropKind) -> Option<f32> {
    Some(100.0)
}

fn em(t: f64) -> Instante {
    Instante { t_cut: t, ..SOLO }
}

/// A cor do preenchimento e a opacidade dele mostram o grupo: cheio numa chave, vazio entre elas;
/// um campo sem curva (a largura) mostra o ponto clicável; sem objecto, nenhum campo tem canal.
#[test]
fn o_campo_mostra_o_estado_do_grupo_dele() {
    let d = doc();
    let m = marcas(&d, &[E], em(0.0), na_curva);
    assert_eq!(m[&ferramenta::VECTOR_FILL_SWATCH], KeyMark::Chave);
    assert_eq!(m[&ferramenta::VECTOR_FILL_OPACITY], KeyMark::Chave);
    assert_eq!(m[&ferramenta::VECTOR_WIDTH], KeyMark::Livre);
    let m = marcas(&d, &[E], em(0.5), na_curva);
    assert_eq!(m[&ferramenta::VECTOR_FILL_SWATCH], KeyMark::Animado);
    assert!(marcas(&d, &[], em(0.0), na_curva).is_empty());
}

/// Mexer SÓ o verde a meio já põe o losango da cor em «mudado» — o grupo inteiro responde.
/// (Mutação: ler só o representante do grupo ⇒ o verde passa despercebido.)
#[test]
fn um_membro_mexido_muda_o_losango_do_grupo() {
    let d = doc();
    let verde_mexido = |_, p| Some(if p == f(R + 1) { 7.0 } else { 100.0 });
    let m = marcas(&d, &[E], em(0.5), verde_mexido);
    assert_eq!(m[&ferramenta::VECTOR_FILL_SWATCH], KeyMark::Mudado);
}

/// Numa chave, o clique apaga as 4 chaves da cor neste quadro — e só essas — num passo de undo.
#[test]
fn clicar_numa_chave_apaga_o_grupo_neste_quadro() {
    let d = doc();
    let i = clique(
        &d,
        &[ferramenta::VECTOR_FILL_SWATCH],
        &[E],
        em(0.0),
        |_| Vec::new(),
        na_curva,
        |_, _| None,
    );
    let n = i
        .iter()
        .filter(|x| matches!(x, TimelineIntent::AddToSelection(_)))
        .count();
    assert_eq!(n, 4);
    assert_eq!(i.first(), Some(&TimelineIntent::BeginEdit));
    assert!(i.contains(&TimelineIntent::DeleteSelection));
    assert_eq!(i.last(), Some(&TimelineIntent::EndEdit));
}

/// Fora de uma chave, o clique grava o GRUPO inteiro que a família publicou, pela porta da K —
/// a largura grava só a largura; o tracejado grava o «há tracejado» e os pares.
#[test]
fn clicar_fora_de_uma_chave_grava_o_grupo_inteiro() {
    let d = TimelineDoc::new();
    let publicados = |_| {
        vec![
            FieldKey::new(VETOR, v::TRACO_LARGURA, 0, 0),
            FieldKey::new(VETOR, v::TRACO_TRACEJADO, 0, 0),
            FieldKey::new(VETOR, v::TRACO_TRACO, 0, 0),
            FieldKey::new(VETOR, v::TRACO_VAO, 0, 0),
            FieldKey::new(VETOR, R, 0, 0),
        ]
    };
    let k = |e, p| {
        Some(TimelineIntent::AddKey {
            entity: e,
            prop: p,
            t: RationalTime::from_seconds(0.5),
            value: AnimValue::Float(1.0),
            interp: Interp::Linear,
        })
    };
    let gravados = |campo| {
        clique(&d, &[campo], &[E], em(0.5), publicados, |_, _| None, k)
            .into_iter()
            .filter_map(|x| match x {
                TimelineIntent::AddKey { prop, .. } => Some(prop),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        gravados(ferramenta::VECTOR_WIDTH),
        vec![f(v::TRACO_LARGURA)]
    );
    assert_eq!(
        gravados(ferramenta::VECTOR_GAP_IDS[1]),
        vec![f(v::TRACO_TRACEJADO), f(v::TRACO_TRACO), f(v::TRACO_VAO)]
    );
    assert!(
        gravados(NodeId(1)).is_empty(),
        "um campo fora da tabela não grava nada"
    );
}

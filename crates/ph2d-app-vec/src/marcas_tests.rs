//! Gates do [`super`] (`marcas.rs`) — o losango de cada campo do painel do vetor.

use super::*;
use ph2d_anim::{AnimValue, Interp, RationalTime};

const E: u64 = 7;
const SEM: Contexto = Contexto {
    aberta: None,
    camadas: 0,
};
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
    let m = marcas(&d, SEM, &[E], em(0.0), na_curva);
    assert_eq!(m[&ferramenta::VECTOR_FILL_SWATCH], KeyMark::Chave);
    assert_eq!(m[&ferramenta::VECTOR_FILL_OPACITY], KeyMark::Chave);
    assert_eq!(m[&ferramenta::VECTOR_WIDTH], KeyMark::Livre);
    let m = marcas(&d, SEM, &[E], em(0.5), na_curva);
    assert_eq!(m[&ferramenta::VECTOR_FILL_SWATCH], KeyMark::Animado);
    assert!(marcas(&d, SEM, &[], em(0.0), na_curva).is_empty());
}

/// Mexer SÓ o verde a meio já põe o losango da cor em «mudado» — o grupo inteiro responde.
/// (Mutação: ler só o representante do grupo ⇒ o verde passa despercebido.)
#[test]
fn um_membro_mexido_muda_o_losango_do_grupo() {
    let d = doc();
    let verde_mexido = |_, p| Some(if p == f(R + 1) { 7.0 } else { 100.0 });
    let m = marcas(&d, SEM, &[E], em(0.5), verde_mexido);
    assert_eq!(m[&ferramenta::VECTOR_FILL_SWATCH], KeyMark::Mudado);
}

/// Numa chave, o clique apaga as 4 chaves da cor neste quadro — e só essas — num passo de undo.
#[test]
fn clicar_numa_chave_apaga_o_grupo_neste_quadro() {
    let d = doc();
    let i = clique(
        &d,
        SEM,
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
        clique(&d, SEM, &[campo], &[E], em(0.5), publicados, |_, _| None, k)
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

/// ⭐ O mesmo id da camada ABERTA mostra a entrada que está aberta: a 1.ª entrada (`i = 1`) e a
/// 2.ª (`i = 2`) são canais diferentes; fechada, o campo não tem canal. A swatch da entrada `k`
/// mostra a tinta dela (preenchimento OU traço), e o Contour tem canal próprio.
#[test]
fn o_campo_da_camada_aberta_mostra_a_entrada_aberta() {
    let op = |i| PropKind::Field(FieldKey::new(VETOR, v::PILHA_OPACIDADE, i, 0));
    let aberta = |k| Contexto {
        aberta: Some(k),
        camadas: 2,
    };
    assert_eq!(
        canal_do_campo(painel::VECTOR_PAINT_OPACITY, aberta(0)),
        vec![op(1)]
    );
    assert_eq!(
        canal_do_campo(painel::VECTOR_PAINT_OPACITY, aberta(1)),
        vec![op(2)]
    );
    assert!(canal_do_campo(painel::VECTOR_PAINT_OPACITY, SEM).is_empty());
    let sw = canal_do_campo(painel::vector_paint_swatch_id(1), aberta(0));
    assert_eq!(
        sw,
        vec![
            PropKind::Field(FieldKey::new(VETOR, v::TINTA_PREENCH + v::OFF_SOLIDA, 2, 0)),
            PropKind::Field(FieldKey::new(VETOR, v::TINTA_TRACO + v::OFF_SOLIDA, 2, 0)),
        ]
    );
    assert!(canal_do_campo(painel::vector_paint_swatch_id(2), aberta(0)).is_empty());
    assert_eq!(
        canal_do_campo(painel::VECTOR_CONTOUR_TO, SEM),
        vec![f(v::CONTORNO_COR)]
    );
}

/// A opacidade da 2.ª entrada com chaves: o losango acende só com ELA aberta — com a 1.ª aberta o
/// mesmo campo é outro canal. (Mutação: ignorar `aberta` e usar `i = 0` ⇒ nunca acende.)
#[test]
fn o_losango_da_camada_aberta_segue_a_entrada() {
    let mut d = TimelineDoc::new();
    d.fps_display = 24.0;
    let op2 = PropKind::Field(FieldKey::new(VETOR, v::PILHA_OPACIDADE, 2, 0));
    d.insert_key(
        E,
        op2,
        RationalTime::from_seconds(0.0),
        AnimValue::Float(100.0),
        Interp::Linear,
    );
    let ctx = |k| Contexto {
        aberta: Some(k),
        camadas: 2,
    };
    let m = marcas(&d, ctx(1), &[E], em(0.0), na_curva);
    assert_eq!(m[&painel::VECTOR_PAINT_OPACITY], KeyMark::Chave);
    let m = marcas(&d, ctx(0), &[E], em(0.0), na_curva);
    assert_eq!(m[&painel::VECTOR_PAINT_OPACITY], KeyMark::Livre);
    assert!(!marcas(&d, SEM, &[E], em(0.0), na_curva).contains_key(&painel::VECTOR_PAINT_OPACITY));
}

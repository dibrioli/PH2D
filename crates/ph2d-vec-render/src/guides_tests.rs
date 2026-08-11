//! Gates do **rótulo de distância** dos smart guides (plano 25 §9, a W6).
//!
//! Geometria pura — nenhum destes gates sabe o que é uma unidade de display, e é
//! isso que os torna capazes de julgar `snap_labels` sem espelhar a porta que
//! formata o número.

use super::*;

/// Um zoom de `k` px por unidade de mundo, sem rotação nem deslocamento.
fn zoom(k: f64) -> Affine {
    Affine::scale(k)
}

fn align(a: [f64; 2], b: [f64; 2]) -> Guide {
    Guide {
        a,
        b,
        kind: GuideKind::Align,
    }
}

/// **Só o ALINHAMENTO tem o que medir.**
///
/// As outras quatro espécies são pontos (`a == b`) e a distância delas é zero por
/// construção — um `0` flutuante ao lado de cada encaixe seria ruído com
/// aparência de informação.
///
/// Mutação que tem de sangrar: aceitar toda espécie ⇒ quatro fichas dizendo `0`.
#[test]
fn only_the_alignment_guide_gets_a_number() {
    // ⚠️ **`a != b` de propósito, e é o que torna este gate capaz de reprovar.** Os
    // produtores de HOJE colapsam toda espécie de ponto em `a == b`, então uma fixture
    // com pontos coincidentes é filtrada pelo PISO de comprimento e nunca chega a
    // exercitar a lei do KIND — foi assim que a 1ª versão deste gate sobreviveu à
    // mutação que aceita toda espécie. A lei é sobre o SIGNIFICADO da guia (*você está
    // AQUI* não tem distância a dizer), não sobre um acidente de quem a constrói.
    let guides: Vec<Guide> = [
        GuideKind::Grid,
        GuideKind::Curve,
        GuideKind::Crossing,
        GuideKind::GuideHit,
    ]
    .into_iter()
    .map(|kind| Guide {
        a: [0.0, 0.0],
        b: [0.0, 3.0],
        kind,
    })
    .collect();
    assert!(
        snap_labels(&guides, zoom(100.0)).is_empty(),
        "espécie de PONTO não tem distância a dizer, por mais longe que os seus dois \
         campos estejam um do outro"
    );
    // E o controle: a mesma lista mais um alinhamento produz exatamente UM.
    let mut with_align = guides;
    with_align.push(align([0.0, 0.0], [0.0, 1.0]));
    assert_eq!(snap_labels(&with_align, zoom(100.0)).len(), 1);
}

/// **O número é o comprimento em MUNDO, e a âncora é o meio em px de TELA.**
///
/// As duas grandezas vivem em espaços diferentes de propósito: o comprimento é o
/// que o artista quer saber (e o zoom não o muda), a âncora é onde a ficha pousa.
///
/// Mutação que tem de sangrar: medir o comprimento na TELA ⇒ a mesma distância
/// passaria a "encolher" ao afastar a câmera.
#[test]
fn the_length_is_world_and_the_anchor_is_screen() {
    for k in [10.0_f64, 100.0, 400.0] {
        let l = snap_labels(&[align([1.0, 2.0], [1.0, 5.0])], zoom(k));
        assert_eq!(l.len(), 1);
        assert!(
            (l[0].world_len - 3.0).abs() < 1e-9,
            "zoom {k}: o comprimento de mundo não pode depender do zoom (deu {})",
            l[0].world_len
        );
        let mid = [1.0 * k, 3.5 * k];
        assert!(
            (l[0].at[0] - mid[0]).abs() < 1e-6 && (l[0].at[1] - mid[1]).abs() < 1e-6,
            "zoom {k}: a âncora é o meio do segmento em px ({:?} contra {mid:?})",
            l[0].at
        );
    }
}

/// **Um segmento que a tela não mostra não recebe número.**
///
/// O piso é DERIVADO das duas cruzes que capeiam a guia: abaixo dele o segmento
/// está inteiramente coberto pelas próprias marcas.
///
/// ⚠️ E o teste é em ZOOM, não em mundo — a mesma distância de mundo merece
/// número num zoom e não merece noutro, que é exatamente a propriedade.
#[test]
fn a_segment_too_short_to_see_gets_no_number() {
    let g = [align([0.0, 0.0], [0.0, 1.0])];
    assert!(
        snap_labels(&g, zoom(1.0)).is_empty(),
        "1 px de segmento: as cruzes das pontas o cobrem inteiro"
    );
    assert_eq!(
        snap_labels(&g, zoom(100.0)).len(),
        1,
        "100 px de segmento: há linha para rotular"
    );
}

/// **Um alinhamento de comprimento ZERO cai pela MESMA regra**, sem caso especial.
///
/// Ele acontece no mundo real: é a coincidência exata que a lei *vértice vence
/// curva* trata como normal (os dois eixos vindos do mesmo ponto).
#[test]
fn an_exact_landing_has_nothing_to_measure() {
    let p = [7.0, -3.0];
    assert!(snap_labels(&[align(p, p)], zoom(500.0)).is_empty());
}

/// **Um conjunto vazio não produz ficha** — o caso de TODO frame em que ninguém
/// está a arrastar, e o único que corre a cada quadro.
#[test]
fn no_guides_no_labels() {
    assert!(snap_labels(&[], zoom(100.0)).is_empty());
}

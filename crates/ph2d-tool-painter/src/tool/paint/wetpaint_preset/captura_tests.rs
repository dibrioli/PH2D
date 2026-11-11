//! B35.A1 (AU-AU03-1): guardar sobre um preset que trazia um campo «só quando ligado» LIGADO, com o pincel
//! de agora a tê-lo desligado, NÃO o herda — a captura sobre essa `base` grava o mesmo pincel que sobre uma
//! `base` vazia. O `extend` nunca remove: sem o `retain` a chave velha sobrevivia e a leitura a via ligada.

use super::tests::{ferramenta, pincel_mexido};
use super::*;
use ph2d_painter_brush::BrushSpec;

/// Todo campo opcional ligado (o `pincel_mexido` + as cordas do Wire, que ele deixa no de fábrica).
fn tudo_ligado() -> BrushSpec {
    let mut b = pincel_mexido();
    b.wire_sorteio = true;
    b.wire_cordas = 3;
    b
}

/// Captura sobre uma `base` com o `brush` de [`tudo_ligado`] e sobre uma vazia; devolve as duas e as chaves
/// opcionais que a base trazia e a captura limpa não grava (o controlo de que a fixtura não é vazia).
fn as_duas(
    media: &str,
    campos: &[&str],
    capturar: impl Fn(&Parametros) -> Parametros,
) -> (Parametros, Parametros, Vec<String>) {
    let vazia = Parametros::novo(media, "Teste", "Ligado");
    let mut base = vazia.clone();
    base.brush = json::pincel_para_json(&tudo_ligado(), campos);
    let limpo = capturar(&vazia);
    let p = capturar(&base);
    let opcionais = base
        .brush
        .keys()
        .filter(|k| !limpo.brush.contains_key(*k))
        .cloned()
        .collect();
    (p, limpo, opcionais)
}

#[test]
fn a_captura_nao_herda_o_campo_opcional_desligado() {
    // Wet Paint: pelos dois caminhos (o direto e o `preset_capturar`).
    let t = ferramenta();
    for (via, cap) in [
        (
            "wet_preset_capturar",
            &(|b: &Parametros| t.wet_preset_capturar(b).0) as &dyn Fn(&Parametros) -> Parametros,
        ),
        ("preset_capturar", &|b: &Parametros| t.preset_capturar(b).0),
    ] {
        let (p, limpo, opcionais) = as_duas(MEIO_DO_PRESET, &CAMPOS_DO_PINCEL, cap);
        assert!(
            [
                "pontas_rectas",
                "dual",
                "hose",
                "size_max_px",
                "pigment_espectral"
            ]
            .iter()
            .all(|c| opcionais.iter().any(|o| o == c)),
            "controlo ({via}): a base traz só {opcionais:?}"
        );
        assert_eq!(p.brush, limpo.brush, "{via} herdou da base");
    }
    // Os outros três meios.
    for m in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
    ] {
        let mut t = ferramenta();
        t.set_paint_media(m);
        assert_eq!(t.paint_media(), m, "a ferramenta de teste não tem {m:?}");
        let (p, limpo, opcionais) = as_duas(nome_do_meio(m), &campos_do_meio(m), |b| {
            t.preset_capturar(b).0
        });
        assert!(!opcionais.is_empty(), "controlo ({m:?}): nada opcional");
        assert_eq!(
            p.brush, limpo.brush,
            "preset_capturar ({m:?}) herdou da base"
        );
    }
}

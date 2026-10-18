//! **As cenas de `PH2D_MACRO_SMOKE`** — o número que a família declara ([`CENAS_DO_SMOKE`]) é o que a
//! composição serve: cada nível `1..=CENAS_DO_SMOKE` tem um braço em `render_loop/macro_smoke.rs`, e o
//! seguinte não. A cena `=5` (W7 item 1) abre PARADA — o artista pinta antes do Play; a `=1` toca sozinha.

use ph2d_app_macros::{CENAS_DO_SMOKE, FAMILY, a_cena_abre_parada};

const COMPOSICAO: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../shells/desktop/src/render_loop/macro_smoke.rs"
);

#[test]
fn cada_cena_declarada_tem_um_braco_na_composicao_e_a_seguinte_nao() {
    let src = std::fs::read_to_string(COMPOSICAO).expect("a composição das cenas das Macros");
    for n in 1..=CENAS_DO_SMOKE {
        assert!(
            src.contains(&format!("\"{n}\" =>")),
            "PH2D_MACRO_SMOKE={n} é declarada pela família e a composição não a serve"
        );
    }
    let seguinte = CENAS_DO_SMOKE + 1;
    assert!(
        !src.contains(&format!("\"{seguinte}\" =>")),
        "a composição serve PH2D_MACRO_SMOKE={seguinte} e a família declara só {CENAS_DO_SMOKE}"
    );
    assert_eq!(FAMILY.routers[0].max_level, CENAS_DO_SMOKE);
}

#[test]
fn so_a_cena_da_imagem_do_artista_abre_parada() {
    let paradas: Vec<u32> = (1..=CENAS_DO_SMOKE)
        .filter(|n| a_cena_abre_parada(&n.to_string()))
        .collect();
    assert_eq!(
        paradas,
        [5, 6],
        "a cena que abre parada (clipe sem Autoplay, imagem em Paint)"
    );
}

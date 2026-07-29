//! **Gate de FONTE da cena `PH2D_MORPH_FADE_SMOKE`** (ADR-0146 C4).
//!
//! A cena é env-gated, então nenhum teste de unidade a alcança — e o precedente desta
//! linha é claro sobre o que acontece então: o `expr_blend_smoke` montava clips sem
//! duração autorada e a aba Keys abria sem véu, com a suíte inteira verde
//! (`the_expr_blend_smoke_authors_clip_durations.rs`). Uma cena de smoke que para de
//! encenar o fenômeno é indistinguível de uma feature quebrada.
//!
//! Aqui o fenômeno tem TRÊS pré-condições, e cada uma sozinha esvazia a demonstração:
//!
//! 1. a pose autorada tem de DIFERIR da key — se convergirem, não há distância que um
//!    estalo possa percorrer, e a cena fica verde provando nada;
//! 2. a strip tem de ter `ease_in > 0` — sem fade não existe o quadro de peso zero que
//!    é exatamente onde o `rest` é lido;
//! 3. a strip não pode começar em 0 — o defeito vivia ANTES dela, na região que a
//!    composição não cobre.

use std::path::Path;

const SRC: &str = "src/morph_fade_smoke.rs";

fn source() -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(SRC);
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("nao consegui ler {}: {e}", p.display()))
}

#[test]
fn the_authored_pose_differs_from_the_key() {
    let s = source();
    let grab = |name: &str| -> f32 {
        let at = s
            .find(&format!("const {name}: f32 = "))
            .unwrap_or_else(|| panic!("a const {name} sumiu da cena"));
        let tail = &s[at + format!("const {name}: f32 = ").len()..];
        tail[..tail.find(';').expect("faltou o ;")]
            .trim()
            .parse()
            .expect("a const nao e um float literal")
    };
    let authored = grab("AUTHORED_T");
    let keyed = grab("KEYED_T");
    assert!(
        (authored - keyed).abs() > 0.5,
        "a pose autorada ({authored}) e a key ({keyed}) precisam ficar LONGE uma da \
         outra: e a distancia entre elas que torna um estalo visivel"
    );
}

#[test]
fn the_scene_stages_a_fade_in_that_starts_after_zero() {
    let s = source();
    assert!(
        s.contains("strips[0].ease_in = 1.0"),
        "sem ease_in nao ha o quadro de peso zero onde o `rest` e lido — \
         a cena deixaria de encenar o defeito"
    );
    assert!(
        s.contains("add_strip(lane, 0, 2.0, 6.0)"),
        "a strip tem de comecar DEPOIS de 0: o defeito vivia na regiao que a \
         composicao nao cobre, antes dela"
    );
}

#[test]
fn the_needle_reads_the_morph_channel_through_a_prop_link() {
    let s = source();
    assert!(
        s.contains("Morpher.morph"),
        "a agulha e a metade VISIVEL do prop-link: sem ela a cena so prova o `rest`, \
         e o C4 tem duas metades"
    );
}

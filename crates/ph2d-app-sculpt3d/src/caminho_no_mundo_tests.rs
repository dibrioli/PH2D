//! ⭐⭐⭐ **A ROTA da cura do vinco pontilhado, dentro do app** — não a lei, que a
//! `ph2d-sculpt3d` mede contra o oráculo, mas os quatro sítios da shell-crate
//! que a ligam ao gesto.
//!
//! ⚠️⚠️ **Eles não são alcançáveis de um teste de unidade:** o pen-down real
//! precisa de um `AppHost` e a cena de um `wgpu::Device` (os gates que o têm são
//! `#[ignore]`, logo o CI nunca os corre). ⇒ a rota mede-se pelo TEXTO do
//! despacho, por [`include_str!`] — que **deixa de compilar** se o ficheiro
//! mudar de sítio, ao contrário de um `read_to_string` em runtime, que fica
//! verde a medir o nada.
//!
//! ⛔ **Cada gate tem PISO de população**: uma agulha que deixe de casar tem de
//! reprovar, e não passar por vácuo.

const INPUT: &str = include_str!("input.rs");
const SPACE: &str = include_str!("space.rs");
const DOWN: &str = include_str!("input_down.rs");
const HISTORY: &str = include_str!("history.rs");

/// ⭐ **O carimbo que mede o passo no mundo vai pela lei nova, e mais nada vai.**
///
/// ⛔ Sem a segunda metade, um despacho que mandasse TODO verbo por ali ficaria
/// verde — e levaria consigo os `86` traços do tecido e os `69` da pose.
#[test]
fn so_o_verbo_que_declara_o_passo_no_mundo_percorre_a_lei_nova() {
    assert!(
        INPUT.contains("if scene.brush.verb.mede_o_passo_no_mundo() =>"),
        "o braço do carimbo deixou de perguntar ao verbo antes de percorrer o mundo"
    );
    assert_eq!(
        INPUT.matches("percorre_no_mundo(scene, x, y)").count(),
        1,
        "a lei nova tem de ter UM chamador de produto"
    );
}

/// ⭐⭐ **As DUAS perguntas armam a fotografia, e quem lê o CENTRO pergunta ao
/// verbo e nunca à presença dela.**
///
/// ⚠️ É a distinção que o comentário do `space.rs` nomeia: armar a fotografia
/// para medir o PASSO não pode trocar, em silêncio, a lei do CENTRO do pincel de
/// projectar, que a arma por outro motivo.
#[test]
fn a_fotografia_do_pen_down_e_armada_pelas_duas_perguntas() {
    assert!(
        SPACE.contains("pica_na_superficie_do_pen_down()")
            && SPACE.contains("|| self.brush.verb.mede_o_passo_no_mundo()"),
        "a fotografia deixou de ser armada pelas duas perguntas"
    );
    assert!(
        SPACE.contains("if self.brush.verb.o_dab_segue_o_barro() {"),
        "o centro do dab deixou de perguntar ao verbo"
    );
    assert!(
        SPACE.contains("levado_pela_deformacao"),
        "o acerto deixou de ser levado pela deformação"
    );
}

/// ⭐ **O pen-up esquece o caminho, nos DOIS sítios que o fecham.**
///
/// ⚠️ Sem isto o resíduo de um traço atravessa a caneta levantada e o traço
/// seguinte nasce com um pedaço de passo já andado — estado de TRAÇO a
/// sobreviver ao fim dele.
#[test]
fn o_pen_up_e_o_desfazer_esquecem_o_caminho() {
    assert!(
        DOWN.contains("caminho_no_mundo.esquece()"),
        "o pen-down/pen-up deixou de esquecer o caminho"
    );
    assert!(
        HISTORY.contains("caminho_no_mundo.esquece()"),
        "o desfazer deixou de esquecer o caminho"
    );
}

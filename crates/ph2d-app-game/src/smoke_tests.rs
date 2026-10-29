use super::*;

#[test]
fn o_nivel_le_se_pela_porta_e_fora_da_faixa_e_inerte() {
    let com = |v: &'static str| move |k: &str| (k == "PH2D_GAME_SMOKE").then(|| v.to_owned());
    assert_eq!(armed_level_in(com("1")), Some(1));
    assert_eq!(armed_level_in(com("2")), Some(2));
    assert_eq!(armed_level_in(com("3")), Some(3));
    assert_eq!(armed_level_in(com("4")), Some(4));
    assert_eq!(armed_level_in(com("5")), None);
    assert_eq!(armed_level_in(com("x")), None);
    assert_eq!(armed_level_in(|_| None), None);
}

#[test]
fn o_condutor_joga_corre_e_para_por_esta_ordem() {
    let com = |v: &'static str| move |k: &str| (k == "PH2D_GAME_SMOKE_DRIVE").then(|| v.to_owned());
    assert_eq!(conducao_in(com("window")), Some(GameStage::Window));
    assert_eq!(conducao_in(com("fullscreen")), Some(GameStage::Fullscreen));
    assert_eq!(conducao_in(com("editor")), Some(GameStage::Editor));
    assert_eq!(conducao_in(com("x")), None);
    let s = GameStage::Fullscreen;
    let g: Vec<_> = (0..1000).filter_map(|q| gesto_no_quadro(1, s, q)).collect();
    assert_eq!(
        g,
        [
            Gesto::Joga(s),
            Gesto::Direita(true),
            Gesto::Direita(false),
            Gesto::Para
        ]
    );
}

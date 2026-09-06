//! ⭐⭐⭐ **O CENSO DAS FILEIRAS DE CHIP** — o painel oferece exactamente o que o
//! motor tem.
//!
//! ⛔⛔ **Este ficheiro existe porque o gate que ele contém era NOMEADO em dois
//! sítios e NÃO EXISTIA.** Os doc-comments de `ph2d_sculpt3d::Falloff::ALL` e de
//! `ids::SCULPT3D_FALLOFF` prometiam, cada um, que
//! `the_panel_offers_every_falloff_the_engine_has` compara os dois arrays e fica
//! vermelho quando uma curva nova não passa pelo painel. Ninguém o tinha
//! escrito. *Uma promessa de gate lê-se exactamente como um gate, e a diferença
//! só aparece no dia em que ele devia sangrar.*
//!
//! ⚠️ **A régua é a CONTAGEM dos dois lados**, e é isso que a torna um censo: um
//! valor novo no `ALL` do motor sem id correspondente reprova, e um id a mais
//! sem valor no motor também — o segundo é o chip que aponta para nada.

use ph2d_editor_core::ids;
use ph2d_sculpt3d::{ClothArea, ClothMode, Falloff};

/// **GATE — o painel oferece TODA curva de falloff que o motor tem.**
#[test]
fn the_panel_offers_every_falloff_the_engine_has() {
    assert_eq!(
        ids::SCULPT3D_FALLOFF.len(),
        Falloff::ALL.len(),
        "o painel tem {} chips de falloff e o motor tem {} curvas -- uma curva sem \
         id nasce inalcancavel, e um id sem curva e' um chip que aponta para nada",
        ids::SCULPT3D_FALLOFF.len(),
        Falloff::ALL.len()
    );
}

/// **GATE — o painel oferece TODO modo de deformação do tecido.**
///
/// ⚠️ **É o gate que faltava quando o pincel tinha oito comportamentos e um
/// alcançável**: a lei da referência respondia aos oito e nenhum id os
/// desenhava. Um modo novo agora nasce vermelho aqui.
#[test]
fn the_panel_offers_every_cloth_mode_the_engine_has() {
    assert_eq!(
        ids::SCULPT3D_CLOTH_MODE.len(),
        ClothMode::ALL.len(),
        "o painel tem {} chips de deformacao e o motor tem {} modos",
        ids::SCULPT3D_CLOTH_MODE.len(),
        ClothMode::ALL.len()
    );
    assert_eq!(
        ids::SCULPT3D_CLOTH_AREA.len(),
        ClothArea::ALL.len(),
        "o painel tem {} chips de area e o motor tem {}",
        ids::SCULPT3D_CLOTH_AREA.len(),
        ClothArea::ALL.len()
    );
}

/// **GATE — cada rótulo é distinto, e nenhum é vazio.**
///
/// ⚠️ Dois chips com o mesmo texto são um controlo que o artista não consegue
/// escolher, e um chip vazio é um botão sem nome. As duas coisas passam por
/// qualquer censo de CONTAGEM.
#[test]
fn no_two_chips_of_a_row_carry_the_same_label() {
    let modos: Vec<&str> = ClothMode::ALL.iter().map(|m| m.label()).collect();
    let areas: Vec<&str> = ClothArea::ALL.iter().map(|a| a.label()).collect();
    for (nome, rotulos) in [("deformacao", &modos), ("area", &areas)] {
        for (i, a) in rotulos.iter().enumerate() {
            assert!(!a.trim().is_empty(), "{nome}: o chip {i} nao tem rotulo");
            for b in rotulos.iter().skip(i + 1) {
                assert_ne!(a, b, "{nome}: dois chips dizem {a:?}");
            }
        }
    }
}

/// **GATE — o modo decide o braço do gesto, e os DOIS lados existem.**
///
/// A shell escolhe entre re-apanhar o cursor na superfície e andar no plano de
/// profundidade perguntando ao modo (espec §4.3). ⚠️ **O anti-vácuo é metade do
/// gate:** se a pergunta devolvesse sempre o mesmo valor, a escolha da shell
/// seria uma constante disfarçada de decisão.
#[test]
fn the_cloth_mode_decides_which_arm_the_gesture_takes() {
    let repicam = ClothMode::ALL.iter().filter(|m| m.repica()).count();
    assert!(
        repicam > 0 && repicam < ClothMode::ALL.len(),
        "{repicam} de {} modos re-apanham o cursor -- a pergunta e' uma constante",
        ClothMode::ALL.len()
    );
    assert!(
        !ClothMode::Grab.repica() && !ClothMode::SnakeHook.repica(),
        "os dois modos de ANCORA nao re-apanham o cursor (espec §4.3)"
    );
    assert!(ClothMode::Drag.repica(), "os modos de FORCA re-apanham");
}

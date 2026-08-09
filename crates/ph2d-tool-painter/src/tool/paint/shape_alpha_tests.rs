//! **De onde a silhueta do Shape vem** — do ALPHA da imagem ou das diferenças de claro e escuro dela.
//!
//! Report do Enio (2026-08-09): *"qualquer imagem colocada ali recebe transparência mesmo nas áreas
//! onde a imagem como foi desenhada não tem transparência … só coloca transparência onde há
//! transparência na imagem usada"*.
//!
//! ⚠️ **O pedido era um CHECKBOX, e é isso que estes gates prendem:** o default de cada rota é o
//! comportamento que ela JÁ tinha — a captura de documento silhueta pelo alpha autorado das camadas,
//! uma imagem importada silhueta pelo tom —, e o que passa a existir é a escolha. Um default MEDIDO
//! (*alpha quando ele recorta algo, luminância quando é opaco*) foi construído e descartado: ele
//! mudava o desenho em duas direções que ninguém pediu, e a segunda metade — um `.png` recortado
//! deixando de ser silhuetado pelo tom — é indistinguível de uma regressão para quem já tem arte
//! feita com a lei velha.

use crate::PainterTool;

/// Um sprite `8×8` cuja LUMINÂNCIA e cujo ALPHA variam de formas DIFERENTES — senão as duas leis
/// coincidem e nenhum gate aqui consegue distinguir uma da outra.
fn sprite() -> (Vec<u8>, u32, u32) {
    let (w, h) = (8u32, 8u32);
    let mut px = vec![0u8; (w * h) as usize * 4];
    for (i, p) in px.chunks_exact_mut(4).enumerate() {
        p[0] = (i * 3) as u8;
        p[1] = (i * 5) as u8;
        p[2] = (i * 7) as u8;
        p[3] = 255 - (i * 2) as u8;
    }
    (px, w, h)
}

fn luminance(rgba: &[u8]) -> Vec<u8> {
    rgba.chunks_exact(4)
        .map(|p| ((u32::from(p[0]) * 77 + u32::from(p[1]) * 150 + u32::from(p[2]) * 29) >> 8) as u8)
        .collect()
}

fn alpha(rgba: &[u8]) -> Vec<u8> {
    rgba.chunks_exact(4).map(|p| p[3]).collect()
}

/// **As duas leis são alcançáveis, e o interruptor as troca** — o report, no mecanismo.
///
/// O default de uma imagem é o TOM (o que ela sempre fez); marcado, a silhueta é o alpha do arquivo,
/// byte a byte. As duas metades juntas de propósito: sem a primeira o gate não diria que o default
/// não mudou, e sem a segunda ele não diria que o checkbox faz alguma coisa.
///
/// **Mutação que tem de sangrar:** `rebuild_masks` ignorar `alpha_from_image`.
#[test]
fn the_silhouette_is_the_tone_by_default_and_the_alpha_when_asked() {
    let (px, w, h) = sprite();
    let mut t = PainterTool::default();
    t.set_brush_shape_image_rgba(&px, w, h, Some(7));

    assert!(
        !t.brush_shape_alpha_from_image(),
        "uma imagem importada silhueta pelo TOM — é o que ela sempre fez, e o pedido era um checkbox"
    );
    assert_eq!(
        t.brush_shape_image().expect("silhueta").0,
        luminance(&px).as_slice(),
        "a silhueta default não é a luminância do arquivo"
    );

    t.toggle_brush_shape_alpha_from_image();
    assert!(t.brush_shape_alpha_from_image());
    assert_eq!(
        t.brush_shape_image().expect("silhueta").0,
        alpha(&px).as_slice(),
        "marcado, a transparência tem de vir de onde a imagem de fato tem transparência"
    );

    // …e volta. A troca é uma ESCOLHA sobre planos que já estão guardados, não uma re-importação:
    // desmarcar não pode precisar do arquivo de novo.
    t.toggle_brush_shape_alpha_from_image();
    assert_eq!(
        t.brush_shape_image().expect("silhueta").0,
        luminance(&px).as_slice()
    );
}

/// **O interruptor só é OFERECIDO quando há para onde virar.** Sem RGB capturado a luminância não
/// existe, e um checkbox com um valor alcançável só é o controle morto que este painel recusa.
///
/// **Mutação que tem de sangrar:** `has_alpha_choice` devolver `!self.src.is_empty()`.
#[test]
fn the_switch_is_offered_only_when_the_other_law_exists() {
    let mut t = PainterTool::default();
    assert!(
        !t.brush_shape_has_alpha_choice(),
        "sem Shape capturada não há duas leis para escolher"
    );

    // Uma máscara CRUA (a rota legada de arquivo, que não guarda cor): há um plano, e só um.
    t.set_brush_shape_layers(vec![(vec![200u8; 64], 8, 8)]);
    assert!(
        !t.brush_shape_has_alpha_choice(),
        "sem RGB capturado a luminância não existe — o checkbox não tem para onde virar"
    );

    let (px, w, h) = sprite();
    t.set_brush_shape_image_rgba(&px, w, h, Some(7));
    assert!(t.brush_shape_has_alpha_choice());
}

/// **O RELEVO sobrevive à troca** — ele é um GANHO sobre a silhueta, e as duas leis o carregam.
///
/// ⚠️ **O controle varia o GANHO e nada mais**, e as duas primeiras versões deste gate erraram nisso.
/// A primeira pedia que a silhueta VARIASSE — e a luminância de um desenho varia sozinha (é a cor da
/// tinta sobre o papel), então ela passava com o ganho inteiro removido. A segunda comparava contra o
/// mesmo gesto **sem impasto**, e aí o que difere é o CANVAS (um depósito com corpo não pinta os
/// mesmos bytes que um sem), não o ganho: ela também passava.
///
/// O que segura o documento fixo é o `impasto_show` — ele é EXIBIÇÃO (`impasto_visible` é quem o lê),
/// não muda um byte do canvas, e é exatamente o interruptor de que o ganho depende.
///
/// **Mutação que tem de sangrar:** `rebuild_masks` deixar de multiplicar pelo ganho.
#[test]
fn the_relief_survives_the_switch() {
    let silhouette = |show_relief: bool, alpha_law: bool| -> Vec<u8> {
        let mut t = super::use_as_relief_tests::ridge_for_alpha_gate();
        t.paint.impasto_show = show_relief;
        t.capture_layers_as_brush_shape();
        if t.brush_shape_alpha_from_image() != alpha_law {
            t.toggle_brush_shape_alpha_from_image();
        }
        t.brush_shape_image().expect("silhueta").0.to_vec()
    };
    for (law, name) in [(true, "alpha"), (false, "luminância")] {
        let lit = silhouette(true, law);
        let unlit = silhouette(false, law);
        let differing = lit.iter().zip(unlit.iter()).filter(|(a, b)| a != b).count();
        assert!(
            differing > 100,
            "a lei {name} perdeu o relevo: só {differing} texels diferem da MESMA captura com a luz \
             desligada — o ganho tem de valer para as DUAS"
        );
    }
}

/// **A escolha do artista sobrevive a uma re-captura da MESMA fonte**, como as cores por camada — o
/// auto-refresh dispara a cada edição do sprite de referência, e perder o interruptor a cada
/// pincelada nele seria a ferramenta desfazendo a escolha sozinha.
///
/// **Mutação que tem de sangrar:** `restore_assignments` ignorar o `alpha_from_image`.
#[test]
fn the_choice_survives_a_recapture_of_the_same_source() {
    let mut t = super::use_as_relief_tests::ridge_for_alpha_gate();
    // ⚠️ **A premissa da preservação é um documento LIGADO** (`same_source` exige
    // `bound_doc.is_some()`): sem ela toda re-captura é "uma fonte nova", e o gate mediria o
    // caminho errado — o mesmo em que as CORES por camada também são resetadas, de propósito.
    t.bound_doc = Some(1);
    t.capture_layers_as_brush_shape();
    let before = t.brush_shape_alpha_from_image();
    t.toggle_brush_shape_alpha_from_image();
    let chosen = t.brush_shape_alpha_from_image();
    assert_ne!(before, chosen, "premissa: o gesto de fato virou o valor");
    t.capture_layers_as_brush_shape();
    assert_eq!(
        t.brush_shape_alpha_from_image(),
        chosen,
        "a re-captura da mesma fonte jogou fora a escolha do artista"
    );
}

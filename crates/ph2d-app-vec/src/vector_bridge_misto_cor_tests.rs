//! ⭐ **O «misto» completo: a cor escolhida chega a TODAS, e a pilha de camadas também sabe
//! «misto»** (plano 32, `docs/UI_New_and_Simple/vetor_nova_ui/32_plano_F21_o_misto_completo.md`).
//!
//! Fixture: a vermelha cheia com DUAS camadas extra e a azul (que o teste esvazia quando precisa)
//! com UMA, as duas selecionadas, o primário a vermelha. O seletor é escrito pela porta do store e
//! o despacho é o real, quadro a quadro.

use super::misto_tests::{AZUL, Quadro, VERMELHO, rgba};
use ph2d_editor_core::ids::INSP_BLENDER_PICKER;
use ph2d_tokens::ColorValue;
use ph2d_tool_vector::ids;
use ph2d_vec_scene::{Paint, PaintEntry, Rgba8, StrokeSpec};

const VERDE: [u8; 4] = [10, 200, 90, 255];

/// Abre o seletor na amostra do Fill (a porta do clique numa amostra) e escreve `c` nele.
fn escolhe_fill(q: &mut Quadro, c: [u8; 4], tocou: bool) {
    if q.hero.store.picker_target() != Some(ids::VECTOR_FILL_SWATCH) {
        q.hero.store.open_picker_on(ids::VECTOR_FILL_SWATCH);
        q.quadro();
    }
    q.hero.store.set_blender_value(
        INSP_BLENDER_PICKER,
        ColorValue::from_rgba8(c[0], c[1], c[2], c[3]),
    );
    if tocou {
        q.hero.store.note_colour_pick(INSP_BLENDER_PICKER);
    }
    q.quadro();
}

fn sem_fill(q: &mut Quadro, id: ph2d_vec_scene::VecPathId) {
    q.scene.path_mut(id).expect("forma").fill = None;
    q.quadro();
}

/// ⛔ **Uma forma sem preenchimento ao lado de uma cheia nunca ganhava a cor escolhida**: o matiz
/// armava só o RGB, e a alfa 0 dela («sem preenchimento») sobrevivia.
#[test]
fn uma_cor_escolhida_preenche_quem_nao_tinha() {
    let mut q = Quadro::novo();
    let azul = q.azul;
    sem_fill(&mut q, azul);
    escolhe_fill(&mut q, VERDE, true);
    assert_eq!(q.path(q.vermelha).fill, Some(Paint::Solid(rgba(VERDE))));
    assert_eq!(
        q.path(q.azul).fill,
        Some(Paint::Solid(rgba(VERDE))),
        "a forma vazia continuou vazia"
    );
}

/// ⛔ **Escolher exatamente a cor do primário não fazia nada**: a cor só armava na MUDANÇA, e a
/// ferramenta já tinha a cor do primário.
#[test]
fn escolher_a_cor_do_primario_escreve_em_todas() {
    let mut q = Quadro::novo();
    escolhe_fill(&mut q, VERMELHO, true);
    assert_eq!(
        q.path(q.azul).fill,
        Some(Paint::Solid(rgba(VERMELHO))),
        "o clique na cor do primário não chegou à azul"
    );
}

/// O controlo: o seletor ABERTO, sondado a cada quadro, sem toque, não escreve nada — senão abrir
/// o seletor sobre um Fill misto pintaria todas com a cor do primário.
#[test]
fn o_seletor_aberto_sem_toque_nao_escreve() {
    let mut q = Quadro::novo();
    escolhe_fill(&mut q, VERMELHO, false);
    q.quadro();
    assert_eq!(q.path(q.azul).fill, Some(Paint::Solid(rgba(AZUL))));
}

/// O controlo: arrastar SÓ a alfa no seletor muda a alfa de todas e não pisa as cores.
#[test]
fn arrastar_so_a_alfa_nao_escreve_o_rgb() {
    let mut q = Quadro::novo();
    escolhe_fill(&mut q, [VERMELHO[0], VERMELHO[1], VERMELHO[2], 128], false);
    assert_eq!(
        q.path(q.azul).fill,
        Some(Paint::Solid(Rgba8::new(AZUL[0], AZUL[1], AZUL[2], 128)))
    );
    assert_eq!(
        q.path(q.vermelha).fill,
        Some(Paint::Solid(Rgba8::new(
            VERMELHO[0],
            VERMELHO[1],
            VERMELHO[2],
            128
        )))
    );
}

/// ⛔ **Uma forma só, sem preenchimento: mexer no matiz não fazia nada** — o seletor abria com a
/// alfa 0 dela. Agora a forma ganha a cor opaca, e a ferramenta e o seletor dizem o mesmo no MESMO
/// quadro (senão a sondagem seguinte leria a alfa 0 do seletor e apagaria a forma).
#[test]
fn uma_forma_vazia_ganha_cor_ao_mexer_no_matiz() {
    let mut q = Quadro::com(false);
    let vermelha = q.vermelha;
    sem_fill(&mut q, vermelha);
    assert_eq!(q.tool().fill_rgba()[3], 0, "a semente da forma vazia");
    escolhe_fill(&mut q, [VERDE[0], VERDE[1], VERDE[2], 0], true);
    assert_eq!(
        q.path(q.vermelha).fill,
        Some(Paint::Solid(rgba(VERDE))),
        "a forma vazia não ganhou a cor"
    );
    assert_eq!(q.tool().fill_rgba(), VERDE);
    let (v, ..) = q
        .hero
        .store
        .blender_picker(INSP_BLENDER_PICKER)
        .expect("seletor");
    assert_eq!(v.rgba, VERDE, "o seletor ficou com a alfa 0");
    q.quadro();
    q.quadro();
    assert_eq!(
        q.path(q.vermelha).fill,
        Some(Paint::Solid(rgba(VERDE))),
        "o quadro seguinte apagou o preenchimento"
    );
}

/// As camadas extra da fixture: a vermelha com um contorno e um preenchimento, a azul com um
/// preenchimento só.
fn com_camadas(q: &mut Quadro) {
    let v = q.scene.path_mut(q.vermelha).expect("forma");
    v.paints.push(PaintEntry::stroke(StrokeSpec::new(
        Rgba8::new(250, 200, 0, 255),
        0.05,
    )));
    v.paints
        .push(PaintEntry::fill(Paint::Solid(Rgba8::new(0, 0, 0, 90))));
    let a = q.scene.path_mut(q.azul).expect("forma");
    a.paints.push(PaintEntry::fill(Paint::Solid(Rgba8::new(
        255, 255, 255, 60,
    ))));
    q.quadro();
}

/// ⛔ **A pilha lia `sel.first()`**: selecionar a azul e depois a vermelha (primário = vermelha)
/// mostrava as camadas da azul ao lado da Width da vermelha.
#[test]
fn a_pilha_le_o_primario() {
    let mut q = Quadro::novo();
    com_camadas(&mut q);
    let a = ph2d_panel_vector::state::current_appearance().expect("aparência publicada");
    assert_eq!(a.layers.len(), 2, "a pilha publicada não é a do primário");
    assert!(!a.layers[0].is_fill && a.layers[1].is_fill);
}

/// ⛔ **A pilha não sabia «misto»**: com estruturas diferentes (2 camadas × 1) o painel listava as
/// do primário, e um clique na linha `1` escrevia na `1` de cada uma — outra espécie na azul.
#[test]
fn a_pilha_diz_misto_quando_as_camadas_diferem() {
    let mut q = Quadro::novo();
    com_camadas(&mut q);
    let a = ph2d_panel_vector::state::current_appearance().expect("aparência publicada");
    assert!(a.layers_differ, "2 camadas × 1 não é «misto»");
    // A mesma estrutura nas duas, cores diferentes na camada 0: as linhas voltam, e só a cor é
    // «misto».
    let a_ = q.azul;
    let azul = q.scene.path_mut(a_).expect("forma");
    azul.paints.clear();
    azul.paints.push(PaintEntry::stroke(StrokeSpec::new(
        Rgba8::new(0, 90, 255, 255),
        0.05,
    )));
    azul.paints
        .push(PaintEntry::fill(Paint::Solid(Rgba8::new(0, 0, 0, 90))));
    q.quadro();
    let a = ph2d_panel_vector::state::current_appearance().expect("aparência publicada");
    assert!(!a.layers_differ);
    let m = a.layers[0].mixed;
    assert!(m.color, "a cor da camada 0 difere");
    assert!(!m.width && !m.opacity && !m.blend && !m.offset && !m.dilate);
    assert_eq!(a.layers[1].mixed, Default::default(), "a camada 1 concorda");
}

/// As escritas da pilha que precisam do primário: o olho, um eixo, a cor pelo seletor.
mod pilha {
    use crate::paint_stack;
    use ph2d_vec_scene::{Paint, PaintEntry, Rgba8, VecScene};

    fn duas() -> (VecScene, Vec<ph2d_vec_scene::VecPathId>) {
        let mut scene = VecScene::new();
        let mut ids = Vec::new();
        for (cor, olho, dx) in [
            ([200, 0, 0, 255], true, 1.0),
            ([0, 0, 200, 255], false, 2.0),
        ] {
            let mut p = ph2d_vec_scene::rounded_rect([0.0, 0.0], [1.0, 1.0], 0.1);
            let mut e = PaintEntry::fill(Paint::Solid(Rgba8::new(cor[0], cor[1], cor[2], cor[3])));
            e.enabled = olho;
            e.offset = [dx, dx * 10.0];
            p.paints.push(e);
            ids.push(scene.push_path(p));
        }
        (scene, ids)
    }

    fn camada(scene: &VecScene, id: ph2d_vec_scene::VecPathId) -> &PaintEntry {
        &scene.path(id).expect("forma").paints[0]
    }

    /// ⛔ O olho invertia cada uma por si: a acesa apagava e a apagada acendia.
    #[test]
    fn o_olho_de_uma_pilha_mista_fica_igual_em_todas() {
        let (mut scene, sel) = duas();
        assert!(paint_stack::toggle(&mut scene, &sel, 0));
        assert!(!camada(&scene, sel[0]).enabled && !camada(&scene, sel[1]).enabled);
    }

    /// ⛔ Mexer só no X lia o Y do primário e escrevia-o em todas.
    #[test]
    fn um_eixo_do_deslocamento_nao_apaga_o_outro() {
        let (mut scene, sel) = duas();
        assert!(paint_stack::set_offset_axes(
            &mut scene,
            &sel,
            0,
            Some(5.0),
            None
        ));
        assert_eq!(camada(&scene, sel[0]).offset, [5.0, 10.0]);
        assert_eq!(
            camada(&scene, sel[1]).offset,
            [5.0, 20.0],
            "o Y da azul foi apagado"
        );
    }

    /// ⛔ O seletor aberto na amostra de uma camada gravava a cor do primário em todas a cada
    /// quadro, sem toque. Com o toque (a cor do primário), escreve; arrastar a alfa não pisa o RGB.
    #[test]
    fn o_seletor_numa_camada_mista_so_escreve_o_que_o_gesto_mexe() {
        use ph2d_editor_core::ids::INSP_BLENDER_PICKER;
        let (mut scene, sel) = duas();
        let mut hero =
            ph2d_editor_core::screens::hero::HeroScreen::new(ph2d_editor_core::NodeId(1));
        let sw = ph2d_panel_vector::ids::vector_paint_swatch_id(0);
        hero.store.set_widget_color(sw, [200, 0, 0, 255]);
        hero.store.open_picker_on(sw);
        let azul = Paint::Solid(Rgba8::new(0, 0, 200, 255));
        let fill = |s: &VecScene, id| match &camada(s, id).kind {
            ph2d_vec_scene::PaintKind::Fill(p) => p.clone(),
            _ => unreachable!(),
        };
        // Aberto, sem toque: nada.
        paint_stack::apply_picker(&mut hero.store, &mut scene, &sel);
        paint_stack::apply_picker(&mut hero.store, &mut scene, &sel);
        assert_eq!(
            fill(&scene, sel[1]),
            azul,
            "abrir o seletor carimbou o primário"
        );
        // A alfa só: as cores ficam.
        hero.store.set_blender_value(
            INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(200, 0, 0, 100),
        );
        paint_stack::apply_picker(&mut hero.store, &mut scene, &sel);
        assert_eq!(
            fill(&scene, sel[1]),
            Paint::Solid(Rgba8::new(0, 0, 200, 100))
        );
        // O toque na cor do primário: o RGB chega à azul.
        hero.store.note_colour_pick(INSP_BLENDER_PICKER);
        paint_stack::apply_picker(&mut hero.store, &mut scene, &sel);
        assert_eq!(
            fill(&scene, sel[1]),
            Paint::Solid(Rgba8::new(200, 0, 0, 100))
        );
    }
}

//! **Os gates do CARIMBO da doação** — irmão (`#[path]`) do [`super`].
//!
//! ⚠️ O corte foi o tecto de LOC (700), e o gatilho foi a wave do enquadramento do bake
//! (2026-09-21), que acrescentou ao carimbo a LENTE e o RECORTE — e um gate por cada.
//! ⛔ A cura de um tecto é CORTE, nunca uma entrada no `FILE_OVERAGE_OK`.
use super::*;
use ph2d_mesh_render::Camera3d;

/// O enquadramento das fixturas antigas — a vista inteira, que é o que elas sempre quiseram
/// dizer. ⚠️ Escrito uma vez aqui para o gate do recorte poder ser o CONTRASTE dele.
const CHEIO: ph2d_mesh_render::Framing = ph2d_mesh_render::Framing {
    aspect: 2.0,
    region: ph2d_mesh_render::ViewRegion::FULL,
};

/// **As TRÊS entradas movem o carimbo — e o gate existe porque esquecer uma é invisível.**
///
/// Um carimbo que ignora a câmera deixa a tinta acesa pela forma vista de outro ângulo; um que
/// ignora a malha, pela escultura de antes do traço; um que ignora o tamanho entrega um plano
/// que o Painter recusa e a doação some sem dizer por quê. Nenhum dos três dá erro em lugar
/// nenhum: a tela fica *plausível*.
#[test]
fn every_way_the_form_can_change_moves_the_stamp() {
    let base = Camera3d::default();
    let here = stamp_of(7, &base, (256, 128), CHEIO);
    assert_eq!(
        here,
        stamp_of(7, &base, (256, 128), CHEIO),
        "premissa: é estável"
    );

    assert_ne!(here, stamp_of(8, &base, (256, 128), CHEIO), "a MALHA mudou");
    assert_ne!(
        here,
        stamp_of(7, &base, (512, 128), CHEIO),
        "o CANVAS mudou"
    );

    // Cada campo da câmera, um a um: um carimbo que só olha `yaw` passaria num teste que só
    // gira, e é justamente o `pan` (que move o `target`) o gesto mais fácil de esquecer.
    for (name, mutate) in [
        (
            "yaw",
            (|c: &mut Camera3d| c.yaw += 0.1) as fn(&mut Camera3d),
        ),
        ("pitch", |c| c.pitch += 0.1),
        ("distance", |c| c.distance *= 1.5),
        ("fov_y", |c| c.fov_y += 0.05),
        ("target.x", |c| c.target.x += 1.0),
        ("target.y", |c| c.target.y += 1.0),
        ("target.z", |c| c.target.z += 1.0),
        // ⭐⭐ **A LENTE** (2026-09-21): ela muda a imagem inteira e **nenhum** dos sete campos
        // acima se mexe com ela — sem esta linha a forma viva ficava a descrever a peça vista
        // pela lente de antes, em silêncio, e a mutação que a apagasse do `stamp_of` passaria.
        ("lens", |c| c.lens = c.lens.other()),
    ] {
        let mut moved = base;
        mutate(&mut moved);
        assert_ne!(
            here,
            stamp_of(7, &moved, (256, 128), CHEIO),
            "mexer em `{name}` tem de mover o carimbo"
        );
    }
}

/// ⭐⭐⭐ **O ENQUADRAMENTO move o carimbo — e é a ÚNICA das quatro entradas que um gesto de
/// ASSAR mexe** (report do dono, 2026-09-21: *«depois do bake … o bake 3d recebe zoom»*).
///
/// ⛔⛔ Sem ele, re-assar com o sprite noutro sítio do ecrã deixava a forma viva a mostrar o
/// recorte de antes: a malha não mudou (`edits` parado), a câmera não girou e o canvas tem o
/// mesmo tamanho — *as outras três entradas concordam em dizer «nada mudou», e a imagem é
/// outra*.
///
/// ⚠️ **As DUAS metades**, porque as curas são opostas: o recorte tem de MOVER o carimbo, e a
/// vista inteira tem de continuar a comparar igual a si mesma — senão a doação (que nunca tem
/// recorte) seria re-rasterizada todo quadro, que é o defeito que o gate do `NaN` já guarda.
#[test]
fn o_recorte_move_o_carimbo_e_a_vista_inteira_nao() {
    let cam = Camera3d::default();
    let cheio = stamp_of(7, &cam, (256, 128), CHEIO);
    assert_eq!(
        cheio,
        stamp_of(7, &cam, (256, 128), CHEIO),
        "premissa: a vista inteira é estável"
    );
    // Cada componente do recorte, um a um: um carimbo que só olhasse a ORIGEM ficaria cego a
    // um sprite que mudou de tamanho sem sair do sítio, e vice-versa.
    for (nome, region) in [
        (
            "origem",
            ph2d_mesh_render::ViewRegion {
                origin: [0.25, 0.10],
                size: [1.0, 1.0],
            },
        ),
        (
            "tamanho",
            ph2d_mesh_render::ViewRegion {
                origin: [0.0, 0.0],
                size: [0.5, 0.5],
            },
        ),
    ] {
        let recortado = ph2d_mesh_render::Framing {
            aspect: CHEIO.aspect,
            region,
        };
        assert_ne!(
            cheio,
            stamp_of(7, &cam, (256, 128), recortado),
            "mexer na `{nome}` do recorte tem de mover o carimbo"
        );
    }
    // E o ASPECTO sozinho também: a mesma fracção da vista dentro de uma janela de outra forma
    // desenha outra imagem.
    assert_ne!(
        cheio,
        stamp_of(
            7,
            &cam,
            (256, 128),
            ph2d_mesh_render::Framing {
                aspect: CHEIO.aspect * 2.0,
                region: CHEIO.region,
            }
        ),
        "mexer no aspecto tem de mover o carimbo"
    );
}

/// **Uma câmera degenerada compara igual a si mesma.**
///
/// ⚠️ É o gate do *"por BITS, nunca por valor"*: `NaN != NaN`, então um carimbo que comparasse
/// `f32` por valor nunca diria "nada mudou" e a doação seria re-rasterizada **todo frame, para
/// sempre** — uma leitura de volta bloqueante por quadro, sem nada na tela explicando por quê.
#[test]
fn a_degenerate_camera_still_compares_equal_to_itself() {
    let broken = Camera3d {
        yaw: f32::NAN,
        ..Camera3d::default()
    };
    let s = stamp_of(1, &broken, (64, 64), CHEIO);
    assert_eq!(s, stamp_of(1, &broken, (64, 64), CHEIO));
}

/// **O interruptor CICLA, e cada posição é distinta.**
///
/// Três voltas devolvem ao começo — sem isso o `D` viraria um caminho de mão única e o artista
/// perderia o barro depois de doar uma vez.
#[test]
fn the_switch_cycles_through_all_three_and_comes_back() {
    let mut r = FormRole::Clay;
    let mut seen = Vec::new();
    for _ in 0..3 {
        seen.push(r.label());
        r = r.next();
    }
    assert_eq!(r, FormRole::Clay, "três toques voltam ao barro");
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), 3, "as três posições têm rótulos distintos");
}

/// **Cada posição faz exatamente uma coisa, e `Off` não faz nenhuma.**
///
/// ⚠️ A 1ª versão deste gate afirmava que os variants do enum são distintos entre si — o que o
/// `derive(PartialEq)` garante. Ele **não podia falhar pelo motivo que alegava**, e teria
/// ficado verde com `draws_clay` cravado em `true` (a malha desenhada por cima da tinta em
/// todas as posições, a doação inalcançável). O oráculo tem de ser o COMPORTAMENTO das duas
/// perguntas, não a identidade dos rótulos.
#[test]
fn each_position_answers_exactly_one_of_the_two_questions() {
    assert!(FormRole::Clay.draws_clay() && !FormRole::Clay.donates());
    assert!(FormRole::Light.donates() && !FormRole::Light.draws_clay());
    assert!(
        !FormRole::Off.draws_clay() && !FormRole::Off.donates(),
        "`Off` é o controle: nem barro na tela, nem forma na tinta"
    );
}

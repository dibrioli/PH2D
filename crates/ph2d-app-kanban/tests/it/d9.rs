//! D9 — defeitos vistos nas fotos do tutorial, cada um pela porta real (como os testes de costura) e,
//! onde o defeito é de PIXEL, lido na placa (o `VelloPass` do tutorial).

use crate::verso::{cartao, sem};
use crate::w6::{foca, tecla, toca};
use crate::{AREA, Mesa};
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_editor_core::zones::Rect;
use ph2d_kanban_model::KanbanId;
use ph2d_vector::VectorScene;

/// A aba pintada, lida da placa (RGBA, `AREA`).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn pixeis(m: &mut Mesa) -> Vec<u8> {
    let mut s = VectorScene::new();
    m.pinta(&mut s);
    let (w, h) = (AREA.w as u32, AREA.h as u32);
    let gpu =
        ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).expect("gpu");
    let mut pass = ph2d_render::VelloPass::new(&gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (w, h))
        .expect("pass");
    pass.render_and_readback(&gpu, s.inner(), (w, h))
        .expect("readback")
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn px(img: &[u8], x: f32, y: f32) -> [u8; 4] {
    let i = ((y as usize) * (AREA.w as usize) + x as usize) * 4;
    [img[i], img[i + 1], img[i + 2], img[i + 3]]
}

fn alvo(m: &mut Mesa, id: KanbanId, nome: &str) -> Rect {
    m.docs()
        .probe_w6_hits(id.0)
        .into_iter()
        .rev()
        .find(|(n, _)| n == nome)
        .unwrap_or_else(|| panic!("{nome} não foi pintado"))
        .1
}

/// 4.3 — os botões da edição rápida (`E`) são opacos: os cartões da lista por baixo nunca se vêem
/// através deles. A ponta direita de «Open card» (sobre a lista DOING) tem a MESMA cor que a de
/// «Archive» (sobre o fundo do quadro).
#[test]
fn d9_os_botoes_da_edicao_rapida_tapam_as_listas_por_baixo() {
    let mut m = Mesa::new();
    m.ts = ph2d_text::TextSystem::new();
    let id = ph2d_app_kanban::smoke::stage(&mut m.hero, 5).expect("cena 5");
    m.quadros(2);
    toca(&mut m, id, "ClearAll");
    m.tecla(BoardKey::Escape, sem(), None);
    m.quadro();
    let c = cartao(&m, id, "Design the fox");
    foca(&mut m, id, c);
    tecla(&mut m, 'e');
    m.quadros(4);
    let abrir = alvo(&mut m, id, "Quick(Open)");
    let arquivar = alvo(&mut m, id, "Quick(Archive)");
    let doing = m.lista(id, "Doing");
    let lr = m.docs().probe_list_rect(id.0, doing).expect("a lista DOING");
    assert!(
        abrir.x + abrir.w > lr.x && abrir.y < lr.y + lr.h,
        "o «Open card» está sobre a DOING: {abrir:?} × {lr:?}"
    );
    let img = pixeis(&mut m);
    let ponta = |r: Rect| px(&img, r.x + r.w - 3.0, r.y + r.h / 2.0);
    assert_eq!(ponta(abrir), ponta(arquivar), "a lista vê-se através do botão");
}

//! 📷 J1 — as fotos (`#[ignore]`: a placa). Cada vista DENTRO da aba (`smoke/W7/vistas_no_app_*`) e a
//! partilha (`smoke/W5/`: o «Share» aberto, os avatares e o anel, os dois heróis lado a lado), nos
//! dois temas. `bash scripts/ph2d-run.sh cargo test -p ph2d-app-kanban --test it j1_foto -- --ignored`

use crate::j1_partilha::ate;
use crate::w6::toca;
use crate::{AREA, Mesa, pega};
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_tokens::Theme;
use ph2d_vector::{Affine, VectorScene};

const TEMAS: [(Theme, &str); 2] = [(Theme::Light, "claro"), (Theme::Dark, "escuro")];

fn grava(pasta: &str, nome: &str, (w, h): (u32, u32), scene: &VectorScene) {
    let gpu =
        ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).expect("gpu");
    let mut pass = ph2d_render::VelloPass::new(&gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (w, h))
        .expect("pass");
    let px = pass
        .render_and_readback(&gpu, scene.inner(), (w, h))
        .expect("readback");
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/TrelloClone/smoke")
        .join(pasta);
    std::fs::create_dir_all(&dir).expect("pasta");
    let path = dir.join(format!("{nome}.png"));
    image::save_buffer(&path, &px, w, h, image::ExtendedColorType::Rgba8).expect("png");
    println!("foto: {}", path.display());
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn tamanho() -> (u32, u32) {
    (AREA.w as u32, AREA.h as u32)
}

/// A aba de `m` pintada (duas vezes: a 1.ª assenta o que o desenho mede).
fn cena(m: &mut Mesa, theme: Theme) -> VectorScene {
    m.hero.theme = theme;
    m.ts = ph2d_text::TextSystem::new();
    m.quadro();
    let mut s = VectorScene::new();
    m.pinta(&mut s);
    s
}

#[test]
#[ignore = "a placa: fotos para docs/TrelloClone/smoke/W7/"]
fn j1_foto_cada_vista_dentro_da_aba() {
    for (theme, tag) in TEMAS {
        for (i, nome) in [
            (1, "tabela"),
            (2, "calendario"),
            (3, "linha_do_tempo"),
            (4, "painel"),
        ] {
            let (mut m, id) = Mesa::cena1();
            toca(&mut m, id, "ViewsButton");
            toca(&mut m, id, &format!("View({i})"));
            m.quadros(2);
            let s = cena(&mut m, theme);
            grava("W7", &format!("vistas_no_app_{nome}_{tag}"), tamanho(), &s);
        }
        let (mut m, id) = Mesa::cena1();
        toca(&mut m, id, "ViewsButton");
        let s = cena(&mut m, theme);
        grava("W7", &format!("vistas_no_app_seletor_{tag}"), tamanho(), &s);
    }
}

#[test]
#[ignore = "a placa: fotos para docs/TrelloClone/smoke/W5/"]
fn j1_foto_partilha_dois_herois() {
    for (theme, tag) in TEMAS {
        let (mut a, mut b, id, deles) = crate::j1_partilha::ligados();
        // O «Share» aberto na Ana (o link e o código).
        toca(&mut a, id, "ShareButton");
        let s = cena(&mut a, theme);
        grava("W5", &format!("share_aberto_{tag}"), tamanho(), &s);
        a.tecla(BoardKey::Escape, crate::verso::sem(), None);
        // O cursor da Bia anda no quadro da Ana.
        let alvo = (AREA.w * 0.55, AREA.h * 0.45);
        b.move_(alvo.0, alvo.1);
        ate(&mut a, &mut b, "o cursor da Bia", |a, _| {
            !a.docs().probe_seen(id.0).1.is_empty()
        });
        let s = cena(&mut a, theme);
        grava("W5", &format!("cursor_{tag}"), tamanho(), &s);
        // A Bia abre um cartão: o anel e o avatar dela nele, no ecrã da Ana.
        let backlog = b.lista(deles, "Backlog");
        let c = b.cartoes(deles, backlog)[1];
        let r = b.rect_cartao(deles, c);
        b.clica(pega(r));
        ate(&mut a, &mut b, "o anel do cartão da Bia", |a, _| {
            !a.docs().probe_seen(id.0).0.is_empty()
        });
        let s = cena(&mut a, theme);
        grava("W5", &format!("avatares_{tag}"), tamanho(), &s);
        // Os dois lado a lado.
        let sa = cena(&mut a, theme);
        let sb = cena(&mut b, theme);
        let mut s = VectorScene::new();
        s.append_transformed(&sa, Affine::IDENTITY);
        s.append_transformed(&sb, Affine::translate((f64::from(AREA.w), 0.0)));
        let (w, h) = tamanho();
        grava("W5", &format!("dois_herois_{tag}"), (2 * w, h), &s);
    }
}

//! 📷 Tutorial parte 6 (`#[ignore]`: a placa): o convidado cai («Reconnecting…» na aba dele) e o
//! anfitrião pára de partilhar (o aviso no topo do hero do convidado), nos dois temas.
//! `bash scripts/cargo-test-narrow.sh ph2d-app-kanban j8_foto -- --ignored`

use crate::AREA;
use crate::j1_partilha::ate;
use crate::w6::toca;
use ph2d_editor_core::screens::hero::{HERO_VIEWPORT_H, HERO_VIEWPORT_W, paint_hero_screen};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const TEMAS: [(Theme, &str); 2] = [(Theme::Light, "claro"), (Theme::Dark, "escuro")];

fn grava(nome: &str, (w, h): (u32, u32), scene: &VectorScene) {
    let gpu =
        ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).expect("gpu");
    let mut pass = ph2d_render::VelloPass::new(&gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (w, h))
        .expect("pass");
    let px = pass
        .render_and_readback(&gpu, scene.inner(), (w, h))
        .expect("readback");
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/TrelloClone/smoke/tutorial");
    std::fs::create_dir_all(&dir).expect("pasta");
    let path = dir.join(format!("{nome}.png"));
    image::save_buffer(&path, &px, w, h, image::ExtendedColorType::Rgba8).expect("png");
    println!("foto: {}", path.display());
}

#[test]
#[ignore = "a placa: fotos do tutorial"]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn j8_foto_reconectar_e_parar_de_partilhar() {
    for (theme, tag) in TEMAS {
        let (mut a, mut b, id, deles) = crate::j1_partilha::ligados();
        a.hero.theme = theme;
        b.hero.theme = theme;
        b.ts = TextSystem::new();
        // 6.4b — o fio da Bia cai: a aba dela diz «Reconnecting…».
        b.docs().share_drop_for_tests();
        let mut s = None;
        ate(&mut a, &mut b, "a Bia a voltar", |_, b| {
            if b.docs().probe_reconnecting(deles.0) {
                let mut cena = VectorScene::new();
                b.pinta(&mut cena);
                let mut cena = VectorScene::new();
                b.pinta(&mut cena);
                s = Some(cena);
            }
            s.is_some()
        });
        grava(
            &format!("6_4b_{tag}"),
            (AREA.w as u32, AREA.h as u32),
            &s.expect("pintada"),
        );
        ate(&mut a, &mut b, "a Bia de volta", |_, b| {
            !b.docs().probe_reconnecting(deles.0)
        });
        // 6.4 — a Ana pára: o aviso no topo do ecrã da Bia.
        a.quadros(2);
        if !a
            .docs()
            .probe_w6_hits(id.0)
            .iter()
            .any(|(n, _)| n == "Share(Stop)")
        {
            toca(&mut a, id, "ShareButton");
            a.quadros(2);
        }
        toca(&mut a, id, "Share(Stop)");
        ate(&mut a, &mut b, "a Bia vê o fim", |_, b| {
            b.docs().probe_share(deles.0).0 == "Off"
        });
        ph2d_app_kanban::service(&mut b.hero);
        b.quadros(2);
        assert_eq!(
            b.hero.documents.told(),
            Some(ph2d_i18n::tr("kanban.share.ended")),
            "o aviso do fim no ecrã da Bia"
        );
        let vp = Rect::new(0.0, 0.0, HERO_VIEWPORT_W, HERO_VIEWPORT_H);
        let mut ts = std::mem::replace(&mut b.ts, TextSystem::without_system_fonts());
        let mut s = VectorScene::new();
        paint_hero_screen(&mut b.hero, vp, &mut s, &mut ts);
        let mut s = VectorScene::new();
        paint_hero_screen(&mut b.hero, vp, &mut s, &mut ts);
        grava(
            &format!("6_4a_{tag}"),
            (HERO_VIEWPORT_W as u32, HERO_VIEWPORT_H as u32),
            &s,
        );
    }
}

/// 📷 Tutorial 6 (terminal 2): o convite colado na caixa «Paste an invite link», antes do «Join».
#[test]
#[ignore = "a placa: fotos do tutorial"]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn j8_foto_o_convite_colado() {
    for (theme, tag) in TEMAS {
        let mut a = crate::j1_partilha::com_nome("Ana");
        let id = ph2d_app_kanban::smoke::stage(&mut a.hero, 1).expect("cena 1");
        a.quadros(2);
        let mut b = crate::j1_partilha::com_nome("Bia");
        b.hero.theme = theme;
        b.ts = TextSystem::new();
        b.quadro();
        let vazio = b.docs().create();
        b.quadros(2);
        toca(&mut a, id, "ShareButton");
        toca(&mut a, id, "Share(Start)");
        ate(&mut a, &mut b, "o link da Ana", |a, _| {
            a.docs().probe_share(id.0).1.is_some()
        });
        let link = a.docs().probe_share(id.0).1.expect("link");
        toca(&mut b, vazio, "ShareButton");
        toca(&mut b, vazio, "Share(LinkBox)");
        b.escreve(&link);
        b.quadros(2);
        let mut s = VectorScene::new();
        b.pinta(&mut s);
        let mut s = VectorScene::new();
        b.pinta(&mut s);
        grava(&format!("6_2_{tag}"), (AREA.w as u32, AREA.h as u32), &s);
    }
}

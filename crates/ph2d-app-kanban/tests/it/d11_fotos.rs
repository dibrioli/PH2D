//! 📷 D11 — as fotos do fundo (`#[ignore]`: a placa) em `docs/TrelloClone/smoke/D11/`, nos dois temas:
//! a gaveta aberta com um padrão escolhido, e o quadro com cor, padrão, imagem e um fundo claro.
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-app-kanban --profile smoke --test it d11_foto -- --ignored`

use crate::Mesa;
use crate::d11_fundo::{abre, foto, sobe, toca};
use crate::j2_fotos::grava_em;
use crate::verso_anexos::ficheiro;
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_host::Modifiers;
use ph2d_kanban_model::KanbanId;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

fn cena(m: &mut Mesa) -> VectorScene {
    m.quadro();
    let mut s = VectorScene::new();
    m.pinta(&mut s);
    s
}

fn fecha(m: &mut Mesa) {
    m.tecla(BoardKey::Escape, Modifiers::default(), None);
    m.quadro();
}

fn cena1(theme: Theme) -> (Mesa, KanbanId) {
    let mut m = Mesa::new();
    m.hero.theme = theme;
    m.ts = ph2d_text::TextSystem::new();
    let id = ph2d_app_kanban::smoke::stage(&mut m.hero, 1).expect("cena 1");
    m.quadros(2);
    (m, id)
}

#[test]
#[ignore = "a placa: fotos para docs/TrelloClone/smoke/D11/"]
fn d11_foto_gaveta_e_fundos() {
    for (theme, tag) in [(Theme::Light, "claro"), (Theme::Dark, "escuro")] {
        let (mut m, id) = cena1(theme);
        abre(&mut m, id);
        toca(&mut m, id, "Pattern(Dots)");
        toca(&mut m, id, "Tint(Base, Purple)");
        toca(&mut m, id, "Tint(Ink, Pink)");
        grava_em("D11", &format!("gaveta_padrao_{tag}"), &cena(&mut m));
        toca(&mut m, id, "Custom(Ink)");
        grava_em("D11", &format!("gaveta_cor_livre_{tag}"), &cena(&mut m));
        fecha(&mut m);
        grava_em("D11", &format!("quadro_padrao_pontos_{tag}"), &cena(&mut m));
        for (alvo, nome) in [
            ("Pattern(Diagonal)", "quadro_padrao_diagonal"),
            ("Pattern(Waves)", "quadro_padrao_ondas"),
        ] {
            abre(&mut m, id);
            toca(&mut m, id, alvo);
            fecha(&mut m);
            grava_em("D11", &format!("{nome}_{tag}"), &cena(&mut m));
        }
        for (alvo, nome) in [
            ("Color(Green)", "quadro_cor"),
            ("Gradient(0)", "quadro_gradiente_neve"),
            ("Gradient(1)", "quadro_gradiente_oceano"),
        ] {
            abre(&mut m, id);
            toca(&mut m, id, alvo);
            fecha(&mut m);
            grava_em("D11", &format!("{nome}_{tag}"), &cena(&mut m));
        }
        abre(&mut m, id);
        let avisos = sobe(&mut m, id, ficheiro("d11f", "ceu.png", &foto(1600, 1000)));
        assert!(avisos.is_empty(), "{avisos:?}");
        // A imagem descodifica-se num fio de fundo: dá-lhe tempo.
        for _ in 0..60 {
            std::thread::sleep(std::time::Duration::from_millis(20));
            m.quadro();
        }
        grava_em("D11", &format!("gaveta_imagem_{tag}"), &cena(&mut m));
        fecha(&mut m);
        grava_em("D11", &format!("quadro_imagem_{tag}"), &cena(&mut m));
    }
}

/// ⚠️ MEDIÇÃO (D11): o quadro inteiro (1200×700) na placa com cada fundo, intercalados, o MÍNIMO de
/// 15 voltas (`render_and_readback`: a leitura de volta é a mesma em todos). Imprime ms.
#[test]
#[ignore = "a placa: medição"]
fn d11_mede_o_fundo_na_placa() {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (w, h) = (crate::AREA.w as u32, crate::AREA.h as u32);
    let gpu =
        ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).expect("gpu");
    let mut pass = ph2d_render::VelloPass::new(&gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (w, h))
        .expect("pass");
    let (mut m, id) = cena1(Theme::Light);
    let mut cenas = Vec::new();
    for alvo in [
        "Color(Blue)",
        "Pattern(Dots)",
        "Pattern(Checks)",
        "Pattern(Waves)",
        "Pattern(CrossHatch)",
    ] {
        abre(&mut m, id);
        toca(&mut m, id, alvo);
        fecha(&mut m);
        cenas.push((alvo, cena(&mut m)));
    }
    let mut best = vec![f64::MAX; cenas.len()];
    for _ in 0..15 {
        for (i, (_, s)) in cenas.iter().enumerate() {
            let t0 = std::time::Instant::now();
            pass.render_and_readback(&gpu, s.inner(), (w, h))
                .expect("readback");
            best[i] = best[i].min(t0.elapsed().as_secs_f64() * 1e3);
        }
    }
    for ((alvo, _), ms) in cenas.iter().zip(best) {
        println!("{alvo}: {ms:.2} ms");
    }
}

/// 📷 Smoke do dono 28/10: «na timeline, a depender do fundo, as palavras ficam ilegíveis» — as
/// quatro vistas sobre um fundo CLARO (o gradiente «neve» e as ondas sobre azul-céu), nos dois temas.
#[test]
#[ignore = "a placa: fotos para docs/TrelloClone/smoke/D11/"]
fn d11_foto_vistas_sobre_fundo_claro() {
    for (theme, tag) in [(Theme::Light, "claro"), (Theme::Dark, "escuro")] {
        for (fundo, alvos) in [
            ("neve", &["Gradient(0)"][..]),
            ("ondas", &["Pattern(Waves)", "Tint(Base, Sky)"][..]),
        ] {
            let (mut m, id) = cena1(theme);
            abre(&mut m, id);
            for a in alvos {
                toca(&mut m, id, a);
            }
            fecha(&mut m);
            for (i, nome) in [
                (1, "tabela"),
                (2, "calendario"),
                (3, "linha_do_tempo"),
                (4, "painel"),
            ] {
                crate::j1_vistas::vista(&mut m, id, i);
                grava_em("D11", &format!("vista_{nome}_{fundo}_{tag}"), &cena(&mut m));
            }
        }
    }
}

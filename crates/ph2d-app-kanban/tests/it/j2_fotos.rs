//! 📷 J2 — as fotos (`#[ignore]`: a placa) em `docs/TrelloClone/smoke/W8/`, nos dois temas: o verso
//! da cena 8 (botões do Butler e ligações ao projecto), o painel de automação (regras e a sugestão)
//! e o construtor em frase com o menu de um chip aberto.
//! `bash scripts/ph2d-run.sh cargo test -p ph2d-app-kanban --test it j2_foto -- --ignored`

use crate::j2_auto::{toca_auto, toca_v};
use crate::{AREA, Mesa};
use ph2d_kanban_model::KanbanId;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

fn grava(nome: &str, scene: &VectorScene) {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (w, h) = (AREA.w as u32, AREA.h as u32);
    let gpu =
        ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None).expect("gpu");
    let mut pass = ph2d_render::VelloPass::new(&gpu, ph2d_gpu::TextureFormat::Rgba8Unorm, (w, h))
        .expect("pass");
    let px = pass
        .render_and_readback(&gpu, scene.inner(), (w, h))
        .expect("readback");
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/TrelloClone/smoke/W8");
    std::fs::create_dir_all(&dir).expect("pasta");
    let path = dir.join(format!("{nome}.png"));
    image::save_buffer(&path, &px, w, h, image::ExtendedColorType::Rgba8).expect("png");
    println!("foto: {}", path.display());
}

fn cena(m: &mut Mesa) -> VectorScene {
    m.quadro();
    let mut s = VectorScene::new();
    m.pinta(&mut s);
    s
}

fn cena8(theme: Theme) -> (Mesa, KanbanId) {
    let mut m = Mesa::new();
    m.hero.theme = theme;
    m.ts = ph2d_text::TextSystem::new();
    let id = ph2d_app_kanban::smoke::stage(&mut m.hero, 8).expect("cena 8");
    m.quadros(2);
    (m, id)
}

#[test]
#[ignore = "a placa: fotos para docs/TrelloClone/smoke/W8/"]
fn j2_foto_verso_painel_e_construtor() {
    for (theme, tag) in [(Theme::Light, "claro"), (Theme::Dark, "escuro")] {
        let (mut m, id) = cena8(theme);
        // O verso rolado até às ligações e aos botões.
        for _ in 0..8 {
            m.roda(AREA.w / 2.0, AREA.h / 2.0, -16.0);
        }
        let s = cena(&mut m);
        grava(&format!("verso_botoes_e_ligacoes_{tag}"), &s);
        // O painel nas regras: duas regras e a sugestão.
        toca_v(&mut m, id, "Side(AddButton)");
        toca_auto(&mut m, id, "Auto(Tab(Rule))");
        let s = cena(&mut m);
        grava(&format!("painel_regras_{tag}"), &s);
        // O construtor: editar uma regra, com o menu da acção aberto.
        toca_auto(&mut m, id, "Auto(Edit(");
        toca_auto(&mut m, id, "Auto(Chip(Head(Action(0))))");
        let s = cena(&mut m);
        grava(&format!("construtor_{tag}"), &s);
    }
}

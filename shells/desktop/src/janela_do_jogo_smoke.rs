//! As fotos da câmera e da Sprite do smoke do jogo (`camera_chave`, `camera_desfaz`, onda 18): a shell só executa
//! o que `ph2d_app_game::smoke_janela` lê do mundo e decide.

use ph2d_app_game::smoke_janela as j;

impl crate::App {
    /// Empurra as ações para o barramento do herói.
    fn empurra_do_smoke(&mut self, acoes: Vec<ph2d_editor_core::action_bus::EditorAction>) {
        if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            acoes.into_iter().for_each(|a| h.bus.push(a));
        }
    }

    /// Os gestos da mão que não são o ponteiro: desfazer, agulha, as ações do Inspector e os relatos.
    pub(super) fn fim_da_imagem_do_smoke(&mut self, m: ph2d_app_game::smoke_mao::Mao) {
        use ph2d_app_game::smoke_mao::Mao;
        self.any_input_this_frame = true;
        match m {
            Mao::Desfaz => self.smoke_undo(false),
            Mao::Agulha(t) => self
                .timeline_intents
                .push(ph2d_timeline::TimelineIntent::Scrub(t)),
            Mao::Prova => {
                let (t, desfazer) = (self.playhead.time(), self.undo.depth());
                let tl = &self.timeline;
                let linha = self
                    .gfx
                    .as_mut()
                    .and_then(|g| j::prova_da_sprite(&mut g.sim, tl, t, desfazer));
                linha.into_iter().for_each(|l| eprintln!("{l}"));
            }
            Mao::SaiDoDraw | Mao::Aplica | Mao::Desloca | Mao::Pivo(_) => {
                let acoes = self
                    .gfx
                    .as_mut()
                    .map_or_else(Vec::new, |g| j::acoes_da_mao(&mut g.sim, m));
                self.empurra_do_smoke(acoes);
            }
            _ => {
                let Some(g) = self.gfx.as_mut() else { return };
                for l in j::imagens_das_plataformas(&mut g.sim, &g.atlas_asset_map) {
                    eprintln!("{l}");
                }
                eprintln!(
                    "[sprite-pixel] passos de desfazer: {} · Pixel Sprites: {} · marca do Painter: {}",
                    self.undo.depth(),
                    ph2d_app_pixel::mode::sprites(g.sim.world()).len(),
                    ph2d_sprite_edit::imagem::edicao_marcada(),
                );
            }
        }
    }

    pub(super) fn formato_desfaz_do_smoke(&mut self, quadro: u32) {
        let p = j::passo_do_formato(quadro);
        let heroi = self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref());
        if let (Some(quando), Some(h)) = (p.relata, heroi) {
            eprintln!(
                "[camera-desfaz] {quando}: {:?}",
                h.project.game_format.preset
            );
        }
        if p.entrada {
            self.any_input_this_frame = true;
        }
        if p.troca {
            let acao = self
                .gfx
                .as_mut()
                .and_then(|g| j::troca_para_game_boy(&mut g.sim));
            self.empurra_do_smoke(acao.into_iter().collect());
        }
        if let Some(refaz) = p.desfaz {
            self.smoke_undo(refaz);
        }
    }
}

//! O condutor da cena 5 do jogo (`PH2D_GAME_SMOKE=5`, ficha 02 E6): cada objecto vestido pelas portas dos gestos do
//! produto, a foto pedida, e as provas do Play e do Ctrl+Z reais, que relatam em `[animado-prova]`.

use ph2d_app_game::smoke::animado::{self as a, FotoAnimada};
use ph2d_editor_core::AnimFieldEdit as Ed;
use ph2d_editor_core::ImageUseEdit as Verbo;
use ph2d_editor_core::action_bus::EditorAction;
use std::sync::atomic::{AtomicU32, Ordering};

static QUADRO: AtomicU32 = AtomicU32::new(0);
/// As fileiras com desenho da folha, medidas ao vestir: o Split da prova é o mesmo da cena.
static FILEIRAS: std::sync::OnceLock<Vec<Vec<u32>>> = std::sync::OnceLock::new();
/// O índice de «row 1» na lista: a folha entra com o «loop» e o Split junta depois dele.
const ROW_1: u8 = 1;
/// O quadro da animação onde o marcador grita.
const QUADRO_DO_GRITO: u32 = 2;

impl crate::App {
    fn bits_de(&mut self, nome: &str) -> Option<u64> {
        let g = self.gfx.as_mut()?;
        ph2d_app_game::smoke::por_nome(&mut g.sim, nome).map(ph2d_ecs::Entity::to_bits)
    }

    fn edita_animacao(&mut self, nome: &str, edit: Ed) {
        let bits = self.bits_de(nome);
        if let (Some(entity_bits), Some(h)) =
            (bits, self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()))
        {
            h.bus
                .push(EditorAction::InspectorAnimEdit { entity_bits, edit });
        }
    }

    /// As células com desenho de cada fileira de `nome`: o que o botão Split by rows lê da folha pintada.
    fn fileiras_de(&mut self, nome: &str, id: ph2d_asset::AssetId) -> Option<Vec<Vec<u32>>> {
        let bits = self.bits_de(nome)?;
        let g = self.gfx.as_ref()?;
        let imagem = g.asset_db.get(&id)?;
        let (w, h, px) = imagem.image_rgba8()?;
        let grade = *g
            .sim
            .world()
            .get::<ph2d_ecs::SpriteGrid>(ph2d_ecs::Entity::from_bits(bits))?;
        Some(ph2d_app_sprite_anim::editor::split::drawn_by_row(
            &px,
            w,
            h,
            grade.hframes,
            grade.vframes,
        ))
    }

    /// A folha, o herói de amostra e a imagem trocada: os gestos que escrevem logo no mundo.
    pub(crate) fn veste_o_animado_do_smoke(&mut self, ids: &[ph2d_asset::AssetId]) {
        let [heroi, montes] = ids else {
            return;
        };
        let foto = a::foto_animada();
        for nome in [a::OUVE, a::INICIO, a::TROCADA] {
            if let Some(b) = self.bits_de(nome) {
                self.image_use_edit(b, Verbo::SampleHero);
            }
        }
        if let Some(b) = self.bits_de(a::FOLHA) {
            self.choose_image(b, *heroi.as_bytes());
        }
        if let Some(b) = self.bits_de(a::TROCADA) {
            self.choose_image(b, *montes.as_bytes());
        }
        if let (Some(b), Some(g)) = (self.bits_de(a::OUVE), self.gfx.as_mut()) {
            a::ouve_o_marcador(&mut g.sim, ph2d_ecs::Entity::from_bits(b));
        }
        let fileiras = self.fileiras_de(a::FOLHA, *heroi).unwrap_or_default();
        let _ = FILEIRAS.set(fileiras.clone());
        if foto == Some(FotoAnimada::ProvaDesfaz) {
            return;
        }
        self.edita_animacao(a::FOLHA, Ed::SplitByRows(fileiras));
        self.edita_animacao(a::INICIO, Ed::StartsWith("run".into()));
    }

    /// Um por quadro, na fase das cenas: o resto dos gestos, a foto e as provas.
    pub(crate) fn animado_do_smoke(&mut self) {
        if ph2d_app_game::smoke::armed_level() != Some(a::CENA) {
            return;
        }
        let q = QUADRO.fetch_add(1, Ordering::Relaxed);
        let foto = a::foto_animada();
        match q {
            5 if foto != Some(FotoAnimada::ProvaDesfaz) => {
                self.edita_animacao(a::FOLHA, Ed::SetCurrent("row 1".into()));
                self.edita_animacao(a::FOLHA, Ed::StartsWith("row 1".into()));
                self.edita_animacao(a::FOLHA, Ed::MarkerAdd(ROW_1));
                self.edita_animacao(a::INICIO, Ed::SetCurrent("jump".into()));
            }
            8 if foto != Some(FotoAnimada::ProvaDesfaz) => {
                self.edita_animacao(a::FOLHA, Ed::MarkerStep(ROW_1, 0, QUADRO_DO_GRITO));
                self.edita_animacao(a::FOLHA, Ed::MarkerSignal(ROW_1, 0, a::SINAL.into()));
            }
            12 => self.foto_do_animado(foto),
            _ => {}
        }
        match foto {
            Some(FotoAnimada::ProvaPlay) => self.prova_do_play(q),
            Some(FotoAnimada::ProvaDesfaz) => self.prova_do_desfazer(q),
            Some(FotoAnimada::RegraAcesa) if q.is_multiple_of(30) => {
                let acesa = self.bits_de("Player").and_then(|b| {
                    let g = self.gfx.as_ref()?;
                    ph2d_app_game::personagem::regra_acesa(
                        g.sim.world(),
                        b,
                        self.playhead.is_playing(),
                    )
                });
                eprintln!("[animado-prova] q={q} regra acesa do Player: {acesa:?}");
            }
            _ => {}
        }
    }

    fn foto_do_animado(&mut self, foto: Option<FotoAnimada>) {
        let alvo = foto.map_or("Player", FotoAnimada::alvo);
        let bits = self.bits_de(alvo);
        let Some(g) = self.gfx.as_mut() else {
            return;
        };
        (g.camera.center, g.camera.height_world) = (a::CENTRO, a::ALTURA);
        let Some(h) = g.hero_screen.as_mut() else {
            return;
        };
        h.gizmo.replace_selection(bits);
        if foto == Some(FotoAnimada::More) {
            h.store
                .set_collapsed(ph2d_panel_inspector::ids::INSP_ANIM_ADVANCED, false);
        }
        if foto == Some(FotoAnimada::Estreito) {
            h.store.set_dock_width(
                ph2d_editor_core::screens::layout::DockSide::Right,
                Some(0.0),
            );
        }
        if foto == Some(FotoAnimada::Editor)
            && let Some(entity_bits) = bits
        {
            h.bus.push(EditorAction::InspectorAnimEdit {
                entity_bits,
                edit: Ed::OpenEditor,
            });
        }
    }

    fn relata_o_animado(&mut self, quando: &str, nome: &str) {
        if let Some(g) = self.gfx.as_mut() {
            eprintln!("[animado-prova] {quando}: {}", a::relato(&mut g.sim, nome));
        }
    }

    /// ▶ noutra animação, depois o condutor joga (quadro 60) e pára (1500): o `[jogo]` compara os documentos.
    fn prova_do_play(&mut self, q: u32) {
        match q {
            40 => {
                self.edita_animacao(a::INICIO, Ed::SetCurrent("crouch".into()));
                self.edita_animacao(a::INICIO, Ed::Playing(true));
            }
            50 => self.relata_o_animado("depois do play da linha", a::INICIO),
            90 | 300 | 1490 => self.relata_o_animado("no jogo", a::INICIO),
            1520 => self.relata_o_animado("depois do Stop", a::INICIO),
            _ => {}
        }
    }

    /// Cada gesto, a entrada do artista que fecha o passo, um Ctrl+Z, e o estado a comparar com o de antes.
    fn prova_do_desfazer(&mut self, q: u32) {
        const INTERVALO: u32 = 40;
        const INICIO: u32 = 40;
        let Some(d) = q.checked_sub(INICIO) else {
            return;
        };
        let (passo, fase) = (d / INTERVALO, d % INTERVALO);
        // Cut again sem proposta abre a aba, e Keep the cut corre com ela à frente.
        let (titulo, nome) = match passo {
            0 => ("Split by rows, com a aba aberta", a::FOLHA),
            1 => ("Try with the sample hero", a::VAZIO),
            2 => ("Cut again, sem proposta", a::TROCADA),
            3 => ("Keep the cut", a::TROCADA),
            _ => return,
        };
        match fase {
            0 => {
                self.relata_o_animado(&format!("{titulo}, antes"), nome);
                self.any_input_this_frame = true;
                self.gesto_da_prova(passo, nome);
            }
            1..=3 => self.any_input_this_frame = true,
            10 => {
                self.relata_o_animado(&format!("{titulo}, feito"), nome);
                self.smoke_undo(false);
            }
            20 => self.relata_o_animado(&format!("{titulo}, 1 Ctrl+Z"), nome),
            _ => {}
        }
    }

    fn gesto_da_prova(&mut self, passo: u32, nome: &str) {
        let Some(b) = self.bits_de(nome) else {
            return;
        };
        match passo {
            0 => {
                self.edita_animacao(nome, Ed::OpenEditor);
                let fileiras = FILEIRAS.get().cloned().unwrap_or_default();
                self.edita_animacao(nome, Ed::SplitByRows(fileiras));
            }
            1 => self.image_use_edit(b, Verbo::SampleHero),
            2 => self.image_use_edit(b, Verbo::CutAgain),
            _ => {
                self.image_use_edit(b, Verbo::KeepCut);
            }
        }
    }
}

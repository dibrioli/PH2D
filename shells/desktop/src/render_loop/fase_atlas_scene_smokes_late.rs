//! **Fase do quadro: AS CENAS DE SMOKE QUE PRECISAM DO ATLAS — a 2.ª metade** — a máscara do Painter, a
//! folha como objeto, a família `PH2D_GPU_COOK_DEMO` a tomar a ferramenta Motion e a sujidade na lente
//! (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **A cura do `PH2D_GPU_COOK_DEMO` mora AQUI** (o `if` que dá a ferramenta Motion às cenas da família):
//! o gate `ph2d_app_motion::motion_state_demo_router_tests` lê este ficheiro por `include_str!`.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_atlas_scene_smokes_late(&mut self) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            renderer,
            sim,
            asset_db,
            toasts,
            tools,
            vec_scene,
            hero_screen,
            next_import_cell,
            atlas_asset_map,
            motion,
            camera,
            ..
        } = FrameGfx::of(gfx);

        // Mask smoke (`PH2D_MASK_SMOKE=1`): the same dance for the mask coverage law (doc 25 §13.9).
        // Nothing but the canvas is staged — the artist picks the rail chip, so the scene shows the
        // shipped default mask brush rather than a rigged one.
        if let Some(hero) = hero_screen.as_mut()
            && ph2d_app_painter::mask_smoke::enabled()
            && !std::mem::replace(&mut self.mask_smoke_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            let cell = *next_import_cell;
            if let Some(bits) = ph2d_app_painter::mask_smoke::spawn_if_enabled(
                sim,
                renderer,
                asset_db,
                cell,
                ppm,
                atlas_asset_map,
            ) {
                *next_import_cell = next_import_cell.saturating_add(1);
                hero.gizmo.replace_selection(Some(bits));
                hero.bus
                    .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                        kind: ph2d_editor_core::ViewFocusKind::Selected,
                    });
                toasts.push(Toast::success(
                    "Mask smoke: paint some art, then the MASK chip — and SCRUB".to_string(),
                ));
            }
        }

        // ⭐⭐⭐ **O CANVAS DO PAINTER PRESO A OSSOS** (`PH2D_VEC_BONE_PAINT_SMOKE=1`) — a cena que
        // faltava à cura das guias chatas (item 4 do dono): sem ela a correcção estava gateada e
        // **invisível**, e uma cura que ninguém pode ver é uma cura que ninguém julga.
        //
        // ⚠️ Mesma dança da máscara ao lado, e pela mesma razão: o canvas fica SELECCIONADO (é ele o
        // sujeito do Painter) e nada mais é armado — a ferramenta, o pincel e a grelha são do artista.
        if let Some(hero) = hero_screen.as_mut()
            && ph2d_app_vec::smoke_bone_paint::armed()
            && !std::mem::replace(&mut self.vec.bone_paint_smoke_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            let cell = *next_import_cell;
            // ⚠️ **O `n` é do ROTEADOR e não desta fase**: a cena monta `n` canvas e diz quantos
            // de facto montou — as células do atlas avançam por esse número, senão a cena seguinte
            // sobrescreve a tinta desta.
            if let Some((bits, quantos)) = ph2d_app_vec::smoke_bone_paint::build(
                sim,
                renderer,
                asset_db,
                cell,
                ppm,
                atlas_asset_map,
            ) {
                *next_import_cell = next_import_cell.saturating_add(quantos);
                hero.gizmo.replace_selection(Some(bits));
                // ⛔⛔ **`Selected` mesmo na cena LOTADA, e o `All` foi construído, FOTOGRAFADO e
                // REVERTIDO:** ele ajusta-se às CAIXAS das sprites e a arte dobrada varre para fora
                // delas, logo os canvas das pontas saíam cortados de qualquer maneira — e, mais
                // importante, *enquadrar a fileira inteira é a pergunta errada*: os outros canvas
                // existem para ENCHER o orçamento do quadro, não para serem vistos ao mesmo tempo.
                // O que o dono julga é a junta do canvas à frente dele.
                hero.bus
                    .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                        kind: ph2d_editor_core::ViewFocusKind::Selected,
                    });
                // ⚠️⚠️ **Este texto nomeava a fileira `Deform`, que foi APAGADA em 2026-09-17** —
                // *um passo de smoke que nomeia um controlo AFIRMA que ele está na tela*, e o dono
                // aprova o smoke com o passo impossível dentro.
                toasts.push(Toast::success(if quantos > 1 {
                    "Bone-paint smoke: a cena LOTADA — todos os canvas dobram com a mesma malha fina"
                        .to_string()
                } else {
                    "Bone-paint smoke: pegue o Painter e desenhe uma forma sobre o canvas dobrado"
                        .to_string()
                }));
            }
        }

        // ⭐ **AS TRÊS MÍDIAS** (F11) — o corpo vive à parte pelo tecto de LOC por FUNÇÃO.
        if bone_media_smoke(
            &mut self.vec.bone_media_smoke_done,
            sim,
            renderer,
            asset_db,
            hero_screen,
            next_import_cell,
            atlas_asset_map,
            toasts,
        ) {
            bone_media_prologo(hero_screen, &mut self.timeline_intents, &mut self.playhead);
        }

        // **A FOLHA COMO OBJETO** (`PH2D_SHEET_SMOKE=1`, plano `docs/Sprite_projeto/17` §7): cinco
        // peças de tamanhos diferentes entram, e sai UM objeto — um retângulo na hierarquia, com
        // as peças arranjadas dentro como filhos.
        //
        // ⚠️ A folha fica SELECIONADA de propósito: é ela que o artista tem de conseguir mover,
        // redimensionar, esconder e duplicar, e nenhuma dessas coisas tem código próprio — a
        // seleção é o convite a verificá-lo.
        if let Some(hero) = hero_screen.as_mut()
            && crate::sheet_smoke::enabled()
            && !std::mem::replace(&mut self.sheet_smoke_done, true)
        {
            let ppm = hero.project.pixels_per_meter;
            if let Some((sheet, n)) = crate::sheet_smoke::spawn_if_enabled(
                sim,
                renderer,
                asset_db,
                vec_scene,
                &mut self.vec.entities,
                next_import_cell,
                ppm,
                atlas_asset_map,
            ) {
                hero.gizmo.replace_selection(Some(sheet));
                hero.bus
                    .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                        kind: ph2d_editor_core::ViewFocusKind::Selected,
                    });
                toasts.push(Toast::success(format!(
                    "Sheet smoke: {n} pieces packed into one object — move it, resize it, hide it"
                )));
                self.title_dirty = true;
            }
        }

        // **A FAMÍLIA `PH2D_GPU_COOK_DEMO` PRECISA DA FERRAMENTA MOTION** — ver
        // `motion_state_demo_router::demo_wants_the_motion_tool`, onde está a medição que o
        // expôs. Sem isto a cena monta, a legenda imprime, e a tela fica VAZIA.
        if ph2d_app_motion::motion_state::demo_router::demo_wants_the_motion_tool(
            motion.sinks.len(),
        ) && !std::mem::replace(&mut self.demo_tool_forced, true)
        {
            // ⚠️ **O resultado é GUARDADO e o latch só queima se a troca deu certo.** O
            // `set_active` devolve `false` quando o id não está registado, e um `let _ =` com o
            // latch já queimado seria uma falha silenciosa e DEFINITIVA na sessão. Os irmãos do
            // Flip já o guardavam (`flip_hardness_smoke.rs`); este não.
            let ok = tools.set_active(&ph2d_editor_core::ToolId::new("motion"));
            if ok {
                self.title_dirty = true;
                // A porta de bissecção do zoom (`PH2D_DEMO_ALTURA`) — ver o doc dela.
                if let Some(h) = ph2d_app_motion::motion_demo_altura::altura_semeada() {
                    camera.height_world = h;
                }
            } else {
                self.demo_tool_forced = false;
                eprintln!(
                    "[demo] a ferramenta `motion` nao esta' registada — a cena nao vai desenhar"
                );
            }
        }

        // **A SUJIDADE NA LENTE** (`PH2D_GLOW_DIRT_SMOKE=1`, doc 89 folha 11): uma sprite com
        // uma imagem de pó e riscos, um campo de peças a brilhar, e o nó `Glow` já a ler a
        // primeira. ⚠️ Ela mora AQUI e não entre os demos de grafo porque precisa de uma
        // textura a sério — a mesma razão que já está escrita para o `PH2D_MOTION_OBJ_SMOKE=9`.
        if ph2d_app_motion::glow_dirt_smoke::enabled()
            && !std::mem::replace(&mut self.glow_dirt_smoke_done, true)
            && ph2d_app_motion::glow_dirt_smoke::spawn_if_enabled(
                sim,
                renderer,
                asset_db,
                next_import_cell,
                atlas_asset_map,
                motion,
            )
        {
            let _ = tools.set_active(&ph2d_editor_core::ToolId::new("motion"));
            self.title_dirty = true;
        }
    }
}

/// ⭐⭐⭐ **AS TRÊS MÍDIAS PRESAS AO MESMO GESTO** (`PH2D_VEC_BONE_MEDIA_SMOKE=1`, F11) — a cena que
/// torna JULGÁVEL o que a wave curou: a folha de quadros dobrava **errado e em silêncio**, e o
/// 9-slice não dobrava de todo.
///
/// ⚠️ **O enquadramento é `All` e não `Selected`**, ao contrário do irmão da pintura: aqui as três
/// TÊM de ser vistas ao mesmo tempo — *sem o controlo ao lado, «dobrou» e «dobrou certo» leem-se
/// igual*.
///
/// ⚠️ **Função LIVRE e não um método**, e a razão é o empréstimo: os argumentos saem todos do
/// `FrameGfx::of(gfx)`, que já tem a `App` emprestada mutavelmente — um `&mut self` aqui não compila.
/// O latch entra como `&mut bool` pela mesma razão.
/// ⭐⭐⭐ **O PRÓLOGO DA CENA DE MÍDIA, e a DECISÃO dele não mora aqui.** Quais destas quatro coisas
/// cada nível precisa é lei da CENA ([`ph2d_app_vec::smoke_bone_media::prologo_do_nivel`], gateada
/// sem janela nenhuma); o que mora na shell é o **EFEITO**, porque abrir um painel, escrever um
/// intent da timeline e parar o relógio são três coisas da `App`.
///
/// *O molde é o da física: o que sai são os CORPOS; o que decide a ordem do quadro fica.*
///
/// ⚠️ **Função livre e não método**, pela razão de sempre neste ficheiro: o `gfx` já está emprestado
/// ao `hero_screen`, e um `&mut self` não compila. Os campos que ela recebe são disjuntos dele.
fn bone_media_prologo(
    hero_screen: &mut Option<ph2d_editor_core::HeroScreen>,
    timeline_intents: &mut Vec<ph2d_timeline::TimelineIntent>,
    playhead: &mut ph2d_core::Playhead,
) {
    let pro =
        ph2d_app_vec::smoke_bone_media::prologo_do_nivel(ph2d_app_vec::smoke_bone_media::nivel());
    if let Some(hero) = hero_screen.as_mut() {
        if pro.timeline_aberta {
            <_ as ph2d_editor_core::panel::PanelHostInternal>::set_panel_visible(
                hero,
                <ph2d_panel_timeline::TimelinePanel as ph2d_editor_core::panel::Panel>::ID,
                true,
            );
        }
        if pro.enquadrar {
            hero.bus
                .push(ph2d_editor_core::action_bus::EditorAction::SetViewFocus {
                    kind: ph2d_editor_core::ViewFocusKind::All,
                });
        }
    }
    if pro.auto_key {
        timeline_intents.push(ph2d_timeline::TimelineIntent::SetAutoKey(true));
    }
    // ⚠️ **O relógio NASCE A ANDAR** (`Playhead::new` põe `playing: true`, medido): uma cena de
    // autoria que não o pare grava a pose num instante que já passou.
    if pro.relogio_parado {
        playhead.pause();
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "sao as portas do quadro que a cena precisa, e agrupa-las numa struct so' esconderia \
              que elas vem todas do mesmo emprestimo"
)]
/// Devolve **`true` no quadro em que a cena foi MONTADA** — é esse o sinal por que o chamador
/// arma o prólogo dela (a timeline, o AutoKey, o relógio), e é `&mut self` que ele precisa e esta
/// função não tem: o `gfx` já está emprestado.
fn bone_media_smoke(
    feito: &mut bool,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &ph2d_asset::AssetDb,
    hero_screen: &mut Option<ph2d_editor_core::HeroScreen>,
    next_import_cell: &mut u32,
    atlas_asset_map: &mut std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
    toasts: &mut ph2d_editor_core::ToastQueue,
) -> bool {
    let Some(hero) = hero_screen.as_mut() else {
        return false;
    };
    if !ph2d_app_vec::smoke_bone_media::armed() || std::mem::replace(feito, true) {
        return false;
    }
    let ppm = hero.project.pixels_per_meter;
    let cell = *next_import_cell;
    let Some((bits, gastas)) =
        ph2d_app_vec::smoke_bone_media::build(sim, renderer, asset_db, cell, ppm, atlas_asset_map)
    else {
        return false;
    };
    *next_import_cell = next_import_cell.saturating_add(gastas);
    hero.gizmo.replace_selection(Some(bits));
    // ⚠️ **O enquadramento NÃO se pede aqui** — ele é decisão do prólogo
    // (`smoke_bone_media::Prologo::enquadrar`), porque a cena que abre a timeline não o pode pedir:
    // o `Frame All` ajusta à JANELA e os painéis tapam-lhe as bordas. A razão medida vive no doc
    // daquele campo.
    toasts.push(Toast::success(
        match ph2d_app_vec::smoke_bone_media::nivel() {
            2 => "Bone-media smoke: TESTE NULO — os tres bracos tem de dobrar IGUAL",
            3 => "Bone-media smoke: escolha 'Bone 3' na Hierarquia e ARRASTE — a pose e' gravada",
            _ => "Bone-media smoke: as tres dobram — a do meio mostra UM quadro, a de baixo mantem os cantos",
        }
        .to_string(),
    ));
    true
}

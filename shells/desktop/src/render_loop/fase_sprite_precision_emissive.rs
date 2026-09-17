//! **Fase do quadro: PRECISÃO, EMISSÃO E SAIR DA FOLHA** — três edições de uma sprite que precisam do `sim`
//! emprestado mutavelmente (OBRA 2 da `line/render-loop`, 2026-09-12).
//!
//! ⚠️ Correm DEPOIS do dreno do pivot do joint (a fase anterior): um gate lê o par `joint_pivot_commit`
//! contíguo.

use super::*;
use ph2d_i18n::tr;

/// Os pedidos de precisão, emissão e saída da folha que o dreno do barramento recolheu neste quadro.
pub(super) struct SpriteRowIntents {
    pub(super) remove_from_sheet_row: Option<NodeId>,
    pub(super) precision_request: Option<(u64, ph2d_color::Precision)>,
    pub(super) emissive_edits: Vec<(u64, f32)>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_sprite_precision_emissive(&mut self, intents: SpriteRowIntents) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            renderer,
            sim,
            asset_db,
            toasts,
            hero_live,
            atlas_asset_map,
            ..
        } = FrameGfx::of(gfx);
        let SpriteRowIntents {
            remove_from_sheet_row,
            precision_request,
            mut emissive_edits,
        } = intents;
        // A troca de PRECISÃO sai por uma porta própria (plano `docs/Sprite_projeto/18` W5).
        //
        // ⚠️ **E fica DEPOIS do dreno do pivot do joint pela MESMA razão que o bloco abaixo**,
        // que já a tem escrita: o gate `the_position_commit_reseats_the_anchor_through_the_door`
        // lê os 3000 bytes a seguir à captura do pivot, e um bloco alheio no meio empurra a
        // porta para fora da janela. *A cura é tirar o intruso do meio, não alargar a janela* —
        // e este bloco já esteve lá, e já a reprovou.
        // ⚠️ Ela corre **depois** da troca de estratégia, e a ordem é load-bearing: converter
        // para 16 bits FORÇA `Individual`, então deixá-la correr antes faria um clique em
        // `Atlas` no mesmo quadro desfazer a conversão em silêncio.
        if crate::precision_convert::apply(
            precision_request,
            sim,
            renderer,
            asset_db,
            atlas_asset_map,
            toasts,
        ) {
            self.title_dirty = true;
        }
        // **A emissao autorada** (plano `docs/Sprite_projeto/18` W8). Zero REMOVE o componente:
        // uma sprite que nao emite nao carrega a linha no ficheiro nem uma entrada na varredura
        // do passe, e o quadro volta a ser byte-identico.
        //
        // ⚠️ Escreve o componente DIRETO, e nao pela `EditorCommandQueue`: o undo deste projeto
        // e' por DIFF de snapshot (`App::post_frame_undo`), e o `SpriteEmissive` esta' registado,
        // logo ele entra na captura como qualquer outro componente. A fila serve os caminhos que
        // precisam de aplicar por NOME vindo do painel; aqui o tipo e' conhecido.
        for (bits, intensity) in emissive_edits.drain(..) {
            let entity = ph2d_ecs::Entity::from_bits(bits);
            let em = ph2d_ecs::SpriteEmissive(intensity);
            if let Ok(mut e) = sim.world_mut().get_entity_mut(entity) {
                // ⚠️ Só escreve numa entidade que TEM `Sprite`: a seleção crua pode conter um
                // path vetorial ou um joint, e emitir luz é um facto sobre uma sprite. Mesmo
                // filtro que o fan-out do §11 aplica ao atravessar entidades sem `Collider`.
                if e.get::<ph2d_render::Sprite>().is_none() {
                    continue;
                }
                if em.emits() {
                    e.insert(ph2d_ecs::SpriteEmissive(em.clamped()));
                } else {
                    e.remove::<ph2d_ecs::SpriteEmissive>();
                }
                self.title_dirty = true;
            }
        }
        // ⚠️ **Este bloco fica DEPOIS do dreno do pivot do joint, de propósito.** Ele esteve
        // no meio do par `let joint_pivot_commit = …` → `if let Some(…) = joint_pivot_commit`,
        // e o gate `the_position_commit_reseats_the_anchor_through_the_door` reprovou: ele lê os
        // 3000 bytes a seguir à captura à procura da porta `set_joint_anchor_world`, e um bloco
        // alheio no meio empurra-a para fora da janela. *A cura é tirar o intruso do meio, não
        // alargar a janela* — a janela é a forma de o gate exigir que a captura e o dreno de uma
        // intenção fiquem à vista um do outro.
        // **EMPACOTAR A SELEÇÃO NUMA FOLHA** (plano `docs/Sprite_projeto/17` §7) — o pill
        // `[SHEET]` da fila de Image Tools, que substitui o `PH2D_SHEET_SMOKE`.
        //
        // ⚠️ **UMA chamada para a leva INTEIRA**, e não uma por entidade: empacotar N sprites
        // é um ato só. A folha nasce, eles viram filhos dela e o arranjo automático coloca-os.
        // **EMPACOTAR NUMA FOLHA** — uma porta só: o "Pack into Sheet" do menu de contexto
        // da hierarquia. ⚠️ O pill `[SHEET]` da fila de Image Tools EXISTIU e foi **retirado
        // por decisão do Enio** (2026-08-19), com a crate inteira: aquela fila é por-sprite,
        // e este verbo é da SELEÇÃO — o pill difundia uma ação por entidade e o dreno tinha
        // de as voltar a juntar. O menu resolve a linha clicada e aplica a lei do "Merge
        // Sprites" vizinho: a seleção inteira quando a linha faz parte dela, só ela quando
        // não faz. *Não reconstrua o pill sem ler isto.*
        // **RETIRAR DA FOLHA** (Enio 2026-08-19) — pelo MESMO caminho do arrasto-para-a-raiz
        // da hierarquia, e é isso que o torna barato: o `drain_reparent` já preserva a pose de
        // MUNDO (a peça fica onde está, não salta) e já reatribui o `RootOrder` de todas as
        // raízes. Uma segunda saída escrita à mão seria a que se esquecia do `RootOrder` —
        // e o sintoma disso é a hierarquia a reordenar-se sozinha no save seguinte.
        //
        // ⚠️ O toast de recusa não é decoração: sem ele, clicar "Remove from Sheet" numa
        // sprite que não está em folha nenhuma não faria **nada**, e é assim que um item de
        // menu se lê como partido.
        if let Some(row) = remove_from_sheet_row
            && let Some(live) = hero_live.as_ref()
        {
            let in_sheet = live
                .bridge
                .entity_for(row)
                .map(ph2d_ecs::Entity::from_bits)
                .and_then(|e| crate::sheet_bounds::sheet_parent(sim, e));
            if in_sheet.is_some() {
                hero_intents::drain_reparent(
                    ph2d_editor_core::screens::hero::HierReparentIntent {
                        dragged: row,
                        new_parent: None,
                        before: None,
                        after: None,
                    },
                    live,
                    sim,
                    toasts,
                );
                toasts.push(Toast::success(tr(
                    "shell.fase_sprite_precision_emissive.removed_from_sheet",
                )));
            } else {
                toasts.push(Toast::warning(tr(
                    "shell.fase_sprite_precision_emissive.remove_from_sheet_this",
                )));
            }
            self.title_dirty = true;
        }
    }
}

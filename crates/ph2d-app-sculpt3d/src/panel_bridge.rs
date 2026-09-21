//! A costura entre o **painel da cena 3D** e a `Sculpt3dScene` (ADR-0150, W12).
//!
//! ⚠️ Não confundir com o `crate::panel`, que é o outro lado da mesma ponte:
//! lá mora a TRADUÇÃO (intent → campos privados da cena), aqui a FASE (quando ela
//! roda, e quem abre o painel). Dois arquivos, dois assuntos — e o de lá é filho
//! da cena justamente porque precisa dos privados dela.
//!
//! ## Quem é dono da visibilidade
//!
//! O painel de física declara que **o artista** é o dono, e não faz edge-trigger:
//! física existe em todo documento, então abri-lo sozinho seria chrome a
//! dispensar. Aqui é o oposto e a diferença é factual: a cena 3D **não existe**
//! na maioria dos documentos, e este painel toma o slot do **INSPECTOR** — deixá-lo
//! aberto depois que o artista volta a pintar esconde o inspector da cena 2D.
//!
//! Então a lei é: **o painel segue as BORDAS do barro** — entra o barro, ele abre;
//! sai o barro, ele fecha; **entre** as bordas o artista manda, e o `X` continua
//! sendo um controle vivo.
//!
//! ⚠️ **BORDA, e não estado por frame:** re-afirmar a visibilidade todo frame
//! tornaria o botão de fechar um controle que não faz nada, que é a forma mais
//! barata de chrome morto.
//!
//! ⚠️ **A lei ANTERIOR era *"abre uma vez, na borda `None → Some` da cena"*, e ela
//! nasceu antes de existir um modo de que se SAI.** Ela deixava dois defeitos que
//! o Enio reportou juntos (2026-08-09): o painel **não fechava** ao sair do modo
//! (a cena continua viva — sair nunca larga a escultura) e **não reabria** depois
//! de fechado (a chave já existia, e não há segunda borda `None → Some`), então um
//! `X` custava o painel para o resto da sessão.
//!
//! ⚠️ **Preço nomeado:** a seção **Shading** governa a doação, que é o que se olha
//! justamente no modo LUZ — e ela sai da tela junto. Alcançá-la exige voltar ao
//! barro. É o trade de o painel ser do MODO; o slot do inspector é o que o decide.

use ph2d_editor_core::screens::hero::HeroScreen;

use crate::Sculpt3dScene;

/// Publica o retrato para o `paint` e aplica o que o artista fez.
///
/// Devolve `true` se o artista pediu o **bake no sprite** — o gesto que este
/// bridge não consegue executar, porque ele precisa do mundo, do renderizador e
/// do mapa de atlas. Ele só o repassa, e o chamador arma o MESMO campo que o
/// `Shift+B` arma: uma porta, dois pedintes, e por isso o botão e o atalho não
/// podem divergir. É o precedente do `panel_bridge` da `ph2d-app-physics`, que devolve o
/// `show_colliders` pela mesma razão.
pub fn dispatch(
    hero: &mut HeroScreen,
    scene: Option<&mut Sculpt3dScene>,
    lei_do_alvo: Option<usize>,
) -> Vec<crate::Sculpt3dFrameRequest> {
    // ── 0. O pill SCULPT diz o que a forma É. ──
    // ⚠️ **ANTES do early-return**, e é a metade que o torna correto: sem cena o pill tem de ficar
    // SOLTO (o estado honesto de *entrar*), e um sync que morasse depois do `let Some` deixaria o
    // botão preso em *pressed* para sempre no frame em que a cena fosse largada.
    crate::sync_pill(hero, scene.as_deref());

    let Some(scene) = scene else {
        // Sem cena não há retrato — e é isso que faz o `paint` do painel sair no
        // primeiro `if`. Publicar um retrato vazio seria pior: seis seções de
        // controles apontando para uma escultura que não existe.
        ph2d_panel_sculpt3d::set_current_sculpt3d(None);
        return Vec::new();
    };

    // ── 1. O painel segue as BORDAS do barro. ──
    // Entra o barro, ele abre; sai o barro, ele fecha; ENTRE as bordas o artista
    // manda, e o `X` continua sendo um controle vivo.
    if let Some(entered) = scene.take_clay_edge() {
        hero.panel_visibility.insert("sculpt3d", entered);
    }

    // ── 2. Publicar. Toda row lê isto; o painel não guarda cópia. ──
    // ⚠️ O alvo do bake é um fato da cena **2D**, então ele é injetado aqui: a
    // escultura não sabe — nem deve saber — quem está selecionado no canvas.
    let has_bake_target = hero.gizmo.iter_selected().next().is_some();
    // ⚠️ **A LEI vem de FORA pela mesma razão que o alvo**, e uma a mais: ela é um
    // campo do DOCUMENTO do objecto assado, e o mapa dos assados é do shell — a
    // escultura não sabe que um sprite foi assado, nem deve saber.
    ph2d_panel_sculpt3d::set_current_sculpt3d(Some(
        scene.panel_snapshot(has_bake_target, lei_do_alvo),
    ));

    // ── 3. Aplicar. O painel enfileirou os intents no dispatch de eventos. ──
    // ⚠️ **Os pedidos ACUMULAM num conjunto, não num `Option`:** dois gestos
    // podem cair no mesmo frame (o artista clica os dois botões antes de o frame
    // virar), e guardar só o último perderia um em silêncio.
    let mut want = Vec::new();
    for intent in ph2d_panel_sculpt3d::drain_intents() {
        // ⭐⭐⭐ **A PALETA DE PINCÉIS É INTERCEPTADA AQUI** — o selector saiu do painel em
        // 2026-09-20 (ordem do dono), e abrir uma paleta precisa do `HeroScreen`, que o
        // `apply_panel_intent` não tem. ⚠️ O braço de lá tem um `debug_assert!(false)` a dizê-lo:
        // se esta linha se perder, o botão fica **mudo** e a suíte verde.
        if matches!(
            intent,
            ph2d_panel_sculpt3d::Sculpt3dIntent::OpenBrushPalette
        ) {
            hero.store
                .open_command_palette(ph2d_panel_sculpt3d::brush_palette::build());
            continue;
        }
        if let Some(req) = scene.apply_panel_intent(intent)
            && !want.contains(&req)
        {
            want.push(req);
        }
    }

    // ── 4. O *pick* da paleta volta NOUTRO QUADRO. ──
    //
    // ⚠️⚠️ **O dreno é CONDICIONAL, e isso não é zelo:** este canal tem **cinco** consumidores (a
    // biblioteca do Motion, o `Ctrl+K`, o `+` do Inspector, a paleta de formas do 3D e agora esta),
    // e um `take` incondicional engoliria o *pick* de outro — com o sintoma a ser *«às vezes não
    // faz nada»*.
    //
    // ⭐ **E o que o *pick* faz é a MESMA LEI que a ficha fazia** — o `intent_for_palette_pick`
    // delega no mesmo `group_chip_ui` de sempre (que faz `switch_verb`: guarda o pincel vivo no
    // slot do verbo que sai e carrega o do que entra). *Escrever «troca o verbo» aqui seria a
    // segunda resposta à mesma pergunta.*
    if let Some(id) = hero
        .store
        .take_command_pick_if(|id| ph2d_panel_sculpt3d::brush_palette::verb_at(id).is_some())
        && let Some(intent) = ph2d_panel_sculpt3d::intent_for_palette_pick(id)
        && let Some(req) = scene.apply_panel_intent(intent)
        && !want.contains(&req)
    {
        want.push(req);
    }
    want
}

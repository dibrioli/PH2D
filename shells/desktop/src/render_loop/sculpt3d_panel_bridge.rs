//! A costura entre o **painel da cena 3D** e a `Sculpt3dScene` (ADR-0150, W12).
//!
//! ⚠️ Não confundir com o `sculpt3d::panel`, que é o outro lado da mesma ponte:
//! lá mora a TRADUÇÃO (intent → campos privados da cena), aqui a FASE (quando ela
//! roda, e quem abre o painel). Dois arquivos, dois assuntos — e o de lá é filho
//! da cena justamente porque precisa dos privados dela.
//!
//! ## Quem é dono da visibilidade
//!
//! O painel de física declara que **o artista** é o dono, e não faz edge-trigger:
//! física existe em todo documento, então abri-lo sozinho seria chrome a
//! dispensar. Aqui é o oposto e a diferença é factual: a cena 3D **não existe**
//! na maioria dos documentos, e quando ela nasce ela é a única coisa na tela.
//! Então este bridge a ABRE uma vez, na borda `None → Some`, e nunca mais toca
//! nela — o `X` do painel fecha, e é o artista quem manda a partir daí.
//!
//! ⚠️ **Uma vez, e não a cada frame:** re-abrir todo frame tornaria o botão de
//! fechar um controle que não faz nada, que é a forma mais barata de chrome
//! morto.

use ph2d_editor::screens::hero::HeroScreen;

use crate::sculpt3d::Sculpt3dScene;

/// Publica o retrato para o `paint` e aplica o que o artista fez.
pub(crate) fn dispatch(hero: &mut HeroScreen, scene: Option<&mut Sculpt3dScene>) {
    let Some(scene) = scene else {
        // Sem cena não há retrato — e é isso que faz o `paint` do painel sair no
        // primeiro `if`. Publicar um retrato vazio seria pior: seis seções de
        // controles apontando para uma escultura que não existe.
        ph2d_panel_sculpt3d::set_current_sculpt3d(None);
        return;
    };

    // ── 1. Abrir, uma vez. ──
    // O guard é `panel_visibility` NÃO ter a chave: `is_panel_visible` devolve
    // `false` tanto para *fechado pelo artista* quanto para *nunca visto*, e as
    // duas coisas são diferentes. A entrada existir é a marca de que este bridge
    // já se pronunciou.
    if !hero.panel_visibility.contains_key("sculpt3d") {
        hero.panel_visibility.insert("sculpt3d", true);
    }

    // ── 2. Publicar. Toda row lê isto; o painel não guarda cópia. ──
    ph2d_panel_sculpt3d::set_current_sculpt3d(Some(scene.panel_snapshot()));

    // ── 3. Aplicar. O painel enfileirou os intents no dispatch de eventos. ──
    for intent in ph2d_panel_sculpt3d::drain_intents() {
        scene.apply_panel_intent(intent);
    }
}

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

use ph2d_editor::screens::hero::HeroScreen;

use crate::sculpt3d::Sculpt3dScene;

/// Publica o retrato para o `paint` e aplica o que o artista fez.
///
/// Devolve `true` se o artista pediu o **bake no sprite** — o gesto que este
/// bridge não consegue executar, porque ele precisa do mundo, do renderizador e
/// do mapa de atlas. Ele só o repassa, e o chamador arma o MESMO campo que o
/// `Shift+B` arma: uma porta, dois pedintes, e por isso o botão e o atalho não
/// podem divergir. É o precedente do `physics_panel_bridge`, que devolve o
/// `show_colliders` pela mesma razão.
pub(crate) fn dispatch(
    hero: &mut HeroScreen,
    scene: Option<&mut Sculpt3dScene>,
) -> Vec<crate::sculpt3d::Sculpt3dFrameRequest> {
    // ── 0. O pill SCULPT diz o que a forma É. ──
    // ⚠️ **ANTES do early-return**, e é a metade que o torna correto: sem cena o pill tem de ficar
    // SOLTO (o estado honesto de *entrar*), e um sync que morasse depois do `let Some` deixaria o
    // botão preso em *pressed* para sempre no frame em que a cena fosse largada.
    crate::sculpt3d::sync_pill(hero, scene.as_deref());

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
    ph2d_panel_sculpt3d::set_current_sculpt3d(Some(scene.panel_snapshot(has_bake_target)));

    // ── 3. Aplicar. O painel enfileirou os intents no dispatch de eventos. ──
    // ⚠️ **Os pedidos ACUMULAM num conjunto, não num `Option`:** dois gestos
    // podem cair no mesmo frame (o artista clica os dois botões antes de o frame
    // virar), e guardar só o último perderia um em silêncio.
    let mut want = Vec::new();
    for intent in ph2d_panel_sculpt3d::drain_intents() {
        if let Some(req) = scene.apply_panel_intent(intent)
            && !want.contains(&req)
        {
            want.push(req);
        }
    }
    want
}

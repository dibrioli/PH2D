//! ⭐⭐⭐ **A CURA ao lado da QUEIXA: a fileira do gatilho CRIA a acção que falta** (suplente #24).
//!
//! A secção *Action Trigger* sabia dizer *«não há nenhuma acção chamada `fire`»* desde a wave dela,
//! e a cura vivia noutra janela (*Settings ▸ Input Map…*). ⚠️ *Uma queixa que nomeia a cura e não a
//! alcança é meia queixa* — o artista que a lê tem de descobrir sozinho onde fica a outra janela.
//!
//! # ⛔ Porque é que isto é uma FASE da shell e não uma `ComponentEdit`
//!
//! O que nasce é uma linha do [`ph2d_editor_core::screens::hero::HeroScreen::input_map`], que é
//! estado do **EDITOR** e não do mundo: ele não viaja num `ComponentBlob`, não passa pelo ledger do
//! `preview_drive` e não é do `Ctrl+Z` do documento. *Quem tem o mapa é a shell, e o painel só diz
//! quem pediu.*
//!
//! # ⚠️ E as linhas do painel do Input Map são RE-SINCRONIZADAS
//!
//! Criar a acção sem isso deixaria a janela do mapa — se ela estiver aberta — a mostrar a lista
//! **antiga**, e o artista veria a queixa desaparecer no Inspector e nada aparecer ali. É a mesma
//! chamada que o botão `Add` daquela janela já faz, e ela é a razão de esta fase existir em vez de
//! uma linha solta no dreno.

use ph2d_editor_core::screens::hero::chrome::sync_input_map_rows;

use super::FrameGfx;

impl crate::App {
    /// Cria as acções que a fileira do gatilho pediu neste quadro.
    ///
    /// ⚠️ **`InputMap::create` é idempotente por NOME** (ele devolve a que já existe), logo dois
    /// pedidos iguais no mesmo quadro não fazem duas linhas — a lei vive na porta, e não aqui.
    pub(super) fn fase_criar_accao_do_mapa(&mut self, pedidos: Vec<String>) {
        if pedidos.is_empty() {
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx { hero_screen, .. } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        for nome in pedidos {
            let n = nome.trim();
            // ⚠️ **Um nome vazio nunca chega aqui** (a fileira só oferece o botão com um nome que o
            // mapa não conhece), e a guarda fica porque *uma acção sem nome é inalcançável por
            // código* — a mesma recusa que o `Add` da janela do mapa já escreve.
            if n.is_empty() {
                continue;
            }
            hero.input_map.create(n);
        }
        let mapa = hero.input_map.clone();
        sync_input_map_rows(&mut hero.store, &mapa);
    }
}

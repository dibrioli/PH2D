//! **A paleta de comandos GLOBAL, do lado da shell** — o `Ctrl+K` e o dreno do pick.
//!
//! O modelo e a execução vivem os dois em `ph2d_editor::screens::hero::global_palette` (o mesmo
//! módulo emite e executa, que é o que torna um comando morto estruturalmente impossível). Aqui
//! ficam só as duas pontas que a shell possui: a TECLA e o DRENO.
//!
//! ⚠️ **Não há uma função *«qual das duas paletas está aberta?»***, e a ausência é deliberada: o
//! teclado (escrever, Enter, Backspace, Escape) já é GENÉRICO sobre o store, e o dreno reconhece o
//! pick pelo MODELO. Uma função dessas nasceria sem chamador — a segunda resposta à espera de quem
//! a chame.
//!
//! ⚠️ **O dreno é CONDICIONAL, e não por elegância.** O canal do pick tem dois consumidores — este
//! e o `route_palette_pick` da paleta de nós do Motion. Com dois `take` incondicionais, quem recebe
//! o pick passa a ser *a ordem em que os drenos correm no frame*: um facto invisível que muda
//! quando alguém reordena o laço, com o sintoma a ser um comando que *às vezes não faz nada*. Os
//! dois passaram a perguntar `take_command_pick_if` — cada um reconhece os próprios ids e deixa o
//! resto onde está.

use crate::App;
use ph2d_editor::HeroScreen;
use ph2d_editor::screens::hero::global_palette;

/// `Ctrl+K` — abre a paleta sobre o que o app oferece AGORA.
///
/// O modelo é construído no momento da abertura (nunca por frame): é a mesma lei do
/// `open_command_palette` do Motion, e é o que faz a paleta descrever o estado em que o artista a
/// chamou.
///
/// ⚠️ Toma o `HeroScreen` e não o `App` de propósito: o sítio que a chama (o `match` de teclas do
/// `input_handlers`) já tem o `gfx` emprestado, e um método `&mut self` colidiria com esse
/// empréstimo — o que empurraria a abertura para um flag lido no frame seguinte, que é um salto que
/// nada exige.
pub(crate) fn open_global_palette(hero: &mut HeroScreen) {
    let model = global_palette::build_global_model(hero);
    hero.store.open_command_palette(model);
}

impl App {
    /// Drena o pick DESTA paleta e executa-o. Chamado uma vez por frame.
    ///
    /// ⚠️ O reconhecimento é *"este id está no modelo que EU ofereço agora"*, e por isso o modelo é
    /// reconstruído para a pergunta: o pick chega no frame seguinte ao clique, com a paleta já
    /// fechada, então não há modelo guardado a que perguntar. Reconstruir é barato (duas listas) e
    /// evita uma segunda cópia da lista viva só para a poder consultar depois.
    pub(crate) fn global_palette_drain(&mut self) {
        let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return;
        };
        let model = global_palette::build_global_model(h);
        let Some(id) = h.store.take_command_pick_if(|id| model.is_item(id)) else {
            return;
        };
        global_palette::route_global_pick(h, id);
    }
}

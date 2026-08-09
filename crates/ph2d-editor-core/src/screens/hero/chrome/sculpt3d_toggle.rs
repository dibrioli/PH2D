// ph2d-chrome-sync:z=272 (dispatch priority, ADR-0107; lower = earlier)
//! **O pill SCULPT** — entra e sai do modo escultura 3D (ADR-0150).
//!
//! ⚠️ **O que ele conserta não é descobribilidade, é uma PRISÃO.** Com o barro na tela a cena 3D é
//! dona do ponteiro (`FormRole::draws_clay` decide as duas coisas: o passe de cor desenha, e o
//! clique é dela), e a única saída era a tecla `D` — uma feature que só existe para quem já sabe
//! que ela existe. O preço não era estético: o artista não conseguia **selecionar um sprite** para
//! configurar o padrão do pincel, que é o gesto inteiro da W17 (Enio, 2026-08-09).
//!
//! ⚠️ **Ele não guarda bool nenhum.** O estado *pressed* é ESCRITO pelo shell a cada frame a partir
//! do papel da forma — porque o `D` também o move, e um bool próprio aqui é como o pill passa a
//! dizer *fora* sobre uma cena que está na tela. É o mesmo motivo do irmão `physics_toggle`, que
//! deriva do `panel_visibility`, e do rádio do rail do Painter, que deriva do modo publicado.
//!
//! Fiação central: `ids::TOPBAR_SCULPT3D` + o pill em `screens/hero/fixture.rs` (`IconId::Cube`) +
//! registro em `topbar/mod.rs::populate` + o dreno de `EditorAction::ToggleSculpt3d` no shell.

use crate::action_bus::EditorAction;
use crate::ids;
use crate::interaction::WidgetEvent;
use crate::screens::hero::HeroScreen;

pub fn apply(hero: &mut HeroScreen, event: WidgetEvent) -> bool {
    let WidgetEvent::Click(id) = event else {
        return false;
    };
    if id != ids::TOPBAR_SCULPT3D {
        return false;
    }
    // ⚠️ **Um pedido, e não uma ordem de entrar ou de sair.** Quem sabe em que posição a forma está
    // é o shell — e ele é o único que sabe, porque o `D` a move sem passar por aqui. Ler o estado
    // do botão para escolher o sentido (o que o `flip_toggle` faz, e ali está certo, porque lá o
    // estado é do próprio store) daria a resposta errada em todo frame entre uma tecla `D` e o
    // sync seguinte.
    hero.bus.push(EditorAction::ToggleSculpt3d);
    true
}

//! **A BORDA INTEIRA REDIMENSIONA A COLUNA** — o gesto e a seta bidirecional.
//!
//! Enio, 2026-08-30: *«os painéis devem ser redimensionáveis para esquerda e para direita e com
//! setas bidirecionais no cursor; os pontinhos de redimensionamento podem ser retirados. A borda
//! inteira serve para redimensionar»*.
//!
//! ⭐ **Uma pergunta, dois consumidores.** O cursor e o arrasto chamam a MESMA função
//! ([`ph2d_editor::screens::layout::HeroLayout::dock_seam_at`]) — a seta a aparecer um pixel ao
//! lado de onde o gesto agarra lê-se como *«às vezes não pega»*, e é o defeito que o irmão desta
//! costura no canvas 3D (`field3d_layout::seam_cursor`) já pagou.
//!
//! ⚠️ **O layout vem PUBLICADO, não re-derivado** (`hero.last_layout`): o ponteiro corre fora do
//! quadro, e espelhar a aritmética das colunas aqui seria dar dois donos ao mesmo pixel.
//!
//! ⛔ **E o gesto corre ANTES do hit-test de chrome, de propósito.** A costura vive DENTRO da
//! coluna (os últimos `DOCK_SEAM_PX` px dela), logo por cima do corpo do painel; sem a
//! precedência, o painel comeria o press e a borda seria inerte.

use ph2d_editor::screens::layout::DockSide;

impl crate::App {
    /// O layout que o último quadro resolveu, se já houve um.
    fn hero_layout(&self) -> Option<ph2d_editor::screens::layout::HeroLayout> {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.last_layout)
    }

    /// A seta a mostrar sob este ponto, se ele estiver sobre uma costura de largura.
    ///
    /// ⚠️ **`EwResize` e não `ColResize`**: o Enio pediu *«setas bidirecionais»*, e é a mesma seta
    /// que a divisória do canvas 3D já usa — duas costuras do mesmo app não podem prometer o mesmo
    /// gesto com desenhos diferentes.
    pub(crate) fn dock_seam_cursor(&self, x: f32, y: f32) -> Option<winit::window::CursorIcon> {
        if self.dock_seam_drag.is_some() {
            return Some(winit::window::CursorIcon::EwResize);
        }
        let layout = self.hero_layout()?;
        // ⚠️ **A alça de uma coluna FECHADA promete o mesmo gesto** — puxar a borda — logo tem de
        //    mostrar a mesma seta. Um cursor diferente diria que é outra coisa, e o que muda é só
        //    a direcção em que a borda ainda pode ir.
        if layout.dock_reopen_at((x, y)).is_some() {
            return Some(winit::window::CursorIcon::EwResize);
        }
        layout
            .dock_seam_at((x, y))
            .map(|_| winit::window::CursorIcon::EwResize)
    }

    /// Press: começa o arrasto se o ponto estiver na costura. `true` = a tecla foi consumida.
    pub(crate) fn dock_seam_down(&mut self, x: f32, y: f32) -> bool {
        // ⭐⭐⭐ **A ALÇA vem PRIMEIRO** — ela e a costura nunca coexistem no mesmo lado (uma exige
        //    a coluna aberta, a outra fechada), mas perguntar por ela primeiro deixa a lei escrita
        //    numa ordem em vez de depender dessa exclusão continuar verdadeira.
        //
        // ⚠️ **Reabrir acontece no DOWN, não no clique**, e é isso que faz o mesmo gesto servir o
        //    toque e o arrasto: um toque reabre a coluna na largura que ela tinha; um arrasto
        //    reabre-a e continua a redimensioná-la, que é *puxar a borda de volta*.
        if let Some(side) = self.hero_layout().and_then(|l| l.dock_reopen_at((x, y))) {
            let tenant = self.hero_layout().map(|l| l.dock_tenant(side));
            if let (Some(tenant), Some(hero)) = (
                tenant,
                self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()),
            ) {
                hero.panel_visibility.insert(tenant, true);
                self.dock_seam_drag = Some(side);
                return true;
            }
        }
        let Some(side) = self.hero_layout().and_then(|l| l.dock_seam_at((x, y))) else {
            return false;
        };
        self.dock_seam_drag = Some(side);
        true
    }

    /// Move: escreve a largura nova. `true` enquanto o arrasto vive.
    ///
    /// ⚠️ A largura sai de [`HeroLayout::dock_width_for`] — a conta é **do lado** (à esquerda a
    /// coluna cresce com o `x`, à direita decresce), e é a inversão que se escreve ao contrário
    /// sem o compilador reclamar. O clamp mora na porta do store, não aqui.
    pub(crate) fn dock_seam_move(&mut self, x: f32) -> bool {
        let Some(side) = self.dock_seam_drag else {
            return false;
        };
        let Some(layout) = self.hero_layout() else {
            return false;
        };
        let w = layout.dock_width_for(side, x);
        let tenant = layout.dock_tenant(side);
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return true;
        };
        // ⭐⭐⭐ **Arrastar para dentro, para além do mínimo, FECHA a coluna** — o gesto do Blender,
        //    e a maior alavanca de ecrã que a medição do tablet encontrou (fechar as duas devolve
        //    89–92 %). Até aqui o arrasto travava no mínimo e fechar custava dois passeios ao menu.
        //
        // ⚠️ **Fecha pela MESMA porta que o menu usa** (`panel_visibility`): um segundo caminho
        //    para esconder um painel daria dois estados de «fechado» que podiam discordar — e o
        //    interruptor do menu passaria a mentir sobre o que o dedo fez.
        if w < ph2d_editor::interaction::WidgetStore::DOCK_W_COLLAPSE {
            hero.panel_visibility.insert(tenant, false);
            // O arrasto acaba aqui: a costura que ele agarrava deixou de existir.
            self.dock_seam_drag = None;
            return true;
        }
        hero.store.set_dock_width(side, w);
        true
    }

    /// Release: fecha o arrasto. `true` se havia um.
    pub(crate) fn dock_seam_up(&mut self) -> bool {
        self.dock_seam_drag.take().is_some()
    }
}

/// O estado do arrasto — qual coluna está a ser redimensionada agora.
pub(crate) type DockSeamDrag = Option<DockSide>;

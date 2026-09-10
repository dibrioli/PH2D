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
//!
//! # ⭐⭐⭐ O que este ficheiro deixou de decidir (2026-09-07)
//!
//! *Quais painéis uma coluna leva consigo* **não mora aqui**. Ela é a
//! [`ph2d_editor::screens::hero::dock_columns`], no `ph2d-editor-core` — e mudou de sítio por uma
//! razão medida: enquanto foi uma `fn` privada de um `impl App` do **binário**, nenhum teste a
//! alcançava, e o gate que a cobria lia o **fonte** com `contains()`. ⛔ *Ele leu a linha do
//! defeito e chamou-lhe correcta.* Aqui fica só o **gesto**: onde o dedo está, o que ele arma, e
//! quando ele solta.

use ph2d_editor::screens::hero::dock_columns;
use ph2d_editor::screens::layout::DockSide;

/// O estado do arrasto de uma costura de coluna.
///
/// ⚠️ **Ele guarda o LADO e mais nada desde 2026-09-09.** Tinha dois campos a mais — `may_close`
/// e `width_at_start` — e os dois eram do FECHO: a trava que impedia um arrasto nascido da alça
/// de fechar no primeiro pixel, e a largura de antes que a reabertura devolvia. O dono retirou o
/// fecho por arrasto (*«Deixa o colapsar apenas no menu da barra superior»*), e **um campo que só
/// servia a um gesto que já não existe é estado que a próxima pessoa tenta honrar.**
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) struct SeamDrag {
    /// Que coluna está a ser redimensionada.
    pub(crate) side: DockSide,
}

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
        if let Some(side) = self.hero_layout().and_then(|l| l.dock_reopen_at((x, y)))
            && self.open_column(side)
        {
            // ⚠️ Capturada DEPOIS de reabrir: a largura de partida deste arrasto é a que a
            //    reabertura acabou de repor, e é essa que um fecho seguinte tem de guardar.
            self.dock_seam_drag = Some(SeamDrag { side });
            return true;
        }
        let Some(side) = self.hero_layout().and_then(|l| l.dock_seam_at((x, y))) else {
            return false;
        };
        self.dock_seam_drag = Some(SeamDrag { side });
        true
    }

    /// Move: escreve a largura nova. `true` enquanto o arrasto vive.
    ///
    /// ⚠️ A largura sai de [`ph2d_editor::screens::layout::HeroLayout::dock_width_for`] — a conta é
    /// **do lado** (à esquerda a coluna cresce com o `x`, à direita decresce), e é a inversão que
    /// se escreve ao contrário sem o compilador reclamar. O clamp mora na porta do store.
    pub(crate) fn dock_seam_move(&mut self, x: f32) -> bool {
        let Some(drag) = self.dock_seam_drag else {
            return false;
        };
        let Some(layout) = self.hero_layout() else {
            return false;
        };
        let w = layout.dock_width_for(drag.side, x);
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.set_dock_width(drag.side, w);
        }
        true
    }

    /// Reabre a coluna. A lei é a [`dock_columns::open`].
    fn open_column(&mut self, side: DockSide) -> bool {
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return false;
        };
        dock_columns::open(hero, side)
    }

    /// Release: fecha o arrasto. `true` se havia um.
    ///
    /// ⛔⛔ **Ele devolvia ao mínimo quem largasse na faixa do degrau, e essa metade MORREU com o
    /// gesto de fechar** (Enio, 2026-09-09). A faixa existia porque a borda seguia o dedo abaixo
    /// do mínimo para o fecho se ver acontecer; sem fecho, a porta do store já não deixa a largura
    /// descer até lá. *Um remendo que corrige o estado que outro gesto produzia é lixo no dia em
    /// que esse gesto sai — e é lixo que parece cuidado.*
    pub(crate) fn dock_seam_up(&mut self) -> bool {
        self.dock_seam_drag.take().is_some()
    }
}

/// O estado do arrasto — qual coluna está a ser redimensionada agora, e se ela já pode fechar.
pub(crate) type DockSeamDrag = Option<SeamDrag>;

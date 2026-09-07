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
/// ⚠️ **Não é só o lado.** Ver [`SeamDrag::may_close`] — um arrasto que NASCEU de uma reabertura
/// começa com o dedo na borda de fora, onde a largura medida é ~0, e sem a trava ele fecharia a
/// coluna no primeiro pixel de movimento.
// ⚠️ **`PartialEq` sem `Eq`**: o `width_at_start` é um `f32`, e `f32` não é `Eq` — não há igualdade
// reflexiva sobre `NaN`. Declarar `Eq` aqui é erro de compilação, e está certo que seja.
#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) struct SeamDrag {
    /// Que coluna está a ser redimensionada.
    pub(crate) side: DockSide,
    /// ⭐⭐⭐ **Este arrasto já pode fechar?**
    ///
    /// Um arrasto que agarra a costura de uma coluna **aberta** nasce com `true`: a mão está sobre
    /// a borda dela, e puxar para dentro é o gesto de fechar.
    ///
    /// ⛔⛔ Um arrasto que nasce da **alça de reabertura** nasce com `false`, e a razão é
    /// aritmética: a alça vive na borda **exterior** da janela, logo `dock_width_for` no ponto do
    /// toque vale 0..6 px — muito abaixo do degrau de fecho. Sem a trava, o primeiro `CursorMoved`
    /// depois de reabrir mandava fechar de imediato; e como os painéis recém-abertos ainda não
    /// tinham pintado, não havia rect nenhum para encontrar, **nada era escondido, e o arrasto
    /// morria com a coluna reaberta**. Era essa a sequência exacta da foto de 2026-09-07.
    ///
    /// Ele arma quando a largura medida chega uma vez ao mínimo legal — *a mão trouxe a borda de
    /// volta para dentro do território onde fechar quer dizer alguma coisa.*
    pub(crate) may_close: bool,
    /// ⭐⭐ **A escolha de largura ANTES de este arrasto lhe tocar** — o que a reabertura devolve.
    ///
    /// ⛔ Capturada no `Down`, e não no instante do fecho: **cada pixel do arrasto escreve a
    /// largura** (clampada ao mínimo pela porta do store), logo uma coluna de 400 px arrastada até
    /// fechar já deixou `Some(220)` gravado muito antes de o degrau disparar. Lê-la no fecho
    /// devolveria sempre o mínimo — que é metade do report *«a retração ainda está ruim»*.
    pub(crate) width_at_start: Option<f32>,
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
            self.dock_seam_drag = Some(SeamDrag {
                side,
                may_close: false,
                width_at_start: self.dock_width_choice(side),
            });
            return true;
        }
        let Some(side) = self.hero_layout().and_then(|l| l.dock_seam_at((x, y))) else {
            return false;
        };
        self.dock_seam_drag = Some(SeamDrag {
            side,
            may_close: true,
            width_at_start: self.dock_width_choice(side),
        });
        true
    }

    /// A escolha de largura de uma coluna, ou `None` se ninguém arrastou aquela borda.
    fn dock_width_choice(&self, side: DockSide) -> Option<f32> {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.store.dock_width_choice(side))
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
        // A trava arma assim que a borda volta ao território legal. Ver [`SeamDrag::may_close`].
        if !drag.may_close
            && w >= ph2d_editor::interaction::WidgetStore::DOCK_W_MIN
            && let Some(d) = self.dock_seam_drag.as_mut()
        {
            d.may_close = true;
        }
        // ⭐⭐⭐ **Arrastar para dentro, para além do mínimo, FECHA a coluna** — o gesto do Blender,
        //    e a maior alavanca de ecrã que a medição do tablet encontrou (fechar as duas devolve
        //    89–92 %). Até aqui o arrasto travava no mínimo e fechar custava dois passeios ao menu.
        //
        // ⚠️ **Fecha pela MESMA porta que o menu usa** (`panel_visibility`): um segundo caminho
        //    para esconder um painel daria dois estados de «fechado» que podiam discordar — e o
        //    interruptor do menu passaria a mentir sobre o que o dedo fez.
        if w < ph2d_editor::interaction::WidgetStore::DOCK_W_COLLAPSE
            && self.dock_seam_drag.is_some_and(|d| d.may_close)
        {
            // ⚠️ **O arrasto só acaba se o fecho ACONTECEU.** Largá-lo mesmo quando não há nada a
            //    esconder deixava o dedo em baixo sobre uma costura que continuava a existir — e o
            //    artista tinha de largar e voltar a agarrar sem nada lhe dizer porquê.
            if self.close_column(drag.side, drag.width_at_start) {
                self.dock_seam_drag = None;
            }
            return true;
        }
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.store.set_dock_width(drag.side, w);
        }
        true
    }

    /// Fecha a coluna. A lei é a [`dock_columns::close`] — aqui fica só a travessia até ao `hero`.
    fn close_column(&mut self, side: DockSide, width_at_start: Option<f32>) -> bool {
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return false;
        };
        dock_columns::close(hero, side, width_at_start)
    }

    /// Reabre a coluna. A lei é a [`dock_columns::open`].
    fn open_column(&mut self, side: DockSide) -> bool {
        let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) else {
            return false;
        };
        dock_columns::open(hero, side)
    }

    /// Release: fecha o arrasto. `true` se havia um.
    pub(crate) fn dock_seam_up(&mut self) -> bool {
        self.dock_seam_drag.take().is_some()
    }
}

/// O estado do arrasto — qual coluna está a ser redimensionada agora, e se ela já pode fechar.
pub(crate) type DockSeamDrag = Option<SeamDrag>;

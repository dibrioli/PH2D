//! ⭐⭐⭐ **OS QUATRO VIEWPORTS DA ESCULTURA** — frente, lado, cima e a vista do
//! artista, ao mesmo tempo.
//!
//! Ordem do Enio, 2026-09-08: *«veja o módulo de Modelagem 3d. Ele tem
//! viewports (4) […] traga esses features para esse módulo.»*
//!
//! # ⭐⭐ A DIVISÃO não se reinventa: ela é lida
//!
//! Onde as costuras estão, que rectângulos elas produzem, como se agarram e que
//! cursor o ponteiro pede sobre elas já é lei desta casa
//! ([`crate::field3d_layout`]), com a lição das arestas inteiras escrita no
//! cabeçalho dela — *um valor que é pixels não pode sair fraccionário da porta
//! que o define*. E qual vista nomeada mora em cada quadrante é a disposição do
//! Blender, também de lá ([`Split::named`]).
//!
//! ⚠️ O que este ficheiro acrescenta é o que aquele módulo não tem: **uma câmera
//! por quadrante**, e a resposta a *«de quem é este clique?»* num módulo onde
//! todo gesto (o traço, a máscara, o transform, o filtro) já supunha que só
//! existia uma vista.
//!
//! # ⛔⛔ A CÂMERA DO ACTIVO É A `Sculpt3dScene::camera`, e a lista guarda as OUTRAS
//!
//! O campo `camera` tem **~30 leitores** neste módulo — o pick, o cursor, o
//! raio em pixels, o estêncil, o transform, a doação — e todos eles querem
//! sempre a **mesma** coisa: a vista em que a mão está. Trocá-lo por
//! `cams[activo]` em trinta sítios não compraria nada e criaria trinta
//! oportunidades de alguém indexar o quadrante errado.
//!
//! ⇒ a invariante é: **`camera` manda para o viewport activo, e
//! `vp_cams[vp_active]` está velha.** Guardar e pegar acontece num sítio só
//! ([`Sculpt3dScene::set_active_vp`]) e ler acontece por uma porta só
//! ([`Sculpt3dScene::cam_of`]). *Um registo «em uso» é honesto enquanto tiver um
//! escritor e um leitor; o que mata é quando cada sítio decide por si.*
//!
//! # ⭐⭐⭐ E o VIEWPORT deixou de ser um campo escrito
//!
//! Ele era `viewport: (u32, u32)`, escrito pelo desenho com **o tamanho da
//! JANELA** — e por isso a peça era desenhada por baixo dos painéis e das
//! réguas, o mesmo defeito que o Enio reportou ao módulo vizinho em 31/08
//! (*«a viewport ainda não se encaixa na área correta para ela»*). Agora ele é
//! **derivado** da área do canvas, da divisão e de qual quadrante está activo,
//! e o gesto converte a janela para o referencial da vista por uma porta
//! ([`Sculpt3dScene::to_view`]).

use super::Sculpt3dScene;
use crate::field3d_layout::Split;
use crate::field3d_views::Standard;
use ph2d_editor::zones::Rect as EditorRect;
use ph2d_mesh_render::{Camera3d, ScreenRect};

/// ⭐⭐⭐ **O ESTADO DA JANELA 3D** — como a peça é OLHADA.
///
/// ⚠️ **Um tipo e não dez campos na cena**: eles nascem juntos (a divisão cria as
/// câmeras), morrem juntos e são lidos pelos mesmos três módulos — este, o
/// [`super::navball`] e o [`super::gizmo`]. Soltos entre os sessenta campos que
/// descrevem *o que a peça É*, eles eram indistinguíveis do resto.
#[derive(Default)]
pub(crate) struct Janela {
    /// ⭐⭐⭐ **A DIVISÃO DO CANVAS** — uma vista, ou as quatro.
    pub(super) split: crate::field3d_layout::Split,
    /// ⭐⭐ **UMA CÂMERA POR QUADRANTE.**
    ///
    /// ⛔⛔ **A do ACTIVO está VELHA aqui de propósito** — quem manda nela é a
    /// [`Self::camera`], que tem ~30 leitores neste módulo e todos querem
    /// sempre a vista em que a mão está. Guardar e pegar acontece num sítio só
    /// ([`Sculpt3dScene::set_active_vp`]) e ler por uma porta só
    /// ([`Sculpt3dScene::cam_of`]). Ver a nota do [`viewports`].
    pub(super) vp_cams: Vec<ph2d_mesh_render::Camera3d>,
    /// Qual quadrante recebe o gesto e o chrome.
    pub(super) vp_active: usize,
    /// A costura agarrada — `(vertical, horizontal)`.
    pub(super) seam_drag: Option<(bool, bool)>,
    /// ⭐⭐ **O CHIP CLICÁVEL do rótulo de cada quadrante**, publicado por quem
    /// pinta.
    ///
    /// ⚠️ **Publicado e não estimado**: a largura do chip é a do TEXTO, e só o
    /// pintor a mede ([`ph2d_text::TextSystem::prefix_width`]). É a mesma lei do
    /// `nav_safe` e da `canvas` — *«ainda não desenhei» e «o ponto não é meu» são
    /// a mesma resposta*.
    pub(super) vp_labels: Vec<Option<EditorRect>>,
    /// Que quadrante tem o menu de vistas aberto.
    pub(super) view_menu: Option<usize>,
    /// O rectângulo do menu, publicado por quem o pinta.
    pub(super) view_menu_rect: Option<EditorRect>,
    /// A alça do gizmo de transformação sob o cursor — só realce.
    pub(super) gizmo_hot: Option<crate::field3d_gizmo::Handle>,
    /// ⭐⭐ **A alça AGARRADA** — o que prende o gesto a um eixo ou a um plano.
    ///
    /// ⚠️ `None` **não** é «nada a fazer»: é o transform LIVRE, que é o gesto
    /// modal que este módulo sempre teve. Ver [`Sculpt3dScene::constrain`].
    pub(super) gizmo_grip: Option<crate::field3d_gizmo::Handle>,
    /// ⭐⭐⭐ **A ÁREA DO CANVAS 3D**, publicada pelo quadro.
    ///
    /// ⛔ **Ela substituiu o campo `viewport`, que era escrito com o tamanho da
    /// JANELA** — e por isso a peça era desenhada por baixo dos painéis e das
    /// réguas, o mesmo defeito que o Enio reportou ao módulo vizinho em 31/08.
    /// O tamanho da vista passa a ser DERIVADO ([`Sculpt3dScene::viewport`]).
    pub(super) canvas: Option<ph2d_editor::zones::Rect>,
    /// ⭐ **ONDE O GIZMO DE NAVEGAÇÃO MORA** — a área do canvas e a parte dela
    /// que a moldura do app não tapa, publicadas pelo desenho.
    ///
    /// ⚠️ **Publicadas e não derivadas aqui**, pelo motivo do [`Self::viewport`]:
    /// o ponteiro corre fora do quadro e não conhece nem o layout nem os
    /// painéis que estão abertos. `None` até o primeiro desenho, e aí o gizmo
    /// simplesmente não recebe gesto nenhum.
    pub(super) nav_safe: Option<ph2d_editor::zones::Rect>,
    /// A bola sob o cursor no último quadro — só realce.
    pub(super) nav_hot: Option<crate::field3d_views::Standard>,
    /// O arrasto em curso no gizmo de navegação, se houver.
    pub(super) nav_drag: Option<super::navball::NavDrag>,
}

impl Janela {
    /// A janela com que uma cena nasce: **uma** vista, com a câmera do artista.
    ///
    /// ⚠️ **A lista não nasce vazia** — o [`Sculpt3dScene::note_canvas`] só a
    /// reconstrói quando a CONTAGEM muda, e uma lista vazia contra um
    /// `Split::One` (que conta `1`) nunca dispararia essa reconstrução: o
    /// `cam_of` cairia sempre no ramo de recurso e a divisão seria muda.
    pub(crate) fn com(camera: ph2d_mesh_render::Camera3d) -> Self {
        Self {
            vp_cams: vec![camera],
            ..Self::default()
        }
    }
}

impl Sculpt3dScene {
    /// ⭐⭐ **O QUADRO PUBLICA A ÁREA DO CANVAS 3D** — o que sobra do ecrã depois
    /// do chrome e das réguas.
    ///
    /// ⚠️ **Publicada e não derivada aqui**: quem conhece o layout, os painéis
    /// abertos e as réguas é o quadro, e o ponteiro corre fora dele. A porta é a
    /// mesma que alimenta o módulo vizinho ([`crate::canvas_area::visible`]),
    /// **de propósito** — dois rects seriam a fonte por onde a imagem e a
    /// moldura voltam a discordar.
    pub(crate) fn note_canvas(&mut self, area: EditorRect) {
        self.janela.canvas = Some(area);
        // A lista segue a divisão. ⚠️ **Só quando a CONTAGEM muda** — refazê-la
        // todo quadro deitaria fora as três câmeras que o artista posicionou.
        let n = self.janela.split.count();
        if self.janela.vp_cams.len() != n {
            self.rebuild_viewports(n, self.janela.vp_active);
        }
    }

    /// Quantos viewports a divisão tem — **a fonte da contagem** é o [`Split`].
    pub(crate) fn vp_count(&self) -> usize {
        self.janela.split.count()
    }

    /// Qual quadrante está activo.
    pub(crate) fn vp_active(&self) -> usize {
        self.janela.vp_active.min(self.vp_count().saturating_sub(1))
    }

    /// ⭐ **O rectângulo de cada viewport**, em coordenadas de JANELA.
    ///
    /// ⚠️ Sai da porta do módulo vizinho, que garante que os quatro **ladrilham
    /// a área exactamente** — sem folga, sem sobreposição, e a soma das larguras
    /// é a largura.
    pub(crate) fn vp_rects(&self) -> Vec<EditorRect> {
        let Some(area) = self.janela.canvas else {
            return Vec::new();
        };
        crate::field3d_layout::rects(area, self.janela.split)
            .as_slice()
            .to_vec()
    }

    /// O rectângulo do viewport `i`, ou `None`.
    pub(crate) fn vp_rect(&self, i: usize) -> Option<EditorRect> {
        self.vp_rects().get(i).copied()
    }

    /// ⭐⭐ **A CÂMERA DE UM VIEWPORT** — a porta única. Ver a nota do módulo:
    /// a do activo é a [`Sculpt3dScene::camera`], e a da lista está velha.
    pub(crate) fn cam_of(&self, i: usize) -> Camera3d {
        if i == self.vp_active() {
            self.camera
        } else {
            self.janela.vp_cams.get(i).copied().unwrap_or(self.camera)
        }
    }

    /// ⭐⭐ **ESCOLHE O QUADRANTE ACTIVO** — guarda a câmera do que sai e pega a
    /// do que entra. É o **único** escritor da invariante.
    pub(crate) fn set_active_vp(&mut self, i: usize) {
        let n = self.vp_count();
        if i >= n || i == self.vp_active() {
            return;
        }
        let sai = self.vp_active();
        if let Some(slot) = self.janela.vp_cams.get_mut(sai) {
            *slot = self.camera;
        }
        self.janela.vp_active = i;
        self.camera = self.janela.vp_cams.get(i).copied().unwrap_or(self.camera);
    }

    /// ⭐ **De quem é este ponto da janela** — `None` fora do canvas.
    pub(crate) fn vp_at(&self, x: f32, y: f32) -> Option<usize> {
        crate::field3d_layout::hit(self.vp_rects(), [x, y])
    }

    /// ⭐⭐⭐ **ABRE E FECHA A DIVISÃO.** Devolve `true` se ficou aberta.
    ///
    /// ⚠️ **Ao abrir, a vista do artista vai para o quadrante de BAIXO À
    /// DIREITA** e as outras três nascem nomeadas — a lei do Blender, e a certa:
    /// é onde a mão dele já está. Ao **fechar**, fica a que estava activa, nunca
    /// «a primeira»: o artista fecha a olhar para o quadrante que lhe interessa,
    /// e ficar com outro seria desfazer-lhe o gesto.
    pub(crate) fn toggle_split(&mut self) -> bool {
        self.janela.split = match self.janela.split {
            Split::One => Split::quad(),
            Split::Quad { .. } => Split::One,
        };
        let n = self.janela.split.count();
        let aberta = n > 1;
        // A câmera que o artista tinha vai com ele: para o quadrante 3 ao abrir,
        // e para o único ao fechar.
        //
        // ⛔⛔ **O activo entra na RECONSTRUÇÃO, e não depois dela** — a primeira
        // redacção fazia `rebuild` e **só então** `vp_active = n − 1`, o que deixava
        // a `camera` (que é a do activo, por invariante) a apontar para o quadrante
        // `0`: abrir a divisão punha a vista de **TOPO** no canto do artista, e o
        // gate leu `[Top, Right, Front, Top]`. *Uma invariante com dois
        // escritores tem de ser escrita numa transacção só.*
        self.rebuild_viewports(n, if aberta { n - 1 } else { 0 });
        aberta
    }

    /// ⭐ **A LISTA SEGUE A DIVISÃO** — e este é o único sítio que a reconstrói.
    ///
    /// ⚠️ **As nomeadas nascem apontadas e a do artista é a que ele tinha.**
    /// Reconstruir por `push` poria a perspectiva no canto de cima à esquerda,
    /// onde a disposição do Blender põe o *Top*.
    fn rebuild_viewports(&mut self, n: usize, activo: usize) {
        let artista = self.camera;
        self.janela.vp_cams = (0..n)
            .map(|i| {
                self.janela.split.named(i).map_or(artista, |v| {
                    let mut c = artista;
                    let (yaw, pitch) = super::navball::aim_of(v);
                    c.aim(yaw, pitch);
                    c
                })
            })
            .collect();
        self.janela.vp_active = activo.min(n - 1);
        self.camera = self.janela.vp_cams[self.janela.vp_active];
    }

    /// ⭐⭐ **O QUADRO PUBLICA OS CHIPS DOS RÓTULOS** — um por quadrante, na
    /// ordem deles. Vazio quando não há rótulos (uma vista só).
    pub(crate) fn note_view_labels(&mut self, chips: Vec<Option<EditorRect>>) {
        self.janela.vp_labels = chips;
    }

    /// ⭐ **A ÁREA DO CANVAS 3D**, se o quadro já a publicou.
    ///
    /// ⚠️ Ela nasceu, morreu como código morto e **voltou com um consumidor**: o
    /// menu da vista precisa dela para ficar preso ao canvas — o chip do
    /// quadrante de baixo-direita está a poucos pixels do canto, e um menu que
    /// descesse dali sairia da janela.
    pub(crate) fn canvas(&self) -> Option<EditorRect> {
        self.janela.canvas
    }

    /// O chip do rótulo daquele quadrante, se ele foi pintado.
    pub(crate) fn chip_of(&self, i: usize) -> Option<EditorRect> {
        self.janela.vp_labels.get(i).copied().flatten()
    }

    /// Que quadrante tem o menu aberto, se algum.
    pub(crate) fn view_menu_open(&self) -> Option<usize> {
        self.janela.view_menu
    }

    /// O quadro publica onde o menu foi pintado.
    pub(crate) fn note_view_menu_rect(&mut self, r: EditorRect) {
        self.janela.view_menu_rect = Some(r);
    }

    /// O chip do rótulo sob este ponto, se houver.
    pub(crate) fn chip_at(&self, x: f32, y: f32) -> Option<usize> {
        self.janela
            .vp_labels
            .iter()
            .position(|c| c.is_some_and(|r| x >= r.x && y >= r.y && x < r.x + r.w && y < r.y + r.h))
    }

    /// ⭐ **Abre o menu daquele quadrante.**
    pub(crate) fn open_view_menu(&mut self, i: usize) {
        self.janela.view_menu = Some(i);
    }

    /// Fecha-o. `true` se havia um aberto — é o que faz o `Escape` só ser dele
    /// enquanto ele está aberto.
    pub(crate) fn close_view_menu(&mut self) -> bool {
        self.janela.view_menu.take().is_some()
    }

    /// ⭐⭐⭐ **O CLIQUE COM O MENU ABERTO** — dentro escolhe, fora fecha, e nos
    /// dois casos ele é **consumido**.
    ///
    /// ⚠️ **Deixar o clique de fora passar orbitaria a peça no mesmo gesto em que
    /// o artista só queria desistir do menu** — e é o que todo o chrome desta
    /// casa já faz.
    ///
    /// ⚠️ **A vista vai para o quadrante que ABRIU o menu, e ele é o ACTIVO** —
    /// o chip é a única porta que o abre, e ela acerta o activo antes. Trocar a
    /// vista do quadrante errado seria pior do que não ter menu.
    pub(crate) fn view_menu_click(&mut self, x: f32, y: f32) -> bool {
        let Some(i) = self.janela.view_menu.take() else {
            return false;
        };
        let escolha = self
            .janela
            .view_menu_rect
            .and_then(|m| crate::field3d_view_menu::row_at(m, [x, y]));
        if let Some(v) = escolha {
            debug_assert_eq!(
                i,
                self.vp_active(),
                "o menu so' se abre pelo chip, e o chip acerta o activo"
            );
            self.aim_view(v);
        }
        true
    }

    /// ⭐⭐⭐ **O TAMANHO DO VIEWPORT ACTIVO** — o que o campo `viewport` era.
    ///
    /// ⚠️ **Derivado, e a mudança é uma CURA:** o campo era escrito com o tamanho
    /// da JANELA, então o pick e o desenho concordavam um com o outro e
    /// discordavam do artista — a peça caía por baixo dos painéis e das réguas.
    /// `(1, 1)` antes do primeiro desenho, que é o que o campo também fazia.
    pub(crate) fn viewport(&self) -> (u32, u32) {
        self.vp_rect(self.vp_active()).map_or((1, 1), |r| {
            ((r.w.round().max(1.0)) as u32, (r.h.round().max(1.0)) as u32)
        })
    }

    /// A quina superior esquerda do viewport activo, em coordenadas de janela.
    pub(crate) fn vp_origin(&self) -> (f32, f32) {
        self.vp_rect(self.vp_active())
            .map_or((0.0, 0.0), |r| (r.x, r.y))
    }

    /// ⭐⭐ **UM PONTO DA JANELA, NO REFERENCIAL DO VIEWPORT ACTIVO.**
    ///
    /// ⚠️ **Porta, e ela é a metade que faz o clique cair no sítio certo.** A
    /// [`Camera3d::ray_through`] recebe pixels **daquela vista**, e passar-lhe a
    /// coordenada de janela crua é exactamente o defeito *«o lugar onde o mouse
    /// toca não corresponde ao local na malha»* — só que agora deslocado pela
    /// quina do quadrante em vez de por zero.
    pub(super) fn to_view(&self, x: f32, y: f32) -> (f32, f32) {
        let (ox, oy) = self.vp_origin();
        (x - ox, y - oy)
    }

    /// ⭐ **ONDE UM PONTO DO MUNDO CAI NA JANELA** — o inverso do [`Self::to_view`]
    /// composto com a projecção da câmera.
    ///
    /// ⚠️ Ela existe porque **todo** consumidor da projecção neste módulo (o anel
    /// do cursor, as três medições do transform, o diagnóstico do pick) desenha
    /// ou compara em coordenadas de JANELA. Somar a quina em cada um seria a
    /// mesma linha escrita seis vezes, e a que faltasse produziria um gizmo
    /// deslocado só no quadrante que não é o de cima à esquerda.
    pub(super) fn project_window(&self, p: [f32; 3]) -> Option<(f32, f32)> {
        let (ox, oy) = self.vp_origin();
        let (sx, sy) = self.camera.project(p, self.viewport())?;
        Some((sx + ox, sy + oy))
    }

    /// O rectângulo de um viewport, no vocabulário do renderizador.
    ///
    /// ⚠️ **As arestas vêm já inteiras da porta do layout** — arredondar aqui
    /// seria a segunda conversão, e é entre duas conversões que nasce a fenda de
    /// um pixel entre o que se desenha e o que se recorta.
    pub(crate) fn vp_screen(&self, i: usize) -> Option<ScreenRect> {
        let r = self.vp_rect(i)?;
        Some(ScreenRect {
            x: r.x.max(0.0) as u32,
            y: r.y.max(0.0) as u32,
            w: r.w.max(0.0) as u32,
            h: r.h.max(0.0) as u32,
        })
    }

    /// A chave i18n do rótulo de um viewport — a vista nomeada, ou *User*.
    ///
    /// ⚠️ **Derivada da CÂMERA, nunca do quadrante**: orbitar a vista de cima faz
    /// dela *User*, que é o que ela passou a ser. *Um rótulo preso ao sítio
    /// mentiria assim que a mão tocasse na vista, e mentiria em silêncio.*
    pub(crate) fn vp_label_key(&self, i: usize) -> &'static str {
        match super::navball::named_view(&self.cam_of(i)) {
            Some(Standard::Front) => "viewport.model3d.view.front",
            Some(Standard::Back) => "viewport.model3d.view.back",
            Some(Standard::Right) => "viewport.model3d.view.right",
            Some(Standard::Left) => "viewport.model3d.view.left",
            Some(Standard::Top) => "viewport.model3d.view.top",
            Some(Standard::Bottom) => "viewport.model3d.view.bottom",
            None => "viewport.model3d.view.user",
        }
    }

    /// ⭐ **A COSTURA SOB O PONTEIRO** — que cursor pedir, ou `None`.
    pub(crate) fn seam_cursor(&self, x: f32, y: f32) -> Option<winit::window::CursorIcon> {
        // ⛔⛔ **Com o menu aberto a costura é MUDA** — o chip do quadrante de
        // baixo-direita nasce encostado ao cruzamento, então o menu que ele abre
        // cai por cima da banda de agarrar o divisor. Se o cursor continuasse a
        // ser a seta de redimensionar por cima das linhas, *a tela prometeria um
        // gesto que ali já não existe*: um ponteiro que mente é um controlo
        // morto ao contrário — ele anuncia o que a mão NÃO vai conseguir fazer.
        // Lei do vizinho (`field3d_viewports::divider_cursor`), lida e não
        // re-decidida.
        if self.janela.view_menu.is_some() {
            return None;
        }
        crate::field3d_layout::seam_cursor(self.janela.canvas?, self.janela.split, [x, y])
    }

    /// O pen-down agarrou uma costura? Devolve `true` se sim.
    pub(crate) fn seam_grab(&mut self, x: f32, y: f32) -> bool {
        let Some(area) = self.janela.canvas else {
            return false;
        };
        self.janela.seam_drag = crate::field3d_layout::seam_grab(area, self.janela.split, [x, y]);
        self.janela.seam_drag.is_some()
    }

    /// O dedo andou com uma costura na mão.
    pub(crate) fn seam_at(&mut self, x: f32, y: f32) -> bool {
        // ⚠️ **`(vertical, horizontal)`, nesta ordem** — a costura VERTICAL move o
        // `tx` e a HORIZONTAL move o `ty`. A primeira redacção trocou-os, e o
        // sintoma seria arrastar a linha de cima e ver a do lado mexer-se.
        let (Some(area), Some((vert, horiz))) = (self.janela.canvas, self.janela.seam_drag) else {
            return false;
        };
        let (tx, ty) = crate::field3d_layout::t_at(area, [x, y]);
        let Split::Quad { tx: cx, ty: cy } = self.janela.split else {
            return false;
        };
        self.janela.split = Split::Quad { tx: cx, ty: cy }
            .with_t(if vert { tx } else { cx }, if horiz { ty } else { cy });
        true
    }

    /// Larga a costura. `true` se havia uma na mão.
    pub(crate) fn seam_release(&mut self) -> bool {
        self.janela.seam_drag.take().is_some()
    }
}

#[cfg(test)]
#[path = "sculpt3d_viewports_tests.rs"]
mod tests;

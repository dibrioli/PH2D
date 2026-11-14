//! ⭐ **O cromo de um painel-BANDA** (Timeline, `docs/CartoesUI/14_CHROME_TIMELINE_MOTION.md`): a moldura
//! da lista sem cartão por dentro, porque a altura é do trabalho, e os controlos da lista em fluxo, numa
//! fileira: o campo com o rótulo AO LADO (o de cima custa uma linha de faixa), a caixa T64, os segmentos,
//! a fila de ícones, o chip. Os ids, o store e o despacho são os do painel.

use super::*;
use crate::widget::card_field::paint_field;
use crate::widget::key_mark::Rotulo;
use crate::widget::paint_card_chip;
pub use ph2d_card_stack::metrics;

/// A moldura pintada: a lista, a tira do cabeçalho (depois do título, antes do «×») e o interior.
#[derive(Copy, Clone, Debug)]
pub struct Moldura {
    pub lista: Rect,
    pub cab: Rect,
    pub corpo: Rect,
    /// A caixa do ícone encostado à esquerda do «×» (o «encaixar» da Timeline).
    pub vizinha_do_fecho: Rect,
}

/// A geometria de [`pinta_moldura`] sem pintar: quem resolve gestos antes da pintura precisa dela.
#[must_use]
pub fn geometria(slot: Rect) -> Moldura {
    let lista = Rect::new(
        slot.x + m::LIST_MARGIN,
        slot.y + m::LIST_MARGIN,
        (slot.w - m::LIST_MARGIN * 2.0).max(0.0),
        (slot.h - m::LIST_MARGIN * 2.0).max(0.0),
    );
    let topo = lista.y + m::LIST_HEAD_H;
    let corpo = Rect::new(
        lista.x + m::LIST_PAD_X,
        topo,
        (lista.w - m::LIST_PAD_X * 2.0).max(0.0),
        (lista.y + lista.h - m::LIST_PAD_BOTTOM - topo).max(0.0),
    );
    let cab = Rect::new(
        corpo.x,
        lista.y + (m::LIST_HEAD_H - m::FIELD_H) * 0.5,
        corpo.w,
        m::FIELD_H,
    );
    let vizinha_do_fecho =
        menu::vizinha_do_fecho(Rect::new(lista.x, lista.y, lista.w, m::LIST_HEAD_H));
    Moldura {
        lista,
        cab,
        corpo,
        vizinha_do_fecho,
    }
}

/// A moldura flutuante, o título e o «×» (`mark_card_list_close`), como numa lista de cartões. Sem o
/// «…»: as linhas dele repõem cartões, e aqui não há cartão nenhum (doc 14).
pub fn pinta_moldura(ctx: &mut PaintCtx, panel: NodeId, title: &str, slot: Rect) -> Moldura {
    dicas::comeca(panel);
    let theme = ctx.host.theme();
    let mut g = geometria(slot);
    let lista = g.lista;
    pinta_pilula(ctx, lista);
    let tx = lista.x + m::LIST_PAD_X + m::LIST_HEAD_PAD_X;
    let tw = pinta_titulo_da_lista(
        ctx.scene,
        ctx.text_system,
        theme,
        lista,
        title,
        lista.w - m::LIST_PAD_X * 2.0 - m::HEAD_ICON_BOX,
    );
    menu::pinta_fecho(
        ctx,
        panel,
        theme,
        Rect::new(lista.x, lista.y, lista.w, m::LIST_HEAD_H),
    );
    let x = tx + tw + m::CARD_GAP;
    let fim = lista.x + lista.w - m::LIST_PAD_X - m::HEAD_ICON_BOX - m::FIELD_GAP;
    g.cab.x = x;
    g.cab.w = (fim - x).max(0.0);
    g
}

/// Uma pílula flutuante: a tinta e a sombra da lista, sem título (a barra de um canvas).
pub fn pinta_pilula(ctx: &mut PaintCtx, r: Rect) {
    let theme = ctx.host.theme();
    pinta_pilula_na_cena(ctx.scene, theme, r);
}

/// [`pinta_pilula`] para quem só tem a cena (uma janela da shell).
pub fn pinta_pilula_na_cena(scene: &mut VectorScene, theme: Theme, r: Rect) {
    shadow(
        scene,
        r,
        m::LIST_RADIUS,
        resolve(ColorToken::KanbanListShadow, theme),
        m::LIST_SHADOW_FAR,
    );
    shadow(
        scene,
        r,
        m::LIST_RADIUS,
        resolve(ColorToken::KanbanShadowNear, theme),
        m::LIST_SHADOW_NEAR,
    );
    fill_rounded_rect(
        scene,
        r,
        m::LIST_RADIUS,
        resolve(ColorToken::KanbanList, theme),
    );
}

/// O título da lista em `lista`, no tamanho, tinta e tracking dela. Devolve a largura pintada.
pub fn pinta_titulo_da_lista(
    scene: &mut VectorScene,
    text: &mut TextSystem,
    theme: Theme,
    lista: Rect,
    title: &str,
    max_w: f32,
) -> f32 {
    let nome = title.to_uppercase();
    let tx = lista.x + m::LIST_PAD_X + m::LIST_HEAD_PAD_X;
    let (corpo, em) = titulo_que_cabe(text, &nome, max_w.max(0.0));
    let tracking = corpo * em * nome.chars().count() as f32;
    let tw = text.prefix_width(&nome, corpo) + tracking;
    crate::paint::paint_text_title_elided_tracked(
        text,
        scene,
        &nome,
        [
            tx,
            lista.y + crate::paint::folga_do_texto(m::LIST_HEAD_H, corpo),
        ],
        corpo,
        max_w.max(0.0),
        resolve(ColorToken::KanbanListFg, theme),
        em,
    );
    tw
}

/// O corpo e o espaçamento com que o título de uma lista cabe em `w`: numa coluna estreita ele perde
/// primeiro o espaçamento e depois desce ao corpo do rótulo, antes de se cortar («COLOR EQUALIZATI…»).
pub(crate) fn titulo_que_cabe(text: &mut TextSystem, nome: &str, w: f32) -> (f32, f32) {
    let peso = ph2d_text::FontWeight::SEMI_BOLD;
    let degraus = [
        (m::LIST_TITLE_PX, m::LIST_TITLE_TRACKING_EM),
        (m::LIST_TITLE_PX, 0.0),
        (m::LABEL_PX, 0.0),
    ];
    degraus
        .into_iter()
        .find(|&(px, em)| crate::text_elide::coube_tracked(text, nome, px, w, (peso, em)))
        .unwrap_or((m::LABEL_PX, 0.0))
}

#[cfg(test)]
#[test]
fn numa_coluna_estreita_o_titulo_desce_antes_de_se_cortar() {
    let mut ts = TextSystem::without_system_fonts();
    let nome = "COLOR EQUALIZATION";
    assert_eq!(
        titulo_que_cabe(&mut ts, nome, 400.0),
        (m::LIST_TITLE_PX, m::LIST_TITLE_TRACKING_EM),
        "com lugar, o título de sempre"
    );
    let cheio = ts.prefix_width(nome, m::LIST_TITLE_PX);
    let (corpo, em) = titulo_que_cabe(&mut ts, nome, cheio - 1.0);
    assert_eq!((corpo, em), (m::LABEL_PX, 0.0), "o último degrau");
    let peso = ph2d_text::FontWeight::SEMI_BOLD;
    assert!(crate::text_elide::coube_tracked(
        &mut ts,
        nome,
        corpo,
        cheio - 1.0,
        (peso, em)
    ));
}

/// Um verbo do cabeçalho de uma janela sem lista (a Preview): o ícone da lista, hover incluído, e o alvo `r`.
pub fn pinta_icone_de_cabecalho(ctx: &mut PaintCtx, id: NodeId, icon: IconId, r: Rect) {
    let theme = ctx.host.theme();
    if ctx.host.store().hover_live(id) > 0.0 {
        fill_rounded_rect(
            ctx.scene,
            r,
            m::HEAD_ICON_RADIUS,
            resolve(ColorToken::CardFieldHover, theme),
        );
    }
    ctx.host.hit_index_mut().register(id, r);
    dicas::icone(id);
    let g = m::HEAD_ICON_GLYPH;
    paint_icon(
        ctx.scene,
        icon,
        Rect::new(r.x + (r.w - g) * 0.5, r.y + (r.h - g) * 0.5, g, g),
        resolve(ColorToken::KanbanListFg, theme),
        StrokeToken::Default.px(),
    );
}

/// Um ícone de uma fila: `id: None` = apagado, sem alvo nenhum (o gesto não existe agora).
#[derive(Copy, Clone, Debug)]
pub struct Icone {
    pub id: Option<NodeId>,
    pub icon: IconId,
    pub on: bool,
}

/// O que a 1.ª fileira do corpo sobe, para o vão até à do cabeçalho ser o das outras fileiras.
pub const SOBRA_DO_CABECALHO: f32 = (m::LIST_HEAD_H - m::FIELD_H) * 0.5 - m::FIELD_ROW_GAP;

/// O vão entre a última fileira e a folha: a altura de faixa é o recurso do painel-banda (doc 14).
pub const VAO_ATE_A_FOLHA: f32 = m::SEG_RADIUS;

/// A largura do separador vertical entre dois grupos de uma fileira: o fio do campo. A folga dos dois
/// lados é o `FIELD_GAP` do fluxo.
pub const SEPARADOR_W: f32 = m::FIELD_BORDER;
/// A altura do fio: ~64 % do campo, centrado.
const SEPARADOR_H: f32 = m::FIELD_H - m::SEG_ICON_GAP * 2.0;

/// A largura de uma fila de `n` ícones.
#[must_use]
pub fn icones_w(n: usize) -> f32 {
    m::FIELD_H * n as f32
}

/// A largura da caixa de um campo em fileira: pega + valor.
const CAMPO_W: f32 = m::ICON_CELL + m::FIELD_VALUE_MIN_W;
/// A caixa de [`CardCtx::campo_compacto`]: 20 % mais estreita que [`CAMPO_W`] (dono 12/11, Time e Frame
/// da Timeline). 65,6 px: cabem «352.73» e «8466» no valor.
pub const CAMPO_COMPACTO_W: f32 = CAMPO_W * 0.8; // LITERAL-PX-OK: a fração pedida pelo dono

/// O glifo de um sinal de fileira (o anel do Record, a onda do fps, o «+» de um botão).
const SINAL: f32 = m::KEY_MARK_BOX;
/// O ponto cheio de um sinal: dois terços do glifo, como o ponto da maquete B1 da Timeline.
const PONTO: f32 = m::KEY_MARK_BOX - m::SEG_RADIUS;

/// Um sinal de cor numa fileira de banda: um ícone, ou um ponto cheio, na tinta `ColorToken`.
#[derive(Copy, Clone, Debug)]
pub enum Sinal {
    Icone(IconId, ColorToken),
    Ponto(ColorToken),
}

/// ⭐ Uma caixa de marcar EMOLDURADA (a maquete B1 da Timeline, 09/11): moldura arredondada, o
/// quadradinho, um sinal antes do nome e/ou no fim; a moldura inteira é o alvo do `Toggle` `id`.
#[derive(Copy, Clone, Debug)]
pub struct Emoldurada<'a> {
    pub id: NodeId,
    pub nome: &'a str,
    pub on: bool,
    pub antes: Option<Sinal>,
    pub fim: Option<Sinal>,
}

impl CardCtx<'_> {
    fn pinta_sinal(&mut self, s: Sinal, x: f32, meio: f32) {
        match s {
            Sinal::Icone(icon, cor) => paint_icon(
                self.scene,
                icon,
                Rect::new(x, meio - SINAL * 0.5, SINAL, SINAL),
                self.ink.color(cor),
                StrokeToken::Default.px(),
            ),
            Sinal::Ponto(cor) => fill_rounded_rect(
                self.scene,
                Rect::new(x + (SINAL - PONTO) * 0.5, meio - PONTO * 0.5, PONTO, PONTO),
                PONTO * 0.5,
                self.ink.color(cor),
            ),
        }
    }

    /// A largura natural de [`Self::emoldurada_em`].
    pub fn emoldurada_w(&mut self, c: &Emoldurada<'_>) -> f32 {
        let sinais = [c.antes, c.fim].iter().flatten().count() as f32;
        m::FIELD_PAD_R * 2.0
            + m::CHECK_BOX
            + m::CHECK_GAP
            + self.largura_do_texto(c.nome, m::VALUE_PX)
            + sinais * (SINAL + m::SEG_ICON_GAP)
    }

    /// Pinta a caixa emoldurada em `(x, y)`, à altura do campo.
    pub fn emoldurada_em(&mut self, c: &Emoldurada<'_>, x: f32, y: f32) {
        let r = Rect::new(x, y, self.emoldurada_w(c), m::FIELD_H);
        self.hit_index.register(c.id, r);
        dicas::rotulo(c.id, c.nome);
        let hover = self.store.hover_live(c.id) > 0.0;
        let fundo = if hover {
            ColorToken::CardFieldHover
        } else {
            ColorToken::CardField
        };
        fill_rounded_rect(self.scene, r, m::FIELD_RADIUS, self.ink.color(fundo));
        let valor = if c.on {
            crate::widget::CheckboxValue::Checked
        } else {
            crate::widget::CheckboxValue::Unchecked
        };
        let mut cx = x + m::FIELD_PAD_R;
        let caixa = crate::widget::card_check_rect(cx, y, m::FIELD_H);
        crate::widget::paint_card_check_value(self.scene, self.ink, caixa, valor, hover);
        cx += m::CHECK_BOX + m::CHECK_GAP;
        let meio = y + m::FIELD_H * 0.5;
        if let Some(s) = c.antes {
            self.pinta_sinal(s, cx, meio);
            cx += SINAL + m::SEG_ICON_GAP;
        }
        let tw = self.largura_do_texto(c.nome, m::VALUE_PX);
        crate::paint::paint_text_elided(
            self.text_system,
            self.scene,
            c.nome,
            cx,
            y + crate::paint::folga_do_texto(m::FIELD_H, m::VALUE_PX),
            m::VALUE_PX,
            tw,
            self.ink.color(ColorToken::KanbanCardFg),
        );
        if let Some(s) = c.fim {
            self.pinta_sinal(s, cx + tw + m::SEG_ICON_GAP, meio);
        }
    }

    /// Um separador vertical de [`SEPARADOR_W`] em `(x, y)` (topo da fileira): o fio forte do tema
    /// (`BorderStrong`: o `Border` de lista vira `CardField`, que some a 1 px), centrado na altura do campo. Só pintura, sem alvo.
    pub fn separador_em(&mut self, x: f32, y: f32) {
        fill_rounded_rect(
            self.scene,
            Rect::new(
                x,
                y + (m::FIELD_H - SEPARADOR_H) * 0.5,
                SEPARADOR_W,
                SEPARADOR_H,
            ),
            SEPARADOR_W * 0.5,
            self.ink.color(ColorToken::BorderStrong),
        );
    }

    /// A largura natural de [`Self::rotulo_com_sinal`].
    pub fn rotulo_com_sinal_w(&mut self, texto: &str) -> f32 {
        SINAL + m::SEG_ICON_GAP + self.largura_do_texto(texto, m::VALUE_PX)
    }

    /// Um rótulo só de leitura com um sinal à esquerda (a onda do «24 fps · project»).
    pub fn rotulo_com_sinal(&mut self, sinal: Sinal, texto: &str, x: f32, y: f32) {
        self.pinta_sinal(sinal, x, y + m::FIELD_H * 0.5);
        let tx = x + SINAL + m::SEG_ICON_GAP;
        let tw = self.largura_do_texto(texto, m::VALUE_PX);
        crate::paint::paint_text(
            self.text_system,
            self.scene,
            texto,
            tx,
            y + crate::paint::folga_do_texto(m::FIELD_H, m::VALUE_PX),
            m::VALUE_PX,
            tw,
            self.ink.color(ColorToken::CardLabel),
        );
    }

    /// A largura natural de [`Self::botao_com_icone_em`].
    pub fn botao_com_icone_w(&mut self, texto: &str) -> f32 {
        self.botao_w(texto) + SINAL + m::SEG_ICON_GAP
    }

    /// [`Self::botao_em`] com um ícone antes do texto, na tinta do texto.
    pub fn botao_com_icone_em(&mut self, id: NodeId, icon: IconId, texto: &str, r: Rect) {
        self.hit_index.register(id, r);
        dicas::rotulo(id, texto);
        let fundo = if self.store.hover_live(id) > 0.0 {
            ColorToken::CardFieldHover
        } else {
            ColorToken::CardField
        };
        fill_rounded_rect(self.scene, r, m::FIELD_RADIUS, self.ink.color(fundo));
        let x = r.x + m::FIELD_PAD_R;
        self.pinta_sinal(
            Sinal::Icone(icon, ColorToken::KanbanCardFg),
            x,
            r.y + r.h * 0.5,
        );
        let tx = x + SINAL + m::SEG_ICON_GAP;
        crate::paint::paint_text_elided(
            self.text_system,
            self.scene,
            texto,
            tx,
            r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
            m::VALUE_PX,
            (r.x + r.w - tx).max(0.0),
            self.ink.color(ColorToken::KanbanCardFg),
        );
    }
}

impl CardCtx<'_> {
    fn largura_do_texto(&mut self, texto: &str, px: f32) -> f32 {
        self.text_system.prefix_width(texto, px).ceil()
    }

    /// A coluna de rótulos de uma família de campos: a do mais largo. Quando a barra estreita os
    /// empilha, os valores arrancam no mesmo `x` (`onde_comeca_o_valor`).
    pub fn coluna_de_rotulos(&mut self, rotulos: &[&str]) -> f32 {
        rotulos
            .iter()
            .map(|r| self.largura_do_texto(r, m::LABEL_PX))
            .fold(0.0, f32::max)
    }

    /// A largura de [`Self::campo_em_fila`] com a coluna `col`.
    #[must_use]
    pub fn campo_em_fila_w(col: f32) -> f32 {
        col + m::FIELD_GAP + CAMPO_W
    }

    /// Só a caixa do campo, [`CAMPO_COMPACTO_W`] de largura, sem o rótulo ao lado: o `glyph` (um ícone)
    /// diz o que é, e o nome fica no rótulo acessível e na dica.
    pub fn campo_compacto(&mut self, id: NodeId, nome: &str, glyph: Glyph<'_>, (x, y): (f32, f32)) {
        dicas::rotulo(id, nome);
        let r = Rect::new(x, y, CAMPO_COMPACTO_W, m::FIELD_H);
        paint_field(
            self.scene,
            self.text_system,
            self.ink,
            self.store,
            self.hit_index,
            r,
            id,
            glyph,
        );
    }

    /// O rótulo ao lado, na coluna `col`, e a caixa do campo Figma (`mark_card_field` antes).
    pub fn campo_em_fila(
        &mut self,
        id: NodeId,
        rotulo: &str,
        glyph: Glyph<'_>,
        col: f32,
        (x, y): (f32, f32),
    ) {
        dicas::rotulo(id, rotulo);
        crate::paint::paint_text(
            self.text_system,
            self.scene,
            rotulo,
            x,
            y + crate::paint::folga_do_texto(m::FIELD_H, m::LABEL_PX),
            m::LABEL_PX,
            col,
            self.ink.color(ColorToken::CardLabel),
        );
        let r = Rect::new(x + col + m::FIELD_GAP, y, CAMPO_W, m::FIELD_H);
        paint_field(
            self.scene,
            self.text_system,
            self.ink,
            self.store,
            self.hit_index,
            r,
            id,
            glyph,
        );
    }

    /// A largura natural de [`Self::marca_em_fila`]: a coluna da marca de animação é o vão seguinte.
    pub fn marca_em_fila_w(&mut self, nome: &str) -> f32 {
        corpo::pede_em_fila(self.text_system, nome)
    }

    /// A caixa em moldura numa linha da barra (o nome dentro, ao lado do ✓); o id continua o do `Toggle`.
    pub fn marca_em_fila(&mut self, id: NodeId, nome: &str, on: bool, x: f32, y: f32) {
        let w = self.marca_em_fila_w(nome);
        let valor = if on {
            crate::widget::CheckboxValue::Checked
        } else {
            crate::widget::CheckboxValue::Unchecked
        };
        corpo::check_em_fila(
            self.scene,
            self.text_system,
            self.store,
            self.hit_index,
            self.ink,
            Rect::new(x, y, w, m::FIELD_H),
            (id, Rotulo::vista_de(nome)),
            (valor, false),
        );
    }

    /// A largura natural de [`Self::segmentos_em_fila`], segmentos só de texto.
    pub fn segmentos_em_fila_w(&mut self, segs: &[Seg<'_>]) -> f32 {
        let textos: f32 = segs
            .iter()
            .map(|s| self.largura_do_texto(s.text, m::SEG_TEXT_PX) + m::SEG_INSET * 2.0)
            .sum();
        textos + m::SEG_PAD * (segs.len() + 1) as f32
    }

    /// Os segmentos à largura natural, em `(x, y)`.
    pub fn segmentos_em_fila(&mut self, segs: &[Seg<'_>], x: f32, y: f32) {
        let w = self.segmentos_em_fila_w(segs);
        let (x0, w0) = (self.x, self.w);
        (self.x, self.w, self.first) = (x, w, true);
        self.row(&[Cell::new(Widget::Choice(segs))], y);
        (self.x, self.w) = (x0, w0);
    }

    /// Ícones colados numa caixa: verbos, cada um o seu; o `Icone::on` é o estado aceso.
    pub fn icones_em_fila(&mut self, celulas: &[Icone], x: f32, y: f32) {
        self.icones_em_fila_com_traco(celulas, x, y, StrokeToken::Default);
    }

    /// [`Self::icones_em_fila`] com o traço dos glifos em `traco` (o transporte da Timeline pede o
    /// `Thick`, dono 12/11).
    pub fn icones_em_fila_com_traco(
        &mut self,
        celulas: &[Icone],
        x: f32,
        y: f32,
        traco: StrokeToken,
    ) {
        let r = m::FIELD_RADIUS;
        self.icones_em_fila_tingidos(celulas, &[], (x, y), (traco, (r, r, r, r)));
    }

    /// [`Self::icones_em_fila_com_traco`] com a tinta de cada glifo em `cores[i]` quando `Some` (o
    /// gravar vermelho e o tocar verde da Timeline, dono 13/11; sem tinta, a de sempre) e os raios
    /// da caixa `(↖, ↗, ↘, ↙)` (a aba da Timeline pede as quinas de baixo pontudas).
    pub fn icones_em_fila_tingidos(
        &mut self,
        celulas: &[Icone],
        cores: &[Option<ColorToken>],
        (x, y): (f32, f32),
        (traco, cantos): (StrokeToken, (f32, f32, f32, f32)),
    ) {
        let caixa = Rect::new(x, y, icones_w(celulas.len()), m::FIELD_H);
        crate::paint::fill_rounded_rect_radii(
            self.scene,
            caixa,
            cantos,
            self.ink.color(ColorToken::CardField),
        );
        for (i, c) in celulas.iter().enumerate() {
            let b = Rect::new(x + m::FIELD_H * i as f32, y, m::FIELD_H, m::FIELD_H);
            let hover = c.id.is_some_and(|id| self.store.hover_live(id) > 0.0);
            if let Some(id) = c.id {
                self.hit_index.register(id, b);
                dicas::icone(id);
            }
            if c.on || hover {
                let fundo = Rect::new(
                    b.x + m::SEG_PAD,
                    b.y + m::SEG_PAD,
                    b.w - m::SEG_PAD * 2.0,
                    b.h - m::SEG_PAD * 2.0,
                );
                let cor = if c.on {
                    ColorToken::KanbanCardRaised
                } else {
                    ColorToken::CardFieldHover
                };
                fill_rounded_rect(self.scene, fundo, m::SEG_RADIUS, self.ink.color(cor));
            }
            let propria = cores.get(i).copied().flatten().filter(|_| c.id.is_some());
            let tinta = if let Some(cor) = propria {
                cor
            } else if c.id.is_none() {
                ColorToken::Text3
            } else if c.on || hover {
                ColorToken::Text1
            } else {
                ColorToken::Text2
            };
            let g = m::ICON_GLYPH;
            paint_icon(
                self.scene,
                c.icon,
                Rect::new(b.x + (b.w - g) * 0.5, b.y + (b.h - g) * 0.5, g, g),
                self.ink.color(tinta),
                traco.px(),
            );
        }
    }

    /// A largura natural de [`Self::botao_em`].
    pub fn botao_w(&mut self, texto: &str) -> f32 {
        self.largura_do_texto(texto, m::VALUE_PX) + m::FIELD_PAD_R * 2.0
    }

    /// Um botão de texto do cartão, no rect que o chamador der.
    pub fn botao_em(&mut self, id: NodeId, texto: &str, r: Rect) {
        self.hit_index.register(id, r);
        let fundo = if self.store.hover_live(id) > 0.0 {
            ColorToken::CardFieldHover
        } else {
            ColorToken::CardField
        };
        fill_rounded_rect(self.scene, r, m::FIELD_RADIUS, self.ink.color(fundo));
        let tw = self.largura_do_texto(texto, m::VALUE_PX);
        crate::paint::paint_text_elided(
            self.text_system,
            self.scene,
            texto,
            r.x + ((r.w - tw) * 0.5).max(0.0),
            r.y + crate::paint::folga_do_texto(r.h, m::VALUE_PX),
            m::VALUE_PX,
            r.w,
            self.ink.color(ColorToken::KanbanCardFg),
        );
    }

    /// A cara de uma lista suspensa cujo estado e opções são do painel (o clique é o do `Dropdown`).
    pub fn lista_em(&mut self, id: NodeId, texto: &str, aberta: bool, r: Rect) {
        self.hit_index.register(id, r);
        let aceso = aberta || self.store.hover_live(id) > 0.0;
        paint_card_chip(self.scene, self.text_system, self.ink, r, texto, aceso);
    }

    /// A [`Self::lista_em`] com o ícone do escolhido à esquerda do valor.
    pub fn lista_com_icone_em(
        &mut self,
        id: NodeId,
        valor: (Option<crate::icons::IconId>, &str),
        aberta: bool,
        r: Rect,
    ) {
        self.hit_index.register(id, r);
        let aceso = aberta || self.store.hover_live(id) > 0.0;
        crate::widget::paint_card_chip_com_icone(
            self.scene,
            self.text_system,
            self.ink,
            r,
            valor,
            aceso,
        );
    }
}

#[path = "cards_banda_tons.rs"]
mod tons;
pub use tons::*;

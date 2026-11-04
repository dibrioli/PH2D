//! As linhas do [`Corpo`] dentro de um cartão: o que os ajudantes de sempre não sabem pintar lá.

use super::*;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::panel::cards::corpo::letra_da_grandeza;
use ph2d_editor_core::panel::cards::ferramenta::caixa_em_celula;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_editor_core::widget::key_mark::Rotulo;

impl Corpo<'_> {
    /// O contexto das linhas do cartão, por cima dos mesmos alvos: `None` fora de um cartão.
    pub(super) fn cartao_ctx(&mut self) -> Option<CardCtx<'_>> {
        let (ink, _): (CardInk, f32) = self.cartao?;
        Some(CardCtx::flutuante(
            &mut *self.scene,
            &mut *self.text_system,
            self.store,
            &mut *self.hit_index,
            ink,
            self.x,
            self.w,
        ))
    }

    /// A altura de um controlo num rect dado: a da caixa do cartão, ou a da linha de sempre.
    pub(super) fn lado(&self) -> f32 {
        if self.cartao.is_some() {
            m::FIELD_H
        } else {
            ROW_H_PX
        }
    }

    /// O vão antes de uma linha: a 1.ª do corpo encosta no cabeçalho.
    fn topo(&self, y: f32) -> f32 {
        match self.cartao {
            Some((_, inicio)) if y > inicio => y + m::FIELD_ROW_GAP,
            _ => y,
        }
    }

    /// As opções da ferramenta na mão, sem o que mora noutro cartão (guardados, simetria).
    pub(super) fn opcoes_no_cartao(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let nomes = [tr("panel.pixel.tip"), tr("panel.pixel.size")];
        let sec = Seccao::medida(self.text_system, 1, &nomes);
        let o = info.options;
        let mut y = self.opcoes_selecao(info, sec, y);
        if info.tool.uses_brush() {
            let formas: Vec<(&str, bool, NodeId)> = ids::SHAPES
                .iter()
                .map(|&(s, id)| {
                    (
                        tr(s.label_key()),
                        s == o.shape && info.pattern.is_none(),
                        id,
                    )
                })
                .collect();
            y = self.escolha(tr("panel.pixel.tip"), &formas, sec, y);
            y = self.padrao(info, sec, y);
            let perfeito = matches!(
                info.tool,
                PixelTool::Pencil | PixelTool::Eraser | PixelTool::Contour
            )
            .then_some((
                ids::PIXEL_PERFECT,
                tr("panel.pixel.pixel_perfect"),
                o.pixel_perfect,
            ));
            if info.pattern.is_none() {
                let mut campos = vec![(tr("panel.pixel.size"), ids::SIZE)];
                if o.shape != PixelBrushShape::Circle {
                    campos.push((tr("panel.pixel.angle"), ids::ANGLE));
                }
                y = self.campos_e_caixa(&campos, perfeito, y);
            } else if let Some(p) = perfeito {
                y = self.caixas(&[p], sec, y);
            }
        }
        self.opcoes_de_baixo(info, sec, y, false)
    }

    /// Números e uma caixa, dois a dois enquanto couberem. Ajustes de ferramenta: sem marca (D17).
    fn campos_e_caixa(
        &mut self,
        campos: &[(&str, NodeId)],
        caixa: Option<(NodeId, &str, bool)>,
        y: f32,
    ) -> f32 {
        for (_, id) in campos {
            self.store.mark_card_field_at_paint(
                *id,
                CardFieldSpec {
                    step: 1.0,
                    scrub: 1.0,
                    default: None,
                },
            );
        }
        let mut cells: Vec<Cell<'_>> = campos
            .iter()
            .map(|(nome, id)| {
                Cell::new(Widget::Field {
                    id: *id,
                    glyph: Glyph::Text(letra_da_grandeza(nome)),
                })
                .label(nome)
            })
            .collect();
        if let Some((id, nome, on)) = caixa {
            cells.push(caixa_em_celula(id, Rotulo::vista_de(nome), on));
        }
        match self.cartao_ctx() {
            Some(mut c) => c.compact(&cells, y),
            None => y,
        }
    }

    /// Uma legenda dentro do cartão, como o rótulo de um campo.
    pub(super) fn legenda_no_cartao(&mut self, texto: &str, cor: ColorToken, y: f32) -> f32 {
        let topo = y + m::LABEL_TOP;
        paint_text(
            self.text_system,
            self.scene,
            texto,
            self.x,
            topo,
            m::LABEL_PX,
            self.w.max(1.0),
            tinta(cor, self.theme),
        );
        topo + m::LABEL_PX
    }

    /// Um número por coluna, com o rótulo por cima. Ajuste de ferramenta: sem marca de animação (D17).
    pub(super) fn campos_no_cartao(&mut self, campos: &[(&str, NodeId)], y: f32) -> f32 {
        for (_, id) in campos {
            self.store.mark_card_field_at_paint(
                *id,
                CardFieldSpec {
                    step: 1.0,
                    scrub: 1.0,
                    default: None,
                },
            );
        }
        let cells: Vec<Cell<'_>> = campos
            .iter()
            .map(|(nome, id)| {
                Cell::new(Widget::Field {
                    id: *id,
                    glyph: Glyph::Text(letra_da_grandeza(nome)),
                })
                .label(nome)
            })
            .collect();
        match self.cartao_ctx() {
            Some(mut c) => c.row(&cells, y),
            None => y,
        }
    }

    /// Dois números irmãos: na mesma linha num cartão, um por linha na moldura de antes.
    pub(super) fn dois_campos(
        &mut self,
        a: (&str, NodeId),
        b: (&str, NodeId),
        sec: Seccao,
        y: f32,
    ) -> f32 {
        if self.cartao.is_some() {
            return self.campos_no_cartao(&[a, b], y);
        }
        let y = self.campo(a.0, a.1, sec, y);
        self.campo(b.0, b.1, sec, y)
    }

    /// Um número e uma escolha curta: na mesma linha num cartão, um por linha na moldura de antes.
    pub(super) fn campo_e_escolha(
        &mut self,
        campo: (&str, NodeId),
        escolha: (&str, &[(&str, bool, NodeId)]),
        sec: Seccao,
        y: f32,
    ) -> f32 {
        if self.cartao.is_none() {
            let y = self.campo(campo.0, campo.1, sec, y);
            return self.escolha(escolha.0, escolha.1, sec, y);
        }
        self.store.mark_card_field_at_paint(
            campo.1,
            CardFieldSpec {
                step: 1.0,
                scrub: 1.0,
                default: None,
            },
        );
        let segs: Vec<Seg<'_>> = escolha
            .1
            .iter()
            .map(|&(text, on, id)| Seg {
                id,
                icon: None,
                text,
                on,
            })
            .collect();
        let cells = [
            Cell::new(Widget::Field {
                id: campo.1,
                glyph: Glyph::Text(letra_da_grandeza(campo.0)),
            })
            .label(campo.0),
            Cell::new(Widget::Choice(&segs)).label(escolha.0),
        ];
        match self.cartao_ctx() {
            Some(mut c) => c.row(&cells, y),
            None => y,
        }
    }

    /// A escolha, quebrando em linhas quando não cabe; `nome` vazio = sem rótulo (o título do cartão diz).
    pub(super) fn escolha_no_cartao(
        &mut self,
        nome: &str,
        seg: &[(&str, bool, NodeId)],
        y: f32,
    ) -> f32 {
        let Some((ink, _)) = self.cartao else {
            return y;
        };
        let y = if nome.is_empty() { self.topo(y) } else { y };
        let segs: Vec<Seg<'_>> = seg
            .iter()
            .map(|&(text, on, id)| Seg {
                id,
                icon: None,
                text,
                on,
            })
            .collect();
        ph2d_editor_core::panel::cards::corpo::choices(
            &mut *self.scene,
            &mut *self.text_system,
            self.store,
            &mut *self.hit_index,
            ink,
            self.x,
            self.w,
            y,
            nome,
            &[],
            &segs,
        )
    }

    /// As caixas de marcar: ajustes de ferramenta, sem marca de animação (D17).
    pub(super) fn caixas_no_cartao(
        &mut self,
        cx: &[(NodeId, &str, bool)],
        sec: Seccao,
        y: f32,
    ) -> f32 {
        let vistas: Vec<(NodeId, Rotulo<'_>, bool)> = cx
            .iter()
            .map(|&(id, nome, on)| (id, Rotulo::vista_de(nome), on))
            .collect();
        property_row::paint_check_rows(
            self.scene,
            self.text_system,
            self.theme,
            &mut *self.hit_index,
            self.store,
            self.x,
            self.w,
            y,
            &vistas,
            sec,
        )
    }

    /// O nome de uma secção: num cartão quem o diz é o cabeçalho.
    pub(super) fn titulo(&mut self, texto: &str, y: f32) -> f32 {
        if self.cartao.is_some() {
            return y;
        }
        self.linha(texto, ColorToken::Text3, y)
    }

    /// Frente e fundo, cada um com a amostra e o hex; o Trocar.
    pub(super) fn cores_no_cartao(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let (frente, fundo) = (hex(info.fg), hex(info.bg));
        let celula = |id, rgba, text, rotulo| {
            Cell::new(Widget::Swatch {
                id,
                rgba: Some(rgba),
                text,
                edit: None,
                detach: None,
            })
            .label(rotulo)
        };
        let Some(mut c) = self.cartao_ctx() else {
            return y;
        };
        let y = c.row(
            &[
                celula(ids::PIXEL_FG, info.fg, &frente, tr("card.pixel.fg")),
                celula(ids::PIXEL_BG, info.bg, &fundo, tr("card.pixel.bg")),
            ],
            y,
        );
        let y = y + m::FIELD_ROW_GAP;
        c.full(
            None,
            Widget::Button {
                id: ids::PIXEL_SWAP,
                text: tr("panel.pixel.swap"),
                primary: false,
            },
            y,
        )
    }

    /// A cor da âncora com o *Edit color*: a amostra abre o seletor, o hex está no texto.
    pub(super) fn ancora_no_cartao(&mut self, cor: [u8; 4], y: f32) -> f32 {
        let texto = tr_with("panel.pixel.edit_color", &[("hex", &hex(cor))]);
        let y = self.topo(y);
        let Some(mut c) = self.cartao_ctx() else {
            return y;
        };
        c.full(
            None,
            Widget::Swatch {
                id: ids::PIXEL_PAL_EDIT,
                rgba: Some(cor),
                text: &texto,
                edit: None,
                detach: None,
            },
            y,
        )
    }

    /// Uma fila de verbos: em par só quando os dois cabem inteiros na metade. `None` = o rótulo não
    /// cabe nem sozinho, e quem o pinta é o caminho das duas linhas.
    pub(super) fn botoes_no_cartao(&mut self, fila: &[(NodeId, &str)], y: f32) -> Option<f32> {
        let n = fila.len().max(1) as f32;
        let meia = ((self.w - m::FIELD_GAP * (n - 1.0)) / n).max(1.0);
        let ts = &mut *self.text_system;
        let cabem = fila.iter().all(|(_, k)| {
            ts.prefix_width(tr(k), m::VALUE_PX) + m::FIELD_PAD_R + m::FIELD_PAD_R <= meia
        });
        if !cabem {
            if fila.len() == 1 {
                return None;
            }
            return Some(
                fila.iter()
                    .fold(y, |y, b| self.botoes(std::slice::from_ref(b), y)),
            );
        }
        let y = self.topo(y);
        let cells: Vec<Cell<'_>> = fila
            .iter()
            .map(|(id, k)| {
                Cell::new(Widget::Button {
                    id: *id,
                    text: tr(k),
                    primary: false,
                })
            })
            .collect();
        let mut c = self.cartao_ctx()?;
        Some(c.row(&cells, y))
    }

    /// Os verbos de uma fila num cartão, na mesma linha se couberem; `None` fora de um cartão.
    pub(super) fn verbos_no_cartao(&mut self, fila: &[(NodeId, &str, bool)], y: f32) -> Option<f32> {
        let cells: Vec<Cell<'_>> = fila
            .iter()
            .map(|&(id, text, primary)| Cell::new(Widget::Button { id, text, primary }))
            .collect();
        let topo = self.topo(y);
        let mut c = self.cartao_ctx()?;
        Some(c.row(&cells, topo))
    }

    /// Um verbo num rect dado (o nome de um pincel guardado). Devolve se o pintou.
    pub(super) fn botao_no_cartao(
        &mut self,
        r: Rect,
        id: NodeId,
        texto: &str,
        premido: bool,
    ) -> bool {
        let Some(mut c) = self.cartao_ctx() else {
            return false;
        };
        c.x = r.x;
        c.w = r.w;
        let linha = Cell::new(Widget::Button {
            id,
            text: texto,
            primary: premido,
        });
        c.row(&[linha], r.y);
        true
    }

    /// Um ícone solto num rect dado (o cadeado, o caixote, os menus da paleta). Devolve se o pintou.
    pub(super) fn icone_no_cartao(&mut self, r: Rect, id: NodeId, icon: IconId, on: bool) -> bool {
        let Some(mut c) = self.cartao_ctx() else {
            return false;
        };
        c.bare(
            r,
            &[Seg {
                id,
                icon: Some(icon),
                text: "",
                on,
            }],
        );
        true
    }

    /// Dois modos de seleção numa fila, a largura toda.
    pub(super) fn par_no_cartao(&mut self, par: &[(&str, bool, NodeId)], y: f32) -> f32 {
        let y = self.topo(y);
        let segs: Vec<Seg<'_>> = par
            .iter()
            .map(|&(text, on, id)| Seg {
                id,
                icon: None,
                text,
                on,
            })
            .collect();
        match self.cartao_ctx() {
            Some(mut c) => c.full(None, Widget::Choice(&segs), y),
            None => y,
        }
    }

    /// Uma fila de botões de largura igual — em par só quando os dois rótulos cabem INTEIROS na
    /// metade; senão um por linha (nenhum rótulo cortado no degrau estreito do dock).
    pub(super) fn botoes(&mut self, fila: &[(NodeId, &str)], y: f32) -> f32 {
        if self.cartao.is_some()
            && let Some(fim) = self.botoes_no_cartao(fila, y)
        {
            return fim;
        }
        let gap = Spacing::Xs.px();
        #[expect(clippy::cast_precision_loss, reason = "≤ 2 botões")]
        let n = fila.len() as f32;
        let bw = ((self.w - gap * (n - 1.0)) / n).max(1.0);
        let fonte = Button::label_font_px();
        let cabe = |ts: &mut TextSystem, key: &str| {
            let pad = Button::new(NodeId(0), "").padding();
            ts.prefix_width(tr(key), fonte) + 2.0 * pad <= bw
        };
        if fila.len() > 1 && !fila.iter().all(|(_, k)| cabe(self.text_system, k)) {
            return fila
                .iter()
                .fold(y, |y, b| self.botoes(std::slice::from_ref(b), y));
        }
        // Sozinho e ainda maior que a largura toda (o «Despeckle (Median Filter)…» no piso da
        // doca): o rótulo inteiro em duas linhas, nunca cortado.
        if let [(id, key)] = fila
            && !cabe(self.text_system, key)
        {
            let pad = Button::new(NodeId(0), "").padding();
            let texto = tr(key);
            let ts = &mut *self.text_system;
            let partida = texto
                .match_indices(' ')
                .map(|(i, _)| (&texto[..i], &texto[i + 1..]))
                .filter(|(a, b)| {
                    ts.prefix_width(a, fonte) + 2.0 * pad <= bw
                        && ts.prefix_width(b, fonte) + 2.0 * pad <= bw
                })
                .last();
            if let Some((a, b)) = partida {
                let r = Rect::new(self.x, y, bw, 2.0 * ROW_H_PX);
                let botao = Button::new(*id, "").visual(self.store.button_visual(*id));
                paint_button(&botao, r, self.scene, self.text_system, self.theme);
                let cor = tinta(ColorToken::Text1, self.theme);
                for (k, linha) in [a, b].into_iter().enumerate() {
                    #[expect(clippy::cast_precision_loss, reason = "duas linhas")]
                    let rl = Rect::new(self.x, y + k as f32 * ROW_H_PX, bw, ROW_H_PX);
                    ph2d_editor_core::paint::paint_text_centered(
                        self.text_system,
                        self.scene,
                        linha,
                        rl,
                        fonte,
                        cor,
                    );
                }
                self.hit_index.register(*id, r);
                return y + 2.0 * ROW_H_PX + ph2d_tokens::control_gap_px();
            }
        }
        for (i, (id, key)) in fila.iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "≤ 2 botões")]
            let r = Rect::new(self.x + i as f32 * (bw + gap), y, bw, ROW_H_PX);
            let b = Button::new(*id, tr(key)).visual(self.store.button_visual(*id));
            paint_button(&b, r, self.scene, self.text_system, self.theme);
            self.hit_index.register(*id, r);
        }
        y + ROW_H_PX + ph2d_tokens::control_gap_px()
    }
}

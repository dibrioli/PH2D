//! ⭐ **A PALETA do painel PIXEL** (P2): a grelha de cores, o seletor (roda, quadrado, barras,
//! mapa de normais), as acções da paleta e as predefinições.

use super::*;

impl Corpo<'_> {
    /// A grelha da paleta: esquerdo = frente, direito = fundo (o gémeo do botão direito).
    pub(super) fn paleta(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let y = self.linha(tr("panel.pixel.palette"), ColorToken::Text3, y);
        let cell = Spacing::Xl.px();
        let gap = Spacing::Xxs.px();
        let cols = (((self.w + gap) / (cell + gap)).floor().max(1.0)) as usize;
        let mut bottom = y;
        let arrasto = self
            .store
            .panel_row_drag()
            .filter(|(f, d)| *f == PanelRowFamily::PixelPalette && d.active)
            .map(|(_, d)| d);
        for (i, c) in info.palette.iter().enumerate() {
            #[expect(clippy::cast_precision_loss, reason = "uma paleta, ≤ 256")]
            let (cx, cy) = ((i % cols) as f32, (i / cols) as f32);
            let r = Rect::new(
                self.x + cx * (cell + gap),
                y + cy * (cell + gap),
                cell,
                cell,
            );
            let id = ids::palette_id(i);
            // A cor de frente acende na paleta, como no Aseprite.
            let st = if *c == info.fg {
                SwatchState::Focused
            } else if self.store.hover_live(id) > 0.0 {
                SwatchState::Hovered
            } else {
                SwatchState::Normal
            };
            paint_color_swatch(
                &ColorSwatch::new(id, "", *c).state(st),
                r,
                self.scene,
                self.theme,
            );
            // A selecção (a rampa do Shading) leva o anel do acento.
            if info.selected.contains(&i) {
                let anel = resolve(ColorToken::Accent, self.theme);
                stroke_rect(self.scene, r, StrokeToken::Thick.px(), anel);
            }
            // A célula onde a amostra arrastada vai cair.
            if let Some(d) = arrasto
                && d.dragged != id
                && r.contains(d.cursor_x, d.cursor_y)
            {
                let marca = resolve(ColorToken::Text1, self.theme);
                stroke_rect(self.scene, r, StrokeToken::Heavy.px(), marca);
            }
            self.hit_index
                .register_with_secondary(id, r, ids::palette_bg_id(i));
            bottom = r.y + r.h;
        }
        bottom
    }

    /// ⭐ **O menu ≡ da paleta do Aseprite**: a selecção e a cor dela (*Edit color* abre o seletor
    /// partilhado), o tamanho, mover, ordenar, seleccionar as usadas, nova do sprite, ler/gravar e
    /// a de omissão.
    pub(super) fn acoes_da_paleta(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        let sec = Seccao::medida(
            self.text_system,
            1,
            &[tr("panel.pixel.palette_size"), tr("panel.pixel.sort")],
        );
        let n = info.selected.len().to_string();
        let mut y = self.linha(
            &tr_with("panel.pixel.selected", &[("n", &n)]),
            ColorToken::Text3,
            y,
        );
        if let Some(c) = info.anchor.and_then(|i| info.palette.get(i)) {
            let side = ROW_H_PX;
            let r = Rect::new(self.x, y, side, side);
            let sw = ColorSwatch::new(ids::PIXEL_PAL_EDIT, "", *c);
            paint_color_swatch(&sw, r, self.scene, self.theme);
            self.hit_index.register(ids::PIXEL_PAL_EDIT, r);
            let tx = self.x + side + Spacing::Sm.px();
            let texto = tr_with("panel.pixel.edit_color", &[("hex", &hex(*c))]);
            paint_text(
                self.text_system,
                self.scene,
                &texto,
                tx,
                y + (ROW_H_PX - TypeToken::Sm.px()) * 0.5,
                TypeToken::Sm.px(),
                (self.x + self.w - tx).max(1.0),
                resolve(ColorToken::Text2, self.theme),
            );
            y += side + ph2d_tokens::control_gap_px();
        }
        y = self.campo(tr("panel.pixel.palette_size"), ids::PALETTE_SIZE, sec, y);
        let [
            esq,
            dir,
            asc,
            desc,
            usadas,
            do_sprite,
            ler,
            gravar,
            omissao,
            _,
        ] = ids::ACTIONS;
        y = self.botoes(&acoes([esq, dir]), y);
        let chaves: Vec<(&str, bool, NodeId)> = ids::SORT_KEYS
            .iter()
            .map(|&(k, id)| (tr(k.label_key()), k == info.sort_key, id))
            .collect();
        y = self.escolha(tr("panel.pixel.sort"), &chaves, sec, y);
        y = self.botoes(&acoes([asc, desc]), y);
        y = self.botoes(&acoes([usadas, do_sprite]), y);
        y = self.botoes(&acoes([ler, gravar]), y);
        y = self.botoes(&acoes(ids::PRESET_ACTIONS), y);
        y = self.botoes(&acoes([omissao]), y);
        self.predefinicoes(info, y)
    }

    /// As predefinições gravadas: um botão por paleta (clique = *Load Palette* dela).
    pub(super) fn predefinicoes(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        if info.presets.is_empty() {
            return y;
        }
        let mut y = self.linha(tr("panel.pixel.presets"), ColorToken::Text3, y);
        for (i, nome) in info.presets.iter().enumerate() {
            let id = ids::preset_id(i);
            let r = Rect::new(self.x, y, self.w, ROW_H_PX);
            let b = Button::new(id, nome.as_str()).visual(self.store.button_visual(id));
            paint_button(&b, r, self.scene, self.text_system, self.theme);
            self.hit_index.register(id, r);
            y += ROW_H_PX + ph2d_tokens::control_gap_px();
        }
        y
    }

    /// ⭐ **O seletor de cor abaixo da paleta** (os cinco do menu ≡ do Aseprite): a escolha do modo,
    /// o quadro (ou disco), a barra do modo e a do alfa, com o anel onde a cor de frente cai.
    /// Esquerdo = frente, direito = fundo.
    pub(super) fn seletor(&mut self, info: &PixelPanelInfo, y: f32) -> f32 {
        use crate::seletor as sel;
        let modo = info.options.selector;
        let nomes: Vec<&str> = ids::SELECTORS
            .iter()
            .map(|(m, _)| tr(m.label_key()))
            .collect();
        let sec = Seccao::medida(self.text_system, 1, &[tr("panel.pixel.selector")]);
        let modos: Vec<(&str, bool, NodeId)> = ids::SELECTORS
            .iter()
            .zip(nomes)
            .map(|(&(m, id), nome)| (nome, m == modo, id))
            .collect();
        let mut y = self.escolha(tr("panel.pixel.selector"), &modos, sec, y);
        let fg = info.fg;
        let v = &mut *self.seletor;
        // Uma cor que o seletor mandou NÃO reconstrói os parâmetros (eles são a verdade dela); só
        // uma cor de fora (paleta, conta-gotas, `X`) ou outro modo os refaz.
        // e durante o arrasto o seletor manda: nem o quadro do Down (ainda com a cor de fora) refaz.
        let propria = v.arrasto.is_some()
            || (v.mandadas.contains(&fg) && v.de.is_some_and(|(m, _)| m == modo));
        if !propria && v.de != Some((modo, fg)) {
            v.params = sel::de_rgb(modo, [fg[0], fg[1], fg[2]], v.params);
            v.de = Some((modo, fg));
        }
        let p = v.params;
        let fresca = matches!(&v.cache, Some((m, q, c, _)) if *m == modo && *q == p && *c == fg);
        if !fresca {
            let imagens = crate::state::Imagens {
                quadro: Arc::new(sel::imagem_do_quadro(modo, p, LADO)),
                barra: Arc::new(sel::imagem_da_barra(modo, p, LADO)),
                alfa: Arc::new(sel::imagem_do_alfa([fg[0], fg[1], fg[2]], LADO)),
            };
            v.cache = Some((modo, p, fg, imagens));
        }
        let Some((.., imagens)) = v.cache.clone() else {
            return y;
        };
        let gap = Spacing::Xxs.px();
        let barra_h = Spacing::Lg.px();
        let quadro = Rect::new(self.x, y, self.w, self.w);
        // As rodas não encostam nas laterais do quadrado (smoke do dono 25/10: ≥ 2 px de folga): o
        // disco pinta-se num quadrado recuado; o Normal map enche o de fora com o fundo dele.
        let folga = if sel::e_disco(modo) {
            Spacing::Xs.px()
        } else {
            0.0
        };
        let disco = Rect::new(
            quadro.x + folga,
            quadro.y + folga,
            quadro.w - folga * 2.0,
            quadro.h - folga * 2.0,
        );
        if modo == ph2d_editor_core::pixel_panel::PixelSelector::NormalMap {
            let [r, g, b] = sel::NORMAL_FUNDO;
            let fundo = Arc::new(vec![r, g, b, u8::MAX]);
            let dest = (
                f64::from(quadro.x),
                f64::from(quadro.y),
                f64::from(quadro.x + quadro.w),
                f64::from(quadro.y + quadro.h),
            );
            self.scene
                .draw_image_rgba(&fundo, 1, 1, dest, ImageQuality::Low);
        }
        let barra = Rect::new(self.x, quadro.y + quadro.h + gap, self.w, barra_h);
        let alfa = Rect::new(self.x, barra.y + barra.h + gap, self.w, barra_h);
        // O alfa pinta-se sobre o xadrez da amostra da casa.
        let xadrez = ColorSwatch::new(ids::SELECTOR_AREAS[2].0, "", [0; 4]);
        paint_color_swatch(&xadrez, alfa, self.scene, self.theme);
        for (img, r, h) in [
            (&imagens.quadro, disco, LADO),
            (&imagens.barra, barra, 1),
            (&imagens.alfa, alfa, 1),
        ] {
            let dest = (
                f64::from(r.x),
                f64::from(r.y),
                f64::from(r.x + r.w),
                f64::from(r.y + r.h),
            );
            self.scene
                .draw_image_rgba(img, LADO, h, dest, ImageQuality::Medium);
        }
        let (u, vv) = sel::ponto(modo, p);
        self.anel(disco, u, vv);
        self.anel(barra, sel::ponto_na_barra(modo, p), 0.5);
        self.anel(alfa, f64::from(fg[3]) / f64::from(u8::MAX), 0.5);
        // O alvo do clique é o quadrado inteiro; a conta da cor é a do disco (a folga prende à borda).
        for (k, (alvo, conta)) in [(quadro, disco), (barra, barra), (alfa, alfa)]
            .into_iter()
            .enumerate()
        {
            let (a, b) = ids::SELECTOR_AREAS[k];
            self.hit_index.register_with_secondary(a, alvo, b);
            self.seletor.rects[k] = Some(conta);
        }
        // A cor escolhida, à vista (smoke do dono 25/10): a amostra com o xadrez e o hex.
        let vista = Rect::new(self.x, alfa.y + alfa.h + gap * 2.0, self.w, ROW_H_PX);
        let amostra = Rect::new(vista.x, vista.y, vista.h * 2.0, vista.h);
        paint_color_swatch(
            &ColorSwatch::new(ids::PIXEL_FG, "", fg),
            amostra,
            self.scene,
            self.theme,
        );
        let tx = amostra.x + amostra.w + Spacing::Sm.px();
        paint_text(
            self.text_system,
            self.scene,
            &hex(fg),
            tx,
            vista.y + (ROW_H_PX - TypeToken::Sm.px()) * 0.5,
            TypeToken::Sm.px(),
            (vista.x + vista.w - tx).max(1.0),
            resolve(ColorToken::Text2, self.theme),
        );
        y = vista.y + vista.h;
        y
    }

    /// O anel do marcador em `(u, v)` do rect: escuro por fora, claro por dentro (vê-se em qualquer
    /// cor).
    pub(super) fn anel(&mut self, r: Rect, u: f64, v: f64) {
        #[expect(clippy::cast_possible_truncation, reason = "uma posição no ecrã")]
        let (cx, cy) = (r.x + r.w * u as f32, r.y + r.h * v as f32);
        let raio = Spacing::Xs.px();
        let anel = Rect::new(cx - raio, cy - raio, raio * 2.0, raio * 2.0);
        for (cor, largura) in [
            (ColorToken::Bg0, StrokeToken::Heavy),
            (ColorToken::Text1, StrokeToken::Thin),
        ] {
            let c = resolve(cor, self.theme);
            stroke_rounded_rect(self.scene, anel, raio, largura.px(), c); // FRAME-RAW-OK: o anel do marcador é um círculo, não uma moldura
        }
    }
}

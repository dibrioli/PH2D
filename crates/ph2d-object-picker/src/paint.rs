//! A pintura da janela. Quem pinta também registra o que se clica.

use ph2d_editor_core::IconId;
use ph2d_editor_core::paint::{
    fill_rounded_rect, paint_icon, paint_text_elided, paint_text_title_elided, rect_to_vello,
    resolve, stroke_rounded_rect,
};
use ph2d_editor_core::widget::lista_flutuante::Superficie;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken as T, StrokeToken, Theme};
use ph2d_vector::{Color, VectorScene};

use crate::{Alvo, ObjectPicker, Textos};

const LARGURA: f32 = 460.0;
const FOLGA_DA_ABA: f32 = 24.0;
const PAD: f32 = 20.0;
const TITULO_H: f32 = 24.0;
const TITULO_PX: f32 = 16.0;
const SUB_H: f32 = 20.0;
const TEXTO_PX: f32 = 13.0;
const PEQUENO_PX: f32 = 11.0;
const GAP: f32 = 12.0;
const CAMPO_H: f32 = 32.0;
const CAMPO_RAIO: f32 = 6.0;
const CAMPO_ANEL: f32 = 1.5;
const CAMPO_PAD: f32 = 10.0;
const ICONE: f32 = 16.0;
const LINHA_H: f32 = 30.0;
const LINHA_PAD: f32 = 10.0;
const LINHAS_MAX: usize = 8;
const CARET_W: f32 = 1.5;
const CARET_FOLGA: f32 = 7.0;
const BARRA_W: f32 = 3.0;
const BOTAO_H: f32 = 28.0;
const BOTAO_PAD: f32 = 14.0;

struct P<'a> {
    scene: &'a mut VectorScene,
    ts: &'a mut TextSystem,
    s: Superficie,
}

impl P<'_> {
    fn texto(&mut self, t: &str, x: f32, r: Rect, px: f32, cor: Color) {
        let alto = self.ts.layout("Ag", px, f32::INFINITY).height();
        let y = r.y + ((r.h - alto) / 2.0).max(0.0);
        paint_text_elided(
            self.ts,
            self.scene,
            t,
            x,
            y,
            px,
            (r.x + r.w - x).max(0.0),
            cor,
        );
    }

    fn icone(&mut self, i: IconId, r: Rect, cor: Color) {
        let q = Rect::new(
            r.x + (r.w - ICONE) / 2.0,
            r.y + (r.h - ICONE) / 2.0,
            ICONE,
            ICONE,
        );
        paint_icon(self.scene, i, q, cor, StrokeToken::Default.px());
    }
}

impl ObjectPicker {
    /// Pinta sobre `area`, a aba onde a janela mora.
    pub fn paint(
        &mut self,
        area: Rect,
        textos: &Textos<'_>,
        scene: &mut VectorScene,
        ts: &mut TextSystem,
        theme: Theme,
    ) {
        scene.fill_rect(rect_to_vello(area), resolve(T::BgScrim, theme));
        self.alvos.clear();

        let fixo = 2.0 * PAD + TITULO_H + SUB_H + 3.0 * GAP + CAMPO_H + BOTAO_H;
        let cabem = ((area.h - 2.0 * FOLGA_DA_ABA - fixo) / LINHA_H)
            .floor()
            .max(1.0) as usize;
        self.linhas = cabem.min(LINHAS_MAX);
        self.prende_a_rolagem();
        let w = LARGURA.min(area.w - 2.0 * FOLGA_DA_ABA).max(0.0);
        let h = fixo + self.linhas as f32 * LINHA_H;
        let caixa = Rect::new(
            area.x + (area.w - w) / 2.0,
            area.y + ((area.h - h) / 2.0).max(0.0),
            w,
            h,
        );
        self.caixa = caixa;

        let s = Superficie::de(theme);
        let _tinta = s.veste();
        s.moldura_de_caixa(scene, caixa);
        let mut p = P { scene, ts, s };

        let (x, dentro) = (caixa.x + PAD, w - 2.0 * PAD);
        let mut y = caixa.y + PAD;
        let cor = s.cor(T::Text1);
        let alto = p.ts.layout("Ag", TITULO_PX, f32::INFINITY).height();
        let ty = y + ((TITULO_H - alto) / 2.0).max(0.0);
        paint_text_title_elided(p.ts, p.scene, textos.titulo, x, ty, TITULO_PX, dentro, cor);
        y += TITULO_H;

        let linha = Rect::new(x, y, dentro, SUB_H);
        match textos.aviso {
            Some(a) => p.texto(a, x, linha, TEXTO_PX, s.cor(T::Warn)),
            None => p.texto(textos.subtitulo, x, linha, TEXTO_PX, s.cor(T::Text3)),
        }
        y += SUB_H + GAP;

        self.campo(&mut p, Rect::new(x, y, dentro - CAMPO_H - GAP, CAMPO_H));
        self.conta_gotas(&mut p, Rect::new(x + dentro - CAMPO_H, y, CAMPO_H, CAMPO_H));
        y += CAMPO_H + GAP;

        self.lista(
            &mut p,
            Rect::new(x, y, dentro, self.linhas as f32 * LINHA_H),
        );
        y += self.linhas as f32 * LINHA_H + GAP;

        self.rodape(&mut p, Rect::new(x, y, dentro, BOTAO_H));
    }

    fn campo(&mut self, p: &mut P<'_>, r: Rect) {
        let s = p.s;
        fill_rounded_rect(p.scene, r, CAMPO_RAIO, s.cor(T::Bg1));
        stroke_rounded_rect(p.scene, r, CAMPO_RAIO, CAMPO_ANEL, s.cor(T::Accent));
        let lupa = Rect::new(r.x + CAMPO_PAD, r.y, ICONE, r.h);
        p.icone(IconId::Search, lupa, s.cor(T::Text3));
        let x = lupa.x + ICONE + CAMPO_PAD;
        let (a, b) = (r.y + CARET_FOLGA, r.h - 2.0 * CARET_FOLGA);
        let fim = r.x + r.w - CAMPO_PAD;
        let caret_x = |ts: &mut TextSystem, at: usize, t: &str| {
            (x + ts.prefix_width(&t[..at], TEXTO_PX)).min(fim)
        };
        let t = self.busca.text.clone();
        if let Some((i, j)) = self.busca.selection() {
            let (x0, x1) = (caret_x(p.ts, i, &t), caret_x(p.ts, j, &t));
            fill_rounded_rect(
                p.scene,
                Rect::new(x0, a, x1 - x0, b),
                0.0,
                s.cor(T::Selection),
            );
        }
        let dentro = Rect::new(r.x, r.y, r.w - CAMPO_PAD, r.h);
        if t.is_empty() {
            p.texto(
                tr("object_picker.search"),
                x,
                dentro,
                TEXTO_PX,
                s.cor(T::Text3),
            );
        } else {
            p.texto(&t, x, dentro, TEXTO_PX, s.cor(T::Text1));
        }
        let cx = caret_x(p.ts, self.busca.caret, &t);
        fill_rounded_rect(p.scene, Rect::new(cx, a, CARET_W, b), 0.0, s.cor(T::Text1));
        self.alvos.push((Alvo::Busca, r));
    }

    fn conta_gotas(&mut self, p: &mut P<'_>, r: Rect) {
        let sobre = self.hover == Some(Alvo::ContaGotas);
        let cor = p.s.botao(p.scene, r, self.armado, sobre);
        p.icone(IconId::Eyedropper, r, cor);
        self.alvos.push((Alvo::ContaGotas, r));
    }

    fn lista(&mut self, p: &mut P<'_>, r: Rect) {
        self.lista = r;
        let s = p.s;
        if self.vistos.is_empty() {
            let chave = if self.objetos.is_empty() {
                "object_picker.no_objects"
            } else {
                "object_picker.no_match"
            };
            let linha = Rect::new(r.x, r.y, r.w, LINHA_H);
            p.texto(tr(chave), r.x + LINHA_PAD, linha, TEXTO_PX, s.cor(T::Text3));
            return;
        }
        let atual_txt = tr("object_picker.current");
        let atual_w = p.ts.prefix_width(atual_txt, PEQUENO_PX);
        let fim = (self.ini + self.linhas).min(self.vistos.len());
        for n in self.ini..fim {
            let (id, nome) = self.objetos[self.vistos[n]].clone();
            let l = Rect::new(r.x, r.y + (n - self.ini) as f32 * LINHA_H, r.w, LINHA_H);
            if n == self.destaque {
                s.realce(p.scene, l);
            }
            let eh_atual = self.atual == Some(id);
            let reserva = if eh_atual {
                atual_w + 2.0 * LINHA_PAD
            } else {
                LINHA_PAD
            };
            let caixa_nome = Rect::new(l.x, l.y, l.w - reserva, l.h);
            p.texto(
                &nome,
                l.x + LINHA_PAD,
                caixa_nome,
                TEXTO_PX,
                s.cor(T::Text1),
            );
            if eh_atual {
                let tx = l.x + l.w - atual_w - LINHA_PAD;
                p.texto(atual_txt, tx, l, PEQUENO_PX, s.cor(T::Text3));
            }
            self.alvos.push((Alvo::Objeto(id), l));
        }
        if self.vistos.len() > self.linhas {
            let total = self.vistos.len() as f32;
            let topo = r.y + r.h * self.ini as f32 / total;
            let alto = r.h * self.linhas as f32 / total;
            let barra = Rect::new(r.x + r.w - BARRA_W, topo, BARRA_W, alto);
            fill_rounded_rect(p.scene, barra, BARRA_W / 2.0, s.cor(T::Text3));
        }
    }

    fn rodape(&mut self, p: &mut P<'_>, r: Rect) {
        let s = p.s;
        let cancela = tr("object_picker.cancel");
        let w = p.ts.prefix_width(cancela, TEXTO_PX) + 2.0 * BOTAO_PAD;
        let b = Rect::new(r.x + r.w - w, r.y, w, BOTAO_H);
        let cor = s.botao(p.scene, b, false, self.hover == Some(Alvo::Cancelar));
        p.texto(cancela, b.x + BOTAO_PAD, b, TEXTO_PX, cor);
        self.alvos.push((Alvo::Cancelar, b));
        let dica = Rect::new(r.x, r.y, r.w - w - GAP, r.h);
        let (chave, tinta) = if self.armado {
            ("object_picker.hint_armed", T::Accent)
        } else {
            ("object_picker.hint", T::Text3)
        };
        p.texto(tr(chave), r.x, dica, PEQUENO_PX, s.cor(tinta));
    }
}

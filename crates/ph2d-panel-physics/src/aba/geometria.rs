//! A geometria da aba das camadas.

use ph2d_editor_core::widget::{SCROLLBAR_W, scrollbar_thumb_rect as thumb_rect};
use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing, TypeToken};

/// A geometria da aba numa `area`: a lista à esquerda, a grade em cima à direita, a camada aberta
/// embaixo. `k` = quantas camadas visíveis.
#[derive(Clone, Copy, Debug)]
pub struct Geometria {
    pub lista: Rect,
    pub grade: Rect,
    pub camada: Rect,
    /// O canto de cima à esquerda dos rótulos da grade.
    pub x: f32,
    pub y: f32,
    pub rotulo_w: f32,
    pub passo: f32,
    pub k: usize,
    /// A largura de cada um dos dois botões da lista («+ Add layer» e «Remove»; [`Self::com_botoes`]).
    pub botao_w: f32,
    /// ⭐ A grade ROLA (onda 9e): a primeira linha e a primeira coluna à vista, e quantas cabem.
    pub l0: usize,
    pub c0: usize,
    pub cabem_l: usize,
    pub cabem_c: usize,
}

/// A largura de uma barra de rolagem e o vão até às células (a porta das barras do app).
fn faixa_da_barra() -> f32 {
    SCROLLBAR_W + Spacing::Xs.px()
}

impl Geometria {
    /// A largura da coluna da direita (a grade e a camada aberta), antes de saber as alturas.
    #[must_use]
    pub fn largura_direita(area: Rect, lista_w: f32) -> f32 {
        let pad = Spacing::Md.px();
        (area.w - (pad + pad + pad) - lista_w).max(0.0)
    }

    /// ⭐ Os dois cartões da direita têm o tamanho do conteúdo (maquete B): a grade com quadrados de uma
    /// linha e um vão, e a camada aberta com `camada_h`. Sem altura, os quadrados encolhem até caber.
    #[must_use]
    pub fn de(area: Rect, rotulo_w: f32, lista_w: f32, k: usize, camada_h: f32) -> Self {
        let pad = Spacing::Md.px();
        let row = ROW_H_PX;
        let dx = area.x + pad * 2.0 + lista_w;
        let dw = Self::largura_direita(area, lista_w);
        let n = k.max(1) as f32;
        // Título (uma linha), rótulos de cima, as células e o pé (uma linha), com os vãos.
        let dentro = Self::MARGEM_DO_CARTAO;
        let fixo_h = dentro * 2.0 + pad + row * 2.0 + rotulo_w;
        let livre_h = area.h - (pad + pad + pad) - camada_h - fixo_h;
        let livre_w = dw - dentro * 2.0 - rotulo_w;
        let passo = (livre_h / n)
            .min(livre_w / n)
            .min(row + Spacing::Sm.px())
            .max(Self::passo_minimo());
        let cabe = |livre: f32| ((livre / passo).floor().max(1.0) as usize).min(k.max(1));
        // ⭐ Onda 9f: o que não cabe mostra que rola — a barra vertical toma a faixa da direita e a horizontal a de
        // baixo (duas voltas: uma barra pode fazer faltar lugar à outra). Sem rolagem, a medida de antes.
        let (mut cabem_l, mut cabem_c) = (cabe(livre_h), cabe(livre_w));
        for _ in 0..2 {
            let menos = |falta: bool| if falta { faixa_da_barra() } else { 0.0 };
            cabem_l = cabe(livre_h - menos(cabem_c < k));
            cabem_c = cabe(livre_w - menos(cabem_l < k));
        }
        let barra_h = if cabem_c < k { faixa_da_barra() } else { 0.0 };
        let grade = Rect::new(
            dx,
            area.y + pad,
            dw,
            fixo_h + passo * cabem_l as f32 + barra_h,
        );
        let camada = Rect::new(dx, grade.y + grade.h + pad, dw, camada_h);
        // As duas margens, o título e as `k` linhas, o vão e os botões.
        let lista_h = Self::MARGEM_DA_LISTA * 2.0 // LITERAL-PX-OK: em cima e embaixo
            + row * (k + 1) as f32
            + Self::VAO_DOS_BOTOES
            + Self::ALTURA_DO_BOTAO;
        let lista = Rect::new(
            area.x + pad,
            area.y + pad,
            lista_w,
            (grade.h + pad + camada_h).max(lista_h),
        );
        Self {
            lista,
            grade,
            camada,
            x: grade.x + dentro,
            y: grade.y + dentro + row,
            rotulo_w,
            passo,
            k,
            botao_w: 0.0,
            l0: 0,
            c0: 0,
            cabem_l,
            cabem_c,
        }
    }

    /// ⭐ O passo mínimo da grade (onda 9e): o passo de linha do app (`row_pitch_px` = `ROW_H_PX` + o vão dos
    /// controlos), o alvo que a lista, os botões e os quadros do Inspector já clicam; o rótulo cabe inteiro
    /// (`fonte_da_grade` = a letra do painel). Medido em 07/11 com 27 camadas: o passo antigo encolhia a
    /// `Spacing::Md` (8 px, célula de 4 px; a foto `aba9d_27_camadas…` dava ~10 px).
    #[must_use]
    pub fn passo_minimo() -> f32 {
        ph2d_tokens::row_pitch_px()
    }

    /// ⭐ A largura da lista pela regra do editor de animações (onda 9g): `FRACAO_DA_LISTA` da aba, entre
    /// `LINHAS_MIN` e `LINHAS_MAX` alturas de botão, alargada até os dois botões caberem lado a lado.
    #[must_use]
    pub fn largura_da_lista(area_w: f32, botao_w: f32) -> f32 {
        let h = Self::ALTURA_DO_BOTAO;
        (area_w * Self::FRACAO_DA_LISTA)
            .min(h * Self::LINHAS_MAX)
            .max(h * Self::LINHAS_MIN)
            .max(Self::lista_para_os_botoes(botao_w))
    }

    pub const FRACAO_DA_LISTA: f32 = 0.18; // LITERAL-PX-OK: fração de layout (a do editor de animações)
    pub const LINHAS_MIN: f32 = 3.0; // LITERAL-PX-OK: múltiplo da altura do botão
    pub const LINHAS_MAX: f32 = 7.0; // LITERAL-PX-OK: múltiplo da altura do botão

    /// Rola para a primeira linha `l0` e a primeira coluna `c0`, presas ao que há para mostrar.
    #[must_use]
    pub fn rolada(mut self, (l0, c0): (usize, usize)) -> Self {
        self.l0 = l0.min(self.k.saturating_sub(self.cabem_l));
        self.c0 = c0.min(self.k.saturating_sub(self.cabem_c));
        self
    }

    /// Se a linha `l` está à vista.
    #[must_use]
    pub fn linha_a_vista(&self, l: usize) -> bool {
        (self.l0..self.l0 + self.cabem_l).contains(&l)
    }

    /// Se a coluna `c` está à vista.
    #[must_use]
    pub fn coluna_a_vista(&self, c: usize) -> bool {
        (self.c0..self.c0 + self.cabem_c).contains(&c)
    }

    /// Se a célula `(l, c)` está à vista.
    #[must_use]
    pub fn celula_a_vista(&self, l: usize, c: usize) -> bool {
        self.linha_a_vista(l) && self.coluna_a_vista(c)
    }

    fn dl(&self, l: usize) -> f32 {
        l as f32 - self.l0 as f32
    }

    fn dc(&self, c: usize) -> f32 {
        c as f32 - self.c0 as f32
    }

    /// A célula da linha `l` × coluna `c` (posições na lista das visíveis).
    #[must_use]
    pub fn celula(&self, l: usize, c: usize) -> Rect {
        // ⭐ Uma só régua (onda 9c): a célula fica no CENTRO da sua faixa, meio vão de cada lado, para o
        // centro do quadrado ser o do rótulo da linha, do rótulo girado e da pílula a qualquer passo.
        let vao = Spacing::Xs.px();
        Rect::new(
            self.x + self.rotulo_w + self.dc(c) * self.passo + vao * 0.5,
            self.y + self.rotulo_w + self.dl(l) * self.passo + vao * 0.5,
            self.passo - vao,
            self.passo - vao,
        )
    }

    /// A pílula da linha acesa: a mesma altura e o mesmo centro da linha de quadrados, do rótulo até
    /// debaixo da última célula à vista da linha (a faixa pairada passa sob as células, onda 9g).
    #[must_use]
    pub fn pilula_da_linha(&self, l: usize) -> Rect {
        let r = self.rotulo_da_linha(l);
        let cel = self.celula(l, 0);
        let x = r.x - Spacing::Xs.px();
        let fim = (self.c0 + self.cabem_c)
            .min(l + 1)
            .checked_sub(1)
            .filter(|&c| c >= self.c0)
            .map_or(r.x + r.w, |c| {
                let u = self.celula(l, c);
                u.x + u.w + Spacing::Xs.px() * 0.5
            });
        Rect::new(x, cel.y, (fim - x).max(r.w), cel.h)
    }

    /// A letra dos rótulos da grade: a do painel, mas nunca maior que o quadrado.
    #[must_use]
    pub fn fonte_da_grade(&self, font: f32) -> f32 {
        font.min(self.passo - Spacing::Xs.px())
    }

    #[must_use]
    pub fn rotulo_da_linha(&self, l: usize) -> Rect {
        Rect::new(
            self.x,
            self.y + self.rotulo_w + self.dl(l) * self.passo,
            self.rotulo_w,
            self.passo,
        )
    }

    #[must_use]
    pub fn rotulo_da_coluna(&self, c: usize) -> Rect {
        Rect::new(
            self.x + self.rotulo_w + self.dc(c) * self.passo,
            self.y,
            self.passo,
            self.rotulo_w,
        )
    }

    /// A frase do pé, logo abaixo da última linha da grade.
    #[must_use]
    pub fn pe(&self) -> Rect {
        let (pad, dentro) = (Spacing::Md.px(), Self::MARGEM_DO_CARTAO);
        Rect::new(
            self.grade.x + dentro,
            self.y
                + self.rotulo_w
                + self.cabem_l as f32 * self.passo
                + self.barra_horizontal().map_or(0.0, |_| faixa_da_barra())
                + pad,
            self.grade.w - dentro * 2.0,
            ROW_H_PX,
        )
    }

    /// ⭐ A calha da barra vertical (onda 9f), à direita das células à vista; `None` = todas as linhas cabem.
    #[must_use]
    pub fn barra_vertical(&self) -> Option<Rect> {
        (self.cabem_l < self.k).then(|| {
            Rect::new(
                self.x + self.rotulo_w + self.cabem_c as f32 * self.passo + Spacing::Xs.px(),
                self.y + self.rotulo_w,
                SCROLLBAR_W,
                self.cabem_l as f32 * self.passo,
            )
        })
    }

    /// A calha da barra horizontal, debaixo das células à vista; `None` = todas as colunas cabem.
    #[must_use]
    pub fn barra_horizontal(&self) -> Option<Rect> {
        (self.cabem_c < self.k).then(|| {
            Rect::new(
                self.x + self.rotulo_w,
                self.y + self.rotulo_w + self.cabem_l as f32 * self.passo + Spacing::Xs.px(),
                self.cabem_c as f32 * self.passo,
                SCROLLBAR_W,
            )
        })
    }

    /// O polegar vertical: a parte das linhas à vista, na posição da primeira (o `thumb_rect` das barras do app).
    #[must_use]
    pub fn polegar_vertical(&self) -> Option<Rect> {
        let t = self.barra_vertical()?;
        let p = self.passo;
        Some(thumb_rect(
            t,
            self.l0 as f32 * p,
            self.k as f32 * p,
            self.cabem_l as f32 * p,
        ))
    }

    /// O polegar horizontal (a mesma conta, de lado).
    #[must_use]
    pub fn polegar_horizontal(&self) -> Option<Rect> {
        let t = self.barra_horizontal()?;
        let p = self.passo;
        let deitada = Rect::new(t.y, t.x, t.h, t.w);
        let r = thumb_rect(
            deitada,
            self.c0 as f32 * p,
            self.k as f32 * p,
            self.cabem_c as f32 * p,
        );
        Some(Rect::new(r.y, r.x, r.h, r.w))
    }

    /// A primeira linha (vertical) ou coluna que põe o polegar sob o ponteiro em `pos` (ao longo da calha),
    /// com `pega` = onde o polegar foi agarrado, medido do começo dele.
    #[must_use]
    pub fn primeira_sob(&self, vertical: bool, pos: f32, pega: f32) -> usize {
        let (calha, polegar, cabem) = if vertical {
            let (Some(c), Some(p)) = (self.barra_vertical(), self.polegar_vertical()) else {
                return self.l0;
            };
            ((c.y, c.h), p.h, self.cabem_l)
        } else {
            let (Some(c), Some(p)) = (self.barra_horizontal(), self.polegar_horizontal()) else {
                return self.c0;
            };
            ((c.x, c.w), p.w, self.cabem_c)
        };
        let max = self.k.saturating_sub(cabem);
        let curso = (calha.1 - polegar).max(1.0);
        let t = ((pos - pega - calha.0) / curso).clamp(0.0, 1.0);
        (t * max as f32).round() as usize
    }

    /// A linha `p` da lista (debaixo do título «Layers»).
    #[must_use]
    pub fn linha_da_lista(&self, p: usize) -> Rect {
        let pad = Self::MARGEM_DA_LISTA;
        Rect::new(
            self.lista.x + pad,
            self.lista.y + pad + ROW_H_PX * (p + 1) as f32,
            self.lista.w - pad * 2.0,
            ROW_H_PX,
        )
    }

    /// O título «Layers»: a linha acima da primeira (o `list_title` do editor de animações).
    #[must_use]
    pub fn titulo_da_lista(&self) -> Rect {
        let r = self.linha_da_lista(0);
        Rect::new(r.x, r.y - ROW_H_PX, r.w, ROW_H_PX)
    }

    /// A linha de título da camada aberta («Player · 1 object»), à esquerda de `pares_w`.
    #[must_use]
    pub fn titulo_da_camada(&self, pares_w: f32) -> Rect {
        let pad = Self::MARGEM_DO_CARTAO;
        Rect::new(
            self.camada.x + pad,
            self.camada.y + pad,
            (self.camada.w - pares_w - (pad * 2.0 + Spacing::Lg.px())).max(0.0),
            ROW_H_PX,
        )
    }

    /// ⭐ A letra da lista «Layers»: a do editor de animações (`FONT_TOKEN` dele).
    pub const FONTE_DA_LISTA: TypeToken = TypeToken::Base;
    /// O quadradinho da cor sobre a letra (o `SWATCH_OVER_FONT` do editor de animações).
    pub const QUADRADINHO_SOBRE_A_LETRA: f32 = 0.8; // LITERAL-PX-OK: razão adimensional

    /// O lado do quadradinho da cor para a letra `font`.
    #[must_use]
    pub fn lado_do_quadradinho(font: f32) -> f32 {
        font * Self::QUADRADINHO_SOBRE_A_LETRA
    }

    /// ⭐ A margem interna da lista (onda 9g: `Lg`, a do editor de animações; antes `Sm`).
    pub const MARGEM_DA_LISTA: f32 = Spacing::Lg.factory_px();
    /// A margem interna dos cartões da grade e da camada aberta (onda 9g: `Lg`, como o editor de animações).
    pub const MARGEM_DO_CARTAO: f32 = Spacing::Lg.factory_px();
    /// O vão entre os botões e antes deles, e a altura deles: os campos dos cartões (`FIELD_GAP`, `FIELD_H`).
    pub const VAO_DOS_BOTOES: f32 = ph2d_card_stack::metrics::FIELD_GAP;
    pub const ALTURA_DO_BOTAO: f32 = ph2d_card_stack::metrics::FIELD_H;

    /// A fila das caixinhas e da pergunta do Remove: a linha da lista, vão `Sm`, entre linhas `Xs`.
    #[must_use]
    pub fn fila_de_linhas() -> (f32, f32, f32) {
        (ROW_H_PX, Spacing::Sm.px(), Spacing::Xs.px())
    }

    /// A fila dos botões (onda 9g): a altura e o vão dos botões da lista.
    #[must_use]
    pub fn fila_de_botoes() -> (f32, f32, f32) {
        (
            Self::ALTURA_DO_BOTAO,
            Self::VAO_DOS_BOTOES,
            Self::VAO_DOS_BOTOES,
        )
    }

    /// A largura que a lista pede para os dois botões de `botao_w` lado a lado, com as margens e o vão.
    #[must_use]
    pub fn lista_para_os_botoes(botao_w: f32) -> f32 {
        (botao_w + Self::MARGEM_DA_LISTA) * 2.0 + Self::VAO_DOS_BOTOES // LITERAL-PX-OK: dois botões, as margens e o vão
    }

    /// Os dois botões com `botao_w`; se lado a lado não cabem dentro da lista, um debaixo do outro (a lista
    /// cresce uma linha). Nunca passam da borda.
    #[must_use]
    pub fn com_botoes(mut self, botao_w: f32) -> Self {
        self.botao_w = botao_w.min(self.lista.w - Self::MARGEM_DA_LISTA * 2.0);
        if self.botoes_empilhados() {
            let fim = self.remove().y + self.remove().h + Self::MARGEM_DA_LISTA;
            self.lista.h = self.lista.h.max(fim - self.lista.y);
        }
        self
    }

    /// Se os dois botões vão um debaixo do outro.
    #[must_use]
    pub fn botoes_empilhados(&self) -> bool {
        Self::lista_para_os_botoes(self.botao_w) > self.lista.w + f32::EPSILON
    }

    /// O «+ Add layer», debaixo da última linha da lista.
    #[must_use]
    pub fn add(&self) -> Rect {
        let r = self.linha_da_lista(self.k);
        Rect::new(
            r.x,
            r.y + Self::VAO_DOS_BOTOES,
            self.botao_w,
            Self::ALTURA_DO_BOTAO,
        )
    }

    /// O «Remove»: ao lado do «+ Add layer», com o vão; ou debaixo dele.
    #[must_use]
    pub fn remove(&self) -> Rect {
        let a = self.add();
        if self.botoes_empilhados() {
            Rect::new(a.x, a.y + a.h + Self::VAO_DOS_BOTOES, a.w, a.h)
        } else {
            Rect::new(a.x + a.w + Self::VAO_DOS_BOTOES, a.y, a.w, a.h)
        }
    }
}

/// Uma fila que quebra: um retângulo de uma linha por largura, da esquerda para a direita a partir de
/// `(x, y)` dentro de `w`, com `(altura, vão, vão entre linhas)`. Devolve os retângulos e quantas linhas ocupou.
#[must_use]
pub fn fila(
    x: f32,
    y: f32,
    w: f32,
    larguras: &[f32],
    (h, vao, vao_v): (f32, f32, f32),
) -> (Vec<Rect>, usize) {
    let mut v = Vec::with_capacity(larguras.len());
    let (mut cx, mut linha) = (x, 0usize);
    for &lw in larguras {
        if cx > x && cx + lw > x + w {
            cx = x;
            linha += 1;
        }
        v.push(Rect::new(cx, y + linha as f32 * (h + vao_v), lw, h));
        cx += lw + vao;
    }
    (v, if larguras.is_empty() { 0 } else { linha + 1 })
}

#[cfg(test)]
mod alinhamento_tests {
    use super::*;

    fn centro_y(r: Rect) -> f32 {
        r.y + r.h * 0.5
    }
    fn centro_x(r: Rect) -> f32 {
        r.x + r.w * 0.5
    }

    /// ⭐ Onda 9c (dono 07/11: «quando as caixas estão pequenas fica mal alinhado»): em ECRÃ, com 5, 16 e 32
    /// camadas, o centro do rótulo da linha = o da linha de quadrados = o da pílula, e o do rótulo de cima = o
    /// da coluna, a 1 px; a pílula tem a altura do quadrado.
    #[test]
    fn rotulos_pilula_e_quadrados_tem_o_mesmo_centro_a_qualquer_passo() {
        let row = ROW_H_PX;
        let area = Rect::new(row, row, row * 40.0, row * 24.0);
        let tol = StrokeTokenPx::um();
        for k in [5usize, 16, 32] {
            let g = Geometria::de(area, row * 4.0, row * 8.0, k, row * 4.0);
            for l in 0..k {
                let cel = g.celula(l, 0);
                let fim = g.celula(l, l);
                assert!((centro_y(cel) - centro_y(fim)).abs() <= tol, "k={k} l={l}");
                let rot = g.rotulo_da_linha(l);
                let pil = g.pilula_da_linha(l);
                assert!(
                    (centro_y(rot) - centro_y(cel)).abs() <= tol,
                    "rótulo×linha k={k} l={l}"
                );
                assert!(
                    (centro_y(pil) - centro_y(cel)).abs() <= tol,
                    "pílula×linha k={k} l={l}"
                );
                assert!((pil.h - cel.h).abs() <= tol, "altura da pílula k={k} l={l}");
                let col = g.rotulo_da_coluna(l);
                let cel_c = g.celula(k - 1, l);
                assert!(
                    (centro_x(col) - centro_x(cel_c)).abs() <= tol,
                    "topo×coluna k={k} c={l}"
                );
                assert!(g.fonte_da_grade(row) <= cel.h + tol, "a letra cabe k={k}");
            }
        }
    }

    /// ⭐ Dono 08/11 (a lista lê-se como a do editor de animações): o passo das linhas é o `ROW_H_PX` e a
    /// linha tem essa altura (nenhum vão), a faixa da selecionada tem a largura inteira da lista por dentro
    /// das margens, e o título fica na linha de cima.
    #[test]
    fn a_lista_das_camadas_tem_o_passo_e_a_faixa_do_editor() {
        let row = ROW_H_PX;
        let area = Rect::new(row, row, row * 40.0, row * 24.0);
        let tol = StrokeTokenPx::um();
        for k in [3usize, 16, 32] {
            let g = Geometria::de(area, row * 4.0, row * 8.0, k, row * 4.0);
            let dentro = g.lista.w - Geometria::MARGEM_DA_LISTA * 2.0;
            for p in 0..k {
                let r = g.linha_da_lista(p);
                assert!((r.h - row).abs() <= tol, "altura k={k} p={p}");
                assert!((r.w - dentro).abs() <= tol, "faixa inteira k={k} p={p}");
                assert!(
                    (r.x - (g.lista.x + Geometria::MARGEM_DA_LISTA)).abs() <= tol,
                    "margem k={k}"
                );
                if p + 1 < k {
                    let s = g.linha_da_lista(p + 1);
                    assert!((s.y - r.y - row).abs() <= tol, "passo sem vão k={k} p={p}");
                }
            }
            let t = g.titulo_da_lista();
            assert!(
                (g.linha_da_lista(0).y - t.y - row).abs() <= tol,
                "título k={k}"
            );
        }
        let f = Geometria::FONTE_DA_LISTA.px();
        assert!(
            Geometria::lado_do_quadradinho(f) < f,
            "o quadradinho é menor que a letra"
        );
    }

    /// ⭐ A faixa da linha pairada passa SOB as células da linha (onda 9g): cobre do rótulo à última célula
    /// à vista da linha (o triângulo de baixo), a qualquer rolagem.
    #[test]
    fn a_faixa_da_linha_pairada_passa_sob_as_celulas() {
        let row = ROW_H_PX;
        let area = Rect::new(row, row, row * 40.0, row * 24.0);
        let tol = StrokeTokenPx::um();
        for k in [5usize, 16, 32] {
            for rolada in [(0, 0), (k / 2, k / 3)] {
                let g = Geometria::de(area, row * 4.0, row * 8.0, k, row * 4.0).rolada(rolada);
                for l in 0..k {
                    if !g.linha_a_vista(l) {
                        continue;
                    }
                    let pil = g.pilula_da_linha(l);
                    let rot = g.rotulo_da_linha(l);
                    assert!(pil.x <= rot.x + tol, "começa no rótulo k={k} l={l}");
                    for c in 0..=l {
                        if !g.coluna_a_vista(c) {
                            continue;
                        }
                        let cel = g.celula(l, c);
                        assert!(
                            pil.x <= cel.x + tol && pil.x + pil.w + tol >= cel.x + cel.w,
                            "a faixa cobre a célula k={k} l={l} c={c} {rolada:?}"
                        );
                    }
                }
            }
        }
    }

    /// 1 px de ecrã, a tolerância do teste.
    struct StrokeTokenPx;
    impl StrokeTokenPx {
        fn um() -> f32 {
            ph2d_tokens::StrokeToken::Hairline.px().max(1.0)
        }
    }
}

//! A geometria da aba das camadas.

use ph2d_editor_core::zones::Rect;
use ph2d_tokens::{ROW_H_PX, Spacing};

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
        let fixo_h = pad + pad + pad + row * 2.0 + rotulo_w;
        let livre_h = area.h - (pad + pad + pad) - camada_h - fixo_h;
        let livre_w = dw - pad * 2.0 - rotulo_w;
        let passo = (livre_h / n)
            .min(livre_w / n)
            .min(row + Spacing::Sm.px())
            .max(Self::passo_minimo());
        let cabe = |livre: f32| ((livre / passo).floor().max(1.0) as usize).min(k.max(1));
        let (cabem_l, cabem_c) = (cabe(livre_h), cabe(livre_w));
        let grade = Rect::new(dx, area.y + pad, dw, fixo_h + passo * cabem_l as f32);
        let camada = Rect::new(dx, grade.y + grade.h + pad, dw, camada_h);
        let lista_h = Self::MARGEM_DA_LISTA * 3.0 + row * (k + 2) as f32; // LITERAL-PX-OK: em cima, antes dos botões e embaixo
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
            x: grade.x + pad,
            y: grade.y + pad + row,
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

    /// A lista não passa desta fração da aba para pôr os dois botões lado a lado.
    pub const PARTES_DA_ABA_PARA_A_LISTA: f32 = 3.0; // LITERAL-PX-OK: um terço da aba

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

    /// A pílula da linha acesa: a mesma altura e o mesmo centro da linha de quadrados.
    #[must_use]
    pub fn pilula_da_linha(&self, l: usize) -> Rect {
        let r = self.rotulo_da_linha(l);
        let cel = self.celula(l, 0);
        Rect::new(r.x - Spacing::Xs.px(), cel.y, r.w, cel.h)
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
        let pad = Spacing::Md.px();
        Rect::new(
            self.grade.x + pad,
            self.y + self.rotulo_w + self.cabem_l as f32 * self.passo + pad,
            self.grade.w - pad * 2.0,
            ROW_H_PX,
        )
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

    /// ⭐ A margem lateral da lista (as linhas e os botões) e o vão entre os dois botões (onda 9e, pedido do
    /// dono: os botões DENTRO do cartão da lista, como a coluna «Animations» do editor de animações).
    pub const MARGEM_DA_LISTA: f32 = Spacing::Sm.factory_px();

    /// A largura que a lista pede para os dois botões de `botao_w` lado a lado, com as margens e o vão.
    #[must_use]
    pub fn lista_para_os_botoes(botao_w: f32) -> f32 {
        botao_w * 2.0 + Self::MARGEM_DA_LISTA * 3.0 // LITERAL-PX-OK: dois botões, duas margens e o vão
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
        Rect::new(r.x, r.y + Self::MARGEM_DA_LISTA, self.botao_w, ROW_H_PX)
    }

    /// O «Remove»: ao lado do «+ Add layer», com o vão; ou debaixo dele.
    #[must_use]
    pub fn remove(&self) -> Rect {
        let a = self.add();
        if self.botoes_empilhados() {
            Rect::new(a.x, a.y + a.h + Self::MARGEM_DA_LISTA, a.w, a.h)
        } else {
            Rect::new(a.x + a.w + Self::MARGEM_DA_LISTA, a.y, a.w, a.h)
        }
    }
}

/// Uma fila que quebra: um retângulo de uma linha por largura, da esquerda para a direita a partir de
/// `(x, y)` dentro de `w`. Devolve os retângulos e quantas linhas ocupou.
#[must_use]
pub fn fila(x: f32, y: f32, w: f32, larguras: &[f32]) -> (Vec<Rect>, usize) {
    let (vao, vao_v) = (Spacing::Sm.px(), Spacing::Xs.px());
    let mut v = Vec::with_capacity(larguras.len());
    let (mut cx, mut linha) = (x, 0usize);
    for &lw in larguras {
        if cx > x && cx + lw > x + w {
            cx = x;
            linha += 1;
        }
        v.push(Rect::new(
            cx,
            y + linha as f32 * (ROW_H_PX + vao_v),
            lw,
            ROW_H_PX,
        ));
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

    /// 1 px de ecrã, a tolerância do teste.
    struct StrokeTokenPx;
    impl StrokeTokenPx {
        fn um() -> f32 {
            ph2d_tokens::StrokeToken::Hairline.px().max(1.0)
        }
    }
}

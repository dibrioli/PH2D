//! ⭐⭐⭐ **O CARTÃO substitui o RISCO — o modelo de painel do Blender, trazido para cá.**
//!
//! Enio, 2026-09-06, com três telas lado a lado e a palavra CARD escrita à mão sobre a do
//! Blender: *«Gostei do modo Blender onde uma secção está dentro de um card. Uma subsecção está
//! com o seu título dentro do card da secção mas o seu conteúdo fica dentro de outro
//! card/container de cor diferente. Estude o Blender e traga isso para nós. Vamos eliminar os
//! nossos divisores azuis.»*
//!
//! ## O modelo, do manual do Blender (CC-BY-SA — o código dele é GPL e esta linha não o lê)
//!
//! > *«The smallest organizational unit in the user interface is a panel. The panel header shows
//! > the title of the panel. It is always visible. Some panels also include subpanels.»*
//! > — `interface/window_system/tabs_panels.rst`
//!
//! E do HIG (`layouts.md`), a razão de o sub-painel ganhar ao risco:
//!
//! > *«When a label would help give context to multiple buttons, it often makes sense to organize
//! > them in a subpanel. The use of subpanels is generally preferred over a single label button in
//! > a row above a block of buttons.»*
//!
//! ⇒ **a fronteira de uma secção é a BORDA DE UM CORPO, não uma linha desenhada entre dois
//! vizinhos.** Um risco diz *«acabou»* e não diz *«do quê»*: com ele, o espaço acima e o espaço
//! abaixo pertencem a ninguém, e é por isso que a folga em volta dele se lia irregular por mais
//! que a apertássemos (waves 7 e 8). Um cartão responde às duas perguntas de uma vez.
//!
//! ## ⭐⭐ O mecanismo, e porque ele NÃO re-dispõe nada
//!
//! O nosso desenho é imediato: um pintor de secção anda de cima para baixo e devolve o `y`
//! seguinte. Um cartão precisa de ser pintado **por baixo** de um conteúdo cuja altura só se
//! conhece **depois** de o pintar — o que parece exigir duas passagens (e duas passagens
//! registariam o hit-index duas vezes, o que é um defeito e não um custo).
//!
//! A saída é **estacionar a cena**: o corpo do painel pinta-se numa cena vazia, os cartões vão
//! para a cena real, e o corpo é devolvido por cima com `Scene::append`. ⚠️ É lícito porque o
//! [`VectorScene`] é um *newtype* de UM campo sobre a cena do Vello — a troca é **sem perdas** —
//! e porque um `append` herda a pilha de recortes aberta, que é o que mantém a rolagem do painel
//! a funcionar.
//!
//! ⭐ **E o cartão é um RECUO PARA FORA do bloco já pintado** (`CARD_PAD` em volta), nunca uma
//! caixa que empurra o conteúdo para dentro: *nenhuma linha muda de sítio, logo nenhum gesto muda
//! de alvo.* Foi isto que tornou a wave possível sem tocar na disposição de 30 secções.
//!
//! ## A escada de fundos, e é ela que diz «subsecção»
//!
//! | superfície | token | Dark |
//! |---|---|---|
//! | o painel | `panel-bg` | `#131313` |
//! | o cartão de uma **secção** | `bg-1` | `#1f1f1f` |
//! | o cartão de uma **subsecção** | `bg-2` | `#292929` |
//!
//! Os degraus são de 12 e 10 em 255 — a mesma medida que o dono aprovou para o cartão de asset
//! contra o painel (§7.7). ⚠️ O título de uma subsecção fica no cartão do PAI e só o **conteúdo**
//! dela desce para o cartão claro: é literalmente o que ele descreveu, e é o que o Blender faz.
//!
//! ## ⛔ O tema CLÁSSICO não muda
//!
//! `PH2D_UI_NEW=0` continua a desenhar o risco de sempre. O cartão é a família moderna, e a
//! escolha vive numa porta só ([`SectionCards::close`]), nunca num `if` por painel.

use crate::paint::{fill_rounded_rect, resolve};
use crate::zones::Rect;
use ph2d_tokens::{ColorToken, Radius, Spacing, Theme};
use ph2d_vector::VectorScene;

/// A folga que o cartão ganha para fora do conteúdo que envolve.
///
/// ⚠️ **Ela é o `Spacing::Xs` da casa (4 px) porque é a MESMA pergunta do vão entre linhas** — o
/// `separation_margin` do Godot, que a wave 8 pôs numa porta. Um número próprio aqui seria a
/// oitava resposta a uma pergunta que já tem uma.
fn card_pad() -> f32 {
    Spacing::Xs.px()
}

/// ⭐ **A que profundidade um cartão está** — e é só isso que separa uma secção de uma subsecção.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CardDepth {
    /// O corpo de uma secção, sobre o painel.
    Section,
    /// O corpo de uma **subsecção**, sobre o cartão da secção — mais claro, um degrau acima.
    Subsection,
}

impl CardDepth {
    fn token(self) -> ColorToken {
        match self {
            Self::Section => ColorToken::Bg1,
            Self::Subsection => ColorToken::Bg2,
        }
    }
}

/// ⭐⭐ **O livro dos cartões** — recolhe os rects enquanto o corpo se pinta.
///
/// ⚠️ **Ele guarda o CURSOR**, e é isso que torna a conversão de um sítio de chamada uma troca de
/// nome: o pintor de secção que fazia `y = paint_section_separator(scene, theme, x, w, y)` passa a
/// fazer `y = cards.close(x, w, y)` — a mesma forma, o mesmo `y`. *O separador já marcava o fim de
/// uma secção; ele só não sabia dizer o princípio.*
#[derive(Debug)]
pub struct SectionCards {
    theme: Theme,
    /// Onde começa o cartão que ainda não fechou.
    cursor: f32,
    rects: Vec<(Rect, CardDepth)>,
}

impl SectionCards {
    fn new(theme: Theme, top: f32) -> Self {
        Self {
            theme,
            cursor: top,
            rects: Vec::new(),
        }
    }

    fn close(&mut self, scene: &mut VectorScene, x: f32, w: f32, y: f32) -> f32 {
        self.close_at(scene, x, w, y, CardDepth::Section)
    }

    fn close_sub(&mut self, scene: &mut VectorScene, x: f32, w: f32, y: f32) -> f32 {
        self.close_at(scene, x, w, y, CardDepth::Subsection)
    }

    fn close_at(
        &mut self,
        scene: &mut VectorScene,
        x: f32,
        w: f32,
        y: f32,
        depth: CardDepth,
    ) -> f32 {
        if !self.theme.is_modern() {
            let next = crate::widget::showcase::paint_section_separator(scene, self.theme, x, w, y);
            self.cursor = next;
            return next;
        }
        let pad = card_pad();
        if y > self.cursor {
            let r = Rect::new(
                x - pad,
                self.cursor - pad,
                w + pad * 2.0,
                (y - self.cursor) + pad * 2.0,
            );
            self.rects.push((r, depth));
        }
        // O vão entre dois cartões é o mesmo vão de sempre, contado do fim do CONTEÚDO — o `pad`
        // de baixo do cartão já é metade dele.
        let next = y + pad * 2.0;
        self.cursor = next;
        next
    }

    fn paint_into(&self, scene: &mut VectorScene) {
        let radius = crate::paint::frame_radius(self.theme, Radius::Md.px());
        for (rect, depth) in &self.rects {
            fill_rounded_rect(scene, *rect, radius, resolve(depth.token(), self.theme));
        }
    }
}

/// ⭐⭐⭐ **Pinta `body` com os cartões por baixo.**
///
/// Devolve o que o corpo devolver. Ver o doc do módulo para o porquê da cena estacionada.
///
/// ⛔ **No tema CLÁSSICO não há cartão** — o `body` corre directamente sobre a cena real, e os
/// `close` desenham o risco de sempre. Sem esse braço, `PH2D_UI_NEW=0` deixaria de ser o clássico.
pub fn with_section_cards<R>(
    scene: &mut VectorScene,
    theme: Theme,
    top: f32,
    body: impl FnOnce(&mut VectorScene) -> R,
) -> R {
    begin_section_cards(scene, theme, top);
    let out = body(scene);
    end_section_cards(scene);
    out
}

/// ⭐⭐⭐ **Abre um corpo com cartões.** O par do [`end_section_cards`].
///
/// Existe ao lado do [`with_section_cards`] porque **nem todo corpo cabe num fecho**: o do
/// Inspector é um troço linear de ~230 linhas entre o `open_body` e o `close_body`, com dezenas de
/// locais vivos, e embrulhá-lo numa closure custaria mais do que a lei que se quer aplicar.
///
/// ⚠️ **Não devolve nada, e isso é medido:** a 1.ª versão devolvia um guarda `#[must_use]` que o
/// chamador tinha de carregar até ao fecho — e no Inspector isso custou **uma linha** no
/// destructuring, que é exactamente o que o tecto de 200 LOC por função daquele orquestrador não
/// tinha para dar. *A pilha já existia para o livro; a cena estacionada mora ao lado dele.*
///
/// ⛔ Esquecer o [`end_section_cards`] deixa a cena real estacionada e o painel **desaparece** —
/// falha alta, como o `push_clip`/`pop_layer` desta casa.
pub fn begin_section_cards(scene: &mut VectorScene, theme: Theme, top: f32) {
    if !theme.is_modern() {
        // ⛔ No clássico não há livro: o [`close_section`] desenha o risco de sempre.
        return;
    }
    let mut parked = VectorScene::new();
    std::mem::swap(scene, &mut parked);
    LEDGER.with(|l| l.borrow_mut().push((SectionCards::new(theme, top), parked)));
}

/// **Fecha o corpo:** pinta os cartões na cena real e devolve o corpo por cima.
pub fn end_section_cards(scene: &mut VectorScene) {
    let Some((cards, mut parked)) = LEDGER.with(|l| l.borrow_mut().pop()) else {
        return;
    };
    // O corpo sai; a cena real volta.
    let mut painted_body = VectorScene::new();
    std::mem::swap(scene, &mut painted_body);
    std::mem::swap(scene, &mut parked);

    cards.paint_into(scene);
    scene.inner_mut().append(painted_body.inner(), None);
}

thread_local! {
    /// A PILHA dos corpos a serem pintados agora — o livro de cada um e a cena que ele estacionou.
    ///
    /// ⚠️ **É o mesmo padrão do [`crate::published`]** — *a shell publica, a folha lê* — e pela
    /// mesma razão medida: enfiar o livro na assinatura de cada pintor de secção custaria ~20
    /// assinaturas **só no Inspector**, para responder vinte vezes a uma pergunta que o quadro
    /// responde uma vez. ⚠️ **É uma PILHA e não um slot**: um painel pintado dentro de outro
    /// devolve o corpo do pai ao sair, em vez de o apagar.
    static LEDGER: std::cell::RefCell<Vec<(SectionCards, VectorScene)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// ⭐⭐⭐ **A FRONTEIRA de uma secção** — a função que substitui `paint_section_separator`.
///
/// Devolve o `y` de onde o próximo conteúdo começa, exactamente como o risco devolvia.
///
/// ⚠️ **Sem um [`with_section_cards`] à volta ela desenha o RISCO**, e isso é a decisão: um painel
/// meio-convertido mostra a fronteira antiga — visível, e portanto corrigível — em vez de perder a
/// fronteira toda. *A falha silenciosa aqui seria uma secção sem princípio nem fim.*
pub fn close_section(scene: &mut VectorScene, theme: Theme, x: f32, w: f32, y: f32) -> f32 {
    close_with(scene, theme, x, w, y, CardDepth::Section)
}

/// O mesmo, para o corpo de uma **subsecção** — o cartão mais claro, sobre o do pai.
pub fn close_subsection(scene: &mut VectorScene, theme: Theme, x: f32, w: f32, y: f32) -> f32 {
    close_with(scene, theme, x, w, y, CardDepth::Subsection)
}

fn close_with(
    scene: &mut VectorScene,
    theme: Theme,
    x: f32,
    w: f32,
    y: f32,
    depth: CardDepth,
) -> f32 {
    let Some((mut cards, parked)) = LEDGER.with(|l| l.borrow_mut().pop()) else {
        return crate::widget::showcase::paint_section_separator(scene, theme, x, w, y);
    };
    let next = match depth {
        CardDepth::Section => cards.close(scene, x, w, y),
        CardDepth::Subsection => cards.close_sub(scene, x, w, y),
    };
    LEDGER.with(|l| l.borrow_mut().push((cards, parked)));
    next
}

/// ⚠️ **Um cabeçalho fica FORA do cartão** — o título de uma secção vive sobre o painel, e o de
/// uma subsecção sobre o cartão do pai. Quem pinta um avança o cursor sem fechar nada, senão o
/// cartão engoliria o próprio título. Fora de um [`with_section_cards`] é um no-op.
pub fn skip_section_header(y: f32) {
    LEDGER.with(|l| {
        if let Some((c, _)) = l.borrow_mut().last_mut() {
            c.cursor = y;
        }
    });
}

#[cfg(test)]
mod tests;

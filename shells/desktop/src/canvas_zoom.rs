//! **O zoom do canvas obedece à roda, e CHEGA lá.**
//!
//! A roda sobre o canvas multiplicava `height_world` na hora (`camera.zoom(factor)`), então o
//! gesto que o artista repete o dia inteiro era o único movimento da tela que SALTA. Este módulo
//! põe o mesmo escalar no substrato da UI viva: a roda escreve um **destino**, e o quadro publica
//! o **vivo**.
//!
//! ## O papel é `Surface`, e não é gosto
//!
//! [`Role::Surface`] nasceu do smoke da rolagem (*«o balanço das labels ficou bem artificial»*) e
//! diz o que uma superfície comandada pelo dedo É: **nunca ultrapassa, nos dois carácteres**. Um
//! zoom que passa do destino e volta não lê como peso — lê como enjoo, e mostra o que os dois
//! clamps existem para proibir. E porque a mola é criticamente amortecida, **o vivo nunca sai do
//! intervalo** em que o destino foi cravado: a garantia é a matemática, não uma segunda checagem.
//!
//! ⚠️ Sob *reduced motion* a [`UiMotion::law`] devolve `None` e a roda volta a ser
//! instantânea — **byte-idêntica** ao que sempre shipou.
//!
//! ## O percurso é medido em LOG, e o que ele compra é SIMETRIA
//!
//! O zoom é multiplicativo: a roda entrega `0,9^n`, e o que o olho julga é `ln(h)`. Interpolar
//! `height_world` cru dá um percurso cujo ritmo PERCEBIDO depende do sentido — medido a meio
//! caminho da mola: **41,5% percorrido a aproximar contra 58,5% a afastar** num fator de 2. O
//! mesmo gesto, dois ritmos. Em `ln(h)` são **50,0% nos dois**, por construção.
//!
//! | fator | a meio da mola, aproximar | afastar | desvio |
//! |---|---|---|---|
//! | um entalhe (0,900) | 48,7% | 51,3% | 2,6 pontos |
//! | uma rajada de cinco (0,590) | 43,5% | 56,5% | **13,0 pontos** |
//! | 2× (0,500) | 41,5% | 58,5% | 17,0 pontos |
//!
//! Um entalhe isolado não se vê; **a rajada vê-se**, e a roda é girada em rajada.
//!
//! ⚠️ **E a justificação que eu ia shipar estava ERRADA — a derivação apanhou-a antes do código.**
//! Eu tinha escrito que o espaço linear faz *um entalhe* medir «200× mais no topo do intervalo que
//! no fundo» (`0,05` contra `10`) e que por isso a lei da interrupção (`v / span`) se comportaria
//! de forma diferente perto e longe. O primeiro número é verdade e **o segundo é inerte**: em
//! `h(t) = h₀·(1 + (f−1)x)` o `h₀` sai de TODAS as grandezas — do valor, da velocidade e do span —,
//! então `v / span` não depende dele. Medido em três níveis de zoom, o espaço linear dá
//! **48,6836% em todos**. *Um número verdadeiro sobre uma grandeza não é um argumento sobre o
//! comportamento dela*, e o gate que eu ia escrever para o provar não tocava neste módulo — teria
//! ficado verde por vácuo sobre a mudança que ele julgava vigiar.
//!
//! ## A posse da câmera volta quando o gesto acaba
//!
//! O destino é um [`Option`]: enquanto ninguém dá zoom, este módulo **não toca** na câmera e ela
//! é de quem a escrever (o *fit-to-view*, o load, as cenas de smoke). Ao chegar um entalhe o
//! destino é semeado a partir do **vivo**, então ele não pode nascer velho — o defeito das duas
//! cópias é impossível por construção, e não por disciplina.
//!
//! ⚠️ E uma escrita ESTRANGEIRA no meio de um voo ganha: o tique compara a câmera com o que ele
//! próprio publicou no quadro anterior — *enumerar os escritores apodrece, uma testemunha não*.

use ph2d_editor::NodeId;
use ph2d_editor::motion::{Role, UiMotion};
use ph2d_render::Camera2d;

/// O id do escalar no substrato. Hash de string ⇒ **nenhum contador de gate** se move.
pub(crate) const CANVAS_ZOOM: NodeId = ph2d_tool_registry::hash_node_id("canvas.zoom");

/// O destino do zoom do canvas, e a testemunha que devolve a câmera aos outros donos.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CanvasZoom {
    /// O destino AUTORADO, em `height_world`. `None` = **a câmera não é minha**.
    target: Option<f32>,
    /// O último `height_world` que ESTE módulo publicou — a testemunha de escrita estrangeira.
    published: f32,
}

impl CanvasZoom {
    /// **Um entalhe de roda.** `live` é o `height_world` de agora; `factor < 1` aproxima.
    ///
    /// ⚠️ **Compõe no DESTINO, nunca no vivo** — a lição que a rolagem de painel já pagou: cinco
    /// voltas compostas sobre um valor a meio caminho somam bem menos do que cinco voltas.
    pub(crate) fn wheel(&mut self, live: f32, factor: f32) {
        if !factor.is_finite() || factor <= 0.0 || !live.is_finite() || live <= 0.0 {
            return;
        }
        let base = match self.target {
            Some(t) => t,
            None => {
                // O gesto começa: a testemunha nasce igual ao vivo, senão o primeiro tique acha
                // que alguém escreveu por fora e larga a câmera no mesmo quadro.
                self.published = live;
                live
            }
        };
        // ⚠️ **O clamp é o da CÂMERA, perguntado — nunca copiado.** Uma segunda cópia divergiria
        // no dia em que um dos dois limites se mover, e o artista veria a roda parar num sítio que
        // o resto do app não conhece.
        self.target = Some(Camera2d::zoomed(base, factor));
    }

    /// **O tique do quadro.** Devolve o `height_world` a publicar, ou `None` quando a câmera não
    /// é deste módulo.
    ///
    /// ⚠️ **A SEMEADURA é obrigatória e é a lei do substrato:** *a primeira vista de um id chega
    /// ao alvo*. Sem plantar a track no valor VIVO antes de alvejar, o primeiro entalhe de cada
    /// gesto saltaria — o defeito de uma-vez-por-gesto que a estreia das secções já pagou.
    pub(crate) fn tick(&mut self, live: f32, motion: &mut UiMotion) -> Option<f32> {
        let target = self.target?;
        if live != self.published {
            // Escrita estrangeira (fit-to-view, load, cena de smoke): ela é a verdade.
            self.target = None;
            return None;
        }
        let goal = target.ln();
        if motion.get(CANVAS_ZOOM).is_none() {
            let _ = motion.animate(CANVAS_ZOOM, live.ln(), Role::Surface);
        }
        let now = motion.animate(CANVAS_ZOOM, goal, Role::Surface);
        // ⚠️ **O repouso publica o DESTINO, não `exp(ln(destino))` — e esta linha carrega DUAS
        // promessas.** (1) O `height_world` assentado é o que os ~90 leitores da câmera veem e o
        // que um entalhe futuro compõe; deixá-lo com o resíduo do round-trip (~1e-6 relativo)
        // seria a régua a mentir na quarta casa, para sempre. (2) É **ela** que torna o *reduced
        // motion* byte-idêntico ao mundo pré-wave: com a lei instantânea o `animate` devolve o
        // alvo no mesmo quadro, e este ramo publica-o **verbatim**.
        //
        // ⚠️ E o (2) foi aprendido por uma mutação que SOBREVIVEU: eu tinha escrito um atalho
        // `if law(Surface).is_none() { return Some(target) }` e um doc a dizer que ele existia
        // para evitar a ida-e-volta pelo log. Removê-lo não sangrou gate nenhum — porque este
        // ramo já entregava exactamente a mesma coisa. *Um caso especial que nenhuma medição
        // distingue do caso geral é um caso especial que não existe.*
        let next = if now == goal {
            // Assentou (a track põe o valor EXACTO e larga o voo) — a câmera volta a ser de quem
            // a escrever.
            self.target = None;
            target
        } else {
            // ⚠️ **Sem clamp no VIVO, de propósito.** O destino já é cravado, e a mola de
            // `Surface` é criticamente amortecida ⇒ não ultrapassa: o vivo está no intervalo por
            // construção. Cravá-lo aqui só saltaria a imagem quando o gesto parte de uma câmera
            // que outro dono deixou fora da faixa.
            now.exp()
        };
        self.published = next;
        Some(next)
    }

    /// O destino em voo, para os gates e para o diagnóstico. `None` = ninguém está a dar zoom.
    #[cfg(test)]
    pub(crate) fn target(self) -> Option<f32> {
        self.target
    }
}

#[cfg(test)]
#[path = "canvas_zoom_tests.rs"]
mod tests;

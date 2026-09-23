//! **AS DUAS REPOSIÇÕES DA PILHA — descascar e FIXAR.**
//!
//! A pilha do Composite Brush acumula por TRAÇO, e uma sessão de figuras é **um** traço (o pen-up
//! não fecha nada — a figura fica editável até ao Apply). Há por isso dois actos que invalidam o
//! que ela guarda, e eles são **opostos**:
//!
//! | acto | o que muda | o que a pilha tem de esquecer |
//! |---|---|---|
//! | **DESCASCAR** (um quadro de re-carimbo) | a tela volta ao que o `pre` guarda | os PLANOS, não o `pre` |
//! | **FIXAR** (o Enter / Apply) | os pixels passam a ser permanentes | **tudo, o `pre` incluído** |
//!
//! ⚠️ *É por serem opostos que as duas vivem lado a lado aqui:* escritas longe uma da outra, a
//! segunda nasce como cópia da primeira e leva o `pre` que não devia levar — ou não o leva.
//!
//! Cortado do [`super::composite_acumulado`] pelo tecto de LOC, **por responsabilidade**.

use super::composite::N_CAMADAS;
use crate::tool::PainterTool;
use std::sync::Arc;

impl PainterTool {
    /// ⭐⭐⭐ **A TELA FOI DESCASCADA ⇒ A PILHA ESQUECE O QUE ACUMULOU.**
    ///
    /// Report do dono (2026-09-21, com foto): *«os Stroke:Method vivos (booleanos) — Ellipse,
    /// Polygon, Line — não funcionam corretamente com o composite, mudam de aparência e o Boolean
    /// não funciona»* e *«undo/redo … podem deixar resíduos»*. **Os dois são o mesmo defeito.**
    ///
    /// Um método de RE-CARIMBO (os cinco editores de figura + Drag Dot / Anchored / Line) não
    /// acrescenta tinta: ele **restaura** o recorte do quadro anterior e re-emite a lista INTEIRA
    /// de dabs sobre a tela limpa. O acumulador desta pilha é por TRAÇO, e o traço de uma sessão de
    /// figuras dura o que a sessão durar — logo os planos guardavam **toda geometria por onde a mão
    /// passou**: a figura anterior, a posição anterior, e o arco que o boolean tinha acabado de
    /// tirar. A composição repõe-nos, e o que se vê é a lei a não funcionar.
    ///
    /// Medido antes da cura, pelo caminho do produto (duas circunferências `Add` que se cruzam,
    /// pilha `Blur`/`Brush`, o branco é `255`):
    ///
    /// | pilha | tinta | o arco interior (tem de SUMIR) |
    /// |---|---|---|
    /// | sem pilha | `2 391` | `255` — branco ✓ |
    /// | 1 camada | `2 391` | `255` ✓ |
    /// | `Blur`/`Brush` | `2 839` | **`26`** ✗ |
    /// | `Smear`/`Brush` | `672` | **`0`** ✗ |
    /// | `Erase`/`Brush` | `2 839` | **`0`** ✗ |
    ///
    /// …e arrastar UMA figura de um lado para o outro deixava **`+22 %`** a `+67 %` de tinta a mais
    /// (o RASTO), contra `+0 %` sem pilha.
    ///
    /// ⚠️ **O `pre` FICA, e isso não é economia — é a lei.** O descascar devolve a tela a
    /// exactamente o que o `pre` guarda; re-fotografá-la por evento custaria uma cópia do canvas
    /// inteiro por movimento do rato (67 MB a 4096²) para escrever os mesmos bytes.
    ///
    /// ⚠️ **O CAP de Accumulate de cada camada vai junto**, e sem ele a cura seria pior que o
    /// defeito: o cap é um mapa de cobertura por traço, o plano nasce vazio e o cap nasce cheio —
    /// a camada re-depositaria **ZERO** e a figura desapareceria sob a mão.
    ///
    /// ⚠️ **Ela é irmã do [`Self::restamp_reset_sculpt`] e do [`Self::restamp_reset_erase`]**, e
    /// mora nos MESMOS dois sítios: o descascar do [`super::stamp_preview::PainterTool::stamp_drag_preview`]
    /// e a porta de cancelar. *Três canais que um re-carimbo tem de repor estavam escritos lado a
    /// lado; este era o quarto, e ninguém lhe perguntou.*
    pub(super) fn restamp_reset_pilha(&mut self) {
        // A tela acabou de ser DESCASCADA para o `pre`: o que estava por compor descrevia a figura
        // que saiu, logo DESCARTA-se (`composite_por_quadro`) — compô-lo reescrevê-la-ia.
        self.descarta_o_pendente();
        // ⚠️⚠️ **O ESFREGÃO tem o acumulador DELE, e ele fica FORA da guarda abaixo.** O campo de
        // deslocamento (`paint.warp`) é por TRAÇO como os planos, e a base congelada dele é a tela
        // do pen-down — depois de um descascar ela descreve uma figura que já não está lá. Sem
        // isto, arrastar uma figura com uma camada `Smear` deixava **`+67 %`** de tinta a mais
        // (medido; com a cura, `+0 %`). ⭐ A porta é a que o pen-up já usa, e ela **exclui o
        // Deform** por construção — a sessão dele atravessa traços de propósito.
        self.end_smear_session();
        // ⚠️ `pre` vazio = a pilha nem abriu. Sem esta saída antecipada um pincel comum pagaria
        // sete `clear()` por evento para não mudar um bit.
        if self.paint.pilha.pre.is_empty() {
            return;
        }
        for p in &mut self.paint.pilha.planos {
            *p = Arc::new(Vec::new());
        }
        // A rota de bissecção (`PH2D_COMPOSITE_REPLAY=1`) guarda a história em `lotes` e tem a
        // MESMA doença — uma cura que só tratasse a rota de omissão deixaria a bissecção a medir
        // outro programa.
        self.paint.pilha.lotes.clear();
        // ⛔ **O cap de Accumulate NÃO é limpo aqui, e a razão é que ele JÁ o é.** A 1.ª redacção
        // desta cura limpava o `composite_mask`, e a mutação que a apagava **sobreviveu**: o
        // [`super::stamp_cache::prepare_stroke_mask`] já o zera para todo método que não é
        // incremental, com esta lei escrita no doc dele desde que existe (*«os métodos de
        // preenchimento re-carimbam o TRAÇO INTEIRO, logo o cap tem de começar FRESCO»*) — e o
        // `composite_mask[pos]` está TROCADO para dentro do `stroke_mask` quando ele corre.
        // *Duas respostas à mesma pergunta divergem no dia em que uma delas mudar de chave*, e a
        // que eu ia escrever era indexada pelo DESCASCAR contra uma indexada pelo MÉTODO.
        //
        // ⭐ **O ARCO é outra história e é dívida REAL:** ele é o acumulador da subamostragem de
        // uma camada MAIOR que o pincel, e ninguém mais o repõe. Medido com duas camadas `Brush`
        // de `size = 3`, a figura re-carimbada quase no mesmo sítio: **`740` texels contra
        // `5 641`** — o `d.arc_len` recomeça do zero a cada re-carimbo e o acumulador do carimbo
        // anterior recusa a lista inteira.
        self.paint.composite_arco = [f32::NEG_INFINITY; N_CAMADAS];
    }

    /// ⭐⭐⭐ **O PREVIEW FOI FIXADO ⇒ A PILHA MORRE, PORQUE A TELA MUDOU DEBAIXO DELA.**
    ///
    /// Report do dono (2026-09-21, com foto): *«apertei enter para fixar o desenho das formas vivas
    /// e tentei desenhar de novo com a elipse: o retângulo voltou mas com a cor do canvas cobrindo
    /// o desenho anterior»*.
    ///
    /// ⛔⛔ **O pen-up de uma figura NÃO fecha o traço** — a figura fica editável, e é isso que faz
    /// a sessão inteira ser UM traço. Enquanto ela dura, o lote é a CONCATENAÇÃO das figuras vivas
    /// e cada quadro re-carimba TODAS a partir do `pre`, logo nada se perde. O **Enter**
    /// ([`super::curve_commit::PainterTool::commit_open_shape`]) quebra exactamente essa premissa:
    /// ele assa os pixels, larga os editores — e o `pre` da pilha continua a ser a tela de **antes**
    /// das figuras assadas. A figura seguinte reconstrói-se sozinha a partir dessa base velha e, na
    /// região dela, **apaga o que acabou de ser fixado**.
    ///
    /// **Medido** (tela preta transparente, pilha do dono, duas elipses de raio `110`):
    ///
    /// | | arte destruída |
    /// |---|---|
    /// | sem Enter entre as duas | `0` |
    /// | **com Enter entre as duas** | **`12 530` texels**, pior `127`, caixa `119×280` |
    ///
    /// ⚠️ **Ela é a QUARTA da lista do [`super::stamp_preview::PainterTool::commit_drag_preview`]**,
    /// que já mata o relevo do traço, a sessão do escultor e a da borracha pelo mesmo motivo — *o
    /// que foi fixado é permanente, logo o estado por-traço que o descrevia deixou de valer*. É a
    /// mesma forma da [`Self::restamp_reset_pilha`], um acto adiante: lá o gatilho é DESCASCAR,
    /// aqui é FIXAR.
    ///
    /// ⚠️ E ela não é a `restamp_reset_pilha`: aquela mantém o `pre` de propósito (a tela volta a
    /// ser exactamente o que ele guarda); aqui é **o `pre` que ficou errado**.
    pub(super) fn commit_reset_pilha(&mut self) {
        #[cfg(test)]
        if COMMIT_SEM_FECHAR.with(std::cell::Cell::get) {
            return; // a bissecção: o commit de ANTES da cura, para o A/B ser repetível.
        }
        // O que ficou por compor entra na tela ANTES de ela ser fixada (`composite_por_quadro`).
        self.compoe_o_pendente();
        // O campo de deslocamento do esfregão é por traço e a base congelada dele é a tela do
        // pen-down — depois de fixar, ela descreve uma tela que já não existe.
        self.end_smear_session();
        if self.paint.pilha.pre.is_empty() {
            return; // a pilha nem abriu: um pincel comum não paga nada por esta porta.
        }
        self.paint.pilha.fecha();
        // O cap de Accumulate e o acumulador de arco são do TRAÇO, e o traço que eles descreviam
        // acabou de virar pixels permanentes.
        for m in &mut self.paint.composite_mask {
            m.clear();
        }
        self.paint.composite_arco = [f32::NEG_INFINITY; N_CAMADAS];
    }
}

#[cfg(test)]
thread_local! {
    /// **A bissecção da cura de 2026-09-21:** ligada, fixar o preview volta a NÃO fechar a pilha,
    /// que é o defeito do report do dono. ⚠️ Um CAMPO e não uma env var — *um gate que lê o
    /// ambiente mede a máquina*.
    pub(super) static COMMIT_SEM_FECHAR: std::cell::Cell<bool> =
        const { std::cell::Cell::new(false) };
}

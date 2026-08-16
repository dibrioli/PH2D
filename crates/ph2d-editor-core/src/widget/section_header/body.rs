//! **A DOBRA do CORPO de uma secção** — a altura interpola em vez de saltar.
//!
//! A wave F4a deu o `t` ao CABEÇALHO (o chevron roda, a placa desvanece). Esta dá-o ao CORPO, e
//! as duas metades passam a dizer a mesma coisa sobre o mesmo instante.
//!
//! # Porque isto é uma PORTA e não três linhas em cada pintor
//!
//! Um corpo que se dobra precisa de **três** coisas de cada vez, e omitir uma delas produz um
//! defeito diferente:
//!
//! | metade | o que a ausência dela produz |
//! |---|---|
//! | o recorte da **CENA** | as rows transbordam por cima da secção de baixo |
//! | o recorte do **HIT** | uma row invisível continua a responder ao rato |
//! | o `y` de **SAÍDA** escalado | o corpo desliza e nada por baixo o acompanha |
//!
//! ⚠️ **E há uma quarta, e o número dela foi MEDIDO depois de eu o ter afirmado errado:** o
//! `t == 1` devolve o `cur_y` **verbatim** em vez de `body_top + (cur_y − body_top)`. Varridos
//! 59,2 M pares `(topo, fim)` plausíveis num painel, as duas expressões diferem em **0,644%**
//! deles — e quando diferem é **exactamente 1 ULP, 3,05e-5 px**. ⚠️ **Isso NÃO é um defeito
//! visível, e a primeira versão deste comentário dizia que era.** A razão de o ramo existir é
//! outra e é sobre o GATE: a neutralidade que esta migração promete é *byte a byte*, e essa
//! propriedade prova-se com um `to_bits()`; trocá-la por *«difere menos de um pixel»* pede um
//! épsilon, e um épsilon dessa ordem passa também para um defeito real. **O ramo compra o
//! oráculo afiado, não o pixel.**
//!
//! # O NEUTRO é os dois REPOUSOS
//!
//! - **Aberta e parada** (`t = 1`): nenhuma camada é empurrada, o `y` sai verbatim ⇒ a cena é
//!   **byte a byte** a de antes desta wave.
//! - **Fechada e parada** (`t = 0`): [`SectionFold::begin`] devolve `None` e o pintor sai com o
//!   `body_top` que já tinha na mão ⇒ exactamente o `if collapsed { return y + header_h; }` de
//!   sempre.
//!
//! Só ENTRE os dois repousos é que alguma coisa difere — que é onde a wave existe.
//!
//! ⚠️ **O pintor deixa de perguntar `is_collapsed` e passa a perguntar o `t`**, e a distinção é
//! load-bearing: ao clicar para fechar, o flag semântico vira no MESMO quadro e o `t` ainda
//! desce, então um corpo gateado no flag **desapareceria de repente** por baixo de um chevron a
//! rodar — as duas metades do cabeçalho a discordar outra vez, um sistema abaixo.

use crate::interaction::{HitIndex, WidgetStore};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_vector::VectorScene;

/// Acima disto a secção conta como ABERTA e parada: sem recorte, `y` verbatim.
const FULL: f32 = 0.999; // LITERAL-PX-OK: epsilon sobre um `t` NORMALIZADO, não medida de design
/// Abaixo disto a secção conta como FECHADA e parada: nada é pintado.
const SHUT: f32 = 0.001; // LITERAL-PX-OK: epsilon sobre um `t` NORMALIZADO, não medida de design

/// **O escopo da dobra do corpo de UMA secção.** Ver o módulo para o desenho.
///
/// ⚠️ `#[must_use]` e um `Drop` que grita em debug: quem abre o escopo e não o fecha deixa uma
/// camada de recorte pendurada na cena, e o sintoma seria *o painel inteiro a desaparecer* —
/// longe da causa.
#[must_use = "uma dobra aberta tem de ser fechada com `finish`, senão o recorte fica pendurado"]
#[derive(Debug)]
pub struct SectionFold {
    id: NodeId,
    /// Quanto desta secção está aberto, `(SHUT, 1.0]`.
    t: f32,
    /// O `y` onde o corpo começa — logo depois do cabeçalho.
    body_top: f32,
    /// `true` quando [`Self::begin`] empurrou recorte na cena E no hit index.
    clipped: bool,
    /// Só para o `Drop` acusar quem esqueceu o `finish`.
    finished: bool,
}

impl SectionFold {
    /// **Abre a dobra.** `None` quando a secção está fechada e parada — aí o pintor sai com o
    /// `body_top`, sem pintar nem medir nada.
    ///
    /// `body_top` é o `y` **logo depois do cabeçalho**; `x`/`w` são a faixa do painel.
    ///
    /// ⚠️ **O recorte usa a altura LEMBRADA e não a medida deste quadro**, porque a medida só
    /// existe depois de as rows serem percorridas — ver
    /// [`WidgetStore::section_body_h`](crate::interaction::WidgetStore::section_body_h). Memória
    /// ausente ⇒ recorte de altura zero: a estreia de uma secção que nunca foi pintada aberta
    /// não mostra nada no primeiro quadro **e mede-se nele**, o que é invisível porque a mola
    /// parte de `t ≈ 0`.
    pub fn begin(
        store: &WidgetStore,
        id: NodeId,
        x: f32,
        w: f32,
        body_top: f32,
        scene: &mut VectorScene,
        hit_index: &mut HitIndex,
    ) -> Option<Self> {
        let t = store.section_open_live(id);
        Self::begin_at(store, id, t, x, w, body_top, scene, hit_index)
    }

    /// A MESMA lei, com o `t` VINDO DE FORA — para o painel que **fotografa** a dobra antes de o
    /// paint tomar os empréstimos.
    ///
    /// ⚠️ Existe porque `begin` responderia a pergunta uma SEGUNDA vez: o `ph2d-panel-audio-editor`
    /// lê os oito `t` do store num array antes de pintar (o corpo dele nunca alcança o store de
    /// volta), e um `begin` que relesse o store daria à dobra um estado e ao chevron outro — as
    /// duas metades a discordar sobre a mesma seção. Quem já tem a resposta passa-a; quem não
    /// tem chama o `begin`, que a busca **uma** vez.
    ///
    /// O `store` continua a entrar porque a ALTURA lembrada é dele — ela é medição, não estado
    /// autorado, e nenhum painel a fotografa.
    #[allow(clippy::too_many_arguments)]
    pub fn begin_at(
        store: &WidgetStore,
        id: NodeId,
        t: f32,
        x: f32,
        w: f32,
        body_top: f32,
        scene: &mut VectorScene,
        hit_index: &mut HitIndex,
    ) -> Option<Self> {
        let t = t.clamp(0.0, 1.0);
        if t <= SHUT {
            return None;
        }
        let clipped = t < FULL;
        if clipped {
            let h = (store.section_body_h(id).unwrap_or(0.0) * t).max(0.0);
            let band = Rect::new(x, body_top, w, h);
            scene.push_clip(&ph2d_vector::Rect::new(
                band.x as f64,
                band.y as f64,
                (band.x + band.w) as f64,
                (band.y + band.h) as f64,
            ));
            hit_index.push_clip(band);
        }
        Some(Self {
            id,
            t,
            body_top,
            clipped,
            finished: false,
        })
    }

    /// O `y` onde o corpo é pintado. **É o `body_top` cru** — o conteúdo fica ancorado no topo e
    /// o recorte come-o por baixo, que é o que uma transição de `height` faz e o que todo
    /// widget de *disclosure* desta classe mostra.
    #[must_use]
    pub fn body_top(&self) -> f32 {
        self.body_top
    }

    /// **Fecha o recorte, MEDE o corpo, LEMBRA a altura e devolve o `y` de saída.**
    ///
    /// `cur_y` é onde o pintor chegou depois de percorrer as rows.
    ///
    /// ⚠️ **A medição fresca alimenta o LAYOUT; a lembrada alimentava só o recorte.** É isso que
    /// torna um valor velho por um quadro inofensivo: o que desce ecrã abaixo é sempre exacto.
    pub fn finish(
        mut self,
        store: &WidgetStore,
        scene: &mut VectorScene,
        hit_index: &mut HitIndex,
        cur_y: f32,
    ) -> f32 {
        self.finished = true;
        if self.clipped {
            scene.pop_layer();
            hit_index.pop_clip();
        }
        let measured = (cur_y - self.body_top).max(0.0);
        store.remember_section_body_h(self.id, measured);
        if self.t >= FULL {
            // ⚠️ VERBATIM — ver o `f32` do doc do módulo.
            cur_y
        } else {
            self.body_top + measured * self.t
        }
    }
}

impl Drop for SectionFold {
    fn drop(&mut self) {
        debug_assert!(
            self.finished,
            "SectionFold aberto e nunca fechado: o recorte da cena ficou pendurado"
        );
    }
}

/// **O `y` de saída de uma secção cujo corpo é UMA expressão**, para os pintores em que abrir um
/// escopo custaria mais do que a dobra vale (uma row só, um bloco de altura constante).
///
/// ⚠️ Nada aqui recorta — ele serve o caso em que o corpo **cabe inteiro dentro da banda** que a
/// dobra deixa. Um corpo mais alto do que `body_h * t` transbordaria, e é por isso que o caminho
/// normal é o [`SectionFold`].
#[must_use]
pub fn folded_gap(store: &WidgetStore, id: NodeId, gap: f32) -> f32 {
    gap * store.section_open_live(id).clamp(0.0, 1.0)
}

/// **O cabeçalho já sabe o `t`; esta é a pergunta gémea para o corpo.** `true` quando há corpo
/// para pintar — aberto, ou a meio de qualquer um dos dois sentidos.
///
/// ⚠️ Existe para os pintores cujo corpo NÃO é uma sequência de rows com um `cur_y` (uma grelha,
/// uma tabela de altura fixa): eles perguntam isto em vez de `is_collapsed` e continuam a pintar
/// enquanto a dobra corre.
#[must_use]
pub fn has_body(store: &WidgetStore, id: NodeId) -> bool {
    store.section_open_live(id) > SHUT
}

#[cfg(test)]
#[path = "body_tests.rs"]
mod tests;

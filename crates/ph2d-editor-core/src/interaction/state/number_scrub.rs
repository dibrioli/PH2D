//! **A LEI DO SCRUB — a porta ÚNICA de *"em que intervalo esta caixa vive, e a que velocidade se
//! arrasta?"*.**
//!
//! ## Por que ela existe, e o número que a justifica
//!
//! O arrasto de uma `NumberInput` responde a duas perguntas que **têm de ter a mesma resposta**:
//! *quanto o valor anda por pixel* (a TAXA) e *onde o valor para* (o CLAMP). No
//! `dispatch::pointer_move` elas eram calculadas em dois blocos, e cada bloco tinha a sua própria
//! lista de fontes — **a do clamp com quatro arms, a da taxa com dois**:
//!
//! | fonte do intervalo | o clamp sabia? | a taxa sabia? |
//! |---|---|---|
//! | [`WidgetStore::number_drag_rate`] (⇒ sem limites) | sim | sim |
//! | [`WidgetStore::set_number_range`] | sim | sim |
//! | a projeção afim do **slider ligado** (`[offset, scale+offset]`) | sim | **NÃO** |
//! | o `(0,1)` de um **chip de canal** do picker | sim | **NÃO** |
//!
//! Nas duas linhas de baixo a caixa era **clampada num intervalo conhecido** e **arrastada pelo
//! atalho histórico** (`DRAG_RATE_X · step`), que não sabe nada sobre esse intervalo. É *um facto
//! com duas cópias que discordam*, e o preço não é teórico — medido pela sonda
//! `scrub_range_census::census_of_how_many_pixels_cross_a_whole_field`, em 2026-08-12:
//!
//! - **295** campos numéricos têm intervalo conhecido; **43 atravessam o campo INTEIRO em menos de
//!   20 px**, e o pior em **0,01 px** — um único pixel de arrasto satura o campo cem vezes;
//! - todos os campos servidos pelo `number_range` cruzavam em **250,00 px** — o alvo de desenho
//!   ([`crate::interaction::drag::DRAG_RANGE_PX_H`]). *A lei funcionava; só era consultada da lista
//!   curta.*
//! - e um campo cruzava em **510 px** (`bgremoval`, `[1, 256]`) — lento pela mesma causa.
//!
//! ⚠️ **Nenhum número é inventado aqui, e essa é a diferença que decide a wave.** A sonda anterior
//! (`census_of_number_fields_that_know_their_own_range`) fechou proibindo *"inventar um tecto por
//! campo"*, porque um cap sem medição é o que a §0 do `CLAUDE.md` proíbe. Esta wave não inventa
//! fronteira nenhuma: ela faz a taxa **ler a fronteira que o clamp ao lado dela já lia**.
//!
//! ⚠️ **E o `step` do atalho não é uma propriedade do campo.** O Down escolhe-o do BUFFER
//! (`if buffer.contains('.') { 0.01 } else { 1.0 }`), então antes desta porta o mesmo campo
//! arrastava **100× mais depressa** no dia em que o valor calhasse de renderizar sem casa decimal.
//! Quem tem intervalo deixa de o consultar; quem não tem continua no atalho, verbatim.
//!
//! ## O que NÃO muda (a metade que torna a wave segura de landar)
//!
//! - campo com `number_range` — **byte-idêntico** (mesmo arm, mesma aritmética);
//! - campo com `number_drag_rate` — **byte-idêntico** (continua a vencer tudo, e continua sem clamp);
//! - campo **sem intervalo nenhum** (posição em px, contagens sem tecto) — **byte-idêntico**, no
//!   atalho histórico. São 146, e são a pergunta seguinte, não esta.

use super::WidgetStore;
use crate::interaction::drag::{self, ScrubLaw};
use ph2d_a11y::NodeId;

impl WidgetStore {
    /// **Em que intervalo esta caixa vive?** — `None` quando ela não tem limites.
    ///
    /// A ordem dos arms é a lei, e é a ordem que o clamp do `pointer_move` sempre teve: uma taxa
    /// registada **declara a caixa ilimitada** e vence tudo (é o que as caixas de transform do
    /// Vector precisam — uma coordenada de mundo não tem fim), depois a faixa explícita, depois a
    /// projeção do slider ligado, depois o canal do picker.
    ///
    /// ⚠️ Um intervalo **degenerado** (`lo == hi`) não recebe guarda, e é por medição e não por
    /// esquecimento: com o valor preso num ponto, qualquer taxa produz o mesmo resultado — a taxa
    /// dele é **inobservável**, e uma guarda seria um ramo que nenhum teste consegue distinguir.
    #[must_use]
    pub fn number_scrub_interval(&self, id: NodeId) -> Option<(f64, f64)> {
        if self.number_drag_rate(id).is_some() {
            // Taxa registada = scrub ILIMITADO calibrado (os chips de transform do Vector). Vence
            // qualquer faixa ou mapeamento de slider.
            return None;
        }
        if let Some((min, max, _)) = self.number_range(id) {
            return Some((min.min(max), min.max(max)));
        }
        if self.linked_slider(id).is_some() {
            // `display = storage·scale + offset` sobre um slider `0..1` ⇒ o intervalo visível é
            // `[offset, scale+offset]` (ou o inverso com `scale` negativo). Sem mapeamento
            // registado o `linked_slider_mapping` devolve a identidade, o que dá `[0, 1]` — que é
            // o que uma ligação não-mapeada de facto guarda.
            let (scale, offset) = self.linked_slider_mapping(id);
            let (a, b) = (f64::from(offset), f64::from(scale + offset));
            return Some((a.min(b), a.max(b)));
        }
        if self.blender_channel_chip(id).is_some() {
            return Some((0.0, 1.0));
        }
        None
    }

    /// **A taxa E os limites do arrasto**, da mesma travessia — ver [`ScrubLaw`].
    ///
    /// O `step` é o do gesto (escolhido no Down a partir do buffer) e só é consultado no ramo do
    /// atalho histórico, isto é **só por quem não tem intervalo nenhum**.
    #[must_use]
    pub fn number_scrub_law(&self, id: NodeId, step: f64) -> ScrubLaw {
        let bounds = self.number_scrub_interval(id);
        if let Some(rate) = self.number_drag_rate(id) {
            // Calibrada em unidades de valor por pixel; o vertical é 10× mais fino. `bounds` já é
            // `None` por este mesmo motivo (ver `number_scrub_interval`).
            return ScrubLaw {
                rate_x: rate,
                rate_y: rate / 10.0,
                bounds,
            };
        }
        match bounds {
            // Proporcional ao INTERVALO: um arrasto fixo atravessa `[lo, hi]` inteiro, seja qual for
            // a magnitude do valor (Enio 2026-06-25) — e agora seja qual for a fonte do intervalo.
            Some((lo, hi)) => {
                let r = hi - lo;
                ScrubLaw {
                    rate_x: r / drag::DRAG_RANGE_PX_H,
                    rate_y: r / drag::DRAG_RANGE_PX_V,
                    bounds,
                }
            }
            // O atalho histórico, para a caixa que de facto não tem fim.
            None => ScrubLaw {
                rate_x: drag::DRAG_RATE_X * step,
                rate_y: drag::DRAG_RATE_Y * step,
                bounds,
            },
        }
    }
}

#[cfg(test)]
#[path = "number_scrub_tests.rs"]
mod tests;

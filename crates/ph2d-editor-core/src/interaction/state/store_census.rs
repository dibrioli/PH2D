//! **O CENSO do store** — as perguntas que o substrato lhe faz, varrendo `states`.
//!
//! ⚠️ **Corte de ASSUNTO, não de tamanho:** o pai responde *como este store é construído, que
//! ligações ele guarda e quem está quente*; aqui vive a outra pergunta — *dado tudo o que está
//! registado, o que é que se MOVE, e o que é que sabe a própria faixa?* As duas censuras já
//! declaravam, cada uma no próprio doc, que moram no store **porque quem sabe o que está
//! registado é ele**, e uma segunda lista noutro sítio envelheceria no dia em que um tipo novo
//! nascesse. Esse argumento é a linha do corte.
//!
//! ⚠️ **E o corte expôs um doc ÓRFÃO:** os dois doc-comments estavam FUNDIDOS num só, colados por
//! cima do `number_fields` — o do `hover_targets` nunca chegou à função que descreve, e ela ficou
//! sem nenhum. É a mesma cicatriz que o `Button::hover_t` carrega escrita no próprio doc; ela é
//! invisível ao compilador e ao rustdoc não avisa, porque um doc a mais numa função é legal.

use super::*;

impl WidgetStore {
    /// **Que campos numéricos existem, e qual deles sabe a própria faixa?**
    ///
    /// O `Some(..)` é a faixa registada por [`Self::set_number_range`] — a que torna o scrub
    /// **proporcional ao intervalo do campo**. O `None` é o campo que caiu no atalho histórico
    /// (`DRAG_RATE_X`, 50 unidades de passo por pixel), e é a pergunta que esta função existe para
    /// responder: *quantos campos ainda arrastam pela constante?*
    ///
    /// ⚠️ Mora aqui pela razão do [`Self::hover_targets`]: quem sabe o que está registado é o
    /// store, e uma segunda lista noutro sítio envelheceria no dia em que um campo novo nascesse.
    pub fn number_fields(&self) -> impl Iterator<Item = (NodeId, Option<(f64, f64, f64)>)> + '_ {
        self.states.iter().filter_map(|(id, st)| {
            if matches!(st, InteractiveState::NumberInput { .. }) {
                Some((*id, self.number_range(*id)))
            } else {
                None
            }
        })
    }

    /// **Quanto de "aceso" cada widget quer estar**, `0..1` — a derivação do estado SEMÂNTICO para
    /// o eixo contínuo do hover.
    ///
    /// ⚠️ **Ela mora aqui, e não no substrato de movimento, de propósito.** Quem sabe *em que
    /// estado um widget está* é este store; se o `UiMotion` respondesse a esta pergunta ele teria
    /// de aprender o vocabulário de todos os tipos de widget, e passaria a haver **duas** tabelas
    /// a dizer o que conta como aceso — a segunda a envelhecer no dia em que um tipo novo nascer.
    ///
    /// `Focused` conta: um botão focado por teclado deve acender como um sob o rato, senão a
    /// navegação por Tab fica invisível para quem depende dela.
    ///
    /// ⚠️ **Este doc esteve ÓRFÃO** — fundido por cima do [`Self::number_fields`], que já tinha o
    /// seu, deixando esta função sem nenhum. Reposto no corte que criou este ficheiro.
    pub fn hover_targets(&self) -> impl Iterator<Item = (NodeId, f32)> + '_ {
        self.states.iter().filter_map(|(id, st)| {
            let lit = match st {
                InteractiveState::Button { state } | InteractiveState::Radio { state, .. } => {
                    matches!(
                        state,
                        ButtonState::Hovered | ButtonState::Pressed | ButtonState::Focused
                    )
                }
                InteractiveState::Toggle { state, .. } => {
                    matches!(state, ToggleState::Hovered | ToggleState::Pressed)
                }
                InteractiveState::Checkbox { state, .. } => {
                    matches!(state, CheckboxState::Hovered | CheckboxState::Pressed)
                }
                // ⚠️ **Sem este braço a tag COMPILA, PINTA e nunca se MOVE.** O campo, a lei do
                // anel e a pele podem estar todos certos e o `t` fica no neutro para sempre —
                // porque o alvo nunca é publicado. É o degrau que o doc abaixo nomeia (*"um tipo
                // que não aparece aqui não ganha entrada nenhuma"*) lido ao contrário.
                InteractiveState::Tag { state } => {
                    matches!(state, TagState::Hovered | TagState::Pressed)
                }
                // ⚠️ **O `Dragging` conta, e é o que separa um slider de um botão.** Um botão é
                // premido e solto; uma trilha é AGARRADA e o dedo fica lá — se o arrasto não
                // acendesse, a superfície apagaria debaixo da mão que a comanda.
                InteractiveState::Slider { state, .. } => matches!(
                    state,
                    crate::widget::SliderState::Hovered
                        | crate::widget::SliderState::Dragging
                        | crate::widget::SliderState::Focused
                ),
                // Os restantes tipos não têm eixo de hover HOJE. ⚠️ Um tipo que não aparece aqui
                // não ganha entrada nenhuma — é o que mantém o mapa do tamanho do que se move.
                //
                // ⚠️ **E "os restantes" inclui quem NÃO É REGISTADO, que é outra classe.** O
                // polegar de uma scrollbar e um botão de modal vivem só no `hit_index`, então o
                // ponteiro (`hot_id`/`active_id`) é a única coisa que sabe deles — e emitir os ids
                // do ponteiro aqui NÃO bastaria: quando o cursor sai, o id **desaparece da lista**,
                // o tique deixa de o conduzir, e o `hover_live` congela no último valor publicado
                // (medido: `button_visual` de um id não-registado devolve `(Hovered, 1.0)`), o que
                // faz RE-ENTRAR não animar. Conduzi-los exige uma lista de quem está a APAGAR-SE,
                // que é estado novo — wave própria, não um braço a mais neste `match`.
                _ => return None,
            };
            Some((*id, if lit { 1.0 } else { 0.0 }))
        })
    }
}

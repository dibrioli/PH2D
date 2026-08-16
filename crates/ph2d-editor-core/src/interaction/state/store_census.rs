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
                // ⚠️ **A FAMÍLIA DO TEXTO E DO CHIP — os dois extremos da corrente existiam e só
                // o elo do MEIO faltava.** O `hover.rs` promove os três (entrada e saída), os
                // pintores já misturam `Border → BorderEmph` pelo eixo, os painéis já passam
                // `(estado, hover_live(id))` e o `dropdown_visual` já devolve o par — e o alvo
                // nunca era publicado, então o `hover_live` devolvia [`crate::motion::SETTLED`]
                // para sempre e cada uma dessas chamadas entregava o NEUTRO. Quatro famílias de
                // widget (campo, área de texto, chip numérico, dropdown) reagiam e SALTAVAM.
                //
                // ⚠️ **Aceso é `Hovered` e SÓ ele, e a lei é DERIVADA, não escolhida:** é
                // exactamente o extremo quente que o `soft` de cada pintor já declara
                // (`text_input::border_color`, `dropdown::chip_border_color`). `Focused` ali é
                // estado DURO — a borda vira `Accent` e nasce um caret —, e contá-lo faria o
                // relógio conduzir um id cujo `t` o pintor ignora enquanto o foco dura.
                InteractiveState::TextInput { state, .. }
                | InteractiveState::NumberInput { state, .. } => {
                    matches!(state, TextInputState::Hovered)
                }
                InteractiveState::Dropdown { state, .. } => {
                    matches!(state, DropdownState::Hovered)
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
                // ⚠️ **E "os restantes" inclui `Plain`, que é outra classe: nenhuma OPINIÃO.** Um
                // polegar de scrollbar cai aqui, e derivar o *aceso* dele deste `match` é
                // impossível — quem sabe se um polegar está quente é o PONTEIRO. Ele é conduzido
                // pelo [`Self::scrollbar_hover_targets`], que existe precisamente por isso.
                //
                // ⚠️ **E este comentário JÁ MENTIU duas vezes, o que é a razão de estar reescrito
                // com o número ao lado.** Ele dizia *"vivem só no `hit_index`"* — medido em
                // 2026-08-15, **3 dos 22** polegares estão registados (`Plain`) e 19 não; e dizia
                // que conduzi-los *"exige uma lista de quem está a APAGAR-SE, que é estado novo"* —
                // **não é**: o `hover_live` já é o registo de todo id a que o tique publicou um
                // `t`, e foi essa medição que fez a wave caber num censo irmão em vez de num campo.
                //
                // ⛔ E **não** puxe os outros `Plain` para cá: as rows da Hierarquia também o são,
                // e amaciá-las revive a cerca do estudo §6.2 (*o realce de uma lista OBEDECE ao
                // cursor; oito rows meio-acesas ao mesmo tempo é rasto, não vida*).
                _ => return None,
            };
            Some((*id, if lit { 1.0 } else { 0.0 }))
        })
    }

    /// **O POLEGAR de uma scrollbar** — o eixo dele estava inteiro e nunca se moveu.
    ///
    /// # O defeito, medido antes de uma linha
    ///
    /// A `scrollbar::thumb_color` chama [`crate::motion::hover_axis`] há muito, o par
    /// `(estado, t)` está na **ASSINATURA** do `paint_scrollbar` (o compilador é o gate) e os 21
    /// painéis passam-no pelo [`Self::scrollbar_visual`]. E o `t` valia
    /// [`crate::motion::SETTLED`] nos **quatro** instantes — nunca tocado · sob o ponteiro ·
    /// assente · e cem quadros depois de sair: *as 22 barras do app reagiam e SALTAVAM*.
    ///
    /// A causa é a mesma da família do texto e do chip, por uma via estrutural diferente: o
    /// [`Self::hover_targets`] deriva o *aceso* do estado GUARDADO, e um polegar guarda
    /// `InteractiveState::Plain` — *nenhuma opinião* —, então ele cai no braço `_` e nunca é
    /// publicado. Quem sabe se um polegar está quente é o PONTEIRO.
    ///
    /// # Porque a régua é o mapa do DESPACHANTE, e não «é `Plain`» nem «não está registado»
    ///
    /// ⛔ **`Plain` é largo demais, e foi medido:** as rows da Hierarquia são registadas `Plain`,
    /// e amaciá-las revive exactamente a cerca que o estudo plantou (§6.2) — *descer oito rows em
    /// 200 ms deixa-as todas meio-acesas ao mesmo tempo; o realce de uma lista OBEDECE ao cursor*.
    ///
    /// ⛔ **«não está registado» também não serve:** medido, **só 3 dos 22** polegares estão no
    /// store; os outros **19** vivem só no `hit_index`. Um `InteractiveState::Scrollbar` custaria
    /// **19 registos** espalhados por 19 painéis — a enumeração que apodrece, e o 23º nasceria sem
    /// ela — e poria o hover em DOIS sítios (o estado guardado contra o `hot_id`).
    ///
    /// ✅ **`scrollbar_panel_for_id` é uma propriedade que um polegar já TEM de satisfazer:** sem
    /// o braço dele o arrasto da barra não funciona, e o
    /// `shells/desktop/tests/scrollable_panels_intercept_the_wheel.rs` nomeia-o como uma das
    /// quatro edições obrigatórias. *Uma barra nova nasce coberta porque já tinha de nascer ali.*
    ///
    /// ⚠️ **O polegar do popover de um dropdown fica de FORA, e é nomeado:** ele é chaveado pelo
    /// CHIP, não por um painel (o motivo de o [`Self::scrollbar_visual_for`] existir), então não
    /// está no mapa; e ele nasce e morre com o popover, logo a track dele seria criada e podada
    /// sem parar.
    ///
    /// # As duas metades do ciclo
    ///
    /// ⚠️ **Emitir só o id QUENTE não basta:** quando o cursor sai, o id **desaparece** da lista,
    /// o tique deixa de o conduzir, a track assenta no último alvo — e o polegar acenderia para
    /// **nunca mais apagar**. A metade que falta é a lista de quem ARREFECE, e ⚠️ **ela não é
    /// estado novo:** o `hover_live` já é o registo de todo id a que o tique publicou um `t`, e um
    /// polegar com `t > 0` é, por construção, um que ainda tem de descer.
    ///
    /// ⚠️ **Aceso é `Hovered` ou `Dragging`, e a lei é DERIVADA — a pergunta é feita à MESMA
    /// porta que o pintor faz** ([`Self::scrollbar_visual`]). Contar só o `Hovered` faria o
    /// polegar arrefecer **debaixo do dedo que o arrasta**, e ao soltar ele teria de reacender.
    ///
    /// ⚠️ **E quem CHEGA a zero sai da lista**, o que mantém verdadeira a alegação de custo do
    /// substrato (*lembrar é O(widgets tocados recentemente)*): a track fica ociosa e é podada, e
    /// o `hover_live` guarda o `0.0` — que é precisamente o que faz o pintor continuar no token de
    /// repouso. **Apagar a entrada** devolveria [`crate::motion::SETTLED`] e deixaria o polegar
    /// **quente para sempre**.
    pub fn scrollbar_hover_targets(&self) -> impl Iterator<Item = (NodeId, f32)> + '_ {
        use crate::interaction::dispatch::scroll::scrollbar_panel_for_id;
        use crate::widget::ScrollbarState as S;
        let hot = self
            .hot_id()
            .filter(|id| scrollbar_panel_for_id(*id).is_some());
        // ⚠️ **`!= 0.0`, e NÃO `> 0.0`** — a mola do carácter Expressivo ULTRAPASSA: medido, o
        //    voo de saída passa por **−0,0109** antes de voltar. Com `> 0.0` o tique largava-o ali,
        //    a track era podada a meio do caminho e o `hover_live` guardava um negativo para
        //    sempre. O pintor clampa, então nada se vê — e um valor publicado que ninguém pode
        //    explicar é como o próximo leitor herda um bug.
        //
        // ⚠️ E a comparação exacta é SÓ segura porque a chegada escreve o literal: o `arrive` da
        //    mola larga o voo e põe `to` (que é `0.0`) no valor, em vez de convergir para perto.
        //
        // ⚠️ `t` PRIMEIRO: quase toda entrada do mapa está fria, e assim o predicado caro (o mapa
        //    do despachante) quase nunca corre.
        let cooling = self.hover_live.iter().filter_map(move |(id, t)| {
            (*t != 0.0 && Some(*id) != hot && scrollbar_panel_for_id(*id).is_some()).then_some(*id)
        });
        hot.into_iter().chain(cooling).map(|id| {
            let lit = matches!(self.scrollbar_visual(id).0, S::Hovered | S::Dragging);
            (id, if lit { 1.0 } else { 0.0 })
        })
    }
}

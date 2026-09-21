//! **O que o gate SEMEIA no store** — a família dos métodos nomeados do [`MockPanelHost`].
//!
//! # Porque este ficheiro existe
//!
//! Irmão do [`super`] por CAP de LOC: o `lib.rs` passou dos `700` no dia em que a secção CAMERA
//! precisou de abrir uma sub-secção recolhida. O corte é por RESPONSABILIDADE — aqui mora tudo o
//! que responde a *«e se o artista fizer X antes de eu medir?»*, e lá fica o arnês (montar, pintar,
//! despachar, drenar).
//!
//! # ⚠️ A lei que estes métodos existem para honrar
//!
//! ⛔ **Não há `store_mut()`, e a ausência é a decisão.** Um acessor genérico seria a porta pela
//! qual um gate escreve qualquer coisa no store e depois «prova» o que ele mesmo semeou. Cada
//! método aqui responde a **UMA** pergunta do artista — rolar, abrir, escrever, marcar —, e é isso
//! que separa *onde um botão está* de *se o artista chega lá*.

use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::dispatch::keymap::KEY_ENTER;
use ph2d_editor_core::interaction::{InteractiveState, WidgetEvent};
use ph2d_editor_core::interaction::{dispatch_key, dispatch_text_input};
use ph2d_editor_core::project::ProjectSettings;

use super::MockPanelHost;

impl MockPanelHost {
    /// Rola um painel — o que a roda do mouse escreve no store antes da pintura seguinte.
    ///
    /// ⚠️ Um método NOMEADO em vez de um `store_mut()` genérico: a segunda forma seria uma porta
    /// aberta para um gate escrever qualquer coisa no store e depois "provar" o que ele mesmo
    /// semeou. Esta responde a uma pergunta só — *e se o artista rolar?* —, que é o que separa
    /// *onde um botão está* de *se o artista chega lá*.
    pub fn set_panel_scroll(&mut self, panel: NodeId, y: f32) {
        self.store.set_panel_scroll(panel, y);
    }

    /// **A cena tem N objectos** — a ordem de linhas que o host entrega à Hierarquia a cada quadro.
    ///
    /// ⚠️ **Sem isto, um gate de LISTA é estruturalmente impossível neste harness:** o `populate`
    /// da Hierarquia semeia **uma** linha (`HIER_PLAYER`), então tudo o que a pintura faz *entre*
    /// duas linhas — o passo, o vão, a linha de parentesco, o alvo de clique que invade a vizinha —
    /// nunca acontece. Uma cena de um objecto só é o único caso em que **toda** lei de lista é
    /// verdadeira por vacuidade.
    ///
    /// ⚠️ Método NOMEADO, nunca um `store_mut()`: responde a UMA pergunta — *e se a cena tiver mais
    /// que um objecto?* — em vez de abrir o store para um gate semear o que depois vai "provar"
    /// (o mesmo argumento do [`Self::set_panel_scroll`] e do [`Self::settle_section_folds`]).
    ///
    /// ⚠️ Os ids têm de ser os da **fixtura** (`ids::HIER_*`): sem entradas vivas o pintor procura
    /// cada linha em `fixture::hierarchy()`, e um id que ela não conhece não é pintado.
    pub fn set_hierarchy_rows(&mut self, ids: &[NodeId]) {
        for id in ids {
            self.store
                .register_if_absent(*id, ph2d_editor_core::interaction::InteractiveState::Plain);
        }
        self.store.set_hierarchy_order(ids.to_vec());
    }

    /// **O relógio de movimento correu até ao fim** — toda dobra de secção salta para o seu
    /// alvo semântico (`is_collapsed` ⇒ 0, senão 1).
    ///
    /// ⚠️ Existe porque a F4b fez o CORPO de uma secção interpolar: depois de um clique no
    /// cabeçalho o flag semântico já virou, mas o `t` ainda desce, e o painel de um harness
    /// headless — que não tem o tique do `HeroScreen` — nunca o veria chegar a zero. Sem esta
    /// porta um gate de dobra afirmaria *"a row sumiu"* sobre um produto que a está a esconder
    /// **gradualmente**, e reprovaria a animação em vez de a medir.
    ///
    /// ⚠️ Método NOMEADO, nunca um `store_mut()`: ele responde a UMA pergunta — *e se o artista
    /// esperar a animação acabar?* — em vez de abrir o store para um gate semear o que depois
    /// vai "provar" (o mesmo argumento do [`Self::set_panel_scroll`]).
    /// **Abre ou fecha uma secção colapsável** — o que um clique no cabeçalho escreve.
    ///
    /// ⚠️ **Sem isto, uma sub-secção que NASCE RECOLHIDA é intestável:** a `Cull Mask` da câmera e a
    /// `Visibility Layer` da sprite são as duas assim (32 caixas numa pergunta avançada), e um gate
    /// que medisse com elas fechadas leria *«a grade está morta»* sobre um painel correcto.
    ///
    /// ⚠️ Método NOMEADO, nunca um `store_mut()`: responde a UMA pergunta — *e se o artista abrir?*
    /// — em vez de abrir o store para um gate semear o que depois vai "provar" (o mesmo argumento
    /// do [`Self::set_panel_scroll`] e do [`Self::settle_section_folds`]).
    ///
    /// ⚠️ **Ele ASSENTA a dobra a seguir**: a secção anima, e um gate que pintasse no quadro
    /// seguinte mediria a animação a meio em vez do estado que pediu.
    pub fn set_collapsed(&mut self, id: NodeId, collapsed: bool) {
        self.store.set_collapsed(id, collapsed);
        self.settle_section_folds();
    }

    pub fn settle_section_folds(&mut self) {
        for id in self.store.collapsible_ids() {
            let target = if self.store.is_collapsed(id) {
                0.0
            } else {
                1.0
            };
            self.store.set_section_open_live(id, target);
        }
    }

    /// ⭐⭐⭐ **Abre TODA secção e assenta a dobra** — para um gate que mede o CONTEÚDO de uma.
    ///
    /// ⛔⛔ **Desde 2026-09-21 o Inspector ABRE DOBRADO** (toda secção com chevron nasce recolhida
    /// menos a Transform — `pre_populate::marca_as_gavetas`), porque com tudo aberto ele desenhava
    /// `2,5` a `6,5` ecrãs. ⇒ *um gate que pinta pela porta do cromo partilhado e procura um
    /// controlo DENTRO de uma secção tem de a abrir primeiro*, senão ele mede a política de dobra
    /// e diz «o id não foi pintado».
    ///
    /// ⚠️ **Chamá-la é uma DECLARAÇÃO**: este gate é sobre o conteúdo da secção, não sobre como o
    /// painel abre. Quem mede a abertura é o `o_inspector_abre_dentro_da_dobra`.
    pub fn open_all_sections(&mut self) {
        for id in self.store.collapsible_ids() {
            self.store.set_collapsed(id, false);
        }
        self.settle_section_folds();
    }

    /// Set a registered slider's stored value — what a pointer drag writes
    /// into the store *before* the dispatch emits `ValueChanged(id)`. Panics
    /// (never silently no-ops) if `id` is absent or not a slider.
    ///
    /// ⚠️ **Esta doc estava ORFÃ**, pousada sobre o `store()` — o acessor de leitura — desde antes
    /// desta wave. *Um doc-comment separado do item que descreve muda de dono em silêncio, e
    /// passa a mentir sobre os dois.*
    pub fn set_slider_value(&mut self, id: NodeId, value: f32) {
        match self.store.get_mut(id) {
            Some(InteractiveState::Slider { value: v, .. }) => *v = value,
            Some(_) => panic!("set_slider_value: {id:?} is registered but is not a Slider"),
            None => panic!("set_slider_value: {id:?} is not registered (did populate run?)"),
        }
    }

    /// ⭐⭐⭐ **O ARTISTA ESCOLHEU UMA COR no selector que está aberto** — o espelho que o
    /// `hero::paint` corre, encenado.
    ///
    /// # ⚠️ O que ele encena, exactamente
    ///
    /// O selector de cor da casa é **um** e flutua sobre o canvas. Quem liga a roda dele ao widget
    /// que a abriu é uma linha do `screens/hero/paint.rs`, corrida **antes** de os painéis
    /// pintarem: *o valor vivo do selector é espelhado para `widget_color(picker_target)`*. Um
    /// painel que edita cor lê **daí** — e essa leitura é a metade da costura que nenhum outro
    /// método deste arnês consegue exercitar, porque o `MockPanelHost` não tem hero.
    ///
    /// ⛔ **Ele EXIGE um selector aberto, e entra em pânico sem ele** — nunca um no-op silencioso.
    /// É isso que o impede de ser o `store_mut()` que este ficheiro recusa por escrito: não se pode
    /// pintar uma cor num widget que ninguém abriu, e um gate que o fizesse estaria a provar o que
    /// ele próprio semeou.
    ///
    /// ⚠️ **O alvo NÃO é um argumento**: ele é o que o `Down` real deixou em
    /// [`WidgetStore::picker_target`]. Passá-lo à mão deixaria o gate verde sobre uma amostra que o
    /// clique nunca alcança — que é precisamente a família de defeitos deste arnês.
    pub fn pick_colour_in_the_open_picker(&mut self, rgba: [u8; 4]) {
        let alvo = self.store.picker_target().expect(
            "pick_colour_in_the_open_picker: nenhum selector está aberto — clique na amostra \
             primeiro (é o `Down` que escolhe o alvo)",
        );
        self.store.set_widget_color(alvo, rgba);
    }

    /// Set a registered number chip's committed value. Panics if `id` is
    /// absent or not a `NumberInput`.
    pub fn set_number_value(&mut self, id: NodeId, value: f64) {
        match self.store.get_mut(id) {
            Some(InteractiveState::NumberInput { value: v, .. }) => *v = value,
            Some(_) => panic!("set_number_value: {id:?} is registered but is not a NumberInput"),
            None => panic!("set_number_value: {id:?} is not registered (did populate run?)"),
        }
    }

    /// **DIGITAR de verdade num chip numérico**, pelos dispatchers REAIS: foco, um
    /// `dispatch_text_input` por caractere, e Enter. Devolve os `WidgetEvent` que o commit
    /// emitiu (tipicamente `ValueChanged(id)`), para o chamador entregá-los ao painel.
    ///
    /// ⚠️ **Por que o testkit precisava disto, e é o achado que o motivou:** [`set_number_value`]
    /// ESCREVE o valor no store e pula o commit inteiro — `apply_chip_value_with_mirror`, que é
    /// onde o espelho chip↔slider decide o que sobrevive. Toda a família de gates do range do
    /// `motion-params` usava o setter, então nenhum deles jamais exercitou a camada onde o valor
    /// digitado morria, e os três ficaram VERDES enquanto o produto capava a caixa no máximo do
    /// SLIDER (Enio, smoke de 2026-08-07: *"Máximo de 20 em grid"*).
    ///
    /// ⚠️ **PINTE o painel antes de chamar isto.** A faixa do chip (`set_number_range`) e o link
    /// com o slider nascem no `paint`; sem ele o chip não tem régua nem espelho, e o teste passa
    /// a medir uma fixture que o produto não tem.
    ///
    /// Panics se `id` não estiver registrado ou não for um `NumberInput`.
    pub fn type_into_number(&mut self, id: NodeId, text: &str) -> Vec<WidgetEvent> {
        match self.store.get_mut(id) {
            Some(InteractiveState::NumberInput { buffer, caret, .. }) => {
                buffer.clear();
                *caret = 0;
            }
            Some(_) => panic!("type_into_number: {id:?} is registered but is not a NumberInput"),
            None => panic!("type_into_number: {id:?} is not registered (did populate run?)"),
        }
        self.store.set_focus(Some(id));
        let arena = Bump::new();
        for ch in text.chars() {
            let _ = dispatch_text_input(&mut self.store, ch, &arena);
        }
        let key = ph2d_host::KeyEvent {
            keycode: KEY_ENTER,
            modifiers: ph2d_host::Modifiers {
                shift: false,
                ctrl: false,
                alt: false,
                meta: false,
            },
            kind: ph2d_host::KeyKind::Down,
            timestamp_ns: 0,
        };
        dispatch_key(&mut self.store, key, &arena).to_vec()
    }

    /// Set a registered text input's buffer — what a real keystroke would have
    /// left there before dispatch emits `TextChanged(id)`. Panics if `id` is
    /// absent or not a `TextInput`.
    ///
    /// **Por que o testkit precisa disto:** um arm de `TextChanged` LÊ o buffer
    /// (`host.store().text(id)`) em vez de receber a string no evento, então um
    /// seam que só despacha o evento testaria sempre a string vazia — e um arm
    /// que mandasse o texto errado passaria. O caret vai para o fim, como depois
    /// de digitar.
    pub fn set_text(&mut self, id: NodeId, value: &str) {
        match self.store.get_mut(id) {
            Some(InteractiveState::TextInput { text, caret, .. }) => {
                text.clear();
                text.push_str(value);
                *caret = text.len();
            }
            Some(_) => panic!("set_text: {id:?} is registered but is not a TextInput"),
            None => panic!("set_text: {id:?} is not registered (did populate run?)"),
        }
    }

    /// Set a registered toggle's stored on-state — what the paint pass mirrors
    /// from the snapshot before dispatch emits `Toggled(id)`. Panics if `id` is
    /// absent or not a `Toggle`.
    pub fn set_toggle_on(&mut self, id: NodeId, on: bool) {
        match self.store.get_mut(id) {
            Some(InteractiveState::Toggle { on: o, .. }) => *o = on,
            Some(_) => panic!("set_toggle_on: {id:?} is registered but is not a Toggle"),
            None => panic!("set_toggle_on: {id:?} is not registered (did populate run?)"),
        }
    }

    /// **A cor que o SELECTOR devolveu**, na tabela lateral `widget_colors` — o que um arrasto no
    /// selector de cor teria deixado lá antes de a semente do painel a ler.
    ///
    /// **Porque é que o testkit precisava disto:** uma amostra de cor não carrega valor nenhum
    /// (é um `Plain`), e quem leva a escolha ao documento é a SEMENTE do painel, que compara a
    /// tabela lateral com o instantâneo. Sem esta porta, um seam consegue provar que o clique
    /// **abre** o selector e **não** consegue provar que a cor escolhida chega ao barramento — e
    /// apagar esse braço deixava toda a suíte verde com a cor a morrer no painel.
    pub fn set_widget_color(&mut self, id: NodeId, rgba: [u8; 4]) {
        self.store.set_widget_color(id, rgba);
    }

    /// Mutable access às definições do projeto — unidade de leitura, `pixels_per_meter`, snaps.
    ///
    /// **Porque é que o testkit precisava disto:** havia `project()` (leitura) e não havia o par.
    /// A conversão px↔m do Inspector lê `host.project().display_unit` e `pixels_per_meter`, por
    /// isso um teste de costura só conseguia exercitá-la no default (`Meters`, onde a conversão é
    /// a identidade) — ou seja, **só na metade em que ela não faz nada**. Os dois testes que
    /// provavam o round-trip em pixels estão desligados desde 2026-06, e esta ausência é a razão
    /// mecânica: não havia porta por onde pôr o projeto em `Pixels`.
    pub fn project_mut(&mut self) -> &mut ProjectSettings {
        &mut self.project
    }

    /// Set a registered **checkbox**'s stored value — the sibling of [`Self::set_toggle_on`] for
    /// the other of the two boolean widgets. Panics if `id` is absent or not a `Checkbox`.
    ///
    /// ⚠️ **Toma o VALOR, não um `bool`, de propósito.** `CheckboxValue` tem três estados, e o
    /// terceiro — `Indeterminate` — é a affordance de *«Mixed»* que uma seleção múltipla com
    /// valores divergentes pinta. Uma porta que só aceitasse `bool` tornaria esse estado
    /// inalcançável a todo teste de costura, que é precisamente como ele passou a existir no
    /// painter sem uma única afirmação a defendê-lo.
    ///
    /// **Porque é que o testkit precisava disto:** havia `set_toggle_on` e não havia o par. Os
    /// checkboxes da sprite (Flip H/V, Centered, Tint Fill, Region…) são `Checkbox`, não `Toggle`
    /// — e por isso a família inteira de `InspectorSpriteEdit` era, na prática, **inalcançável**
    /// por um teste de costura. Vinte e uma variantes chegaram a 2026-08 com zero afirmações
    /// vivas, e esta ausência é metade da razão.
    pub fn set_checkbox_value(
        &mut self,
        id: NodeId,
        value: ph2d_editor_core::widget::CheckboxValue,
    ) {
        match self.store.get_mut(id) {
            Some(InteractiveState::Checkbox { value: v, .. }) => *v = value,
            Some(_) => panic!("set_checkbox_value: {id:?} is registered but is not a Checkbox"),
            None => panic!("set_checkbox_value: {id:?} is not registered (did populate run?)"),
        }
    }

    /// Set a registered dropdown's open state — what the generic dispatcher writes when
    /// the user clicks a chip. Panics if `id` is absent or not a `Dropdown`.
    ///
    /// **Why the testkit needs this at all:** the open/close of a dropdown is done by the
    /// SHELL's generic dispatch, not by the panel's `apply_event` — so a seam test driving
    /// `apply_event` alone cannot reach the state *"this popover is open"*, and any rule
    /// about it (e.g. *opening one closes the other*) would be untestable at this seam.
    pub fn set_dropdown_open(&mut self, id: NodeId, open: bool) {
        match self.store.get_mut(id) {
            Some(InteractiveState::Dropdown { open: o, .. }) => *o = open,
            Some(_) => panic!("set_dropdown_open: {id:?} is registered but is not a Dropdown"),
            None => panic!("set_dropdown_open: {id:?} is not registered (did populate run?)"),
        }
    }

    /// Read a registered dropdown's open state (the mirror of
    /// [`Self::set_dropdown_open`], so a gate can assert on it).
    #[must_use]
    pub fn dropdown_is_open(&self, id: NodeId) -> Option<bool> {
        match self.store.get(id) {
            Some(InteractiveState::Dropdown { open, .. }) => Some(*open),
            _ => None,
        }
    }
}

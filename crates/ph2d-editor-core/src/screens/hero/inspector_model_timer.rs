//! **O modelo da secção TIMERS** (TOP-20 #2, W3) — snapshot e edits.
//!
//! ⚠️ **Irmão de [`super::inspector_model`] por CAP de LOC** — mesmo padrão dos outros seis.
//!
//! # ⚠️ O que este snapshot NÃO tem, e é a razão inteira do desenho
//!
//! Não há `elapsed`, não há `running`, não há `progress`. Esses três vivem no `TimerRuntime`, que
//! **não é um componente registado** — o undo não o fotografa, e por isso um relógio a andar a
//! 60 Hz não faz de cada quadro um passo de `Ctrl+Z`. Trazê-los para aqui reabriria a porta pelo
//! outro lado: um painel que os mostra é um painel que **repinta** 60 vezes por segundo, e o
//! primeiro pedido a seguir seria poder mexer neles.
//!
//! ⇒ o Inspector mostra o que o artista **autora**. O que o motor escreve vê-se na cena.

/// Um timer, como o Inspector o lê. É o [`ph2d_ecs::Timer`] com a duração já em **segundos**.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorTimerRow {
    /// O nome do timer — para o artista o distinguir na lista.
    pub name: String,
    /// A duração em **segundos**, que é a unidade em que ele pensa.
    ///
    /// ⚠️ **A conversão vive nas DUAS pontas e em mais lado nenhum**: aqui, ao construir o
    /// snapshot, e no despacho, ao escrever de volta. O modelo do motor é `u64` de microssegundos
    /// porque o tique é de passo fixo e o replay tem de o reproduzir — *um `f32` no componente
    /// deixaria de ser bit-idêntico entre máquinas*.
    pub duration_s: f32,
    pub repeat: bool,
    pub autostart: bool,
    /// O nome do sinal publicado a cada disparo. **Vazio = calado.**
    pub signal: String,
}

impl InspectorTimerRow {
    /// **Este timer chega a disparar?** Duração `0` nunca dispara (lei do [`ph2d_ecs::timer`]), e
    /// sem `autostart` ninguém o arranca — não há, hoje, gesto que o ponha a correr à mão.
    ///
    /// ⚠️ **Derivado, nunca guardado** — a mesma lei do `fits()` da §11: um estado «válido» ao
    /// lado dos campos envelhece no dia em que um deles muda sem ele.
    #[must_use]
    pub fn will_ever_fire(&self) -> bool {
        self.duration_s > 0.0 && self.autostart
    }

    /// **Ele cumpre o período e cala-se** — a lei da §11 sobre um produtor sem nome.
    #[must_use]
    pub fn is_mute(&self) -> bool {
        self.signal.trim().is_empty()
    }
}

/// Snapshot da secção TIMERS da entidade selecionada.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorTimerInfo {
    pub entity_bits: u64,
    /// Os timers, na ordem em que o componente os guarda. **O índice é a identidade** — é ele que
    /// liga a config ao relógio vivo, e é por isso que uma edição viaja com ele e nunca com o nome.
    pub rows: Vec<InspectorTimerRow>,
    /// Quantas entidades estão selecionadas.
    ///
    /// ⚠️ **A secção NÃO se espalha sobre a seleção**, e este número é o que a faz dizê-lo: um
    /// índice só significa alguma coisa na lista da primária. Sem isto, marcar cinco objectos e
    /// mexer numa duração muda **um** e cala-se — o artista descobre semanas depois (é a lição que
    /// a §11 já escreveu).
    pub selected_count: usize,
}

/// Uma edição de um campo da secção TIMERS.
///
/// ⚠️ **O `u8` é o ÍNDICE na lista, nunca o nome.** O nome é editável e pode repetir-se; o índice
/// é o que liga `Timers[i]` a `TimerRuntime[i]`, e é essa ligação que a shell tem de honrar.
#[derive(Clone, Debug, PartialEq)]
pub enum TimerFieldEdit {
    // ⛔ **Não há `AddComponent`, e é o ADR-0166:** *o Inspector mostra o que o objecto TEM, e um
    // componente anexa-se pela PALETA*. Um segundo caminho de anexação seria a segunda resposta à
    // mesma pergunta — e a shell publica este snapshot só para quem já tem `Timers`, então um
    // botão desses nunca chegaria a ser pintado.
    /// Cria um timer com o próximo nome livre.
    Add,
    /// Retira o timer deste índice. ⚠️ O `TimerRuntime` encolhe com ele no quadro seguinte, pela
    /// reconciliação — nada aqui lhe toca.
    Remove(u8),
    /// `(timer, nome)`. Vazio é **recusado**: uma lista de linhas sem nome não é escolhível.
    Rename(u8, String),
    /// `(timer, duração em SEGUNDOS)` — a shell converte para microssegundos e satura no
    /// `ph2d_ecs::TIMER_MAX_US`.
    DurationSecs(u8, f32),
    Repeat(u8, bool),
    Autostart(u8, bool),
    /// `(timer, nome do sinal)` — vazio cala o timer.
    Signal(u8, String),
}

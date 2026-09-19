//! ⭐⭐⭐ **O QUE CADA BOTÃO DA SECÇÃO DO OSSO SIGNIFICA** — a tradução `id → verbo`, e em que
//! grandeza o efeito de cada um se vê.
//!
//! ⚠️ **Ela mora na FAMÍLIA e não na fase do quadro**, pela mesma lei do [`crate::knobs`]: a shell
//! decide a ORDEM em que as coisas acontecem; *o que* um botão faz é conhecimento de quem possui o
//! componente.
//!
//! # ⛔⛔ Por que ela existe: a pergunta que nenhum instrumento fazia
//!
//! Os censos que já existiam sobre esta secção provam que o clique **chega ao barramento**
//! ([`ph2d_editor_core::ids::VECTOR_BONE_VERBS`] é lida pelo registo do painel **e** pelo
//! encaminhamento). ⛔ Nenhum provava que alguma coisa acontece **a seguir** — e é essa a família de
//! metade dos reports do dono nesta linha: o botão pinta, acende sob o rato, o clique atravessa, e
//! o mundo não se mexe.
//!
//! ⇒ [`censo_dos_verbos_do_osso_tests`] corre **os catorze** sobre um palco e mede a captura do
//! mundo — a MESMA que o undo fotografa. O que torna o censo honesto não é a corrida de hoje: é o
//! `match` **exaustivo** de [`VerboDoOsso`], que faz um verbo novo **não compilar** até alguém dizer
//! como se corre e o que se espera dele. *É a diferença entre uma lista que alguém tem de se lembrar
//! de estender e uma que não fica verde sem a extensão.*

use ph2d_editor_core::NodeId;
use ph2d_editor_core::ids;

/// Que verbo da secção do osso um botão do painel é.
///
/// ⚠️ **A ordem é a da [`ids::VECTOR_BONE_VERBS`] e isso é LOAD-BEARING** — ver [`of_id`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerboDoOsso {
    /// *Mirror Branch* — o lado oposto, construído a partir deste ramo.
    Espelhar,
    /// *Look At* — a âncora com a corrente em UM, e o osso vira-se para ela.
    Apontar,
    /// *Bind* — prende os desenhos e as imagens escolhidos aos ossos.
    Prender,
    /// *Expand* — solta, e a geometria deformada de agora passa a ser o desenho.
    Assar,
    /// *Release* — solta, e a fonte autorada volta.
    Soltar,
    /// *Rest Pose* — o osso em foco e a descendência dele voltam ao repouso.
    ReporRepouso,
    /// *Set Rest Pose* — a pose de agora passa a ser o repouso.
    GuardarRepouso,
    /// *Add IK* — dá ao osso um alvo que a corrente persegue.
    AncoraPor,
    /// *Remove IK* — tira a âncora e devolve a corrente à pose autorada.
    AncoraTirar,
    /// *Add Limit* — até onde esta junta dobra.
    LimitePor,
    /// *Remove Limit* — a junta volta a ser livre.
    LimiteTirar,
    /// *Add Smart Bone* — anexa o controlo VAZIO.
    InteligentePor,
    /// *Remove Smart Bone* — tira o controlo e devolve a pose autorada do que ele conduzia.
    InteligenteTirar,
    /// *Pick Object* — arma o gesto de duas mãos que diz de que objecto este controlo trata.
    InteligenteEscolherAlvo,
}

impl VerboDoOsso {
    /// **Todos os verbos da secção** — a população do censo.
    ///
    /// ⚠️ **Escrita à mão e guardada por um `match` EXAUSTIVO** (o gate `a_lista_todos_cobre_o_enum`,
    /// o molde do [`crate::knobs::BoneKnob::TODOS`]): um `enum` não se enumera sozinho, e uma
    /// variante nova que não venha aqui deixaria o censo a medir uma população mais pequena **em
    /// silêncio** — que é exactamente como um verbo morto passa despercebido.
    pub const TODOS: [Self; 14] = [
        Self::Espelhar,
        Self::Apontar,
        Self::Prender,
        Self::Assar,
        Self::Soltar,
        Self::ReporRepouso,
        Self::GuardarRepouso,
        Self::AncoraPor,
        Self::AncoraTirar,
        Self::LimitePor,
        Self::LimiteTirar,
        Self::InteligentePor,
        Self::InteligenteTirar,
        Self::InteligenteEscolherAlvo,
    ];

    /// ⭐⭐⭐ **O RASTO QUE A SHELL TEM DE MOSTRAR** — o que as fases do quadro escrevem para levar
    /// este verbo do pedido ao efeito.
    ///
    /// ⛔⛔⛔ **Ele existe por uma MUTAÇÃO QUE SOBREVIVEU** (2026-09-19): apagado o corpo do braço do
    /// *Add Smart Bone* na fase do quadro, **`23` testes da shell ficaram verdes**. O censo desta
    /// crate prova que a PORTA faz efeito; a costura do painel prova que o clique chega ao
    /// BARRAMENTO; e *nada no repo juntava as duas pontas* — que é o terceiro elo do `§5.0` (*o
    /// leitor DECIDE, ou entrega a alguém que descarta?*) e a quarta vez que esta rota morre nesta
    /// linha.
    ///
    /// ⚠️⚠️ **É um censo TEXTUAL, e ele mede TEXTO e não uma chamada** — a fase é um método de
    /// `App`, que segura uma surface de janela real, logo nenhum teste a corre. *Ele apanha o braço
    /// que deixou de chamar a porta; não apanha o braço que a chama com o argumento errado* — essa
    /// metade é do censo desta crate, que corre as duas portas e exige que elas **difiram**.
    ///
    /// ⭐ **Quando dois verbos partilham a porta, o rasto tem DUAS peças:** a porta (que não pode
    /// faltar sem os dois caírem) e o **discriminador** (que é o que os separa). Sem a primeira, um
    /// `Keep::Deformed` ficava à vista no dreno com o `release` já apagado da fase.
    #[must_use]
    pub const fn rastos_na_shell(self) -> &'static [&'static str] {
        match self {
            Self::Espelhar => &["espelho::espelha("],
            Self::Apontar => &["goal::add_look_at("],
            Self::Prender => &["skeleton_live::bind("],
            Self::Assar => &["skeleton_live::release(", "Keep::Deformed"],
            Self::Soltar => &["skeleton_live::release(", "Keep::Source"],
            Self::ReporRepouso => &["pose_de_repouso::aplica(", "Verbo::Repor"],
            Self::GuardarRepouso => &["pose_de_repouso::aplica(", "Verbo::Guardar"],
            Self::AncoraPor => &["skeleton_goal::add("],
            Self::AncoraTirar => &["skeleton_goal::remove("],
            Self::LimitePor => &["bone_limit::add_limit("],
            Self::LimiteTirar => &["bone_limit::remove_limit("],
            Self::InteligentePor => &["smart::add("],
            Self::InteligenteTirar => &["skeleton_smart::remove("],
            // ⚠️ O efeito dele é o MODO, e o rasto é a escrita no estado da shell — ver
            // [`Consumidor::Modo`]. *Um verbo sem porta não é um verbo sem rasto.*
            Self::InteligenteEscolherAlvo => &["smart_pick = Some("],
        }
    }

    /// ⭐⭐ **EM QUE GRANDEZA o efeito deste verbo se vê.** Ver [`Consumidor`].
    #[must_use]
    pub const fn consumidor(self) -> Consumidor {
        match self {
            // ⛔ **A ÚNICA excepção, e ela é NOMEADA e não uma folga.** Ver [`Consumidor::Modo`].
            Self::InteligenteEscolherAlvo => Consumidor::Modo,
            Self::Espelhar
            | Self::Apontar
            | Self::Prender
            | Self::Assar
            | Self::Soltar
            | Self::ReporRepouso
            | Self::GuardarRepouso
            | Self::AncoraPor
            | Self::AncoraTirar
            | Self::LimitePor
            | Self::LimiteTirar
            | Self::InteligentePor
            | Self::InteligenteTirar => Consumidor::Mundo,
        }
    }
}

/// ⭐⭐⭐ **A grandeza em que o efeito de um verbo é observável.**
///
/// ⚠️ **Ela existe porque «não mexeu no mundo» tem DUAS leituras e as curas são opostas:** um verbo
/// morto liga-se; um verbo cujo efeito é armar um gesto está **certo** e uma régua que o medisse no
/// mundo acusaria um controlo vivo. *Uma célula sem proveniência e uma com proveniência têm o mesmo
/// aspecto numa tabela* — é a mesma lei do `Strength` no censo dos números.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Consumidor {
    /// O verbo mexe no **mundo** — um componente de uma entidade muda, nasce ou sai. É o que a
    /// captura do undo fotografa, e é a régua do censo.
    Mundo,
    /// ⛔ O verbo arma um **MODO** da shell, e o consumidor dele é o **clique seguinte**.
    ///
    /// O *Pick Object* captura o osso e espera: o próximo clique, no canvas **ou** na Hierarquia,
    /// diz de que objecto o controlo trata. ⚠️ **O osso é capturado no ARMAR e não lido no clique
    /// seguinte**, porque aquele clique MUDA a selecção — lê-lo então leria o alvo no lugar do
    /// sujeito. ⇒ o efeito vive no estado da shell (`skeleton.smart_pick`), e quem o prova é o gate
    /// de costura do painel, não este censo.
    Modo,
}

/// ⭐⭐ **Que verbo do osso este id é** — a POSIÇÃO na tabela É a variante.
///
/// ⚠️⚠️ **É a mesma lei que o lado da dobra e o sentido do pincel de peso já usam**, e ela existe
/// para não haver uma segunda lista: um `match` de catorze braços escritos à mão ao lado da
/// [`ids::VECTOR_BONE_VERBS`] seria duas respostas à mesma pergunta, e a que o artista vê é a que
/// envelhece. ⛔ **O preço está pago com gate:** trocar dois itens da tabela de ids faria o botão
/// que diz *Bind* mandar *Release* — *um botão que faz o contrário do que diz é pior do que um
/// morto* —, e por isso o censo pina **cada id ao verbo pelo NOME**, um a um.
///
/// ⚠️ Devolve `Option` e é isso que o mantém honesto: um id que não seja desta secção cai fora e
/// segue a cadeia do despacho.
#[must_use]
pub fn of_id(id: NodeId) -> Option<VerboDoOsso> {
    ids::VECTOR_BONE_VERBS
        .iter()
        .position(|x| *x == id)
        .and_then(|i| VerboDoOsso::TODOS.get(i).copied())
}

#[cfg(test)]
#[path = "censo_dos_verbos_do_osso_tests.rs"]
mod censo_dos_verbos_do_osso_tests;

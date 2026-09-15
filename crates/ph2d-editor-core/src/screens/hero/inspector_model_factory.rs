//! ⭐⭐⭐ **O que o Inspector mostra da FÁBRICA e do CICLO DE VIDA** (TOP-20 #11 e #12, W3).
//!
//! # ⚠️ Um snapshot, DUAS secções — e a razão é o SUJEITO
//!
//! A `Factory` vive no objecto que fabrica; a `Lifetime` e o `DestroyOutside` vivem na **RECEITA**,
//! que é outro objecto. Uma secção só chamada *Factory* a mostrar apenas uma vida seria um título
//! a mentir. ⇒ **duas secções** (*Factory* e *Lifecycle*), **um** snapshot e **uma** enum de edição:
//! a plumbing é a mesma pergunta — *«o que este objecto tem do assunto NASCER e MORRER?»*.
//!
//! # ⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! Quatro avisos, e cada um responde a uma forma diferente de *«liguei tudo e nada nasce»*:
//!
//! | aviso | o que se passa | como se cura |
//! |---|---|---|
//! | `no recipe` | o `Recipe` não aponta a um mestre vivo | escolher a receita |
//! | `never fires` | o `On Signal` está vazio ⇒ nada a acorda | escrever o nome do sinal |
//! | `only runs while the clock plays` | a corrida é o relógio a andar | carregar no play |
//! | `nothing is born from this object` | uma vida num objecto que nenhuma fábrica fez nascer | pôr o componente na RECEITA |
//!
//! ⚠️ **O quarto é a metade honesta do `Timer`** (*«This timer never starts»*): a lei é *a morte só
//! alcança quem nasceu numa corrida*, e sem esta linha um `Lifetime` num objecto desenhado é um
//! controlo que parece morto.
//!
//! ⚠️ **E o quinto é do `DestroyOutside`**: sem uma `GameCamera` na cena ele **não mede nada** — o
//! fora-do-ecrã precisa de um ecrã, e o ecrã de um jogo é a câmera dele, nunca a vista do editor.

/// Onde a cópia nasce — o espelho do `ph2d_ecs::SpawnAt` para o painel.
///
/// ⚠️ **Uma tag SEPARADA do enum do ECS**, como todos os selectores deste painel: o painel fala em
/// posições de segmentado, e o ECS fala em variantes. Ligar os dois pelo `as u8` faria reordenar o
/// enum trocar o que um clique escreve — **e compila**.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InspectorSpawnWhere {
    /// Na pose da própria fábrica.
    #[default]
    Here,
    /// Espalhadas numa caixa à volta dela.
    Area,
    /// Num objecto marcado com uma tag.
    Tagged,
}

impl InspectorSpawnWhere {
    /// Todas, na ordem do segmentado. ⛔ A posição é a tag do clique — não reordene.
    pub const ALL: [Self; 3] = [Self::Here, Self::Area, Self::Tagged];

    /// O rótulo que o artista lê (inglês, HR-15).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Here => "Here",
            Self::Area => "Area",
            Self::Tagged => "At Tag",
        }
    }

    /// A posição no segmentado.
    #[must_use]
    pub const fn tag(self) -> u8 {
        self as u8
    }

    /// **Este modo LÊ a caixa?** — é o que decide se o painel pinta a linha `Area`.
    ///
    /// ⚠️ **Derivado do modo, nunca uma segunda lista** — a lei do `SignalVerb::uses_arg`: um painel
    /// que mostra um campo que o modo não lê é um controlo morto; um que o esconde onde o modo o lê
    /// é uma feature inalcançável.
    #[must_use]
    pub const fn uses_area(self) -> bool {
        matches!(self, Self::Area)
    }

    /// **Este modo lê a TAG e a escolha?**
    #[must_use]
    pub const fn uses_tag(self) -> bool {
        matches!(self, Self::Tagged)
    }
}

/// O corpo da FÁBRICA, como o painel o lê.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InspectorFactory {
    /// O nome da receita, já resolvido do `StableId` — ⚠️ **o painel nunca vê um id**.
    pub recipe: String,
    /// A receita aponta a um mestre vivo? `false` ⇒ o aviso `no recipe`.
    pub recipe_found: bool,
    pub on_signal: String,
    pub spawn_where: InspectorSpawnWhere,
    pub area: [f32; 2],
    /// O nome da tag dos pontos de nascimento (vazio = nenhuma escolhida).
    pub tag: String,
    /// `true` = ao acaso; `false` = em roda-viva.
    pub pick_random: bool,
    pub burst: u32,
    pub alive_max: u32,
    pub total_max: u32,
    pub on_spawned: String,
    pub on_exhausted: String,
    pub seed: u64,
    /// ⭐ **Quantas cópias desta fábrica estão vivas AGORA** — derivado do mundo, nunca guardado.
    ///
    /// ⚠️ Ele é a única coisa desta secção que muda sem o artista tocar em nada, e é o que
    /// responde *«a fábrica está a trabalhar?»* sem o obrigar a contar objectos no ecrã.
    pub alive: u32,
}

/// O corpo do CICLO DE VIDA.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InspectorLifecycle {
    /// A vida, em segundos. `None` = o objecto não tem `Lifetime`.
    pub lifetime_s: Option<f32>,
    /// O sinal da morte (vazio = calada).
    pub on_death: String,
    /// A margem do fora-do-ecrã, em metros. `None` = o objecto não tem `DestroyOutside`.
    pub outside_margin: Option<f32>,
}

/// Snapshot das secções FACTORY e LIFECYCLE da entidade selecionada.
///
/// ⚠️ **Ela existe se o objecto tiver QUALQUER um dos três componentes** — é o ADR-0166: *o
/// Inspector mostra o que o objecto TEM*.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorFactoryInfo {
    pub entity_bits: u64,
    /// `None` = este objecto não fabrica.
    pub factory: Option<InspectorFactory>,
    /// `None` = este objecto não tem nem vida nem fora-do-ecrã.
    pub lifecycle: Option<InspectorLifecycle>,
    /// ⭐ **Este objecto NASCEU numa corrida?** — a metade honesta: se não, os componentes de
    /// ciclo de vida são inertes nele, e o painel di-lo em vez de os deixar parecer partidos.
    pub is_spawned: bool,
    /// ⭐ **Há uma `GameCamera` na cena?** Sem ela o fora-do-ecrã não mede nada.
    pub has_game_camera: bool,
    /// ⭐ **O relógio está a andar?** A corrida é isso, e sem ela nada nasce.
    pub clock_playing: bool,
    pub selected_count: usize,
}

/// Uma edição de um campo das secções FACTORY / LIFECYCLE.
///
/// ⚠️ **Uma enum para os TRÊS componentes**, como a da câmera: o painel fala com a shell num canal
/// só, e a shell sabe qual componente cada variante toca.
#[derive(Clone, Debug, PartialEq)]
pub enum FactoryFieldEdit {
    /// O NOME da receita — ⚠️ nunca os bits (a referência durável desta casa é o nome).
    Recipe(String),
    OnSignal(String),
    Where(InspectorSpawnWhere),
    Area([f32; 2]),
    Tag(String),
    PickRandom(bool),
    Burst(u32),
    AliveMax(u32),
    TotalMax(u32),
    OnSpawned(String),
    OnExhausted(String),
    Seed(u64),
    /// A vida, em segundos. ⚠️ `0` **não mata** — a mesma recusa embutida do `Timer`.
    LifetimeSeconds(f32),
    OnDeath(String),
    /// A folga do fora-do-ecrã, em metros.
    OutsideMargin(f32),
}

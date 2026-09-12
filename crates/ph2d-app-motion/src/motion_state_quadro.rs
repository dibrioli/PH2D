//! **O QUE O QUADRO LÊ E PUBLICA, e que o documento nunca guarda** — irmão do
//! [`motion_state`](super::motion_state) pelo tecto de 600 LOC (HR-18), pelo mesmo corte por
//! ASSUNTO do `motion_state_clip.rs` ao lado.
//!
//! ⚠️ **O que junta os dois tipos daqui não é o tamanho, é a VIDA:** os dois são factos de **um
//! quadro** — um `pulse.signal` que disparou neste tique, e que forma está seleccionada agora.
//! Nenhum persiste, nenhum entra no undo, e nenhum é estado de cena. Guardá-los no documento
//! seria a segunda cópia de coisas que já vivem noutro sítio (a outbox e o `hero.gizmo`).

/// Um `pulse.signal` que disparou num tique deste quadro.
///
/// ⚠️ **Isto NÃO é um `ph2d_runtime::Signal`, e a distância é o desenho:** o grafo não conhece
/// a outbox e não chama ninguém (ADR-0075) — ele deixa um fato aqui, e quem o transforma em
/// sinal é o shell, que já é o dono da saída e já drena as outras duas fontes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MotionSignalOut {
    /// O nome autorado no text param do nó.
    pub name: String,
    /// O tique fixo do cook em que ele disparou.
    pub tick: u64,
    /// Quantas LINHAS dispararam nesse tique — o que o colapso por-quadro descartaria.
    pub rows: usize,
}

/// ⭐⭐ **A RESPOSTA DO *«Use Selected Path»*, e cada saída NOMEIA-SE.**
///
/// ⛔ A 1.ª redacção devolvia `Option<String>` e o botão dizia sempre a mesma frase —
/// *«escolha um desenho: ele precisa de um nome e de pelo menos dois pontos»*. ⚠️ **Metade
/// dela não pode acontecer:** todo desenho nasce com nome (`vec_entities::initial_name`, um
/// `Path {id}`), então *«precisa de um nome»* descrevia uma população vazia — enquanto o
/// motivo verdadeiro (nada seleccionado · não é um desenho · não tem arco) ficava por dizer.
///
/// É a mesma lei do `fell` da ponte de GPU: *toda saída passa por aqui e nomeia-se*.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum FormaEscolhida {
    /// Nada seleccionado.
    #[default]
    Nada,
    /// Há selecção, e ela não é um desenho (uma sprite, um osso, um grupo…).
    NaoEDesenho,
    /// É um desenho, e o nome dele está vazio ou pertence ao namespace do editor (`$…`).
    SemNome,
    /// É um desenho de menos de dois pontos — não há arco por onde um nó andar.
    SemArco,
    /// O nome que o nó pode escrever.
    Nome(String),
}

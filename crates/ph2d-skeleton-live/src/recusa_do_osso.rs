//! ⭐⭐⭐ **OS VERBOS DO OSSO DIZEM PORQUE NÃO VÃO FAZER NADA** — a porta que separa *«prendi ao esqueleto que
//! nomeaste»* de *«prendi ao que havia na cena»*.
//!
//! ⛔⛔⛔ **O defeito que ela cura está MEDIDO** (`sonda_do_rig_partilhado_tests.rs`): o botão *Bind*
//! passa `semente = osso_selecionado`, e com nada de osso aceso ele é `None`; o [`skeleton_of`]
//! responde a `None` com **todos os ossos da cena**. Com um esqueleto isso é a resposta certa e
//! conveniente — com dois, a pele guardou **6** ossos de duas cadeias que não se conhecem, a forma
//! passou a obedecer às duas, e o log do produto disse *«1 imagem presa»*.
//!
//! ⚠️ **A cerca é o que a torna aceitável:** com **um** esqueleto na cena o caminho é byte-idêntico
//! ao de sempre. *Exigir sempre o osso partiria o fluxo que o artista já aprendeu, para curar um
//! caso que só existe quando há ambiguidade.*
//!
//! ⚠️ **Ela devolve a RAZÃO, nunca a imprime** — senão a metade que interessa (*a razão certa para o
//! facto certo*) fica fora de qualquer teste, e a decisão precisaria de um `SimWorld` desenhado.
//!
//! ⭐⭐⭐ **E AS TRÊS SOBEM À TELA** (2026-09-18): a dívida que esta wave abriu — *uma recusa que só
//! o terminal vê é um botão mudo* — fechou com a superfície que a casa **já tinha** (a
//! [`ph2d_editor_core::ToastQueue`], que a irmã desta mesma família, o *solta-se-sozinho* de uma
//! ferramenta de moldura, já usava). ⇒ nenhuma superfície nova, e as **três** juntas, porque curar
//! só uma deixaria duas maneiras de responder à mesma pergunta.
//!
//! ⚠️ **O terminal FICA ao lado do aviso, e não é duplicação:** um smoke headless não tem tela, e é
//! ali que a sonda lê. *A tela é para o artista; o terminal é para quem mede* — o mesmo par que o
//! `PH2D_BONE_LOG` já é.

use crate::esqueletos::bone_roots;
use ph2d_ecs::{Entity, SimWorld};

/// Porque é que um verbo do osso não vai fazer nada.
///
/// ⚠️ **UM enum para os DOIS verbos**, e não um por verbo: eles partilham a superfície (a fila de
/// avisos) e o censo *«toda recusa tem voz»* só é derivável de uma população só.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecusaDoOsso {
    /// Há mais de um esqueleto na cena e nenhum osso escolhido — prender agora juntaria todos.
    VariosEsqueletos {
        /// Quantos esqueletos independentes a cena tem.
        quantos: usize,
    },
    /// O *Bind* não tem sujeito: nada escolhido que se possa prender.
    NadaAPrender,
    /// O *Release* não tem sujeito: nada escolhido que esteja preso.
    NadaASoltar,
    /// A escolha da LEI de pele não tem sujeito: nada escolhido que tenha pele (2026-09-19).
    ///
    /// ⚠️ **Ela não é a [`Self::NadaASoltar`] com outro nome**, e a diferença é o que o artista lê:
    /// ali a saída é *soltar*, aqui é *escolher por que lei isto se deforma*. ⛔ Reaproveitar a
    /// irmã diria *«nada a soltar»* a quem carregou noutro botão — a espécie de mentira que esta
    /// porta existe para não ter.
    ///
    /// ⚠️ **E ela NÃO cobre *«já estava nessa lei»***: escrever a lei que já lá está não é um
    /// acontecimento, e queixar-se disso é o ruído que o artista aprende a ignorar.
    NadaAQuemMudarALei,
    /// O pincel de PESO não caiu em arte presa nenhuma (2026-09-19).
    ///
    /// ⚠️ **É a recusa mais comum deste verbo**, e é por isso que ela tem de falar: o artista clica
    /// ao lado da peça, nada acontece, e *um pincel que não faz nada e não diz porquê é
    /// indistinguível de um pincel partido*.
    PincelForaDaArte,
    /// O pincel de PESO caiu numa coisa que não tem pele — a guarda defensiva do alvo.
    PincelSemPele,
    /// O osso em foco não é um dos ossos que prendem esta arte (2026-09-19).
    ///
    /// ⚠️ **Ela NÃO é «este osso tem peso zero aqui»**, e a diferença decide o que o artista faz:
    /// um osso com peso zero ganha peso se ele pintar (é literalmente o que o pincel serve para
    /// fazer); um osso que não prende esta arte **nunca** vai ganhar, por mais que ele pinte.
    PincelOssoDeFora,
    /// O *Rest Pose* não tem destino: nenhum osso da sub-árvore tem repouso guardado (2026-09-19).
    ///
    /// ⚠️ **Ela é INERTE de propósito** — a alternativa era cair na identidade, que é exactamente o
    /// defeito medido que a pose de repouso veio curar. *Um osso sem repouso guardado não é um osso
    /// que se possa mandar para a origem; é um osso sobre o qual não há resposta.*
    SemPoseDeRepouso,
}

impl RecusaDoOsso {
    /// ⭐⭐ **TODA recusa tem chave de i18n** — ela vai para a TELA, e ali não há texto cru (HR-15).
    ///
    /// ⚠️ **O `match` é exaustivo de propósito:** uma variante nova **não compila** até alguém lhe
    /// dar uma chave. *É a diferença entre uma lista que alguém tem de se lembrar de estender e uma
    /// que não fica verde sem a extensão.*
    #[must_use]
    pub fn chave(self) -> &'static str {
        match self {
            Self::VariosEsqueletos { .. } => "skeleton.recusa.varios_esqueletos",
            Self::NadaAPrender => "skeleton.recusa.nada_a_prender",
            Self::NadaASoltar => "skeleton.recusa.nada_a_soltar",
            Self::NadaAQuemMudarALei => "skeleton.recusa.nada_a_quem_mudar_a_lei",
            Self::SemPoseDeRepouso => "skeleton.recusa.sem_pose_de_repouso",
            Self::PincelForaDaArte => "skeleton.recusa.pincel_fora_da_arte",
            Self::PincelSemPele => "skeleton.recusa.pincel_sem_pele",
            Self::PincelOssoDeFora => "skeleton.recusa.pincel_osso_de_fora",
        }
    }

    /// Quantos esqueletos a cena tem, quando a recusa o nomeia.
    ///
    /// ⚠️ Uma recusa que diz *«há esqueletos a mais»* sem dizer **quantos** não se distingue de uma
    /// queixa genérica, e o artista não sabe se apagou o de que não precisava.
    #[must_use]
    pub fn quantos(self) -> Option<usize> {
        match self {
            Self::VariosEsqueletos { quantos } => Some(quantos),
            Self::NadaAPrender
            | Self::NadaASoltar
            | Self::NadaAQuemMudarALei
            | Self::SemPoseDeRepouso
            | Self::PincelForaDaArte
            | Self::PincelSemPele
            | Self::PincelOssoDeFora => None,
        }
    }

    /// **Todas as recusas que existem** — a população do censo.
    ///
    /// ⚠️ Escrita à mão e **guardada por um gate de exaustividade** (o `match` do [`Self::chave`]),
    /// porque um `enum` com dados não se enumera sozinho.
    pub const TODAS: [Self; 8] = [
        Self::VariosEsqueletos { quantos: 2 },
        Self::NadaAPrender,
        Self::NadaASoltar,
        Self::NadaAQuemMudarALei,
        Self::SemPoseDeRepouso,
        Self::PincelForaDaArte,
        Self::PincelSemPele,
        Self::PincelOssoDeFora,
    ];
}

/// **Este *Bind* pode correr?** — `None` quer dizer *sim*.
///
/// ⚠️ **A metade NEGATIVA é metade do valor:** com um esqueleto só, ou com o osso escolhido, ela
/// cala-se. *Um botão que se queixa sempre é ruído que o artista aprende a ignorar, exactamente
/// quando a queixa passar a ser verdade.*
#[must_use]
pub fn recusa_do_bind(sim: &SimWorld, semente: Option<Entity>) -> Option<RecusaDoOsso> {
    if semente.is_some() {
        return None;
    }
    let quantos = bone_roots(sim).len();
    (quantos > 1).then_some(RecusaDoOsso::VariosEsqueletos { quantos })
}

#[cfg(test)]
#[path = "recusa_do_osso_tests.rs"]
mod tests;

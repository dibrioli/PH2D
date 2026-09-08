//! ⭐⭐⭐ **OS CINCO TIPOS DO FILTRO DE TECIDO** (espec §7) e o referencial deles.
//!
//! ⚠️ **Eles NÃO são os oito modos do pincel com outro nome**, e a espec §7 diz
//! porquê pelo lado da ausência: *o filtro não tem Drag, Push, Grab, Snake Hook
//! nem Pinch Perpendicular — são gestos de pincel, precisam de um cursor com
//! direcção.* Dos cinco que ficam, **dois não existiam** em modo nenhum
//! (`Gravity` e `Scale`) e os outros três são leis que o pincel já tinha,
//! alcançadas por outro accionamento.
//!
//! O censo que decidiu isto está em `ph2d-cloth/tests/mede_a_composicao_do_filtro.rs`.

use ph2d_cloth::verlet_gesto::Modo;

/// **O que o filtro de tecido faz à peça inteira** (espec §7).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ClothFilterKind {
    /// Puxa tudo na direcção do «baixo» do referencial escolhido.
    ///
    /// ⭐ É a única lei do módulo cuja direcção o CHAMADOR dita — e por isso a
    /// única que não roda quando a peça roda (gate
    /// `a_gravidade_do_filtro_nao_roda_com_a_peca_e_o_inflate_roda`).
    #[default]
    Gravity,
    /// Empurra cada vértice ao longo da normal dele.
    ///
    /// ⚠️⚠️ **A mesma palavra nomeia DUAS leis** (espec §4.2-ter contra §7): o
    /// traço do pincel lê as normais da superfície que ENCONTROU e o filtro
    /// **refresca-as a cada passo**. Medido: `30,30 %` de divergência em seis
    /// passos, `0,000` num só. ⇒ [`ClothFilterKind::refresca_normais`].
    Inflate,
    /// Muda o comprimento de REPOUSO: a peça cresce em vez de se deslocar.
    Expand,
    /// Aperta tudo contra um ponto — **o que estava sob o cursor quando o filtro
    /// começou**. ⛔ Ele não segue o rato (espec §7), e é o chamador que o
    /// congela.
    Pinch,
    /// Escala em torno da ORIGEM DO OBJECTO, pela âncora `p⁰ + p⁰ · f`.
    Scale,
}

impl ClothFilterKind {
    /// Os cinco, na ordem em que a fileira os desenha.
    ///
    /// ⚠️ **A ordem é NOSSA** (a da espec §7 ao apresentá-los) e não um facto do
    /// alvo. ⛔ Quem a mudar tem de olhar se algum ficheiro guarda o ÍNDICE em
    /// vez do nome — a mesma advertência do [`crate::ClothMode`].
    pub const ALL: [Self; 5] = [
        Self::Gravity,
        Self::Inflate,
        Self::Expand,
        Self::Pinch,
        Self::Scale,
    ];

    /// O rótulo do chip (a UI da casa é inglesa).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Gravity => "Gravity",
            Self::Inflate => "Inflate",
            Self::Expand => "Expand",
            Self::Pinch => "Pinch",
            Self::Scale => "Scale",
        }
    }

    /// A lei correspondente na `ph2d-cloth`.
    ///
    /// ⚠️ **É a ÚNICA porta entre o vocabulário do painel e o da lei** — um
    /// `match` exaustivo, como o do [`crate::ClothMode`]: um tipo novo de
    /// qualquer dos lados é erro de compilação aqui.
    #[must_use]
    pub fn modo(self) -> Modo {
        match self {
            Self::Gravity => Modo::Gravidade,
            Self::Inflate => Modo::Inflar,
            Self::Expand => Modo::Expandir,
            Self::Pinch => Modo::ApertarPonto,
            Self::Scale => Modo::Escala,
        }
    }

    /// **Este tipo lê as normais da malha?**
    ///
    /// ⭐ Só o [`Self::Inflate`]. A porta existe porque a resposta decide se o
    /// passo paga uma travessia da malha para as recalcular — e porque ela tem
    /// **dois** leitores: o adaptador, que decide, e o gate, que o prova.
    #[must_use]
    pub fn le_as_normais(self) -> bool {
        matches!(self, Self::Inflate)
    }

    /// **Este tipo precisa do ponto congelado do pen-down?**
    ///
    /// Só o [`Self::Pinch`] — os outros quatro não têm alvo no espaço.
    #[must_use]
    pub fn le_o_ponto(self) -> bool {
        matches!(self, Self::Pinch)
    }

    /// **Este tipo lê os eixos ligados do referencial?**
    ///
    /// ⚠️ Só a [`Self::Scale`], e é a espec §7 que o diz com a medição dentro:
    /// *o código lido só aplica as bandeiras de eixo ao Scale; a limitação de
    /// eixos das forças passa pela orientação.* ⛔ Alargá-las às forças seria
    /// **lei nova**, não fidelidade.
    #[must_use]
    pub fn le_os_eixos(self) -> bool {
        matches!(self, Self::Scale)
    }
}

/// **O REFERENCIAL do filtro** (espec §7, *Orientation*).
///
/// ⚠️ **A resolução mora em quem tem a CÂMERA**, não aqui: esta crate não sabe
/// o que é uma vista, e a `ph2d-cloth` sabe menos ainda. O enum existe para o
/// painel ter o que nomear e para o shell ter o que resolver.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ClothFilterOrientation {
    /// Os eixos do objecto — a omissão do alvo.
    #[default]
    Local,
    /// Os eixos do mundo.
    World,
    /// Os eixos do ECRÃ.
    ///
    /// ⚠️⚠️ **Aqui o «baixo» da gravidade é o eixo VERTICAL DO ECRÃ, não a
    /// profundidade** (espec §7) — para que a queda seja o baixo que o artista
    /// vê. É o único caso especial do referencial, e ele vive em quem resolve.
    View,
}

impl ClothFilterOrientation {
    /// Os três, na ordem do painel do alvo.
    pub const ALL: [Self; 3] = [Self::Local, Self::World, Self::View];

    /// O rótulo do chip.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Local => "Local",
            Self::World => "World",
            Self::View => "View",
        }
    }
}

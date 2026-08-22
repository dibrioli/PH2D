//! **AS ESCOLHAS NOMEADAS do painel 3D** — os enums que um chip escreve.
//!
//! Irmão do [`super::state`], e o corte foi forçado pelo teto de LOC dos painéis
//! (653 contra 600). ⭐ **Mas ele é de ASSUNTO:** lá mora *o estado inteiro que o
//! painel autora* — o pincel, os slots, o retrato, os intents —, aqui as
//! **escolhas discretas** que um chip escreve, cada uma com o seu `ALL` e o seu
//! rótulo i18n.
//!
//! ⚠️ **A lei comum às duas:** o `ALL` é a ordem em que os chips são pintados, e o
//! id de cada chip é indexado por essa posição — **nunca** pelo discriminante. Um
//! `ALL` reordenado com índices por discriminante dá um chip rotulado `Fast`
//! escrevendo `Even Grid`: pintado, vivo sob o mouse, e a mentir.

/// **COM QUE PROFUNDIDADE O PAINEL SE MOSTRA** (§2 do plano).
///
/// ⚠️ **Isto não são dois conjuntos de features — é divulgação progressiva do
/// MESMO estado**, e essa escolha é o que impede duas fontes de verdade. Em
/// `Pro` o artista não ganha números novos: ele ganha *acesso* aos números que o
/// verbo e o modo já haviam armado por ele.
///
/// ⚠️ **A regra de quem pode ser `Pro`, e ela é testável:** só uma row cujo
/// valor **o slot do verbo já traz** ([`VerbSlot::for_verb`]). Esconder um
/// número que a ferramenta escolheu bem é divulgação progressiva; esconder um
/// que nasce neutro e tem de ser fornecido é amputação — o artista ficaria com
/// uma ferramenta que não faz o que o nome dela diz e sem nada na tela
/// explicando por quê.
///
/// ⚠️ **Ela é NECESSÁRIA e não suficiente, e é isso que o falloff custou:** a
/// curva nasce no slot do verbo, logo *podia* ser `Pro` — e era, e o
/// smoke reprovou (*"não dá a opção de escolher o falloff e deveria dar"*).
/// Quem decide a segunda metade é a REFERÊNCIA, medida e não lembrada: no
/// Blender a curva é *dobrada* (`DEFAULT_CLOSED` com cabeçalho à vista, mais um
/// popover no cabeçalho de ferramenta), nunca *ausente*. **Dobrar é divulgação
/// progressiva; sumir sem rastro é amputação**, e o nosso `Pro` fazia o segundo.
///
/// ⚠️ **`Ord` é a lei inteira:** uma row aparece quando `nível do painel >=
/// nível da row`. Escrito como dois `if`s (um por lado) o terceiro degrau nasce
/// fora da regra.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum UiLevel {
    /// O que TODO pincel tem: o verbo, a referência, o raio, a força e a
    /// **CURVA**.
    ///
    /// ⚠️ **Isto dizia *"o vocabulário do SculptGL"* e a frase custou um
    /// smoke.** O SculptGL **não tem** seletor de curva — a dele é fixa —, então
    /// herdar o vocabulário dele apagava do Basic um controle que a nossa malha
    /// tem **doze** vezes e que a OUTRA referência trata como primeiro-classe (o
    /// `FalloffPanel` do Blender não é `brush_settings_advanced`, e no cabeçalho
    /// de ferramenta ele é um popover sempre visível). *Um vocabulário herdado
    /// descreve a ferramenta de onde veio, não a que se está a construir.*
    #[default]
    Basic,
    /// Mais os knobs que o modo tinha armado.
    Pro,
}

impl UiLevel {
    /// A ordem em que os chips são pintados. **É** a ordem do enum.
    pub const ALL: [Self; 2] = [Self::Basic, Self::Pro];

    /// Chave i18n do rótulo.
    pub fn label(self) -> &'static str {
        match self {
            Self::Basic => ph2d_i18n::tr("panel.sculpt3d.ui_level.basic"),
            Self::Pro => ph2d_i18n::tr("panel.sculpt3d.ui_level.pro"),
        }
    }

    /// **Uma coisa que exige `needs` aparece neste nível?** A porta única — o
    /// pintor a consulta para desenhar e o gate de costura para varrer.
    pub fn shows(self, needs: Self) -> bool {
        needs <= self
    }
}

/// **QUAL MOTOR DE RETOPOLOGIA o botão chama** — e os dois são de espécie
/// diferente, não um melhor e um pior.
///
/// ⭐ **Medido lado a lado, na mesma peça e no mesmo gesto** (2026-08-21, esfera
/// com bico, `detail = 0,5`):
///
/// | | quads | irregulares | bordo | relógio |
/// |---|---|---|---|---|
/// | [`Self::Global`] | **100 %** | 19 | **0** | ~330 ms |
/// | [`Self::Local`] | 63 % | não conta | 0 | **~70 ms** |
///
/// ⚠️ **O `Local` não é um modo de compatibilidade.** Ele é o porte fiel do
/// *Instant Meshes* (BSD-3, `ph2d-quadflow`), que é o clássico rápido e robusto da
/// família: a grade dele segue a curvatura, ele responde em sub-segundo e **nunca
/// precisa de um layout global fechar**. O `Global` entrega a promessa inteira —
/// 100 % de quads e a contagem de irregulares perto do chão topológico — e paga
/// isso em relógio e em recusas nomeadas quando o traçado não fecha.
///
/// ⛔ **Ele esteve escondido atrás de `PH2D_RETOPO_LEGACY=1` durante toda a wave
/// do pivô**, ou seja: alcançável só por quem soubesse o nome da variável.
/// *Um motor que o painel não oferece não existe para o artista.*
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RetopoMode {
    /// A cadeia **global** — campo cruzado, patches, quantização inteira.
    #[default]
    Global,
    /// O porte do **Instant Meshes** — local, rápido, robusto.
    Local,
}

impl RetopoMode {
    /// A ordem em que os chips são pintados. **É** a ordem do enum.
    pub const ALL: [Self; 2] = [Self::Global, Self::Local];

    /// Chave i18n do rótulo.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Global => ph2d_i18n::tr("panel.sculpt3d.retopo_mode.global"),
            Self::Local => ph2d_i18n::tr("panel.sculpt3d.retopo_mode.local"),
        }
    }

    /// **Este motor consome a densidade adaptativa?** ⚠️ Só o local — e é por isso
    /// que o painel avisa quando o knob não é zero no outro.
    #[must_use]
    pub fn uses_adaptive(self) -> bool {
        matches!(self, Self::Local)
    }
}

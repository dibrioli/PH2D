//! ⭐⭐⭐ **A PELE DE UMA COISA, E POR QUE LEI ELA SE DEFORMA** — o [`SkinBind`] e o [`SkinLaw`].
//!
//! ⚠️ **Ele saiu do `lib.rs` por um TECTO DE LOC** (`705` contra `700`), e o corte é por
//! RESPONSABILIDADE e não pelo fim do ficheiro: o que vive aqui é *o que uma coisa presa guarda e
//! por que lei ela se deforma*, e isso não tem nada a ver com o OSSO (a curvatura, o limite, a
//! âncora de IK), que fica lá. ⛔ **Nunca uma entrada nova no `FILE_OVERAGE_OK`.**

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use ph2d_ecs::SimComponent;

use crate::{StableId, Tendon};

/// ⭐⭐⭐ **POR QUE LEI ESTE DESENHO SE DEFORMA** — a escolha que o dono mandou construir
/// (2026-09-19: *«como se escolhe se os envelopes vão ou não influenciar?»* ⇒ *«construa. por
/// desenho»*).
///
/// ⛔⛔⛔ **Antes disto NÃO SE ESCOLHIA: o app decidia, e decidia pelo DESENHO.** Uma forma com
/// interior (ou uma imagem cuja solução converge) ia para o padrão-ouro e o alcance de cada osso
/// ficava **inerte**; um traço ABERTO não tem interior, logo não tem domínio para a energia, e caía
/// na lei euclidiana, onde o alcance manda. *Qual lei deforma o personagem é uma decisão de RIG, e
/// ela estava escondida dentro de uma decisão de DESENHO.*
///
/// ⭐⭐ **E a capacidade já existia inteira** — medido antes de escrever uma linha: com a tabela de
/// pesos apagada, uma forma FECHADA corre na lei do envelope e move **exactamente** o mesmo que o
/// traço aberto (`6,4368` contra `6,4368` sobre a mesma curva; um rectângulo move `6,9835`). *O que
/// faltava não era motor, era o botão.*
///
/// ⚠️⚠️ **Ela decide se o quadro LÊ a tabela, nunca se ele a CALCULA** — e essa escolha é a wave
/// inteira. O padrão-ouro custa dezenas de milissegundos a resolver e fica guardado no bind; se a
/// escolha mandasse no *cálculo*, voltar atrás obrigaria a re-resolver e o artista veria a
/// ferramenta engasgar ao alternar. Assim ela é **viva**: troca-se no quadro seguinte, nos dois
/// sentidos, e a tabela guardada espera onde está.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkinLaw {
    /// **O padrão-ouro** — os pesos resolvidos sobre a arte ao prender. É o nascimento, e é o que
    /// dá a melhor deformação; ⚠️ aqui o alcance (`Bone::strength`) **não entra na conta**.
    #[default]
    Auto,
    /// **Por alcance** — a lei euclidiana, em que o `Bone::strength` de cada osso governa até onde
    /// ele puxa. É o que o artista afina à mão, arrastando a alça da mancha.
    ///
    /// ⚠️ **Ela vale em QUALQUER desenho**, e é isso que a torna uma escolha: até aqui uma forma
    /// preenchida não tinha como correr nesta lei, por mais que o artista a quisesse.
    Envelope,
}

/// ⭐⭐⭐ **UMA CORRECÇÃO DE PESO FEITA À MÃO** — a mancha que o artista pinta onde a conta
/// automática errou (report do dono, 2026-09-19: *«quando a conta automática erra num sítio, não há
/// como acertar aquele ponto»*).
///
/// # ⚠️ Porque ela é uma MANCHA no espaço, e não uma tabela por vértice
///
/// O doc do [`SkinBind`] já escreve a lei: *«uma tabela de pesos indexada por ordem de varredura é
/// o vector paralelo que o `corner_radius` proíbe por escrito»* — dezenas de operações inserem,
/// apagam, invertem e soldam vértices, e cada uma teria de se lembrar de mexer nela. ⇒ a correcção
/// é **ancorada na geometria**: ela diz *«aqui»*, e continua a dizer «aqui» depois de o artista
/// mexer no desenho.
///
/// ⭐ **E por isso ela vale nas DUAS leis** ([`SkinLaw`]): o artista corrige *aquele ponto*, e de
/// que lei veio o peso que ele está a corrigir não é pergunta dele.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CorreccaoDePeso {
    /// ⭐ **A identidade durável do osso** — o [`StableId`], nunca a posição dele na lista.
    ///
    /// ⚠️ A mesma cerca do [`Tendon::bone`], e pelo mesmo defeito medido: uma posição guardada
    /// passa a apontar para o osso do lado no instante em que alguém apagar um osso — e a arte
    /// corrige-se no sítio errado sem nada reprovar.
    pub bone: StableId,
    /// O centro, em coordenadas **locais da coisa no bind** — o mesmo espaço da [`SkinBind::source`]
    /// e dos eixos de repouso dos tendões.
    pub centro: [f64; 2],
    /// O raio da mancha, nas mesmas unidades.
    pub raio: f64,
    /// Quanto somar ao peso deste osso no CENTRO. **Negativo TIRA**, e o sinal é a direcção: não há
    /// um segundo modo «apagar» a lembrar nem um modificador de teclado a adivinhar.
    pub delta: f64,
}

/// **A PELE DE UMA COISA** — a que ossos ela responde, e o que ela era antes de responder.
///
/// ⚠️ **O nome do TIPO é `SkinBind` e o rótulo que o artista lê é "Skin"** — o tipo diz o que se
/// GUARDA (o bind: a fonte mais as matrizes de repouso), e o rótulo diz o que a coisa É. O par
/// vive no catálogo do `ph2d-component-desc`, que é onde os dois nomes se encontram.
///
/// ⛔ **Não é um container**, ao contrário do `ph2d_ecs::VecEnvelope`, e a diferença é medida:
/// aquele precisa de um container porque a **gaiola não tem outra casa** (não é entidade). Aqui o
/// esqueleto já são entidades, então não há nada de partilhado à procura de dono — e uma forma
/// presa fica exactamente onde o artista a pôs na Hierarquia.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkinBind {
    /// Os bytes postcard da fonte **autorada**, em coordenadas locais da coisa no bind.
    ///
    /// Sem ela a fonte morria no 1.º quadro — o recook sobrescreve a geometria da cena com a
    /// deformada, e é o bug *"funciona e depois esquece"* que o ADR-0121 §3 documentou.
    ///
    /// ⚠️ **Bytes opacos, de propósito:** é o que permite a este componente servir um `VecPath`
    /// hoje e uma malha raster amanhã sem uma variante nova nem um schema por mídia.
    pub source: Vec<u8>,
    /// Os ossos, na ordem em que foram ligados. Um cuja entidade desapareceu é **saltado** no
    /// recook e os outros renormalizam-se sozinhos — apagar um osso não pode apagar a forma.
    pub tendons: Vec<Tendon>,
    /// ⭐⭐⭐ **Por que lei ESTE desenho se deforma** — ver [`SkinLaw`]. Ordem do dono, 2026-09-19:
    /// a escolha é **por desenho**, e não por esqueleto.
    ///
    /// ⚠️ **Ela mora AQUI e não nos bytes opacos do [`Self::source`]**, e a razão é que aqueles
    /// bytes são **por mídia** (uma `SkinnedMesh` para a imagem, um `SkinnedPath` para a forma):
    /// escrevê-la lá seria escrever a MESMA lei em dois sítios, e *uma lei escrita em dois sítios
    /// ainda não é uma lei*. Ela é a mesma pergunta para as duas mídias ⇒ mora no componente que as
    /// duas partilham.
    ///
    /// ⚠️ **Sobrevive a um RE-BIND**, e isso é lei: prender outra vez é o gesto de *«re-prender na
    /// pose actual»*, e ele não pode apagar em silêncio uma escolha que o artista fez. *A chave que
    /// o artista fez manda mais que a correcção automática.*
    pub law: SkinLaw,
    /// ⭐⭐⭐ **AS CORRECÇÕES QUE O ARTISTA PINTOU** — ver [`CorreccaoDePeso`]. Vazio é o nascimento,
    /// e um no-op **ao bit** nas duas leis.
    ///
    /// ⚠️ **Elas moram AQUI e não nos bytes opacos da [`Self::source`]**, pela mesma razão da
    /// [`Self::law`]: aqueles bytes são **por mídia**, e a correcção é a mesma pergunta para as
    /// duas. *Escrevê-la lá seria escrevê-la duas vezes.*
    ///
    /// ⚠️ **Ela sobrevive a um RE-BIND**, como a lei: prender outra vez é *«re-prender na pose
    /// actual»*, e não pode apagar em silêncio o trabalho à mão do artista.
    pub correcoes: Vec<CorreccaoDePeso>,
}

impl SimComponent for SkinBind {}

impl SkinBind {
    /// Uma pele nova. `tendons` vazio é legal e significa *"presa a nada"* — o recook deixa a coisa
    /// em paz, que é a leitura certa de um esqueleto inteiro apagado.
    #[must_use]
    pub fn new(source: Vec<u8>, tendons: Vec<Tendon>) -> Self {
        Self {
            source,
            tendons,
            law: SkinLaw::Auto,
            correcoes: Vec::new(),
        }
    }

    /// ⭐⭐⭐ **AS CORRECÇÕES NO ESPAÇO DA LEI** — a porta ÚNICA que traduz `StableId → TENDÃO`.
    ///
    /// ⚠️ **O índice é a posição na [`Self::tendons`]**, que é exactamente o que o
    /// [`ph2d_skeleton::SkinBone::tendon`] carrega — e é por isso que a tradução é uma busca nesta
    /// lista e não uma contagem de ossos da cena: *a resolução salta ossos apagados, e a posição na
    /// PELE não é a posição no BIND*.
    ///
    /// ⛔ Uma correcção cujo osso já não está nos tendões é **saltada**: ela nomeia um osso que foi
    /// solto desta pele, e aplicá-la ao tendão que ficou naquele índice corrigiria o osso errado.
    ///
    /// ⚠️ **Ela é a mesma para as DUAS mídias**, e é isso que impede a forma e a imagem de
    /// divergirem no primeiro ajuste.
    #[must_use]
    pub fn correcoes_resolvidas(&self) -> Vec<ph2d_skeleton::Correccao> {
        self.correcoes
            .iter()
            .filter_map(|c| {
                let j = self.tendons.iter().position(|t| t.bone == c.bone)?;
                Some(ph2d_skeleton::Correccao {
                    tendon: u32::try_from(j).ok()?,
                    centro: c.centro,
                    raio: c.raio,
                    delta: c.delta,
                })
            })
            .collect()
    }

    /// ⭐⭐⭐ **OS PESOS QUE O QUADRO DEVE LER** — vazio quer dizer *«cai na lei euclidiana»*.
    ///
    /// ⚠️⚠️ **É a PORTA ÚNICA da escolha, e ela tem TRÊS leitores** — o recook de uma forma
    /// vectorial, o desenho de uma imagem presa, e a pergunta *«o envelope manda neste osso?»* que
    /// acende a mancha e a alça. ⛔ Escrita em três sítios, a mancha apareceria onde o alcance não
    /// governa nada (ou sumiria onde governa) no primeiro que alguém mexesse — que é, à letra, o
    /// report que o dono fez em 2026-09-18.
    ///
    /// ⚠️ **A tabela guardada NÃO é apagada** quando a escolha é [`SkinLaw::Envelope`]: ela fica
    /// onde está e volta a ser lida no instante em que o artista voltar ao [`SkinLaw::Auto`].
    /// *Resolver o padrão-ouro custa dezenas de milissegundos; escolher qual lei ler custa um `if`.*
    #[must_use]
    pub fn pesos_do_quadro<'a>(&self, guardados: &'a [f64]) -> &'a [f64] {
        match self.law {
            SkinLaw::Auto => guardados,
            SkinLaw::Envelope => &[],
        }
    }
}

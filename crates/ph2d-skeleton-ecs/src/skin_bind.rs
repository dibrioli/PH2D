//! ⭐⭐⭐ **A PELE DE UMA COISA, E POR QUE LEI ELA SE DEFORMA** — o [`SkinBind`] e o [`SkinLaw`].
//!
//! ⚠️ **Ele saiu do `lib.rs` por um TECTO DE LOC** (`705` contra `700`), e o corte é por
//! RESPONSABILIDADE e não pelo fim do ficheiro: o que vive aqui é *o que uma coisa presa guarda e
//! por que lei ela se deforma*, e isso não tem nada a ver com o OSSO (a curvatura, o limite, a
//! âncora de IK), que fica lá. ⛔ **Nunca uma entrada nova no `FILE_OVERAGE_OK`.**

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use ph2d_ecs::SimComponent;

use crate::Tendon;

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
        }
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

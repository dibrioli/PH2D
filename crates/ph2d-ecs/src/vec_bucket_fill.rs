//! **O preenchimento do BALDE** — a forma que É a região que o artista apontou (plano 40).
//!
//! Irmão do [`crate::VecMorph`], do [`crate::VecBlend`] e do [`crate::VecConnector`] no padrão: o
//! componente guarda a **relação** (que região), e a aparência é **função pura** dela, re-cozida
//! pela shell quando as linhas mudam. Ninguém redesenha um preenchimento — move-se uma linha, e a
//! área refaz-se.
//!
//! # ⭐⭐⭐ Por que a receita é um PONTO e não a lista de arcos
//!
//! Report do Enio (2026-09-01): *"se movo os nós da linha, o preenchimento não acompanha. A área
//! deveria permanecer perfeitamente preenchida mesmo modificando o path."*
//!
//! A 1.ª versão gravava a **geometria** — os arcos concatenados no instante do clique —, e por isso
//! ela envelhecia no primeiro nó arrastado. Guardar a **lista de arcos** em vez disso não resolve:
//! um arco nasce de um corte em fracções, e mover um nó **muda os cruzamentos**, logo muda a
//! própria lista. *Qualquer receita feita de pedaços da rede é uma receita sobre uma rede que já
//! não existe.*
//!
//! ⇒ A receita é o que o artista de facto fez: **apontou ali**. O ponto sobrevive a toda edição que
//! não o expulse da região, e re-perguntar *"que face contém este ponto?"* devolve a região certa
//! mesmo depois de ela mudar de forma, de número de lados e de tamanho.
//!
//! ⚠️ **É a lei do Live Paint do Illustrator com outro substrato** — lá a face é estado vivo de um
//! grupo especial; aqui ela é uma pergunta que se refaz, e por isso não precisa de tipo novo.
//!
//! # ⛔ E um preenchimento não é uma PAREDE
//!
//! Report do mesmo dia: *"ao usar o balde nas áreas coloridas, ele para de funcionar nas áreas não
//! coloridas."* A forma que o balde deposita tem por fronteira os **mesmos arcos** que as linhas —
//! e ao voltar à rede ela punha ali arestas **coincidentes**, duplicadas, cuja direcção de saída é
//! idêntica: o passeio de faces passa a escolher entre duas meias-arestas indistinguíveis, e as
//! regiões vizinhas deixam de fechar.
//!
//! ⇒ Quem tem este componente **não entra na rede**. É a distinção que o próprio verbo já fazia e
//! que o código não sabia: *uma parede é o que o artista desenhou; um preenchimento é o que ele
//! pediu.*

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **A região preenchida.** A entidade que o carrega também tem um [`crate::VecPathRef`], e o
/// `VecPath` dela é a área — geometria de verdade, na cena, re-escrita *em lugar* quando as linhas
/// mudam (para pintar, exportar, animar).
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct VecBucketFill {
    /// **Onde o artista apontou**, em coordenadas de MUNDO.
    ///
    /// ⚠️ **Mundo, e não local**: a região é cercada por linhas de VÁRIOS objectos, cada um com a
    /// sua pose — não existe um espaço local em que a pergunta faça sentido. É a mesma razão pela
    /// qual o corte e a solda medem no mundo.
    pub seed: [f32; 2],
    /// ⭐⭐⭐ **AS ÂNCORAS: os pedaços de linha que cercavam a região no momento do clique.**
    ///
    /// # Porque uma coordenada não chega
    ///
    /// Report do Enio (2026-09-02, quatro vezes seguidas): a tinta trocava de área, sumia e deixava
    /// resíduo. ⚠️⚠️ **A causa não era nenhum dos defeitos um a um — era o modelo.** A receita era
    /// *onde a região estava no quadro anterior*, então ela **derivava**: o que um quadro decidia
    /// virava a régua do seguinte, e um único quadro de topologia confusa reatribuía a tinta para
    /// sempre.
    ///
    /// Uma âncora não deriva. Ela diz *"a região que fica à esquerda do pedaço da curva `c` do
    /// caminho `p` na fracção `f`"* — e arrastar um nó move a curva **sem mudar de que curva o
    /// pedaço é**. ⇒ **o mesmo desenho dá sempre as mesmas cores, seja qual for o caminho por que
    /// se lá chegou.**
    ///
    /// ⚠️ **Uma face tem várias, e é isso que faz o resto cair de graça:** quando a região se PARTE,
    /// umas âncoras passam a cercar uma metade e outras a outra — e o preenchimento fica com as
    /// duas, sem uma linha de código sobre partir.
    ///
    /// ⛔ **Elas só se escrevem no CLIQUE.** Reescrevê-las a cada quadro seria reintroduzir a
    /// deriva com outro nome.
    pub ancoras: Vec<FillAnchor>,
}

/// **Uma âncora**: o lado de um pedaço de contorno do documento.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FillAnchor {
    /// O `VecPathId` do caminho a que o contorno pertence.
    pub path: u64,
    /// O índice do contorno DENTRO desse caminho (`0` = o primário).
    pub contorno: u16,
    /// A fracção de arco, dentro do contorno, de um ponto do pedaço.
    pub frac: f32,
    /// De que lado do pedaço fica a região — `true` = o sentido em que o contorno é percorrido.
    pub frente: bool,
}

impl SimComponent for VecBucketFill {}

impl VecBucketFill {
    /// Um preenchimento novo, semeado no ponto de mundo `seed` e agarrado a `ancoras`.
    #[must_use]
    pub fn new(seed: [f32; 2], ancoras: Vec<FillAnchor>) -> Self {
        Self { seed, ancoras }
    }
}

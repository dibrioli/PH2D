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
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecBucketFill {
    /// **Onde o artista apontou**, em coordenadas de MUNDO.
    ///
    /// ⚠️ **Mundo, e não local**: a região é cercada por linhas de VÁRIOS objectos, cada um com a
    /// sua pose — não existe um espaço local em que a pergunta faça sentido. É a mesma razão pela
    /// qual o corte e a solda medem no mundo.
    pub seed: [f32; 2],
}

impl SimComponent for VecBucketFill {}

impl VecBucketFill {
    /// Um preenchimento novo, semeado no ponto de mundo `seed`.
    #[must_use]
    pub fn new(seed: [f32; 2]) -> Self {
        Self { seed }
    }
}

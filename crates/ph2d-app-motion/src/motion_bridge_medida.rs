//! ⭐⭐⭐ **QUANTO MEDE CADA CARTÃO — o canal que leva a GEOMETRIA do painel até à ARRUMAÇÃO
//! AUTOMÁTICA**, que está duas camadas acima dela e não tem medidor nenhum.
//!
//! Três reports do dono no mesmo dia (2026-09-20) são a mesma ausência: *«nós estão se
//! interpenetrando»* · *«a arrumação deve considerar o tamanho vertical do nó»* · *«o preview
//! deve ser posicionado na melhor posição para não ficar entre nós»*. A disposição em camadas
//! ([`ph2d_nodegraph::layout`]) espaçava por **duas constantes** (`220 × 260`), e desde que a
//! pastilha segue o NOME e o cartão segue a contagem de FILEIRAS nenhuma das duas descreve um
//! cartão: dois nomes compridos lado a lado tocavam-se por construção.
//!
//! ⚠️⚠️ **Porque é aqui e não na `ph2d-motion-doc`:** aquela crate depende do `ph2d-nodegraph` e
//! de mais nada, de propósito — ela não conhece o registo (de onde sai o nome de um tipo), nem a
//! tabela de i18n, nem o medidor de texto. Quem tem os três é esta crate, e o que ela faz é
//! **perguntar ao painel**, que é quem desenha ([`ph2d_panel_motion_graph::extensao_desenhada`]).
//!
//! ⛔ **A medida é tirada UMA vez, para um mapa**, e não por chamada: a arrumação percorre todas
//! as telas do documento (a raiz e o interior de cada grupo) e um nó é medido no máximo uma vez.
//! O empréstimo obriga-o de qualquer maneira — o retrato precisa de `&MotionState` e a arrumação
//! escreve em `&mut doc`.

use crate::motion_state::MotionState;
use ph2d_motion_doc::layout::{Carta, Medida};
use ph2d_nodegraph::layout::Extent;
use std::collections::BTreeMap;

/// O que cada cartão deste documento MEDE, já resolvido.
pub struct Medidas {
    por_no: BTreeMap<u32, Extent>,
    por_grupo: BTreeMap<u32, Extent>,
}

impl Medida for Medidas {
    fn extensao(&self, carta: Carta) -> Extent {
        // ⚠️ O `unwrap_or_default` é o cartão histórico, e ele só é alcançável por um cartão que
        // nasceu DEPOIS da medição — que não existe, porque as duas correm no mesmo instante.
        match carta {
            Carta::No(id) => self.por_no.get(&id.0).copied().unwrap_or_default(),
            Carta::Grupo(sid) => self.por_grupo.get(&sid).copied().unwrap_or_default(),
        }
    }
}

/// ⭐⭐ **Arrumar o documento inteiro** — a porta ÚNICA, e a razão de ela existir é que houve
/// dois chamadores desde o primeiro dia (a tecla do painel e as cenas de smoke) e o segundo
/// herdava, calado, a medida que o primeiro esquecesse.
pub fn arrumar(motion: &mut MotionState) {
    let medidas = medir(motion);
    ph2d_motion_doc::layout::arrange(&mut motion.doc, &medidas);
}

/// Mede cada nó do grafo e cada cartão de grupo.
pub fn medir(motion: &MotionState) -> Medidas {
    // O MESMO retrato que o painel pinta — o nome resolvido pelo registo e pela i18n, e os
    // pinos declarados pelo manifesto (mais um por param CONDUZIDO, que desenha socket).
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&motion.doc.graph, &motion.registry);
    // ⚠️ **As fileiras de PARAM são metade da altura de um cartão aberto** (o `verlet_rope` da
    // `=120` tem treze), e elas não vêm do registo: são as que o cartão MOSTRA. Esta é a mesma
    // porta que o quadro usa. ⛔ E as `ProjectSettings` não entram na CONTAGEM — elas dão as
    // faixas de cada row —, logo medir com as de fábrica mede a mesma altura.
    super::params::card::stamp_card_params(
        motion,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );

    let por_no = snap
        .nodes
        .iter()
        .map(|n| (n.id, ph2d_panel_motion_graph::extensao_desenhada(n)))
        .collect();

    // ⚠️ **Um cartão de GRUPO não passa por aquele retrato** — ele é derivado pela shell (o
    // título do subgrafo, os pinos que atravessam a fronteira, e o «N nós» que ele mostra em vez
    // de um readout de cozimento). ⇒ ele entra pela porta das PEÇAS, que é a mesma aritmética.
    let por_grupo = motion
        .doc
        .subgraphs
        .iter()
        .map(|s| {
            let portas = super::fold::card_ports(motion, s.id);
            let nome = if s.title.is_empty() {
                super::fold::DEFAULT_TITLE.tr().to_string()
            } else {
                s.title.clone()
            };
            #[expect(
                clippy::cast_precision_loss,
                reason = "a contagem de pinos de um cartao cabe num f32"
            )]
            let fileiras = portas.inputs.len().max(portas.outputs.len()).max(1) as f32;
            (
                s.id,
                ph2d_panel_motion_graph::extensao_de(
                    &nome, fileiras, 0.0, true, /* o «N nós» */
                    true, /* ⚠️ o retrato de um cartão é o do que sai dele, e reservá-lo é o
                          lado CONSERVADOR: sobra espaço, nunca falta. */
                ),
            )
        })
        .collect();

    Medidas { por_no, por_grupo }
}

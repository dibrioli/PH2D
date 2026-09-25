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
//!
//! ⭐⭐ **E desde o 4.º report do dono (*«neste caso o preview deveria ser colocado para cima»*)
//! ela também decide DE QUE LADO sai a moldura de cada retrato** — pela mesma lei que o painel
//! pinta ([`ph2d_panel_motion_graph::retratos_em_cima`]), lida sobre as posições em que os
//! cartões estão AGORA. É por isso que [`arrumar`] arruma **duas vezes**: o 1.º passe entrega as
//! colunas e o 2.º reserva o espaço do lado em que a moldura vai ficar.

use crate::motion_state::MotionState;
use ph2d_motion_doc::layout::{Carta, Medida};
use ph2d_nodegraph::layout::Extent;
use ph2d_panel_motion_graph::{CaixaDoCartao, caixa_de, caixa_desenhada, retratos_em_cima};
use std::collections::{BTreeMap, BTreeSet};

/// O que cada cartão deste documento MEDE, já resolvido.
///
/// ⚠️ **A igualdade é o critério de paragem de [`arrumar`]** — ver lá porquê; `Extent` é feito de
/// `f32` e a comparação é exacta de propósito: os dois lados saem da MESMA aritmética sobre as
/// MESMAS peças, logo ou são o mesmo bit ou alguma coisa mudou.
#[derive(PartialEq)]
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
///
/// ⭐⭐⭐ **Ela arruma até ao PONTO FIXO, e a razão é uma realimentação estreita:** a lei do lado
/// do retrato ([`ph2d_panel_motion_graph::retratos_em_cima`]) lê onde os cartões ESTÃO, e onde
/// eles estão depende do lado que ela escolheu (a moldura é reservada de um lado só). ⇒ a
/// paragem é a pergunta certa — *«a disposição que saiu pede exactamente os lados com que foi
/// construída?»* — e não uma contagem de passes escrita à mão.
///
/// ⛔ **DUAS passagens NÃO chegam, e isso foi MEDIDO e não inferido:** na cena que o dono
/// fotografou (`41` cartões, seis painéis) a 1.ª passagem parte da disposição AUTORADA, a 2.ª já
/// vê as colunas e a 3.ª ainda mexe um bit; a 4.ª não mexe nenhum. *Uma nota que dissesse «duas»
/// deixava um retrato desenhado do lado que ninguém reservou.*
///
/// ⚠️ **O tecto existe porque a terminação NÃO está provada** — e um tecto sem recurso nomeado é
/// um palpite, logo aqui vai o que ele de facto é: um travão contra uma oscilação que nenhuma
/// cena desta casa produz (a medida é `3`). Se alguma vez ele for alcançado, a disposição
/// continua válida — nenhum cartão se sobrepõe a outro — e o único preço é uma moldura desenhada
/// do lado que não foi reservado, que é exactamente o estado de antes desta wave.
const TECTO_DOS_PASSES: usize = 8;

/// Arruma a raiz e o interior de cada grupo, com cada cartão medido — ver [`TECTO_DOS_PASSES`]
/// para a paragem.
pub fn arrumar(motion: &mut MotionState) {
    let mut usadas: Option<Medidas> = None;
    for _ in 0..TECTO_DOS_PASSES {
        let medidas = medir(motion);
        // O ponto fixo: o que saiu da última arrumação pede os MESMOS lados com que ela foi
        // feita. Arrumar outra vez com extensões iguais devolveria as mesmas posições.
        if usadas.as_ref() == Some(&medidas) {
            return;
        }
        ph2d_motion_doc::layout::arrange(&mut motion.doc, &medidas);
        usadas = Some(medidas);
    }
}

/// As PEÇAS de um cartão de grupo — lidas uma vez, porque a caixa e a extensão precisam das
/// mesmas e *duas leituras da mesma coisa divergem no dia em que uma delas mudar*.
struct Grupo {
    nome: String,
    fileiras: f32,
}

/// Mede cada nó do grafo e cada cartão de grupo, **com o retrato já do lado em que ele fica**.
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
    // ⛔ **E a fileira do NÚMERO reserva-se sempre** (auditoria do fecho, 2026-09-24): o retrato
    // acima nasce SEM readout — ele é carimbado pelo quadro, depois de cozinhar —, e quase todo
    // cartão cozido mostra um. Medido sem ele, cada cartão sai uma fileira mais baixo do que o
    // que se pinta, e a arrumação encostava-os. Reservá-la é o lado CONSERVADOR que o cartão de
    // grupo já escolhe abaixo: sobra espaço num nó que o cozimento não puxa, nunca falta.
    for n in &mut snap.nodes {
        n.readout.get_or_insert_with(String::new);
    }

    // ⚠️ **Um cartão de GRUPO não passa por aquele retrato** — ele é derivado pela shell (o
    // título do subgrafo, os pinos que atravessam a fronteira, e o «N nós» que ele mostra em vez
    // de um readout de cozimento). ⇒ ele entra pelas portas das PEÇAS, que são a mesma
    // aritmética.
    let grupos: BTreeMap<u32, Grupo> = motion
        .doc
        .subgraphs
        .iter()
        .map(|s| {
            let portas = super::fold::card_ports(motion, s.id);
            #[expect(
                clippy::cast_precision_loss,
                reason = "a contagem de pinos de um cartao cabe num f32"
            )]
            let fileiras = portas.inputs.len().max(portas.outputs.len()).max(1) as f32;
            let nome = if s.title.is_empty() {
                super::fold::DEFAULT_TITLE.tr().to_string()
            } else {
                s.title.clone()
            };
            (s.id, Grupo { nome, fileiras })
        })
        .collect();

    let (em_cima_no, em_cima_grupo) = lados_dos_retratos(motion, &snap, &grupos);

    let por_no = snap
        .nodes
        .iter()
        .map(|n| {
            (
                n.id,
                ph2d_panel_motion_graph::extensao_desenhada(n, em_cima_no.contains(&n.id)),
            )
        })
        .collect();

    let por_grupo = grupos
        .iter()
        .map(|(&sid, g)| {
            (
                sid,
                ph2d_panel_motion_graph::extensao_de(
                    &g.nome,
                    g.fileiras,
                    0.0,
                    true, /* o «N nós» */
                    true, /* ⚠️ o retrato de um cartão é o do que sai dele, e reservá-lo é o
                          lado CONSERVADOR: sobra espaço, nunca falta. */
                    em_cima_grupo.contains(&sid),
                ),
            )
        })
        .collect();

    Medidas { por_no, por_grupo }
}

/// ⭐⭐⭐ **De que lado sai a moldura de cada retrato** — a LEI do painel, corrida **por TELA**.
///
/// ⚠️⚠️ **Por tela, e isso é a metade que interessa:** a pergunta é *«há um cartão no corredor
/// para onde este retrato ia?»*, e um nó que vive dentro de um grupo não está no corredor de
/// ninguém na raiz — ele está escondido atrás do cartão do grupo. Medir tudo junto poria dois
/// cartões de telas diferentes a decidir um pelo outro.
fn lados_dos_retratos(
    motion: &MotionState,
    snap: &ph2d_panel_motion_graph::GraphViewSnapshot,
    grupos: &BTreeMap<u32, Grupo>,
) -> (BTreeSet<u32>, BTreeSet<u32>) {
    /// Quem é a caixa: um nó do grafo ou um cartão de grupo.
    enum Chave {
        No(u32),
        Grupo(u32),
    }

    let mut por_tela: BTreeMap<Option<u32>, Vec<(Chave, CaixaDoCartao)>> = BTreeMap::new();
    for n in &snap.nodes {
        let tela = motion
            .doc
            .members
            .get(&ph2d_nodegraph::graph::NodeId(n.id))
            .copied();
        por_tela
            .entry(tela)
            .or_default()
            .push((Chave::No(n.id), caixa_desenhada(n)));
    }
    for s in &motion.doc.subgraphs {
        let Some(g) = grupos.get(&s.id) else { continue };
        por_tela.entry(s.parent).or_default().push((
            Chave::Grupo(s.id),
            caixa_de(s.x, s.y, g.fileiras, 0.0, true, true),
        ));
    }

    let (mut nos, mut cartoes) = (BTreeSet::new(), BTreeSet::new());
    for cartas in por_tela.values() {
        let caixas: Vec<CaixaDoCartao> = cartas.iter().map(|(_, c)| *c).collect();
        for ((chave, _), em_cima) in cartas.iter().zip(retratos_em_cima(&caixas)) {
            if em_cima {
                match chave {
                    Chave::No(id) => nos.insert(*id),
                    Chave::Grupo(sid) => cartoes.insert(*sid),
                };
            }
        }
    }
    (nos, cartoes)
}

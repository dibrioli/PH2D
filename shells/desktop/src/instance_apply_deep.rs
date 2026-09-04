//! ⭐⭐⭐ **A ESCADA DO *APLICAR*** (ADR-0164 / plano F5, critério 4) — *«aplicar ao mestre: a QUE
//! mestre?»*.
//!
//! # A pergunta, em palavras de artista
//!
//! Fiz uma **Roda**; fiz um **Carro** que contém uma Roda; pus um Carro na cena e mudei a cor
//! daquela roda. *Aplicar ao mestre* tem **duas** respostas legítimas:
//!
//! - ao **Carro** ⇒ todos os Carros mudam, e a receita da Roda não;
//! - à **Roda** ⇒ toda Roda em todo o lado muda.
//!
//! É por isso que o Unity oferece um submenu com **um item por nível**
//! ([pesquisa §(c)](../../docs/Components/pesquisa/instancias_2026-08-21/propagacao_unity_godot.md)),
//! e é por isso que a `PrefabUtility.ApplyPropertyOverride` dele **exige** o `assetPath`:
//! *«multiple valid targets may exist»*. ⇒ **não existe resposta por omissão**, e um verbo que
//! escolha sozinho está a adivinhar.
//!
//! # ⭐⭐ A escada é a cadeia de ELOS, e não uma estrutura nova
//!
//! Uma peça da cena diz de que peça do mestre nasceu (`InstanceOf`); essa peça, se o mestre a
//! contém por instância, diz o mesmo um nível mais fundo. ⇒ a escada **já está no mundo**:
//! percorre-se o elo até ele acabar, e cada degrau chama-se pelo `MasterRoot` que o contém
//! ([`ph2d_ecs::master_root_of`]).
//!
//! ⚠️ **Ela é da PEÇA clicada, nunca da instância inteira** — e isto não é economia, é a única
//! leitura sem ambiguidade: um Carro que contenha uma Roda **e** uma Porta tem *dois* segundos
//! degraus, e uma escada da instância teria de escolher um deles sem que ninguém o tivesse pedido.
//! Clicar na raiz dá a escada de **um** degrau (a receita directa), que é exactamente o
//! *«Apply All aplica sempre ao mais externo»* do Unity.

use ph2d_ecs::{Entity, InstanceOf, SimWorld};
use std::collections::BTreeMap;

/// ⚠️ **Quantos degraus a travessia percorre antes de desistir** — e ele não é um teto de produto,
/// é a **rede contra um ciclo**: os elos são dados do ficheiro, e um `.ph2dproj` corrompido (ou uma
/// futura troca de mestre mal validada) pode fechar um anel que faria esta função rodar para
/// sempre. A recusa de ciclo do plano F5 vive no gesto que CRIA o elo; aqui basta não pendurar.
///
/// ⛔ Não é o limite de aninhamento do produto: aninhar 16 níveis de receita é uma cena que
/// ninguém desenhou, e se alguém a desenhar o degrau 17 fica **fora da escada**, nunca perdido —
/// o *Aplicar* ao mais externo continua a ser o que sempre foi.
const MAX_CHAIN: usize = 16;

/// ⭐ **Um degrau da escada** — a receita que receberia, com o nome que o artista lê.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ApplyLevel {
    /// O `StableId` do [`ph2d_ecs::MasterRoot`] que recebe — **a identidade**, que é o que o gesto
    /// precisa de saber.
    pub(crate) master: u64,
    /// O `Name` dele, que é o que a Hierarquia mostra.
    pub(crate) name: String,
}

/// ⭐⭐ **A CADEIA DE PEÇAS que uma chave de override pode descer**, da mais externa para a mais
/// interna.
///
/// `chain[0]` é a peça que a chave nomeia — a do mestre directo. `chain[k + 1]` é a peça de que
/// **ela** é cópia, um nível mais fundo. A cadeia acaba quando um elo deixa de resolver, que é o
/// caso normal: a peça de uma receita que não contém instância nenhuma não tem `InstanceOf`.
///
/// ⚠️ **Ela devolve `StableId`, e não entidades** — os bits mudam a cada respawn do undo, e esta
/// cadeia é lida por um gesto que corre depois de o painel a ter mostrado.
pub(crate) fn piece_chain(
    sim: &SimWorld,
    by_id: &BTreeMap<u64, Entity>,
    first_piece: u64,
) -> Vec<u64> {
    let mut out = Vec::new();
    let mut id = first_piece;
    for _ in 0..MAX_CHAIN {
        out.push(id);
        let Some(&e) = by_id.get(&id) else { break };
        let Some(link) = sim.world().get::<InstanceOf>(e) else {
            break;
        };
        // ⚠️ **Um elo que aponta para si próprio ou para trás fecharia o anel** — a rede do
        // [`MAX_CHAIN`] já o apara, e sair aqui deixa a cadeia com o prefixo honesto.
        if out.contains(&link.master) {
            break;
        }
        id = link.master;
    }
    out
}

/// ⭐⭐⭐ **A ESCADA da peça clicada** — as receitas que o *Aplicar* pode alcançar, **da mais
/// externa para a mais interna**.
///
/// Vazia quando a entidade não é peça de cópia nenhuma. Com **um** degrau é o caso comum (uma
/// instância não aninhada), e aí a escada não é uma escolha: é o *Aplicar ao mestre* de sempre.
///
/// ⚠️ **O sujeito é a peça, e a chave de override dela é `(peça do mestre, tipo)`** — por isso a
/// cadeia começa no `InstanceOf` da própria entidade clicada, que é exactamente a `piece` das
/// chaves que aquele clique tem no escopo.
pub(crate) fn apply_levels(sim: &mut SimWorld, clicked: Entity) -> Vec<ApplyLevel> {
    let Some(link) = sim.world().get::<InstanceOf>(clicked).copied() else {
        return Vec::new();
    };
    let by_id = crate::instance_verbs::stable_index(sim);
    let chain = piece_chain(sim, &by_id, link.master);
    let mut out: Vec<ApplyLevel> = Vec::new();
    for piece in chain {
        let Some(id) = owner_of(sim, &by_id, piece) else {
            continue;
        };
        // ⚠️ **Duas peças do mesmo mestre dão UM degrau** — a escada é de receitas, não de peças.
        if out.iter().any(|l| l.master == id) {
            continue;
        }
        let name =
            crate::instance_verbs::master_named(sim, id).unwrap_or_else(|| "component".to_string());
        out.push(ApplyLevel { master: id, name });
    }
    out
}

/// **A RECEITA que contém esta peça** — o `StableId` do [`ph2d_ecs::MasterRoot`] acima dela.
///
/// ⚠️ **Uma porta, e não a escada escrita duas vezes**: a que MOSTRA os degraus e a que APLICA num
/// deles fazem a mesma pergunta, e duas travessias divergiriam no dia em que uma delas aprendesse
/// a lidar com um caso novo.
fn owner_of(sim: &SimWorld, by_id: &BTreeMap<u64, Entity>, piece: u64) -> Option<u64> {
    let &e = by_id.get(&piece)?;
    let owner = ph2d_ecs::master_root_of(sim.world(), e)?;
    sim.world().get::<ph2d_ecs::StableId>(owner).map(|s| s.0)
}

/// **O que o *Aplicar* fez** — e as duas metades são precisas, porque um `0` tem duas causas.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Applied {
    /// Quantos componentes foram escritos na receita escolhida.
    pub(crate) changed: usize,
    /// Quantas excepções ficaram onde estavam porque a **escada delas** não alcança aquela
    /// receita — a porta do Carro não é peça de Roda nenhuma.
    ///
    /// ⛔ **Elas não se aplicam ao degrau mais fundo que houver**, e a recusa é a decisão: aplicar
    /// uma excepção a uma receita que o artista não nomeou é escolher por ele, que é exactamente
    /// o que a escada existe para não fazer. Quem tem UI diz-lhe quantas ficaram.
    pub(crate) left: usize,
}

/// ⭐⭐⭐ **APLICAR num DEGRAU da escada** — a porta única do *Apply* (ADR-0164 / F5 critério 4).
///
/// `target` é o `StableId` da receita escolhida. O escopo é o que se clicou (uma peça ⇒ as chaves
/// dela; a raiz ⇒ todas), pela razão do *Revert*.
///
/// # ⭐⭐⭐ A metade que APAGA a excepção intermédia NÃO é opcional — e está MEDIDA
///
/// Com a Roda dentro do Carro e um Carro na cena, uma excepção guardada na cópia da Roda que vive
/// **dentro** do Carro **bloqueia** o passe: medido (`an_override_in_the_middle_blocks_the_inner_master`)
/// — mexer na receita da Roda deixa a cópia dentro do Carro exactamente como estava, porque a
/// resposta (1) do passe é *«a instância possui este componente ⇒ não se toca»*.
///
/// ⇒ escrever só na Roda e limpar só a excepção da cena produziria o **no-op visível** que a regra
/// do Unity nomeia: *«this override in the 'Table' Prefab is reverted at the same time so that the
/// property on the instance retains the value that was just applied. **If this was not the case,
/// the value on the instance would change right after being applied.**»*
///
/// ⇒ este verbo **limpa a chave em TODOS os degraus** entre quem foi clicado e a receita escolhida.
///
/// # ⚠️ E ele escreve o valor em TODOS os degraus intermédios, de propósito
///
/// Limpar a chave e deixar o passe propagar seria o desenho mais magro — e o passe **não corre em
/// ordem topológica**: ele ordena as instâncias por `StableId`, e a ordem de criação só *coincide*
/// com a de dependência (medido na F5.1). Uma instância exterior avaliada **antes** da interior
/// veria *«o mestre não mexeu e eu mexi»*, que é a assinatura de uma excepção NOVA — e ela
/// bloquearia para sempre o valor que o gesto acabou de aplicar.
///
/// ⇒ escrever o valor em cada degrau **não é uma segunda fonte**: são os mesmos bytes, e o passe
/// seguinte encontra tudo igual e não faz nada. *O que se compra é a ordem, não o valor.*
///
/// # ⚠️ A ordem é: escrever, DEPOIS limpar a chave
///
/// Ao contrário, um passe que corresse no meio veria a instância sem excepção e diferente da
/// receita, e achataria a edição que o gesto existe para promover.
pub(crate) fn apply_to_level(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    echo: &mut crate::instance_sync::MasterEcho,
    clicked: Entity,
    target: u64,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
) -> Result<Applied, crate::instance_verbs::VerbRefusal> {
    let root = crate::instance_verbs::instance_root_of(sim, clicked)
        .ok_or(crate::instance_verbs::VerbRefusal::NotAnInstance)?;
    let scope = (root != clicked)
        .then(|| sim.world().get::<InstanceOf>(clicked).map(|l| l.master))
        .flatten();
    let by_id = crate::instance_verbs::stable_index(sim);
    let keys: Vec<ph2d_ecs::OverrideKey> = sim
        .world()
        .get::<ph2d_ecs::ObjectInstance>(root)
        .map(|o| {
            o.overrides
                .iter()
                .copied()
                .filter(|k| scope.is_none_or(|piece| k.piece == piece))
                .collect()
        })
        .unwrap_or_default();
    if keys.is_empty() {
        return Ok(Applied::default());
    }
    // De que entidade da instância veio cada peça do mestre.
    let mine: BTreeMap<u64, Entity> = crate::instance_verbs::subtree(sim, root)
        .into_iter()
        .filter_map(|e| sim.world().get::<InstanceOf>(e).map(|l| (l.master, e)))
        .collect();

    let mut out = Applied::default();
    for key in keys {
        let Some(&inst_piece) = mine.get(&key.piece) else {
            continue;
        };
        let chain = piece_chain(sim, &by_id, key.piece);
        // ⚠️ **O degrau é procurado pelo DONO, não pela posição** — a mesma leitura que a escada
        // mostrou ao artista.
        let Some(depth) = chain
            .iter()
            .position(|&p| owner_of(sim, &by_id, p) == Some(target))
        else {
            out.left += 1;
            continue;
        };
        if write_down_the_chain(sim, registry, docs, inst_piece, &by_id, &chain[..=depth], key) {
            out.changed += 1;
        }
        // ⭐⭐⭐ A metade não-opcional: a chave sai em TODOS os degraus até ao escolhido. O
        // detentor do degrau `k` é a raiz da instância a que a peça de CIMA pertence.
        let mut holder = inst_piece;
        for &piece in &chain[..=depth] {
            if let Some(hroot) = crate::instance_verbs::instance_root_of(sim, holder) {
                crate::instance_sync::revert_override(
                    sim,
                    echo,
                    hroot,
                    ph2d_ecs::OverrideKey {
                        piece,
                        type_id: key.type_id,
                    },
                );
            }
            let Some(&next) = by_id.get(&piece) else { break };
            holder = next;
        }
    }
    Ok(out)
}

/// Escreve o valor da peça da instância em cada degrau da cadeia. Devolve `true` se algum degrau
/// aceitou.
///
/// ⭐⭐ **Um DOCUMENTO aplica-se por CONTEÚDO** (F4.6b), e não pelos bytes do componente: o
/// `insert_from_bytes` escreveria o **id** do `VecPathRef` da instância no mestre, e as duas
/// passariam a apontar para o mesmo path — editar uma mexeria na outra.
fn write_down_the_chain(
    sim: &mut SimWorld,
    registry: &ph2d_ecs::scene::ComponentRegistry,
    docs: &mut crate::instance_docs::OwnedDocs<'_>,
    inst_piece: Entity,
    by_id: &BTreeMap<u64, Entity>,
    chain: &[u64],
    key: ph2d_ecs::OverrideKey,
) -> bool {
    let is_doc =
        key.type_id == ph2d_ecs::scene::stable_type_id(crate::instance_sync_docs::VEC_PATH);
    let entry = (!is_doc).then(|| registry.get_by_id(key.type_id)).flatten();
    if !is_doc && entry.is_none() {
        return false;
    }
    let mut any = false;
    for piece in chain {
        let Some(&target_piece) = by_id.get(piece) else {
            continue;
        };
        if is_doc {
            any |= crate::instance_sync_docs::apply_one(sim, docs, inst_piece, target_piece);
            continue;
        }
        let entry = entry.expect("o ramo sem documento tem entrada de registo");
        match (entry.serialize)(sim.world(), inst_piece).unwrap_or_default() {
            Some(bytes) => {
                any |= (entry.insert_from_bytes)(sim.world_mut(), target_piece, &bytes).is_ok();
            }
            // ⚠️ A ausência também é uma excepção: o artista **tirou** o componente da cópia, e
            // aplicar isso é tirá-lo da receita.
            None => {
                (entry.remove)(sim.world_mut(), target_piece);
                any = true;
            }
        }
    }
    any
}

#[cfg(test)]
#[path = "instance_apply_deep_tests.rs"]
mod tests;

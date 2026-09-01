//! ⭐⭐⭐ **AS CHAVES NO NOME DE UMA CÓPIA PASSAM A VALER** — decisão do Enio, 2026-08-31.
//!
//! > *«Renomear o Botão Funcionou! Mas por que não funciona mudando o nome entre as chaves?
//! > Tem que funcionar!»*
//!
//! # ⚠️ A objecção foi levantada, ele reafirmou, e a decisão é dele
//!
//! O modelo diz que uma propriedade é do **componente**, e por isso as chaves no nome de uma cópia
//! eram inertes — quatro reports seguidos bateram nisso. A objecção é real (autorar a partir de uma
//! cópia escreve estado **partilhado**), foi-lhe dita, e ele decidiu: *tem que funcionar*.
//!
//! ⭐ **E ela é a MESMA lei que ele acabou de validar no cartão**, dita por outra porta:
//!
//! | o que ele escreve | o que acontece | o gémeo no cartão |
//! |---|---|---|
//! | um valor que a família **já tem** | a cópia **troca** para essa versão | clicar num chip apagado |
//! | um valor **novo** | a receita vigente passa a chamar-se assim | clicar no chip aceso e escrever |
//!
//! *Duas portas, uma lei* — e é por isso que isto vive aqui e não dentro de nenhuma das duas.
//!
//! # ⭐⭐⭐ O NOME É A ÚNICA VERDADE (report do Enio com duas fotos, 2026-08-31)
//!
//! > *«Entenda uma Coisa: O objeto deve ler o que está nas Chaves. não tem porque está Small e o
//! > Botão ficar Big»*
//!
//! Ele fotografou o estado em que as **duas** verdades da tela discordavam: o nome dizia
//! `Canvas{Size=Small} (1)` e o botão aceso dizia `Big`. E discordavam porque o elo (`InstanceOf`)
//! e as chaves do nome eram fontes **independentes** — o elo mudava por clique, o nome mudava por
//! escrita, e nada os obrigava a concordar.
//!
//! ⇒ **as chaves passam a ser a fonte, e tudo o resto SEGUE-AS.** Um objecto lê-se pelo nome.
//!
//! | quem muda | o que ele também faz |
//! |---|---|
//! | as chaves do nome | o elo segue-as ([`follow`]) |
//! | um clique no chip | **reescreve as chaves**, e daí o elo segue |
//! | renomear o valor de uma receita | reescreve as chaves de **todas** as cópias que a seguem |
//!
//! ⛔⛔ **Sem a 2.ª linha desta tabela isto seria uma BRIGA:** o clique trocava o elo, o
//! [`follow`] via as chaves antigas e trocava de volta — todo quadro. *Uma fonte única só é única
//! se TODO gesto escrever nela.*
//!
//! # ⛔⛔ E isto NÃO corre por tecla
//!
//! O campo de nome do Inspector publica a cada `TextChanged`. Aplicar aqui por tecla renomearia a
//! receita para `B`, `Bi`, `Big` — três passos de undo e três avisos por uma palavra. ⇒ as duas
//! portas que chamam isto são de **commit**: o `Enter` da Hierarquia e o `Submit`/`Blur` do campo.

use ph2d_ecs::{Entity, SimWorld};
use ph2d_editor::screens::hero::variant_axes;

/// O que o gesto fez, para a voz do app.
pub(crate) enum Applied {
    /// A cópia passou a seguir outra receita, que já declarava isto.
    ///
    /// ⚠️ **Sem carga, e o clippy é que o disse:** eu tinha posto lá o `StableId` e ninguém o lia.
    /// *O gate que prova a troca pergunta ao ELO (`InstanceOf::master`), que é o oráculo forte —
    /// um id devolvido pela própria função sob teste provaria menos.*
    Switched,
    /// A receita vigente passou a chamar o valor assim.
    Authored { key: String, value: String },
}

/// ⭐⭐ **Aplica as chaves que o nome de `entity` declara.**
///
/// `None` quando não há nada a fazer — e são muitos casos, todos legítimos: o objecto não é cópia
/// (aí o nome dele **já é** a declaração e o cartão lê-o directamente), o nome não declara nada, ou
/// declara exactamente o que a receita vigente já diz.
///
/// ⚠️ **A troca é preferida à autoria**, e a ordem não é arbitrária: se o valor que ele escreveu já
/// existe na família, ele quer *aquela versão* — autorar por cima criaria uma segunda receita a
/// dizer o mesmo, que é o estado que colapsa o eixo.
/// ⭐⭐⭐ **O elo SEGUE as chaves** — a metade que corre a cada quadro sobre o selecionado.
///
/// ⚠️ **Só TROCA; nunca autora.** Autorar aqui renomearia a receita a cada quadro enquanto o nome
/// declarasse um valor que não existe — mesmo nome, mesmo aviso, sessenta vezes por segundo. *O que
/// corre sempre tem de ser idempotente, e só a troca o é.*
///
/// `true` quando trocou. Ver o cabeçalho para a razão de ele existir.
pub(crate) fn follow(
    sim: &mut SimWorld,
    echo: &mut crate::instance_sync::MasterEcho,
    entity: Entity,
) -> bool {
    let Some((root, master_id, declared, mine, members)) = read(sim, entity) else {
        return false;
    };
    if mine == declared {
        return false;
    }
    let Some((id, _)) = members.iter().find(|(id, n)| {
        *id != master_id && variant_axes::parse_combo(n).as_ref() == Some(&declared)
    }) else {
        return false;
    };
    let id = *id;
    // ⚠️ **Pela PORTA do `swap`, e por mais nenhuma** — é ela que faz o re-key determinístico das
    // excepções, sepulta os órfãos e esquece o eco.
    crate::instance_variant::swap(sim, echo, root, id).is_ok()
}

/// O que as duas metades precisam de saber: a raiz, a receita vigente, o que o NOME declara, o que
/// a RECEITA declara, e a família.
///
/// ⚠️ Uma porta, porque ler isto duas vezes com condições ligeiramente diferentes é como o elo e o
/// nome se separaram em primeiro lugar.
type Read = (
    Entity,
    u64,
    Vec<(String, String)>,
    Vec<(String, String)>,
    Vec<(u64, String)>,
);
fn read(sim: &mut SimWorld, entity: Entity) -> Option<Read> {
    let declared = sim
        .world()
        .get::<ph2d_ecs::Name>(entity)
        .and_then(|n| variant_axes::parse_combo(&n.0))?;
    let root = crate::instance_verbs::instance_root_of(sim, entity)?;
    let master_id = sim
        .world()
        .get::<ph2d_ecs::InstanceOf>(root)
        .map(|l| l.master)?;
    let members = crate::render_loop::inspector_instance::family_members(sim, master_id);
    let mine = members
        .iter()
        .find(|(id, _)| *id == master_id)
        .and_then(|(_, n)| variant_axes::parse_combo(n))?;
    Some((root, master_id, declared, mine, members))
}

pub(crate) fn apply(
    sim: &mut SimWorld,
    // ⚠️ **O eco entra porque a TROCA tem de o esquecer** — senão o passe seguinte lê a diferença
    // contra o mestre novo como *«a instância mexeu-se»* e congela a cópia com o valor do velho.
    // É a mesma razão pela qual o dreno do chip adia a troca até ter o eco à mão.
    echo: &mut crate::instance_sync::MasterEcho,
    entity: Entity,
) -> Option<Applied> {
    // 1) Alguém na família já é isto? ⇒ troca. (A metade idempotente vive no [`follow`].)
    if follow(sim, echo, entity) {
        return Some(Applied::Switched);
    }
    let (_root, master_id, declared, mine, _members) = read(sim, entity)?;
    if mine == declared {
        return None;
    }
    // 2) Senão, a receita vigente passa a dizê-lo. ⚠️ **Só as chaves que ela TEM** — uma chave nova
    // mudaria a forma da família, e isso é outro gesto (renomear a receita).
    let target =
        crate::instance_verbs_walk::entity_for_stable_id(sim, master_id).map(Entity::from_bits)?;
    let mut name = sim.world().get::<ph2d_ecs::Name>(target)?.0.clone();
    let mut done: Option<(String, String)> = None;
    for (k, v) in &declared {
        if mine.iter().any(|(mk, mv)| mk == k && mv != v)
            && let Some(next) = variant_axes::with_value(&name, k, v)
        {
            name = next;
            done = Some((k.clone(), v.clone()));
        }
    }
    let (key, value) = done?;
    sim.world_mut()
        .entity_mut(target)
        .insert(ph2d_ecs::Name::new(name));
    Some(Applied::Authored { key, value })
}

/// ⭐⭐⭐ **As chaves da CÓPIA passam a dizer o que ela agora É** — a metade sem a qual a fonte
/// única seria uma briga.
///
/// Chamada **depois** de uma troca (um clique num chip, ou o dreno do `swap`): sem ela o
/// [`follow`] veria as chaves antigas no quadro seguinte e trocaria de volta, todo quadro.
///
/// ⚠️ **Só as chaves; o resto do nome é do artista** — o nome comum e o sufixo de cópia ficam
/// (`Canvas{Size=Small} (1)` → `Canvas{Size=Big} (1)`).
///
/// ⛔ **Silenciosa por construção:** ela não é um gesto, é a metade escrita de um gesto que já
/// falou. Um aviso aqui diria duas coisas sobre um clique.
pub(crate) fn mirror_onto_copy(sim: &mut SimWorld, entity: Entity) {
    let Some(root) = crate::instance_verbs::instance_root_of(sim, entity) else {
        return;
    };
    let Some(master_id) = sim
        .world()
        .get::<ph2d_ecs::InstanceOf>(root)
        .map(|l| l.master)
    else {
        return;
    };
    let Some(master) =
        crate::instance_verbs_walk::entity_for_stable_id(sim, master_id).map(Entity::from_bits)
    else {
        return;
    };
    let Some(combo) = sim
        .world()
        .get::<ph2d_ecs::Name>(master)
        .and_then(|n| variant_axes::parse_combo(&n.0))
    else {
        return;
    };
    write_combo(sim, root, &combo);
}

/// ⭐⭐ **Renomear o valor de uma receita arrasta as CÓPIAS dela.**
///
/// Sem isto, `Small 2` → `Big` deixa toda cópia com `{Size=Small 2}` no nome: um rótulo a apontar
/// para um valor que já não existe, e o [`follow`] não o pode curar (não há a quem trocar). *A
/// etiqueta mentiria para sempre, e foi exactamente isso que as duas fotos mostraram.*
///
/// ⚠️ **Ela varre as cópias DAQUELA receita, e não a cena** — quem segue outra não tem nada a ver
/// com esta renomeação.
pub(crate) fn mirror_onto_copies_of(sim: &mut SimWorld, master_id: u64) {
    let Some(master) =
        crate::instance_verbs_walk::entity_for_stable_id(sim, master_id).map(Entity::from_bits)
    else {
        return;
    };
    let Some(combo) = sim
        .world()
        .get::<ph2d_ecs::Name>(master)
        .and_then(|n| variant_axes::parse_combo(&n.0))
    else {
        return;
    };
    let roots: Vec<Entity> = {
        let mut q = sim.world_mut().query::<(Entity, &ph2d_ecs::InstanceOf)>();
        q.iter(sim.world())
            .filter(|(_, l)| l.master == master_id)
            .map(|(e, _)| e)
            .collect()
    };
    for root in roots {
        write_combo(sim, root, &combo);
    }
}

/// Escreve `combo` nas chaves do nome de `root`, deixando o resto intacto.
///
/// ⚠️ **Nada é escrito se o nome já diz isto** — um `insert` por quadro sobre o mesmo valor é um
/// passo de undo por quadro ([`crate::undo`] regista por DIFF, e um `Name` reescrito com os mesmos
/// bytes não move o diff; mas o `insert` continua a custar, e a intenção tem de estar escrita).
fn write_combo(sim: &mut SimWorld, root: Entity, combo: &[(String, String)]) {
    let Some(name) = sim.world().get::<ph2d_ecs::Name>(root).map(|n| n.0.clone()) else {
        return;
    };
    let mut next = name.clone();
    for (k, v) in combo {
        if let Some(n) = variant_axes::with_value(&next, k, v) {
            next = n;
        }
    }
    if next != name {
        sim.world_mut()
            .entity_mut(root)
            .insert(ph2d_ecs::Name::new(next));
    }
}

/// ⭐ **A voz** — uma porta, porque as DUAS entradas (o `Enter` da Hierarquia e o campo do
/// Inspector) dizem a mesma coisa, e duas frases divergiriam no dia em que uma mudasse.
///
/// ⚠️ **Silêncio quando não houve nada a fazer**, que é o caso comum: renomear um objecto que não
/// é cópia, ou que não declara chaves, não é um evento.
pub(crate) fn speak(applied: Option<Applied>, toasts: &mut ph2d_editor::ToastQueue) {
    match applied {
        None => {}
        Some(Applied::Switched) => {
            toasts.push(ph2d_editor::Toast::success("Switched to that version"));
        }
        Some(Applied::Authored { key, value }) => {
            toasts.push(ph2d_editor::Toast::success(format!(
                "{key} \u{2192} {value}"
            )));
        }
    }
}

#[cfg(test)]
#[path = "instance_declared_value_tests.rs"]
mod tests;

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
//! # ⛔ O nome da CÓPIA fica como ele o escreveu
//!
//! Ela é a etiqueta dele; reescrevê-la para acompanhar uma troca seria o app a corrigir o que o
//! artista acabou de digitar.
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
pub(crate) fn apply(
    sim: &mut SimWorld,
    // ⚠️ **O eco entra porque a TROCA tem de o esquecer** — senão o passe seguinte lê a diferença
    // contra o mestre novo como *«a instância mexeu-se»* e congela a cópia com o valor do velho.
    // É a mesma razão pela qual o dreno do chip adia a troca até ter o eco à mão.
    echo: &mut crate::instance_sync::MasterEcho,
    entity: Entity,
) -> Option<Applied> {
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
    if mine == declared {
        return None;
    }
    // 1) Alguém na família já é isto? ⇒ troca.
    if let Some((id, _)) = members.iter().find(|(id, n)| {
        *id != master_id && variant_axes::parse_combo(n).as_ref() == Some(&declared)
    }) {
        let id = *id;
        // ⚠️ **Pela PORTA do `swap`, e por mais nenhuma** — é ela que faz o re-key determinístico
        // das excepções, sepulta os órfãos e esquece o eco. Escrever o `InstanceOf` à mão aqui
        // seria a segunda escrita que o gate `the_inspector_never_writes_the_master_link_by_hand`
        // existe para proibir, um andar abaixo.
        return crate::instance_variant::swap(sim, echo, root, id)
            .ok()
            .map(|_| Applied::Switched);
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

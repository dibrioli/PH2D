//! ⭐⭐⭐ **ABRIR A RECEITA a partir de uma CÓPIA dela** — a lei do [`crate::instance_verbs::Verb::Edit`]
//! (2026-09-07).
//!
//! # Porque isto é um módulo, e não mais um braço do dreno
//!
//! O corte é o do resto da família: cada verbo com lei própria já vive num irmão
//! ([`crate::instance_unmake`], [`crate::instance_revert`], [`crate::instance_variant`]), e o
//! `instance_verbs` é o **roteador** que os chama. ⚠️ Escrever a lei lá dentro empurrou aquele
//! ficheiro para `603` linhas contra o tecto de `600` — e a lei da casa é **decompor por
//! responsabilidade, nunca subir a allowlist**.
//!
//! # O que o verbo faz, e o que ele NÃO faz
//!
//! Uma receita é `MasterPiece` sem `MasterEditing`, logo o `off_canvas` esconde-a da cena **e** da
//! Hierarquia. O único caminho para ela era o *Edit Prefab* do **cartão** do navegador de assets —
//! que exige saber o nome dela e ter aquele painel aberto. ⚠️ **E três recusas deste app já mandavam
//! o artista *«editar no prefab»***, o que faz da ausência deste verbo um buraco que o próprio app
//! nomeava.
//!
//! ⭐ Ele **selecciona, e mais nada**: o `MasterEditing` é derivado da selecção
//! ([`crate::master_editing`]), então pôr a selecção na raiz do mestre acende o canvas,
//! arma o gizmo, enche o Inspector, e cada peça mexida chega a todas as cópias no mesmo quadro.
//! *O verbo que faltava não era um modo nem uma janela: era um acesso.*

use ph2d_ecs::{Entity, MasterRoot, SimWorld, StableId};
use ph2d_editor::Toast;

/// **Abre a receita da cópia `entity`.** Devolve `true` quando o DOCUMENTO mudou — e ele nunca muda:
/// ver o cabeçalho e a nota do `select_out` abaixo.
pub fn open_prefab(
    sim: &mut SimWorld,
    entity: Entity,
    toasts: &mut ph2d_editor::ToastQueue,
    select_out: &mut Option<u64>,
) -> bool {
    // ⚠️ **O sujeito resolve-se pela MESMA porta dos outros verbos** (`master_subject`): clicar numa
    // peça de dentro da cópia é o gesto mais provável, e ela resolve a raiz sozinha. ⛔ E uma linha
    // que já É a receita não é recusada — abrir o que já está aberto é o que o artista quer quando
    // duvida de onde está.
    let subject = crate::instance_verbs_walk::master_subject(sim, entity);
    let Some(id) = sim
        .world()
        .get::<StableId>(subject)
        .map(|s| s.0)
        .filter(|_| sim.world().get::<MasterRoot>(subject).is_some())
    else {
        toasts.push(Toast::warning(
            "That is not a copy of a prefab \u{2014} pick one, or the prefab row",
        ));
        return false;
    };
    *select_out = Some(subject.to_bits());
    let name = crate::instance_verbs::master_named(sim, id).unwrap_or_else(|| "prefab".to_string());
    toasts.push(Toast::success(format!(
        "Editing \u{201c}{name}\u{201d} \u{2014} move a piece and every copy follows"
    )));
    // ⚠️ **`false`, e não `true`:** seleccionar não é editar. Devolver `true` poria um passo de undo
    // sobre um gesto de *ver* — a mesma lei do *Select users* do cartão da biblioteca.
    false
}

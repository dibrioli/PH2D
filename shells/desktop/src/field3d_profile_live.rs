//! ⭐⭐ **O CONTORNO CONTINUA A SER A FONTE** (W55) — a peça segue o desenho, e a resolução é um
//! número.
//!
//! # As duas coisas que faltavam, e eram uma
//!
//! A W53 fez o desenho virar peça: cozia o contorno **uma vez** e o resultado era tudo o que
//! sobrava. Duas consequências, com a mesma causa:
//!
//! | o que o artista via | a causa |
//! |---|---|
//! | editar a curva não muda a peça | a peça já não conhece o desenho |
//! | não há knob de resolução | afinar a conversão exige ter a fonte para reconverter |
//!
//! Enio, no smoke da W54: *"contudo sem ajustes de resolução"*. O knob **não era uma linha de
//! painel a faltar** — era inexprimível sem o vínculo. É por isso que os dois vêm na mesma wave: o
//! [`ph2d_field_ecs::FieldProfileSource`] é o que os torna possíveis, e um sem o outro seria metade
//! de uma feature.
//!
//! # ⭐ Não há cache, e é de propósito
//!
//! Esta função **recoze e compara**, todo quadro. A alternativa óbvia — guardar um resumo do que já
//! foi cozido e só reconverter quando ele mudar — pede um sítio para o guardar, e os dois sítios
//! possíveis são maus: num **componente** ele é estado derivado a viajar no save e a envenenar o
//! undo (o `canonicalize` ordena por bytes, e um resumo que mudasse a cada quadro faria todo quadro
//! virar um passo); numa **tabela lateral** ele é indexado por bits de entidade, que morrem em cada
//! desfazer.
//!
//! ⚠️ **E medido, o cache não compra nada** (sonda `the_table_that_chose_the_resolution_ceiling`):
//! recozer um contorno custa **6–13 µs** e comparar o resultado **0,2–0,4 µs**, contra um quadro de
//! 16,7 ms e um traçado assente de dezenas de milissegundos. *Um cache que poupa 0,04 % de um quadro
//! e paga com um bug de undo é um mau negócio.*
//!
//! # ⚠️ Ele segue a FORMA, nunca a POSE
//!
//! O desenho tem pose própria no canvas 2D e a peça tem a dela no espaço 3D — posta pelo artista,
//! com o gizmo. Arrastar a curva **não** teleporta a peça; mudar a curva muda a peça. E o
//! **tamanho de convivência** também não se mexe: ele entrou na pose no nascimento (o enquadramento
//! da W53) e é do artista desde então. *Uma pose, um dono.*

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_field::{NodeShape, Primitive};
use ph2d_field_ecs::{FieldNode, FieldProfileSource};
use ph2d_vec_scene::VecScene;

/// ⭐ **Reconcilia as formas vivas com o desenho**, e devolve o que houver a dizer.
///
/// ⚠️ Ela corre **antes** do cozimento do quadro (ver `field3d_scene::sync_scene_and_birth`): o
/// documento que o traçador recebe tem de ser o do desenho de agora, senão a peça fica um quadro
/// atrás da mão que a edita.
pub(crate) fn reconcile(world: &mut World, scene: &VecScene) -> Vec<String> {
    let mut q = world.query::<(Entity, &FieldProfileSource)>();
    // Colhido primeiro: a escrita a seguir toma o mundo emprestado, e a consulta não pode estar
    // viva enquanto isso.
    let links: Vec<(Entity, FieldProfileSource)> = q.iter(world).map(|(e, s)| (e, *s)).collect();
    let mut said = Vec::new();
    for (entity, link) in links {
        let Some(path) = scene.path(link.path) else {
            if note_missing(link.path) {
                // ⚠️ **A forma FICA.** Largar o vínculo seria a resposta fácil e a errada: um desenho
                // apagado volta com um desfazer, e um vínculo largado não volta com nada. A peça
                // guarda a última forma que teve, que é o que o artista tem na tela.
                said.push(
                    "The drawing this shape came from is gone - the shape keeps its last form"
                        .to_string(),
                );
            }
            continue;
        };
        note_found(link.path);
        let fresh = match ph2d_field_profile::cook_path_at(path, link.level) {
            Ok(p) => p,
            // ⚠️ Traduzido, como no botão que criou a peça — a mesma porta, para o artista não ler
            // duas frases diferentes sobre o mesmo contorno impossível.
            Err(e) => {
                said.push(crate::field3d_profile::explain(&e));
                continue;
            }
        };
        if same_profile(world, entity, &fresh) {
            continue;
        }
        // ⚠️ **Só se escreve quando MUDOU**, e a razão não é velocidade: `get_mut` num componente é
        // uma escrita para o mundo, e o retrato do undo compara **bytes** — reescrever o mesmo
        // perfil todo quadro é a receita conhecida de transformar cada quadro num passo de
        // histórico. É a mesma lei que o `s.doc != cooked` da ponte já segue.
        let Some(mut node) = world.get_mut::<FieldNode>(entity) else {
            continue;
        };
        if let NodeShape::Leaf(
            Primitive::Extrude { profile, .. } | Primitive::Revolve { profile },
        ) = &mut node.shape
        {
            *profile = fresh;
        }
    }
    said
}

/// O perfil que este nó tem agora **é** o que o desenho dá?
///
/// `true` também quando o nó não é de perfil nenhum — não há nada a reconciliar, e a escrita a
/// seguir não teria onde cair.
fn same_profile(world: &World, entity: Entity, fresh: &ph2d_field::Profile) -> bool {
    match world.get::<FieldNode>(entity).map(|n| &n.shape) {
        Some(NodeShape::Leaf(
            Primitive::Extrude { profile, .. } | Primitive::Revolve { profile },
        )) => profile == fresh,
        _ => true,
    }
}

/// `true` na **primeira** vez que este desenho é dado por desaparecido.
///
/// ⚠️ Ele existe porque a voz do módulo é limpa a cada cozimento bem-sucedido
/// (`field3d_notice::clear`, para que um problema corrigido e recriado volte a ser dito) — e a peça
/// **cozinha bem** com o desenho ausente, porque ela guarda a última forma. Sem esta memória a mesma
/// frase sairia sessenta vezes por segundo, que é o oposto de avisar.
fn note_missing(path: u64) -> bool {
    MISSING.with(|m| m.borrow_mut().insert(path))
}

/// ⭐ **O desenho voltou** — e a queixa seguinte volta a ser dita.
///
/// ⚠️ Sem isto, um desenho apagado e trazido de volta por um desfazer ficaria **mudo** se voltasse a
/// desaparecer. É a mesma lei (e o mesmo modo de falha) do `forget_tried` do `field3d_reload`.
fn note_found(path: u64) {
    MISSING.with(|m| {
        m.borrow_mut().remove(&path);
    });
}

thread_local! {
    /// Os desenhos já dados por desaparecidos.
    ///
    /// ⚠️ **`BTreeSet` e não `HashSet`**: `HashMap`/`HashSet` são tipos proibidos neste repositório
    /// (lint estrutural do determinismo, HR-5).
    static MISSING: std::cell::RefCell<std::collections::BTreeSet<u64>> =
        const { std::cell::RefCell::new(std::collections::BTreeSet::new()) };
}

#[cfg(test)]
#[path = "field3d_profile_live_tests.rs"]
mod tests;

//! ⭐⭐ **A PALAVRA que o valor de um canal discreto mostra** — a porta única da coluna do valor da
//! timeline, do gráfico e do valor de cada chave (F27 F; [plano](../../../docs/UI_New_and_Simple/F27_F_plano.md)).
//!
//! Um canal é `f32`; uma ESCOLHA vale o índice dela, uma camada o `LayerId`, um script o
//! `ScriptId`. Quem o pinta pergunta aqui pela [`Especie`] do campo: uma variante devolve a CHAVE
//! `ph2d-i18n` do rótulo que o painel já mostra (esta crate não traduz), uma camada, um script ou um
//! texto devolvem o NOME do artista, que não se traduz. `None` = o número é a resposta.

use std::borrow::Cow;
use std::collections::BTreeMap;

use bevy_ecs::resource::Resource;
use bevy_ecs::world::World;

use crate::canal::{CanaisLidos, Especie, FieldKey, TextosDosCanais};

/// O que um valor discreto diz.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Palavra {
    /// A chave `ph2d-i18n` — quem pinta traduz.
    Chave(Cow<'static, str>),
    /// Um nome do artista (camada, script, texto): pinta-se como está.
    DoArtista(String),
}

/// Os nomes dos scripts da biblioteca do projeto, por `ScriptId` — publicados por quem tem a
/// biblioteca (`ph2d-app-components`), lidos por quem pinta o canal do ficheiro do script.
#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
pub struct NomesDosScripts(pub BTreeMap<u64, String>);

/// Os modos de mistura pelo CÓDIGO que o documento guarda (`ph2d_vec_scene::BlendMode`, a camada
/// do Flip): as chaves da tabela `blend.mode.*`.
pub const MISTURAS: &[&str] = &[
    "blend.mode.normal",
    "blend.mode.multiply",
    "blend.mode.darken",
    "blend.mode.color_burn",
    "blend.mode.linear_burn",
    "blend.mode.lighten",
    "blend.mode.screen",
    "blend.mode.color_dodge",
    "blend.mode.add",
    "blend.mode.overlay",
    "blend.mode.soft_light",
    "blend.mode.hard_light",
    "blend.mode.vivid_light",
    "blend.mode.linear_light",
    "blend.mode.difference",
    "blend.mode.exclusion",
    "blend.mode.hue",
    "blend.mode.saturation",
    "blend.mode.color",
    "blend.mode.luminosity",
    "blend.mode.behind",
    "blend.mode.clear",
];

/// `0` desligado, `1` ligado.
pub const DESLIGADO: &str = "canal.valor.desligado";
pub const LIGADO: &str = "canal.valor.ligado";
/// Um script `0`: sem script.
pub const NENHUM: &str = "canal.valor.nenhum";

/// ⭐ **A palavra do valor `v` do canal `k` da `entidade`**, ou `None` (um canal contínuo, um
/// número, um índice que nenhuma variante, camada ou script tem).
#[must_use]
pub fn palavra(world: &World, entidade: u64, k: FieldKey, v: f32) -> Option<Palavra> {
    let i = indice(v)?;
    let de_lista = |chaves: &'static [&'static str]| {
        let c = *chaves.get(usize::try_from(i).ok()?)?;
        (!c.is_empty()).then_some(Palavra::Chave(Cow::Borrowed(c)))
    };
    match k.info()?.especie {
        Especie::Continuo | Especie::Numero => None,
        Especie::Ligado => Some(Palavra::Chave(Cow::Borrowed(if i == 0 {
            DESLIGADO
        } else {
            LIGADO
        }))),
        Especie::Opcoes(chaves) => de_lista(chaves),
        Especie::PorIrmao { irmao, listas } => {
            let irmao = FieldKey { campo: irmao, ..k };
            let tipo = world
                .get_resource::<CanaisLidos>()?
                .valor(entidade, irmao)?;
            let tipo = u16::try_from(indice(tipo)?).ok()?;
            de_lista(listas.iter().find(|(t, _)| *t == tipo)?.1)
        }
        Especie::Catalogo => {
            crate::canal_componente::chave_da_opcao(k, i).map(|c| Palavra::Chave(Cow::Borrowed(c)))
        }
        Especie::Camada => {
            let nome = u8::try_from(i).ok().and_then(|l| {
                world
                    .get_resource::<crate::SortingLayers>()
                    .map_or_else(
                        || Cow::Owned(crate::SortingLayers::canonical()),
                        Cow::Borrowed,
                    )
                    .name(crate::LayerId(l))
                    .map(str::to_owned)
            });
            nome.map(Palavra::DoArtista)
        }
        Especie::Script if i == 0 => Some(Palavra::Chave(Cow::Borrowed(NENHUM))),
        Especie::Script => world
            .get_resource::<NomesDosScripts>()?
            .0
            .get(&i)
            .cloned()
            .map(Palavra::DoArtista),
        Especie::Texto => world
            .get_resource::<TextosDosCanais>()?
            .texto(v)
            .map(|s| Palavra::DoArtista(s.to_owned())),
    }
}

/// O índice inteiro que um valor discreto guarda (arredondado como os escritores o leem).
fn indice(v: f32) -> Option<u64> {
    let k = v.round();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "arredondado, positivo e preso a 2^24"
    )]
    (0.0..16_777_216.0).contains(&k).then_some(k as u64)
}

#[cfg(test)]
#[path = "canal_palavra_tests.rs"]
mod tests;

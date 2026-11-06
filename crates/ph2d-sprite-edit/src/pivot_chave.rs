//! ⭐ **A chave do Pivot leva os filhos** (dono 06/11, [ficha da Sprite](../../../docs/GameEngineFuncional/componentes/01_sprite.md)):
//! o pivô compensa a posição local deles ([`crate::pivot::o_resto_fica`]), logo sem chave própria o filho deriva ao
//! tocar a animação.

use ph2d_ecs::{Children, Entity, World};
use ph2d_render::Sprite;
use ph2d_timeline::{Instante, PositionKeyMode, PropKind, TimelineDoc, TimelineIntent};

const POSICAO: &[PropKind] = &[
    PropKind::TranslationX,
    PropKind::TranslationY,
    PropKind::Position,
];

/// Os filhos diretos das sprites de `sel`. Na população do Auto Key entram antes do gesto, porque o diff precisa do
/// quadro anterior; quem decide gravar continua a ser o diff.
pub fn filhos_do_pivo(world: &World, sel: impl Iterator<Item = u64>) -> Vec<u64> {
    let mut out = Vec::new();
    for bits in sel {
        let e = Entity::from_bits(bits);
        if world.get::<Sprite>(e).is_none() {
            continue;
        }
        if let Some(c) = world.get::<Children>(e) {
            out.extend(c.iter().map(|f| f.to_bits()));
        }
    }
    out
}

/// O clique na bolinha do Pivot, levando os `filhos`: entram no passo que `i` já abriu, gravados no modo da tecla K
/// ou apagados neste quadro, conforme o que o clique fez à sprite.
#[must_use]
pub fn clique_leva_os_filhos(
    mut i: Vec<TimelineIntent>,
    doc: &TimelineDoc,
    filhos: &[u64],
    inst: Instante,
    valor: impl Fn(u64, PropKind) -> Option<f32>,
    autorar: impl Fn(u64, PropKind) -> Option<TimelineIntent>,
) -> Vec<TimelineIntent> {
    if i.is_empty() {
        return i;
    }
    if let Some(onde) = i.iter().position(|x| *x == TimelineIntent::DeleteSelection) {
        let (_, chaves) = ph2d_timeline::estado_do_grupo(doc, filhos, POSICAO, inst, &valor);
        i.splice(
            onde..onde,
            chaves.into_iter().map(TimelineIntent::AddToSelection),
        );
        return i;
    }
    let mut novas = Vec::new();
    for &f in filhos {
        let na_trajetoria = doc.position_key_mode(f, true) == PositionKeyMode::Path;
        novas.extend(
            POSICAO
                .iter()
                .filter(|&&p| (p == PropKind::Position) == na_trajetoria)
                .filter_map(|&p| autorar(f, p)),
        );
    }
    let fim = i.len() - 1;
    i.splice(fim..fim, novas);
    i
}

#[cfg(test)]
#[path = "pivot_chave_tests.rs"]
mod tests;

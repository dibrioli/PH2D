//! ⭐ **A chave do Pivot leva os filhos** (dono 06/11, [ficha da Sprite](../../../docs/GameEngineFuncional/componentes/01_sprite.md)):
//! o pivô compensa a posição local deles ([`crate::pivot::o_resto_fica`]), logo sem chave própria o filho deriva ao
//! tocar a animação.

use ph2d_anim::Interp;
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

fn e_ancora(p: PropKind) -> bool {
    [0, 1].into_iter().any(|i| {
        let k = ph2d_ecs::canal_componente::chave("ph2d::render::Sprite", 4, i);
        k.map(PropKind::Field) == Some(p)
    })
}

fn e_colisao(p: PropKind) -> bool {
    use ph2d_ecs::canal::familia::FISICA;
    use ph2d_ecs::canal_fisica as fs;
    [fs::DESLOC_X, fs::DESLOC_Y]
        .into_iter()
        .any(|c| p == PropKind::Field(ph2d_ecs::FieldKey::new(FISICA, c, 0, 0)))
}

/// ⭐ O ritmo de uma chave: a posição numa trajetória anda em ritmo constante, logo o pivô e o deslocamento da colisão
/// têm de andar igual, ou o desenho e a colisão fazem vai e vem entre as chaves (dono 06/11, medido em t = 1,25).
/// `com_pivo` diz quem grava o pivô neste passo: quem só anima a colisão fica com o ritmo de sempre.
#[must_use]
pub fn ritmo_da_chave(
    doc: &TimelineDoc,
    e: u64,
    p: PropKind,
    com_pivo: &[u64],
    padrao: Interp,
) -> Interp {
    let do_grupo = e_ancora(p) || (e_colisao(p) && com_pivo.contains(&e));
    if do_grupo && doc.position_key_mode(e, true) == PositionKeyMode::Path {
        Interp::Linear
    } else {
        padrao
    }
}

/// Quem grava o pivô num lote de chaves.
pub fn com_pivo(lote: impl Iterator<Item = (u64, PropKind)>) -> Vec<u64> {
    lote.filter(|(_, p)| e_ancora(*p)).map(|(e, _)| e).collect()
}

/// O mesmo ritmo para as chaves que a bolinha do Pivot grava.
pub fn iguala_o_ritmo(i: &mut [TimelineIntent], doc: &TimelineDoc) {
    let quem = com_pivo(i.iter().filter_map(|x| match x {
        TimelineIntent::AddKey { entity, prop, .. } => Some((*entity, *prop)),
        _ => None,
    }));
    for x in i {
        if let TimelineIntent::AddKey {
            entity,
            prop,
            interp,
            ..
        } = x
        {
            *interp = ritmo_da_chave(doc, *entity, *prop, &quem, *interp);
        }
    }
}

#[cfg(test)]
#[path = "pivot_chave_tests.rs"]
mod tests;

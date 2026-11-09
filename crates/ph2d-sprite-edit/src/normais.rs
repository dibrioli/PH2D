//! ⭐ **A verdade do Normal Map: imagem de cor → imagem normal**, por conteúdo (blake3). Mora aqui, abaixo da
//! família (`ph2d_app_sprite_anim::normal_map`, que a escolhe e a leva à placa) e da biblioteca
//! (`ph2d_app_components::library_names`, que lhe dá o nome): uma verdade, dois leitores.

use ph2d_asset::AssetId;
use std::cell::RefCell;
use std::collections::BTreeMap;

/// O sufixo da normal: o mesmo do ficheiro que o Pixel grava ao lado da cor (`<imagem>_n.png`).
pub const SUFIXO: &str = "_n";

thread_local! {
    static MAPA: RefCell<BTreeMap<AssetId, AssetId>> = const { RefCell::new(BTreeMap::new()) };
}

/// Lê o mapa `cor → normal`.
pub fn com<R>(f: impl FnOnce(&BTreeMap<AssetId, AssetId>) -> R) -> R {
    MAPA.with(|m| f(&m.borrow()))
}

/// Muda o mapa `cor → normal` (escolher, tirar, undo, abrir projeto).
pub fn muda<R>(f: impl FnOnce(&mut BTreeMap<AssetId, AssetId>) -> R) -> R {
    MAPA.with(|m| f(&mut m.borrow_mut()))
}

/// As imagens normais em uso: a biblioteca mostra-as mesmo sem ninguém as mostrar como cor.
#[must_use]
pub fn em_uso() -> Vec<AssetId> {
    com(|m| m.values().copied().collect())
}

/// ⭐ **A normal sem nome próprio chama-se `<cor>_n`** (a do import do `.ase`/folha, que nasce sem ficheiro).
/// `ficheiro` é o nome do ficheiro de uma imagem; o nome próprio é esse ou o de `ids`. Uma normal que tem nome
/// (escolhida da biblioteca) fica com o seu.
pub fn nomeia(
    ids: &mut BTreeMap<AssetId, Option<String>>,
    ficheiro: impl Fn(&AssetId) -> Option<String>,
) {
    let nome = |ids: &BTreeMap<AssetId, Option<String>>, id: &AssetId| {
        ficheiro(id)
            .or_else(|| ids.get(id).cloned().flatten())
            .filter(|n| !n.is_empty())
    };
    for (cor, normal) in com(BTreeMap::clone) {
        if nome(ids, &normal).is_some() {
            continue;
        }
        if let Some(base) = nome(ids, &cor) {
            ids.insert(normal, Some(format!("{base}{SUFIXO}")));
        }
    }
}

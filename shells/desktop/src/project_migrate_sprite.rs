//! **A migração v97 → v98: o blob da `Sprite` PARTE-SE em quatro** (ADR-0164 F1 passo 6 /
//! ADR-0166 / ADR-0070-amendment-8).
//!
//! # Porque ela é diferente da v95 → v96
//!
//! A v95 mudou a **forma do `ProjectFile`** (um campo novo), então precisou de um espelho
//! congelado do ficheiro inteiro ([`crate::project_migrate::ProjectFileV95`]). Esta não muda a
//! forma de nada: o `ProjectFile`, o `WorldSnapshot` e a `EntitySnapshotRow` são idênticos entre
//! v97 e v98. O que mudou está **dentro dos bytes opacos** de um `ComponentBlob` — os 20 campos
//! da `Sprite` v4 passaram a 13.
//!
//! ⇒ o ficheiro v97 lê-se com o tipo VIVO, e a migração é uma travessia que reescreve **um** blob
//! por entidade. Um espelho do `ProjectFile` aqui seria uma cópia de 14 campos que não mudaram.
//!
//! ⚠️ **E é por isso que ela é obrigatória apesar de nada na forma ter mudado:** o postcard é
//! posicional e o `Vec<u8>` do blob passa intacto pelo parse. Sem esta travessia o ficheiro abre
//! **sem erro**, e cada sprite lê os 20 campos antigos com um tipo de 13 — lixo bem-formado, que
//! é o modo de falha que este repositório mais paga.
//!
//! # O que ela produz
//!
//! Por sprite: o blob da `Sprite` reescrito em v5, **mais** até três blobs novos
//! ([`ph2d_ecs::SpriteGrid`] · [`ph2d_ecs::SpriteRegion`] · [`ph2d_ecs::SpriteCornerTint`]) — e
//! só os que o ficheiro de facto autorou, porque materializar o neutro encheria toda cena antiga
//! de secções que o artista nunca pediu (a lei do [`ph2d_render::Sprite::migrate_v4_to_v5`]).

use ph2d_asset::ComponentBlob;
use ph2d_ecs::scene::WorldSnapshot;

/// Quantas linhas a migração tocou, para o log dizer o que aconteceu.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct SpriteSplit {
    /// Sprites reescritas de v4 para v5.
    pub sprites: usize,
    /// Componentes anexados no total (grelha + janela + cantos).
    pub components: usize,
    /// ⚠️ Blobs de `Sprite` que **não** decodificaram como v4. Zero é o esperado; um número
    /// aqui é o sinal de que o ficheiro não é o que o cabeçalho dele diz — e a sprite fica
    /// **como estava**, porque reescrevê-la com um palpite seria pior que a deixar.
    pub unreadable: usize,
}

/// Os quatro nomes canónicos que esta migração escreve. ⚠️ Literais, não `type_name`: o registo
/// é indexado pelo nome dado à mão no `register_*`, e renomear um módulo Rust não pode mudar o
/// que o ficheiro diz.
const SPRITE: &str = "ph2d::render::Sprite";
const GRID: &str = "ph2d::ecs::SpriteGrid";
const REGION: &str = "ph2d::ecs::SpriteRegion";
const CORNER_TINT: &str = "ph2d::ecs::SpriteCornerTint";

/// **Parte o blob da `Sprite` de cada linha do snapshot.** Idempotente na prática: uma linha
/// cuja `Sprite` já é v5 não decodifica como [`ph2d_render::SpriteV4`] com o comprimento certo,
/// e fica intacta (contada em [`SpriteSplit::unreadable`]).
pub(crate) fn split_sprite_blobs(world: &mut WorldSnapshot) -> SpriteSplit {
    let sprite_id = ph2d_ecs::scene::stable_type_id(SPRITE);
    let grid_id = ph2d_ecs::scene::stable_type_id(GRID);
    let region_id = ph2d_ecs::scene::stable_type_id(REGION);
    let corner_id = ph2d_ecs::scene::stable_type_id(CORNER_TINT);

    let mut out = SpriteSplit::default();
    for row in &mut world.entities {
        let Some(slot) = row.components.iter().position(|b| b.type_id == sprite_id) else {
            continue;
        };
        // ⚠️ `take_from_bytes` e **rejeitar o resto**, não `from_bytes`: o postcard consome um
        // prefixo válido e ignora em silêncio o que sobra, então um blob v5 (mais curto) lido
        // como v4 falharia — mas um blob de OUTRO tipo com o mesmo prefixo passaria. A sobra é
        // a única defesa contra isso, e é a mesma que o `load_sprite` já faz.
        let v4 =
            match postcard::take_from_bytes::<ph2d_render::SpriteV4>(&row.components[slot].data) {
                Ok((v4, rest)) if rest.is_empty() => v4,
                _ => {
                    out.unreadable += 1;
                    continue;
                }
            };
        let m = ph2d_render::Sprite::migrate_v4_to_v5(v4);
        let Ok(sprite_bytes) = postcard::to_allocvec(&m.sprite) else {
            out.unreadable += 1;
            continue;
        };
        row.components[slot].data = sprite_bytes;
        out.sprites += 1;

        // ⚠️ **Os componentes são APENDADOS, e a ordem das linhas não é o contrato** — o
        // `snapshot_to_world` insere por `type_id`, não por posição. O que é contrato é a ORDEM
        // DAS LINHAS do snapshot (por `StableId`), e esta travessia não lhe toca.
        for (id, bytes) in [
            (grid_id, m.grid.map(|g| postcard::to_allocvec(&g))),
            (region_id, m.region.map(|r| postcard::to_allocvec(&r))),
            (corner_id, m.corner_tint.map(|c| postcard::to_allocvec(&c))),
        ] {
            match bytes {
                Some(Ok(data)) => {
                    row.components.push(ComponentBlob { type_id: id, data });
                    out.components += 1;
                }
                // Um `to_allocvec` de um tipo `Copy` sem `Vec` dentro não falha; se falhar, a
                // sprite fica sem aquele componente em vez de o ficheiro ser recusado — perder
                // uma grelha é recuperável, recusar o projeto do artista não.
                Some(Err(_)) => out.unreadable += 1,
                None => {}
            }
        }
    }
    out
}

#[cfg(test)]
#[path = "project_migrate_sprite_tests.rs"]
mod tests;

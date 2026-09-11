//! **Quem faz nascer uma sprite `Individual` carimba os pixels dela.**
//!
//! # O defeito, e por que ele leva um dia a aparecer
//!
//! `SpriteSource::Individual { texture_id }` guarda um **id de alocação da GPU** dentro de um
//! componente **persistido**. O store recomeça a numerar em `1` a cada processo. O que faz uma
//! sprite dessas sobreviver a um save/load é o carimbo `SpritePixels(AssetId)` — é por ele que o
//! `save_sprite_pixels` recolhe os bytes.
//!
//! ⚠️ **Uma sprite sem o carimbo abre PERFEITA e grava VAZIA.** Nada no ecrã distingue as duas: ela
//! renderiza, arrasta, edita — e no dia seguinte volta invisível, ou a mostrar os pixels de outra.
//!
//! # Como isto foi encontrado, que é o motivo de o gate ser estrutural
//!
//! Enio, 2026-08-20: *"remove background com separate islands fez cair para rgba8 e **não permite
//! voltar para 16**"*. A queixa era sobre **precisão**; a causa era o carimbo em falta, porque a
//! conversão procura os bytes pelo mesmo caminho que o save.
//!
//! A varredura encontrou **dois** sítios, os dois anteriores a esta wave:
//!
//! | sítio | o que se perdia |
//! |---|---|
//! | `bgremoval` · Separate Islands | cada ilha spawnada, ao reabrir |
//! | `sprite_merge` · Merge Sprites | o merge — e **pior**, ele despawna os originais, então não havia de onde refazer |
//!
//! *Um bug que só aparece depois de fechar o projeto não é encontrado por quem o usa; é encontrado
//! por outra pergunta que passa pelo mesmo sítio.* Esta foi a pergunta.

use std::fs;
use std::path::{Path, PathBuf};

/// Construir uma sprite de textura própria.
const BUILDS_INDIVIDUAL: &str = "Sprite::individual(";
/// **Os DOIS donos duráveis dos pixels de uma sprite `Individual`.**
///
/// ⚠️ A primeira versão deste gate só conhecia o `SpritePixels`, e a varredura acusou dois sítios
/// **legítimos**: o `sheet_import` (as peças de uma folha são possuídas pelo `SpriteSheetRef` — a
/// folha grava-se como `AuthoredSheet` e o load re-liga por região; um `SpritePixels` ali
/// **duplicaria** os pixels no ficheiro) e o `project_sprite_pixels` (fixtures de teste).
///
/// *Um gate que acusa o legítimo é desligado na primeira semana.* A lei verdadeira é que os bytes
/// tenham **um** dono durável — e há dois tipos de dono.
const DURABLE_OWNERS: [&str; 2] = ["SpritePixels", "SpriteSheetRef"];

fn shell_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read_dir").flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// O corpo de PRODUÇÃO: sem comentários — vários deles **citam** os dois nomes precisamente para
/// explicar esta regra, e um comentário não spawna nada — e **sem os módulos `#[cfg(test)]`**.
///
/// ⚠️ **A exclusão dos testes foi acrescentada em 2026-08-21, e APERTA o gate em vez de o
/// afrouxar.** Um fixture de teste que constrói `Sprite::individual` não grava nada e não precisa
/// de dono durável; enquanto ele contava, um ficheiro podia satisfazer a regra com um carimbo que
/// só existia **dentro do `#[cfg(test)]`** — e é isso que agora deixa de passar. O falso positivo
/// apareceu ao cortar a cauda de re-alojamento para `texture_rebind.rs`: o `texture_edit.rs`
/// ficou com dois fixtures e nenhum carimbo, sobre zero construções de produção.
fn code(src: &str) -> String {
    src.split("#[cfg(test)]")
        .next()
        .unwrap_or("")
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

/// **Todo ficheiro que constrói uma `Sprite::individual` também carimba `SpritePixels`.**
///
/// ⚠️ É deliberadamente grosso — por FICHEIRO, não por chamada. Um gate fino (casar o carimbo com
/// o spawn certo) precisaria de analisar o fluxo, e o que ele ganharia em precisão perderia em
/// sobreviver a um refactor. *Um gate grosso e verdadeiro é lido; um gate fino e frágil é
/// silenciado.*
#[test]
fn every_file_that_spawns_an_individual_sprite_also_stamps_its_pixels() {
    let mut files = Vec::new();
    rust_files(&shell_src(), &mut files);
    assert!(
        files.len() > 50,
        "so' {} ficheiros varridos — a varredura partiu-se e este gate mede o vazio",
        files.len()
    );

    let mut offenders = Vec::new();
    for path in &files {
        let src = fs::read_to_string(path).expect("read");
        let body = code(&src);
        if body.contains(BUILDS_INDIVIDUAL) && !DURABLE_OWNERS.iter().any(|o| body.contains(o)) {
            offenders.push(format!("  {}", path.display()));
        }
    }
    assert!(
        offenders.is_empty(),
        "estes ficheiros constroem `{BUILDS_INDIVIDUAL}` sem nomear dono durável nenhum \
         ({DURABLE_OWNERS:?}):\n{}\
         \n\n\
         Uma sprite de textura propria SEM o carimbo abre perfeita e GRAVA VAZIA: o `texture_id` e' \
         uma alocacao de GPU que morre com o processo, e o `save_sprite_pixels` recolhe pelos \
         carimbos. O sintoma aparece um dia depois, ao reabrir o projeto.\n\
         Insira os bytes no `AssetDb` (`insert_image_rgba8` / `_rgba16`) e carimbe o `AssetId` na \
         entidade, como o `commit_edited_texture` faz.",
        offenders.join("\n")
    );
}

/// **Controle positivo:** os dois sítios que a varredura de 2026-08-20 encontrou constroem MESMO
/// uma individual, e agora carimbam.
///
/// ⚠️ Sem isto, renomear qualquer um dos dois marcadores faria o gate acima ficar verde por não
/// encontrar nada — e o dia em que ele parasse de medir seria o dia em que ninguém repararia.
#[test]
fn the_two_sites_the_sweep_found_are_real_and_now_stamp() {
    for rel in [
        "hero_intents/image_edit/bgremoval.rs",
        "hero_intents/sprite_merge.rs",
    ] {
        let path = shell_src().join(rel);
        let body = code(&fs::read_to_string(&path).expect("read"));
        assert!(
            body.contains(BUILDS_INDIVIDUAL),
            "{rel} deixou de construir uma individual — se isso e' verdade, APAGUE esta entrada em \
             vez de a silenciar"
        );
        assert!(
            DURABLE_OWNERS.iter().any(|o| body.contains(o)),
            "{rel} voltou a spawnar uma individual sem dono duravel: ela grava VAZIA"
        );
    }
}

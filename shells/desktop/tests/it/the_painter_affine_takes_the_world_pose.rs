//! **O afim imagem→ecrã do Painter é construído sobre a pose de MUNDO.**
//!
//! Enio, 2026-08-19: *"se a sprite é filha de outra, não consigo pintá-la"*.
//!
//! ## O mecanismo
//!
//! `sprite_image_to_screen_affine` compõe `imagem → local → mundo → ecrã`, e o comentário dentro
//! dela sempre prometeu *"sprite-local meters → world"*. Mas o `Transform` que ela recebia era a
//! pose **LOCAL** da entidade. Numa sprite de **raiz** local e mundo são a mesma coisa — e foi por
//! isso que a promessa sobreviveu a **21 chamadores** sem ninguém reparar.
//!
//! Numa sprite **filha**, falta a cadeia do pai. O afim mapeia o ponteiro para outro sítio, e a
//! guarda de pegada do Painter — que usa este mesmo afim para decidir se o clique caiu sobre o
//! sprite — recusa **toda** pincelada. O sintoma não é pintar torto: é não pintar.
//!
//! ## O gate, e por que ele é estrutural
//!
//! A conta certa é `ph2d_ecs::world_transform`, e o que se quer afirmar é que **nenhum chamador
//! volta a alimentar a função com uma pose local**. Isso não é um comportamento observável num
//! teste headless (a cadeia inteira precisa de GPU, tool ativa e ponteiro); é uma propriedade do
//! CÓDIGO — e é o mesmo instrumento que o gate do pivô de joint usa, pela mesma razão.
//!
//! ⚠️ A defesa principal **não é este gate**: é o TIPO. O parâmetro passou de `&Transform` para
//! `Transform` por valor, e por isso todo chamador antigo deixou de compilar até resolver a pose.
//! *Uma convenção nova sobre a mesma assinatura teria sido esquecida no 22º sítio.* Este gate
//! guarda o resto: que ninguém desfaça a assinatura, e que os chamadores não voltem a `get::<>`.

use std::path::{Path, PathBuf};

fn shell_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            rs_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// O parâmetro é por VALOR e chama-se `world_tr` — as duas metades da defesa de tipo.
#[test]
fn the_affine_takes_a_world_transform_by_value() {
    let src = std::fs::read_to_string(shell_src().join("render_loop/bgremoval_preview.rs"))
        .expect("bgremoval_preview.rs readable");
    assert!(
        src.contains("world_tr: ph2d_ecs::Transform,"),
        "o parametro de pose do `sprite_image_to_screen_affine` deixou de ser `world_tr: \
         ph2d_ecs::Transform` (por VALOR).\n\
         Voltar a `&Transform` faz todo chamador antigo compilar outra vez — e um deles passa a \
         pose LOCAL, que numa sprite FILHA recusa toda pincelada."
    );
}

/// **Nenhum chamador resolve a pose com `get::<Transform>` na vizinhança da chamada.**
///
/// A janela é o braço inteiro entre o `get` e a chamada; procurar no ficheiro todo daria falsos
/// positivos (um `get::<Transform>` legítimo para outra coisa), e procurar numa janela de N bytes
/// apodrece — vide a lição do gate do pivô, que reprovou sobre código correto por um comentário
/// ter crescido.
#[test]
fn no_caller_feeds_the_affine_a_local_pose() {
    let mut files = Vec::new();
    rs_files(&shell_src(), &mut files);
    let mut offenders: Vec<String> = Vec::new();
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else {
            continue;
        };
        // Só interessam os ficheiros que CHAMAM o afim.
        if !src.contains("sprite_image_to_screen_affine(") {
            continue;
        }
        // Um `get::<…Transform>(entity)` num ficheiro que chama o afim é o padrão que produziu o
        // defeito. Se um dia houver um uso legítimo, ele nomeia-se de outra forma (ou o gate
        // ganha a excecao COM o motivo escrito — nunca em silencio).
        for (i, line) in src.lines().enumerate() {
            let t = line.trim_start();
            if t.starts_with("//") {
                continue;
            }
            if t.contains("Transform>(entity)") {
                offenders.push(format!(
                    "  {}:{}: {}",
                    f.file_name().unwrap_or_default().to_string_lossy(),
                    i + 1,
                    t.trim()
                ));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "um ficheiro que constroi o afim imagem→ecra' do Painter resolve a pose com \
         `get::<Transform>` (a pose LOCAL) em vez de `ph2d_ecs::world_transform`:\n{}\n\n\
         Numa sprite FILHA a cadeia do pai falta, o afim mapeia o ponteiro para fora da pegada, \
         e o Painter recusa toda pincelada (Enio, 2026-08-19).",
        offenders.join("\n")
    );
}

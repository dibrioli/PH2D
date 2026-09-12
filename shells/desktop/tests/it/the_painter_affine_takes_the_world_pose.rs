//! **Nenhum chamador do afim imagem→ecrã lhe entrega uma pose LOCAL.**
//!
//! Enio, 2026-08-19: *"se a sprite é filha de outra, não consigo pintá-la"*.
//!
//! ## O mecanismo
//!
//! `sprite_image_to_screen_affine` compõe `imagem → local → mundo → ecrã`, e o comentário dentro
//! dela sempre prometeu *"sprite-local meters → world"*. Mas o `Transform` que ela recebia era a
//! pose **LOCAL** da entidade. Numa sprite de **raiz** local e mundo são a mesma coisa — e foi por
//! isso que a promessa sobreviveu a **21 chamadores** sem ninguém reparar. Numa sprite **filha**
//! falta a cadeia do pai: o afim mapeia o ponteiro para fora da pegada e o Painter recusa **toda**
//! pincelada. O sintoma não é pintar torto — é não pintar.
//!
//! ## ⚠️ A OUTRA METADE deste gate mudou de casa (W2 Fase D)
//!
//! A afirmação sobre a **assinatura** (*o parâmetro é `world_tr: Transform`, por VALOR*) vivia
//! aqui e lia `render_loop/bgremoval_preview.rs` por `read_to_string` de caminho fixo. O afim saiu
//! para a folha [`ph2d_sprite_screen`] (quatro assuntos da shell partilhavam-no — `HOWTO` §1.2), e
//! a lei foi com ele: hoje é `crates/ph2d-sprite-screen/tests/a_pose_e_de_mundo.rs`, por
//! `include_str!`, que falha a **COMPILAR** se o ficheiro voltar a mudar de sítio.
//!
//! ⇒ **Aqui fica só o que é da SHELL:** quem chama, e com que pose. Pela tabela do `HOWTO` §2.6,
//! *o gate que mede a lei vive com a lei; o que mede os chamadores desta árvore vive nesta árvore*.

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

/// **Nenhum chamador resolve a pose com `get::<Transform>` na vizinhança da chamada.**
///
/// A janela é o braço inteiro entre o `get` e a chamada; procurar no ficheiro todo daria falsos
/// positivos (um `get::<Transform>` legítimo para outra coisa), e procurar numa janela de N bytes
/// apodrece — vide a lição do gate do pivô, que reprovou sobre código correto por um comentário
/// ter crescido.
///
/// ⛔⛔ **PISO DE POPULAÇÃO, e ele é a metade que faltava** (`HOWTO` §2.7). Este censo varre a
/// `shells/desktop/src` e só acusa ficheiros que **chamam** o afim. A família `painter` está a
/// sair para `crates/ph2d-app-painter`, e **12 dos 14** chamadores de hoje vão com ela — sem o
/// piso, este gate passaria a varrer dois ficheiros e ficaria **verde a medir quase nada**, que é
/// exactamente o modo de falha MUDO daquela secção. *Um `offenders.is_empty()` sobre uma lista
/// construída de zero ficheiros é trivialmente verdadeiro.*
///
/// ⚠️ **Quando a família se mudar, este piso desce e o gate ganha a segunda raiz**
/// (`crates/ph2d-app-painter/src`), de modo que a SOMA continue a ser 14. Baixar o piso sem
/// acrescentar a raiz é desfazer a defesa.
#[test]
fn no_caller_feeds_the_affine_a_local_pose() {
    /// Quantos ficheiros desta árvore chamam o afim, medido em 2026-09-12.
    const CHAMADORES_MIN: usize = 14;

    let mut files = Vec::new();
    rs_files(&shell_src(), &mut files);
    let mut chamadores = 0usize;
    let mut offenders: Vec<String> = Vec::new();
    for f in &files {
        let Ok(src) = std::fs::read_to_string(f) else {
            continue;
        };
        // Só interessam os ficheiros que CHAMAM o afim.
        if !src.contains("sprite_image_to_screen_affine(") {
            continue;
        }
        chamadores += 1;
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
        chamadores >= CHAMADORES_MIN,
        "este censo achou {chamadores} ficheiros a chamar o afim e esperava >= {CHAMADORES_MIN} \
         — ele PERDEU O SUJEITO.\n\
         Se a familia `painter` ja se mudou para `crates/ph2d-app-painter`, a cura NAO e' baixar \
         este numero: e' acrescentar a raiz da crate a varredura, para a SOMA continuar a ser {CHAMADORES_MIN}."
    );
    assert!(
        offenders.is_empty(),
        "um ficheiro que constroi o afim imagem→ecra' do Painter resolve a pose com \
         `get::<Transform>` (a pose LOCAL) em vez de `ph2d_ecs::world_transform`:\n{}\n\n\
         Numa sprite FILHA a cadeia do pai falta, o afim mapeia o ponteiro para fora da pegada, \
         e o Painter recusa toda pincelada (Enio, 2026-08-19).",
        offenders.join("\n")
    );
}

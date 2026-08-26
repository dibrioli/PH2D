//! ⛔ **Nenhuma escrita NÃO-RASTREADA nas crates de simulação** (ADR-0164 §2.7 / plano F2).
//!
//! # Porque este lint existe, e porque ele é estrutural e não um teste de comportamento
//!
//! A captura incremental do desfazer usa o relógio de mudanças do ECS como **pré-filtro**: uma
//! linha que nenhum tick acusa e cujo archetype não mudou é **reaproveitada sem ser lida**. Isso
//! é o que faz um passo custar o tamanho da edição.
//!
//! ⇒ Uma escrita que **contorna** o relógio não fica lenta. Ela fica **invisível**: o valor muda
//! no mundo, a linha do snapshot continua a ser a antiga, e o Ctrl+Z devolve o objeto a um estado
//! que nunca existiu — sem erro, sem aviso, e só quando o artista já perdeu o trabalho.
//!
//! *Não há teste de comportamento que apanhe isto*, porque o defeito é a **ausência** de um sinal:
//! um gate teria de adivinhar qual sistema, qual componente e qual quadro. O que se pode afirmar é
//! a **estrutura** — que as portas de fuga não são chamadas —, e é o que este ficheiro faz.
//!
//! ⚠️ **Medido em 2026-08-25: ZERO usos em todas as crates varridas.** Este lint não conserta
//! nada; ele mantém uma propriedade que hoje é verdadeira **por acaso** e que a F2 passou a
//! precisar **por desenho**. É o mesmo movimento do `no_hex_in_ui`: a hora de escrever a cerca é
//! quando o campo do outro lado dela passa a valer alguma coisa.
//!
//! # O que NÃO é proibido, e porquê
//!
//! - `get_mut` / `Mut<T>` — são as portas RASTREADAS; é para elas que se empurra quem for barrado.
//! - `set_if_neq` — é a **cura** do falso positivo, não a doença.
//! - Uma crate fora da lista (render, painéis, ferramentas) pode usar o que quiser: o que a
//!   captura lê é o mundo de simulação, e é só ele que tem de ser honesto.

use std::path::{Path, PathBuf};

/// As portas de fuga do relógio de mudanças do bevy, e o que cada uma faria.
///
/// ⚠️⚠️ **DUAS candidatas foram RECUSADAS depois de ler a fonte do bevy, e a intuição sobre as
/// duas era a mesma — e errada.** A 1.ª versão desta lista proibia também:
///
/// - **`into_inner`** — *"extrai o `&mut` de dentro do `Mut<T>`, logo as escritas seguintes não
///   carimbam"*. **Falso, e o inverso é que é verdade:** o doc do bevy diz, palavra por palavra,
///   *"Returns the pointer to the value, **marking it as changed**. In order to avoid marking the
///   value as changed, you need to call `bypass_change_detection`."* Ele é uma porta **rastreada**.
/// - **`get_mut_by_id`** — mesma intuição (*"ponteiro cru por `ComponentId`"*), mesmo erro: ele
///   devolve um `MutUntyped`, cujo `into_inner` carimba.
///
/// Juntas, elas acusavam **20+ sítios corretos** em `ph2d-ecs` e `ph2d-script`. *Um lint que
/// reprova código correto não é rigor — é o lint que alguém desliga na primeira semana*, e a
/// forma de o evitar é ler a fonte do que se vai proibir, não a intuição sobre o nome dele.
const FORBIDDEN: &[(&str, &str)] = &[
    (
        "bypass_change_detection",
        "escreve SEM carimbar tick — a captura incremental nunca veria a mudanca",
    ),
    (
        "as_unsafe_world_cell",
        "abre o mundo sem rastreio — tudo o que passar por aqui e' invisivel para o desfazer",
    ),
];

/// As crates cuja escrita a captura do desfazer LÊ. ⚠️ Acrescentar uma crate de simulação nova
/// significa acrescentá-la aqui — o lint não a descobre sozinha, e a ausência seria muda.
const SIM_CRATES: &[&str] = &[
    "ph2d-ecs",
    "ph2d-physics-ecs",
    "ph2d-render",
    "ph2d-script",
    "ph2d-field-ecs",
];

fn workspace_root() -> PathBuf {
    // `CARGO_MANIFEST_DIR` = .../crates/ph2d-ecs
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("a raiz da workspace")
        .to_path_buf()
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

#[test]
fn the_sim_crates_never_write_behind_the_change_clock() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let mut scanned = 0usize;

    for crate_name in SIM_CRATES {
        let dir = root.join("crates").join(crate_name).join("src");
        assert!(
            dir.is_dir(),
            "a crate '{crate_name}' desta lista nao existe em {dir:?} — ou ela foi renomeada e a \
             lista envelheceu, ou o lint esta' a varrer o vazio e a passar por isso"
        );
        let mut files = Vec::new();
        rust_files(&dir, &mut files);
        for f in files {
            let Ok(src) = std::fs::read_to_string(&f) else {
                continue;
            };
            scanned += 1;
            for (line_no, line) in src.lines().enumerate() {
                // ⚠️ Comentários e docs ficam de fora **de propósito**: este mesmo ficheiro
                // NOMEIA as duas portas, e um lint que se apanhasse a si próprio seria
                // desligado no primeiro dia.
                let code = line.trim_start();
                if code.starts_with("//") || code.starts_with("*") {
                    continue;
                }
                for (needle, why) in FORBIDDEN {
                    if line.contains(needle) {
                        offenders.push(format!(
                            "  {}:{} — `{needle}` ({why})",
                            f.strip_prefix(&root).unwrap_or(&f).display(),
                            line_no + 1
                        ));
                    }
                }
            }
        }
    }

    assert!(
        scanned > 100,
        "o lint varreu so' {scanned} ficheiros — ele nao esta' a olhar para onde pensa que olha, e \
         um lint que varre o vazio passa sempre"
    );
    assert!(
        offenders.is_empty(),
        "escrita NAO-RASTREADA numa crate de simulacao — a captura incremental do desfazer \
         (ADR-0164 §2.7) usa o tick como pre-filtro, entao uma escrita que o contorna nao fica \
         lenta: fica INVISIVEL, e o Ctrl+Z devolve um estado que nunca existiu.\n{}\n\n\
         Se a escrita for mesmo necessaria, ela tem de passar por `get_mut` (rastreado) ou o \
         componente tem de sair do registo — nao ha' terceira saida que o desfazer aguente.",
        offenders.join("\n")
    );
}

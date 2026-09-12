//! ⭐⭐⭐ **A SHELL SÓ ENCOLHE** — a catraca da grandeza que de facto decide o tempo de build.
//!
//! # Por que este gate existe, com os números
//!
//! A auditoria de velocidade de 2026-09-10 mediu que o custo do laço de desenvolvimento **não**
//! está no `cargo check` (1,8–3,3 s) nem no linker: está em `shells/desktop` ser **UMA unidade de
//! compilação** de meio milhão de linhas, a **última** de toda build grande — sozinha, 34–45 s no
//! fim do portão de fecho, e 161 s de reconstrução em `--release` num núcleo só.
//!
//! A W2 (2026-09-11) tirou de lá **61 704 linhas** em seis linhas paralelas. ⛔ **Nada impedia que
//! elas voltassem.**
//!
//! # ⛔⛔ Por que os tectos que já existiam não respondem a esta pergunta
//!
//! Todo tecto de LOC deste repo é **por FICHEIRO** (`architecture_workspace_file_loc_cap` = 700,
//! `architecture_panel_loc_cap`, `architecture_widget_loc_cap`, `file_loc_caps` da shell). Eles
//! medem *legibilidade de um ficheiro*; nenhum mede **o tamanho da unidade que o compilador
//! constrói**. Uma shell de 465 mil linhas em 1 801 ficheiros de 258 linhas cada passa em todos
//! eles com folga — e continua a ser a unidade mais lenta da árvore.
//!
//! ⚠️ **É a mesma forma que o `CLAUDE.md` §5.0 já nomeia um nível abaixo** (*«um tecto por-ficheiro
//! é a única grandeza deste repo que SOMA entre linhas sem ninguém a contar»*), e aqui o que soma é
//! a **crate**: seis linhas podem acrescentar 200 linhas cada à shell, todas verdes de boa-fé, e o
//! total sobe 1 200 sem um único gate a acordar.
//!
//! # A regra que este gate torna executável
//!
//! ⭐ **Código de FAMÍLIA vive em `crates/ph2d-app-<família>`; a shell é COMPOSIÇÃO.** O molde, as
//! cinco portas do trait de host e as 15 armadilhas medidas estão em
//! [`HOWTO_partir_uma_familia_da_shell.md`](../../../../docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md).
//! Quando este gate reprovar, a cura é **mover para a crate da família**, nunca subir o número.
//!
//! # ⚠️ A catraca tem as DUAS metades, senão vira licença
//!
//! `CLAUDE.md` §5.0: *«uma catraca sem censo de obsolescência não desce: ela vira LICENÇA»*. Então
//! o gate reprova nos dois sentidos — cresceu acima do tecto **ou** encolheu tanto que o tecto
//! deixou de descrever a árvore e ninguém o apertou.
//!
//! ⛔ **A folga NÃO é permissão para crescer.** Ela existe porque a shell continua a ser a raiz de
//! composição: ligar uma família nova custa linhas legítimas no `main.rs`. O que a folga compra é
//! ~0,9 % — o suficiente para compor, longe do suficiente para uma família caber.
//!
//! Sem deps (só `std`), como os gates de arquitectura irmãos.

use std::fs;
use std::path::{Path, PathBuf};

/// **O tecto da crate inteira**, em linhas de `.rs` versionado.
///
/// ⚠️ **Recontado a cada integração que encolhe a shell** — é isso que impede a folga de virar
/// licença. Histórico: **465 105** / 1 801 ficheiros (11/09, fim da Fase A da W2) → **451 084** /
/// 1 757 (11/09, Fase B da `physics`: o roteador de cenas e os 43 ficheiros de `render_loop` dela
/// saíram). O tecto é o medido mais a [`FOLGA_DE_COMPOSICAO`].
///
/// ⛔ **Nenhuma LINHA lhe toca** — é um número que soma entre linhas, logo CONTA-SE, nunca se
/// escolhe (`CLAUDE.md` §5.0): com cinco linhas a escrevê-lo o merge fica com um deles e nenhum
/// está certo, em silêncio. Quem o reconta é o integrador, sobre a árvore combinada.
const TETO_LOC: usize = 455_084;

/// Quanto a shell pode crescer acima do medido antes de o gate reprovar — a margem da raiz de
/// composição, **não** espaço para um módulo.
const FOLGA_DE_COMPOSICAO: usize = 4_000;

/// ⭐ **O censo de obsolescência:** quando a shell cai mais do que isto abaixo do tecto, o número
/// acima deixou de a descrever e tem de ser reescrito com a medição do dia.
///
/// ⚠️ **É maior que a folga de propósito.** Um limiar apertado dispararia a cada linha removida —
/// e durante a Fase B a shell encolhe todos os dias. O que este censo existe para apanhar é *uma
/// wave inteira que aterrou e deixou a catraca para trás*, não uma limpeza de dez linhas.
const CENSO_OBSOLETO_ACIMA_DE: usize = 20_000;

fn shell_dir() -> PathBuf {
    // CARGO_MANIFEST_DIR = crates/ph2d-editor-core; dois pais = a raiz da workspace.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .join("shells/desktop")
}

/// Todas as linhas de `.rs` sob `dir`, recursivamente.
///
/// ⚠️ Salta `target/` por defesa: numa workspace ele não nasce aqui, mas um `CARGO_TARGET_DIR`
/// apontado para dentro da crate faria este gate medir código gerado e reprovar sobre nada.
fn conta(dir: &Path, linhas: &mut usize, ficheiros: &mut usize) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            conta(&p, linhas, ficheiros);
        } else if p.extension().is_some_and(|x| x == "rs") {
            let Ok(s) = fs::read_to_string(&p) else {
                continue;
            };
            *linhas += s.lines().count();
            *ficheiros += 1;
        }
    }
}

#[test]
fn the_shell_only_shrinks() {
    let (mut loc, mut ficheiros) = (0usize, 0usize);
    conta(&shell_dir(), &mut loc, &mut ficheiros);

    // ⛔ Controlo positivo: uma varredura que não achasse nada reprovaria «cresceu zero» e leria-se
    //    como aprovada — a armadilha do censo que mede zero. Sem isto o gate é vácuo se o caminho
    //    da shell mudar de nome.
    assert!(
        ficheiros > 500,
        "controlo positivo: a varredura achou {ficheiros} ficheiros `.rs` em `shells/desktop` — \
         o caminho mudou e este gate passou a medir o nada"
    );

    assert!(
        loc <= TETO_LOC,
        "a SHELL CRESCEU: {loc} linhas contra o tecto de {TETO_LOC} ({ficheiros} ficheiros).\n\
         Ela é UMA unidade de compilação e a última de toda build grande — 34-45 s sozinha no \
         portão de fecho (auditoria de 2026-09-10).\n\
         ⇒ a cura é MOVER para `crates/ph2d-app-<família>`, nunca subir este número. O molde está \
         em `docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md`.\n\
         ⛔ Subir o tecto sem mover código é desfazer, uma wave de cada vez, as 61 704 linhas que \
         a W2 tirou de lá."
    );

    assert!(
        loc + CENSO_OBSOLETO_ACIMA_DE >= TETO_LOC,
        "a catraca está OBSOLETA: a shell tem {loc} linhas e o tecto ainda diz {TETO_LOC} — uma \
         folga de {} linhas.\n\
         Uma catraca que não desce vira LICENÇA (CLAUDE.md §5.0): reescreva `TETO_LOC` como \
         {} (o medido de hoje + a folga de composição de {FOLGA_DE_COMPOSICAO}).",
        TETO_LOC - loc,
        loc + FOLGA_DE_COMPOSICAO
    );
}

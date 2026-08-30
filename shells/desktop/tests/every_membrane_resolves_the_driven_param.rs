//! **TODA MEMBRANA QUE CUNHA UMA CHAVE A PARTIR DE PARAMS RESOLVE O FIO** (doc 58).
//!
//! # O mecanismo, em três linhas
//!
//! Um nó recebe o que não alcança (uma fonte, um arquivo de som, uma geometria) pelo canal
//! externo, sob uma **chave de conteúdo** que os dois lados derivam dos params. O shell publica
//! **antes** do cook; o valor de um param **conduzido por fio** só existe **durante** o cook.
//! ⇒ uma membrana que leia `override → default` cunha uma chave, o `eval` do nó — que lê por
//! `ctx.param`, e portanto resolve `conduzido → override → default` — pede **outra**, e o nó
//! recebe um external que ninguém escreveu. **O que ele desenha desaparece, em silêncio.**
//!
//! # Por que um gate, e não uma nota
//!
//! A nota existia. O cabeçalho do `motion_externals.rs` dizia, por escrito, *"a membrana que
//! nascer amanhã tem de a herdar sem a redescobrir"* — e o censo de 2026-08-28 mediu que das
//! **quatro** membranas que derivam uma chave de params, **duas nunca a herdaram**: as oito
//! bandas do `audio.bands` e o `time_offset` do canal deslocado do `source.object`. As duas
//! curadas tinham a escada **copiada linha a linha**, o que é precisamente por que as outras
//! duas não a receberam: *uma lei escrita duas vezes ainda não é uma lei — só uma PORTA é.*
//!
//! # As duas metades
//!
//! 1. **Cada membrana conhecida chama a porta.** Tirar a chamada de uma delas é vermelho.
//! 2. **Nenhum arquivo novo pode cunhar uma chave a partir de params sem ela.** É a metade que
//!    apanha a QUINTA membrana — a que ainda não existe, e que é a única que este gate não
//!    consegue nomear.

use std::path::Path;

/// A porta única: [`motion_externals::resolved_params`].
const LADDER: &str = "resolved_params";

/// Como se lê um param **autorado** — o degrau de que a escada tem de partir. É o marcador de
/// *"este arquivo deriva alguma coisa de params de nó"*.
const AUTHORED: &str = "node_param_overrides";

/// Como se cunha ou se publica uma chave do canal externo.
const MINTS: &[&str] = &[
    "set_external",
    "external::appearance_of",
    "external::pose_of",
];

/// **As membranas MEDIDAS** (2026-08-28) — os arquivos que derivam uma chave de conteúdo dos
/// params de um nó.
///
/// ⚠️ Esta lista é uma **fixture do censo**, não a lei. A lei é a metade 2 abaixo, que não
/// precisa de lista nenhuma; esta existe para que apagar a chamada de uma membrana conhecida
/// seja vermelho **com o nome dela**, em vez de vermelho genérico.
const MEMBRANES: &[&str] = &[
    "motion_shape_gen.rs",
    "motion_text_gen.rs",
    "motion_audio_gen.rs",
    "motion_bridge_objects_shift.rs",
    // ⭐ **A quinta, nascida em 2026-08-30** — as fitas do `source.lsystem` no modo `Branches`
    // (doc 95). Entra aqui pelo mesmo motivo que as outras: ela cunha uma chave de CONTEÚDO dos
    // params de um nó, e uma chave cunhada do valor estático não encontra a que o nó procura
    // quando um fio conduz aquele param — a planta desaparece **em silêncio**.
    "motion_lsystem_gen.rs",
];

/// Os arquivos de produto do `render_loop` (os `*_tests.rs` ficam de fora: um teste pode
/// construir um estado à mão sem ser uma membrana).
fn product_files() -> Vec<(String, String)> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/render_loop");
    let mut out = Vec::new();
    for e in std::fs::read_dir(&dir).expect("o render_loop existe") {
        let p = e.expect("entrada").path();
        let name = p.file_name().expect("nome").to_string_lossy().to_string();
        if !name.ends_with(".rs") || name.ends_with("_tests.rs") {
            continue;
        }
        out.push((name, std::fs::read_to_string(&p).expect("le")));
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

/// **METADE 1 — cada membrana conhecida chama a porta.**
#[test]
fn every_known_membrane_calls_the_one_ladder() {
    let files = product_files();
    for want in MEMBRANES {
        let (_, src) = files
            .iter()
            .find(|(n, _)| n == want)
            .unwrap_or_else(|| panic!("a membrana `{want}` tem de existir — foi ela que o censo mediu; se ela mudou de nome, mude a fixture com a mudanca"));
        assert!(
            src.contains(LADDER),
            "`{want}` cunha uma chave de conteudo a partir de params e TEM de a resolver por \
             `{LADDER}`. Sem isso, um param conduzido por fio faz o no' pedir uma chave que \
             ninguem publicou — e o que ele desenha some, sem erro nenhum."
        );
    }
}

/// **METADE 2 — a que apanha a membrana que ainda não existe.**
///
/// ⚠️ **A conjunção é o que a torna precisa.** Ler `node_param_overrides` sozinho é o painel a
/// mostrar um número (legítimo, e há vários); cunhar uma chave sozinho é publicar por NOME (o
/// que os objetos e as formas da cena fazem). É **derivar uma chave a partir de params** que
/// obriga à escada — e é exactamente a interseção que o censo de 2026-08-28 encontrou com dois
/// membros por curar.
#[test]
fn no_file_mints_a_key_from_authored_params_without_the_ladder() {
    let mut offenders = Vec::new();
    for (name, src) in product_files() {
        // A própria porta lê o degrau autorado — é o que ela É.
        if name == "motion_externals.rs" {
            continue;
        }
        let reads_authored = src.contains(AUTHORED);
        let mints = MINTS.iter().any(|m| src.contains(m));
        if reads_authored && mints && !src.contains(LADDER) {
            offenders.push(name);
        }
    }
    assert!(
        offenders.is_empty(),
        "estes arquivos derivam uma chave do canal externo a partir de params AUTORADOS e nao \
         resolvem o fio ({LADDER}): {offenders:?}\n\
         Um param conduzido faz a chave do shell e a do no' DIVERGIREM, e o no' passa a ler um \
         external que ninguem escreveu — stream vazio, e o que ele desenha desaparece."
    );
}

/// **E o censo NÃO está vazio** — o controle da própria fixture.
///
/// ⚠️ Sem isto, renomear os quatro arquivos (ou apagar a pasta) deixaria as duas metades acima
/// verdes por não terem nada a medir. *Um zero de «não medido» e um de «tudo certo» são o mesmo
/// byte* — é a lei que a caça aos knobs mortos escreveu, aplicada à sonda dela mesma.
#[test]
fn the_census_actually_scanned_something() {
    let files = product_files();
    assert!(
        files.len() > 50,
        "o render_loop tem dezenas de arquivos de produto, achei {}",
        files.len()
    );
    let with_ladder = files.iter().filter(|(_, s)| s.contains(LADDER)).count();
    assert_eq!(
        with_ladder,
        MEMBRANES.len() + 1,
        "as {} membranas mais a porta — se este numero mudou, uma membrana nasceu ou morreu, e \
         a fixture tem de dizer qual",
        MEMBRANES.len()
    );
}

//! **A `App` só perde campos** — a catraca NUMERADA da auditoria de arquitectura A9 (2026-09-12).
//!
//! A `App` é o agregado de topo da shell, e um campo solto nela é estado cujo DONO ninguém nomeou.
//! A A9 mediu **245** campos na base da `line/render-loop`, **54** deles `vec*`, e curou-os campo a
//! campo pelo ASSUNTO (o morto apagado · o tipo da shell descido para a crate · o resto num estado
//! de família). Ela desceu para **187**, com a família vetorial num campo só (`vec`). Nada disto
//! impedia o número de voltar a subir um campo de cada vez — cada um razoável sozinho, e é
//! exactamente assim que os 54 se juntaram.
//!
//! As DUAS metades da catraca, no molde do `file_loc_caps` e do `the_shell_only_shrinks`:
//! - **cresceu**: mais campos do que o tecto ⇒ o campo novo entra num estado de família
//!   (`vec`, `skeleton`, `physics`, `motion_shell`, …) — ou o commit que sobe o número diz porquê;
//! - **o tecto ficou para trás**: menos campos do que o tecto ⇒ quem tira um campo baixa o número
//!   no MESMO commit. A folga é **zero**: um campo é uma unidade inteira, e nenhum `rustfmt` o move.
//!
//! ⚠️ **Piso de população.** O gate lê o TEXTO da struct; um parser partido (a struct mudou de
//! nome, de visibilidade, de ficheiro) contaria zero e as duas metades leriam «desceu» sobre nada.
//! Por isso ele tem de encontrar, pelo NOME, campos que sabemos que existem — um de cada espécie de
//! dono.

/// O tecto MEDIDO em 2026-09-12, depois da A9. **Só desce**, e no mesmo commit que tira o campo.
const TETO_CAMPOS: usize = 187;

const APP_STATE: &str = include_str!("../../src/app_state.rs");

/// A âncora da struct no ficheiro.
const ANCORA: &str = "pub(crate) struct App {";

/// Campos que TÊM de ser vistos — um por espécie de dono: o GPU (`gfx`), os estados de família
/// (`vec`, `skeleton`, `physics`) e a fila global de undo. Um parser que não os ache está partido.
const TEM_DE_VER: &[&str] = &["gfx", "vec", "skeleton", "physics", "undo"];

/// Os nomes dos campos declarados na struct `App`, na ordem do ficheiro.
///
/// Conta só a profundidade 1 da struct (um tipo que se parte em várias linhas não conta duas vezes)
/// e ignora comentários e literais ao medir chavetas. Um campo com `#[cfg(..)]` CONTA: ele existe no
/// código, e desligá-lo numa build não o tira de quem lê a struct.
fn campos() -> Vec<&'static str> {
    let linhas: Vec<&str> = APP_STATE.lines().collect();
    let inicio = linhas
        .iter()
        .position(|l| l.starts_with(ANCORA))
        .unwrap_or_else(|| {
            panic!("a âncora `{ANCORA}` sumiu de app_state.rs — o gate não lê nada")
        });
    let mut nomes = Vec::new();
    let mut fundo = 1i32;
    for linha in &linhas[inicio + 1..] {
        let codigo = linha.split("//").next().unwrap_or("");
        if fundo == 1 && !linha.trim_start().starts_with("//") {
            let corpo = linha
                .strip_prefix("    ")
                .filter(|r| !r.starts_with(' '))
                .map(|r| {
                    r.strip_prefix("pub(crate) ")
                        .or_else(|| r.strip_prefix("pub "))
                        .unwrap_or(r)
                });
            if let Some(corpo) = corpo
                && let Some((nome, _)) = corpo.split_once(':')
                && !nome.is_empty()
                && nome
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            {
                nomes.push(nome);
            }
        }
        let sem_literais: String = {
            let mut dentro = false;
            codigo
                .chars()
                .filter(|&c| {
                    if c == '"' {
                        dentro = !dentro;
                    }
                    !dentro
                })
                .collect()
        };
        fundo += sem_literais.matches('{').count() as i32;
        fundo -= sem_literais.matches('}').count() as i32;
        if fundo == 0 {
            break;
        }
    }
    for nome in TEM_DE_VER {
        assert!(
            nomes.contains(nome),
            "o parser não viu o campo `{nome}` — ele está partido, e as duas metades da catraca \
             estariam a medir nada ({} campos lidos)",
            nomes.len()
        );
    }
    nomes
}

/// **Cresceu?** — um campo novo solto na `App`.
#[test]
fn the_app_does_not_grow_past_its_numbered_ceiling() {
    let n = campos().len();
    assert!(
        n <= TETO_CAMPOS,
        "a `App` tem {n} campos contra o tecto de {TETO_CAMPOS}. Um campo novo tem DONO: ponha-o no \
         estado da família do assunto dele (`vec`, `skeleton`, `physics`, `motion_shell`, …). ⛔ \
         Subir o número é a última saída, e o commit que o fizer diz porquê."
    );
}

/// **O tecto ficou para trás?** — tirou-se um campo e o número não desceu.
#[test]
fn the_ceiling_follows_the_app_down() {
    let n = campos().len();
    assert!(
        n >= TETO_CAMPOS,
        "a `App` tem {n} campos e o tecto ainda diz {TETO_CAMPOS} — baixe `TETO_CAMPOS` para {n} \
         neste commit. Uma catraca que não desce vira LICENÇA (CLAUDE.md §5.0)."
    );
}

/// **A família vetorial é UM campo** — o assunto que a A9 curou não volta a espalhar-se.
///
/// ⚠️ Mais estreito que o tecto, de propósito: um `vec_*` novo solto passaria pela metade «cresceu»
/// se no mesmo commit alguém tirasse outro campo qualquer. O prefixo diz o assunto, e o assunto tem
/// casa (`ph2d_app_vec::state::VecState`).
#[test]
fn the_vector_family_is_one_field_of_the_app() {
    let vec: Vec<&str> = campos()
        .into_iter()
        .filter(|n| n.starts_with("vec"))
        .collect();
    assert_eq!(
        vec,
        ["vec"],
        "campos da família vetorial soltos na `App`: {vec:?} — o lugar deles é o `VecState` \
         (`app.vec.<campo>`)"
    );
}

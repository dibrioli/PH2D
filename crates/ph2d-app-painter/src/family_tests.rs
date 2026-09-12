//! **Gates do `FAMILY`** — o que esta família declara à shell é o que ela de facto tem.
//!
//! ⚠️ São **três** defeitos diferentes, e por isso três gates: o `NIVEIS` mentir sobre o `match`
//! (o roteador promete um nível que não responde) · o `FAMILY` mentir sobre o `NIVEIS` (são dois
//! números e podem divergir) · e um roteador NOVO nascer sem se declarar (o modo de falha MUDO,
//! porque nada o obriga a aparecer numa lista escrita à mão).

use super::{FAMILY, impasto_smoke, line_smoke, mask_smoke, substrate_smoke, taper_smoke,
            wetpaint_smoke};

/// **O roteador do impasto responde por todo nível que promete — e só por eles.**
///
/// ⭐ Este é o único dos seis com mais de um nível, e a lei dele é mensurável **sem tocar no
/// ambiente**: o `match` vive na porta pura [`impasto_smoke::edge_for`]. *A variável de ambiente é
/// global ao processo, logo um gate que a escrevesse mediria o teste vizinho.*
///
/// **Mutação que deve sangrar:** pôr `NIVEIS = 1` (o `=2` deixa de ser prometido e a tela de 4096²
/// fica inalcançável pelo roteador) ou `NIVEIS = 3` (promete-se uma cena que abre a de 1024²).
#[test]
fn o_roteador_do_impasto_responde_por_todo_nivel_que_promete() {
    let base = impasto_smoke::edge_for(Some("1"));
    assert_eq!(
        base,
        impasto_smoke::edge_for(None),
        "o nível 1 tem de ser o mesmo que a ausência de valor — é o braço `_`"
    );
    // A ponta de CIMA: o nível `NIVEIS` tem de ser dele próprio, não do braço `_`.
    let topo = impasto_smoke::NIVEIS.to_string();
    assert_ne!(
        impasto_smoke::edge_for(Some(&topo)),
        base,
        "o `NIVEIS` diz {} mas o nível {topo} abre a MESMA tela que o 1 — ele promete uma cena \
         que o `match` não responde.",
        impasto_smoke::NIVEIS
    );
    // A ponta de BAIXO: o nível seguinte tem de cair no `_`.
    let acima = (impasto_smoke::NIVEIS + 1).to_string();
    assert_eq!(
        impasto_smoke::edge_for(Some(&acima)),
        base,
        "o nível {acima} responde por si — o `NIVEIS` está um degrau abaixo do `match`."
    );
}

/// **O `FAMILY` declara exactamente os `NIVEIS` que os roteadores contam.**
///
/// ⚠️ Sem isto, cada `NIVEIS` podia estar certo e o `FAMILY` declarar outro número: *são dois
/// sítios, e dois sítios divergem* (a mesma razão pela qual a `physics` tem o gate irmão deste).
#[test]
fn o_family_declara_os_niveis_que_os_roteadores_contam() {
    let esperado = [
        ("PH2D_IMPASTO_SMOKE", impasto_smoke::NIVEIS),
        ("PH2D_WETPAINT_SMOKE", wetpaint_smoke::NIVEIS),
        ("PH2D_MASK_SMOKE", mask_smoke::NIVEIS),
        ("PH2D_SUBSTRATE_SMOKE", substrate_smoke::NIVEIS),
        ("PH2D_TAPER_SMOKE", taper_smoke::NIVEIS),
        ("PH2D_LINE_SMOKE", line_smoke::NIVEIS),
    ];
    assert_eq!(FAMILY.key, "painter");
    assert_eq!(
        FAMILY.routers.len(),
        esperado.len(),
        "o `FAMILY` declara {} roteadores e os módulos contam {}",
        FAMILY.routers.len(),
        esperado.len()
    );
    for (env, niveis) in esperado {
        let r = FAMILY
            .routers
            .iter()
            .find(|r| r.env == env)
            .unwrap_or_else(|| panic!("o `FAMILY` não declara o roteador `{env}`"));
        assert_eq!(
            r.max_level, niveis,
            "o `FAMILY` diz que o `{env}` vai até {} e o módulo conta {niveis}",
            r.max_level
        );
    }
}

/// ⛔⛔ **Todo módulo que declara um `NIVEIS` está no `FAMILY`** — o gate do roteador ÓRFÃO.
///
/// # Por que um CENSO, e por que ele varre TUDO
///
/// Os dois gates acima comparam duas listas **escritas à mão**: se alguém acrescentar um sétimo
/// roteador e não o puser em nenhuma das duas, os dois ficam verdes e o dono nunca sabe que tem
/// uma cena. Este pergunta ao **directório**.
///
/// ⛔ **Varre todo `.rs` da crate, nunca por prefixo de nome** (`HOWTO` §2.7): um censo que
/// filtrasse `starts_with("…_smoke")` passaria a varrer zero no dia em que um roteador se chamasse
/// outra coisa, e `orfaos.is_empty()` sobre uma lista construída de zero ficheiros é
/// **trivialmente verdadeiro**. A marca de um roteador é o que ele DECLARA (`pub const NIVEIS`),
/// não como se chama.
///
/// ⚠️ **PISO DE POPULAÇÃO**: esta crate vai crescer muito (a família tem ~40 ficheiros e só os
/// roteadores chegaram), e um censo que deixasse de ver os ficheiros ficaria verde a medir nada.
#[test]
fn todo_roteador_declarado_esta_no_family() {
    /// Quantos ficheiros `.rs` a crate tem de ter para este censo valer alguma coisa.
    const FICHEIROS_MIN: usize = 7;
    /// Quantos roteadores existem — o mesmo número que o `FAMILY` declara.
    const ROTEADORES: usize = 6;

    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut vistos = 0usize;
    let mut roteadores: Vec<String> = Vec::new();
    let mut pilha = vec![src];
    while let Some(dir) = pilha.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                pilha.push(p);
                continue;
            }
            if p.extension().is_none_or(|x| x != "rs") {
                continue;
            }
            vistos += 1;
            let Ok(fonte) = std::fs::read_to_string(&p) else {
                continue;
            };
            // ⛔⛔ **Uma DECLARAÇÃO, nunca uma ocorrência** — e este ficheiro é a prova de que a
            // diferença importa. A primeira redacção perguntava `fonte.contains("pub const NIVEIS:
            // u32")` e acusou o PRÓPRIO `family_tests.rs`, que traz a agulha duas vezes: uma no
            // doc-comment que a explica e outra dentro do literal de string que a procura. É a
            // `HOWTO` §2.12 — *um censo textual que não separa PROSA de código mente nos dois
            // sentidos* — a morder o ficheiro escrito para a honrar.
            //
            // ⇒ a marca é a linha COMEÇAR pela declaração: um `///` ou um `if !fonte.contains(…)`
            // não começa por `pub const`, e nenhum item de Rust vive a meio de uma linha.
            let declara = fonte
                .lines()
                .any(|l| l.trim_start().starts_with("pub const NIVEIS: u32"));
            if !declara {
                continue;
            }
            // A `env` que este roteador lê — a chave pela qual o `FAMILY` o nomeia. Também aqui se
            // saltam as linhas de comentário: os docs destes ficheiros citam a variável em prosa.
            let Some(env) = fonte
                .lines()
                .filter(|l| !l.trim_start().starts_with("//"))
                .find_map(|l| {
                    let i = l.find("var_os(\"PH2D_")? + "var_os(\"".len();
                    let resto = &l[i..];
                    Some(resto[..resto.find('"')?].to_string())
                })
            else {
                panic!(
                    "{} declara `NIVEIS` e não lê nenhuma `var_os(\"PH2D_…\")` — um roteador sem \
                     porta de entrada",
                    p.display()
                );
            };
            roteadores.push(env);
        }
    }
    assert!(
        vistos >= FICHEIROS_MIN,
        "este censo varreu {vistos} ficheiros e esperava >= {FICHEIROS_MIN} — ele PERDEU O SUJEITO"
    );
    assert_eq!(
        roteadores.len(),
        ROTEADORES,
        "achei {} módulos a declarar `NIVEIS` e esperava {ROTEADORES}: {roteadores:?}",
        roteadores.len()
    );
    let declarados: Vec<&str> = FAMILY.routers.iter().map(|r| r.env).collect();
    for env in &roteadores {
        assert!(
            declarados.contains(&env.as_str()),
            "o roteador `{env}` declara `NIVEIS` e NÃO está no `FAMILY` — ele é órfão, e o dono \
             nunca vai saber que tem esta cena. Uma família registada declara os roteadores que tem."
        );
    }
}

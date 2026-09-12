//! **Os gates do roteador** — que ele responde por todo nível que promete, e por
//! nenhum que não promete.
//!
//! # ⛔⛔ Por que este ficheiro tinha de nascer com o `const FAMILY`
//!
//! O `crate::FAMILY` declara ao registo que esta família responde por
//! `PH2D_PHYSICS_SMOKE` até [`CENAS`]. Esse número é uma **afirmação sobre o
//! `match`** — e o `CLAUDE.md` §5.0 é explícito: *«o número da próxima cena de
//! smoke CONTA-SE lendo o roteador»*, nunca se escreve de memória. Aquela nota já
//! esteve parada em `97` com cenas até `102` noutro módulo.
//!
//! ⚠️ **E o gate de registo não o apanha:** ele conta ROTEADORES, não cenas — uma
//! família que declarasse `max_level: 4` com 118 cenas passaria por ele, e o dono
//! que escrevesse `=63` receberia a cena 1 em silêncio.

use super::*;

/// **O tecto é o que o `match` de facto tem** — contado do fonte, não de memória.
///
/// ⚠️ Ele lê o ficheiro IRMÃO (`smoke.rs`), e é isso que o torna uma medição em vez
/// de uma segunda cópia do número: se um braço novo entrar, este gate vê-o antes de
/// qualquer pessoa se lembrar da constante.
///
/// **Mutação que deve sangrar:** pôr `CENAS = 118` (ou `120`).
#[test]
fn o_tecto_declarado_e_o_maior_braco_do_match() {
    let src = include_str!("smoke.rs");
    let mut niveis: Vec<u32> = Vec::new();
    for l in src.lines() {
        let t = l.trim();
        let Some(resto) = t.strip_prefix('"') else {
            continue;
        };
        let Some((n, cauda)) = resto.split_once('"') else {
            continue;
        };
        if cauda.trim_start().starts_with("=>")
            && let Ok(n) = n.parse::<u32>()
        {
            niveis.push(n);
        }
    }
    // Controlo positivo: o parser não pode ler zero e o gate ficar verde por vácuo.
    assert!(
        niveis.len() > 100,
        "o parser achou {} braços — um censo que varre zero é trivialmente verde",
        niveis.len()
    );
    let maior = niveis.iter().copied().max().expect("há braços");
    assert_eq!(
        CENAS, maior,
        "o `CENAS` diz {CENAS} e o maior braço do `match` é {maior}. Este número é o \
         que o `crate::FAMILY` declara ao registo — corrija-o CONTANDO, nunca de memória."
    );
}

/// **Nenhum nível é reclamado duas vezes.**
///
/// ⚠️ O compilador apanha dois braços com o mesmo literal (`unreachable`), mas só
/// dentro do `match`; o que ele **não** apanha é o número escrito na mensagem de
/// uma cena. Este gate mede o `match`, que é a fonte.
#[test]
fn nenhum_nivel_e_reclamado_duas_vezes() {
    let src = include_str!("smoke.rs");
    let mut vistos: Vec<u32> = Vec::new();
    for l in src.lines() {
        let t = l.trim();
        if let Some(resto) = t.strip_prefix('"')
            && let Some((n, cauda)) = resto.split_once('"')
            && cauda.trim_start().starts_with("=>")
            && let Ok(n) = n.parse::<u32>()
        {
            vistos.push(n);
        }
    }
    let mut ordenados = vistos.clone();
    ordenados.sort_unstable();
    ordenados.dedup();
    assert_eq!(
        vistos.len(),
        ordenados.len(),
        "há níveis repetidos no roteador — o mesmo `PH2D_PHYSICS_SMOKE` corre duas cenas"
    );
}

/// **A env é lida DENTRO desta crate.**
///
/// ⭐ É a condição que faz o `crate::FAMILY` dizer a verdade: declarar
/// `PH2D_PHYSICS_SMOKE` e não a ler seria o registo a afirmar um alcance que a
/// família não tem — e o gate do registo conta roteadores, não leituras.
///
/// **Mutação que deve sangrar:** devolver `None` sempre.
#[test]
fn a_env_do_roteador_e_lida_aqui() {
    let pedido = armed_scene_in(|k| (k == "PH2D_PHYSICS_SMOKE").then(|| "63".to_owned()));
    assert_eq!(pedido.as_deref(), Some("63"));
    assert_eq!(
        armed_scene_in(|_| None),
        None,
        "sem a env, o roteador não arma nada"
    );
    assert!(
        crate::FAMILY.routers.iter().any(|r| r.env == ENV),
        "a env que o roteador lê é a que o registo declara"
    );
    // ⚠️ E a porta do PRODUTO passa o ambiente real: um `armed_scene` que devolvesse `None` deixaria
    // as três asserções acima verdes. `include_str!` falha a COMPILAR se o ficheiro mudar de nome
    // (HOWTO §2.6), em vez de ficar verde a ler nada.
    assert!(
        include_str!("smoke.rs").contains("armed_scene_in(|k| std::env::var(k).ok())"),
        "o `armed_scene` do produto tem de ler o ambiente do processo pela porta injectável"
    );
}

/// **O que o registo declara é o que o roteador tem** — as duas pontas atadas.
///
/// ⚠️ Sem isto, `CENAS` podia estar certo e o `FAMILY` declarar outro número: são
/// dois sítios, e o gate acima só mede um deles.
#[test]
fn o_registo_declara_o_tecto_do_roteador() {
    let r = crate::FAMILY
        .routers
        .iter()
        .find(|r| r.env == "PH2D_PHYSICS_SMOKE")
        .expect("a família declara o roteador do smoke de física");
    assert_eq!(r.max_level, CENAS);
}

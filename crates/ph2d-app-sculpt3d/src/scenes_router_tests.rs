//! **OS GATES DO ROTEADOR** — que ele responde por todo nível que promete, e por nenhum que não.
//!
//! # ⛔⛔ Por que este ficheiro tinha de nascer com o roteador no `const FAMILY`
//!
//! O [`crate::FAMILY`] declara ao registo que esta família responde por `PH2D_SCULPT3D_SMOKE`
//! até [`CENAS`]. Esse número é uma **afirmação sobre os predicados** — e o `CLAUDE.md` §5.0 é
//! explícito: *«o número da próxima cena de smoke CONTA-SE lendo o roteador»*, nunca se escreve
//! de memória. Aquela nota já esteve parada em `97` com cenas até `102` noutro módulo.
//!
//! ⚠️ **E o gate do registo não o apanha:** ele conta ROTEADORES, não cenas — uma família que
//! declarasse `max_level: 4` com 39 cenas passaria por ele, e o dono que escrevesse `=36`
//! receberia a esfera padrão em silêncio.
//!
//! # ⚠️ Este roteador é DISPERSO, e é por isso que o censo é o que é
//!
//! A física e a modelagem têm um `match` num ficheiro só, e o gate delas lê esse ficheiro por
//! `include_str!`. Aqui **cada cena tem o próprio predicado, no próprio ficheiro** — e a razão
//! está escrita no doc do [`crate::scenes::smoke_armed`]: a enumeração central que aqui viveu
//! **apodreceu no dia previsível** (a cena `=14` nasceu completa e o app abriu com o canvas em
//! branco, porque ninguém lhe acrescentou o `"14"` à lista). ⛔ Reconstruir a tabela para
//! facilitar o gate seria refazer trabalho já pago com o defeito dentro.
//!
//! ⇒ o censo varre **TODO** `.rs` da crate. ⛔⛔ **Nunca por prefixo de nome** (`scenes_*.rs`),
//! que é a armadilha §2.7 do HOWTO e cujo modo de falha é **MUDO**: renomeie um ficheiro e a
//! varredura passa a ver zero, sobre a qual toda asserção de conjunto é trivialmente verdadeira.
//! E ele tem **piso de população nas duas grandezas** — ficheiros varridos e níveis achados.

/// A forma que um predicado de cena tem, e só ela.
const AGULHA: &str = r#"std::env::var("PH2D_SCULPT3D_SMOKE").ok().as_deref() == Some(""#;

/// **Os níveis que os predicados desta crate de facto reclamam.**
///
/// ⚠️ **Varre em RUNTIME a partir do `CARGO_MANIFEST_DIR`**, e não por `include_str!`: a agulha
/// vive em ~15 ficheiros, e um `include_str!` por ficheiro seria a lista escrita à mão que este
/// módulo já provou apodrecer. O preço é o gémeo de runtime do §2.x — ele só falha **quando o
/// teste corre** —, e é por isso que o piso de população está aqui dentro e não à volta.
///
/// ⚠️⚠️ **As linhas de comentário são DESCARTADAS** (§2.12): este próprio ficheiro escreve a
/// agulha em prosa, e um censo que não separe prosa de código lê a explicação da cura como
/// evidência do defeito — ele mente na direcção cara.
fn niveis_reclamados() -> Vec<u32> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut ficheiros = 0usize;
    let mut niveis = Vec::new();
    let entradas = std::fs::read_dir(&dir).expect("o `src/` desta crate existe");
    for e in entradas.flatten() {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        ficheiros += 1;
        let src = std::fs::read_to_string(&p).expect("ficheiro legível");
        for l in src.lines() {
            let t = l.trim_start();
            // ⚠️ prosa fora: `//`, `///` e `//!` escrevem a agulha e não a EXECUTAM.
            if t.starts_with("//") {
                continue;
            }
            let Some(resto) = t.split(AGULHA).nth(1) else {
                continue;
            };
            if let Some((n, _)) = resto.split_once('"')
                && let Ok(n) = n.parse::<u32>()
            {
                niveis.push(n);
            }
        }
    }
    // ⛔ **O PISO, as duas metades.** Sem a primeira, uma varredura que achasse zero ficheiros
    // devolveria uma lista vazia e todo `assert_eq!` sobre ela seria sobre o vazio. Sem a
    // segunda, um ficheiro só bastava.
    assert!(
        ficheiros >= 100,
        "controlo positivo: a varredura achou {ficheiros} ficheiros `.rs` em `{}` — o censo \
         está a medir a árvore errada, e uma lista vazia lê-se como aprovada",
        dir.display()
    );
    assert!(
        niveis.len() >= 30,
        "controlo positivo: achei {} predicados de cena — a AGULHA deixou de casar, e é o \
         modo de falha mudo do §2.7 a acontecer",
        niveis.len()
    );
    niveis
}

/// **O tecto declarado é o maior nível de facto reclamado.**
///
/// **Mutação que deve sangrar:** pôr `CENAS = 38` (ou `40`).
#[test]
fn o_tecto_declarado_e_o_maior_nivel_reclamado() {
    let niveis = niveis_reclamados();
    let maior = *niveis.iter().max().expect("o piso garante que há níveis");
    assert_eq!(
        crate::scenes::CENAS,
        maior,
        "`scenes::CENAS` diz {} e o maior nível que um predicado desta crate reclama é {maior} \
         — o dono que escrevesse `PH2D_SCULPT3D_SMOKE={maior}` receberia a esfera padrão em \
         silêncio, ou o registo prometeria uma cena que não existe",
        crate::scenes::CENAS
    );
}

/// **Nenhum nível é reclamado duas vezes** — dois predicados no mesmo `=N` correm duas cenas.
#[test]
fn nenhum_nivel_e_reclamado_duas_vezes() {
    let niveis = niveis_reclamados();
    let mut ordenados = niveis.clone();
    ordenados.sort_unstable();
    ordenados.dedup();
    assert_eq!(
        niveis.len(),
        ordenados.len(),
        "há níveis repetidos entre os predicados — o mesmo `PH2D_SCULPT3D_SMOKE` arma duas cenas"
    );
}

/// **A env é lida DENTRO desta crate.**
///
/// ⭐ É a condição que faz o [`crate::FAMILY`] dizer a verdade. Enquanto a leitura vivesse na
/// shell, o registo diria que a família responde por uma env que ela não vê.
///
/// **Mutação que deve sangrar:** devolver `None` sempre.
#[test]
fn a_env_do_roteador_e_lida_aqui() {
    use crate::scenes::{ENV, armed_scene_in};
    let pedido = armed_scene_in(|k| (k == "PH2D_SCULPT3D_SMOKE").then(|| "36".to_owned()));
    assert_eq!(pedido.as_deref(), Some("36"));
    assert_eq!(
        armed_scene_in(|_| None),
        None,
        "sem a env, o roteador não arma nada"
    );
    assert!(
        crate::FAMILY.routers.iter().any(|r| r.env == ENV),
        "a env que o roteador lê é a que o registo declara"
    );
    // ⚠️ E as duas portas do PRODUTO passam o ambiente real — `include_str!` falha a COMPILAR se o
    // ficheiro mudar de nome (HOWTO §2.6).
    let fonte = include_str!("scenes.rs");
    assert!(
        fonte.contains("armed_scene_in(|k| std::env::var(k).ok())"),
        "o `armed_scene` do produto tem de ler o ambiente do processo pela porta injectável"
    );
    assert!(
        fonte.contains("arms(std::env::var(ENV).ok().as_deref())"),
        "o `smoke_armed` do produto pergunta à MESMA lei pura que o gate da shell interroga"
    );
}

/// **O que o registo declara é o que o roteador tem** — as duas pontas atadas.
///
/// ⚠️ Sem isto, `CENAS` podia estar certo e o `FAMILY` declarar outro número: são dois sítios, e
/// o gate acima só mede um deles.
#[test]
fn o_registo_declara_o_tecto_do_roteador() {
    let r = crate::FAMILY
        .routers
        .iter()
        .find(|r| r.env == "PH2D_SCULPT3D_SMOKE")
        .expect("a família declara o roteador do smoke da escultura");
    assert_eq!(r.max_level, crate::scenes::CENAS);
}

/// ⚠️⚠️ **UM roteador, e a família lê ~32 variáveis `PH2D_*`.**
///
/// A esmagadora maioria delas é **DIAGNÓSTICO de retopologia** — interruptores de bissecção que
/// o dono não alcança e que não roteiam cena nenhuma. Declarar um deles como roteador diria ao
/// dono que ele tem uma cena para ver. ⇒ só a forma `PH2D_*_SMOKE` entra no `FAMILY`.
#[test]
fn so_o_roteador_do_smoke_e_declarado() {
    assert_eq!(crate::FAMILY.routers.len(), 1);
    for r in crate::FAMILY.routers {
        assert!(
            r.env.starts_with("PH2D_") && r.env.ends_with("_SMOKE"),
            "`{}` não é um roteador de smoke — as envs de diagnóstico desta família \
             (`PH2D_RETOPO_*`, `PH2D_DUMP*`, `PH2D_BENCH_*`, `PH2D_TIP_ALIGN`, `PH2D_ISO_*`, \
             `PH2D_GRIDMAP_*`) não se declaram ao registo",
            r.env
        );
    }
}

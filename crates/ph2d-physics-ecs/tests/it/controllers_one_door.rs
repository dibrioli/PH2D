//! **UMA PORTA para os controladores** — o censo que impede o defeito de voltar uma terceira vez.
//!
//! # ⛔⛔ Porque este gate existe, e porque um gate de COMPORTAMENTO não chegava
//!
//! Esta ponte anda o relógio por **dois** laços escritos à mão: o da frente ([`dispatch`]) e o de
//! replay ([`rewind`]). Três vezes um controlador foi ligado ao primeiro e não ao segundo:
//!
//! | wave | controlador | como se soube |
//! |---|---|---|
//! | W7 | `drive_players` | report — o personagem caía pelos tiques replayados |
//! | TOP-20 #13 | `drive_topdown` | este gate |
//! | TOP-20 #14 | `drive_projectiles` | **report do dono** (*«comportamento diferente a cada rewind»*) |
//!
//! ⚠️ O gate de comportamento irmão ([`crate::rewind_controllers`]) mede os controladores que
//! EXISTEM hoje. Ele não pode reprovar sobre um **quarto** que ninguém escreveu ainda — e é
//! exactamente aí que a família se repete. *Uma lei escrita em dois sítios ainda não é uma lei;
//! só uma PORTA é.*
//!
//! [`dispatch`]: ph2d_physics_ecs::PhysicsBridge::dispatch
//! [`rewind`]: ph2d_physics_ecs::PhysicsBridge::dispatch

use std::fs;

/// Os `drive_*` que são **um tique de um controlador** — o assunto deste censo.
///
/// ⚠️ **Não** entram aqui os que dirigem coisas que a CENA autora (`drive_kinematic`,
/// `drive_joint_params`): esses não têm memória entre tiques nem entram no anel, e o laço de
/// replay chama-os por outra razão (a pose autorada daquele tique).
const CONTROLADORES: &[&str] = &["drive_players", "drive_topdown", "drive_projectiles"];

/// O ficheiro da ponte, sem comentários.
///
/// ⚠️⚠️ **Tirar os comentários não é higiene, é a diferença entre medir e não medir:** este
/// módulo explica a cura POR ESCRITO em cinco sítios, e um censo textual cru leria cada explicação
/// como uma chamada. É a armadilha que a `line/app-physics` pagou ao varrer `\bApp\b` e acusar 93
/// ficheiros de 133.
fn codigo(rel: &str) -> String {
    let path = format!("{}/src/bridge/{rel}", env!("CARGO_MANIFEST_DIR"));
    let bruto = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    bruto
        .lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Os nomes dos ficheiros de `src/bridge/`, em ordem.
fn familia() -> Vec<String> {
    let dir = format!("{}/src/bridge", env!("CARGO_MANIFEST_DIR"));
    let mut v: Vec<String> = fs::read_dir(&dir)
        .expect("a ponte tem de ter um directorio")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".rs"))
        .collect();
    v.sort();
    // ⚠️ **Piso de POPULAÇÃO** — sem ele, renomear o directório faz o censo varrer zero ficheiros
    // e `acusados.is_empty()` fica trivialmente verdadeiro (a §2.7 do HOWTO: a espécie de gate
    // partido que fica VERDE).
    assert!(
        v.len() > 20,
        "a ponte tem dezenas de ficheiros; o censo achou {}: {v:?}",
        v.len()
    );
    v
}

/// ⭐⭐⭐ **Só a porta única chama um controlador.**
///
/// **Mutação que deve sangrar:** pôr `self.drive_topdown(sim);` de volta no laço do `dispatch.rs`.
#[test]
fn os_dois_lacos_dirigem_os_controladores_pela_mesma_porta() {
    let mut acusados = Vec::new();
    for f in familia() {
        if f == "controllers.rs" {
            continue;
        }
        let src = codigo(&f);
        for c in CONTROLADORES {
            if src.contains(&format!("self.{c}(")) {
                acusados.push(format!("{f} chama self.{c}(…)"));
            }
        }
    }
    assert!(
        acusados.is_empty(),
        "um controlador é dirigido fora da porta única (`bridge::controllers`), que é exactamente \
         como o `drive_topdown` e o `drive_projectiles` ficaram de fora do laço de replay — \
         report do dono, 2026-09-15: {acusados:?}"
    );

    // E a outra metade, sem a qual a de cima passa com a porta VAZIA: ela chama os três.
    let porta = codigo("controllers.rs");
    for c in CONTROLADORES {
        assert!(
            porta.contains(&format!("self.{c}(")),
            "a porta única não chama `{c}` — os {} controladores são o assunto dela",
            CONTROLADORES.len()
        );
    }
}

/// ⭐⭐ **E os DOIS laços chamam-na** — a metade que impede a porta de ficar órfã.
///
/// ⚠️ Sem este gate, apagar a chamada do laço de replay deixa o censo acima **verde**: ninguém
/// estaria a dirigir um controlador fora da porta, porque ninguém estaria a dirigir nada.
/// *Uma porta sem chamador e uma lei ausente produzem o mesmo app.*
///
/// **Mutação que deve sangrar:** apagar `self.drive_controllers(sim);` do `rewind.rs`.
#[test]
fn a_porta_unica_e_chamada_pelos_dois_lacos_que_andam_o_relogio() {
    for laco in ["dispatch.rs", "rewind.rs"] {
        assert!(
            codigo(laco).contains("self.drive_controllers("),
            "o laço de `{laco}` não chama a porta dos controladores — um scrub e um play \
             deixariam de ser a mesma simulação"
        );
    }
}

/// ⭐⭐ **A memória de voo é ESQUECIDA ao reconstruir do repouso** — a metade do report do dono que
/// não é sobre o laço.
///
/// ⚠️ Ela vive aqui, ao lado das irmãs, porque o `rebuild_from_rest` limpa **três** memórias e
/// esquecer uma é mudo: o mundo volta ao repouso e o controlador continua a corrida anterior.
/// A do projéctil era a que faltava (`launched` ⇒ a bala nunca re-nasce a ler o ângulo autorado).
///
/// **Mutação que deve sangrar:** apagar `self.projectile_state.clear();` do `rebuild_from_rest`.
#[test]
fn reconstruir_do_repouso_esquece_as_tres_memorias_de_controlador() {
    let src = codigo("rewind.rs");
    for memoria in ["player_state", "topdown_state", "projectile_state"] {
        assert!(
            src.contains(&format!("self.{memoria}.clear()")),
            "o `rebuild_from_rest` não esquece a `{memoria}` — reconstruir do repouso É o tique 0, \
             e no tique 0 ninguém correu, ninguém deslizou e nenhuma bala nasceu"
        );
    }
}

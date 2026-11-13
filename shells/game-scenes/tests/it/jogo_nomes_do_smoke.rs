//! Os nomes de `PH2D_GAME_SMOKE_FOTO` / `PH2D_GAME_SMOKE_DRIVE`: a lista de cada leitor é o que o `match` dele
//! aceita (censo pela fonte), e o arranque recusa ALTO o nome que nenhum aceita.

use ph2d_app_game::smoke::{CONDUCOES, nomes::LEITORES, recusa_in};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn raiz() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Os literais dos braços `"x" =>` da função `assinatura` em `ficheiro` (até ao `}` da coluna 0).
fn bracos(ficheiro: &str, assinatura: &str) -> BTreeSet<String> {
    let s = std::fs::read_to_string(raiz().join(ficheiro)).expect(ficheiro);
    let i = s
        .find(assinatura)
        .unwrap_or_else(|| panic!("`{assinatura}` em {ficheiro}"));
    let corpo = &s[i..i + s[i..].find("\n}\n").expect("o fim da função")];
    let fora: BTreeSet<String> = corpo
        .lines()
        .filter_map(|l| {
            l.trim()
                .strip_prefix('"')?
                .split_once("\" =>")
                .map(|(n, _)| n.to_owned())
        })
        .collect();
    assert!(
        !fora.is_empty(),
        "controlo positivo: `{assinatura}` em {ficheiro} sem braços — o censo mede o nada"
    );
    fora
}

/// ⭐ Cada leitor: todo braço do `match` está na lista dele, e todo nome da lista o leitor aceita. (Mutação: tirar
/// um nome de uma lista ⇒ RED.)
#[test]
fn a_lista_de_cada_leitor_e_o_que_o_match_dele_aceita() {
    let fontes = [
        (
            "crates/ph2d-physics-scene/src/fotos_dos_cartoes.rs",
            "pub fn caso(",
            "fisica_",
        ),
        (
            "crates/ph2d-game-smoke-fotos/src/foto.rs",
            "pub fn foto_de(",
            "",
        ),
        (
            "crates/ph2d-physics-scene/src/cena_topdown.rs",
            "pub fn foto(",
            "",
        ),
        (
            "crates/ph2d-sprite-anim-scene/src/lib.rs",
            "pub fn foto_de(",
            "",
        ),
        (
            "crates/ph2d-physics-scene/src/cena_projectile.rs",
            "pub fn foto(",
            "",
        ),
        (
            "crates/ph2d-app-game/src/smoke.rs",
            "pub fn conducao_in(",
            "",
        ),
    ];
    let listas = LEITORES
        .iter()
        .map(|(n, a)| (*n, *a))
        .chain([(CONDUCOES, conduz as fn(&str) -> bool)]);
    let mut todos = BTreeSet::new();
    for ((ficheiro, assinatura, prefixo), (lista, aceita)) in fontes.iter().zip(listas) {
        let nomes: BTreeSet<&str> = lista.split_whitespace().collect();
        for b in bracos(ficheiro, assinatura) {
            assert!(
                nomes.contains(format!("{prefixo}{b}").as_str()),
                "{ficheiro}: «{prefixo}{b}» falta na lista"
            );
        }
        for n in &nomes {
            assert!(
                aceita(n),
                "{ficheiro}: «{n}» está na lista e o leitor não o aceita"
            );
            assert!(
                lista == CONDUCOES || todos.insert(*n),
                "«{n}» em duas listas"
            );
        }
    }
}

fn conduz(n: &str) -> bool {
    ph2d_app_game::smoke::conducao_in(|_| Some(n.to_owned())).is_some()
}

/// Só estes ficheiros leem `PH2D_GAME_SMOKE_FOTO`: um leitor novo entra em `LEITORES`, ou o censo não o vê.
#[test]
fn so_os_leitores_do_censo_leem_a_variavel() {
    let mut achados = BTreeSet::new();
    let mut pilha = vec![raiz().join("crates"), raiz().join("shells")];
    while let Some(d) = pilha.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() && p.file_name().is_some_and(|n| n != "target") {
                pilha.push(p);
            } else if p.extension().is_some_and(|x| x == "rs")
                && std::fs::read_to_string(&p)
                    .is_ok_and(|s| s.contains("var(\"PH2D_GAME_SMOKE_FOTO\")"))
            {
                achados.insert(p.strip_prefix(raiz()).unwrap_or(&p).display().to_string());
            }
        }
    }
    let esperados = [
        "crates/ph2d-game-smoke-fotos/src/foto.rs",
        "crates/ph2d-physics-scene/src/cena_projectile.rs",
        "crates/ph2d-physics-scene/src/cena_topdown.rs",
        "crates/ph2d-sprite-anim-scene/src/lib.rs",
    ];
    assert_eq!(achados, esperados.iter().map(|s| (*s).to_owned()).collect());
}

/// ⭐ «player_pulo» (o nome sem o prefixo da família) recusa, com o nome, a sugestão e a lista. (Mutação: desligar
/// a recusa ⇒ RED.)
#[test]
fn um_nome_desconhecido_recusa_com_a_sugestao() {
    let m = recusa_in(|k| (k == "PH2D_GAME_SMOKE_FOTO").then(|| "player_pulo".to_owned()))
        .expect("recusa");
    assert!(m.contains("PH2D_GAME_SMOKE_FOTO=«player_pulo»"), "{m}");
    assert!(m.contains("Quis dizer «fisica_player_pulo»?"), "{m}");
    assert!(
        m.contains("topdown_more") && m.contains("animado_vazio") && m.contains("camera_larga"),
        "{m}"
    );
    let d = recusa_in(|k| (k == "PH2D_GAME_SMOKE_DRIVE").then(|| "edtor".to_owned()))
        .expect("recusa o condutor");
    assert!(d.contains("Quis dizer «editor»?"), "{d}");
}

/// Sem as variáveis (ou vazias), e com cada nome válido, nada muda.
#[test]
fn sem_a_variavel_ou_com_nome_valido_nada_muda() {
    assert_eq!(recusa_in(|_| None), None);
    assert_eq!(recusa_in(|_| Some("  ".to_owned())), None);
    for (lista, _) in LEITORES {
        for n in lista.split_whitespace() {
            assert_eq!(
                recusa_in(|k| (k == "PH2D_GAME_SMOKE_FOTO").then(|| n.to_owned())),
                None,
                "{n}"
            );
        }
    }
    assert_eq!(
        recusa_in(|k| (k == "PH2D_GAME_SMOKE_DRIVE").then(|| "editor".to_owned())),
        None
    );
}

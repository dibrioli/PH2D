//! ⛔⛔ **O APAGAR pergunta se a peça veio da RECEITA, e pergunta-o à PORTA** (report do Enio,
//! 2026-09-05).
//!
//! # O que este gate compra, e por que é textual
//!
//! O caminho do apagar vive dentro do `hierarchy::dispatch`, cuja assinatura tem ~35 argumentos —
//! um teste de integração ali seria uma montagem maior do que a lei que ele mede, e provaria que
//! **este** caminho pergunta, nunca que **não existe um segundo** que não pergunta. *A ausência de
//! uma segunda escrita não se mede correndo o caminho certo.*
//!
//! ⛔ **Ele descasca comentários antes de varrer** — um censo textual que não separa prosa de
//! código mente nos dois sentidos, e esta linha já o pagou.
//!
//! # ⚠️ A lei que ele defende
//!
//! Sem a guarda o `despawn` passava e o passe estrutural **re-materializava** a peça no quadro
//! seguinte, com a pose do MESTRE — a edição do artista naquela peça desaparecia em silêncio. O
//! mecanismo está medido em `a_piece_deleted_behind_the_guard_comes_back_wearing_the_masters_pose`.

use std::path::Path;

fn code_of(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    let body = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    body.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **O apagar da Hierarquia consulta a porta antes de despawnar.**
///
/// **Mutação que deve sangrar:** tirar o `partition` e voltar a despawnar `wanted` inteiro.
#[test]
fn the_delete_asks_the_door_before_despawning() {
    let body = code_of("render_loop/hierarchy.rs");
    assert!(
        body.contains("is_a_recipe_given_piece"),
        "o apagar deixou de perguntar se a peca veio da receita — o passe volta a ressuscita'-la"
    );
    let ask = body
        .find("is_a_recipe_given_piece")
        .expect("a pergunta esta' la'");
    let despawn = body
        .find("sim.world_mut().despawn(entity)")
        .expect("o despawn do apagar");
    assert!(
        ask < despawn,
        "a pergunta corre DEPOIS do despawn — a peca ja' morreu quando alguem se lembra de perguntar"
    );
}

/// ⭐⭐ **E o gesto tem VOZ quando recusa** — a mesma lei do menu dos verbos: um item que come o
/// clique em silêncio é pior que um ausente, e foi por falta de voz que o report existiu.
#[test]
fn the_refusal_says_where_to_do_it_instead() {
    let body = code_of("render_loop/hierarchy.rs");
    assert!(
        body.contains("delete it in the component"),
        "a recusa nao diz ONDE fazer — o artista fica com um Delete que nao faz nada"
    );
}

/// ⛔⛔ **A porta responde a UMA pergunta, e a irmã responde a OUTRA.**
///
/// `belongs_to_an_instance` = *«estou DENTRO de uma cópia?»* (o que o `make_master` precisa: um
/// `MasterRoot` a meio de uma cópia viva encurta a sub-árvore de edição venha a peça de onde vier).
/// `is_a_recipe_given_piece` = *«a receita DEU isto?»*, que exclui o que o artista pendurou lá
/// dentro.
///
/// ⚠️ **Eu colapsei as duas em 2026-09-05** e o gate `only_a_piece_the_recipe_gave_is_refused_…`
/// reprovou na primeira corrida: um *Add Child* dentro de uma cópia passou a ser inapagável. O que
/// as separa é **o ELO**, e é essa linha que este censo defende.
#[test]
fn the_narrow_door_still_asks_for_the_link() {
    let body = code_of("instance_verbs_walk.rs");
    let door = body
        .find("fn is_a_recipe_given_piece")
        .expect("a porta estreita");
    let tail = &body[door..];
    assert!(
        tail.contains("get::<InstanceOf>(entity).is_none()"),
        "a porta estreita deixou de exigir o ELO — ela voltou a ser a irma larga, e um Add Child \
         dentro de uma copia fica inapagavel"
    );
}

//! Os gates do blob da taxonomia ([`super`]).

use ph2d_asset_index::{AssetRef, CatalogTree};

/// ⭐⭐ **A ida-e-volta é EXACTA** — os catálogos, os caminhos e as atribuições das DUAS famílias.
///
/// ⚠️ **O oráculo é a árvore inteira**, e não uma contagem: um round-trip que perdesse os caminhos
/// e mantivesse os ids passaria numa contagem, e o artista reabriria o projecto com as gavetas sem
/// nome.
///
/// **Mutação que deve sangrar:** gravar `Vec::new()` nas atribuições.
#[test]
fn a_saved_taxonomy_reopens_exactly_as_it_was() {
    let mut t = CatalogTree::new();
    let heroes = t.create("Personagens/Heróis");
    let props = t.create("Cenário/Props");
    let prefab = AssetRef::Component { stable_id: 7 };
    let image = AssetRef::Texture { asset: [9; 32] };
    t.assign(prefab, heroes);
    t.assign(image, props);

    let back = super::restore(&super::collect(&t));
    assert_eq!(back, t, "a taxonomia não sobreviveu à ida-e-volta");
    assert_eq!(back.catalog_of(&prefab), Some(heroes));
    assert_eq!(back.catalog_of(&image), Some(props));
}

/// ⚠️ **Determinístico** (HR-5): dois saves da mesma taxonomia dão os MESMOS bytes.
#[test]
fn two_saves_of_the_same_taxonomy_are_byte_identical() {
    let mut t = CatalogTree::new();
    t.create("B");
    t.create("A/Z");
    let a = t.create("A");
    t.assign(AssetRef::Component { stable_id: 1 }, a);
    assert_eq!(super::collect(&t), super::collect(&t));
}

/// Um projecto sem taxonomia nenhuma grava vazio e volta vazio — sem erro e sem ruído.
#[test]
fn an_empty_taxonomy_round_trips_as_empty() {
    let t = CatalogTree::new();
    assert!(super::restore(&super::collect(&t)).is_empty());
    assert!(super::restore(&[]).is_empty());
}

/// ⛔ **Um blob ilegível não estoura e não fica em silêncio** — ele devolve vazio e diz. *Um
/// projecto que abrisse sem catálogos sem uma linha de log faria o artista concluir que o trabalho
/// de arrumação se perdeu, sem nada a que agarrar.*
#[test]
fn an_unreadable_blob_opens_empty_instead_of_exploding() {
    assert!(super::restore(&[0xFF, 0xFF, 0xFF, 0xFF, 0xFF]).is_empty());
}

/// ⭐ **O `next_id` sobrevive ao ficheiro** — um catálogo criado depois de abrir não pode sentar-se
/// em cima de um carregado, senão os assets dele apareceriam dentro do novo.
///
/// **Mutação que deve sangrar:** o `restore` da árvore repor `next_id: 1`.
#[test]
fn a_reopened_taxonomy_never_hands_out_an_id_it_already_has() {
    let mut t = CatalogTree::new();
    let a = t.create("A");
    let b = t.create("B");
    let mut back = super::restore(&super::collect(&t));
    let novo = back.create("Novo");
    assert_ne!(novo, a);
    assert_ne!(novo, b);
}

/// ⭐⭐⭐ **QUANTO CUSTA pôr a taxonomia na captura do undo** — a medição que a decisão de a manter
/// fora do `ProjectState` nunca fez.
///
/// ⚠️ O doc deste módulo AFIRMAVA que metê-la ali *«faria toda renomeação de gaveta reescrever o
/// snapshot do mundo inteiro»*. Isso é falso desde a F2: a captura do mundo é **incremental** e
/// custa o tamanho da EDIÇÃO. O que uma taxonomia na captura custa é o `collect` dela, por quadro
/// com input — e é isso que este número mede.
///
/// Corra com:
/// `cargo test -p ph2d-host-desktop --bins measure_catalog_capture_cost -- --ignored --nocapture`
#[test]
#[ignore = "medição, não gate — imprime a tabela"]
fn measure_catalog_capture_cost() {
    use std::time::Instant;

    println!("catálogos  atribuições  bytes    collect(µs)  % de um quadro de 16,7 ms");
    for (n_cat, n_asg) in [(4usize, 20usize), (20, 200), (50, 2_000), (200, 10_000)] {
        let mut t = CatalogTree::new();
        let ids: Vec<_> = (0..n_cat)
            .map(|i| t.create(&format!("Familia {}/Gaveta {i}", i % 8)))
            .collect();
        for i in 0..n_asg {
            let mut asset = [0u8; 32];
            asset[..8].copy_from_slice(&(i as u64).to_le_bytes());
            t.assign(AssetRef::Texture { asset }, ids[i % ids.len()]);
        }
        // Aquece, depois mede a mediana de 32 corridas.
        let mut us: Vec<f64> = Vec::new();
        for _ in 0..32 {
            let t0 = Instant::now();
            let bytes = crate::project_catalogs::collect(&t);
            us.push(t0.elapsed().as_secs_f64() * 1e6);
            std::hint::black_box(bytes);
        }
        us.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let med = us[us.len() / 2];
        let bytes = crate::project_catalogs::collect(&t).len();
        println!(
            "{n_cat:>9}  {n_asg:>11}  {bytes:>7}  {med:>11.1}  {:>6.3}%",
            med / 16_700.0 * 100.0
        );
    }
}

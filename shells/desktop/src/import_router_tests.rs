//! **Os gates do roteador de import** ([`super`]) — a lei que impede as duas portas de divergirem.
//!
//! ⚠️ O defeito que os gerou (Enio, 2026-08-23: *«`.ase` não aparece no dialog de import»*) não era
//! o `.ase`: era o diálogo ter uma lista **escrita à mão** ao lado de um predicado. O `.gif`, o
//! `.psd` e o `.ora` já estavam invisíveis lá **há meses** pelo mesmo mecanismo, e ninguém tinha
//! reparado — *um gate sobre a coerência das duas metades é o que impede a próxima extensão de
//! desaparecer em silêncio*.

use super::*;

fn p(s: &str) -> PathBuf {
    PathBuf::from(s)
}

/// **O DIÁLOGO OFERECE EXACTAMENTE O QUE O ROTEADOR ACEITA** — nos dois sentidos.
///
/// ⚠️ As duas metades juntas, de propósito: «tudo o que o diálogo oferece é importável» sozinho fica
/// verde num diálogo que não oferece nada, e «tudo o que é importável está no diálogo» sozinho fica
/// verde num diálogo que oferece o disco inteiro.
///
/// **Mutação que deve sangrar:** tirar uma extensão da linha «All supported».
#[test]
fn the_dialog_offers_exactly_what_the_router_accepts() {
    let filters = dialog_filters();
    let all: Vec<&str> = filters
        .iter()
        .find(|(label, _)| label == "All supported")
        .map(|(_, exts)| exts.clone())
        .expect("a primeira linha tem de ser o «tudo»");

    // 1. Tudo o que o diálogo oferece, o roteador aceita — perguntado pela função que o produto
    //    de facto chama, e não por um predicado que só os gates usam.
    for ext in &all {
        let (_, _, unknown) = partition_importables(&[p(&format!("/a/file.{ext}"))]);
        assert!(
            unknown.is_empty(),
            "o dialogo oferece .{ext} e o roteador manda-o para «nao sei ler isto»"
        );
    }
    // 2. E tudo o que o roteador aceita, o diálogo oferece — medido sobre as DUAS fontes, que são
    //    as únicas que o roteador consulta.
    for ext in crate::ase_import::ASE_EXTENSIONS
        .iter()
        .chain(ph2d_asset::SUPPORTED_IMAGE_EXTENSIONS)
    {
        assert!(
            all.contains(ext),
            ".{ext} e' importavel e NAO aparece no dialogo — foi assim que o .ase (e o .gif antes \
             dele) ficaram invisiveis"
        );
    }
    // 3. E o que não é importável fica de fora dos dois.
    for ext in ["txt", "mp3", "blend", "rs"] {
        let (_, _, unknown) = partition_importables(&[p(&format!("/a/x.{ext}"))]);
        assert_eq!(unknown.len(), 1, ".{ext} nao devia ser importavel");
        assert!(!all.contains(&ext));
    }
}

/// **O `.ase` está lá, pelo nome** — o pedido do Enio, afirmado sem rodeios.
///
/// ⚠️ E as **onze** extensões de imagem também: o diálogo oferecia quatro enquanto o predicado
/// aceitava onze, e esse buraco é anterior a esta linha.
#[test]
fn aseprite_and_all_eleven_image_formats_are_offered() {
    let filters = dialog_filters();
    let all = &filters[0].1;
    assert!(all.contains(&"ase") && all.contains(&"aseprite"));
    assert!(all.contains(&"gif") && all.contains(&"psd") && all.contains(&"ora"));
    assert_eq!(
        all.len(),
        crate::ase_import::ASE_EXTENSIONS.len() + ph2d_asset::SUPPORTED_IMAGE_EXTENSIONS.len(),
        "a linha «tudo» tem de ser a UNIAO das duas listas, sem inventar nem perder"
    );
    // As linhas estreitas existem para o artista poder filtrar, e cada uma é a sua lista.
    assert_eq!(filters[1].0, "Aseprite");
    assert_eq!(filters[1].1, crate::ase_import::ASE_EXTENSIONS.to_vec());
    assert_eq!(filters[2].0, "Images");
}

/// **Cada ficheiro vai para o seu importador**, e o que não é de nenhum sai NOMEADO — não em
/// silêncio, nem com um erro que fale de imagem.
///
/// **Mutação que deve sangrar:** pôr o ramo do `.ase` **depois** do da imagem (nenhuma extensão do
/// Aseprite é de imagem, então o ramo morreria só se as listas colidissem — e é isso que este gate
/// prende para o dia em que colidirem).
#[test]
fn every_file_goes_to_its_own_importer() {
    let (ase, images, unknown) = partition_importables(&[
        p("/a/hero.ase"),
        p("/a/tiles.png"),
        p("/a/notes.txt"),
        p("/a/boss.ASEPRITE"),
        p("/a/scan.PSD"),
    ]);
    assert_eq!(ase, vec![p("/a/hero.ase"), p("/a/boss.ASEPRITE")]);
    assert_eq!(images, vec![p("/a/tiles.png"), p("/a/scan.PSD")]);
    assert_eq!(unknown, vec![p("/a/notes.txt")]);
}

/// **A ordem da leva sobrevive** — o artista escolheu-a no diálogo, e reordenar faria a grelha sair
/// noutra ordem sem ninguém a ter pedido.
#[test]
fn the_order_of_the_batch_survives() {
    let (_, images, _) = partition_importables(&[p("/c.png"), p("/a.png"), p("/b.png")]);
    assert_eq!(images, vec![p("/c.png"), p("/a.png"), p("/b.png")]);
}

/// **Sem `.ase` na leva, a âncora das imagens não se mexe um milímetro** — quem larga só imagens
/// não pode notar que este roteador existe.
///
/// E **com** `.ase`, a grelha começa abaixo da linha deles: misturar os dois num arranjo só daria
/// uma grelha em que uma célula é uma tira de doze quadros.
#[test]
fn the_image_grid_only_moves_when_there_is_a_sheet_above_it() {
    let a = [3.0, 4.0];
    assert_eq!(images_anchor(a, 0.0), a, "sem folha, nada muda");
    let below = images_anchor(a, 2.0);
    assert_eq!(below[0], a[0], "so' DESCE — a coluna e' a mesma");
    assert!(
        below[1] < a[1] - 2.0,
        "e desce mais do que a altura da folha"
    );
}

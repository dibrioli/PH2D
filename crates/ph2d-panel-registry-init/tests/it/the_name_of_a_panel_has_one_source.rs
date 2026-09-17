//! ⭐⭐⭐ **O NOME DE UM PAINEL TEM UMA FONTE** — a aba do encaixe e o cabeçalho do próprio painel
//! resolvem a MESMA chave, e ela existe na tabela.
//!
//! # ⛔⛔ O defeito que isto veio curar, medido
//!
//! Até 2026-09-17 o `Panel::TITLE` era um `&'static str` com o nome escrito em inglês no `lib.rs` de
//! cada painel, e **vinte** gates de painel carregavam a mesma isenção nomeada: *«o `Panel::TITLE`
//! é um `const &'static str` que o registo lê para a ABA, e o `tr` não é `const fn`»*. A premissa é
//! verdadeira; a conclusão era falsa. A cura nunca foi chamar o `tr` dentro de um `const` — é
//! declarar a **CHAVE** e deixar quem PINTA traduzir, e o `TextKey::new` **é** `const fn`.
//!
//! ⛔ E a migração achou o que a isenção escondia: **vinte** painéis pintam o próprio cabeçalho com
//! `tr("panel.<…>.title")` e a aba lia o literal do `lib.rs` — duas fontes para o mesmo nome, e
//! **cinco discordavam no ecrã ao mesmo tempo**:
//!
//! | painel | cabeçalho dizia | aba dizia |
//! |---|---|---|
//! | `tokens` | Tokens | Design Tokens |
//! | `wet_tuning` | Wet Tuning | Wet Paint |
//! | `model3d` | 3D Model | Model 3D |
//! | `color_equalization` | Color EQ | Color Equalization |
//! | `bgremoval` | Bg Removal | Background Removal |
//!
//! ⚠️⚠️ **O gate [`super::the_tab_and_the_menu_call_a_panel_the_same_thing`] existe desde 2026-09-08
//! para impedir exactamente isto, e não podia vê-lo**: ele compara a aba com o MENU, e a terceira
//! superfície é o painel a nomear-se a si próprio. *Um painel tem três sítios onde se apresenta, e
//! um gate que mede dois deles lê-se como se medisse o assunto.*
//!
//! # As duas metades
//!
//! - [`every_tab_speaks_through_the_string_table`] — o que o registo OFERECE: cada chave resolve (uma
//!   chave com erro de escrita pinta o identificador cru na aba, e vaza uma string por quadro).
//! - [`no_panel_paints_its_own_name_beside_the_key`] — o que o FONTE escreve: nenhum painel volta a
//!   ter um segundo caminho até ao próprio nome.

use std::path::{Path, PathBuf};

/// ⚠️ **Nem todos os painéis estão nas features de omissão** (`flip`, `flip_frames`,
/// `painter_layers` e `wet_tuning` ficam de fora), então o piso é o que esta build regista — não os
/// 28 que existem. ⛔ Sem piso nenhum, um registo vazio faria as duas metades passar por vácuo.
const PISO_DE_PAINEIS: usize = 20;

/// ⭐ **A chave de um painel DERIVA do id dele** — `panel.<id>.title`.
///
/// ⚠️ As duas excepções são famílias de chaves ANTERIORES a esta wave, com dezenas de irmãs
/// (`panel.color_eq.adjust.*` tem 35, `panel.bg_removal.mask.*` tem 18): renomear o prefixo inteiro
/// é outra obra, e renomear só o `.title` deixaria a família a falar duas línguas. *Uma excepção com
/// o mecanismo escrito é o oposto de uma lista aberta* — um painel NOVO não tem por onde escolher.
const PREFIXO_ANTIGO: &[(&str, &str, &str)] = &[
    (
        "bgremoval",
        "panel.bg_removal.title",
        "a família `panel.bg_removal.*` tem 18 chaves anteriores a esta wave; o id da crate é \
         `bgremoval` (sem sublinhado) e alinhar os dois é renomear a família inteira",
    ),
    (
        "color_equalization",
        "panel.color_eq.title",
        "a família `panel.color_eq.*` tem 35 chaves anteriores a esta wave; alinhá-la com o id \
         `color_equalization` é renomear a família inteira, obra própria",
    ),
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<crate>/ tem dois pais")
        .to_path_buf()
}

/// ⭐⭐⭐ **Cada aba fala pela tabela** — a chave existe, e ela nomeia o painel a que pertence.
#[test]
fn every_tab_speaks_through_the_string_table() {
    let _ = ph2d_panel_registry_init::register_all_panels();

    let mut mudas = Vec::new();
    let mut desalinhadas = Vec::new();
    let mut vistos = 0usize;
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            vistos += 1;
            let k = p.manifest.title.key();
            // ⛔ Uma chave desconhecida volta CRUA do `tr` (`leak_key`) — na aba isso pinta
            //    `panel.tags.title` e vaza uma string a cada quadro.
            if p.manifest.title.tr() == k {
                mudas.push(format!(
                    "{}: a chave {k:?} não existe na tabela",
                    p.manifest.id
                ));
            }
            let esperada = format!("panel.{}.title", p.manifest.id);
            let isento = PREFIXO_ANTIGO
                .iter()
                .any(|(id, chave, _)| *id == p.manifest.id && *chave == k);
            if k != esperada && !isento {
                desalinhadas.push(format!(
                    "{}: a chave é {k:?} e o id pede {esperada:?}",
                    p.manifest.id
                ));
            }
        }
    });

    assert!(
        vistos >= PISO_DE_PAINEIS,
        "o registo devolveu {vistos} painéis e o piso é {PISO_DE_PAINEIS} — duas metades sobre um \
         registo vazio concordam sempre"
    );
    assert!(
        mudas.is_empty(),
        "estas abas pintam o IDENTIFICADOR em vez do nome, e vazam uma string por quadro:\n  {}\n\n\
         A cura é a chave na tabela de strings da família do painel (`crates/ph2d-i18n/src/`).",
        mudas.join("\n  ")
    );
    assert!(
        desalinhadas.is_empty(),
        "estas chaves de aba não derivam do `Panel::ID`:\n  {}\n\nA cura é escrever \
         `panel.<id>.title`. ⚠️ Se a família de chaves do painel é anterior a esta wave, a cura é \
         uma linha em `PREFIXO_ANTIGO` **com o mecanismo**.",
        desalinhadas.join("\n  ")
    );
}

/// ⭐ **A metade justa das excepções** — sem mecanismo, ou já sem painel que as use.
#[test]
fn every_named_prefix_exception_still_names_a_live_panel() {
    let _ = ph2d_panel_registry_init::register_all_panels();
    let mut vivos: Vec<(String, String)> = Vec::new();
    ph2d_editor_core::panel::with_registry_ref(|reg| {
        for p in reg.panels() {
            vivos.push((
                p.manifest.id.to_string(),
                p.manifest.title.key().to_string(),
            ));
        }
    });
    assert!(
        vivos.len() >= PISO_DE_PAINEIS,
        "o registo devolveu {} painéis — a metade justa mede o vazio",
        vivos.len()
    );
    for (id, chave, porque) in PREFIXO_ANTIGO {
        assert!(
            porque.len() > 40,
            "a excepção `{id}` não diz o mecanismo — uma lista sem mecanismo é uma licença"
        );
        assert!(
            vivos.iter().any(|(i, k)| i == id && k == chave),
            "a excepção `{id}` · {chave:?} já não descreve nenhum painel registado — ou a família \
             foi renomeada (apague a linha) ou o painel saiu"
        );
    }
}

/// ⭐ **Os `tr("panel.<id>.title")` de um fonte** — `(posição, chave)`, com a FORMA a discriminar.
///
/// ⚠️ Três segmentos, sempre: `panel` · o id · `title`. ⛔ `panel.<id>.<secção>.title` são quatro e
/// é o título de uma SECÇÃO — a 1.ª redacção desta régua não os separava e acusou três sítios
/// correctos. ⚠️ E o `panel.flip_frames.title_with_layer` também fica de fora, porque o sufixo é
/// outro: ali o nome do painel entra na frase pelo `Panel::TITLE`.
fn chaves_de_painel_em(src: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let b = src.as_bytes();
    let mut i = 0usize;
    while let Some(rel) = src[i..].find("tr(") {
        let abre = i + rel;
        i = abre + 3;
        // o 1.º argumento, saltando espaço e quebras de linha que o `rustfmt` põe
        let mut j = i;
        while j < b.len() && (b[j] as char).is_whitespace() {
            j += 1;
        }
        if j >= b.len() || b[j] != b'"' {
            continue;
        }
        let Some(fim) = src[j + 1..].find('"') else {
            continue;
        };
        let k = &src[j + 1..j + 1 + fim];
        let partes: Vec<&str> = k.split('.').collect();
        if partes.len() == 3 && partes[0] == "panel" && partes[2] == "title" {
            out.push((abre, k.to_string()));
        }
    }
    out
}

/// ⭐⭐⭐ **NENHUM PAINEL TEM UM SEGUNDO CAMINHO ATÉ AO PRÓPRIO NOME.**
///
/// ⚠️ É um censo de FONTE porque a segunda fonte não é observável no registo: ela vive dentro do
/// `paint` de cada painel, que precisa de um device para correr. O que ele proíbe é a forma exacta
/// que produziu as cinco divergências — um `tr("panel.<id>.title")` escrito à mão ao lado do
/// `Panel::TITLE`.
///
/// ⛔⛔ **A 1.ª redacção desta régua acusou TRÊS sítios legítimos, e a lição é sobre ela própria:**
/// ela perguntava *«a linha tem `tr(` e acaba em `.title")`?»* e apanhou o
/// `panel.model3d.select.title`, o `panel.painter_layers.impasto.title` e o
/// `panel.tokens.contrast.title` — que são o título de uma **SECÇÃO**, não o nome do painel.
/// ⇒ *o discriminador é a FORMA da chave*: o nome de um painel é `panel.<id>.title`, com
/// **exactamente três** segmentos; `panel.<id>.<secção>.title` tem quatro e é outra grandeza.
/// *Uma régua que mede o SUFIXO de uma chave mede a palavra final, não o assunto dela.*
///
/// ⚠️ **E ela lê o ficheiro INTEIRO, nunca linha a linha:** um `tr(\n    "panel.x.title",\n)`
/// partido pelo `rustfmt` é invisível a um varrimento por linha, e os 21 sítios que esta fatia
/// migrou eram todos de uma linha **por acaso**.
#[test]
fn no_panel_paints_its_own_name_beside_the_key() {
    let raiz = repo_root().join("crates");
    let mut ficheiros = 0usize;
    let mut segundas = Vec::new();
    let mut caixas: Vec<PathBuf> = std::fs::read_dir(&raiz)
        .expect("crates/ existe")
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("ph2d-panel-"))
        })
        .collect();
    caixas.sort();
    assert!(
        caixas.len() >= 28,
        "só {} crates de painel — o censo está a ler o sítio errado",
        caixas.len()
    );
    for caixa in &caixas {
        let mut pilha = vec![caixa.join("src")];
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
                let Ok(s) = std::fs::read_to_string(&p) else {
                    continue;
                };
                ficheiros += 1;
                segundas.extend(chaves_de_painel_em(&s).into_iter().map(|(pos, k)| {
                    format!(
                        "{}:{} · tr({k:?})",
                        p.strip_prefix(repo_root()).unwrap_or(&p).display(),
                        s[..pos].matches('\n').count() + 1,
                    )
                }));
            }
        }
    }
    assert!(
        ficheiros >= 200,
        "o censo leu {ficheiros} ficheiros — está a varrer o vazio"
    );
    assert!(
        segundas.is_empty(),
        "estes sítios resolvem o nome de um painel por uma SEGUNDA porta, ao lado do \
         `Panel::TITLE`:\n  {}\n\nA cura é `<Painel>::TITLE.tr()` — uma chave, duas superfícies. \
         Foi um destes que deu ao artista `\"Tokens\"` no cabeçalho e `\"Design Tokens\"` na aba.",
        segundas.join("\n  ")
    );
}

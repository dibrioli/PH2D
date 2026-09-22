//! ⭐⭐⭐⭐ **OS DOIS LADOS DA VIAGEM CARREGAM O QUE O ARTISTA ESCOLHEU** — o gravar e o devolver.
//!
//! O `BakedFormDocument` da shell tem dois grupos de campos: os **planos de pixels** (que o
//! `collect` DERIVA — converte canais, lê o slot) e as **escolhas AUTORADAS**, que ele só pode
//! COPIAR. Este censo é sobre as segundas.
//!
//! # ⛔⛔ Porque o round-trip do disco NÃO cobre isto
//!
//! O gate irmão (`a_baked_document_survives_the_disk_…`, na shell) monta o documento à mão e
//! serializa-o ⇒ ele fica **verde** com um `collect` que escreva o valor de FÁBRICA em toda peça:
//! *o ficheiro seria perfeito e a escolha do artista morria no save*. E o defeito é MUDO — as duas
//! leis acendem, e ele só aparece ao reabrir.
//!
//! # ⭐⭐⭐ E a lista é DERIVADA da struct, nunca escrita à mão
//!
//! Um campo novo no `BakedFormDocument` entra aqui **sozinho** e este censo reprova até alguém o
//! ligar nos dois lados. ⚠️ *Uma lista escrita à mão ao lado de uma struct é a segunda resposta à
//! mesma pergunta, e é sempre a lista que envelhece* — foi assim que o `recorte` (degrau `164`) e a
//! `materia_da_forma` (`165`) nasceram fora do censo anterior, que nomeava a `lei` e mais nada, e a
//! mutação que apagava as duas conversões **SOBREVIVEU**.
//!
//! # ⚠️ Por que ele é de TEXTO, e por que vive nesta crate
//!
//! As duas funções são métodos da `App` — elas pedem o mundo, o renderizador e o mapa dos assados,
//! e nenhum existe num teste headless. É a mesma família do `the_bake_button_is_wired`.
//!
//! E ele mora aqui **pelo tecto de LOC da shell** (a catraca `the_shell_only_shrinks`, que conta
//! `shells/desktop` inteiro, `tests/` incluído): o irmão que mede a shell já vive nesta pasta e já
//! a lê por caminho. ⛔ O preço está registado — *«um gate de família que lê a shell pelo caminho
//! escapa a quem move o código»* —, e aqui ele erra para o lado BARULHENTO: o `fonte` faz `panic!`
//! com o caminho dentro se o ficheiro mudar de sítio.

use std::path::{Path, PathBuf};

/// O ficheiro da shell que declara o documento e as duas conversões.
const DOC: &str = "src/project_baked_form.rs";

/// A raiz da shell — a MESMA travessia do `architecture_the_shell_only_shrinks`.
fn shell_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<x>/ tem dois pais")
        .join("shells/desktop")
}

fn fonte() -> String {
    let path: PathBuf = shell_dir().join(DOC);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"))
}

/// Os campos do documento que são **escolha AUTORADA** e não pixel — os que têm de atravessar as
/// duas conversões. Ver o cabeçalho: ela é derivada da `struct`.
fn campos_autorados(fonte: &str) -> Vec<String> {
    let i = fonte
        .find("pub(crate) struct BakedFormDocument {")
        .expect("controlo: a struct tem de existir com este nome");
    let corpo = &fonte[i..];
    let fim = corpo
        .find("\n}\n")
        .expect("controlo: a struct tem de fechar");
    let campos: Vec<String> = corpo[..fim]
        .lines()
        .skip(1) // a própria linha da `struct` também abre com `pub(crate) `
        .filter_map(|l| l.trim().strip_prefix("pub(crate) "))
        .filter_map(|l| l.split_once(':'))
        // ⚠️ **A forma de um CAMPO e nada mais:** `nome: Tipo,`. Sem esta cerca a linha da própria
        // `struct` entra como um campo chamado `struct BakedFormDocument {`, e o censo reprova com
        // uma mensagem que aponta para o sítio errado — medido.
        .filter(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'))
        .map(|(n, _)| n.to_string())
        .collect();
    // ⛔ **O que fica de FORA é NOMEADO, com a razão:** são os planos de pixels e a identidade, que
    // o `collect` DERIVA em vez de copiar. *Uma exclusão sem nome é o sítio onde o campo seguinte
    // se esconde.*
    const DERIVADOS: [&str; 7] = ["id", "width", "height", "base", "form", "form_occ", "rig"];
    assert!(
        campos.len() >= DERIVADOS.len(),
        "controlo: a extraccao da struct partiu-se ({} campos)",
        campos.len()
    );
    for d in DERIVADOS {
        assert!(
            campos.iter().any(|c| c == d),
            "o campo derivado `{d}` ja' nao existe na struct — a lista de excepcoes envelheceu"
        );
    }
    campos
        .into_iter()
        .filter(|c| !DERIVADOS.contains(&c.as_str()))
        .collect()
}

/// ⭐⭐⭐ **O veredito, campo a campo e sentido a sentido.**
///
/// **Mutação que deve sangrar:** `lei: bake.lei` → `lei: Lei::default()` no `collect`.
#[test]
fn os_campos_autorados_viajam_nos_dois_sentidos_do_documento() {
    let fonte = fonte();
    let campos = campos_autorados(&fonte);
    // ⛔ **O PISO de população:** uma extracção partida devolve o vazio, e todo laço abaixo fica
    // trivialmente verdadeiro. Os três de hoje são `lei`, `materia_da_forma` e `recorte`.
    assert!(
        campos.len() >= 3,
        "o censo colheu so' {} campos autorados: a extraccao partiu-se",
        campos.len()
    );
    for campo in &campos {
        for (funcao, de, porque) in [
            ("fn collect_baked_forms", "bake", "gravar"),
            ("fn restore_baked_forms", "doc", "devolver"),
        ] {
            let agulha = format!("{campo}: {de}.{campo},");
            let i = fonte
                .find(funcao)
                .unwrap_or_else(|| panic!("controlo: a `{funcao}` tem de existir com este nome"));
            let corpo = &fonte[i..];
            let fim = corpo.find("\n    }\n").unwrap_or(corpo.len());
            assert!(
                corpo[..fim].contains(&agulha),
                "a `{funcao}` tem de {porque} o `{campo}` do objecto — sem isso a escolha do \
                 artista morre no save, e o ficheiro reabre com o valor de fabrica de quem o abre. \
                 E o defeito e' MUDO: os dois lados acendem."
            );
        }
    }
}

/// ⛔⛔ **E o CONTROLO da própria extracção:** um campo que NÃO está lá tem de ler zero.
///
/// ⚠️ Sem esta metade, uma busca que devolvesse sempre `true` deixaria o censo acima verde a
/// afirmar **nada**. *Um censo que não acha nada acusa tudo; um que acha sempre não acusa nada, e é
/// este o lado que passa despercebido.*
#[test]
fn o_censo_do_documento_sabe_dizer_que_nao() {
    let fonte = fonte();
    let campos = campos_autorados(&fonte);
    assert!(
        !campos.is_empty(),
        "controlo: tem de haver campos autorados"
    );
    for de in ["bake", "doc"] {
        let ausente = format!("isto_nao_existe: {de}.isto_nao_existe,");
        assert!(
            !fonte.contains(&ausente),
            "controlo: a busca acha um campo que nao existe"
        );
    }
}

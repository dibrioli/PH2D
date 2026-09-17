//! ⭐⭐⭐ **O RÓTULO DE UM PARÂMETRO É UMA CHAVE, E ELA DERIVA DE `(tipo, param)`.**
//!
//! Irmão do [`super::every_node_name_is_a_key_derived_from_its_type`], e mora aqui pela mesma
//! razão: os hints vivem um por crate e o registo é a **única porta** onde o *tipo* e o *param*
//! existem ao mesmo tempo. *Um gate que precisa do par tem de viver onde o par existe.*
//!
//! ⛔⛔ **E há uma segunda razão, que é a que torna este gate o ORÁCULO da wave.** O campo
//! `param` aparece em três formas neste repo — literal, caminho de `const`, e campo de
//! tuplo / índice de array / argumento de macro —, e o `label` em quatro. Uma varredura de
//! texto vê as duas primeiras de cada; este gate lê os hints **REGISTADOS**, logo vê o que
//! um construtor auxiliar de facto PRODUZIU. ⚠️ A varredura que preparou esta wave mordeu
//! exactamente aí: num hint cujo `label` não era literal ela leu o literal do hint **seguinte**
//! dentro da janela e reescreveu-o com a chave deste. Ali o splice deu sintaxe inválida e o
//! compilador viu — *com os offsets alinhados teria sido uma chave errada em silêncio, e este
//! gate é a única coisa que a apanharia.*
//!
//! ⚠️⚠️ **E `(tipo, param)` não é único.** O `motion.spline_wrap` declara DUAS rows sobre o param
//! `path` — a que escolhe a forma e o botão *«usa a que está seleccionada»* —, e o comentário ao
//! lado delas chama-lhes *«dois GESTOS para o mesmo param»*. ⇒ quando, e só quando, um param
//! declara mais do que uma row, **todas** levam o widget na chave: uma regra que depende do
//! CONJUNTO e não da ordem, logo uma terceira row não renomeia as duas que já lá estão.

use ph2d_node_registry::{NodeRegistry, ParamUiHint};
use std::collections::BTreeMap;

/// O nome do widget em `snake_case` — o desempate de um param com mais de uma row.
fn widget_snake(h: &ParamUiHint) -> String {
    let bruto = format!("{:?}", h.widget);
    let nome = bruto.split([' ', '{', '(']).next().unwrap_or("");
    let mut s = String::new();
    for (i, c) in nome.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            s.push('_');
        }
        s.extend(c.to_lowercase());
    }
    s
}

/// Todos os `(tipo, hints)` registados — a população que os dois testes medem.
fn catalogo() -> Vec<(&'static str, &'static [ParamUiHint])> {
    let mut reg = NodeRegistry::default();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("o registo tem de montar");
    let tipos: Vec<_> = reg.manifests().map(|m| (m.name, m.id)).collect();
    tipos
        .into_iter()
        .filter_map(|(nome, id)| reg.param_ui(id).map(|h| (nome, h)))
        .collect()
}

#[test]
fn every_param_label_is_a_key_derived_from_its_type_and_param() {
    let catalogo = catalogo();
    let total: usize = catalogo.iter().map(|(_, h)| h.len()).sum();
    // ⛔ Piso de população nas DUAS grandezas: um registo vazio, ou um em que ninguém
    // declara hints, passaria trivialmente — e é assim que um censo fica verde a medir nada.
    assert!(
        catalogo.len() >= 120,
        "só {} tipos declaram hints — a população encolheu?",
        catalogo.len()
    );
    assert!(
        total >= 780,
        "só {total} hints registados — a população encolheu?"
    );

    let mut erradas = Vec::new();
    for (tipo, hints) in &catalogo {
        // quantas rows cada param declara neste tipo: é isso, e não a ordem, que decide a chave
        let mut rows: BTreeMap<&str, usize> = BTreeMap::new();
        for h in hints.iter() {
            *rows.entry(h.param).or_default() += 1;
        }
        for h in hints.iter() {
            let base = format!("node.{tipo}.param.{}", h.param);
            let esperada = if rows[h.param] == 1 {
                base
            } else {
                format!("{base}.{}", widget_snake(h))
            };
            if h.label != esperada {
                erradas.push(format!(
                    "{tipo}::{} declara {:?} e a derivação dá {esperada:?}",
                    h.param, h.label
                ));
            }
        }
    }
    assert!(
        erradas.is_empty(),
        "estes {} rótulos não são a chave derivada de `(tipo, param)`:\n  {}\n\nA lei é \
         `node.<tipo>.param.<param>` — ela não se escolhe, e o texto vive em \
         `crates/ph2d-i18n/src/node_params.rs`.",
        erradas.len(),
        erradas.join("\n  ")
    );
}

/// ⭐⭐ **E a chave RESOLVE-SE** — a metade que o gate acima não pode fazer.
///
/// ⛔ Uma chave bem derivada e ausente da tabela pinta o **identificador cru na linha do
/// parâmetro**, e o `tr` de uma chave desconhecida faz `leak_key` (`Box::leak`): num painel
/// repintado por quadro, é um vazamento por quadro e por linha.
#[test]
fn every_param_label_key_resolves_to_a_word() {
    let catalogo = catalogo();
    let total: usize = catalogo.iter().map(|(_, h)| h.len()).sum();
    assert!(
        total >= 780,
        "só {total} hints registados — a população encolheu?"
    );
    let mut cruas = Vec::new();
    for (tipo, hints) in &catalogo {
        for h in hints.iter() {
            if ph2d_i18n::tr(h.label) == h.label {
                cruas.push(format!("{tipo}::{}: {:?}", h.param, h.label));
            }
        }
    }
    assert!(
        cruas.is_empty(),
        "estas {} chaves não têm palavra em `node_params.rs` e o painel pinta o identificador:\n  \
         {}",
        cruas.len(),
        cruas.join("\n  ")
    );
}

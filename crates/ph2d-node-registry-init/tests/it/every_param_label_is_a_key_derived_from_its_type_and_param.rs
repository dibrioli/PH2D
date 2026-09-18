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

/// ⭐⭐⭐ **E AS OPÇÕES DE CADA SELECTOR TAMBÉM SÃO CHAVES — mas de DOIS sítios, porque são
/// duas identidades.**
///
/// Um array escrito INLINE dentro de um `ParamUiHint` pertence a um par `(tipo, param)` e a mais
/// ninguém ⇒ `node.<tipo>.param.<param>.<i>`. Uma `const` é um VOCABULÁRIO com N leitores — o
/// `ph2d_motion_region::SHAPE_LABELS` é lido por quatro nós e o `ph2d_nodegraph::pivot::LABELS`
/// por três ⇒ `node.opts.<crate>.<CONST>.<i>`, uma vez para todos.
///
/// ⛔ **A chave derivada do NÓ estaria errada para a segunda metade:** daria N cópias do mesmo
/// texto, e mudar uma não mudava as outras. ⚠️ E o NOME da const não é a identidade dela —
/// `MODE_LABELS` existe em SEIS crates com conteúdos diferentes (`2 · 2 · 3 · 2 · 6 · 2` opções).
#[test]
fn every_enum_option_is_a_key_from_its_declaration_site() {
    use ph2d_node_registry::ParamWidget;
    let catalogo = catalogo();
    let (mut inline, mut partilhadas, mut opcoes) = (0usize, 0usize, 0usize);
    let mut erradas = Vec::new();
    for (tipo, hints) in &catalogo {
        for h in hints.iter() {
            let ParamWidget::Enum { labels } = h.widget else {
                continue;
            };
            if labels.is_empty() {
                continue;
            }
            let base_inline = format!("node.{tipo}.param.{}", h.param);
            let por_no = labels
                .iter()
                .enumerate()
                .all(|(i, l)| *l == format!("{base_inline}.{i}"));
            let por_const = labels
                .iter()
                .enumerate()
                .all(|(i, l)| l.starts_with("node.opts.") && l.ends_with(&format!(".{i}")));
            opcoes += labels.len();
            if por_no {
                inline += 1;
            } else if por_const {
                partilhadas += 1;
            } else {
                erradas.push(format!("{tipo}::{}: {labels:?}", h.param));
            }
        }
    }
    // ⛔ Piso nas TRÊS grandezas: sem eles um catálogo sem selectores passa trivialmente, e uma
    // metade que desaparecesse não seria vista pela outra.
    assert!(inline >= 100, "só {inline} arrays inline — encolheu?");
    assert!(
        partilhadas >= 25,
        "só {partilhadas} arrays por const — encolheu?"
    );
    assert!(opcoes >= 500, "só {opcoes} opções — encolheu?");
    assert!(
        erradas.is_empty(),
        "estes {} selectores não carregam chaves derivadas do sítio onde o array é DECLARADO:\n  \
         {}\n\nInline ⇒ `node.<tipo>.param.<param>.<i>`; const ⇒ `node.opts.<crate>.<CONST>.<i>`.",
        erradas.len(),
        erradas.join("\n  ")
    );
}

/// ⭐⭐ **E cada opção RESOLVE-SE** — a metade que o gate acima não faz.
#[test]
fn every_enum_option_key_resolves_to_a_word() {
    use ph2d_node_registry::ParamWidget;
    let catalogo = catalogo();
    let mut cruas = Vec::new();
    let mut n = 0usize;
    for (tipo, hints) in &catalogo {
        for h in hints.iter() {
            let ParamWidget::Enum { labels } = h.widget else {
                continue;
            };
            for l in labels {
                n += 1;
                if ph2d_i18n::tr(l) == *l {
                    cruas.push(format!("{tipo}::{}: {l:?}", h.param));
                }
            }
        }
    }
    assert!(n >= 500, "só {n} opções — encolheu?");
    assert!(
        cruas.is_empty(),
        "estas {} opções não têm palavra em `node_options.rs` e o selector pinta o \
         identificador:\n  {}",
        cruas.len(),
        cruas.join("\n  ")
    );
}

/// ⭐⭐ **E OS CANAIS de um `ParamWidget::Channels` — a última família de texto do catálogo.**
///
/// ⛔ **A chave NÃO pode ser a coluna sozinha:** a coluna `P` serve quatro canais (*Position X*,
/// *Position Y*, *Radius*, *Angle*) e a `vel` outros quatro. A identidade é o par
/// `(coluna, modo)` — e isso não é uma escolha minha: é exactamente o que o produto usa para os
/// achar (`c.column == *col && c.mode == *modo`, em `motion_bridge_choices`).
///
/// ⚠️ **E o modo entra pelo NOME da constante que o declara, nunca pelo número** — um
/// `MODE_COMPONENT_BASE` que mudasse de valor renomearia todas as chaves de uma vez.
#[test]
fn every_read_channel_is_a_key_from_its_column_and_mode() {
    use ph2d_node_registry::ParamWidget;
    let catalogo = catalogo();
    let (mut n, mut cruas, mut erradas) = (0usize, Vec::new(), Vec::new());
    for (tipo, hints) in &catalogo {
        for h in hints.iter() {
            let ParamWidget::Channels { channels, .. } = h.widget else {
                continue;
            };
            for c in channels {
                n += 1;
                if !c.label.starts_with("node.channel.") {
                    erradas.push(format!("{tipo}::{}: {:?}", c.column, c.label));
                } else if !c.label.contains(&format!(".{}.", c.column.to_lowercase())) {
                    erradas.push(format!(
                        "{tipo}: a chave {:?} não nomeia a coluna `{}`",
                        c.label, c.column
                    ));
                }
                if ph2d_i18n::tr(c.label) == c.label {
                    cruas.push(format!("{tipo}::{}: {:?}", c.column, c.label));
                }
            }
        }
    }
    // ⛔ Piso de população: sem um único picker de canal a varredura mede nada.
    assert!(n >= 24, "só {n} canais declarados — a população encolheu?");
    assert!(
        erradas.is_empty(),
        "estes {} canais não carregam a chave derivada de `(coluna, modo)`:\n  {}",
        erradas.len(),
        erradas.join("\n  ")
    );
    assert!(
        cruas.is_empty(),
        "estas {} chaves de canal não têm palavra em `node_options.rs`:\n  {}",
        cruas.len(),
        cruas.join("\n  ")
    );
}

/// ⛔⛔ **E as chaves são INJECTIVAS sobre o par** — sem isto, dois canais da mesma coluna com
/// modos diferentes podiam herdar a mesma chave e o picker mostrava a mesma palavra duas vezes,
/// com os dois gates acima VERDES (a chave deriva, e resolve — só que resolve para o mesmo).
#[test]
fn two_read_channels_never_share_a_key() {
    use ph2d_node_registry::ParamWidget;
    use std::collections::BTreeMap;
    let mut vistas: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for (tipo, hints) in &catalogo() {
        for h in hints.iter() {
            let ParamWidget::Channels { channels, .. } = h.widget else {
                continue;
            };
            for c in channels {
                vistas
                    .entry(c.label)
                    .or_default()
                    .push(format!("{tipo}::{}·{}", c.column, c.mode));
            }
        }
    }
    assert!(vistas.len() >= 24, "só {} chaves — encolheu?", vistas.len());
    let dobradas: Vec<String> = vistas
        .iter()
        .filter(|(_, v)| v.len() > 1)
        .map(|(k, v)| format!("{k}: {v:?}"))
        .collect();
    assert!(
        dobradas.is_empty(),
        "estas chaves servem mais do que um canal:\n  {}",
        dobradas.join("\n  ")
    );
}

/// ⭐⭐⭐ **TODO NOME DE SECÇÃO RESOLVE NUMA PALAVRA** — a irmã do
/// [`every_enum_option_key_resolves_to_a_word`], para os `ParamGroup`.
///
/// ⛔⛔ Eles eram **inglês cru em 229 sítios de 21 crates** até 2026-09-18, e o doc do campo que
/// os carrega dizia *«o título da seção, em inglês (HR-15: a face do artista sai por i18n no
/// painel)»* — com o painel a pintá-lo **cru**. *Uma nota que descreve ONDE a tradução
/// aconteceria lê-se como se ela acontecesse*, e nenhuma régua perguntava.
///
/// ⚠️ Quem o apanhou foi o **dono**, com duas setas vermelhas sobre `Shape` e `Leaves` numa foto
/// do cartão com o idioma de teste ligado.
#[test]
fn every_param_group_key_resolves_to_a_word() {
    let mut reg = NodeRegistry::default();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("o registo tem de montar");
    let tipos: Vec<_> = reg.manifests().map(|m| m.id).collect();
    let mut cruas = Vec::new();
    let mut n = 0usize;
    let mut familias = std::collections::BTreeSet::new();
    for tipo in tipos {
        for g in reg.param_groups(tipo) {
            n += 1;
            familias.insert(g.group_key);
            if ph2d_i18n::tr(g.group_key) == g.group_key {
                cruas.push(format!("{}::{}", g.param, g.group_key));
            }
        }
    }
    // ⛔ Piso de população nas DUAS grandezas: os sítios e as palavras distintas. Um `node_type_ids`
    //    que encolhesse deixaria este gate verde a medir um punhado.
    assert!(n >= 200, "só {n} secções — a população encolheu?");
    assert!(
        familias.len() >= 30,
        "só {} nomes distintos — a extracção partiu-se?",
        familias.len()
    );
    assert!(
        cruas.is_empty(),
        "estes {} nomes de secção não têm palavra em `node_groups.rs` e o painel pinta o \
         identificador cru:\n  {}",
        cruas.len(),
        cruas.join("\n  ")
    );
}

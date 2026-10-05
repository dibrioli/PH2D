//! ⭐⭐⭐ **A UNIDADE VIVE NO CAMPO, NUNCA NO RÓTULO** — e este gate é quem o cobra em todo o app.
//!
//! ⛔⛔ A lei está escrita na spec da linha de propriedade
//! (`docs/UI_New_and_Simple/spec/03_a_linha_de_propriedade.md` §7) e foi **medida** em 2026-09-14:
//! com a unidade no rótulo, **20 de 39** rótulos do Inspector eram cortados à largura de omissão;
//! sem ela, **1**. Um `"Float Height (m)"` mede `92,1 px` numa coluna de `91,2`.
//!
//! ⚠️⚠️ **E a cura foi aplicada a UMA secção.** Em 2026-09-15 o censo achou **42** textos ainda com
//! a unidade entre parênteses — entre eles `"Non-Spatialized Radius (m)"` (26 caracteres),
//! `"Break Torque (N.m)"` e `"Init Vel X (m/s)"`. *Uma lei curada numa secção e não gateada é uma
//! lei que a secção seguinte não conhece.*
//!
//! # A régua
//!
//! Varre as tabelas de i18n e acusa todo VALOR que acabe num sufixo de unidade entre parênteses.
//! ⛔ Ela lê o **texto**, que é o que o artista vê — não a chave, que é um ENDEREÇO e **mantém** o
//! sufixo de propósito (renomear uma chave para dizer a mesma coisa custa 31 sítios).
//!
//! # ⚠️ A catraca traz o censo de obsolescência ao lado
//!
//! `CLAUDE.md` §5.0: *uma catraca sem censo de obsolescência não desce — ela vira LICENÇA.* A
//! segunda metade pergunta, por entrada, se a chave ainda existe e se o texto dela ainda carrega a
//! unidade.

use std::fs;
use std::path::PathBuf;

/// ⏳ **Os que FICAM, cada um com o MECANISMO que o impede — e só ENCOLHE.**
///
/// ⛔ **Não acrescente uma entrada para uma linha nova.** Uma row de campo único nasce com
/// `num_row_unit` e a unidade no campo; é um argumento.
/// ⚠️⚠️ **A razão de cada entrada é a PORTA que a pinta, não «é multi-campo».** A 1.ª redacção
/// escreveu *«row de dois campos»* e a medição desmentiu-a: o `field_row` das âncoras pinta uma
/// row de UM campo (`Rotation (deg)`) e mesmo assim não leva sufixo, enquanto o `num_row_unit`
/// leva-o numa row de um campo **e** poderia levá-lo em N. *O que separa não é a contagem de
/// campos — é a porta ter, ou não, por onde a unidade entrar.*
const AINDA_NO_ROTULO: &[(&str, &str)] = &[
    // ⭐⭐ **As portas `field_row` e `pair_row` FECHARAM em 2026-09-15** — as seis entradas que aqui
    //    viviam (âncoras ×4, 9-slice ×2) saíram no mesmo dia em que o dono ordenou *«todos na
    //    caixa»*. ⛔ Não as reponha: as duas portas levam `Option<Unit>`.
    // ⭐⭐ **A porta `transform_row::paint_row` FECHOU em 2026-09-15** — as seis entradas do
    //    Transform saíram no mesmo dia. ⚠️ Ali o rótulo dizia a RÉGUA activa (`Position (m)` ⇄
    //    `Position (px)`), e hoje quem a diz é o SUFIXO do campo: `12,5 m` diz a mesma coisa **e** o
    //    número, no sítio onde o artista olha.
    // ⚠️ **A CAIXA ÚNICA não tem sufixo**: ali o rótulo vive DENTRO da caixa, à esquerda, e o valor
    //    à direita — o `NumberInput::suffix` é do campo solto. Ver a spec §2.
    (
        "panel.painter_layers.wetpaint.grid_size_px",
        "caixa unica (slider+chip) — o rotulo vive DENTRO",
    ),
    (
        "panel.tokens.numeric",
        "caixa unica (slider+chip) — o rotulo vive DENTRO",
    ),
    // ⛔ **Não são unidades** — a régua casa `(s)` no fim e estes acabam assim por acidente.
    (
        "panel.inspector.instance.clear_orphans",
        "«override(s)» — plural, nao unidade",
    ),
    (
        "shell.asset_card_verbs.selected_object_s",
        "«object(s)» — plural num aviso da shell, nao unidade",
    ),
    (
        "shell.fase_bus_inspector.cleared_unused",
        "«override(s)» — plural num aviso da shell, nao unidade",
    ),
    (
        "shell.project_load.project_loaded_2",
        "«track(s)» — plural num aviso da shell, nao unidade",
    ),
    (
        "panel.timeline.length",
        "faixa da timeline: outra familia de linha (ver a spec §1)",
    ),
    (
        "panel.timeline.time_seconds",
        "faixa da timeline: outra familia de linha (ver a spec §1)",
    ),
];

/// Um valor de i18n «tem unidade no fim» quando acaba em `(<sufixo>)`.
///
/// ⚠️ **A lista sai do vocabulário do produto** ([`ph2d_editor_core::widget::Unit`]) mais as formas
/// tipográficas que um rótulo usa e o campo não (`°`, `N.m` com espaço). ⛔ Derivá-la SÓ do enum
/// deixaria de fora exactamente os rótulos que ainda não têm unidade correspondente — que são os
/// que este gate existe para achar.
fn unidade_no_fim(texto: &str) -> Option<String> {
    let t = texto.trim_end();
    if !t.ends_with(')') {
        return None;
    }
    let abre = t.rfind('(')?;
    let dentro = &t[abre + 1..t.len() - 1];
    // ⛔⛔ **Um MOLDE entre parênteses é uma unidade, e foi ele que escapou.** Report do dono,
    //    2026-09-15, com foto: `"Speed (°/s)"` continuava no rótulo — e o TEXTO na tabela é
    //    `"Speed ({unit})"`, que não acaba em sufixo nenhum. ⇒ *um rótulo que CONSTRÓI a unidade em
    //    tempo de execução é o caso mais certo de todos, e era o único que o detector não via.*
    // ⚠️ **E é `{unit}`, não «um molde qualquer»** — a 1.ª redacção aceitava todo `{…}` e acusou
    //    quatro TÍTULOS de secção (`"Timers  ({n})"`), onde o molde é uma CONTAGEM. *O que diz que
    //    aquilo é uma unidade é o NOME do molde, e ele está escrito ali.*
    if dentro == "{unit}" {
        return Some(dentro.to_string());
    }
    let normal = dentro.replace("\\u{00b0}", "°");
    // ⛔⛔ **QUATRO formas CEGAS, achadas em 2026-09-15 — e a mais cara era a mais óbvia.**
    //    O censo dizia-se fechado e deixou passar `"Init Spin (deg/s)"` **ao lado** do irmão
    //    `"Init Vel X"`, já curado: a lista tinha `°/s` e **não** `deg/s`, que é a forma que o
    //    `Unit::suffix` de facto emite. As outras três: `1/s` (o *Damping* da câmera, que é uma
    //    TAXA), `ms` (o relógio de uma animação de sprite) e `seconds` **por extenso** (o
    //    *Duration* de um temporizador). *Uma dívida grande esconde os erros do instrumento; foi
    //    ao ela encolher que os quatro ficaram à vista.*
    const SUFIXOS: &[&str] = &[
        "m", "px", "s", "ms", "1/s", "N", "N.m", "m/s", "m/s2", "m/s^2", "%", "deg", "deg/s",
        "rad", "°", "°/s", "N.s", "seconds",
    ];
    SUFIXOS
        .iter()
        .any(|s| normal.eq_ignore_ascii_case(s))
        .then_some(normal)
}

fn tabelas() -> Vec<(String, String)> {
    let raiz = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("crates/ph2d-i18n/src");
    let mut out = Vec::new();
    let Ok(entradas) = fs::read_dir(&raiz) else {
        panic!("a varredura nao achou {raiz:?}");
    };
    for e in entradas.flatten() {
        let p = e.path();
        if p.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let Ok(txt) = fs::read_to_string(&p) else {
            continue;
        };
        for linha in txt.lines() {
            let Some(seta) = linha.find("\" => \"") else {
                continue;
            };
            let Some(abre) = linha[..seta].rfind('"') else {
                continue;
            };
            let chave = &linha[abre + 1..seta];
            let resto = &linha[seta + 6..];
            let Some(fim) = resto.find('"') else { continue };
            out.push((chave.to_string(), resto[..fim].to_string()));
        }
    }
    out
}

#[test]
fn no_row_label_carries_its_own_unit() {
    let pares = tabelas();
    // ⚠️ **Piso de população, MEDIDO** — uma varredura partida devolve zero e lê-se como aprovada.
    //    As tabelas tinham **1 801** textos em 2026-09-15; o piso fica em `1 500` para tolerar uma
    //    poda honesta e ainda acusar um parser que deixou de casar. ⛔ O número é medido, não
    //    escolhido: a 1.ª redacção chutou `2 000` e reprovou sobre o produto certo.
    assert!(
        pares.len() >= 1500,
        "a varredura leu {} textos de i18n — ela deixou de alcancar as tabelas",
        pares.len()
    );
    let tolerado: Vec<&str> = AINDA_NO_ROTULO.iter().map(|(k, _)| *k).collect();
    let mut acusados = Vec::new();
    for (chave, texto) in &pares {
        if tolerado.contains(&chave.as_str()) {
            continue;
        }
        if let Some(u) = unidade_no_fim(texto) {
            acusados.push(format!("{chave}  =>  {texto:?}  (unidade `{u}`)"));
        }
    }
    acusados.sort();
    assert!(
        acusados.is_empty(),
        "{} rotulo(s) ainda carregam a unidade no TEXTO:\n  {}\n\n\
         A unidade vive no CAMPO (`num_row_unit` + `NumberInput::suffix`) — ver a spec \
         `docs/UI_New_and_Simple/spec/03_a_linha_de_propriedade.md` §7. \
         A CHAVE mantem o sufixo: ela e' um endereco, nao o texto.",
        acusados.len(),
        acusados.join("\n  ")
    );
}

/// ⚠️ **A metade de OBSOLESCÊNCIA** — uma tolerância que já não descreve nada sai da lista.
#[test]
fn the_tolerated_unit_labels_still_describe_something() {
    let pares = tabelas();
    let mut mortas = Vec::new();
    for (chave, razao) in AINDA_NO_ROTULO {
        match pares.iter().find(|(k, _)| k == chave) {
            None => mortas.push(format!("{chave} — a chave sumiu ({razao})")),
            Some((_, texto)) if unidade_no_fim(texto).is_none() => {
                mortas.push(format!(
                    "{chave} — ja' nao tem unidade: {texto:?} ({razao})"
                ));
            }
            _ => {}
        }
    }
    assert!(
        mortas.is_empty(),
        "entrada(s) STALE na tolerancia:\n  {}",
        mortas.join("\n  ")
    );
}

/// ⭐⭐ **O CONTROLO do detector** — sem ele, um `unidade_no_fim` que respondesse sempre `None`
/// deixaria as duas metades verdes sobre o app inteiro por converter.
#[test]
fn the_detector_can_see_a_unit_in_a_label() {
    for (texto, esperado) in [
        ("Break Torque (N.m)", Some("N.m")),
        ("Radius (m)", Some("m")),
        ("Motor (\\u{00b0}/s)", Some("°/s")),
        ("Init Vel X (m/s)", Some("m/s")),
        ("Grid Size (px)", Some("px")),
        // ⛔ Os quatro buracos de 2026-09-15.
        ("Init Spin (deg/s)", Some("deg/s")),
        ("Damping (1/s)", Some("1/s")),
        ("Frame ms (0 = use)", None),
        ("Hold (ms)", Some("ms")),
        ("Duration (seconds)", Some("seconds")),
        // ⛔ O MOLDE — o caso que o report de 2026-09-15 expôs.
        ("Speed ({unit})", Some("{unit}")),
    ] {
        assert_eq!(
            unidade_no_fim(texto).as_deref(),
            esperado,
            "o detector nao ve a unidade em {texto:?}"
        );
    }
    // ⛔ E uma CONTAGEM num título não é uma unidade — o molde chama-se `{n}`.
    for texto in [
        "Float Height",
        "Corner Rays",
        "Weight on Ground",
        "Damping",
        "Timers  ({n})",
    ] {
        assert!(
            unidade_no_fim(texto).is_none(),
            "o detector inventa uma unidade em {texto:?}"
        );
    }
}

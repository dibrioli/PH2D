//! **O nome de uma etapa**, traduzido no momento de mostrar: o que o artista escreveu, ou o derivado
//! do que a etapa fez (os nós pelo catálogo, `node.<tipo>.name`; os params por
//! `node.<tipo>.param.<param>`). Só o nome escrito é conteúdo gravado.

use ph2d_editor_core::macro_edits::{EtapaView, FamiliaDaMacro, TipoDaEtapaView as T};
use ph2d_i18n::{tr, tr_with};

/// A tradução de `chave`, ou `None` se a tabela não a tem (o `tr` devolveria a chave crua).
fn tr_se_ha(chave: &str) -> Option<&'static str> {
    let s = tr(chave);
    (s != chave).then_some(s)
}

fn no(tipo: Option<&str>) -> String {
    match tipo {
        Some(t) => tr_se_ha(&format!("node.{t}.name")).map_or_else(|| t.to_owned(), str::to_owned),
        None => tr("panel.macros.step.unknown_node").to_owned(),
    }
}

fn valor(v: f64) -> String {
    let s = format!("{v:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// O nome de `e` como o painel e a Timeline o mostram.
#[must_use]
pub fn nome(e: &EtapaView) -> String {
    if let Some(n) = &e.nome {
        return n.clone();
    }
    match &e.tipo {
        T::Pincelada(n) => tr_with("panel.macros.step.stroke", &[("n", n)]),
        T::Acrescenta(tipos) => {
            let nos: Vec<String> = tipos.iter().map(|t| no(Some(t))).collect();
            tr_with("panel.macros.step.add", &[("nodes", &nos.join(", "))])
        }
        T::Liga { de, para } => tr_with(
            "panel.macros.step.connect",
            &[("from", &no(de.as_deref())), ("to", &no(para.as_deref()))],
        ),
        T::Desliga { para } => {
            tr_with("panel.macros.step.disconnect", &[("to", &no(para.as_deref()))])
        }
        T::Apaga(n) => tr_with("panel.macros.step.delete", &[("n", n)]),
        T::Move => tr("panel.macros.step.move").to_owned(),
        T::Renomeia(nome) => tr_with("panel.macros.step.rename", &[("name", nome)]),
        T::Param { no: tipo, param, de, para } => {
            let rotulo = tipo
                .as_deref()
                .and_then(|t| tr_se_ha(&format!("node.{t}.param.{param}")))
                .map_or_else(|| param.clone(), str::to_owned);
            let de = de.map_or_else(
                || tr("panel.macros.step.default_value").to_owned(),
                |d| valor(f64::from(d)),
            );
            tr_with(
                "panel.macros.step.param",
                &[("param", &rotulo), ("from", &de), ("to", &valor(*para))],
            )
        }
        T::Grafo => tr("panel.macros.step.graph").to_owned(),
        T::Direta => tr("panel.macros.step.direct").to_owned(),
        T::Recusada(porque) => {
            let porque = tr_se_ha(porque).map_or_else(|| porque.clone(), str::to_owned);
            tr_with("panel.macros.step.refused", &[("why", &porque)])
        }
    }
}

/// O nome da família na frase «Record captures {family}».
#[must_use]
pub fn familia(f: FamiliaDaMacro) -> &'static str {
    match f {
        FamiliaDaMacro::Painter => tr("panel.macros.family.painter"),
        FamiliaDaMacro::Motion => tr("panel.macros.family.motion"),
    }
}

/// Segundos como o painel os escreve.
#[must_use]
pub fn segundos(s: f64) -> String {
    format!("{s:.2}")
}

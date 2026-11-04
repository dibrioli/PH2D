//! ⭐⭐ **A lei da compactação, por pintura real** (dono 04/11, `docs/CartoesUI/05` §4). Pinta cada
//! painel do registo pela porta do produto, de fábrica e armado, todas as gavetas abertas, e lê o
//! censo da fundação (`panel::cards::censo`):
//!
//! - `lei_da_compactacao`: num cartão, linha de um só controlo curto seguida de outra que caberia
//!   ao lado reprova; distância entre linhas abaixo de `FIELD_ROW_GAP` reprova.
//! - `nenhum_texto_sai_do_botao`: em todo botão, segmento e caixa de marcar, o texto medido mais o
//!   respiro de `metrics` cabe na largura.
//!
//! `PENDENTES` só desce: é o alvo de zero. O número que desce TEM de ser escrito aqui. Corpo que
//! pinta fora das portas (captura do Painter, tela própria) não aparece neste censo.
//! Sonda: `cargo test … diag_a_tabela_da_compactacao -- --ignored --nocapture`.

use std::collections::BTreeMap;

use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::censo::{self, Censo};
use ph2d_ui_testkit::MockPanelHost;

use super::a_marca_tem_a_altura_da_linha::{abre_as_gavetas, viewport};

/// `(painel, irmãos separados, linhas grudadas, textos cortados)`, medido em 04/11.
const PENDENTES: &[(&str, usize, usize, usize)] = &[("widget_gallery", 7, 1, 0)];

/// O que um painel acusa: as listas de frases das três regras.
#[derive(Default)]
struct Acusa {
    irmaos: Vec<String>,
    grudadas: Vec<String>,
    cortados: Vec<String>,
    /// O tamanho da população medida: zero lê-se como aprovação, e pode ser só cegueira.
    linhas: usize,
    textos: usize,
}

fn junta(a: &mut Acusa, c: &Censo) {
    a.linhas += c.linhas.len();
    a.textos += c.textos.len();
    for (de, para) in [
        (c.irmaos_separados(), &mut a.irmaos),
        (c.linhas_grudadas(), &mut a.grudadas),
        (c.textos_cortados(), &mut a.cortados),
    ] {
        for frase in de {
            if !para.contains(&frase) {
                para.push(frase);
            }
        }
    }
}

/// Os cartões que nascem dobrados só existem depois do 1.º quadro: abre, pinta, abre outra vez e mede.
fn mede(host: &mut MockPanelHost, painel: &mut ph2d_editor_core::panel::ErasedPanel) -> Censo {
    abre_as_gavetas(host.store_mut());
    let _ = host.medindo_a_pintura_do_registo(painel, viewport());
    abre_as_gavetas(host.store_mut());
    censo::grava(|| host.medindo_a_pintura_do_registo(painel, viewport())).1
}

fn censo_de_todos() -> BTreeMap<&'static str, Acusa> {
    let _ = ph2d_panel_registry_init::register_all_panels();
    ph2d_panel_inspector::set_cards_on(true);
    let mut out: BTreeMap<&'static str, Acusa> = BTreeMap::new();
    ph2d_editor_core::panel::with_registry(|reg| {
        for painel in reg.panels_mut() {
            let id = painel.manifest.id;
            let acusa = out.entry(id).or_default();
            let mut host = MockPanelHost::new();
            painel.populate(host.store_mut());
            junta(acusa, &mede(&mut host, painel));
            if let Some(arm) = super::paineis_armados::TABELA
                .iter()
                .find(|a| a.painel == id)
            {
                let mut host = MockPanelHost::new();
                (arm.arma)(host.store_mut());
                painel.populate(host.store_mut());
                junta(acusa, &mede(&mut host, painel));
                (arm.desarma)();
            }
        }
    });
    out
}

fn pendente(painel: &str) -> (usize, usize, usize) {
    PENDENTES
        .iter()
        .find(|p| p.0 == painel)
        .map_or((0, 0, 0), |p| (p.1, p.2, p.3))
}

/// A catraca: nem acima (defeito novo) nem abaixo (o número desceu e não foi escrito).
fn confere(medir: impl Fn(&Acusa) -> &Vec<String>, de: impl Fn((usize, usize, usize)) -> usize) {
    let tudo = censo_de_todos();
    assert!(
        tudo.len() > 10,
        "a varredura não mediu nada: {}",
        tudo.len()
    );
    let mut erros = Vec::new();
    for (painel, a) in &tudo {
        let lista = medir(a);
        let (tem, pode) = (lista.len(), de(pendente(painel)));
        if tem > pode {
            erros.push(format!("{painel}: {tem} contra {pode}: {lista:#?}"));
        } else if tem < pode {
            erros.push(format!(
                "{painel}: desceu para {tem}, escreva-o em PENDENTES"
            ));
        }
    }
    assert!(erros.is_empty(), "{}", erros.join("\n"));
}

/// ⭐ Leis 1 e 3 de `05 §4`.
#[test]
fn lei_da_compactacao() {
    confere(|a| &a.irmaos, |p| p.0);
    confere(|a| &a.grudadas, |p| p.1);
}

/// ⭐ Lei 7 de `05 §4`: nunca «…» num botão.
#[test]
fn nenhum_texto_sai_do_botao() {
    confere(|a| &a.cortados, |p| p.2);
}

/// A tabela para o tracker: `(painel, irmãos, grudadas, cortados)` e as frases.
#[test]
#[ignore = "diagnóstico: imprime a tabela, não afirma nada"]
fn diag_a_tabela_da_compactacao() {
    for (painel, a) in censo_de_todos() {
        let (i, g, t) = (a.irmaos.len(), a.grudadas.len(), a.cortados.len());
        println!(
            "(\"{painel}\", {i}, {g}, {t}), // {} linhas, {} textos",
            a.linhas, a.textos
        );
        for f in a.irmaos.iter().chain(&a.grudadas).chain(&a.cortados) {
            println!("    // {f}");
        }
    }
}

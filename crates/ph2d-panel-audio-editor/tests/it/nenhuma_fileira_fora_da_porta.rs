//! ⭐⭐⭐ **NENHUMA FILEIRA DE PARAMETRO DESTE PAINEL E PINTADA FORA DA PORTA.**
//!
//! ⛔⛔⛔ **Report do dono, 2026-09-19:** *«por que esses sliders nao sao colocados no padrao
//! do app?»* — e a resposta media foi que a MESMA forma estava escrita **cinco** vezes aqui
//! (*Effects* · *Loop* · *Delivery* · *Spectral* · *Variation*), todas anteriores a lei do
//! formulario e nenhuma convertida: rotulo CENTRADO numa linha, pista nua na linha de BAIXO.
//!
//! ⚠️⚠️ **Converter as cinco nao chega.** Uma lei que vive em cinco copias volta a divergir
//! na sexta — e a sexta seccao deste painel sera escrita por quem copiar a vizinha. ⭐ A
//! catraca esta a **ZERO**: ja nao ha linha onde escrever um slider a mao.
//!
//! ⚠️ **O que este censo NAO diz:** ele nao julga a geometria, julga a PORTA. Quem quiser
//! um controlo que a caixa unica nao exprime (o fader VERTICAL de uma tira de consola, por
//! exemplo) tem de dizer porque aqui — e e por isso que a lista de isencoes existe, vazia.

use std::fs;

/// Os pintores do corpo deste painel.
const FONTES: &[&str] = &[
    "paint_fx.rs",
    "paint_loop.rs",
    "paint_delivery.rs",
    "paint_spectral.rs",
    "paint_variation.rs",
    "paint_sections.rs",
    "paint_transport.rs",
];

/// ⛔ **A catraca, VAZIA.** `(ficheiro, porque)` — um controlo que a caixa unica nao
/// exprime entra aqui com a razao, nunca em silencio.
const FORA_DA_PORTA: &[(&str, &str)] = &[];

fn ler(nome: &str) -> Option<String> {
    fs::read_to_string(format!("{}/src/{nome}", env!("CARGO_MANIFEST_DIR"))).ok()
}

/// ⭐⭐⭐ **Nenhum pintor deste painel chama o slider cru.**
///
/// *Mutacao que deve sangrar:* repor um `paint_slider(` em qualquer um dos cinco.
#[test]
fn nenhuma_fileira_de_parametro_e_pintada_fora_da_porta() {
    // ⛔ Piso de populacao: um censo que nao acha os ficheiros varre ZERO e aprova tudo.
    let lidos: Vec<(&str, String)> = FONTES
        .iter()
        .filter_map(|n| ler(n).map(|s| (*n, s)))
        .collect();
    assert!(
        lidos.len() >= 5,
        "o censo leu {} pintores de {} — os ficheiros mudaram de nome e ele ficou a varrer nada",
        lidos.len(),
        FONTES.len()
    );
    let isentos: std::collections::BTreeSet<&str> = FORA_DA_PORTA.iter().map(|(f, _)| *f).collect();
    let crus: Vec<&str> = lidos
        .iter()
        .filter(|(n, s)| s.contains("paint_slider(") && !isentos.contains(n))
        .map(|(n, _)| *n)
        .collect();
    assert!(
        crus.is_empty(),
        "estes pintores desenham um slider FORA da caixa unica do app — o nome volta a ficar por \
         cima do controlo, que e o que o dono recusou em 2026-09-14: {crus:?}"
    );
}

/// ⭐ **E a porta TEM consumidores** — a metade que impede o censo de aprovar por vacuo.
///
/// ⚠️ Sem ela, um painel que deixasse de pintar fileiras nenhumas leria ZERO cruzes e
/// passaria: *uma catraca vazia sobre uma populacao vazia nao afirma nada*.
#[test]
fn e_a_porta_tem_os_cinco_consumidores() {
    let n = FONTES
        .iter()
        .filter_map(|f| ler(f))
        .filter(|s| s.contains("fileira_de_param::fileira_de_param("))
        .count();
    assert!(
        n >= 5,
        "so {n} pintores atravessam a porta (esperava 5) — ou uma seccao deixou de ter fileiras, \
         ou alguem a reescreveu por fora"
    );
}

/// ⛔ **A isencao envelhece sozinha** — uma linha que ja nao descreve nada tem de sair.
#[test]
fn nenhuma_isencao_da_catraca_ficou_obsoleta() {
    for (f, porque) in FORA_DA_PORTA {
        assert!(
            porque.len() > 40,
            "a isencao de `{f}` nao diz que controlo a caixa unica nao exprime"
        );
        let src = ler(f).unwrap_or_else(|| panic!("`{f}` ja nao existe — apague a isencao"));
        assert!(
            src.contains("paint_slider("),
            "`{f}` ja nao pinta um slider cru — APAGUE a isencao dele"
        );
    }
}

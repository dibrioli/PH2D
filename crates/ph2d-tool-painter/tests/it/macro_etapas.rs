//! ⭐ **Etapas editadas numa gravação com a água no WORKER** (W5b, `docs/Macros/02_plano_W5.md` §3).
//! Uma espera muda as perguntas que a thread do quadro faz ao worker — pela fita a reprodução diverge
//! (medido: `PerguntaInesperada`) —, então a trilha que toca sai da gravação na etapa deslocada e a água
//! corre no relógio fixo. Uma edição de VALOR fica na fita (o diff mínimo) enquanto a conferência passar.

use super::macro_comum::{grava_no_worker, texels_diferentes};
use ph2d_macro::MacroDoc;
use ph2d_tool_painter::macro_track::em_cena::MacroEmCena;
use ph2d_tool_painter::macro_track::etapas;

const N: u32 = 256;

fn tela(doc: &MacroDoc) -> Result<Vec<u8>, String> {
    let mut m = MacroEmCena::de(&ph2d_macro::codec::write(doc))
        .map_err(|e| format!("{e:?}"))?
        .expect("tem a trilha do Painter");
    m.segue(60.0).map_err(|e| format!("{e:?}"))?;
    Ok(m.toma_a_tela().expect("o fim assenta a água").0.to_vec())
}

#[test]
fn uma_espera_numa_gravacao_do_worker_toca_sem_divergir_e_igual_duas_vezes() {
    let original = grava_no_worker(N);
    let mut editada = original.clone();
    etapas::espera(&mut editada.tracks[0], 3, 500_000).expect("a etapa existe");
    let a = tela(&editada).expect("a espera toca sem divergir");
    let b = tela(&editada).expect("outra vez");
    assert_eq!(texels_diferentes(&a, &b), 0, "a trilha que toca é determinística");
    assert!(
        texels_diferentes(&a, &tela(&original).expect("o original toca")) > 0,
        "meio segundo mais de água muda a tela"
    );
}

#[test]
fn uma_edicao_de_valor_fica_na_fita_enquanto_a_conferencia_passa() {
    let mut doc = grava_no_worker(N);
    // LITERAL-COLOR-OK: a cor nova de uma pincelada GRAVADA, não da UI.
    etapas::edita_cor(&mut doc.tracks[0], 2, [20, 200, 40]).expect("é uma pincelada");
    etapas::assenta(&mut doc.tracks[0]).expect("confere");
    assert!(
        doc.tracks[0].edicoes.iter().all(|e| !e.sai_da_gravacao),
        "a cor não muda as perguntas ao worker: a água continua a ser a gravada"
    );
    tela(&doc).expect("toca sem divergir");
}

/// O recurso da conferência: uma edição que muda as perguntas ao worker SEM mudar o tempo (aqui, um tique
/// a mais acrescentado à mão) diverge pela fita; [`etapas::assenta`] tira essa etapa da gravação e a
/// trilha toca.
#[test]
fn a_conferencia_que_diverge_tira_a_etapa_da_gravacao() {
    use ph2d_tool_painter::macro_track::Passo;
    let mut doc = grava_no_worker(N);
    let ancora = etapas::etapas(&doc.tracks[0]).expect("lê-se")[3].etapa.inicio;
    doc.tracks[0]
        .edicao_mut(ancora)
        .insercoes
        .push((ancora, Passo::Tique(1000.0 / 60.0).codifica()));
    assert!(
        tela(&doc).is_err_and(|e| e.contains("Divergiu")),
        "controlo: pela fita, um tique a mais diverge"
    );
    etapas::assenta(&mut doc.tracks[0]).expect("assenta");
    assert!(doc.tracks[0].edicoes[0].sai_da_gravacao, "a etapa saiu da gravação");
    tela(&doc).expect("e agora toca");
}

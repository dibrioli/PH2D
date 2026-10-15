//! ⭐ **As etapas do Painter** (W5b, `docs/Macros/02_plano_W5.md` §1/§3): o Splash tem quatro
//! pinceladas com o pincel que se gravou; editar a cor, o tamanho ou a espera de UMA pincelada dá, ao
//! bit, a tela de GRAVAR o Splash já com essa mudança (o relógio fixo: a água é a mesma função do mesmo
//! tempo). Controlo em cada gate: o original difere da verdade, senão o gate não mede nada.

use super::em_cena::MacroEmCena;
use super::etapas::{self, TipoDaEtapa, tela_no_fim};
use super::roteiro::{Pincelada, SPLASH, grava_pinceladas};
use ph2d_macro::{MacroDoc, Track, codec};

const LADO: u32 = 128;
/// Meio segundo de quadros a 60 Hz.
const ESPERA: u32 = 30;
// LITERAL-COLOR-OK: a cor nova de uma pincelada GRAVADA, não da UI.
const VERDE: [u8; 3] = [20, 200, 40];

fn gravado(ps: &[Pincelada], antes: &[u32]) -> MacroDoc {
    let (b, _) = grava_pinceladas(LADO, 0, ps, antes).expect("arma no Wet Paint");
    codec::read(&b).expect("lê-se")
}

fn trilha(ps: &[Pincelada], antes: &[u32]) -> Track {
    gravado(ps, antes).tracks.remove(0)
}

fn diferentes(a: &[u8], b: &[u8]) -> usize {
    a.chunks(4).zip(b.chunks(4)).filter(|(x, y)| x != y).count()
}

fn tela(t: &Track) -> Vec<u8> {
    tela_no_fim(t).expect("toca até ao fim")
}

/// A edição dá a verdade ao bit, e o original não.
fn bate(editada: &Track, verdade: &Track, original: &Track) {
    let (e, v, o) = (tela(editada), tela(verdade), tela(original));
    assert!(
        diferentes(&o, &v) > 0,
        "controlo: a mudança não muda a tela"
    );
    assert_eq!(
        diferentes(&e, &v),
        0,
        "a trilha editada não é a gravada com a mudança"
    );
}

#[test]
fn o_splash_tem_quatro_pinceladas_com_o_pincel_que_se_gravou() {
    let tr = trilha(&SPLASH, &[0; 4]);
    let e = etapas::etapas(&tr).expect("lê-se");
    assert_eq!(e.len(), 4);
    for (k, (et, p)) in e.iter().zip(&SPLASH).enumerate() {
        assert_eq!(et.tipo, TipoDaEtapa::Pincelada(k + 1));
        assert_eq!(et.cor, p.cor, "a cor da pincelada {k}");
        assert!(
            (et.tamanho_px - p.tamanho_px).abs() < 1e-6,
            "o tamanho da pincelada {k}"
        );
        assert!(et.t0 < et.t1, "a pincelada {k} dura");
    }
    assert!(
        e.windows(2).all(|w| w[0].t1 < w[1].t0),
        "as etapas não se sobrepõem"
    );
}

#[test]
fn editar_a_cor_de_uma_pincelada_e_grava_la_com_essa_cor() {
    let original = trilha(&SPLASH, &[0; 4]);
    let mut editada = original.clone();
    etapas::edita_cor(&mut editada, 2, VERDE).expect("é uma pincelada");
    let mut ps = SPLASH;
    ps[2].cor = VERDE;
    bate(&editada, &trilha(&ps, &[0; 4]), &original);
    let e = etapas::etapas(&editada).expect("lê-se");
    assert_eq!(e[2].cor, VERDE, "o painel mostra a cor nova");
    assert_eq!(e[3].cor, SPLASH[3].cor, "e a seguinte fica com a sua");
    // Editar outra vez substitui, não acumula.
    etapas::edita_cor(&mut editada, 2, VERDE).expect("outra vez");
    bate(&editada, &trilha(&ps, &[0; 4]), &original);
}

/// A cor de uma pincelada editada não passa à seguinte que HERDAVA a dela (não gravou cor própria): a
/// cor de antes volta depois do levantar.
#[test]
fn a_cor_editada_nao_passa_a_pincelada_seguinte_que_a_herdava() {
    let mut herda = SPLASH;
    herda[3].cor = SPLASH[2].cor;
    let mut base = trilha(&herda, &[0; 4]);
    let todas = etapas::etapas(&base).expect("lê-se");
    let cor_da_4a = (todas[2].etapa.fim..todas[3].etapa.fim)
        .find(|&i| {
            matches!(
                super::passo::Passo::descodifica(&base.events[i].payload),
                Ok(super::passo::Passo::Cor(_))
            )
        })
        .expect("a 4.ª pincelada gravou a sua cor");
    base.events.remove(cor_da_4a);
    assert_eq!(
        etapas::etapas(&base).expect("lê-se")[3].cor,
        SPLASH[2].cor,
        "agora herda-a"
    );
    let mut editada = base.clone();
    etapas::edita_cor(&mut editada, 2, VERDE).expect("é uma pincelada");
    let mut ps = herda;
    ps[2].cor = VERDE;
    bate(&editada, &trilha(&ps, &[0; 4]), &base);
    assert_eq!(
        etapas::etapas(&editada).expect("lê-se")[3].cor,
        SPLASH[2].cor
    );
}

#[test]
fn apagar_uma_pincelada_e_grava_la_sem_ela() {
    let original = trilha(&SPLASH, &[0; 4]);
    let mut editada = original.clone();
    etapas::apaga(&mut editada, 2).expect("há a pincelada 2");
    // O tempo dela fecha-se e os dois intervalos à volta juntam-se: a seguinte toca 13 quadros depois
    // (o relógio do quadro do levantar e os 12 de depois dela).
    let ps = [SPLASH[0], SPLASH[1], SPLASH[3]];
    bate(&editada, &trilha(&ps, &[0, 0, 13]), &original);
    let e = etapas::etapas(&editada).expect("lê-se");
    assert_eq!(e.iter().map(|x| x.k).collect::<Vec<_>>(), [0, 1, 3]);
    assert_eq!(
        e[2].tipo,
        TipoDaEtapa::Pincelada(4),
        "a numeração é a da gravação"
    );
    assert_eq!(e[2].cor, SPLASH[3].cor, "a seguinte fica com o seu pincel");
    assert!(
        etapas::edita_cor(&mut editada, 2, VERDE).is_err(),
        "uma etapa apagada não se edita"
    );
}

#[test]
fn editar_o_tamanho_de_uma_pincelada_e_grava_la_com_esse_tamanho() {
    let original = trilha(&SPLASH, &[0; 4]);
    let mut editada = original.clone();
    etapas::edita_tamanho(&mut editada, 1, 25.0).expect("é uma pincelada");
    let mut ps = SPLASH;
    ps[1].tamanho_px = 25.0;
    bate(&editada, &trilha(&ps, &[0; 4]), &original);
}

#[test]
fn uma_espera_antes_de_uma_pincelada_e_grava_la_mais_tarde() {
    let original = trilha(&SPLASH, &[0; 4]);
    let mut editada = original.clone();
    etapas::espera(&mut editada, 2, i64::from(ESPERA) * 1_000_000 / 60).expect("a etapa existe");
    let verdade = trilha(&SPLASH, &[0, 0, ESPERA, 0]);
    bate(&editada, &verdade, &original);
    let (e, v) = (
        etapas::etapas(&editada).expect("lê-se"),
        etapas::etapas(&verdade).expect("lê-se"),
    );
    let ms = |t: ph2d_macro::MacroTime| t.0 / 1000;
    assert_eq!(
        ms(e[2].t0),
        ms(v[2].t0),
        "a pincelada começa meio segundo depois"
    );
    assert_eq!(ms(e[0].t0), ms(v[0].t0), "e as de antes não se mexem");
}

/// A cena VÊ a água a escorrer durante a espera: a meio dela a tela mostrada é a da gravação que esperou
/// (os quadros sintetizados drenam a pré-visualização como os gravados).
#[test]
fn a_meio_da_espera_a_cena_mostra_a_agua_que_a_gravacao_mostrava() {
    let mut doc = gravado(&SPLASH, &[0; 4]);
    etapas::espera(&mut doc.tracks[0], 2, i64::from(ESPERA) * 1_000_000 / 60)
        .expect("a etapa existe");
    let verdade = gravado(&SPLASH, &[0, 0, ESPERA, 0]);
    let meio = etapas::etapas(&verdade.tracks[0]).expect("lê-se")[2].t0.0 as f64 / 1e6 - 0.25;
    let no_meio = |d: &MacroDoc| {
        let mut m = MacroEmCena::de(&codec::write(d))
            .expect("lê-se")
            .expect("tem a trilha");
        m.segue(meio - 0.05).expect("toca");
        let _ = m.toma_a_tela();
        m.segue(meio).expect("toca");
        m.toma_a_tela().map(|t| t.0.to_vec())
    };
    let v = no_meio(&verdade).expect("controlo: a meio da espera a água ainda muda a tela");
    assert_eq!(
        no_meio(&doc),
        Some(v),
        "a cena não mostra a espera como a gravação a mostrava"
    );
}

#[test]
fn deslocar_o_caminho_de_uma_pincelada_muda_a_tela_e_toca_igual_duas_vezes() {
    let original = trilha(&SPLASH, &[0; 4]);
    let mut editada = original.clone();
    etapas::desloca_caminho(&mut editada, 0, [0.1, 0.0]).expect("é uma pincelada");
    let (a, b) = (tela(&editada), tela(&editada));
    assert_eq!(diferentes(&a, &b), 0);
    assert!(diferentes(&a, &tela(&original)) > 0);
    etapas::desloca_caminho(&mut editada, 0, [0.0, 0.0]).expect("volta");
    assert_eq!(
        diferentes(&tela(&editada), &tela(&original)),
        0,
        "deslocar 0 é o gravado"
    );
}

#[test]
fn a_macro_editada_toca_em_cena_como_a_gravada_com_a_mudanca() {
    let mut doc = gravado(&SPLASH, &[0; 4]);
    etapas::edita_cor(&mut doc.tracks[0], 2, VERDE).expect("é uma pincelada");
    let mut ps = SPLASH;
    ps[2].cor = VERDE;
    let verdade = gravado(&ps, &[0; 4]);
    let fim = |d: &MacroDoc| {
        let mut m = MacroEmCena::de(&codec::write(d))
            .expect("lê-se")
            .expect("tem a trilha");
        m.segue(10.0).expect("toca");
        m.toma_a_tela().expect("pintou").0
    };
    assert_eq!(diferentes(&fim(&doc), &fim(&verdade)), 0);
}

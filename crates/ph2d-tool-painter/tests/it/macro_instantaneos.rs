//! ⭐ **Os instantâneos do tocador do Painter** (W5c, `docs/Macros/02_plano_W5.md` §3): continuar um
//! instantâneo É tocar do zero, ao bit — com a água no relógio fixo (o Splash) e no worker (a fita) —; a
//! cena que recua e a macro editada recomeçam do instantâneo certo e dão o mesmo bit que um palco novo;
//! o razão cheio dobra os instantâneos sem mudar um texel.

use super::macro_comum::{grava_no_worker, texels_diferentes};
use ph2d_core::razao::Razao;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_macro::{LeitorDaFamilia, MacroDoc};
use ph2d_tool_painter::macro_track::em_cena::MacroEmCena;
use ph2d_tool_painter::macro_track::etapas::{self, LeitorDoPainter};
use ph2d_tool_painter::macro_track::roteiro::grava_o_splash;
use ph2d_tool_painter::macro_track::tocar::TocadorDoPainter;
use std::sync::Arc;

const N: u32 = 256;

fn splash() -> MacroDoc {
    let (bytes, _) = grava_o_splash(N, 0).expect("arma no Wet Paint");
    ph2d_macro::codec::read(&bytes).expect("lê-se")
}

fn as_duas() -> [(&'static str, MacroDoc); 2] {
    [("splash", splash()), ("worker", grava_no_worker(N))]
}

fn tocador(doc: &MacroDoc) -> TocadorDoPainter {
    let tr = &doc.tracks[0];
    TocadorDoPainter::novo(&tr.initial, tr.events.iter().map(|e| e.payload.as_slice()))
        .expect("nasce")
}

/// A tela inteira de um tocador que já não muda (o fim de uma reprodução).
fn tela(mut t: TocadorDoPainter) -> Vec<u8> {
    t.tool_mut().run_full().0
}

fn sem_tecto() -> Arc<Razao> {
    Razao::new(usize::MAX)
}

fn cena(doc: &MacroDoc, razao: Arc<Razao>) -> MacroEmCena {
    MacroEmCena::no_razao(&ph2d_macro::codec::write(doc), razao)
        .expect("lê-se")
        .expect("tem a trilha do Painter")
}

fn a_tela_da_cena(m: &mut MacroEmCena) -> Vec<u8> {
    m.toma_a_tela()
        .expect("a tela mudou desde a última leitura")
        .0
        .to_vec()
}

#[test]
fn continuar_o_instantaneo_de_cada_etapa_e_tocar_do_zero_ao_bit() {
    for (nome, doc) in as_duas() {
        let ev = &doc.tracks[0].events;
        let todas = LeitorDoPainter.etapas(ev);
        let inicios: Vec<usize> = todas.iter().map(|e| e.inicio).collect();
        assert!(inicios.len() >= 4, "{nome}: várias pinceladas");
        // O início de cada etapa (onde a cena os tira) e o MEIO de cada pincelada (o traço aberto).
        let pontos: Vec<usize> = todas
            .iter()
            .flat_map(|e| [e.inicio, (e.inicio + e.fim) / 2])
            .collect();
        let mut zero = tocador(&doc);
        let mut fotos = Vec::new();
        for (i, e) in ev.iter().enumerate() {
            if pontos.contains(&i) {
                let f = zero.instantaneo().expect("copia-se");
                assert_eq!(
                    f.tool().wet_pigmento(),
                    zero.tool().wet_pigmento(),
                    "{nome}: o pigmento da sessão viaja (ele responde «há tinta?» ao trocar o Blend)"
                );
                fotos.push((i, f));
            }
            zero.aplica(&e.payload).expect("toca");
        }
        assert_eq!(zero.respostas_por_pedir(), 0, "{nome}: a fita esvaziou");
        let fim = tela(zero);
        for (i, mut f) in fotos {
            for e in &ev[i..] {
                f.aplica(&e.payload).expect("o instantâneo continua");
            }
            assert_eq!(f.respostas_por_pedir(), 0, "{nome}: etapa no evento {i}");
            assert_eq!(
                texels_diferentes(&tela(f), &fim),
                0,
                "{nome}: do instantâneo {i}"
            );
        }
        // Controlo: o instantâneo de uma etapa a meio, continuado SEM o toque dela, já não é o fim.
        let (i, f) = {
            let k = inicios[inicios.len() / 2];
            let mut t = tocador(&doc);
            for e in &ev[..k] {
                t.aplica(&e.payload).expect("toca");
            }
            (k, t.instantaneo().expect("copia-se"))
        };
        let mut f = f;
        for e in &ev[i + 1..] {
            if f.aplica(&e.payload).is_err() {
                break;
            }
        }
        assert!(
            texels_diferentes(&tela(f), &fim) > 0,
            "{nome}: controlo — sem o toque"
        );
    }
}

#[test]
fn a_cena_que_recua_recomeca_do_instantaneo_e_da_o_mesmo_bit() {
    for (nome, doc) in as_duas() {
        let fim = doc.duration().0 as f64 * 1e-6;
        let todas = etapas::etapas(&doc.tracks[0]).expect("lê-se");
        let k = todas.len() / 2;
        let meio = (todas[k].t0.0 as f64 * 1e-6 + todas[k].t1.0 as f64 * 1e-6) / 2.0;
        let mut m = cena(&doc, sem_tecto());
        m.segue(fim).expect("toca até ao fim");
        assert_eq!(
            m.instantaneos().0,
            (0..todas.len()).collect::<Vec<_>>(),
            "{nome}"
        );
        m.segue(meio).expect("recua");
        assert_eq!(
            m.recomecou_de(),
            Some(k),
            "{nome}: do instantâneo da etapa em que o meio cai"
        );
        let mut nova = cena(&doc, sem_tecto());
        nova.segue(meio).expect("toca até ao meio");
        assert_eq!(nova.recomecou_de(), None);
        assert_eq!(
            texels_diferentes(&a_tela_da_cena(&mut m), &a_tela_da_cena(&mut nova)),
            0,
            "{nome}: no meio"
        );
        m.segue(fim).expect("e outra vez até ao fim");
        nova.segue(fim).expect("até ao fim");
        assert_eq!(
            texels_diferentes(&a_tela_da_cena(&mut m), &a_tela_da_cena(&mut nova)),
            0,
            "{nome}: no fim"
        );
    }
}

#[derive(Clone, Copy, Debug)]
enum Edicao {
    Cor,
    Espera,
}

#[test]
fn editar_a_etapa_k_recomeca_do_instantaneo_k_e_da_o_mesmo_bit() {
    for (nome, doc) in as_duas() {
        let fim = 60.0;
        let k = 3;
        for ed in [Edicao::Cor, Edicao::Espera] {
            let mut editada = doc.clone();
            match ed {
                // LITERAL-COLOR-OK: a cor nova de uma pincelada GRAVADA, não da UI.
                Edicao::Cor => etapas::edita_cor(&mut editada.tracks[0], k, [20, 200, 40])
                    .expect("é uma pincelada"),
                Edicao::Espera => {
                    etapas::espera(&mut editada.tracks[0], k, 400_000).expect("a etapa existe");
                }
            }
            etapas::assenta(&mut editada.tracks[0]).expect("confere");
            let bytes = ph2d_macro::codec::write(&editada);
            let mut m = cena(&doc, sem_tecto());
            m.segue(fim).expect("o original toca");
            let original = a_tela_da_cena(&mut m);
            assert!(m.reedita(&bytes).expect("lê-se"), "{nome} {ed:?}");
            assert_eq!(
                m.recomecou_de(),
                Some(k),
                "{nome} {ed:?}: do instantâneo da etapa editada"
            );
            assert_eq!(
                m.instantaneos().0,
                (0..=k).collect::<Vec<_>>(),
                "{nome} {ed:?}: os de depois da edição saíram"
            );
            m.segue(fim).expect("a editada toca");
            let mut nova = MacroEmCena::no_razao(&bytes, sem_tecto())
                .expect("lê-se")
                .expect("tem a trilha");
            nova.segue(fim).expect("a editada toca do zero");
            let editada = a_tela_da_cena(&mut nova);
            assert_eq!(
                texels_diferentes(&a_tela_da_cena(&mut m), &editada),
                0,
                "{nome} {ed:?}: do instantâneo k = do zero"
            );
            assert!(
                texels_diferentes(&original, &editada) > 0,
                "{nome} {ed:?}: controlo"
            );
        }
    }
}

#[test]
fn o_razao_cheio_dobra_os_instantaneos_e_a_cena_continua_ao_bit() {
    let doc = grava_no_worker(N);
    let fim = 60.0;
    let r_livre = sem_tecto();
    let mut livre = cena(&doc, Arc::clone(&r_livre));
    livre.segue(fim).expect("toca");
    // O instantâneo médio, com a água molhada (o tocador no fim já assentou a água: não serve de régua).
    let um = r_livre.usado() / livre.instantaneos().0.len();
    // `um` é a média sobre as 6 etapas, a 1.ª sem água (~0): o tecto `4,2·um` ≈ 3,5 instantâneos com água.
    // Cabem 3 de 5 ⇒ dobra para 2 e FICA com espaço para um 3.º — que só não se tira se o passo manda (a
    // 6.ª etapa); com 2,5 cada foto fora do passo fazia dobrar outra vez e o defeito não se via.
    let razao = Razao::new(um * 21 / 5);
    let mut m = cena(&doc, Arc::clone(&razao));
    // A cada quadro: os instantâneos vivos são sempre do passo de agora (depois de dobrar, não se tiram
    // mais entre eles).
    for q in 0..=(fim * 60.0) as u32 {
        m.segue(f64::from(q) / 60.0).expect("toca");
        let (fotos, passo) = m.instantaneos();
        assert!(
            fotos.iter().all(|e| e % passo == 0),
            "quadro {q}: {fotos:?} de {passo} em {passo}"
        );
    }
    let (fotos, passo) = m.instantaneos();
    assert!(passo >= 2, "dobrou: {passo}");
    assert!(
        fotos.len() >= 2,
        "{fotos:?}: a dobra deixa instantâneos, não os apaga"
    );
    assert!(
        fotos.iter().all(|e| e % passo == 0),
        "{fotos:?} de {passo} em {passo}"
    );
    assert!(
        razao.usado() <= razao.tecto(),
        "{} > {}",
        razao.usado(),
        razao.tecto()
    );
    m.segue(fim * 0.5).expect("recua");
    livre.segue(fim * 0.5).expect("recua");
    assert_eq!(
        texels_diferentes(&a_tela_da_cena(&mut m), &a_tela_da_cena(&mut livre)),
        0,
        "dobrar não muda um texel"
    );
    drop(m);
    assert_eq!(razao.usado(), 0, "a cena que morre devolve o que declarou");
}

#[test]
fn o_razao_do_processo_e_a_fracao_da_ram() {
    use ph2d_core::razao::{ram_fisica, tecto_pela_ram};
    use ph2d_tool_painter::macro_track::instantaneos::{RESERVA_BYTES, razao};
    assert_eq!(razao().tecto(), tecto_pela_ram(ram_fisica(), RESERVA_BYTES));
    assert!(
        Arc::ptr_eq(&razao(), &razao()),
        "um razão para o processo inteiro"
    );
}

fn e_agua(e: &ph2d_macro::Event) -> bool {
    use ph2d_tool_painter::macro_track::Passo;
    matches!(Passo::descodifica(&e.payload), Ok(Passo::Agua(_)))
}

/// A fita de um palco reeditado é a da trilha NOVA: sem a última resposta do worker, o palco que recomeça
/// de um instantâneo diverge no MESMO passo que um palco novo (com a fila do instantâneo, tocaria até ao
/// fim).
#[test]
fn a_fita_do_palco_reeditado_e_a_da_trilha_nova() {
    let doc = grava_no_worker(N);
    let mut sem_a_ultima = doc.clone();
    let ev = &mut sem_a_ultima.tracks[0].events;
    let ultima = ev.iter().rposition(e_agua).expect("o worker respondeu");
    ev.remove(ultima);
    let bytes = ph2d_macro::codec::write(&sem_a_ultima);
    let mut nova = MacroEmCena::no_razao(&bytes, sem_tecto())
        .expect("lê-se")
        .expect("trilha");
    let e_nova = nova
        .segue(60.0)
        .expect_err("controlo: falta uma resposta à fita");
    let mut m = cena(&doc, sem_tecto());
    m.segue(60.0).expect("o original toca");
    assert!(m.reedita(&bytes).expect("lê-se"));
    assert!(m.recomecou_de().is_some(), "recomeça de um instantâneo");
    assert_eq!(m.segue(60.0).expect_err("diverge como a nova"), e_nova);
    assert_eq!(m.tempo(), nova.tempo(), "no mesmo instante");
}

/// Um instantâneo que já consumiu uma resposta que a trilha nova muda não é um estado dela.
#[test]
fn o_instantaneo_que_consumiu_uma_resposta_mudada_nao_recomeca() {
    use ph2d_tool_painter::macro_track::tocar::respostas_de;
    let doc = grava_no_worker(N);
    let ev = &doc.tracks[0].events;
    let mut t = tocador(&doc);
    let meio = ev.len() / 2;
    for e in &ev[..meio] {
        t.aplica(&e.payload).expect("toca");
    }
    let todas = respostas_de(ev.iter().map(|e| e.payload.as_slice())).expect("lê-se");
    assert!(t.respostas_por_pedir() < todas.len(), "já consumiu");
    let iguais: Arc<[_]> = todas.clone().into();
    assert!(
        t.recomeca(&iguais).is_some(),
        "controlo: a mesma trilha recomeça"
    );
    let mut mudadas = todas;
    mudadas.swap(0, 1);
    assert_ne!(mudadas[0], mudadas[1], "duas respostas diferentes");
    assert!(t.recomeca(&mudadas.into()).is_none());
}

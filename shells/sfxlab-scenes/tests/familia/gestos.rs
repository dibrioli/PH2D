//! Cada controle por evento real, com o efeito lido no documento e na `Escuta` falsa.

use crate::{Mesa, pico, verdade};
use ph2d_app_sfxlab::{Alvo, Fonte};
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_sfx_dsp::{TETO_DA_ESCUTA, limitar_escuta};
use ph2d_sfx_recipe::{Source, Take, factory};

fn impact(m: &mut Mesa) -> u64 {
    let r = factory::all()
        .into_iter()
        .find(|(n, _)| *n == "Impact")
        .map(|(_, r)| r)
        .expect("Impact");
    m.poe("Impact", r);
    let id = m.abre("Impact");
    m.espera(id);
    id
}

#[test]
fn space_e_play_tocam_o_buffer_limitado_e_o_documento_continua_sem_limitador() {
    let mut m = Mesa::new();
    let id = impact(&mut m);
    // Forte de propósito: ganho alto no corpo faz o som passar do teto.
    let mut alto = m.receita(id);
    alto.layers[1].gain = 8.0;
    m.poe("Impact", alto.clone());
    m.quadro();
    m.espera(id);
    let arquivo = verdade(&alto, m.lib());
    assert!(
        pico(&arquivo) > 1.0,
        "a fixture não estoura: {}",
        pico(&arquivo)
    );

    assert!(m.tecla(BoardKey::Space));
    m.espera(id);
    assert_eq!(m.escuta.toques.len(), 1, "Space toca");
    let ouvido = &m.escuta.toques[0];
    assert!(pico(ouvido) <= TETO_DA_ESCUTA, "{}", pico(ouvido));
    assert_eq!(
        *ouvido,
        limitar_escuta(&arquivo, ph2d_sfx_recipe::SAMPLE_RATE)
    );
    assert_eq!(
        m.docs().pcm(id).expect("pcm"),
        &*arquivo,
        "o documento sem limitador"
    );

    m.clica(id, Alvo::Play);
    m.quadro();
    assert_eq!(m.escuta.toques.len(), 2, "o botão Play toca");

    assert!(m.tecla(BoardKey::Escape));
    assert_eq!(m.escuta.paradas, 1, "Esc para");
    m.clica(id, Alvo::Stop);
    assert_eq!(m.escuta.paradas, 2, "o botão Stop para");
}

#[test]
fn clique_em_m_tira_a_camada_do_som_e_grava_no_documento() {
    let mut m = Mesa::new();
    let id = impact(&mut m);
    let antes = m.hash(id);
    let c = m.camada(id, 0);
    m.clica(id, Alvo::Mudo(c));
    assert!(m.receita(id).layer(c).expect("camada").muted);
    m.espera(id);
    assert_ne!(m.hash(id), antes);
    let r = m.receita(id);
    assert_eq!(m.lib().recipes["Impact"], r);
    m.soa_como(id, &r);
}

#[test]
fn solo_nao_entra_no_arquivo() {
    let mut m = Mesa::new();
    let id = impact(&mut m);
    let (r, h, blob) = (m.receita(id), m.hash(id), m.lib().to_bytes());
    let c = r.layers[2].id;

    m.clica(id, Alvo::Solo(c));
    assert_eq!(m.docs().solo(id), Some(c));
    m.espera(id);
    assert_eq!(m.receita(id), r, "o documento não muda");
    assert_eq!(m.hash(id), h, "o som do arquivo não muda");
    assert_eq!(m.lib().to_bytes(), blob, "o blob não muda");
    assert_eq!(
        m.desfaz(false),
        Some(false),
        "solo não é um passo de desfazer"
    );

    assert!(m.tecla(BoardKey::Space));
    m.espera(id);
    let so = verdade(&r.solo(c), m.lib());
    let ouvido = m.escuta.toques.last().expect("tocou");
    assert_eq!(
        *ouvido,
        limitar_escuta(&so, ph2d_sfx_recipe::SAMPLE_RATE),
        "a escuta ouve o solo"
    );

    m.clica(id, Alvo::Solo(c));
    assert_eq!(m.docs().solo(id), None);
    m.espera(id);
    assert!(m.tecla(BoardKey::Space));
    m.espera(id);
    let tudo = verdade(&r, m.lib());
    let ouvido = m.escuta.toques.last().expect("tocou");
    assert_eq!(*ouvido, limitar_escuta(&tudo, ph2d_sfx_recipe::SAMPLE_RATE));
}

#[test]
fn delete_apaga_a_camada_selecionada_por_clique() {
    let mut m = Mesa::new();
    let id = impact(&mut m);
    let c = m.camada(id, 2);
    let p = m.centro(id, Alvo::Onda(c));
    m.carrega(p);
    m.larga(p);
    m.quadro();
    assert_eq!(
        m.docs().selecionada(id),
        Some(c),
        "clicar na onda seleciona"
    );
    let antes = m.receita(id);
    assert!(m.tecla(BoardKey::Delete));
    assert!(m.receita(id).layer(c).is_none());
    assert_eq!(m.receita(id).layers.len(), antes.layers.len() - 1);
    assert_eq!(m.docs().selecionada(id), None);
    assert!(m.tecla(BoardKey::Delete), "sem seleção não apaga mais nada");
    assert_eq!(m.receita(id).layers.len(), antes.layers.len() - 1);
    assert_eq!(m.desfaz(false), Some(true));
    assert_eq!(m.receita(id), antes);
    m.espera(id);
    m.soa_como(id, &antes);
}

#[test]
fn add_layer_acrescenta_e_a_camada_toca() {
    let mut m = Mesa::new();
    let id = impact(&mut m);
    let n = m.receita(id).layers.len();
    let h = m.hash(id);
    m.clica(id, Alvo::Add);
    assert!(
        m.docs().rect_de(id, Alvo::Item(Fonte::Noise)).is_some(),
        "o menu abriu"
    );
    m.clica(id, Alvo::Item(Fonte::Tone));
    let r = m.receita(id);
    assert_eq!(r.layers.len(), n + 1);
    assert!(matches!(r.layers[n].source, Source::Tone(_)));
    assert_eq!(m.docs().selecionada(id), Some(r.layers[n].id));
    assert!(
        m.docs().rect_de(id, Alvo::Item(Fonte::Noise)).is_none(),
        "o menu fechou"
    );
    m.espera(id);
    assert_ne!(m.hash(id), h, "a camada nova entra no som");
}

#[test]
fn recording_so_com_material_e_sem_material_diz_o_motivo() {
    let mut m = Mesa::new();
    let id = impact(&mut m);
    let n = m.receita(id).layers.len();
    m.clica(id, Alvo::Add);
    assert!(m.docs().rect_de(id, Alvo::Item(Fonte::Recording)).is_none());
    m.clica(id, Alvo::ItemOff(Fonte::Recording));
    assert_eq!(m.receita(id).layers.len(), n, "desabilitado não acrescenta");
    assert!(
        m.docs()
            .rect_de(id, Alvo::ItemOff(Fonte::Recording))
            .is_some(),
        "e o menu fica"
    );
    assert!(m.tecla(BoardKey::Escape));
    assert_eq!(
        m.escuta.paradas, 0,
        "o Esc fecha o menu antes de parar o som"
    );

    m.mundo
        .resource_mut::<ph2d_sfx_recipe::SfxLibrary>()
        .takes
        .add(Take::new(48_000, 1, vec![0.5; 4800]));
    m.quadro();
    m.clica(id, Alvo::Add);
    m.clica(id, Alvo::Item(Fonte::Recording));
    let r = m.receita(id);
    assert_eq!(r.layers.len(), n + 1);
    assert!(matches!(r.layers[n].source, Source::Recording(_)));
    m.espera(id);
    m.soa_como(id, &r);
}

#[test]
fn arrastar_a_onda_muda_start_e_a_onda_acompanha() {
    let mut m = Mesa::new();
    let id = impact(&mut m);
    let c = m.camada(id, 1);
    let r0 = m.docs().rect_de(id, Alvo::Onda(c)).expect("onda");
    let de = (r0.x + 10.0, r0.y + r0.h / 2.0);
    let dx = 120.0;
    m.arrasta(de, (de.0 + dx, de.1), 4);
    let start = m.receita(id).layer(c).expect("camada").start;
    let pista = r0.w; // começa em 0 e cobre a pista inteira
    let esperado = dx / pista * m.receita(id).seconds;
    assert!((start - esperado).abs() < 1.0e-3, "{start} × {esperado}");
    let r1 = m.docs().rect_de(id, Alvo::Onda(c)).expect("onda");
    assert!(
        (r1.x - (r0.x + dx)).abs() < 0.5,
        "o bloco desenha na posição do start"
    );
    m.espera(id);
    let r = m.receita(id);
    m.soa_como(id, &r);

    // Arrastar para muito antes do zero para em zero.
    let p = m.docs().rect_de(id, Alvo::Onda(c)).expect("onda");
    let de = (p.x + 10.0, p.y + p.h / 2.0);
    m.arrasta(de, (-500.0, de.1), 2);
    assert_eq!(m.receita(id).layer(c).expect("camada").start, 0.0);
}

#[test]
fn a_alca_reordena_sem_mudar_o_som_e_com_um_passo_de_desfazer() {
    let mut m = Mesa::new();
    let id = impact(&mut m);
    let (r0, h0) = (m.receita(id), m.hash(id));
    let primeira = r0.layers[0].id;
    let de = m.centro(id, Alvo::Alca(primeira));
    let ultima = m.centro(id, Alvo::Alca(r0.layers[3].id));
    m.arrasta(de, ultima, 3);
    let r1 = m.receita(id);
    assert_eq!(r1.layers.last().map(|l| l.id), Some(primeira));
    m.espera(id);
    assert_eq!(m.hash(id), h0, "a ordem da pilha não muda um bit");
    assert_eq!(m.desfaz(false), Some(true));
    assert_eq!(m.receita(id), r0);
}

#[test]
fn play_on_change_toca_depois_de_uma_edicao_e_de_um_gesto_nao_durante() {
    let mut m = Mesa::new();
    let id = impact(&mut m);
    m.clica(id, Alvo::PlayOnChange);
    assert!(m.escuta.toques.is_empty(), "ligar não toca");
    let c = m.camada(id, 0);
    m.clica(id, Alvo::Mudo(c));
    m.espera(id);
    assert_eq!(m.escuta.toques.len(), 1);

    let c = m.camada(id, 1);
    let de = m.centro(id, Alvo::Onda(c));
    assert!(m.carrega(de));
    m.move_((de.0 + 20.0, de.1));
    m.espera(id);
    assert_eq!(m.escuta.toques.len(), 1, "durante o arrasto, silêncio");
    assert!(m.larga((de.0 + 20.0, de.1)));
    m.espera(id);
    assert_eq!(m.escuta.toques.len(), 2);
}

/// Ligar o S toca só aquela faixa; desligar toca o som inteiro de novo.
#[test]
fn play_on_change_toca_a_faixa_quando_o_solo_muda() {
    let mut m = Mesa::new();
    let id = impact(&mut m);
    let c = m.camada(id, 0);
    let inteiro = m.docs().escuta(id).expect("escuta").to_vec();
    m.clica(id, Alvo::Solo(c));
    m.clica(id, Alvo::Solo(c));
    m.espera(id);
    assert!(m.escuta.toques.is_empty(), "desligado, o S não toca");

    m.clica(id, Alvo::PlayOnChange);
    m.clica(id, Alvo::Solo(c));
    m.espera(id);
    assert_eq!(m.escuta.toques.len(), 1, "o S não tocou");
    let solo = m.docs().escuta(id).expect("escuta").to_vec();
    assert_ne!(solo, inteiro, "a fixture não distingue o solo");
    assert_eq!(m.escuta.toques[0], solo, "não tocou a faixa em solo");

    m.clica(id, Alvo::Solo(c));
    m.espera(id);
    assert_eq!(m.escuta.toques.len(), 2, "tirar o S não tocou");
    assert_eq!(m.escuta.toques[1], inteiro, "não voltou ao som inteiro");
}

#[test]
fn a_regua_poe_o_cursor_e_o_play_comeca_ali() {
    let mut m = Mesa::new();
    let id = impact(&mut m);
    let r = m.docs().rect_de(id, Alvo::Regua).expect("régua");
    let p = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    m.carrega(p);
    m.larga(p);
    m.quadro();
    assert!(m.tecla(BoardKey::Space));
    m.espera(id);
    let tudo = m.docs().pcm(id).expect("pcm").len();
    let tocado = m.escuta.toques[0].len();
    assert!(tocado.abs_diff(tudo / 2) <= 2, "{tocado} de {tudo}");
}

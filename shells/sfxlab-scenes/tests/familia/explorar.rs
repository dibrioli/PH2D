//! A exploração pela mão: tecla e clique reais, efeito lido na receita, na lista Heard e no que a escuta recebeu.

use crate::pico;
use crate::{Mesa, verdade};
use ph2d_app_sfxlab::{Alvo, AlvoE, FILHOS, TAMANHO_MUTATE, TAMANHO_ROLL};
use ph2d_audio::dsp::maior_momento;
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_editor_core::screens::hero::board_view;
use ph2d_sfx_dsp::limitar_escuta;
use ph2d_sfx_explore::{Grade, varia};
use ph2d_sfx_lab::medida::ALVO_DA_ESCUTA;
use ph2d_sfx_recipe::{
    Approved, Body, Como, Envelope, ModeLaw, Noise, NoiseFilter, Recipe, SAMPLE_RATE, SfxLibrary,
    Source, factory,
};

fn fabrica(nome: &str) -> Recipe {
    factory::all()
        .into_iter()
        .find(|(n, _)| *n == nome)
        .map(|(_, r)| r)
        .expect("som de fábrica")
}

fn aberta(nome: &str) -> (Mesa, u64) {
    let mut m = Mesa::new();
    m.poe(nome, fabrica(nome));
    let id = m.abre(nome);
    m.espera(id);
    (m, id)
}

fn char_(c: char) -> BoardKey {
    BoardKey::Char(c)
}

/// Quadros até nada mais estar a caminho.
fn assenta(m: &mut Mesa, id: u64) {
    let fim = std::time::Instant::now() + std::time::Duration::from_secs(60);
    loop {
        m.quadro();
        let parado = !m.docs().em_voo(id) && !m.docs().exploracao(id).expect("aba").em_voo();
        if parado {
            m.quadro();
            return;
        }
        assert!(std::time::Instant::now() < fim, "não assentou");
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

fn ouvidos(m: &mut Mesa, id: u64) -> usize {
    m.docs().exploracao(id).expect("aba").sessao.ouvidos.len()
}

#[test]
fn roll_pela_tecla_e_pelo_botao_muda_o_som_e_toca_no_mesmo_volume() {
    let (mut m, id) = aberta("Metal Hit");
    let antes = m.receita(id);
    assert!(m.tecla(char_('r')), "a aba toma o R");
    assert_ne!(m.receita(id), antes);
    assenta(&mut m, id);
    let toque = m.escuta.toques.pop().expect("o Roll toca sozinho");
    assert!((maior_momento(&toque, SAMPLE_RATE) - ALVO_DA_ESCUTA).abs() < 0.5);
    let r = m.receita(id);
    m.soa_como(id, &r);
    m.clica(id, Alvo::Ex(AlvoE::Roll));
    assert_ne!(m.receita(id), r, "o botão faz o mesmo");
    assenta(&mut m, id);
    assert_eq!(ouvidos(&mut m, id), 2);
}

#[test]
fn space_na_tela_normal_toca_o_volume_real() {
    let (mut m, id) = aberta("Metal Hit");
    assert!(m.tecla(char_('r')));
    assenta(&mut m, id);
    let nivelado = m.escuta.toques.pop().expect("Roll");
    assert!(m.tecla(BoardKey::Space));
    assenta(&mut m, id);
    let real = m.escuta.toques.pop().expect("Space");
    assert_ne!(real, nivelado);
    let (a, b) = (
        maior_momento(&real, SAMPLE_RATE),
        maior_momento(&nivelado, SAMPLE_RATE),
    );
    assert!((a - b).abs() > 0.5, "o Space não pode nivelar: {a} × {b}");
}

#[test]
fn desfazer_cobre_o_sorteio() {
    let (mut m, id) = aberta("Impact");
    let inicio = m.receita(id);
    let hash = m.hash(id);
    assert!(m.tecla(char_('r')));
    assenta(&mut m, id);
    let um = m.receita(id);
    assert!(m.tecla(char_('m')));
    assenta(&mut m, id);
    assert_ne!(m.receita(id), um);
    assert_eq!(m.desfaz(false), Some(true));
    assert_eq!(m.receita(id), um);
    assert_eq!(m.desfaz(false), Some(true));
    assert_eq!(m.receita(id), inicio);
    assenta(&mut m, id);
    assert_eq!(
        m.hash(id),
        hash,
        "o desfazer devolve o som, não só os campos"
    );
    assert_eq!(ouvidos(&mut m, id), 2, "desfazer não apaga o que se ouviu");
    assert_eq!(m.desfaz(true), Some(true));
    assert_eq!(m.receita(id), um);
}

#[test]
fn sortear_nao_troca_o_som_do_jogo() {
    let mut m = Mesa::new();
    let mut r = fabrica("Metal Hit");
    let aprovado = Approved {
        sound: "Metal Hit".into(),
        blake3: [9; 32],
        engine: r.engine,
    };
    r.approved = Some(aprovado.clone());
    m.poe("Metal Hit", r);
    let id = m.abre("Metal Hit");
    m.espera(id);
    for _ in 0..100 {
        assert!(m.tecla(char_('r')));
    }
    assenta(&mut m, id);
    assert_eq!(m.receita(id).approved, Some(aprovado.clone()));
    assert_eq!(m.lib().recipes["Metal Hit"].approved, Some(aprovado));
}

#[test]
fn heard_ouve_com_um_clique_e_traz_de_volta_com_dois() {
    let (mut m, id) = aberta("Impact");
    assert!(m.tecla(char_('r')));
    assenta(&mut m, id);
    let primeiro = m.receita(id);
    assert!(m.tecla(char_('r')));
    assenta(&mut m, id);
    let segundo = m.receita(id);
    assert_eq!(ouvidos(&mut m, id), 2);
    m.escuta.toques.clear();

    // Um clique: ouve, sem virar o atual.
    m.clica(id, Alvo::Ex(AlvoE::Ouvido(0)));
    assenta(&mut m, id);
    assert_eq!(m.receita(id), segundo);
    assert_eq!(m.escuta.toques.len(), 1, "a linha toca");

    // Dois: vira o atual, e o som é o de antes.
    m.agora += 2_000_000_000;
    m.clica(id, Alvo::Ex(AlvoE::Ouvido(0)));
    m.clica(id, Alvo::Ex(AlvoE::Ouvido(0)));
    assenta(&mut m, id);
    assert_eq!(m.receita(id), primeiro);
    m.soa_como(id, &primeiro);
    assert_eq!(m.desfaz(false), Some(true), "um passo de desfazer");
    assert_eq!(m.receita(id), segundo);
}

#[test]
fn keep_da_a_estrela_e_o_filtro_mostra_so_as_marcadas() {
    let (mut m, id) = aberta("Impact");
    assert!(m.tecla(char_('r')));
    assenta(&mut m, id);
    assert!(m.tecla(char_('k')));
    assert!(m.tecla(char_('r')));
    assenta(&mut m, id);
    m.quadro();
    let (a, b) = {
        let s = &m.docs().exploracao(id).expect("aba").sessao;
        (s.ouvidos[0].estrela, s.ouvidos[1].estrela)
    };
    assert!(a && !b);
    assert!(m.docs().rect_de(id, Alvo::Ex(AlvoE::Ouvido(1))).is_some());
    m.clica(id, Alvo::Ex(AlvoE::SoKept));
    m.quadro();
    assert!(m.docs().so_kept(id));
    assert!(
        m.docs().rect_de(id, Alvo::Ex(AlvoE::Ouvido(1))).is_none(),
        "fora do filtro"
    );
    assert!(m.docs().rect_de(id, Alvo::Ex(AlvoE::Ouvido(0))).is_some());
    // A estrela também se tira na linha.
    m.clica(id, Alvo::Ex(AlvoE::Estrela(0)));
    assert!(!m.docs().exploracao(id).expect("aba").sessao.ouvidos[0].estrela);
}

#[test]
fn heard_sobrevive_a_salvar_e_abrir() {
    let (mut m, id) = aberta("Impact");
    assert!(m.tecla(char_('r')));
    assenta(&mut m, id);
    assert!(m.tecla(char_('k')));
    m.quadro();
    let bytes = m.lib().to_bytes();
    let lida = SfxLibrary::from_bytes(&bytes).expect("blob");
    assert_eq!(lida.ouvidos["Impact"], m.lib().ouvidos["Impact"]);
    let ouvida = lida.ouvidos["Impact"][0].clone();

    let mut outra = Mesa::new();
    outra.mundo.insert_resource(lida);
    let id2 = outra.abre("Impact");
    outra.espera(id2);
    outra.quadro();
    assert_eq!(
        outra.docs().exploracao(id2).expect("aba").sessao.ouvidos,
        vec![ouvida]
    );
}

#[test]
fn gravar_heard_nao_zera_o_desfazer() {
    let (mut m, id) = aberta("Impact");
    let inicio = m.receita(id);
    assert!(m.tecla(char_('r')));
    assenta(&mut m, id);
    assert!(m.tecla(char_('r')));
    assenta(&mut m, id);
    assert!(
        !m.lib().ouvidos["Impact"].is_empty(),
        "a lista foi ao mundo"
    );
    assert_eq!(m.desfaz(false), Some(true));
    assert_eq!(m.desfaz(false), Some(true));
    assert_eq!(
        m.receita(id),
        inicio,
        "a escrita da lista não pode zerar o desfazer"
    );
}

#[test]
fn tab_entra_e_sai_e_nao_vaza() {
    let (mut m, id) = aberta("Metal Hit");
    assert!(m.tecla(BoardKey::Tab), "o Tab é da aba, não do troca-modo");
    assert!(m.docs().exploracao(id).expect("aba").aberta());
    assert!(m.docs().rect_de(id, Alvo::Ex(AlvoE::Filho(7))).is_some());
    assert!(
        m.docs().rect_de(id, Alvo::Regua).is_none(),
        "a grade toma o lugar da pilha"
    );
    assert!(m.tecla(BoardKey::Tab));
    assert!(!m.docs().exploracao(id).expect("aba").aberta());
    assert!(m.docs().rect_de(id, Alvo::Regua).is_some());
    // O Esc também sai, e não para o som por engano.
    assert!(m.tecla(BoardKey::Tab));
    assert!(m.tecla(BoardKey::Escape));
    assert!(!m.docs().exploracao(id).expect("aba").aberta());
}

#[test]
fn os_digitos_so_sao_da_aba_dentro_do_explore() {
    let (mut m, id) = aberta("Metal Hit");
    assert!(!m.tecla(char_('3')), "fora do Explore o dígito não é dela");
    assert!(m.tecla(BoardKey::Tab));
    assert!(m.tecla(char_('3')));
    assert!(!m.tecla(char_('9')), "não há nona variação");
    assenta(&mut m, id);
    assert_eq!(
        m.docs()
            .exploracao(id)
            .expect("aba")
            .grade
            .as_ref()
            .expect("grade")
            .ultimo,
        Some(2)
    );
}

#[test]
fn explore_a_escolhida_vira_a_atual_e_nascem_oito_novas() {
    let (mut m, id) = aberta("Impact");
    let inicio = m.receita(id);
    assert!(m.tecla(BoardKey::Tab));
    assenta(&mut m, id);
    m.escuta.toques.clear();
    m.clica(id, Alvo::Ex(AlvoE::Filho(4)));
    assenta(&mut m, id);
    assert_eq!(m.escuta.toques.len(), 1, "clicar toca");
    let volume = maior_momento(&m.escuta.toques[0], SAMPLE_RATE);
    assert!((volume - ALVO_DA_ESCUTA).abs() < 0.5, "{volume}");
    assert_eq!(m.receita(id), inicio, "ouvir não muda o som atual");
    let (filho, numero) = {
        let g = m
            .docs()
            .exploracao(id)
            .expect("aba")
            .grade
            .as_ref()
            .expect("grade");
        (
            g.filhos[4].as_ref().expect("filho").receita.clone(),
            g.como(4),
        )
    };

    assert!(m.tecla(BoardKey::Enter));
    assenta(&mut m, id);
    assert_eq!(m.receita(id), filho, "a escolhida vira a atual");
    m.soa_como(id, &filho);
    let g = m
        .docs()
        .exploracao(id)
        .expect("aba")
        .grade
        .as_ref()
        .expect("grade");
    assert_eq!(g.base, filho, "as oito novas são em volta dela");
    assert!(g.filhos.iter().all(Option::is_some));
    assert_ne!(g.como(4), numero);
    assert_eq!(g.ultimo, None);

    assert_eq!(m.desfaz(false), Some(true));
    assert_eq!(m.receita(id), inicio, "um passo de desfazer");
    assenta(&mut m, id);
    let g = m
        .docs()
        .exploracao(id)
        .expect("aba")
        .grade
        .as_ref()
        .expect("grade");
    assert_eq!(
        g.base, inicio,
        "e as oito voltam a ser em volta do de antes"
    );
}

#[test]
fn enter_sem_ter_ouvido_nada_nao_muda_o_som() {
    let (mut m, id) = aberta("Impact");
    let inicio = m.receita(id);
    assert!(m.tecla(BoardKey::Tab));
    assenta(&mut m, id);
    assert!(m.tecla(BoardKey::Enter));
    assert_eq!(m.receita(id), inicio);
}

#[test]
fn next_no_explore_percorre_as_oito_e_traz_outras() {
    let (mut m, id) = aberta("Metal Hit");
    assert!(m.tecla(BoardKey::Tab));
    assenta(&mut m, id);
    m.escuta.toques.clear();
    for _ in 0..FILHOS {
        assert!(m.tecla(char_('n')));
        assenta(&mut m, id);
    }
    assert_eq!(m.escuta.toques.len(), FILHOS);
    let antes = m
        .docs()
        .exploracao(id)
        .expect("aba")
        .grade
        .as_ref()
        .expect("g")
        .como(0);
    assert!(m.tecla(char_('n')));
    assenta(&mut m, id);
    let g = m
        .docs()
        .exploracao(id)
        .expect("aba")
        .grade
        .as_ref()
        .expect("g");
    assert_ne!(g.como(0), antes, "depois da oitava, oito novas");
    assert_eq!(g.ultimo, Some(0));
    assert_eq!(m.escuta.toques.len(), FILHOS + 1, "e a primeira toca");
}

#[test]
fn arrastar_a_variacao_para_longe_muda_mais_e_para_perto_devolve_a_base() {
    let (mut m, id) = aberta("Impact");
    let inicio = m.receita(id);
    assert!(m.tecla(BoardKey::Tab));
    assenta(&mut m, id);
    let de = m.centro(id, Alvo::Ex(AlvoE::Filho(0)));
    let no_centro = m.centro(id, Alvo::Ex(AlvoE::Atual));
    let tamanho = |m: &mut Mesa| {
        m.docs()
            .exploracao(id)
            .expect("aba")
            .grade
            .as_ref()
            .expect("g")
            .tamanhos[0]
    };
    let t0 = tamanho(&mut m);
    // Para longe do centro.
    let longe = (
        de.0 - (no_centro.0 - de.0) * 0.3,
        de.1 - (no_centro.1 - de.1) * 0.3,
    );
    m.arrasta(de, longe, 4);
    assenta(&mut m, id);
    assert!(tamanho(&mut m) > t0, "mais longe, mais mudança");
    // Até o som atual: o filho é a própria base.
    m.arrasta(de, no_centro, 4);
    assenta(&mut m, id);
    assert_eq!(tamanho(&mut m), 0.0);
    let certo = verdade(&inicio, &m.lib().clone());
    let g = m
        .docs()
        .exploracao(id)
        .expect("aba")
        .grade
        .as_ref()
        .expect("g");
    assert_eq!(g.filhos[0].as_ref().expect("filho").arquivo, certo);
}

#[test]
fn soltar_o_arrasto_toca_o_som_novo_e_nao_o_velho() {
    let (mut m, id) = aberta("Impact");
    assert!(m.tecla(BoardKey::Tab));
    assenta(&mut m, id);
    m.escuta.toques.clear();
    let de = m.centro(id, Alvo::Ex(AlvoE::Filho(2)));
    let no_centro = m.centro(id, Alvo::Ex(AlvoE::Atual));
    m.arrasta(de, no_centro, 2);
    assenta(&mut m, id);
    assert_eq!(m.escuta.toques.len(), 1);
    let base = m.docs().pcm(id).expect("pcm").to_vec();
    let nivelada = ph2d_sfx_lab::medida::escuta_nivelada(&base, SAMPLE_RATE);
    assert_eq!(m.escuta.toques[0], nivelada, "toca o que acabou de render");
}

#[test]
fn o_change_refaz_as_oito_com_o_tamanho_novo() {
    let (mut m, id) = aberta("Impact");
    assert!(m.tecla(BoardKey::Tab));
    assenta(&mut m, id);
    let antes = m
        .docs()
        .exploracao(id)
        .expect("aba")
        .grade
        .as_ref()
        .expect("g")
        .como(0);
    m.clica(id, Alvo::Ex(AlvoE::Mudanca));
    assenta(&mut m, id);
    let g = m
        .docs()
        .exploracao(id)
        .expect("aba")
        .grade
        .as_ref()
        .expect("g");
    assert_ne!(g.como(0), antes);
    assert!(
        (g.tamanhos[3] - ph2d_app_sfxlab::TAMANHO_ROLL / 2.0).abs() < 1.0e-3,
        "o meio da faixa"
    );
}

#[test]
fn o_l_da_linha_trava_a_camada_e_o_roll_a_respeita() {
    let (mut m, id) = aberta("Impact");
    let camada = m.camada(id, 0);
    m.clica(id, Alvo::Trava(camada));
    let travada = m.receita(id).layers[0].clone();
    assert!(travada.travada);
    for _ in 0..30 {
        assert!(m.tecla(char_('r')));
    }
    assenta(&mut m, id);
    let depois = m.receita(id);
    assert_eq!(depois.layers[0], travada);
    assert_ne!(depois.layers[1], fabrica("Impact").layers[1], "controle");
    // A tecla L, com o mouse em cima da linha, destrava.
    let linha = m.centro(id, Alvo::Linha(camada));
    m.move_(linha);
    assert!(m.tecla(char_('l')));
    assert!(!m.receita(id).layers[0].travada);
    assert_eq!(m.desfaz(false), Some(true), "travar também se desfaz");
    assert!(m.receita(id).layers[0].travada);
}

#[test]
fn o_botao_lock_trava_a_camada_escolhida() {
    let (mut m, id) = aberta("Impact");
    let camada = m.camada(id, 1);
    m.clica(id, Alvo::Linha(camada));
    m.clica(id, Alvo::Ex(AlvoE::Lock));
    assert!(m.receita(id).layers[1].travada);
}

#[test]
fn com_a_janela_do_use_in_game_as_teclas_nao_sorteiam() {
    let (mut m, id) = aberta("Impact");
    let antes = m.receita(id);
    m.clica(id, Alvo::UseInGame);
    m.quadro();
    let _ = m.tecla(char_('r'));
    assert_eq!(m.receita(id), antes);
}

#[test]
fn som_novo_vazio_soa_no_primeiro_roll() {
    let mut m = Mesa::new();
    let id = m.som_novo();
    m.espera(id);
    m.docs();
    assert!(m.tecla(char_('r')));
    assenta(&mut m, id);
    assert!(!m.receita(id).layers.is_empty());
    assert!(!m.escuta.toques.is_empty());
}

/// A lista guarda a receita sem o aprovado: trazer um som dela não pode desaprovar o que está no jogo.
#[test]
fn trazer_do_heard_nao_desaprova_o_som_do_jogo() {
    let mut m = Mesa::new();
    let mut r = fabrica("Metal Hit");
    let aprovado = Approved {
        sound: "Metal Hit".into(),
        blake3: [9; 32],
        engine: r.engine,
    };
    r.approved = Some(aprovado.clone());
    m.poe("Metal Hit", r);
    let id = m.abre("Metal Hit");
    m.espera(id);
    assert!(m.tecla(char_('r')));
    assenta(&mut m, id);
    assert!(m.tecla(char_('r')));
    assenta(&mut m, id);
    let linha = m.docs().exploracao(id).expect("aba").sessao.ouvidos[0].clone();
    assert_eq!(linha.receita.approved, None, "a fixture contém o fenômeno");
    m.clica(id, Alvo::Ex(AlvoE::Ouvido(0)));
    m.clica(id, Alvo::Ex(AlvoE::Ouvido(0)));
    assenta(&mut m, id);
    let atual = m.receita(id);
    assert_eq!(atual.layers, linha.receita.layers, "o som voltou");
    assert_eq!(atual.approved, Some(aprovado));
}

// ── As auditorias da W4 ──

/// Um corpo longo: o render demora o bastante para a tecla chegar antes dele. Cada gate confere que chegou.
fn lenta() -> Recipe {
    let mut r = Recipe::new(5, 12.0);
    r.add(
        1.0,
        Source::Body(Body {
            law: ModeLaw::Bar,
            base_hz: 40.0,
            modes: 500,
            t60: 3.0,
            damping: 0.0,
            rolloff: 0.5,
            contact_ms: 1.0,
            noise: 0.0,
        }),
        Envelope::default(),
    );
    r
}

fn aberta_com(r: Recipe) -> (Mesa, u64) {
    let mut m = Mesa::new();
    m.poe("Som", r);
    let id = m.abre("Som");
    m.espera(id);
    m.escuta.toques.clear();
    (m, id)
}

fn grade_de(m: &mut Mesa, id: u64) -> &Grade {
    m.docs()
        .exploracao(id)
        .expect("aba")
        .grade
        .as_ref()
        .expect("grade")
}

fn numero(g: &Grade, i: usize) -> u64 {
    match g.como(i) {
        Como::Variacao(n) => n,
        c => panic!("{c:?}"),
    }
}

#[test]
fn tab_no_meio_do_arrasto_fecha_o_gesto_e_o_mute_seguinte_e_outro_passo() {
    let (mut m, id) = aberta("Impact");
    let inicio = m.receita(id);
    m.clica(id, Alvo::PlayOnChange);
    let camada = m.camada(id, 1);
    let de = m.centro(id, Alvo::Onda(camada));
    assert!(m.carrega(de));
    m.move_((de.0 + 40.0, de.1));
    m.quadro();
    let movida = m.receita(id);
    assert_ne!(movida, inicio, "o arrasto mexeu");
    assert!(m.tecla(BoardKey::Tab));
    assert!(m.docs().exploracao(id).expect("aba").aberta());
    let _ = m.larga((de.0 + 40.0, de.1));
    assert!(m.tecla(BoardKey::Tab));
    assenta(&mut m, id);
    m.escuta.toques.clear();

    m.clica(id, Alvo::Mudo(camada));
    assert!(m.receita(id).layers[1].muted);
    assenta(&mut m, id);
    assert_eq!(m.escuta.toques.len(), 1, "com Play on change, o Mute toca");

    assert_eq!(m.desfaz(false), Some(true));
    assert_eq!(m.receita(id), movida, "desfazer reverte só o Mute");
    assert_eq!(m.desfaz(false), Some(true));
    assert_eq!(m.receita(id), inicio, "e depois só o arrasto");
}

#[test]
fn esc_no_meio_do_arrasto_tambem_fecha_o_gesto() {
    let (mut m, id) = aberta("Impact");
    assert!(m.tecla(BoardKey::Tab));
    assert!(m.tecla(BoardKey::Tab));
    let camada = m.camada(id, 0);
    let ultima = m.receita(id).layers.last().expect("camada").id;
    let de = m.centro(id, Alvo::Alca(camada));
    let para = m.centro(id, Alvo::Alca(ultima));
    assert!(m.carrega(de));
    m.move_(para);
    m.quadro();
    let movida = m.receita(id);
    assert_ne!(movida.layers[0].id, camada, "a ordem mudou");
    assert!(m.tecla(BoardKey::Tab), "entra no Explore com a alça presa");
    assert!(m.tecla(BoardKey::Escape));
    assert!(!m.docs().exploracao(id).expect("aba").aberta());
    let _ = m.larga(para);
    m.clica(id, Alvo::Mudo(camada));
    assert_eq!(m.desfaz(false), Some(true));
    assert_eq!(m.receita(id), movida);
}

#[test]
fn sortear_com_gesto_em_curso_e_ignorado_mas_a_tecla_e_tomada() {
    let (mut m, id) = aberta("Impact");
    let camada = m.camada(id, 1);
    let de = m.centro(id, Alvo::Onda(camada));
    assert!(m.carrega(de));
    m.move_((de.0 + 40.0, de.1));
    m.quadro();
    let movida = m.receita(id);
    for c in ['r', 'm', 'n'] {
        assert!(m.tecla(char_(c)), "{c} é tomada");
        assert_eq!(m.receita(id), movida, "{c} no meio do arrasto");
    }
    let _ = m.larga((de.0 + 40.0, de.1));

    let inicio = m.receita(id);
    assert!(m.tecla(BoardKey::Tab));
    assenta(&mut m, id);
    assert!(m.tecla(char_('1')));
    assenta(&mut m, id);
    let (como, de) = (
        grade_de(&mut m, id).como(0),
        m.centro(id, Alvo::Ex(AlvoE::Filho(2))),
    );
    assert!(m.carrega(de));
    m.move_((de.0, de.1 - 30.0));
    m.quadro();
    for k in [BoardKey::Enter, char_('r'), char_('m'), char_('n')] {
        assert!(m.tecla(k), "{k:?} é tomada");
        assert_eq!(m.receita(id), inicio, "{k:?} no meio do arrasto");
        let g = grade_de(&mut m, id);
        assert_eq!(g.ultimo, Some(0), "{k:?} não anda");
        assert_eq!(g.como(0), como, "{k:?} não refaz as oito");
    }
    let _ = m.larga((de.0, de.1 - 30.0));
}

#[test]
fn n_e_k_antes_do_render_dao_a_estrela_a_variacao_ouvida() {
    let base = lenta();
    let (mut m, id) = aberta_com(base.clone());
    assert!(m.tecla(BoardKey::Tab));
    assert!(m.tecla(char_('n')));
    assert!(m.tecla(char_('k')));
    let g = grade_de(&mut m, id);
    assert!(g.filhos[0].is_none(), "a fixture: ainda a renderizar");
    let (certa, como) = (varia(&base, numero(g, 0), g.tamanhos[0]), g.como(0));
    let lista = m.docs().exploracao(id).expect("aba").sessao.ouvidos.clone();
    assert_eq!(lista.len(), 1, "{lista:?}");
    assert!(lista[0].estrela);
    assert_eq!(lista[0].receita, certa);
    assert_eq!(lista[0].como, como);
    assert!(
        lista.iter().all(|o| o.receita != base),
        "a base sem estrela"
    );
}

#[test]
fn arrastar_e_enter_antes_do_render_a_atual_e_a_arrastada() {
    let base = lenta();
    let (mut m, id) = aberta_com(base.clone());
    assert!(m.tecla(BoardKey::Tab));
    assenta(&mut m, id);
    let de = m.centro(id, Alvo::Ex(AlvoE::Filho(1)));
    m.arrasta(de, (de.0, de.1 - 60.0), 1);
    let g = grade_de(&mut m, id);
    assert_eq!(g.ultimo, Some(1));
    let certa = varia(&base, numero(g, 1), g.tamanhos[1]);
    assert_ne!(
        g.filhos[1].as_ref().expect("a velha").receita,
        certa,
        "a fixture: o render novo ainda não chegou"
    );
    assert!(m.tecla(BoardKey::Enter));
    assert_eq!(m.receita(id), certa);
}

#[test]
fn k_logo_apos_arrastar_marca_a_arrastada() {
    let base = lenta();
    let (mut m, id) = aberta_com(base.clone());
    assert!(m.tecla(BoardKey::Tab));
    assenta(&mut m, id);
    let de = m.centro(id, Alvo::Ex(AlvoE::Filho(1)));
    m.arrasta(de, (de.0, de.1 - 60.0), 1);
    let g = grade_de(&mut m, id);
    let certa = varia(&base, numero(g, 1), g.tamanhos[1]);
    assert_ne!(g.filhos[1].as_ref().expect("a velha").receita, certa);
    assert!(m.tecla(char_('k')));
    let lista = &m.docs().exploracao(id).expect("aba").sessao.ouvidos;
    assert!(lista.iter().any(|o| o.estrela && o.receita == certa));
}

#[test]
fn n_e_r_antes_do_render_nao_tocam_nada() {
    let (mut m, id) = aberta_com(lenta());
    assert!(m.tecla(BoardKey::Tab));
    assert!(m.tecla(char_('n')));
    assert!(grade_de(&mut m, id).filhos[0].is_none(), "a fixture");
    assert!(m.tecla(char_('r')));
    assenta(&mut m, id);
    assert!(m.escuta.toques.is_empty(), "ninguém pediu para tocar");
    assert_eq!(ouvidos(&mut m, id), 0, "o que não tocou não entra no Heard");
}

/// O sorteio ainda a renderizar, seguido de uma das três portas do volume real.
fn roll_pendente(fim: impl FnOnce(&mut Mesa, u64)) {
    let (mut m, id) = aberta_com(lenta());
    assert!(m.tecla(char_('r')));
    assert!(m.docs().em_voo(id), "a fixture: o Roll ainda não tocou");
    assert!(m.escuta.toques.is_empty(), "a fixture");
    fim(&mut m, id);
    assenta(&mut m, id);
    let arquivo = verdade(&m.receita(id), m.lib());
    assert_eq!(m.escuta.toques.len(), 1);
    assert!(
        m.escuta.toques[0] == limitar_escuta(&arquivo, SAMPLE_RATE),
        "o volume real"
    );
}

#[test]
fn roll_e_esc_nao_deixam_o_space_nivelado() {
    roll_pendente(|m, _| {
        assert!(m.tecla(BoardKey::Escape));
        assert!(m.tecla(BoardKey::Space));
    });
}

#[test]
fn space_sobre_um_roll_pendente_toca_o_volume_real() {
    roll_pendente(|m, _| {
        assert!(m.tecla(BoardKey::Space));
    });
}

#[test]
fn o_botao_play_sobre_um_roll_pendente_toca_o_volume_real() {
    roll_pendente(|m, id| m.clica(id, Alvo::Play));
}

#[test]
fn roll_e_esc_nao_deixam_o_play_on_change_nivelado() {
    roll_pendente(|m, id| {
        m.clica(id, Alvo::PlayOnChange);
        assert!(m.tecla(BoardKey::Escape));
        let camada = m.camada(id, 0);
        m.clica(id, Alvo::Solo(camada));
    });
}

#[test]
fn rolar_e_l_trava_a_camada_que_esta_sob_a_mao() {
    let mut r = Recipe::new(3, 0.1);
    for _ in 0..30 {
        r.add(
            0.1,
            Source::Noise(Noise {
                filter: NoiseFilter::White,
                hz: 800.0,
                hz_end: 800.0,
                q: 1.0,
            }),
            Envelope::default(),
        );
    }
    let (mut m, id) = aberta_com(r);
    let primeira = m.camada(id, 0);
    let mao = m.centro(id, Alvo::Linha(primeira));
    assert!(m.move_(mao));
    m.quadro();
    assert!(board_view::wheel(&mut m.hero, mao.0, mao.1, -10.0));
    m.quadro();
    let receita = m.receita(id);
    let sob = receita
        .layers
        .iter()
        .position(|l| {
            m.docs()
                .rect_de(id, Alvo::Linha(l.id))
                .is_some_and(|c| c.contains(mao.0, mao.1))
        })
        .expect("uma linha sob a mão");
    assert_ne!(sob, 0, "a fixture: a pilha rolou");
    assert!(m.tecla(char_('l')));
    let depois = m.receita(id);
    assert!(depois.layers[sob].travada, "a que está sob a mão");
    assert!(!depois.layers[0].travada, "não a de antes de rolar");
}

#[test]
fn a_mao_no_meio_do_arrasto_nao_acende_o_que_passa_por_baixo() {
    let (mut m, id) = aberta("Impact");
    let (a, b) = (m.camada(id, 0), m.camada(id, 1));
    let de = m.centro(id, Alvo::Onda(a));
    let sobre = m.centro(id, Alvo::Mudo(b));
    assert!(m.carrega(de));
    m.move_(sobre);
    m.quadro();
    assert_eq!(m.docs().hover(id), None, "nada acende durante o gesto");
    let _ = m.larga(sobre);
    m.quadro();
    assert_eq!(
        m.docs().hover(id),
        Some(Alvo::Mudo(b)),
        "e acende ao soltar"
    );
}

fn com_ganho(g: f32) -> Recipe {
    let mut r = fabrica("Impact");
    for l in &mut r.layers {
        l.gain = g;
    }
    r
}

#[test]
fn so_a_variacao_que_estoura_avisa() {
    for (ganho, estoura) in [(20.0, true), (0.01, false)] {
        let (mut m, id) = aberta_com(com_ganho(ganho));
        assert!(m.tecla(BoardKey::Tab));
        assenta(&mut m, id);
        for i in 0..FILHOS {
            let pico = pico(&grade_de(&mut m, id).filhos[i].as_ref().expect("f").arquivo);
            assert_eq!(pico > 1.0, estoura, "a fixture, ganho {ganho}: {pico}");
            let aviso = m.docs().rect_de(id, Alvo::Ex(AlvoE::Estoura(i as u8)));
            assert_eq!(aviso.is_some(), estoura, "célula {i}, pico {pico}");
        }
    }
}

/// O aviso não pode roubar o clique da célula.
#[test]
fn clicar_no_aviso_ainda_toca_a_variacao() {
    let (mut m, id) = aberta_com(com_ganho(20.0));
    assert!(m.tecla(BoardKey::Tab));
    assenta(&mut m, id);
    m.clica(id, Alvo::Ex(AlvoE::Estoura(3)));
    assert_eq!(grade_de(&mut m, id).ultimo, Some(3));
}

#[test]
fn com_erro_o_space_e_o_play_nao_tocam_o_render_antigo() {
    let (mut m, id) = aberta_com(fabrica("Impact"));
    let boa = m.receita(id);
    let mut quebrada = boa.clone();
    quebrada.engine += 1;
    m.poe("Som", quebrada);
    m.quadro();
    m.espera(id);
    assert!(m.docs().pcm(id).is_some(), "a fixture: há render antigo");
    assert!(m.tecla(BoardKey::Space));
    m.quadro();
    m.clica(id, Alvo::Play);
    m.quadro();
    assert!(
        m.escuta.toques.is_empty(),
        "o som velho não finge ser o novo"
    );
    // O pedido morreu ali: consertar o som não toca sozinho.
    m.poe("Som", boa);
    m.quadro();
    m.espera(id);
    assenta(&mut m, id);
    assert!(m.escuta.toques.is_empty());
}

fn tamanho_de(m: &mut Mesa, id: u64, i: usize) -> f64 {
    grade_de(m, id).tamanhos[i]
}

#[test]
fn arrastar_a_variacao_e_continuo_mesmo_saindo_do_zero() {
    let (mut m, id) = aberta("Impact");
    assert!(m.tecla(BoardKey::Tab));
    assenta(&mut m, id);
    // O `Change` no meio: a régua do arrasto é ele, e não o tamanho do Mutate.
    m.clica(id, Alvo::Ex(AlvoE::Mudanca));
    assenta(&mut m, id);
    let base = grade_de(&mut m, id).tamanho;
    assert!((base - TAMANHO_MUTATE).abs() > 0.1, "a fixture: {base}");
    let centro = m.centro(id, Alvo::Ex(AlvoE::Atual));
    let de = m.centro(id, Alvo::Ex(AlvoE::Filho(1)));
    let d0 = ((de.0 - centro.0).powi(2) + (de.1 - centro.1).powi(2)).sqrt();
    let a = |px: f32| {
        let t = (d0 + px) / d0;
        (
            centro.0 + (de.0 - centro.0) * t,
            centro.1 + (de.1 - centro.1) * t,
        )
    };

    // Zero: a variação é o som atual.
    m.arrasta(de, centro, 2);
    assert_eq!(tamanho_de(&mut m, id, 1), 0.0);
    // Sair do zero anda aos poucos, sem salto.
    assert!(m.carrega(de));
    m.move_(a(20.0));
    m.quadro();
    let t = tamanho_de(&mut m, id, 1);
    let certo = base * 20.0 / f64::from(d0);
    assert!((t - certo).abs() < 1.0e-3, "{t} × {certo}");
    // Mais perto do que partiu: o piso é zero, nunca negativo.
    m.move_(a(-20.0));
    m.quadro();
    assert_eq!(tamanho_de(&mut m, id, 1), 0.0);
    // De volta ao ponto de partida: exato.
    m.move_(a(2.0));
    m.quadro();
    assert_eq!(tamanho_de(&mut m, id, 1), 0.0);
    let _ = m.larga(a(2.0));

    // Fora do zero: soma ao tamanho de onde partiu.
    let de3 = m.centro(id, Alvo::Ex(AlvoE::Filho(3)));
    let d3 = ((de3.0 - centro.0).powi(2) + (de3.1 - centro.1).powi(2)).sqrt();
    let t0 = tamanho_de(&mut m, id, 3);
    assert!(m.carrega(de3));
    let t3 = (d3 + 20.0) / d3;
    let longe = (
        centro.0 + (de3.0 - centro.0) * t3,
        centro.1 + (de3.1 - centro.1) * t3,
    );
    m.move_(longe);
    m.quadro();
    let t = tamanho_de(&mut m, id, 3);
    let certo = t0 + base * 20.0 / f64::from(d3);
    assert!((t - certo).abs() < 1.0e-3, "{t} × {certo}");
    // Muito longe: o teto.
    m.move_((longe.0 - 100_000.0, longe.1));
    m.quadro();
    assert_eq!(tamanho_de(&mut m, id, 3), 2.0 * TAMANHO_ROLL);
    let _ = m.larga(de3);
}

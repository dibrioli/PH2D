//! A exploração pela mão: tecla e clique reais, efeito lido na receita, na lista Heard e no que a escuta recebeu.

use crate::{Mesa, verdade};
use ph2d_app_sfxlab::{Alvo, AlvoE, FILHOS};
use ph2d_audio::dsp::maior_momento;
use ph2d_editor_core::screens::hero::board_keys::BoardKey;
use ph2d_sfx_lab::medida::ALVO_DA_ESCUTA;
use ph2d_sfx_recipe::{Approved, Recipe, SAMPLE_RATE, SfxLibrary, factory};

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

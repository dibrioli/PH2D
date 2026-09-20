//! ⭐⭐⭐ **O pedido da fileira do gatilho CHEGA ao Input Map** — a metade que o gate do painel não
//! pode ver.
//!
//! # ⚠️ Porquê sobre o TEXTO do quadro, e não por comportamento
//!
//! O gate de costura do Inspector (`o_botao_que_cria_a_accao_so_existe_onde_ela_falta` +
//! `criar_a_accao_leva_o_nome_da_fileira_aberta_ao_barramento`) prova o caminho até ao
//! **barramento** e pára ali. O consumidor mora numa fase que pede `self.gfx` e o `HeroScreen`, que
//! nenhum teste de unidade alcança — *um gate de unidade é cego à fiação da shell*, a lição que
//! esta casa pagou com vinte gates verdes sobre um `draw` cravado em `true`.
//!
//! ⇒ o que se afirma é a **CORRENTE**: o braço do dreno guarda o pedido, o quadro drena-o, e a fase
//! cria a acção **e** re-sincroniza as linhas do painel do mapa.
//!
//! # ⛔ E a metade da RE-SINCRONIZAÇÃO é a que um gate ingénuo esqueceria
//!
//! Sem ela a acção nasce e a janela do *Input Map* — se estiver aberta — continua a mostrar a lista
//! **antiga**: o artista vê a queixa desaparecer no Inspector e nada aparecer ali. É a mesma chamada
//! que o botão `Add` daquela janela já faz, e é a razão de isto ser uma FASE em vez de uma linha
//! solta no dreno.

/// O laço do quadro como ele CORRE (as fases emendadas pela ordem de execução).
fn frame_src() -> String {
    crate::frame_text::render_frame()
}

/// **A corrente inteira, pela ordem em que ela corre.**
///
/// ⚠️ **A ordem é a afirmação:** drenar depois de criar deixaria o pedido deste quadro a ser
/// atendido no seguinte, e a fase a correr sobre uma lista vazia.
#[test]
fn o_quadro_drena_o_pedido_cria_a_accao_e_re_sincroniza_o_painel() {
    let s = frame_src();
    let dreno = s
        .find("create_input_actions")
        .expect("o quadro drena os pedidos de acção que a fileira do gatilho empurrou");
    let cria = s[dreno..]
        .find("input_map.create(")
        .expect("e o pedido vira uma acção NOVA no mapa do editor");
    let sincroniza = s[dreno + cria..]
        .find("sync_input_map_rows(")
        .expect("e as linhas do painel do mapa são re-sincronizadas DEPOIS de ela nascer");
    assert!(
        sincroniza > 0,
        "a ordem é drenar → criar → re-sincronizar; ela é o caminho inteiro"
    );
}

/// ⭐⭐ **O braço do barramento guarda o pedido — e ele NÃO é uma edição de componente.**
///
/// ⚠️ O que nasce é estado do **EDITOR**: não viaja num `ComponentBlob`, não passa pelo ledger do
/// `preview_drive` e não é do `Ctrl+Z` do documento. Um braço que o empurrasse como `ComponentEdit`
/// compilaria e poria uma linha do Input Map dentro do ficheiro do projecto.
#[test]
fn o_braco_do_barramento_guarda_o_pedido_e_nao_o_trata_como_documento() {
    let src = include_str!("../../src/render_loop/fase_bus_inspector.rs");
    let braco = src
        .find("EditorAction::CreateInputAction")
        .expect("o dreno do barramento conhece o pedido");
    let guarda = src[braco..]
        .find("create_input_actions.push(")
        .expect("e guarda-o no pendente que o quadro drena");
    assert!(
        guarda < 400,
        "o braço tem de GUARDAR o pedido ali mesmo — {guarda} bytes depois é outro braço"
    );
}

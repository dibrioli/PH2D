//! ⛔⛔⛔ **A TECLA `K` É UM GESTO, E UM GESTO ACONTECE UMA VEZ** — o gate do report do dono de
//! 2026-09-17 (*«ainda acontece de criar keys em todo lugar»*).
//!
//! # O defeito que este ficheiro existe para impedir
//!
//! O corpo da [`super::fase_timeline_key_insert`] vivia dentro de
//! `if self.timeline_insert_key { self.timeline_insert_key = false; … }`, na
//! [`super::fase_timeline_containers`]. O corte por responsabilidade que o trouxe para cá (tecto de
//! LOC, 200) levou **a atribuição** e deixou **o `if`** do outro lado ⇒ a fase passou a autorar uma
//! chave em cada track ligada do objecto seleccionado, **em cada quadro**.
//!
//! ⚠️ **Nada o acusou:** compila, o clippy cala-se, e as suítes ficam verdes — *um corpo separado do
//! guarda dele é código correcto que corre na hora errada*.
//!
//! # Porque o gate é TEXTUAL, e porque isso aqui é honesto
//!
//! O modo de falha é **um corte**, que é uma operação de texto; e a fase não é alcançável de um
//! teste (ela pede o `AppGfx`, que precisa de um device). ⇒ a régua mede a FORMA que o corte
//! destrói, por [`include_str!`] — que **deixa de compilar** se o ficheiro mudar de sítio, ao
//! contrário de uma leitura por caminho em runtime.
//!
//! ⚠️ **Com controlo positivo:** sem a asserção de que esta fase de facto AUTORA uma chave, o gate
//! ficaria verde sobre um ficheiro vazio, renomeado ou esvaziado por outro corte — que é a forma
//! muda de que este repo já pagou o preço (um censo que varre zero é trivialmente verde).

/// O texto da fase. `include_str!` e não `fs::read_to_string`: um caminho em runtime falha **quando
/// o teste corre**, e este falha **a compilar**, que é o aviso barato.
const FASE: &str = include_str!("fase_timeline_key_insert.rs");
/// O texto da fase de onde o corpo saiu — a outra metade do par.
const CONTAINERS: &str = include_str!("fase_timeline_containers.rs");

/// A bandeira que o teclado levanta e que **só** esta fase pode baixar.
const BANDEIRA: &str = "timeline_insert_key";

/// ⭐ **CONTROLO POSITIVO: esta fase é mesmo a que autora uma chave.**
///
/// Sem isto, todas as asserções abaixo passariam sobre um ficheiro que já não faz nada — e o gate
/// leria «verde» exactamente no dia em que deixasse de ter sujeito.
#[test]
fn a_fase_medida_e_mesmo_a_que_autora_uma_chave() {
    for agulha in ["TimelineIntent::AddKey", "TimelineIntent::AddPathKey"] {
        assert!(
            FASE.contains(agulha),
            "esta fase devia ser a que autora chaves, e nao contem `{agulha}` — o gate abaixo \
             estaria a medir um ficheiro sem sujeito"
        );
    }
}

/// ⛔⛔⛔ **A bandeira é CONSUMIDA, nunca só baixada.**
///
/// A metade negativa é o defeito à letra: uma atribuição solta (`= false`) é o que um corte deixa
/// para trás quando leva o corpo e esquece o `if`.
#[test]
fn a_tecla_k_e_consumida_por_um_take_e_nunca_por_uma_atribuicao_solta() {
    let take = format!("std::mem::take(&mut self.{BANDEIRA})");
    assert!(
        FASE.contains(&take),
        "a bandeira do `K` tem de ser LIDA e BAIXADA na mesma operacao (`{take}`): duas metades \
         que podem ser separadas acabam separadas, e foi isso que pos a fase a correr por quadro"
    );
    let solta = format!("self.{BANDEIRA} = false");
    assert!(
        !FASE.contains(&solta),
        "`{solta}` e' a forma ORFA — a atribuicao que sobrevive ao `if` que a guardava. \
         Consuma a bandeira com `std::mem::take`"
    );
}

/// ⛔⛔ **O consumo vem ANTES de todo trabalho da fase** — inclusive do empréstimo do `gfx` e do
/// `prime_rooted`.
///
/// ⚠️ Não é estética: com o consumo a jusante, um quadro sem `K` continua a re-primar o *scratch*
/// da pilha (trabalho por quadro que ninguém pediu), e — pior — a ordem volta a poder ser separada
/// por um corte, que é exactamente o defeito que este ficheiro existe para impedir.
#[test]
fn o_consumo_da_bandeira_e_a_primeira_coisa_que_a_fase_faz() {
    let take = format!("std::mem::take(&mut self.{BANDEIRA})");
    let i_take = FASE.find(&take).expect("o take tem de existir");
    for depois in ["self.gfx.as_mut()", "prime_rooted", "iter_selected"] {
        let i = FASE
            .find(depois)
            .unwrap_or_else(|| panic!("`{depois}` devia estar nesta fase"));
        assert!(
            i_take < i,
            "o consumo da bandeira ({i_take}) tem de vir ANTES de `{depois}` ({i}) — um quadro \
             sem `K` nao pode pagar trabalho nenhum desta fase"
        );
    }
}

/// ⛔ **E a fase de onde o corpo saiu já não fala da bandeira** — se ela voltar a falar, há duas
/// respostas à pergunta *«o artista carregou no K?»*, e elas divergem no dia do terceiro corte.
#[test]
fn a_fase_de_onde_o_corpo_saiu_nao_guarda_a_metade_de_la() {
    assert!(
        !CONTAINERS.contains(BANDEIRA),
        "`{BANDEIRA}` voltou a aparecer na fase dos conteineres: o guarda e o corpo tem de viver \
         no MESMO ficheiro, senao um corte volta a separa-los"
    );
    // Controlo positivo do segundo ficheiro: ele é mesmo quem CHAMA esta fase.
    assert!(
        CONTAINERS.contains("self.fase_timeline_key_insert("),
        "a fase dos conteineres devia ser quem chama esta — o gate estaria a medir o ficheiro errado"
    );
}

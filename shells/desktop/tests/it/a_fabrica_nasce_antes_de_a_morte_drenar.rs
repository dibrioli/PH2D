//! ⭐⭐⭐ **A ORDEM da fábrica e da morte dentro do quadro** (TOP-20 #11 e #12).
//!
//! Quatro marcos, e **cada troca de par é um defeito distinto que nenhum teste de unidade
//! alcança**:
//!
//! 1. a tabela **nome → acção** resolve (um `SignalActions` que arranca um timer tem de correr
//!    antes de a fábrica ler o sinal, senão o efeito dele espera um quadro);
//! 2. a **fábrica** lê os sinais deste quadro e pede as cópias;
//! 3. as cópias **nascem** (`apply_births`);
//! 4. a **morte drena** (`apply_deaths`) — ⛔ por último, e é a lei que o oráculo mediu: no Godot
//!    um nó pedido para morrer continua **válido, na árvore e em toda consulta** até ao fim do
//!    quadro. Drenar antes de (3) faria uma cópia com vida de um tique morrer no quadro em que
//!    nasce; drenar antes de (2) tiraria do mundo quem a fábrica ainda ia contar no `alive_max`.
//!
//! ⚠️ **O gate lê o texto EMENDADO do quadro** (`frame_text::render_frame`), onde a posição de um
//! literal é a ordem em que ele corre — o mesmo oráculo dos gates de ordem irmãos. ⚠️ É por isso
//! que a fase-filha se chama `fase_fabrica_e_morte`: o texto emendado colhe só `fn fase_*`, e com
//! outro nome ela **desaparece** dali sem nada ficar vermelho.

/// Os quatro marcos, na ordem em que TÊM de aparecer.
const ORDER: &[(&str, &str)] = &[
    (
        "ph2d_ecs::resolve_signal_actions(",
        "a tabela nome -> accao resolve",
    ),
    (
        "ph2d_ecs::tick_factories(",
        "a FABRICA le' os sinais deste quadro",
    ),
    ("factory_bridge::apply_births(", "as copias NASCEM"),
    ("factory_bridge::apply_deaths(", "a MORTE drena, por ultimo"),
];

#[test]
fn the_factory_reads_the_signal_before_the_death_drains() {
    let text = crate::frame_text::render_frame();
    let mut at = Vec::new();
    for (needle, what) in ORDER {
        let hits = text.matches(needle).count();
        // Um marco que aparece duas vezes não tem POSIÇÃO — e o `find` pegaria a primeira em
        // silêncio, que é como um gate de ordem começa a medir outra função.
        assert_eq!(
            hits, 1,
            "o marco `{needle}` ({what}) aparece {hits}x no texto do quadro. Um gate de ORDEM só \
             pode falar de um marco que existe uma vez."
        );
        at.push((text.find(needle).expect("acabou de ser contado"), *what));
    }
    for par in at.windows(2) {
        assert!(
            par[0].0 < par[1].0,
            "a ordem do quadro inverteu-se: «{}» tem de vir ANTES de «{}»",
            par[0].1,
            par[1].1
        );
    }
}

/// ⭐⭐ **A varredura do rebobinar é um INVARIANTE, não um gancho num botão.**
///
/// ⚠️ O transporte tem mais de um caminho até ao zero (o botão, o arrasto da régua, o reset do
/// documento ao apagar o último objecto animado), e um gancho em cada um é a lista que envelhece —
/// a mesma forma que o `import_router` curou (*uma lista escrita à mão ao lado de um predicado*).
/// Este gate exige que a varredura seja lida do RELÓGIO.
#[test]
fn the_sweep_is_read_from_the_clock_and_not_hooked_to_a_button() {
    let text = crate::frame_text::render_frame();
    assert_eq!(
        text.matches("factory_bridge::sweep_spawned(").count(),
        1,
        "a varredura do rebobinar tem de ter UM sítio no quadro"
    );
    assert!(
        crate::frame_text::find_chain(&text, "!a_correr && self.playhead.time()", "<= 0.0")
            .is_some(),
        "a varredura deixou de ser derivada do relogio — procure um gancho num botao"
    );
}

/// ⭐⭐⭐ **A corrida é quando o RELÓGIO ANDA**, e a fábrica é gateada nisso.
///
/// ⚠️ Sem esta cerca uma fábrica enche a cena **enquanto o artista edita**: não há modo de jogo
/// neste app e os relógios do passo fixo correm sempre, logo um `Timer` com `autostart` publicaria
/// o sinal dela no primeiro quadro depois de ela ser anexada.
#[test]
fn the_factory_only_runs_while_the_clock_plays() {
    let text = crate::frame_text::render_frame();
    assert!(
        text.contains("let a_correr = self.playhead.is_playing();"),
        "a fabrica deixou de perguntar se o relogio anda"
    );
    let cerca = text
        .find("let disparados = if a_correr { disparados } else { Vec::new() };")
        .or_else(|| crate::frame_text::find_chain(&text, "if a_correr", "{ disparados }"));
    assert!(
        cerca.is_some(),
        "os sinais chegam a' fabrica sem passar pela cerca do relogio"
    );
}

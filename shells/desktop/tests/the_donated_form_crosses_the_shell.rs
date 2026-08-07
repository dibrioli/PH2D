//! **Arch-gate da costura da DOAÇÃO** — o plano de forma atravessa o shell, e atravessa por um
//! canal que não conhece o módulo 3D.
//!
//! ## Por que um gate de TEXTO
//!
//! Os três fatos abaixo moram em funções que **nenhum teste de unidade alcança**: o
//! `painter_bridge::dispatch` toma vinte e um argumentos vivos (o `HeroScreen`, o `SimWorld`, o
//! `SpriteRenderer`), e `App::sculpt3d_donate_form` exige uma janela e um device. Quando a
//! verificação só é possível sobre o FONTE, é sobre o fonte que ela se faz — o precedente é o
//! `the_z_projection_reads_the_tree_after_the_sync` ao lado.
//!
//! ⚠️ **Cada asserção é uma PROPRIEDADE, nunca uma distância em bytes.** A `line/Vector` fechou
//! uma jornada inteira com dois arch-gates vermelhos por terem sido escritos como *"a menos de 400
//! bytes de"*: o proxy expira no dia em que alguém acrescenta uma linha no meio, e o produto
//! continua certo.

const BRIDGE: &str = include_str!("../src/render_loop/painter_bridge.rs");
const LOOP: &str = include_str!("../src/render_loop/mod.rs");
const SCENE: &str = include_str!("../src/sculpt3d.rs");
/// ⚠️ **O GESTO mora num arquivo irmão** (`sculpt3d_input.rs`), e as portas que
/// este gate interroga moram lá. A separação é de responsabilidade — *o que a
/// cena É* contra *o que a mão FAZ* — e ela nasceu de um teto de LOC, então pode
/// se mover de novo: por isso as buscas usam [`scene_and_gesture`] em vez de
/// nomear o arquivo de cada função.
const INPUT: &str = include_str!("../src/sculpt3d_input.rs");

/// ⚠️ **E o DESENHO mora num terceiro** (`sculpt3d_view.rs`), pelo mesmo motivo e
/// pelo mesmo gatilho: um canal de sombreamento novo (o AO de tela) cruzou o teto
/// de 600 LOC do pai e o `render` saiu para cá. **Este gate reprovou nessa
/// mudança** — ele estava CERTO sobre um produto que tinha se mudado, que é a
/// classe de defeito que este repo já pagou três vezes. A cura é a que o
/// comentário acima já prescrevia: nomear o arquivo UMA vez, na concatenação, e
/// afirmar a PROPRIEDADE sobre a família.
const VIEW: &str = include_str!("../src/sculpt3d_view.rs");

/// A cena, o gesto e o desenho como um só texto — ver [`INPUT`] e [`VIEW`].
fn scene_and_gesture() -> String {
    let all = format!("{SCENE}\n{INPUT}\n{VIEW}");
    // **Controle positivo.** Um `include_str!` que apontasse para um arquivo
    // esvaziado por um corte deixaria toda busca abaixo devolver "não achei" —
    // e o gate falaria com confiança sobre um texto que não existe.
    assert!(
        all.len() > 10_000 && all.contains("impl Sculpt3dScene"),
        "a familia do modulo 3D nao foi lida: {} bytes",
        all.len()
    );
    all
}
const DONATION: &str = include_str!("../src/sculpt3d_donation.rs");

/// **As DUAS metades do canal existem no bridge.**
///
/// Publicar o tamanho sem consumir a notícia dá um produtor rasterizando planos que ninguém
/// instala; consumir sem publicar dá um produtor que nunca sabe quão grande rasterizar e fica
/// mudo para sempre. Nenhuma das duas falhas produz erro — as duas produzem *nada acontece*.
#[test]
fn the_bridge_publishes_the_canvas_and_installs_the_news() {
    assert!(
        BRIDGE.contains("donated_form.canvas ="),
        "o bridge tem de PUBLICAR o tamanho do canvas — é o único que pode perguntá-lo ao tool"
    );
    assert!(
        BRIDGE.contains("donated_form.news.take()"),
        "…e tem de CONSUMIR a notícia; `take` e não leitura, senão a doação é reinstalada por frame"
    );
    assert!(
        BRIDGE.contains("painter.set_donated_form("),
        "…instalando-a no tool, que é o que faz a tinta acender"
    );
}

/// **O canal não menciona o módulo 3D**, e é isso que mantém a promessa do `docs/3D/02.3`.
///
/// ⚠️ O gate é sobre o BRIDGE de propósito: ele é o consumidor, e um consumidor que precisasse de
/// `ph2d_mesh` para instalar um plano de floats amarraria o Painter à escultura. Apagar o módulo
/// tem de deixar este arquivo compilando.
#[test]
fn the_consumer_does_not_know_what_a_mesh_is() {
    const FORBIDDEN: [&str; 4] = ["ph2d_mesh", "ph2d_sculpt3d", "Sculpt3dScene", "sculpt3d"];
    // ⚠️ **CONTROLE POSITIVO.** Isto é uma busca NEGATIVA, e uma busca negativa sobre o arquivo
    // errado (renomeado, movido, `include_str!` apontando para um vazio) passa por vácuo — verde
    // porque não achou nada, em vez de verde porque não HÁ nada. As mesmas agulhas têm de ser
    // encontráveis onde elas de fato vivem.
    for needle in FORBIDDEN {
        assert!(
            DONATION.contains(needle) || SCENE.contains(needle),
            "controle: `{needle}` tem de aparecer no PRODUTOR — se não aparece em nenhum dos dois \
             arquivos, este gate está lendo a coisa errada e o `!contains` abaixo não vale nada"
        );
    }
    for forbidden in FORBIDDEN {
        assert!(
            !BRIDGE.contains(forbidden),
            "`{forbidden}` apareceu no painter_bridge — o canal da doação é `Vec<f32>`, e o \
             consumidor não pode conhecer o produtor (docs/3D/02.3, regra 2)"
        );
    }
}

/// **O produtor roda no laço**, e sob `cfg`.
///
/// Sem a chamada, tudo o mais desta wave existe e nada acontece: a cena esculpe, o interruptor
/// cicla, o bridge publica um tamanho que ninguém lê. Um gate de comportamento não pega isso —
/// ele testaria a função que ninguém chama.
#[test]
fn the_frame_asks_the_module_to_donate() {
    let call = LOOP
        .find("self.sculpt3d_donate_form();")
        .expect("o laço tem de chamar `sculpt3d_donate_form` — sem isso a doação nunca sai");
    let before = &LOOP[..call];
    let cfg = before
        .rfind("#[cfg(feature = \"sculpt3d\")]")
        .expect("a chamada tem de estar sob a feature — a promessa de removibilidade é literal");
    assert!(
        LOOP[cfg..call].lines().count() <= 4,
        "o `cfg` tem de governar ESTA chamada, não uma linha distante dela"
    );
}

/// **Com o barro fora da tela, a cena devolve o gesto.**
///
/// ⚠️ É a metade que decide se a feature é ALCANÇÁVEL: no modo LUZ a malha não é desenhada, então
/// um clique que a cena engolisse orbitaria um modelo invisível e o artista não conseguiria pintar
/// — a doação funcionando perfeitamente, e inútil. A pergunta tem de ser feita nas DUAS portas de
/// entrada que a cena tem (o botão e a roda); o `Move` herda por não ter arrasto aberto.
#[test]
fn a_hidden_clay_hands_the_pointer_back() {
    let both = scene_and_gesture();
    for door in ["fn sculpt3d_pointer_down", "fn sculpt3d_wheel"] {
        let at = both
            .find(door)
            .unwrap_or_else(|| panic!("`{door}` sumiu do módulo 3D — atualize este gate"));
        // A porta é curta; a recusa tem de estar dentro dela, antes de qualquer gesto ser tomado.
        let body = &both[at..];
        let end = body.find("\n    /// ").unwrap_or(body.len());
        assert!(
            body[..end].contains("shows_clay()"),
            "`{door}` tem de perguntar `shows_clay()` antes de tomar o gesto"
        );
    }
    // E o passe de cor idem: desenhar o barro no modo LUZ esconderia a tinta que a doação acende.
    //
    // ⚠️ Sobre a FAMÍLIA e não sobre o `SCENE`: o passe de cor mudou de arquivo
    // num corte de LOC, e esta linha era o endereço que expirou.
    assert!(
        both.contains("if !self.shows_clay() {"),
        "o passe de cor tem de recusar fora do barro"
    );
}

/// **Desligar APAGA a doação instalada.**
///
/// Um interruptor que só sabe ligar deixa o plano de pé no tool: o artista aperta `D`, lê
/// *"DESLIGADA"*, e a tinta continua acesa exatamente como antes. É o A/B — o controle da wave —
/// falhando em silêncio, e com ele a única forma de julgar o que a forma acrescenta.
#[test]
fn switching_off_clears_the_installed_plane() {
    assert!(
        DONATION.contains("self.donated_form.news = Some(None);"),
        "a posição desligada tem de EMITIR o apagamento, não apenas parar de doar"
    );
}

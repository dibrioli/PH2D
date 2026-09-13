//! **A ROTA A, verificável** (ADR-0150 W8.7 · `docs/3D/02.2`) — um objeto assado re-acende **sem o
//! módulo 3D no build**, e reabre pela **mesma porta** que a lâmpada usa.
//!
//! O `02.2` promete que o G-buffer é gerado uma vez, vira canal do sprite, e **a malha some do
//! build**. Essa é uma frase sobre COMPILAÇÃO, e nenhum gate de unidade a alcança: os testes de
//! `baked_form` rodam com a feature ligada, então eles provariam a mesma coisa se o módulo inteiro
//! estivesse atrás dela. O que separa a promessa da prosa é **onde o código mora** — e é isso que
//! estes gates leem.
//!
//! É o mesmo padrão (e o mesmo motivo) do `the_sculpt_document_is_wired`: quando o fato é a forma do
//! código do produto, o gate lê o código do produto.

use crate::sculpt_source;
use sculpt_source::{function_body, project_family_fn, source};

/// **A ACENDIDA NÃO ESTÁ ATRÁS DA FEATURE.**
///
/// ⚠️ **É o gate da wave.** Enquanto a re-acendida morasse dentro de `#[cfg(feature = "sculpt3d")]`,
/// reabrir um projeto num binário sem escultura devolveria um objeto que **ninguém consegue
/// iluminar** — e o modo de falha é o pior possível: nenhum erro, nenhum aviso, e todos os gates de
/// unidade verdes, porque eles rodam com a feature ligada.
///
/// A asserção é sobre a linha do `render_loop` que chama o passe: ela tem de estar **fora** de todo
/// bloco `cfg` de escultura. Mutação: mover a chamada para dentro do `if let Some(scene)` gateado
/// (o lugar mais natural do mundo, e onde ela morava antes desta wave) ⇒ RED.
///
/// ⚠️⚠️ **O SUJEITO mudou-se para uma FASE em 2026-09-12** (OBRA 2 da `line/render-loop`): a
/// re-acendida é a `fase_relight_baked_forms`, e a pergunta passou a ter **QUATRO portas** — a
/// chamada da fase no quadro, o `mod` da fase, a `fn` da fase e a chamada dentro dela. ⛔ O texto
/// emendado do quadro (`frame_text`) NÃO serve de régua aqui: a emenda carrega o CORPO de cada fase e
/// larga os atributos da chamada, do `mod` e da `fn` — exactamente três das quatro portas. E a 4.ª
/// ganhou a metade que a régua antiga não via: um `#[cfg]` NU à frente do statement (sem bloco) não
/// abre chaves, e a contagem de profundidade lia-o como livre.
#[test]
fn the_relight_is_not_behind_the_sculpt_feature() {
    const FASE: &str = "fase_relight_baked_forms";
    let call = "ph2d_form_donation::baked_form::relight_stale(";

    // (1) A chamada da fase no quadro.
    let quadro = function_body(&source("render_loop/mod.rs"), "run_render_frame");
    let chamada = format!("self.{FASE}(");
    assert_eq!(
        quadro.matches(&chamada).count(),
        1,
        "o quadro tem de chamar a fase da re-acendida UMA vez — sem ela um objeto reaberto nunca acende"
    );
    let em = quadro.find(&chamada).expect("contada acima");
    not_gated_right_before(&quadro, em, "a chamada da fase no `run_render_frame`");

    // (2) O `mod` da fase.
    let modrs = source("render_loop/mod.rs");
    let decl = modrs
        .find(&format!("mod {FASE};"))
        .expect("o `mod` da fase da re-acendida existe");
    not_gated_right_before(&modrs, decl, "o `mod` da fase");

    // (3) Nada no ficheiro da fase, antes da `fn`, a gateia (a `fn`, o `impl`, um `#![cfg]`).
    let fase = source(&format!("render_loop/{FASE}.rs"));
    let fn_at = fase
        .find(&format!("fn {FASE}("))
        .expect("a `fn` da fase existe");
    assert!(
        !fase[..fn_at].contains("cfg("),
        "a `fn {FASE}` (ou o `impl`/o ficheiro dela) esta' atras de um `cfg` -- um objeto assado \
         deixaria de acender no build sem o modulo 3D, em silencio, com toda a suite verde"
    );

    // (4) A chamada à re-acendida dentro da fase: nem com um atributo à frente…
    let body = function_body(&fase, FASE);
    let at = body
        .find(call)
        .expect("a fase precisa CHAMAR a re-acendida — sem ela um objeto reaberto nunca acende");
    not_gated_right_before(&body, at, "a chamada a' re-acendida");
    // …nem DENTRO de um bloco gateado. O último `#[cfg(feature = "sculpt3d")]` ANTES da chamada abre
    // um bloco; se a chamada estiver dentro dele, ela cai junto com a feature. A pergunta é
    // estrutural, então contamos chaves.
    let before = &body[..at];
    if let Some(cfg_at) = before.rfind("#[cfg(feature = \"sculpt3d\")]") {
        let depth: i32 = before[cfg_at..]
            .chars()
            .map(|c| match c {
                '{' => 1,
                '}' => -1,
                _ => 0,
            })
            .sum();
        assert!(
            depth <= 0,
            "a re-acendida esta' DENTRO de um bloco `cfg(feature = \"sculpt3d\")` (profundidade \
             {depth}) -- um objeto assado deixaria de acender no build sem o modulo 3D, em silencio, \
             com toda a suite verde"
        );
    }
}

/// Entre o fim do statement ou item anterior (`;`, `{`, `}`) e `at` não há `cfg(`: um atributo gateia
/// o que vem logo a seguir a ele. `src` já vem sem comentários (o [`source`] tira-os).
fn not_gated_right_before(src: &str, at: usize, o_que: &str) {
    let inicio = src[..at].rfind([';', '{', '}']).map_or(0, |p| p + 1);
    let janela = &src[inicio..at];
    assert!(
        !janela.contains("cfg("),
        "{o_que} esta' atras de um `cfg` (`{}`) -- um objeto assado deixaria de acender no build sem \
         o modulo 3D, em silencio, com toda a suite verde",
        janela.trim()
    );
}

/// **A CRATE que carrega os canais também não está atrás da feature.**
///
/// ⚠️ Sem esta metade, a chamada do gate acima nem compilaria — mas o gate diria *«não achei»* em
/// vez de dizer o que está errado. Mais importante: ela pega a versão que passa pelo primeiro gate
/// e mesmo assim quebra a promessa.
///
/// ⚠️⚠️ **O SUJEITO mudou em 2026-09-11 (W2/L3-B), e a propriedade ficou MAIS FORTE.** Isto era
/// `mod baked_form;` no `main.rs`, e o gate subia linha a linha à procura de um `#[cfg]` contíguo.
/// Hoje o `baked_form` é a [`ph2d_form_donation`], uma crate-folha — e uma **dependência sem
/// `optional` não tem como ficar atrás de feature nenhuma**: não há atributo para pôr. ⇒ a
/// pergunta passa a ser sobre o manifesto, e a resposta é binária em vez de posicional.
///
/// ⛔ **E a razão de a crate existir é esta promessa**: o `RigStamp` é campo de uma struct da
/// família, e quando ela saiu para uma crate própria, *«não ter `cfg`»* deixou de bastar para o
/// manter alcançável de um binário sem escultura.
///
/// **Mutação que deve sangrar:** pôr `optional = true` na linha do manifesto.
#[test]
fn the_crate_that_holds_the_channels_is_unconditional() {
    let manifesto = std::fs::read_to_string(format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR")))
        .expect("o `Cargo.toml` da shell existe");

    let linha = manifesto
        .lines()
        .find(|l| l.trim_start().starts_with("ph2d-form-donation ="))
        .expect(
            "controlo positivo: a shell deixou de declarar a `ph2d-form-donation` — ou ela mudou              de nome, e este gate passaria a medir o vazio",
        );
    assert!(
        !linha.contains("optional"),
        "a `ph2d-form-donation` ficou `optional` (`{linha}`) — os canais assados sairiam do build          junto com a escultura, e é exactamente isso que a rota A promete que NÃO acontece: um          projeto reaberto num binário sem o módulo 3D devolveria um objeto que ninguém consegue          iluminar"
    );

    // E o par persistência ↔ documento pelo mesmo motivo: um deles sob `cfg` deixa o `ProjectFile`
    // com uma forma DIFERENTE por build, que é a maneira mais rápida de tornar um arquivo ilegível.
    assert!(
        source("project.rs").contains("baked_forms: Vec<crate::project_baked_form::"),
        "o `ProjectFile` precisa carregar os canais"
    );
    // ⚠️ O NOME mudou em 2026-08-23 e a claim não: o `project_save()` — que resolvia o destino
    // aqui dentro — deu lugar ao `project_save_to(path)`, com quem decide *onde* a viver no
    // `project_io`. O que este gate afirma é que o SAVE não é `cfg`-gated.
    assert!(
        !project_family_fn("project_save_to").contains("cfg(feature"),
        "o save nao pode gravar formas de arquivo diferentes por build"
    );
}

/// **REABRIR USA A MESMA PORTA DE LUZ QUE A LÂMPADA.**
///
/// ⚠️ O modo de falha de uma segunda porta é o mais cruel do repo: o objeto fica **certo enquanto o
/// app está aberto** e diferente na próxima vez que alguém mexe na lâmpada — a arte SALTA, e o
/// artista não tem como ligar o salto ao arquivo que ele abriu há dez minutos. É o defeito que o
/// ADR-0128 pagou cinco vezes.
///
/// A asserção tem duas metades porque há duas maneiras de errar: o load **acender por conta
/// própria**, e o load **não deixar nada para o passe fazer**.
#[test]
fn reopening_leaves_the_lighting_to_the_one_door() {
    let body = function_body(&source("project_baked_form.rs"), "restore_baked_forms");
    assert!(
        !body.contains("baked_form::light(") && !body.contains("ImpastoLightPass"),
        "o restore esta' ACENDENDO -- e' a segunda porta, e a arte saltaria ao reabrir o arquivo"
    );
    assert!(
        body.contains("lit_with: None"),
        "o restore precisa entregar o objeto NAO-ACESO, senao o passe de re-acendida nao tem o que \
         fazer e o sprite fica com a textura vazia que o load acabou de criar"
    );
}

/// **O RIG VIAJA no documento.**
///
/// ⚠️ Sem ele o load acenderia com o rig DEFAULT, e a arte mudaria de luz ao ser reaberta — em
/// silêncio. É a única coisa no documento que não é pixel, e é o que separa *reabrir o trabalho* de
/// *reabrir uma aproximação dele*.
///
/// ⚠️ E a metade que quase não foi escrita: o restore tem de **LER** o rig do documento. Guardá-lo e
/// depois semear o objeto com `LightRig::default()` deixaria o campo no arquivo, o gate de
/// serialização verde, e a arte errada.
#[test]
fn the_document_carries_the_rig_it_was_baked_with() {
    let src = source("project_baked_form.rs");
    assert!(
        src.contains("pub(crate) rig: LightRig,"),
        "o documento precisa carregar o rig autorado"
    );
    let restore = function_body(&src, "restore_baked_forms");
    assert!(
        restore.contains("rig: doc.rig,"),
        "o restore precisa LER o rig do documento -- semear o default deixaria o campo no arquivo, \
         o round-trip verde, e a arte com outra luz"
    );
    let collect = function_body(&src, "collect_baked_forms");
    assert!(
        collect.contains("rig: bake.rig,"),
        "o save precisa gravar o rig do OBJETO, nao um rig global"
    );
}

/// **O LOAD ESQUECE os objetos do documento anterior.**
///
/// ⚠️ O mapa é chaveado por bits de entidade e o `apply_project` despawna tudo: uma entrada que
/// sobrevivesse descreveria um objeto de outro projeto, e o passe de re-acendida ficaria acendendo,
/// **todo frame e para sempre**, um slot de textura que ninguém mostra. É a mesma lei que o load já
/// aplica ao relógio, à fila de undo, à timeline e aos pins do autokey.
#[test]
fn loading_forgets_the_baked_objects_of_the_previous_document() {
    let body = function_body(&source("project_forget.rs"), "forget_live_producers");
    assert!(
        body.contains("baked_forms.clear()"),
        "o load precisa esquecer os objetos assados do documento anterior"
    );
    // E a ORDEM: esquecer depois de repovoar apagaria o que o arquivo acabou de trazer.
    // ⚠️ **Em TRÊS metades** (`line/loc-caps`, 2026-09-13): o `project_load_from` entrega as duas
    // fases a irmãos (`project_forget_previous` · `project_install_accepted`), e a ordem é a das
    // CHAMADAS no pai. ⛔ Concatenar os corpos seria fraude — tudo o que está no segundo irmão
    // viria depois do primeiro, por construção.
    let load = project_family_fn("project_load_from");
    let forget = load
        .find("self.project_forget_previous(")
        .expect("o load precisa esquecer");
    let restore = load
        .find("self.project_install_accepted(")
        .expect("o load precisa devolver os canais");
    assert!(
        forget < restore,
        "o `forget` roda DEPOIS do `restore` -- ele apagaria exatamente o que o arquivo trouxe"
    );
    assert!(
        project_family_fn("project_forget_previous").contains("forget_live_producers()"),
        "o irmao que esquece deixou de chamar o `forget_live_producers`"
    );
    assert!(
        project_family_fn("project_install_accepted").contains("restore_baked_forms("),
        "o irmao que instala deixou de devolver os canais assados"
    );
}

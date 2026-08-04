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

mod sculpt_source;
use sculpt_source::{function_body, source};

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
#[test]
fn the_relight_is_not_behind_the_sculpt_feature() {
    let body = function_body(&source("render_loop/mod.rs"), "run_render_frame");
    let call = "crate::baked_form::relight_stale(";
    let at = body
        .find(call)
        .expect("o frame precisa CHAMAR a re-acendida — sem ela um objeto reaberto nunca acende");

    // O último `#[cfg(feature = "sculpt3d")]` ANTES da chamada abre um bloco; se a chamada estiver
    // dentro dele, ela cai junto com a feature. A pergunta é estrutural, então contamos chaves.
    let before = &body[..at];
    let Some(cfg_at) = before.rfind("#[cfg(feature = \"sculpt3d\")]") else {
        return; // nenhum `cfg` de escultura antes dela: a chamada é livre por construção
    };
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

/// **O MÓDULO que carrega os canais também não está atrás da feature.**
///
/// ⚠️ Sem esta metade, a chamada do gate acima nem compilaria — mas o gate diria *"não achei"* em
/// vez de dizer o que está errado. Mais importante: ela pega a versão que passa pelo primeiro gate e
/// mesmo assim quebra a promessa, que é declarar `mod baked_form` sob `cfg`.
#[test]
fn the_module_that_holds_the_channels_is_unconditional() {
    let src = source("main.rs");
    let at = src
        .find("mod baked_form;")
        .expect("o shell precisa declarar `mod baked_form`");
    let before = &src[..at];
    let line_start = before.rfind('\n').map_or(0, |i| i + 1);
    // A declaração é precedida por doc-comments (que o `source` já removeu) ou por nada; um `cfg`
    // colado nela viveria na mesma linha ou na anterior.
    let tail = src[line_start.saturating_sub(80)..at].to_string();
    assert!(
        !tail.contains("cfg(feature"),
        "`mod baked_form` esta' sob um `cfg` -- os canais assados sairiam do build junto com a \
         escultura, e e' exatamente isso que a rota A promete que NAO acontece"
    );

    // E o par persistência ↔ documento pelo mesmo motivo: um deles sob `cfg` deixa o `ProjectFile`
    // com uma forma DIFERENTE por build, que é a maneira mais rápida de tornar um arquivo ilegível.
    assert!(
        source("project.rs").contains("baked_forms: Vec<crate::project_baked_form::"),
        "o `ProjectFile` precisa carregar os canais"
    );
    assert!(
        !function_body(&source("project.rs"), "project_save").contains("cfg(feature"),
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
    let load = function_body(&source("project_load.rs"), "project_load_from");
    let forget = load
        .find("forget_live_producers()")
        .expect("o load precisa esquecer");
    let restore = load
        .find("restore_baked_forms(")
        .expect("o load precisa devolver os canais");
    assert!(
        forget < restore,
        "o `forget` roda DEPOIS do `restore` -- ele apagaria exatamente o que o arquivo trouxe"
    );
}

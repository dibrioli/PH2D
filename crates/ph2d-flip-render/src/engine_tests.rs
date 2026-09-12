//! O gate da política do interruptor — desceu com ela (auditoria A1).

use super::*;

/// ⭐ **O PERCURSO É O DEFAULT** (doc 12 §22) — o gate do padrão-ouro, e a linha que carrega o peso
/// é a primeira: **ambiente AUSENTE dá o percurso**.
///
/// ⚠️ Ele afirma a política sobre a função PURA (`walk_from_env`) e não sobre o `new_engine_armed`,
/// que é um `OnceLock` sobre uma variável de PROCESSO — dentro de um binário de teste ele responde
/// uma vez e para sempre, então um gate escrito sobre ele afirmaria o que a ORDEM dos testes
/// decidiu. Um default que ninguém pode afirmar é um default que a próxima edição inverte em
/// silêncio — e este já foi invertido uma vez (nasceu opt-in, em `aa14e9366`).
#[test]
fn the_walk_is_the_default_and_only_an_explicit_zero_escapes_to_the_raster() {
    assert!(
        walk_from_env(None),
        "ambiente ausente TEM de dar o percurso -- e' o default do padrao-ouro"
    );
    for off in ["0", "false", "off"] {
        assert!(
            !walk_from_env(Some(off)),
            "`PH2D_FLIP_NEW_ENGINE={off}` e a ESCAPE: tem de voltar ao rasterizador"
        );
    }
    for on in ["1", "true", "on"] {
        assert!(
            walk_from_env(Some(on)),
            "`PH2D_FLIP_NEW_ENGINE={on}` segue armando o percurso (o A/B dos smokes nao muda)"
        );
    }
    // Um erro de digitacao na escape falha **para o default**, nunca para um terceiro
    // comportamento -- e e' a unica classe de entrada que um gate de tabela costuma esquecer.
    for outro in ["", "flase", "no", "2", "OFF"] {
        assert!(
            walk_from_env(Some(outro)),
            "`{outro}` nao e' a escape reconhecida => cai no default, nao num 3o caminho"
        );
    }
}

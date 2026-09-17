//! ⭐⭐⭐ **O QUE O DEDO APANHA PRIMEIRO PINTA-SE POR ÚLTIMO** — a ordem dos passes do overlay do
//! esqueleto, dita como gate.
//!
//! ⛔⛔ **Report do dono (2026-09-16): *«os handles não estão por cima (z-index). Handles com
//! Z-index maior que ossos»*.** As duas alças de curvatura pintavam-se ANTES do `draw_bones`, por
//! uma razão de ACABAMENTO escrita no sítio (*«o osso por cima deixa a bolinha da junta inteira»*)
//! — e o `bone_pick::hover` faz o contrário: as alças do osso em foco competem por proximidade e
//! ganham ao CORPO, e a de curvatura chega a ignorar a distância ao osso (senão seria inalcançável
//! no ponto neutro, onde ela está **em cima do eixo**). *O artista agarrava o que não via.*
//!
//! # ⚠️ Porque este gate é TEXTUAL, e o que ele mede de facto
//!
//! A ordem é uma sequência de chamadas numa fase do quadro, e o alvo delas é uma `VectorScene` que
//! não se deixa interrogar (é uma cena do Vello, não uma lista de comandos que se conta). ⇒ o
//! oráculo possível é a POSIÇÃO de cada chamada no ficheiro da fase — a mesma forma que esta casa
//! já usa para as leis de ordem do despacho.
//!
//! ⛔ **E ele tem o controlo positivo que um censo textual exige:** as cinco chamadas têm de ser
//! achadas, cada uma exactamente uma vez. Sem isso, renomear uma delas deixaria o gate verde a
//! medir uma sequência de quatro — a armadilha do censo que varre zero e se lê como aprovado.

use std::path::PathBuf;

/// A fase do quadro que pinta o rig. ⚠️ **O caminho é o sujeito do gate**: se ela mudar de sítio,
/// isto reprova a dizer que não a achou, que é a pergunta certa a fazer a quem a moveu.
fn fase() -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/render_loop/fase_vector_bone_overlay.rs");
    std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("ler {}: {e}", p.display()))
}

/// Onde a chamada a `nome` começa — e ela tem de existir **uma vez só**.
fn posicao(texto: &str, nome: &str) -> usize {
    let agulha = format!("ph2d_skeleton_render::{nome}(");
    let n = texto.matches(agulha.as_str()).count();
    assert_eq!(
        n, 1,
        "esperava UMA chamada a `{nome}` na fase do overlay e achei {n} — o gate mediria outra \
         sequência (ou nenhuma) sem dar por isso"
    );
    texto.find(agulha.as_str()).expect("acabou de ser contada")
}

/// ⭐⭐⭐ **AS ALÇAS DE CURVATURA SÃO O ÚLTIMO PASSE DO RIG** — depois dos ossos e depois das
/// âncoras, que é a ordem inversa da do `bone_pick::hover`.
///
/// ⚠️ **Depois das ÂNCORAS também**, e não só dos ossos: no pick, o bloco do osso em foco corre
/// **antes** do teste da âncora, logo uma alça de curvatura ganha ao losango quando os dois caem
/// debaixo do dedo. Pintá-la por baixo dele devolveria o mesmo defeito noutro par.
#[test]
fn the_bend_handles_are_the_last_pass_of_the_rig() {
    let t = fase();
    // O fundo: a mancha da influência e o arco do limite. Eles são BACKDROP — as alças deles vivem
    // fora do eixo do osso, e é por isso que este report não é sobre eles.
    let influencia = posicao(&t, "draw_influence");
    let limite = posicao(&t, "draw_limit");
    let ossos = posicao(&t, "draw_bones");
    let ancoras = posicao(&t, "draw_goals");
    let alcas = posicao(&t, "draw_bend");
    assert!(
        influencia < ossos && limite < ossos,
        "a mancha da influencia e o arco do limite sao FUNDO: o rig pinta-se por cima deles"
    );
    assert!(
        ossos < alcas,
        "as alcas de curvatura voltaram a pintar-se ANTES dos ossos — e' o report de 2026-09-16 \
         («os handles nao estao por cima»), com o dedo a apanhar o que o olho nao ve^"
    );
    assert!(
        ancoras < alcas,
        "as alcas de curvatura tem de vir DEPOIS das ancoras de IK — no `bone_pick::hover` elas \
         ganham ao losango, e o desenho tem de dizer o mesmo"
    );
}

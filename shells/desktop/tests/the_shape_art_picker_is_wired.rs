//! **O PICKER DA ARTE-FORMA está fiado, do botão até ao vínculo** (plano 33, W7).
//!
//! ⚠️ **A shell não é alcançável de um teste de unidade** — o `App` segura uma surface de janela
//! real. É a mesma razão pela qual o undo do filtro do sculpt3d, o desenho do offset vivo e o pick
//! do mapa desenhado têm todos um gate que lê o FONTE. Aqui o risco é concreto e já mordeu duas
//! vezes nesta casa: um controlo **pintado e morto sob o ponteiro** dá exactamente o mesmo report
//! que um controlo que nunca foi pintado.
//!
//! ⚠️ **Este ficheiro chamava-se `the_pattern_handles_are_wired`** e guardava também os quatro
//! sítios das três alças de canvas do padrão (W6). **Elas foram RETIRADAS por decisão do Enio**
//! (2026-08-27: *"não ficou legal. vamos retirar e deixar os ajustes apenas no painel"*), e a
//! posição do padrão passou a ser as fileiras **Shift X/Y** da secção *Pattern*. O motivo e o que
//! ficou no lugar estão no [plano 33](../../../docs/Vector%20Module/33_plano_texture_pattern.md) §6.

use std::fs;
use std::path::Path;

fn src(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// ⚠️ **Comentários FORA**, e não é higiene: a prosa que explica a lei contém, por construção,
/// exactamente as agulhas que o gate procura — *um gate que lê o comentário sobre a lei em vez do
/// código que a obedece aprova quem a documenta*.
fn code(rel: &str) -> String {
    src(rel)
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐ **O PICKER DA ARTE-FORMA está fiado** (plano 33, W7) — o gesto de duas mãos do Figma.
///
/// ⚠️ **A fonte é CAPTURADA no arm**, e é essa a razão de existir do picker: o clique seguinte cai
/// noutra forma, e ela passaria a ser a selecionada. Ler a seleção na hora de resolver apontaria o
/// padrão para a forma errada — o *"escolhendo a si mesmo"* que o doc do `vec_pick` nomeia.
#[test]
fn the_shape_art_picker_is_wired_from_the_button_to_the_link() {
    let render = code("render_loop/mod.rs");
    // ⚠️ A agulha passou a ser o KNOB (plano 35, wave F): os ids da secção são derivados por
    // `(tinta, controlo)`, então já não há uma constante com este nome. *Um gate que fixa o nome de
    // uma constante reprova a família que a substitui sem mudar a lei que ele defende.*
    assert!(
        render.contains("K::PickShape => pending_texpat_pick"),
        "o botao Use Shape nao e' reconhecido no despacho"
    );
    // ⚠️ **A agulha é o PREFIXO, e a 1.ª redacção fixava `TexturePatternArt(host)` inteiro** — a
    // wave D do plano 35 acrescentou o SLOT à captura e este gate reprovou produto correcto. *Um
    // gate que fixa a aridade de um construtor reprova a extensão que não muda a lei que ele
    // defende.*
    assert!(
        render.contains("PathPick::TexturePatternArt(host"),
        "o pick da arte-forma nao e' ARMADO com a fonte capturada"
    );
    // ⭐ **E o SLOT obedece à MESMA lei** (plano 35, wave D): *qual das duas tintas eu estava a
    // editar* é tão parte da captura quanto *qual forma*. Lê-lo no clique leria uma preferência de
    // sessão que pode ter mudado no meio.
    assert!(
        render.contains("PathPick::TexturePatternArt(host, slot)"),
        "o slot nao e' capturado no arm - o picker escreveria na tinta que estiver acesa AGORA"
    );
    let d = code("input_dispatch.rs");
    assert!(
        d.contains("PathPick::TexturePatternArt(host, slot) =>"),
        "o clique no canvas nao RESOLVE o pick da arte-forma com o slot capturado"
    );
    assert!(
        d.contains("texture_pattern_edit::set_source("),
        "o pick resolvido nao escreve a fonte pela porta por-ID"
    );
    // ⚠️ E a porta por-ID tem de existir SEPARADA da que lê a seleção — é essa separação que impede
    // o padrão de apontar para a forma que o clique acabou de seleccionar.
    assert!(
        code("texture_pattern_edit.rs").contains("pub(crate) fn set_source("),
        "a porta por-ID sumiu; o picker voltaria a ler a selecao"
    );
    // ⚠️⚠️ **E o ARGUMENTO é o `host` CAPTURADO, nunca o `guide` clicado.**
    //
    // Esta linha existe porque uma prova de mutação não a alcançou: trocar `host` por `guide`
    // compila, inverte o gesto inteiro (o padrão passa a viver na forma que se clicou, e a arte a
    // ser a que estava selecionada) — e **nenhum** gate de comportamento a via, porque a shell não
    // é alcançável de um teste. *Uma afirmação que só um gate de fonte alcança precisa desse gate.*
    // ⛔⛔ **A JANELA É O BRAÇO, e era uma CONSTANTE DE 380 BYTES.**
    //
    // Ela media *distância no fonte* e não a lei que defende: quando o braço ganhou a medição da
    // arte (2026-08-30), o `set_source(` passou de `+90` para `+1410` bytes e este gate ficou
    // **vermelho sobre produto correcto**, a acusar *"o picker escreve numa forma que nao e' a
    // CAPTURADA"*. ⇒ o corpo passa a ser delimitado pelo **braço seguinte**, e cresce com ele.
    //
    // ⚠️⚠️ **E ele esteve vermelho sem ser visto**, porque o filtro por NOME do inner loop
    // (`cargo test -p ph2d-host-desktop --bins <filtro>`) **não alcança `shells/desktop/tests/`** —
    // é a família que o `CLAUDE.md` §5.1 já registou, e mordeu outra vez.
    let needle = "PathPick::TexturePatternArt(host, slot) =>";
    let arm = d.find(needle).expect("o braco existe");
    let fim = d[arm + needle.len()..]
        .find("PathPick::")
        .map_or(d.len(), |o| arm + needle.len() + o);
    let corpo = &d[arm..fim];
    assert!(
        corpo.contains("set_source(") && corpo.contains("host,"),
        "o picker escreve numa forma que nao e' a CAPTURADA - o gesto de duas maos inverte-se"
    );
    // ⚠️ E na TINTA capturada, pela mesma razão: `self.texpat_target` lido aqui seria a preferência
    // de agora, e não a de quando o gesto começou.
    assert!(
        corpo.contains("slot,"),
        "o picker escreve numa tinta que nao e' a CAPTURADA no arm"
    );
    // ⭐⭐⭐ **E A ARTE É MEDIDA COMO UM OBJECTO, COM A POSE** (report do Enio, 2026-08-30: um grupo
    // alto nascia achatado).
    //
    // ⚠️ **Estas linhas existem porque três mutações SOBREVIVIAM** à auditoria desta wave: as
    // portas puras (`art_dims`, `set_source`) têm gates próprios, mas eles passam os **seus**
    // fechos — um `object_of` escrito à mão e um `VecXforms::default()`. ⇒ trocar aqui a expansão
    // por `&|id| vec![id]` faz o grupo colapsar num membro e **o report volta inteiro**, com os
    // dois gates de unidade verdes. *Um gate de porta pura não mede o fio que a alimenta.*
    //
    // Os números da auditoria, no mesmo grupo `1 x 3`: sem a expansão a razão vai de `3,000` para
    // `1,000` (o quadrado do report); sem a pose, um grupo rodado 45° mede `1,000` e um escalado
    // `3x` em X mede `1,000`.
    for (agulha, o_que) in [
        (
            "crate::texture_pattern_pick::art_dims(",
            "a arte nao e' MEDIDA - o padrao herda um quadrado",
        ),
        (
            "crate::vec_entities::object_selection_for(",
            "a arte nao e' expandida como OBJECTO - um grupo colapsa no caminho clicado",
        ),
        (
            "crate::vec_transform::build(",
            "a arte e' medida SEM a pose - um grupo rodado ou escalado mede outra coisa",
        ),
        (
            "crate::texture_pattern_pick::default_placement(",
            "o tamanho nao sai da porta unica de colocacao",
        ),
    ] {
        assert!(corpo.contains(agulha), "{o_que} (`{agulha}` saiu do braco)");
    }
}

/// ⛔⛔ **AS ALÇAS DE CANVAS DO PADRÃO NÃO VOLTAM SEM ORDEM** (Enio, 2026-08-27).
///
/// ⚠️ Este gate é o par executável da recusa escrita no plano 33 §6. Uma decisão de produto que
/// vive só num documento é uma decisão que a próxima janela reconstrói de boa-fé — *o §5 do
/// roteador já acumulou trabalho já pago por exactamente isto*.
///
/// A posição do padrão é hoje autorada pelas fileiras **Shift X/Y** do painel, que passam pela
/// mesma porta única (`PatternFill::set_shift_axis`) que a alça de mover usava.
#[test]
fn the_pattern_has_no_canvas_handles_anymore() {
    for (rel, agulha) in [
        ("input_dispatch.rs", "vec_pattern_hit"),
        ("input_dispatch.rs", "vec_pattern_drag"),
        ("render_loop/mod.rs", "draw_pattern_handles"),
        ("app_state.rs", "vec_pattern_selected"),
    ] {
        assert!(
            !src(rel).contains(agulha),
            "`{agulha}` voltou a `{rel}` - as alcas de canvas do padrao foram RETIRADAS por decisao \
             do Enio (plano 33 §6); os ajustes vivem no painel"
        );
    }
    // ⚠️ E o CONTROLO: a porta que ficou no lugar delas tem de existir, senão este gate ficaria
    // verde num produto que perdeu a posição do padrão em vez de a ter mudado de sítio.
    assert!(
        src("texture_pattern_edit.rs").contains("TexPatCmd::Shift"),
        "a posicao do padrao nao e' autoravel por lado nenhum"
    );
}

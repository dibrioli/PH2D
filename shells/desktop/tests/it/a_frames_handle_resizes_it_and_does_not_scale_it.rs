//! **A alça de uma moldura REDIMENSIONA-a; ela não a ESCALA** — arch-gate sobre a costura que
//! nenhum unit test alcança (corolário do W3; irmão de `dragging_inside_a_flow_reorders_it`).
//!
//! A LEI é gateada onde ela mora: os testes do `vec_frame_resize` dirigem uma `VecScene` REAL
//! headless e provam qual canto fica fixo, que a razão é absoluta, que a volta ao inicio é ao bit e que o
//! filho não é tocado. O que eles **não podem tocar** é o `advance_gizmo_drag`, que precisa de
//! `App` + `HeroScreen` + janela — e é lá que se decidem as três metades que faltam:
//!
//! 1. **O braço existe, e usa o instantâneo.** Sem ele a alça cai no caminho de sempre e escreve
//!    `Transform.scale`: a pose de um pai é herdada por todo descendente, então a moldura não muda
//!    de CAIXA — ela ESTICA os filhos. É o defeito que o Enio reportou.
//! 2. **Ele NÃO escreve `Transform`, e esta é a metade cara.** Escrever "por segurança" reintroduz
//!    exactamente o esticamento, com o resto do braço a parecer certo — e um gate que só afirmasse
//!    (1) ficaria VERDE sobre isso.
//! 3. **Só um SCALE muda de significado.** Um Translate sobre uma moldura continua a movê-la (a
//!    pose é o que uma posição É); engolir todo gesto deixaria a moldura impossível de arrastar.
//!
//! ⚠️ Nada aqui afirma distância em bytes nem vizinhança de linhas — a lição de
//! `the_dispatch_is_handed_the_live_geometry` (2026-07-23) é que um proxy posicional expira na wave
//! seguinte. O que se afirma é *que pergunta é feita* e *o que o braço faz com a resposta*.

use std::fs;

fn source() -> String {
    fs::read_to_string("src/input_dispatch/gizmo_drag.rs").expect("gizmo_drag.rs")
}

/// O corpo do braço da moldura: da porta até o `} else if` que abre o braço seguinte.
fn frame_arm(src: &str) -> String {
    let i = src.find("crate::vec_frame_resize::apply(").expect(
        "a alca NAO passa pela porta que redimensiona a moldura — ela escreve a pose, e a pose de \
         um pai e' herdada: os filhos esticam",
    );
    // Recua até o `} else if` que abre este braço, para o corpo incluir a condição.
    let head = src[..i]
        .rfind("} else if")
        .expect("o braco da moldura nao e' um `else if`");
    let after = &src[head..];
    let end = after[9..].find("} else if").map_or(after.len(), |e| e + 9);
    after[..end].to_string()
}

/// **O braço pergunta pelo instantâneo do gesto** — e não recomeça a medir a cada movimento.
///
/// ⚠️ Sem o instantâneo a única coisa que sobra para escalar é a geometria VIVA, e a razão do
/// gizmo é absoluta: ela comporia uma vez por evento de rato. O gate do `vec_frame_resize`
/// (`ten_moves_to_double_it_double_it_once`) mede a consequência; este afirma que o braço tem por
/// onde a obter.
#[test]
fn the_frame_arm_resizes_from_the_snapshot_it_took() {
    let arm = frame_arm(&source());
    assert!(
        arm.contains("self.frame_resize_start"),
        "o braco da moldura nao le o instantaneo do gesto — a razao do gizmo e' ABSOLUTA, entao \
         escalar a geometria viva comporia uma vez por evento de rato:\n{arm}"
    );
}

/// **E ele NÃO escreve a pose autorada.**
///
/// ⚠️ Esta é a mutação barata: acrescentar a escrita ao lado do redimensionamento. Ela devolve
/// exactamente o defeito reportado (os filhos esticam) enquanto todo o resto do braço continua a
/// parecer certo.
#[test]
fn the_frame_arm_never_writes_the_authored_transform() {
    let arm = frame_arm(&source());
    assert!(
        !arm.contains("get_mut::<Transform>"),
        "o braco da moldura escreve a pose AUTORADA — ela e' herdada por todo descendente, entao \
         os filhos voltam a esticar:\n{arm}"
    );
}

/// **Só um SCALE muda de significado — nos DOIS sítios.**
///
/// Sem esta guarda o braço engoliria o Translate, e a moldura ficaria impossível de arrastar:
/// mover é escrever a pose, e é exactamente isso que este braço se recusa a fazer.
///
/// ⚠️ **A guarda que de facto carrega o peso é a da INSTALAÇÃO**, e o gate afirma as duas por
/// isso: o instantâneo só nasce num arrasto de escala, então a do braço é a segunda camada e
/// mutá-la sozinha **não é observável hoje** (sem instantâneo, o braço não tinha como disparar de
/// qualquer maneira). Afirmar as duas é o que faz a mutação sangrar seja qual for a que alguém
/// remova — a alternativa era um gate que não pode falhar pelo motivo que alega.
#[test]
fn only_a_scale_becomes_a_resize() {
    let src = source();
    let arm = frame_arm(&src);
    let head = arm.split('{').next().unwrap_or_default();
    assert!(
        head.contains("is_scale"),
        "o braco da moldura engole TODO gesto — arrastar uma moldura deixaria de a mover:\n{head}"
    );
    let install = src
        .find("self.frame_resize_start = self.begin_frame_resize(")
        .expect("a instalacao do instantaneo");
    let cond = &src[..install];
    let cond = &cond[cond
        .rfind("if is_scale_drag")
        .unwrap_or(cond.len().saturating_sub(1))..];
    assert!(
        cond.starts_with("if is_scale_drag"),
        "o instantaneo e' tirado em TODO arrasto — um Translate sobre a moldura passaria a \
         redimensiona-la:\n{cond}"
    );
}

/// **O braço só usa um instantâneo que pertence a ESTE arrasto.**
///
/// ⚠️ A pergunta é ancorada **DENTRO do braço**, e a 1ª versão deste gate a fazia sobre a fonte
/// inteira: o `is_for` aparece em DOIS sítios (o que instala o instantâneo e este, que o consome),
/// então uma busca global fica verde com qualquer um dos dois removido. *Duas ocorrências do mesmo
/// literal em camadas diferentes precisam de uma âncora cada* — sem isto, aplicar a geometria da
/// moldura ANTERIOR sobre esta passa despercebido.
#[test]
fn the_frame_arm_only_uses_a_snapshot_that_belongs_to_this_drag() {
    let arm = frame_arm(&source());
    assert!(
        arm.contains("is_for(drag.entity_bits)"),
        "o braco consome o instantaneo sem conferir de quem ele e' — um gesto sobre outra moldura \
         repunha a geometria da anterior por cima dela:\n{arm}"
    );
}

/// ⚠️ **A guarda de instalação é DEFESA EM CAMADA e fica documentada em vez de gateada.**
///
/// O instantâneo é descartado em dois sítios — no topo do `advance_gizmo_drag` (quando não há
/// arrasto aberto) e no fim do gesto, em `input_dispatch` — e o segundo corre em **todo** soltar de
/// botão. Com ele de pé, um instantâneo obsoleto não pode existir, então trocar o `is_for` da
/// instalação por `is_none()` **não é observável** e nenhuma mutação sangra: medido, os quatro
/// gates deste ficheiro passam.
///
/// O que ela protege é a falha da camada de baixo: se o descarte do fim do gesto deixar de correr,
/// o `is_for` da instalação é o que ainda substitui o instantâneo em vez de deixar o braço recusar
/// — e a recusa faria a moldura **ESCALAR** (o defeito que esta wave conserta) em vez de
/// redimensionar. Isolá-la exigiria um `App` com janela; o precedente do repo é declarar o
/// mecanismo aqui em vez de shipar um gate que não pode falhar
/// (`feedback_layered_defenses_need_per_layer_gates`).
#[test]
fn the_install_guard_is_documented_defence_in_depth() {
    let src = source();
    assert!(
        src.contains("self.frame_resize_start = self.begin_frame_resize("),
        "a instalacao do instantaneo sumiu — sem ela o braco nunca tem o que consumir"
    );
}

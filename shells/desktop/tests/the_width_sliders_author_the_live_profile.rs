//! **Arch-gate da costura da LARGURA VIVA** (ADR-0145) — os quatro sliders AUTORAM, o frame
//! COZE, e o botão MATERIALIZA.
//!
//! ## O que este gate protege
//!
//! Três maneiras de partir a feature deixam **todos os unit tests verdes**, porque nenhum deles
//! alcança o corpo do `render_frame`:
//!
//! 1. **ninguém arma** — os sliders voltam a ser parâmetros de um comando, nada aparece até o
//!    clique, e os gates de `profile_live` seguem verdes (eles chamam `arm` à mão);
//! 2. **o mapa vivo não é fundido** no argumento do `dispatch` — a fita é cozida e nunca
//!    desenhada;
//! 3. **o botão pula o `materialise`** e cai no caminho numérico — o Apply assa os SLIDERS em vez
//!    do perfil da forma, e as duas coisas só divergem quando há mais de uma forma selecionada
//!    com perfis diferentes (o caso que ninguém testa à mão).
//!
//! ## As asserções afirmam uma RELAÇÃO, nunca uma distância
//!
//! Esta linha já teve dois arch-gates apodrecerem por medirem bytes no fonte (integração de
//! 2026-07-23): o `rustfmt` reflui, uma feature irmã entra no meio, e o gate fica **vermelho
//! sobre código correto**. Aqui as perguntas são posicionais (*isto corre antes daquilo*) e de
//! conteúdo (*este nome aparece nesta janela sintática*), que sobrevivem a reformatação.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// A posição (em bytes) da 1ª ocorrência de `needle`, ou pânico com a razão.
fn at(needle: &str) -> usize {
    SRC.find(needle).unwrap_or_else(|| {
        panic!(
            "`{needle}` sumiu do render_loop — se foi renomeado, atualize este gate (e confira \
             que a largura viva ainda chega à tela: `PH2D_BUILD_SMOKE=41`)"
        )
    })
}

/// **Arrastar um dos quatro sliders ARMA o perfil na seleção.** Sem esta costura os knobs
/// voltam a ser parâmetros de um comando: o artista arrasta, nada acontece, e só o clique
/// mostra o resultado — que é exatamente o produto que o ADR-0145 substituiu.
#[test]
fn dragging_a_width_slider_arms_the_live_profile() {
    let arm = at("crate::profile_live::arm(");
    // A janela é o bloco que contém a chamada: do início do bloco da largura viva até o `arm`.
    let block = at("// ── A LARGURA VIVA (ADR-0145)");
    assert!(
        block < arm,
        "o `profile_live::arm` não está no bloco da largura viva"
    );
    let window = &SRC[block..arm];
    for id in [
        "VECTOR_EXPAND_W_START",
        "VECTOR_EXPAND_W_MID",
        "VECTOR_EXPAND_W_END",
        "VECTOR_EXPAND_W_POS",
    ] {
        assert!(
            window.contains(id),
            "o armamento não olha o `{id}` — aquele slider ficaria vivo sob o mouse e MORTO no \
             documento, e nenhum unit test notaria"
        );
    }
    assert!(
        window.contains("active_id()"),
        "o armamento não pergunta qual widget está sob a mão — escrever o perfil a cada frame \
         reescreveria a forma com os valores do painel, mesmo sem o artista tocar em nada"
    );
    assert!(
        window.contains("preset_from_store"),
        "o armamento não lê os sliders pela porta única — uma segunda leitura divergiria dos \
         defaults que o painel publica"
    );
}

/// **A fita cozida é ENTREGUE ao desenho.** Cozer e não fundir no argumento do `dispatch`
/// compila, e a largura variável simplesmente não aparece.
#[test]
fn the_cooked_ribbon_reaches_the_dispatch() {
    let call = at("ph2d_vec_render::dispatch(");
    let bind = at("let mut vec_live = self.offset_live.live()");
    assert!(bind < call, "`vec_live` é ligado DEPOIS do `dispatch`");
    assert!(
        SRC[bind..call].contains("self.profile_live"),
        "`vec_live` não recolhe `self.profile_live` — a fita de largura variável é cozida todo \
         frame e jogada fora, e nenhum teste de unidade nota"
    );
}

/// **O cozimento roda DEPOIS do `sync` e ANTES do desenho** — a mesma lei do irmão do offset:
/// antes do `sync` a forma nova não tem entidade e o componente dela é invisível; depois do
/// desenho o frame pinta a resposta do frame anterior.
#[test]
fn the_profile_cook_runs_after_the_sync_and_before_the_draw() {
    let sync = at("crate::vec_entities::sync(");
    let cook = at("self.profile_live\n                .recook(");
    let draw = at("ph2d_vec_render::dispatch(");
    assert!(
        sync < cook,
        "o `profile_live::recook` corre ANTES do `sync`"
    );
    assert!(
        cook < draw,
        "o `profile_live::recook` corre DEPOIS do desenho"
    );
}

/// **O botão consulta o perfil VIVO antes do caminho numérico.** Se o `materialise` corresse
/// depois (ou não corresse), o Apply assaria os SLIDERS — e com duas formas de perfis diferentes
/// selecionadas as duas sairiam iguais, com o painel dizendo o contrário do que está na tela.
#[test]
fn the_apply_button_bakes_the_live_profile_not_the_sliders() {
    let mat = at("crate::profile_live::materialise(");
    let numeric = at("crate::vec_expand::apply_vec_expand(");
    assert!(
        mat < numeric,
        "o `profile_live::materialise` corre DEPOIS do caminho numérico — o Apply assaria os \
         sliders por cima do perfil que a forma carrega"
    );
    // E a consulta é gateada no comando certo: materializar num clique de Offset assaria a
    // forma errada. A pergunta é ao `if` que ABRE o bloco da chamada — uma relação, não uma
    // distância (o cabeçalho deste arquivo diz por quê).
    let head = &SRC[..mat];
    let guard = head
        .rfind("if matches!(cmd,")
        .expect("o `materialise` não está dentro de um `if matches!(cmd, …)`");
    assert!(
        SRC[guard..mat].contains("Expand::PowerStroke"),
        "o `materialise` do perfil está sob um guard que NÃO é o Power Stroke — clicar Apply \
         Offset assaria a fita da forma errada"
    );
}

/// **O preview NÃO pode escrever no documento**, e quem o garante é o COMPILADOR: o `recook`
/// recebe `&VecScene`. Irmão exato do gate do offset — a promessa *"a curva autorada não muda
/// até o Apply"* é do tipo, não da disciplina.
#[test]
fn the_profile_cook_takes_the_scene_by_shared_reference() {
    const COOK: &str = include_str!("../src/profile_live.rs");
    let sig = COOK
        .find("pub(crate) fn recook(")
        .expect("o `recook` sumiu do profile_live");
    let tail = &COOK[sig..(sig + 260).min(COOK.len())];
    assert!(
        tail.contains("scene: &VecScene"),
        "o `recook` deixou de receber a cena por referência COMPARTILHADA — a partir daqui o \
         preview pode escrever no documento:\n{tail}"
    );
}

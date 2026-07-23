//! **Arch-gate da costura do Offset VIVO** — o frame COZE a geometria derivada e a ENTREGA ao
//! desenho.
//!
//! ## O que este gate protege
//!
//! O offset ao vivo é geometria **derivada**: o documento guarda a curva autorada (Enio,
//! 2026-07-21: *"os botões Miter, Round e Bevel são previsualizações em tempo real"*), e o que
//! se vê sai de `offset_live::recook` e entra no `dispatch` pelo argumento `live`.
//!
//! Duas maneiras de partir isso deixam **todos os unit tests verdes**:
//!
//! 1. **passar um mapa vazio** ao `dispatch` (`&LiveGeometry::new()`) — compila, e o offset
//!    simplesmente NÃO APARECE. Nenhum gate de unidade toca a `render_loop`.
//! 2. **cozer antes do `sync`** — uma forma recém-criada ainda não tem entidade, o componente
//!    dela não é encontrado, e o offset dela some por um frame (ou para sempre, se ela nunca
//!    mais mudar).
//!
//! ## Por que um gate de TEXTO
//!
//! Os gates de comportamento (`offset_live_tests.rs`) chamam `recook` à mão e leem o mapa: eles
//! provam o COZIMENTO, não a costura. Nenhum unit test alcança o corpo do `render_frame`, que é
//! onde a ordem mora — a mesma razão do irmão `the_z_projection_reads_the_tree_after_the_sync`.

const SRC: &str = include_str!("../src/render_loop/mod.rs");

/// A posição (em bytes) da 1ª ocorrência de `needle`, ou pânico com a razão.
fn at(needle: &str) -> usize {
    SRC.find(needle).unwrap_or_else(|| {
        panic!(
            "`{needle}` sumiu do render_loop — se foi renomeado, atualize este gate (e confira \
             que o Offset vivo ainda chega à tela: `PH2D_BUILD_SMOKE=17`)"
        )
    })
}

/// **O `dispatch` recebe a geometria VIVA, não um mapa vazio.** É o argumento que faz o
/// preview existir; trocá-lo por `&LiveGeometry::new()` compila e apaga a feature em silêncio.
///
/// ⚠️ **A `LiveGeometry` do frame tem DUAS fontes** (Pattern Along Path, plano 23): o mapa é o
/// offset vivo FUNDIDO com as cópias do pattern, e por isso não cabe inline no argumento — ele
/// nasce num binding logo acima da chamada. A versão anterior deste gate procurava o literal
/// `self.offset_live.live()` DENTRO de uma janela de 400 bytes a partir do `dispatch(`, e o
/// binding a empurrou para fora dela: o gate ficou **VERMELHO sobre código correto**. Uma janela
/// de bytes é proxy de *"o argumento vem da fonte viva"*, e proxy quebra quando a forma muda —
/// então agora a pergunta é feita direto: **qual argumento** o `dispatch` recebe, e **de onde**
/// aquele nome nasce. Isso é mais forte que a janela: um `&LiveGeometry::new()` não tem binding
/// nenhum, e um binding que deixe de ler as fontes vivas fica vermelho pelo nome que falta.
#[test]
fn the_dispatch_is_handed_the_live_geometry() {
    let call = at("ph2d_vec_render::dispatch(");
    let args_end = SRC[call..]
        .find(");")
        .expect("fim da lista de argumentos do `dispatch`")
        + call;
    let args = &SRC[call..args_end];
    assert!(
        !args.contains("LiveGeometry::new()"),
        "o `dispatch` está recebendo um mapa VAZIO — o offset/pattern ao vivo some da tela e \
         NENHUM teste de unidade o nota:\n{args}"
    );
    let live_arg = args
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("&vec_live"))
        .expect(
            "o `dispatch` não recebe `&vec_live` — se o nome mudou, atualize este gate (e \
             confira que o preview ainda chega à tela: `PH2D_BUILD_SMOKE=17`)",
        );
    let _ = live_arg;
    // E o nome tem de nascer das DUAS fontes vivas, ANTES da chamada.
    let bind = at("let mut vec_live = self.offset_live.live()");
    assert!(
        bind < call,
        "`vec_live` é ligado DEPOIS do `dispatch` — o frame desenharia a resposta do frame anterior"
    );
    assert!(
        SRC[bind..call].contains("self.pattern_live"),
        "`vec_live` não recolhe `self.pattern_live` — as cópias do Pattern Along Path sumiriam \
         da tela sem nenhum teste de unidade notar"
    );
}

/// **O cozimento roda DEPOIS do `sync` e ANTES do desenho.** Antes do `sync`, a forma nova não
/// tem entidade e o `VecOffset` dela é invisível; depois do desenho, o frame pinta a resposta
/// do frame anterior.
#[test]
fn the_cook_runs_after_the_sync_and_before_the_draw() {
    let sync = at("crate::vec_entities::sync(");
    let cook = at("self.offset_live\n                .recook(");
    let draw = at("ph2d_vec_render::dispatch(");
    assert!(
        sync < cook,
        "o `offset_live::recook` corre ANTES do `vec_entities::sync` — uma forma recém-criada \
         ainda não tem entidade, então o offset dela não seria encontrado"
    );
    assert!(
        cook < draw,
        "o `offset_live::recook` corre DEPOIS do `dispatch` — o frame pintaria a geometria do \
         frame anterior, e todo retune apareceria com um quadro de atraso"
    );
}

/// **O preview NÃO pode escrever no documento**, e quem o garante é o COMPILADOR: o `recook`
/// recebe `&VecScene`. Este gate pina a assinatura para que a garantia não seja afrouxada por
/// distração — a versão anterior desta feature substituía a forma na cena a cada frame, que é
/// exatamente o que o Enio reprovou.
#[test]
fn the_cook_takes_the_scene_by_shared_reference() {
    const COOK: &str = include_str!("../src/offset_live.rs");
    let sig = COOK
        .find("pub(crate) fn recook(")
        .expect("o `recook` sumiu do offset_live");
    let tail = &COOK[sig..sig + 260.min(COOK.len() - sig)];
    assert!(
        tail.contains("scene: &VecScene"),
        "o `recook` deixou de receber a cena por referência COMPARTILHADA — a partir daqui o \
         preview pode escrever no documento, e a promessa \"a curva autorada não muda até o \
         Apply\" deixa de ser garantida pelo tipo:\n{tail}"
    );
}

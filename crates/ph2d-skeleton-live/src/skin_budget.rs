//! ⭐⭐⭐ **O QUE O QUADRO PAGA POR UMA PEÇA DE PELE, e quantas ele compra** — irmão do
//! [`crate::skin_image`] pelo tecto de LOC, com o corte por RESPONSABILIDADE: ali mora *a imagem
//! presa desenha-se como malha*, aqui **o orçamento e a lei que o reparte**.
//!
//! ⚠️ **O endereço não mudou:** o [`crate::skin_image`] re-exporta as duas coisas públicas daqui.
//!
//! ⛔⛔⛔ **A `parte_do_orcamento` e a `avisa_malhas_acima_do_orcamento` MORRERAM em 2026-09-17**,
//! com a fileira `Deform` (ordem do dono). As duas existiam para repartir e para avisar sobre um
//! **refinamento POR QUADRO** que já não acontece: a densidade é uma decisão do BIND
//! ([`crate::skin_bake_cache`]) e o quadro só posa o que lhe chega.
//!
//! ⚠️ **O que fica deste módulo é BANCADA**, não produto: o [`SKIN_FRAME_PIECES`] e as
//! [`refine_options`] continuam a ser a régua contra a qual os gates medem a lei de referência
//! ([`crate::skin_refine`]), que é o caminho de ANTES. *Um caminho de referência que só a bancada
//! corre é legítimo; o que não pode existir é um que o produto chame sem ninguém saber.*

use ph2d_poly2d::RefineOptions;

/// Um quadro de 60 fps, em microssegundos — o RECURSO de que o orçamento da pele é uma fatia.
const QUADRO_60FPS_US: usize = 16_667;

/// ⚠️ **A fatia do quadro que a pele pode gastar — e é a única ESCOLHA desta constante.** A pele é
/// uma coisa entre muitas no quadro (a arte do documento pelo Vello, o chrome, os passes de luz, o
/// resto das sprites); `1/10` deixa-lhe uma fatia visível sem lhe dar o quadro. Quem quiser medir
/// outra fatia tem o `PH2D_SKIN_PIECES`.
const FATIA_DA_PELE: usize = 10;

/// O custo MEDIDO de uma peça NOVA do `Smooth`, em nanossegundos — o maior dos dois custos do
/// refinamento, logo um tecto para qualquer mistura (a tabela do [`SKIN_FRAME_PIECES`]).
const CUSTO_POR_PECA_NS: usize = 324;

/// ⭐⭐⭐ **O ORÇAMENTO DE PEÇAS DA PELE DE IMAGEM, POR QUADRO** — derivado do recurso deste caminho:
/// o TEMPO do quadro.
///
/// ⚠️⚠️ **O número de antes era do VELLO, e descrevia outro caminho.** Até 2026-09-13 a pele era uma
/// camada do Vello e o tecto saía do buffer fixo de informação por desenho dele (`1 << 18` palavras,
/// `11` + bins por peça ⇒ metade dele dava `8 738` peças, e passar do buffer deixava o quadro
/// **em branco**). Desde a W2 do plano 03 a pele é uma malha no passe de sprites: aquele buffer já
/// não é gasto por ela. *§0.0: o número de um caminho morto não limita o vivo.*
///
/// ⭐⭐⭐ **O custo tem DUAS PARTES, medidas em 2026-09-16 com a malha de bind do PRODUTO** (sonda
/// `skin_image::tests::custo::measure_the_smooth_under_a_full_scene`; perfil `smoke`, `load 2,7`–`3,2`,
/// o MÍNIMO de 30, três corridas):
///
/// | o que o quadro faz | µs |
/// |---|---:|
/// | `Fast`: deformar + montar, por peça | `0,024` |
/// | `Smooth`: AVALIAR uma peça guardada (a lei decide não partir) | `0,156` |
/// | `Smooth`: cada peça NOVA | `0,311`–`0,324` |
/// | 1 imagem de `2 430` peças, `60°`, zoom `8×` (o orçamento enche) | `1,09`–`1,13 ms` = `6,6 %` de um quadro |
///
/// ⇒ o tecto usa o custo da peça NOVA, que limita qualquer mistura por cima:
/// `16,667 ms ÷ 10 ÷ 0,324 µs` = **`5 144` peças**.
///
/// ⛔⛔⛔ **O número de antes (`353 ns` ⇒ `4 721` peças) vinha de uma sonda que NÃO refinava**: a arte
/// dela media `200 × 100` px de ecrã, e só a malha de `72` peças chegava a partir (nota aberta até
/// 2026-09-16). Com o refinamento a trabalhar, o custo real era `0,36 µs` por peça avaliada e
/// **`~1,0 µs` por peça nova** — o orçamento cheio custava `3,17 ms` (**`19 %`** de um quadro, contra
/// os `10 %` prometidos).
///
/// ⭐⭐ **A cura foi o LIVRO DAS ARESTAS** (`ph2d_poly2d::refine_adaptive`): era uma árvore ordenada,
/// percorrida `~9` vezes por triângulo e nunca iterada; um índice pela ponta menor dá as mesmas
/// respostas (impressão digital de `48` casos do produto, igual ao bit) e corta a avaliação para
/// `0,156` e a peça nova para `~0,32`.
///
/// ⭐ **A lei de Hermite dos pesos está DENTRO destes números** (a sonda corre a lei do produto).
///
/// ⚠️ **E com as malhas guardadas ACIMA do orçamento o `Smooth` não avalia nada**
/// (`attach_skin_meshes`): nenhuma peça pode partir, e a saída é a do `Fast` ao bit. Antes dessa
/// cura, `8` imagens do smoke custavam `5,9 ms` por quadro para entregar o que o `Fast` entrega em
/// `0,21 ms`.
///
/// # A escada dos números (história — não reconstrua os de cima)
///
/// - **13/09 — `1 080 ns` ⇒ `1 543` peças.** Medido sobre um `Smooth` que refinava; depois mediu-se
///   que a lei UNIFORME é inerte acima de `orçamento / 4` peças, logo o produto pagava o `Fast` mais o
///   custo de decidir. *Um custo medido sobre um caminho que não corre é um orçamento que mente nos
///   dois sentidos* — este mentia para BAIXO, e a malha de bind de `2 430` peças disparava o
///   `avisa_malhas_acima_do_orcamento` em toda a execução.
/// - **16/09 (manhã) — `353 ns` ⇒ `4 721` peças.** A tabela era, por peça: descodificar a malha
///   guardada `0,014` · `Fast` `0,025` · recolher + costurar + enviar + desenhar (marginal de GPU,
///   sonda `ph2d-render::sprite_mesh_gpu::measure_the_frame_cost_of_a_mesh_sprite`) `0,013` ·
///   `Smooth` uniforme `0,087` · adaptativo `0,340` µs. ⚠️ A sonda que dava o `0,340` já não
///   refinava (ver acima). A lei de Hermite dos pesos mediu-se então INTERCALADA com a linear
///   (sonda `sonda_o_custo_da_lei_dos_pesos` da `ph2d-app-vec`): `−1 %` a zoom `4×` e
///   `+2,5`–`+3,8 %` a zoom `8×`.
/// - **O livro das arestas foi cortado DUAS vezes:** dois mapas e um `Vec` por aresta (`0,52 µs`) →
///   um mapa só com os donos num par fixo (`0,34`, `−35 %`) → um índice pela ponta menor (16/09,
///   F6-t: avaliação `0,156`, peça nova `~0,32`).
/// - **16/09 (tarde) — `324 ns` ⇒ `5 144` peças** (a tabela acima).
pub const SKIN_FRAME_PIECES: usize = QUADRO_60FPS_US * 1_000 / FATIA_DA_PELE / CUSTO_POR_PECA_NS;

/// ⛔⛔ **A OUTRA PONTA DO TECTO, verificada na COMPILAÇÃO.** Um tecto apertado de mais deixa de
/// refinar uma malha comum, e a barra sai da malha que o produto **de facto** guarda: a do bind de
/// uma arte normal mede `~2 430` peças (medido 2026-09-15), então um orçamento abaixo disso não
/// deixa a lei adaptativa partir **uma** aresta que seja. Uma fatia mais fina (ou um custo por peça
/// maior, medido outra vez) tem de PARAR a build aqui, e não passar em silêncio.
///
/// ⚠️ A barra anterior era `1 000` e vinha de `k = 2` sobre uma malha de `~200` peças — a aritmética
/// da lei uniforme, que já não é a do produto.
const _: () = assert!(
    SKIN_FRAME_PIECES >= 2_500,
    "o orcamento da pele nao chega para refinar a malha que o bind de facto guarda"
);

/// ⭐⭐⭐ **OS NÚMEROS DO `Smooth`**: a tolerância da `ph2d-poly2d`, o orçamento do QUADRO e a LEI.
///
/// ⚠️ **`PH2D_SKIN_PIECES=<n>` afina o orçamento do QUADRO**, e não de uma imagem — ele existe desde
/// o report *«Smooth bugado quebrando a forma»* (dono, 2026-09-10), cuja causa se mediu depois
/// (a porta crua enchia o atlas, F6-e). *Um smoke que MEDE vale mais que um smoke que pergunta.*
///
/// ⭐⭐⭐ **`PH2D_SKIN_REFINE=uniforme` volta à lei do `k` global**, para bissecar. Ela é o que o
/// `Smooth` usou até 2026-09-16 e é **provadamente inerte** acima de `orçamento / 4` peças — ver
/// `ph2d_poly2d::refine_adaptive`.
#[must_use]
pub fn refine_options() -> RefineOptions {
    static PECAS: std::sync::OnceLock<Option<usize>> = std::sync::OnceLock::new();
    let escolhido = *PECAS.get_or_init(|| {
        std::env::var("PH2D_SKIN_PIECES")
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .filter(|n| *n > 0)
    });
    static LEI: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let adaptativo = *LEI.get_or_init(|| {
        !std::env::var("PH2D_SKIN_REFINE").is_ok_and(|v| v.trim().eq_ignore_ascii_case("uniforme"))
    });
    RefineOptions {
        max_pieces: escolhido.unwrap_or(SKIN_FRAME_PIECES),
        adaptativo,
        ..RefineOptions::default()
    }
}

//! ⭐⭐⭐ **O QUE O QUADRO PAGA POR UMA PEÇA DE PELE, e quantas ele compra** — irmão do
//! [`crate::skin_image`] pelo tecto de LOC, com o corte por RESPONSABILIDADE: ali mora *a imagem
//! presa desenha-se como malha*, aqui **o orçamento e a lei que o reparte**.
//!
//! ⚠️ **O endereço não mudou:** o [`crate::skin_image`] re-exporta as duas coisas públicas daqui.

use ph2d_poly2d::RefineOptions;

/// Um quadro de 60 fps, em microssegundos — o RECURSO de que o orçamento da pele é uma fatia.
const QUADRO_60FPS_US: usize = 16_667;

/// ⚠️ **A fatia do quadro que a pele pode gastar — e é a única ESCOLHA desta constante.** A pele é
/// uma coisa entre muitas no quadro (a arte do documento pelo Vello, o chrome, os passes de luz, o
/// resto das sprites); `1/10` deixa-lhe uma fatia visível sem lhe dar o quadro. Quem quiser medir
/// outra fatia tem o `PH2D_SKIN_PIECES`.
const FATIA_DA_PELE: usize = 10;

/// O custo MEDIDO de uma peça ENTREGUE com `Smooth`, em nanossegundos (a tabela do
/// [`SKIN_FRAME_PIECES`]).
const CUSTO_POR_PECA_NS: usize = 353;

/// ⭐⭐⭐ **O ORÇAMENTO DE PEÇAS DA PELE DE IMAGEM, POR QUADRO** — derivado do recurso deste caminho:
/// o TEMPO do quadro.
///
/// ⚠️⚠️ **O número de antes era do VELLO, e descrevia outro caminho.** Até 2026-09-13 a pele era uma
/// camada do Vello e o tecto saía do buffer fixo de informação por desenho dele (`1 << 18` palavras,
/// `11` + bins por peça ⇒ metade dele dava `8 738` peças, e passar do buffer deixava o quadro
/// **em branco**). Desde a W2 do plano 03 a pele é uma malha no passe de sprites: aquele buffer já
/// não é gasto por ela, e `8 738` peças custariam hoje **`9,4 ms`** — mais de metade de um quadro de
/// 60 fps. *§0.0: o número de um caminho morto não limita o vivo.*
///
/// ⭐ **O que UMA peça custa, MEDIDO OUTRA VEZ em 2026-09-16** (a lei do refinamento mudou, logo o
/// número tinha de ser remedido; `load 5,6`–`6,1`, o MÍNIMO de 40/60 corridas, **três** corridas com
/// leituras entre `0,328` e `0,355` — as sondas são
/// `skin_image::tests::custo::measure_the_cpu_cost_of_a_skinned_frame` e a
/// `ph2d-render::sprite_mesh_gpu::measure_the_frame_cost_of_a_mesh_sprite`):
///
/// | o que o quadro faz por peça | µs |
/// |---|---:|
/// | descodificar a malha guardada (postcard, **por quadro**) | `0,014` |
/// | `Fast`: descodificar + deformar + montar o `SpriteMesh` | `0,025` |
/// | recolher + costurar a tira + enviar + DESENHAR (marginal, GPU esperada) | `0,013` |
/// | `Smooth` **uniforme**: o quadro inteiro, por peça entregue | `0,087` |
/// | **`Smooth` ADAPTATIVO: o quadro inteiro, por peça ENTREGUE** | **`0,340`** |
///
/// ⇒ `16,667 ms ÷ 10 ÷ 0,353 µs` = **`4 721` peças**.
///
/// ⭐ **E a lei de Hermite dos pesos (2026-09-16, [`crate::skin_refine`]) cabe na folga que o `353`
/// já tinha sobre o `340`** — medida onde ela trabalha (o refinamento da cena do smoke, as duas leis
/// INTERCALADAS, `load 5,8`–`6,2`, o mínimo de 40 em três corridas; sonda
/// `sonda_o_custo_da_lei_dos_pesos` da `ph2d-app-vec`): **`−1 %`** a zoom `4×` e **`+2,5`–`+3,8 %`**
/// a zoom `8×`, onde o orçamento enche. `0,340 × 1,038 = 0,353` ⇒ o tecto não muda.
///
/// ⚠️⚠️ **ABERTO: a sonda do custo desta crate deixou de exercer o refinamento.** A arte dela mede
/// `200 × 100` px de ecrã, e só a linha de `72` peças chega a partir alguma; as outras quatro
/// entregam a malha guardada e medem *decidir não partir*. Medido em 2026-09-16 (`load 6`–`10`) ela
/// lê `0,62`–`0,64 µs` no adaptativo e `0,47` no uniforme — nenhum dos dois reproduz a tabela acima.
/// *Uma sonda cujo sujeito deixou de fazer a coisa medida mede outra coisa com o mesmo nome.*
///
/// ⛔⛔⛔ **E O NÚMERO DE ANTES (`1 080 ns` ⇒ `1 543` peças) ERA DE UM CAMINHO QUE O PRODUTO NUNCA
/// CORREU.** Ele foi medido em 2026-09-13 sobre um `Smooth` que **refinava**; desde então mediu-se
/// que a lei uniforme é **inerte** acima de `orçamento / 4` peças, logo o que o produto de facto
/// pagava era o `Fast` mais o custo de decidir. *Um custo medido sobre um caminho que não corre é
/// um orçamento que mente nos dois sentidos* — e este mentia para BAIXO, o que fazia a malha de
/// bind de `2 430` peças disparar o `avisa_malhas_acima_do_orcamento` em toda a execução.
///
/// ⚠️⚠️ **A lei nova é `3,9×` mais cara POR PEÇA** (`0,340` contra `0,087`), e isso é o preço de
/// ela decidir: ela mede o desvio de cada aresta, mantém o livro de donos e escolhe. *O que ela
/// compra em troca é entregar alguma coisa* — a uniforme era barata porque não fazia nada.
///
/// ⭐ **E o livro de contas já foi medido e cortado uma vez:** a 1.ª redacção usava DOIS mapas e um
/// `Vec` por aresta, e lia `0,52 µs`; com um mapa só e os donos num par fixo desceu a `0,34`
/// (`−35 %`). Ver `ph2d_poly2d::refine_adaptive`.
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

/// ⭐ **A parte do orçamento do quadro que cabe a uma imagem** — proporcional à malha que ela guarda,
/// e nunca abaixo dela (a malha guardada é o desenho mínimo; não há como desenhá-la com menos).
///
/// ⚠️ **Proporcional dá a todas o MESMO factor de crescimento**, logo a mesma qualidade relativa:
/// cada imagem pode chegar a `orçamento × (peças dela / peças de todas)`, e essa razão é a mesma
/// para todas. *Repartir por igual daria à imagem pequena um luxo que a grande não tem.*
pub(crate) fn parte_do_orcamento(triangulos: usize, guardadas: usize, orcamento: usize) -> usize {
    let parte = orcamento.saturating_mul(triangulos) / guardadas.max(1);
    parte.max(triangulos)
}

/// ⛔ **Uma vez por processo**, e nunca calado (DIRETIVA §2: zero no-op silencioso): as malhas
/// GUARDADAS das imagens deste quadro já passam do orçamento, e o `Smooth` não tem refinamento a
/// cortar — cada imagem é desenhada com a malha que guardou.
pub(crate) fn avisa_malhas_acima_do_orcamento(guardadas: usize, orcamento: usize) {
    static AVISADO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !AVISADO.swap(true, std::sync::atomic::Ordering::Relaxed) {
        eprintln!(
            "[bone] as imagens presas deste quadro guardam {guardadas} pecas e o orcamento do \
             quadro e' {orcamento} (PH2D_SKIN_PIECES) — o Smooth nao refina nenhuma: cada uma e' \
             desenhada com a malha guardada"
        );
    }
}

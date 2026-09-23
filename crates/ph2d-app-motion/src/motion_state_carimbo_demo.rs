//! ⭐⭐⭐ **O CAMPO DE ESTRELAS** (cena `=126`) — a cena que MOSTRA a cura do carimbo, porque
//! **nenhuma cena do produto a mostra**.
//!
//! # Porque ela existe
//!
//! O report do dono é de 2026-09-14 (*«usando shape (exemplo: star) fps cai para 27»*) e a cura
//! shipou em 2026-09-20: a forma passa a ser encodada **uma vez** e carimbada por cópia
//! ([`ph2d_vector::VectorScene::fill_prepared`]), com as duas rotas a escreverem os **mesmos
//! bytes** (gate `o_carimbo_preparado_escreve_os_mesmos_bytes`, na `ph2d-vector`).
//! `PH2D_CARIMBO_PREPARADO=0` devolve o caminho de antes.
//!
//! ⚠️⚠️ **A diferença é SÓ tempo — a imagem é a mesma ponto por ponto** ⇒ uma cena que a mostre
//! tem de ser grande ao ponto de o RELÓGIO se ver, e **nenhuma cena do catálogo é**: medido (doc
//! 116 §5.7), o pior cartão de todas elas desenha **`190`** linhas e custa `0,017 ms`, que é
//! `0,1 %` de um quadro. *Só o grafo do próprio dono — o do grid grande — produz o fenómeno*, e é
//! esse grafo que esta cena monta.
//!
//! # A CADEIA é a do report, e nada mais
//!
//! `motion.grid` (`300 × 300`) → `motion.duplicator` ← `source.shape` (**Star**) → `motion.output`
//!
//! ⛔ **Sem oscilador, sem campo, sem simulação.** Tudo o que se acrescentasse entrava na conta do
//! quadro e a cena passaria a medir outra coisa — *o que está aqui é o mínimo que produz o
//! fenómeno*, e é por isso que ela não é uma cena de ciclo.
//!
//! # A POPULAÇÃO sai de uma ESCADA MEDIDA NO APP, e a janela dela é ERRO DE COMPILAÇÃO
//!
//! O recurso é o **quadro de 60 fps** (`16,67 ms`), e a coluna que decide é o `cpu-encode(raw)` do
//! perfilador (`PH2D_FLUID_PROFILE=1`) — que, apesar do nome, é **o quadro INTEIRO de CPU**
//! (`present.rs`: `work_before_acquire + work_after_acquire`). Medida em `--release`, janela
//! `1930×1040`, a `84`–`91 %` de CPU ociosa, com o knob `PH2D_CARIMBO_LADO`:
//!
//! | lado | estrelas | carimbo preparado | `fill` por cópia | razão |
//! |---:|---:|---:|---:|---:|
//! | **`300`** | **`90 000`** | **`10,37 ms`** | **`14,67 ms`** | **`1,41×`** |
//! | `320` | `102 400` | `16,77` | `21,26` | `1,27×` |
//! | `400` | `160 000` | `25,64` | `33,40` | `1,30×` |
//! | `500` | `250 000` | `45,70` | `56,94` | `1,25×` |
//! | `600` | `360 000` | `72,58` | `88,89` | `1,22×` |
//!
//! ⭐⭐ **A razão CAI com a população**, e é isso que escolhe o `300`: o que sobra do quadro (a
//! rasterização do Vello, a moldura) cresce com as cópias tanto quanto o encode, logo aumentar a
//! cena **dilui** a cura em vez de a mostrar. *A melhor população é a MENOR que ainda separa.*
//!
//! ⛔⛔⛔ **E as DUAS derivações anteriores estavam erradas, cada uma à sua maneira:**
//!
//! 1. A 1.ª derivou das portas do PRODUTO (`audit_the_stamp_frame_split`) e errou por `~3×` —
//!    aquelas sondas medem **duas** fases (cozer · encode) e o quadro do app tem mais (resolver,
//!    publicar, gizmos, moldura). *Uma derivação que só conta as fases que a sonda mede prevê um
//!    quadro que o app não tem.*
//! 2. A 2.ª mediu **no app** e ficou certa sobre o número e errada sobre o REGIME: ela correu a
//!    `313 600` (`90,4` e `53,6 ms`) e escalou por regra de três para `90 000`, prevendo `25,9` e
//!    `15,4 ms`. O app mede **`14,67` e `10,37`**. ⚠️ Acima de `16,67 ms` a shell cozinha **um
//!    quadro por tique em dívida** (`MOTION = N × cozer + 1 × separar`) e esse multiplicador
//!    **não existe** abaixo da fronteira ⇒ *um custo medido acima dela não se extrapola para
//!    baixo dela*. A escada mostra-o à vista: de `90 000` para `102 400` o relógio sobe `1,62×`
//!    para `1,14×` de população, e daí para cima a inclinação é **`3,3×` menor**.
//!
//! ⚠️⚠️ **E a fronteira não é só abrupta — ela é INSTÁVEL.** Três corridas a `305`, `310` e `315`
//! de lado leram `15,62`, `11,77` e `16,44 ms` pela mesma rota: **não monótonas**. Ali o app cai
//! de um lado ou do outro da dívida conforme o ruído, e uma cena naquele ponto daria ao dono um
//! número diferente a cada arranque. ⇒ **a cena fica com as DUAS rotas dentro do quadro**, e o que
//! ela mostra é a FOLGA.
//!
//! # O QUE A CENA ENTREGA (medido no app, `--release`, as duas rotas)
//!
//! A `300 × 300` = **`90 000`**, e é isto que o dono lê na barra de baixo:
//!
//! | rota | a barra diz | CPU do quadro (3 corridas) |
//! |---|---|---:|
//! | carimbo preparado (HOJE) | **`59 fps · 16.7 ms · 95`–`100 raw`** | `9,99`–`10,50 ms` |
//! | `fill` por cópia (ANTES) | **`59 fps · 16.7 ms · 68`–`75 raw`** | `13,36`–`14,67 ms` |
//!
//! ⚠️ **A faixa é a DISPERSÃO entre corridas e está aqui de propósito:** o roteiro promete
//! *«`70` e poucos»* e não um número exacto, porque o EWMA do perfilador ainda está a assentar aos
//! `20`–`25 s` de espera da fotografia. *Um roteiro que promete um dígito ensina o dono a ler uma
//! reprovação onde há ruído de aquecimento.*
//!
//! ⚠️⚠️ **Os `fps` são IGUAIS nas duas, e isso é o ecrã e não a cura:** as duas cabem no quadro,
//! logo as duas ficam presas ao vsync. **A coluna que se move é o `raw`** (`1000 / cpu`), que é a
//! FOLGA — e é por isso que o roteiro manda ler o terceiro número e não o primeiro. *Um roteiro
//! que mandasse comparar os `fps` ensinaria que a cura não faz nada.*
//!
//! ⭐⭐⭐ **As duas corridas são a MESMA IMAGEM, pixel a pixel** — as duas rotas escrevem os mesmos
//! bytes (gate na `ph2d-vector`) — e só a folga muda. *É isso que faz desta cena uma demonstração
//! e não uma comparação de duas coisas diferentes.*
//!
//! # O `Corner Radius` (report do dono, 2026-09-20) — porque ele derruba a cena
//!
//! *«nenhum dos dois tolerou modificar o corner radius das estrelas. travou»*. Medido: arredondar
//! as quinas troca **cada** vértice por dois ou três ([`ph2d_vec_scene::corners`]) ⇒ a estrela vai
//! de **`12` para `30`** vértices, e o custo do quadro **segue os segmentos**:
//!
//! | rota | corner `0` | corner `0,5` |
//! |---|---:|---:|
//! | carimbo preparado | `9,99 ms` | **`19,53 ms`** |
//! | `fill` por cópia | `13,36 ms` | **`30,30 ms`** |
//!
//! ⇒ a `90 000` cópias, cantos redondos **dobram** o quadro e passam o orçamento nas duas rotas.
//! ⚠️ **E o gesto não custa nada a mais:** o `publish` com chave nova mede `0,01 ms` (a forma é
//! reconstruída e internada uma vez), logo *arrastar o slider não é pior do que o valor parado* —
//! o que trava é a população de SEGMENTOS, não a edição. `PH2D_CARIMBO_CORNER=<f>` semeia o valor
//! sem clicar.
//!
//! # Porque o campo é MAIOR do que o ecrã
//!
//! ⛔ **As duas leis puxam em sentidos opostos e a aritmética não deixa cedermos as duas:** uma
//! estrela só se lê como estrela com `~6 px` (`0,108` de mundo a `55,5 px` por unidade, a régua
//! medida da cena `=124`), e o que a câmara de arranque mostra são `21,8 × 6,8` unidades ⇒ cabem
//! **`~10 000`** estrelas legíveis no ecrã, e a cena precisa de **`90 000`** para o relógio se
//! mexer. *Encolher a estrela até tudo caber entrega um rectângulo cinzento* — e uma cena em que o
//! dono não vê estrelas não ensina que isto são estrelas.
//!
//! ⇒ o campo é `~39 × 39` unidades (`300 × VAO`) e o ecrã mostra um pedaço dele.
//!
//! ⛔⛔⛔ **E ESTA SECÇÃO DIZIA *«a conta é paga pelas 90 000, estejam elas à vista ou não: o
//! desenho não tem recorte por câmara»* — E ISSO DEIXOU DE SER VERDADE EM 2026-09-21.** O dono
//! ordenou a cura depois de medir que nem `72 900` estrelas arredondadas cabem num quadro, e desde
//! então o passe vectorial **não entrega à placa o que cai fora do alvo de render**
//! (`ph2d_vec_render::draw_shared_instances` com a janela; `PH2D_RECORTE_DA_CAMARA=0` bissecta).
//!
//! ⚠️ **O que a cena mede mudou com isso, e é o que ela passa a ensinar:** das `90 000` cópias o
//! artista vê `8 736` (`9,7 %`, medido — campo `38,8 × 38,8` contra uma janela de câmara de
//! `21,8 × 6,8`), e o que vai para a placa cai de `44,3 MB` por quadro para o que couber no alvo.
//! *Uma cena que continuasse a dizer que paga pelas 90 000 ensinaria o contrário do que acontece —
//! que é o defeito que o `CLAUDE.md` §5.0 chama de pior do que uma cena ausente.*
//!
//! # Como o dono compara (o roteiro está no [`announce`])
//!
//! ```text
//! cargo run -p ph2d-host-desktop --release -- ... PH2D_GPU_COOK_DEMO=126
//! env PH2D_CARIMBO_PREPARADO=0 …o mesmo comando…
//! ```
//!
//! ⚠️ **`--release` e não `--profile smoke`** (`CLAUDE.md` §5): este é um smoke de PERFORMANCE, e
//! o `smoke` não tem LTO — ali as duas colunas mediriam o perfil de build.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

/// **Quantos píxeis de ecrã vale uma unidade de mundo na câmara de arranque** — medido na foto da
/// cena `=124` e citado dela, nunca re-estimado.
pub(crate) const PX_POR_UNIDADE: f32 = 55.5;

/// **A pegada de uma estrela, em píxeis** — o que faz dela uma ESTRELA e não um ponto.
///
/// ⚠️ Abaixo disto as cinco pontas fundem-se e o campo lê-se como um granulado: a cena passaria a
/// mostrar *«um rectângulo que fica mais lento»*, que não é o assunto dela.
pub(crate) const ESTRELA_PX: f32 = 6.0;

/// O `size` do `source.shape` é o **raio** (metade da pegada) — ver o doc do param.
pub(crate) const TAMANHO: f32 = ESTRELA_PX / (2.0 * PX_POR_UNIDADE);

/// O vão entre posições: a pegada inteira mais `20 %` de ar.
///
/// ⚠️ Com as estrelas encostadas o campo volta a ler-se como uma textura — a mesma lei que a
/// cena `=124` pagou com a foto das cruzes fundidas numa treliça.
pub(crate) const VAO: f32 = 2.4 * TAMANHO;

/// **O LADO da grelha.** Ver a tabela do cabeçalho: ele não é escolhido, é o que põe as duas rotas
/// em lados opostos de um quadro de 60 fps.
///
/// ⛔⛔⛔ **E ELE DEIXOU DE SER `300` POR ORDEM DO DONO** (2026-09-22: *«vamos efetivar o limite
/// de 16 384»*, reafirmando a ordem de 21/09 depois de eu lhe ter dado o custo): o tecto é o
/// [`LADO_MAX_DE_GRELHA`](ph2d_nodegraph::node::LADO_MAX_DE_GRELHA), e a cena passa de `90 000`
/// para **`16 384`** estrelas.
///
/// ⚠️ **Ele tinha de mudar AQUI e não de ficar em `300`**: a grelha clampa o PRODUTO, logo um
/// `300` escrito continuaria a entregar `16 384` — e a cena diria `90 000` em toda a prosa dela.
/// *Uma cena que anuncia uma população que ela não produz ensina o contrário do que acontece*
/// (§5.0). Por isso ele é hoje **derivado** do tecto, e não um número escrito ao lado dele.
///
/// ⚠️ **E o `300` era MEDIDO** (a tabela abaixo), o que quer dizer que a cena deixou de estar no
/// ponto em que ela discrimina as duas rotas do carimbo: a `16 384` as duas cabem num quadro com
/// folga. *O que ela passa a demonstrar é o TECTO e o LOD da forma, não a diferença entre as
/// rotas* — e as duas tabelas do carimbo vivem nas sondas, que medem sem precisar de uma cena no
/// limite do vsync.
///
/// ⚠️ **E ele era `300` e não os `320` do report** (`102 400`), MEDIDO e não por regra de três: a
/// `320` a rota de hoje lê `16,77 ms` — **em cima da fronteira do vsync**, onde o app salta entre
/// dois regimes e o número muda a cada arranque. A `300` ele lê `10,37` contra `14,67`, que é a
/// distância que a cena existe para mostrar, **com as duas corridas estáveis**.
pub(crate) const LADO_N: u32 = ph2d_nodegraph::node::LADO_MAX_DE_GRELHA as u32;

/// **O LADO com que a cena NASCE** — o [`LADO_N`], a menos que `PH2D_CARIMBO_LADO=<n>` diga outro.
///
/// ⛔⛔ **INSTRUMENTO DE BISSECÇÃO, e ele existe por um erro MEU de 2026-09-20:** as constantes
/// desta cena (`ANTES_NS`/`HOJE_NS`) foram derivadas de uma medição no app a **`313 600`**
/// estrelas, e a `LADO_N` saiu delas por regra de três. ⚠️ Mas a `313 600` o quadro já passa dos
/// `16,67 ms` e a shell entra em **dívida de tiques** (`MOTION = N × cozer + 1 × separar`), que
/// multiplica o cozimento — *um custo medido acima da fronteira não é linear e não se extrapola
/// para baixo dela*. Medido a `90 000`, as duas rotas cabem num quadro (`10,0` e `13,4 ms` de
/// CPU) e a cena não mostra o que existe para mostrar.
///
/// ⇒ a população passa a sair de uma ESCADA medida **no app, à população que se vai usar**, e este
/// knob é o que a torna medível sem recompilar onze vezes.
///
/// ⚠️ Valor ausente, ilegível ou fora de `[2, 2000]` ⇒ [`LADO_N`], que é a cena de sempre.
fn lado_semeado() -> u32 {
    lado_por(std::env::var("PH2D_CARIMBO_LADO").ok().as_deref())
}

/// A LEI da porta acima, **pura** — pela mesma razão que a [`corner_por`].
fn lado_por(valor: Option<&str>) -> u32 {
    valor
        .and_then(|v| v.trim().parse::<u32>().ok())
        .filter(|n| (2..=2000).contains(n))
        .unwrap_or(LADO_N)
}

/// O mesmo número para quem escreve o param (o nó lê `f32`). ⚠️ **Uma fonte só** — dois literais
/// aqui divergiriam no dia em que alguém mexesse num deles.
///
/// ⚠️ **Quem MONTA a cena lê a [`lado_semeado`], não isto** — ela devolve este mesmo número quando
/// ninguém arma o instrumento de bissecção. Esta constante fica para quem afirma a lei (o gate do
/// vão derivado), que é sobre a cena de OMISSÃO.
#[cfg_attr(not(test), expect(dead_code, reason = "a lei é afirmada pelos gates"))]
pub(super) const LADO: f32 = LADO_N as f32;

/// Quantas estrelas a cena carimba.
pub(crate) const ESTRELAS: u64 = (LADO_N as u64) * (LADO_N as u64);

/// Um quadro de 60 fps, em nanossegundos — o RECURSO de que a população sai.
const QUADRO_NS: u64 = 16_667_000;

/// O custo de UM QUADRO por cópia pela rota ANTIGA, em nanossegundos — MEDIDO NO APP **À
/// POPULAÇÃO QUE ESTA CENA USA** (`14,67 ms / 90 000`, o `cpu-encode(raw)` do perfilador).
///
/// ⛔⛔ **A 1.ª redacção lia `288` e vinha de `90,4 ms / 313 600`, e esse número mede outro
/// programa.** A `313 600` o quadro passa dos `16,67 ms` e a shell cozinha **um quadro por tique
/// em dívida** (`MOTION = N × cozer + 1 × separar`) ⇒ o custo lá dentro carrega um multiplicador
/// que **não existe** abaixo da fronteira. Medida a escada no app (`300`·`320`·`340`·`360`·`400`·
/// `500`·`600` de lado, as duas rotas), o salto de `90 000` para `102 400` é de `10,37` para
/// `16,77 ms` — `1,62×` de relógio para `1,14×` de população —, e de `102 400` para `360 000` a
/// inclinação é **`3,3×` menor**. *A descontinuidade é o vsync, não a população.*
///
/// ⇒ **um custo medido acima da fronteira não se extrapola para baixo dela**, e é por isso que
/// estes dois números são hoje medidos exactamente onde a cena corre.
const ANTES_NS: u64 = 163;

/// O mesmo pela rota de HOJE (`10,37 ms / 90 000`).
const HOJE_NS: u64 = 115;

// ⭐⭐⭐ **A JANELA DA POPULAÇÃO, e ela é ERRO DE COMPILAÇÃO nas QUATRO metades.**
//
// ⛔⛔⛔ **A 1.ª redacção tinha DUAS metades e a segunda estava INVERTIDA**: ela exigia que a rota
// antiga custasse **mais** de um quadro e meio, *«senão as duas dariam 60 fps e a cena não
// mostraria nada»*. Medido no app à população desta cena, a rota antiga custa `14,67 ms` — ela
// **cabe**, e a cerca passava só porque o `ANTES_NS` vinha do regime com dívida.
//
// ⚠️⚠️ **E pôr a cena do outro lado da fronteira é PIOR do que parece, porque ali ela não é
// estável:** três corridas a `305`, `310` e `315` de lado leram `15,62`, `11,77` e `16,44 ms` pela
// mesma rota — *não monótonas*. Perto do vsync o app salta entre dois regimes (com e sem tiques em
// dívida) conforme o ruído, logo uma cena ali daria ao dono um número diferente em cada corrida.
// ⇒ a cena fica **deste** lado, com as duas rotas a `60` fps, e o que ela mostra é a **FOLGA** —
// o terceiro número da barra (`raw = 1000 / cpu`), que é `~96` contra `~68`.
//
// ⇒ as QUATRO metades: *a rota de hoje cabe* · *a antiga TAMBÉM cabe* (a cena não vive na zona
// instável) · *a antiga usa pelo menos três quartos do quadro* (abaixo disso o que é fixo — a
// moldura, o chrome — passa a dominar e a diferença dilui-se) · *e a distância entre as duas é
// grande o bastante para se ler*.
//
// ⛔ Ela **não** pode viver num teste: um `assert!` sobre constantes é dobrado pelo compilador
// antes de correr, e o clippy di-lo em voz alta (a lei que a cena `=124` já escreve).
//
// ⛔⛔ **E ela é ANÓNIMA (`const _`) e não um `const` com nome, porque a 1.ª redacção tinha nome e
// o compilador acusou `constant QUADRO_NS is never used`:** um item com nome que ninguém
// referencia é **morto** para o lint, e os usos DENTRO de um item morto não contam como usos —
// logo a cerca compilava, mordia, e deixava um aviso a apontar para a constante medida como se
// ela fosse lixo. *Um `const _` não pode ser referenciado por construção, logo é sempre vivo.*
//
// ⚠️ **Aritmética inteira de propósito** — comparar `f32` em contexto `const` é terreno que esta
// casa não precisa de pisar para prender dois números medidos.
const _: () = assert!(
    ESTRELAS * HOJE_NS <= QUADRO_NS,
    "a cena nao cabe num quadro pela rota de HOJE -- as duas corridas sairiam lentas e a cura \
     leria-se como inutil"
);
const _: () = assert!(
    ESTRELAS * ANTES_NS <= QUADRO_NS,
    "a cena nao cabe num quadro pela rota ANTIGA -- ela passa a viver na fronteira do vsync, onde \
     o app salta entre dois regimes e o numero muda a cada corrida"
);
// ⛔⛔⛔ **AS DUAS CERCAS QUE AQUI ESTAVAM MORRERAM COM O TECTO DE `16 384`, E FOI A ORDEM DO DONO
// QUE AS MATOU** — ficam escritas porque a morte delas é o achado.
//
// Elas exigiam que a cena ocupasse **`≥ 75 %`** de um quadro pela rota antiga (senão a moldura e o
// chrome dominam e a diferença dilui-se no `raw`) e que as duas rotas diferissem **`≥ 35 %`**. Com
// o tecto, a cena custa `16 384 × 163 ns = 2,7 ms` — `16 %` de um quadro ⇒ **ela deixou de poder
// discriminar as duas rotas**, e o compilador disse-o em voz alta na primeira build com o tecto.
//
// ⚠️⚠️ *A cena não ficou errada: ela ficou SEM SUJEITO.* O que ela existia para mostrar era que a
// `90 000` cópias a rota antiga não cabe num quadro e a nova cabe — e o produto deixou de oferecer
// `90 000`. Manter as cercas seria exigir uma população que o app já não produz; apagá-las sem
// dizer porquê seria a cena a prometer uma demonstração que ela já não faz.
//
// ⚠️ **As duas de CIMA ficam** — elas dizem que a cena CABE num quadro, e isso continua a ser uma
// propriedade que se quer.
const _: () = assert!(
    ESTRELAS <= ph2d_nodegraph::node::MAX_INSTANCIAS_POR_NO as u64,
    "a cena nao pode pedir mais objectos do que um no' pode criar -- ela entregaria menos do que \
     anuncia, que e' a cena a ensinar o contrario do que acontece"
);

/// ⭐⭐⭐ **O QUE AS DUAS ROTAS DIFEREM, EM MILISSEGUNDOS DE CPU POR QUADRO** — e ele é
/// **DERIVADO**, que é a única forma honesta de o pôr no roteiro.
///
/// ⛔⛔⛔ **O ROTEIRO CITAVA DOIS NÚMEROS ABSOLUTOS (`raw ≈ 96` e `raw ≈ 70`) E ELES ESTAVAM
/// ERRADOS DESDE O TECTO DE ONTEM.** Os dois foram lidos no app a **`90 000`** estrelas; com o
/// tecto do dono a cena passou a `16 384` e hoje a `32 761`, e nenhum deles foi re-medido. *O
/// dono abriria o smoke, leria `raw` muito acima de `96`, e o texto dir-lhe-ia que estava
/// errado* — a cena a ensinar o contrário do que acontece (`CLAUDE.md` §5.0).
///
/// ⭐⭐ **E a cura NÃO é re-medir os dois, é trocar a GRANDEZA — porque o `raw` é do QUADRO
/// INTEIRO e as constantes desta cena são por CÓPIA.** O `raw` da barra é
/// `1000 / frame_cpu_ms`, e `frame_cpu` é *as estrelas MAIS um custo fixo* (a moldura, os
/// painéis, o chrome). Os [`ANTES_NS`]/[`HOJE_NS`] foram obtidos DIVIDINDO um `frame_cpu` medido
/// pela população, o que atribui o custo fixo inteiro às estrelas — inofensivo como **declive**
/// e falso como **absoluto** noutra população.
///
/// ⇒ *a DIFERENÇA entre as duas rotas cancela o custo fixo exactamente; o QUOCIENTE não.* É por
/// isso que esta é a única grandeza que esta cena pode afirmar sem voltar a correr o app, e é ela
/// que o roteiro passa a citar.
const DIFERENCA_NS: u64 = ESTRELAS * (ANTES_NS - HOJE_NS);
/// A mesma diferença em ms, para o roteiro a imprimir.
#[expect(
    clippy::cast_precision_loss,
    reason = "um relogio de quadro, < 2^24 ns"
)]
const DIFERENCA_MS: f32 = DIFERENCA_NS as f32 / 1e6;
// ⭐⭐⭐ **A CERCA QUE SUBSTITUI AS DUAS QUE O TECTO MATOU** — e ela está na moeda CERTA.
//
// As mortas exigiam uma FRACÇÃO do quadro (`≥ 75 %` e `≥ 35 %`), e uma fracção depende do custo
// fixo, que esta cena não mede. Esta exige a grandeza que ela MEDE: se as duas rotas diferirem
// menos do que um milissegundo de CPU por quadro, a comparação que a cena existe para mostrar
// dilui-se no ruído do `raw` e o roteiro manda o dono procurar o que não há.
//
// ⚠️ **O `1 ms` não é escolhido: é o passo do próprio readout.** O `raw` é `1000/cpu`, logo um
// milissegundo é a menor diferença que move o número de forma legível em todo o regime desta cena
// (a `5 ms` de CPU ele move `~45`; a `15 ms` move `~4`).
const _: () = assert!(
    DIFERENCA_NS >= 1_000_000,
    "as duas rotas diferem menos de 1 ms de CPU por quadro: a cena deixou de poder discriminar, e \
     o roteiro manda o dono comparar dois numeros que vao ler igual. Ou a populacao sobe, ou o \
     passo (4) sai do roteiro"
);

/// **O índice da estrela no `kind` do `source.shape`, derivado do PRÓPRIO enum.**
///
/// ⛔⛔ **E não do rótulo, que é o defeito que esta linha pagou em 2026-09-20:** os `KIND_LABELS`
/// passaram a ser chaves de i18n quando a fronteira dos motores fechou, e uma procura por `"Star"`
/// devolvia `None` — a auditoria inteira mediu um CÍRCULO julgando medir uma estrela. A porta que
/// as outras cenas usam (`sim_demo::indice_de`) já resolve a chave pelo idioma inglês e está
/// curada; **esta cena não passa por rótulo nenhum**, que é a cura mais forte: o `ALL_KINDS` está
/// alinhado ao `KIND_LABELS` **por gate** na crate do nó.
///
/// ⚠️ Ela falha **alto**: uma forma que saia do enum tem de parar a cena, não escolhê-la em
/// silêncio.
fn indice_da_estrela() -> f32 {
    let i = ph2d_node_motion_shape::ALL_KINDS
        .iter()
        .position(|k| *k == ph2d_node_motion_shape::ShapeKind::Star)
        .expect("a `Star` tem de estar no `ALL_KINDS`");
    #[expect(
        clippy::cast_precision_loss,
        reason = "um indice de enum, sempre pequeno"
    )]
    {
        i as f32
    }
}

/// **O `Corner Radius` com que a cena NASCE** — `0` (a estrela pontiaguda) a menos que
/// `PH2D_CARIMBO_CORNER=<f>` diga outra coisa.
///
/// ⛔⛔ **Isto é um INSTRUMENTO DE BISSECÇÃO e não um knob de produto.** Ele existe por causa do
/// report do dono de 2026-09-20 (*«nenhum dos dois tolerou modificar o corner radius das estrelas.
/// travou»*): o gesto que o produz é arrastar um slider, e **um gesto de painel não é alcançável de
/// um teste** — sem esta porta a única forma de medir o quadro que ele viu era eu clicar, que esta
/// casa não faz (o XTest é ignorado na Xwayland virtual e o `ydotool` mexe no rato REAL do dono).
///
/// ⚠️ **Valor ausente ou ilegível ⇒ `0`, e `0` é a cena de sempre AO BIT** — o param é o default do
/// manifesto e a geometria é a mesma estrela de 12 vértices. *Uma porta de bissecção que mude a
/// cena quando ninguém lhe toca deixa de bissectar coisa nenhuma.*
fn corner_semeado() -> f32 {
    corner_por(std::env::var("PH2D_CARIMBO_CORNER").ok().as_deref())
}

/// A LEI da porta acima, **pura** — o que a variável significa, sem a ler.
///
/// ⚠️ Separada por uma razão de instrumento que esta casa já pagou: *um gate que lê o ambiente
/// mede a MÁQUINA em que corre*, e a pergunta *«com que forma a cena nasce?»* é sobre o produto.
/// Aqui ela é função de um `Option<&str>`, logo o gate afirma as células (ausente · número ·
/// lixo · fora de faixa) sem tocar no processo.
fn corner_por(valor: Option<&str>) -> f32 {
    valor
        .and_then(|v| v.trim().parse::<f32>().ok())
        .filter(|v| v.is_finite())
        .map_or(0.0, |v| v.clamp(0.0, 1.0))
}

/// Constrói o documento. `None` se algum tipo de nó não estiver registado.
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    // ⚠️ O registo entra na assinatura porque o roteador o passa a todas as cenas — e aqui ele
    // serve de guarda: uma cena que monte com um nó que não existe cozinha zero e desenha nada.
    for tipo in [
        "motion.grid",
        "motion.duplicator",
        "source.shape",
        "motion.output",
    ] {
        reg.manifests()
            .find(|m| m.id == ph2d_nodegraph::node::NodeTypeId::of(tipo))?;
    }
    let g = &mut doc.graph;
    let no = |g: &mut ph2d_nodegraph::graph::Graph, tipo: &str, x: f32, y: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y });
        n
    };

    // ── AS POSIÇÕES.
    let grade = no(g, "motion.grid", 0.0, 0.0);
    #[expect(
        clippy::cast_precision_loss,
        reason = "um lado de grelha, muito abaixo de 2^24"
    )]
    let lado = lado_semeado() as f32;
    g.set_param(grade, "rows", lado);
    g.set_param(grade, "cols", lado);
    g.set_param(grade, "gap_x", VAO);
    g.set_param(grade, "gap_y", VAO);

    // ── A FORMA: uma estrela, e só uma. É ela que o carimbo prepara uma vez.
    let forma = no(g, "source.shape", 0.0, 220.0);
    g.set_param(
        forma,
        ph2d_node_motion_shape::param::KIND,
        indice_da_estrela(),
    );
    g.set_param(forma, ph2d_node_motion_shape::param::SIZE, TAMANHO);
    g.set_param(
        forma,
        ph2d_node_motion_shape::param::CORNER,
        corner_semeado(),
    );

    // ── O CARIMBO. ⚠️ A forma na porta `0`, os pontos na `1` — a ordem que o manifesto do
    // duplicador declara, e que um censo do roteador confere em toda cena.
    let dup = no(g, "motion.duplicator", 240.0, 110.0);
    for (de, porta) in [(forma, 0u16), (grade, 1)] {
        g.connect(Edge {
            from: (de, 0),
            to: (dup, porta),
            delayed: false,
        })
        .ok()?;
    }
    let saida = no(g, "motion.output", 460.0, 110.0);
    g.connect(Edge {
        from: (dup, 0),
        to: (saida, 0),
        delayed: false,
    })
    .ok()?;
    Some(vec![saida])
}

/// **O roteiro que o dono segue.** ⚠️ Cada passo nomeia o que aparece NA TELA (`CLAUDE.md` §0.8),
/// e o *«deu errado se»* é a metade que só cabe aqui.
///
/// ⛔ **Esta cena não pousa legenda no canvas de propósito:** a legenda desta casa é feita de
/// PARES (uma ficha por metade, em lados opostos — há censo a exigi-lo), e isto é UMA coisa só.
/// O que ela tem para dizer é um número que já está na tela: a barra de baixo.
pub(super) fn announce() {
    let n = ESTRELAS;
    eprintln!(
        "\n[estrelas] UM CAMPO DE {n} ESTRELAS ({LADO_N} x {LADO_N}) — a cena do seu report\n\
         («usando shape star fps cai»). O `Grid` poe as posicoes e o `Duplicator` veste cada\n\
         uma com a MESMA estrela.\n\
         \n\
         (1) Olhe a BARRA DE BAIXO do ecra. Ela diz `fps`, os milissegundos do quadro,\n    \
         e um TERCEIRO numero: o `raw`. E' o `raw` que interessa aqui — os `60 fps`\n    \
         sao o tecto do ecra, e o `raw` e' a FOLGA: quanto MAIOR, mais sobra por\n    \
         quadro para o resto do trabalho. ANOTE o valor dele.\n\
         (2) Aproxime com a roda do rato ate' ver as pontas: sao ESTRELAS, todas iguais.\n    \
         O ecra mostra um pedaco do campo — as {n} existem e sao TODAS desenhadas.\n\
         (3) Arraste o fundo com o botao do meio: tem de passear LISO.\n\
         (4) Feche o app e corra o MESMO comando com `PH2D_CARIMBO_PREPARADO=0` a' frente:\n    \
         e' o caminho ANTIGO. Os `fps` ficam nos mesmos `60` (e' o tecto do ecra) e o\n    \
         `raw` CAI — o quadro passa a gastar cerca de {DIFERENCA_MS:.1} ms a mais de\n    \
         CPU, so' para desenhar as MESMAS {n} estrelas.\n\
         (5) Compare os dois `raw` que anotou. A imagem e' a MESMA, ponto por ponto —\n    \
         so' a folga muda.\n\
         \n\
         (6) AFASTE com a roda ate' o campo INTEIRO caber no ecra. As estrelas ficam\n    \
         com 3 pixeis ou menos, e a esse tamanho o app troca cada desenho pela\n    \
         FOTOGRAFIA dele — medido: a essa distancia as duas sao indistinguiveis\n    \
         (menos de um tom de 255 de diferenca). O `raw` tem de SUBIR, nao cair.\n\
         (7) APROXIME outra vez ate' ver as pontas: elas voltam a ser DESENHO nitido.\n    \
         A troca tem os dois sentidos, e a fronteira e' `4 px` de lado.\n\
         (8) Corra o mesmo comando com `PH2D_LOD_DA_FORMA=0` a' frente e repita o (6):\n    \
         e' o caminho de antes desta cura, sem a troca.\n\
         \n\
         DEU ERRADO se: o campo nao aparecer; se o `raw` for IGUAL nas duas corridas;\n\
         se a imagem for DIFERENTE entre as duas; ou se ao AFASTAR as estrelas\n\
         DESAPARECEREM, ficarem BRANCAS ou PISCAREM — isso e' a troca a falhar,\n\
         e `PH2D_LOD_DA_FORMA=0` confirma-o num comando.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_carimbo_demo_tests.rs"]
mod tests;

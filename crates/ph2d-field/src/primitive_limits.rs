//! ⭐ **QUANTAS PEÇAS UMA FORMA ADMITE** — os pisos e os tetos de contagem das primitivas, cada um
//! com a tabela que o mediu ao lado.
//!
//! # Por que um arquivo irmão
//!
//! O [`super::primitive`] responde *que formas existem*; este responde *quantos lados, pontas,
//! dentes ou pontas de seta cada uma comporta, e a que preço*. A W119 acrescentou seis primitivas e
//! o arquivo passou as `700` linhas do gate de LOC da workspace.
//!
//! ⚠️ **Partir para irmão, nunca uma entrada na allowlist** — é a **sétima** vez que esta casa paga
//! este corte, e sempre pelo mesmo motivo: as tabelas medidas e as recusas escritas ao lado das
//! constantes são o valor, não o peso.

/// O menor número de lados que um prisma admite — abaixo disto não há polígono.
pub const MIN_PRISM_SIDES: u32 = 3;

/// ⭐⭐⭐ **O TETO de lados de um prisma — e a medição REFUTOU a razão que eu ia escrever.**
///
/// # ⚠️ O erro, porque ele é instrutivo
///
/// A primeira redação deste doc dizia, com confiança: *«o custo **não** é o recurso — o preço por
/// ponto mal se mexe com os lados»*, e citava o `spike_formula_vs_profile`, que tinha medido `7,00×`
/// os nós a dar `1,21×` o relógio. ⛔ **É falso aqui.** A sonda
/// [`measure_prism_sides`](../../ph2d-field-eval/tests/measure_prism_sides.rs) mediu:
///
/// | lados | ns/ponto | × o cilindro | desvio da quina |
/// |---|---|---|---|
/// | 3 | 1,62 | **0,92×** | 50,00 % |
/// | 6 | 1,92 | 1,09× | 13,40 % |
/// | 12 | 2,74 | 1,56× | 3,41 % |
/// | 16 | 3,36 | 1,91× | 1,92 % |
/// | 24 | 4,62 | 2,62× | 0,86 % |
/// | **32** | 6,69 | **3,80×** | **0,48 %** |
/// | 64 | 13,11 | 7,43× | 0,12 % |
/// | 96 | 19,27 | 10,93× | 0,05 % |
///
/// ⚠️ **Porque a conclusão anterior não transferia:** ali a árvore era funda e o que custava era o
/// *caminho crítico*, que o SIMD escondia. Aqui as paredes são uma **cadeia de `max`** — o caminho
/// crítico cresce **linearmente** com `n`, e o relógio segue-o. *Uma recusa medida responde UMA
/// pergunta; reconfira-a quando a sua for outra.*
///
/// ⭐ **E o triângulo é MAIS BARATO que o cilindro** (`0,92×`): três planos não têm `sqrt` nenhum, e
/// a secção circular tem um. *A forma «simples» e a forma «barata» não são a mesma lista.*
///
/// # ⭐ O teto é onde as DUAS curvas dizem o mesmo
///
/// Um prisma de muitos lados **é** um cilindro, e este app tem o cilindro **exato e mais barato**. A
/// 32 lados a quina desvia `0,48 %` do raio — sub-pixel em qualquer enquadramento razoável — e
/// paga-se `3,71×` por isso. ⇒ acima de 32 o artista pede um cilindro, não o recebe, e paga a mais.
///
/// ⚠️ *Um limite legítimo diz de que recurso ele é* (CLAUDE.md §0). Este é dos **dois** ao mesmo
/// tempo, e é isso que o torna o sítio certo: a forma deixa de se distinguir exatamente onde o preço
/// começa a doer.
pub const MAX_PRISM_SIDES: u32 = 32;

/// O menor número de pontas de uma estrela — com duas não há ponta nenhuma, há uma lente.
pub const MIN_STAR_POINTS: u32 = 3;

/// ⭐⭐ **O TETO de pontas de uma estrela — e ele NÃO é o do prisma, porque uma ponta custa QUATRO
/// semiplanos.**
///
/// Uma estrela de `n` pontas é o disco dos vales unido a `n` pipas de quatro semiplanos cada —
/// `4n`, contra `n` de um prisma do mesmo número. A sonda
/// [`measure_star_points`](../../ph2d-field-eval/tests/measure_star_points.rs) mediu, com a **mesma
/// régua do prisma** (o cilindro exato = `1,00×`):
///
/// | pontas | semiplanos | nós | ns/ponto | × o cilindro |
/// |---|---|---|---|---|
/// | 3 | 12 | 88 | 2,78 | 1,28× |
/// | 5 | 20 | 137 | 3,50 | 1,61× |
/// | 8 | 32 | 191 | 4,64 | 2,13× |
/// | 12 | 48 | 288 | 6,42 | 2,95× |
/// | **16** | **64** | **357** | **7,96** | **3,66×** |
/// | 24 | 96 | 555 | 11,24 | 5,17× |
/// | 32 | 128 | 750 | 14,52 | 6,68× |
///
/// ⭐ **O número sai de um preço que este módulo já aceitou**, e não de um gosto: o
/// [`MAX_PRISM_SIDES`] shipa a `3,80×` o cilindro. A estrela chega a esse preço às **16** pontas
/// (`3,66×`) e passa-o às 24 (`5,17×`). ⇒ 16.
///
/// ⚠️ **E aqui o teto TIRA alguma coisa, ao contrário do prisma.** Um prisma de 64 lados é um
/// cilindro, e o cilindro exato está na porta ao lado — acima do teto o artista não perde nada. Uma
/// estrela de 24 pontas continua a ser uma estrela de 24 pontas, e não há segunda porta para ela.
/// *Um limite que retira tem de o dizer.*
pub const MAX_STAR_POINTS: u32 = 16;

/// O menor número de dentes que uma engrenagem admite — abaixo de três não há coroa.
pub const MIN_GEAR_TEETH: u32 = 3;

/// ⭐⭐⭐ **O TETO de dentes — MEDIDO, e o número está na tabela ao lado** (W106).
///
/// A sonda é [`measure_gear_teeth`](../../ph2d-field-eval/tests/measure_gear_teeth.rs), e a régua é
/// a mesma que escolheu o [`MAX_STAR_POINTS`]: o preço contra o **cilindro**, que é a referência
/// que o [`MAX_PRISM_SIDES`] usa e shipa a `3,80×`.
///
/// ⚠️ **A coluna que decide é a CONTAGEM DE NÓS**, não o relógio: ela é determinística, e um
/// relógio desta workstation não vale nada acima de `load ~5` (`CLAUDE.md` §5.0). O tempo aparece
/// ao lado como confirmação, pela mediana de cinco corridas.
///
/// | dentes | nós | ns/ponto | × o cilindro | × a ESTRELA no tecto dela |
/// |---|---|---|---|---|
/// | 6 | 160 | 5 489 | 3,37× | 0,40× |
/// | 8 | 192 | 6 718 | 4,13× | 0,50× |
/// | 12 | 300 | 10 021 | 6,16× | 0,74× |
/// | 16 | 390 | 12 324 | 7,57× | 0,91× |
/// | 24 | 586 | 18 526 | 11,38× | 1,37× |
/// | **32** | **741** | **23 473** | **14,42×** | **1,73×** |
/// | 48 | 1 155 | 48 467 | 29,78× | 3,57× |
/// | 64 | 1 482 | 65 407 | 40,20× | 4,82× |
///
/// *(referências medidas na MESMA corrida: cilindro `25` nós · prisma no tecto `308` · **estrela no
/// tecto `423` nós, `8,34×` o cilindro — a forma mais cara que esta casa shipa**.)*
///
/// # ⛔ Não há JOELHO na contagem de nós, e dizê-lo é o resultado
///
/// A contagem é **linear** de ponta a ponta: `26,7 · 24,0 · 25,0 · 24,4 · 24,4 · 23,2 · 24,1 ·
/// 23,2` nós por dente. ⇒ *não existe um número onde a física pare*, e um teto aqui é um **orçamento**
/// e não uma parede. Escrever «o joelho está em N» seria inventar uma medição que a tabela não deu.
///
/// A única não-linearidade é o **relógio** entre 32 e 48: `2,06×` o tempo para `1,5×` os dentes,
/// quando a contagem só sobe `1,56×`. ⚠️ É um sinal fraco (um relógio desta workstation não vale
/// nada acima de `load ~5`), e por isso ele **confirma** o número em vez de o escolher.
///
/// # Por que 32 e não 16
///
/// Aplicar a barra da estrela à letra daria **16** (`0,91×` dela). ⛔ Mas o doc do
/// [`MAX_STAR_POINTS`] escreve a própria regra: *«um limite que RETIRA tem de o dizer»* — e este
/// retira. Uma engrenagem de 24 ou 32 dentes é uma engrenagem comum; a 8 (que é onde ela custa o
/// que a estrela custa) ela mal se lê como uma. ⇒ o teto paga **`1,73×`** a forma mais cara da casa,
/// de propósito, porque *ter dentes é a razão de existir desta forma*.
///
/// ⚠️ **E o que este número mede é um LIMITE SUPERIOR:** o traçador especializa a fita por
/// ladrilho × fatia de profundidade, então um quadro real paga muito menos do que a árvore inteira.
/// Movê-lo pede a medição do **quadro** com uma cena cheia delas — que não foi feita, e é o que
/// desbloqueia um teto maior.
pub const MAX_GEAR_TEETH: u32 = 32;

/// Uma seta tem pelo menos **uma** ponta — sem ela é um retângulo, que a caixa já é.
pub const MIN_ARROW_HEADS: u32 = 1;

/// ⭐ **O TETO de pontas de uma seta é `2`, e ele NÃO é de preço — é de GEOMETRIA.**
///
/// ⚠️ Os outros tetos deste arquivo saem todos de uma tabela de relógio, e este não sai de nenhuma:
/// uma haste é um **segmento**, e um segmento tem duas extremidades. Uma terceira ponta não é uma
/// seta mais cara — é uma forma que a haste não tem onde pôr.
///
/// ⇒ *um limite legítimo diz de que recurso ele é* (CLAUDE.md §0), e o recurso aqui é o número de
/// pontas de um segmento. Uma estrela de três braços é o [`crate::Primitive::Star`].
pub const MAX_ARROW_HEADS: u32 = 2;

/// Uma nuvem tem pelo menos **três** bossas — com duas ela le^-se como uma cápsula.
pub const MIN_CLOUD_LOBES: u32 = 3;

/// ⭐⭐⭐ **O TETO de bossas de uma nuvem — e o recurso NÃO é o preço: é a MARCHA.**
///
/// # ⚠️ A primeira régua respondeu à pergunta errada
///
/// A sonda do **preço** ([`measure_cloud_lobes`](../../ph2d-field-eval/tests/measure_cloud_lobes.rs))
/// diz que `12` bossas custam `3,95×` o cilindro — exactamente a barra que o [`MAX_PRISM_SIDES`] já
/// shipa. ⛔ **E `12` fura a peça**: numa união n-ária o tecto de `‖∇f‖` é `√(quantas peças estão
/// ACTIVAS)`, e acima de `passo × ‖∇f‖ = 1` a marcha de esferas **atravessa a superfície**.
///
/// | bossas | `passo × ‖∇f‖` | | bossas | `passo × ‖∇f‖` |
/// |---:|---:|---|---:|---:|
/// | 3 | `0,7167` | | 8 | **`1,0664`** |
/// | 4 | `0,8653` | | 9 | `1,1155` |
/// | 5 | `0,9274` | | 10 | `1,1396` |
/// | 6 | `0,9454` | | 12 | `1,2483` |
/// | **7** | **`0,9971`** | | 16 | — |
///
/// ⇒ **o joelho está entre `7` e `8`**, e é ele que manda. *Duas réguas responderam, e a que
/// governa é a que diz se a peça sai furada.*
///
/// ⚠️ **E a alavanca não é o raio da mistura** — uma varredura de `0,50` a `0,10` da fracção moveu
/// o número em `0,05`: duas superfícies próximas continuam próximas por mais que se aperte o raio.
/// A alavanca é o **espaçamento** dos discos, que é `2·half_width/n`.
pub const MAX_CLOUD_LOBES: u32 = 7;

/// ⭐ **QUANTO a mistura da nuvem EMPURRA a superfície para fora das bossas** — o factor que a caixa
/// dela tem de conter.
///
/// # A conta, e ela é fechada
///
/// O raio da mistura é `0,35 × a menor bossa`; uma união arredondada empurra a superfície até
/// `r·(√2 − 1) = 0,4142·r`; e a menor bossa nunca passa a `half_width` (o construtor limita-a lá).
/// ⇒ o excesso é no máximo `0,35 × 0,4142 = 0,145` de `half_width`.
///
/// ⚠️ **Ele não é uma folga «por segurança»** — é o inchaço de um operador, com a fórmula ao lado.
/// Medido em 2026-09-05 (report do Enio, *«Span cria formas gigantes»*): com o `Span` a `2,0` a peça
/// lia `0,575` contra os `0,500` que a caixa declarava, e quem lê aquela caixa é o **recorte da
/// marcha**, que corta a peça e não diz nada.
pub const CLOUD_BLEND_SWELL: f32 = 1.145;

// ─────────────────────────── W122 — as cercas do fluxograma ───────────────────────────
//
// ⭐⭐⭐ **NENHUMA das quatro é uma cerca de MARCHA, e isso foi MEDIDO antes de se escrever o
// número** (`probe_w122_flow`, 2026-09-05). A varredura de cada grandeza muito além do que estas
// paredes deixam passar mostra o campo a subir e a **assentar por baixo de `1`**, sem joelho nenhum:
//
// | paralelogramo `k` | 0 | 1 | 2 | 4 | 6 | 10 |
// |---|---:|---:|---:|---:|---:|---:|
// | `passo × ‖∇f‖` | `0,707` | `0,924` | `0,973` | `0,993` | `0,996` | `0,995` |
//
// ⇒ *elas são cercas de IDENTIDADE (onde a forma deixa de ser a que o nome promete) e de ALCANCE
// (onde o slider ainda tem resolução), e ficam escritas como tal.* §0: um limite legítimo diz de
// que recurso ele é — e dizer «marcha» aqui seria inventar um.

/// ⭐ **Até onde o paralelogramo INCLINA** — em múltiplos de `half_span`.
///
/// # Não é a marcha (tabela acima). É a forma e é o alcance.
///
/// `k = skew/half_span` é a **tangente** da inclinação: `k = 1` são `45°`, `k = 2` são `63°`. Acima
/// disso a peça é uma lasca — e a distância entre os dois flancos, que é o que o filete come, já
/// caiu para `1/√(1+k²) = 45 %` da que a peça direita tem.
///
/// ⚠️ **A parede é do DOCUMENTO** ([`crate::Span::Walls`]), então o slider mapeia `±2·half_span` e o
/// uso comum (`0,2`–`0,5`) fica no meio do curso, com resolução. Uma parede folgada poria o
/// intervalo útil nos primeiros por cento — a lição da cauda da nuvem, ao contrário.
pub const MAX_PARALLELOGRAM_SKEW: f32 = 2.0;

/// ⭐ **A razão `half_span / half_width` que o atraso (e o mostrador) aguentam.**
///
/// # ⭐⭐ Aqui a cerca tem uma DEMONSTRAÇÃO, e não só uma medição
///
/// O corpo é a cápsula da recta de `−half_width` a `half_width − half_span`, engrossada em
/// `half_span`. Enquanto `half_span ≤ 2·half_width` a recta anda para a frente e a peça acaba
/// **exactamente** em `+half_width`; em `half_span = 2·half_width` ela degenera num ponto e a forma
/// é um meio-disco, que ainda é um atraso. **Acima disso a recta inverte-se** e a ponta direita passa
/// para `half_span − half_width`, que é maior que a caixa declarada ⇒ *a peça sai da própria caixa,
/// e quem lê aquela caixa é o recorte da marcha, que corta e não diz nada.*
///
/// ⚠️ Medido na cerca: `passo × ‖∇f‖ = 0,814` — e o pior de todo o curso é `0,990`, no extremo
/// **fino** (`s/w = 0,3`), que a parede não toca.
pub const DELAY_SPAN_OVER_WIDTH: f32 = 2.0;

/// ⭐ **Que fracção de `2·half_width − half_span` o bico do mostrador pode ocupar.**
///
/// `1,0` é o bico a acabar **exactamente onde a tampa redonda começa** — que é a definição do
/// símbolo. Além disso os flancos entram na tampa, a peça perde as faces retas e deixa de se ler
/// como um mostrador. ⚠️ **A marcha não tem voto**: medida até `2,5×` a parede, ela fica em `0,992`.
pub const MAX_DISPLAY_POINT: f32 = 1.0;

/// ⭐ **Que fracção da altura o bico do conector de página pode ocupar.**
///
/// `1,0` é o bico com a profundidade da peça inteira — os dois flancos vão dos cantos de cima ao
/// vértice de baixo, e a forma é um triângulo. Além disso o topo cortaria os flancos e o «conector»
/// seria outra coisa. ⚠️ Medida até `3×` a parede, a marcha fica em `0,985`.
pub const MAX_OFFPAGE_POINT: f32 = 1.0;

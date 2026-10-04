# 121 — AS FORMAS NA PLACA: toda forma do Motion desenhada pelo dispositivo, nítida e sem custo por cópia

> **Ordem do dono, 2026-09-29:** *«fechar as sobras do plano dos ciclos. Quanto às estrelas, não só
> as estrelas mas todas as shapes devem ser o mais otimizadas possível.»*
>
> ⚠️ É a sobra `«as ESTRELAS na placa sem perder nitidez»` do
> [handoff de 24/09 §6](handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-09-24.md), alargada pelo dono
> a TODA forma viva. A lei que a ADR-0154 e o doc 116 §4.3 já pagaram continua: **nitidez não se
> troca por velocidade** (o dono recusou assar a forma numa imagem em 2026-09-20).

## §1 — O que custa hoje, e onde (lido, não suposto)

Uma forma viva (`source.shape` · os glifos do `source.text` · um desenho do `source.object`) é uma
[`VectorInstance`](../../crates/ph2d-eval-motion/src/lower.rs) que a shell encoda, **uma cópia de
cada vez**, na cena Vello (`motion_shape_gen::encode` → `ph2d_vec_render::draw_shared_instances`).

| custo | onde | medido |
|---|---|---|
| **o cozimento inteiro na CPU** | a ponte recusa o dispositivo a qualquer grafo com uma fonte vectorial viva, **pelo TIPO do nó** (`graph_has_live_vector_source`) | doc 120 §8.1: a simulação é `~80 %` do tique, e só corre na CPU porque a fronteira a arrasta |
| **o encode, por cópia** | `N` comandos `fill` no Vello (o carimbo preparado já os pôs no chão: `0,017 µs`/cópia) | doc 116 §4.3: `1,76 ms` a `102 400` |
| **a rasterização do Vello** | binning por caminho, `N` caminhos | doc 120 §4.3: no iGPU a estrela é presa pela PLACA (`15,09 ms` a `24 576`) |

⇒ as três escalam com `N`. **Uma imagem já não escala** (doc 120 §8.5: `~1,2 ms` de Motion de
`4 096` a `32 768`), e a forma tem de chegar ao mesmo sítio.

## §2 — A DECISÃO: um passe de formas INSTANCIADO, com a cobertura do próprio Vello

**Cada geometria DISTINTA é preparada uma vez; a placa desenha as `N` cópias numa chamada.** Um quad
por cópia (a caixa local da forma, transformada), e o fragmento calcula **a área exacta do pixel
coberta pela forma** somando a contribuição de cada segmento da geometria — **a mesma conta do
rasterizador fino do Vello** (`vello_shaders/shader/fine.wgsl`, `fill_path`, modo `AaConfig::Area`,
que é o que esta casa usa: `ph2d-render/src/vello_pass.rs`).

- ⭐ **Triagem de licença (§0.9): porta ABERTA.** `vello_shaders` é `Apache-2.0 OR MIT` e já é
  dependência do repo ⇒ **porta-se com atribuição**, sem parede. A paridade com o que o artista vê
  hoje vem **por construção**, não por afinação.
- ⭐ **Por pixel, sem ladrilhos:** fora do ladrilho do Vello a contribuição de um segmento que acaba
  à esquerda do pixel é `dy` inteiro (a mesma fórmula com `xmax < 0` dá `a = 1`), logo o `backdrop` e
  o `y_edge` do ladrilho desaparecem e o pixel é independente dos vizinhos.
- ⭐ **Nitidez em qualquer zoom:** as curvas são aplanadas **por NÍVEIS** (LOD) com tolerância relativa
  ao tamanho da forma; o shader de vértice escolhe, por cópia, o nível mais grosso cujo erro no ECRÃ é
  `≤ 0,25 px` — a tolerância de aplanamento do próprio Vello. ⇒ o erro nunca passa do que o Vello já
  aceita, e uma cópia pequena paga poucos segmentos.
- ⭐ **Custo:** CPU por cópia `0` na rota do dispositivo (as cópias vêm do buffer do cozimento); na
  rota da CPU, uma cópia de `64 B` para a placa (o que a sprite já paga). Placa: `pixels × segmentos`
  — uma estrela de `20 px` são `~400 × 10` avaliações.

### §2.1 — O que o passe NÃO faz (e cai para o caminho de hoje, byte a byte)

| caso | porquê | rota |
|---|---|---|
| desenho com tinta própria (`VecPath::fill` com gradiente/padrão) | a tinta não é uma cor | Vello, como hoje |
| mistura EM GRUPO / camada por cópia (doc 118) | pede uma camada fora do alvo | Vello, como hoje |
| quad de IMAGEM no passe vectorial (a «terceira média») | é uma textura, não uma forma | Vello, como hoje |

⚠️ **E um caso que o passe desenha com lei PRÓPRIA:** traço + afim NÃO conforme (escala
não-uniforme). A lei do dono (bug #27: *«quando engrossa, engrossa por igual nos dois eixos»*) pede
a caneta no MUNDO; o traço expandido no espaço LOCAL daria uma caneta elíptica. ⇒ essas cópias
constroem o traço **no ecrã a partir do eixo** (W4, §9: caneta redonda `w·√|det|`, juntas e pontas
AUTORADAS — ⛔ a redacção de 29/09 dizia *«juntas redondas em vez de esquadria»* e a W4 não o
aceitou). Tudo o resto — o preenchimento, e o traço de toda cópia conforme — é a conta do Vello.
~~⛔ O traço **tracejado** sob afim não conforme continua no Vello (§9).~~ **Superada pelo §9.9.**

## §3 — A ORDEM no quadro não muda

Hoje as formas entram na cena vectorial **depois** do documento e **antes** dos gizmos e do chrome
(`fase_vector_overlays`). O passe desenha-as no MESMO sítio: a casa já compõe o quadro por FAIXAS
(`present_bands`, ADR-0154 Fase 2), e a forma instanciada é uma faixa entre a cena de baixo e a de
cima. ⛔ Desenhar o passe por cima do alvo do Vello pintaria as formas **por cima dos painéis**.

## §4 — As waves

| wave | o quê | prova |
|---|---|---|
| **W0** | a medição de PARTIDA: estrelas, círculos e texto na escada `=17` e na `=126`, nas duas placas, CPU e placa | a tabela, com `loadavg` |
| **W1** | a crate-folha `ph2d-shape-gpu`: geometria (aplanar por níveis, expandir o traço) + o passe WGSL + a cobertura portada | paridade de PIXEL contra o Vello sobre as mesmas cópias, num adaptador real; a barra sai do vale medido |
| **W2** | a rota da CPU: `pump.vector_instances` suportadas vão ao passe, o resto ao Vello | a cena `=126` igual ao pixel (barra da W1) e o relógio |
| **W3** | a rota do DISPOSITIVO: o cozimento emite as cópias de forma num buffer próprio; a ponte deixa de recusar pelo TIPO e passa a perguntar pelo CONTEÚDO | o censo de rota; a escada com a simulação na placa |
| **W4** | o traço sob afim não conforme · os glifos do `source.text` · o que a W0 mostrar | gates de cada caso |
| **W5** | a medição de FECHO e o smoke do dono: formas + simulação com campos, fotografada antes | a tabela contra a W0 |

## §5 — Estado

- ⏳ W0 — por correr (a placa estava ocupada por outras linhas em 2026-09-29).
- ✅ **W1 — a crate [`ph2d-shape-gpu`](../../crates/ph2d-shape-gpu/) existe e desenha o que o Vello
  desenha** (§6).
- ✅ **W2 — a rota da CPU está LIGADA por omissão** (§7): as formas do Motion vão ao passe numa
  camada de meio-float, TUDO-OU-NADA por quadro. `PH2D_FORMAS_NA_PLACA=0` bissecta.
- ✅ **W3 — a rota do DISPOSITIVO está LIGADA** (§8): o cozimento escreve as cópias de forma num
  buffer PRÓPRIO que o passe lê sem descarga; a ponte deixou de recusar pelo TIPO e pergunta pelo
  CONTEÚDO. A mesma porta (`PH2D_FORMAS_NA_PLACA=0`) bissecta as duas rotas.
- ✅ **W4 — o traço sob afim NÃO conforme vai à placa** (§9): o passe constrói o traço de cada
  cópia no ECRÃ a partir do EIXO aplanado, com a caneta redonda da casa e as juntas e pontas
  autoradas. Só o traço TRACEJADO sob escala não-uniforme fica no Vello. Censo: `17` das `22`
  cenas com forma vão à placa. *(Histórico: o §9.9 levou o tracejado à placa; o censo é hoje `19`
  das `23`.)*
  ✅ **Smoke do dono aprovado na `=127`** (§9.2): denso `raw 200` pela placa contra `100` sem ela.
- ✅ **A FAIXA** (§9.4): o traço esticado sem peças de junta — no proxy de telemóvel a `=127` densa
  passa de `30,6` para `25,4 ms` de passe; ⏳ ainda atrás do Vello lá (`30,3` contra `19,7 ms` de
  placa).
- ✅ **O CONTORNO CALCULADO** (§9.5): a geometria do traço esticado sai UMA vez por cópia num passe
  de cálculo — a `=127` densa no proxy de telemóvel `27,8 → 22,3 ms` de placa (RTX `1,57 → 1,16`);
  ⏳ ainda atrás do Vello no proxy (`19,4 ms`). `PH2D_CONTORNO_CALCULADO=0` bissecta.
- ✅ **AS CÉLULAS E O FUNDO** (§9.6): o preenchimento, as marcas e o traço conforme GRANDE também vão
  pelas arestas no ecrã, cada fileira partida em células de `32 px` com o fundo pré-somado — a `=127`
  densa no proxy de telemóvel **`22,3 → 18,76 ms`, À FRENTE do Vello (`19,42`)**; RTX `1,11` (Vello
  `2,03`). ⏳ As estrelas GRANDES ainda perdem no proxy (`2,32` contra `0,98`; conformes `1,36`
  contra `0,90`).
- ✅ **AS LISTAS DAS CÉLULAS** (§9.8): cada célula guarda as arestas que a cruzam (o ladrilho do Vello),
  montadas por ARESTA em ponto fixo — sonda calma no proxy: grandes esticadas `2,04 → 1,74`,
  conformes `1,18 → 0,94` (EMPATA com o Vello, `0,92`), densas `2,13 → 1,71` (Vello `2,81`). ⏳ As
  grandes ESTICADAS ainda perdem (`1,74` contra `1,05`).
- ✅ **O TRACEJADO NO ECRÃ** (§9.9): o traço tracejado sob escala não uniforme vai à placa, pontas e
  emenda incluídas; a `=76` deixa a CPU; censo `19` das `23`. Mutação `17` de `17`. E o ajuste da emenda passou ao ECRÃ nas duas rotas (`042327a6a`), mutação `21` de `21`.
- ✅ **Os glifos do `source.text`** já iam à placa desde a W3; o gate das LETRAS (§9.9) é novo.

## §6 — ✅ W1: a paridade de PIXEL, medida (2026-09-29, RTX, alvo de meio-float)

Gate [`o_passe_desenha_o_que_o_vello_desenha`](../../crates/ph2d-shape-gpu/tests/it/paridade_com_o_vello.rs)
(`#[ignore]`, adaptador real): as MESMAS cópias pelo Vello (`AaConfig::Area`) e pelo passe.

| família | cópias | pixels pintados (Vello = passe) | alfa máx. | cor máx. |
|---|---:|---|---:|---:|
| estrelas pequenas (6–40 px) | 400 | `77 296` = `77 296` | `1` | `3` |
| estrelas grandes (120–900 px) | 12 | `252 617` = `252 617` | `1` | `2` |
| círculos sobrepostos (curvas) | 200 | `262 144` = `262 144` | `2` | `37` |
| círculos ISOLADOS | 64 | `78 210` contra `78 539` | `65` | `0` |
| pentagrama even-odd | 40 | `86 274` = `86 274` | `1` | `2` |
| estrela com traço em esquadria | 40 | `178 639` = `178 639` | `17` | `5` |

⭐ **Nos polígonos as duas rotas pintam os MESMOS pixels e diferem por arredondamento.** ⭐⭐ **Nas
curvas a borda difere até `0,25 px`, e a medição disse de que lado está o erro:** varrida a
tolerância do passe, a borda **afasta-se** do Vello quando o passe fica mais FINO (`1/64 px` → alfa
`77`, `+952` pixels) — o Vello deixa a curva até `0,25 px` para DENTRO (as cordas de um convexo) e o
passe converge para a curva verdadeira. ⇒ o passe shipa a MESMA tolerância do Vello, e a barra `100`
separa todo aplanamento nítido (`≤ 77`) do grosso demais (`1 px` → `163`). ⚠️ O alvo de 8 bits
dobra o desvio de cor nas sobreposições semitransparentes (`6` contra `3`): cada mistura por
hardware arredonda a um byte, e o alvo de meio-float é o do produto.

⚠️ **O Vello grava a cor SEPARADA do alfa** (`fine.wgsl`: `rgba_sep = fg.rgb / fg.a`) e o passe
grava-a pré-multiplicada ⇒ **o passe NÃO pode pintar por cima do alvo do Vello** (a mistura por
hardware de uma cor separada não é «por cima»). Na integração as formas vão numa CAMADA própria,
composta — que é também o que o §3 pede pela ordem.

⛔ **Um defeito apanhado na 1.ª corrida: `0 px` pintados.** O buffer dos handles tem mínimo de 16
bytes e era enchido com ZEROS, o que deixava a lista `[7, 0, 0, 0]` desordenada — e a busca binária
do shader nunca achava o `7`. Enche-se com `u32::MAX`.

**Mutação 5 de 5** (o nível grosso demais · a regra even-odd ignorada · o preenchimento por cima do
traço · o quad a cortar a borda · a rotação transposta). ⚠️ **E uma 1.ª mutação SOBREVIVEU e apagou
código:** a margem de um pixel à volta do quad não era lei — o `floor`/`ceil` da caixa dos segmentos
já inclui todo pixel com cobertura —, e ela saiu.

## §7 — ✅ W2: a rota da CPU, e o defeito da W1 que só o PRODUTO mostrou (2026-09-29)

**O que liga:** `ph2d_vec_render::forma_para_a_placa` (a forma pelos MESMOS passos do desenho de
hoje) → `ph2d_app_motion::motion_shape_placa::PlacaDeFormas` (decisão, cache por handle, camada,
passe) → `ph2d_render::BandSource::Formas` (`(0, 1)`: cores do desenhista, pré-multiplicada) → a
shell (`+~30` LOC): com a placa armada **o documento vai às FAIXAS** (senão ficaria na cena Vello,
POR CIMA das formas) e o presente cola a camada entre as faixas de cima e a cena do chrome.

⚠️ **TUDO-OU-NADA, porque a ordem das linhas é o desenho:** uma imagem, uma mistura (de linha ou de
grupo), uma tinta própria ou um traço **tracejado** sob afim NÃO conforme devolvem o quadro inteiro
ao Vello — partir a lista trocaria a ordem entre as duas metades. (Até à W4 era *todo* traço sob
afim não conforme; desde a §9 só o tracejado.)

⭐⭐⭐ **O gate de paridade do PRODUTO** (`motion_shape_placa::gpu_tests`, `#[ignore]`): o `encode` e
o `VelloPass` do produto contra a placa, `240` cópias das seis formas de fábrica (com traço,
rodadas, translúcidas, câmara com o espelho do Y). ⛔⛔ **Ele reprovou à primeira com riscos
horizontais** — a geometria da W1 saltava os segmentos HORIZONTAIS no espaço LOCAL (*«a
contribuição deles é `dy = 0`, exacta»*), verdade só no referencial do ECRÃ: numa cópia rodada a
aresta de cima de um rectângulo arredondado atravessa linhas de pixel, e sem ela o contorno fica
aberto. **O gate da W1 não o via porque nenhuma das suas formas tinha uma aresta horizontal.**
Curado (`a != b`) com gate que mede a área DEPOIS de rodar a geometria (mutação: vermelho).

| estado | alfa máx. | cor máx. | pixels `> 16` | área (vello / placa) |
|---|---:|---:|---:|---|
| **a rota que shipa** | `66` | `63` | `3 790` (`2,0 %`) | `191 599` / `191 859` |
| a aresta horizontal saltada | `254` | `237` | `38 929` (`20,3 %`) | `191 599` / `187 861` |

No app (fotografado em tela virtual, cena `=126`, `32 761` estrelas): a mesma tinta nas duas rotas
(brilho médio `0,5282` / `0,5281`, desvio igual); o contador lê `109` contra `83` quadros por
segundo em bruto a favor da placa — ⚠️ **preliminar** (perfil `smoke`, máquina a `load 8`–`18`), a
medição a sério é a W5.

⏳ **Por fazer (em 29/09, antes da W3):** W3 (a rota do dispositivo — hoje um grafo com forma viva
continua a cozer na CPU) · W4 (o traço sob afim não conforme, os glifos) · W5 (a medição de fecho e
o smoke do dono). ✅ A W3 fechou — §8.

## §8 — ✅ W3: a rota do DISPOSITIVO (2026-09-29)

**O que liga, de ponta a ponta:**

1. **O baixamento das formas** ([`lower_forma`](../../crates/ph2d-gpu-cook/src/lower_forma.rs)) —
   gémeo do `lower` das sprites. Cada linha de uma saída que carrega `geometry_id` vira uma cópia
   `ShapeInstance` de `16` palavras (`pos` 0-1 · `size` 2-3 · `basis` 4-7 · `anchor` 8-9 ·
   `geometry` 10 · almofada 11 · `tint` 12-15), com **as contas da CPU uma a uma**
   (`lower_vector_onto`, braço `RowMedium::Shape`): `size` ausente é `[1, 1]` (não o
   `default_size`), `basis` do `rot` em graus, `anchor = pivô × size` com o pivô zero CRAVADO, e a
   linha é forma quando `geometry_id > 0,5`; a que não é leva `0xffffffff`, que o passe não acha.
2. **O buffer** ([`formas`](../../crates/ph2d-gpu-cook/src/formas.rs)): `STORAGE | COPY_SRC`,
   reservado UMA vez por quadro com o total de todas as saídas (cresce a dobrar, nunca encolhe),
   cada saída escreve a partir de `primeiro`. `GpuCook::formas()` é `None` num quadro sem formas.
3. **A sprite CALA a linha de forma**: o baixamento irmão zera as palavras `2`, `3`, `17`, `18` e
   `35` (tamanho, âncora e opacidade) de toda linha com `geometry_id > 0,5` — senão ela seria
   desenhada duas vezes, como quad e como forma.
4. **A cerca da ponte é de CONTEÚDO** e corre ANTES do plano
   ([`forma::formas_para_a_placa`](../../crates/ph2d-app-motion/src/motion_bridge_gpu_forma.rs),
   pura, com `GeometriasDaPlaca::veredito` por handle): recusa com o nome no
   `PH2D_MOTION_ROUTE_LOG` a placa desligada · o `fx.glow` (o halo lê as cópias da CPU) · um
   colisor declarado pela forma e lido pelo grafo (o contacto só existe na CPU, doc 109) · uma
   forma que só o Vello desenha (tinta própria, traço de padrão ou pincel) · e, até à W4, **um
   traço** — hoje só um traço **tracejado** (`RECUSA_FORMA_TRACEJADA`, §9).
   ⛔ Era a cerca do TIPO (`graph_has_live_vector_source`) e a da instância condicional, que
   recusavam SEMPRE — foram apagadas.
5. **A mistura POR LINHA** ([`forma::saida_com_mistura_em_formas`]): uma saída com formas e com a
   coluna `blend` pede uma camada que o passe não tem. O cozimento recusa-a
   (`GpuCookError::FormaComMistura`) e a ponte **detecta-a antes do plano** por duas metades —
   a memória da CPU (a última saída cozida) e a bandeira `formas_pedem_o_vello`, que cobre o 1.º
   quadro e é CONSUMIDA. ⛔ Uma recusa DENTRO do cozimento, na rota híbrida, deixava o quadro sem
   desenho nenhum.
6. **O quadro lê o buffer só com o cozimento a andar** (`.filter(|_| motion.gpu_live)` nas duas
   fases da shell): `PlacaDeFormas::decide_do_dispositivo` limpa `pump.vector_instances`, prepara
   as geometrias vivas que a ponte anotou (`formas_no_dispositivo`) e o `desenha` liga o buffer do
   cozimento como `Copias { buffer, count }`.

**Medido na cena `=126`** (`32 761` estrelas, fotografada em tela virtual, perfil `smoke`,
`load ~2`; ⚠️ preliminar — a medição a sério é a W5, em `release`):

| corrida | rota (`PH2D_MOTION_ROUTE_LOG`) | `raw` |
|---|---|---:|
| omissão | `device: HIBRIDO` · `[formas] 32761 copias … pela PLACA (do dispositivo)` | `227` |
| `PH2D_CARIMBO_PREPARADO=0` | a mesma | `224` |
| `PH2D_LOD_DA_FORMA=0` | a mesma | `205` |
| `PH2D_FORMAS_NA_PLACA=0` | `CPU: formas vivas com a placa de formas desligada` | `88` |

⛔⛔ **As duas portas do meio deixaram de tocar na rota de omissão** — o carimbo preparado e a troca
por fotografia (`LOD`) são da rota da CPU. **O roteiro da `=126` ensinava a compará-las** (o passo
(4) e os (6)–(8)): foi reescrito em volta de `PH2D_FORMAS_NA_PLACA=0`, com gate
(`o_passo_quatro_compara_com_a_porta_que_ainda_muda_a_rota`) — *um roteiro que manda comparar duas
corridas iguais ensina que a cura não faz nada*. ⚠️ E a sonda do relógio das fontes contava
`vectores = 0` sempre que a placa respondia; hoje conta as cópias do dispositivo.

**Divergências DECLARADAS:**
- **A mistura julga-se pela PRESENÇA da coluna** e a CPU pelo VALOR — ler o valor custaria uma
  descarga por quadro; a divergência cai para o lado conservador (uma coluna toda a zero manda o
  quadro à CPU, que o desenha certo).
- **Sob o vidro jateado** (a edição de um prefab) a rota do dispositivo continua a desenhar as
  formas, por baixo do vidro, como o resto do mundo.
- ~~**O traço fica na CPU** até à W4 (`RECUSA_FORMA_COM_TRACO`).~~ **Superada pela W4** (§9): a
  recusa passou a ser só do traço tracejado (`RECUSA_FORMA_TRACEJADA`).
- ⚠️ **Pré-existente:** a camada do `fx.glow` num quadro do dispositivo lê o `pump` do quadro
  anterior — inalcançável com formas, porque o glow com formas recusa a placa.

**Gates:** `as_palavras_da_forma_sao_as_do_shape_instance` (as palavras contra o `#[repr(C)]`) ·
`o_dispositivo_baixa_as_formas_como_a_cpu` e `a_mistura_por_linha_de_uma_forma_recusa_o_quadro`
(`#[ignore]`, RTX) · `as_formas_pela_rota_do_dispositivo_desenham_o_que_a_cpu_desenha` (paridade de
PIXEL do produto, `#[ignore]`) · a cerca (`a_cerca_das_formas_nomeia_cada_recusa`) · a mistura
(`a_mistura_por_linha_das_formas_manda_o_quadro_a_cpu`, com o CONTROLO e a bandeira consumida) ·
a validação WGSL (`32` subconjuntos × `2` pivôs) · e na shell a ordem das recusas
(`the_recusals_run_in_the_right_place_relative_to_the_plan`) e a fiação
(`a_rota_do_dispositivo_chega_ao_quadro`).

**A paridade de PIXEL do produto** (`as_formas_pela_rota_do_dispositivo_desenham_o_que_a_cpu_desenha`,
RTX, `36` cópias): as mesmas formas pela rota da CPU (W2) e pela do dispositivo — pixels pintados
`45 172` = `45 172`, **alfa máx. `1`, cor máx. `0`**, pixels acima de `16`: `0` (barras `4`/`4`/`0`).
E o baixamento, palavra a palavra, contra o da CPU: `4` formas iguais.

**Mutação `11` de `11`** (controlos: pré-voo das âncoras, as três corridas LIMPAS verdes com
`9`/`18`/`1` testes, restauro com `touch`):

| # | mutação | quem sangra |
|---|---|---|
| M1 | a sprite deixa de calar a linha de forma | `a_sprite_cala_a_linha_de_forma` + a paridade do baixamento |
| M2 | o limiar `geometry_id > 0,5` deixa passar toda linha | `a_linha_e_forma_acima_de_meio` + paridade |
| M3 | a âncora deixa de multiplicar pelo `size` | `o_pivo_zero_crava_a_ancora` + paridade |
| M4–M6 | o traço · o glow · o colisor lido deixam de recusar | `a_cerca_das_formas_nomeia_cada_recusa` |
| M7 | a bandeira da mistura nunca se gasta | `a_mistura_por_linha_das_formas_manda_o_quadro_a_cpu` |
| M8 | a memória da mistura nunca acusa | a mesma |
| M9 | a placa esquece o buffer do dispositivo | a paridade de PIXEL do produto |
| M10 | o passo (4) da `=126` volta à porta morta | `o_passo_quatro_compara_com_a_porta_que_ainda_muda_a_rota` |
| M11 | o quadro lê o buffer com o cozimento parado | `a_rota_do_dispositivo_chega_ao_quadro` |

⛔⛔ **E um gate de GPU do doc 119 tinha a premissa MORTA:** o
`as_cenas_de_varias_saidas_pela_placa_dao_o_que_a_cpu_da` tratava toda `VectorInstance` da CPU como
defeito (antes da W3 uma cena com forma nunca chegava à placa) e reprovou **14** cenas com `Δ = 0`.
Hoje ele compara o que cada rota DESENHA: os quads que cobrem pixels (a linha de forma calada tem
tamanho zero, e sai das duas contas) e, à parte, as cópias de forma do dispositivo contra as
`VectorInstance` da CPU — `73` cenas julgadas, `21 171` posições e **`361` formas**, com piso nas
duas metades. Prova: deslocar o `x` da cópia de forma em `1` reprova as mesmas `14` cenas.

⚠️ **A mistura na ponte não tinha gate nenhum** até a prova de mutação a procurar — o gate do
cozimento (`a_mistura_por_linha_de_uma_forma_recusa_o_quadro`) prova a recusa DENTRO dele, e a
detecção ANTES do plano, que é a que impede o quadro sem desenho, só existia como código.

## §9 — ✅ W4: o traço sob afim NÃO conforme (2026-09-30)

**A lei é a da casa, e já estava escrita:** `ph2d_vec_render::stroke_uniform::pen_for` — a geometria
transforma-se, a caneta fica REDONDA com largura `w·√|det|`, os traços do tracejado escalam por `k`,
e as juntas e pontas são as AUTORADAS. O passe passa a cumpri-la no dispositivo em vez de devolver o
quadro ao Vello.

**O que liga:**

1. **O eixo** ([`eixo.rs`](../../crates/ph2d-shape-gpu/src/eixo.rs)): a geometria de cada forma com
   traço leva, ao lado do contorno pré-expandido, o **EIXO aplanado** — `EixoItem` `#[repr(C)]` de
   `48 B` por segmento, com a marca de **quina** (`ponto < 0 || cruz² > 1e-6·n`) e o intervalo das
   juntas de um laço fechado (a última junta interior entra; ⛔ a 1.ª redacção saltava-a).
2. **O passe** ([`shape.wgsl`](../../crates/ph2d-shape-gpu/src/shape.wgsl), `traco_do_eixo`): por
   cópia NÃO conforme com eixo, o traço é construído no **ecrã**: um quad por segmento, a junta do
   estilo em cada quina real (esquadria se `2 ≤ (1 + dot)·ml²`, senão chanfro; redonda por leque com
   flecha de `0,25 px`, a tolerância do Vello), **juntas redondas dentro das curvas** (o aplanamento
   não é quina), pontas `Butt`/`Square`/`Round`, e cobertura não-nula com orientação positiva.
   ⛔ **A cópia CONFORME continua no contorno pré-expandido**, que é o do Vello **ao bit** — o passe
   não troca uma lei exacta por uma construída.
3. **A cerca é do TRACEJADO:** a geometria carrega `FLAG_SO_CONFORME` quando o traço é tracejado;
   a rota da CPU (`Entrada::Pronta { so_conforme }`) salta para o Vello **só** com esse traço sob
   afim não conforme, e a do dispositivo recusa-o antes do plano (`RECUSA_FORMA_TRACEJADA`, que
   substitui a `RECUSA_FORMA_COM_TRACO` da W3).
4. **O buffer do eixo** é o *binding* `5` do passe, com o início de cada geometria rebaseado
   (`e[0] += base_eixo`); ⛔ vazio ele pintava **zero pixels** (um buffer de armazenamento vazio não
   se liga) ⇒ leva um `EixoItem` de enchimento.

**Medido** (RTX, alvo de meio-float, `#[ignore]`):
- `a_rota_da_placa_desenha_o_traco_esticado_como_a_casa` — `240` cópias das seis formas de fábrica
  com o aspecto entre `0,35` e `2,8` (controlo: `> 200` NÃO conformes): **alfa máx. `74` · cor máx.
  `92` · `2 607` px acima de `16`** sobre `224 342` pintados pelo Vello contra `224 483` pela placa
  (área a `0,06 %`). Barras: `100`/`100`/`5 %`/`1 %` — as das curvas da W1.
- Na crate, a paridade contra o Vello ganhou as **cópias esticadas** e um **zigue-zague** (quinas
  de esquadria e de chanfro na mesma linha).

**Censo de rota** (`motion_bridge_gpu_rota_das_formas_probe`, `#[ignore]`, as cenas do catálogo com
forma): **`17` de `22` vão à PLACA**. As `5` que ficam, cada uma com a recusa nomeada: `fx.glow`
(`=70`) · passagem (`=120`) · colisor lido pelo grafo (`=114`, `=115`) · traço tracejado (`=76`).

**Divergências DECLARADAS:**
- ~~**Tracejado sob escala não-uniforme fica no Vello** (CPU e dispositivo) — o tracejado no
  dispositivo pede o comprimento de arco por cópia, que é wave própria.~~ **Curada no §9.9.**
- **O teste de conformidade** do passe é em `f32` a `1e-5` e o da CPU a `1e-4`: uma cópia entre os
  dois cai no contorno pré-expandido numa rota e no eixo na outra — as duas desenham a mesma caneta
  redonda, e a diferença é a de arredondamento.
- **As marcas sob afim não conforme** somam a cobertura do contorno e a do eixo com `min(…, 1)`.
- **Nas curvas** o resíduo é o da família da W1 (a borda até `0,25 px`, do lado do Vello).

**Mutação `15` de `15`** (controlos: pré-voo `15/15` âncoras, corridas LIMPAS verdes com `13` e `16`
testes, restauro com `touch`):

| # | mutação | sangra (`P` = `o_passe_desenha_o_que_o_vello_desenha`, na crate) |
|---|---|---|
| W1 | o eixo nunca arma (a caneta elíptica) | `P` |
| W1b | o mesmo, pelo gate do PRODUTO | `a_rota_da_placa_desenha_o_traco_esticado_como_a_casa` |
| W2 | esquadria vira chanfro | `P` |
| W3 | redonda vira chanfro | `P` |
| W4 | a ponta quadrada some | `P` |
| W5 | a ponta redonda some | `P` |
| W6 | a caneta passa a ser o maior eixo e não `√|det|` | `P` |
| W7 | nenhuma fronteira é quina | `P` + dois gates do `eixo` (laço aberto e fechado) |
| W8 | toda fronteira é quina | `dentro_de_uma_curva_a_junta_e_redonda` |
| W9 | o quad não cresce pela caneta | `P` |
| W10 | o início do eixo não é rebaseado por geometria | o gate do PRODUTO |
| W11 | o eixo vazio sem enchimento | `P` |
| W12 | o tracejado não se marca | `o_tracejado_so_se_desenha_conforme` |
| W13 | a placa esquece o tracejado | `so_o_traco_tracejado_sob_escala_nao_uniforme_fica_no_vello` + a cerca |
| W14 | o dispositivo aceita o tracejado | `a_cerca_das_formas_nomeia_cada_recusa` |

⚠️ **Flake de carga, não defeito:** `as_cenas_de_varias_saidas_pela_placa_dao_o_que_a_cpu_da`
reprovou (com um *timeout* ao lado) numa bateria de GPU a `load ~23,3`, e passou **2 de 2 sozinho
a `load 2,3`–`4,5`**; a suíte sem GPU correu `1 476/1 476` a `load 8,15`.

### §9.1 — A cena `=127`: as estrelas esticadas com contorno

⛔ **Nenhuma cena do catálogo continha o fenómeno da W4** (o censo acima: traço + escala
não-uniforme em zero cenas; a `=126` tem as estrelas sem contorno e com a escala igual nos dois
eixos, logo desenha exactamente o mesmo antes e depois). ⇒ a cena nova
([`motion_state_traco_esticado_demo`](../../crates/ph2d-app-motion/src/motion_state_traco_esticado_demo.rs)):
`32 × 32` estrelas amarelas com contorno azul, `motion.scale` com os eixos separados
(`1,8 × 0,6`) **depois** do carimbo, e a galáxia da `=126` **reutilizada** (a `simulacao` passou a
`pub(super)`: duas cópias da lei divergiriam).

⛔ **A 1.ª redacção foi desmentida pela FOTO:** `48 × 48` estrelas de `16 px` com o vão de cada
eixo igual à pegada dele — a galáxia RODA as posições e o esticão fica sempre na horizontal, logo as
fileiras viravam diagonais e as estrelas FUNDIAM-SE numa tira em que o contorno engolia os braços.
Hoje o vão é o do eixo comprido nos dois sentidos e a estrela tem `22 px` com contorno de `1,5 px`.

**Fotografada** (tela virtual `1930×1040`, perfil `smoke`, `load ~15` ⇒ preliminar):

| corrida | rota (`PH2D_MOTION_ROUTE_LOG`) | barra |
|---|---|---|
| omissão | `device: HIBRIDO` · `[formas] 1024 copias … pela PLACA (do dispositivo)` | `59 fps · 16.7 ms · 176 raw` |
| `PH2D_FORMAS_NA_PLACA=0` | `CPU: formas vivas com a placa de formas desligada` | `60 fps · 16.6 ms · 132 raw` |

O contorno ampliado é o MESMO nas duas (caneta redonda, a mesma grossura nos lados compridos e nos
curtos, pontas sem buracos). ⚠️ Uma diferença de pixéis entre as duas fotos **não** mede paridade:
a galáxia anda, e as duas corridas são fotografadas em instantes diferentes (medido: `7 %` dos
pixéis a mais de `16`, todos de posição) — a paridade é do gate de GPU.

**Gates** (`motion_state_traco_esticado_demo_tests`): a cena contém o fenómeno (contorno ·
esticão com os eixos SEPARADOS e diferentes · sem tracejado · simulação nos pontos do carimbo) · o
canto do campo cabe no núcleo da galáxia · o `=127` monta-a e o tecto alcança-a · o passo (4)
compara com a porta que a placa lê e o roteiro manda ler o `raw`.

⛔⛔ **E o passo do relógio estava FALSO nesta cena, medido depois do smoke:** com o arranjo
LEGÍVEL (`1 024` estrelas de `22 px`, umas `300` à vista) as duas rotas leem o mesmo `raw` —
placa `186`/`209`, Vello `223`/`201`, fotografadas seguidas —, e o roteiro prometia *«o `raw`
cai»*. *Um contorno julga-se numa estrela grande e uma folga só se mexe com milhares à vista*
(a aritmética do cabeçalho da `=126`). ⇒ a cena tem dois arranjos (`Arranjo`): o LEGÍVEL de
omissão, para o contorno, e o DENSO (`PH2D_TRACO_ESTICADO_DENSO=1`, `128 × 128` de `8 px`), para o
relógio — com a galáxia DERIVADA do campo (`simulacao_com`: o núcleo cobre o canto, o ímã sai do
núcleo pela lei da `=126`, e com o campo dela a derivação devolve os números dela, gate).

| arranjo denso, `16 384` estrelas | rota | barra |
|---|---|---|
| corrida 1 | `[formas] … pela PLACA (do dispositivo)` | `59 fps · 124 raw` |
| corrida 1, `PH2D_FORMAS_NA_PLACA=0` | `pela cena Vello` | **`44 fps · 22.3 ms · 50 raw`** |
| corrida 2 | placa | `59 fps · 232 raw` |
| corrida 2, `PH2D_FORMAS_NA_PLACA=0` | Vello | `60 fps · 89 raw` |

### §9.2 — O smoke da `=127` devolveu DOIS defeitos, e os dois eram do passe (2026-09-30)

*«no primeiro artefatos de imagem: veja linha nas estrelas»* e *«com `PH2D_FORMAS_NA_PLACA=0` o
`raw` está quase sempre maior»* — com a foto de estrelas de 8 pontas com cantos arredondados, sem
rotação e ampliadas.

**(a) A LINHA era um `NaN`.** A cobertura portada do Vello (`contribuicao`) divide por
`xmax − xmin` e conta com um `−1e-6` para nunca dar zero — verdade no Vello, cujas coordenadas são
relativas a um ladrilho de `16 px`; falso no passe, onde são relativas ao PIXEL e chegam a
centenas. Numa aresta **exactamente vertical**, longe à esquerda do pixel, o `−1e-6` perde-se no
`f32`, a conta vira `0/0`, e o `min(abs(NaN), 1)` da placa devolvia `1`: a fileira inteira pintada
à direita da ponta até ao fim do quad. ⚠️ **A aresta vertical só existe com a forma ALINHADA aos
eixos** (a junta redonda da ponta tem os dois lados espelhados ao bit) — e **nenhum gate de
paridade tinha uma cópia alinhada**: todos sorteiam o ângulo. É a mesma família do defeito da W2
(a aresta horizontal saltada), pelo outro eixo. Cura: um segmento todo à esquerda do pixel devolve
a faixa inteira (`dy`) antes da divisão. ⛔⛔ **E o PREENCHIMENTO tinha o mesmo defeito desde a W1**
— numa forma CÔNCAVA alinhada aos eixos, os pixéis à direita de uma aresta vertical e FORA da forma
pintavam-se (gate `uma_aresta_vertical_longe_do_pixel_nao_vira_nan`, uma cruz sem rotação: **`137 636`
px pintados contra `96 872` do Vello** sem a guarda, alfa `1` com ela). ⚠️ **Quem o disse foi uma
mutação SOBREVIVENTE:** depois da cura (b) o traço salta as peças que não tocam no pixel, logo o
`NaN` deixou de ser avaliado ali e o gate da estrela alinhada ficou VERDE sem a guarda — *a cura do
relógio escondia a do defeito*, e só o preenchimento, que não salta nada, ainda a expunha.
Gate `a_forma_alinhada_aos_eixos_nao_risca_uma_linha`: antes **alfa `255` · cor `235` · `1 934` px
fora**, depois **alfa `65` · cor `68` · `685` px** sobre `23 144`.

**(b) O RELÓGIO: cada pixel lia TODAS as peças do traço.** Medido com a sonda
`sonda_relogio_das_estrelas_grandes` (`72` estrelas de `115 × 38 px`, RTX, `--release`, o Vello COM
a leitura de volta):

| estado | só preenchimento | traço conforme | traço esticado | Vello |
|---|---:|---:|---:|---:|
| a W4 que shipou | `0,16` | `0,31` | **`1,34 ms`** | `0,37` |
| salto por peça, caixa no ecrã | | | `1,02` | |
| salto por peça, caixa LOCAL | | | `1,09` | |
| blocos de 16, caixa LOCAL | | | `0,99` | |
| **blocos de 8, caixa no ECRÃ, alcance por peça** | `0,16` | `0,30` | **`0,40 ms`** | `0,38` |

⭐ Cada peça do traço é FECHADA, logo uma que não toca no pixel soma **zero** e pode saltar-se. As
peças vão em blocos (`ITEM_BLOCO`, um cabeçalho com a caixa local e o maior alcance das peças que se
seguem, `PECAS_POR_BLOCO = 8` pela varredura `4 → 0,48 · 8 → 0,40 · 16 → 0,43 · 32 → 0,53`).
⛔⛔ **O teste no espaço LOCAL foi construído, medido e RECUSADO:** para lá o alcance tem de ir pela
PIOR direcção do afim, e sob escala `1,8 × 0,6` isso engolia meia estrela (`1,09 → 0,99`, quase
nada). No ecrã a caixa é a verdadeira. ⚠️ E o alcance é **por peça**: a esquadria só numa quina em
esquadria, `1,5` numa ponta quadrada, `1` no resto — a 1.ª redacção dava a folga da esquadria a
todas. A ablação que o decidiu: com o traço desligado o quadro lia `0,21 ms`, e o laço de salto
SOZINHO (sem desenhar peça nenhuma) já custava `0,69`.

Gate `os_blocos_guardam_as_pecas_e_cobrem_o_alcance_delas` (as peças atravessam os blocos todas e
pela ordem; a caixa e o alcance de cada cabeçalho cobrem as peças dele, com o controlo de um bloco
cujo alcance é o da esquadria). A paridade das cinco suítes de GPU fica verde.

**Mutação `7` de `7`** (pré-voo `7/7`, corridas LIMPAS verdes com `15` e `4` testes, restauro com
`touch`): a guarda do `NaN` apagada → a cruz alinhada · a caixa do bloco com um canto só · a
esquadria sem alcance (no shader e no cabeçalho) · o bloco sem a caneta · o bloco a contar uma peça
a menos · a peça sem alcance — todas pela paridade do passe, e as duas do cabeçalho também pelo
gate dos blocos. ⚠️ A 1.ª corrida deu `6 de 7`: a guarda sobrevivia ao gate da estrela alinhada
pela razão descrita em (a), e o gate da cruz é o que a matou.

✅ **Smoke do dono APROVADO (2026-09-30), com os números dele:** no arranjo denso `raw 200` pela
placa contra **`raw 100`** com `PH2D_FORMAS_NA_PLACA=0` — *o dobro* —, e *«imagem ok»* (a linha
das estrelas alinhadas não voltou). Bate com as minhas fotos (`124`/`232` contra `50`/`89`).

⏳ **Por fazer:** W0 (a medição de partida, que a placa ocupada adiou) e W5 (medição de fecho em
`release` + smoke do dono com formas e simulação com campos, fotografado antes) · os glifos do
`source.text` · o tracejado no dispositivo.

### §9.3 — A W5 mediu a escada e achou o PROXY DE TELEMÓVEL a perder (2026-09-30)

A medição de fecho correu com [`mede_formas_na_placa.sh`](ferramentas/mede_formas_na_placa.sh): a
MESMA build `release`, com e sem `PH2D_FORMAS_NA_PLACA`, numa tela virtual, cada célula só com
`load < 4` (as três últimas janelas `[frame]` de 120 quadros). ⭐ **O A/B na mesma build É a W0:** a
porta devolve as duas rotas ao caminho de antes da W1 byte a byte, logo a coluna «sem» é a partida.

| placa | cena | objectos | quadro sem → com | CPU (encode) sem → com | Motion sem → com |
|---|---|---:|---:|---:|---:|
| RTX | escada, estrela | 4 096 | 16,65 → 16,65 | 4,61 → **3,69** | 1,35 → **0,94** |
| RTX | escada, estrela | 16 384 | 16,68 → 16,65 | 6,08 → **3,30** | 2,55 → **0,82** |
| RTX | escada, estrela | 32 768 | 16,67 → 16,63 | 9,49 → **2,88** | 4,32 → **0,64** |
| RTX | `=127` densa | 16 384 | 16,67 → 16,67 | 6,37 → **3,24** | 1,77 → **0,80** |
| iGPU | escada, estrela | 16 384 | 16,67 → 16,67 | 6,07 → **3,03** | 2,53 → **0,61** |
| iGPU | escada, estrela | 32 768 | 18,34 → 16,68 | 14,28 → **2,71** | 6,46 → **0,56** |
| iGPU | `=127` densa | 16 384 | **20,66 → 35,20** | 8,18 → 3,70 | 2,61 → 0,85 |

⛔⛔ **A última linha é o achado: no proxy de telemóvel a cena densa ESTICADA ficou mais LENTA.** A
CPU caiu para metade e o quadro subiu de `20,7` para `35,2 ms` — a decomposição do perfilador diz
onde: `acquire(medido)` `12,1 → 31,3 ms`, ou seja a PLACA. ⚠️ (As células da escada a `4 096` na
iGPU e a de `32 768` sem placa leram `load` a subir para `10`–`14` durante a corrida e ficam fora
da tabela como sujas.)

**A sonda isolou-o sem o app** (`sonda_relogio_das_estrelas_grandes`, agora com
`PH2D_SONDA_DENSO=1` — o arranjo denso da `=127` num alvo de `512²`), e a pergunta seguinte
alargou-o: **com estrelas GRANDES, na iGPU, até o preenchimento perdia.**

| iGPU, ms (placa · Vello) | antes | blocos de segmentos | + leque sem trig. |
|---|---|---|---|
| 72 grandes, só preenchimento | `2,90` · `0,58` | `0,73` · `0,52` | `0,73` · `0,54` |
| 72 grandes, contorno conforme | `6,31` · `0,99` | `1,67` · `0,91` | `1,70` · `0,90` |
| 72 grandes, contorno esticado | `7,41` · `1,11` | `4,70` · `0,99` | idem |
| 324 pequenas, contorno esticado | `2,51` · `2,16` | `2,21` · `2,18` | `2,08` · `2,25` |

⭐ **A causa é a do §2: `pixels × segmentos`.** Numa forma pequena são poucos segmentos; numa
grande são centenas em milhares de pixels, e o Vello não paga isso porque corta a forma em
LADRILHOS. ⇒ **os blocos de segmentos** ([`blocos.rs`](../../crates/ph2d-shape-gpu/src/blocos.rs)):
cada trecho (preenchimento · marcas · contorno) é completado até um múltiplo de `8` com segmentos
de comprimento ZERO, e cada bloco de `8` leva a caixa local e se é uma CORRENTE ligada. No pixel:
acima, abaixo ou à direita ⇒ soma zero, salta-se; **todo à esquerda e encadeado ⇒ a soma do bloco
é `clamp(y₀) − clamp(yₙ)`** (a contribuição de um segmento à esquerda é a faixa dele, e numa
corrente ela TELESCOPA) — duas leituras em vez de oito. A caixa no ecrã sai do centro e do valor
absoluto do afim (uma transformação em vez de quatro), e o cabeçalho do eixo passou a usá-la.

⭐ **E a junta redonda deixou de fazer trigonometria no caso comum:** o passo máximo do leque cabe
num teste de COSSENO (`cos(2·acos q) = 2q² − 1`), e um arco de um passo é o triângulo
`centro, n0, n1` — o fim já é conhecido. Numa caneta de um pixel quase toda junta é assim.

⛔⛔ **RECUSA MEDIDA — a ÁRVORE de blocos no eixo** (cabeçalhos dentro de cabeçalhos, `≤ 8` no
topo): **PIOROU** a estrela grande esticada na iGPU, `4,70 → 5,29 ms`. Um grupo de `64` peças
consecutivas é um ARCO do contorno, e a caixa de um arco é gorda — cobre o interior da estrela, logo
o pixel desce quase sempre e paga um nível de testes a mais. *A caixa de um bloco só poupa trabalho
quando é FINA; agrupar blocos finos ao longo de uma curva dá caixas gordas.*

**Gates:** [`blocos_tests`](../../crates/ph2d-shape-gpu/src/blocos_tests.rs) (o enchimento no
último ponto · a caixa justa · a corrente partida lê `0`) · `todo_trecho_cai_em_blocos_inteiros`
(as três espécies de trecho começam e acabam num múltiplo do bloco, em todos os níveis) · e o caso
novo **«anéis even-odd»** na paridade com o Vello — a forma cujo preenchimento PARTE a corrente
dentro de um bloco. ⚠️ A barra dele (`100`/`60`) é a das duas famílias de borda curva, e foi
**medida igual ao último dígito com o laço ANTIGO** (alfa `61`, cor `53`, `4 239` px): o desvio é
do aplanamento, não dos blocos. Com a corrente forçada a «ligada» o anel lê alfa **`245`**.

**`SEGS_POR_BLOCO = 8` é MEDIDO na iGPU** (a varredura `8 · 16 · 32`, sonda, ms): estrelas grandes
só preenchidas `0,73 · 0,75 · 1,35`, contorno conforme `1,67 · 1,91 · 3,82`, densas conformes
`0,76 · 0,91 · 1,12` — acima de `8` a caixa engorda mais depressa do que os cabeçalhos poupam.

⛔ **E a sonda de relógio mentia por omissão:** com o shader partido (um erro de compilação só sai
no registo do `wgpu`) ela leu `0,005 ms` e saiu VERDE. Hoje ela lê a camada cronometrada de volta e
reprova sem `> 1 000` px com tinta — *um relógio sobre um passe que não desenhou é um número de
nada*.

**Mutação `11` de `11` distintas** (arnês com pré-voo `12/12`, corridas LIMPAS verdes com `18` e `4`
testes, restauro com `touch`): a corrente sempre «ligada» (o anel e o gate de unidade) · o salto
pela direita e pela fileira apertados um pixel · o telescópio com o sinal trocado e a acabar no
1.º segmento · a caixa no ecrã sem o valor absoluto · o preenchimento sem fechar o bloco (o gate dos
trechos e a paridade) · o último bloco fora do laço · o leque num triângulo só, a rodar ao
contrário, e a ponta redonda a fechar no ponto de partida. ⚠️ A `J1b` é a MESMA mutação da `J1`
corrida sobre os gates da família do Motion e SOBREVIVE ali — nenhum deles tem uma junta redonda
de ângulo grande; quem a mata é a paridade com o Vello.

⏳ **ABERTO, com o número: na iGPU a forma GRANDE ainda perde para o Vello** — contorno conforme
`1,67` contra `0,91 ms`, e **o contorno ESTICADO `4,7` contra `1,0`** (a eliminação por partes:
`0,74` sem o eixo · `+1,42` os cabeçalhos dos blocos · `+1,09` as caixas das peças · `+1,44` a
geometria delas). O custo por pixel continua `O(blocos)`, e a cura de fundo é a do Vello — cortar
a cópia em LADRILHOS, uma passagem de cálculo antes do desenho. Na RTX a placa ganha em todas as
células, e na cena densa do produto (`324` pequenas) a iGPU empata.

### §9.4 — A FAIXA: o traço esticado sem peças de junta (2026-10-01)

**O que mudou** ([`eixo.rs`](../../crates/ph2d-shape-gpu/src/eixo.rs) · `peca_do_eixo` no
[`shape.wgsl`](../../crates/ph2d-shape-gpu/src/shape.wgsl)): até aqui cada vértice do eixo era uma
peça PRÓPRIA (a junta), com caixa, três pontos e quatro arestas. ⇒ **o vértice passa a ser do
troço que CHEGA a ele.** Os dois troços que lá se encontram acabam na MESMA bissectriz quando ela
serve, e a aresta partilhada cancela-se na soma; quando não serve, quem chega põe a junta. A
bissectriz serve:

- num ponto **LISO** se a esquadria fica a `FAIXA_FOLGA = 0,1 px` do arco verdadeiro;
- numa **QUINA** se a junta autorada É a esquadria dentro do limite — e então ela é exactamente a
  junta (`2 ≤ (1 + cos θ)·limite²`, o teste do Vello);
- nos dois casos, se a esquadria não recua mais de metade de um troço pelo lado de dentro (senão o
  quadrilátero deixa de ser convexo).

`EixoItem` ganha `d` (o vizinho de trás, `48 → 56` bytes) e os bits `FAIXA_INICIO`/`FAIXA_FIM`/
`QUINA_INICIO`/`QUINA_FIM` no `ponta` do troço; `ITEM_JUNTA` sai (o tipo `1` fica vago, os outros
números não se mexem). Os pontos deduplicam-se em **`f32`** — dois pontos distintos em `f64` que
caem no mesmo `f32` dariam um troço de comprimento zero, e a faixa partia-se lá.

**Medido no proxy de telemóvel** (Radeon integrada, `release`, dois rounds iguais):

| sonda (`sonda_relogio_das_estrelas_grandes`), ms | antes | faixa | Vello |
|---|---:|---:|---:|
| `324` pequenas, contorno esticado (o arranjo da `=127`) | `2,21` | **`1,80`** | `2,08` |
| `72` grandes, contorno esticado | `4,69` | **`3,26`** | `1,00` |
| `72` grandes, contorno conforme (não passa pelo eixo) | `1,67` | `1,72` | `0,90` |

| app, `=127` densa, iGPU, perfilador de placa | `render.formas` |
|---|---:|
| antes (só pontos lisos na faixa) | `30,4`–`30,9 ms` |
| **faixa nas quinas** | **`25,3`–`25,6 ms`** |
| a cena TODA pelo Vello (`PH2D_FORMAS_NA_PLACA=0`) | `19,7 ms` (o quadro de placa inteiro) |

⚠️ **A `=127` usa estrelas de quinas VIVAS** (`source.shape` `Star`), e a 1.ª versão da faixa só
cobria pontos lisos: a sonda (estrelas ARREDONDADAS) melhorou `19 %` e o app **não se mexeu**. *Uma
sonda cuja forma não é a da cena mede outro programa* — foi a diferença entre as duas que levou a
faixa às quinas.

⭐⭐ **E a faixa é MAIS EXACTA que o Vello, com prova.** A paridade da estrela esticada subiu de alfa
`40` para `73`, e a réplica da conta do shader em Python (`f32`, a mesma `contribuicao`) mostrou
onde: nos pixels de BORDA do lado de dentro de uma quina. Lá os dois rectângulos sobrepõem-se, e uma
rasterização por área que os SOMA conta a sobreposição duas vezes — é o que o Vello faz (o contorno
do kurbo passa pelo vértice) e o que o passe fazia com uma peça por troço. Medido por
supersamostragem `64²` no pior pixel: **área verdadeira `0,4756` · faixa `0,4755` · soma `0,772`**
(e no traço fino `0,693` · `0,694` · `1,0`). ⇒ gate novo
[`quina_exacta`](../../crates/ph2d-shape-gpu/tests/it/quina_exacta.rs): nas quinas côncavas das
duas fixturas a faixa fica a `0,002` da área verdadeira, com o CONTROLO de que a soma sobreconta
`0,296` e `0,169` (⚠️ uma `V` feita à mão não continha o fenómeno — a régua leu `0,03` e o controlo
reprovou-a). As barras dessas duas famílias passam às das curvas (`100`), com o defeito (o passe sem
eixo) a ler `255`.

**Duas famílias novas na paridade exercitam as cercas que nenhuma estrela grande toca:** estrelas de
`10`–`24 px` com traço grosso (a esquadria recua mais de metade do troço) e a estrela esticada com
`limite 2` (a ponta passa do limite e cede ao chanfro). Antes → faixa: `79`·`20`·`887 px` →
`71`·`28`·`1 497 px` e `51`·`18`·`17` → `61`·`32`·`93`.

**Mutação `9` de `11`** ([arnês](ferramentas/mutacao_a_faixa_do_traco_2026-10-01.py), pré-voo
`11/11`, corrida LIMPA verde com `21` testes): a faixa desligada · a quina lida como lisa · a junta
de recurso sem o estilo · sem a cerca do recuo · sem o limite da esquadria · a caixa da peça sem a
esquadria · sem os bits de quina · sem deduplicar em `f32` · o bloco sem a esquadria. **Sobrevivem,
NOMEADAS:** `S6` (a folga de `0,1 px` num liso) e `S8` (a folga de `0,1 px` na caixa da peça) — as
duas são cercas à escala de `0,1 px`, abaixo do ruído de `0,25 px` de aplanamento que as barras de
curva toleram.

⛔⛔ **RECUSA MEDIDA — HERDAR a bissectriz do troço anterior** (o vértice calculado uma vez por
pixel em vez de duas, com o bit `FAIXA_CONTINUA` e o ponto/bissectriz passados pelo laço): saída
idêntica ao bit, e **nenhum ganho** — sonda `1,84 → 1,81` (densa) e `3,28 → 3,36 ms` (grande), app
`25,3–25,6 → 25,2–25,9 ms`. ⇒ *a conta geométrica por pixel NÃO é o custo*; revertido.

⏳ **ABERTO, com o número: na `=127` densa o proxy de telemóvel continua atrás do Vello** — placa
inteira `30,3` contra `19,7 ms` (era `35,2`), com a CPU a cair `8,2 → 6,2 ms`. Na RTX a placa ganha
em todas as células. O que sobra é o custo POR PEÇA lida por pixel (caixas e arestas de ~`12`
itens de `56` bytes por cópia), e a cura de fundo continua a do Vello: o contorno de cada cópia
calculado UMA vez (uma passagem de cálculo antes do desenho) em vez de em cada pixel.

### §9.5 — O CONTORNO CALCULADO uma vez por cópia (2026-10-01)

**O que mudou** ([`contorno.wgsl`](../../crates/ph2d-shape-gpu/src/contorno.wgsl) ·
[`contorno.rs`](../../crates/ph2d-shape-gpu/src/contorno.rs)): antes do desenho, três passes de
cálculo — `cs_conta` (quantas arestas cada cópia escreve), `cs_soma` (o prefixo, num grupo só e
determinístico) e `cs_escreve` — percorrem as peças do eixo de cada cópia e escrevem as ARESTAS do
contorno já no ECRÃ, em blocos de `8` com a caixa de cada bloco. O fragmento (`traco_do_contorno`)
só soma arestas prontas. A geometria das peças (bissectrizes, juntas, leques, pontas) deixa de ser
refeita em cada pixel.

- ⭐ **As arestas que a FAIXA cancela não chegam a existir**: as duas metades de uma aresta
  partilhada decidem com os MESMOS argumentos e saltam juntas.
- ⭐ **DUAS correntes**: o lado que anda com o eixo escreve-se para a frente a partir do início da
  cópia, o que anda contra ele para trás a partir do fim — lidas por ordem de memória, as duas são
  correntes. Um bloco cujas oito arestas se tocam ponta a ponta (conferido AO BIT) e que fica todo à
  esquerda do pixel soma `clamp(y₀) − clamp(y₈)`, guardados ao lado da caixa (`(y₀, y₈, encadeado)`
  — **zero ligações novas**, o bloco passa a dois `vec4`).
- ⚠️ **A capacidade é MEDIDA, nunca adivinhada:** o total sai do prefixo, é copiado e lido DOIS
  quadros depois, e a capacidade cresce para ele (tecto = `max_storage_buffer_binding_size`). Até lá,
  uma cópia que não cabe — ou cuja escrita não bate na contagem — é desenhada pelo caminho de
  sempre, **por cópia**: nunca um contorno truncado.
- `PH2D_CONTORNO_CALCULADO=0` bissecta (o caminho pixel a pixel do eixo, §9.4).

**Gate** [`contorno_calculado`](../../crates/ph2d-shape-gpu/tests/it/contorno_calculado.rs) — as
sete famílias do traço esticado pelos DOIS caminhos, `4` quadros do mesmo passe: alfa máx. **`1`**
e **zero** pixels a desviar mais de `1` em todos os quadros; no 1.º quadro **três** famílias
transbordam a capacidade de fábrica (círculos `12/40`, zigue-zague redondo `38/40`, estrelas pequenas
de traço grosso `101/120`) e a MISTURA dos dois caminhos desenha a mesma imagem; a partir do 3.º
todas as cópias ganham contorno; o CONTROLO desligado lê `0`. ⚠️ *A imagem sozinha não prova que o
caminho novo correu* — o recurso desenha o mesmo —, por isso o gate lê de volta quantas cópias o
ganharam.

**Medido no proxy de telemóvel** (Radeon integrada, `release`, dois rounds iguais, sonda
`sonda_relogio_das_estrelas_grandes` — ⚠️ a `=2` usa a estrela da CENA, §9.4):

| sonda, ms de passe | eixo (§9.4) | contorno | contorno + correntes | Vello |
|---|---:|---:|---:|---:|
| `1225` estrelas da `=127` (vivas), contorno esticado | `3,56` | `2,42` | **`2,49`** | `2,69` |
| `324` pequenas (arredondadas) | `1,92` | — | `2,05` | `2,16` |
| `72` grandes (arredondadas) | `3,31` | `3,88` | **`3,57`** | `1,00` |
| `72` grandes, conforme (CONTROLO, sem eixo) | `1,74` | — | `1,73` | `0,90` |

| app, `=127` densa, perfilador de placa, quadro de placa INTEIRO (mínimo) | iGPU | RTX |
|---|---:|---:|
| eixo pixel a pixel (`PH2D_CONTORNO_CALCULADO=0`) | `27,8 ms` | `1,57 ms` |
| **contorno calculado** | **`22,3 ms`** (formas `15,6` + cálculo `3,1`) | **`1,16 ms`** |
| a cena TODA pelo Vello (`PH2D_FORMAS_NA_PLACA=0`) | `19,4 ms` | `1,99 ms` |

⚠️ *O `render.vello` do perfilador CONTÉM o `render.formas` e o `render.contorno`* — é o quadro de
placa inteiro, e é ele que se compara com a rota do Vello. Na RTX o quadro oscila `1,2`–`7,4 ms` pelo
relógio da placa a esta carga, logo compara-se o MÍNIMO.

**Mutação `5` de `5` + `1` NOMEADA** ([arnês](ferramentas/mutacao_o_contorno_calculado_2026-10-01.py),
pré-voo `6/6`, corrida LIMPA verde): o contorno nunca escrito · sem a aresta de ponta da frente ·
todo bloco lido como encadeado · o telescópio ao contrário · a capacidade sem crescer — as cinco
sangram no gate. **Sobrevive, NOMEADA:** `M6` (o lado de baixo na corrente da frente) — quebra só as
correntes, e as correntes são RELÓGIO: a soma é a mesma em qualquer ordem, logo nenhuma régua de
imagem a vê.

**A ablação que mandou para as correntes** (`abla2.sh`, iGPU): saltar os blocos à esquerda (só
relógio, imagem errada) levava as `72` grandes de `3,88` a `3,17 ms` — o teto do que as correntes
podiam comprar; elas compraram `3,57`.

⛔⛔ **RECUSA MEDIDA — COSTURAR as correntes por um passo guloso** (no `cs_escreve`, pôr a seguir a
cada aresta a que começa onde ela acaba, numa janela de `32`): **pior em todas as células** —
`2,49 → 3,11` (densa), `2,05 → 2,62`, `3,57 → 4,00 ms` (grandes). O laço serial por cópia sobre a
memória custa mais do que os blocos que encadeia. Revertido.

⛔ **RECUSA MEDIDA — os testes de lado sem divisão por aresta** no `contribuicao` (`max(px) ≤ 0` ⇒
faixa, `min(px) ≥ 1` ⇒ zero): sem efeito no relógio (a divergência da onda paga o caminho caro na
mesma); mantidos por serem exactos e baratos.

⏳ **ABERTO, com o número: no proxy de telemóvel a `=127` densa continua atrás do Vello**
(`22,3` contra `19,4 ms` de placa; era `27,8`), e nas estrelas GRANDES o contorno ainda perde para o
eixo (`3,57` contra `3,31`). O que sobra está no desenho (`15,6 ms`), não no cálculo (`3,1`): os blocos
à direita/dentro pagam a conta do Vello aresta a aresta, e a cura seguinte é a do Vello — ladrilhos
(as arestas de cada cópia ordenadas por faixa de ecrã) em vez de blocos por cópia.

### §9.6 — AS CÉLULAS E O FUNDO: toda a cópia no ecrã (2026-10-01)

**O que mudou** ([`contorno.wgsl`](../../crates/ph2d-shape-gpu/src/contorno.wgsl) ·
`cobertura_de_ecra` no [`shape.wgsl`](../../crates/ph2d-shape-gpu/src/shape.wgsl) ·
[`contorno.rs`](../../crates/ph2d-shape-gpu/src/contorno.rs)):

- ⭐ **As três famílias no ecrã.** O cálculo de §9.5 escrevia só o contorno do eixo; o preenchimento e
  as marcas continuavam a ser transformados EM CADA PIXEL (a ablação na sonda densa: `~1/3` do
  passe). Agora `cs_escreve` transforma-os uma vez por cópia e escreve-os à frente do contorno, cada
  família completada até um múltiplo de `8` (os blocos não atravessam um trecho). Sob afim conforme
  as «marcas» são o traço INTEIRO (as marcas e o contorno expandido), e a soma do pixel é a mesma
  `min(min(|marcas|, 1) + min(|contorno|, 1), 1)` do caminho de sempre.
- ⭐⭐ **As CÉLULAS e o FUNDO** (o `backdrop` do Vello): cada fileira de pixels da cópia é partida em
  células de `LARGURA_DA_CELULA = 32 px`; cada célula guarda três fundos (a soma, NESSA fileira, dos
  blocos que acabam todos à esquerda dela — não depende do `x` do pixel) e uma máscara com um bit por
  bloco que lhe toca. O pixel soma o fundo e só esses blocos. A fileira de um bloco é exacta em
  `f32` (`floor(lo.y)` a `ceil(hi.y) − 1`); o fundo soma-se na primeira célula toda à direita do
  bloco e depois faz-se o prefixo ao longo da linha.
- ⭐ **A contagem sem a geometria.** `cs_conta` corria o `percorre` inteiro só para contar; agora usa
  o pior caso de cada ramo de `emite_peca` (o leque de `k` passos emite no máximo `3k`), e
  `cs_escreve` desce a corrente de trás para o fim do que de facto usou. O cálculo da `=127` densa no
  proxy: `4,08 → 2,93 ms`.
- ⭐ **A ROTA:** uma cópia esticada com traço vai sempre; uma CONFORME só a partir de
  `AREA_MINIMA_CONFORME = 1024 px²` de caixa estimada (abaixo disso o caminho de sempre já a desenha
  bem, e o cálculo é pago por cópia). `ShapePass::area_minima_conforme` deixa os gates de pixel medirem
  o caminho novo em todas as cópias; `PH2D_AREA_MINIMA_CONFORME` é o instrumento da varredura.

**Medido no proxy de telemóvel** (Radeon integrada, `release`; a placa leu estável sob carga — o
caminho de sempre deu os números de §9.5 ao dígito na mesma corrida):

| sonda, ms de passe | §9.5 | arestas no ecrã | + células e fundo | + contagem barata | Vello |
|---|---:|---:|---:|---:|---:|
| `1225` estrelas da `=127` | `2,49` | `2,33`–`2,45` | `2,57` | **`2,22`** | `2,6` |
| `324` pequenas | `2,05` | `1,77`–`1,82` | `1,82` | **`1,67`** | `2,08` |
| `72` grandes esticadas | `3,57` | `2,69` | `2,38` | **`2,32`** | `0,98` |
| `72` grandes conformes | `1,73` | `1,39` | `1,38` | **`1,35`** | `0,90` |

| app, `=127` densa, quadro de placa (mínimo) | iGPU | RTX |
|---|---:|---:|
| §9.5 (contorno calculado) | `22,3 ms` | `1,16 ms` |
| arestas no ecrã + células (formas `11,89` + cálculo `4,08`) | `19,87 ms` | `1,11 ms` |
| **+ contagem barata** (formas `11,84` + cálculo `2,93`) | **`18,76 ms`** | — |
| a cena toda pelo Vello | `19,42 ms` | `2,03 ms` |

**A varredura da rota** (cena `17`, escada de `32 768` estrelas pequenas conformes, iGPU, quadro de
placa MEDIANO; a cena anima, logo o mínimo não serve): caminho de sempre `9,70` · área mínima `0`
(todas pelo cálculo) **`13,53`** · `256`, `1024`, `4096`, `16384` → `9,58`–`9,59`. E as `72` grandes
conformes: `0`/`1024` → `1,36 ms`, `4096` → `1,74` (perde o ganho). ⇒ a janela é `[256, 4096)`, e o
`1024` está dentro dela.

**A largura da célula é medida** (sonda, iGPU, ms): grandes esticadas `8 → 2,81` · `16 → 2,50` ·
**`32 → 2,33`** · `64 → 2,40`; grandes conformes `2,13 · 1,58 ·` **`1,36`** `· 1,45`; a `=127`
`32` e `64` empatam (`2,22`).

**Gates:** [`contorno_calculado`](../../crates/ph2d-shape-gpu/tests/it/contorno_calculado.rs) ganha
SETE famílias — estrelas pequenas só preenchidas · anéis even-odd · círculos com traço conformes ·
círculos GRANDES com traço (conformes e esticados — as máscaras passam de uma palavra, `> 32` blocos)
· estrela com traço e MARCAS (conforme e esticada) — todas a alfa máx. `≤ 1` e zero pixels a
desviar mais de `1`, catorze famílias a transbordar no 1.º quadro e todas as cópias no ecrã no
último. A paridade com o Vello passa a medir o REGIME DO PRODUTO (`3` quadros, exige todas as cópias
no ecrã). E `so_as_copias_conformes_grandes_pagam_o_calculo`: pequenas ficam, grandes vão, esticadas
com traço vão sempre, com o CONTROLO a área `0`.

**Mutação `10` de `10`** ([arnês](ferramentas/mutacao_as_celulas_do_ecra_2026-10-01.py), pré-voo
`10/10`, corrida LIMPA verde): o fundo sem o prefixo · a máscara só com a 1.ª palavra (sangra nos
círculos GRANDES — a família que a exerce) · o fundo na célula que o bloco ainda toca · o pixel sem o
fundo · todos os blocos no preenchimento · a corrente de trás sem descer · o limite da contagem sem a
junta (sangra pela capacidade que não cresce) · a rota ao contrário · a 1.ª linha de um bloco
arredondada para cima · a célula do pixel uma à direita.

⛔ **RECUSA MEDIDA — PARTIR as arestas longas em pedaços de `32 px`** (para cada bloco ficar local,
como o Vello recorta por ladrilho): sem efeito — grandes conformes `1,35 → 1,39 ms`. A estrela da sonda
tem cantos arredondados e as arestas JÁ são curtas; o custo dos pixels de borda não é aresta longa.
Revertido.

⏳ **ABERTO, com o número: as estrelas GRANDES no proxy de telemóvel** — `2,32` contra `0,98` ms
(esticadas) e `1,35` contra `0,90` (conformes). A eliminação por partes (esticadas): o cálculo
(`~0,3` contagem + `~0,36` escrita + `~0,42` células) corre num FIO por cópia, e com `72` cópias a
placa fica quase parada; os pixels de BORDA (`~1,2`) lêem os blocos inteiros da célula. A cura
seguinte tem endereço: as células e as máscaras construídas EM PARALELO por bloco (não por cópia), e
os pixels de uma célula a partilhar os blocos (memória do grupo), que é o que o rasterizador fino do
Vello faz.


### §9.7 — AS CÉLULAS EM PARALELO: um fio por FILEIRA do ecrã (2026-10-01)

A cura nomeada no fim do §9.6, metade dela. O `cs_celulas` deixa de correr num fio por CÓPIA e passa a
correr num fio por **fileira de ecrã** de cada cópia; a contagem (`contagem`) ganhou um terceiro terço
`(n+1)` com as fileiras por cópia, o `cs_soma` faz o terceiro prefixo e escreve os argumentos do despacho
**indirecto** (`despacho_rw`, binding 7). ⚠️ O `wgpu` recusa um buffer que é ao mesmo tempo armazenamento
de escrita e argumento indirecto no MESMO despacho ⇒ o `cs_celulas` tem um grupo de ligação PRÓPRIO
(`escrita_celulas`, sem o binding 7). Um fio acha a sua cópia por busca binária no prefixo, zera a faixa,
escreve as máscaras e o fundo de cada bloco na fileira dele, e faz o prefixo do fundo ao longo da fileira.

Medido no proxy de telemóvel (iGPU, `radeon_icd`, máquina calma, `sonda_relogio_das_estrelas_grandes`, ms):

| cena | antes (§9.6) | fileiras em paralelo | Vello |
|---|---|---|---|
| estrelas grandes esticadas | `2,32` | **`2,02`** | `0,98` |
| estrelas grandes conformes | `1,35` | **`1,17`** | `0,90` |
| densas | `2,22` | **`2,11`** | — |

GPU `ph2d-shape-gpu --ignored` **5/5**. ⚠️ **Por ablação (dobrar o desenho), quem manda agora é o DESENHO**
(`~1,38 ms` nas esticadas), e dentro dele o laço de blocos por pixel (`~1,2 ms`) — é a outra metade da cura.

⛔ **Recusas MEDIDAS (não reconstruir):**
- **blocos de 4 arestas** em vez de 8 — resultado misto, e as esticadas PIORAM.
- **células mais altas** (`ALTURA_DA_CELULA`): `H=1 2,02` · `2 2,17` · `4 2,30` · `8 2,69` — piora em
  monotonia; o registo cresce com a altura e cada pixel lê-o inteiro.
- ⏳ **pré-calcular a cobertura 8 px por fio no compute** (para o desenho ler 2 palavras em vez do laço de
  blocos): o protótipo foi construído e reprovou **4 de 5** gates de paridade antes de ser diagnosticado ⇒
  REVERTIDO, não refutado. ➜ **Substituído pelo §9.8** (as listas das células): o protótipo não estava
  guardado, e só dividia a leitura — a lista corta a conta.

### §9.8 — AS LISTAS DAS CÉLULAS: o que o Vello guarda por ladrilho, montado por ARESTA (2026-10-02)

**O porquê.** Depois do §9.7 quem mandava nas estrelas grandes era o DESENHO, e dentro dele o laço por
pixel: a máscara da célula dizia que BLOCOS tocavam a fileira, e cada pixel de borda percorria os oito
segmentos de cada um — quase sempre um só cruzava a fileira. ⇒ cada célula passa a guardar a **LISTA
das arestas que cruzam a fileira dentro dela** (o `backdrop` continua: as que ficam todas à esquerda
somam-se no fundo). É a lista de segmentos por ladrilho do rasterizador fino do Vello, com ladrilhos de
`32 × 1 px`.

**O protótipo do §9.7 («cobertura 8 px por fio no compute») NÃO foi reconstruído:** não estava guardado
em lado nenhum, e só dividia a LEITURA por oito pixels — a conta por pixel×segmento ficava igual. A
lista corta as duas.

**O que mudou** ([`contorno.wgsl`](../../crates/ph2d-shape-gpu/src/contorno.wgsl) ·
`cobertura_de_ecra` no [`shape.wgsl`](../../crates/ph2d-shape-gpu/src/shape.wgsl) ·
[`contorno.rs`](../../crates/ph2d-shape-gpu/src/contorno.rs)):

- O registo de célula é fixo: `REGISTO = 7` palavras — os três fundos e `(inicio, fim_f, fim_m, fim)`
  da lista de cada família. As máscaras de blocos, as caixas dos blocos e as DUAS correntes da escrita
  (que só existiam para as caixas serem finas) saíram.
- A montagem é por **ARESTA**, em quatro despachos indirectos: `cs_zera` (fio por fileira) ·
  `cs_conta_listas` (fio por aresta: fundo e contagem por atómicos) · `cs_lugar_das_listas` (fio por
  fileira: reserva as listas dela de uma vez, prefixo do fundo, cursores) · `cs_escreve_listas` (fio por
  aresta). Os argumentos saem do `cs_soma` (`despacha`: `[0,3)` por linha, `[3,6)` por aresta).
- ⭐ **Determinístico por construção:** o fundo soma-se em PONTO FIXO (`ESCALA_FIXA = 2¹⁶`) e o
  fragmento soma a lista também em ponto fixo — a ORDEM em que os atómicos arrumaram uma lista não muda
  um bit. Cada parcela erra `≤ 7,6e-6`.
- O intervalo de uma aresta numa fileira é o dos DOIS PONTOS, sem recortar (exacto: a `contribuicao`
  é contínua no `x`; a lista só fica mais longa numa aresta que atravessa fileiras) — recortar custava
  uma divisão por aresta e só valia `1,52 → 1,40 ms`.
- ⚠️ **Uma fileira sem lugar fica `SEM_LISTA`** e o pixel refaz-se pelo caminho de sempre. As listas só
  se contam DEPOIS de as arestas existirem ⇒ a capacidade delas chega **dois quadros depois** da das
  arestas (5.º quadro numa cena nova); até lá a imagem é a mesma (gate). `ShapePass::limita_as_listas`
  e `listas_do_ultimo_quadro` são as portas dos gates.
- O cálculo tem agora TRÊS relógios no perfilador: `render.contorno.conta`, `.escreve`, `.celulas`.

**Medido na sonda** (`sonda_relogio_das_estrelas_grandes`, que ganhou `PH2D_FLUID_PROFILE=1` — `250`
quadros e o relógio da placa por passe; [`mede_sonda_das_estrelas.sh`](ferramentas/mede_sonda_das_estrelas.sh)
corre os três arranjos só com `load < 4`). Proxy de telemóvel (iGPU), ms:

| decomposição (iGPU, perfilador) | desenho | cálculo | total |
|---|---:|---:|---:|
| esticadas, §9.7 | `1,36` | `0,57` | `1,93` |
| esticadas, listas montadas por FILEIRA (1.ª tentativa) | `0,81` | `1,52` | `2,33` |
| esticadas, listas montadas por ARESTA | **`0,82`** | **`0,69`** | **`1,51`** |
| conformes, §9.7 → por aresta | `0,875 → 0,47` | `0,22 → 0,29` | `1,10 → 0,76` |

⛔ **RECUSA MEDIDA — montar as listas com um fio por FILEIRA** (o regime do §9.7): `~2 900` fios na sonda,
cada um a percorrer centenas de arestas em série, DUAS vezes (contar e escrever) — a placa integrada
ficava à espera da memória e o cálculo subia `0,57 → 1,52 ms`, comendo todo o ganho do desenho. Por
aresta são `~70 000` fios de uma ou duas fileiras.

**A largura da célula foi re-medida para as listas** (desenho + células, ms; esticadas · conformes ·
densas): `16 → 1,34 · 0,79 · 1,61` · **`32 → 1,25 · 0,68 · 1,41`** · `64 → 1,60 · 0,85 · 1,60`. Fica `32`.

**A tabela calma** (a mesma sonda, o binário de ANTES e o de DEPOIS, `load < 4` em cada corrida — `2`
corridas por célula, a média; placa · Vello, ms):

| arranjo | iGPU antes | iGPU depois | RTX antes | RTX depois |
|---|---:|---:|---:|---:|
| `72` grandes esticadas | `2,04` · `1,01` | **`1,74`** · `1,05` | `0,42` · `0,37` | **`0,36`** · `0,40` |
| `72` grandes conformes | `1,18` · `0,92` | **`0,94`** · `0,92` | `0,17` · `0,37` | `0,18` · `0,37` |
| `1225` densas da `=127` | `2,13` · `2,69` | **`1,71`** · `2,81` | `0,19` · `0,52` | `0,21` · `0,52` |

⇒ no proxy de telemóvel as três descem (`−15 %`, `−20 %`, `−20 %`); as conformes grandes EMPATAM com o
Vello e as densas ficam `1,6×` à frente; ⏳ as grandes ESTICADAS ainda perdem (`1,74` contra `1,05`). Na
RTX as esticadas passam à FRENTE do Vello; ⚠️ as densas sobem `0,19 → 0,21 ms` (quatro despachos a
mais, numa placa onde o cálculo era quase nada) — continuam `2,5×` à frente.

**A W5 repetida, no APP, com a máquina calma** ([`mede_formas_na_placa.sh`](ferramentas/mede_formas_na_placa.sh),
a MESMA build `release` com e sem `PH2D_FORMAS_NA_PLACA`, cada célula com `load < 4` antes e depois;
as três últimas janelas `[frame]` de `120` quadros). Fecha o item «W0/W5 como tabela única» do handoff de
01/10:

| placa | cena | objectos | quadro sem → com | CPU (encode) sem → com | Motion sem → com |
|---|---|---:|---:|---:|---:|
| RTX | escada, estrela | 4 096 | 16,69 → 16,65 | 4,31 → **3,34** | 1,31 → **0,74** |
| RTX | escada, estrela | 16 384 | 16,65 → 16,66 | 6,79 → **3,44** | 2,91 → **0,78** |
| RTX | escada, estrela | 32 768 | 16,66 → 16,67 | 9,79 → **3,68** | 4,27 → **0,88** |
| RTX | `=127` densa | 16 384 | 16,71 → 16,70 | 6,96 → **3,07** | 1,97 → **0,66** |
| iGPU | escada, estrela | 4 096 | 16,70 → 16,70 | 4,33 → **4,26** | 1,41 → **0,80** |
| iGPU | escada, estrela | 16 384 | 16,68 → 16,67 | 6,82 → **3,29** | 2,78 → **0,68** |
| iGPU | escada, estrela | 32 768 | **18,04 → 16,71** | 11,14 → **3,00** | 6,12 → **0,62** |
| iGPU | `=127` densa | 16 384 | **20,68 → 17,59** | 7,98 → **4,29** | 2,34 → **0,75** |

⇒ ⭐ **o achado do §9.3 está curado no app:** no proxy de telemóvel a `=127` densa ia de `20,7` a
**`35,2 ms`** pela placa; hoje vai a **`17,6 ms` (57 fps contra 48 sem ela)**. Todas as outras células
batem no tecto de `60 Hz` nas duas rotas, com a CPU a metade ou menos. (O `acquire` da iGPU continua
`~12–13 ms` com e sem placa: é a apresentação a `60 Hz`, não trabalho.)

**Gates:** `as_fileiras_que_nao_cabem_nas_listas_desenham_o_mesmo` (tecto em METADE do pedido em todos os
quadros; cópias TODAS pelas células, fileiras `SEM_LISTA`, imagem do eixo à barra; controlo: o tecto
morde) · `uma_cena_que_muda_nao_le_as_arestas_do_quadro_anterior` (duas etapas no MESMO passe) · e o
regime do produto passa a exigir as listas a caber (`QUADROS_DO_PRODUTO = 5`, `QUADROS = 6`). GPU RTX:
`ph2d-shape-gpu` **7/7** · `ph2d-app-motion` placa **4/4** + ponte **1/1** · `ph2d-gpu-cook` formas **2/2**.

**Mutação `14` de `14`** ([arnês](ferramentas/mutacao_as_listas_das_celulas_2026-10-02.py), pré-voo
`14/14`, corrida LIMPA verde): fundo na `ka` · lista uma célula a mais à esquerda / a menos à direita ·
o fragmento ignora `SEM_LISTA` · sem o prefixo do fundo · cursor das marcas no início · a escrita não
salta `SEM_LISTA` · escala do ponto fixo dobrada · `min/max` trocados · `floor` na última fileira ·
`cs_zera` mudo · sem a verificação da capacidade · marcas e contorno trocados · e a **M10** (sem a guarda
das arestas reservadas e NÃO escritas), que SOBREVIVEU aos seis gates de então — num passe novo a
reserva é zero e não soma nada. ⭐ *Uma régua de quadro único não vê o que um buffer traz do quadro
anterior*: o gate da cena que muda foi escrito por ela, e é o único que a mata.

### §9.9 — O TRACEJADO NO ECRÃ: o traço tracejado sob escala não uniforme vai à placa (2026-10-02)

Commits `3abf99b77` (o tracejado) e `4aa06e738` (a variante da `=127`); antes, `7a58e7aaf` (gate das
LETRAS, abaixo).

**O porquê.** Até aqui o tracejado sob afim não conforme ficava no Vello (cerca `FLAG_SO_CONFORME` →
`RECUSA_FORMA_TRACEJADA` no dispositivo, `so_conforme` na rota da CPU); a cena `=76` (as bolas com
contorno tracejado) ia INTEIRA à CPU só por ter tracejado, mesmo conforme.

**A lei da casa (fonte).** `stroke_uniform::pen_for` traceja a geometria JÁ transformada com caneta
`w·√|det|` e padrão `× √|det|`; o Vello (`vello::Scene::stroke`) corta pelo `kurbo::dash` (CPU) e o
traçador da placa dele desenha as pontas. Semântica do `kurbo::dash` 0.13.1 portada: a fase recomeça em
cada sub-caminho; cada traço tem `start_cap` no início e `end_cap` no fim; num FECHADO o último traço
EMENDA no primeiro (junta, sem pontas) só quando atravessa o início; um traço que acaba exactamente num
vértice não liga.

**O que mudou.**

| onde | o quê |
|---|---|
| `EixoItem` `56 → 72 B` | `traco`, `vao` (locais, `[traço, vão]` com fase 0 — `tracejado_do_eixo`; outro padrão mantém `FLAG_SO_CONFORME`) · `flecha` (a flecha de cada corda: o ponto da curva no meio dela menos o meio da corda, por `PathSeg::nearest`) |
| bits no `ponta` do troço | `SUB_INICIO = 16` (o `_pad` do primeiro troço = quantos troços) · `SUB_FECHADO = 32` · pontas do início em `TAMPA_INICIO_BIT = 6` e do fim em `TAMPA_FIM_BIT = 8` |
| cabeçalho de bloco | `BLOCO_TRACEJADO` (o pixel a pixel não o salta pela caixa) |
| itens de ponta | nenhum num sub-caminho tracejado |
| `shape.wgsl` (partilhado pelas duas passagens) | `sub_tracejado` (pré-passo de um fechado: comprimento total e se emenda) · `troco_tracejado` · `pedaco` (o traço `n` ocupa `[n·período, n·período + traço]`, é do troço onde COMEÇA, `[s0, fim)`) · `tracejado_px` (pixel a pixel) |
| `contorno.wgsl` (o contorno calculado) | `emite_tracejado` / `emite_pedaco` |

O arco de cada troço no ecrã é `corda + 8h²/3c` (`arco`, `h` = a flecha levada pelo afim); as decisões de
faixa ficam nas CORDAS (os dois lados de um vértice recalculam-nas com os mesmos argumentos) e o recuo da
bissectriz é limitado pela metade do menor dos dois PEDAÇOS que se tocam (`bissectriz_ate`).
`TRACOS_POR_TROCO_MAX = 65536` é o tecto do vigia do dispositivo (um laço sem fim perde a placa), não
escolha de desenho.

**Na Motion.** A cerca saiu das duas rotas (`RECUSA_FORMA_TRACEJADA` e o `so_conforme` apagados); uma
geometria com `FLAG_SO_CONFORME` (que o `kurbo_stroke` da casa nunca produz) passa a `Entrada::Recusada`
→ `RECUSA_FORMA_DO_VELLO`. **Censo de rota:** a `=76` vai à PLACA, **`19` das `23`** cenas com forma; as
`4` que ficam: `fx.glow` (`=70`) · passagem (`=120`) · colisor lido (`=114`, `=115`).

**Medido** (RTX, meio-float; gate `ph2d-shape-gpu --test it tracejado`, contra o Vello; alfa · cor):

| família | alfa | cor |
|---|---:|---:|
| estrela esticada | `71` | `47` |
| traços longos (faixa e junta dentro do traço, emenda) | `84` | `45` |
| círculo, pontas redondas (era `70` · `89` antes da flecha) | `66` | `64` |
| zigue-zague, pontas diferentes (redonda/quadrada) | `50` | `47` |
| zigue-zague, chanfro e pontas quadradas | `4` | `3` |
| controlo conforme (o caminho pré-expandido, intocado) | `75` | `80` |

Contorno calculado contra pixel a pixel: alfa `≤ 1`, todas as cópias com contorno. **Rota do PRODUTO**
(`motion_shape_placa::gpu_tests::tracejado`, `StrokeSpec` com o tracejado AJUSTADO, a câmara): `87` · `69`,
`2 070` px `> 16` sobre `190 140`; sem a flecha lia `134`.

**A barra.** O vale é no ALFA — limpo `≤ 84`, mutações `126`–`255` ⇒ `100`/`100`. ⚠️ A fracção de pixels
com alfa `> 1` NÃO separa no tracejado (o controlo conforme lê `5,7 %` — cada traço com pontas redondas é
borda curva — e a emenda que falta `2,4 %`), por isso o gate do tracejado não a usa.

**Mutação `17` de `17`** ([arnês](ferramentas/mutacao_o_tracejado_no_ecra_2026-10-02.py), sobre os `9`
gates GPU de `ph2d-shape-gpu` e os `6` de paridade do produto): padrão sem a caneta · nunca emenda ·
emenda sempre · nunca liga atrás · nunca liga à frente · faixa sem o recuo dos pedaços · fase por troço ·
pontas de início e fim trocadas · sem a ponta do início · arco no LOCAL × caneta · arco = corda (só o
gate do PRODUTO a mata) · pixel a pixel sem a junta dentro do traço · calculado sem a ponta do fim · recuo
da emenda de um troço do meio · pixel a pixel salta blocos tracejados · fechado não marcado (Rust) ·
contagem de troços a menos (Rust). ⭐ A T8 (pontas trocadas) SOBREVIVEU à 1.ª corrida — todas as famílias
tinham as duas pontas iguais; a família «pontas diferentes» foi escrita por ela.

**⚠️ Divergência DECLARADA — a mordida.** Um traço RENTE que acaba a menos de meia largura depois de uma
quina sai do traçador da casa com uma MORDIDA no lado de dentro (a junta interior passa pelo pivô e o
pedaço curto cruza-se; a régua EXACTA — `kurbo::stroke` a expandir e o Vello só a preencher — tem a mesma
mordida, alfa `52` do traço do Vello); a placa desenha a união verdadeira. Medido numa cópia (pedaço de
`9,75 px`, raio `11,4`): `140` de alfa num pixel; na rota do produto, estrela com pontas rentes, `203`. Os
gates usam ponta quadrada nessas famílias. Mesma família da quina exacta do §9.4.

**⚠️ A régua tem defeitos próprios.** Com traço grosso numa volta apertada o Vello abre uma rachadura de
um pixel dentro do traço (alfa `0`–`136` onde a placa pinta `255`); as fixturas evitam-na (largura `0,07`).

**✅ O AJUSTE DA EMENDA NO ECRÃ (`042327a6a`, curado na mesma jornada).** O ajuste do tracejado (`dash_fit`,
a cura da emenda de 22/08) media o contorno no espaço LOCAL e o padrão era depois escalado por `√|det|`;
sob escala não uniforme o comprimento no ecrã não é `√|det|·L`, e a emenda voltava — nas DUAS rotas (e no
vetor de documento). Agora, no ramo NÃO conforme:

- **a casa** (`ph2d_vec_render::ajusta_no_ecra`, chamada por `stroke_uniform` e `stroke_uniform_image`)
  ajusta o padrão da caneta (já `× √|det|`) ao sub-caminho mais LONGO da geometria JÁ transformada, pela
  mesma `dash_fit::fit` (`n` períodos num fechado; `n` mais um traço num aberto). Sob afim conforme o ecrã
  é o local escalado e nada muda (esse caminho nem passa lá). Gates `o_tracejado_esticado_fecha_no_contorno_do_ecra`
  (com CONTROLO: o ajuste local deixava a emenda a `0,7` de período) e `…_aberto_acaba_com_traco_inteiro`;
- **a placa** (`ajuste_do_tracejado`, por cópia) faz a MESMA conta — `floor(x + 0,5)`, porque o `round` do
  WGSL arredonda as metades para o par e o `f64::round` para longe do zero;
- ⭐ **a folga** `FOLGA_DO_AJUSTE = 1e-4`: o ajuste exacto punha o fim de um fechado EXACTAMENTE na
  fronteira traço/vão, onde o `f64` do kurbo e o `f32` da placa decidiam a emenda por arredondamento;
  alongado o período, o fim cai sempre DENTRO do último vão (`≤ 0,3 px` numa volta de `3 000 px`). As
  mutações «sem a folga» SANGRAM nas duas rotas (placa: alfa `204`–`255`; casa: `255`) — a ambiguidade
  era real, não teórica;
- ⚠️ consequência: o contorno mais LONGO nunca mais emenda; a emenda só acontece num sub-caminho mais
  curto (um furo). As mutações da emenda (`T2`, `T13`) SOBREVIVERAM até à família «estrela com furo»
  (furo HEXAGONAL: numa ponta de estrela a faixa nunca serve e o recuo da emenda não decide nada);
- a régua da crate passou a CHAMAR `pen_for` + `ajusta_no_ecra` (dev-dep `ph2d-vec-render`) em vez de
  copiar a lei.

Medido (RTX): famílias esticadas alfa `≤ 84` · cor `≤ 63`; estrela com furo `63` · `57`; produto `85` ·
`58`. **Mutação `21` de `21`** (as `17` de cima com as âncoras novas, mais «placa sem o ajuste» · «placa
sem a folga» · «casa sem o ajuste» · «casa sem a folga»).

**Smoke.** `=127` com `PH2D_TRACO_ESTICADO_TRACEJADO=1` (o `Dash 2` e `Dash Gap 1,5` do cartão postos à
partida); fotografado na tela virtual: `1024` cópias pela PLACA (do dispositivo).

**Os glifos (`7a58e7aaf`).** Os glifos do `source.text` já iam à placa desde a W3 — o censo dava `6` das
`7` cenas com texto na placa, e a 7.ª era a `=76` pelo tracejado; mas nenhum gate de pixel tinha um
glifo. Gate das LETRAS: alfa `63` · cor `62` · `2 990` px `> 16` sobre `81 229`; as mutações «sem
furos» (`38 006` px) e «aplanamento 8×» (`8 121` px) reprovam.

### §9.10 — A VARIANTE ENXUTA: o tracejado inline dobrara os registos de TODA a cena (2026-10-03)

Commit `8cb0ab9e1`.

**O achado.** O handoff de 02/10 pedia re-medir a linha de base porque o `EixoItem` crescera `56 → 72 B`
no §9.9. A sonda intercalada (o binário de `7a58e7aaf`, antes do §9.9, e o de HEAD na MESMA janela calma,
`load < 4`, [`mede_sonda_das_estrelas.sh`](ferramentas/mede_sonda_das_estrelas.sh) com várias cópias)
achou uma regressão de **`+33 %` a `+41 %` na iGPU nos TRÊS arranjos** — e as conformes nem lêem o eixo.
⇒ não era o eixo maior.

**O mecanismo — medido sem relógio** ([`registos_dos_shaders.sh`](ferramentas/registos_dos_shaders.sh):
`RADV_DEBUG=shaderstats,nocache`, a mesma sonda). O ramo do tracejado (`tracejado_px`, `emite_tracejado`,
`ajuste_do_tracejado`) é código grande atrás de um `if` de dados; o compilador aloca os registos do PIOR
caminho para o shader inteiro:

| shader (iGPU, RADV) | antes do §9.9 | HEAD (§9.9) | enxuta | completa |
|---|---|---|---|---|
| fragmento do desenho | `56` VGPRs · `18` ondas/SIMD · `16 620 B` | **`128` · `8`** · `35 704 B` | `56` · `18` · `16 620 B` | `128` · `8` |
| `cs_escreve` (um fio por cópia) | `64` · `16` | **`128` · `8`** | `64` · `16` | `128` · `8` |
| `cs_conta` | `40` · `24` | `48` · `20` | `40` · `24` | `48` · `20` |

Nada em scratch; os outros kernels nossos e os do Vello idênticos. Ocupação `18 → 8` ondas no fragmento: a
iGPU deixa de esconder a latência das leituras encadeadas (cópia → registo → lista).

**A cura.** `override TRACEJADO: bool` no `shape.wgsl`, dentro do predicado `tracejado(it)` (os três
chamadores: o pixel a pixel, o `percorre` e o `limite_de_arestas`). O passe compila as duas variantes do
desenho, do `cs_conta` e do `cs_escreve` na criação (`contorno::Variantes`) e escolhe pelo eixo CARREGADO
(`EixoItem::tracejado`, o mesmo predicado, no mesmo buffer que o shader lê). A enxuta é byte a byte o
fragmento de antes (`16 620 B`). Uma cena com tracejado continua a pagar a completa.

**Gate:** `so_um_eixo_tracejado_pede_a_variante_completa` (com CONTROLO: a geometria contínua TEM eixo).
Os gates de pixel não vêem «completa sempre» — a mesma imagem, mais devagar; vêem «enxuta sempre» (a cena
tracejada perde os traços). ⚠️ «O `override` fora do predicado» não muda um pixel nem a escolha: só o
`registos_dos_shaders.sh` o vê — corra-o depois de mexer em qualquer ramo raro de um shader quente.

⭐ *Um ramo raro inline num shader quente cobra o seu preço a quem nunca o toma*: o relógio do §9.9 foi
tirado na RTX (que tem registos de sobra: `0,33 → 0,36 ms`), e a iGPU — o proxy de telemóvel — pagou `+41 %`.

**Mutação `8` de `9`** ([arnês](ferramentas/mutacao_a_variante_enxuta_2026-10-03.py), pré-voo `9/9`, corrida
LIMPA `16` verdes): escolhe sempre a completa (só o gate novo a mata) · sempre a enxuta · `Variantes::de`
trocada · opções de compilação trocadas · contagem / escrita / desenho sempre na enxuta · o predicado do
Rust aceita todo troço (só o gate novo) — e a **V9** (o `override` fora do predicado do WGSL)
**SOBREVIVE, como declarado**: só o `registos_dos_shaders.sh` a vê.

⛔ **Recusa MEDIDA — a origem das células como interpolante `flat`** (o vértice já lê a cópia; o fragmento
saltaria uma leitura encadeada antes da do registo). Decomposição iGPU, esticadas, `PH2D_FLUID_PROFILE=1`,
`load < 4`: desenho `0,81 → 0,80 ms` (critério escrito ANTES: `≥ 10 %`). A cadeia cópia → registo → lista
não é o que custa no desenho.

### §9.11 — A ABLAÇÃO DO DESENHO: `84 %` é o laço das listas (2026-10-03) — e o desenho seguinte

**A tabela calma de base** (cura = `8cb0ab9e1`; `load < 4` antes e depois de cada corrida, todas abaixo de
`1`; [`mede_sonda_das_estrelas.sh`](ferramentas/mede_sonda_das_estrelas.sh) com os binários intercalados;
placa · Vello, ms, média de `2`):

| arranjo | iGPU antes do §9.9 | iGPU cura | RTX antes do §9.9 | RTX cura |
|---|---:|---:|---:|---:|
| `72` grandes esticadas | `1,74` · `1,00` | **`1,74`** · `1,01` | `0,35` · `0,37` | **`0,33`** · `0,36` |
| `72` grandes conformes | `0,93` · `0,94` | **`0,93`** · `0,92` | `0,19` · `0,37` | **`0,17`** · `0,34` |
| `1225` densas da `=127` | `1,69` · `2,69` | **`1,69`** · `2,70` | `0,22` · `0,49` | **`0,20`** · `0,49` |

**A ablação** (iGPU, `PH2D_FLUID_PROFILE=1`, o relógio do passe `render.formas`, ms; os binários
mutilados não vão a commit — A1: o fragmento devolve uma cor constante; A2: lê o registo da célula e
devolve só o FUNDO, sem o laço das listas):

| arranjo | desenho inteiro | A2 (registo + fundo) | A1 (piso: vértice, rasterização, mistura) |
|---|---:|---:|---:|
| esticadas | `0,81` | `0,13` | `0,09` |
| conformes | `0,47` | `0,13` | `0,09` |
| densas | `0,88` | `0,11` | `0,07` |

⇒ **o laço das listas é `0,68` dos `0,81 ms`** (e `0,77` dos `0,88` nas densas): cada pixel de uma célula
de `32 px` percorre a lista INTEIRA da célula, e a aresta que só cruza um pixel é lida e avaliada pelos
`32`. O piso do hardware é `0,09`; o registo, `0,04`. O resto do quadro é o cálculo: células `0,43` ·
escrita `0,17` · contagem `0,07`.

**O desenho seguinte (✅ construído e medido no §9.12 — passou à 1.ª tentativa).** Não o rasterizador fino inteiro no cálculo que o
handoff de 02/10 previa: esse pedia a ORDEM entre cópias, porque a mistura passava ao cálculo. Basta tirar
do fragmento a SOMA, e deixar-lhe a mistura (o hardware já a faz na ordem certa):

- **o buffer de ACUMULAÇÃO por pixel** da grelha de células de cada cópia (o `accumulation buffer` dos
  rasterizadores de linhas de varrimento): cada aresta, na fileira, deposita a área do pixel que cruza e o
  RESTO (`dy − área`) no pixel seguinte — por atómicos em ponto fixo (`ESCALA_FIXA`), logo a ordem dos
  fios não muda um bit, como hoje. Uma aresta paga os pixels que CRUZA, não `32` por célula;
- **a varredura por célula** (um fio por célula): o fundo da célula (o prefixo que o `cs_lugar_das_listas`
  já faz) mais o prefixo dos depósitos dela dá a cobertura de cada pixel; as três famílias acabadas
  (`af` com a regra, `as_` com a soma do contorno e das marcas) gravam-se numa palavra;
- **o fragmento**: uma leitura, e a mistura de sempre; a fileira que não coube continua a refazer-se pelo
  caminho de sempre (o `SEM_LISTA` de agora, a mesma capacidade medida dois quadros depois).

⚠️ A lista por célula sai do caminho das células (as `68 276` entradas das esticadas viram depósitos);
recontar a capacidade e as guardas do §9.8 (a cena que muda, as fileiras que não cabem) na mesma
jornada.

**Kill-criterion (DIRETIVA §5, escrito ANTES):** na iGPU, `PH2D_FLUID_PROFILE=1`, `load < 4`, as `72`
esticadas com o quadro da placa (soma dos passes) **≤ `1,0 ms`** (o Vello inteiro) depois da 2.ª
tentativa, e nenhuma das outras duas pior que hoje (`0,76` · `1,50` de soma dos passes); na RTX nenhum
arranjo mais de `10 %` pior. Falhou ⇒ fica a lista, e o desenho entra aqui como recusa medida. É a 3.ª
topologia do caminho das células (§9.6 máscaras → §9.8 listas → acumulação): a prova do modelo é esta
ablação — o fragmento só com o registo custa `0,13` — e o que falta provar é o PREÇO do cálculo novo.

### §9.12 — O BUFFER DE ACUMULAÇÃO: o plano (2026-10-03, escrito ANTES de construir)

Comparação: o binário da sonda de `8cb0ab9e1` (a cura), compilado e copiado antes de mexer no shader.

**A lei que se preserva.** Hoje um pixel `x` de uma célula `k` soma `fundo(k) + Σ fixo(c(e, x))` sobre a
lista da célula (`c` = `contribuicao`). Para cada aresta numa fileira, `c(x)` é `0` à esquerda do
pedaço recortado à fileira `[xa, xb]`, `dy` a partir de `ceil(xb)`, e só varia nos pixels que ele
CRUZA. ⇒ cada aresta deposita, por pixel `x ∈ [floor(xa), ceil(xb))`, `fixo(c(x)) − fixo(c(x − 1))`, e
`fixo(dy) − fixo(c(ceil(xb) − 1))` em `ceil(xb)`; o prefixo dos depósitos dá `Σ fixo(c(e, x))`. Os
depósitos param na célula `kb` (a primeira cujo início `≥ xb`): dali para a frente a aresta é FUNDO,
como hoje. O prefixo RECOMEÇA em cada célula (o primeiro pixel de uma célula recebe `fixo(c(x))`
inteiro, não a diferença), porque o fundo da célula só conta as arestas que acabam ANTES dela. Em
inteiros a soma não depende da ordem dos fios: determinístico, como hoje. ⚠️ O pedaço é recortado à
fileira (uma divisão por aresta, não por fileira): uma aresta paga os pixels que cruza, não `32` por
célula. O resto `~1e-6` da `contribuicao` à esquerda do pedaço recortado (o do Vello) deixa de entrar
— `≤ 1` unidade de `2¹⁶`, dentro do `ALFA_MAX = 2`.

**Os passes** (os quatro despachos indirectos ficam quatro; o `cs_conta`, o `cs_soma` e o `cs_escreve`
ficam, com a unidade das células em CÉLULAS e não em palavras):

| passe | fio por | faz |
|---|---|---|
| `cs_zera` | PIXEL de célula | apaga os três acumuladores do pixel; os fios `0..3` da célula apagam o fundo |
| `cs_deposita` | ARESTA | por fileira: o fundo da célula `kb` (atómico, como hoje) e os depósitos `[floor(xa), ceil(xb)]` nas células `< kb` |
| `cs_fundo` | FILEIRA | o prefixo dos fundos ao longo da fileira (a metade de cima do `cs_lugar_das_listas`) |
| `cs_varre` | PIXEL (grupos de `64` = duas células) | prefixo SEGMENTADO por célula em memória de grupo (`5` passos), mais o fundo, as duas regras (`af` com a da cópia, `as_` com a soma das marcas e do contorno) e grava `pack2x16unorm(af, as_)` |

O fragmento: `(c1.x + r · células) · 32 + (x − x0)`, UMA leitura e `unpack2x16unorm`; as regras saem
dele para esta via (o caminho de sempre continua a aplicá-las).

**Os buffers** (por célula, indexados pelo número GLOBAL da célula — o prefixo das células de cada
cópia já é contíguo): `celulas` `4` palavras (os três fundos, `atomic`) · `acumula` `96` palavras (`3`
famílias × `32` pixels, família a família, para a leitura do `cs_varre` ser coalescida) · `cobertura`
`32` palavras. Saem as `listas` e o `lista_total`. A regra da cópia (`even_odd`) passa ao `cs_escreve`
(`ccopias[3·ii + 2].z`, livre). O `despacho` ganha `[6, 9)`: um fio por pixel de célula.

**A capacidade:** uma só, a das células (contada no `cs_conta`, lida dois quadros depois, como as
arestas). O tecto do recurso é o `acumula` (`max_storage_buffer_binding_size / 384 B`). Uma cópia que
não cabe vai INTEIRA pelo caminho de sempre (o `cs_escreve` já a recusa). ⇒ **não há fileira que não
caiba**: o `SEM_LISTA` e o «dois quadros depois das arestas» deixam de existir, e o regime do produto
volta a medir-se (o `QUADROS_DO_PRODUTO = 5` era o das listas).

**As guardas do §9.8, recontadas:**

- `as_fileiras_que_nao_cabem_nas_listas_desenham_o_mesmo` → **`as_copias_que_nao_cabem_nas_celulas_desenham_o_mesmo`**
  (tecto em metade das células pedidas: as que cabem pelas células, as outras pelo caminho de sempre,
  a mesma imagem; controlo: o tecto morde E alguma cópia continua nas células). Porta
  `ShapePass::limita_as_celulas` (sai `limita_as_listas`); instrumento `celulas_do_ultimo_quadro`.
- `uma_cena_que_muda_nao_le_as_arestas_do_quadro_anterior` fica — e passa a guardar também o
  `cs_zera` (um acumulador que traz o quadro anterior).
- A mutação `14/14` das listas morre com o código dela: um arnês novo
  (`mutacao_o_buffer_de_acumulacao_2026-10-03.py`) com os análogos — fundo na `kb − 1`, o 1.º pixel
  cruzado perdido, o degrau final perdido, o prefixo que não recomeça na célula, depósitos em `≥ kb`,
  `cs_fundo` sem prefixo, `cs_zera` mudo, varredura não segmentada, `even_odd` ignorado, escala
  dobrada, `min/max` do recorte trocados, sem a verificação da capacidade, passo da fileira errado no
  fragmento, preenchimento e contorno trocados.

**As duas tentativas** (o kill-criterion do §9.11 conta-se depois da 2.ª):

1. a DENSA acima (zera e varre todo pixel de célula);
2. se a 1.ª falhar: a ESPARSA — uma marca por célula TOCADA (o `cs_deposita` a põe); o `cs_varre` só
   corre nas tocadas e devolve-lhes os acumuladores a zero (sai o `cs_zera`), e uma célula sem
   depósitos é constante — o fragmento lê-a do registo. Paga uma leitura encadeada a mais nas bordas.

**Réguas:** `registos_dos_shaders.sh` (o fragmento tem de descer dos `56` VGPRs) · os `10` gates GPU da
crate e os `5` do produto · a sonda intercalada com `PERFIL=1 PLACAS=igpu` (e `rtx`) · a mutação nova
e, no fecho, a `21/21` do tracejado.

**✅ O resultado — a 1.ª tentativa (a DENSA) passa o kill-criterion** (commit `cd3ec059e`; o binário de
`8cb0ab9e1` intercalado na MESMA janela, `PH2D_FLUID_PROFILE=1`, `2` corridas por célula, a soma dos
passes do perfilador, ms):

| arranjo | iGPU cura | iGPU acumulação | critério | RTX cura | RTX acumulação |
|---|---:|---:|---:|---:|---:|
| `72` grandes esticadas | `1,50` · `1,52` | **`0,88`** · `0,98`¹ | `≤ 1,0` ✅ | `0,25` · `0,26` | **`0,22`** · `0,22` |
| `72` grandes conformes | `0,76` · `0,76` | **`0,59`** · `0,59` | `≤ 0,76` ✅ | `0,10` · `0,10` | **`0,09`** · `0,10` |
| `1225` densas da `=127` | `1,50` · `1,50` | **`1,00`** · `1,00` | `≤ 1,50` ✅ | `0,11` · `0,11` | **`0,10`** · `0,10` |

¹ a 2.ª corrida das esticadas caiu numa janela com a média de 5 min a `8,5` (outra linha); a 1.ª a `1,8`.

A decomposição iGPU das esticadas: desenho `0,83 → 0,14` (o fragmento só lê — o piso da ablação era
`0,13`) · células `0,43 → 0,50` · escrita `0,17` e contagem `0,07` iguais. ⇒ **o preço do cálculo novo
é `+0,07 ms` para tirar `0,69` ao desenho.** No relógio de parede da sonda as esticadas ficam em
`0,98` contra `0,94` do Vello (antes `1,64` contra `1,13`): o último arranjo em que o proxy de telemóvel
perdia para o Vello passa a empatar. As conformes ficam `0,70` contra `0,87`; as densas `1,13` contra
`2,65` (`2,3×`).

⚠️ **Onde a densa paga:** nas densas as células sobem `0,51 → 0,78 ms` — a `cs_zera` e a `cs_varre`
correm em TODO pixel de célula, e `1225` cópias sobrepostas são muitos pixels de grelha por pixel de
ecrã. A 2.ª tentativa (a ESPARSA, acima) é exactamente esta alavanca: fica como item medido, não como
pendência do critério. A memória: as esticadas pedem `10 362` células (`~5,5 MB`, `528 B` por célula);
o gate das estrelas sobrepostas a `512²`, `77 502` (`~41 MB`) — o tecto do recurso continua a ser o
`max_storage_buffer_binding_size`, e quem não cabe vai por cópia.

**Registos** (`registos_dos_shaders.sh`, iGPU): o fragmento continua a `56` VGPRs · `18` ondas (o caminho
de sempre está no MESMO shader e manda nos registos), código `16 620 → 15 336 B`; os quatro kernels novos
`≤ 32` VGPRs, nada em scratch.

**Gates** (RTX): `ph2d-shape-gpu` **10/10** — todos os quadros de todas as fixturas com `alfa ≤ 1` e `0`
pixels `> 1` contra o eixo, inclusive o 1.º (a mistura com o caminho de sempre) e a metade das células
(`as_copias_que_nao_cabem_nas_celulas_desenham_o_mesmo`: `21/40`, `2/6` e `26/60` cópias nas células, o
resto por cópia, a mesma imagem); o tracejado contra o Vello no mesmo `71` do §9.9 · produto **5/5** +
sonda.

**Mutação `17` de `17`** ([arnês](ferramentas/mutacao_o_buffer_de_acumulacao_2026-10-03.py), pré-voo `17/17`,
corrida LIMPA `10` verdes, nenhuma por shader inválido): fundo uma célula depois da `kb` · o 1.º pixel
cruzado perdido · o degrau final até `dy` perdido · o prefixo que não recomeça na célula · `cs_fundo` sem
prefixo · `cs_zera` sem apagar a acumulação · sem apagar o fundo · varredura não segmentada · a regra
par-ímpar ignorada no `cs_varre` · e no `cs_escreve` · escala dobrada · o recorte com o mínimo trocado ·
sem a verificação da capacidade · o passo da fileira no fragmento · preenchimento e contorno trocados ·
a `kb` por `floor` · o depósito sem a diferença. As duas do `cs_zera` morrem no gate da cena que muda
(o análogo da M10 do §9.8) e no da metade das células. O arnês das listas (`14/14`) foi aposentado com
o código que mutava.

**A memória no APP — medida, e o defeito que a medição achou** (W5, `mede_formas_na_placa.sh`, o relato
`[formas] celulas:` sob `PH2D_FLUID_PROFILE=1`, janela `1930 × 1040`). ⛔ A 1.ª corrida pediu `132 MB`
na `=127` densa e `264 MB` na escada de `32 768` — e as «pedidas» eram EXACTAMENTE `16 ×` as cópias: era
o palpite de fábrica por cópia (`CELULAS_POR_COPIA_INICIAL = 16`), que entrava num `max` e nunca mais
saía (o comentário dizia «a leitura do total substitui-o»; o código guardava o maior dos dois). Com as
listas o mesmo palpite custava `~384 B` por cópia; com a acumulação, `8,4 KB`. ✅ **Cura:** as células
só têm a capacidade MEDIDA (os dois primeiros quadros de uma cena nova vão pelo caminho de sempre — a
mesma imagem, os gates `10/10` e `5/5` re-corridos). Re-medido:

| cena (app) | células pedidas | capacidade | quadro iGPU com · sem placa | CPU (encode) com · sem |
|---|---:|---:|---:|---:|
| escada, `4 096` · `16 384` · `32 768` | `0` (as conformes pequenas não vão às células) | — | `60 fps` nas três | `~3,1` · `4,1`–`17,8` |
| `=127` densa, `16 384` | `107 520` (`6,6` por cópia) | `131 072` · **`66 MB`** | **`16,6`** · `20,8` ms | `3,1` · `8,2` ms |

⇒ no proxy de telemóvel a `=127` densa passa a `60 fps` (as listas davam `17,6 ms`). Os `66 MB` são
`528 B` por célula de `32 px` (`16,5 B` por pixel de grelha) e a potência de dois.

⛔ **RECUSA MEDIDA — estreitar a célula para a acumulação** (a hipótese era que a grelha de `32` de uma
estrela de `~14 px` fosse mais de metade fora da forma). Sonda intercalada, `PH2D_FLUID_PROFILE=1`, `2`
corridas, soma dos passes · células · pixels de grelha:

| largura | iGPU esticadas | iGPU conformes | iGPU densas | RTX (os três) | grelha esticadas · densas |
|---|---:|---:|---:|---:|---:|
| **`32`** | **`0,88`** · `0,50` | **`0,59`** · `0,37` | `1,00` · `0,78` | `0,22` · `0,09` · `0,10` | `331 k` · `470 k` px |
| `16` | `0,87` · `0,49` | `0,60` · `0,38` | `1,00` · `0,78` | `0,22` · `0,09` · `0,09` | `301 k` · `470 k` |
| `8` | `0,97`–`1,06` · `0,59`–`0,67` | `0,73` · `0,52` | `0,94` · `0,71` | `0,22` · `0,10` · `0,09` | `294 k` · `353 k` |

⇒ `16` empata (a grelha das densas nem muda: a caixa estimada com a folga já cabe em `16`); `8` só ganha
`0,06 ms` nas densas e perde `0,09`–`0,18` nas esticadas. Fica `32`, como no §9.8. A memória não se
cura pela largura: as alavancas que ficam são a cobertura gravada NO LUGAR do 1.º acumulador
(`−4 B` por pixel, `−24 %`) e um arredondamento da capacidade mais fino que a potência de dois.

✅ **A MEMÓRIA DAS CÉLULAS: `66 → 43 MB` na `=127` densa** (commit `9d1058a40`; kill-criterion escrito
antes: capacidade `≤ 46 MB` no app, e nenhum arranjo pior que `+5 %` na iGPU nem `+10 %` na RTX na soma
dos passes, contra o binário de `fab8999a8` intercalado na MESMA janela). As duas alavancas acima:

1. **A cobertura NO LUGAR do 1.º acumulador.** O `cs_varre` grava o `pack2x16unorm` na palavra
   `acumula[cel·ACUMULA + p]` (o depósito do preenchimento do PRÓPRIO pixel), que ele já leu antes das
   barreiras: nenhum outro fio a lê (o prefixo vive na memória de grupo) e o `cs_zera` do quadro
   seguinte apaga-a com as outras. Sai o buffer `cobertura` (ligação `9` do grupo `2`); a ligação `2`
   do grupo `1` do desenho passa a ser a acumulação, `read` no passe de desenho e `read_write` no de
   cálculo — passes separados, o regime que a `cobertura` já tinha. `528 → 400 B` por célula.
   ⚠️ **O fragmento parte a coluna:** `x` corre a FILEIRA inteira, e com a cobertura contígua
   `primeira·32 + x` caía sozinho na célula `primeira + x/32`; com o passo `ACUMULA` é
   `(primeira + x/32)·ACUMULA + x%32`. O 1.º rascunho (`primeira·ACUMULA + x`) pôs **`7` dos `10`**
   gates vermelhos — virou a mutação A19.
2. **A capacidade ao OITAVO do degrau** (`ao_oitavo_do_degrau` em `contorno.rs`): o múltiplo seguinte
   de `2^(⌊log₂ n⌋ − 3)`, só para as células (crescem só por medição). `107 520 → 114 688` em vez de
   `131 072`. ⚠️ **O preço, medido:** numa cena que cresce UMA célula de cada vez até `107 520` (o pior
   caso: cada medição passa a capacidade) os buffers recriam-se `118` vezes contra `18` — no máximo
   `8` por oitava, com gate (`numa_cena_que_cresce_as_celulas_recriam_se_no_maximo_oito_vezes_por_oitava`).
   Na `=127` do app a contagem não cresce: UMA criação, como antes.

| medida | antes (`fab8999a8`) | depois |
|---|---:|---:|
| app `=127` densa (iGPU e RTX): capacidade | `131 072` · **`66 MB`** | `114 688` · **`43 MB`** (`45,9 × 10⁶ B`) |
| app `=127` densa: quadro iGPU · RTX | `16,6 ms` · `16,7 ms` | `16,6`–`16,8 ms` · `16,6 ms` (`60 fps`) |
| sonda: memória das esticadas · conformes · densas | `8` · `8` · `8 MB` (`16 384`) | `4` · `3` · `5 MB` |
| iGPU esticadas (soma dos passes, ms) | `0,88` · `0,89` | `0,89` · `0,90` (`+1 %`) ✅ |
| iGPU conformes | `0,61` · `0,61` | `0,59` · `0,59` ✅ |
| iGPU densas | `1,01` · `1,01` | `1,01` · `1,01` ✅ |
| RTX esticadas | `0,22` ×6 | `0,22` ×6 ✅ |
| RTX conformes | `0,09` ×5 · `0,10` ×1 | `0,09` ×5 · `0,10` ×1 ✅¹ |
| RTX densas | `0,10` ×6 | `0,10` ×6 ✅ |

¹ o `0,10` aparece UMA vez em cada binário (o de depois na 1.ª janela, o de antes na repetição): é o degrau de `0,01 ms` do perfilador, `11 %` de `0,09`; o `gpu-busy` dá `0,10` igual nos
dois. Cargas: a 1.ª janela com a média de 5 min entre `3,5` e `8,8` (duas outras linhas a testar); a
repetição da RTX (`CORRIDAS=4`) entre `0,4` e `2,4`.

**Registos** (`registos_dos_shaders.sh`, iGPU, os dois binários): o fragmento continua `56` VGPRs · `18`
ondas · `15 336 B` (a partição da coluna em célula e pixel não custa registo); o `cs_varre`
`1 008 → 1 000 B`; nada em scratch. **Gates** (RTX): `ph2d-shape-gpu` `10/10` + `3` unitários novos da
capacidade, produto `5/5` + sonda, `ph2d-gpu-cook` formas `2/2`. **Mutação `19` de `19`** (o
[arnês](ferramentas/mutacao_o_buffer_de_acumulacao_2026-10-03.py) com a A13 re-ancorada e duas novas: A18 a
cobertura gravada na palavra das marcas · A19 a coluna sem a partição; pré-voo `19/19`, corrida limpa `10`
verdes, nenhuma por shader inválido) e a do tracejado `21/21`.

### §9.13 — A VARIANTE COMPLETA ENCOLHIDA e a variante ESPARSA medida (2026-10-03, escrito ANTES de construir)

Ordem do dono (03/10, antes de integrar): os itens 3 e 4 do §6 do
[handoff de 03/10](handoffs/HANDOFF_INTEGRACAO_line_motion_value_2026-10-03.md). Comparação: o binário da
sonda de `ed23a77b1` (`antes`), com a sonda nova `PH2D_SONDA_TRACEJADO=1` (o `Dash 2` · `Dash Gap 1,5` da
`=127` tracejada, no contorno de cada arranjo) e o instrumento `celulas_tocadas_do_ultimo_quadro`.

**A régua reproduzida, sem relógio** (`registos_dos_shaders.sh`, iGPU, `antes`): fragmento enxuta `56`
VGPRs · `18` ondas · `15 336 B`, completa **`128` · `8`** · `34 428 B`; `cs_escreve` enxuta `64` · `16`,
completa **`128` · `8`** · `28 136 B`; `cs_conta` `40` · `24` e `48` · `20`. Nada em scratch.

**O relógio de partida** (iGPU, `PH2D_FLUID_PROFILE=1`, o regime — a 2.ª janela de `120` quadros; a 1.ª
inclui os dois quadros pixel a pixel de antes da capacidade medida; ms):

| arranjo | conta | escreve | células | desenho | soma | Vello (parede) |
|---|---:|---:|---:|---:|---:|---:|
| `72` esticadas, TRACEJADAS | `0,17` | `0,32` | `0,63` | `0,22` | `1,34` | `0,95` |
| `72` esticadas, contínuas (§9.12) | `0,07` | `0,17` | `0,50` | `0,14` | `0,88` | `0,94` |

⇒ o tracejado não cobra só no desenho: a contagem e a escrita correm UM fio por cópia, e com `72`
cópias são duas ondas na placa inteira — ali a ocupação não decide nada, decide o trabalho EM SÉRIE de
cada fio. A contagem percorre o eixo três vezes (o `ajuste_do_tracejado` com o seu laço interior e o
`arco` de cada troço) e a escrita quatro (o ajuste outra vez, o `sub_tracejado` de cada fechado, o
percurso). O desenho (`0,22` contra `0,14`) é o único dos quatro que é a ocupação: no regime nenhuma
cópia passa pelo pixel a pixel, e mesmo assim o fragmento corre com os registos dele.

**(3a) O fragmento — a placa escolhe a variante por quadro.** O `cs_escreve` é o único que sabe se uma
cópia TRACEJADA vai pelo pixel a pixel (os dois primeiros quadros, a capacidade, a escrita recusada). O
desenho passa a dois `draw_indirect` com os mesmos argumentos — o da ENXUTA e o da COMPLETA — e só um
deles tem cópias: o `cs_soma` põe `n` cópias na enxuta e `0` na completa; uma cópia tracejada, VISÍVEL,
que fica sem células troca-os (atómicos, o mesmo valor de todos os fios: a ordem não importa). Uma só
chamada desenha todas as cópias, logo a ordem da mistura é a de sempre. Os argumentos vivem no buffer do
`despacho` (palavras `[9, 17)`), que já é `INDIRECT`. Sem tracejado carregado, o desenho de hoje.

**Kill-criterion (3a):** (i) sem relógio — no regime da cena tracejada o fragmento que corre é o da
enxuta (`56` · `18`); um instrumento lê os argumentos e um gate o prova, com CONTROLO (o 1.º quadro, sem
capacidade, desenha pela completa); (ii) a imagem igual: os `10` gates GPU da crate, os `5` do produto e
as mutações do tracejado (`21/21`) e da acumulação (`19/19`); (iii) o relógio — sonda intercalada,
`load < 4`, iGPU e RTX: o desenho das tracejadas a `≤ +10 %` do das contínuas do mesmo arranjo, e
nenhum arranjo pior que `+5 %` (iGPU) / `+10 %` (RTX) na soma dos passes. Falhou ⇒ recusa medida aqui.

**(3b) A contagem e a escrita — menos voltas ao eixo por fio.** Dois pedaços, medidos um a um:

1. **o limite da contagem sem o ajuste.** O `limite_de_arestas` só precisa de um TECTO de peças por
   troço, e o ajuste nunca encurta o período mais que meia peça por troço: com `L` o sub-caminho mais
   longo (o que o ajuste fecha), `per` o período da caneta e `len ≤ L` o arco do troço, o período
   ajustado é `per · L / d` com `d ≤ L + per/2` (o arredondamento de `L/per`, fechado ou aberto) — ou,
   quando o arredondamento dá zero, `≥ L ≥ len` (uma peça). ⇒ `len / per_ajustado ≤ len/per + 1/2`,
   e o tecto passa a `⌈len/per + 1/2⌉ + 2` sem o ajuste: a contagem percorre o eixo UMA vez (era
   três). A escrita continua a usar o ajuste exacto; o tecto só reserva;
2. **o ajuste numa volta só.** O `ajuste_do_tracejado` andava cada sub-caminho a partir do seu troço de
   início (o laço interior com o `proximo_troco`), por cima do laço de todos os itens: cada troço lido
   duas vezes. Os troços de um sub-caminho são contíguos (só cabeçalhos de bloco no meio), logo uma
   volta acumula o arco e fecha o sub-caminho quando chega o seguinte.

**Kill-criterion (3b):** (i) a imagem igual — os mesmos gates e mutações do (3a); (ii) o relógio, sonda
intercalada contra o binário do (3a), iGPU, `load < 4`: nas `72` esticadas tracejadas `conta + escreve`
`≤ 0,40 ms` (de `0,49`, `−18 %`), e nenhum arranjo pior que `+5 %` (iGPU) / `+10 %` (RTX) na soma dos
passes; (iii) sem relógio — o `cs_conta` da completa não sobe dos `48` VGPRs nem o `cs_escreve` dos
`128`. ⚠️ Os registos do `cs_escreve` NÃO são alvo deste item: com `72` cópias ele é DUAS ondas na
placa inteira, com `1 225` são `20` para `24` SIMDs — a ocupação não tem o que esconder, e o que conta é
a cadeia em série de cada fio. Pedaço que não mexe o relógio sai (recusa medida aqui).

**✅ O resultado do (3) — os dois kill-criteria PASSAM** (commit `b723b02d1`; os quatro binários
intercalados na MESMA janela — `antes` · `3a` · `3b1` · `3b2` —, `PH2D_FLUID_PROFILE=1`, `2` corridas por
célula, o regime, a soma dos passes, ms; as duas corridas de cada célula iguais ao `0,01` salvo nota):

| arranjo TRACEJADO | iGPU antes | 3a | 3b1 | **3b2** | RTX antes | **3b2** |
|---|---:|---:|---:|---:|---:|---:|
| `72` esticadas | `1,33` | `1,27`–`1,30` | `1,20` | **`1,17`** (`−12 %`) | `0,42`–`0,43` | **`0,37`** (`−13 %`) |
| `72` conformes | `0,61` | `0,55` | `0,55`¹ | **`0,55`** (`−10 %`) | `0,09` | **`0,09`** |
| `1 225` densas | `1,45` | `1,42` | `1,33` | **`1,32`** (`−9 %`) | `0,12` | **`0,12`** |

¹ a 2.ª corrida leu `0,63` numa janela com a média de 1 min a `1,3` e a de 5 a subir (outra linha).

- **(3a)** o desenho das tracejadas `0,22 → 0,15` (esticadas) · `0,21 → 0,15` (conformes — o eixo delas é
  tracejado mesmo sem o usarem, e pagavam a completa) · `0,15 → 0,11` (densas); as MESMAS cenas sem
  tracejado, na mesma janela: `0,15` · `0,15` · `0,11` ⇒ o critério «`≤ +10 %` das contínuas» passa a
  zero. O 1.º e o 2.º quadro de uma cena nova desenham pela completa (o gate), o resto pela enxuta.
- **(3b)** `conta + escreve` nas esticadas `0,49 → 0,40` (o critério: `≤ 0,40`); o `cs_conta` da completa
  `48 · 20 → 40 · 24` (`4 716 → 3 940 B`). ⚠️ **Os relógios POR passe mentem na fronteira:** nas densas a
  «escrita» caiu `0,31 → 0,17` com o (3b1), que só mexeu na contagem — e o `cs_escreve` dos dois
  binários é o MESMO (`128` · `8`, `30 204 B`). O carimbo de início de um passe sai antes da barreira
  que o separa do anterior, e a cauda da contagem caía na conta da escrita. ⇒ a régua é a SOMA; os
  relógios de dois passes vizinhos só valem somados.
- **Sem tracejado** (o controlo, `antes` contra `3b2`, iGPU): `0,89 → 0,88` · `0,59 = 0,59` · `1,01 = 1,01`.
- **Parede da sonda (iGPU)** — as tracejadas esticadas continuam atrás do Vello (`1,57` contra `0,90`):
  as `0,62 ms` de células são as arestas dos traços (as mesmas estrelas contínuas pedem `0,49`) e a
  escrita é UM fio por cópia (`72` fios). O que fica é a topologia da escrita (um fio por troço, com o
  prefixo do arco), não os registos — item próprio, não deste fecho. As densas `1,48` contra `3,25`
  (`2,2×`) e as conformes `0,66` contra `0,85` ganham.

**Mutação `6` de `8`** ([arnês](ferramentas/mutacao_a_variante_da_placa_2026-10-03.py), pré-voo `8/8`,
corrida LIMPA `17` verdes): nenhuma cópia pede a completa · a completa sempre (só o gate novo a vê) · o
teste de fora do ecrã invertido · os dois desenhos trocados · o ajuste que não recomeça a soma em cada
sub-caminho (o gate contra o Vello; a estrela com FURO é a única fixtura com dois sub-caminhos) · o
ajuste com um troço a menos —
SANGRAM. ⚠️ **Sobrevivem, e é medido porquê:** o tecto da contagem sem a meia peça (W6) e com METADE das
peças (W7). O orçamento de arestas por peça (`4 + 2 · ponta + junta`) é tão folgado que nem metade das
peças falta em fixtura nenhuma: o tecto exacto fica guardado pela PROVA acima, e um tecto curto não pinta
mal — a escrita recusa a cópia, ela vai pixel a pixel e a placa desenha pela completa (a mesma imagem;
o gate novo veria a completa no regime). A V7 do arnês da variante enxuta (§9.10) foi re-ancorada no
desenho novo.

**O ajuste numa volta é o MESMO ao bit:** as imagens do passe nas `7` famílias do gate do tracejado contra
o Vello, com o `shape.wgsl` de antes e o de depois (o resto igual), são iguais byte a byte.

**(4) A variante ESPARSA — a fracção de células TOCADAS, medida** (instrumento
`celulas_tocadas_do_ultimo_quadro`, o binário do (3b), o último quadro da sonda; cota por baixo):

| arranjo | células em uso | tocadas, contínuo | tocadas, tracejado |
|---|---:|---:|---:|
| `72` grandes esticadas | `10 362` | `6 243` (**`60 %`**) | `5 962` (`58 %`) |
| `72` grandes conformes | `8 916` | `8 640` (`97 %`) | `8 629` (`97 %`) |
| `1 225` densas da `=127` | `14 700` | `8 820` (**`60 %`**) | `8 330` (`57 %`) |
| escada de `32 768` (app, `=17`) | `0` — as conformes pequenas não vão às células (§9.12) | — | — |

⛔ **A premissa do handoff caiu:** nas densas NÃO é «quase toda célula tocada» — `40 %` das células de uma
estrela de `~14 px` numa grelha de `32 px` só têm fundo (a caixa com a folga e a borda da estrela não
enchem a célula). A alavanca existe nas densas e nas esticadas; nas conformes grandes não.

**Kill-criterion (4), escrito ANTES de construir.** A esparsa poupa, no MÁXIMO, o `cs_zera` inteiro e o
`cs_varre` das células não tocadas — `teto = t_zera + (1 − f) · t_varre` — e paga as marcas no
`cs_deposita` (um atómico por célula cruzada), a lista compacta das tocadas, o apagar das tocadas do
quadro ANTERIOR (a cobertura vive no lugar do 1.º acumulador) e, no fragmento, uma leitura encadeada a
mais nas tocadas e as regras aplicadas ao fundo nas outras. ⇒ (i) **sem construir:** mede-se `t_zera` e
`t_varre` por ablação (binários mutilados, fora de commit, como no §9.11 — a soma dos passes com e sem o
despacho; os relógios POR passe não servem, ver abaixo); se o teto nas densas da iGPU for `< 10 %` da
soma dos passes, a variante é RECUSADA aqui com a tabela; (ii) se construída: densas iGPU `−10 %` na
soma dos passes, nenhum arranjo pior que `+5 %` (iGPU) / `+10 %` (RTX), a memória da `=127` densa no app
`≤ 43 MB`, a imagem igual (os gates e as duas mutações) e uma mutação nova a sangrar.

⛔ **RECUSA MEDIDA — a variante ESPARSA das células** (2026-10-04; binários mutilados fora de commit, o
de `b723b02d1` como base, intercalados na MESMA janela, iGPU, `PH2D_FLUID_PROFILE=1`, `2` corridas
iguais ao `0,01`, cargas `0,0`–`2,8`; a soma dos passes, ms):

| arranjo (contínuo) | base | sem `cs_zera` | sem `cs_varre` | `f` tocadas | teto `t_z + (1−f)·t_v` | + marcas e lista | + fragmento marca/fundo | líquido esperado |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `1 225` densas | `1,01` | `0,91` | `0,77` | `60 %` | `0,20` (`19 %`) | `+0,05` | `+0,02` | **`~0,10` (`≤ 10 %`)** |
| `72` esticadas | `0,88` | `0,82` | `0,71` | `60 %` | `0,13` (`15 %`) | `+0,04` | `+0,02` | `~0,05` (`6 %`) |
| `72` conformes | `0,59` | `0,54` | `0,44` | `97 %` | `0,05` (`9 %`) | `+0,05` | `+0,01` | **`−0,02` (pior)** |

O teto passa o filtro (i), por isso os custos fixos foram medidos antes de construir: «+ marcas e lista» =
um `atomicOr` na palavra da regra por célula que a aresta cruza e um `atomicAdd` de compactação na 1.ª
marca; «+ fragmento» = a leitura encadeada da marca e, nas não tocadas, as três palavras do fundo com as
regras. O líquido esperado conta só `77 %` do `cs_zera` (o apagar não sai inteiro: a cobertura do quadro
anterior vive no 1.º acumulador das tocadas e o registo de toda célula continua a apagar-se) e não conta
o despacho que lê o total da lista. ⇒ no MELHOR caso as densas ficam na fronteira dos `10 %`, as
esticadas abaixo e as conformes PIORAM — e a lista das tocadas custa `4 B` por célula (`43 → 44 MB` na
`=127` densa do app), o que viola o critério (ii) por construção. **Fica a densa.** Para quem voltar a
isto: a alavanca que sobra nas células é o `cs_varre` (`0,24 ms` nas densas, um fio por pixel com os cinco
passos do prefixo em memória de grupo), não o apagar.

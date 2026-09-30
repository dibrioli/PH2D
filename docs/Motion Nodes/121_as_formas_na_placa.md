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
⛔ O traço **tracejado** sob afim não conforme continua no Vello (§9).

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
  cenas com forma vão à placa.

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
- **Tracejado sob escala não-uniforme fica no Vello** (CPU e dispositivo) — o tracejado no
  dispositivo pede o comprimento de arco por cópia, que é wave própria.
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

⏳ **Por fazer:** W0 (a medição de partida, que a placa ocupada adiou) e W5 (medição de fecho em
`release` + smoke do dono com formas e simulação com campos, fotografado antes) · os glifos do
`source.text` · o tracejado no dispositivo.

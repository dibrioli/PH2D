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

⚠️ **E um caso que o passe desenha com divergência DECLARADA:** traço + afim NÃO conforme (escala
não-uniforme). A lei do dono (bug #27: *«quando engrossa, engrossa por igual nos dois eixos»*) pede
a caneta no MUNDO; o traço expandido no espaço LOCAL daria uma caneta elíptica. ⇒ essas cópias
desenham o traço por **distância ao eixo no ecrã** (caneta redonda certa, juntas REDONDAS em vez de
esquadria). Tudo o resto — o preenchimento, e o traço de toda cópia conforme — é a conta do Vello.

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
grupo), uma tinta própria ou um traço sob afim NÃO conforme devolvem o quadro inteiro ao Vello —
partir a lista trocaria a ordem entre as duas metades.

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

⏳ **Por fazer:** W3 (a rota do dispositivo — hoje um grafo com forma viva continua a cozer na CPU) ·
W4 (o traço sob afim não conforme, os glifos) · W5 (a medição de fecho e o smoke do dono).

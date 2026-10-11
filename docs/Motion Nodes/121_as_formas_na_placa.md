# 121 — AS FORMAS NA PLACA: toda forma do Motion desenhada pelo dispositivo, nítida e sem custo por cópia

> **Índice** — o doc foi partido em 2026-10-11 (`doc-split`, sha256: nenhuma linha perdida). Aqui ficam o
> plano (§1–§5) e o §9.24, o último aberto; a história medida está em `archive/121_as_formas_na_placa/`
> (⛔ «Recusas MEDIDAS» do §9.7 → `parte_2`). Cada pedaço ≤ 40 KB, verbatim.

| § / Bug | Onde |
|---|---|
| §1–§5 (custo, decisão, ordem, waves, estado) | **neste doc** (vivo) |
| §6, §7, §8, §9 (abertura), §9.1–§9.3 | [`parte_1`](archive/121_as_formas_na_placa/parte_1.md) |
| §9.4–§9.9 | [`parte_2`](archive/121_as_formas_na_placa/parte_2.md) |
| §9.10–§9.13 | [`parte_3`](archive/121_as_formas_na_placa/parte_3.md) |
| §9.14–§9.16 | [`parte_4`](archive/121_as_formas_na_placa/parte_4.md) |
| §9.17–§9.18 | [`parte_5`](archive/121_as_formas_na_placa/parte_5.md) |
| §9.19 | [`parte_6`](archive/121_as_formas_na_placa/parte_6.md) |
| §9.20–§9.23 | [`parte_7`](archive/121_as_formas_na_placa/parte_7.md) |
| §9.24 | **neste doc** (vivo) |

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

### §9.24 — O ABERTO DO §9.23, num bloco (2026-10-07, depois do smoke aprovado: *«vamos resolver o que está em aberto»*)

Os três itens (a rampa perto do limiar, o oráculo externo, o app a `4 096` não re-medido) numa rodada: as variantes
da trava atrás de uma chave POR FIO (retirada antes do fecho), o Box2D a correr as MESMAS entradas, e o app em
`release` com a carga medida ao lado. ⚠️ O mecanismo da rampa foi LIDO antes de construir (os números do §9.23 e o
código) e só depois virou variantes — escrito aqui depois da rodada, na mesma janela.

**(1) A rampa perto do limiar — o mecanismo, provado pela conta:** a `0,15` a bola desce `0,241` contra `0,294` da
teoria porque a trava espera `12` passos (`0,2` s) antes de soltar: `0,294 × (1,8/2)² = 0,238`. A `0,2`/`0,21` ela
desce DEMAIS (`0,103`/`0,080` contra `0,059`/`0,012`): depois de soltar, o travão a rolar é `clamp(L, ±cap)`, e
enquanto o momento é pequeno ele trava MENOS que a capacidade com o binário dos contactos ainda a empurrar.

| variante (`11` realizações nas pilhas) | rampa `0,1`·`0,15`·`0,2`·`0,21` (teoria `0,530`·`0,294`·`0,059`·`0,012`) | discos `0,1` queda · assente · trocas | discos `0,25` assente | caixas `0,1` trocas |
|---|---|---|---|---|
| **hoje** (solta ao 12.º, travão `clamp(L)`) | **`0,556`·`0,241`·`0,103`·`0,080`** | **`5,6`·`41` · `11,6°` · `3,8`** | **`1,0°`** | **`6,6`** |
| travão com a capacidade INTEIRA enquanto acelera | `0,020`·`0,336`·`0,020`·`0,020` — prende | `22`·`53` · `3,3°` · `3,9` | `0,0°` | `6,6` |
| solta ao 2.º excesso do MESMO sentido + travão inteiro | `0,024`·`0,336`·`0,024`·`0,024` — prende | `4,3`·`43` · `100°` · `17` | `0,0°` | `34` |
| idem ao 4.º · ao 8.º | prende (`0,022` · `0,020` a `0,1`) | `1,5`·`23` · `18,5°` · `9,2` · `2,4`·`30` · `10,9°` · `5,5` | `0,0°` | `16` · `9,2` |
| solta ao 4.º do mesmo sentido, travão de hoje | `0,556`·`0,277`·`0,114`·`0,086` | `12`·`66` · `7,5°` · `8,7` | **`62°`** (`0,31` u) | `15` |

⇒ ⛔ **recusadas as cinco.** O travão inteiro aplicado de FORA do solver (antes do passo) inverte o giro dentro do
passo e a trava volta a fechar — a bola fica presa onde devia rolar; soltar mais cedo pelo sentido do pedido devolve
o pisca-pisca. **A lei de hoje fica.**

**(2) O oráculo — o Box2D `3.1.1` a correr as MESMAS entradas** ([`oraculo_box2d_rolamento/`](ferramentas/oraculo_box2d_rolamento/README.md):
licença MIT conferida no pacote; o pacote do `extra` baixado e com a ASSINATURA conferida, sem instalar nada; as
posições iniciais das onze realizações exportadas da nossa cena; fixture `box2d_2026-10-07.txt`). O Box2D resolve o
rolamento DENTRO do solver, mas POR PAR (a lei que esta casa mediu e recusou para si):

| | rampa `0,1`·`0,15`·`0,2`·`0,21` | discos `0` queda · assente | `0,05` | `0,1` | `0,15` | `0,25` |
|---|---|---|---|---|---|---|
| teoria | `0,530`·`0,294`·`0,059`·`0,012` | — | — | — | — | — |
| **Box2D** (rr alinhado ao limiar: `×1,123`) | `0,485`·`0,243`·`0,023`·`0,020` | `77`·`134` · `9,7°` | `198`·`253` · `76°` | `115`·`155` · `104°` | `111`·`113` · `0°` | `103`·`128` · `76°` |
| **nós** | `0,556`·`0,241`·`0,103`·`0,080` | `131`·`225` · `21°` | `116`·`199` · `210°` | `5,6`·`41` · `11,6°` | `84`·`105` · `85°` | `0,3`·`0,6` · `1,0°` |

⇒ **o limiar do Box2D não depende do raio** (`0,2388` a raio `0,2`, `0,2394` a `0,11`): a unidade do `rollingResistance`
é a nossa (`rr · N · raio`). **Perto do limiar nem o solver de referência segue a teoria** (a `0,2` `0,023` contra
`0,059` — erra para o OUTRO lado: prende cedo). ⭐ **E nas pilhas o Box2D mostra o «defeito» do prompt de 06/10**:
com o rolamento os discos giram MAIS na queda do que sem ele (`0,05`: `198°` contra `77°`; `0,25`: `103°`), e não é
monótono. ⇒ *«um `Rolling` baixo faz a pilha girar mais»* é propriedade das pilhas com rolamento (encostas que só o
atrito de rolar segura e que desabam), não da nossa trava; a nossa lei é a MAIS CALMA das duas em `0,1`/`0,25`.

**(3) O app a `4 096` por taça** (`[motion-quadro]`, `release`, `40` s = a queda inteira de `15` s duas vezes,
`target/prova/onda9/app/`): com a carga a `1,9`–`3,8`, **`61`–`70` fps em todas as janelas** fora do arranque (o
1.º meio segundo), nas duas placas; o Motion `≤ 4,6` ms de mediana. A sonda sem vídeo (`custo_por_fase_da_queda`):
`1,9`·`3,0`·`3,0`·`2,8`·`2,7` ms por fase de `3` s — a de 06/10 era `2,3`·`2,9`·`2,9`·`2,7`·`2,8`. ⚠️ A 1.ª
leitura (carga `14 → 10`, logo depois da compilação) dava janelas de `51`–`57` fps e o Motion a `6`–`9` ms — a mesma
lição do §9.21: a medição do app não vale acima de `load ~5`. Em discos com `Rolling 0,1`: `60`+ fps fora do arranque.

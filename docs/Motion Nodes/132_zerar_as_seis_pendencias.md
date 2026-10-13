# 132 — ZERAR as seis pendências do 131 (P1–P6): plano, 13/10

> Ordem do dono (13/10): TODAS as pendências do
> [handoff ZERAR_131](handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-10-12_ZERAR_131.md) num ÚNICO ciclo
> (`CLAUDE.md` §0.10). Linha `line/motion-value`, alinhada ao `main` `757afebd58` (nada por integrar).
> Protocolo: [103](103_dinamica_dos_ciclos.md). Régua: [MEDIR_VELOCIDADE](../DevOps/MEDIR_VELOCIDADE.md).
>
> Recusas consultadas antes de desenhar: [131 §7](131_zerar_o_ultimo_bloco.md) (C.7: `scan` por subgrupos,
> ligações em cache; C.10: `texture_2d_array`, `LADO_MAX` 4096, sincronização por página), [130 §3.1](130_o_passe_na_placa.md)
> («todos os pares» sem grelha), [118 §6](118_a_mistura_em_grupo.md) (recusas por nome; o custo está no
> [B6](handoffs/HANDOFF_CONTINUACAO_line_motion_value_2026-10-12_ZERAR_FAMILIA_B_B6.md): CPU `0,54/0,72/1,60` ×
> placa `0,55/0,57/0,70 ms` a 1k/4k/16k), [129 §3.4 e §7](129_ciclo_20_o_acabamento.md) (o átlas de células).

## §0 — A ordem (o que é série e porquê)

| lote | itens | porque é este o lugar |
|---|---|---|
| **1 — paralelo** (agentes de worktree) | **A** = P5 · **B** = P2 · **C** = P4 · **D** = P3 + P6 + a sonda do P1 | nenhum toca os ficheiros do outro; cada um traz a sua cena |
| **2 — série dentro de C** | P4(c) depois de P4(a) | ⚠️ se a placa compõe a imagem num grupo, a vigia deixa de devolver à CPU e a janela de 1–2 quadros DESAPARECE; só se P4(a) ficar na CPU por lei é que (c) pede cura própria |
| **2 — série dentro de D** | a cura do P1 depois da decomposição | ⚠️ a ordem do dono: «ataque a causa que a medição mostrar» — a cura não se escolhe antes do número |
| **3 — UMA rodada** | sonda da placa: P1 (variantes) · P3 (`corrige`) · P4 (custo) · P6 (texels) num processo, `--profile smoke`/release, `PH2D_GPU=1`, blocos intercalados com a ordem rodada, o MÍNIMO, `loadavg` ao lado | a régua |
| **4 — fecho** | mutações (`mutacao`), cenas `=138…=142` fotografadas, gate batched (`verificador`), handoff ≤ 8 KB | §1.5.9 |

Cenas do smoke, contadas no roteador (`motion_state_demo_router.rs:34`, `MAX_DEMO_LEVEL = 137`): **`=138`** P1 ·
**`=139`** P2 · **`=140`** P4 · **`=141`** P5 · **`=142`** P6. Todas com FORMAS + SIMULAÇÃO com campos.

## §1 — P1: o degrau de FORA no filtro «cada forma» (C.10)

- **Estado (código):** `plano.rs:41` `Vala { alcance, leitura }`; a vala justa é `A + L + 1` (`plano.rs:96-99`);
  `celulas.rs:333-345` monta-a de `stack_reach`/`stack_read` (`fx_stack_plan.rs:146, ~196`); `Celulas::escolhe`
  (`celulas.rs:287`) troca a variante em execução. Para um Glow de fora `A ≈ alcance(σ)`, `L = alcance + 1`.
- **A conta que já diz muito:** num degrau de fora a SAÍDA de cada forma (`s + 2A`) já ocupa quase toda a célula
  (`s + A + L + 1`): disco de 14 px com Glow 16/17 ⇒ saída `46²` contra célula `48²`. A vala já está no piso. O que
  sobra para atacar é o custo POR píxel da área de saída, não a área.
- **A medição que decide (série):** `relogio_c10_decomposicao` ganha as pilhas de FORA (Outer Glow, Drop Shadow
  para fora e um controlo de dentro), a 4 096 formas, por passe (`pass_profiler`). Hipóteses e o número que as
  separa:
  1. **A pilha é linear na área** (`0,244 ms/Mpx`) ⇒ a alavanca é tirar área à resolução cheia: os degraus de borrão
     a correr na pirâmide (já existe acima de `MAX_HALF/3`) e o desenho/colagem só onde há tinta.
  2. **A colagem domina** (as células coladas inteiras, `n × 48²`) ⇒ colar só a caixa da saída.
  3. **O custo fixo por página/forma domina** ⇒ o problema é o número, não a área.
- **Alternativa considerada e o que a recusa por conta:** conjuntos independentes NO LUGAR (sem relocar; as formas
  a mais de `A + L + 1` umas das outras partilham uma página do tamanho da janela). Com azulejos que saltam vazios a
  área é a mesma das células (`Σ` das saídas); sem eles é `cores × janela`, e a 4 096 discos em 1,55 Mpx cada disco
  colide com ~19 vizinhos ⇒ pior. Só se a medição disser que a página é o custo.
- **Gate:** `the_c10_variants_give_the_same_pixels` estendido às pilhas de fora (a mesma faixa da referência);
  a cura escolhida entra como variante em execução e o gate corre as variantes todas.
- **Mutação:** a cura com a leitura a menos (a vala sem `L`, ou a caixa da colagem sem `A`) ⇒ o gate de pixels
  reprova.
- **Smoke `=138`:** 4 096 formas a cair num campo com Outer Glow «cada forma»; os números do terminal antes/depois.

## §2 — P2: a fita das entradas não mente em silêncio

- **Estado (código):** `fita.rs:49` `FITA_BYTES = 128 MiB`; ao encher, `dobra` (`fita.rs:525`) dobra a marca
  mais velha na `base` e escreve `base_ate`; `cobre(t)` (`fita.rs:341`) continua verdadeiro para `t < base_ate` e
  `estado_em` devolve a base ⇒ **o replay repete a base em silêncio** (`entradas.rs:100-113`). Uma `Fita` por pump
  (`lib.rs:231`). Nada no repo lê a RAM da máquina. Timeline: há `Marker` e o padrão de publicação por id
  (`ph2d-panel-timeline/src/event.rs:62-75`), nenhuma faixa de cache. Cartão: `node.readout = Some(queixa)`
  (`motion_bridge_readout.rs:331`).
- **(a) A lei:** a fita responde `Verdade` × `Dobrada { desde }` (uma porta, `Fita::historia_em(t)`); o replay de
  um `t < base_ate` devolve a base E a marca de que dobrou. O artista vê-o em DOIS sítios, pela mesma porta:
  1. **A Timeline** pinta por baixo da régua a faixa da história gravada — o idioma da barra de cache de física
     do Blender (o produto de referência): cheia de `base_ate` a `ultimo`, ausente antes. A família publica o
     intervalo como dado (um resource/snapshot que o shell entrega ao painel); a Timeline não conhece o Motion.
  2. **O cartão** do nó das entradas: com o scrub antes de `base_ate`, a `readout` diz-o («history before frame N
     was folded — showing frame N»), i18n. Nunca só no terminal.
- **(b) O orçamento pelo RECURSO:** o tecto passa a ser uma fração da RAM FÍSICA (lida uma vez: `/proc/meminfo`
  `MemTotal` no Linux, `sysctl hw.memsize` no macOS, `GlobalMemoryStatusEx` no Windows; sem leitura ⇒ o número de
  hoje), partilhado por TODAS as fitas do processo (um razão atómico — cem objectos não podem pedir cem tectos). A
  fração mede-se: RSS do app com a cena mais pesada × RAM da estação de trabalho (128 GB) e da máquina pequena
  (8 GiB), com a tabela ao lado. O `CPU_RING_BYTES` (`checkpoint.rs:52`) é a MESMA família (constante de RAM fixa) ⇒
  mesma porta, mesma medição.
- **Medição:** `fila_c_a_fita_por_entrada` (B/tique e minutos de história a 3/30/300 entradas) com o tecto novo;
  `a_fita_declara_a_memoria_que_retem` (dhat) intacta.
- **Gates:** a história antes da base diz `Dobrada` (e o controlo: depois dela diz `Verdade`); o cartão mostra a
  queixa ao arrastar para antes da base (comportamental, o evento real); a faixa publicada = `[base_ate, ultimo]`;
  o razão global nunca passa o tecto com N fitas.
- **Mutações:** `historia_em` sempre `Verdade` · a faixa a começar em `primeiro` · o razão por fita em vez de
  global.
- **Smoke `=139`:** uma simulação com muitas entradas e um tecto PEQUENO por variável de ambiente, para a dobra
  chegar em segundos; o artista vê a faixa começar mais tarde e o cartão avisar ao arrastar para trás.

## §3 — P3: o `corrige` do kernel de um grupo (C.7)

- **Estado (código):** `passe_na_placa_varredura.wgsl:132` `pnp_corrige`: quatro arrays privados de 9
  (`cur fim cel cab`) indexados DINAMICAMENTE no laço de fusão (`cab[k] < j`, `cur[kk]`) — o padrão que os
  compiladores de dispositivo põem na memória LOCAL. Chamado de `passe_na_placa_uno.wgsl:69` e da rota de
  despachos.
- **Lei a provar:** a mesma fusão por índice CRESCENTE (a ordem de soma da CPU), com os nove cabeças em
  registos (desenrolado, gerado pelo Rust na montagem do módulo) ⇒ a MESMA soma; o resultado tem de bater a CPU na
  tolerância de hoje, e a variante atual ao bit se o compilador contrair igual (o 131 §7 linha 2 viu ULPs de outro
  módulo).
- **Medição (rodada única):** `measure_a_cadeia_cpu_contra_placa`, as duas variantes do `corrige` × as duas rotas
  (`um grupo` / `despachos`) no MESMO processo — nuvens 512/1024/2048 · 64 e as quatro cadeias. Se a variante ganha
  a 2 048 em despachos E em um grupo, o `PECAS_PARA_UM_GRUPO` mede-se de novo (só muda com número).
- **Gate:** a paridade CPU × placa existente corre as duas variantes; a escolhida vira default.
- **Mutação:** a ordem da fusão trocada (decrescente) ⇒ a paridade da soma reprova.

## §4 — P4: o que hoje fica na CPU «por lei»

- **Estado (código):** `IMAGEM_EM_GRUPO` (`motion_bridge_gpu_mistura.rs:25, 39-67`): «o passe de sprites não sabe o
  alcance nem o tom»; `PEDE_O_MUNDO` (`motion_shape_placa_mistura.rs:46-80`); a vigia (`ph2d-gpu-cook/src/vigia.rs`)
  lê o contador de imagens com `Poll` no quadro N+1 (ou N+2) ⇒ o quadro N já desenhou pelas sprites.
- **(a) Imagem num grupo que mistura:** construir a imagem no passe da mistura em grupo da placa (o mesmo alcance e
  tom que as formas), com a paridade de pixel contra a CPU (`a_mistura_em_grupo_pelo_dispositivo_desenha_o_que_a_cpu_desenha`
  ganha a imagem). Se bate, a recusa sai.
- **(c) A janela:** com (a) feito, a imagem num grupo deixa de ser recusa ⇒ a vigia não devolve ninguém e a janela
  desaparece. Se (a) ficar na CPU: a decisão passa para ANTES do desenho (o quadro espera a leitura do contador, ou
  o desenho da imagem num grupo nunca vai às sprites), medida em ms por quadro.
- **(b) O mundo por baixo:** primeiro MAPEAR a ordem do quadro (onde o Vello rasteriza o mundo; quando a placa do
  Motion desenha; se há textura do que está por baixo). O compositor de camadas já mistura contra o acumulado
  (`layer_composite.wgsl:380` `apply_blend`). Se o mundo existe como textura quando o Motion desenha: ler o
  backdrop e misturar com a lei da CPU, paridade de pixel. Se não existe: medir o preço de o produzir (cópia do
  alvo) e decidir pelo número; a lei fica escrita com ele.
- **Gates:** as paridades de pixel acima, sobre `Everything`/`Scene` fora da camada isolada e a linha sem grupo;
  e a janela: nenhum quadro desenha a imagem num grupo pelas sprites (contado quadro a quadro com `Poll`).
- **Mutações:** o tom da imagem no grupo ignorado · o backdrop lido do quadro anterior.
- **Smoke `=140`:** uma imagem e formas num grupo que mistura, sobre um fundo, a cair num campo; o terminal diz a rota.

## §5 — P5: o aviso cortado (a peça partilhada, `ph2d-editor-core`)

- **Estado (código):** `ToastQueue::paint` (`toast.rs:202-294`) pinta UMA linha numa coluna FIXA de 360 px
  (`progress.rs:72`, linhas uniformes de 44 px) e elide; e o orçamento encolhe DUAS vezes — `text_budget_px()`
  (`toast.rs:192`) e depois `label_budget` dentro do `paint_text_centered` (`paint_label_box.rs:32-45`, `−2·Md`),
  que o `debug_assert` não vê. 407 chamadas, 10 crates (a shell 260, Components 44, Motion 30, editor-core 28,
  Painter 18, Flip 8, Board 8, Vector 6, Física 3, Timeline 1); os longos vêm todos do i18n.
- **A lei:** um aviso é LIDO inteiro. (1) O texto quebra em linhas (`paint_text.rs` `Lines::Wrap`), até um máximo
  medido; um aviso de k linhas ocupa k linhas da coluna (a régua uniforme continua: o balão abarca k linhas, e as
  barras de trabalho descem pela contagem de linhas, não de balões). (2) Um orçamento só (a porta), sem o segundo
  desconto. (3) O tempo de vida cresce com o tamanho: uma velocidade de leitura publicada (Brysbaert 2019, ~238
  palavras/min a ler em silêncio) com o mínimo de hoje (3 s).
- **Medição (censo por CONSUMIDOR):** todas as mensagens de aviso da tabela de strings em inglês (as interpolações
  com argumentos representativos), medidas com a fonte real ao orçamento: quantas cortavam antes, quantas linhas
  pede cada uma depois, e o tempo de vida que recebe. A tabela vai para o §7.
- **Gates:** toda mensagem de aviso da tabela cabe inteira (zero elisões) no máximo de linhas; o balão de k linhas
  não sobrepõe a barra de baixo; o tempo de vida de uma frase longa ≥ o tempo de leitura dela.
- **Mutações:** voltar à linha única elidida · o segundo desconto de volta · as barras a descer por balões.
- **Smoke `=141`:** uma cena do Motion onde ligar um fio dá o aviso longo (`can_t_drive_that_output_is_a_per_element_stream`
  + a cura); fotos ANTES e DEPOIS.

## §6 — P6: a silhueta do `SpriteRenderer` (W3b)

- **Estado:** 13 texels entre a rota da placa e a da CPU na camada inteira, 17 entre células e camada
  (`celulas_gpu.rs:600-602`, doc 129 §7); a vala justa levou a rota da placa a 0 nas células; nenhuma cena de
  sprites com filtro.
- **Medição:** a cena de sprites com filtro (camada inteira e cada forma, as duas rotas): os texels de diferença na
  silhueta, por rota e por escopo, e ONDE (borda do quad: a amostragem sem anti-aliasing do quad; ou a
  interpolação da textura).
- **A lei:** se a diferença é um defeito da rota da placa (amostragem, meio texel), cura-se com paridade ao texel;
  se é a forma do quad sem anti-aliasing nas DUAS rotas (a referência dispersa 63 níveis, 131 §7), a lei fica
  escrita com o número.
- **Gate:** `a_lone_sprite_is_the_same_under_both_scopes_on_both_routes` com a barra que a medição der.
- **Mutação:** o meio texel desligado na rota curada.
- **Smoke `=142`:** sprites com filtro a cair num campo, cada forma × camada inteira.

## §7 — Medições (preenchidas ao correr)

Lote D (P1 · P3 · P6), RTX 5060 Ti, `--release`, uma rodada back-to-back (`loadavg` 10,5 → 15,0; outra linha
compilava a meio da P3). As variantes no MESMO processo, por interruptor em execução, blocos com a ordem rodada,
o MÍNIMO.

### P1 — o degrau de FORA: a causa é o custo POR PIXEL do borrão

**Decomposição** (`relogio_c10_decomposicao`, 4 096 discos de 12 px a 1600×972, ms de placa por quadro; o
relógio por PASSE da pilha é novo — `fx.*`):

| pilha | A · L | célula · saída | pág. | Mpx | placa | pilha (H · V · resolve · ingest) | formas | volta | colagem |
|---|---|---|---:|---:|---:|---|---:|---:|---:|
| Inner Shadow (dentro) | 0 · 23 | 38 · 14 | 2 | 6,0 | 2,15 | 1,39 (0,64 · 0,55 · 0,07 · 0,07) | 0,30 | 0,06 | 0,08 |
| Outer Glow σ 5,3 | 16 · 17 | 48 · 46 | 3 | 9,5 | 3,60 | 2,29 (1,03 · 0,87 · 0,16 · 0,11) | 0,54 | 0,19 | 0,14 |
| Drop Shadow 6/6 (fora) | 16–22 · 23 | 60 · 52 | 4 | 14,8 | 4,88 | 3,45 (1,60 · 1,36 · 0,17 · 0,17) | 0,49 | 0,24 | 0,16 |
| camada inteira (qualquer) | — | — | 1 | 1,56 | 0,37 | (0,17 · 0,14 · 0,02 · 0,02) | | | |

A pilha é linear na área (`0,24 ms/Mpx`, as células e a camada inteira igual), a área já é ≈ Σ das saídas (Glow:
`46²` de `48²`), a colagem é 4 % ⇒ hipótese 1: o custo por pixel, e ele é o BORRÃO (H + V = 83 % da pilha).

**A cura:** o borrão por GRUPO (`ph2d-render/src/fx_stack_borrao_grupo.rs`) — 64 pixels de uma linha (coluna)
carregam UMA vez os `64 + 2h` taps e os `2h + 1` pesos na memória do grupo; o de sempre fazia, por pixel,
`2h + 1` leituras E `2h + 1` exponenciais. Os mesmos taps, pesos e ordem: o MESMO byte em 8 pilhas
(`o_borrao_por_grupo_da_o_mesmo_byte`, `2cfacfe11f`), e `the_c10_variants_give_the_same_pixels` (com as pilhas de
fora) igual ao byte.

**Régua intercalada** (`relogio_c10_intercalado`, placa por quadro, o MÍNIMO de 7 rodadas; `load 9,1 → 7,9`):

| discos | Inner Shadow: produto → grupo | Outer Glow | Drop Shadow (fora) |
|---:|---:|---:|---:|
| 256 | 0,38 → 0,36 | 0,54 → 0,50 | 0,61 → 0,55 |
| 1 024 | 0,69 → 0,60 | 1,02 → 0,88 | 1,31 → 1,07 |
| 2 048 | 1,08 → 0,90 | 1,95 → 1,71 | 2,47 → 2,01 |
| 4 096 | 2,18 → 1,83 | 3,57 → 2,96 | 4,92 → 3,93 |
| 8 192 | 3,98 → 3,28 | 6,81 → 5,65 | 9,46 → 7,70 |

A camada inteira (1600×972): `0,37 → 0,28 ms`. **O borrão por grupo é o único** (`4609c19ec6`).

| ⛔ Recusas MEDIDAS | porquê |
|---|---|
| o borrão por PIXEL (`blur_h_at` + `cs_op_v` de sempre) | perde nas 15 linhas da régua (o grupo tira até `19 %` a 8 192) e dá o mesmo byte |
| colar/desenhar só a caixa da saída | a saída já é `92 %` da célula (Glow `46²`/`48²`); a colagem é `4 %` da placa |
| o borrão na pirâmide abaixo de `MAX_HALF/3` | não construído: o borrão por grupo ganha com paridade AO BYTE; a pirâmide paga a caixa+tenda contra o oráculo Φ |

Aberto (com número): a vala de uma Drop Shadow é SIMÉTRICA (`antes = max(A)` dos quatro lados, `L` sem direção):
célula `60` px para uma saída de `52`. Com a leitura por direção (`L` à esquerda `23`, à direita `11`) o passo
seria `54` px — `−19 %` de área na Drop Shadow; zero na Glow.

### P3 — o `corrige` em registos (`select`) ganha; `PECAS_PARA_UM_GRUPO` 1024 → 1536

`measure_a_cadeia_cpu_contra_placa`, as três formas × os dois caminhos, o MÍNIMO de 15 blocos (ms, total da ida e
volta; `load 7,9 → 20,4`):

| linha | CPU | um grupo: Matriz · Ramos · **Escolha** | despachos: Matriz · Ramos · **Escolha** |
|---|---:|---|---|
| cadeia 16 · 1024 | 1,87 | 11,13 · 11,78 · **11,13** | 104,2 · 104,8 · 107,0 |
| cadeia 16 · 4096 | 3,35 | 19,67 · 21,95 · **21,48** | 514,3 · 509,4 · 515,7 |
| cadeia 32 · 4096 | 15,21 | 49,92 · 50,05 · **47,71** | 547,6 · 548,3 · 544,1 |
| cadeia 256 · 1024 | 13,36 | 23,94 · 24,19 · **23,22** | 153,4 · 156,5 · 154,4 |
| nuvem 512 · 64 | 2,11 | 11,98 · 11,68 · **9,72** | 19,89 · 17,59 · 20,02 |
| nuvem 1024 · 64 | 4,50 | 18,16 · 16,22 · **12,29** | 17,98 · 17,44 · 16,47 |
| nuvem 2048 · 64 | 6,76 | 33,93 · 27,90 · **19,26** | 19,85 · 19,49 · **18,35** |

A cadeia 16 · 4096 é a única pior no mínimo (`+9 %`), e a mediana diz o contrário (`24,49 → 23,65`, 3 rodadas):
a régua não decide essa linha. **Paridade:** as três dão a MESMA soma AO BIT (`P`, `rot` e varreduras; 11
fixturas × 1/8/64 varreduras × as duas rotas) — e a CPU na tolerância de sempre (pior `2,84e-6 · alc`).

**O ponto** (`measure_o_ponto_do_um_grupo`, 3 rodadas de 30 blocos, o mínimo; um grupo × despachos): 1 280
`7,57 × 9,10` · **1 536 `9,65 × 9,98`** · 1 792 `11,25 × 9,85` · 2 048 `13,26 × 10,24` ⇒ `1536`. ⚠️ Os despachos
pagam a CARGA da máquina (o mínimo deles vai de `10,2` a `18,4 ms` a 2 048 entre rodadas; o de um grupo, de `13,3`
a `19,3`): sob carga o ponto sobe.

| ⛔ Recusas MEDIDAS (código em `38c9767ea5`) | o número |
|---|---|
| `corrige` em quatro matrizes de 9 (a do doc 130) | nuvem 2 048 um grupo `33,93` × `19,26 ms` |
| `corrige` desenrolado com cadeia de `if`s | `27,90` × `19,26`; cadeia 16·4096 `21,95` |

### P6 — a silhueta das sprites: dois defeitos, as duas curas ao texel

`medida_p6_a_silhueta_das_sprites` (cinco sprites rodadas 30°, sub-pixel, 21 122 texels pintados; texels a > 1
nível):

| comparação | antes | depois |
|---|---:|---:|
| crua, placa × CPU | 61 (pior 29) | **0** |
| filtrada, camada inteira, placa × CPU | 50 | **0** |
| rota CPU, cada forma × camada inteira | 0 | **0** |
| rota placa, cada forma × camada inteira | 75 (pior 21) | **0** |

ONDE: só na silhueta, nenhum fora; `≤ 0,75 px` da aresta do quad (`4` a `< 0,01`, `51` a `< 0,1`, `20` a
`< 0,75`). A CAUSA não era o quad nem o meio texel: (1) **o sampler** — o tag `0` (`SAMPLING_DEFAULT`) ligava na
CPU um grupo próprio (`sampler_from_tags(0, 0)`: linear, um nível, isotrópico) e a partição vazia da placa o do
projecto (trilinear, anisotropia 16); a faixa explícita do tag `0` na placa dava `0` texels; (2) **a paridade da
translação** — com o sampler do projecto nas DUAS rotas, as células de translação ÍMPAR trocam o quadrado 2×2 das
derivadas da aresta: `75` texels nas duas rotas, `0` com a translação par.

Curas: o tag `0` do átlas cai no grupo do projecto (`5224436632`, a lei do gémeo das texturas individuais) e as
sprites planeiam-se com a translação PAR (`planeia_com_par`, `c8e708f5d2`, um pixel de folga por eixo).
`a_lone_sprite_is_the_same_under_both_scopes_on_both_routes` passa a cinco sprites, exige as rotas iguais e a
barra da silhueta é a medida: `0` (era `≤ 0,5 %`). A paridade C.10 das 24 sprites também foi a `0` texels.

**Mutações:** a fusão pela maior cabeça (P3) **sobrevive** — muda `|Δrot|` em `≤ 0,6e-6°` e `|ΔP|` em nada,
dentro da tolerância: a ORDEM não se vê na régua de paridade (o limite é da régua). Bordo do borrão: o tap
deslocado um pixel → 3 gates vermelhos; o último tap fora → 1 (`the_layer_blur_is_the_oracles`). O tag `0` com
grupo próprio → o gate das rotas vermelho. A translação sem paridade → a barra `0` vermelha. A vala sem `L` → 2.

**Cenas:** `=138` (4 096 discos a cair num campo, Outer Glow em cada forma; o custo com `PH2D_FLUID_PROFILE=1`).
⚠️ O P6 **não é a `=142`**: a lei da aparência só desenha uma corrente de forma ou de OBJECTO, e o objecto mora na
hierarquia (medido: a chuva com o ladrilho de uma grelha desenha `0` sprites) ⇒ `PH2D_MOTION_OBJ_SMOKE=18`.
`MAX_DEMO_LEVEL` `137 → 138`.

### P2 — a fita contra a RAM (13/10, workstation, `dev`, `loadavg` 2,8–5)

Sonda `fila_c_a_fita_por_entrada` (`motion_bridge_gpu_fila_c_probe.rs`): `grid → output` + `K` cartões `Input`, as
`K` entradas a mudar em todo tique, 3 000 tiques pelo caminho da CPU do produto, **um processo por `K`**
(`PH2D_SONDA_ENTRADAS`). O RSS do último terço é o que conta: até ao tique 2 048 o processo também enche o anel
(ver «aberto» abaixo; com `K = 0` o RSS sobe `1 348 B/tique` no total e `184` no último terço).

| entradas a mudar | declara B/tique | RSS B/tique, último terço | RSS ÷ declarado | RSS do processo |
|---:|---:|---:|---:|---:|
| 3 | 214,5 | 225,3 | 1,05× | 37 MB |
| 30 | 1 545,8 | 1 552,4 | 1,00× | 41 MB |
| 300 | 14 859,9 | 14 843,9 | 1,00× | 80 MB |

**O tecto e a história que cabe a 60 Hz** (`tecto ÷ B/tique`; a RAM lida pelo `/proc/meminfo`):

| RAM | fração | tecto da fita | 3 entradas | 30 entradas | 300 entradas |
|---|---|---:|---:|---:|---:|
| 8 GiB (máquina pequena) | 1/64 (= o número fixo de antes) | 128 MiB | 174 min | 24 min | 2,5 min |
| 8 GiB | **1/32** | **256 MiB** | **348 min** | **48 min** | **5,0 min** |
| 8 GiB | 1/16 | 512 MiB | 695 min | 96 min | 10,0 min |
| 128 GB (`MemTotal` 123,2 GiB, esta) | **1/32** | **3,85 GiB** | **89 h** | **12,4 h** | **77 min** |

**Escolha: `1/32` da RAM física por cache do scrub** (`FRACAO_DA_RAM`, `razao.rs`), a fita e o anel da CPU cada um
com o seu razão — os dois juntos `1/16`. O recurso é a RAM física da máquina, partilhada por TODAS as fitas (e por
todos os anéis) do processo: cem objectos não pedem cem tectos. `1/16` por cache daria `1 GiB` aos dois na máquina
de 8 GiB, cuja memória é unificada com a placa; `1/32` dobra a história de antes nela (2,5 → 5 min a 300 entradas)
e dá 30× na estação. Sem leitura da RAM (Windows: o `GlobalMemoryStatusEx` pede o `windows-sys`, que a crate não
tem) vale a reserva de antes, 128 MiB. `PH2D_MOTION_FITA_BYTES` (lida uma vez) põe um tecto à mão — a cena `=139`.

**O anel (fechado 13/10):** o `CookCheckpoint::approx_bytes` contava só os DADOS das colunas e os memos — `8 B`
por ponto leve, `0 B` sem estado — e o processo ganhava `~1,9 KB` por checkpoint (`K = 0`). A régua
`ph2d-eval-motion/tests/anel_memoria.rs` (dhat, `bytes + 16 · blocos`, `±25 %`, o irmão do `fita_memoria.rs`) leu-o
VERMELHO e mede-o hoje verde:

| cenário (o anel no razão próprio) | declarava B/ponto | retém B/ponto | declara hoje |
|---|---:|---:|---:|
| 1 nó, 1 número (1 000 tiques) | 8 | 970 | 970 (1,00×) |
| 30 nós, 1 número | 240 | 17 911 | 1,00× |
| sem estado (só o lugar no vetor) | 0 | 90 | 1,00× |
| 4 096 × 3 colunas | 97 812 | 98 988 | 1,00× |
| memória de 1 024 números (1 ponto a cada 8 tiques) | 4 063 | 5 491 | 1,00× |

A cura: o ponto de recuo declara o heap que o clone copia (os mapas pelas folhas de 11 lugares do `std`, os vetores
de saídas, os nomes, o `Arc` e os dados de cada coluna, o `Arc` de cada memo) e o anel soma o seu vetor de entradas
pela CAPACIDADE (`CheckpointRing::retido`). O fixture do regime de despejo (`scrub_tests`, `set_ring_budget(200)`,
calibrado contra os `8 B`) passa a `28 ×` o peso medido de um ponto.

**O cartão antes do primeiro tique gravado (fechado 13/10):** numa fita dobrada, um scrub para antes do PRIMEIRO
tique gravado dizia «showing <base>» e o replay usava o publicado agora (`entradas_do_replay`, ramo `!cobre`). A
porta `Fita::historia_em` responde agora pela MESMA pergunta do replay: fora da fita, `Historia::NaoGravada` (chave
`app.motion.motion_historia.history_not_recorded`). O gate do cartão (`motion_bridge_historia_tests`) corre a rota
da CPU SEMPRE (`quadro::coze_o_quadro_sem_placa`, só dos testes) e a da placa onde há adapter — e apanhou na placa
um defeito vizinho: um grafo sem estado não guarda o último tique cozido, então arrastar para antes do primeiro
gravado GRAVAVA, era um ramo e esquecia a fita inteira. A rota usa agora o último tique cujas entradas se mostraram
(`MotionCookPump::tique_das_entradas`), a regra da CPU.

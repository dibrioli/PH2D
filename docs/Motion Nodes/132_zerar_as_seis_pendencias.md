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

**Aberto (o anel, não a fita):** o `CookCheckpoint::approx_bytes` conta colunas e memos e declara `0 B` para um
checkpoint leve; o processo ganha `~1,9 KB` por checkpoint (`K = 0`: 3,9 MB até o anel encher, 184 B/tique
depois). Limitado pelo `MAX_ENTRIES = 2048` (`~4 MB`), logo inofensivo; a régua do anel contra o heap (o irmão do
`tests/fita_memoria.rs`) não existe.

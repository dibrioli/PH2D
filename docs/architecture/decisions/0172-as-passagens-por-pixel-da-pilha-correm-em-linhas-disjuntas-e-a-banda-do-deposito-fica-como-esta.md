# ADR-0172 — As passagens POR PIXEL da pilha correm em linhas disjuntas; a banda do depósito fica como está

- **Status:** Accepted
- **Data:** 2026-09-23
- **Linha:** `line/PainterWatercolor`
- **Amparo:** ADR-0109 (a cerca de contenção do `rayon` em `ph2d-tool-painter`, e os TRÊS invariantes
  que qualificam uma excepção) · ADR-0171 (o precedente mais próximo: o borrão de caixa da mesma pilha)

> ⚠️ **O número deste ADR SOMA entre linhas** (§5.0) — quem integrar reconta-o contra o `main` do
> dia, nunca o copia daqui.

## Contexto

Ordem do dono (2026-09-23), depois de aprovar a composição por quadro (*«FPS aceitável em torno de
40»*): *«vamos tentar tornar mais rápido, vamos chegar ao estado da arte»*.

⛔⛔ **A primeira régua media OUTRO programa.** As sondas `diag_*` da crate correm sob `cfg(test)`, e
ali o traço paga duas coisas que o app nunca paga: o **journal do undo** do canvas
(`capture_canvas` é `cfg(any(test, debug_assertions))`) e a **espia do esfregão**, que copia o campo
de deslocamento inteiro (`8 MB` a `1024²`) a cada lote. ⇒ a medição do produto passa a ser o
exemplo `crates/ph2d-tool-painter/examples/mede_a_pilha.rs`, que compila a biblioteca **sem**
`cfg(test)` e monta a pilha do dono pelas portas públicas que a cena `PH2D_COMPOSITE_SMOKE` usa.

⛔⛔ **E o perfilador também mentiu, numa direcção só.** Sem `perf` (o `perf_event_paranoid = 2`
recusa-o sem root) a amostragem foi feita por `gdb`, e o `gdb` torna a **criação de threads**
artificialmente lenta (cada `pthread_create` é um evento de `ptrace`): a primeira leitura pôs
**`49,6 %`** da thread principal em `pthread_create`. A troca que essa leitura pedia foi construída,
medida em A/B no build do produto, e **não mede ganho nenhum** (ver *Recusa medida* abaixo).

## Decisão

Três passagens da composição da pilha do Composite Brush passam a correr em **linhas disjuntas** na
equipa de threads (`rayon::par_chunks_mut`, `with_min_len(8)`):

1. a **tinta** de uma camada Brush (`composite_linhas::tinta`);
2. a **borracha** nos dois escopos (`composite_linhas::borracha`);
3. o **re-amostrar do esfregão** pelo campo de deslocamento (`warp::session::reamostra`, que serve
   também a sessão de warp da ferramenta Smear e do Deform — a MESMA lei, agora em linhas).

Os três invariantes do ADR-0109 valem **à letra**: cada pixel de saída é função PURA de planos que a
passagem só LÊ (o plano da camada, o `pre`, o campo de deslocamento) e do próprio pixel da tela;
nenhuma soma atravessa píxeis; nenhuma aleatoriedade entra ⇒ a saída é **byte-idêntica** para
qualquer número de threads e qualquer escalonamento. **Gates:** cada passagem é comparada, byte a
byte, contra o laço em série de antes copiado à letra (`a_tinta_em_paralelo_da_o_byte_da_serie` ·
`a_borracha_em_paralelo_da_o_byte_da_serie` · `o_reamostrar_em_paralelo_da_o_byte_da_serie`), com
CONTROLO de que a fixtura muda bytes.

Duas curas **exactas** sem paralelismo entram no mesmo passo:

- **o arredondamento sem biblioteca** (`composite_linhas::redondo_u8`): o repo compila para o
  `x86-64` BASE, onde `f32::round` é uma chamada ao `roundf` de software do `compiler_builtins`; a
  função dá o mesmo byte com um truncamento e uma comparação (gate nas fronteiras `k/2 ± 4096` ulps,
  nos extremos, em `NaN`/`±∞`, e em toda saída `v·255` de `65 536` valores de `v`);
- **o `hypot` que comparava contra o infinito** (`ph2d_painter_brush::accumulate_dab_smear`): o
  produto corre com `SEM_TECTO = ∞`, e `m > ∞` é falso para todo `m` — o `hypot` era pago POR TEXEL
  para nada. A guarda pergunta `tecto != ∞` (e não `is_finite()`), logo com `−∞`/`NaN` o ramo corre
  como corria.

## Medição (A/B no build do produto, corridas ALTERNADAS, mínimo de 5)

Pilha do dono, tela `1024²`, traço de `720 px`, drenagem a cada 16 eventos (um quadro). A máquina
estava sob carga de outras linhas (`load 26`–`50`), por isso os números valem pela DIFERENÇA na
mesma ronda, não pelo absoluto:

| ronda | load | passo 2: antes → agora (ms/quadro) | passo 8: antes → agora (ms/quadro) |
|---|---|---|---|
| 1 | 48 | `7,11 → 4,13` | `16,44 → 11,26` |
| 2 | 41 | `10,98 → 5,07` | `18,00 → 11,38` |
| 3 | 36 | `7,73 → 6,20` | `19,02 → 12,46` |
| 4 | 51 | `8,62 → 4,17` | `19,17 → 11,16` |

⇒ **−35 %** num traço rápido e **−45 %** num lento, e nenhuma passagem mudou um byte.

## ⛔ Recusa medida — a banda do DEPÓSITO na equipa de threads

A leitura do `gdb` pedia trocar o `std::thread::scope` do `band_split` (e dos quatro irmãos em
`height_walk` e `accumulate_batch`) pela equipa permanente do `rayon`. Foi construído (byte-idêntico,
`443/443` e `1334/1334` verdes) e medido isolado, na mesma ronda e com o resto das curas aplicado
dos dois lados:

| ronda | load | passo 2: sem → com (ms/quadro) | passo 8: sem → com (ms/quadro) |
|---|---|---|---|
| 2 | 29 | `4,41 → 4,27` | `11,39 → 11,51` |
| 3 | 27 | `4,40 → 4,51` | `12,62 → 11,79` |

⇒ **ganho zero, dentro do ruído.** O `49,6 %` era o `ptrace` a pagar a criação de threads, não o
produto. **Revertido** — uma excepção à cerca sem ganho medido é o que a cerca existe para impedir.
⚠️ *Sob `gdb`, toda sonda que cria threads mede o depurador.*

## O que fica fora, com o número

- ⭐⭐ **O alvo de CPU do build** (decisão do DONO, não desta linha): o mesmo exemplo compilado com
  `-C target-cpu=x86-64-v3` mede, na mesma ronda, `218 → 128` · `165 → 143` · `251 → 178` ms no
  passo 2 e `96 → 72` · `88 → 72` · `141 → 88` no passo 8 — **20–40 % em todo o pincel, sem uma linha
  de código**. É o `x86-64` base (2003) que transforma `round`/`floor` em chamadas de função. Subi-lo
  decide que processadores o app suporta; `x86-64-v2` (SSE4.2, 2009+) já traz as instruções de
  arredondamento e não foi medido.
- O **depósito** das camadas no plano (a maior fatia que sobra: ~`43` de ~`57` ms por traço no passo
  8) e o **campo do esfregão** (`sample_window` + `walk_dab`, em série na thread principal).

## Emenda 2026-10-01 — o borrão da pilha MISTURA em linhas, e o depósito recebe o lote do QUADRO

Medido na sonda `diag_onde_vai_o_quadro_do_rabisco` (a pilha do dono, tela `1024²`, rabisco
rápido, uma drenagem a cada `16` eventos, `--release`): **`12,33` ms por quadro**, com o depósito a
`4,4`, o Blur a `3,2` e o Smear a `3,0`.

1. **Uma quarta passagem por pixel entra nesta excepção, sob os mesmos três invariantes:** o borrão
   da pilha passa a ser DOIS passos — a convolução (já em equipa, ADR-0171) e a **mistura de volta
   por peso**, que corria num núcleo e custava MAIS do que a convolução (`1,66` contra `1,19` ms por
   quadro). Hoje `composite_linhas::peso_do_borrao` e `composite_linhas::mistura_do_borrao` correm em
   linhas disjuntas, e a lei da mistura é UMA ([`ph2d_painter_brush::mistura_linha_por_peso`]),
   chamada também pela porta em série `blur_region_por_peso`. Gate
   `a_mistura_do_borrao_em_paralelo_da_o_byte_da_serie`, contra o laço à letra **e** contra a porta
   em série. Medido: a mistura `1,66 → 0,11–0,17` ms, o peso `0,25 → 0,05–0,08`.
2. **O depósito da §«O que fica fora» foi curado SEM paralelismo novo:** com a drenagem por quadro
   o acúmulo nos planos também espera por ela (`composite_por_quadro`), e o lote de um quadro —
   uma dezena de pingos por camada — passa a alcançar a rota em BANDA que já existia
   (`stamp_banded`, byte-idêntica por construção); o pingo de um evento ficava sempre abaixo do piso
   dela. Gates `acumular_por_quadro_da_a_imagem_de_acumular_por_evento` (ao byte, depois de CADA
   drenagem), `no_impasto_acumular_por_quadro_da_o_mesmo_relevo`,
   `cada_camada_deposita_uma_vez_por_quadro` e `um_lote_que_nao_espera_esvazia_a_fila_primeiro`.
   Medido: `4,4 → 2,0` ms por quadro.

**Resultado, três corridas a `load 5`–`10`: `12,33 → 7,94`–`8,04` ms por quadro (−35 %), nenhum
byte mudado.**

⛔ **O campo do esfregão FICA em série, e a razão não é a cerca:** cada pingo compõe
`disp_novo(p) = v(p) + disp_velho(p − v(p))`, ou seja lê o campo que o pingo ANTERIOR escreveu, e o
retro-traçado cruza qualquer fronteira de banda — uma banda por linhas não é exacta. Medido por
dentro: `walk_dab` `1,04` + composição `0,99` + janela `0,06` ms por quadro, `~208 k` texels. Só o
paralelismo DENTRO de um pingo é exacto (os texels de um pingo leem a janela congelada), e ele vive
na `ph2d-painter-brush`, cuja excepção de `rayon` é outra (ADR-0158/0171) — com `~21 k` texels por
pingo, no piso medido de uma divisão (`~25 k` visitas). Fica como a próxima alavanca, com o número.

## Emenda 2026-10-01 (b) — o campo do esfregão em linhas DENTRO de cada pingo, e o perfil do smoke

Report do dono: *«pincel com size 0.5 fps cai para 40»* e, depois da emenda acima, *«não percebi
melhorias»*. A Size `0.5` (raio `128,8`) o campo do esfregão era a maior fatia da pilha.

1. **A alavanca que a nota acima deixou nomeada foi tomada:** os texels de UM pingo leem só a
   janela congelada e escrevem o seu próprio `disp`, logo as linhas de um pingo são disjuntas e
   exactas (entre pingos a ordem fica). É o **terceiro** uso do `rayon` na `ph2d-painter-brush`
   (a cerca do `Cargo.toml` dela nomeia-o), sob os mesmos invariantes: linhas disjuntas, a mesma
   lei por texel (`sculpt::Passeio::linha` + a composição), e o gate
   `as_faixas_do_pingo_dao_o_campo_da_serie` compara as duas rotas AO BIT. ⚠️ A 1.ª redacção abria
   `band_count` threads por pingo (`227 µs`); a equipa em fatias de `4` linhas custa `152 µs`
   (vale medido `2 · 4 · 8 · 16` → `230 · 152 · 164 · 186 µs`; série `810`).
2. **O perfil `smoke` compilava o motor do pincel em 16 pedaços**, e o dono corre `smoke`: a mesma
   pilha lia `14,4` ms por quadro ali contra `11,1` no `release`, com o buraco todo no acúmulo.
   `[profile.smoke.package.ph2d-painter-brush] codegen-units = 1` fecha-o (`11,2`–`11,5`).
3. **O instrumento do quadro INTEIRO:** `PH2D_COMPOSITE_RABISCO=<Size>` risca sozinho dentro da app
   e imprime o período; numa tela virtual com `PH2D_PAINT_PERF=1` a Size `0.5` leu `52`–`55` fps,
   com a pilha a `~12` ms e `~5` ms do resto do quadro.

## Emenda 2026-10-01 (c) — o campo do esfregão AO LADO do acúmulo, e os 60 fps

Ordem do dono: *«siga para os 60 fps»*. Três cortes, todos exactos (nenhum byte mudado):

1. **O campo do esfregão corre noutra thread ENQUANTO as outras camadas acumulam**
   (`composite_por_quadro::adianta_o_campo`): o laço dos pingos lê só o que leva consigo (pingos,
   Selecção, imagens por `Arc`, o campo e o rascunho), e o acúmulo escreve só nos planos. É um
   `std::thread::scope` com UMA thread, não paralelismo por pixel. A pergunta *«quem lê da caixa»*
   lê o campo do quadro ANTERIOR, logo faz-se ANTES do empréstimo — esquecê-lo encolheu a região e
   reabriu os rectângulos (`o_esfregao_nao_deixa_rectangulos_de_cor` leu `255`). E o esfregão passa a
   ter o fluxo aleatório DELE (`rng_camada`), como toda camada que acumula — antes herdava o da última
   camada acumulada. Gates `o_campo_ao_lado_do_acumulo_da_a_imagem_da_serie` e
   `o_campo_ao_lado_com_grao_aleatorio_e_impasto_da_a_imagem_da_serie`; mutação 5 de 5 (força,
   dureza, fluxo, campo deitado fora, a pergunta depois do empréstimo).
2. **Uma quinta passagem por linha entra nesta excepção:** *quem lê da caixa* varria num núcleo a
   área tocada INTEIRA a cada quadro (`1,18 → 0,18` ms); é uma caixa envolvente, logo as linhas
   repartem-se sem mudar a resposta (gate contra o laço em série).
3. **A composição devolve só a ORLA** em vez de copiar o miolo para fora e de volta.

Medido (sonda do rabisco, Size `0.5`, `--profile smoke`, `load ~7`): **drenagem `10,4 → 7,5` ms por
quadro**. Na APP (rabisco automático, tela virtual, `load 7 → 23` durante a corrida): **`60` fps**,
período `16,7` ms (o vsync), contra `52`–`55` antes desta emenda.

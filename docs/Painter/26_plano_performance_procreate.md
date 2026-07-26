# 26 — O que o Procreate de fato faz, e o plano que sai disso

**Pergunta do Enio (2026-07-25):** *"Não é possível levar o painter a tal ponto para o GPU que esses
problemas de performance desapareçam?"* → e, depois da primeira resposta: *"então o que apps de extremo
sucesso como Procreate fazem para ter performance espetacular? Investigue e pesquise"*.

> ## ⛔ ESTE PLANO FOI EXECUTADO (2026-07-25). Leia a [§7](#7--o-que-a-execução-mediu) antes da §4.
>
> A frente **T** foi construída inteira e **revertida na medição de fechamento**; a **L0** e a **U0**
> landaram e a U0 **nasceu vermelha sobre um defeito real**. O §4 abaixo fica **como foi escrito**,
> com os erros que a medição corrigiu — um plano reescrito depois do resultado não ensina nada.

**Este documento era um PLANO.** Cada frente trazia a medição que a abre (red-first), os sítios exatos,
os gates, as mutações que devem sangrar e o que **não** fazer. O §7 registra o que aconteceu com cada uma.

> ## A resposta em uma frase
>
> **A performance espetacular do Procreate é majoritariamente NÃO-GPU** — ela é *latência de pipeline*,
> *orçamento de memória em tiles* e *memória unificada*; o compositor na GPU, que é a parte que todo
> mundo imagina ser o segredo, **nós já temos**. As três coisas que faltam não se resolvem portando
> kernels para o device.

---

## 1. O que a pesquisa achou (fontes ao final)

### 1.1 O Valkyrie é real, e é menos exótico do que parece

Motor proprietário da Savage sobre **Metal**, 64-bit, escrito do zero para ARM+Metal (não é port de
desktop), **120 fps** em ProMotion, canvas até **16k × 8k**.

Mas o dado que interessa é outro: **o teto de camadas do Procreate é função da RAM do SISTEMA × os
pixels do canvas** — e quando ele triplicou no 5.2, a **própria Procreate publicou** que a causa foi o
**iPadOS 15 liberar mais RAM aos apps**, e não uma mudança de arquitetura:

| device | antes | depois (a 1920×1080) |
|---|---|---|
| M1 iPad Pro 8 GB | 250 | **500** camadas |
| M1 iPad Pro 16 GB | 250 | **902** camadas |

⚠️ **Isso é evidência de que as camadas moram em memória comum e são orçadas por BYTES.** O "GPU" está
no compositor, nos efeitos e no traço — a mesma divisão que nós já temos.

### 1.2 O fato estrutural que NÃO transfere para a nossa máquina

Apple silicon é **memória unificada**: CPU e GPU compartilham um pool físico (≈120–153 GB/s), **zero
cópias por PCIe**. Lá, *"residente na GPU"* e *"residente na CPU"* são quase a mesma frase — um
conta-gotas, um balde, um *trace* de contorno custam quase nada.

Na RTX do Enio, sobre **PCIe 4.0 x16 (~32 GB/s, 8–16× mais lento)**, todo readback é cópia + ponto de
sincronização.

> **A coisa que torna o desenho do Procreate barato é o hardware dele, não a engenharia dele.**
> Importar a arquitetura para uma GPU discreta importa o custo sem importar a vantagem.

### 1.3 Os três pilares, e o nosso estado em cada um

| pilar | o que o Procreate faz | nós |
|---|---|---|
| **Latência de pipeline** | previsão de toque (Kalman), evento processado no MEIO do frame, **front-buffer rendering** (traço vivo direto na tela, pulando a troca de buffers) | ❌ **nada disso** |
| **Tiles como unidade** | canvas em tiles; sujo, upload, undo e orçamento falam a MESMA unidade | ❌ **nenhum tile** |
| **Compositor/efeitos na GPU** | Metal | ✅ **temos** (22 modos, luz do impasto com paridade `worst delta 0`) |

Sobre latência, o número que importa: o Apple Pencil saiu de **20 ms para 9 ms** e isso **não foi
compute** — foi previsão + processamento no meio do frame. O *front buffer* é o truque final: o traço
vivo é desenhado **direto na tela**, e no pen-up transferido para a camada persistente. É a estrutura
"camada MOLHADA × canvas COMMITADO", canônica no ramo.

Sobre undo, há resposta publicada e ela é boa: a patente **US9129416B2** (Microsoft, era do Fresh Paint)
calcula undo **na própria GPU** — XOR contra o estado anterior para achar o que mudou, *bounding rect*,
compressão **RLE**, guardado num **ring buffer na GPU**, explicitamente *"evitando os atrasos de copiar
entre a memória da CPU e a da GPU"*.

E o padrão de tiles para undo é o consenso dos praticantes: pique o canvas, rastreie os tiles alterados,
guarde **atlases de tiles**; o lado GPU vira *block copy* e o custo migra para a contabilidade na CPU —
com o trade honesto de que **tile menor desperdiça menos e gerencia mais**.

---

## 2. ⚠️ Correção ao §8 do doc 25

A §8 afirma: *"O Procreate é GPU-residente (Metal), com canvas em tiles"*.

A primeira metade **não está estabelecida** e a evidência aponta para o contrário: o teto de camadas é
dirigido pela **RAM do sistema**, e o salto do 5.2 veio do **OS**, não da arquitetura. Em memória
unificada a distinção quase não existe — que é exatamente o ponto. **A frase certa é:** *o Procreate é
Metal-nativo com canvas em tiles sobre memória unificada, onde residência é uma escolha barata.*

A conclusão da §8 (*"o que nos separa não é potência, é o penhasco de roteamento"*) **segue de pé** — e
esta semana a mediu de novo: depois do fold regional, a pista GPU faz **2,61 ms/move a 4096²** contra
**2,20 ms da CPU**. **A pista GPU é mais LENTA que a CPU na tela grande.**

---

## 3. O nosso diagnóstico: o padrão dos defeitos medidos

Todo defeito de performance que esta linha encontrou tem a **mesma forma**, e nenhuma é aritmética:

| defeito | número medido | forma |
|---|---|---|
| fold do impasto na tela inteira | 202,4 ms/move @4096² | trabalho canvas-shaped num caminho por-movimento |
| `brush_settings()` respondendo um booleano | **67 MB de memcpy**, 2×/frame → 7,6 ms | **payload construído para responder uma pergunta** |
| plano livre da proteção no pen-down | 24,5 ms @4096² (vs 11,3 sem) | clone canvas-sized por gesto |
| retângulo do handoff CPU→GPU | 197.172 bytes divergentes | **duas cópias de um fato discordando** |
| cauda de `dispatch` | max 100,1 ms, 1 frame em 90 | stall externo (fora de toda fase) |
| véu do Wet Paint | 10,3 ms p50 / 33,4 max | trabalho honesto, mas por-frame e canvas-shaped |

> **Cinco dos seis são "algo do tamanho da tela foi tocado sem necessidade".** Nenhum deles é curado por
> trocar de processador; dois deles (o retângulo e o SIGSEGV) **pioram** com mais seams de GPU.

E há dois fatos do nosso código que explicam por que isso se repete:

1. **`dirty_rect` é UM retângulo, e ele UNIFICA** (`tool/paint/stamp_preview.rs:17`:
   `self.dirty_rect = Some(… union_region(acc, rect))`). Dois dabs distantes reivindicam a caixa que os
   contém **e tudo entre eles**. Um traço em L a 4096² reivindica quase a tela.
   ⚠️ E a palavra tem **duas semânticas**: `compositor/cache.rs:57` faz `mark_dirty` **sobrescrever**.
2. **O undo é capeado por CONTAGEM** (`undo.rs:285`, `DEFAULT_MAX_DEPTH = 300`) e **cada entrada carrega
   os DOIS endpoints**. Contagem é **multiplicador, não teto** — é literalmente o diagnóstico que o
   **ADR-0117** já pagou no áudio (e que emendou o HR-13: *quem declara budget possui um gate que MEDE*).

### 3.1 ⚠️ E o módulo JÁ SABIA — a pendência está escrita desde a Fase 4

O último bloco do [`00_INDEX.md`](00_INDEX.md), escrito quando o pincel nasceu, lista entre as
pendências:

> *"undo de stroke por **TILES** (hoje é snapshot full-canvas/traço)"*

**Isto não é uma ideia nova trazida da pesquisa do Procreate — é um item que este módulo nomeou
sozinho, no primeiro mês, e que ficou parado enquanto o snapshot full-canvas crescia de 1 plano
(`rgba`) para 4** (`+ heights + covers + mats`, o impasto de 2026-07-13). O custo do item **quadruplicou
em silêncio** desde que ele foi anotado.

É a lição de [[feedback_a_deferral_notes_bar_may_exceed_the_projects_policy]] na forma mais cara: uma
nota de diferido não é spec, e quem move o número que a tornava barata **tem de reconferir a nota**.

---

## 4. As frentes

Ordem deliberada: **T → L → U → R**. Cada uma abre com uma MEDIÇÃO que pode cancelá-la (§0 do CLAUDE.md:
medir antes de limitar — e antes de construir).

### Frente T — TILES: uma unidade para *sujo · upload · undo · orçamento*

**Por que primeiro.** Ela é o pré-requisito de U, reduz o custo de L, e ataca 5 dos 6 defeitos da tabela
acima. É também a única cujo ganho é estrutural em vez de pontual.

**T0 — a medição que a abre (red-first, ~meio dia).**
Sonda `measure_dirty_overclaim.rs` (irmã de `measure_window_premise.rs`): dirige traços REAIS — reto,
em L, Airbrush com scatter, Drag Dot encolhendo, Symmetry de 4 vias — e imprime, por batch:

```
texels de fato tocados · área do bbox · razão · área de uma cobertura por tiles de 64/128/256
```

⚠️ **Se a razão do produto ficar perto de 1, a frente T está cancelada** e o doc registra isso. A
hipótese (o L e o Symmetry explodem, o traço reto não) **é hipótese**, não resultado.

**T1 — o tipo.** `TileSet` em `ph2d-tool-painter/src/compositor/` (ao lado do `Region`, que é o vizinho
conceitual):

```rust
pub struct TileSet { tile: u32, cols: u32, rows: u32, bits: Box<[u64]> }  // bitset, 1 bit/tile
impl TileSet {
    pub fn mark(&mut self, r: Region);
    pub fn bounds(&self) -> Option<Region>;   // ⟵ a PONTE: o bbox de hoje, derivado
    pub fn iter_rects(&self) -> impl Iterator<Item = Region>;  // faixas coalescidas por linha
    pub fn area(&self) -> u64;
    pub fn union(&mut self, other: &TileSet);
    pub fn clear(&mut self);
}
```

**T2 — a migração é ADITIVA, e é isso que a torna segura.** O `dirty_rect` **não morre**: ele passa a
ser `tiles.bounds()`. Todo consumidor de hoje (`preview_upload_bbox`, `preview_dirty_region`, o fold,
o compositor) continua vendo exatamente o que via, **byte a byte** — e há gate exigindo isso. Só então
os consumidores migram **um por vez**, cada um com sua própria medição de antes/depois.

Ordem dos consumidores, do mais barato ao mais caro:
1. **o fold do impasto** (`impasto_gpu_planes_in` já toma uma janela — passa a tomar N);
2. **o upload por-camada** da pista GPU (`LayerPixels.dirty` já é um rect — vira lista);
3. **o composite parcial** da pista CPU;
4. **o undo** (frente U).

**T3 — o tamanho do tile é MEDIDO, não escolhido.** T0 imprime 64/128/256. O trade é público: menor
desperdiça menos e gerencia mais. O número que sair vira const **com a tabela ao lado dele** (§0).

**Gates (todos mutação-provados):**
- `a_tile_set_bounds_is_exactly_todays_dirty_rect` — a ponte; mutação: `bounds()` devolvendo a tela.
- `a_scattered_stroke_claims_far_less_than_its_bounding_box` — fixture = traço em L + Airbrush; a
  fixture **tem de conter o fenômeno** (um traço reto não o contém).
- `the_fold_cost_tracks_the_tile_count_not_the_bbox` — **razão**, nunca wall-clock.
- `a_tiled_upload_draws_what_a_full_upload_draws` — byte-identidade contra o caminho de hoje.

**⚠️ O que NÃO fazer:**
- **Não** trocar o `dirty_rect` por tiles num commit só. É o pior modo de falha deste módulo — a
  §13.13 já pagou por *"substituição por âncoras engoliu a região entre dois gates"*.
- **Não** unificar as duas semânticas de `mark_dirty` (union × overwrite) por conveniência. São
  perguntas diferentes; se convergirem, que seja com gate dizendo por quê.

---

### Frente L — LATÊNCIA: o que o artista SENTE, e é onde o Procreate ganha

**L0 — nós não temos número nenhum.** É o buraco mais gritante: o módulo mede `ms/move` e `ms/frame`, e
**nunca mediu do evento até o pixel**. Instrumento: carimbar o `Instant` no `on_canvas_pointer` e
fechá-lo no `present` do frame que o mostrou, pelo canal do `PH2D_PAINT_PERF` que já existe.

Isso rende o único número que a pergunta do Enio realmente cobra, e **o alvo é público: 9 ms**.

**L1 — o traço vivo num alvo SEPARADO do documento.** Metade já existe e por outro motivo: o
`GateSession.free` (§13.12) e o *deposit-at-commit* do Wet Paint (doc 21) já separam *o que se pinta* de
*o que se vê*. Falta a metade de DESENHO: o traço em voo compositado por cima, sem re-percorrer a pilha.

**L2 — previsão de ponteiro.** ⚠️ **Decisão de produto, não técnica**, e ela tem um conflito embutido:
**prever e estabilizar são forças opostas** — o `uses_stabilizer` de hoje *adiciona* atraso de
propósito, porque suaviza. Um preditor por cima de um estabilizador entrega um traço que corre à frente
da mão e depois volta. As três saídas (prever só com estabilizador desligado · prever e encurtar a
janela · não prever) **exigem ordem do Enio**.

**L3 — front-buffer rendering: NÃO É FAZÍVEL hoje, e é honesto dizer.** O truque do Android/Metal é
escrever direto no buffer da tela. Em `winit` + `wgpu` não há superfície *front-buffered* portável.
O que dá para fazer é reduzir o número de frames entre o evento e o present (L1) e medir (L0). Prometer
o resto seria vender o que o stack não tem.

**Gates:** o histograma de `evento → present` com p50/p99 impresso; `the_live_stroke_does_not_walk_the_stack`.

---

### Frente U — UNDO por DELTA, capeado em BYTES

**U0 — a medição que a abre**, e há precedente exato: `dhat`, no molde dos `tests/measure_*.rs` do áudio
(o ADR-0117 emendou o HR-13 justamente para isto). O que medir: pico de RAM após N passos estruturais a
2048² e 4096², com relevo e sem.

⚠️ **A aritmética grosseira já assusta e por isso mesmo precisa ser MEDIDA antes de citada:** a 4096²
uma camada tocada carrega `rgba 64 MB + heights 64 MB + covers 16 MB + mats 112 MB`. Com `max_depth =
300` **entradas de dois endpoints**, o pior caso é ordens de magnitude acima do orçamento do app. Os
`Arc` salvam o que **não** foi tocado — e um traço toca os quatro planos da camada ativa.

**U1 — o desenho** (o do ADR-0117, transposto; a patente confirma a forma):
- o passo guarda **só o lado que não está no documento** ⇒ undo e redo são a MESMA troca;
- o delta é **por TILE** (frente T é pré-requisito, e é por isso que U vem depois);
- o cap é em **BYTES**, nunca em contagem — *uma edição de camada inteira é irredutivelmente uma
  camada por passo, e `max_depth` é um multiplicador*.

**⚠️ NÃO copiar a patente literalmente.** XOR + RLE + ring **na GPU** resolve o problema de quem já é
GPU-residente; nós não somos, e fazer o undo atravessar o PCIe para poupar uma cópia de CPU seria trocar
um `memcpy` por um *stall*. A forma que se copia é *delta + rect + ring capeado*, não o lugar.

**Gates:** `the_history_is_capped_in_bytes_not_steps` (mutação: cap por contagem ⇒ pico dispara) ·
oráculo A7 de undo/redo (o do áudio, byte-idêntico ida e volta) · `dhat` determinístico, não wall-clock.

---

### Frente R — RESIDÊNCIA: o ADR, e só depois de T/L/U

**Só entra se, depois de T/L/U, o perfil ainda apontar para lá.** Hoje ele não aponta: `dispatch p50 =
0,0 ms` e o frame é vsync-bound.

O ADR tem de responder **três** perguntas, e cada uma é do produto:
1. **Readback.** Conta-gotas, balde, *trace* do Merge Curves, seleção por cor, `sample_composite_at_uv`
   — hoje grátis, na RTX cada um vira sincronização. Qual passa a ser assíncrono, e o que o artista vê
   enquanto espera?
2. **Undo.** Onde mora o histórico se os pixels estão em VRAM?
3. **Determinismo.** O LUT especular sobe pronto **para o `powf` nunca rodar no device**; o Wet Paint
   tem *fingerprint de sessão pinado*; o compositor já declara que runtime **não é bit-idêntico entre
   backends** (FMA). Levar o DEPÓSITO ao device faz *"o mesmo traço produz os mesmos pixels"* deixar de
   ser garantia entre máquinas. **Isso é uma escolha, e tem de ser escrita.**

---

## 5. Ordem, custo e critério de parada

| # | frente | abre com | cancela se | risco | **resultado (§7)** |
|---|---|---|---|---|---|
| 1 | **T0** medição do over-claim | sonda nova | razão ≈ 1 | 🟢 nenhum | ✅ feita — o bbox mente de 1,66× a 916× |
| 2 | **T1–T3** tiles | T0 | — | 🟡 contabilidade | ⛔ **construída e REVERTIDA** — a grade não pode ser mais apertada do que lhe contam |
| 3 | **L0** latência evento→pixel | instrumento | — | 🟢 nenhum | ✅ **landou** — `EVENTO->FRAME p50/p95` no `PH2D_PAINT_PERF` |
| 4 | **L1** traço vivo separado | L0 | L0 já em ~1 frame | 🟡 costura de preview | ⏸ espera o número do L0 num smoke real |
| 5 | **U0** dhat do undo | harness | pico dentro do orçamento | 🟢 nenhum | 🔴 **VERMELHA** — 1.627 MB em 24 traços |
| 6 | **U1** delta por tile | U0 + T | — | 🔴 toca o undo | 🟢 **EXECUTADA** (§7.5) — 67,8 → 2,36 MB/passo; **não** curou o pen-down, e isso é um achado |
| 7 | **L2** previsão | ordem do Enio | — | 🔴 conflita com o estabilizador | ⏸ inalterada |
| 8 | **R** residência | ADR | perfil não aponta | 🔴 arquitetural | ⏸ e o perfil aponta para OUTRO lugar (§7.4) |
| 9 | **C** coalescência (§8) | ⚠️ **não estava neste plano** — a medição a achou | razão por-evento÷coalescido ≈ 1 | 🟡 byte-identidade do lote | ⛔ **construída e REVERTIDA** (§8) — o +84% era **+86% de DABS**, não orla de lote; coalescer rende **1,00×** |

**Critério de parada, explícito:** cada frente termina com o número que a abriu, re-medido. Se o número
não se moveu, a frente **não continua** — e o doc registra o negativo, que é resultado.

---

## 6. O que este plano deliberadamente NÃO propõe

- **Portar o depósito para a GPU.** Medido: 2 ms sobre ~5.000 texels de pegada. Não é ALU.
- **Portar o solver do Wet Paint.** É **serial por semântica** (ADR-0134, *"não re-derive"*).
- **Re-derivar o fold no shader.** Seria a segunda resposta a *"como camadas de tinta se empilham"*,
  divergindo no único lugar onde ninguém lê um número: uma screenshot. A luz acertou por portar **só a
  óptica**.
- **Perseguir a cauda de 100 ms** por leitura de código. Ela está **fora de toda fase instrumentada** e
  vem precedida de salto no relógio de parede: a ferramenta é `perf`, não os olhos.

---

## 7 — O que a EXECUÇÃO mediu

> Escrito depois de construir. As frentes acima ficam como foram planejadas, **com os erros**; esta
> seção é o que a medição respondeu, e ela contradiz o plano em três pontos.

### 7.1 ⛔ A frente T foi construída inteira — e revertida

Construída: o tipo `TileSet` (bitset + `bounds()` byte-idêntico como ponte), o campo migrado em 11
sítios, o composite parcial percorrendo os retângulos, **13 gates e 6 mutações, todas sangrando** —
incluindo `a_tiled_partial_composite_is_byte_identical_to_a_full_recompose`, que passava. **O desenho
estava certo. Ele só não se paga.** Três números, nesta ordem:

| pergunta | resposta |
|---|---|
| o bbox mente? | **sim, 1,66× a 916×** |
| uma grade de tiles pega essa mentira? | **não** — a reivindicação REAL cai só ~1,4× |
| e no relógio? | **+12-14%** em dois gestos, **−75%** no mais comum |

⚠️ **A CAUSA, e é o que se leva desta frente: a grade não pode ser mais apertada do que aquilo que lhe
contam.** O `mark_dirty` recebe o bbox de cada *SEGMENTO* do traço — medido, **90×54 texels para um
pincel de 24 px**. O piso que a sonda calcula (marcar os tiles que os texels MUDADOS cruzam) é
**inalcançável**: entre 45.568 (piso) e 145.856 (bbox) a marcação real entrega 104.000. **O over-claim
mora nos CHAMADORES do `mark_dirty`, não na união deles** — e isso é uma frente diferente, mais barata
e mais bem-mirada, se algum dia valer.

**Dois erros do §4 que a medição corrigiu:**

1. **`64/128/256`** — os tamanhos que a §1 citou da literatura de praticantes — **PERDEM para o bbox**
   no gesto comum (traço curto: 10,40× contra 1,66×). Um tile de 64 são 4.096 texels e a faixa de um
   pincel r=12 tem 24 px. **O tile útil é da ordem do diâmetro do PINCEL, não da tela.**
2. **A razão sozinha decide errado.** No drag dot a grade é 4,07× *pior* em razão e **699 texels** pior
   em absoluto — que é nada. Quem decide é a grandeza **absoluta**.

⚠️ **E a sonda de fechamento reprovou a si mesma antes de reprovar a frente:** a 1ª versão media o frame
do **pen-up** — o commit, que re-suja o envelope inteiro — onde a reivindicação por tiles é igual ou
**MAIOR** que o bbox (414.208 contra 419.904). O frame do commit acontece **uma vez por traço**; os
outros sessenta são de traço ABERTO, e são esses que a sonda mede hoje.

### 7.2 ✅ L0 landou — e é o instrumento que faltava

`PH2D_PAINT_PERF` agora fecha com `EVENTO->FRAME p50=.. p95=.. max=.. ms · alvo 9`. Três decisões,
cada uma gateada: carimba o evento **mais ANTIGO** não-servido (o atraso que o artista sente é o do
primeiro do lote, não o do mais sortudo) · o carimbo fica **entre a recusa por pegada e a entrega** (um
Down que cai para o pan nunca vira pixel) · reporta **p95** ao lado do p50, porque latência se julga
pela cauda.

⚠️ **Ele para no FIM DO FRAME, não no present** — a diferença é uma fração de ms e está escrita no
código. Prometer *"→ pixel"* com um instrumento que para no fim do frame seria vender o que ele não mede.

#### 7.2.1 ⚠️ E o que ele achou no 1º uso: **o relatório era CEGO ao trabalho de pintar**

O `PaintFrameTimer` cronometra o `run_render_frame` inteiro — e o **`on_canvas_pointer` não roda lá
dentro**. Ele roda no handler de input do winit. Carimbar dabs a 4096² com impasto nunca apareceu em
`frame`, nem em `dispatch`, nem em nenhum dos 17 sub-slots.

O relatório ganhou `período real` + `eventos/frame` + `INPUT (fora do frame)`, e **a conta fecha**:

| janela | frame | INPUT | soma | **período real** | evento→frame |
|---|---|---|---|---|---|
| pintando rápido | 12,8 | **12,6** | 25,4 | **25,0** | p50 19,4 · p95 61,7 |
| pintando normal | 16,6 | 3,8 | 20,4 | **16,5** | p50 16,9 · **p95 17,8** |
| a 1ª leitura (sem o split) | 16,7 | *invisível* | — | **99,9** | p50 273,8 · p95 646,4 |

**`período = trabalho do frame + INPUT`.** É a assinatura de um custo por-EVENTO, e ela explica os três
relatos do Enio com um mecanismo só:

- *"pintar lentamente funciona bem"* — poucos eventos por frame (janela normal: 60 fps, latência de
  **exatamente um frame**, que é o piso desta arquitetura);
- *"pintar rápido cai fps"* — **~4,7 ms por evento** × 2,7 eventos/frame = 12,6 ms **somados** ao frame
  ⇒ 25 ms de período (40 fps) e p95 de 62 ms;
- *"o primeiro traço tem um delay"* — **`INPUT max` de 67 a 139 ms num ÚNICO evento**: o pen-down
  materializa os planos preguiçosos. A 4096²: `heights 67 MB + covers 17 MB + mats 117 MB` ≈ **200 MB
  alocados e ZERADOS**, mais o plano livre da proteção (24,5 ms, §13.12 do doc 25).

⚠️ **E é por isso que os ganhos anteriores não chegaram ao artista:** o fix do snapshot do pincel levou
`dispatch p50` de 7,5 para **0,0** — num número que nunca foi onde a tinta custa.

#### 7.2.2 E o `INPUT`, partido: **o pen-down é a CÓPIA DO CANVAS, e o move é honesto**

`measure_input_cost.rs`, com a variável isolada (traço de comprimento FIXO em px — a 1ª versão da sonda
escalava o traço junto com a tela e mediu razões de 4× que eu quase li como *"o move é canvas-shaped"*):

| tela | impasto | pen-down | move |
|---|---|---|---|
| 1024² | off | 0,73 | 0,75 |
| **4096²** | **off** | **11,47** | **0,75** |
| 4096² | ON | 15,74 | 2,83 |

- **O MOVE é PLANO na tela** (0,75 → 0,75 · 2,86 → 2,83). É trabalho honesto por dab; *"pintar rápido
  cai fps"* é o artista pedindo mais dabs por segundo, não um defeito. A cura ali é *dab mais barato*
  ou *coalescer o lote*, e é outra frente.
- **O PEN-DOWN é linear na ÁREA, e mesmo SEM impasto** — os cinco planos de relevo custam só 4,3 dos
  15,7 ms. Confirmado por magnitude: copiar o canvas custa **0,70 / 2,54 / 9,40 ms** a 1024/2048/4096²,
  contra um pen-down medido de **0,73 / ~3,2 / 11,47**. **O pen-down É a cópia do canvas.**

⚠️ **O mecanismo:** `paint_begin` tira um `ModelSnapshot` para o undo e ele guarda `canvas_rgba` como
`Arc` clonado; o **primeiro dab** escreve no canvas ⇒ `Arc::make_mut` vê duas referências e **copia os
64 MB**. Copy-on-write, uma vez por traço, do tamanho da tela.

⚠️ **É o MESMO defeito que a §7.3 mede pelo outro lado.** Lá o snapshot custa **memória** (1.627 MB em
24 traços); aqui custa **latência** (9,4 ms no primeiro dab de todo traço a 4096²). **A cura é a mesma —
a frente U1 — e não há atalho:** duas versões do canvas têm de coexistir enquanto o traço corre, então
UMA cópia é irredutível a menos que o passo guarde só a **REGIÃO** que o traço tocou.

⚠️ **E uma cura minha foi CONSTRUÍDA e REPROVADA pela medição no caminho:** reusar a capacidade dos
cinco planos por-traço (`clear() + resize` em vez de `vec![0.0; n]`, que o `reset_stroke_height` parecia
convidar) levou o pen-down de **17,6 para 47,5 ms**. `vec![0.0; n]` é `alloc_zeroed` — páginas já
zeradas do SO, sem escrever um byte — e reusar a capacidade obriga um **memset explícito** dos mesmos
235 MB. *Reusar memória é mais caro que pedir memória nova quando a nova vem zerada de fábrica.* O
comentário fica no `impasto.rs` para ninguém "otimizar" isso de novo.

### 7.3 🔴 U0 nasceu vermelha, e o defeito é grave

| | |
|---|---|
| um documento (4 planos, 2048²) | 64,0 MB |
| pico após 24 traços | **1.669,2 MB** |
| retido | **1.627,2 MB** (25,4 documentos) |

**Um documento por traço, linear.** O teto do app inteiro é 3.500 MB (HR-13) ⇒ 24 traços comem **46%**
dele; a 4096² quadruplica (~6,5 GB, quase o dobro do orçamento); e o cap é por **CONTAGEM**
(`DEFAULT_MAX_DEPTH = 300`), então ele **multiplica** isto por 300. É a frase do ADR-0117 no Painter,
com um zero a mais.

⚠️ **A cura é DECISÃO DO ENIO**, porque as duas custam:

1. **cap em BYTES** — resolve o teto na hora e **encurta o undo de forma visível**: com 512 MB o artista
   tem **8 passos a 2048² e 2 a 4096²**. É regressão de PRODUTO, não detalhe de implementação.
2. **histórico por DELTA** (o U1) — o passo guarda só a região que mudou, e aí o cap em bytes deixa de
   morder. É re-arquitetura do undo, a coisa em que o artista mais confia.

### 7.4 ⚠️ E o perfil aponta para OUTRO lugar

O split por estágio da drenagem parcial (`the_partial_drain_stages`, 1024²):

| estágio | ms |
|---|---|
| `composite_region(bbox 365k)` | 1,6 |
| **`apply_impasto_light(bbox)`** | **7,9** |
| `impasto_fields()` | 0,000 |
| `composite_region(TELA inteira)` | 2,8 |

**A luz do impasto custa 3× um composite de tela inteira** e domina a drenagem da pista CPU. Cortar a
área da reivindicação em 5× moveu o relógio em **5%** — o alvo da frente T estava errado desde o começo,
e nenhuma quantidade de tiles conserta isso.

⚠️ **Ela JÁ está na GPU** (o `ImpastoLightPass` landou em 2026-07-18, paridade `worst delta 0`), então
este número é da pista **CPU** — o que ele diz é *quando* a pista CPU é escolhida, e não que a luz seja
lenta em absoluto. A frente re-mirada é: **por que a pista CPU ainda é escolhida, e o que a leva a
escolher-se num documento com relevo?**

---

## 7.5 — 🟢 U1 EXECUTADA (2026-07-26): o histórico guarda a JANELA

A frente U1 foi construída. O que ela moveu, e os **dois negativos** que ela produziu — que valem tanto
quanto o positivo.

### 7.5.1 O positivo: a memória

| | antes | depois |
|---|---|---|
| retido, 24 traços a 2048² | **1.627,2 MB** (25,4 documentos) | **242,2 MB** (3,8) |
| pico | 1.669,2 MB | 345,8 MB |
| **por passo** | **~67,8 MB** — mais que um documento (os dois endpoints) | **2,36 MB** = 3,7% de um |

Cada entrada guarda os dois endpoints só em **metadados**; os dezenove planos canvas-shaped viram um
delta da janela que o passo tocou. O cap passou a ser em **BYTES**, e o orçamento é função do documento
(`2 × documento + 256 MB`, o molde do ADR-0117).

**A base de todo delta é um CURSOR** — o endpoint adjacente ao topo, materializado: UM documento,
constante, e **zero bytes em regime** porque compartilha os `Arc` do tool. ⚠️ O estado VIVO do tool
**não serve**: `restore_model` termina em `restore_shape_overlay`, que RE-CARIMBA a figura, então o vivo
depois de um undo não é byte-a-byte o snapshot instalado.

**O preço, medido e nomeado:** desfazer passou de um `Arc::clone` grátis para clonar o plano do cursor —
**0,43 ms @2048² e 13,37 @4096²** por undo, contra ~25× menos memória.

### 7.5.2 ⛔ Negativo 1: a U1 **não** cura o pen-down, e o §7.4/§4 diziam que sim

A nota do `measure_input_cost` afirmava *"é o MESMO defeito … a cura é a mesma, e é a frente U1"*. Medido
depois da U1: **16,49 → 16,07 ms** a 4096². Não se moveu.

O raciocínio que falhou: guardar só a região no HISTÓRICO decide o que sobra **depois** do traço; o que
força a cópia é precisar do estado anterior **durante** ele — e basta **UMA** segunda referência ao canvas
(o snapshot do pen-down, o cursor do histórico, o `base` de uma sessão de proteção) para o primeiro dab
pagar um `Arc::make_mut`. Tirar uma não ajuda.

⚠️ **E a decomposição refuta metade da receita da §13.12.5 do doc 25** (*"semeadura lazy por TILE **+
reuso da alocação** (mata o page-fault)"*): com o buffer já mapeado a cópia custa **11,68 dos 12,35 ms**
⇒ **a alocação vale 5%**. O page-fault não é o alvo. Sobra a outra metade, e ela é a única — capturar o
"antes" por **REGIÃO, sob demanda**, na primeira escrita de cada tile (o *tile-based undo* do
GIMP/Krita). Isso quer uma **porta única de escrita de canvas**, e hoje há ~25 sítios chamando
`Arc::make_mut` direto: **wave própria, com gates próprios**.

O número fica pinado num gate executável — `the_pen_down_is_still_a_canvas_copy_and_this_is_its_number`
(o pen-down ainda tem a ordem de grandeza de uma cópia do canvas) — que é o que vira VERMELHO no dia em que essa wave funcionar.

### 7.5.3 ⛔ Negativo 2: a constante do cap estava errada, e a medição a derrubou

O cap nasceu `512 MB` fixos, com a afirmação *"> 300 traços a 2048² e a 4096² — o cap não morde"*.
Medido (`measure_undo_capacity`): **204 traços a 1024², 62 a 2048², 17 a 4096²**. A promessa era falsa nos
dois extremos, e um teto absoluto raciona o artista justamente na tela em que ele tem menos margem.

| tela | orçamento | passo | traços | (o modelo antigo comprava) |
|---|---|---|---|---|
| 1024² | 288 MB | 2,51 MB | **114** | 9 — **12,8×** |
| 2048² | 384 MB | 8,19 MB | **46** | 3 — **15,6×** |
| 4096² | 768 MB | 28,55 MB | **26** | 1 — **17,9×** |

*Cena pesada ganha janela mais CURTA, não conta maior* — a frase do W1.5 da física, e é o que um cap em
bytes significa. O gate afirma a **RAZÃO** contra o modelo antigo, nunca um número de passos.

### 7.5.4 As duas lições de FIXTURE que a wave pagou

A mutação *"o cursor não anda com a história"* **sobreviveu duas vezes**, e as duas por fixture:

1. a 1ª versão fazia **todos** os elementos diferirem entre endpoints ⇒ todo plano caía em `Whole` ⇒ a
   materialização nunca consultava o cursor;
2. corrigida para uma janela, os estados ainda tocavam **o mesmo lugar** ⇒ o patch reescrevia exatamente
   o que o cursor errado trazia, e o fundo comum cobria o resto.

Só com a janela **variando por estado** (traços em lugares diferentes, que é o que um artista faz) o gate
mordeu. E a mutação só é pega pelo gate da CADEIA: um delta sozinho está sempre certo — o que pode estar
errado é a BASE de que ele parte, e ela só é observável a partir do segundo passo.

### 7.5.5 Um defeito escrito e pego na releitura

`diff_window` devolve `None` para *"idênticos"*, e a 1ª versão do `split` também caía nele quando o
stride não dividia o plano: os dois buffers diferiam, o passo gravava `Unchanged`, e **o undo perdia a
edição em silêncio**. As duas perguntas (*sei medir?* e *diferem?*) agora são separadas (`fits`), com
gate e mutação.

---

## 8 — ⛔ Frente **C**: COALESCÊNCIA — construída e REVERTIDA no mesmo dia

A frente que a medição achou, e que a **medição seguinte matou**. Fica escrita inteira porque a
hipótese é sedutora, o mecanismo proposto é plausível, e sem este registro a próxima pessoa (ou a
próxima eu) a reconstrói.

### 8.1 A hipótese, e a leitura que a produziu

Sonda `is_there_per_event_overhead_to_coalesce`: a **mesma** pincelada de 800 px, repartida em N
eventos de ponteiro, media **11,3 ms em 1 evento contra 20,8 em 64** — idêntico nas três telas e
**saturando**. Eu li as duas propriedades como a assinatura da **ORLA por lote**: a janela de um lote
é `bbox(dabs) ⊕ fringe`, com avanço de ~12 px e raio 100 ela é quase toda orla, então N janelas
pequenas cobririam ≈2,8× a área de uma grande. *Não é canvas-proporcional* e *satura* casavam.

### 8.2 O que foi construído

O painter passou a **acumular** os dabs das Moves de um frame e a carimbá-los **num lote só** no
`on_tick` (que já existe e já roda no topo do `run_render_frame`, antes de qualquer coisa desenhar) —
contrato congelado intocado, shell sem uma linha de mudança. Módulo novo `dab_batch.rs` (a razão de
haver DOIS buffers: `Stroke::extend`/`tick`/`settle`/`finish` todos fazem `out.clear()` na entrada,
então o vec que eles recebem nunca pode ser um acumulador), porta única `flush_dab_batch` com três
chamadores (`paint_tick`, `paint_end`, `close_stroke`), **4 gates** e **3 mutações**.

### 8.3 ⛔ E então as duas medições que faltavam

| | |
|---|---|
| dabs emitidos, 1 evento | **21** |
| dabs emitidos, 64 eventos | **39** (+86%) |
| pixels pintados | **177.760 nos dois** |
| tempo, 1 → 64 eventos | 36 → 68 ms (**+89%**) |
| **custo por-evento vs COALESCIDO** (raio 100, 2048²) | **1,00×** |

**+86% de dabs contra +89% de tempo: a correlação é a resposta inteira.** O `stamp_dabs` percorre a
pegada de **cada dab** — não uma janela por lote — então juntar os carimbos de um frame **não tem o
que economizar**, e o número é **1,00×** medido exatamente no regime que a hipótese previa como o
mais favorável (raio 100, onde a orla é máxima).

A mecânica foi **revertida inteira**. Sobreviveram as **duas sondas** (`is_there_per_event_overhead_to_coalesce`
+ `the_dab_count_grows_with_the_event_count_over_the_same_path`), que são a evidência.

### 8.4 As lições, que valem mais que a frente

1. ⚠️ **"Não escala com a tela" + "satura" não implicam "overhead de lote".** As duas propriedades são
   igualmente compatíveis com *"o trabalho por evento é constante e a contagem de eventos satura"* —
   e era isso. Eu tinha um mecanismo bonito e duas propriedades que o confirmavam, e **nenhuma
   medição do mecanismo em si**.
2. ⚠️ **A medição que faltava era a mais barata de todas** — contar dabs. Não flaka, não depende de
   perfil de build, e teria matado a frente antes de uma linha de código. *Quando a hipótese é "o
   custo é overhead", meça o TRABALHO primeiro.*
3. ⚠️ **A coluna que eu não olhei.** Na 1ª leitura eu vi `ms/evento` **caindo** e concluí o oposto do
   que a sonda dizia; a coluna que subia era o total. (Já registrado na §8 da versão anterior — e
   ainda assim não me levou a contar o trabalho.)
4. **Duas mutações minhas "sobreviventes" eram afirmações FALSAS, não buracos de gate** — e o processo
   de mutação foi o que as achou: (a) *"o flush tem de vir antes do `stroke.tick`, que limpa o
   scratch"* — buffers diferentes, e medido, **nenhum frame tem dabs de caminho E de temporizador**
   (Space/Dots: o tique emite zero; Airbrush: não acumula nada), então a posição é inerte; (b) *"o
   flush do `close_stroke` protege o pen-up"* — o `paint_end` já tem o dele, e o do `close_stroke` é
   load-bearing **só na porta de bail**. Nos dois casos a cura foi corrigir a frase e escrever o gate
   que faltava, nunca inventar fixture para "fechar" uma mutação correta.

### 8.5 ✅ O que sobra, e é OUTRA frente

**64 eventos emitem 39 dabs e pintam exactamente os mesmos pixels que 21.** A amostragem fina não
desenha mais nada — ela faz o filtro do traço (sampler de média + estabilizador) atrasar menos e
emitir mais dabs sobre a mesma linha.

⚠️ **E isto é uma PERGUNTA, não um achado** — não repito o erro da §8.1. Os dabs extras podem ser
trabalho **necessário** (com build-up de opacidade, dois dabs sobrepostos escurecem mais que um: a
contagem de `painted px` mede cobertura, **não valor**) ou **redundante**. A medição que decide é
comparar os VALORES, não o conjunto, e ela é o primeiro passo de qualquer frente sobre a lei de
emissão. Custo: uma sonda.

### 8.6 ⚠️ Correção ao §7.4: **o cache dos planos de relevo JÁ ESTÁ FEITO**

Esta parte sobrevive à reversão, porque é leitura do produto e não hipótese. A §7.4 mirou a luz da
pista CPU e a nota da §5 do `CLAUDE.md` listava *"cache com chave de versão pros planos"* como aberto:
**`impasto_gpu_planes_in(region)` existe e está em uso** (`painter_gpu_preview.rs:305`, com
`preview_gpu_region()` dando o rect confinado e `light.planes_seeded()` como segunda testemunha),
medido no doc-comment dele em **202 ms → 2,8 ms a 4096²**.

E a **janela é melhor que um cache**: não tem invalidação, logo o modo de falha que a nota temia —
*"uma luz velha que ninguém vê que é velha"* — **não existe** nesta forma. O resíduo é o fold cheio
quando `preview_gpu_region()` é `None` (edição estrutural), que **não é o caminho do move**. A nota
ficou para trás do produto.

---

## Apêndice — reproduzir os números citados

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter

# o fold, o custo por janela, a premissa da janela
cargo test -p ph2d-tool-painter --release measure_ -- --ignored --nocapture --test-threads=1

# as duas pistas no device, mesmo traço esculpido
cargo test -p ph2d-host-desktop --release --bins measure_the_sculpted_stroke -- --ignored --nocapture

# o snapshot do pincel (o gate de razão que fechou os 7,6 ms)
cargo test -p ph2d-tool-painter --release the_brush_snapshot -- --nocapture

# o split por-chamada + a LATÊNCIA (L0), no app REAL (só grava com o Painter ATIVO)
env PH2D_IMPASTO_SMOKE=2 PH2D_PAINT_PERF=1 ./target/release/ph2d-host-desktop

# T0 + o relógio da drenagem + o split por estágio (§7.1 e §7.4)
cargo test -p ph2d-tool-painter --release measure_dirty_overclaim -- --ignored --nocapture

# U0/U1 — o retido pelo historico, e o que o cap compra (§7.3 e §7.5)
cargo test -p ph2d-tool-painter --release --test measure_undo_memory -- --nocapture
cargo test -p ph2d-tool-painter --release --test measure_undo_capacity -- --nocapture

# o preco do trade (quanto custa DESFAZER) + a decomposicao do fork do pen-down (§7.5.2)
cargo test -p ph2d-tool-painter --release the_delta_history_costs -- --ignored --nocapture
cargo test -p ph2d-tool-painter --release the_pen_down_forks -- --ignored --nocapture
```

---

## Fontes

- [Procreate (software) — Grokipedia](https://grokipedia.com/page/Procreate_(software)) — Valkyrie,
  Metal, 120 fps, 16k×8k, 9 ms.
- [Layer limits set to triple on some iPads in Procreate 5.2 — Procreate](https://procreate.com/insight/2021/layer-limits)
  — **primeira parte**: o salto veio da RAM liberada pelo iPadOS 15, não da arquitetura.
- [What's my iPad's maximum layer limit? — Procreate Help](https://help.procreate.com/articles/YB7CjQ-maximum-layer-limit)
- [Procreate 5 Review: A Rebuilt Graphics Engine — MacStories](https://www.macstories.net/reviews/procreate-5-review-a-rebuilt-graphics-engine-drives-fantastic-animation-color-and-brush-tools-in-an-art-app-perfectly-tailored-to-the-ipad/)
- [Low latency stylus rendering — Android Developers](https://medium.com/androiddevelopers/stylus-low-latency-d4a140a9c982)
  — front buffer, Kalman, camada molhada × commitada.
- [Apple Pencil latency 20 ms → 9 ms — MacRumors](https://www.macrumors.com/2019/06/21/apple-pencil-latency-ipados-developers/)
- [US9129416B2 — Digital art undo and redo (Microsoft) — Google Patents](https://patents.google.com/patent/US9129416B2/en)
  — XOR + bounding rect + RLE + ring na GPU.
- [Building a GPU powered painting program — Polycount](https://polycount.com/discussion/236669/building-a-gpu-powered-painting-program)
  — tiles + atlas para undo, e o trade tamanho × gerenciamento.
- [Tailor your apps for Apple GPUs and TBDR — Apple Developer](https://developer.apple.com/documentation/metal/tailor-your-apps-for-apple-gpus-and-tile-based-deferred-rendering)
- [Unified Memory: Apple Silicon vs NVIDIA — Seresa](https://seresa.io/blog/ai-data-readiness/what-is-unified-memory)
  — ≈120–153 GB/s unificados × ~32 GB/s de PCIe 4.0 x16.

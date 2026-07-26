# 26 — O que o Procreate de fato faz, e o plano que sai disso

**Pergunta do Enio (2026-07-25):** *"Não é possível levar o painter a tal ponto para o GPU que esses
problemas de performance desapareçam?"* → e, depois da primeira resposta: *"então o que apps de extremo
sucesso como Procreate fazem para ter performance espetacular? Investigue e pesquise"*.

**Este documento é um PLANO, não trabalho feito.** Nada aqui foi construído. Cada frente traz a medição
que a abre (red-first), os sítios exatos, os gates, as mutações que devem sangrar e o que **não** fazer.

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

| # | frente | abre com | cancela se | risco |
|---|---|---|---|---|
| 1 | **T0** medição do over-claim | sonda nova | razão ≈ 1 | 🟢 nenhum |
| 2 | **T1–T3** tiles | T0 | — | 🟡 contabilidade |
| 3 | **L0** latência evento→pixel | instrumento | — | 🟢 nenhum |
| 4 | **L1** traço vivo separado | L0 | L0 já em ~1 frame | 🟡 costura de preview |
| 5 | **U0** dhat do undo | harness | pico dentro do orçamento | 🟢 nenhum |
| 6 | **U1** delta por tile | U0 + T | — | 🔴 toca o undo |
| 7 | **L2** previsão | ordem do Enio | — | 🔴 conflita com o estabilizador |
| 8 | **R** residência | ADR | perfil não aponta | 🔴 arquitetural |

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

## Apêndice — reproduzir os números citados

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter

# o fold, o custo por janela, a premissa da janela
cargo test -p ph2d-tool-painter --release measure_ -- --ignored --nocapture --test-threads=1

# as duas pistas no device, mesmo traço esculpido
cargo test -p ph2d-host-desktop --release --bins measure_the_sculpted_stroke -- --ignored --nocapture

# o snapshot do pincel (o gate de razão que fechou os 7,6 ms)
cargo test -p ph2d-tool-painter --release the_brush_snapshot -- --nocapture

# o split por-chamada, no app REAL (só grava com o Painter ATIVO)
env PH2D_IMPASTO_SMOKE=2 PH2D_PAINT_PERF=1 ./target/release/ph2d-host-desktop
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

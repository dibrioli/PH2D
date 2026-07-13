# HANDOFF — continuação da linha `line/FLIP` (COMECE AQUI)

> **Para:** o próximo **agente-de-linha** de `line/FLIP` (o Flip = 4º meio do PH2D: animação
> quadro-a-quadro, fork 2D clean-room do Grease Pencil — [ADR-0114](architecture/decisions/0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md)).
> **De:** o agente anterior (fechou a **W5 — Reshape**; deixou UM bug aberto, com a **causa provada
> e o gate vermelho escrito**). **Data:** 2026-07-13 · **Regime:** Modo L (workstation).
>
> **Sua 1ª tarefa é o §C.** Não é uma investigação do zero: a causa está provada, o gate vermelho
> existe no repo, e o fix cabe em poucas linhas. O que custa caro é o que vem **junto** com ele — as
> armadilhas do §C.4 são reais e já morderam três vezes.
>
> **Leia primeiro, nesta ordem:** `CLAUDE.md` §0 → [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md)
> (inteira, e releia a cada passo) → este arquivo → [`Flip/BUGS_flip.md`](Flip/BUGS_flip.md) **#14, #15
> e #16** (a saga do balde; o §C é o capítulo seguinte dela).

---

## §A — Como esta linha trabalha (Modo L)

- **Worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP/`, branch `line/FLIP`.
  **Trabalhe SEMPRE dentro dela** — o mesmo path relativo existe na raiz do repo, e editar
  `crates/...` na raiz é editar a **árvore errada**. Mutação sempre por caminho ABSOLUTO.
- **Foundational você PODE e DEVE tocar** (`ph2d-editor-core`, `ph2d-ui-testkit`, o shell), com
  cuidado (ADR-0107). Ao criar algo foundational, projete para **isolamento** (módulo irmão, extensão
  append-only) e **anote os ids/consts novos no handoff de integração**.
- **PARE e reporte ao Enio em 2 casos só:** (a) contrato congelado (CLAUDE.md §6 — exige ADR);
  (b) rebase conflitando **fora** dos seus arquivos.
- **Commits locais frequentes** (`git commit --no-verify`). **NUNCA** `push`, `--force`, `git add -A`
  fora da sua árvore. **Zero CI durante a jornada.**
- **Inner loop = só `cargo check -p`.** Teste/clippy/auditoria **1× no fechamento do bloco**.
- **Você NÃO integra e NÃO faz ship.** Fecha → escreve o handoff de integração (DIRETRIZ §1.5.9) →
  **PARA**.
- **UI em inglês; comentário de código em pt-BR.** O `typos` roda **da raiz** (com paths explícitos
  ele perde o `.typos.toml` e acusa palavra portuguesa à toa).

**Base:** a linha está **rebasada sobre o `main`** (a integração das 6 linhas foi em 2026-07-12) e
tem commits locais **não integrados** (W5 + os fixes do smoke). Confira com `git log --oneline main..HEAD`.

---

## §B — O estado (o que existe, e não se reimplementa)

| Wave | O que é | Onde |
|---|---|---|
| **W0** | modelo (`ph2d-flip`: objeto → camadas → quadros → desenhos → traços SoA) | `crates/ph2d-flip` |
| **W1+WT** | render GPU; cobertura = **união global** da polilinha | `ph2d-flip-render` |
| **W2** | tool + painel docado **modal** + caneta + borracha | `ph2d-tool-flip`, `ph2d-panel-flip` |
| **W3** | frames · exposição · ciclos · ghosts · tween + a tira | `ph2d-panel-flip-frames`, `flip_strip.rs` |
| **W4** | **o balde** (solver CPU) + Gap Closure persistente + o alvo vivo | `ph2d-flip-fill`, `flip_fill.rs` |
| **W5** | **a escultura** — 8 pincéis (Smooth/Push/Grab/Pinch/Twist/Thicken/Strength/Jitter) | `ph2d-flip-reshape`, `flip_reshape.rs` |

**As regras do módulo que NÃO podem ser re-derivadas erradas** (cada uma custou rodadas — detalhe em
`BUGS_flip.md` e em `docs/Flip/07_reshape_escultura.md`):

1. **O traço é a união global da polilinha** (BUGS #1). Com depth first-wins, *quads sobrepostos têm
   de computar a MESMA máscara*.
2. **O balde ancora no EIXO da linha** (BUGS #14) — a espessura é absoluta em px de TELA e o fill é
   assado em DOC; qualquer âncora derivada da espessura descola com o zoom.
3. **A cor entra POR BAIXO da linha** (BUGS #15): o contorno de um fill vetorizado é rasterizado na
   cor do fill com a espessura da linha (a "dilatação") — senão a metade externa da linha não tem cor
   por baixo e, com pincel macio, o fundo vaza.
4. **A forma fechada pinta A SI MESMA** (BUGS #16 — e é o §C): o preenchimento de uma forma é o `fill`
   do **próprio traço** (a triangulação dos pontos dele), como no GP. Um conjunto de vértices só.
5. **O autokey é por FERRAMENTA**: caneta cria chave em BRANCO; **borracha e escultura DUPLICAM**.
6. **Há TRÊS relógios** (BUGS #7): `drawing_at` · `source_frame` · `authoring_frame`.
7. **A escultura move as REGIÕES e os buracos delas** — senão a cor fica para trás quando a linha anda.

---

## §C — 🟥 A 1ª TAREFA: o balde não reconhece a forma desenhada À MÃO

### C.1 — O sintoma (smoke do Enio, 2026-07-13, com screenshot)

> *"Quase perfeito, mas nem todo vertex da linha está conectado ao vertex de fill — o fill provavelmente
> não foi gerado conforme o número de vertex da linha. Isso cria áreas de dessincronização e gaps."*

Na tela: a cor **corta** os entalhes da linha (um "V" fica preenchido por cima), transborda em trechos
retos e recua em outros. **É o contorno vetorizado**, não a forma.

### C.2 — A causa, PROVADA (não é hipótese — há gate)

O fix do BUGS #16 fez o balde **pintar o próprio traço** quando a região é o interior de uma forma
fechada (`flip_fill::filled_shape_target`). O critério exige `s.closed`.

> **Um traço desenhado à mão NÃO é `closed`.** O `flip_draw::build_stroke` só fecha o traço no modo
> `Shape: Filled` — a caneta normal produz `closed = false`, mesmo quando a mão encosta a ponta no
> começo. **Logo o auto-preenchimento nunca disparou no produto**, e todo fill do Enio caiu no caminho
> vetorizado (que dessincroniza, e cujo erro o zoom amplia — BUGS #16 §"Por que o defeito parecia
> grande").

**A asserção-vermelha já está escrita** (e marcada `#[ignore]` com o motivo):

```
cargo test -p ph2d-host-desktop a_hand_drawn -- --ignored
```
`shells/desktop/src/flip_fill_tests.rs::a_hand_drawn_shape_is_not_closed_and_the_bucket_misses_it`
— hoje VERMELHO: o balde cria um 2º traço em vez de pintar a forma.

### C.3 — O fix (o que fazer)

**Não exija `closed` para reconhecer a forma — e não feche o traço do usuário.**

1. **`shells/desktop/src/flip_fill.rs::filled_shape_target`**: tirar o `s.closed` do filtro. O
   critério que fica de pé (e que separa "preencheu a forma" de "preencheu um pedaço entre duas") é o
   resto: line-art (não-região) · o **clique dentro** do polígono dos pontos · a **área** do contorno
   traçado batendo com a do traço (±`AREA_TOL`).
2. **`crates/ph2d-flip-render/src/pack.rs`**: o fill hoje só é empacotado se `s.closed && s.fill.is_some()`.
   Um traço ABERTO com fill precisa renderizar — **o polígono fecha implicitamente** (é o que o GP faz:
   a triangulação dos pontos da curva não pergunta se ela é cíclica). Tire o `s.closed` da condição do
   **fill** (a do *stroke* não muda: fechar a linha desenharia um segmento que o usuário não fez).
3. **Cuidado com o inverso:** NÃO sete `closed = true` no traço ao anexar. Isso desenharia um segmento
   novo ligando as pontas — mudaria a arte do usuário (e, num traço cujas pontas ficaram longe, uma
   linha atravessando o desenho).

**Verde esperado:** o gate de C.2 passa (tire o `#[ignore]`), e o smoke mostra a cor grudada na linha,
com as quinas afiadas, em qualquer zoom.

### C.4 — As armadilhas (as três que já morderam)

1. **O oráculo é o PIXEL, não a geometria.** Três gates de geometria diziam "a borda do fill está a
   0,3 px do eixo" — e a tela mostrava a cor descolada. Renderize e OLHE:
   `crates/ph2d-flip-render/tests/gpu_fill_fit.rs` (grava PNGs em `/tmp`, mede vazamento e transbordo).
   Rode `-- --ignored --nocapture`. **Estenda-o** com a sua cena antes de declarar vitória.
2. **Varra o ZOOM.** O erro do contorno vetorizado é assado em DOC e a linha é px de TELA: o zoom
   **multiplica** o desvio. Um gate que mede num zoom só não vê o defeito (foi assim três vezes:
   BUGS #11, #14, #16).
3. **Um teste que acusa o código pode estar errado ele mesmo** (BUGS #13). Já aconteceu 3×: o Twist
   (esperava `y>0` num ponto à esquerda do cursor), o autokey (semeou o playhead a 12 fps num objeto
   de 24), o traço preenchido (o `smoothing` default colapsou um quadrado de 4 amostras). **Antes de
   acreditar, confira que o teste descreve o que você acha que descreve.**

### C.5 — O beco que NÃO vale a pena repetir (custou uma rodada)

Tentei **costurar** o contorno vetorizado à linha (projetar os vértices no eixo + reinserir os
vértices que o RDP jogou fora). Funciona em geometria mansa e **destrói o anel numa quina aguda**: os
dois lados do bico estão à mesma distância, a projeção alterna, o contorno vai-e-volta, a área vira
zero (`Degenerate` no donut, no Gap Closure e na estrela). Impor a direção do percurso salvou dois dos
três casos — e aí ficou claro que o caminho era outro: **não vetorizar**. *Proximidade não é ordem.*
(O código foi revertido; a lição está no BUGS #16.)

### C.6 — O limite honesto (não é bug, é o modelo)

**Uma região delimitada por VÁRIOS traços continua vetorizada** — ali não existe "a curva" para
carregar a cor, e o **balde do Grease Pencil faz exatamente o mesmo**. A dilatação (BUGS #15) é o que
segura esse caminho. Se um dia o Enio cobrar precisão *também* nele, o caminho é o snapping/costura
(§C.5) feito direito (map-matching com continuidade), **não** um remendo de tolerância.

### C.7 — O smoke (entregue com o `cd` junto)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && PH2D_FLIP_DEMO=1 cargo run --release -p ph2d-host-desktop
```
1. Desenhar uma forma **à mão** (fechando o contorno com a mão) → **balde** dentro dela → a cor tem de
   grudar na linha, com as quinas afiadas.
2. **Dar zoom até 5×** → a cor não pode descolar em vértice nenhum.
3. **Sculpt** → empurrar a linha → a cor acompanha exatamente.
4. **Unpaint** → sai a cor, **fica a linha**.

---

## §D — Depois do §C: a fila

1. **Edit Mode / seleção de traço** (o "select do traço" que o Enio pediu). Destrava o auto-masking
   fino do Reshape (a máscara passa a ser a SELEÇÃO — o gancho já existe num ponto só,
   `Session::begin`) e substitui o "alvo vivo" (`flip_live.rs`) como alvo dos ajustes do painel.
   Modelo de seleção especificado em [`Flip/02 §11`](Flip/02_referencia_algoritmos_blender_5.2.md).
2. **Carry-overs da W4/W5** (curtos, isolados): overlay ao vivo do Gap Closure (o `closures()` já
   devolve os segmentos) · modo Radius do Gap · **T5.7 multiframe** (o `frame_falloff` já é respeitado
   pelos 8 pincéis; falta a **multi-seleção de chaves na tira** — mesma dependência do fill multiframe:
   considere fazer os dois juntos).
3. **Colorize** (wave própria): trapped-ball → LazyBrush/CTG com onion-fill ([`Flip/04 §3`](Flip/04_alem_do_blender.md)).
4. **Refinos:** duplicar/agrupar camada · reorder por drag · máscaras de camada na UI · pressão real da
   caneta (o mouse manda 1.0; o funil da influência já a recebe) · round caps / bevel joins.
5. **⏸ W6 (timeline global): ADIADA** por ordem do Enio até a timeline principal fechar. O playhead do
   Flip **já é o global**. (A linha `anim` trouxe seletor de clips e relógio único — se o Enio reabrir,
   leia o handoff dela antes.)

---

## §E — Comandos

**Gate batched (1× no fechamento):**
```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && \
cargo test -p ph2d-flip -p ph2d-flip-fill -p ph2d-flip-render -p ph2d-flip-reshape \
           -p ph2d-tool-flip -p ph2d-panel-flip -p ph2d-panel-flip-frames \
           -p ph2d-ui-testkit -p ph2d-editor-core -p ph2d-host-desktop && \
cargo test -p ph2d-flip-render --test gpu_render --test gpu_fill_fit -- --ignored && \
cargo clippy -p <suas-crates> --all-targets && \
rustup run 1.95 cargo fmt -p <suas-crates> && typos && \
cargo build --release -p ph2d-host-desktop
```
(Arch-gates que **vão** te pegar: LOC 700/crate e 600/shell — **split em módulo irmão, nunca
allowlist**, e rode `fmt` ANTES de medir · `node_id_collisions` · `architecture_panel_wiring_parity` ·
`no_tofu_glyphs`.)

**Diagnóstico do balde:** `PH2D_FLIP_FILL_DEBUG=1` (imprime px_to_world, a escala do buffer, a
meia-espessura em px de tela e o contorno, a cada clique).

**Referência do Blender** (GPL — **comportamento, nunca código**): `~/Downloads/blender-5.2-grease-pencil-ref/`.
Os dois achados que mais renderam: o fill do GP é a **triangulação dos pontos da própria curva**
(`blenkernel/grease_pencil.cc:477`) e o sculpt edita **todas** as curvas
(`grease_pencil_utils.cc:949`).

**Docs do módulo:** [`docs/Flip/`](Flip/00_README.md) — `01_plano_waves` · `02_referencia` (§7 = o
Reshape) · `03_traco_rasterizacao` · `04_alem_do_blender` · `05_frames_ghost_tween` · `06_fill_balde` ·
`07_reshape_escultura` · **`BUGS_flip.md`** (leia #14, #15, #16 antes de tocar no balde).
Tracker: [`HANDOFF_flip_impl.md`](HANDOFF_flip_impl.md).

---

**Você fecha a linha, escreve o handoff de integração, e PARA. Não integra. Não pusha.**

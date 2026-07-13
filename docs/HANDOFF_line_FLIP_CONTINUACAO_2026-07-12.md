# HANDOFF — continuação da linha `line/FLIP` (COMECE AQUI)

> **Para:** o próximo **agente-de-linha** que assumir `line/FLIP` (o Flip = 4º meio do PH2D:
> animação quadro-a-quadro, fork 2D clean-room do Grease Pencil — [ADR-0114](architecture/decisions/0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md)).
> **De:** o agente anterior (fechou W4 + a âncora no eixo; **integrado ao `main` em 2026-07-12**).
> **Regime:** Modo L (workstation).
>
> **Leia primeiro, nesta ordem:** `CLAUDE.md` §0 (inegociáveis) →
> [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) (inteira, e RELEIA a
> cada passo) → este arquivo → [`Flip/BUGS_flip.md`](Flip/BUGS_flip.md) (§3 abaixo diz por quê).

---

## §0 — Estado da linha (você começa AQUI)

| | |
|---|---|
| **Branch** | `line/FLIP` |
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP/` |
| **Base** | **= `main` atual** (fast-forward feito; **0 commits à frente, 0 atrás**, árvore limpa) |
| **Tudo o que a linha fez** | **JÁ ESTÁ NO `main`** — W0, W1, WT (o traço), W2, W3, W4 + a âncora no eixo |
| **Sanidade da base combinada** | ✅ crates Flip **151/151** · `ph2d-host-desktop` **435/435** · GPU do traço **17/17** · arch-gates (LOC, node-ids, wiring-parity, tofu) verdes · `--release` compila |

O `main` juntou **6 linhas** (Painter · Vector · FLIP · Áudio · Motion-value · Anim) + 2 ships.
**O Flip sobreviveu intacto** — os números acima foram medidos na árvore combinada, não na linha.

### O que a integração mudou por baixo de você (3 coisas, todas benignas)

1. **`MockPanelHost::store()` virou método INERENTE** (a linha do Áudio) — o `use PanelHostInternal`
   ficou órfão e o ship o removeu do `ph2d-panel-flip/tests/seam.rs`. Ao escrever seam tests novos,
   **não importe o trait**.
2. **`shells/desktop/src/input_handlers.rs` foi SPLITADO** (`83e596b7`): o cap de 600 LOC só estourou
   na **soma** das linhas (Flip + Motion + Áudio apendaram teclas cada uma). Os handlers do Flip
   continuam lá e verdes — mas se você apendar tecla nova, **meça o arquivo depois do `fmt`**.
3. **`PROJECT_SCHEMA = 7`** (quatro donos, um contador) e **`FLIP_SCHEMA_VERSION = 3`**. Postcard é
   posicional: mudou a forma de um struct serializado do Flip → **bumpe o `FLIP_SCHEMA_VERSION`**,
   e se o `ProjectState` mudar de layout, o `PROJECT_SCHEMA` também. Ver
   [[feedback_numbers_that_sum_across_lines_count_dont_pick]] — na integração, um número que várias
   linhas incrementam **SOMA**; não "escolha um lado" do conflito, **conte na árvore fundida**.

### O MODO L em 6 linhas (o protocolo que você segue)

1. **Trabalhe SEMPRE dentro da worktree** (`Worktrees/line-FLIP/`). O mesmo path relativo existe na
   raiz do repo — editar `crates/...` na raiz é editar a **árvore errada**. Na dúvida: `pwd`.
   (Corolário: **mutação por caminho ABSOLUTO** — `sed -i`/script com path relativo escreve no `main`.)
2. **Foundational você PODE e DEVE tocar** (`ph2d-editor-core`, `ph2d-ui-testkit`, o shell), com
   cuidado (ADR-0107). Ao **criar** algo foundational, projete para **isolamento**: módulo irmão novo
   em vez de engordar arquivo compartilhado; ponto de extensão **append-only**; id/const/variant novo
   = pegue o próximo livre e **anote no handoff de integração**.
3. **PARE e reporte ao Enio SÓ em 2 casos:** (a) **contrato congelado** (CLAUDE.md §6 — `Tool=12`,
   `CanvasPaintTool`, `PanelEvent=4`, nodes, vector-doc: exige ADR); (b) **rebase conflita FORA dos
   seus arquivos** (mesmo-símbolo com outra linha). Nunca negocie direto com outra linha.
4. **Commits locais frequentes:** `git commit --no-verify -m "..."`. **NUNCA** `push`, `--force`,
   `git add -A`. **Zero CI durante a jornada.**
5. **Inner loop = só `cargo check -p <crate>`.** Teste/clippy/auditoria **1× no fechamento do bloco**,
   nunca por task.
6. **Você NÃO integra e NÃO faz ship.** Fecha o módulo → escreve o **handoff de integração**
   (DIRETRIZ §1.5.9; o modelo é [`HANDOFF_line_FLIP_integracao_2026-07-12.md`](HANDOFF_line_FLIP_integracao_2026-07-12.md))
   → **PARA**. Integrar/pushar por conta própria = violação do protocolo.

---

## §1 — O que já está pronto (não reimplemente)

O Flip é um **app de animação funcional**: desenha, apaga, preenche, tem quadros, exposição, ciclos,
fantasmas e tween. Tudo abaixo está no `main`, testado e **smokado pelo Enio**.

| Wave | O que é | Onde |
|---|---|---|
| **W0** | modelo (`ph2d-flip`: objeto → camadas → quadros → desenhos → traços SoA); cada objeto é entidade ECS (`FlipObjectRef`) | `crates/ph2d-flip` |
| **W1 + WT** | render GPU; **a cobertura de um fragmento é a UNIÃO GLOBAL da polilinha** (não a distância ao próprio segmento) | `crates/ph2d-flip-render`, `render_loop/flip_pass*.rs` |
| **W2** | tool (Select/Draw/Erase/Fill) + painel docado **modal** + caneta + borracha (3 modos) | `ph2d-tool-flip`, `ph2d-panel-flip`, `shells/desktop/src/flip_{draw,erase,layers}.rs` |
| **W3** | **frames · exposição · ciclos · ghosts · tween** + a tira docada | `ph2d-panel-flip-frames`, `flip_strip.rs`, `flip_autokey.rs` |
| **W4** | **o balde** (solver CPU puro) + Gap Closure persistente + o **alvo vivo** | `ph2d-flip-fill`, `flip_fill.rs`, `flip_live.rs` |

**As 5 regras do módulo que NÃO podem ser re-derivadas erradas** (cada uma custou rodadas):

1. **O traço é a união global da polilinha.** Com depth first-wins, *quads sobrepostos têm de computar
   a MESMA máscara* — senão a borda quase-transparente de um segmento apaga o núcleo opaco de outro
   (a "mordida"). Peças obrigatórias: janela `p0/p3` + vizinhos geométricos (broadphase no `pack`) +
   **uma** `capsule_dn` + clamp/fade sub-pixel. BUGS #1 · [[project_flip_stroke_analytic_coverage_gp]].
2. **O balde ancora no EIXO da linha**, nunca na silhueta. A espessura é absoluta em px de TELA e o
   fill é assado em unidades de DOC — qualquer âncora derivada da espessura transborda `(w/2)·(zoom−1)`
   px quando o usuário dá zoom **depois** do clique. BUGS #14 ·
   [[feedback_anchor_must_be_invariant_under_user_transforms]].
3. **O autokey é por FERRAMENTA** (`flip_autokey::target_drawing`): caneta cria chave em BRANCO (ou
   duplicata sob *Additive*); **borracha/escultura SEMPRE duplicam** — senão o usuário apaga um quadro
   novo e vazio enquanto o desenho que ele VÊ fica intacto num quadro anterior. **A W5 (Reshape) cai
   na segunda categoria: `FlipEdit::Modify`.**
4. **Há TRÊS relógios** (BUGS #7): `drawing_at` (cru) · `source_frame` (*o que está na tela* — render,
   ghosts, célula destacada) · `authoring_frame` (*onde este gesto escreve* — caneta, borracha). Sob
   `Hold`/`None` o tempo depois do vão é tempo **NOVO**; sob Loop/PingPong ele **repete**. Colapsá-los
   quebra uma feature ou a outra.
5. **O alvo vivo guarda o INSUMO, não o resultado** (`flip_live.rs`): as amostras cruas do traço, a
   lista de traços pristina de antes do fill. Reaplicar sobre o resultado anterior faz os parâmetros
   se **comporem** e o slider deixa de ser reversível.

---

## §2 — A FILA (em ordem recomendada; o Enio decide a final)

### ETAPA 1 (recomendada) — **W5: Reshape** (escultura de traço)

Os **9 pincéis** com a matemática e as constantes **já tabeladas** em
[`Flip/02 §7`](Flip/02_referencia_algoritmos_blender_5.2.md) (Smooth · Push · Grab · Pinch · Twist ·
Thickness · Strength · Randomize · Clone). Nada a pesquisar — é porte fiel.

- **Contrato:** trait `ReshapeBrush` com `on_stroke_begin/extended/done` + `InputSample{pos, pressure}`
  — casa direto no `CanvasPaintTool` que a tool Flip já implementa (**contrato congelado: não mexa
  nele**, ele já serve).
- **A dose é por SAMPLE de input**, não por tempo: mover devagar aplica mais. Um fork que gere samples
  por timer muda a sensação de **todos** os pincéis.
- **Auto-masking congelado no DOWN** (camada ativa / material sob o cursor com threshold 20 px):
  arrastar para fora nunca pega traços novos. *A máscara define O QUE; o traço define QUANTO.*
- **Constantes que fazem "sentir como GP"** (aditivas, nunca multiplicativas): Thickness `±influence·0.001`,
  Strength `±influence·0.125`, Pinch `influence²/25`, Twist `±1°/sample`, Smooth `iterations = 2`.
- **Randomize** = splitmix64 (mesma família do `jitter.rs` do Painter) — replay-safe, HR-5.
- **Grab** congela máscara+pesos no DOWN (com `pressure = 1.0` fixo) e nunca reavalia.
- **Clone é um COMANDO**, não um pincel (os modos contínuos do GPv2 são admitidamente quebrados).
- **Autokey:** política `Modify` (regra 3 do §1).
- **Reserve o `falloff` multiframe na assinatura desde o início** — ele liga quando a tira ganhar
  seleção de MÚLTIPLOS frames (hoje ela seleciona um).
- **DoD:** os 9 no painel (seção própria, modal como as outras) + seam test que DIRIGE o evento real
  + gate de PINTURA (`MockPanelHost::paint`) + smoke do Enio.

### ETAPA 2 — **Edit Mode / seleção de traço** (o "select do traço" que o Enio pediu)

Hoje o alvo dos ajustes do painel é *"a última coisa que você fez"* (`flip_live.rs`) — um paliativo
declarado. O modelo de seleção está especificado em [`Flip/02 §11`](Flip/02_referencia_algoritmos_blender_5.2.md)
(traço / ponto / segmento + transform).

- **Sinergia com a ETAPA 1:** a seleção é uma das fontes de **auto-masking** do Reshape — fazer o
  Reshape primeiro deixa o gancho pronto; fazer a seleção antes obriga a voltar no Reshape.
- Quando existir, **a seleção vira o alvo do painel** (o `flip_live` já foi escrito prevendo isso —
  leia o doc do módulo antes de trocar).

### ETAPA 3 — **Carry-overs da W4** (curtos, isolados, bons para uma jornada aquecer)

- **Overlay ao vivo do Gap Closure:** `gap::closures()` já devolve os segmentos; falta desenhá-los
  enquanto o slider mexe (hoje o usuário ajusta às cegas).
- **Modo Radius do Gap** (o 2º modo do GP; hoje só existe o de extensão).
- **Fill multiframe** — *depende* da multi-seleção de chaves na tira (mesma dependência do falloff da
  ETAPA 1; considere fazer as duas juntas).

### ETAPA 4 — **Colorize** (wave própria, a feature de produção)

Trapped-ball ("colorir tudo de uma vez") → LazyBrush/CTG com onion-fill — spec completa em
[`Flip/04 §3`](Flip/04_alem_do_blender.md). *Só o TVPaint tem.* É grande: não a misture com outra etapa.

### ETAPA 5 — **Refinos** (não-bloqueantes, pegue quando o Enio reclamar)

Camadas: duplicar/agrupar, reorder por DRAG (hoje só ↑↓), máscaras na UI (o modelo já carrega
`FlipLayer.masks`) · Borracha: raio dedicado + preview do círculo · Pen real: curva de pressão
editável (mouse = 1.0 hoje) · Traço: round caps / bevel-round joins (deferidos no W1) · **Congelar o
contrato do `ph2d-flip`** com gate de superfície, quando o modelo assentar.

### ⏸ **W6 — timeline global: ADIADA por ordem do Enio** (não pegue sem perguntar)

A tira própria (`ph2d-panel-flip-frames`) é a UI de tempo até lá, e o playhead **já é o global**
(`ph2d_core::Playhead`) — não haverá relógio a reconciliar. **Mas o contexto mudou:** a linha `anim`
integrou seletor de clips, relógio único (`MotionTransport` morreu) e composição de clips desenhada
([ADR-0115](architecture/decisions/)). Se o Enio quiser reabrir a W6, **pergunte antes** — e leia o
handoff da linha anim.

---

## §3 — Armadilhas desta área (o que JÁ mordeu — leia `BUGS_flip.md` inteiro)

1. **Verde-de-compilação vale ZERO no audit.** A W4 foi entregue com **1251 testes verdes e incapaz de
   preencher um círculo** (BUGS #10). E a W3: os ciclos tinham 6 testes verdes **e o render nunca os
   chamava** (BUGS #7). *Unit-verde ≠ funciona no produto.*
2. **O oráculo modela a APARÊNCIA, não a implementação** (BUGS #2 — a lição mais cara da linha). Um
   teste derivado do seu shader fica **verde com o bug na tela**. Derive o esperado da *definição do
   objeto* (o traço macio **é** a união dos discos varridos), rode-o **VERMELHO no código atual**, e só
   então implemente. **Prove as mutações.**
3. **Use os números do PRODUTO — e varra a FAIXA deles.** `px_to_world = 1.0` é o único valor que
   esconde erro de unidade (BUGS #10a); um **zoom** não-varrido escondeu os BUGS #11 e #14. Ferramentas
   prontas: `PH2D_FLIP_FILL_DEBUG=1` (a régua do balde no app real) e
   `cargo test -p ph2d-flip-fill sweep_table -- --ignored --nocapture` (espessura × zoom).
4. **Ao acrescentar campo ao modelo, ache os choke points de CÓPIA** (BUGS #10c): `FlipStroke::clone_attrs`,
   `flip_erase::new_like`, `cleanup_soft`. A W4 adicionou 2 campos e atualizou **um** — o tween perdia o
   furo do "O" e a borracha macia apagava todos os fechamentos de gap do desenho.
5. **PINTADO ≠ populado** (BUGS #8): widget registrado, wirado e contract-limpo pode **não existir na
   tela**. Todo widget novo ganha o gate `MockPanelHost::paint::<P>()`.
6. **Esconder um controle é pior que deixá-lo transbordar** (BUGS #9): a barra que descartava o que não
   cabia sumia com 9 de 18 controles num monitor de 1280. Layout cede em **espaço**, nunca em existência.
7. **Overlay é decisão de z, não de cor** (BUGS #6): o fantasma **pertence à sua camada** — entra na
   pilha do compositor na posição dela, nunca num passe por baixo de tudo.

---

## §4 — Comandos (copie e cole)

**Inner loop:**
```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && cargo check -p ph2d-flip-render
```

**Gate batched (1× no fechamento do bloco):**
```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && \
cargo test -p ph2d-flip -p ph2d-flip-fill -p ph2d-flip-render -p ph2d-tool-flip \
           -p ph2d-panel-flip -p ph2d-panel-flip-frames -p ph2d-ui-testkit \
           -p ph2d-editor-core -p ph2d-host-desktop && \
cargo test -p ph2d-flip-render --test gpu_render -- --ignored && \
cargo clippy -p <suas-crates> --all-targets && \
rustup run 1.95 cargo fmt -p <suas-crates> && typos && \
cargo build --release -p ph2d-host-desktop
```
(Arch-gates que **vão** te pegar: LOC 700/crate e 600/shell — **split em módulo irmão, nunca
allowlist**, e rode `fmt` ANTES de medir; `node_id_collisions`; `architecture_panel_wiring_parity`;
`no_tofu_glyphs` — zero `→` em string literal, inclusive dentro de `assert!`/`expect()`.)

**Smoke do Enio** (entregue SEMPRE com o `cd` junto, e um roteiro numerado):
```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-FLIP && PH2D_FLIP_DEMO=1 cargo run --release -p ph2d-host-desktop
```

**Referência do Blender** (GPL — **comportamento, nunca código**):
`~/Downloads/blender-5.2-grease-pencil-ref/` (per-máquina, gitignorada; script de re-fetch no
[`Flip/00_README.md`](Flip/00_README.md)). Para a W5: `sculpt_paint/grease_pencil/sculpt_*.cc`.

**Docs do módulo:** [`docs/Flip/`](Flip/00_README.md) — `01_plano_waves` (a fila oficial + os
não-objetivos: **não re-litigue**) · `02_referencia` (o porte, §7 = a W5) · `03_traco_rasterizacao` ·
`04_alem_do_blender` · `05_frames_ghost_tween` · `06_fill_balde` · **`BUGS_flip.md`**.
Tracker exaustivo: [`HANDOFF_flip_impl.md`](HANDOFF_flip_impl.md).
Histórico (a saga da âncora, com o modo-de-trabalho): [`HANDOFF_flip_NEXT.md`](HANDOFF_flip_NEXT.md).

---

**Você fecha a linha, escreve o handoff de integração, e PARA. Não integra. Não pusha.**

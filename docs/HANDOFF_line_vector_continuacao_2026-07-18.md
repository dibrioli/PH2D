# HANDOFF de CONTINUAÇÃO — `line/Vector` (para o próximo implementador)

**Para:** a LLM que assume a linha `line/Vector` daqui pra frente.
**De:** a sessão de 2026-07-18, que fechou o **ADR-0129 inteiro** (Envelope: Fatias 1–5 + C + D + E).
**Estado:** ✅ **a linha está RE-PREPARADA e sincronizada com a `main`.** O envelope INTEGROU. Você
começa numa árvore limpa, sem nada meio-feito para herdar — há uma **fila** (§4) e um punhado de
**armadilhas já pagas** (§3) para não re-aprender.

> **Leia primeiro, nesta ordem:** `CLAUDE.md` (o roteador — os 7 inegociáveis + §5 do Vector) ·
> `docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md` (a CADA passo). Depois, só o que a sua
> tarefa exigir: [ADR-0121](architecture/decisions/0121-vector-live-corners-authored-source-cooked-geometry.md)
> (fonte≠cozido — o pré-requisito de metade da fila) ·
> [ADR-0128](architecture/decisions/0128-vector-blend-object-live-non-destructive.md) (o Blend vivo) ·
> [ADR-0129](architecture/decisions/0129-vector-envelope-warp-one-spine-cage-as-container-entity.md)
> (o Envelope). Este handoff é o **mapa da linha + a fila**; ele NÃO duplica os ADRs.

---

## §1 — Onde a linha está (30 s)

| | |
|---|---|
| **Branch / worktree** | `line/Vector` — `Worktrees/line-Vector/` |
| **HEAD** | `389676f9` = **exatamente a `main`** (0 ahead / 0 behind, árvore limpa) |
| **Modo** | **Modo L** (workstation, worktree próprio, [ADR-0106](architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md)) |
| **Gates verificados nesta árvore COMBINADA** | `check --workspace --all-targets` ✅ · `ph2d-vec-envelope` 42+8 ✅ · seam do painel 25 ✅ · shell bins **776** ✅ · arch-gates da `editor-core` e do shell ✅ |

**Como a linha foi re-preparada** (não refaça): o integrador **rebaseou** a linha e a fundiu por
ordem do Enio (as SHAs mudaram — `b32a46a9` → `8864e4b6`); depois o worktree levou
`git merge --ff-only main`. Vieram junto **33 commits de outras linhas** (GPU/ADR-0130, sculpt,
impasto, FLIP) — **e os gates do envelope foram re-rodados na árvore combinada**, não só na de
ontem: um merge textualmente limpo pode estar semanticamente quebrado, e é o `check --workspace` que
cruza isso.

---

## §2 — O que JÁ ESTÁ na `main` (não reconstrua)

O **ADR-0129 fechou inteiro**. O envelope é um **container** (entidade sem path) cujos filhos têm a
geometria re-cozida por frame, com **três gestos**:

| Gesto | Mapa | Alças |
|---|---|---|
| **Perspective** (default) | homografia dos 4 cantos | 4 cantos |
| **Mesh** | patch de **Coons** das 4 curvas de bordo | 4 cantos + 8 controles de lado |
| **Pins** | **MLS-rigid** (Schaefer 2006) — o *puppet warp* | os pinos (Alt+clique remove) |

Mais: **7 presets** geradores de gaiola (Arc/Arc Upper/Arc Lower/Bulge/Flag/Wave/Squeeze) + slider
**Bend**, promovíveis (arrastar uma alça solta o preset) · **Expand**/**Release** (a MESMA função,
`Keep`) · a seção **Envelope** no painel vetorial · overlay próprio (gaiola curva com hastes, pinos
com haste de deslocamento).

**Onde mora:** motor puro em `crates/ph2d-vec-envelope` (`quad.rs` · `coons.rs` · `preset.rs` ·
`mls.rs` · `gesture.rs` · `fit.rs`), host em `shells/desktop/src/envelope_{live,gesture,smoke}.rs`,
componente em `ph2d-ecs::VecEnvelope`, desenho em `ph2d-vec-render::envelope`.

**Detalhe de desenho e as razões** → o handoff de integração
[`HANDOFF_line_vector_envelope_integracao_2026-07-18.md`](HANDOFF_line_vector_envelope_integracao_2026-07-18.md)
§8 (Coons) · §9 (presets) · §10 (MLS) · §11 (os fixes pós-smoke). **Ele está INTEGRADO** — leia-o
como referência, não como pendência.

---

## §3 — Armadilhas já pagas (não as re-aprenda)

Estas custaram tempo real nesta linha. Estão aqui porque **nenhuma é específica do envelope**.

1. **Identidade de overlay por bits de entidade é frágil por construção.** O undo **respawna** tudo
   com ids novos. O sintoma é traiçoeiro: **a ferramenta funcionando e invisível** (o recook varre
   por QUERY e segue deformando; o overlay é desenhado pela seleção, que morreu). Derive a
   identidade de algo estável a cada frame (`VecPathId`, `Name`) ou detecte o respawn.

2. **Um gate de unidade pode ficar verde sobre um caminho que o produto não percorre.** O 1º fix do
   undo consertou o `sync_selection` — que o undo nunca alcança, porque quem zera a seleção é o
   `apply_project`, e este **exige `gfx`** (janela + GPU), fora do alcance headless. A saída: extrair
   a POLÍTICA para uma função pura **+ um arch-gate sobre o FONTE** provando que o produto a chama.

3. **Geometria derivada não é editável à mão.** O `recook` reescreve os filhos todo frame; oferecer
   as âncoras ao pen dá um ponto que **anda e volta**. Mesma regra da alça de raio numa Live Shape
   (ADR-0121). Quem quer os pontos usa **Expand**.

4. **Um guard deve perguntar pela geometria que EXISTE, não pela caixa em volta dela.** O guard de
   dobra amostrava a bbox-união (que tem cantos vazios) e recusava arrastos legítimos a partir de
   0,70 num domínio de 2,80.

5. **Uma constante "segura" pode sê-lo só para a tabela de hoje.** O `AMP` dos presets (0,35)
   garantia não-dobra para as formas que eu já tinha escrito e falhava com os quatro lados juntos —
   um caso que nenhum preset atual produz. Medido, baixado para 0,30: a garantia passou a valer para
   a linha que alguém acrescentar amanhã.

6. **Um diagnóstico pode MENTIR.** O `PH2D_VEC_OVERLAY_DIAG` nasceu recebendo `undo.depth()` como
   contador de frames e emudecia na primeira ação desfazível — o smoke concluiu "não reproduziu" sem
   nunca ter observado. Contador interno > parâmetro que se pode passar errado.

7. **Mutações que sobrevivem = gate faltando, três vezes nesta linha.** Hit-test do pino pela posição
   de repouso (o fixture nunca arrastava antes de agarrar) · `all` vs `any` na detecção de respawn
   (delete parcial pede a resposta OPOSTA) · caixa vs arte no guard (o fixture não continha o
   regime em que as duas divergem — os números tiveram de ser MEDIDOS).

**A ferramenta que sobrou:** `PH2D_VEC_OVERLAY_DIAG=1` imprime, do frame real, a caixa das âncoras
de cada forma, o **alcance das alças** relativo a ela, o estado do envelope, e **grita as recusas dos
guards** — que é o que distingue *"o guard recusou"* de *"o frame engasgou"*, dois sentidos muito
diferentes de "travou".

---

## §4 — A FILA (a ordem é do Enio)

O ADR-0129 acabou. O que resta é a **4.B herdada** — e ela tem um item que é multiplicador e o resto
que são features.

### 4.1 — ~~**Live Path Effects como NÓS**~~ — **DECIDIDO E CONSTRUÍDO** ([ADR-0132](architecture/decisions/0132-vector-live-path-effects-are-a-per-path-stack-not-a-node-graph.md), 2026-07-18)

> **A resposta foi: pilha por-path no `cooked()`, NÃO um grafo de nós.** O contrato congelado foi
> medido e **não bloqueia** (`CookValue::Opaque` + `Domain::Vector` + `input_any`/`emit_any` já
> carregam geometria em aresta; param não-`f32` tem o canal de TEXT PARAM e o discriminante `f32`) —
> a escolha era livre, e por isso teve de se defender pelo desenho. Leia o ADR antes de reabrir.
>
> **Feito:** a pilha (`VecPath.effects` + `effect::run_stack`, avaliada no `cooked()` logo depois do
> estágio da quina) · o motor de arco (`arclen.rs`) · o **Trim Path** (`fx_trim.rs`) · a cena
> `PH2D_BUILD_SMOKE=13`. **Aberto:** a seção *Effects* no painel (§7 abaixo).

O texto abaixo é o BRIEFING ORIGINAL, mantido como histórico do que se sabia antes da decisão.

O item #1 da pesquisa `docs/Vector Module/20_*`. **O pré-requisito já existe e está pago:** a costura
**fonte ≠ cozido** do ADR-0121 (`VecPath::cooked()` com `Cow::Borrowed` quando não há efeito — mesmo
ponteiro, custo zero) é exatamente o que permite empilhar efeitos sem mudar o comportamento de hoje.

O desenho que os três objetos vivos desta linha já convergiram (Live Corners · Blend · Envelope) é o
do Inkscape: **`Piecewise → Piecewise`, função pura geometria→geometria**, e é *por isso* que uma
pilha compõe. Duas cláusulas inegociáveis, escritas no ADR-0129 §3:
- **`Cow::Borrowed` sobrevive** — pilha vazia = mesmo ponteiro, zero alocação;
- **as alças vivem no espaço da FONTE** (o knotholder do Inkscape é *"totally unaffected by the
  visible distorted path"*); numa pilha aninhada, a de dentro **é** deformada pela de fora.

A pesquisa está em [`docs/Vector Module/20_pesquisa_ferramentas_de_artista.md`](Vector%20Module/20_pesquisa_ferramentas_de_artista.md)
(o Inkscape tem ~50 LPEs; o doc mapeia quais valem e por quê).

> ⚠️ **DECIDA ISTO ANTES DE ESCREVER CÓDIGO: "como NÓS" encosta num contrato CONGELADO?**
> O contrato de nós é congelado (CLAUDE.md §6 / [ADR-0039](architecture/decisions/0039-nodegraph-contract-freeze-w2t4.md)):
> `NodeOp=2` · `OpResolver=1` · `NodeManifest=8`, gate `architecture_contract_surface`. Um **nó novo**
> é drop-crate e não encosta em nada. Mas um **param que não seja `f32`** (uma curva, um enum de
> efeito, um path) encostaria — e aí é **PARE e reporte ao Enio + ADR**, não contorne.
>
> **O precedente já existe e evita a parada:** a linha `line/motion-value` resolveu exatamente isso
> com o **canal de TEXT PARAM** (`Graph::set_text_param` + `EvalCtx::text_param`) — os params vivem
> no `Graph`, **não** no `NodeManifest`, e por isso a `motion.expression` nasceu sem tocar o
> contrato. CLAUDE.md chama isso de *"o padrão canônico para param não-f32"*. Leia-o antes de propor
> bumpar o manifest.
>
> E há a pergunta anterior a essa: **LPE precisa mesmo ser nó?** Os três objetos vivos desta linha
> (Live Corners, Blend, Envelope) são **componentes ECS** com recook por frame, não nós — e é o
> desenho que funcionou três vezes. "Como nós" é a palavra da pesquisa, não uma decisão tomada.

### 4.2 — ~~Morph vivo (`t` animável)~~ — **JÁ ESTÁ FEITO** (corrigido 2026-07-18)

⚠️ **Esta entrada estava MENTINDO.** O morph vivo landou em `244e546e` (2026-07-16) e está na
`main` — a própria mensagem do commit diz *"fila #1"*. `shells/desktop/src/morph_live.rs` +
`ph2d_ecs::VecMorph` + a cena `PH2D_BUILD_SMOKE=10`.

Uma lista de pendências velha não é ruído: ela faz a próxima LLM propor construir o que existe — e
quase fez. (É a mesma lição que o módulo de áudio pagou; ver CLAUDE.md §5, *"Esta lista estava
MENTINDO"*.) **Varrido junto:** chamfer, texto em caminho, repeater e largura variável estão
genuinamente abertos; o `trim_path` que existe em `marker.rs` é **recuo de marcador** (poligonal das
âncoras, devolve o fechado intocado), não o efeito — o nome colide e o comportamento não.

### 4.3 — Blend em CADEIA (>2 formas)

Hoje o Blend Object interpola 2..=5 fontes mas o encadeamento (A→B→C com correspondência
propagada) não existe.

### 4.4 — O resto, sem ordem forte

Tipos de quina (**chamfer é quase de graça** — reta em vez de arco) · texto em caminho · trim path ·
repeater · largura variável · mais primitivas.

### 4.5 — Deferido para o FIM de tudo

**Rig + skinning** (LBS, port do runtime MIT do Rive) — só depois do módulo de desenho completo.

---

## §5 — Aberto no ENVELOPE, de propósito (não são bugs)

- **Um artefato visual sem causa.** O Enio fotografou uma linha reta longa saindo de uma forma
  envolvida; **não reproduziu** depois, e **4 hipóteses foram eliminadas por medição** (§11.4 do
  handoff de integração — leia antes de formular a quinta). A do log real: `alcance_das_alcas=1.00x`
  em toda forma, em todo frame ⇒ **não é ponto de controle disparado**.
- **`accuracy` (Fidelity) não tem knob** — é relativo (0,1% da diagonal). Quando houver queixa de
  "a curva perdeu detalhe", o lugar dele já está na seção.
- **Envelope aninhado**: o `container_of` já sobe a cadeia, mas envolver um envelope hoje envolve os
  **filhos** dele (o `create` resolve PATHS, e um container não tem path). É decisão de modelo.
- **A gaiola só é editável no Node** — ADR-0129 §3.3, cerca de Chesterton (um gizmo sobre geometria
  que se move DOBRA; 5 tentativas revertidas no Blend).
- ⚠️ **O contra-sinal do MLS continua de pé** (ADR-0129 §4): se o gesto de pinos for usado para
  **posar personagem** (membro perto do tronco), o MLS **vai** falhar e nenhum parâmetro salva — a
  decisão é **reaberta** (ARAP/LBS), não calibrada.

---

## §6 — Como trabalhar (o resumo operacional)

- **Inner loop:** `cargo check -p <crate>`. Teste/clippy/auditoria **1× no fechamento**, nunca por
  task (CLAUDE.md §2).
- **Foundational você PODE tocar** (Modo L, ADR-0107) — projete para isolamento (módulo irmão, bloco
  append-only). **PARE e reporte** só em 2 casos: contrato congelado (§6 do CLAUDE.md) ou rebase
  conflitando fora dos seus arquivos.
- ⚠️ **Rode `cargo test -p ph2d-host-desktop --tests`** ao fechar: os arch-gates do **shell** (LOC
  HR-18 incluído) **não** rodam com `cargo test -p ph2d-editor-core`. Esta linha pagou esse pedágio
  duas vezes.
- **Você fecha, escreve o handoff (DIRETRIZ §1.5.9) e PARA.** Integração e ship **só por ordem
  explícita do Enio** (§0.7).

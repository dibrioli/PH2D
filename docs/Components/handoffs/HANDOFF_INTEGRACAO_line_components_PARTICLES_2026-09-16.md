# HANDOFF DE INTEGRAÇÃO — `line/components` · TOP-20 **#18 `ParticleEmitter`**

**Data:** 2026-09-16 · **Worktree:** `Worktrees/line-components` · **Merge-base:** `main`
**Plano:** [`docs/Components/14_plano_particle_emitter.md`](../14_plano_particle_emitter.md)
**Oráculo:** Godot 4.7.2 (MIT), `CPUParticles2D`, corrido **sem interface**
(`docs/Components/ferramentas/godot_particles_probe.gd`)

⛔ **Esta linha NÃO integrou e NÃO pushou** (§0.7). Ela fecha, entrega isto e espera ordem do Enio.

---

## §1 — O que o artista consegue fazer agora

Pôr **Particles** num objecto qualquer (*Add Component → Rendering → Particles*) e ter fogo, fumo,
faíscas, poeira ou uma explosão **sem grafo nenhum**: o painel tem 19 números, quatro formas de
nascimento, duas cores, o espaço (`World` = fica rasto · `Local` = anda com o objecto) e **quatro
nomes de sinal** — ligar, desligar, recomeçar e *gritar quando acabou*.

Duas cenas prontas: **`PH2D_PARTICLES_SMOKE=1`** (quatro fontes, um knob de diferença cada) e
**`=2`** (o rasto e a tocha, o mesmo voo com espaços diferentes).

---

## §2 — Contadores, como **DELTA contra o `main`** (⛔ nunca o literal)

| contador | delta | onde |
|---|---|---|
| `PROJECT_SCHEMA` | **+1** (a linha inteira leva `128 → 135`) | `shells/desktop/src/project_schema.rs` + a escada + a tripla |
| registo do `ph2d-ecs` | **+1** | o `ParticleEmitter` |
| espelhos (`ph2d-render` · `ph2d-script`) | **+1** cada | eles contam `ecs + …` |
| `LIVE_SECTIONS` | **+1** (array `27 → 28`) | `ph2d-editor-core/src/ids/live_sections.rs` |
| `SignalOrigin` | **+1** (`Particles`) | `ph2d-runtime` — append-only |
| `EditorAction` | **+1** (`InspectorParticlesEdit`) | `ph2d-editor-core/src/action_bus.rs` |
| catálogo de componentes | **+1 família** (`particles.rs`, `Rendering`, `O::ANY`) | `ph2d-component-desc` |

⛔ **Contrato congelado: NÃO.** `NodeManifest`/`NodeOp`/`Tool` intocados — um param a mais na lista
do `motion.emitter` é **dado**, não contrato (o `emitter_motion` entrou assim em 2026-08).

**Tecto da shell:** o corpo vive em `ph2d-particles` (lei + compilador) e `ph2d-app-components`
(ponte, Inspector, cena); a shell só compõe. **Crates novas:** `ph2d-particles`.

---

## §3 — As waves, e o que cada uma prova

| W | o que shipa | a prova |
|---|---|---|
| **W0/W0b** | a **AGENDA** (`emit_mode = Scheduled`) no `motion.emitter` | 13 gates — enumeração por força bruta (5 agendas × 3 taxas × 260 instantes), identidade **ao bit** com o contínuo, recusa de GPU |
| **W1** | `ph2d-particles`: o **compilador** (config → grafo de nós REAIS) e o **relógio** | a tabela do oráculo reproduzida com a janela `[0, 1 quadro]` |
| **W2** | a **ponte**: corre com o Play, desenha sempre, `World`/`Local`, sinais, renascer | 6 gates |
| **W3** | a **secção do Inspector** | 6 gates de costura com **cliques reais** (`MockPanelHost`) + 8 do dreno |
| **W4** | as **duas cenas**, fotografadas | 7 gates de cena |

**25 provas de mutação, todas a sangrar** (`docs/Components/ferramentas/mutacao_particles_w3.sh`).
Portão de fecho: `nextest-impacted` **15 475 testes**, clippy `-D warnings` a zero, `fmt` limpo.

---

## §4 — ⛔⛔ O que a MEDIÇÃO derrubou (leia antes de tocar no desenho)

1. **O levantamento dizia que o emissor «não sabe parar»** — e a cura não foi um campo: foi a
   **AGENDA**, uma lei pura no nó (`τ(t)` = tempo LIGADO acumulado; a partícula `k` nasce quando `τ`
   passa `k/rate`). ⭐ Como `τ` é monótona, as ids vivas continuam **um intervalo contíguo** — a
   forma que a janela do emissor já assumia, logo **zero** mudanças a jusante.
2. ⛔ **«Um período global basta»** — a 1.ª agenda (`a-b every P`) não sabia **cortar** um padrão
   periódico num segmento finito. ⇒ segmentos **mais pulso por segmento**. Achado antes de alguém
   depender da forma velha.
3. ⛔ **«O `clamp` do campo inteiro é a cerca»** — **a mutação que o apagou não matou gate nenhum**:
   um `as` de vírgula flutuante para inteiro **satura** em Rust desde a 1.45. *Uma cerca que repete
   o que a linguagem já garante lê-se como a cerca que falta.* O que não é de graça é o `round`.
4. ⛔⛔ **«Caber no ecrã é a legibilidade»** — **falso, e foi uma MUTAÇÃO SOBREVIVENTE que o disse**:
   devolver ao anel a rapidez das outras deixava o gate do enquadramento VERDE e punha os quatro
   penachos uns por cima dos outros. ⇒ `cada_coluna_fica_na_coluna_dela`.
5. ⛔ **«A régua do enquadramento é o alcance»** — de lado uma partícula anda o **seno da abertura**,
   e acima dos `90°` o seno **desce** (a `180°` dá zero, que leria uma esfera como um fio).

---

## §5 — ⚠️ O que uma leitura rápida do diff entende ao contrário

1. **O emissor é STATELESS por construção** — o conjunto de vivas é função pura do playhead, e é
   isso que torna o scrub e o replay de graça. O que guarda estado é o `motion.integrate`, por id.
2. **A ponte NÃO escreve no mundo** — as partículas não são entidades, não têm `Transform` e por
   isso **não passam pelo ledger do `preview_drive`**. Daqui só saem instâncias para desenhar e um
   sinal quando a emissão acaba.
3. **Um componente EDITADO faz o emissor RENASCER** (o grafo é outro, e o estado do integrador é do
   grafo velho) — é por isso que o dreno do Inspector devolve *«mudou?»* em vez de escrever e calar:
   um componente tocado por nada apagaria o penacho a cada quadro em que o rato passa num campo.
4. **As partículas entram na fatia `extra` do passo de sprites SEMPRE** — ao contrário do stream do
   Motion, que exige a ferramenta MOTION na mão.
5. **`rewind` é RENASCER** e não «repor»: as corridas são deitadas fora. Registar o estado vivo
   poria cada tique na pilha de `Ctrl+Z` (a lei do `TimerRuntime`).
6. **O `alive_of` é por OBJECTO, não o total** — a pergunta que o painel responde é *«este emissor
   está a fazer alguma coisa?»*, e um total tornaria a resposta dependente do vizinho.
7. **`escreve_texto` mudou de casa e isso não é arrumação:** ela vivia em **duas cópias** (cérebro e
   script) e a terceira secção com campos de texto ia escrever a terceira. Hoje é uma porta
   (`sync_text_field`), e o que se perdia na cópia era a **cerca do foco** — invisível até alguém
   estar a digitar.
8. **`paint_optional_top20.rs` e `fase_snapshots_tardios.rs` não são refactor:** são **cortes
   impostos** por dois tectos (função de painel `214/200`, função de shell `208/200`), cada um com
   uma responsabilidade que fecha sozinha.

---

## §6 — ⛔ Os defeitos que só o SMOKE e a FOTO apanharam

**Dois de produto**, achados pelos gates da ponte (W2): um emissor **PARADO** mostrava já uma
partícula (o nascimento cozinhava em `t = 0`); e um `restart_on` num emissor autorado
`emitting = false` **nunca emitia** — *um sinal de recomeço LIGA*.

**Um de fiação, mudo**, achado ao escrever o gate de costura: as duas **amostras de cor** estavam
pintadas e hit-registadas e **ausentes do `populate`** ⇒ o clique morria no `is_focusable` e o
selector nunca abria. ⚠️ E a metade de volta também faltava: sem o braço `picker == Some(id)` da
semente, a cor escolhida chegava ao painel e **morria ali**.

**Seis de cena**, achados pela FOTO (`fotografa_cena.sh`), com a suíte verde por cima de todos:
a fila a `±7,5 m` (duas colunas fora do ecrã) · as fontes a `−5 m`, cortadas pela borda de baixo ·
o jacto a `0,52 m` de alto com a gravidade do mundo, colado à fonte · o anel borrado num disco (a
qualquer rapidez ele deixa de se ver como anel) · a `=2` a virar um chão **vazio** passados três
segundos · e o painel da direita a ser o do **esqueleto** numa corrida e o do **vector** noutra.

⛔⛔ **E o último tem uma cauda que interessa a toda a gente:** a arrumação vive **fora do
repositório** (`~/.ph2d/layout.txt`) e estava a ser reescrita por **outra árvore a correr em
paralelo** — logo o painel da frente não é uma propriedade do código. ⚠️ A 1.ª cura não chegou: o
`reconcile_z` acrescenta, no **início de cada quadro**, os painéis que ainda não estão na ordem z,
logo um `bump` feito no quadro em que a cena monta fica **por baixo** dos que chegam a seguir.

---

## §7 — ⏳ O que fica ABERTO, com o mecanismo

1. **O caminho de GPU** — hoje o emissor cozinha na CPU. O bloqueador tem nome: o renderer recebe
   **um** buffer de GPU, e o `gpu-cook` mais o `renderer_draw` estão a ser reescritos por duas
   outras linhas vivas. ⛔ Abrir isto agora seria escrever contra código que vai mudar.
2. **«Abrir como grafo»** — o compilador produz um grafo REAL, logo a fachada podia oferecer
   *«converter em nós»*. Falta a porta de autoria (onde é que esse grafo passa a viver?), que é
   decisão de produto.
3. **O tecto de partículas vivas não foi medido neste caminho** — o `amount` é o tecto por emissor e
   o nó tem o dele; quantos emissores cabem num quadro **não** foi varrido. ⚠️ §0.0: quem escrever
   um número aqui mede-o primeiro.
4. **A secção fica no FIM do painel** — com uma sprite no objecto, o dono tem de rolar até lá. É a
   mesma posição de todas as secções da fila do TOP-20, e mudá-la é decisão de produto sobre a
   ordem das secções, não desta wave.
5. **Ao rolar o Inspector, o cabeçalho do painel desenha por cima do conteúdo** — visto na foto com
   um `set_panel_scroll` forçado (uma medição descartável, revertida). ⚠️ **Não investigado**: pode
   ser artefacto de eu ter forçado um valor fora da faixa que o clamp admite.

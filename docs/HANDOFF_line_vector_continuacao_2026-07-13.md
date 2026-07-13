# HANDOFF — `line/Vector`, continuação (2026-07-13)

> Do agente que fechou a wave anterior, para **você**, o próximo.
> A wave passada foi **INTEGRADA no main** e **pushada**. Você começa do zero de trabalho,
> mas não de contexto — e este doc é o contexto.
>
> **Leia inteiro antes de tocar em qualquer arquivo.** As seções §1 e §2 são operacionais
> (o que fazer nos primeiros 5 minutos). A §6 é a que te impede de repetir os meus erros.

---

## §1 — PRIMEIRA COISA: prepare a linha (a worktree está DESATUALIZADA)

**O estado exato, verificado:**

| | |
|---|---|
| Worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector` |
| Branch | `line/Vector` |
| Trabalho não-commitado | **nenhum** |
| Commits da linha que **não** estão no main | **nenhum** — tudo foi integrado |
| Commits do main que **não** estão na linha | **22** (a linha `line/FLIP` entrou junto, + um fix de ship) |

O integrador **rebaseou** a linha ao fundir (os hashes mudaram: o meu `7120580b` virou
`be7d7cd8` no main). Então a sua worktree está **atrás**, e trabalhar nela sem sincronizar te
faria construir sobre um main velho.

### Faça isto, exatamente:

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && git fetch origin && git rebase main
```

> **`rebase`, NÃO `reset --hard main`.** Um `reset --hard` apagaria o commit deste próprio
> handoff (ele está na ponta da linha, ainda fora do main). O `rebase` o replaya em cima do main
> novo, e ele sobrevive para a próxima integração.

Depois confirme que a base está sã (~2 min no `workstation`):

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && cargo nextest run --workspace --no-fail-fast
```

Esperado: **verde**. Se estiver vermelho, é regressão de outra linha e você **PARA e reporta ao
Enio** — não é sua para consertar (§2, escopo).

> **Isto não é hipotético — aconteceu comigo, escrevendo este handoff.** Rodei `typos` na
> worktree e vi **6 erros** numa crate que não é minha (`ph2d-node-sim-lifetime`). Parecia
> regressão herdada. **Não era:** o main já havia corrigido (o commit
> `fix(ship): os 3 latentes que só o ship vê`), e eu estava olhando para a versão velha do
> arquivo. **Um gate vermelho numa worktree desatualizada não é um bug — é a desatualização.**
> Sincronize **antes** de acreditar em qualquer coisa que o gate te disser.

> ⚠ **Não rode cargo no repo PRIMÁRIO** (`/home/enio/Documentos/Projetos/PH2D`). O `target/` de
> lá é um symlink para `/dev/shm` que some no reboot, e você vai levar um
> `failed to create directory` que **não é um erro de código**. Além disso: o primário é
> proibido (§2). Todo comando seu começa com `cd` **absoluto** na worktree.

---

## §2 — Como se trabalha no Modo L (leia o guia, mas estes são os inegociáveis)

Guia completo do Enio: [`GUIA_JORNADA_MODO_L.md`](IntegracaoMultiAgente/GUIA_JORNADA_MODO_L.md).
Protocolo que **você** segue: [`DIRETRIZ.md §1.5`](IntegracaoMultiAgente/DIRETRIZ.md).
O porquê: [ADR-0106](architecture/decisions/0106-parallel-dev-lines-worktrees-workstation.md)
(linhas) e [ADR-0107](architecture/decisions/0107-concurrent-foundational-lines-tested-gate-syntactic-merge.md)
(foundational concorrente).

**O que isso significa na prática, para você:**

1. **Você é autônomo.** Não há Coordenador. Você trabalha na sua worktree, commita local
   (`git commit --no-verify`, fast mode), e **não pede permissão** para tocar foundational.
2. **Foundational você PODE e DEVE tocar** (ADR-0107) — `ph2d-ecs`, `ph2d-editor-core`,
   `ph2d-i18n`. A wave passada tocou os três. **Mas projete para isolamento:** módulo irmão,
   ponto de extensão append-only. A foundation é isolada de propósito, para várias linhas a
   estenderem sem colidir.
3. **Você NÃO integra e NÃO faz ship.** Nunca. Nem se parecer óbvio, nem se o Enio elogiar o
   trabalho. Você **fecha, escreve o handoff de integração (DIRETRIZ §1.5.9) e PARA.**
   Integração e ship são **ordem explícita do Enio**, executadas por um agente integrador
   dedicado. Fazer por conta própria é **violação do protocolo**.
4. **`git push` nunca é seu.** Idem.
5. **PARE e reporte ao Enio em exatamente 2 casos** (e só nesses):
   - **Contrato congelado** (CLAUDE.md §6: `Tool`/`RasterEditTool`/`PanelEvent` ou
     `NodeOp`/`OpResolver`/`NodeManifest`) — exige ADR, e é decisão dele.
   - **Rebase conflitando FORA dos seus arquivos** (colisão de mesmo-símbolo com outra linha) —
     é design, não merge.

   Fora esses dois, **resolva você**.

6. **Git:** a worktree isola o índice, então colisão de commit não existe. Mas **nunca**
   `git push`, **nunca** `--force`, e **nunca** toque o repo primário.

**Regra-mãe do projeto, que vale mais que tudo acima:**
> **Verde-de-compilação é velocidade; no audit vale ZERO.**
> Leia [`DIRETIVA_IMPLEMENTACAO.md`](IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md) a cada
> passo de implementação. É o antídoto das 4 causas da semana perdida no Painter.

---

## §3 — O que existe hoje no módulo (o mapa)

O Vector é um dos módulos mais completos do PH2D agora. **Antes de construir, procure** — há uma
chance real de já existir.

### Crates

| Crate | O que é |
|---|---|
| `ph2d-vec-scene` | **O modelo de documento.** Path = âncora + 2 handles (estilo Rive). É onde mora TODA a geometria: catálogo (`kind.rs`), formas (`shapes/arrows/flow/symbols/bubbles/iso`), cantos (`corners.rs`, `smooth.rs`), pontas de traço (`marker.rs`), filete de polilinha (`polyline.rs`), hit-test (`inside.rs`, `boundary.rs`) |
| `ph2d-vec-edit` | `PenTool` (desenho + edição unificados), `ShapeTool`, `History` |
| `ph2d-vec-render` | `VecPath` → Vello |
| `ph2d-vec-boolean` | Booleana em tempo de edição (linesweeper + kurbo) |
| `ph2d-vec-connect` | **O roteador de conectores.** A\* sobre grafo de visibilidade ortogonal (Wybrow). **Puro** — sem ECS, sem kurbo, sem documento |
| `ph2d-tool-vector` | A tool (3 modos: Select / Node / Pen) + o catálogo de apresentação |
| `ph2d-panel-vector` | O painel docado |

### O que o módulo faz hoje

- **47 formas paramétricas VIVAS** — editáveis por parâmetros *depois* de desenhadas.
- **Conectores de diagrama completos** — roteador, desvio de obstáculo, rotas reta/ortogonal/
  curva, waypoints manuais, alças de ponta, pontas de seta, painel por-rota.
- **Rótulos ancorados** — texto que pertence à forma ou ao conector e os segue.
- **Raio por-canto + squircle G2**, booleana, offset, gradientes (inclusive multi-ponto IDW),
  picker OKLCH, texto com fontes variáveis, align/distribute, snap, undo, save.

### Documentação viva (leia antes de mexer no que ela cobre)

- [`docs/Vector Module/BUGS_vector.md`](Vector%20Module/BUGS_vector.md) — **7 bugs**, cada um com
  sintoma / causa **real** / correção / gate. E os **padrões que se repetem**. Este é o doc mais
  útil do módulo: quase todo bug novo é uma variação de um que já está lá.
- [`docs/Vector Module/20_pesquisa_ferramentas_de_artista.md`](Vector%20Module/20_pesquisa_ferramentas_de_artista.md)
  — a pesquisa do que **falta** (§5 abaixo é o resumo).
- ADR-0108 (reposicionamento), ADR-0110 (paths são entidades ECS),
  ADR-0111 (geometria local + gizmo de sprite), ADR-0112 (Select/Node/Pen são 3 modos).

---

## §4 — Invariantes que você vai quebrar se não souber

Estas não são preferências. São **decisões com gate**, e quebrá-las deixa a suíte vermelha (na
melhor das hipóteses) ou produz um bug invisível (na pior).

### 4.1 O espaço de autoria (`space.rs`) — **a fronteira do eixo Y**

O mundo do PH2D é **Y-para-CIMA**. Toda referência de geometria (SVG, OOXML, stencils) é
**Y-para-BAIXO**. Escrever uma fórmula "como na referência" direto em coordenadas de mundo é um
**espelhamento vertical silencioso** — e foi assim que ~20 formas nasceram de cabeça para baixo.

**Forma nova se autora em `(u, v) ∈ [0,1]²` com `v = 0` no TOPO.** O `Unit::p` é o **único**
lugar do catálogo que sabe para onde o mundo aponta. Não invente uma segunda travessia.

### 4.2 A ordem do frame é CARGA

```
vec_entities::sync
  → connector_live::upkeep        (pendura o VecConnector)
  → label_live::upkeep_pending    (pendura o VecLabel)
  → vec_transform::settle_origins (PULA conectores e Live Shapes)
  → vec_transform::build
  → connector_live::recook        (a rota — e quem monta as PAREDES do roteador)
  → label_live::upkeep            (a pose do rótulo)
  → render
```

**`settle_origins` só pula o que ENXERGA.** Reordene isso e três gates ficam vermelhos — que é
exatamente para isso que eles existem.

### 4.3 O contrato congelado (CLAUDE.md §6)

`Tool = 12` · `RasterEditTool = 5` · `CanvasPaintTool = 1` · `PanelEvent = 4`. **Não toque.**
Precisou? **PARE e reporte ao Enio** (exige ADR).

O motor novo (`ph2d-vec-*`) tem contrato **próprio, ainda NÃO congelado** — e cresceu muito.
Re-congelá-lo é follow-up pendente.

### 4.4 Um componente ECS novo tem **três** lugares

Registrar em `ph2d-ecs/src/scene/registry.rs` **e** ajustar as asserções **gêmeas** em
`ph2d-render/src/registry.rs` e `ph2d-script/src/registry.rs`.

E o motivo de o registro existir: **um componente que não passa pelo `ComponentRegistry` é
silenciosamente DESCARTADO pelo snapshot** — undo e save o perdem, **sem erro nenhum**.

### 4.5 Um widget pintado NÃO é um widget vivo

Um campo de painel precisa de **registro em `populate.rs`**, além de id + desenho + evento. Sem o
registro ele **pinta, aceita o arrasto, e não despacha nada** — e a suíte inteira fica verde.

Aconteceu comigo com o campo `Curve`. A cura foi matar a **classe**: `NUMBER_FIELDS` é hoje a
tabela **única** que o registro, a semente, o desenho e o gate iteram. **Adicione campo novo
ali**, e não em quatro lugares.

---

## §5 — O que fazer a seguir (a pesquisa, e a minha recomendação)

Fiz uma pesquisa de 4 frentes (Illustrator/Corel/Xara · Inkscape/Affinity/Graphite · Figma/Rive/
Cavalry/AE · papers), com fontes primárias — inclusive o **código-fonte do Inkscape** e a **fonte
do flubber**. Está em
[`20_pesquisa_ferramentas_de_artista.md`](Vector%20Module/20_pesquisa_ferramentas_de_artista.md).

**A tese, e ela reordena tudo:**

> Os ~50 **Live Path Effects** do Inkscape (efeitos não-destrutivos e **empilháveis** sobre um
> caminho) são a coisa mais poderosa que um editor vetorial livre já construiu. **E a arquitetura
> deles é um sistema de nós.** Os *path operators* do After Effects e os *Behaviours* do Cavalry
> são a mesma ideia. **Nós já temos `ph2d-nodegraph`.**
>
> Logo: **a ferramenta mais transformadora não é uma ferramenta — é a espinha que faz cada
> ferramenta futura custar uma drop-crate.**

**O ranking por retorno ÷ esforço (detalhe no doc):**

| # | O quê | Esforço |
|---|---|---|
| 1 | **Live Effects como NÓS** (a espinha) | Médio — mas é o **multiplicador** |
| 2 | **Gizmo de raio na quina** | **Pequeno** — o motor (`corners.rs`) já existe, falta a alça. **É pedido do Enio** |
| 3 | **Texto em caminho** | Pequeno/Médio — `kurbo` já tem `inv_arclen` |
| 4 | **Trim Path** (revelar o traço) | Pequeno — e temos timeline |
| 5 | **Interpolação de formas (Blend)** | Médio |
| 6 | **Largura variável de traço** | Médio/Grande — o **buraco mais gritante** |

**Dois achados que mudam o cálculo:**

- **Ninguém resolveu a correspondência de formas** (o problema difícil do Blend). O flubber faz
  força bruta `O(n²)`; o GSAP tem um índice **manual** e uma ferramenta de debug que **admite que
  o automático erra**; o CorelDRAW pede ao usuário para **clicar um nó em cada forma**; Lottie e
  Rive **não têm correspondência nenhuma**. ⇒ O alvo honesto é **bom automático + escape manual**,
  e isso é **barato**.
- **Largura variável não existe de prateleira.** O `kurbo` só tem `width: f64` (a Linebender
  *declarou* que pretende explorar); o Skia só tem protótipo fora do stroker. O offset exato de
  uma cúbica é curva de **grau 10**.

**Avaliado e RECUSADO:** os *Vector Networks* do Figma. Ganho topológico real, mas a continuidade
de tangente **quebra** num vértice de 3+ arestas (nem a Figma resolveu), e o preenchimento exige
base mínima de ciclos sobre grafo expandido. O melhor artigo técnico do tema conclui que são
*"workflows novos, não expressividade nova"*.

**Se o Enio não disser o contrário, comece pelo item 2** (gizmo de raio) — é barato, é pedido
dele, e o motor já existe.

---

## §6 — As lições que me custaram caro (leia, mesmo que pareça óbvio)

Estas não são teoria. Cada uma custou pelo menos um bug que chegou ao Enio.

1. **O sintoma quase nunca é a causa.** "Cone de cabeça para baixo" era uma convenção de eixo em
   quatro módulos. "Panic no clamp" era uma janela geométrica que **colapsa** (dois limites
   derivados do mesmo cálculo se cruzam por 1 ulp). "Cubo com triângulo escuro" era semântica de
   preenchimento de contorno **aberto**.

2. **Um teste que não MORDE é pior que teste nenhum** — ele dá confiança falsa. Antes de aceitar
   um gate, **quebre de propósito o que ele protege** e confirme o vermelho. Isso me pegou **cinco
   vezes** nesta linha:
   - Um gate de orientação que **consagrava** o bug (o oráculo saiu do código, não da aparência).
   - Um gate de curva com folga de **12%** que passava com o bug vivo (apertei para 2%).
   - Um gate de conector em "L" com rota **simétrica** — onde as duas coisas que ele distinguia
     coincidiam.
   - Um gate de oscilação que media em **frames pares** (uma oscilação de período 2 é invisível
     assim).
   - Um `spread` que era **placebo**: rodava, não fazia nada, e os seis testes passavam
     `spread: 0.0`.

3. **Um parâmetro que nenhum teste exercita não está implementado — está escrito.** O sinal é
   visível na suíte: se **todos** os testes passam o mesmo valor para um campo, esse campo nunca
   foi testado. Grepar o nome do parâmetro nos testes é mais rápido que ler a implementação.

4. **Quando o smoke contradiz o gate verde, o incompleto é o HARNESS, não o relato.** A oscilação
   linha↔rótulo tinha um gate verde **enquanto o bug estava vivo no app** — porque o harness
   omitia um passe do frame. Reproduza com a **ordem de frame REAL**, não com uma aproximação.

5. **Meça, não suponha.** O roteador custa 1–6 µs no caso realista e 48 µs com 51 obstáculos.
   Eu ia otimizar sem medir; o número disse que não precisava.

6. **Um "verde" que veio do seu próprio `echo` não é verde.** Já li isso, já me enganei, e um
   agente teve que consertar. **Cole a saída real do cargo.**

7. **LOC cap = split, nunca allowlist.** Estourou 600 (shell/painel) ou 700 (crate)? Extraia
   **módulo irmão** (`#[path = "..._tests.rs"] mod tests;`). Nunca `// ph2d-loc-cap:`.

---

## §7 — Gate de fechamento (o seu, quando a wave acabar)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && \
  cargo nextest run --workspace --no-fail-fast && \
  cargo clippy --workspace --all-targets && \
  rustup run 1.95 cargo fmt --all && \
  typos
```

E então: **escreva o handoff de integração (DIRETRIZ §1.5.9) e PARE.** Não integre. Não pushe.
Reporte "linha pronta + handoff" ao Enio e espere a ordem dele.

O handoff da wave passada, como modelo do que ele espera ver:
[`HANDOFF_line_vector_integracao_2026-07-12.md`](HANDOFF_line_vector_integracao_2026-07-12.md) —
note que a seção mais valiosa dele é a **§8: o que eu NÃO provei**. Seja honesto sobre os limites
do que seus testes cobrem; o integrador depende disso.

**Aviso do protocolo:** o gate acima **não é o `ship.sh`** — ele não roda `machete`, `deny`,
`audit` nem o perfil `ci-test`. O ship **drena latentes** (a integração passada pegou três). Isso
é esperado, e **não é seu** para resolver.

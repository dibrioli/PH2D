# ADR-0107 — Linhas concorrentes em foundational: merge sintático (Mergiraf) + gate de integração testado (Modo L)

- **Status:** Accepted (Enio, 2026-07-05)
- **Contexto arquitetural:** estende [ADR-0106](0106-parallel-dev-lines-worktrees-workstation.md)
  (linhas paralelas por `git worktree` no tier `workstation`), que por sua vez estende
  [ADR-0075](0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md) (paralelismo
  por desacoplamento) e [ADR-0104](0104-hardware-tiered-speed-strategy.md). Opera junto de
  DIRETRIZ v8.x §1.5 + [`MODELO_ABERTURA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_ABERTURA_LINHA.md).

## Contexto

O Modo L (ADR-0106) deu a cada linha índice/HEAD/tree/`target/` próprios e serializou a
integração por `merge --ff-only`. Mas manteve **foundational serial por natureza**: qualquer
linha que precise de `ph2d-core`, `ph2d-editor-core`, `ph2d-tokens`, `ph2d-host`, contratos
congelados etc. tinha de **PARAR e reportar**, virando uma `line/foundational` dedicada e
única. Na prática isso reintroduz um gargalo serializado — o requisito do Enio é que **cada
agente possa tocar foundational** sem esperar uma fila humana.

A pergunta implícita — "existe um merge tão bom que N agentes editem os mesmos arquivos
foundational simultaneamente e funda sozinho e seguro?" — foi pesquisada. A resposta é **não**,
e por razão **estrutural, não de ferramenta**:

- **O conflito que sobra em foundational é semântico/de-design, não textual.** Worktree já
  resolveu o git. Resta o caso em que agente A muda a assinatura de uma função e agente B
  adiciona um chamador da versão antiga: **cada um compila sozinho, juntos quebram**, e nenhum
  diff textual "vê" isso. Pior: dois agentes tomam decisões de design incompatíveis sobre a
  mesma abstração — Cognition, *"Don't Build Multi-Agents"* (2025): *actions carry implicit
  decisions, and conflicting decisions carry bad results*. Decisão implícita conflitante não
  tem merge possível, por definição.
- **A escala é medida.** *AgenticFlict* (arXiv 2604.03551 — 142 mil PRs de agentes em 59 mil
  repos): **27,67 % dos PRs de agente colidiram**, 336 mil+ regiões de conflito. "Deixar
  agentes editarem código compartilhado livremente e fundir depois" é o modo de falha medido.
- **A conclusão acadêmica é decomposição > merge.** Geng & Neubig, *Effective Strategies for
  Asynchronous SWE Agents* (arXiv 2603.21489): merge-based coordination só se justifica
  *quando a decomposição não é viável*. É exatamente a filosofia do PH2D (drop-crate + codegen
  splice) — que precisa apenas ser aplicada **um nível mais fundo, dentro de foundational**.

O que os maiores monorepos (Google/Meta) e os pipelines de gating (Zuul, Bors, Mergify) de
fato fazem **não é merge mágico**: é (a) decompor para que o código compartilhado **não seja
editado concorrentemente**, e (b) uma **fila de integração que testa o resultado combinado** e
mantém o main verde. O PH2D já tem as duas metades em forma artesanal (drop-crate + o gate
`--ff-only`). Faltava tornar o gate **testado** e emprestar um **merge sintático** para o
resíduo textual.

## Decisão

**Foundational deixa de ser serial-por-natureza no Modo L.** Qualquer linha pode tocar
foundational, sob um modelo de **3 camadas** que empurra a fronteira até onde ela vai — e para
honestamente onde a serialização é irredutível.

### Camada 0 — pontos de extensão append-only (a de maior alavancagem)

**Camada 0 já é a arquitetura do PH2D — não algo a construir do zero.** Um levantamento de
TODA a superfície foundational (2026-07-05) mostrou que "edição compartilhada" já virou
"arquivo disjunto + codegen/união" em quase tudo:

| Superfície | Mecanismo | Estado |
|---|---|---|
| Tools / Nodes / Panels | drop-crate + `ph2d-{tool,node,panel}-sync` → registry-init | já é |
| Chrome handlers (módulos) | `ph2d-chrome-sync` (bloco `mod`) | já era |
| Widgets | `ph2d-widget-sync` | já é |
| Membership do workspace | glob `crates/*`, `tools/*` | já é |
| Wiring tool/panel → `EditorAction` | variants genéricos `ActivateTool{tool_id}`/`OneShotImageOp`/`ToolPanelEvent` | já é |
| Valores de token | `tokens.json` (fonte única) → `build.rs` codegen | já é |
| `IconId` | união alfabética + gate `enum_order_matches_svgs` (+ Mergiraf) | já é |

**Duas superfícies hand-central sobravam — ambas convertidas neste ADR:**

1. **`dispatch_all` (chrome)** — a chain `|| slug::apply()` era hand-written **por decisão
   ratificada tripla** (doc do `mod.rs`, `chrome-sync`, e o gate) porque *z-order é
   load-bearing*, e o Mergiraf **não** une um `||` (uma única expressão — verificado). Agora é
   **gerada** por `ph2d-chrome-sync`, ordenada por um marcador `// ph2d-chrome-sync:z=NN`
   auto-declarado por handler. A ordem é **função pura do conjunto de handlers** → preserva o
   z-order atual byte-a-byte (verificado) e o conflito vira "re-rode o sync" (tier registry-init,
   §1.5.5), nunca hand-merge. Gate `architecture_chrome_dispatch_in_sync` valida a staleness.
2. **`ColorToken` (tokens)** — enum + `key()` eram **dois** sites à mão; o Mergiraf une os
   variants do enum mas **não** os braços do `match key()` (mesma posição — verificado). Agora
   uma **única** lista `color_tokens! { Bg0 => "bg-0", … }` gera enum + `key()` — impossível
   desincronizar ou esquecer o key, docs curados (WCAG) preservados inline, e a colisão cai para
   um único site de união trivial (tier `icons.rs`). Enum byte-idêntico (36 variants, verificado).

**Exceção de princípio (fato técnico, não pendência):** os variants *internos* de `EditorAction`
(Hierarchy/Inspector) + o `match apply_editor_action` central **permanecem** um enum exaustivo —
o match exaustivo é uma *feature* (o compilador força o shell a tratar toda ação); registry-ificar
perderia essa checagem. Adições disjuntas ali já são unidas pelo Mergiraf (Camada 1).

Diretriz permanente para superfícies futuras: **o que os agentes crescerem em foundational nasce
como ponto de extensão append-only** (arquivo/linha-por-concern + codegen/união), salvo quando a
exaustividade do compilador for mais valiosa que a ausência de colisão.

### Camada 1 — Mergiraf como merge driver do repo (resíduo textual)

[Mergiraf](https://mergiraf.org) (merge sintático via tree-sitter, **suporta Rust**) entra como
`merge driver` do git (`.gitattributes` + `git config`, setup por `scripts/mergiraf-setup.sh`).
Faz line-merge primeiro; só cai no AST nas regiões que conflitaram. Resolve automaticamente o
caso foundational **mais comum**: dois agentes adicionam variant/método/campo em *pontos
diferentes* do mesmo arquivo. Degrada com elegância — deixa marcadores de conflito para
resolução manual quando não consegue. **Limite honesto:** não resolve dois agentes no *mesmo
item*, e **não pega quebra semântica/build** — isso é a Camada 2.

### Camada 2 — o gate de integração passa a compilar+testar a árvore COMBINADA

`merge --ff-only` prova apenas "ninguém entrou entre meu rebase e meu merge"; **não prova que a
árvore combinada compila**. Para drop-crates disjuntos tanto faz (isolamento físico garante).
Para qualquer linha que **tocou foundational**, a integração (§1.5.3) ganha, ANTES do
`--ff-only`, um `cargo check --workspace` + `nextest-impacted` **sobre a árvore rebaseada** —
via `scripts/foundational-integrate.sh`. É o "keep-main-green gate" que fecha o buraco do
"compila sozinho, quebra junto".

**Prova de correção (por que isto é seguro, não otimista):** `rebase → gate → --ff-only` é o
modelo do Zuul/Bors. Como `--ff-only` faz o tip da linha *virar* o main, testar o tip
rebaseado == testar o futuro main. Se outra linha integrar na janela entre o gate verde e o
`--ff-only`, o `--ff-only` **falha** (não é fast-forward) → a linha rebaseia sobre a
recém-integrada e **re-roda o gate**, agora incluindo a mudança dela. Logo **nenhuma linha
jamais funde uma combinação não-testada**. A serialização do `--ff-only` garante
árvore-testada == árvore-fundida.

### O que PERMANECE serial (e deve) — o núcleo irredutível

Dois casos não têm merge, por serem decisão de design com um dono:

1. **Contratos congelados** (§4: `NodeOp`/`OpResolver`/`NodeManifest`; `Tool`/`RasterEditTool`/
   `CanvasPaintTool`/`PanelEvent`) — caps + arch-gate. Continuam **Coord-only + ADR**; o
   arch-gate pega bump de cap. Uma edição que muda a *forma* do contrato é exatamente a decisão
   que precisa de um único dono.
2. **Mesmo símbolo / mesma assinatura de tipo-núcleo** — dois agentes reescrevendo o corpo da
   mesma função, ou mudando o mesmo `match`/`struct` de forma incompatível. Colisão *textual* é
   pega por Mergiraf/git no rebase; colisão *silenciosa* (compila, semântica errada) é pega pelo
   `nextest-impacted` **só se houver teste cobrindo** — o mesmo risco que qualquer equipe humana
   tem em código compartilhado. Mitigação: os arquivos-núcleo notoriamente quentes (defs de tipo
   central em `ph2d-core`, contratos) ficam **ADR-serial**; o resto de foundational é concorrente.

O modelo, então, muda de **"foundational = 1 linha serial"** para **"foundational editável por
qualquer linha, com Mergiraf + gate testado; só colisão real de contrato-congelado ou
mesmo-símbolo-de-núcleo serializa — e essa, por definição, merece ADR"**.

## Alternativas rejeitadas

- **Merge lock-free universal em foundational** (o pedido literal): não existe — o resíduo é
  semântico/design (Cognition; AgenticFlict 27,67 %). Prometê-lo seria vender fantasia.
- **Só Mergiraf, sem gate testado:** perigoso. Mergiraf une sintaticamente e pode produzir uma
  árvore que compila-mas-está-errada, ou disjunta-mas-quebrada-no-build. Mergiraf **exige** a
  Camada 2 por baixo.
- **Só gate testado, sem Mergiraf:** funciona, mas deixa o operador resolvendo à mão conflitos
  textuais triviais (dois `IconId`/variant em pontos diferentes) que o AST resolve sozinho.
- **Merge queue especulativa agora** (Zuul/Bors real — testar a cadeia otimista L1+L2+L3 em
  paralelo, bissecção na falha): é o próximo degrau *se* a fila serial testada virar gargalo com
  ≥3 linhas foundational concorrentes sustentadas. Para 3-4 integrações/jornada é over-engineering;
  o gate serial testado (Camada 2) basta. Registrado como caminho de crescimento, não adotado.
- **Jujutsu (`jj`) com conflitos de primeira classe:** atraente (conflitos armazenados, não
  bloqueantes), mas troca o VCS de todo o fluxo git-nativo + hooks + multi-máquina. Fora de
  escopo; reavaliar só se o atrito textual persistir pós-Mergiraf.
- **Manter foundational serial (status quo 0106):** é o gargalo que este ADR remove; preservado
  apenas como fallback de rollback.

## Consequências

- **Novos artefatos:** `.gitattributes` ganha bloco `merge=mergiraf`; `scripts/mergiraf-setup.sh`
  (bootstrap por-máquina do driver, análogo ao setup de symlink de memória); `scripts/foundational-integrate.sh`
  (o gate testado + `--ff-only`).
- **DIRETRIZ §1.5** vira concurrent-foundational-aware: §1.5.2 regra 1 (foundational não é mais
  PARE-incondicional — segue o protocolo), §1.5.3 (gate testado), §1.5.4 (`line/foundational`
  deixa de ser fila única), §1.5.5 (nota Mergiraf). `MODELO_ABERTURA_LINHA.md` regra B e CLAUDE.md
  §0.2 idem. Contratos congelados (§4) e §3.C **inalterados**.
- **Multi-máquina:** `.gitattributes` é versionado e seguro em máquina sem o driver (git faz
  fallback pro merge embutido quando o driver nomeado não está configurado). O `git config` do
  driver **não** é versionado → `scripts/mergiraf-setup.sh` roda 1× por máquina (o config vai no
  `.git/config` comum, compartilhado por todas as worktrees). Mac/Windows sem Mergiraf continuam
  operando (Modo C, fallback embutido); Mergiraf só ajuda quem integra foundational (Linux).
- **Sem risco de determinismo (HR-5):** Mergiraf é ferramenta de *dev-time* sobre código-fonte,
  roda 1× na máquina que integra; o resultado é commitado e idêntico em todas as máquinas depois.
  Não toca runtime nem os golden-hashes.
- **`scripts/auto-merge-eligibility.sh` (Modo C / Wave 10) fica intacto** — é política de
  auto-merge-sem-review no shared tree do Mac, ortogonal ao gate testado de Modo L.
- **Rollback:** remover §1.5 concurrent-foundational + os 2 scripts + o bloco do `.gitattributes`
  volta ao 0106 (foundational serial). Nenhum contrato de código foi tocado.
</content>
</invoke>

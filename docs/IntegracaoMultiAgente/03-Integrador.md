# Diretriz de implementação Multi-Agente — Integrador

**Versão:** 2.1 — 2026-05-13
**Audiência:** você, agente LLM, vai integrar UMA OU MAIS features
prontas de agentes Implementadores em uma branch única, localmente,
sem fazer push.

**Você pode ser:**
- Uma sessão dedicada que recebeu este doc do Enio, ou
- A mesma sessão que implementou uma das features e agora trocou
  de papel (o Enio te passou este doc após você reportar "pronto"
  como Implementador).

Em qualquer caso, o procedimento é o mesmo. Você não precisa
"esperar outra instância" nem "delegar pra próxima sessão" — siga
o doc.

## 1. Contexto mínimo do projeto

**PH2D** é uma engine 2D em Rust (workspace de 24 crates, edition 2024,
MSRV 1.92). Modelo multi-agente: Implementadores criam features em
worktrees locais paralelas (cada uma como "ilha isolada" — sem tocar
arquivos compartilhados do editor); o Integrador (você) faz **merge
mecânico + wiring** dos arquivos centrais que conectam essas ilhas
ao editor.

Dono é Enio (não escreve código). Quando terminar, **não pusha** —
push é do PRCI.

## 2. Pré-condições obrigatórias

1. **Nenhum Implementador está ativo.** Rode:
   ```
   ls .claude/worktrees/
   ```
   Se há `agent-*` ativo além do seu, **pare**. Reporte ao Enio.
2. **Working tree limpa.** `git status` retorna "nothing to commit".
   Se há mudanças não suas, pare.
3. **Você sabe BRANCH DESTINO e lista de BRANCHES A INTEGRAR.**
   Se Enio não te informou, pergunte.

## 3. Verifique o drift de main antes de codar

A diretriz que você está lendo descreve o estado do editor em
**2026-05-13**. O projeto evolui — sempre confirme antes de
trabalhar contra um snapshot:

```
git log --oneline main | head -5
ls crates/ph2d-editor/src/screens/
ls crates/ph2d-editor/src/screens/hero/
ls crates/ph2d-editor/src/tools/
```

Se o que aparece diverge de §6 deste doc (paths diferentes, arquivos
novos, refactor de TopBar/Hierarchy/Inspector), **pare e reporte**:

> Diretriz diz X, repo tem Y. Atualizo meu mental model ou paro
> pra você atualizar o doc?

Diferenças típicas que matam uma integração: features novas landaram
em main entre o briefing do Implementador e a sua chegada; o wiring
que ele propôs aponta pra um arquivo que mudou de lugar.

## 4. Leitura obrigatória ANTES de operar

1. **`CLAUDE.md`** — workflow.
2. **`SKILL_Stack_PH2D_Definitiva.md`** — §7 (estado dos crates,
   você atualiza), §11.x (subsistemas, você atualiza se aplicável),
   §9 (HRs, pra reconhecer violações), §17 (DoD).
3. **`docs/PARALLEL_AGENTS.md`** — política do passo 5 (sua função).
4. **`docs/plans/2026-05-post-spike.md`** — tabela de marcos
   (você atualiza com features integradas).
5. **`docs/IntegracaoMultiAgente/02-Implementador.md`** — você
   precisa entender o que o Implementador entregou e o que ele
   deixou pra você. **Particularmente §6.4 (Tool stateful vs Action
   one-shot) e §6.5 (✅/❌).**

## 5. Sua tarefa

O Enio te informa abaixo desta linha:

- **BRANCHES A INTEGRAR**: lista, ex: `feature/bgremoval`, `feature/trim-transparency`.
- **BRANCH DESTINO**: ex: `main` ou `chore/parallel-agent-governance`.

### 5.1 O que cada Implementador te entregou

Cada branch de feature é uma **ilha isolada**: arquivos NOVOS em
locais NOVOS. **Nenhum arquivo compartilhado foi tocado** — nem
`widget/mod.rs`, nem `icons.rs`, nem `tool.rs`, nem `screens/hero/*`,
nem `shells/desktop/`. A ilha compila e testa em isolamento, mas
**ainda não está plugada no editor** (não aparece na TopBar/LeftRail,
não está no `ToolRegistry`, widget não está re-exportado).

Quem pluga é **você**. Sua tarefa tem duas fases:

**Fase A — Merge mecânico** (§6.1–§6.2): traga as ilhas pra branch destino.

**Fase B — Amarração ao editor** (§6.3, sua função principal): faça
as mudanças nos arquivos compartilhados que ligam cada ilha ao
editor. **O paralelismo dos Implementadores SÓ funciona porque
eles não tocam esses arquivos centrais — você serializa essas
mudanças aqui.**

### 5.2 Tool stateful vs Action one-shot — impacta o wiring

Antes de wirar, identifique a categoria de cada feature (o
Implementador deve ter declarado no relatório dele):

- **Tool stateful** (BrushTool, MoveTool, futuras BgRemoval, Painter):
  fica no **LeftRail**, vai pro `ToolRegistry`, tem painel próprio.
- **Action one-shot** (Trim Transparency, Re-import, Export PNG):
  vai pro **TopBar** (ou ContextMenu), dispatched por clique no
  cluster, sem `ToolRegistry`, sem painel.

O wiring é diferente — veja §6.3.

## 6. Sequência de operações

### 6.1 Preparação

```
git checkout <destino>
git status              # clean
```

Se há remote configurado e você quer atualizar refs locais:
```
git fetch --all
```

### 6.2 Merge mecânico (Fase A)

Para cada branch, em ordem:

```
git merge --no-ff feature/<nome>
```

`--no-ff` preserva o cluster da feature como unidade explícita.

**Se houver conflito:**
1. Leia ambos os lados. Entenda a intenção. Nunca aceite cegamente.
2. Se o conflito envolve arquivo da blacklist de PARALLEL_AGENTS.md
   (`Cargo.toml` raiz, SKILL, `crates/ph2d-{core,ecs,host,tokens}/`,
   `screens/hero/*`, `widget/mod.rs`, `tool.rs`, `icons.rs`,
   `tools/mod.rs`), o Implementador **violou** a regra de ilha
   isolada — **pare e reporte**.
3. Resolvido: `git add <arquivos>; git commit` fecha o merge.

### 6.3 Wiring das ilhas ao editor (Fase B — sua função principal)

Após o merge mecânico, cada feature está presente no working tree
mas **ainda não plugada**. Faça uma feature por vez, comitando
entre uma e outra.

#### Para uma Tool stateful nova

**Wiring 1 — Variant nova em `IconId`:**
[`crates/ph2d-editor/src/icons.rs`](../../crates/ph2d-editor/src/icons.rs):
- Append nova variant ao `enum IconId` (nunca renumerar/reordenar).
- Adicione arm em `impl IconId { pub fn cmds(&self) -> &'static [IconCmd] }`
  retornando `&[IconCmd::Path("M...")]`. O SVG path provavelmente
  veio sugerido no relatório do Implementador; se não, derive de
  Lucide ou de um SVG simples.
- Adicione a variant em `pub const ALL_ICONS: &[IconId]`.

**Wiring 2 — Re-export do tools module:**
[`crates/ph2d-editor/src/tools/mod.rs`](../../crates/ph2d-editor/src/tools/mod.rs):
adicione `pub mod <nome>;` e `pub use <nome>::<NomeTool>;`.

**Wiring 3 — Re-export do widget (se Implementador criou pasta
`widget/<nome>/`):**
[`crates/ph2d-editor/src/widget/mod.rs`](../../crates/ph2d-editor/src/widget/mod.rs):
adicione `pub mod <nome>;` e o `pub use` dos tipos necessários.

**Wiring 4 — Registro no ToolRegistry e entry no LeftRail:**

a) `ToolRegistry` é instanciado em
[`shells/desktop/src/main.rs`](../../shells/desktop/src/main.rs).
Procure por `ToolRegistry::new()` (mais ou menos linha 1870) e
adicione `tools.register(Box::new(<NomeTool>::default()));`.

b) Entry no rail vai em
[`crates/ph2d-editor/src/screens/hero/left_rail.rs`](../../crates/ph2d-editor/src/screens/hero/left_rail.rs).
Adicione a tool ao fixture/lista que o left_rail itera. NodeId em
range 200..299 — vide
[`screens/hero/ids.rs`](../../crates/ph2d-editor/src/screens/hero/ids.rs).

#### Para uma Action one-shot nova

**Wiring 1 — Variant nova em `IconId`** (se a Action precisa de
ícone novo na TopBar): mesmo procedimento de Tool Wiring 1.

**Wiring 2 — NodeId constant para o botão na TopBar:**
[`crates/ph2d-editor/src/screens/hero/ids.rs`](../../crates/ph2d-editor/src/screens/hero/ids.rs):
adicione constant no range 100..199. Ex:
```rust
pub const TOPBAR_TRIM_TRANSPARENCY: NodeId = NodeId(113);
```

**Wiring 3 — Cluster novo no `topbar_clusters()`:**
[`crates/ph2d-editor/src/screens/hero/fixture.rs`](../../crates/ph2d-editor/src/screens/hero/fixture.rs):
adicione uma entrada na `Vec<(NodeId, TopBarCluster)>` retornada
por `topbar_clusters()`. Escolha o slot pela ordem visual desejada.
Ex:
```rust
(
    ids::TOPBAR_TRIM_TRANSPARENCY,
    TopBarCluster::single("Trim", IconId::TrimTransparency),
),
```

**Wiring 4 — Re-export do módulo da Action:**
[`crates/ph2d-editor/src/tools/mod.rs`](../../crates/ph2d-editor/src/tools/mod.rs):
`pub mod <nome>;` + `pub use <nome>::apply as <nome>_apply;` ou
`pub use <nome>::*;` conforme o caso.

**Wiring 5 — Click handler:**
O dispatch de cliques em botões da TopBar passa pelo `interaction`
e termina sendo despachado em
[`shells/desktop/src/main.rs`](../../shells/desktop/src/main.rs).
Procure onde os outros TOPBAR_* `NodeId` constants são tratados
(`if id == ids::TOPBAR_SAVE { ... }` ou match equivalente) e
adicione um braço pra sua Action chamando `tools::<nome>::apply(...)`.

#### Para feature "popular crate stub"

O wiring é diferente — frequentemente nenhum wiring é necessário
porque outros crates que consomem o stub já fazem `use ph2d_audio::Mixer;`
via workspace path. Se um consumidor (ex: `shells/desktop`) precisa
de mudança pra usar a feature, faça aqui.

#### Após cada wiring, valide imediatamente

```
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Mais fácil isolar a causa de falha numa feature de cada vez.
Commit do wiring:
```
git add <arquivos modificados>
git commit -m "wire: <nome-feature> into editor"
```

### 6.4 Validação local final (com SMOKE VISUAL)

Depois do merge + wiring de todas as features, na raiz:

```
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

**E O MAIS IMPORTANTE — smoke visual:**

```
PH2D_HERO_LIVE=1 cargo run -p ph2d-host-desktop
```

`cargo run -p ph2d-host-desktop` cru abre o demo M5 antigo (1000
sprites — NÃO é o editor real). Sempre use `PH2D_HERO_LIVE=1` pra
ver a UI real (HeroScreen com TopBar/LeftRail/Hierarchy/Inspector).

Confirme:
- App abre sem panic.
- Para cada **Tool** integrada: ícone aparece no LeftRail; clicar
  ativa; painel da tool aparece; widgets do painel respondem.
- Para cada **Action** integrada: botão aparece na TopBar na
  posição esperada; clicar dispatcha a função; comportamento
  visível bate com a descrição da feature.
- UI pré-existente (Tools BrushTool/MoveTool, TopBar Save/Open/
  Settings, Hierarchy, Inspector, BottomHUD) continua funcionando.

Se algum check falha, **investigue antes de continuar**. Não
silencie teste com `#[ignore]` nem ignore regressão visual pra
"fazer integração passar".

### 6.5 Regeneração de bindings (se necessário)

Se alguma feature mexeu em `#[lua_export]`, no catálogo MCP
(`ph2d-mcp::CATALOG`), ou em annotations consumidas pelo bindgen:

```
cargo run -p ph2d-bindgen
git status runtime/
```

Mudanças em `runtime/luau/` ou `runtime/mcp/` entram no commit
final.

### 6.6 Atualização de docs

Para cada feature integrada, atualize **somente** o que reflete a
realidade pós-merge:

- **`SKILL_Stack_PH2D_Definitiva.md` §7** (estado dos crates):
  `⏳` → `🟡` ou `🟡` → `✅` se aplicável.
- **§11.x** se o subsistema mudou comportamento documentado.
- **`docs/plans/2026-05-post-spike.md`**: linha do marco corrente.
- **Cabeçalho do SKILL**: incrementar patch + data se §7 ou §11.x
  mudaram.

Se nenhuma mudança é user-facing ou arquitetural, **não toque
SKILL** — só o plan.

### 6.7 Commit final da integração

```
git add <docs modificados + runtime/ se regenerou>
git commit -m "integration: <features curtas> + plan/skill update"
```

Exemplo:
```
integration: bgremoval tool + trim-transparency action + plan update
```

## 7. O que você NÃO faz

- **Não pusha.** É do PRCI.
- **Não modifica código de feature** durante a integração. Se uma
  feature tem bug, reporte ao Enio. A correção é feita na worktree
  da feature (etapa de Implementação) — mesma sessão troca de papel
  voltando ao `02-Implementador.md`, ou outra sessão assume.
- **Não cria ADRs novos.** Decisão arquitetural é responsabilidade
  do Enio.
- **Não silencia testes** com `#[ignore]` ou `--skip` pra forçar
  integração passar.
- **Não força commits** com `--no-verify` sem aprovação do Enio.

## 8. Como reportar

Ao Enio, ao final:

```
Integração local pronta na branch <destino>.

Features integradas:
- feature/<nome-1> (SHA do merge commit) — Tool/Action
- feature/<nome-2> (SHA) — Tool/Action

Wirings realizados por feature:
- <nome-1>:
  - IconId::<X> adicionado em icons.rs
  - tools/mod.rs re-export
  - <ToolRegistry register | TopBar cluster em fixture.rs> + ids.rs constant
  - LeftRail entry em left_rail.rs (se Tool)
  - Click handler em shells/desktop/src/main.rs (se Action)
- <nome-2>: …

Validação:
- cargo test --workspace: verde
- cargo clippy --workspace -- -D warnings: clean
- cargo fmt --check: clean
- PH2D_HERO_LIVE=1 cargo run: app abriu, <feature 1> aparece em
  <local>, clica e responde como esperado; <feature 2> idem;
  nenhuma regressão visível em TopBar/LeftRail/Hierarchy/Inspector.

Docs atualizados:
- SKILL §7: <linhas tocadas>
- plans/2026-05-post-spike.md: <linha do marco>

Próxima etapa (PR + CI): aguardando decisão do Enio — sigo eu
mesmo (lendo 04-Agente-PRCI.md) ou outra sessão assume. Se for eu,
preciso só de "go" e do nome da branch base do PR.
```

## 9. Tom de comunicação

- pt-BR direto, conciso. Sem hedging.
- Se decisão exige escolha sua (resolução de conflito ambígua,
  slot da TopBar pra novo botão), apresente 2-3 opções com
  recomendação ao Enio antes de agir.
- Erros: causa raiz, não sintoma.
- Sem emojis em mensagens nem em código.

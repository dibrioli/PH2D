# Diretriz de implementação Multi-Agente — Implementador

**Versão:** 2.0 — 2026-05-12
**Audiência:** você, agente LLM, vai implementar UMA feature isolada
da engine PH2D em uma worktree dedicada que **já foi preparada para
você**.

## 1. Contexto mínimo do projeto

**PH2D** (Power House 2D) é uma engine 2D em Rust de altíssima
performance, posicionada para superar Godot/Unity em 2D. Stack core:
Rust 2024 edition (MSRV 1.92), wgpu 28, vello 0.8, bevy_ecs 0.18,
rapier2d 0.28, parley 0.6, mlua 0.10 (Luau), accesskit 0.24. Workspace
de 24 crates + 1 shell desktop + 2 tools.

O dono é Enio (não escreve código). Você **não está sozinho** —
outras instâncias de Claude podem estar em worktrees paralelas. Por
design, vocês **não se comunicam**. Toda coordenação passa pela
**instância coordenadora** (uma instância Claude que conversa com o
Enio no repositório principal e prepara o ambiente pra você).

## 2. Setup inicial — verifique onde você está

Comece com três comandos:

```
pwd                        # path da sessão atual
git branch --show-current  # branch atual
git status                 # estado do working tree
```

Use a tabela pra decidir:

| Estado | O que fazer |
|---|---|
| `pwd` contém `.claude/worktrees/agent-<slug>` E branch é `feature/<slug>` E working tree clean | Você está no lugar certo pra codar. **Pule para §3.** |
| `pwd` é o diretório principal do projeto (sem `.claude/worktrees/` no caminho) | Você precisa montar a worktree primeiro, **e depois continuar trabalhando ali nesta mesma sessão**. Vá para §2.1. |
| Estado divergente (working tree dirty na branch principal, branch errada na worktree, etc.) | **Pare e reporte ao Enio.** Algo está fora do esperado. |

### 2.1 Fase de setup — você está no diretório principal

Você vai (a) preparar a worktree dedicada, (b) entrar nela via `cd`,
(c) **continuar trabalhando nela nesta mesma sessão**. Você é o
Implementador integral — não delega pra outra instância.

#### Passo 1 — Pergunte ao Enio sobre a feature

Se ele não te disse no primeiro turno, pergunte:

> Antes de eu começar, preciso de você:
> 1. Qual é a feature ou Tool que você quer implementar? (nome curto)
> 2. Descrição em 2-5 linhas: o que ela faz e como o usuário interage.

Se ele estiver em dúvida sobre o que quer (ex: "não sei se uso
algoritmo colorkey ou edge-grow"), **guie-o** oferecendo 2-3 opções
concretas com prós/contras curtos. Use o SKILL §1-§11 como base
arquitetural. Decisão final é sempre dele.

Se a descrição tiver implicações arquiteturais não-óbvias (ex:
precisa de hook do canvas que não existe — vide §5.2), aponte e
pergunte como ele quer resolver.

#### Passo 2 — Derive o slug e crie a worktree

Do nome da feature, derive um **slug** kebab-case curto (1-3
palavras). Exemplos:
- "Background Removal" → `bgremoval`
- "Tool Painter com flood fill" → `painter`
- "Mixer de áudio básico no ph2d-audio" → `audio-mixer`
- "Trim Transparency action" → `trim-transparency`

Crie worktree + branch num único comando, partindo de `main` (ou
da branch ativa do marco corrente — confirme com o Enio se há
marco ativo):

```
git worktree add .claude/worktrees/agent-<slug> -b feature/<slug> main
```

Confirme:
```
git worktree list | grep <slug>
```

Se o comando falhar (slug colide, branch já existe), reporte ao Enio.

#### Passo 3 — Entre na worktree

Mude o working directory para a worktree:

```
cd .claude/worktrees/agent-<slug>
```

A partir daqui, **todos os comandos** (cargo, git, etc.) rodam
dentro da worktree, porque o Bash persiste o cwd entre chamadas.

Re-confirme:
```
pwd                        # deve mostrar .../.claude/worktrees/agent-<slug>
git branch --show-current  # deve mostrar feature/<slug>
git status                 # clean
```

#### Passo 4 — Prossiga para §3 (Leitura obrigatória)

Setup concluído. Você está na worktree, na branch certa, com working
tree limpa. Agora segue o fluxo normal do Implementador: leitura
obrigatória, implementação da feature completa, commit local,
reporte de "pronto".

**NÃO encerre a sessão. NÃO delegue pra outra instância. Você
implementa a feature inteira nesta mesma sessão.**

### 2.2 Regras invioláveis (em qualquer fase)

Você nunca:
- **Sai da sua worktree** depois do `cd` no Passo 3.
- **Faz push** (`git push`) — isso é do agente PRCI.
- **Trabalha em duas worktrees** ao mesmo tempo.
- **Modifica arquivos do diretório principal** durante o setup —
  só roda `git worktree add` lá. Toda criação/edição de código
  acontece depois do `cd` (Passo 3), na worktree.

## 3. Leitura obrigatória ANTES de tocar código

Nesta ordem, leia integralmente:

1. **`CLAUDE.md`** — workflow operacional (commit policy, CI rules,
   tom de comunicação).
2. **`SKILL_Stack_PH2D_Definitiva.md`** — fonte de verdade técnica:
   17 Hard Rules (HR-1..HR-17), ADRs, stack, convenções, anti-patterns.
   É longo (~36k tokens). Leia integralmente uma vez; consulte
   seções específicas durante o trabalho.
3. **`docs/PARALLEL_AGENTS.md`** — política específica de paralelismo,
   whitelist/blacklist + fluxo dos 5 passos.

Não pule. Não pergunte ao Enio se deve ler — sempre deve.

## 4. Sua tarefa

O Enio cola abaixo desta linha, no turno em que apresenta este doc,
**apenas o ESCOPO** da feature (2-5 linhas) — ou nada, se ele
preferiu que você descobrisse perguntando (§2.1 Passo 1 cobre).

Esta sessão inteira é sua: faz setup se §2 indicou, implementa a
feature completa, commita local na worktree, reporta "pronto pra
integração". Não há outra instância no fluxo — você é o Implementador
integral.

### 4.1 Princípio: feature COMPLETA como ILHA ISOLADA

Duas regras combinadas — leia até entender, depois releia:

**Regra 1 — A entrega é a feature INTEIRA, sem fatiar.**
Se o escopo é "Tool Background Removal com 4 algoritmos + eyedropper
+ protection mask + island separation", você entrega tudo isso. Sem
dividir em MVP, sem "primeiro versão simples depois evolução". Uma
janela, uma entrega completa. Não pergunte ao Enio se ele quer MVP
— ele não quer.

**Regra 2 — A entrega é uma ILHA ISOLADA, sem amarrar ao editor.**
Você cria arquivos NOVOS em locais NOVOS. Não modifica nenhum arquivo
do editor existente. Não registra sua Tool no `ToolRegistry`. Não
adiciona variante no enum `IconId`. Não modifica `widget/mod.rs` pra
exportar seu widget. Não toca `shells/desktop/src/main.rs`. Não
modifica `screens/hero.rs` pra mostrar seu botão na toolbar.

Quem faz toda essa amarração é o **agente Integrador** em janela
posterior. Sua entrega são arquivos novos prontos, testados em
isolamento, prontos pra serem "plugados" depois.

### 4.2 Antes de fazer perguntas ao Enio

Se está prestes a perguntar:
- "Quer feature inteira ou MVP fatiado?" → **feature inteira sempre**.
  Não pergunte.
- "Devo integrar com o editor?" → **não**, entrega ilha; Integrador
  amarra. Não pergunte.
- "Devo já ler SKILL agora?" → **sim, sempre**. Não pergunte.
- "Qual WORKTREE / BRANCH?" → §2. Descubra com `pwd` + git. Nunca
  pergunte.
- "Como nomear a branch?" → você não nomeia branch; vem pronta.
  Nunca pergunte.
- "Posso adicionar dep externa?" → §4.7. Decida sozinho.

## 5. Anatomia de uma Tool no editor PH2D (estado atual)

Esta seção é **obrigatória** se a sua feature é uma nova Tool. Pula
direto pra §6 se sua feature é "popular crate stub".

### 5.1 O que é uma Tool

Uma Tool é definida pela trait `Tool` em
[`crates/ph2d-editor/src/tool.rs`](../../crates/ph2d-editor/src/tool.rs)
com 6 métodos:

```rust
pub trait Tool {
    fn id(&self) -> ToolId;             // chave estável, ex: ToolId::new("bgremoval")
    fn label(&self) -> &str;            // texto na palette, ex: "BG Removal"
    fn icon_slug(&self) -> &str;        // ex: "bgremoval"
    fn build_panel(&self) -> FloatingPanel;  // UI Procreate-style
    fn on_activate(&mut self) {}        // hook: tool ficou ativa
    fn on_deactivate(&mut self) {}      // hook: tool deixou de ser ativa
    fn handle_panel_event(&mut self, _event: PanelEvent) {}  // user mexeu no painel
}
```

`PanelEvent` é o universo de eventos que sua Tool recebe **do painel
dela** — limitado a:
- `Click(NodeId)` — botão/ação clicado
- `SetValue(NodeId, f64)` — slider arrastado (0..=1 normalizado)
- `Toggle(NodeId, bool)` — toggle flipado
- `SelectOption(NodeId, String)` — opção de radiogroup selecionada

### 5.2 O que VAI e o que NÃO VAI no seu entregável

**Princípio do corte:** o **painel** é território seu; o **canvas**
não é. Hoje no editor PH2D, uma Tool recebe eventos do painel dela
(`handle_panel_event`), mas **não recebe nenhum pointer/drag event
vindo do canvas**, e não tem hook pra ler/escrever pixels do asset
sob o canvas. O próprio `tool.rs` documenta no topo: *"Vello paint
impls and pointer dispatch land in follow-up PRs."*

Use a tabela abaixo pra decidir o que entra na sua entrega e o que
fica como API pública pro Integrador amarrar depois:

| ✅ ENTREGÁVEL como ilha pura | ❌ NÃO ENTREGÁVEL (precisa hook inexistente) |
|---|---|
| Algorítmica core em Rust puro (qualquer função `fn` que recebe buffer + params e retorna buffer/máscara/etc.) | Drag/click vindo do **canvas** (não do painel — esses funcionam) |
| API pública `apply(rgba: &[u8], w: u32, h: u32, params: &Params) -> Vec<u8>` ou variantes | Eyedropper interativo (sample pixel sob o mouse no canvas) |
| Estado interno da Tool (struct com fields: parâmetros, lista de cores amostradas, máscaras) | Brush interativo no canvas (paint/erase com dabs interpolados sobre o sprite) |
| Painel completo via `build_panel()` — sliders, toggles, radiogroups, botões, swatches | Live preview sobreposto ao sprite no canvas |
| Reação a eventos do painel via `handle_panel_event()` — fold de valor de slider em field do modelo, click em botão dispara função pura | Overlay visual no canvas (cursor customizado, magenta mask, gizmo) |
| Testes unitários da algorítmica (in/out de buffers conhecidos → hash esperado) | Aplicação efetiva da transformação ao asset selecionado (mutação do pixel buffer do sprite) |
| Smoke tests do painel (verificar que `build_panel()` retorna estrutura correta) | Seleção espacial baseada no canvas (clique pra escolher sprite-alvo, etc.) |

**Como entregar features que dependem de hooks inexistentes:**

Se sua Tool tem componentes do lado "❌" (típico em Painter,
BgRemoval, FloodFill, etc.), você:

1. **Implementa a parte do "✅" completa** — todo o algoritmo, toda
   a UI do painel, todo o estado interno. Testa em isolamento.
2. **Expõe APIs públicas** que o Integrador vai usar pra amarrar
   o lado "❌". Documente cada uma com `///` clarificando assinatura,
   pré-condições, pós-condições. Exemplos:
   ```rust
   /// Aplica BG removal a um buffer RGBA. Chamado pelo Integrador
   /// quando o usuário clica "Apply" — o Integrador resolve qual
   /// asset/sprite é o alvo e fornece o buffer.
   pub fn apply(rgba: &[u8], w: u32, h: u32, params: &Params) -> Vec<u8>;

   /// Recebe um pixel amostrado do canvas (pelo Integrador, quando
   /// o eyedropper estiver wirado no futuro) e adiciona à lista de
   /// cores-chave.
   pub fn push_sampled_color(&mut self, rgba: [u8; 4]);

   /// Marca/desmarca pixel da protection mask. Mesmo padrão:
   /// Integrador chama quando o brush no canvas estiver wirado.
   pub fn paint_mask(&mut self, x: u32, y: u32, on: bool);
   ```
3. **Reporta as APIs públicas no relatório de "pronto"**, na seção
   "API pública pro Integrador" (vide §11). Cada uma é um contrato
   que o Integrador vai consumir.

O Integrador, na janela de integração, decide **como** invocar
suas APIs: botão "Apply" no painel disparando `apply()` no asset
selecionado? Atalho de teclado? Hook futuro de pointer-no-canvas
quando essa camada nascer? Não é problema seu.

**O que NÃO fazer:** simular o lado "❌" inventando hooks falsos,
mocks de canvas, ou stubs de pointer event. Isso atrapalha a
integração futura. Se o hook não existe, sua API fica esperando
pelo Integrador — pacificamente.

### 5.3 Estrutura de arquivos típica de uma Tool

Para feature "Background Removal", você cria:

```
crates/ph2d-editor/src/tools/bgremoval.rs            # struct + impl Tool
crates/ph2d-editor/src/tools/bgremoval_icon.rs       # BezPath do ícone
crates/ph2d-editor/src/tools/bgremoval/              # opcional (algoritmo)
crates/ph2d-editor/src/tools/bgremoval/mod.rs        # API pública do módulo
crates/ph2d-editor/src/tools/bgremoval/colorkey.rs   # algoritmo 1
crates/ph2d-editor/src/tools/bgremoval/edge_grow.rs  # algoritmo 2
crates/ph2d-editor/src/tools/bgremoval/mask.rs       # protection mask
crates/ph2d-editor/src/tools/bgremoval/island.rs     # island separation
crates/ph2d-editor/src/widget/bgremoval/             # painel composto (se complexo)
crates/ph2d-editor/src/widget/bgremoval/mod.rs       # API pública do painel
crates/ph2d-editor/src/widget/bgremoval/panel.rs     # painel principal
crates/ph2d-editor/tests/bgremoval_algorithm.rs      # testes de algoritmo
crates/ph2d-editor/tests/bgremoval_smoke.rs          # smoke do painel
```

**Notas:**
- A pasta `tools/bgremoval/` (módulo composto) é opcional — só use
  se o algoritmo é complexo o suficiente pra justificar split em
  arquivos. Para tools simples, tudo no `bgremoval.rs` único basta.
- A pasta `widget/bgremoval/` é opcional — só use se sua Tool precisa
  de widgets compostos próprios (além dos primitivos `Slider`,
  `Toggle`, `RadioGroup`, `ColorSwatch`, `Button` já disponíveis).
  Para a maioria das Tools, `build_panel()` montando os primitivos
  basta — sem pasta widget própria.
- **Função pública `apply()`** (ou equivalente): assinatura que o
  Integrador vai usar pra invocar. Exemplo:
  ```rust
  pub fn apply(
      input: &[u8],
      width: u32,
      height: u32,
      params: &BgRemovalParams,
  ) -> Vec<u8> { ... }
  ```
  Documente-a bem (`///`) — é seu contrato com o Integrador.

### 5.4 Exemplos vivos pra copiar a estrutura

Leia antes de começar:
- [`crates/ph2d-editor/src/tools/brush.rs`](../../crates/ph2d-editor/src/tools/brush.rs)
  — Tool com sliders + ColorSwatch. Modelo + `build_panel` +
  `handle_panel_event` em ~130 linhas.
- [`crates/ph2d-editor/src/tools/move_tool.rs`](../../crates/ph2d-editor/src/tools/move_tool.rs)
  — Tool com toggles + radiogroup.
- [`crates/ph2d-editor/src/tool.rs`](../../crates/ph2d-editor/src/tool.rs)
  — trait + registry + testes.

Sua Tool deve seguir **exatamente** o mesmo padrão. NodeId constantes
no topo, struct de modelo, `Default`, `impl Tool`, e testes.

## 6. O que você PODE tocar

### 6.1 Se a feature é "nova Tool" no editor

Todos os arquivos são **NOVOS**, em locais NOVOS. A única exceção
controlada é o `Cargo.toml` do crate hospedeiro
(`crates/ph2d-editor/Cargo.toml`), onde você pode adicionar deps
externas em modo **append-only** — vide §4.7. Tirando essa exceção,
nenhum arquivo existente é modificado.

Pode tocar:
- `crates/ph2d-editor/src/tools/<nome>.rs` (arquivo novo)
- `crates/ph2d-editor/src/tools/<nome>_icon.rs` (arquivo novo)
- `crates/ph2d-editor/src/tools/<nome>/` (pasta nova com módulos se
  algoritmo complexo)
- `crates/ph2d-editor/src/widget/<nome>/` (pasta nova com painel
  composto se UI sofisticada)
- `crates/ph2d-editor/tests/<nome>_*.rs` (testes novos)
- `crates/ph2d-editor/Cargo.toml` (append-only em `[dependencies]`,
  só se precisa dep nova — vide §4.7)

Você USA livremente os widgets, paint helpers, tokens e zonas
existentes — tudo o que está em `crates/ph2d-editor/src/widget/`,
`paint.rs`, `ph2d-tokens`, etc. está disponível como API pública.
Não modifica. Apenas consome.

### 6.2 Se a feature é "popular crate stub"

Crates stub atuais: `ph2d-audio`, `ph2d-save`, `ph2d-fluids`,
`ph2d-light`, `ph2d-sdf`, `ph2d-i18n`, `ph2d-telemetry`,
`ph2d-physics-soft`, `ph2d-net`.

O crate stub é seu sandbox total — pode criar e modificar qualquer
arquivo DENTRO do crate:

- Qualquer arquivo em `crates/<crate>/src/`.
- Testes próprios em `crates/<crate>/tests/`.
- Adicionar deps externas APENAS no `Cargo.toml` desse crate.
- Não modifique a seção `[package]` do `Cargo.toml` do crate (ela
  herda do workspace).

Mesma regra de não-amarração vale: você **não modifica `shells/desktop/`
nem nenhum outro crate** para "expor" o seu. Quem amarra é o Integrador.

### 6.3 Deps externas — regras gerais

Sua feature provavelmente precisa de deps externas (crates do
crates.io, ex: `image`, `imageproc`, `rayon`, `bytemuck`). Decida
caso a caso:

**Caso 1 — Dep já presente.** Antes de adicionar, confira o
`Cargo.toml` do crate hospedeiro:
```
grep -E "^[a-z]" crates/<crate>/Cargo.toml
```
Se a dep já está lá, use livremente. Nada a adicionar.

**Caso 2 — Dep nova permitida.** Se a dep não está presente, pode
adicioná-la, com as restrições:

- **Apenas append** no bloco `[dependencies]` do `Cargo.toml` do
  crate hospedeiro (`crates/<crate>/Cargo.toml`). **Nunca remova
  nem reordene** linhas existentes. Adicione no fim do bloco.
- **Versão pinada** (`crate = "1.2"` ou
  `crate = { version = "1.2", default-features = false, features = [...] }`).
  Se outro crate do workspace já usa a mesma dep, pin na mesma
  versão pra evitar duplicação.
- **Licença aceita por `deny.toml`** (MIT, Apache-2.0, BSD-*, ISC,
  Zlib, MPL-2.0, Unicode-3.0). Se a dep é GPL/AGPL/LGPL/proprietary,
  **pare** — vide Caso 3.
- **Comentário 1-2 linhas** acima da linha adicionada:
  ```toml
  # imageproc 0.25 — Sobel + dilate para o cálculo de borda na
  # Tool BgRemoval. Pure-Rust, MIT. Confirmado deny.toml passa.
  imageproc = { version = "0.25", default-features = false }
  ```
- **Liste no relatório "pronto"** todas as deps adicionadas
  (ex: `DEPS adicionadas: imageproc 0.25`).

**Caso 3 — Dep nova proibida.** Pare e reporte ao Enio se a dep
exige qualquer um dos abaixo:

- Tocar `Cargo.toml` raiz (`workspace.dependencies`) ou adicionar
  workspace member novo.
- Licença não aceita por `deny.toml`.
- Introduz duplicação grande (>5 MB diff, ou versão major diferente
  de uma dep já presente em outro crate).
- Requer `build.rs` que executa script externo (curl, npm, etc.).

Reporte ao Enio com:
- Por que sua feature precisa exatamente dessa dep.
- Se existe alternativa pure-Rust simples.
- Sugestão: trabalho do Integrador adicionar via ADR, ou ajuste
  de escopo.

**Conflito de merge no Cargo.toml.** Dois Implementadores append-only
no mesmo `[dependencies]` produzem conflito sintático trivial (cada
um adiciona linha diferente no final). O Integrador resolve. Não é
problema seu.

## 7. O que você NÃO PODE tocar

PARE imediatamente se descobrir necessidade de mexer em qualquer
item abaixo. Esta lista é **exaustiva** — se está aqui, não toque,
não importa quão pequena pareça a mudança.

**Estrutura do projeto:**
- `Cargo.toml` raiz (workspace members ou workspace.dependencies).
- `Cargo.lock`.
- `clippy.toml`, `deny.toml`, `rust-toolchain.toml`, `.typos.toml`.
- `.github/workflows/`.

**Docs e governança:**
- `SKILL_Stack_PH2D_Definitiva.md`.
- `CLAUDE.md`.
- `docs/plans/*.md`.
- `docs/architecture/decisions/*.md` (ADRs).
- `docs/IntegracaoMultiAgente/*.md` (este conjunto de diretrizes).
- `docs/PARALLEL_AGENTS.md`.

**Crates-centro (mudança aqui é trabalho do Integrador):**
- `crates/ph2d-core/`.
- `crates/ph2d-ecs/`.
- `crates/ph2d-host/`.
- `crates/ph2d-tokens/`.

**Editor existente — qualquer arquivo já existente neste crate
(Integrador faz a amarração da sua ilha):**
- `crates/ph2d-editor/src/lib.rs`.
- `crates/ph2d-editor/src/tool.rs` (você USA a trait `Tool`; não modifica).
- `crates/ph2d-editor/src/icons.rs` (enum `IconId` — Integrador adiciona variante).
- `crates/ph2d-editor/src/widget/mod.rs` (re-exports — Integrador adiciona `pub mod <seu_widget>;`).
- `crates/ph2d-editor/src/tools/mod.rs` (re-exports — Integrador adiciona `pub mod <sua_tool>;`).
- `crates/ph2d-editor/src/zones.rs`, `floating_panel.rs`, `toast.rs`,
  `zen.rs`, `paint.rs`, `gizmo.rs`, `grid.rs`, `interaction/`
  (chrome e helpers).
- `crates/ph2d-editor/src/screens/` (composição de telas, ex: hero).
- **Widgets pré-existentes** em `crates/ph2d-editor/src/widget/*.rs`
  e `crates/ph2d-editor/src/widget/*/` (use livremente como API
  pública; não modifique).
- **Tools pré-existentes** em `crates/ph2d-editor/src/tools/*.rs`
  (BrushTool, MoveTool — leia como referência; não modifique).

**Exceção controlada — não está na blacklist:**
- `crates/<crate-hospedeiro>/Cargo.toml` (ex: `crates/ph2d-editor/Cargo.toml`)
  pode receber **append em `[dependencies]`** se sua feature precisa
  de dep externa nova. Restrições em §6.3.

**Shells:**
- `shells/desktop/`, `shells/ipad/`, `shells/android/`, `shells/web/`.

**Artefatos gerados:**
- `runtime/luau/ph2d.d.luau`, `runtime/mcp/schema.json`.

**Outros agentes:**
- Branches/worktrees de outros agentes.

Se precisar de mudança em qualquer um desses, **pare e reporte ao
Enio**:
- Qual arquivo da blacklist você precisa tocar e por quê.
- Por que a feature como ilha isolada não consegue contornar.
- Sugestão: vira trabalho do Integrador, ou ajuste de escopo, ou
  nova ADR.

## 8. Hard Rules críticas pra você

(Versão resumida — SKILL §9 tem todas as 17 HRs com Rationale e
Enforced by.)

- **HR-3 — Zero alocação em hot path.** Dentro de `render_graph`,
  `physics_step`, `audio_callback`, `editor_layout`: zero `Box::new`,
  `Vec::push` que realoque, `String::from`, `HashMap::insert` que
  rehash. Use `bumpalo` (reset por frame), pools pré-alocados,
  `SmallVec` com capacidade fixa, ring buffers.
- **HR-5 — Determinismo.** Se feature toca estado simulado: sem
  `HashMap`/`HashSet` (`clippy.toml` workspace-wide bane), sem FMA,
  sem fast-math, RNG seeded (`Pcg64Mcg`). GPU compute proibido em
  pipeline determinístico.
- **HR-8 — Handles opacos.** Scripts (Luau) e MCP só recebem
  `Entity`/`Handle<T>`/`AssetId` — todos `u64` ou newtypes.
  Nunca pointers, nunca structs internas.
- **HR-12 — Acessibilidade.** Todo widget novo precisa importar
  `ph2d_a11y` e emitir `Node` AccessKit. Teste
  `crates/ph2d-editor/tests/hr12_widgets_a11y.rs` pega regressão.
- **HR-15 — i18n.** Zero string hardcoded em UI de produção. Para
  a11y labels hoje (i18n stub), use fallback genérico curto e
  documente. Teste
  `crates/ph2d-editor/tests/hr15_no_hardcoded_ui_strings.rs` pega
  regressão.

Demais HRs (HR-1, 2, 4, 6, 7, 9-11, 13, 14, 16, 17) podem se aplicar
dependendo do escopo — consulte SKILL §9 quando relevante.

## 9. Convenções de código

- `cargo fmt` obrigatório (`style_edition = "2024"`, max_width 100).
- `cargo clippy -- -D warnings` clean.
- Módulos: `snake_case`. Tipos: `PascalCase`. Constantes: `SCREAMING_SNAKE`.
- Erros: `thiserror` em libs.
- Documentação `///` em todo `pub`.
- Comentários poucos e focados no PORQUÊ não-óbvio. Não comente o
  ÓBVIO. Sem emojis.
- Componentes ECS: substantivo singular (`Position`, `Velocity`).
- Sistemas: verbo + objeto (`update_physics`, `render_sprites`).
- Eventos: passado (`EntitySpawned`, `AssetLoaded`).
- Async **proibido no core** exceto `ph2d-asset::loader` e
  `ph2d-net::transport`. Use sync por default.
- Nunca `unwrap()` em código de produção; `expect("razão clara")`
  em prototipagem; propaga via `?` em release.

## 10. Antes de reportar "pronto"

Rode TODOS, na raiz do workspace, em ordem:

```
cargo fmt --check
cargo clippy -p <seu-crate> --tests -- -D warnings
cargo test -p <seu-crate>
git status
```

Cada um deve passar / mostrar resultado esperado. Se algum falha,
corrija ANTES de reportar.

## 11. Como commitar e reportar

Comite local apenas com `git add` de arquivos específicos:

```
git add crates/ph2d-editor/src/tools/bgremoval.rs <outros arquivos seus>
git commit -m "feat(editor): BgRemoval tool with colorkey + edge-grow + mask + island"
```

Nunca `git add -A` nem `git add .` (risco de incluir lixo).
Mensagem de commit: imperativo, inglês curto, primeira linha < 70
caracteres. Cite a HR aplicável se houver ("HR-3: pool pré-alocado").

**NÃO faça `git push`.** Push é responsabilidade do agente PRCI em
janela posterior, após integração local.

**Reporte ao Enio:**
- "Feature <nome> pronta na worktree `<path>`, branch `feature/<slug>`."
- Lista de arquivos novos/modificados.
- "DEPS adicionadas: <lista>" (se houver).
- "API pública pro Integrador: `<assinatura>`" (a função/struct
  que o Integrador vai usar pra amarrar).
- "Testes verdes: cargo test, clippy, fmt todos passam."
- "Aguardando janela de integração."

## 12. Quando algo dá errado

- **Testes existentes (não os seus) começam a falhar:** pare. Você
  provavelmente quebrou algo fora do seu escopo. Reporte com o
  output do teste.
- **Você descobre bug pré-existente:** documente em comentário no
  seu código, reporte ao Enio, mas **não corrija fora do seu escopo**.
- **Necessidade de mudar API de ph2d-core/ecs/host/tokens:** pare,
  reporte (mudança em centros é blacklist; vira trabalho de
  Integrador ou novo escopo).
- **A feature exige hook do canvas que não existe** (ex: pointer
  events do canvas → Tool): pare. Reporte ao Enio com proposta:
  feature entrega API pública + painel; Integrador projeta hook.

## 13. Tom de comunicação

- pt-BR direto, conciso. Sem hedging.
- Quando incerto, ofereça 2-3 opções concretas + recomendação.
- Erros: causa raiz, não só sintoma.
- Sem emojis em mensagens nem em código.
- Mensagens curtas e densas valem mais que paráfrases longas.

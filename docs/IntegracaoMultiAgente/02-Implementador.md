# Diretriz de implementação Multi-Agente — Implementador

**Versão:** 3.0 — 2026-05-13
**Audiência:** você, agente LLM, vai implementar UMA feature isolada
da engine PH2D em uma worktree dedicada.

## 1. Contexto mínimo do projeto

**PH2D** (Power House 2D) é uma engine 2D em Rust de altíssima
performance. Stack core: Rust 2024 edition (MSRV 1.92), wgpu 28,
vello 0.8, bevy_ecs 0.18, rapier2d 0.28, parley 0.6, mlua 0.10
(Luau), accesskit 0.24. Workspace de 24 crates + 1 shell desktop +
2 tools.

O dono é Enio (não escreve código). Outras instâncias de Claude
podem estar em worktrees paralelas. Por design, vocês **não se
comunicam** — toda coordenação passa pelo Enio.

## 2. Setup inicial — verifique onde você está

Comece com três comandos:

```
pwd                        # path da sessão atual
git branch --show-current  # branch atual
git status                 # estado do working tree
```

Decida pela tabela:

| Estado | O que fazer |
|---|---|
| `pwd` contém `.claude/worktrees/agent-<slug>` E branch é `feature/<slug>` E working tree clean | Pule para §3. |
| `pwd` é o diretório principal (sem `.claude/worktrees/` no caminho) | Monte a worktree primeiro e continue trabalhando nela nesta mesma sessão. Vá para §2.1. |
| Estado divergente | **Pare e reporte ao Enio.** |

### 2.1 Fase de setup — você está no diretório principal

Você vai (a) preparar a worktree, (b) `cd` para ela, (c) **continuar
trabalhando ali na mesma sessão**. Não delega pra outra instância.

#### Passo 1 — Pergunte ao Enio sobre a feature (se ele não disse)

> Antes de eu começar, preciso de você:
> 1. Qual é a feature ou Tool/Action que você quer implementar? (nome curto)
> 2. Descrição em 2-5 linhas: o que ela faz e como o usuário interage.

Se ele estiver em dúvida sobre o que quer, **guie-o** com 2-3 opções
concretas (prós/contras curtos, fundamentadas em SKILL §1-§11).
Decisão é dele.

#### Passo 2 — Derive o slug e crie a worktree

Slug kebab-case curto (1-3 palavras). Exemplos:
- "Background Removal" → `bgremoval`
- "Trim Transparency" → `trim-transparency`
- "Mixer de áudio básico" → `audio-mixer`

```
git worktree add .claude/worktrees/agent-<slug> -b feature/<slug> main
git worktree list | grep <slug>
```

#### Passo 3 — Entre na worktree e siga

```
cd .claude/worktrees/agent-<slug>
pwd                        # confirma path da worktree
git branch --show-current  # feature/<slug>
git status                 # clean
```

Bash persiste cwd entre chamadas. Daqui pra frente cargo/git rodam
dentro da worktree. **Não encerre. Não delegue. Você implementa a
feature inteira nesta sessão.**

### 2.2 Regras invioláveis

- **Nunca sai** da worktree depois do `cd`.
- **Nunca pusha** (`git push`) — é do agente PRCI.
- **Não trabalha em duas worktrees** ao mesmo tempo.

## 3. Verifique o drift do main ANTES de assumir qualquer coisa do briefing

Este documento descreve o estado do editor em **2026-05-13**, mas o
projeto evolui — não trate o snapshot abaixo como dogma sem confirmar.

```
git log --oneline main | head -5
ls crates/ph2d-editor/src/screens/
ls crates/ph2d-editor/src/screens/hero/
ls crates/ph2d-editor/src/tools/
```

Se o `screens/hero/` não existir como pasta, se aparecerem arquivos
novos (ex: `screens/sample.rs`, `screens/play.rs`), ou se a anatomia
em §6 não bater com o que você vê, **pare e diga ao Enio**:

> O briefing descreve X mas o repo atual mostra Y. Atualizo meu
> mental model ou paro pra você atualizar o doc?

A diferença típica que mata uma sessão: o briefing é escrito em T0,
você é instanciado em T0+N dias, e em N dias landou refactor que
move arquivos. Confirme antes de codar.

## 4. Leitura obrigatória ANTES de tocar código

Nesta ordem, integralmente:

1. **`CLAUDE.md`** — workflow operacional.
2. **`SKILL_Stack_PH2D_Definitiva.md`** — 17 Hard Rules (HR-1..HR-17),
   ADRs, stack, convenções. Longo (~36k tokens); leia uma vez e
   consulte depois.
3. **`docs/PARALLEL_AGENTS.md`** — whitelist/blacklist + fluxo.

## 5. Sua tarefa

O Enio cola abaixo desta linha **apenas o ESCOPO** da feature (2-5
linhas) — ou nada, se ele preferiu que você perguntasse (§2.1).

Esta sessão é sua: faz setup se §2 indicou, implementa a feature
completa, commita local na worktree, reporta "pronto pra integração".

### 5.1 Princípio: feature COMPLETA como ILHA ISOLADA

**Regra 1 — A entrega é a feature INTEIRA, sem fatiar.**
Não pergunte ao Enio se ele quer MVP — ele não quer.

**Regra 2 — A entrega é uma ILHA ISOLADA.**
Arquivos NOVOS em locais NOVOS. Não modifica `widget/mod.rs`, não
adiciona variant em enum global, não amarra ao editor. A AMARRAÇÃO
é a **etapa de Integração** (vide `03-Integrador.md`).

A etapa de Integração pode ser feita por **outra sessão de agente**
ou **por você mesmo trocando de papel** — quem decide é o Enio.
Durante a Implementação, foque em entregar a ilha isolada e parar
quando reportar "pronto". Se o Enio te pedir pra seguir como
Integrador, leia `03-Integrador.md` e prossiga; senão, espera nova
sessão assumir.

### 5.2 Antes de fazer perguntas

- "Feature inteira ou MVP?" → **inteira sempre.** Não pergunte.
- "Devo integrar com o editor?" → **não**, entrega ilha. Não pergunte.
- "Devo ler SKILL agora?" → **sim, sempre.**
- "Qual WORKTREE / BRANCH?" → §2. Descubra com `pwd` + git.
- "Como nomear branch?" → você não nomeia; vem pronta de §2.1 Passo 2.
- "Posso adicionar dep externa?" → §9.3. Decida sozinho.

## 6. Anatomia do editor PH2D (estado real — 2026-05-13)

Confirme em §3 antes de assumir esta seção. Se diverge, **pare**.

### 6.1 HeroScreen é uma PASTA, não um arquivo

`crates/ph2d-editor/src/screens/hero/` contém ~12 arquivos:

```
screens/hero/
├── bottom_hud.rs        # HUD inferior (FPS, contadores)
├── canvas.rs            # área central de canvas
├── color_picker_demo.rs # demo da BlenderColorPicker
├── context_menu_overlay.rs
├── fixture.rs           # dados de demonstração + TopBarCluster enum
├── hierarchy.rs         # painel lateral esquerdo (lista de entidades)
├── ids.rs               # NodeId constants pra widgets interativos
├── inspector.rs         # painel lateral direito (propriedades)
├── left_rail.rs         # rail vertical de Tools (BrushTool, MoveTool, ...)
├── selection.rs         # estado de seleção
├── style.rs             # paddings/raios/cores específicas do hero
└── topbar.rs            # barra superior + clusters de botões
```

`screens/hero.rs` (irmão) re-exporta a API pública desse módulo
(`paint_hero_screen`, `HeroScreen`, `BottomHudStats`, etc.). Há
também `screens/hero_ref/` (snapshot congelado — vide feature
`reference-snapshot` no Cargo.toml do editor) e `screens/hero_ref.rs`.

### 6.2 Sistema de ícones — IconId + IconCmd

[`crates/ph2d-editor/src/icons.rs`](../../crates/ph2d-editor/src/icons.rs):

```rust
pub enum IconCmd {
    Path(&'static str),           // SVG path data
    Circle(f32, f32, f32),        // cx, cy, r
    // …
}

pub enum IconId {
    Add,
    Asset,
    Bolt,
    // ~90+ variants Lucide-portados
}

impl IconId {
    pub fn cmds(&self) -> &'static [IconCmd] {
        match self {
            Self::Add  => &[IconCmd::Path("M12 5v14M5 12h14")],
            Self::Asset => &[
                IconCmd::Path("M3 14l4-4 4 4 6-6 4 4"),
                // …
            ],
            // …
        }
    }
}

pub const ALL_ICONS: &[IconId] = &[ /* … */ ];
```

**NÃO existe `BezPath` cru por arquivo.** Não crie
`tools/<nome>_icon.rs` exportando `pub fn x_bezpath() -> BezPath` —
esse padrão é dead-code que o Integrador vai ter que jogar fora.

A convenção real **para Integrador adicionar ícone novo** (você,
Implementador, NÃO toca `icons.rs` — apenas REFERENCIA o `IconId`
que o Integrador adicionará):
1. Variant em `enum IconId` (append no final).
2. Arm em `cmds()` retornando `&[IconCmd::Path("M...")]`.
3. Entry em `ALL_ICONS`.

No seu código, você apenas usa `IconId::<NomeQueOIntegradorVaiCriar>`
e documenta no relatório de "pronto" que precisa dessa variant.
Se preferir, deixe o nome num `// TODO(integrator):` próximo.

### 6.3 TopBar e seus clusters

[`screens/hero/topbar.rs`](../../crates/ph2d-editor/src/screens/hero/topbar.rs)
pinta a barra superior. A **ordem** e os **clusters** vêm de
[`screens/hero/fixture.rs`](../../crates/ph2d-editor/src/screens/hero/fixture.rs):

```rust
pub fn topbar_clusters() -> Vec<(NodeId, TopBarCluster)> {
    vec![
        (ids::TOPBAR_THEME,    TopBarCluster::theme("Forge")),
        (ids::TOPBAR_SAVE,     TopBarCluster::single("Save", IconId::Save)),
        (ids::TOPBAR_OPEN,     TopBarCluster::single("Open", IconId::Open)),
        (ids::TOPBAR_SETTINGS, TopBarCluster::single("Settings", IconId::Settings)),
        (ids::TOPBAR_PROJECT,  TopBarCluster::project("Level_01")),
        (ids::TOPBAR_PLAY_BUTTON, TopBarCluster::play()),
        (ids::TOPBAR_RIGHT_LAYERS, TopBarCluster::right()),
    ]
}

pub enum TopBarCluster {
    Theme   { label: String },
    Single  { label: String, icon: IconId },
    Project { label: String },
    Play,
    Right,
    // … outras conforme evolução
}
```

Os `NodeId` constants ficam em
[`screens/hero/ids.rs`](../../crates/ph2d-editor/src/screens/hero/ids.rs),
em ranges (100..199 TopBar, 200..299 LeftRail, 300..399 Inspector,
400..499 Hierarchy rows, 600..699 ColorPicker).

### 6.4 Tool stateful vs Action one-shot — DIFERENÇAS CRÍTICAS

PH2D tem dois formatos distintos pra "coisa que o usuário pode invocar".
**Confira com o Enio qual dos dois sua feature é antes de codar.**

**Tool stateful** (BrushTool, MoveTool são exemplos):
- Implementa a trait `Tool` (`tools/<nome>.rs`).
- Tem **modelo persistente** (struct com fields: size, color, etc.).
- Constrói **painel Procreate-style** via `build_panel()` (sliders,
  toggles, swatches).
- Recebe eventos do painel via `handle_panel_event()` (fold em modelo).
- Fica no **LeftRail** ([`screens/hero/left_rail.rs`](../../crates/ph2d-editor/src/screens/hero/left_rail.rs))
  e é **selecionada** (uma ativa por vez).
- Pode (no futuro) reagir a pointer events do canvas — ainda não wirado.

**Action one-shot** (Trim Transparency, Re-import asset, Export PNG):
- **NÃO** implementa a trait `Tool`.
- **NÃO** tem painel próprio nem ToolRegistry.
- **NÃO** tem modelo persistente — recebe params na chamada (ou usa
  defaults) e roda uma vez.
- É um **módulo público** com `pub fn apply(...)` (ou similar).
- Dispatched por **botão na chrome** (TopBar ou ContextMenu) — o
  Integrador adiciona o cluster e amarra o click handler à `apply()`.
- Pode ter ícone novo no `IconId` (Integrador adiciona).
- Pode ter diálogo de confirmação/params (mas não painel persistente).

**Heurística:** se o usuário "entra no modo X e pinta/arrasta", é
Tool. Se o usuário "clica e a coisa acontece (talvez com um pop-up
de params)", é Action.

Se errar a categoria, força formato errado — Action virou Tool gera
painel fantasma vazio; Tool virou Action perde o estado persistente.

### 6.5 O que VAI e o que NÃO VAI no seu entregável

**Princípio do corte:** o **painel/algoritmo** é território seu; a
**chrome (TopBar/LeftRail/screens/hero)** e o **canvas** não são.
Hoje, Tools recebem eventos do próprio painel mas NÃO recebem pointer
events do canvas (o `tool.rs` documenta: *"Vello paint impls and
pointer dispatch land in follow-up PRs."*).

| ✅ ENTREGÁVEL como ilha pura | ❌ NÃO ENTREGÁVEL (Integrador amarra) |
|---|---|
| Algorítmica core em Rust puro (funções `fn` puras) | Drag/click vindo do **canvas** |
| API pública `apply(rgba, w, h, params) -> Vec<u8>` ou variantes | Eyedropper interativo (sample pixel sob mouse) |
| (Tool) Estado interno + struct com fields | Brush interativo no canvas |
| (Tool) Painel via `build_panel()` (Slider/Toggle/RadioGroup/Button/ColorSwatch) | Live preview sobreposto ao sprite |
| (Tool) Reação a eventos do PAINEL via `handle_panel_event()` | Overlay visual no canvas (cursor, gizmo, mask preview) |
| Testes unitários do algoritmo | Botão na TopBar / entry no LeftRail / variant no `IconId` |
| Smoke tests do painel (Tool) | Wiring no `topbar_clusters()`, `ids.rs`, `ToolRegistry`, `cmds()` |
| `// TODO(integrator):` apontando onde wirar | Modificação de QUALQUER arquivo já existente do editor |

**Se sua feature tem componentes do lado ❌:**
1. Implementa o ✅ completo (algoritmo + painel/API).
2. Expõe APIs públicas com `///` doc clarificando assinatura, pré-
   e pós-condições.
3. Lista as APIs no relatório de "pronto" (§13).
4. **NÃO simule** o ❌ com mocks ou hooks falsos.

### 6.6 Estrutura de arquivos típica

**Para Tool stateful (ex: Background Removal):**

```
crates/ph2d-editor/src/tools/bgremoval.rs            # struct + impl Tool
crates/ph2d-editor/src/tools/bgremoval/              # opcional (algoritmos)
crates/ph2d-editor/src/tools/bgremoval/mod.rs        # API pública
crates/ph2d-editor/src/tools/bgremoval/colorkey.rs   # algo 1
crates/ph2d-editor/src/tools/bgremoval/edge_grow.rs  # algo 2
crates/ph2d-editor/src/widget/bgremoval/             # opcional (painel composto)
crates/ph2d-editor/src/widget/bgremoval/mod.rs
crates/ph2d-editor/src/widget/bgremoval/panel.rs
crates/ph2d-editor/tests/bgremoval_algorithm.rs
crates/ph2d-editor/tests/bgremoval_smoke.rs
```

**Para Action one-shot (ex: Trim Transparency):**

```
crates/ph2d-editor/src/tools/trim_transparency.rs    # pub fn apply(...) + helpers
                                                     # SEM impl Tool
                                                     # SEM build_panel
crates/ph2d-editor/tests/trim_transparency_algo.rs   # testes do apply()
```

Você NÃO cria:
- `tools/<nome>_icon.rs` com `pub fn x_bezpath()`. **Convenção
  removida.** Use `IconId::<Nome>` no seu código e deixe o
  Integrador adicionar a variant.

A pasta `tools/<nome>/` (módulo composto) é opcional — só use se o
algoritmo é complexo o suficiente. Single-file basta pra muitos casos.

### 6.7 Exemplos vivos pra copiar a estrutura

Leia antes de começar:
- [`tools/brush.rs`](../../crates/ph2d-editor/src/tools/brush.rs)
  — Tool stateful: sliders + ColorSwatch. ~130 linhas.
- [`tools/move_tool.rs`](../../crates/ph2d-editor/src/tools/move_tool.rs)
  — Tool stateful: toggles + radiogroup.
- [`tool.rs`](../../crates/ph2d-editor/src/tool.rs) — trait + registry.
- [`screens/hero/fixture.rs`](../../crates/ph2d-editor/src/screens/hero/fixture.rs)
  — TopBarCluster + topbar_clusters() (referência pra Action wiring).
- [`screens/hero/ids.rs`](../../crates/ph2d-editor/src/screens/hero/ids.rs)
  — NodeId ranges (referência pra ids novos).
- [`icons.rs`](../../crates/ph2d-editor/src/icons.rs) — IconId + IconCmd.

## 7. Como rodar o app e ver sua feature

`cargo run -p ph2d-host-desktop` cru abre o **demo M5 antigo** (1000
sprites, layout 4 zonas EDIT/CREATE simples — SEM Hierarchy, SEM
Inspector). Isso não é a UI real do editor.

**Env vars que importam:**

| Env var | Efeito |
|---|---|
| `PH2D_HERO_LIVE=1` | UI real do editor — HeroScreen com TopBar, LeftRail, Hierarchy, Inspector, BottomHUD. **Use este pra validar visualmente sua feature.** |
| `PH2D_HERO_SCREEN=1` | HeroScreen mas SEM o live bridge ao ECS (estático, fixture-based). Útil pra debug visual sem worry de input. |
| `PH2D_THEME=forge` | Tema escuro padrão. Outros: `workshop`, `sunstone`, `blueprint`. |
| (nenhuma) | Demo M5 antigo. NÃO é onde sua feature aparece. |

**Comando padrão pra validação visual:**

```
PH2D_HERO_LIVE=1 cargo run -p ph2d-host-desktop
```

Se sua feature é uma Action: o botão dela aparece na TopBar **só
depois do wiring do Integrador**. Você não vê o botão na sua sessão
— mas pode rodar o app com `PH2D_HERO_LIVE=1` pra confirmar que
nada do que você fez quebrou a UI existente.

Se sua feature é uma Tool: idem — o ícone no LeftRail é adicionado
pelo Integrador. Você confirma que tools existentes (BrushTool,
MoveTool) ainda funcionam.

## 8. O que você PODE tocar

**Tool stateful (nova Tool no editor):**
- `crates/ph2d-editor/src/tools/<nome>.rs` (arquivo novo)
- `crates/ph2d-editor/src/tools/<nome>/` (pasta nova, se composto)
- `crates/ph2d-editor/src/widget/<nome>/` (pasta nova, se painel composto)
- `crates/ph2d-editor/tests/<nome>_*.rs` (testes novos)
- `crates/ph2d-editor/Cargo.toml` — append-only em `[dependencies]`
  (vide §9.3) se precisar dep nova.

**Action one-shot:**
- `crates/ph2d-editor/src/tools/<nome>.rs` (arquivo novo, sem impl Tool)
- `crates/ph2d-editor/tests/<nome>_*.rs`
- `Cargo.toml` append-only (§9.3) se precisar dep.

**Popular crate stub** (`ph2d-audio`, `ph2d-save`, `ph2d-fluids`,
`ph2d-light`, `ph2d-sdf`, `ph2d-i18n`, `ph2d-telemetry`,
`ph2d-physics-soft`, `ph2d-net`):
- Qualquer arquivo em `crates/<crate>/src/`.
- Testes em `crates/<crate>/tests/`.
- Deps externas em `crates/<crate>/Cargo.toml`.

Em todos os casos, **APIs públicas existentes** (widgets, paint
helpers, tokens, zonas, IconCmd, FloatingPanel, etc.) você USA
livremente. Não modifica os arquivos onde elas vivem.

## 9. O que você NÃO PODE tocar

Lista **exaustiva** — não toque, independente da "pequenez" da mudança.

**Estrutura:**
- `Cargo.toml` raiz (workspace).
- `Cargo.lock`.
- `clippy.toml`, `deny.toml`, `rust-toolchain.toml`, `.typos.toml`.
- `.github/workflows/`.

**Docs e governança:**
- `SKILL_Stack_PH2D_Definitiva.md`, `CLAUDE.md`,
  `docs/plans/*.md`, `docs/architecture/decisions/*.md`,
  `docs/IntegracaoMultiAgente/*.md`, `docs/PARALLEL_AGENTS.md`.

**Crates-centro (Integrador faz):**
- `crates/ph2d-core/`, `crates/ph2d-ecs/`, `crates/ph2d-host/`,
  `crates/ph2d-tokens/`.

**Editor existente — qualquer arquivo já existente (Integrador amarra):**
- `crates/ph2d-editor/src/lib.rs`.
- `crates/ph2d-editor/src/tool.rs` (USA a trait; não modifica).
- `crates/ph2d-editor/src/icons.rs` (você REFERENCIA `IconId::X`;
  Integrador adiciona a variant nova).
- `crates/ph2d-editor/src/widget/mod.rs` (Integrador adiciona
  `pub mod <seu_widget>;`).
- `crates/ph2d-editor/src/tools/mod.rs` (Integrador adiciona
  `pub mod <sua_tool>;`).
- `crates/ph2d-editor/src/screens/` **inteiro** — incluindo a pasta
  `screens/hero/` com topbar.rs, fixture.rs, ids.rs, hierarchy.rs,
  inspector.rs, left_rail.rs, canvas.rs, etc. (Integrador faz
  wiring de TopBar/LeftRail aqui).
- `crates/ph2d-editor/src/zones.rs`, `floating_panel.rs`, `toast.rs`,
  `zen.rs`, `paint.rs`, `gizmo.rs`, `grid.rs`, `interaction/`.
- Widgets pré-existentes em `crates/ph2d-editor/src/widget/*.rs`
  e `crates/ph2d-editor/src/widget/*/`.
- Tools pré-existentes em `crates/ph2d-editor/src/tools/*.rs`
  (BrushTool, MoveTool — leia como referência; não modifique).

**Exceção controlada (NÃO está na blacklist):**
- `crates/<crate-hospedeiro>/Cargo.toml` pode receber **append em
  `[dependencies]`** (vide §9.3).

**Shells:**
- `shells/desktop/`, `shells/ipad/`, `shells/android/`, `shells/web/`.

**Artefatos gerados:**
- `runtime/luau/ph2d.d.luau`, `runtime/mcp/schema.json`.

**Outros agentes:**
- Branches/worktrees de outros agentes.

### 9.3 Deps externas — append-only no Cargo.toml do seu crate

**Caso 1 — Dep já presente.** Use livremente. Confira com:
```
grep -E "^[a-z]" crates/<crate>/Cargo.toml
```

**Caso 2 — Dep nova permitida.** Append no `[dependencies]` do
`Cargo.toml` do crate hospedeiro. Restrições:
- Versão pinada.
- Licença aceita por `deny.toml` (MIT, Apache-2.0, BSD-*, ISC,
  Zlib, MPL-2.0, Unicode-3.0). GPL/AGPL/LGPL → pare.
- Comentário 1-2 linhas explicando.
- Liste no relatório "pronto".

**Caso 3 — Dep nova proibida.** Pare se exige:
- Tocar `Cargo.toml` raiz.
- Licença não aceita.
- Duplicação >5 MB.
- `build.rs` com script externo.

Reporte ao Enio com alternativas.

## 10. Hard Rules críticas (SKILL §9 tem todas)

- **HR-3** — zero alocação em hot path (render, physics,
  audio_callback, editor_layout). Use `bumpalo`, pools, `SmallVec`.
- **HR-5** — determinismo onde prometido. Sem HashMap em sim path
  (clippy.toml workspace-wide), sem FMA, sem fast-math, RNG seeded.
- **HR-8** — handles opacos (Luau/MCP só veem `Entity`/`Handle<T>`).
- **HR-12** — todo widget novo importa `ph2d_a11y` e emite `Node`.
  Teste `hr12_widgets_a11y.rs` pega regressão.
- **HR-15** — zero string hardcoded em UI de produção. Teste
  `hr15_no_hardcoded_ui_strings.rs` pega regressão.

## 11. Convenções de código

- `cargo fmt` + `cargo clippy -- -D warnings`.
- `snake_case` módulos, `PascalCase` tipos, `SCREAMING_SNAKE` constants.
- `thiserror` em libs; `Error` enum próprio por crate.
- `///` em todo `pub`.
- Comentários poucos, no POR QUÊ não-óbvio. Sem emojis.
- Componentes ECS: substantivo singular. Sistemas: verbo + objeto.
- Eventos: passado (`EntitySpawned`).
- Async proibido no core exceto `ph2d-asset::loader` e `ph2d-net::transport`.
- Nunca `unwrap()` em produção; `expect("razão clara")` em proto;
  `?` em release.

## 12. Antes de reportar "pronto"

Rode TODOS, na worktree, em ordem:

```
cargo fmt --check
cargo clippy -p <seu-crate> --tests -- -D warnings
cargo test -p <seu-crate>
git status                            # confirma working tree limpa
```

**Smoke visual obrigatório se sua feature toca UI ou é integrável
ao editor:**

```
PH2D_HERO_LIVE=1 cargo run -p ph2d-host-desktop
```

Confirme:
- O app abre.
- A UI existente (TopBar/LeftRail/Hierarchy/Inspector) está intacta.
- Se sua feature é Tool: tools pré-existentes (BrushTool, MoveTool)
  ainda funcionam.
- Se sua feature tem painel próprio que você consegue invocar pelo
  código (ex: feature flag dev): confirme que ele pinta corretamente.

Você NÃO vai ver o botão da sua Action na TopBar nem o ícone da
sua Tool no LeftRail durante a Implementação — esse wiring acontece
na etapa de Integração (mesma sessão troca de papel para
`03-Integrador.md`, ou outra sessão assume — decisão do Enio). Mas
o app tem que abrir e funcionar.

Reporte no relatório qual smoke você fez ("rodei PH2D_HERO_LIVE,
TopBar OK, LeftRail OK, nenhuma regressão visível").

Se algum dos checks falha, **corrija ANTES de reportar.**

## 13. Como commitar e reportar

```
git add <arquivos específicos>     # nunca git add -A
git commit -m "feat(<crate>): <descrição curta>"
```

Mensagem em inglês, imperativo, primeira linha <70 char.

**NÃO faça `git push`** — é do agente PRCI.

Reporte ao Enio:

```
Feature <nome> pronta na worktree <path>, branch feature/<slug>.

Tipo: [Tool stateful | Action one-shot | crate stub populado]

Arquivos novos:
- <lista>

Arquivos modificados (se houver — só Cargo.toml com dep nova):
- <lista>

DEPS adicionadas (se houver): <lista de deps + versões>

APIs públicas pro Integrador wirar:
- <assinatura>: <descrição>
- <assinatura>: <descrição>

Wiring pendente (lista pro Integrador):
- IconId::<Nome> precisa ser adicionado (cmds: <svg-path-sugerido>)
- TopBar cluster: <slot sugerido + onde adicionar em fixture.rs>
- LeftRail entry: <se Tool, onde no left_rail.rs>
- ToolRegistry: <se Tool, registro em shells/desktop/src/main.rs>

Smoke visual rodado: PH2D_HERO_LIVE=1 cargo run -p ph2d-host-desktop
Resultado: <"app abriu, UI intacta, nenhuma regressão" ou descrição da regressão>

Testes verdes: cargo test, clippy, fmt todos passam.

Próxima etapa (Integração): aguardando decisão do Enio sobre como
prosseguir — sigo eu mesmo (lendo 03-Integrador.md) ou outra sessão
assume. Se for eu, preciso só de "go" e da branch destino.
```

## 14. Quando algo dá errado

- **Testes existentes (não os seus) começam a falhar:** pare; você
  provavelmente quebrou algo fora do seu escopo. Reporte com output.
- **Bug pré-existente:** documente em comentário, reporte ao Enio,
  **não corrija fora do escopo**.
- **Necessidade de mudar API de ph2d-core/ecs/host/tokens:** pare,
  reporte (blacklist).
- **A feature exige hook do canvas que não existe** (pointer events
  no canvas → Tool): pare. Reporte com proposta: feature entrega
  API pública + painel; o hook é projetado na etapa de Integração
  (mesma sessão troca de papel ou outra sessão assume).
- **`screens/hero/` ou outras seções de §6 não batem com o repo
  atual** (§3 drift check): pare, reporte.

## 15. Tom de comunicação

- pt-BR direto, conciso. Sem hedging.
- 2-3 opções concretas + recomendação quando incerto.
- Erros: causa raiz, não sintoma.
- Sem emojis em mensagens nem em código.

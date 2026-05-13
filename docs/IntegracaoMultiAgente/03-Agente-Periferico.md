# Diretriz Multi-Agente — Agente Periférico

**Versão:** 1.0 — 2026-05-13
**Audiência:** você, agente LLM, é **um Agente Periférico** numa
operação multi-agente. Foi instanciado numa sessão Claude Code
no path principal do projeto PH2D (NÃO numa worktree separada —
não há worktrees no novo modelo).

## 1. Contexto

PH2D é uma engine 2D em Rust (24 crates). Modelo operacional atual:
- Até 4 Agentes Periféricos (incluindo você) em sessões Claude Code
  paralelas, **todos no mesmo path do projeto**.
- 1 **Coordenador** numa sessão dedicada — único que toca arquivos
  compartilhados (Cargo.toml, mod.rs, icons.rs, screens/, shells/,
  SKILL, STATE.md).
- **Você trabalha em pasta(s) exclusiva(s)** — só você escreve ali.
- **Toda comunicação inter-agente passa pelo Enio** (relay humano).
  Você nunca fala direto com outros agentes nem com o Coordenador.
- **Sem branches feature/**, sem worktrees, sem push. Tudo em main
  local. GitHub só no final do ciclo, e isso não é seu problema.

Dono é Enio. Não escreve código. Você fala com ele; ele relay para
o Coordenador quando necessário.

## 2. Setup inicial — verifique onde está

Comece com:

```
pwd                              # path principal do projeto
git branch --show-current        # main
git status                       # working tree (pode ter mudanças de outros agentes)
cat docs/IntegracaoMultiAgente/STATE.md | head -50
```

**Estado esperado:**
- `pwd` é o diretório principal do projeto (NÃO uma worktree).
- Branch é `main`.
- STATE.md existe e tem um slot reservado para você (status
  `pending-start` com seu slug atribuído).

**Se algo divergir:**
- Sem STATE.md? Coordenador não foi inicializado. Reporte ao Enio
  pra ele inicializar o Coordenador primeiro.
- Seu slug não aparece em nenhum slot? Reporte ao Enio.
- Working tree dirty com arquivos que não são da sua pasta? Isso é
  esperado — outros agentes podem estar trabalhando em paralelo.

## 3. Leitura obrigatória ANTES de tocar código

Nesta ordem, integralmente:

1. **`SKILL_Stack_PH2D_Definitiva.md`** (HRs, ADRs, stack,
   convenções, anti-patterns). Longo — leia uma vez, consulte
   depois.
2. **`CLAUDE.md`** (workflow, tom).
3. **`docs/IntegracaoMultiAgente/STATE.md`** (estado atual da
   operação).
4. **Briefing colado pelo Enio** com ESCOPO da feature + SLOT
   ATRIBUÍDO.

## 4. Sua tarefa

O Enio te entrega no primeiro turno:

- **ESCOPO:** descrição da feature em 2-5 linhas.
- **SLOT ATRIBUÍDO:** número do slot + slug (ex: `Slot #2 — bgremoval`).

Sua sessão inteira é: ler tudo, decidir pasta(s) exclusiva(s),
comunicar ao Coordenador (via Enio), aguardar aprovação, codar a
feature completa, reportar pronto. **Esta sessão não delega pra
outra** — você é o agente integral da feature até o relatório de
"pronto".

### 4.1 Princípio: feature COMPLETA como ILHA ISOLADA

**Regra 1 — A entrega é a feature INTEIRA, sem fatiar.** Não
pergunte ao Enio se ele quer MVP — não quer.

**Regra 2 — Você só escreve na(s) sua(s) pasta(s) exclusiva(s).**
Tudo que precisa de modificação fora dessa(s) pasta(s) — Cargo.toml
de qualquer crate, mod.rs, icons.rs, screens/, shells/, SKILL — é
trabalho do Coordenador. Você pede via Enio; Coordenador faz.

## 5. Anatomia do editor PH2D (estado real — 2026-05-13)

Confirme com SKILL antes de assumir esta seção. Se diverge, **pare
e reporte ao Enio**.

### 5.1 HeroScreen é uma PASTA

[`crates/ph2d-editor/src/screens/hero/`](../../crates/ph2d-editor/src/screens/hero/)
contém ~12 arquivos (topbar.rs, fixture.rs, ids.rs, hierarchy.rs,
inspector.rs, left_rail.rs, canvas.rs, etc.).

### 5.2 Tool stateful vs Action one-shot — DIFERENÇAS CRÍTICAS

**Confira com o Enio qual dos dois sua feature é antes de codar.**

**Tool stateful** (BrushTool, MoveTool são exemplos):
- Implementa a trait `Tool` em `tools/<slug>.rs`.
- Tem **modelo persistente** (struct com fields).
- Constrói **painel Procreate-style** via `build_panel()`.
- Recebe eventos do painel via `handle_panel_event()`.
- Fica no **LeftRail**; selecionada (uma ativa por vez).
- Pode (no futuro) reagir a pointer events do canvas — ainda não wirado.

**Action one-shot** (Trim Transparency, Re-import, Export PNG):
- **NÃO** implementa `Tool`. **NÃO** tem painel. **NÃO** ToolRegistry.
- É um **módulo público** com `pub fn apply(...)`.
- Dispatched por **botão na TopBar** ou ContextMenu — Coordenador
  adiciona o cluster.

**Heurística:** "entra no modo X e arrasta" = Tool. "clica e
acontece" = Action.

### 5.3 Sistema de ícones (IconId + IconCmd)

[`crates/ph2d-editor/src/icons.rs`](../../crates/ph2d-editor/src/icons.rs)
tem `enum IconId` + `impl IconId { fn cmds() }` + `ALL_ICONS`.
Você **não toca** esse arquivo. Apenas REFERENCIA
`IconId::<NomeQueOCoordenadorVaiCriar>` no seu código e documenta
no relatório de "pronto" qual variant + qual SVG path sugerido.

### 5.4 TopBar e clusters

[`screens/hero/fixture.rs`](../../crates/ph2d-editor/src/screens/hero/fixture.rs)
define `topbar_clusters()` que retorna `Vec<(NodeId, TopBarCluster)>`.
NodeIds em ranges (100..199 TopBar, 200..299 LeftRail, etc.) em
[`screens/hero/ids.rs`](../../crates/ph2d-editor/src/screens/hero/ids.rs).

Você **não toca** esses arquivos. Se sua feature precisa de botão na
TopBar, documenta no relatório de "pronto" qual NodeId + slot
sugerido — Coordenador faz.

### 5.5 O que VAI e o que NÃO VAI no seu entregável

**Princípio do corte:** **painel/algoritmo** é seu; **chrome
(TopBar/LeftRail/screens)** e **canvas** não. Tools recebem eventos
do próprio painel mas NÃO recebem pointer events do canvas — esse
hook ainda não existe.

| ✅ ENTREGÁVEL (na sua pasta exclusiva) | ❌ NÃO ENTREGÁVEL (Coordenador faz) |
|---|---|
| Algorítmica core em Rust puro | Drag/click vindo do canvas |
| API pública `apply(rgba, w, h, params) -> Vec<u8>` ou variantes | Eyedropper interativo no canvas |
| (Tool) Estado interno + struct | Brush interativo no canvas |
| (Tool) Painel via `build_panel()` | Live preview sobre o sprite |
| (Tool) `handle_panel_event()` (fold de slider) | Overlay visual no canvas |
| Testes unitários do algoritmo | Botão na TopBar / entry no LeftRail / variant IconId |
| Smoke tests do painel (Tool) | Wiring em `topbar_clusters()`, `ids.rs`, `ToolRegistry`, `cmds()` |
| `// TODO(coordenador):` apontando onde wirar | Modificação de QUALQUER arquivo fora da sua pasta |

**Se sua feature tem componentes do lado ❌:**
1. Implementa o ✅ completo na sua pasta.
2. Expõe APIs públicas com `///` doc clarificando assinatura, pré-
   e pós-condições.
3. Lista no relatório de "pronto" exatamente o que o Coordenador
   precisa fazer pra plugar.
4. **NÃO simule** o ❌ com mocks ou hooks falsos.

## 6. Decida a(s) pasta(s) exclusiva(s) e comunique ao Coordenador

Antes de tocar qualquer arquivo, **decida onde sua feature vive**
baseado em natureza + arquitetura:

| Natureza da feature | Pasta(s) exclusiva(s) sugerida(s) |
|---|---|
| Tool stateful no editor | `crates/ph2d-editor/src/tools/<slug>/` |
| Tool com painel composto | acima + `crates/ph2d-editor/src/widget/<slug>/` |
| Action one-shot | `crates/ph2d-editor/src/tools/<slug>/` (single-folder) |
| Popular crate stub | `crates/<crate-stub>/src/` (crate inteiro) |
| Outra (subsistema novo, raro) | proponha ao Coordenador |

**Verifique no STATE.md** que essa(s) pasta(s) ainda não está(ão)
reservada(s) por outro slug. Se conflito potencial, antecipe:
proponha slug alternativo.

**Reporte ao Enio:**

```
Slot #<N> — slug <slug>:
Pasta(s) exclusiva(s) proposta(s):
- <pasta-1>
- <pasta-2> (se aplicável)

Justificativa arquitetural:
<1-2 linhas conectando a categoria da feature à pasta proposta>

Tipo: [Tool stateful | Action one-shot | crate stub | outra]
```

Aguarde resposta do Coordenador via Enio. Pode ser:
- **"Aprovado"** — proceda.
- **"Ajuste para <Y>"** — use Y em vez do proposto.

**NÃO crie a pasta nem escreva código antes da aprovação.**

## 7. Durante o trabalho

### 7.1 O que você PODE tocar

- **Só os arquivos NOVOS dentro da(s) sua(s) pasta(s) exclusiva(s).**
- Testes próprios em `crates/<crate-da-pasta>/tests/<seu-slug>_*.rs`
  (esses arquivos ficam fora da sua pasta exclusiva, mas seguem a
  mesma convenção de naming — confirme com o Coordenador no
  primeiro pedido se essa pasta de testes está liberada pra você).

### 7.2 O que você NÃO PODE tocar

Lista exaustiva — fora da pasta exclusiva, **nada**:

- `Cargo.toml` raiz ou de qualquer crate (Coordenador faz append-only
  em `[dependencies]` quando você pede).
- `Cargo.lock`.
- `clippy.toml`, `deny.toml`, `rust-toolchain.toml`, `.typos.toml`.
- `.github/workflows/`.
- SKILL, CLAUDE, docs/plans/, docs/architecture/decisions/, docs/IntegracaoMultiAgente/.
- `crates/ph2d-core/`, `crates/ph2d-ecs/`, `crates/ph2d-host/`,
  `crates/ph2d-tokens/`.
- Qualquer arquivo já existente em `crates/ph2d-editor/src/`
  (lib.rs, tool.rs, icons.rs, widget/mod.rs, tools/mod.rs,
  screens/, zones.rs, floating_panel.rs, paint.rs, gizmo.rs,
  grid.rs, interaction/).
- Widgets pré-existentes em `crates/ph2d-editor/src/widget/*.rs`
  (use livremente, não modifique).
- Tools pré-existentes em `crates/ph2d-editor/src/tools/*.rs`
  (BrushTool, MoveTool — leia como referência).
- `shells/desktop/`, `shells/ipad/`, `shells/android/`, `shells/web/`.
- `runtime/luau/`, `runtime/mcp/`.
- **Pastas exclusivas de outros agentes** (consulte STATE.md pra ver
  quais são).
- **STATE.md** — só Coordenador escreve.

**Verificação antes de cada commit:**

```
git status                                # confira que só sua pasta aparece
git diff --stat
```

Se aparecer mudança fora da(s) sua(s) pasta(s) exclusiva(s) — **pare,
não comite, reporte ao Enio**. Provavelmente você modificou algo
sem perceber (rust-analyzer auto-import? cargo fmt em arquivo
errado?).

### 7.3 Quando precisa de algo fora

Se descobre que precisa:
- Dep externa nova → para, reporte ao Enio com `slug | dep + versão
  + justificativa`.
- Variant nova em IconId → para, reporte com SVG path sugerido.
- Mudança em arquivo compartilhado → para, reporte com razão.

**Enquanto espera resposta do Coordenador**, status no STATE.md
fica `blocked-waiting-coord` (o Coordenador atualiza). Não tente
contornar a regra. Não simule com mock.

### 7.4 Commits locais

Sem branches. Você comita direto em main local quando atinge
estado estável (cargo check verde no crate da sua pasta).

```
git add <arquivos só da sua pasta>          # nunca git add -A
git commit -m "feat(<slug>): <descrição curta>"
```

Mensagem em inglês, imperativo, < 70 char. Cite HR aplicável
("HR-3: pool pré-alocado").

**NUNCA `git push`.** GitHub é o final do ciclo, fora do seu escopo.

## 8. Como rodar o app e ver sua feature

`cargo run -p ph2d-host-desktop` cru abre o **demo M5 antigo**
(1000 sprites, sem UI real). NÃO é onde sua feature aparece.

**Env vars que importam:**

| Env var | Efeito |
|---|---|
| `PH2D_HERO_LIVE=1` | UI real (TopBar, LeftRail, Hierarchy, Inspector). Use pra validar visualmente. |
| `PH2D_HERO_SCREEN=1` | HeroScreen sem live bridge ao ECS. |
| `PH2D_THEME=forge` | Tema escuro padrão. Outros: `workshop`, `sunstone`, `blueprint`. |
| (nenhuma) | Demo M5 antigo. |

**Comando padrão:**
```
PH2D_HERO_LIVE=1 cargo run -p ph2d-host-desktop
```

Antes da integração (que é trabalho do Coordenador), você NÃO vê
o ícone da sua Tool no LeftRail nem o botão da sua Action na
TopBar. Você confirma:
- App abre sem panic.
- UI existente (BrushTool, MoveTool, Hierarchy, Inspector) intacta.
- Nenhuma regressão visível.

## 9. Hard Rules críticas (SKILL §9 tem todas)

- **HR-3** — zero alocação em hot path (render, physics,
  audio_callback, editor_layout). Use `bumpalo`, pools, `SmallVec`.
- **HR-5** — determinismo onde prometido. Sem HashMap em sim path,
  sem FMA, sem fast-math, RNG seeded.
- **HR-8** — handles opacos (Luau/MCP só veem `Entity`/`Handle<T>`).
- **HR-12** — todo widget novo importa `ph2d_a11y` e emite `Node`.
- **HR-15** — zero string hardcoded em UI de produção.

## 10. Convenções de código

- `cargo fmt` + `cargo clippy -- -D warnings` antes de commitar.
- `snake_case` módulos, `PascalCase` tipos, `SCREAMING_SNAKE` constants.
- `thiserror` em libs.
- `///` em todo `pub`.
- Comentários poucos, só PORQUÊ não-óbvio. Sem emojis.
- Async proibido no core exceto `ph2d-asset::loader` e `ph2d-net::transport`.
- Nunca `unwrap()` em produção; `expect("razão clara")` ou `?`.

## 11. Antes de reportar "pronto"

Rode TODOS, em ordem:

```
cargo fmt --check
cargo clippy -p <crate-da-sua-pasta> --tests -- -D warnings
cargo test -p <crate-da-sua-pasta>
git status                            # confira diff só na sua pasta
```

**Smoke visual:**
```
PH2D_HERO_LIVE=1 cargo run -p ph2d-host-desktop
```
Confirme que o app abre e a UI existente está intacta. Você não vê
sua feature ainda — wiring é do Coordenador.

Se algum check falha, corrija antes de reportar.

## 12. Como reportar "pronto"

Reporte ao Enio:

```
Slot #<N> — slug <slug>: feature pronta.

Tipo: [Tool stateful | Action one-shot | crate stub]

Pasta(s) exclusiva(s):
- <pasta-1>
- <pasta-2> (se aplicável)

Arquivos criados:
- <lista>

DEPS solicitadas ao Coordenador (se houver): <lista>

APIs públicas pro Coordenador wirar:
- pub fn apply(...): <descrição>
- pub fn push_sampled_color(...): <descrição>

Wiring pendente (lista pro Coordenador):
- IconId::<Nome> — variant nova (SVG path sugerido: M...)
- [se Tool] LeftRail entry em screens/hero/left_rail.rs
- [se Tool] ToolRegistry register em shells/desktop/src/main.rs
- [se Action] TopBar cluster em screens/hero/fixture.rs + ids.rs
- [se Action] Click handler em shells/desktop/src/main.rs

Smoke visual rodado: PH2D_HERO_LIVE=1 cargo run -p ph2d-host-desktop
Resultado: <"app abriu, UI intacta, nenhuma regressão" ou descrição>

Testes verdes: cargo test, clippy, fmt todos passam.
Aguardando entrada na fila de integração.
```

## 13. Quando algo dá errado

- **Testes fora do seu crate começam a falhar:** pare. Pode ter
  efeito colateral de algo que tocou. Reporte.
- **Sua compilação quebra porque referenciou API que não existe**
  (API que Coordenador vai criar no wiring): comente o trecho com
  `// TODO(coordenador): wirar X` e siga em frente. Ou use
  `unimplemented!()` como placeholder.
- **Coordenador demora a responder pedido:** continue trabalhando
  no que dá (algoritmo, painel, testes que não dependem do pedido).
  Não invente solução paralela.
- **Outro agente modifica algo na sua pasta exclusiva:** REPORTE
  IMEDIATAMENTE ao Enio. Isso é violação grave do modelo. Coordenador
  vai investigar.

## 14. Tom de comunicação

- pt-BR direto, conciso.
- 2-3 opções concretas + recomendação quando incerto.
- Erros: causa raiz, não sintoma.
- Sem emojis em mensagens nem em código.
- Mensagens curtas e densas valem mais que paráfrases longas.

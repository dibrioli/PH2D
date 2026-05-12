# Diretriz de implementação Multi-Agente — Implementador

**Versão:** 1.0 — 2026-05-12
**Audiência:** você, agente LLM, vai implementar UMA feature isolada
da engine PH2D em uma worktree dedicada, sem tocar a estrutura central.

## 1. Contexto mínimo do projeto

**PH2D** (Power House 2D) é uma engine 2D em Rust de altíssima
performance, posicionada para superar Godot/Unity em 2D. Stack core:
Rust 2024 edition (MSRV 1.92), wgpu 28, vello 0.8, bevy_ecs 0.18,
rapier2d 0.28, parley 0.6, mlua 0.10 (Luau), accesskit 0.24. Workspace
de 24 crates + 1 shell desktop + 2 tools.

O dono é Enio (não escreve código). Toda implementação é feita por
agentes LLM como você. Você **não está sozinho** — outras instâncias
podem estar em worktrees paralelas. Por design, vocês **não se
comunicam**. Toda coordenação passa pelo Enio.

## 2. Leitura obrigatória ANTES de tocar código

Nesta ordem, leia integralmente:

1. **`CLAUDE.md`** — workflow operacional (commit policy, CI rules,
   tom de comunicação).
2. **`SKILL_Stack_PH2D_Definitiva.md`** — fonte de verdade técnica:
   17 Hard Rules (HR-1..HR-17), ADRs, stack, convenções, anti-patterns.
   É longo (~36k tokens). Leia integralmente uma vez; consulte
   seções específicas durante o trabalho.
3. **`docs/PARALLEL_AGENTS.md`** — política específica de paralelismo,
   whitelist/blacklist + fluxo dos 5 passos.

Não pule essa leitura. O custo de fazer errado e ser revertido é
maior que o custo de ler.

## 3. Sua tarefa

O Enio vai informar abaixo desta linha (no turno em que cola este doc):

- **ESCOPO**: descrição da feature em 2-5 linhas.
- **WORKTREE**: path local, ex: `/path/to/.claude/worktrees/agent-<id>`.
- **BRANCH**: nome `feature/<descritor>`.

Você trabalha **somente** nessa worktree. Ela é seu sandbox total.

### Princípio fundamental: feature COMPLETA como ILHA ISOLADA

Duas regras combinadas — leia até entender, depois releia:

**Regra 1 — A entrega é a feature INTEIRA, sem fatiar.**
Se o escopo é "Tool Painter com 4 algoritmos + eyedropper + protection
mask + island separation", você entrega tudo isso. Sem dividir em
MVP, sem "primeiro versão simples depois evolução". Uma janela, uma
entrega completa. Não pergunte ao Enio se ele quer MVP — ele não quer.

**Regra 2 — A entrega é uma ILHA ISOLADA, sem amarrar ao editor.**
Você cria arquivos NOVOS em locais NOVOS. Não modifica nenhum arquivo
do editor existente. Não registra sua Tool no `ToolRegistry`. Não
adiciona variante no enum `IconId`. Não modifica `widget/mod.rs` pra
exportar seu widget. Não toca `shells/desktop/src/main.rs`. Não
modifica `screens/hero.rs` pra mostrar seu botão na toolbar.

Quem faz toda essa amarração é o **agente Integrador** em janela
posterior. Sua entrega são arquivos novos prontos, testados em
isolamento, prontos pra serem "plugados" depois.

Pense no modelo como **fabricação por peça**: você fabrica uma peça
acabada que vai pra estante; o Integrador depois pega a peça e
instala na máquina. Isso permite que múltiplos Implementadores
trabalhem em paralelo sem colidir, porque cada um cria arquivos
diferentes em locais diferentes — nunca editam os mesmos arquivos
compartilhados.

### Antes de fazer perguntas

Se você está prestes a perguntar:
- "Quer feature inteira ou MVP fatiado?" → **feature inteira sempre**.
  Não pergunte.
- "Devo integrar com o editor?" → **não**, você entrega a ilha;
  Integrador amarra depois. Não pergunte.
- "Devo já ler SKILL agora?" → **sim, sempre**, antes de qualquer
  outra coisa. Não pergunte.
- "Qual WORKTREE / BRANCH?" → se o Enio não te informou no mesmo
  turno deste doc, peça **uma vez** e prossiga.

O resto das dúvidas só faz sentido depois de você ler os 3 docs da §2.

## 4. O que você PODE tocar

### 4.1 Se a feature é "nova Tool" no editor — modelo ilha isolada

Todos os arquivos são **NOVOS**, em locais NOVOS. Nenhum arquivo
existente é modificado. A regra é: se o arquivo já existe no
repositório, você não toca.

- **Lógica da Tool**: arquivo novo em
  `crates/ph2d-editor/src/tools/<nome>.rs`. Implementa a trait
  `Tool` (definida em `crates/ph2d-editor/src/tool.rs` — você USA
  a trait, não modifica o arquivo). Toda a algoritmia, estado
  interno, ações da Tool ficam aqui.
- **UI/painel da Tool**: pasta NOVA em
  `crates/ph2d-editor/src/widget/<nome>/` (escolha o nome igual à
  Tool). Dentro:
  - `mod.rs` — API pública do painel da Tool.
  - Sub-arquivos do painel (ex: `panel.rs`, `picker.rs`, `paint.rs`).
  HR-12 e HR-15 valem aqui — vide §6.
- **Ícone da Tool**: arquivo novo em
  `crates/ph2d-editor/src/tools/<nome>_icon.rs` exportando uma
  função pública que retorna a `BezPath` do ícone. **Não toque** o
  enum `IconId` em `crates/ph2d-editor/src/icons.rs` — o Integrador
  adiciona a variante depois.
- **Testes próprios**: arquivos novos em
  `crates/ph2d-editor/tests/<nome>_*.rs`.

Você USA livremente os widgets, paint helpers, tokens e zonas
existentes — tudo o que está em `crates/ph2d-editor/src/widget/`,
`paint.rs`, `ph2d-tokens`, etc. está disponível como API pública.
Não modifica. Apenas consome.

### 4.2 Exemplo concreto — Tool Painter

Para uma feature "Tool Painter com flood-fill + eyedropper +
protection mask + island separation", você criaria EXATAMENTE estes
arquivos NOVOS:

```
crates/ph2d-editor/src/tools/painter.rs              # impl Tool, algoritmos
crates/ph2d-editor/src/tools/painter_icon.rs         # BezPath do ícone
crates/ph2d-editor/src/widget/painter/mod.rs         # API do painel
crates/ph2d-editor/src/widget/painter/panel.rs       # painel principal
crates/ph2d-editor/src/widget/painter/eyedropper.rs  # sub-widget
crates/ph2d-editor/src/widget/painter/mask_brush.rs  # sub-widget
crates/ph2d-editor/src/widget/painter/paint.rs       # Vello lowering
crates/ph2d-editor/tests/painter_no_alloc.rs         # HR-3
crates/ph2d-editor/tests/painter_smoke.rs            # smoke test
```

E **NÃO toca** os seguintes arquivos (mesmo que pareça que precise —
quem faz isso é o Integrador):

```
crates/ph2d-editor/src/lib.rs              # (re-exports do crate)
crates/ph2d-editor/src/icons.rs            # (adicionar variante IconId::Painter)
crates/ph2d-editor/src/widget/mod.rs       # (declarar `pub mod painter;`)
crates/ph2d-editor/src/tool.rs             # (registrar no ToolRegistry)
crates/ph2d-editor/src/screens/hero.rs     # (botão na toolbar)
shells/desktop/src/main.rs                 # (qualquer wiring)
```

Você reporta "pronto" com a ilha completa funcional em isolamento
(testes verdes, clippy clean). O Integrador faz a amarração depois.

### 4.3 Se a feature é "popular crate stub"

Crates stub atuais: `ph2d-audio`, `ph2d-save`, `ph2d-fluids`,
`ph2d-light`, `ph2d-sdf`, `ph2d-i18n`, `ph2d-telemetry`,
`ph2d-physics-soft`, `ph2d-net`.

O crate stub é seu sandbox total — pode criar e modificar qualquer
arquivo DENTRO do crate:

- Qualquer arquivo em `crates/<crate>/src/`.
- Testes próprios em `crates/<crate>/tests/`.
- Adicionar deps externas APENAS no `Cargo.toml` desse crate (nunca
  no workspace raiz).
- Não modifique a seção `[package]` do `Cargo.toml` do crate (ela
  herda do workspace).

Mesma regra de não-amarração vale: você **não modifica `shells/desktop/`
nem nenhum outro crate** para "expor" o seu. Quem amarra é o Integrador.

## 5. O que você NÃO PODE tocar

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
- `crates/ph2d-editor/src/zones.rs`, `floating_panel.rs`, `toast.rs`,
  `zen.rs`, `paint.rs`, `style.rs`, `interaction/` (chrome e helpers).
- `crates/ph2d-editor/src/screens/` (composição de telas, ex: hero).
- **Widgets pré-existentes** em `crates/ph2d-editor/src/widget/*.rs`
  e `crates/ph2d-editor/src/widget/*/` (use livremente como API
  pública; não modifique).
- **Tools pré-existentes** em `crates/ph2d-editor/src/tools/*.rs`
  (BrushTool, MoveTool, etc.).

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

## 6. Hard Rules críticas pra você

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
  `Entity`/`Handle<T>`/`AssetId` — todos `u64` ou newtypes equivalentes.
  Nunca pointers, nunca structs internas.
- **HR-12 — Acessibilidade.** Todo widget novo precisa importar
  `ph2d_a11y` e emitir `Node` AccessKit. Teste
  `crates/ph2d-editor/tests/hr12_widgets_a11y.rs` pega regressão.
- **HR-15 — i18n.** Zero string hardcoded em UI de produção. Para
  a11y labels hoje (i18n stub), use fallback genérico curto e
  documente em comentário; o macro `t!()` chega quando i18n shipar.
  Teste `crates/ph2d-editor/tests/hr15_no_hardcoded_ui_strings.rs`
  pega regressão.

Demais HRs (HR-1, 2, 4, 6, 7, 9-11, 13, 14, 16, 17) podem se aplicar
dependendo do escopo — consulte SKILL §9 quando relevante.

## 7. Convenções de código

- `cargo fmt` obrigatório (`style_edition = "2024"`, max_width 100).
- `cargo clippy -- -D warnings` clean.
- Módulos: `snake_case`. Tipos: `PascalCase`. Constantes: `SCREAMING_SNAKE`.
- Erros: `thiserror` em libs; cada crate tem `Error` enum próprio.
- Documentação `///` em todo `pub`.
- Comentários poucos e focados no PORQUÊ não-óbvio (constraint oculta,
  invariante sutil, workaround específico). Não comente o ÓBVIO.
  Não inclua emojis.
- Componentes ECS: substantivo singular (`Position`, `Velocity`).
- Sistemas: verbo + objeto (`update_physics`, `render_sprites`).
- Eventos: passado (`EntitySpawned`, `AssetLoaded`).
- Async **proibido no core** exceto `ph2d-asset::loader` e
  `ph2d-net::transport`. Use sync por default.
- Nunca `unwrap()` em código de produção; `expect("razão clara")`
  em prototipagem; propaga via `?` em release.

## 8. Antes de reportar "pronto"

Rode TODOS, na raiz do workspace, em ordem:

```
cargo fmt --check
cargo clippy -p <seu-crate> --tests -- -D warnings
cargo test -p <seu-crate>
git status
```

Cada um deve passar / mostrar resultado esperado. Se algum falha,
corrija ANTES de reportar.

## 9. Como commitar e reportar

Comite local apenas com `git add` de arquivos específicos:

```
git add crates/ph2d-editor/src/tools/painter.rs crates/ph2d-editor/tests/painter_test.rs
git commit -m "feat(editor): Painter tool with stroke smoothing"
```

Nunca `git add -A` nem `git add .` (risco de incluir lixo).
Mensagem de commit: imperativo, inglês curto, primeira linha < 70
caracteres. Cite a HR aplicável se houver ("HR-3: pool pré-alocado").

**NÃO faça `git push`.** Push é responsabilidade do agente PRCI
em janela posterior, após integração local.

**Reporte ao Enio:**
- "Feature <nome> pronta na worktree <path>, branch `feature/<nome>`."
- Lista de arquivos novos/modificados.
- "Testes verdes confirmados: cargo test, clippy, fmt todos passam."
- "Aguardando janela de integração."

## 10. Quando algo dá errado

- **Testes existentes que você não tocou começam a falhar:** pare.
  Você provavelmente quebrou algo fora do seu escopo. Reporte ao
  Enio com o output do teste.
- **Você descobre bug pré-existente:** documente em comentário no
  seu código, reporte ao Enio, mas **não corrija fora do seu escopo**.
- **Compilação quebra após `git pull`:** você não deveria estar
  pulando — confirme com o Enio.
- **Necessidade de adicionar dep externa nova ao workspace raiz:**
  pare, reporte ao Enio (mudança em `Cargo.toml` raiz é blacklist).
- **Necessidade de mudar API de ph2d-core/ecs/host/tokens:** pare,
  reporte ao Enio (mudança em centros é blacklist; vira trabalho
  de Integrador ou novo escopo).

## 11. Tom de comunicação

- pt-BR direto, conciso. Sem hedging.
- Quando incerto, ofereça 2-3 opções concretas + recomendação.
- Erros: explique a causa raiz, não só o sintoma.
- Não use emojis em mensagens nem em código.
- Mensagens curtas e densas valem mais que paráfrases longas.

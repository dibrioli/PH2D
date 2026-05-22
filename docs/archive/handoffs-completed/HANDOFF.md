# HANDOFF — Claude assume no Mac

> **⚠️ DOCUMENTO HISTÓRICO — handoff de bootstrap em 2026-05-08.**
>
> Tudo abaixo deste banner descreve o estado **antes** do spike fechar.
> Não use o corpo deste arquivo para decidir próxima ação — vá direto
> para os docs canônicos abaixo.
>
> **Estado em 2026-05-09:**
> - Spike de scripting fechado (ADR-0019, Luau ratificado).
> - M1-M12 do plano pós-spike implementados e mergeados em `main`
>   (PRs #1-#28). Cobertura: ph2d-host, ph2d-core, ph2d-gpu, ph2d-ecs,
>   ph2d-render (sprites), ph2d-asset (hot reload blake3), ph2d-script
>   (Luau sandbox + reset+restore), ph2d-input (gilrs + Pencil stub),
>   ph2d-mcp + ph2d-bindgen, ph2d-physics (Rapier determinístico),
>   ph2d-vector + ph2d-text (Vello + parley wrappers), ph2d-tokens +
>   ph2d-a11y + ph2d-editor (4 zonas Procreate-style + FloatingPanel +
>   ZenMode + ToastQueue + ToolRegistry).
> - M13 em curso: tool palette UI shipada (PR #30); design system em
>   handoff para Claude Design (vide `docs/design/PROMPT_CLAUDE_DESIGN.md`);
>   crates `ph2d-sdf`, `ph2d-light`, `ph2d-physics-soft`, `ph2d-fluids`,
>   `ph2d-audio`, `ph2d-net`, `ph2d-i18n`, `ph2d-save`, `ph2d-telemetry`
>   ainda stubs aguardando projeto-piloto que dite ordem.
>
> **Docs canônicos para LLM nova (ler nesta ordem):**
> - [`SKILL_Stack_PH2D_Definitiva.md`](../SKILL_Stack_PH2D_Definitiva.md) — fonte de verdade técnica (stack, HRs, ADRs)
> - [`CLAUDE.md`](../CLAUDE.md) — workflow operacional (CI, memória)
> - [`docs/plans/2026-05-post-spike.md`](plans/2026-05-post-spike.md) — plano de marcos com status atualizado
> - [`docs/design/PROMPT_CLAUDE_DESIGN.md`](design/PROMPT_CLAUDE_DESIGN.md) — brief do design system em curso
> - [`docs/architecture/decisions/`](architecture/decisions/) — ADRs 0003, 0019, 0020, 0021, 0022, 0023 (todos Accepted)
> - **Memória persistente:** `~/.claude/projects/.../memory/MEMORY.md`
>
> Referências no corpo abaixo a `docs/spike/2026-05-plan.md` apontam para
> o plano do spike (fechado em 2026-05-08), não para a fase atual.
> Caminhos antigos do Windows (`C:\Users\dibri\...`) também são
> históricos — projeto migrado para `/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/`.

**Versão:** 1.0
**Data deste handoff:** 2026-05-08
**Origem:** Claude Opus 4.7 (1M context) trabalhando no Windows 11 do Enio
**Destino:** Claude assumindo no Mac (modelo a definir; provavelmente Opus ou Sonnet vigente)

---

## Antes de qualquer coisa, leia esta seção

Você não é "AI assistant ajudando com tarefas". Você é o **único developer da PH2D — Power House Game Engine**. Enio é o decisor de produto e arquitetura, dono do projeto, mas não escreve código. Ele descreve direção, aprova decisões grandes, e te confia execução. Tudo que está implementado e tudo que vier a ser implementado é seu output.

Esta posição implica responsabilidades específicas:

- **Você decide engenharia.** Quando a decisão é técnica e cabe nas Hard Rules (HR-1..HR-17), decide e executa. Não pergunta "posso fazer X?" para coisas que estão claramente dentro do escopo.
- **Você pergunta produto.** Quando a decisão muda comportamento user-facing, escopo, ou cruza um tiebreaker do §13 do SKILL, pergunte ao Enio antes de implementar.
- **Você é cético.** Inclusive sobre claims técnicos antigos (incluindo os meus). Versões mudam, APIs mudam, "fato verificado em maio" pode estar errado em junho.
- **Você documenta.** ADR para cada decisão arquitetural; comentário inline para cada decisão sutil. Outra LLM (talvez você de novo, sem este contexto) vai precisar entender depois.

---

## Onde está tudo (ordem canônica de leitura)

Tudo no diretório `_PH2D_definitiva/`. Ordem recomendada de absorção:

1. **`SKILL_Stack_PH2D_Definitiva.md`** — fonte de verdade. ~720 linhas. Lê inteiro antes de tocar em código. Glossário no §2, Hard Rules numeradas no §9 (citáveis por ID `HR-N`), tiebreakers no §18.
2. **`docs/spike/2026-05-plan.md`** — plano operacional do spike de scripting de 3 semanas (deadline 2026-05-29). Define os 16 critérios pass/fail e o cronograma. Você vai executar isso.
3. **`docs/architecture/decisions/0003-ecs-choice.md`** — ECS canônico (flecs-rs vs bevy_ecs 0.18) está pendente de decisão por medição em C11 do spike.
4. **`docs/architecture/decisions/0019-spike-scripting-output.md`** — placeholder para o output do spike. Você preenche ao fim das 3 semanas.

Outros ADRs (ADR-0001, 0002, 0004 a 0011) ainda não foram escritos — são parte do trabalho. Veja §19 do SKILL.

---

## Decisões já fechadas — não retroceder

Se algo na lista abaixo te parecer errado, abre ADR documentando o argumento; **não** reverta silenciosamente.

### Linguagem e runtime de scripting (em validação no spike)
- **Luau strict** via **mlua 0.10** (feature `luau`).
- Coroutines como primitiva temporal canônica; **sem async/await** na engine.
- Bytecode pré-compilado no ship build.
- WASM via wasmtime + Component Model (wit-bindgen) para hot path CPU-bound.
- Hot reload por **reset+restore (modelo Defold)**; estado canônico vai pro World, não em closures Lua.
- Mensageria estilo Defold como mecanismo de desacoplamento (hash interning, FIFO same-sender→same-target).
- Storage lateral (FSM/BT/Dialogue) fora do ECS, com restrições de tipo (HR-16).
- Bridge Luau↔WASM: **Luau chama Rust; Rust chama WASM** (single FFI boundary).
- Sandbox em dois níveis: trusted (project) vs untrusted (asset script).

### Stack core (versões pinadas em §5 do SKILL)
- Rust 2024 edition, MSRV 1.85+ (algumas deps exigem 1.88).
- wgpu 29, vello 0.8 (alpha — risco assumido em ADR-0004 quando for escrito), kurbo 0.13, parley + harfrust + skrifa para texto, taffy 0.10 para layout, rapier2d 0.32, wasmtime 44, quinn 0.11 + web-transport-quinn 0.11.
- **kurbo NÃO tem `PathOps`** (não existe). Boolean ops via `linesweeper` (beta). Não tente importar `kurbo::PathOps` — falha de build, não erro de digitação.

### Plataformas (matriz §4 do SKILL)
- iOS/macOS: Metal 3 mínimo (iPhone 12+). **iOS NÃO tem Vulkan**; backend é Metal direto via wgpu.
- Android: Vulkan 1.3 mínimo (Android 13+).
- Windows: D3D12 mínimo. **Windows é alvo de release, não de dev primário** — Mac é primary, Windows secundário para validar D3D12.
- Web: WebGPU em Chrome 121+/Safari 18+/Firefox 141+.

### Arquitetura
- 1 core Rust + 3 shells (desktop, iPad, Android) + 1 web target. Detalhe em §6 do SKILL.
- Editor é a engine (HR-7) — feature flag `editor` corta 100% em release de jogo.
- Toda interação com SO via trait `PlatformHost` (HR-1).
- Asset = hash blake3 (HR-6); paths não são identidade.

### Modelos LLM para o spike
- **Claude 4.7+** (Anthropic) e **Gemini 3.1+ Pro** (Google) são os modelos de validação para C8, C15 e C16 do spike.
- Observação registrada: ambos são closed-source flagship. Pós-spike, considerar adicionar um modelo open-weights como hedge contra risco vendor. Decisão de Enio.

---

## Princípios existenciais (memorizar)

### LLM é o único programador
PH2D é desenvolvida exclusivamente por LLM (você). Isso muda o peso de tudo:

- API ambígua → bloqueio.
- Doc desatualizado → bloqueio.
- Log não-estruturado → bloqueio em debug.
- MCP tool faltando → fricção catastrófica.
- Naming inconsistente → completion ruim → código ruim.

Custos que humano tolera, LLM não tolera. Quando estiver em dúvida sobre prioridade, pergunte: "isso facilita ou dificulta para outra LLM gerar/usar/debugar este código?"

### Hard Rules (HR-1 a HR-17) são lei
Estão no §9 do SKILL, com `Rule | Rationale | Enforced by`. Citáveis por ID em commit message ("HR-3: pool pré-alocado em vez de Vec"). Se você precisa violar uma HR, pare e escreva ADR. Não viole silenciosamente.

### Tiebreakers (§18 do SKILL) resolvem conflitos
Ordem importa:
1. Performance no hot path > tudo
2. Determinismo onde prometido > conveniência
3. Segurança (sandbox, MCP governance) > facilidade
4. Acessibilidade > UX bonita
5. UX nativa de iPad > uniformidade de codebase
6. APIs estáveis > APIs elegantes
7. Reproducibilidade de build > velocidade de build
8. Compreensibilidade por LLM > brevidade clever

---

## Próxima ação imediata — semana 0 (bootstrap Mac)

Você acabou de chegar no Mac. Antes de qualquer código:

### 0.1 — Verificação de toolchain
Rode (PowerShell-equivalente em zsh/bash):

```bash
rustc --version || echo MISSING_RUST
cargo --version || echo MISSING_CARGO
xcode-select -p || echo MISSING_XCODE_CLT
git --version
node --version
pnpm --version
```

### 0.2 — Setup esperado no Mac (se faltar algo, instale)
- **Xcode CLT:** `xcode-select --install`
- **Rust 1.85+:** `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`. Após instalar, `rustup default stable`, `rustup target add aarch64-apple-ios x86_64-apple-ios aarch64-apple-ios-sim wasm32-unknown-unknown`
- **Homebrew (se não tiver):** `/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"`
- **Ferramentas via brew:** `brew install git node pnpm cmake ninja` (cmake/ninja podem ser necessários para deps C de wasmtime).
- **Vulkan SDK:** opcional na semana 0, instalar quando começar shader work em wgpu Vulkan backend (mac usa Metal por default).

### 0.3 — Inicializar repositório Git
O diretório `_PH2D_definitiva/` ainda não é git repo (verificado em 2026-05-08). Inicialize:

```bash
cd _PH2D_definitiva
git init
git add SKILL_Stack_PH2D_Definitiva.md docs/
git commit -m "Initial: SKILL canônico v2.0 + plano spike + ADRs stub"
```

Decida com Enio se o remote vai ser GitHub privado ou outro provider antes de adicionar `origin`.

### 0.4 — Cargo workspace skeleton (~30 minutos)
Crie `Cargo.toml` no root com workspace listando os 21 crates do §7 do SKILL. Cada crate inicialmente com `lib.rs` vazio + comment `//! TODO: spike S{semana}`. Não tente popular tudo — só skeleton para `cargo check` passar.

`rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.85"  # ou superior se vello/parley exigirem
components = ["rustfmt", "clippy", "rust-src"]
targets = ["aarch64-apple-darwin", "aarch64-apple-ios", "wasm32-unknown-unknown"]
profile = "default"
```

`rustfmt.toml`:

```toml
style_edition = "2024"
max_width = 100
```

`.gitignore` mínimo: `target/`, `.DS_Store`, `*.swp`, `node_modules/`, `dist/`, `runtime/ts/dist/`.

### 0.5 — Confirmar com Enio antes de começar a semana 1
Avise: "Setup completo. Pronto para iniciar semana 1 do spike (C1, C2, C11). Confirmando antes de começar."

---

## Semana 1 do spike (após bootstrap)

Conforme `docs/spike/2026-05-plan.md`. Resumo das ações:

1. **Branch:** `git checkout -b spike/scripting-foundation`.
2. **C1 Foundation:** crate `ph2d-script` com mlua 0.10 (feature `luau`). Hello world Luau rodando 60 frames sem panic. Fixture em `tests/spike/hello.{rs,luau}`.
3. **C11 ECS shootout:** implementar a mesma fixture mínima (200 entities, 1 hierarquia, 3 components, 2 systems, 1 observer/hook em delete) em **dois feature flags**: `--features bevy-ecs` e `--features flecs-ecs`. Coletar métricas listadas em C11.
4. **C2 Query overhead:** bench Luau vs Rust nativo. Criterion-based.
5. **Gate semana 1:** ECS canônico decidido + ADR-0003-rev2 atualizado. **Não avance para semana 2 sem fechar isso.**

### Decisão default em C11
Se medições forem ambíguas: **bevy_ecs vence**. Razões:
- Rust idiomático (sem unsafe FFI).
- Comunidade Rust gamedev majoritária.
- Training data abundante para LLM (você).
- Lifecycle hooks (`Component::on_remove`) em 0.18 cobrem o caso de storage lateral (D4 dos esclarecimentos).

flecs precisa **vencer claramente** em hierarquias/observers/queries para justificar o custo do binding C.

---

## Como decidir quando estiver em dúvida

### Decida solo (sem perguntar a Enio)
- Padrões de código dentro das convenções §10 do SKILL.
- Detalhes de implementação que não cruzam HR.
- Escolha entre 2 caminhos quando os dois são tecnicamente equivalentes.
- Refatorações dentro de um crate.
- Testes adicionais.
- Nomes de tipos/módulos consistentes com convenções.

### Pergunte ao Enio
- Mudanças em qualquer Hard Rule (HR-1..HR-17).
- Adição de dependência fora da tabela §5.
- Decisões que custam tempo/dinheiro (instalar SDK proprietário, comprar dispositivo, etc.).
- Decisões user-facing (UX do editor, naming público de APIs TS/Luau, formato de save).
- Plataformas alvo (matriz §4).
- Modelos LLM canônicos.
- Ambiguidade em tiebreaker (§18) — se dois critérios apontam direções opostas e não há ordem clara.

### Ao perguntar, prefira:
- Apresentar 2-3 opções com trade-offs concretos.
- Recomendar uma com rationale.
- Pedir sim/não, não "o que você acha?".

Enio aprecia decisão pronta apresentada. Não aprecia "consulte vibrational alignment com sua visão".

---

## Como Enio gosta de trabalhar (observado)

### Tom
- Direto. Sem floreios.
- Crítico — quando ele pergunta "o que pensa?", quer crítica honesta, não validação.
- Opinionado — espera que você seja opinionado também.
- Técnico — assume que você sabe o domínio. Não simplifica demais.

### Formato de resposta
- Markdown estruturado com headers, tabelas, listas.
- Código em blocos com linguagem.
- Conciso — densidade alta, sem inflar com prosa.
- Quando há análise técnica longa, separe em seções com headers.

### Idioma
- **Português brasileiro** com diacríticos corretos (não "nao" em vez de "não").
- Termos técnicos e identificadores em inglês.
- Comentários em código em inglês curto.

### Quando ele aceita "em estudo" como resposta
- Ele aceita honestidade. Se você não sabe, fale "não sei, vou medir/perguntar/investigar".
- Não aceita opinião disfarçada de fato.

---

## Lições do agente anterior (eu) — coisas que errei

Compartilho para você não repetir:

1. **Confiei em claims técnicos do SKILL original sem verificar.** Encontrei depois: `kurbo::PathOps` não existe; `wgpu = 24` estava 5 majors atrás; MSRV 1.83 incompatível com edition 2024; Vello é Y-down, não Y-up; QuickJS GC não tem budget em ms. Lição: **verifique versões e claims via web search ou docs antes de usar como fundação.**

2. **Subestimei tamanho do scope inicial.** Primeiro draft do SKILL tinha gaps grandes (text rendering, a11y, save migration, MCP governance, memory budgets, concurrency model). Lição: **engines têm muito cross-cutting; não confie só no que está na cabeça do owner.**

3. **Aceitei mudança bevy_ecs → flecs sem questionar inicialmente.** Quase passou batido até notar que era mudança grande sem rationale. Lição: **toda mudança que toca camada inteira precisa ADR; sem ADR, devolva para autor.**

4. **Quase aprovei "spike de 2 semanas".** Era irreal para 16 critérios. Lição: **se o plano parece apertado para 2 semanas, é porque é. Empurre para 3.**

5. **Quase aceitei "GC budget de 1ms" para QuickJS sem checar.** É design impossível dado o GC stop-the-world. Lição: **qualquer claim performance-quantitativo precisa fundamentação ou marca de "soft target".**

---

## Sistema de memória pessoal (importante)

Você (Claude) tem um sistema de memória persistente em `C:\Users\dibri\.claude\projects\c--Users-dibri-OneDrive-Documentos-_PH2D-Game-Engine\memory\` (no Windows; no Mac será `~/.claude/projects/...` equivalente). Use para:

- Salvar feedback do Enio quando ele corrige seu approach (`feedback_*`).
- Salvar contexto de projeto que muda (`project_*`).
- Salvar referências externas (`reference_*`).
- Salvar perfil do Enio (`user_*`).

Não salve coisas derivaveis do código (paths, convenções já documentadas no SKILL). Não salve estado de tarefas em progresso.

Se você é uma sessão nova chegando neste projeto, leia `MEMORY.md` no diretório de memory primeiro. Você pode descobrir feedback acumulado de sessões anteriores.

---

## Quando o spike terminar (deadline 2026-05-29)

1. Preencher `docs/spike/2026-05-report.md` com tabela de medições (uma linha por critério).
2. Atualizar `docs/architecture/decisions/0003-ecs-choice.md` para `Status: Accepted` com tabela final de C11.
3. Preencher `docs/architecture/decisions/0019-spike-scripting-output.md` com veredicto.
4. Reescrever `§11.7` do SKILL refletindo Luau + ECS escolhido.
5. Atualizar `§5` (Stack canônico) se ECS canônico mudou.
6. Abrir PR para `main` (ou equivalente) com revisão final por Enio.

Após aprovação: começar implementação real do core. Próximo plano operacional será criado conforme cronograma macro do projeto.

---

## Última nota

A barra é alta. PH2D pretende ser melhor que Unity e Godot em 2D — não "competente em 2D", **melhor**. Em três eixos específicos:

1. Qualidade vetorial/SDF/iluminação (compute-first).
2. Ferramentas de artista (Apple Pencil first-class, editor unificado).
3. Produtividade com agentes de IA (você é o developer; isso decide tudo).

Tratamento de gambiarra é "obrigado, mas não". Quando o caminho fácil te tentar, lembre-se: outra LLM vai ler isso depois. Provavelmente você de novo, sem este contexto. Escreva pensando nessa LLM.

Boa sorte. Você tem todas as ferramentas. Vai.

— Claude Opus 4.7 (1M context), 2026-05-08

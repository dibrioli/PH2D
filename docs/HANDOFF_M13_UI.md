# HANDOFF — Próximo agente assume implementação da biblioteca UI (M13)

**Versão:** 1.0
**Data:** 2026-05-09
**Origem:** Claude Opus 4.7 (1M context) — créditos esgotados
**Destino:** próximo agente Claude em conta nova
**Plano operacional:** [`docs/plans/2026-05-ui-components.md`](plans/2026-05-ui-components.md)

> Este documento é o briefing exaustivo pro próximo agente. Cole na
> primeira mensagem da sessão nova. Em conjunto com o plano operacional
> linkado acima, ele tem tudo que precisa pra executar M13 do início
> ao fim sem pedir permissão.

---

## 1. Quem você é e quem é o Enio

Você é o ÚNICO desenvolvedor do PH2D. Enio Brito (dibrioli@gmail.com)
é o dono e único decisor de produto. Ele NÃO escreve código. Aprova
decisões grandes, escolhe direção, confere CI visualmente. Tudo que
está implementado até agora foi feito por outras instâncias suas
(Claude). Você não está "ajudando um humano" — você É o engenheiro.

Implicações:
- Decisões de engenharia dentro das Hard Rules (HR-1..HR-17): você
  decide e executa, sem pedir.
- Decisões de produto / user-facing / fora das HRs: pergunta ao Enio
  com 2-3 opções concretas + recomendação.
- Você é cético inclusive sobre claims técnicos antigos (versões
  mudam, APIs mudam, "fato verificado em maio" pode estar errado).
- Você documenta: ADR para cada decisão arquitetural; comentário
  inline para decisão sutil. Próxima LLM vai precisar entender.

---

## 2. O que é o PH2D

Engine 2D em Rust de altíssima performance. Posicionamento: superar
Godot e Unity em 2D em três eixos onde elas são fracas — qualidade
vetorial/SDF, ferramentas de artista (canvas-first inspirado em
Procreate), e produtividade com agentes de IA (você é o developer).

Stack canônico (versões verificadas em 2026-05-09):
- Rust 2024 edition, MSRV 1.92, toolchain 1.95, resolver "3"
- wgpu 28 (downgrade de 29 para alinhar com vello 0.8)
- vello 0.8 alpha (rasterização vetorial 100% compute em GPU)
- parley 0.6 alpha (text layout via harfrust + skrifa)
- bevy_ecs 0.18 (standalone, sem o resto do Bevy)
- mlua 0.10 com feature `luau` (gameplay scripting)
- rapier2d 0.28 com `enhanced-determinism` (físicas determinísticas)
- winit 0.30 (apenas em shells/desktop)
- gilrs 0.11 (gamepad)
- accesskit 0.24 (acessibilidade cross-platform)
- glam 0.30, blake3 1, postcard 1, image 0.25

NÃO use OpenGL, async runtime no core (exceto asset loader/net),
HashMap em simulation crates (HR-5/ADR-0022), ou hex literals
fora de ph2d-tokens.

---

## 3. Onde estamos no projeto

Plano operacional macro: M1..M13 marcos pós-spike.
Estado em 2026-05-09:
- M1-M12 ✅ implementados e mergeados em `main` (PRs #1-#28).
- M13 🟡 em curso — PR #31 aberta na branch `m13/design-library`.
  Tool palette UI shipada. Design system canônico do Claude Design
  importado em `docs/design/`. ph2d-tokens reescrito a partir do
  tokens.json (4 themes OKLCH).

Crates implementados (14): ph2d-host, ph2d-core, ph2d-gpu, ph2d-ecs,
ph2d-render (sprites), ph2d-asset (hot reload blake3), ph2d-script
(Luau sandbox + reset+restore), ph2d-input (gilrs + Pencil stub),
ph2d-mcp + tools/ph2d-bindgen, ph2d-physics (Rapier determinístico),
ph2d-vector (Vello wrapper), ph2d-text (parley wrapper), ph2d-tokens
(design tokens), ph2d-a11y (AccessKit), ph2d-editor (4 zonas +
FloatingPanel + ZenMode + ToastQueue + ToolRegistry + 5 widgets seed).

Crates stub aguardando projeto-piloto (10): ph2d-sdf, ph2d-light,
ph2d-physics-soft, ph2d-fluids, ph2d-audio, ph2d-net, ph2d-i18n,
ph2d-save, ph2d-telemetry.

Branch atual: `m13/design-library` (NÃO criar branch nova; trabalhe
nela). PR #31 ABERTA, NÃO criar nova.

---

## 4. Sua missão

Executar [`docs/plans/2026-05-ui-components.md`](plans/2026-05-ui-components.md)
integralmente. O plano contém 6 fases:
- Fase 0: pré-requisitos (icons port + paint helpers)
- Fase 1: refinar 5 widgets existentes (Button, Slider, Toggle,
  RadioGroup, ColorSwatch)
- Fase 2: 9 atomic novos (Checkbox, TextInput, TextArea, NumberInput,
  ProgressBar, Spinner, Avatar, Divider, Tag)
- Fase 3: 8 compostos (Tabs, Dropdown, Combobox, Vector3Editor,
  ListItem, Card, Tooltip, ContextMenu)
- Fase 4: 3 surfaces (Modal, Toast restyle, Popover)
- Fase 5: 2 complexos (TreeView, ColorPicker structure)
- Fase 6: integração final + commit + push + comentário em PR #31

Total: ~25 widgets, estimado em ~45h calendário.

LOOP por widget:
1. Implementar seguindo o "contrato canônico" no topo do plano
2. Rodar local: `cargo test -p ph2d-editor` + `cargo clippy
   --workspace --all-targets -- -D warnings` + `typos arquivo` +
   `cargo fmt --all`
3. Auditoria: 8 itens do checklist no plano
4. Fix tudo que falhou
5. Próximo widget

NÃO IMPLEMENTAREMOS A INTERFACE GRÁFICA DO APP nesta tarefa. Só os
componentes da biblioteca, isolados. Montar a tela hero
(02-editor-main) é trabalho de plano separado pós-componentes.

---

## 5. Docs pra ler ANTES de começar (ordem canônica)

Repositório: `/Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/`

a) [`CLAUDE.md`](../CLAUDE.md) — workflow operacional (CI, memória).
   1 minuto.

b) Sua memória persistente em
   `/Users/dibrioli/.claude/projects/-Volumes-MAC-EXTERNO-PROJETOS--PH2D-definitiva/memory/MEMORY.md`
   — índice de feedback acumulado, perfil do Enio, estado do projeto,
   paths canônicos. LEIA TODOS os memory files referenciados (5
   arquivos pequenos). Especialmente:
   - `feedback_communication_style.md` (pt-BR direto, opções concretas,
     decisão recomendada primeiro)
   - `feedback_ci_handling.md` (Enio confere CI visualmente; sempre
     fornecer link da run; não polling)
   - `user_role.md` (Enio = dono e decisor)

c) [`SKILL_Stack_PH2D_Definitiva.md`](../SKILL_Stack_PH2D_Definitiva.md)
   — fonte de verdade técnica. ~970 linhas. NÃO leia inteiro de cara —
   saiba que está lá. Consulte seções específicas conforme precisar:
   - §5 (stack canônico + versões)
   - §9 (Hard Rules HR-1..HR-17 — cite por ID em commits)
   - §10.1 (Rust style — comentários em INGLÊS curto, NUNCA pt-BR)
   - §11.9 (Editor UI — onde a biblioteca vive)
   - §15 (anti-patterns)
   - §18 (tiebreakers — 8 prioridades em ordem)
   - §19 (índice de ADRs)

d) [`docs/architecture/decisions/0023-ui-ux-baseline.md`](architecture/decisions/0023-ui-ux-baseline.md)
   — UI/UX ratificada (canvas-first, Procreate-inspired, WCAG 2.2 AA,
   AccessKit, FloatingPanel primitive).

e) [`docs/plans/2026-05-ui-components.md`](plans/2026-05-ui-components.md)
   — **SEU PLANO**. Leia integralmente. Decora a tabela de fases.

f) [`docs/design/README.md`](design/README.md) — overview do pacote do
   Claude Design.

g) [`docs/design/component-library.html`](design/component-library.html)
   — referência visual de TODOS os widgets × estados. Abra no navegador
   para olhar (Enio fará isso quando você reportar progresso). Você
   lê o HTML cru pra entender estrutura/tokens/estados.

h) [`docs/design/accessibility.md`](design/accessibility.md) — mapping
   de Roles AccessKit por widget. Use como contrato.

i) [`docs/design/interactions.md`](design/interactions.md) +
   [`gestures.md`](design/gestures.md) +
   [`animation.md`](design/animation.md) — especificações de
   comportamento (consulte conforme implementa).

j) [`docs/design/tokens.json`](design/tokens.json) — source of truth
   dos tokens. ph2d-tokens é mirror manual deste JSON (codegen futuro).
   Se precisar de token novo, adiciona aqui PRIMEIRO + reflita em
   ph2d-tokens.

k) [`crates/ph2d-editor/src/widget/button.rs`](../crates/ph2d-editor/src/widget/button.rs)
   — widget canônico de referência. Padrão a seguir nos novos.

l) [`crates/ph2d-tokens/src/lib.rs`](../crates/ph2d-tokens/src/lib.rs)
   + [`color.rs`](../crates/ph2d-tokens/src/color.rs) — entenda a API
   de tokens. Veja `oklch_to_srgb` se precisar adicionar cor nova.

---

## 6. Histórico de tentativas e abandonos (não repita esses erros)

A trajetória até a biblioteca canônica passou por várias tentativas
fracassadas de framework UI alheio. Cada uma morreu por motivo
diferente. Saiba quais NÃO retomar:

- **egui** (PR #29, abandonada) — pivot de imediate-mode UI. Morreu
  por bug de font texture (paint_jobs=5 mas textures_set=0, font não
  entrava no delta), preocupação de perf (redesenha cada frame), e
  version mismatch. Reverteu pra Vello custom. NÃO RETOMAR.

- **Slint** — investigada, rejeitada. Triple-license GPL/SFSL/Paid;
  inviável pra app proprietário. NÃO RETOMAR.

- **Xilem/Masonry** (oficial Linebender pra Vello) — rejeitada. Xilem
  0.4 → vello 0.6 → wgpu 26. Nosso projeto em vello 0.8/wgpu 28.
  Triple downgrade inviável. Reavaliar quando Xilem alcançar vello
  0.8+ (futuro distante).

- **GPUI** (do Zed editor, Apache 2.0) — considerada, rejeitada
  recentemente. Motivos: NÃO usa wgpu (Metal/blade direct), backend
  mismatch certo; projetada pra editor de código não game editor;
  in-game UI categorical NO; API instável + docs escassos; time do
  Zed disse publicamente "use por sua conta e risco". NÃO RETOMAR.

- **Design library v1 (HTML mockup feito por Claude)** — rejeitada por
  Enio: "não ficou bom".

- **Design library v2 (sdf3d-studio inspired, PR #31 commit a695c56
  + 061a4aa)** — Enio: "ainda não ficou bom. Vou usar Claude Design".
  Preservada como `docs/design/component-library-v2-legacy.html` para
  contexto histórico.

- **Design library v3 (Claude Design output, importada commit
  4bbd12a)** — APROVADA, source-of-truth atual.

A decisão final: continuar com Vello custom + biblioteca própria
implementada a partir do design v3. ESSE é o caminho que você
executa. Não tente reabrir as decisões acima.

---

## 7. Workflow operacional — regras duras

**CI / GitHub Actions:**
- Enio confere CI visualmente. NÃO polle.
- Push fornece SEMPRE o link da run via `gh run list
  --workflow=spike.yml --limit=1`.
- Job falhou → fornecer link direto do job (`gh run view --job=<id>`).
- Para ESTA tarefa específica: **NÃO PUSHE NEM DISPARE CI durante a
  execução das Fases 0-5**. Só ao fim da Fase 6 (commit consolidado
  único + push único).

**Commits:**
- Use HEREDOC pra mensagens. Sempre adicione trailer:
  `Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>`
- Cite Hard Rule no commit quando aplicável ("HR-12: a11y node em
  todo widget novo").
- Commit local ao fim de CADA fase (preserva progresso pra próxima
  sessão se context apertar).

**Subagents:**
- NÃO use Agent tool com `isolation: "worktree"`. Memória registrada:
  outputs leakaram pro main worktree e corromperam ~1h de trabalho.
  Se precisar de research paralelo, use Explore agent (read-only).

**Permissões:**
- O plano já foi aprovado pelo Enio. NÃO peça permissão pra cada
  passo. Vá do início ao fim.
- ASK SE: bater em decisão de produto user-facing, contradição entre
  design system e Hard Rule, bug em pré-requisito que bloqueia tudo.

**Comunicação com Enio:**
- Idioma: português brasileiro com diacríticos corretos.
- Tom: direto, denso, sem floreios. Crítico quando ele pergunta opinião.
- Formato: markdown estruturado (headers, tabelas, listas, código).
- Concisão alta. Resposta longa só quando análise técnica exige.
- Quando reportar progresso entre fases: 1 parágrafo curto + tabela
  de status. Sem celebração.
- Identificadores e termos técnicos em inglês.

**Estilo de código:**
- `cargo fmt` é lei.
- `cargo clippy -- -D warnings` é lei.
- `typos` é lei. Comentários em INGLÊS CURTO sempre. Nunca pt-BR em
  código (SKILL §10.1).
- Naming: snake_case módulos, PascalCase tipos, SCREAMING_SNAKE
  consts.

**Hard Rules dignas de lembrar para esta tarefa:**
- HR-1: core platform-agnostic. Nada de `target_os` em widgets.
- HR-3: sem alocação dinâmica em hot path. Widgets fazem build_a11y
  e paint frequente — evite allocs desnecessárias.
- HR-7: editor é a engine. `cfg(feature = "editor")` separa em
  release de jogo. Já está configurado.
- HR-12: TODO widget popular AccessibilityTree. Sem a11y::Node = bug.
- HR-15: strings de UI via Fluent (i18n não wired ainda; para esta
  tarefa, usar `String` literal em label é OK temporariamente).

**Tiebreakers (ordem importa):**
1. Performance no hot path > tudo
2. Determinismo onde prometido > conveniência
3. Segurança > facilidade
4. Acessibilidade > UX bonita
5. UX nativa de iPad > uniformidade de codebase
6. APIs estáveis > APIs elegantes
7. Reproducibilidade de build > velocidade de build
8. Compreensibilidade por LLM > brevidade clever

---

## 8. Ferramentas disponíveis

- Read/Write/Edit: prefira Edit com replace_all pra renames seguros.
- Bash: cargo, gh, git, typos, find, grep. Use Bash tool, não shell scripts.
- Agent: SÓ pra Explore (read-only). Sem worktree.
- WebFetch/WebSearch: pra docs externas (vello, parley, AccessKit).
- TodoWrite: use pra trackear progresso dentro de cada fase. Limpe
  todos completados ao começar nova fase.

**Comandos canônicos:**
- `cargo build --workspace`
- `cargo test --workspace` (passa atualmente, manter)
- `cargo test -p ph2d-editor` (foco do trabalho)
- `cargo test -p ph2d-tokens` (28 testes; não quebrar)
- `RUSTFLAGS="-D warnings" cargo clippy --workspace --all-targets`
- `cargo fmt --all -- --check`
- `typos` (full repo) ou `typos crates/ph2d-editor/src/widget/<file>`

---

## 9. Estado atual do repo que você herda

- Branch: `m13/design-library` (HEAD = 8737e72 ou commit subsequente
  com este HANDOFF + plano)
- PR #31 aberta: "M13: Claude Design handoff + ph2d-tokens codegen"
- CI verde (rodada 25617111205 sucesso).
- Working tree limpo (cargo build/test/clippy/typos/fmt todos verdes
  localmente).
- 90 testes em ph2d-editor passando + 28 em ph2d-tokens.
- 5 widgets seed prontos: Button, Slider, Toggle, RadioGroup,
  ColorSwatch (precisam ser refinados na Fase 1 do plano).
- ph2d-tokens completo: ColorToken (~30 slots), Theme (4 themes),
  Spacing (9-tier), TypeToken (9-tier), Radius, Motion, Layer.
  Comentários em inglês.
- Design package em `docs/design/` (87 SVGs, 17 telas HTML, tokens.json,
  4 specs MD, audit.md, README.md).

---

## 10. Como começar (passo-a-passo dos primeiros 30 min)

1. Cumprimente Enio em pt-BR: "Pronto. Assumindo o handoff M13 UI."
   (1 frase, sem floreio)

2. Em paralelo (1 mensagem com múltiplos tool calls):
   - Read MEMORY.md
   - Read CLAUDE.md
   - Read docs/plans/2026-05-ui-components.md
   - Read docs/design/README.md
   - Read crates/ph2d-editor/src/widget/button.rs (padrão)

3. Verifique status do repo: `git status --short`,
   `git log --oneline -3`,
   `cargo test --workspace --quiet 2>&1 | grep "test result" | tail -5`.

4. Verifique CI: `gh run list --branch m13/design-library --limit 1`.
   Deve estar verde (rodada antes deste handoff). Se não, REPORTE pro
   Enio antes de iniciar Fase 0.

5. Crie TodoWrite com itens da Fase 0 (3 tasks) + marca a primeira
   como in_progress.

6. Comece Fase 0.1 (icons port). 87 SVGs em
   `docs/design/icons/*.svg`. Estratégia recomendada: ler todos os
   SVGs com `find docs/design/icons -name "*.svg" | head`, escolher
   uma representação Rust (ex: `IconId` enum + função
   `icon_path(IconId) -> kurbo::BezPath` que parseia a string SVG
   path "d" attribute via `kurbo::BezPath::from_svg`). Adicionar
   `icons.rs` em `crates/ph2d-editor/src/`.

7. Loop nos demais itens conforme plano.

---

## 11. Frases de emergência

**Se context window apertar perto do fim de uma fase:**
- Faça commit local imediato com o que está pronto + workspace verde.
- Atualize `docs/plans/2026-05-ui-components.md` marcando ✅ as tasks
  concluídas + adicione 1 linha em "Lessons learned" se houver.
- Reporte pro Enio: "Fim de fase X. N widgets completos. Próxima
  fase: Y. Continuo?"

**Se você bater num bloqueio técnico real (não apenas dúvida):**
- NÃO chute. Pare, descreva o bloqueio em 3-5 linhas, ofereça 2-3
  alternativas com trade-offs, recomende uma. Aguarde Enio.

**Se você descobrir que uma decisão anterior estava errada:**
- Documente em ADR novo (próximo número livre, atualmente seria
  ADR-0024). Atualize SKILL §19 índice. NÃO reverta silenciosamente.

---

## 12. Última nota

A barra é alta. PH2D pretende ser melhor que Unity e Godot em 2D —
não "competente em 2D", MELHOR. Em três eixos: qualidade
vetorial/SDF/iluminação, ferramentas de artista (Apple Pencil
first-class), e produtividade com agentes de IA (você é o developer
— isso decide tudo).

Tratamento de gambiarra é "obrigado, mas não". Quando o caminho fácil
te tentar (pular auditoria, copiar widget sem entender, hardcoded hex,
comentário em pt-BR), lembre: outra LLM vai ler depois. Provavelmente
você de novo, sem este contexto. Escreva pensando nessa LLM.

Vai do início ao fim. Sem parar. Sem pedir permissão. Boa sorte.

— Claude Opus 4.7 (1M context), 2026-05-09

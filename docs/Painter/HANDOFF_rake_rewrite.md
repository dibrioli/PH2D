# HANDOFF — Arrancar o Rake atual e reescrever do zero (sofisticado, moderno, funcional)

> **Para:** um novo agente (contexto fresco).
> **De:** sessão de 2026-06-24.
> **Veredito do Enio:** *“Temos um rake que nunca funciona.”* Chega de remendo incremental — **rip-out + rewrite limpo.**
> **Crates (seu isolamento):** `ph2d-painter-brush`, `ph2d-tool-painter`, `ph2d-panel-painter-layers`. Nada fora disso sem reportar.

---

## §0 — Missão

O **Rake** (“a rotação da textura segue a direção do traço”) já foi remendado **duas vezes** nesta sessão e **continua ruim**. Sua missão tem três atos, nesta ordem:
1. **Arrancar** a implementação atual de Rake **inteira** e **deixar o código limpo e coerente** (sem campos/funções órfãos).
2. **Pesquisar** como motores maduros fazem “rotação segue o traço” e desenhar uma implementação **sofisticada, moderna e madura**.
3. **Implementar** o novo Rake e **provar que funciona** (e2e, não só unit verde).

Você **vai implementar** nesta rodada (diferente do handoff de Shape/Grain). Mas comece pelo checkpoint (§1) e só code depois de entender a causa-raiz (§3) — senão você vai repetir os meus erros.

---

## §1 — ⛔ PRIMEIRA AÇÃO: checkpoint

```bash
cd /Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva
git status                                  # não toque em WIP/`??` alheio
git tag rake-pre-rewrite-2026-06-24         # ponto de retorno garantido
git log --oneline -5                         # anote os SHAs do Rake atual (abaixo)
bash scripts/slot-seed.sh slot-1            # use o CARGO_TARGET_DIR impresso em TODO cargo
```
Linha-de-base de regressão (rode e anote os verdes **antes** de mexer):
```bash
CARGO_TARGET_DIR=<slot> cargo test -p ph2d-painter-brush --lib   # hoje: 132
CARGO_TARGET_DIR=<slot> cargo test -p ph2d-tool-painter  --lib   # hoje: 127
```
Commits do Rake atual (para referência — **NÃO use `git revert`**, eles estão entrelaçados com trims de LOC e o Jitter Spacing veio por cima; remova cirurgicamente):
- `1cba06cc` — Rake v1 (lerp por-dab). **Falhou** (anárquico).
- `c6f56f56` — Rake v2 (`advance_rake`, acumulador de long-baseline). **Ainda ruim** — é o que você vai arrancar.

---

## §2 — Contexto operacional (inegociáveis)

- **Fast mode:** commits locais com `git commit --no-verify` (msg termina com `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`). **NUNCA `git push`/CI** — o Coordenador faz, só quando o Enio mandar. `git add -- <seus paths>` (nunca `-A`).
- **HR-5 / determinismo:** brush é **transcendental-free** (sem `sin/cos/atan2`; usa o passo de 1° baqueado em `texture.rs::rotate_by_degrees` e splitmix64 em `jitter.rs`) e **reproduzível por seed**. Um brush “Rake off” precisa ser **byte-idêntico** ao baseline sem-Rake. Se o novo Rake precisar de RNG (não deveria), respeite a ordem-de-draw fixa do `jitter.rs`.
- **Caps de LOC:** workspace 600/arquivo; painel 600/arq + 200/fn. Gates: `cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap` e `architecture_panel_loc_cap`. ⚠️ O parser de fn do painel conta `'`/`"`/`{}` dentro de `//` — **não use apóstrofo em comentário novo** nos arquivos grandes (mascara funções).
- **Brush NÃO é contract-gateado** — liberdade para mexer em `Dab`/`TextureSettings`/`dab_basis`/stroke. Mas registre a decisão num **ADR curto**.
- **Inner loop = `cargo check -p <crate>`** no slot CoW; teste/clippy/audit **1× no fechamento**, não por task. **≤3 cargos simultâneos**.
- **As 4 causas da semana perdida** (`docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md`): costura-não-testada / “audit”=compilar / isolamento-órfão / alvo-irrefutável. **Verde-de-compilação vale ZERO no audit.** Releia a cada etapa.
- **Referência (GPL, só comportamento):** clean-room do Blender Texture Paint em `reference/blender-texture-paint/`. Procreate/Krita são proprietário/AGPL — pesquise **efeito**, nunca copie código.

---

## §3 — Causa-raiz (já diagnosticada — leia para NÃO repetir meus erros)

Um audit profundo já estabeleceu **por que o Rake nunca funcionou**. Internalize isto antes de codar:

> **A math do frame de textura (`dab_basis`) está CORRETA. O problema é a FONTE da direção.**

- O Rake alimenta `dab_basis` com a **direção do dab** (`dab_dir`). Hoje essa direção é **reconstruída a jusante**, no tool, a partir da **corda entre dabs consecutivos** (`dab_tangent` → `advance_rake`).
- Mas os dabs ficam a **~3px** um do outro, **sobre um spline Catmull-Rom suavizado pelo estabilizador** (`stroke.rs::walk_smoothed`, `stabilizer: 0.5` por padrão). A **direção** dessa corda de 3px é dominada por curvatura local + lag do lazy-mouse = **ruído**. Suavizar a *saída* (lerp/acumulador) **não recupera** uma direção cuja *entrada* já está corrompida. Por isso v1 (lerp) ficou anárquico e v2 (long-baseline) ainda não convence.
- Detalhes do audit que economizam seu tempo:
  - `dab_tangent` (`tool/paint/brush_settings.rs`) usa corda **forward** nos dabs interiores e **backward** no último → base inconsistente por batch (cada batch costuma ter 1–3 dabs).
  - **Convenção de eixo:** em `dab_basis`, `u = base = direção do traço`, e `sample` mapeia `rel·u` no **x** da textura ⇒ um padrão de listras/hachura fica **ao longo** do traço. Se, depois de estabilizar a direção, a hachura aparecer **90° rotacionada** do esperado, troque para `u = perp(base)`. (Isto é cosmético — só relevante DEPOIS que a direção estiver estável.)
  - Rake desliga os caches de stamp (`TextureSettings::is_cacheable`/`is_canvas_cacheable` exigem `!rake`) — cada dab precisa do próprio frame. Mantenha isso (ou proponha algo melhor no design).

### 💡 A virada de chave (o insight que torna a solução “madura”)
**O motor de stroke JÁ CALCULA uma direção limpa — e a joga fora.** Em `stroke.rs::walk_space` (linha ~383):
```rust
let dir = [(to[0] - from[0]) / seg, (to[1] - from[1]) / seg];  // direção do segmento do spline — LIMPA
```
…mas `dab_at(pos, pressure, overlap)` cria o `Dab` **sem** essa direção. O `Dab` (`stroke.rs:27`) carrega `center`, `radius_px`, `coverage`, `rotation`, `color` — **não tem heading**. Então o tool tenta **reconstruir** algo que o motor já tinha em mãos, e reconstrói **mal** (a partir dos centros dos dabs, não da geometria do caminho).

**A arquitetura madura:** a direção do traço é uma **propriedade de primeira classe do dab**, computada **uma vez, no motor**, onde a tangente do caminho é conhecida e pode ser suavizada corretamente (EMA da tangente do spline, em escala de pixels reais do input). `dab_basis` então só **lê** essa direção. Isso **elimina** toda a engenharia-reversa a jusante.

(Você é livre para validar/escolher outra abordagem na pesquisa — mas tem que bater este insight: a direção tem que vir de onde a geometria do caminho existe, não dos centros dos dabs.)

---

## §4 — ATO 1: arrancar o Rake atual + limpar

Remova **cirurgicamente** (sem `git revert`). Superfície a deletar:

**`crates/ph2d-tool-painter/src/tool/paint/stamp_cache.rs`**
- `advance_rake`, `RAKE_LERP`, `RAKE_BASELINE_MIN_PX`, e o `mod rake_tests`.
- Nos loops `stamp_dabs_ramped` e `stamp_dabs_per_pixel`: os locais `rake` / `rake_dir` / `rake_accum`, as chamadas `advance_rake(...)`, e os write-backs `self.paint.rake_dir = …` / `self.paint.rake_accum = …`. O que sobra alimenta `dab_basis` com a **nova** fonte de direção (ato 3).

**`crates/ph2d-tool-painter/src/tool/paint.rs`**
- Campos `rake_dir` e `rake_accum` em `PaintState`, suas linhas no `Default`, e os resets em `paint_begin`.

**`crates/ph2d-tool-painter/src/tool/paint/brush_settings.rs`**
- `dab_tangent` (a fonte de corda ruidosa) — se o novo design move a direção para o motor, esta função morre. Confirme que não há outro consumidor antes de apagar.

**NÃO apague (são a UI/efeito, não o bug):**
- `TextureSettings.rake: bool` (`texture.rs:288`) — o toggle do usuário **fica**.
- O **branch** de Rake em `dab_basis` (`texture.rs` ~`else if s.rake { normalize_or(dab_dir, …) }`) — a math fica; só muda **de onde vem o `dab_dir`**.
- O checkbox de Rake no painel (`paint_texture.rs` + `populate.rs`), `uses_dab_rotation`, e o bypass de cache.

Ao final do Ato 1: o código compila, os testes passam, **o Rake vira no-op** (toggle existe mas não gira nada) — um estado limpo e honesto sobre o qual você reconstrói. Commit local: `refactor(painter): rip out the broken Rake (clean slate for rewrite)`.

---

## §5 — ATO 2: pesquisa (curta, focada) + design

Você tem `WebSearch`/`WebFetch`. Estude “**drawing-direction / rake / rotation-follows-stroke**” em motores maduros e extraia o **algoritmo**, não a UI:
- **Krita** — *Brush rotation sensor* “Drawing Angle” + “Fuzzy Dab”/“Fuzzy Stroke”, e o parâmetro de **smoothing/fade** do ângulo de desenho (como ele estabiliza a direção em traços lentos/curvas).
- **Blender** — brush *Rake* / “texture angle = Rake” (a referência clean-room que já seguimos): como `paint_stroke.cc` deriva e suaviza a direção.
- **MyPaint** — `direction`/`direction_angle` *state inputs* e a constante de suavização (`smudge`/`slowtracking`-style): exemplo maduro de heading filtrado por velocidade.
- **Procreate** — Shape *Rotation* (modo que segue a direção do traço) e o conceito de *Azimuth* (referência de UX/comportamento).
- **Photoshop** — Shape Dynamics → Angle Jitter → Control = **Direction** (como “direção” é definida e estabilizada).

Perguntas que o design tem que responder:
1. **Como obter uma direção ESTÁVEL** a cada dab: EMA da tangente do spline no motor? Velocidade filtrada do estabilizador? Janela de pixels reais? (defenda a escolha; lembre que a entrada limpa são os **samples de ponteiro**, vários px, não as cordas de 3px).
2. **Início do traço** (sem direção ainda) e **traço parado/lento** (direção indefinida): qual o fallback? (manter a última direção; ou usar o Angle base até haver deslocamento mínimo).
3. **Curvas fechadas e reversões** (~180°): como evitar “chicotada”/wrap pelo caminho longo.
4. **Suavização vs responsividade:** uma constante (tipo o “fade” do Krita) — exponha como knob? ou fixa? (recomendo fixa e bem-calibrada primeiro; knob é follow-up).
5. **Convenção de eixo** (§3): textura alinha **ao longo** ou **através** do traço? Decida e documente.

Entregue um doc curto: `docs/Painter/rake_rewrite_design.md` (causa-raiz + abordagem escolhida + pseudo-código + casos-limite + convenção de eixo).

### Arquitetura recomendada (valide antes de assumir)
- **Adicione um campo de heading ao `Dab`** (ex.: `pub dir: [f32; 2]` — unit, `[0,0]` = indefinido). Em `stroke.rs`, mantenha um **heading suavizado por EMA** da tangente do caminho (atualizado em `walk_space`/`walk_smoothed`, onde `dir` já é computado) e carimbe-o em **todo** dab via `dab_at`. Reset no `begin`. Cuide dos métodos interativos (Line/Curve/Anchored) e do airbrush — todos passam por `dab_at`, então todos herdam a direção de graça.
- **`dab_basis`** passa a receber `dab.dir` como `dab_dir` (em vez do tangente reconstruído). Branch de Rake fica igual.
- **Tool**: deleta toda a reconstrução; só repassa `d.dir` para `dab_basis`. Simples e limpo.
- **Cache**: com heading por-dab, o frame varia por dab numa curva ⇒ o bypass de cache do Rake continua correto. (Se quiser, proponha um cache mais esperto como follow-up — não bloqueie o MVP nisso.)

---

## §6 — ATO 3: implementar + PROVAR

- Implemente a abordagem do §5. Mantenha **byte-idêntico** o caminho sem-Rake (o heading no `Dab` é ignorado quando `!rake`; prove com teste de baseline: mesmos dabs com/sem o campo novo quando Rake off).
- **Determinismo:** se não usar RNG (recomendado), nada a fazer; se usar, gate + ordem fixa.
- **Teste headless que prova que o Rake SEGUE o traço** (o que faltou nas tentativas anteriores): pinte um stroke em **curva** (`begin`+`extend`× por um arco) e **asserte que a direção do frame de textura acompanha a tangente do arco** dentro de uma tolerância, e que é **estável** (sem oscilação dab-a-dab). Use o harness de `crates/ph2d-painter-brush/src/stroke/tests.rs` (`straight_spec`, `collect_stroke`, `pt`) como base. Um stroke reto deve dar direção ~constante; um arco, direção que gira monotonicamente.
- **Audit e2e real** (não unit): o Enio testa caneta; você valida shaders/forma com `cargo test --features gpu -- --ignored` (roda headless no Metal). Descreva no relatório **como** provou que funciona.
- **Gates de fechamento:** `architecture_*_loc_cap`, clippy `-p` dos crates tocados, `cargo fmt --all`. Vigie LOC ao adicionar o campo no `Dab` e o EMA no stroke (pode precisar extrair helper).

Commits locais por bloco coerente (rip-out; engine heading; tool simplification; teste; painel se mexer). Sem push.

---

## §7 — Guard-rails anti-regressão

- **Checkpoint feito (§1).** Retorno: `git checkout rake-pre-rewrite-2026-06-24 -- crates/...`.
- **Rake off = baseline byte-idêntico.** Teste obrigatório.
- **Não regrida o Jitter Rotate** (`rotation` no `Dab`): ele compõe **por cima** do frame de Rake em `dab_basis` (multiplicação complexa). O novo heading entra como `base`; o `extra_rot` do jitter continua compondo igual. Prove que os dois coexistem.
- **Não toque** no Shape/Grain (outro handoff: `docs/Painter/HANDOFF_shape_grain_dual_texture.md`) nem em wash/efeitos.
- **Unit-verde ≠ funciona.** O critério é visual/e2e: a textura **gira com o traço, suave, em curvas e em traços lentos**, sem anarquia.

---

## §8 — Mapa de arquivos (preciso)

**Engine (`crates/ph2d-painter-brush/src/`):**
- `stroke.rs` — `Dab` (`:27`, **adicione o heading aqui**); `walk_space` (`:374`, `dir` limpo na `:383`); `walk_smoothed` (`:426`); `dab_at` (`:516`, **carimbe o heading aqui**); `begin`/`extend`/`finish`/`fill_segment`/`tick` (todos os caminhos que emitem dab).
- `stroke/tests.rs` — harness (`straight_spec`/`collect_stroke`/`pt`) para o **teste do arco**.
- `texture.rs` — `dab_basis` (branch de Rake), `TextureSettings.rake` (`:288`), `rotate_by_degrees`, `is_cacheable`/`is_canvas_cacheable` (`:333`/`:344`), `uses_dab_rotation` (`:267`).

**Tool (`crates/ph2d-tool-painter/src/tool/`):**
- `paint.rs` — `PaintState` (apague `rake_dir`/`rake_accum`); `stamp_dabs_inner`.
- `paint/stamp_cache.rs` — apague `advance_rake`/consts/tests; simplifique os 2 loops para repassar `d.dir`.
- `paint/brush_settings.rs` — apague `dab_tangent` (se sem outro consumidor).

**Painel (`crates/ph2d-panel-painter-layers/src/`):** `paint_texture.rs` (checkbox Rake) + `populate.rs` — **mantêm**; só toque se mudar a UX (não é o foco).

---

## §9 — Critérios de aceitação

1. ✅ Checkpoint/tag + baseline verde registrados.
2. ✅ Rake antigo **removido**, código limpo (sem órfãos), testes verdes, Rake vira no-op intermediário.
3. ✅ Design documentado (`rake_rewrite_design.md`) com causa-raiz + abordagem + casos-limite + ADR curto.
4. ✅ Novo Rake implementado: **byte-idêntico com Rake off**, e **com Rake on a textura segue o traço suavemente** (reto = direção constante; curva = direção girando; lento/parado = sem oscilação).
5. ✅ **Teste headless do arco** que falharia com a implementação antiga e passa com a nova.
6. ✅ Relatório ao Enio: o que era o bug, o que você fez, **como provou que funciona** (e2e), e custo/risco. Commits locais, sem push.

---

> Ponto de retorno: tag `rake-pre-rewrite-2026-06-24`.
> Regra-mãe: a direção do Rake **nasce no motor, onde o caminho é conhecido** — não se reconstrói dos centros dos dabs. Tudo o mais é detalhe.
> Boa sorte. 🎨

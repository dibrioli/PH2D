# HANDOFF — `line/Painter` REABERTA (2026-08-10)

> Para o agente que **assume** esta linha. Ela foi reaberta do zero sobre o `main` do dia,
> **logo depois** de a jornada anterior integrar. Leia isto ANTES de tocar em código — ele
> existe para você não reconstruir o que já foi construído nem re-derivar o que já foi medido.
>
> O bloco de primeira mensagem é o [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](../../IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md).
> **A FASE 0 dele não é cerimônia:** toda janela abre na RAIZ (que está em `main`) e o mesmo
> path relativo existe nas duas árvores — editar `crates/…` da raiz **compila e commita sem
> um único erro**, e só aparece na integração. `cd` + `pwd` + `git branch --show-current`
> antes de abrir qualquer arquivo.

## 1. Onde a linha está (medido, não lembrado)

| | |
|---|---|
| Worktree | `Worktrees/line-Painter/` (JÁ EXISTE — não crie) |
| Branch | `line/Painter` |
| HEAD | `76788440a` — **idêntico ao `main`**, zero commits próprios |
| Base | o `main` de 2026-08-10, com a jornada anterior **já integrada** |
| Tier | `workstation` ⇒ **Modo L** (DIRETRIZ §1.5) |

Setup já feito na reabertura: `git pull --ff-only origin main` (up to date) · `git worktree add -b`
· `cargo check -p ph2d-core` (warm-up do `target/` próprio, verde) · `bash scripts/mergiraf-setup.sh` (OK).

⚠️ O **primário** está **5 commits à frente de `origin/main`** (a reorganização de docs do
integrador, não pushada). Isso não é seu: **não pushe**, não é dívida da linha.

## 2. O que JÁ EXISTE — não reconstrua

A jornada anterior **integrou em 2026-08-10** (27 commits, 78 arquivos, +6.659/−339) e foi
**smokada e aprovada pelo Enio em 09/08**. O registro completo — mecanismo por mecanismo — está em
[`HANDOFF_INTEGRACAO_line_Painter_grid_stamp_2026-08-09.md`](HANDOFF_INTEGRACAO_line_Painter_grid_stamp_2026-08-09.md)
e resumido na entrada do `CLAUDE.md` §5. Em uma linha cada:

- **A regressão do Composite Brush** ([BUGS #22](../BUGS_painter.md)) — dois tempos de vida
  certos sozinhos (a esfregada resolve do congelado no pen-down · o composite promete por BATCH);
  curada **pela porta de quem deposita**, com três dobras alternativas construídas e MEDIDAS antes.
- **Os quatro "Use as …" leem a APARÊNCIA** (o bake responde a mesma pergunta) — incluindo a
  rota do **Per-Layer Color**, que não passa pelo flatten.
- **O GRID STAMP** — carimbo preso a uma grade própria; o `+1` era **um** dos cinco amostradores
  do stamp cacheado, e o `1,64×` era o raio compensando a cauda do falloff.
- **A Shape sai COZIDA** — construída COM relevo vivo, smokada, **REPROVADA e retirada dentro da
  própria linha** (843 linhas). Não a reconstrua sem ordem: *"não ficou bom, embora funcione"*.

**Superfície de colisão daquela jornada: VAZIA** (`PROJECT_SCHEMA` 69 intocado · contrato
congelado 4/4 · nenhum ADR · zero `Cargo.toml`). Hoje o `main` diz `PROJECT_SCHEMA` **70**
(degrau da `line/physics`) — se você bumpar, **CONTE contra o `main` do dia**, não escolha.

## 3. A fila — FECHADA em 2026-08-12 por ordem do Enio

> *"acho que tudo que vc listou já foi resolvido. Marque como resolvido. Apenas Accumulate deve
> ser estudo e comparado com o blender"* (Enio, 2026-08-12).
>
> ⚠️ **Fechado NÃO é sinônimo de consertado**, e a distinção é o que torna esta seção útil: cada
> item abaixo diz de que TIPO é o fecho. Os que são **decisão** ou **aceite com número** não
> voltam à fila sem um report novo; os dois que a medição mostrou vivos estão nomeados como tais.

### O único item ABERTO

**O ACCUMULATE do RELEVO** (a D3 do [doc 35](../35_accumulate_vs_blender.md)) — e ele está **a um
veto de distância**, não a um projeto.

O estudo foi feito e mediu as três divergências candidatas: a **D1** (o flag inerte em força máxima)
foi **construída e REFUTADA** — as duas leis coincidem ali, e o Blender tem a mesma inércia; a **D2**
(o knob de espaçamento invertendo a própria promessa, 1,02× → 8,17×) foi **corrigida e gateada**.
Sobra a D3: *o relevo não vê o flag*, que era a pergunta original do Enio.

A receita está pronta no [doc 20 §9.1/§12/§13](../20_accumulate_na_mesma_pincelada.md) e a
bifurcação do §6 tem recomendação com argumento de invariante (**arco**, porque *relógio de parede*
viola I2 e é inexprimível sob os shape editors). **Falta só o veto do Enio.**

### Os itens FECHADOS, com o tipo do fecho

| # | item | fecho |
|---|---|---|
| 1 | `watercolor_app_params_incremental_matches_full_*` | **ACEITE COM NÚMERO** — re-medido em 12/08: **Δ2 num único pixel** (byte 168460, px 131,82), contra uma tolerância de gate de ≤1. Sub-visível (Δ2 de 255 = 0,8%). Seguem `#[ignore]` com o diagnóstico completo no doc-comment |
| 2 | costura vertical do smear | **NÃO REPRODUZ** — a fixture não contém o fenômeno e o suspeito é o pipeline de DISPLAY, não o motor. Volta com foto nova + `PH2D_PREVIEW_DIAG=1` |
| 3 | dobra do Brush na fonte do smear | **DECISÃO** — aproximação nomeada (dobra na posição PINTADA); o exato é um *scatter* que pode deixar buracos na fonte |
| 4 | endurecimento da borda da MÁSCARA | **ACEITE** — as DUAS leis de acúmulo possíveis foram tentadas e cada uma tem artefato (produto = endurece · envelope = CONTAS, reprovado na tela). E a medição de 09/08 mostrou que **não é da máscara**: o brush digital sozinho endurece igual |
| 5 | cauda do taper no IMPASTO | **DECISÃO, com CONTROLE no gate** — o termo fica FORA do resolve, e o número é o motivo (`0` linhas entintadas com o restore contra `20` sem ele): *perder a tinta do artista é pior que uma cauda reta* |
| 6 | cauda reta em Watercolor / Wet Paint | **INERENTE** — reconstroem de acumuladores / de um fluido que não rebobina; não é a mesma causa do impasto |
| 7 | custo do resolve do taper | **SEM DÍVIDA** — um carimbo a mais do traço, uma vez, no pen-up; o número sai de graça no próximo `PH2D_PAINT_PERF=1` |
| 8 | `stroke.rs` no teto de LOC | **MEDIDO E FALSO** — o handoff dizia 697/700; medido em 12/08 são **691**. Nada a fazer |

⛔ **O que NÃO se repete** (medido, não opinião), caso um destes volte à mesa:
- No item 1: **`pad += 2·raio` não é a cura** — a janela é `dirty ⊕ 4·pad` por eixo, então num
  pincel de 80 px ela vira **o canvas inteiro todo quadro**, exatamente o custo que o caminho
  incremental existe para evitar. E três hipóteses já foram **REFUTADAS por medição**: ruído
  numérico (Δ0 em 2560 px), a soma-prefixo do `box_blur` (exata em `f64` ⇒ ainda Δ2), o `settled`
  (duas ablações deixam 139 dos 152 px de pé). A causa é **RAIO DE INVALIDAÇÃO**, vive no ARO,
  escala com o pincel (12 px a r=20 · 152 a r=80 · 361 a r=160) e o termo de borda a amplifica 17×.
- No item 4: a cura **não** é a lei da cobertura.

## 4. Armadilhas operacionais DESTA linha (custaram tempo real)

- ⚠️ **Os `--ignored` desta crate exigem `--test-threads=1` com a máquina CALMA.** Em paralelo,
  **cinco** kills de relógio dão vermelho e **nenhum é código**: mesmo binário, o
  `smear_perf_kill_criterion` mede **11,36/20,04 ms/move** sob `load 41` e **5,50/5,60** sob
  `load 0,6`. *Antes de acreditar num vermelho de relógio, olhe o `load average`.*
- ⚠️ **Rode a suíte do Painter em DEBUG também.** Precedente registrado nesta linha (o
  `ph2d-flip-colorize` panicava só em debug); `--release` sozinho esconde `wrapping_*`.
- ⚠️ **Todo comando de Bash começa com o `cd` da worktree.** A cwd escorrega para o primário —
  já aconteceu duas vezes nesta linha, uma delas mandando cinco edições para o `main` e fazendo
  a medição dizer *"sem ganho"*.
- ⚠️ **Nenhum smoke desta máquina significa nada com o `load average` acima de ~5** (o log do
  Wet Paint tem o detector embutido: um dígito de `ns/celula` = máquina sã, três dígitos = o log
  não fala sobre o código).

## 5. Smokes que valem (todos `--release`)

`PH2D_IMPASTO_SMOKE=1` / `=2` · `PH2D_WETPAINT_SMOKE=1` · `PH2D_MASK_SMOKE=1` ·
`PH2D_TAPER_SMOKE=1` · e as cenas da última jornada (Grid Stamp · a Shape cozida · a pilha ·
os quatro "Use as …" com Per-Layer Color ligado), descritas no handoff de integração de 09/08.
Diagnóstico: `PH2D_PAINT_PERF=1` · `PH2D_PREVIEW_DIAG=1` · `PH2D_FLUID_PROFILE=1`.

## 6. Como esta linha FECHA (DIRETRIZ §1.5.2.4 + §1.5.9)

Gate batched **1× no fechamento** (`scripts/nextest-impacted.sh` + clippy `--all-targets` +
auditoria ≥2 lentes sobre o diff acumulado) → **escreva o handoff de integração em
`docs/Painter/handoffs/`** (nunca na raiz de `docs/`) → reporte *"linha pronta + handoff"* →
**PARE**. Você **não** roda `foundational-integrate.sh`, **não** integra e **não** pusha:
integração e ship são de um agente integrador dedicado, e só por ordem EXPLÍCITA do Enio.

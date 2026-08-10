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

## 3. O que está ABERTO — com o preço já medido ao lado

Ordem sugerida de ataque é a de baixo para cima em custo de descoberta; **nenhum destes é
"comece a codar"** — os dois primeiros têm o próximo passo já nomeado e ele **não é código**.

1. **Os dois `watercolor_app_params_incremental_matches_full_*` seguem RED, `#[ignore]`**
   (`crates/ph2d-tool-painter/src/tool/paint/tests.rs:13054` e `:13074`).
   **Diagnóstico FECHADO — não re-derive:** é **RAIO DE INVALIDAÇÃO**
   (`pad +0 → 152 px · +64 → 38 · +128 → 1 · +2·raio → 0`), escala com o pincel
   (12 px a r=20 · 152 a r=80 · **361** a r=160), vive no **ARO**, com o termo de borda
   amplificando **17×**.
   ⛔ **REFUTADAS por medição** (não repita): ruído numérico (mesmo estado, união × retângulo
   dentro dela = **Δ0 em 2560 px**) · a soma-prefixo do `box_blur` (torná-la exata em `f64`
   deixa os dois gates em Δ2 — hipótese construída e medida) · o `settled` (duas ablações
   deixam **139 dos 152 px** de pé).
   ⛔ **E `pad += 2·raio` NÃO é a cura, e isso é medição:** a janela é `dirty ⊕ 4·pad` por eixo,
   então num pincel de 80 px ela vira **o canvas inteiro todo quadro** — exatamente o custo que
   o caminho incremental existe para evitar. **Falta a grandeza NOMEADA de alcance `2·raio`.**
2. **A costura vertical do smear NÃO reproduz na fixture** — e isso é achado, não falta de
   esforço (a foto mostra uma coluna de altura inteira, não a bbox de um dab ⇒ o suspeito é o
   pipeline de **DISPLAY**). **O próximo passo não é código:** é a armadilha do
   [BUGS #11](../BUGS_painter.md) — `PH2D_PREVIEW_DIAG=1` / `PH2D_PREVIEW_DUMP=<dir>`.
3. **A dobra do Brush na fonte do smear é aproximação NOMEADA** — ela dobra na posição
   **PINTADA**, não na pré-imagem; dobrar em `p − disp(p)` seria exato e é um **scatter**, que
   pode deixar buracos na fonte. Fica escrito para ninguém a "consertar" sem saber o trade.
4. **O endurecimento da borda da MÁSCARA** (doc 25 §13.10.4) segue aberto, com o número num
   teste executável. **As duas leis de acúmulo possíveis já foram tentadas** (produto = endurece ·
   envelope = CONTAS, reprovado na tela pelo Enio): a próxima hipótese tem de estar noutro lugar
   — o overlay, os defaults do pincel de máscara, ou **aceitar**. E a rampa da tinta hoje rastreia
   a da máscara EXATAMENTE ⇒ curar um cura os dois.
5. **A cauda do taper no IMPASTO** — o termo está FORA do resolve, com o número ao lado
   (`0` linhas entintadas com o restore contra `20` sem ele) e CONTROLE no gate.
   **O próximo passo não é código:** é descobrir de onde a cor do impasto de fato vem no canvas
   (o replay a reproduz em Digital e **não** em Impasto — ablação feita, hipótese não).
6. **Watercolor e Wet Paint seguem com cauda reta no arrasto**, por motivo **diferente** do
   impasto (reconstroem de acumuladores / de um fluido que não rebobina).
7. **O custo do resolve do taper não está medido** (um carimbo a mais do traço, uma vez, no
   pen-up) — o número sai do próximo `PH2D_PAINT_PERF=1`.
8. **`crates/ph2d-painter-brush/src/stroke.rs` em 697/700** — o próximo acréscimo ali **orça o
   split** (por assunto, para um IRMÃO; ver o precedente das dezenas de splits desta linha).

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

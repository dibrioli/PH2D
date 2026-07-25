# Handoff — `line/Painter`: levar o Painter para a GPU (continuação)

**Para:** o agente NOVO que assume a linha. **De:** o agente anterior, 2026-07-25, logo após a
integração da jornada de 23–25/07. **Ordem do Enio:** *"continuar a tarefa de levar o painter para o GPU
o máximo que for possível"*.

> ## Leia isto antes de qualquer código
>
> **O plano já existe, é medido, e está em ondas fecháveis sozinhas:**
> [`docs/Painter/25_avaliacao_gpu.md`](Painter/25_avaliacao_gpu.md) §7. **As Ondas 1, 2 e 5a/5b/5c já
> foram feitas e integraram** — não as reconstrua. **A sua é a Onda 3.**
>
> ⚠️ E a lei que governa tudo aqui não é "porte o que der": é **§0 do `CLAUDE.md` — MEÇA antes de
> limitar**. O censo já existe (`measure_gpu_frontier.rs`); a tabela dele é que decide a ordem, não a
> intuição.

---

## 1. Onde você está

| | |
|---|---|
| branch | `line/Painter` (worktree `Worktrees/line-Painter/`, **já existe**) |
| estado | **integrada** em 2026-07-25 e **rebaseada** sobre o `main` de hoje; árvore limpa, `cargo check -p ph2d-tool-painter -p ph2d-render` verde |
| nada pendente | `git rev-list --count main..HEAD` = **0** |

⛔ **FASE 0 do [MODELO_TROCA_DE_AGENTE_NA_LINHA](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md)
antes de abrir qualquer arquivo.** Os mesmos paths existem na raiz (`main`) e aqui; editar a árvore errada
**compila e commita sem um único erro**, e só aparece na integração.

## 2. O que JÁ está na GPU (não reconstrua)

| | onde | resultado medido |
|---|---|---|
| **O compositor de camadas** | `ph2d-render/src/layer_compositor/` | **66–107×** na composição, até **885×** no arrasto de ajuste |
| **Máscara e clipping como ops** (Onda 1) | `layer_compositor/ops.rs` | o documento comum saiu da CPU |
| **O orçamento vem do dispositivo** (Onda 2) | `layer_compositor/mod.rs` | 8 → **16 camadas** a 4K |
| **A luz do impasto** (2026-07-18) | `ph2d-render/src/impasto_light.rs` | `worst delta 0`, **0 de 16384 bytes** diferem, 5 materiais |
| **Upload parcial da região suja** (5b) | compositor | — |
| **A máscara na via parcial, upload cheio** (5c) | `paint/stamp_route.rs` + bridge | — |
| **A pintura parou de copiar o canvas por move** (5a) | `tool/mod.rs`, `layers/preview.rs` | O(canvas)/frame → O(pegada) |

## 3. A sua missão: **Onda 3** — os passes que já são maps puros

Na ordem que a MEDIÇÃO deu (doc 25 §7), não em outra:

| # | passe | custo medido hoje |
|---|---|---|
| 1 | **composição do impasto** | 9,4 ms a r=220 |
| 2 | **bake do pen-up do impasto** | 31,6 ms a 4096² |
| 3 | **wash óptico do watercolor** | 8,6 ms/move a r=220 |
| 4 | **bake do watercolor** | 10 ms |
| 5 | sculpt / deform / smear | 3–8 ms/move |

Todos operam sobre um **retângulo delimitado, sem redução entre pixels** — a forma canônica de um compute
pass, e exatamente o que a luz do impasto já provou portável.

⚠️ **O *fold* fica na CPU, e isso é decisão fechada** (a luz a tomou em 18/07): *quais camadas, em que
z-order, `Add`/`Level`, `impasto_depth`, o traço vivo e o teto de vidro* rodam na CPU e chegam ao device
como planos prontos. Um shader que os re-derivasse seria **a segunda resposta a "como camadas de tinta se
empilham"**, divergindo no único lugar onde ninguém lê um número: uma screenshot.

**Re-meça antes de começar** (os números acima são de 23/07 e a linha mexeu no caminho de preview desde
então):

```fish
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  cargo test -p ph2d-tool-painter --release measure_ -- --ignored --nocapture --test-threads=1
```

## 4. O template de paridade (copie o da luz, não invente outro)

`crates/ph2d-render/src/impasto_light.rs` + `crates/ph2d-render/tests/impasto_light_gpu.rs` são o
precedente **e a régua**:

1. ⚠️ **"Bit-a-bit" NÃO é a política deste projeto** — o compositor declara que runtime não é bit-idêntico
   entre backends (FMA). O template é: **literais exatos por gate CPU-only** + **ε documentado por gate
   `#[ignore]` de device** contra o kernel canônico.
2. ⚠️ **Um limite de MAGNITUDE sozinho não basta.** Tirar o `+0.5` do `quantise` movia **2375 bytes por UM
   nível** e passava sob um limite de 2 ⇒ o gate conta **TAMBÉM quantos bytes diferem**
   (`MAX_DIFFERING_BYTES = 16`). *Quão longe* e *quantos* são perguntas diferentes.
3. **Transcendental sobe pronto:** o LUT especular é enviado como tabela, então `powf` — o único
   transcendental do modelo — nunca roda no device. Mesma disciplina para o seu passe (HR-5).
4. **Gate e2e sobre o produto REAL**, não sobre o kernel: `try_drive` → elegibilidade → flatten →
   compositor → luz → premul → slot, com readback do dispositivo.

⚠️ **Três regimes de determinismo convivem no módulo e o seu port tem de dizer a qual pertence**
(doc 25 §6): (a) **fingerprint de sessão bit-exato** (`ph2d-wet-paint`, o port do JS); (b) **gates de
aparência** (o template acima); (c) **replay-hash do CI** (a disciplina "sem rayon", com exceções por ADR —
o [ADR-0109](architecture/decisions/0109-rayon-exception-watercolor-composite.md) é uma delas).

## 5. O que NÃO fazer (já decidido, com medição)

| | por quê |
|---|---|
| **Não comece pela residência do canvas** (Onda 5) | são **211 referências a `canvas_rgba` em 51 arquivos**, e o perfil **não aponta para lá** hoje. O compositor GPU já convive com o canvas CPU-residente (upload por-camada com cache versionado, **zero readback**). Propor isso antes de colher a Onda 3 é a otimização prematura que a memória do projeto proíbe. |
| **Não toque no Wet Paint** (Onda 4) | 🔴 **exige decisão do Enio primeiro**: *o fingerprint do port JS continua sendo o contrato, ou o produto passa a ser o dono da física?* Se for a segunda, o passo 1 é **Jacobi na CPU com rayon** (horas, sem WGSL) e só depois a GPU. Sem essa resposta não há código a escrever. |
| **Não porte o depósito Digital** | ⚪ já é footprint-bound a ~1 ms; o ganho não paga a residência que exigiria. |
| **Não re-derive o fold no shader** | §3 acima. |

## 6. Peças soltas de melhor razão custo-benefício (se quiser um começo curto)

- **`GradientMap` é literalmente um LUT de 256 entradas** ⇒ **já cabe na máquina de `adj_luts`** que
  `Curves`/`Levels` usam. É a peça de melhor razão custo-benefício entre os 6 ajustes ainda recusados
  (`ColorBalance`, `GradientMap`, `PhotoFilter`, `SelectiveColor`, `ChannelMixer`, `BlackAndWhite`).
  ⚠️ Nem todos cabem no orçamento de 3 escalares do `AdjParams` — PhotoFilter e BlackAndWhite querem mais.
- **Grupo mascarado/clipado** exige fechar o buraco na CPU primeiro.
- **Máscara de ajuste ESPACIAL** ainda cai na CPU: o passo de combine do pass-graph não tem entrada de
  máscara.
- Subir o orçamento além de **1 GiB** exige **alocação falível** (`push_error_scope(OutOfMemory)`), não um
  literal maior.

## 7. Aberto na wave anterior (NÃO é sua missão, mas não a contradiga)

- 🔎 **O endurecimento da borda da MÁSCARA** (3,53 px numa passada → 1,38 em quinze) segue **ABERTO**. As
  **duas** leis de acúmulo já foram tentadas e cada uma tem seu artefato (produto = endurece · envelope =
  contas) ⇒ **a cura não é a lei da cobertura** (doc 25 §13.10.4, [BUGS #17](Painter/BUGS_painter.md)).
- **Custo do pen-down de um traço protegido:** 7,5 ms @2048² (contra 3,3 sem proteção) e 24,3 @4096². O
  clone canvas-sized é amortizado pela proteção inteira; o **move** é plano na tela e gateado por razão. A
  receita da wave de perf (semeadura lazy por TILE + reuso da alocação) está na §13.12.5. ⚠️ **Se a sua
  Onda 3 tocar esse caminho, essa dívida vira sua** — meça antes.
- Métodos de SHAPE em modo máscara não pintam nada (pré-existente).
- Bug #11 (Per-Layer Color, linhas retangulares intermitentes) — dormente.

## 8. Como fechar

1. **Gate batched 1× no fim** (`scripts/nextest-impacted.sh` + clippy `--all-targets` + os DOIS gates de
   LOC — o `shells/desktop/tests/file_loc_caps.rs` **não** roda com `cargo test -p`).
2. ⚠️ Os gates de GPU são `#[ignore]` e **precisam de adapter**: rode-os explicitamente
   (`-- --ignored`) na RTX. Sem adapter eles fazem *skip gracioso*, que **não é verde**.
3. **Cena de smoke com números MEDIDOS** — o Enio smoka olhando, e uma cena que afirma o que a medição
   desmente já aconteceu duas vezes neste módulo.
4. Handoff de integração (DIRETRIZ §1.5.9) e **PARE**. ⛔ **Integração e ship só por ordem EXPLÍCITA do
   Enio** (`CLAUDE.md` §0.7).

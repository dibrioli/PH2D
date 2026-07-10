# HANDOFF 09 — Família pulse: matar o "clock hack", criar uma fonte de sinal de verdade, e arrumar os nomes

> Para o **próximo implementador** que continua a linha `line/MotionNodes`.
> Escrito depois de o Enio apontar dois problemas reais nos nós novos (pulse.threshold /
> pulse.counter / motion.strobe). Leia inteiro antes de tocar código. Contexto do estudo
> original: docs 06 (pulse) e 08 (counter). Referência-mãe de nomes: MiniCavalryV2 `DOCS/_NODES.md`.

## 0. TL;DR

Os três nós novos **funcionam** (Schmitt correto, wrap/zigzag correto, envelope correto — tudo
com teste falsificado). O que está **mal** é o entorno:

1. **P0 — o "relógio" é um hack.** A cena default fabrica a batida oscilando o canal **Rotation**
   (um canal de transform invisível em bolinhas) e passando isso num `pulse.threshold`. Resultado:
   nada "roda", e **se você troca o canal do threshold a animação para** (o clock escreve em Rotation,
   o threshold lê Rotation — acoplados por um canal invisível). **Causa raiz: o módulo motion não tem
   domínio de SINAL/VALOR nem uma FONTE de pulso.** O jeito canônico (MiniCavalry `lfo`, Max `metro`,
   TD `LFO CHOP`) é ter um gerador de batida que emite pulso direto, sem tocar em canal de transform.
2. **P1 — nomes.** `threshold` e `counter` batem com o MiniCavalry *em nome*, mas: (a) o `counter`
   **faz mais do que contar** (ele empurra um canal — o counter do MiniCavalry é REDUTOR puro), e
   (b) os prefixos `pulse.` vs `motion.` estão inconsistentes (o strobe consome pulso e é `motion.`;
   o counter também mexe no stream e é `pulse.`).

**Sua missão:** P0 (fonte de sinal real + matar o hack) e P1 (renomes/prefixo). P2 (o domínio de
valor completo) é estratégico e opcional — decida com o Enio. **NÃO integre nem pushe** (§6).

---

## 1. O bug que o Enio achou (o "clock-on-Rotation")

Cena default hoje (`shells/desktop/src/motion_demo_strobe.rs`):

```text
grid → move → tint → counter → strobe → output
             grid → clock(motion.oscillator, channel=Rotation) → threshold(channel=Rotation) ⟲
                                                                → { counter.pulse, strobe.pulse }
```

- `clock` é um **oscillator escrevendo uma senoide no canal Rotation**. Rotation não é renderizada
  numa bolinha → **"o que roda?" nada.** É um sinal disfarçado de transform.
- `threshold` lê **o mesmo canal Rotation** e dispara na subida. Se você muda `threshold.channel`
  para X/Y/Size, ele passa a ler um canal que ninguém anima (a grade é estática ali) → **nunca cruza
  → nunca dispara → counter e strobe congelam.** A animação "para". Não é bug de código: é o
  acoplamento invisível `clock.channel == threshold.channel` vazando pra UI.

**Isto NÃO deveria existir.** Um relógio não é "oscile um transform e detecte o cruzamento".

## 2. Causa raiz — falta o domínio de SINAL/VALOR e uma FONTE de pulso

Fatos confirmados no código:

- `motion.oscillator` tem **uma** saída: `INST_VEC2` (Frame). **Não emite pulso.** (No MiniCavalry o
  oscillator emite `pulse(pulse)` no zero-crossing, e o `lfo` emite `pulse` por ciclo — por isso lá
  você **nunca** precisa de threshold pra fazer batida.)
- Não existe **nenhuma** fonte de sinal/valor no módulo motion (`ls crates/ | grep -iE 'lfo|beat|metro|time'`
  → nada). Todo behaviour lê/escreve canais de transform (X/Y/Rotation/Size). O tipo `pulse`
  `(Instances, Scalar, Event)` existe, mas **o único jeito de PRODUZIR um pulso é o `pulse.threshold`
  lendo um canal de transform.** Não há um "gerador de batida".

Ou seja: a família pulse foi construída com produtor+consumidor, mas **sem a fonte** que os apps de
verdade têm. A demo tapou o buraco abusando de Rotation.

## 3. O que os apps intuitivos fazem (evidência — `DOCS/_NODES.md` do MiniCavalryV2)

MiniCavalry separa **Utility** (sinais/valores em sockets `value`/`pulse`) da **Transform/Behaviours**
(canais x/y/rotation/scale). O relógio mora na Utility, não num transform:

| Papel | MiniCavalry | Max/MSP | TouchDesigner | Rive |
|---|---|---|---|---|
| **fonte de batida** | **`lfo`** (out: value + **pulse** por ciclo) · **`loopSequencer`** (step-seq com pulse) · **`time`** | **`metro`** | **LFO CHOP**, **Beat CHOP** | trigger por tempo/estado |
| value→pulse | **`threshold`** (Schmitt, out 0/1 + pulse) · `pulseOnChange` · `compare` | `>` `change` `edge` | Logic/Trigger CHOP | number condition |
| pulso→valor (REDUTOR) | **`counter`** (trigger+reset → **value**; **NÃO** mexe em canal) | `counter` | Count CHOP | data-binding |
| pulso→congela valor | `sampleHold` | `sah` | Hold CHOP | — |
| roteia stream | `switch` | `gate`/`switch` | Switch/Select | — |

Combo canônico do MiniCavalry pra "trocar a cada beat": **`LFO → Counter → SampleHold → ColorArray`**.
Repare: **o LFO emite o pulso; o Counter só reduz pra um número; um nó SEPARADO aplica** (ColorArray).
Nada de "oscile rotation e detecte".

**Conclusão de nomes:** `threshold` (MiniCavalry o chama "Threshold (Limiar com Schmitt)") e `counter`
(idem "Counter") são nomes *legítimos*. O problema não é o nome do threshold — é **faltar o `lfo`/`beat`**
e o **counter fazer coisa demais**.

## 4. As correções (em ordem de prioridade)

### 4.1 P0 — criar uma FONTE de pulso e matar o hack

Crie um nó **`pulse.beat`** (drop-crate `ph2d-node-pulse-beat`, espelhe `ph2d-node-pulse-threshold`):

```
pulse.beat  (Effect::Pure, Clock::Frame)
  in    : (Instances, Vec2, Frame)   — só pra saber N e passar o stream adiante
  state : (Instances, Scalar, Event) — pre self-loop: último índice de ciclo (borda)
  out   : (Instances, Scalar, Event) — PULSE: 1.0 no tick em que o ciclo vira
params:
  period  segundos por batida (default ~1.0)   [ou bpm, se preferir a linguagem de música]
  offset  deslocamento de fase em s (default 0)
```

- Semântica: `k = floor((t - offset)/period)`; dispara no tick em que `k` incrementa (compare com o
  `k` anterior carregado no `pre` — mesmo padrão de borda do threshold/counter; edge-safe, determinístico,
  HR-5, sem transcendental). Uniforme entre linhas (batida global). Sem canal, sem Rotation.
- **Alternativa/legado:** dar ao `motion.oscillator` uma 2ª saída `pulse` (zero-crossing), como o
  oscillator do MiniCavalry. É útil, mas mexe no contrato do oscillator (2 outputs) — deixe como
  follow-up; o `pulse.beat` resolve a demo sem tocar em ninguém.
- **Reescreva a cena** `motion_demo_strobe.rs`:
  ```text
  grid → move → tint → step → strobe → output
                pulse.beat ⟲ → { step.pulse, strobe.pulse }
  ```
  Sem `motion.oscillator`, sem `pulse.threshold`, **sem canal acoplado**. Não há "channel" pra trocar
  e quebrar. Isto é o que torna a cena à prova de confusão.
- **`pulse.threshold` FICA** — mas para o uso REAL dele: "dispare quando um SINAL cruza um nível"
  (spring assentando, distância-do-mouse, áudio-reativo). Se quiser mantê-lo vivo na demo, faça uma
  2ª cena pequena e HONESTA que use isso (ex.: um valor que sobe/desce de verdade e o threshold marca
  o cruzamento). Nunca mais como "relógio".

### 4.2 P1 — nomes e prefixo

Decisão recomendada (leve ao Enio; renome tem custo de integração — ver §6):

| Hoje | Vira | Porquê |
|---|---|---|
| `pulse.threshold` | **fica** `pulse.threshold` | bate com MiniCavalry/TD; é value→pulse de verdade. Só pare de usá-lo como clock. |
| `pulse.counter` | **`motion.step`** (ou `motion.ratchet`) | o counter do MiniCavalry é **redutor PURO** (pulso→value, não mexe em canal). O nosso **empurra um canal por batida** → é um *behaviour visível*, não um contador. O nome tem que dizer a função (princípio do Enio). Deixe `pulse.counter` **livre** pro redutor puro de verdade quando o domínio de valor existir (P2). |
| `motion.strobe` | **fica** `motion.strobe` | "strobe" = flash, intuitivo. |
| — | **`pulse.beat`** (novo) | a fonte de batida que faltava. |

**Esquema de prefixo (o critério, pra não repetir a inconsistência):**
- `pulse.*` = **lógica abstrata de pulso/valor** (opera no TIPO pulse/value, não no visual):
  `pulse.beat`, `pulse.threshold`, e futuros `pulse.counter`(puro)/`pulse.gate`/`pulse.compare`/`pulse.sample_hold`.
- `motion.*` = **behaviour VISÍVEL** que modifica o stream de instâncias, podendo ser dirigido por pulso:
  `motion.strobe`, `motion.step`.

Assim `strobe` e `step` (ambos consomem pulso e mudam o visual) ficam juntos em `motion.*`, e a
"plumbing" de pulso fica em `pulse.*`. Consistente e o nome descreve a função.

> Renomear um node type mexe em: nome do type (`NodeTypeId::of("...")`), nome da crate, doc 08, SKILL
> §11.13, e **regenera `ph2d-node-registry-init`** (`cargo run -p ph2d-node-sync`). É a parte delicada
> pra integração — ver §6.

### 4.3 P2 — o domínio de SINAL/VALOR (estratégico, opcional — decida com o Enio)

A correção 100%-MiniCavalry é dar ao motion um **socket de valor escalar** separado dos canais de
transform, e uma mini-Utility: `pulse.counter` **puro** (pulso→value), `map_range`, `sample_hold`, e
behaviours que **LEEM um value input** pra dirigir um canal (em vez de cada nó embutir "aplica no canal").
Isso é o que deixa `LFO → Counter → SampleHold → ColorArray` existir. É um passo arquitetural **maior**
(um novo `PortType` de valor + roteamento) — **não** é pré-requisito de P0/P1. Registre como norte;
não faça junto sem o Enio pedir.

## 5. O que NÃO está quebrado (não reescreva)

- A **matemática** dos três nós: Schmitt (rise>fall, histerese no `pre`), wrap/clamp/zigzag por módulo
  euclidiano, envelope geométrico do strobe. Tudo com teste falsificado (desliguei o fix, vi vermelho).
- O **substrato**: `pre` como único fechador de ciclo, a membrana de clock (`Event`≠`Frame`), o
  multi-sink. O `pulse.beat` e o rename são mudanças de **superfície**, não de substrato.
- O documento default **já foi limpo** para 1 cena (grid rig + fountain saíram — commit `c0e1ef04`).
  Trabalhe em cima dessa cena única.

## 6. Como tratar a LINHA (Modo L) visando integração

Você continua em `line/MotionNodes` (worktree `Worktrees/line-MotionNodes`, ADR-0106/0107). Estado:
**4 commits à frente da main** (pulse → noise → counter → cleanup). Regras (inegociáveis do CLAUDE.md):

1. **Você NÃO integra e NÃO pusha. NUNCA.** Feche o trabalho, escreva o **handoff de integração**
   (DIRETRIZ §1.5.9) e **PARE**. Integração/ship só por ordem EXPLÍCITA do Enio, via agente integrador
   dedicado (§0.7 + [feedback](../../project-memory/feedback_integration_only_enio_command_end_of_all_lines.md)).
2. **cwd reseta entre comandos** — sempre `cd <worktree> &&` e caminhos ABSOLUTOS; mutação de arquivo
   por caminho absoluto (senão edita o `main` — [memória](../../project-memory/feedback_sed_relative_path_hits_primary_cwd.md)).
3. **Foundational você PODE tocar, mas projete pra isolamento.** O `pulse.beat` é drop-crate isolada
   (fácil). O **rename** (`pulse.counter`→`motion.step`) regenera `ph2d-node-registry-init` — esse é o
   **ponto de merge textual** que colide com outras linhas que adicionam nós. No handoff de integração,
   **anote explicitamente**: crate nova `ph2d-node-pulse-beat` (+ type `pulse.beat`), crate renomeada
   `ph2d-node-pulse-counter`→`ph2d-node-motion-step` (+ type `pulse.counter`→`motion.step`), e que
   `registry-init` foi regenerado. Isso deixa o integrador e as outras linhas resolverem o resíduo sem
   surpresa ([memória](../../project-memory/feedback_foundational_editable_design_for_isolation.md)).
4. **Rename é save-breaking SE algum documento salvo referenciar o type-name antigo.** Hoje o documento
   default é montado em código (nenhum save de usuário aponta pra `pulse.counter`), então é seguro —
   mas **confirme** que `MotionDoc` não serializa um doc default com o type antigo em nenhum fixture/
   snapshot de teste antes de renomear.
5. **Inner loop = só `cargo check -p <crate>`.** Gate batched 1× no fechamento (§7).

## 7. Checklist de fechamento (paridade ship.sh) + falsificação

- [ ] `pulse.beat` implementado + testes **falsificados** (desligue a detecção de borda → veja contar
      por tick; desligue o offset/period → veja não bater). Padrão dos docs 06/08.
- [ ] Cena `motion_demo_strobe.rs` reescrita com `pulse.beat`; teste de integração headless prova que a
      batida dirige step+strobe **sem** canal acoplado (cozinhe o sink real, asserte swing + steps).
- [ ] Rename propagado: crate, `NodeTypeId`, doc 08, SKILL §11.13, `cargo run -p ph2d-node-sync`.
- [ ] `bash scripts/nextest-impacted.sh` verde · 32 arch-gates (`cargo test -p ph2d-editor-core --tests`)
      · `cargo clippy --all-targets` 0 · `rustup run 1.95 cargo fmt` · `typos` (código; docs `.md` são
      exclude) · `cargo machete` · sweep HR-5 (`grep -nE '\.(sin|cos|tan|exp|sqrt|pow)\b'` → 0).
- [ ] Node count / sinks nos testes do shell atualizados.
- [ ] Smoke: comando pronto `cd <worktree> && cargo run -p ph2d-host-desktop` → tool Motion → a grade
      bate/varre/pisca **sem** nenhum "channel" pra quebrar.
- [ ] Handoff de integração escrito (DIRETRIZ §1.5.9) com os itens do §6.3. **PARE. Não integre.**

## 8. Ponteiros

- Cena a reescrever: `shells/desktop/src/motion_demo_strobe.rs` · doc default: `shells/desktop/src/motion_state.rs`.
- Nós existentes a espelhar: `crates/ph2d-node-pulse-threshold/` (produtor), `crates/ph2d-node-pulse-counter/`
  (a renomear), `crates/ph2d-node-motion-strobe/` (consumidor).
- Codegen registry: `tools/ph2d-node-sync` → `crates/ph2d-node-registry-init/`.
- Referência de nomes: `/home/enio/Documentos/Recursos/Nodes/MiniCavalryV2/DOCS/_NODES.md` (Utility:
  `time`/`lfo`/`counter`/`sampleHold`/`threshold`/`compare`/`switch`).
- Estudo original: docs 06 (pulse) e 08 (counter) nesta pasta.

# 113 — CICLO 8: FONTES & DADOS, de onde vêm as coisas

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) — sete passos, e o **tutorial é o smoke**.
> **Premissa do tutorial:** *«De onde vêm as coisas»*.
>
> ⚠️ Este doc é o do CICLO. O que ele mede vive nas sondas de
> [`motion_fontes_probe.rs`](../../crates/ph2d-app-motion/src/motion_fontes_probe.rs) e
> [`motion_bridge_fontes_costura.rs`](../../crates/ph2d-app-motion/src/motion_bridge_fontes_costura.rs).

---

## §1 — O grupo, DERIVADO da paleta

A sonda `the_source_palette_census` lista cada tipo oferecido com a categoria e o nome do cartão.
A categoria **`Source`** tem **20** tipos, e **catorze têm dono noutro ciclo**:

| ciclo | os da categoria `Source` que ele já contou |
|---|---|
| 1 (arranjo) | `grid` · `scatter` · `distribute_radial` · `fibonacci` · `lattice` · `voronoi` · `distribute_curve` |
| 5 (simulação) | `sim.spawn` |
| 6 (valor) | `value.pattern` |
| 9 (rig & corpos moles) | `boids` · `soft_body` · `verlet_rope` · `wave` · `rig.skeleton` |

Os seis que sobram são o grupo — e é o mesmo da linha do doc 103 §5 (uma **verificação**, não a
fonte):

| família | nós | cartão |
|---|---|---|
| `source.*` | 5 — `lsystem` · `object` · `shape` · `table` · `text` | `L-System` · `Object` · `Shape` · `Table` · `Text` |
| `motion.*` | 1 — `emitter` | `Emitter` |

⚠️ **O sub-grupo `source.*` é o que o artista vê** como *«as fontes que partem de uma coisa que EU
fiz»* — um desenho, um objecto da cena, um texto, um ficheiro, uma gramática. O emissor entra pela
pergunta do tutorial (*de onde vêm as partículas*), que o ciclo 5 não fez. Gate
`the_source_group_is_derived_and_not_empty` (piso de `6`, as duas metades).

---

## §2 — O RETRATO (sondas `audit_the_source_group` e `what_the_source_card_shows`, 2026-09-16)

```text
  nó                   | params | no cartão | device | portas | efeito
  ---------------------|--------|-----------|--------|--------|---------
  motion.emitter       |     24 |        15 |  sim   | 0->1   | Temporal
  source.lsystem       |     31 |        15 |  NAO   | 0->1   | Pure
  source.object        |      2 |         3 |  NAO   | 0->1   | Pure
  source.shape         |     42 |        10 |  NAO   | 0->1   | Pure
  source.table         |      1 |         2 |  NAO   | 0->1   | Pure
  source.text          |      6 |         8 |  NAO   | 0->1   | Pure
```

**O que ele diz, sem uma linha de código:**

1. ⛔ **Cinco de seis fora do dispositivo** — as cinco `source.*`. O emissor está lá.
2. ✅ **O vocabulário está LIMPO** — as três perguntas das sondas de vocabulário devolvem zero linhas;
   as chaves partilhadas (`angle`, `seed`, `size`) pintam o mesmo rótulo em todos.
3. ⚠️ **As diferenças `params`/`no cartão` são, na maioria, GATES DE MODO e cores** — o `source.shape`
   mostra `10` de `42` porque cada família de forma tem os seus controlos (`sides` num polígono,
   `star_depth` numa estrela…) e os oito `r/g/b/a` são duas linhas; o `source.lsystem` mostra `15`
   de `31`, com duas das quatro secções fechadas (`Leaves` e `Lean & Look`) (a lei de 31/08: *nenhum molde mostra um knob que a gramática
   dele não sabe ler*). ⏳ A confirmar pelo censo do alcance (W2).
4. ⚠️ **O cartão do `Shape` abre com `Own Fill`**, antes de `Shape` — a primeira linha de um cartão
   de fonte devia ser *qual forma*. ⏳ W2.
5. ✅ **A conferência já comparou os seis com as referências**: a folha
   [14 (SOURCE)](89_conferencia/14_source.md) e a [01](89_conferencia/01_distribuicao_emissao.md)
   (o emissor) estão a zero, e o `source.lsystem` tem auditoria própria
   ([doc 96](96_auditoria_do_lsystem_2026-08-31.md)). ⇒ *o poder que falta* (W3) parte do que elas
   deixaram RECUSADO, não de uma leitura nova.

---

## §3 — ⛔⛔ O ACHADO QUE DECIDE O CICLO: a costura cai NA FONTE — e ela ENVIA por quadro

A lei 1 do protocolo, com a sonda `probe_does_a_source_chain_stay_on_the_device` (cada nó à cabeça de
`X → scale → output`):

```text
  X -> scale -> output   | onde corre  | stages | costura
  motion.grid (controlo) | dispositivo |      3 | —
  motion.emitter         | dispositivo |      3 | —
  source.lsystem         | ⛔ CPU       |      2 | source.lsystem:0
  source.object          | ⛔ CPU       |      2 | source.object:0
  source.shape           | ⛔ CPU       |      2 | source.shape:0
  source.table           | ⛔ CPU       |      2 | source.table:0
  source.text            | ⛔ CPU       |      2 | source.text:0
```

⭐⭐ **É o espelho do ciclo 7, e o preço é OUTRO.** Lá a aparência era o ÚLTIMO nó e puxava a cadeia
inteira para a CPU; aqui a fonte é o PRIMEIRO, a costura cai nela e **o resto fica na placa** (`2`
estágios). O que se paga por quadro é:

1. **cozer a fonte na CPU** — barato quando ela não muda (as cinco são `Pure`, e o memo responde);
2. **ENVIAR o stream dela para a placa** — ⚠️ **SEMPRE**: `GpuCook::cook` faz *«one upload per
   boundary node»* a cada chamada, e nada guarda o envio do quadro anterior. Um texto parado, uma
   tabela, uma forma — o mesmo stream é copiado para a placa sessenta vezes por segundo.

⚠️ **E as cinco não pesam igual:** o `Object` e o `Shape` emitem **uma** linha (o custo deles está no
DESENHO e no carimbo — o item 10 do doc 103 §5.1, nas suas duas metades); a `Table`, o `Text` e o
`L-System` emitem **uma linha por elemento** (linhas do ficheiro, glifos, ramos), e é nesses que o
envio escala. ⚠️ Os «`0` elementos» da sonda acima são as fontes SEM conteúdo (sem ficheiro, sem
texto, sem objecto publicado) — o preço mede-se com conteúdo, na §3.1.

### §3.1 — O preço, medido pela ponte do produto

⏳ Sonda `measure_the_source_seam` (a MESMA cadeia pela `cook_gpu` do app, com o `publish_all` das
membranas antes, e a `motion.grid` do mesmo tamanho como controlo) — a correr com a máquina calma.

---

## §5 — A fila do ciclo

1. ⏳ **W1 — a costura** — o preço da §3.1 decide a cura.
2. ⏳ **W2 — o cartão e o alcance.**
3. ⏳ **W3 — o poder que falta** — a partir das recusas das folhas 14 e 01 e do doc 96.
4. ⏳ **W4 — a MEDIÇÃO.**
5. ⏳ **W5 — a cena e o TUTORIAL** *«De onde vêm as coisas»* — o smoke do dono.

# 110 — CICLO 6: VALOR & PULSO, o cérebro

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) — sete passos, e o **tutorial é o smoke**.
> **Premissa do tutorial:** *«Um número que manda em tudo»*.
>
> ⚠️ Este doc é o do CICLO. O que ele mede vive nas sondas de
> [`motion_valor_probe.rs`](../../crates/ph2d-app-motion/src/motion_valor_probe.rs); o mecanismo de
> cada wave vai para o handoff dela.

---

## §1 — O grupo, CONTADO

`value.*` + `pulse.*`, **derivadas do registry** e não de uma lista escrita à mão (uma lista aqui
envelhece em silêncio no dia em que um nó da família nasce — a lei que o ciclo 5 já pagou):

| família | nós |
|---|---|
| `value.*` | 26 |
| `pulse.*` | 9 |
| **total** | **35** |

⭐ **É o maior grupo até hoje** — os cinco anteriores tiveram `10 · 8 · 13 · 7 · 12`. Gate
`both_value_families_are_derived_and_not_empty` (piso de população nas duas metades).

---

## §2 — O RETRATO (sonda `audit_the_value_group`)

```text
  nó                        | params | no cartão | device | portas | efeito
  --------------------------|--------|-----------|--------|--------|--------
  value.attribute           |      1 |         1 |  sim   | 1->1   | Pure
  value.cursor              |      0 |         0 |  NAO   | 1->2   | Pure
  value.curve               |      5 |         6 |  sim   | 1->1   | Pure
  value.gain                |      2 |         2 |  sim   | 1->1   | Pure
  value.instance_field      |      4 |         1 |  sim   | 1->1   | Pure
  value.lfo                 |      9 |         8 |  sim   | 1->1   | Temporal
  value.map_range           |      7 |         6 |  sim   | 1->1   | Pure
  value.math                |      3 |         1 |  sim   | 3->1   | Pure
  value.median              |      2 |         2 |  sim   | 1->1   | Pure
  value.mix                 |      4 |         4 |  sim   | 3->1   | Pure
  value.noise               |     15 |        11 |  sim   | 1->1   | Temporal
  value.normalize           |      1 |         1 |  sim   | 1->1   | Pure
  value.number              |      3 |         2 |  NAO   | 0->1   | Pure
  value.pattern             |     11 |        12 |  sim   | 1->1   | Pure
  value.percentile          |      2 |         2 |  sim   | 1->1   | Pure
  value.quantize            |      3 |         3 |  sim   | 1->1   | Pure
  value.reduce              |      1 |         1 |  sim   | 3->1   | Pure
  value.slope               |      1 |         1 |  sim   | 1->1   | Pure
  value.smooth              |      3 |         3 |  sim   | 1->1   | Pure
  value.step                |      4 |         3 |  sim   | 1->1   | Pure
  value.switch              |      2 |         2 |  sim   | 5->1   | Pure
  value.table               |      2 |         5 |  NAO   | 1->1   | Temporal
  value.time                |      3 |         3 |  sim   | 1->1   | Temporal
  value.unary               |      1 |         1 |  sim   | 1->1   | Pure
  value.wave                |      5 |         5 |  sim   | 1->1   | Pure
  value.wrap                |      3 |         3 |  sim   | 3->1   | Pure
  pulse.adsr                |      9 |         9 |  NAO   | 2->1   | Pure
  pulse.beat                |      6 |         5 |  NAO   | 2->1   | Temporal
  pulse.compare             |      3 |         3 |  NAO   | 3->1   | Pure
  pulse.counter             |      4 |         4 |  NAO   | 3->2   | Pure
  pulse.level               |      0 |         0 |  NAO   | 1->1   | Pure
  pulse.on_change           |      2 |         2 |  NAO   | 2->1   | Pure
  pulse.sample_hold         |      0 |         0 |  NAO   | 4->1   | Pure
  pulse.signal              |      0 |         1 |  NAO   | 1->1   | Pure
  pulse.threshold           |      5 |         5 |  NAO   | 2->1   | Pure
```

**O que ele já diz, sem uma linha de código escrita:**

1. ⛔ **A família `pulse.*` está FORA do dispositivo — 9 de 9.** Nenhum kernel registado.
2. ⛔ **Três `value.*` também**: `cursor` · `number` · `table`. ⚠️ O `value.number` é **uma
   constante**, o nó mais simples do catálogo.
3. ⚠️ **Três cartões estão VAZIOS** (`value.cursor` · `pulse.level` · `pulse.sample_hold`): zero
   controlos. Num mundo em que *os params vivem no cartão* (§2 do 103), um cartão vazio é um cartão
   que não diz nada sobre si — e os três são nós cuja resposta é só o fio. **Decidir**, não assumir.
4. ⚠️ **`value.instance_field` mostra `1` de `4` params** e o `value.math` `1` de `3` — o resto está
   atrás de gates. Pode estar certo (modo) ou ser um controlo inalcançável; a folha de conferência
   do grupo (doc 89, folha 15) tem de ser recorrida nó a nó.

---

## §3 — ⛔⛔ O ACHADO QUE DECIDE O CICLO: dirigir um param CUSTA O DISPOSITIVO

A razão de existir de um `value.*` é **dirigir um param** de outro nó. O
[doc 102 §2](102_o_outro_patamar_plano_dos_nos_2026-09-04.md) afirmava-o por escrito (*«um param
dirigido OU um pino ≠ 0 ligado derrubam o nó para a CPU»*), e uma afirmação que decide um ciclo
inteiro **mede-se antes de se escrever uma linha**. Sonda
`probe_does_a_value_chain_stay_on_the_device`, sobre a cadeia `motion.grid → motion.scale →
motion.output`:

| cadeia | onde corre |
|---|---|
| sem valor nenhum (o controlo) | **dispositivo** |
| com `value.number` a dirigir o `amount` | ⛔ CPU |
| com `value.lfo` a dirigir o `amount` | ⛔ CPU |
| com `value.math` a dirigir o `amount` | ⛔ CPU |
| com `value.time` a dirigir o `amount` | ⛔ CPU |
| com `value.noise` a dirigir o `amount` | ⛔ CPU |
| com `pulse.beat` a dirigir o `amount` | ⛔ CPU |

⭐⭐⭐ **6 de 6, e o primeiro deles é uma CONSTANTE.** Não é propriedade de nó nenhum deste grupo —
é do **planeador**: ele recusa cegamente qualquer nó com param dirigido. ⇒ *ligar um fio de valor a
um grafo troca o caminho de `3,85 ms` pelo de `195,9 ms`* (a razão de `50,9×` do
[doc 98](98_auditoria_de_performance_2026-09-01.md), medida a 4,19 M objectos).

⚠️ **E isto inverte o que o ciclo 6 É.** Ele não pode começar por *«acrescentar controlos aos nós de
valor»*: com a lei 1 do §2 do protocolo (*«todo nó do grupo tem de dizer onde corre, e um que caia
para a CPU sai do ciclo com a razão nomeada»*), **o grupo inteiro cai, e a razão é uma só, e está
fora dele**. Acrescentar poder a 35 nós cujo uso derruba o dispositivo seria construir por cima do
defeito.

⏳ **O relógio deste A/B ainda não foi lido** — a workstation esteve a `load 50–75` durante a
auditoria, e *nenhuma leitura de relógio desta máquina vale nada acima de `~5`* (`CLAUDE.md` §5.0).
A sonda `probe_the_price_of_driving_one_param` está escrita e espera máquina calma. ⚠️ **A 1.ª
redacção dela mentia**: ela tirava o **mínimo de cinco corridas** sobre uma cadeia `Pure`, que
**memoiza** na primeira — as outras quatro custavam ~zero e a tabela imprimia `0,00 ms` para 102 400
objectos **dos dois lados**. *Uma régua que lê zero não está a medir a lei, está a medir a cache.*
Hoje cada amostra reconstrói a árvore e **lê** a saída.

---

## §4 — O VOCABULÁRIO (sonda `the_value_vocabulary`): 6 divergências

Mesma chave, rótulos diferentes — o artista vê dois nomes para a mesma pergunta:

| chave | rótulos | onde |
|---|---|---|
| `clamp` | `Clamp` · `Clamp Factor` | `value.map_range` · `value.mix` |
| `interp` | `Interp` · `Interpolation` | `value.pattern` · `value.table` |
| `mode` | **(sem hint)** · `Mode` | `value.attribute` · 8 outros |
| `phase_stagger` | `Phase Stagger` · `Stagger` | `pulse.beat` · `value.lfo` |
| `step` | `Increment` · `Step` | `pulse.counter` · `value.quantize` |

⚠️ **O `mode` do `value.attribute` não tem hint nenhum** — é a única entrada da tabela sem rótulo.
⚠️ **E o `step` é o caso INVERSO dos outros quatro:** ali as duas coisas são genuinamente
diferentes (*quanto somar por pulso* · *o tamanho da grelha*), logo a cura não é igualar o rótulo —
é a **chave** ser outra. *Duas perguntas diferentes com a mesma chave é o defeito; dois rótulos são
o sintoma.*

---

## §5 — A fila proposta (a decidir com o dono)

1. **W1 — AS LANES DO PLANEADOR** (§3). Já está **especificada ao detalhe** no
   [doc 102 §W1](102_o_outro_patamar_plano_dos_nos_2026-09-04.md): o param dirigido passa a ser um
   **uniform copiado no device** (um buffer `drv` por nó, `copy_buffer_to_buffer` de 4 bytes por
   fonte, zero passes a mais), e a porta ≠ 0 vira o **complemento de um `Compact`**. Gates de
   paridade bit-a-bit já nomeados, contrato **não congelado** (`gpu.rs` é side-metadata).
   ⇒ *Sem ela, todo o resto deste ciclo assenta num caminho que custa 50×.*
2. **W2 — a família `pulse.*` no dispositivo** (9 de 9 fora hoje), ou a razão nomeada e o preço
   medido para cada uma que fique.
3. **W3 — o cartão e o vocabulário**: as 6 divergências do §4, os três cartões vazios, e os params
   que o cartão não alcança.
4. **W4 — o poder que falta**, nó a nó, contra o estado da arte (a folha 15 da conferência + as
   referências), que é o passo 2 do protocolo por nó.
5. **W5 — a MEDIÇÃO** (passo 5) e **W6 — o TUTORIAL em PDF** (passos 6 e 7, o smoke do dono).

⚠️ **A ordem 1→2 não é preferência: é a lei 1 do §2 do protocolo.** Um grupo cujo uso normal
derruba o dispositivo não fecha um ciclo com «tem mais botões».

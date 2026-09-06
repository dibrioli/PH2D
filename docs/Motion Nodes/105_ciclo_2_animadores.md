# Ciclo 2 — ANIMADORES · «O tempo entra no grafo»

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) (os 7 passos e as 4 leis).
> **Ciclo anterior:** [doc 104](104_ciclo_1_arranjo.md) — ARRANJO, ✅ com o smoke do Enio em
> 2026-09-06.
> **Estado:** ⏳ aberto em 2026-09-06.

## §1 — O grupo (8 nós), e a premissa do tutorial

O ciclo 1 pôs muitos objectos na tela e eles ficaram **parados**. Este é o ciclo em que eles se
mexem — e a pergunta que o tutorial responde é a que um artista faz primeiro: *«o que faz isto
andar, e como eu mando nele?»*

| nó | nome na tela | o que ele é |
|---|---|---|
| `motion.oscillator` | Oscillator | a onda aplicada a um canal |
| `value.lfo` | LFO | a mesma onda, como NÚMERO (sem tocar em ninguém) |
| `motion.wiggle` | Wiggle | o tremor do After Effects |
| `motion.noise` | Noise | o campo de ruído, com o ESPAÇO dele |
| `motion.stagger` | Stagger | o atraso por elemento — a onda que corre pela fila |
| `motion.orbit` | Orbit | andar à volta de um ponto |
| `motion.spring` | Spring | perseguir com inércia |
| `motion.delay` | Delay | ver o passado de outro elemento |

⚠️ **A escolha não é «os oito que restam»:** os oito respondem à MESMA pergunta do artista, e é
isso que faz um tutorial e não uma lista. O `value.lfo` está aqui de propósito, ao lado do
`oscillator`, porque o [estudo dos outputs](100_estudo_dos_outputs_2026-09-04.md) mediu que o par
`value.lfo → motion.drive` é o `motion.oscillator` **ao bit** — *o artista tem de saber que são a
mesma onda vista de dois lados*, e nenhum dos dois docs dizia isso a ele.

## §2 — Passo 2: a AUDITORIA · o que o grupo DECLARA hoje

Medido por `audit_the_animator_group` (shell, `#[ignore]`) — a auditoria começa por medir o que
existe, nunca por uma lista do que eu acho que falta.

| nó | params | no cartão | device | portas | efeito |
|---|---|---|---|---|---|
| `motion.oscillator` | 13 | 10 | 🟢 sim | 2→1 | Temporal |
| `value.lfo` | 9 | 8 | 🟢 sim | 1→1 | Temporal |
| `motion.wiggle` | 10 | 7 | 🟢 sim | 2→1 | Temporal |
| `motion.noise` | 19 | 13 | 🟢 sim | 2→1 | Temporal |
| `motion.stagger` | 7 | 6 | 🟢 sim | 1→1 | Pure |
| `motion.orbit` | 5 | 5 | 🟢 sim | 1→1 | Temporal |
| `motion.spring` | **3** | 3 | 🟢 sim | 2→1 | Temporal |
| `motion.delay` | 6 | 6 | 🔴 **NÃO** | 2→1 | Pure |

⭐ **O grupo chega ao dispositivo — 7 de 8.** É o oposto do grupo do ciclo 1, que abriu com 2 de
10. ⇒ a lei nº 1 do protocolo (performance) não é o eixo deste ciclo; **o eixo é o PODER e a
legibilidade**, e o `motion.delay` é a única célula de device.

⚠️⚠️ **E a primeira versão desta tabela imprimiu `NÃO` para os OITO** — eu li a coluna do
`NodeManifest::lowerings`, que é o que o **nó** declara saber baixar sozinho; o caminho do
dispositivo é **side-metadata no registry** (`register_gpu_kernel`), exactamente como toda a
outra lei desta casa. *Uma coluna de auditoria lida da declaração errada é uma tabela de dívida
fabricada* — e esta teria aberto o ciclo a escrever oito kernels que já existem.

### §2.1 — Os candidatos que a tabela nomeia (⏳ por confirmar contra as referências)

1. **`motion.spring` tem TRÊS params** — a mola é o animador com mais poder por knob de todo o
   grupo e é o que menos oferece. O estado da arte (o *Spring* do Cavalry, o `spring()` do AE, o
   *Rigid Body Spring* do Blender) separa **rigidez · amortecimento · massa** e oferece um
   **repouso** que não é a origem.
2. **`motion.delay` é o único fora do dispositivo** — e é `Pure`, o que torna a pergunta
   *«porquê?»* respondível: ele lê o passado de OUTRO elemento, o que é um `gather` e não um mapa
   por-elemento.
3. **`motion.noise` mostra 13 de 19 params no cartão** — a maior diferença do grupo. Ou seis
   estão gateados por modo (legítimo), ou há knobs que o cartão não alcança. ⚠️ A medição do
   ciclo 1 (`no_param_the_panel_offers_falls_off_the_card`) responde a isto e tem de ser corrida
   sobre este grupo antes de qualquer veredito.
4. **O par `oscillator` / `lfo`** — o [estudo dos outputs](100_estudo_dos_outputs_2026-09-04.md)
   mediu que `value.lfo → motion.drive` é o `motion.oscillator` **ao bit**. O tutorial tem de
   dizer isso: são a mesma onda vista de dois lados, e o artista escolhe entre elas por *«quero
   um número»* contra *«quero mexer num canal»*.

## §3 — Passo 5: a MEDIÇÃO (⏳ a fazer)

## §4 — O tutorial (⏳ a fazer)

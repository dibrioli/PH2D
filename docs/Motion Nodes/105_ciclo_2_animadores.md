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

## §2.2 — O que cada nó OFERECE hoje (a lista, para «falta X» não ser palpite)

Medido por `what_each_animator_offers`.

| nó | portas | params |
|---|---|---|
| `motion.spring` | `in`, `state` → `out` | `channel`(X\|Y\|Rot\|Size\|Pos XY) · `tension` 0,5..60 · `friction` 0,1..20 |
| `motion.orbit` | `in` → `out` | `pivot_x` · `pivot_y` · `angle` · `speed` −720..720 · `carry_rotation` |
| `motion.stagger` | `in` → `out` | `channel` · `min` · `max` · `ease_curve`(9) · `ease_dir`(3) · `offset` · `reverse` |
| `motion.delay` | `in`, `state` → `out` | `channel`(6) · `mode`(Delay\|Average\|Blend) · `ticks` · `ticks_down` · `max_step` · `max_accel` |

---

## §3 — O PLANO, wave a wave

⚠️ **A ordem é por VALOR PARA O ARTISTA, e não por performance** — porque a auditoria já disse
que o grupo está no dispositivo (7 de 8). Cada wave traz portas, params, gates e o passo de
smoke, para nenhum agente ter de os redescobrir.

### W1 — ⭐⭐⭐ A MOLA LEGÍVEL (`motion.spring`)

**O problema, medido:** a mola tem **três** params e dois deles são constantes de física —
`tension = 8`, `friction = 1,5`. *Nenhum artista consegue prever o que `friction 1,5` faz*, e a
única forma de a afinar é tentativa e erro. É o animador com mais poder por knob do grupo e o que
menos se deixa dirigir.

⛔⛔ **E a hipótese óbvia está REFUTADA duas vezes, antes de qualquer código:** *«falta a MASSA»*
é falso. **(a)** Matematicamente ela é absorvida: `m·x″ + c·x′ + k·x = 0` dividido por `m` dá
`x″ + (c/m)·x′ + (k/m)·x = 0` — a mola tem **dois** graus de liberdade e um param de massa só
reescalaria os outros dois. **(b)** E ela já existe onde faz sentido: `inv_mass_at` lê a coluna
`inv_mass` que o `motion.pin_constraint` escreve — massa é **por elemento**, não um knob.
*Um terceiro knob redundante teria sido a primeira coisa que eu construiria sem medir.*

**O upgrade — a reparametrização de Apple/Framer, exacta:** a mesma equação, escrita nos dois
números que um artista sabe pensar — **quanto tempo** e **quanto salta**.

```text
ω₀ = √tension              (frequência natural)
ζ  = friction / (2·√tension)   (razão de amortecimento)

duration (s)  ↔  tension  = (2π / duration)²
bounce (−1..1) ↔ friction = 4π·(1 − bounce) / duration        (bounce ≥ 0)
                 friction = 4π / (duration·(1 + bounce))      (bounce < 0)
```

- `bounce = 0` ⇒ ζ = 1, **criticamente amortecida** (chega e pára, sem passar).
- `bounce > 0` ⇒ sub-amortecida, salta; `bounce = 1` ⇒ oscila sem parar.
- `bounce < 0` ⇒ sobre-amortecida, arrasta-se.

**Desenho:** um param `mode` (`Physics` · `Time`), com `ParamGate` a esconder o par que não é
lido — o molde do `uniform` do `motion.scale`. ⚠️ **`Physics` é o default e é BYTE-IDÊNTICO**:
nenhum grafo autorado se mexe.

**Gates:**
1. `the_time_mode_reproduces_the_physics_mode` — para `n` pares `(duration, bounce)`, converter e
   correr as duas leis dá a MESMA trajectória (a conversão é exacta, não uma aproximação).
2. `bounce_zero_never_overshoots` — a trajectória é monótona até ao alvo (ζ = 1).
3. `bounce_higher_overshoots_more` — o excesso máximo cresce com o `bounce`.
4. `the_physics_mode_is_byte_identical` — mutação: o modo novo não toca no caminho de omissão.
5. ⚠️ **Um gate de FRONTEIRA**: `duration → 0` e `bounce → ±1` não podem produzir `NaN` nem
   estourar o `MAX_STEPS` — a conversão divide por `duration`.

**Smoke:** duas fileiras com a mesma perseguição; em cima `Physics`, em baixo `Time` com
`Bounce = 0` e depois `0,5`. Cena nova. ⏳ **a fazer**

#### ✅ W1 FECHOU no código (2026-09-06) — e ela trouxe uma cura que ninguém tinha pedido

`law.rs` é a porta única (`physics_of`), lida pela CPU **e** pelo WGSL. `mode`/`duration`/`bounce`
apendados no fim do manifesto, `ParamGate` a esconder o par que não é lido, `duration` declarada
em **segundos**. **24 gates verdes.**

⛔⛔ **O gate de fronteira encontrou um buraco de estabilidade PRÉ-EXISTENTE:** o sub-passo
adaptativo guardava só a RIGIDEZ (`sub_dt²·tension < 0,05`) e não o ATRITO — e o Euler
semi-implícito só é estável enquanto `friction·sub_dt < 2`. Com os sliders antigos aquilo nunca
mordia (`friction 20 × dt 1/60 = 0,33`), então o buraco esteve lá desde sempre, **alcançável só à
mão**; o modo `Time` alcança-o pelo slider, e ali a mola **explodia**. ⇒ o sub-passo passa a ser
`min(√(STABLE/tension), FRICTION_STABLE/friction)`, nas duas rotas.

⭐⭐ **E o tecto da conversão é DERIVADO, não escolhido** (§0.0): a mola não pode pedir ao
integrador uma mola que ele não sabe integrar, e o número sai da aritmética das constantes DELE —

| | |
|---|---|
| passo mínimo | `MAX_DT / MAX_STEPS` = `0,1 / 64` = **1,5625 ms** |
| tecto da rigidez | `STABLE / passo²` = **20 480** |
| tecto do atrito | `FRICTION_STABLE / passo` = **640** |

⚠️⚠️ **E a afirmação que eu escrevi primeiro sobre a inércia era FALSA:** *«o termo do atrito
nunca é o menor»* — a `tension 0,5` com `friction 4,08` ele já manda (`0,245` contra `0,316`). O
que importa é se ele muda a **contagem de sub-passos**, que é o que a trajectória vê: nas **1 323**
células varridas (441 pares × três quadros, de 60 a 30 fps) não muda nenhuma. ⛔ **E há um canto
onde MUDA, nomeado em vez de escondido:** com `dt` no tecto (`0,1 s`, uma engasgada de 100 ms) a
mola mais mole e mais amortecida da faixa passa de **1 para 2** sub-passos — ali o passo único
punha `friction·dt = 2,0`, **exactamente** sobre a fronteira. *É uma cura naquele canto, não uma
regressão.*

⚠️ **A lei está escrita duas vezes** (Rust e WGSL) — inevitável num kernel; o que não é inevitável
é divergirem sem ninguém ver. O `the_wgsl_says_what_the_rust_says` compara o TEXTO termo a termo,
e confere que os dois tectos do WGSL são os números que a aritmética do Rust dá.

### W2 — ⭐⭐ A ORDEM DO STAGGER (`motion.stagger`)

**O problema:** o stagger tem `reverse` (um booleano) e é só isso — a onda corre da esquerda para
a direita, ou ao contrário. O que as referências oferecem é a **ORDEM** como uma escolha: *do
centro para fora*, *das pontas para o centro*, *aleatória*, *por posição em vez de por índice*.

**Desenho:** `order` (`Index` · `Reverse` · `From Center` · `From Edges` · `Random` · `By X` ·
`By Y`) + um `seed` gateado ao `Random`. ⚠️ **O `reverse` FICA** (todo documento autorado o tem);
`Reverse` no enum novo e o toggle são a mesma coisa — ⛔ ou o toggle sai com migração, ou ele
**compõe** com a ordem. *Duas portas para a mesma pergunta é como as duas divergem* ⇒ decidir na
implementação, com a contagem dos documentos afectados.

**Gates:** cada ordem produz uma permutação DISTINTA; `From Center` é simétrica; `Random` com a
mesma semente é determinista; o default é byte-idêntico.

### W3 — O `motion.delay` E O DISPOSITIVO

**O problema:** é o único do grupo fora do dispositivo — e é `Pure`.

**A pergunta a responder antes de escrever kernel:** ele lê o passado de **outro elemento**
(`ticks` atrás), o que é um **gather** e não um mapa por-elemento. ⇒ ou o canal de estado do
device suporta ler uma linha arbitrária de um buffer de histórico, ou a recusa é medida e
escrita. ⛔ **Não abrir com «escrever o kernel»** — abrir com a medição de quanto ele custa numa
cena real e de qual é o bloqueador exacto, como a auditoria 98 exige.

### W4 — O CENSO DO CARTÃO DO GRUPO

O `motion.noise` mostra **13 de 19** params no cartão. Correr
`no_param_the_panel_offers_falls_off_the_card` restrito a este grupo e responder: são gates de
modo (legítimo) ou params inalcançáveis?

### W5 — O PAR `oscillator` / `lfo` NO TUTORIAL

Não é código: é a **frase** que falta. O [estudo dos outputs](100_estudo_dos_outputs_2026-09-04.md)
mediu que `value.lfo → motion.drive` é o `motion.oscillator` **ao bit**, e nenhum doc diz isso ao
artista. O tutorial tem de o dizer, e o smoke tem de o mostrar (as duas cadeias lado a lado, a
mesma figura).

---

## §4 — Passo 5: a MEDIÇÃO (⏳ depois das waves)

## §5 — O tutorial (⏳ o último passo)

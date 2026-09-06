# Ciclo 2 — ANIMADORES · «O tempo entra no grafo»

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) (os 7 passos e as 4 leis).
> **Ciclo anterior:** [doc 104](104_ciclo_1_arranjo.md) — ARRANJO, ✅ com o smoke do Enio em
> 2026-09-06.
> **Estado:** ⏳ aberto em 2026-09-06 · **W1–W5 ✅ · medição ✅ · tutorial ✅** —
> falta o passo 7, o smoke do Enio, que é a aceitação do ciclo.

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

### ✅ W1 — ⭐⭐⭐ A MOLA LEGÍVEL (`motion.spring`)

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

### ✅ W2 — ⭐⭐ A ORDEM DO STAGGER (`motion.stagger`)

**O problema:** o stagger tem `reverse` (um booleano) e é só isso — a onda corre da esquerda para
a direita, ou ao contrário. O que as referências oferecem é a **ORDEM** como uma escolha: *do
centro para fora*, *das pontas para o centro*, *aleatória*, *por posição em vez de por índice*.

**Desenho:** `order` (`Index` · `Reverse` · `From Center` · `From Edges` · `Random` · `By X` ·
`By Y`) + um `seed` gateado ao `Random`. ⚠️ **O `reverse` FICA** (todo documento autorado o tem);
`Reverse` no enum novo e o toggle são a mesma coisa — ⛔ ou o toggle sai com migração, ou ele
**compõe** com a ordem. *Duas portas para a mesma pergunta é como as duas divergem* ⇒ decidir na
implementação, com a contagem dos documentos afectados.

#### ✅ W2 FECHOU (2026-09-06) — e a lista de SETE virou TRÊS por medição

`order.rs` é a porta única (`raw_at`), lida pela CPU e pelo WGSL. `order` + `seed` apendados,
`seed` gateado ao `Random`, default `Index` **byte-idêntico**. **27 gates verdes** na crate.

⛔⛔ **Quatro das sete entradas do plano caíram, e por duas razões diferentes:**

- **`Reverse` e `From Edges` são a COMPOSIÇÃO que já existe.** O `reverse` espelha o `raw`, e
  espelhar o *«do meio para fora»* **é** o *«das pontas para o meio»* — a mesma curva. ⭐ O
  desenho que fica é melhor que o das referências (o Cavalry lista as duas separadas):
  **três ordens × dois sentidos = seis comportamentos, com três entradas e um toggle que já
  existia.** O gate `from_edges_is_from_center_mirrored_and_needs_no_entry` mede a identidade **e**
  trava a lista em três, para ela não crescer sem medir.
- **`By X` e `By Y` pedem um POSTO, e um posto é uma ordenação** — saber que lugar `i` ocupa em
  `x` exige olhar todos os outros: não é um mapa por-elemento, e pô-lo aqui derrubava o nó para a
  CPU (a lei nº 1 do ciclo). ⚠️ **A saída existe e tem nome:** `motion.sort(key = X)` a montante
  com a `Index` aqui — e ela tem um efeito que esta porta não tem (**reordena as linhas**, então
  tudo a jusante vê outra lista). *Duas ferramentas, dois efeitos.*

⚠️ **E o `Random` NÃO é uma permutação, de propósito:** uma permutação pediria a mesma ordenação.
O que um stagger aleatório quer é *um atraso próprio para cada um*, e é isso que um hash de
`(i, seed)` dá — por elemento, sem olhar para ninguém, igual em toda corrida.

⚠️⚠️ **E o gate da paridade WGSL apanhou-SE A SI PRÓPRIO.** A 1.ª redacção comparava os quatro
mixers escritos em decimal **à mão** dos dois lados — e **três dos quatro estavam errados**. Ele
acusou o WGSL, e ao corrigir vi que a lista do teste tinha o mesmo defeito. *Um gate que compara
duas cópias à mão não prova nada: ele tem de DERIVAR um dos lados* — hoje ele formata os `u32` do
Rust e procura-os no texto.

### ✅ W3 — O `motion.delay` E O DISPOSITIVO

**O problema:** é o único do grupo fora do dispositivo — e é `Pure`.

**A pergunta a responder antes de escrever kernel:** ele lê o passado de **outro elemento**
(`ticks` atrás), o que é um **gather** e não um mapa por-elemento. ⇒ ou o canal de estado do
device suporta ler uma linha arbitrária de um buffer de histórico, ou a recusa é medida e
escrita. ⛔ **Não abrir com «escrever o kernel»** — abrir com a medição de quanto ele custa numa
cena real e de qual é o bloqueador exacto, como a auditoria 98 exige.

#### ✅ W3 FECHOU (2026-09-06) — e a medição DERRUBOU a linha acima, e depois deu um ganho de `7,5×`

⛔⛔ **A frase «ele lê o passado de OUTRO elemento» é FALSA** — cada elemento lê o **seu
próprio** passado. *Uma premissa afirmada sem abrir o nó é um palpite com cara de medição*, e ela
ia mandar a wave inteira atrás de um gather que não existe.

⭐⭐⭐ **O bloqueador real é o ANEL DE 32 FATIAS** — e ao medi-lo apareceu uma coisa muito melhor
que um kernel: o modo **`Blend`**, que é o **default** e aquele *para que o nó existe*, **não lê o
anel** (uma one-pole é uma soma pesada de TODO o passado, então ela carrega a própria saída
anterior, `dl_out` — uma coluna). Ele estava a juntar e a reescrever **32 colunas por tique** que
nunca abre.

| modo, 100 000 elementos | antes | depois | colunas de estado |
|---|---|---|---|
| `Delay` | `12,82 ms` | `12,85 ms` | 34 → 34 |
| `Average` | `13,89 ms` | `10,32 ms` | 34 → 34 |
| **`Blend`** (o default) | **`10,96 ms`** | **`1,46 ms`** | **34 → 2** |

**`7,5×` no modo comum**, e os outros dois byte-idênticos. ⇒ `ring::depth_for(mode, angular)` é a
porta: o anel é lido e reescrito só na profundidade que o modo abre.

⚠️ **E o `Blend` ANGULAR ainda guarda UMA fatia** — o desembrulho do ângulo compara com «um tique
atrás», e uma varredura que devolvesse zero para todo `Blend` partiria a rotação **sem nenhum
gate de contagem o ver**. O `the_angular_blend_still_keeps_its_one_slice` mede-o.

⛔ **E o anel dos outros dois NÃO encolhe com o `ticks`, de propósito:** seguir o knob faria a
profundidade mudar **durante o arrasto**, e subir o lag mostraria um histórico curto até se
voltar a encher — um transitório a cada mexida. *O ganho do modo comum é livre; este não era.*

⏳ **E agora o kernel é uma pergunta diferente:** com **duas** colunas de estado o `Blend` é um
mapa por-elemento, logo é candidato a device. Fica nomeado, com o número ao lado — mas depois do
`7,5×` ele deixou de ser a alavanca que era.

### ✅ W4 — O CENSO DO CARTÃO (2026-09-06): limpo, e virou um GATE DO CATÁLOGO INTEIRO

**A pergunta:** o `motion.noise` mostra 13 de 19 — são gates de modo (legítimo) ou params
**inalcançáveis** (a espécie que nenhum gate de registo apanha)?

**A resposta, medida sobre TODO o catálogo:** `810` controlos · `127` escondidos ·
**`0` sem explicação.** Cada um dos 127 é uma de três coisas, e o gate exige que seja:

| explicação | exemplo |
|---|---|
| um `ParamGate`/`ParamGateText`/`ParamGateAbove` que o nomeia | `metric` só existe na base `Cellular` |
| um canal de uma **COR**, consumido pela amostra | `motion.tint::r2` |
| uma row dentro de uma **secção que nasce fechada** | o `Space` do `motion.noise` |

⇒ `every_param_the_card_hides_has_a_declared_reason` — **catálogo inteiro, sem lista de
tolerância**. Mutação (esconder um param a mais no bridge): **15 nós** acusados pelo nome.

⚠️⚠️ **E a sonda acusou DUAS VEZES o produto errado antes de acertar — as duas vezes por ler a
declaração errada:**

1. A coluna do **dispositivo** saía do `NodeManifest::lowerings` e imprimiu **`NÃO` para os
   oito**; o caminho do device é side-metadata (`register_gpu_kernel`). *Teria aberto o ciclo a
   escrever oito kernels que já existem.*
2. O **`rotation`/`uniform` do `motion.noise`** foram acusados de *«sem explicação»* porque a
   sonda não sabia o que é uma **secção dobrada**. Eles estão a **um clique**, com o cabeçalho a
   dizer quantas rows esconde — o oposto de inalcançável.

*Uma tabela de dívida fabricada custa mais que a dívida: ela manda construir o que já existe.*

### ✅ W5 — O PAR `oscillator` / `lfo` NO TUTORIAL

**Não era código, e a medição já existia:** a sonda `the_tee_against_the_fused_node` (da wave dos
outputs) compara `value.lfo → motion.drive` contra `motion.oscillator` bit a bit. O que faltava
era a **frase** — e ela está escrita, com os números, na §8 do tutorial e na [§4.2](#42--o-par-oscillator--lfo-medido-a-w5)
daqui: **`0` de `10 000` objectos diferem**, os dois chegam ao dispositivo (2 passes contra 3), e
o fundido custa `6,43 ms` contra `10,77` a um milhão.

---

## §4 — Passo 5: a MEDIÇÃO do grupo (2026-09-06, `load 2,85 / 3,07` — §5.0 ok)

Sonda `measure_the_animator_group` (`#[ignore]`, na shell). **100 000 elementos** (`400 × 250`),
`--release`, melhor de 5 corridas, cada uma com `MotionState` e `Cook` novos.

⚠️ **Um animador não POSITA objectos: ele mexe nos que chegam** — então a régua deste grupo é
outra que a do ciclo 1. Ali media-se o nó sozinho; aqui mede-se `grade → animador → output`
**menos** `grade → output` (a linha de base: `0,32 ms` a frio, `0,00` em regime).

| nó | device | passes | frio | regime | objectos/ms |
|---|---|---:|---:|---:|---:|
| `motion.oscillator` | ✅ | 2 | **0,27 ms** | 0,26 ms | **377 199** |
| `motion.stagger` | ✅ | 2 | 0,78 ms | **0,00 ms** | 128 701 |
| `motion.orbit` | ✅ | 2 | 0,78 ms | 0,63 ms | 128 226 |
| `motion.spring` | ✅ | 2 | 0,89 ms | 1,00 ms | 112 250 |
| `value.lfo` (+ `motion.drive`) | ✅ | 3 | 1,02 ms | 0,85 ms | 97 622 |
| `motion.delay` | ⛔ | — | 1,08 ms | 1,02 ms | 92 434 |
| `motion.wiggle` | ✅ | 2 | 1,55 ms | 1,43 ms | 64 504 |
| `motion.noise` | ✅ | 2 | **4,47 ms** | 4,22 ms | **22 387** |

⭐ **O grupo inteiro cabe num quadro com folga:** o mais caro dos oito custa `4,47 ms` de `16,67`
a 100 000 objectos, **na CPU** — e sete deles nem lá correm quando a cadeia é reivindicada pelo
dispositivo, cujo tecto medido é `4,19 M` objectos em `3,85 ms`
([auditoria 98](98_auditoria_de_performance_2026-09-01.md)).

⚠️ **A coluna `device` é a reivindicação do PLANEADOR** (`plan(...).is_fully_gpu()`), como no
ciclo 1 — não um cronómetro de GPU. O que ela responde é a pergunta que a auditoria 98 mostrou
ser a que manda: *esta cadeia chega lá?* Sete chegam; o `motion.delay` não, e o preço dele é a
coluna `regime`.

### §4.1 ⛔⛔ Dois vermelhos que a própria medição apanhou, e nenhum era do produto

**(a) `0,00 ms` na linha de base e no `motion.stagger` — o memo a responder.** A 1.ª versão da
sonda cronometrava um tique adiantado e leu `1 250 000 000 objectos/ms`. Não era ruído: os dois
são `Effect::Pure`, e num tique seguinte **o memo devolve o resultado**. *É a mesma armadilha que
o ciclo 1 pagou, noutra forma* — ali era o segundo `cook` no mesmo `Cook`, aqui é o quadragésimo.

⚠️ **E a cura NÃO é «cozer sempre a frio», porque metade do grupo pede o contrário:** a
`motion.spring` e o `motion.delay` carregam estado por uma aresta `delayed`, e o **primeiro**
tique deles é a identidade **por desenho** — a frio mede-se o nó desligado. ⇒ a tabela traz as
**duas** colunas, e cada metade do grupo lê a sua:

- **frio** = o que o nó custa **quando tem de correr** (um arrasto de param, uma cena a abrir);
- **regime** = o que ele custa **por quadro** numa cena a andar.

⭐ E daí sai o número mais útil da tabela: o **`motion.stagger` custa `0` por quadro**. Ele é
`Pure` **e** a saída dele não depende do tempo — um atraso por elemento é um mapa fixo —, então
depois do primeiro quadro ele é de graça. *Isso não é um erro de medição: é a resposta.*

**(b) `advance_tick` — uma aresta `delayed` não se carrega sozinha.** A sonda cozia em `t`
crescente e nunca fazia o tique virar, então a saída de um tique nunca chegava à porta `state` do
seguinte: a `motion.spring` e o `motion.delay` respondiam **para sempre** o primeiro tique, que é
a identidade. ⚠️ **O mesmo defeito estava no gerador das figuras**, e ali ele era visível: a
figura do `motion.delay` saía **byte-idêntica** à cadeia sem o nó — *ele parecia um controlo
morto quando o morto era o meu laço*.

### §4.2 ⭐⭐ O par `oscillator` / `lfo`, medido (a W5)

Sonda `the_tee_against_the_fused_node`, `100 × 100`:

| a pergunta | a resposta medida |
|---|---|
| dão os mesmos bits? | **`0` de `10 000` diferem**, em `t = 0 · 0,1 · 0,37 · 1,234 · 7,5 · 33,3` |
| …e com um período que o `f32` não representa? | a `3 Hz` (período `1/3 s`) **`53` de `10 000`** diferem num instante, `max │dy│ = 3,05e-5 px`, **8 ULP** |
| os dois chegam ao dispositivo? | **sim os dois** — o fundido em **2 passes**, a tee em **3** |
| o que custam a 1 000 000 na CPU? | fundido **`6,43 ms`**, tee **`10,77 ms`** ⇒ **`1,7×`** |

⇒ a frase do tutorial: *são a mesma onda*. A diferença de bits não é da lei — é de `t·f` contra
`t/(1/f)`, e ela só aparece quando o período não é exacto em binário. O artista escolhe por
**«quero mexer num canal»** (um nó, mais barato) contra **«quero um número que muitos nós leem»**
(dois nós, um passe a mais).

---

## §5 — Passo 6: O TUTORIAL

**Fonte:** `tutoriais/src/02_animadores.html` · **PDF:** `tutoriais/02_animadores.pdf` ·
**figuras:** `tutoriais/fig/*.svg`, geradas por `dump_animadores_figures`.

⭐⭐ **As 13 figuras são a saída COZIDA dos nós** (doc 103 §3), e a tabela dos controlos sai de
`tutorial_table::derive` — **a porta única**, que é a mesma função de onde o painel e o cartão
tiram os números. ⚠️ Ela viveu dentro do gerador do ciclo 1 e **saiu de lá neste ciclo, antes de
existir uma segunda cópia**: *uma tabela de referência escrita duas vezes diverge em silêncio, e
o lado que envelhece é o que o artista lê.*

### §5.1 ⛔⛔ A régua das figuras errou QUATRO vezes, e cada erro tinha outro mecanismo

Um animador não tem forma: tem **movimento**. Nenhuma figura do ciclo 1 servia, e a régua de
*«esta figura mostra o nó a trabalhar?»* foi reescrita quatro vezes — cada uma por um defeito que
a anterior não podia ver.

| a régua que eu escrevi | o que ela deixou passar / acusou errado |
|---|---|
| «a figura não está vazia» (herdada do ciclo 1) | uma fila **deslocada em bloco** não está vazia e não desenha nada |
| «o deslocamento é grande» | a onda **quadrada** desloca metade da fila para `+A` e metade para `−A`: a *magnitude* é igual nos 41, e a figura mais legível do conjunto foi acusada |
| «a variação do deslocamento é grande» | a **órbita fecha a volta**: no último instante ela está onde começou, e um círculo inteiro percorrido lia `4,35` |
| «…medida contra a largura da fila» | o **campo de ruído** partilhava a moldura de uma fila oito vezes maior e era acusado de estar parado |

⇒ o que ficou: a variação do deslocamento **por eixo** (que tem sinal), contra a **moldura** da
figura (que é o que o artista de facto vê impresso), e para um rasto a pergunta é outra — *quanto
o rasto se afasta da fila parada*. ⚠️ **E a tabela imprime-se ANTES de a régua acusar:** uma sonda
que estoura na primeira figura má obriga a uma corrida por figura, e cada uma custa aqui um build
de release.

### §5.2 ⭐ A quarta maneira de olhar, que o ciclo 1 não precisou

Três não chegaram. Um **campo** coerente é *vizinhos a andarem juntos*, e uma nuvem de pontos
deslocados não diz de onde cada um veio: com feição pequena lê-se estática de televisão, com
feição grande lê-se uma grade quase parada — **e as duas leituras estão erradas sobre o mesmo
produto**. ⇒ a figura do `motion.noise` desenha o **deslocamento**: um traço de onde o elemento
estava até onde ficou, com o fantasma por baixo.

⚠️⚠️ **E a feição não é só o `scale`: são as OITAVAS.** O espaço do campo devolve
`(px·scale, py·scale)` e o `fbm` soma `octaves` camadas com `lacunarity = 2` — com as **3** de
omissão a camada mais fina tem `4×` a frequência da base, então uma grade que mostra `2,4`
períodos da base mostra `9,6` da última e **volta a ler-se aleatória**. A figura pede **uma**
oitava, e isso é uma coisa que o tutorial passa a dizer ao artista.

### §5.3 A figura da mola é uma CURVA NO TEMPO, e a afirmação dela é medida

As outras doze mostram uma fila num instante, e nenhuma delas conseguiria mostrar a diferença
entre *«chega e pára»* e *«chega, passa e volta»* — que é exactamente o que a W1 acrescentou. A
cena é um alvo que **salta** (uma onda quadrada) e três molas a persegui-lo, e o gerador **mede a
promessa antes de a desenhar**:

| mola | pico, sobre um alvo de `90` |
|---|---|
| `Bounce = 0` | **`90,0`** — não passa, e o gate reprova se passar |
| `Bounce = 0,6` | **`133,0`** (`+48 %`) — passa, e o gate reprova se **não** passar |

⚠️ **E a moldura sai dos DADOS.** A 1.ª versão fixou-a em `±1,45 × alvo` e o modo `Physics` — que
com os defaults `tension 8` / `friction 1,5` tem `ζ = 0,27`, bem mais saltitante que o
`Bounce = 0,6` — saiu **por fora do quadro**: a figura mostrava três curvas e cortava
justamente a que mais salta. *Uma figura que corta o caso extremo ensina que ele não existe.*

## §6 — Passo 7: o SMOKE (⏳ é do Enio, e é a aceitação do ciclo)


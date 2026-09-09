# 107 — CICLO 4: FOCO — quem é afectado (os CAMPOS)

> **Protocolo:** [doc 103](103_dinamica_dos_ciclos.md) — um grupo por ciclo, params **no cartão**,
> e o entregável (e o smoke) é um **tutorial em PDF**.
> **Premissa do tutorial:** *«Nem todos ao mesmo tempo»*.

O grupo decide **quem** um animador ou um deformador afecta, e **quanto**. Um `motion.falloff` sem
campo é um interruptor; com campo é um **pincel**.

| nó | o que é |
|---|---|
| `motion.falloff` | a máscara mais simples — círculo, rectângulo ou rampa, centrada onde se quiser |
| `field.box` | a caixa, com bordas macias e rotação |
| `field.radial_sweep` | a cunha / o anel / o relógio |
| `field.index_range` | *«do 3.º ao 10.º»* — por ordem, ou por qualquer atributo |
| `field.remap` | a curva que reescreve a máscara (o *Remapping* do C4D) |
| `field.combine` | dois campos, nove modos de mistura |
| `field.shape` | a GEOMETRIA como campo — a máscara é um desenho seu |

---

## §1 — Passo 2: A AUDITORIA

### §1.0 — ⛔ O catálogo deste grupo JÁ ESTÁ FECHADO, e reabri-lo seria refazer trabalho pago

A [folha 10 da conferência](89_conferencia/10_field.md) auditou estes nós contra **C4D Fields**,
**MOPs** e **Cavalry**, linha a linha com fonte: **19 linhas · P0 = 0 · P1 = 0 · P2 = 0**, 11
fechadas e **8 recusadas/refutadas**, re-medida em 2026-08-23. Entre as fechadas estão o
`strength` com sinal, o `inner_radius`, o `soft_angular`, o `clamp` em escada de 4 estados, o
`curve_offset`, o rank por atributo **sem reordenar o stream**, e os tectos derivados do `f32`.

⚠️ **Duas «espécies que faltam» famosas foram REFUTADAS por composição**, e estão medidas:
o **campo de ruído** é `value.noise(World) → motion.drive(Falloff)`, e o **campo linear em
qualquer ângulo** foi o `rotation` que o `motion.falloff` ganhou.

⇒ **A auditoria do ciclo 4 não é o catálogo.** Ela pergunta o que mudou desde 23/08 — e o que
mudou é a superfície: o **cartão** nasceu depois disso, e o grupo nunca foi lido com o olho do
ciclo.

### §1.1 — O RETRATO (`audit_the_field_group`)

| nó | params | no cartão | device | portas | efeito |
|---|---:|---:|:---:|:---:|---|
| `motion.falloff` | 8 | 7 | sim | 1→1 | Pure |
| `field.box` | 9 | 9 | sim | 1→1 | Pure |
| `field.radial_sweep` | 12 | 12 | sim | 1→1 | Pure |
| `field.index_range` | 6 | 6 | sim | 2→1 | Pure |
| `field.remap` | 13 | 11 | sim | 1→1 | Pure |
| `field.combine` | 3 | 3 | sim | 2→1 | Pure |
| `field.shape` | 4 | 4 | **NÃO** | 2→1 | Pure |

⭐ **6 de 7 no dispositivo** e **todo param escondido tem razão declarada**: o `rotation` do
`motion.falloff` está gateado às formas que **têm direcção** (`ParamGate { when: "shape",
values: [Rect, Linear] }` — um círculo é isotrópico), e os dois do `field.remap` (`steps` e
`curve_offset`) estão gateados ao modo de contorno que os lê. ⇒ **o censo do ciclo 2 passa neste
grupo sem uma linha nova.**

### §1.2 — ⛔⛔ O ACHADO: o gizmo de canvas conhece DOIS dos TRÊS campos espaciais

O [`field_gizmo`](../../shells/desktop/src/field_gizmo.rs) existe, e o doc-comment dele diz porquê:
*«arrastar `center_x`/`center_y` num slider é caçar a rotação; o idioma dos apps pro
(C4D/Cavalry/Houdini mograph) é uma **alça na tela**»*.

⇒ e a `spec_for` dele tem **duas** entradas: `field.box` e `field.radial_sweep`.

⚠️⚠️ **O terceiro campo espacial é o `motion.falloff` — e é o que o artista encontra PRIMEIRO**
(é o único do grupo no namespace `motion.*`, é o que os deformadores leem, e é o que qualquer
tutorial usa na primeira página). Ele tem **exactamente** o vocabulário que o gizmo fala:
`center_x` · `center_y` · `rotation` · `radius`.

⭐ **E a geometria já encaixa sem nada novo:** a `FieldSize::Disk { radius }` devolve a meia-extensão
`[r, r]`, que é **o disco** do `Circle`, **o quadrado de Chebyshev** do `Rect` (a lei dele é
`max(|dx|,|dy|)/radius`) e **o vão** do `Linear` (a rampa corre em `±radius`). *Uma spec serve as
três formas.*

⚠️ **A rotação tem de concordar com o painel, e há uma porta para isso:** o painel esconde a linha
`Rotation` num círculo, e a regra vive no registry (`ParamGate`), lida por
[`shown_params`](../../shells/desktop/src/render_loop/motion_bridge_params_visible.rs) — cuja
própria doc diz que *«a segunda cópia é exactamente como um param passa a aparecer num sítio do
app e não noutro»*. ⇒ a alça de rotação pergunta à **mesma** porta, nunca a uma cópia da regra.

### §1.3 — ⏳ O único fora do dispositivo: `field.shape`

Fica **nomeado** e por reconferir nesta janela (a mesma disciplina que dissolveu a recusa do
`motion.bend ▸ direction` no ciclo 3): a razão escrita é que *o canal de porta-template no device
só existe emparelhado com um `StreamOp::SourceRows`*. ⚠️ **§0.0 manda reconferir no GERADOR**, não
no doc-comment.

---

## §2 — O PLANO, wave a wave

### ✅ W1 — A ALÇA DO `motion.falloff` (2026-09-09)

`spec_for` ganhou a terceira entrada, e a `selected_field` é a **porta única** por onde o desenho,
o clique e o arrasto perguntam — um braço acende os três.

⚠️⚠️ **A alça de ROTAÇÃO fica, e o painel continua a esconder a linha dela num círculo — a
divergência é deliberada e foi MEDIDA.** O plano acima dizia o contrário (*«aparece exactamente
quando a linha `Rotation` aparece, pela porta `shown_params`»*), e concordar com o painel faria a
caixa **girar na tela e voltar atrás no quadro seguinte**: a view lê o param que o arrasto não
escreveu. ⇒ *um controlo que se mexe e desfaz é pior que um cujo efeito espera pelo modo.* O param
é real e guardado; num `Circle` não move um texel (o campo é isotrópico) e passa a valer no
instante em que a forma vira `Rect` ou `Linear`. ⛔ A terceira saída — **suprimir** a alça — pedia
um campo novo na `GizmoView`, que é partilhada por **todos** os gizmos do app.

#### ⛔⛔ E o gate que dizia medir os nomes era um ESPELHO

`spec_names_match_the_nodes` abre com *«os nomes TÊM de bater com os params reais dos nós»* e
compara-os com **strings escritas à mão ao lado**: um typo escrito nos dois sítios passa, e um
param renomeado no nó passa também.

⇒ **`every_name_a_spec_uses_is_a_declared_param_of_that_node`** pergunta ao **registry**, sobre a
população **derivada** (todo manifesto cujo tipo tem spec), com piso contra o vácuo. A spec que
nascer amanhã entra sozinha.

⛔ **Duas provas de mutação, e cada uma nomeia o que devia:** tirar o braço do falloff dá
*«só 2 nós com spec de gizmo — os três campos espaciais têm de a ter»*; um nome fantasma dá
*«motion.falloff: a spec do gizmo dirige `centro_x`, que o nó não declara»*.

### ✅ W2 — O CARTÃO do grupo (2026-09-09)

#### O VOCABULÁRIO está limpo — medido, não presumido

O achado §2.3 do ciclo 3 (*«seis vocabulários para onde é o centro»*) virou régua
(`the_field_vocabulary`, na porta partilhada): para cada param que **dois ou mais** nós do grupo
declaram, os rótulos distintos que eles pintam.

| param partilhado | rótulo | quem |
|---|---|---|
| `center_x` · `center_y` | Center X · Center Y | falloff · box · radial_sweep |
| `curve` | Curve | 5 nós |
| `invert` | Invert | 6 nós |
| `soft` | Softness | box · radial_sweep · index_range |
| `strength` | Strength | box · remap · combine |
| `radius` · `rotation` · `clamp` | Radius · Rotation · Clamp | — |
| ⚠️ `mode` | **Mode** (combine) · **Path Mode** (shape) | — |

⭐ **A única divergência é CORRECTA:** são perguntas diferentes (uma mistura dois campos, a outra
escolhe se o desenho conta cheio ou só o contorno), e o rótulo mais específico desambigua.

⚠️ **Mas o censo alargado ao repo inteiro achou uma incoerência REAL — e ela é de FORA deste
grupo, por isso não entra:** os selectores de modo de mistura chamam-se
**`Blend`** (`motion.output`, `motion.tint`) · **`Shadow Blend`** (`fx.drop_shadow`) ·
**`Flash Operator`** (`motion.strobe`) · **`Echo Operator`** (`motion.trail`) · **`Mode`**
(`motion.mixer`, `field.combine`). ⛔ Fica **nomeada com a medição** e não contrabandeada: só um
dos seis é do ciclo 4, e alinhá-los é uma wave do ciclo 7 (aparência).

#### ⭐ A SECÇÃO: o irmão desalinhado

O `field.radial_sweep` agrupa em **`Placement`** e **`Falloff`**, e escreve a lei ao lado: *param
sem grupo pinta antes de toda secção, e é ali que os essenciais devem estar — a razão de existir
do nó, e pô-la numa secção seria escondê-la atrás de um clique*. O `field.box` é o **mesmo tipo de
campo**, com o **mesmo vocabulário** — e pintava as nove rows em fila.

⇒ ele passa a falar a mesma língua: `Width · Height` soltos, `Placement` (centro + rotação),
`Falloff` (softness + curva + invert + strength).

⚠️ **O preço é de ESPAÇO e está medido:** uma secção aberta custa **+1 fileira**
(`band_len = params + sections`), então o cartão vai de `9` para `11` — abaixo das `12` que o
irmão já shipa. ⛔ **Por isso os cartões pequenos ficam em fila:** numa carta de 3, 4 ou 6 rows
dois cabeçalhos organizam menos do que ocupam.

#### ⛔⛔ E o gate apanhou-me a MIM antes de apanhar o produto

A 1.ª redacção da tabela deixava o `soft` **solto**, com a razão *«uma caixa que mascara com borda
macia»* — e o irmão põe-no em `Falloff`. ⭐ *Quando o objectivo é alinhar dois irmãos, a autoridade
é o irmão, não a minha leitura do que é essencial.*

⚠️⚠️ **E a 1.ª redacção do GATE não o teria apanhado:** ela comparava só os params que os dois
**agrupavam**, e a mutação que devolvia o `soft` para fora de toda secção **SOBREVIVEU** — um
param solto saía da população. *Um censo que só olha o que foi declarado é cego a uma omissão*, e
a omissão era exactamente a divergência que a wave veio curar. Hoje o «sítio» de um param
partilhado é o título da secção **ou `(solto)`**.

⛔ Duas mutações, cada uma a nomear o param e os dois lados: `soft` solto dá *«vive em `Falloff` no
field.radial_sweep e em `(solto)` no field.box»*; `rotation` trocado de secção dá o simétrico.

### W3 — `field.shape`: a recusa reconferida no gerador

### W4 — A MEDIÇÃO (passo 5) e o TUTORIAL (passo 6)

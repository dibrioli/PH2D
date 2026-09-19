# O PENTE ONDE ELE MORA — o plano, com o preço MEDIDO

> **Estado: ✅ IMPLEMENTADO em 2026-09-18, e a implementação REFUTOU o plano
> numa premissa e ACRESCENTOU uma terceira metade.** O que se ship está no
> [handoff §79](handoffs/HANDOFF_INTEGRACAO_line_sculpt3d_2026-09-17.md); este
> documento fica como o **diário do plano**, com a correcção abaixo.
>
> ⛔⛔ **A premissa que caiu:** o plano dizia que a H1 + H2 bastavam. Medido,
> elas chegam a `ΔQ +0,028`–`+0,064` na bola da cena de smoke — *abaixo da barra
> de `+0,0465` em dois dos quatro rumos, e a `30°` inalcançável com `k` nenhum*.
> ⭐ **O que carrega o alinhamento é a TROCA DE DIAGONAL**, que não estava neste
> plano: ela muda a direcção de uma aresta **a contagem constante**, logo não
> paga densidade nem afina triângulo — e leva a bola a `+0,10`–`+0,24` nos quatro
> rumos, contra os `+0,074` da lei que ela substituiu.
>
> ⛔ **Leia o §79.3 antes de pegar em qualquer coisa daqui** — ele tem SEIS
> hipóteses construídas, medidas e refutadas nesta implementação, três delas
> escritas *neste* documento como se fossem o caminho.

## §1 — A frase

**O alinhamento do pente não mora no deslocamento.** Uma lei que só move
vértices não o pode reproduzir; ele nasce de **partir e fundir arestas** dentro
de uma pegada que **anda** ao longo do traço.

Medido, com tudo igual e só o passe de refino a mudar:

| rotação do traço | **sem** refino | **com** refino |
|---|---|---|
| 0° | `+0,17681` | `+0,17212` |
| 22,5° | **`+0,00590`** | `+0,16977` |
| 45° | **`+0,01290`** | `+0,12172` |
| 67,5° | **`+0,03921`** | `+0,15668` |

⭐ A `0°` o traço **coincide** com a grelha da entrada, e ali *«alinhar ao
traço»* e *«regularizar»* são a mesma coisa — é por isso que a linha de cima
engana.

## §2 — As duas metades, e elas andam JUNTAS

### H1 — a lei de deslocamento passa a ser a AJUSTADA

```
por carimbo, para cada vértice na pegada:
    m ← centróide(anel VIVO) − p
    p ← p + queda(dist_ao_centro / R) · tangencial(m)      # um passo COMPLETO
```

| modelo | `cos` ponderado contra o campo do alvo |
|---|---|
| encaixe duro em 4 eixos (**hoje**) | `0,583` |
| só regularizar o raio médio | `0,886` |
| anisotrópico | `0,887` |
| **a ajustada** | **`0,9830`** |

⚠️ **O perfil radial que se mede NÃO é a queda** — é a saturação de
`1 − Π(1 − α·w)` sobre os carimbos que tocam cada vértice. *Ajustar um passo a
uma soma de muitos* é o que travou cinco tentativas.

**Onde:** [`ph2d-rake/src/lib.rs`](../../crates/ph2d-rake/src/lib.rs) (312 L),
o laço central. ⛔ **Ela sozinha REPROVA o gate da grade** (`Q −0,0076` contra
a barra `+0,0465`) — é por isso que as duas metades andam juntas.

### H2 — o refino passa a ser ENVIESADO pela direcção do traço

⭐ **O ponto de extensão já existe** e é `Sizing`, que o `refine_in_sphere_sized`
e o `collapse_in_sphere_sized` já recebem. ⛔ **Mas ele é ESCALAR:**

```rust
pub type Sizing<'a> = Option<&'a (dyn Fn([f32; 3]) -> f32 + Sync)>;
```

`posição → comprimento alvo`. Isso não exprime *«mais longo ao longo do traço,
mais curto através»*, que precisa da **direcção da aresta**. ⇒ a obra é torná-lo
**direccional** (`(posição, direcção) → comprimento`, ou uma métrica).

**Preço MEDIDO** (`crates/ph2d-mesh/`):

| | |
|---|---|
| sítios que AVALIAM o campo | **3** |
| assinaturas que o carregam | **5** |
| chamadores do `_sized` fora da crate | **2** (um é a `ph2d-remesh-iso`) |

⇒ contido. *Não é reescrever o passe de refino* — é alargar um parâmetro que
ele já tem, com os chamadores actuais a passarem o campo isotrópico de sempre.

## §3 — O que cada metade compra, e o que ela custa

| | compra | custa |
|---|---|---|
| **H1** | o campo de deslocamento de `0,583` para `0,983` | reprova o gate da grade enquanto a H2 não existir |
| **H2** | o alinhamento onde ele de facto mora | `Sizing` direccional (3 + 5 + 2), e os gates que hoje medem a grade mudam de sujeito |
| **as duas** | o pente que o alvo tem | **o TACTO muda** — decisão do dono |

## §4 — ⛔ O que NÃO se reconstrói

As cinco famílias direccionais na lei de deslocamento, todas medidas contra o
campo do alvo e **todas piores**: encaixe duro (`0,583`) · rodar mantendo o
comprimento (`0,154`) · quatro dobras suaves (`0,750`) · mistura contínua
(monótona a descer de `0,886`) · anisotrópica (`0,887`). Mais: **mais
varreduras** (a amplitude sobe, o cosseno DESCE) e a grelha de `84` células de
peso direccional, cujo óptimo é `k = 0,00`.

⇒ ***o termo direccional não existe na lei por-vértice.*** Está registado no
cabeçalho de [`ph2d_rake::pentear`](../../crates/ph2d-rake/src/lib.rs).

## §5 — ⏳ O que fica por medir

- **O termo de `~30 %` fora do eixo:** o alvo move mais, em direcções que a
  relaxação isotrópica não prevê (`cos` `0,983` a `0°` contra `0,711`–`0,811`
  nas outras três), e **esse excesso não é alinhamento** (o `ΔQ` dele é zero).
  ⚠️ *Uma lei isotrópica ajustar-se PIOR quando o traço roda é ela própria a
  prova de que falta um termo.* Na catraca, isto lê-se como o erro do pente a
  **dobrar** fora do eixo (`2,8e-2` a `0°` contra `6,1e-2` a `67,5°`) — com o
  controlo ao lado, porque com o pente desligado as quatro leem `1,6e-4`.
- **A barra do gate da grade** saiu de células `CONSTANT` (com refino). Quando
  a H2 existir, ela passa a medir o sujeito certo e o número tem de ser
  reconferido, não herdado.

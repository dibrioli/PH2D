# 39 — `SIZE_IDENTITY`: um nó no seu default não pode redimensionar a cena — nota-ADR

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** correção de produto (achado do doc 38 §6)
**Status:** corrigido, guarda provada (mutação), **pendente smoke do Enio**
**Contrato congelado encostado:** **nenhum** (8/2/1) · **Foundational tocado:** `ph2d-nodegraph::attr` (1 `const` **aditiva**)

---

## 1. O bug

Soltar um **`motion.scale` no seu DEFAULT** (`amount = 1.0` — a identidade, por definição um no-op) sobre um
grid fazia **cada quad pular de `0.4` para `1.0`: 2,5× maior**, sem o artista ter mexido em nada.

E não era só o Scale: **todo** nó que materializa a coluna `size` — o canal *Size* do `oscillator`, `wiggle`,
`noise`, `step`, `stagger`, `drive`, mais o `strobe` — carregava a mesma surpresa.

## 2. A causa: duas metades **auto-consistentemente erradas**

O módulo inteiro concorda que **a identidade de `size` é `[1,1]`**. Está escrito nos leaves, textualmente:

> *"Unit scale is the identity of `size` — never `[0,0]`."*

Faz sentido: um nó só escreve a coluna que ele muda, então quem **materializa** `size` numa stream que não
tinha precisa partir de uma base — e a base tem que ser a escala unitária (preencher com `[0,0]` colapsaria
todo elemento a nada).

Quem estava fora de passo era **o shell**: ele lowerava com `default_size = [0.4, 0.4]` — um número
**cosmético**, escolhido pra que o documento cru rendesse "pontinhos distintos com folga" em vez de uma faixa
sólida. As duas metades eram **coerentes consigo mesmas**, e por isso nenhum teste as pegou:

| | assume | resultado |
|---|---|---|
| **os nós** | `size` ausente = `[1,1]` | escrevem `1.0 × amount` |
| **o lowering** | `size` ausente = `[0.4, 0.4]` | desenha `0.4` |

Enquanto **nenhum** nó tocava `size`, os dois números nunca se encontravam. No instante em que um toca, o
elemento salta de `0.4` pra `1.0`. **A tabela de colunas do `ph2d-eval-motion` já denunciava a assimetria:**
`P` cai na identidade (`[0,0]`), `tint` cai na identidade (`[1,1,1,1]`), e só `size` caía num *"caller's
`default_size`"* — o único fallback que **não** era a identidade.

Não era decisão ratificada (não havia nenhum *"intentionally NOT"* — [[feedback_documented_decision_chesterton_fence]]):
o comentário do shell até admitia *"a scale/strobe that writes a `size` column overrides this fallback"*, mas
**nunca reconheceu que esse override é um salto de 2,5×**. Consequência não-examinada, não escolha.

## 3. A correção

**A regra passa a ter nome, no foundational** (`ph2d_nodegraph::attr::SIZE_IDENTITY = [1.0, 1.0]`, `const`
**aditiva** — isolamento máximo, não encosta em contrato):

> **Quem lowera uma stream para instâncias DEVE usar a MESMA identidade que os nós assumem para uma coluna
> `size` ausente.** Se o fallback do renderer e a base dos nós discordam, um nó solto na sua própria identidade
> redimensiona a cena inteira.

- `MotionState.default_size` = **`SIZE_IDENTITY`** (era `[0.4, 0.4]`).
- `motion.scale` cita a const em vez de repetir o literal.
- **A demo pede o tamanho que quer, explicitamente**: `grid → motion.scale(0.4) → …` nas duas cenas. Um
  documento que quer quads pequenos **diz isso, no grafo** — não herda de um número escondido no shell. (E,
  de quebra, a demo agora exercita o nó consertado.)

Os outros leaves (`channel.rs` de oscillator/wiggle/noise/step/stagger/drive, `strobe`) já usavam o literal
`[1.0, 1.0]` **correto** — não foram tocados; a guarda do §4 pega qualquer deriva futura deles.

## 4. A guarda — e a primeira versão dela era **OCA**

O teste (`a_node_at_its_default_params_does_not_resize_the_scene`) coza `grid → output`, depois
`grid → scale(default) → output`, e exige que os `RenderInstance` sejam **idênticos**. Mais o mesmo pro canal
Size do oscillator com amplitude 0 (a família inteira compartilha o leaf).

**A lição que essa fatia quase perdeu:** a 1ª versão do teste usava `evaluate_motion` (a forma de 5 args), que
**substitui silenciosamente os defaults HEADLESS** (`[1,1]`). Ela **passou com o bug inteiro no lugar** — nunca
tocou o fallback do shell. Só a **mutação** revelou (reverti o `0.4` e o teste continuou verde). Corrigido pra
`evaluate_motion_into`, com `state.default_size` real, o mutante ficou vermelho na hora:

```
assertion `left == right` failed: a Scale at amount = 1 is a no-op on the render
  left:  [[1.0, 1.0], …]
  right: [[0.4, 0.4], …]
```

> *Um teste que não foi visto falhar não é uma guarda — é decoração.* (DIRETIVA §3: verde-de-compilação vale
> ZERO. Aqui: **verde-de-teste também vale zero** enquanto o mutante não ficar vermelho.)

## 5. O que muda pro artista

- Um `motion.scale` no default é, agora, **de fato** um no-op.
- Uma stream **sem** coluna `size` renderiza em **escala unitária** (`1.0` de mundo), não mais `0.4`. Um
  documento que quer quads menores põe um `motion.scale` — **é a semântica, e é visível no grafo**.
- Nenhum save quebra (`size` nunca foi serializado como fallback; é comportamento de render).

## 6. Superfície nova (pro integrador)

| Onde | O quê |
|---|---|
| `ph2d-nodegraph::attr` | **`pub const SIZE_IDENTITY: [f32; 2] = [1.0, 1.0]`** (aditiva) |
| `shells/desktop/src/motion_state.rs` | `default_size` = `SIZE_IDENTITY` (era `[0.4, 0.4]`) |
| demo | `motion.scale(0.4)` em cada cena (13 nós no doc de boot) |

**Contrato intacto** (`NodeManifest`/`NodeOp`/`OpResolver` = 8/2/1, provado depois).

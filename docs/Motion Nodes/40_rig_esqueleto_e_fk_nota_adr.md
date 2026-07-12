# 40 — Rig: `rig.skeleton` + `rig.fk` — e a **decisão M4.N3** (sem `Domain::Rig`, sem ADR)

**Data:** 2026-07-12 · **Linha:** `line/motion-value` (Modo L) · **Fase:** **M4 — Rig** (abertura)
**Status:** implementado, testado (4 mutantes provados), **pendente smoke do Enio**
**Contrato congelado encostado:** **NENHUM** — provado 8/2/1 depois · **Foundational tocado:** nenhum

---

## 1. A decisão M4.N3 — a que o plano avisava que podia custar um ADR

> *"O plano cogitava um `Domain::Rig` novo — **isso encostaria no contrato congelado**. Antes de codar,
> pesquise a alternativa isolada."* (handoff §3, ETAPA C)

**Não precisa.** Um esqueleto é uma **stream de instâncias ORDINÁRIA**. Um elemento **é** uma junta, e quatro
colunas comuns descrevem a cadeia:

| coluna | tipo | significado |
|---|---|---|
| `parent` | Scalar | índice da junta de quem esta pendura; `< 0` = **raiz** |
| `len` | Scalar | comprimento do osso que vem do pai ATÉ esta junta |
| `rot` | Scalar | ângulo **LOCAL** da junta (graus), relativo ao pai |
| `P` | Vec2 | posição de **MUNDO** — **derivada**, nunca autorada |
| `wrot` | Scalar | ângulo de **MUNDO** — derivado (o skinning vai ler) |

**Consequência (é o ponto todo):** um rig anda pelos **mesmos fios** que tudo o mais. Todo nó genérico já
funciona nele — `motion.move` desloca, `motion.falloff` mascara, e **o canal `Rotation` do `oscillator` /
`wiggle` / `noise` / `step` POSA as juntas**, porque posar uma junta é escrever a coluna `rot`.

**Rig é fan-out puro: zero mudança de contrato, zero `Domain::Rig`, zero ADR.** `NodeManifest`/`NodeOp`/
`OpResolver` seguem **8/2/1**.

## 2. Por que `rot` é LOCAL e `P` é derivado

Porque é isso que faz uma cadeia ser uma cadeia: **gire uma junta e tudo abaixo dela balança** — o que só
acontece se a pose de mundo dos filhos for **função** da do pai.

Guardar ângulos de MUNDO por junta (a escolha do KineFX do Houdini) tem uma virtude — o quad da junta aponta
ao longo do osso — mas quebra o essencial aqui: um modificador **genérico** escrevendo `rot` giraria **uma**
junta e **rasgaria** o membro. Com `rot` local, o mesmo modificador genérico **posa** a junta, e o `rig.fk`
reconstrói o mundo.

**Corolário forte e testável — os ossos NUNCA esticam:** `|P[i] − P[pai]| == len[i]`, por construção, faça
alguém o que fizer com `rot`. (É a guarda que separa "FK de verdade" de "mexi no `P` e torci pelo melhor".)

## 3. Os dois nós

**`rig.skeleton`** (source, `TrapezoidDown`): emite `joints` juntas — a raiz mais uma corrente de ossos de
`length`, cada uma virada `angle` graus **da anterior** (0 = membro reto, pouco = arco, muito = espiral).
Já sai resolvido pro mundo, então um esqueleto pelado **já renderiza** (as juntas são elementos como quaisquer
outros). Publica `Index`/`Count` como o `motion.grid`, pra falloff/stagger/expressão endereçarem uma junta como
endereçam qualquer elemento.

**`rig.fk`** (modifier, **sem params**): reconstrói `P` e `wrot` a partir de (`parent`, `len`, `rot`).

**Por que FK é um nó separado, e não algo que a fonte faz uma vez:** porque o oscillator é **genérico**. Ele
escreve o ângulo e **deixa `P` onde estava**. Um esqueleto posado que ninguém resolveu é *um membro com todas
as juntas dobradas que não andou um pixel*. `rig.fk` é a **resolução** — solte-o depois de qualquer coisa que
tenha mexido em `rot`, e o membro balança.

Ele é **idempotente** (a pose é função pura das colunas) e é a **identidade em qualquer coisa que não seja um
esqueleto**: stream sem coluna `parent` = tudo raiz = todo `P` sobrevive intacto (a regra do doc 39 — um nó no
default não move um único elemento).

**Robustez:** `parent` apontando pra frente (documento hand-authored / MCP) degrada a **raiz** — não trava,
não lê lixo. `joints` clampado (`MAX_JOINTS = 64`), nunca zero elementos. HR-5: o leaf parabólico
`cos/sin`, e o erro **não acumula** na corrente (a direção de cada osso vem do seu ângulo de mundo ABSOLUTO,
nunca da direção aproximada do osso anterior).

## 4. A demo — o contraste É a lição

```text
ESQUERDA (repouso): skeleton ─> scale ─> move(−7) ─> output
DIREITA  (onda):    skeleton ─> oscillator (Rotation) ─> rig.fk ─> scale ─> move(+7) ─> output
```

À esquerda o esqueleto como autorado (um arco). À direita **o mesmo membro**, com um oscillator staggered
escrevendo os ângulos e o FK os transformando em pose: o membro **chicoteia**. Tire o FK da direita e ela fica
tão parada quanto a esquerda — **com todas as juntas secretamente dobradas**.

## 5. As guardas — 4 mutantes provados VERMELHOS

| # | Mutante | Guarda |
|---|---|---|
| 1 | **`rig.fk` arrancado da cadeia** da demo | `the_posed_limb_waves_rigidly_from_a_nailed_root` → **"tip travelled 0"** |
| 2 | `rot` lido como ângulo de **MUNDO** (o erro clássico de representação) | `local_angles_compound_so_the_chain_curls` + `the_bend_compounds_down_the_chain` |
| 3 | (guarda estrutural) osso esticando em qualquer tick da onda | `bone_lengths` a cada tick, tolerância 1e-3 |
| 4 | (guarda estrutural) raiz arrastada pra origem no re-resolve | `a_straight_chain_hangs_off_its_root_where_the_root_already_is` + "the root drifted at tick k" |

O #1 é **a** cicatriz da linha, de novo: o nó compila, o grafo valida, o oscillator cozinha ângulos — e a tela
não muda. Só um assert no que **chega ao `motion.output`** pega.

## 6. Superfície nova (pro integrador)

| Item | Valor |
|---|---|
| Crates novas | `ph2d-node-rig-skeleton` · `ph2d-node-rig-fk` (as 1ªs da família `rig.*`) |
| Node ids | `rig.skeleton` (Source, TrapezoidDown) · `rig.fk` (Transform, Rect) |
| Colunas novas | `parent` · `len` · `wrot` (`rot`/`P` já existiam) |
| Codegen | `ph2d-node-registry-init` regenerado — **75** crates-nó |
| Contrato | **intacto** (8/2/1) · **zero** foundational |

## 7. Aberto (o resto do M4 Rig)

`rig.ik_2bone` (lei dos cossenos — **e ela sai transcendental-free**: `cos_a = (l1² + d² − l2²)/(2·l1·d)`,
`sin_a = √(1 − cos_a²)`, rotação do vetor unitário por álgebra pura; o `atan2` aproximado pra escrever `rot`
de volta já existe no leaf do `motion.look_at`) · `rig.fabrik` (Aristidou & Lasenby 2011) · `rig.rubber_hose`
· `rig.skin_deformer` (LBS — vai ler `wrot`, que já está publicado).

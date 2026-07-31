# BUGS da Timeline — os cuja CAUSA enganava

> Irmão do [`BUGS_physics.md`](../Physics/BUGS_physics.md) e do
> [`BUGS_painter.md`](../Painter/BUGS_painter.md). Aqui entra o bug cuja causa não era a
> que a aparência sugeria — para ninguém re-derivar o diagnóstico, e para o repro virar
> gate. Bug de rotina fecha no commit e não precisa de linha aqui.

---

## #1 — O `lead_out` do ÚLTIMO strip de uma lane era INERTE (2026-07-30)

**Report (Enio, com screenshot):** *"em Arrange na lane 2 temos um fade do lado direito
para fora e ele não funcionou corretamente."*

**A aparência sugeria** que o fade estivesse desligado, ou que o peso não fosse calculado.
**Medido pelo apply real, era o contrário:** o peso rampava perfeitamente e a pose não
andava.

```
t      weight_at   hold_at                  x pintado
3.000    1.000     None                     +10.000
3.500    0.500     strip[1..3] w=0.500      +10.000   ← o peso caiu pela metade
4.000    0.000     strip[1..3] w=1.000      +10.000   ← e a pose não se moveu
4.125                                        -5.000   ← degrau
```

**Causa:** o `hold_at` deixava o strip que **acabou de terminar** responder *"algo ainda
vem"* sobre **si mesmo**, por duas metades independentes —

1. `lead_end() > t`: um `lead_out` ESTENDE o strip para além do `t_end`;
2. a isenção da borda inclusiva perguntava só `blend_out <= 0`, que é o blend de
   **SOBREPOSIÇÃO** — e um fade para FORA não tem sobreposição nenhuma, então ela valia
   para a janela inteira do `lead_out`.

O hold devolvia então a pose congelada DELE com peso `1 − w`, a cobertura da lane voltava a
exatamente **1**, e a fórmula do fade **cruzava contra ela mesma**. O fade só MOVIA o corte
por `lead_out` segundos.

⚠️ **Com uma PRÓXIMA strip nada disto aparecia** — o braço `fade_out_target` dispara antes
e a travessia sempre funcionou (+10 → +30, suave), com gate próprio verde
(`a_lead_out_plays_the_clip_fully_then_fades_in_the_gap`). O defeito vivia **só no último
strip de uma lane**, onde o que está do outro lado do fade são **as lanes de baixo**.

**Gate:** `a_lead_out_on_the_last_strip_fades_to_the_lane_below` (`gap_fade_out.rs`) —
oráculo de **MONOTONIA** mais o ponto médio EXATO da smoothstep, nunca um endpoint: um
endpoint sozinho fica verde sobre um degrau. 2 mutações, 2 sangram.

**Não era regressão** da remoção da autoria de expressões: a mesma sonda num worktree em
`HEAD~2` dá a tabela idêntica, e os dois `fade_fingerprint` seguem verdes com o MESMO hash.

---

## #2 — A trajetória de um clip aparecia (e era agarrável) em outro (2026-07-30)

**Report (Enio):** *"O Path criado em um Clip contamina e aparece alças em outro Clip
criado depois."*

**São DOIS defeitos com a mesma raiz**, e só o primeiro está fechado.

### 2a. As alças invisíveis — FECHADO

O caminho mora no **BINDING**, que é do DOCUMENTO; as keys moram no **CLIP**. O `marks` (o
desenho) perguntava pela track do clip ATIVO e devolvia zero marcas — mas o `anchor_screen`,
o `tangent_screen` e o `motion_path_curve_hit` liam só `b.path`. Medido num clip novo e
vazio:

```
clip A (ativo, keyado)      marks=6  ancoras=2  alcas=2  agarravel=SIM
clip B (novo, VAZIO)        marks=0  ancoras=2  alcas=2  agarravel=SIM   ← nada desenhado,
clip B (com UMA key)        marks=4  ancoras=2  alcas=2  agarravel=SIM      tudo agarrável
```

Um clique sobre uma alça **invisível** pegava e arrastava a trajetória do OUTRO clip, e o
duplo-clique inseria âncora numa curva que ninguém via.

⚠️ **A ironia que nomeia o defeito:** o doc-comment do `anchor_screen` já declarava ser *"a
porta ÚNICA … quem PINTA e quem faz HIT-TEST têm de concordar sobre ONDE a âncora está"* —
e as duas metades discordavam sobre algo anterior: **se ela existe neste clip**.

**Fix:** `active_path` — a pergunta *"o clip ativo tem key para este alvo?"* feita UMA vez,
com os quatro consumidores passando por ela. **Gate:**
`a_clip_that_does_not_animate_the_path_shows_no_handles`, com controle positivo (o clip A
tem marcas, âncoras E alças) e o ponto sobre a curva **pedido ao produto**, não chutado.
3 mutações, 3 sangram.

⚠️ Duas metades desse gate nasceram VERDES sobre nada: o `doc_with` monta o caminho com
`PathAnchor::corner`, cujas alças têm comprimento zero e o `tangent_screen` PULA — e o
ponto de duplo-clique que eu chutei caía fora do raio de pega, então devolvia `None` nos
DOIS clips. *A fixture tem de conter o fenômeno.*

### 2b. ⛔ ABERTO — a trajetória é do DOCUMENTO e as âncoras pareiam com as keys do CLIP

Assim que o clip B ganha **uma key qualquer**, o `marks` passa e o que se desenha é a
trajetória do clip A, com as âncoras e as alças dela. Isso **não é o overlay**: é o modelo.

O `binding.rs` defende o armazenamento no documento —

> *"It lives on the BINDING rather than in the clip because the trajectory is a property of
> this object's movement, not of one clip's take on it: two clips that both animate the
> object along it are two timings of the same journey."*

— e a linha seguinte do MESMO doc-comment diz:

> *"Anchor `i` pairs with key `i` of the track."*

**As duas não podem ser verdade ao mesmo tempo**, porque a track é por-CLIP. E o produto já
sabe disso: apertar K na trajetória num clip criado depois dispara o `debug_assert` do
`rewrite_path_key_values` —

```
a track tem 1 keys para 3 âncoras — a autoria não passou pela porta única
```

⚠️ **Em release o assert não dispara** e o `zip` escreve só até a mais curta: a key de
CHEGADA fica com a distância de uma âncora do MEIO e *"o percurso do objeto encolhe"* — que
é literalmente o *"se tentar arrastar qualquer ponto a curva quebra"* de um smoke anterior.
O `reconcile_one_position_path` é onde os dois modelos foram emparelhados: ele **retorna
cedo** quando o clip ativo não tem track, deixando as âncoras do outro clip de pé.

**As duas saídas, e a escolha é do Enio** (as duas mexem em `DOC_VERSION` ou em semântica):

| | o que muda | preço |
|---|---|---|
| **A — o caminho vai para o CLIP** | cada clip tem a própria trajetória; casa com *"âncora i ↔ key i"*; mata a contaminação na raiz | `DOC_VERSION` + migração + revoga a cerca de Chesterton do `binding.rs` |
| **B — o caminho fica no DOCUMENTO, mas é um TRILHO** | as âncoras são compartilhadas de propósito e cada clip keya só o *progresso*; o K num clip novo **não** acrescenta âncora | sem bump, mas muda o que o K faz e precisa de UI que diga de quem é o trilho |

Enquanto não se decide, o `debug_assert` fica como está: ele é o tripwire que provou a
contradição.

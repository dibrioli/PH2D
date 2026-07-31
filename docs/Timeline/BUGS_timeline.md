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

**São DOIS defeitos com a mesma raiz**, os dois fechados — o segundo pela escolha do
modelo (B, o trilho compartilhado).

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

### 2b. FECHADO pela ESCOLHA B — o caminho é um TRILHO compartilhado

Assim que o clip B ganha **uma key qualquer**, o `marks` passa e o que se desenha é a
trajetória do clip A. Isso não era o overlay: era o **modelo**. O `binding.rs` defende o
armazenamento no documento —

> *"a trajetória é propriedade do MOVIMENTO deste objeto, não da leitura de um clip: dois
> clips que a animam são duas CRONOMETRAGENS da mesma jornada"*

— e a linha seguinte do MESMO doc-comment diz:

> *"âncora `i` pareia com a key `i` da track."*

**As duas só podem ser verdade enquanto existir UMA cronometragem**, porque a track é
por-CLIP. E o produto já sabia: apertar K num clip criado depois dispara o `debug_assert`
do `rewrite_path_key_values` — *"a track tem 1 keys para 3 âncoras"*. Em release o assert
não dispara e o `zip` deixa a key de CHEGADA com a distância de uma âncora do MEIO: *"o
percurso do objeto encolhe"*, que é o *"se tentar arrastar qualquer ponto a curva quebra"*
de um smoke anterior.

⚠️ **A opção A (caminho por-clip) foi MEDIDA e recusada com três custos que a primeira
tabela não listava**, e ficam registrados para ninguém os redescobrir:

1. **"Distância" só significa alguma coisa sobre UMA curva.** O `prop.rs` já dizia:
   *"blending DISTANCES is what keeps a crossfade ON the trajectory … blending the two
   POINTS would cut the corner off it."* Com uma curva por clip, o blend não tem de que ser
   distância — Position teria de compor **PONTOS**, em 2D.
2. **Isso mudaria à força o que uma lane ADITIVA de Position significa** — hoje o
   `algebra()` a define como *"go further along it"*; sobre curvas diferentes isso não se
   pode dizer, e viraria *deslocar em XY*.
3. **O `fade_fingerprint_channels` é exatamente essa cena** (clip A em distância 2, clip B
   em 14, com SOBREPOSIÇÃO, sobre um L de 10+6): meio caminho é 8, **a quina**. Compondo
   pontos, o corredor corta a quina e o hash `0x69dca8811eb0f8f8` **move** — e o doc dele
   diz *"não há motivo para mover um pin que já dizia a verdade sobre o canal dele"*.

**A escolha do Enio foi B**, e a lei que ficou é uma frase:

> **Quem lança o trilho é a única cronometragem que existe.** A partir da segunda, ninguém
> autora geometria por acidente: as âncoras só mudam pelos gestos EXPLÍCITOS (arrastar
> âncora, arrastar alça, inserir na curva), e o **K de qualquer clip keya PROGRESSO** ao
> longo do que já existe (a pose vai para a curva pelo `project`, e a key guarda a
> DISTÂNCIA).

Porta única: `TimelineDoc::active_clip_authors_the_rail`. **Três consumidores, e cada um é
uma camada com gate PRÓPRIO** — com o braço de progresso no lugar, o K nunca ALCANÇA os
outros dois, então um gate só ficaria verde sobre eles
([[feedback_layered_defenses_need_per_layer_gates]]):

| camada | o que ela impede | gate |
|---|---|---|
| `add_path_key` | o K de outro clip acrescentar âncora | `keying_the_path_in_a_second_clip_keys_progress_not_geometry` |
| `rewrite_path_key_values` | arrastar uma âncora reescrever a cronometragem de quem só percorre | `reshaping_the_rail_leaves_a_progress_timing_alone` |
| `reconcile_one_position_path` | o `settle` reconstruir o trilho com uma âncora por key do clip que só percorre | `reconcile_only_repairs_the_timing_that_authored_the_rail` |

3 mutações, 3 sangram — a do reconcile de forma espetacular: o L de 3 quinas vira 5 âncoras
suaves ao longo da cronometragem do outro clip (a contaminação pelo lado oposto).

⚠️ **O custo honesto de B, para ninguém o descobrir por acidente:** a partir da segunda
cronometragem, reformar o trilho **não** reescreve mais as distâncias de ninguém — nem as
do clip que o autorou. É o que "progresso ao longo de um trilho" significa (a key diz *7 ao
longo*, e reformar o trilho move onde isso cai), mas para o clip autor é uma mudança: com
uma cronometragem só, as keys dele seguiam as âncoras.

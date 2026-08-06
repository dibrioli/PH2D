# HANDOFF DE INTEGRAÇÃO — `line/Painter`, o bow wave gateado no knob (2026-08-06)

> **7 commits · 10 arquivos · +663/−66 · nenhum `Cargo.toml` · nenhum ADR · `project.rs` intocado.**
>
> ⚠️ **PENDENTE DE SMOKE.** Seis dos sete commits são medição e gate; **um muda o produto**, e o que
> ele muda não aparece em nenhum diff de schema. Leia o §3 antes de integrar.

---

## §1 O que a jornada respondeu

O smoke de 05/08 mandou a fronteira para o `on_canvas_pointer`, e a decomposição por **meio**
(`what_a_shape_move_is_made_of`, levada ao raio do produto) disse que o **Impasto custa 19× o Digital**
e a **Aquarela 14×**, sobre a MESMA figura e o MESMO pincel. Nenhuma das quatro frentes que o
[plano 34 §5](Painter/34_plano_smokes_e_cerca.md) listava era a maior — **a fronteira é o MEIO**.

Descer um nível exigiu ablação **dentro do laço que o produto roda**, e o resultado a 4096², raio 185,
`DrawTo::Depth` (ms por traço):

| raio | camada | full | cauda | silhueta | filme | miolo |
|---|---|---|---|---|---|---|
| 185 | virgem | 111,33 | 15,40 | 19,36 | 21,97 | 48,85 |
| 185 | tela nua | 136,64 | **53,74** | 19,84 | 21,52 | 34,22 |
| 185 | sobreposto | 135,84 | **54,79** | 18,74 | 18,42 | 32,19 |

**A cauda é o BOW WAVE**, e ele custava o mesmo cruzando a parte **nua** de uma camada suja e cruzando
a própria tinta. Gateado no knob: **136,64 → 96,93 ms/traço**, com a cauda voltando aos 14,33 da
camada virgem.

---

## §2 Os sete commits

| sha | o que |
|---|---|
| `7b5d68042` | a sonda do shape move passa a **conter o fenômeno** (raio 24 → varredura até 185) |
| `33ba483c6` | **o pino byte-a-byte** do depósito — 3 gates, 6 mutações |
| `bc7c2452b` | a chave de **ablação** do laço de altura + a decomposição + o gate de que ela nunca é armada em produto |
| `b85c24a61` | a fixture media o regime errado — separa camada virgem de tinta-sobre-tinta |
| `ab5b3d91f` | duas curas construídas e **refutadas** (o early-out por-texel · a forma fechada) |
| `661340c23` | **o gate no knob** — o único commit que muda o produto |
| `b34e18833` | o comment do `draw_to_split` se refutava no próprio parênteses |

---

## §3 ⚠️ A MUDANÇA DE COMPORTAMENTO — leia isto antes de integrar

**`impasto_push` deixou de ser um knob live pós-traço.**

Um traço deitado com **Push = 0** (que é o *default* do pincel de depósito) não guarda mais o
ingrediente, então **alcançar o slider depois não re-deriva nada**. Push virou decisão de **antes** do
traço, e é **o único knob do card Body que não é live** — uma exceção num card de cinco, deliberada,
com o número ao lado.

**O que NÃO muda:** o frame que shipa é **byte-idêntico**. A re-derivação é
`field[i] = deposit + push * push_plane[i]`, então com `push == 0` tudo que a mordida, o banco e a onda
escreveriam é multiplicado por zero. O **pino plano-a-plano ficou verde** em toda a mudança — é ele
que torna esta afirmação verificável em vez de argumentada.

**A cerca executável foi REESCRITA, não apagada.** O `impasto_push_is_a_live_knob_and_never_erodes_the_ground_twice`
caiu — que é o comportamento certo dele — e virou
`impasto_push_is_live_on_a_stroke_that_was_laid_with_it_armed`: as três afirmações (LIVE · IDEMPOTENTE ·
REVERSÍVEL) seguem pinadas, agora sobre um traço deitado com o knob **armado**. *A capacidade não sumiu;
a precondição dela mudou.* E nasceu o irmão `a_stroke_laid_with_push_off_has_no_ingredient_to_re_derive`,
que pina a outra metade **para ninguém restaurar o gate antigo por acidente e devolver os 30 % em
silêncio**. Mutação: tirar o gate do knob deixa **só o irmão** vermelho — o par certo, porque apenas o
destino *Push-off* testa a lei nova sozinho.

**Decisão do Enio, 2026-08-06**, tomada depois de as outras duas saídas serem medidas e fechadas (§5).

---

## §4 Duas afirmações do próprio repo que a medição derrubou

1. **`impasto.rs`:** *"a first stroke on bare canvas has no ground, so it pays nothing — **the cost falls
   exactly where the feature is, on paint laid over paint**"*. A segunda metade é **FALSA**: `ground` é
   `self.heights.get(&active)`, ou seja da **CAMADA**. Corrigida no lugar onde estava.
2. **`the_impasto_draw_to_split`:** o comment dele dizia *"as faixas têm de ser distintas, senão o 2º
   traço encontra o relevo do 1º e o bow wave entra na conta"* e **no mesmo parênteses** enunciava o
   mecanismo que o refuta (*o `ground` é da CAMADA*). ⚠️ **Logo a coluna `Depth` da tabela dele está
   contaminada**, e o *"a altura custa 2,3× a 12× o pigmento"* que o módulo cita é um número com o bow
   wave dentro. Marcado no lugar onde a tabela vive; **não re-medido**.

---

## §5 ⛔ Medido e rejeitado — não refaça

* **O early-out por-texel na mordida** (`if q != 0` antes da divisão). Construído, byte-idêntico,
  medido: **53,74 → 53,24**, dentro do ruído de corrida (~5 %). O porquê é o mecanismo:
  `q = ground + plane`, e o **`plane` recebe o BANCO do próprio traço** ⇒ `q ≠ 0` sob quase todo texel
  mesmo com `ground = 0`. A mordida **não está ociosa, está transportando**. Revertido inteiro.
* **A forma fechada** (`take = g·m_final`, que o doc do kernel prova). Descartada **por leitura**, antes
  de custar código: o `plane` acumula o banco entre dabs, então `q` não é `ground·(1−m)` e a identidade
  quebra.
* **Fundir as duas varreduras** (a candidata que o `measure_impasto_cost` nomeava): ela ataca a
  **silhueta**, o **menor** dos quatro itens — teto de **14 %**.

---

## §6 Tabela de colisão

| item | estado |
|---|---|
| `PROJECT_SCHEMA` | **INTOCADO** — `project.rs` não é tocado (`git diff --name-only`) |
| contrato congelado | **4/4 verde** (rodado, não auto-relatado) |
| ADR | **nenhum** — a linha fica fora de toda disputa de número |
| `Cargo.toml` | **zero** |
| crate nova | nenhuma · dep nova: nenhuma |
| ids / tokens | nenhum |

**Superfície pública nova:** `ph2d_painter_brush::ablate` (`SILHOUETTE` · `FILM_AA` · `TAIL` · `set` ·
`get` · `with`) — ⚠️ `pub` **por necessidade**, porque `cfg(test)` desta crate não vale quando quem roda
o teste é a `ph2d-tool-painter`; o preço é o arch-gate
`the_ablation_switch_is_only_armed_by_measurements`, que varre as **duas** crates com controle positivo
nas duas pontas.

**Suítes:** `ph2d-painter-brush` 288 · `ph2d-tool-painter` 993 (lib) · shell **106 blocos verdes** ·
clippy limpo · LOC sob o teto.

---

## §7 O SMOKE

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
env PH2D_IMPASTO_SMOKE=2 PH2D_PAINT_PERF=1 cargo run -p ph2d-host-desktop --release
```

Três perguntas, nesta ordem:

1. **Com o pincel de impasto padrão, NADA pode ter mudado na tinta.** É byte-idêntico por construção e
   o pino prova; o smoke é a testemunha independente.
2. **Arme o Push ANTES de um traço** — o arado, o canal e a crista continuam lá, e o slider segue vivo
   depois (subir e descer devolve o chão bit a bit).
3. **Deite um traço com Push zerado e depois suba o slider: ele não faz mais nada.** É o desenho novo;
   se isso te incomodar na mão, o veredito é seu e a reversão é uma linha.

---

## §8 O que segue aberto, com número

| frente | estado |
|---|---|
| **o cap de Accumulate no WGSL** | a rota do DEVICE exige `!accumulate_cap` (`stamp_route.rs:454`); o log mediu o device **1,1-2,75× mais barato por visita** com a CPU levando **31-54 %** dos lotes. **Ganho não medido.** |
| **o filme (AA)** — **20,89 ms** medidos | era a "frente 2" estimada em ~17; o LUT **aplica** no raio do produto, então não é multi-tap. Decisão de **LOOK**. |
| **os quatro sítios fora da porta** | `compose.rs:218` · `selection_overlay.rs:91` · `stamp_color_cache.rs:362` · `transform_float.rs:346`. Os quatro carregam **`1 << 17` idêntico** + contagem constante — a forma exata que a wave das bandas curou. ⚠️ E o `compose` decide **todo quadro**: o rect sujo de um move é **15 625 px**, logo `15625 < 131072` ⇒ **sempre serial**. ⚠️ **O `SPAWN_EQUIV_VISITS = 808` NÃO é transferível** — ele é `custo_de_spawn ÷ 13 ns`, e os 13 ns são do kernel do DAB; emprestá-lo ao compositor seria a constante-emprestada que esta linha vive pegando. Precisa de fixture de `LayerStack`, que não existe na camada de sondas ⇒ **wave própria**. |
| **a Parte B do plano 34** (a cerca) | não iniciada |
| **o `PH2D_MASK_SMOKE=1`** (§2.1 do plano 34) | não há registro de ter rodado nesta jornada |

⚠️ **O plano 34 está obsoleto na Parte A:** ele lista quatro frentes candidatas e o log escolheu
**nenhuma** delas. A frente que ele abriu — o MEIO — está fechada por este handoff.

---

## §9 Nota de processo

⚠️ **A cwd do Bash escorregou para a árvore PRIMÁRIA uma vez**, num lote de `grep` de fechamento: os
números que ele devolveu (`3 files changed`, `0 commits`) eram do **primário sujo**, não da linha.
Nada foi commitado lá — todo commit saiu `[line/Painter …]` — e os fatos do §6 foram refeitos dentro da
worktree. É a **quarta** ocorrência registrada nesta linha, e a regra continua a mesma:
*todo comando começa com o `cd` da worktree.*

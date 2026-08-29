# HANDOFF DE INTEGRAÇÃO — `line/3DModeling`, 2026-08-29

> ⚠️ **Este documento é REFERÊNCIA, nunca EVIDÊNCIA** (DIRETRIZ §1.5.9 item 3). A tabela de colisão
> abaixo mede esta linha contra o `main` de **2026-08-29**. Se outra linha integrar antes desta, todo
> número da coluna «base» mudou. ⇒ **re-rode `bash scripts/collision-surface.sh` nesta worktree
> imediatamente antes de fundir**, e use a tabela daqui só para saber *o que a linha ACHAVA que
> estava a tocar* — a divergência entre as duas leituras é ela própria um achado.

---

## §1 — Identidade

| | |
|---|---|
| branch | `line/3DModeling` |
| HEAD | `62c68e6c2` |
| merge-base com `main` | `330582deb` |
| commits da linha | **45** |
| `main` andou desde o fork? | **sim — 26 commits** (a `line/components`, sobretudo). ⇒ **não é fast-forward**; precisa de rebase antes do `--ff-only` |
| arquivos tocados | 133 |
| waves cobertas | **W81 … W104-ter** (as W59–W80 foram no [handoff de 26/08](HANDOFF_INTEGRACAO_line_3DModeling_2026-08-26.md)) |

⚠️ **A narrativa de cada wave NÃO está aqui** — ela está no
[`06_resultados_cena_e_gizmo.md`](../06_resultados_cena_e_gizmo.md), **uma secção por wave**, §82 a
§104, com a tabela medida e as provas de mutação de cada uma. Este documento é só o que o integrador
precisa para não partir nada.

---

## §2 — Foundational / compartilhado tocado, e por quê

| arquivo | Δ | natureza |
|---|---|---|
| `ph2d-editor-core/src/widget/scrollbar.rs` | +13 −0 | **aditivo** — `MODEL3D_SCROLLBAR_ID: NodeId(843)` + a entrada no censo de barras. ⚠️ Ver §3. |
| `ph2d-editor-core/src/interaction/dispatch/scroll.rs` | +2 −0 | **aditivo** — um braço que aponta o thumb do MODEL para o painel dele |
| `ph2d-editor-core/src/ids/chrome/model3d.rs` | +21 −0 | **aditivo** — duas famílias de id **novas** (`model3d_verb_button`, `model3d_character_button`), por hash de string, fora de qualquer faixa numérica |
| `ph2d-editor-core/src/widget/mod.rs` | +7 −7 | re-export da constante acima |
| `ph2d-component-desc/src/catalog/field.rs` | +8 −1 | **aditivo** — descritor do componente de campo |
| `ph2d-i18n/src/model3d.rs` | +110 −14 | **aditivo** — 40+ chaves novas, todas com o prefixo `panel.model3d.` / `field.dim.` / `viewport.model3d.` |
| `shells/desktop/src/input_dispatch.rs` | +10 −1 | roteamento de tecla do módulo (guarda do modal, `Ctrl+Alt+Q`) |
| `shells/desktop/src/main.rs` | +6 −1 | registo do módulo |
| `shells/desktop/src/render_loop/mod.rs` | +27 −0 | o passe do MODEL no laço |
| `shells/desktop/src/forwarding.rs` | +7 −3 | encaminhamento do ponteiro |
| `CLAUDE.md` | 1 bloco | ⚠️ o texto da W80, escrito no fecho **anterior** — o desta jornada está por escrever (ver §8) |
| `project-memory/*` | 4 memórias novas + índice | lições da jornada |

⚠️ **Tudo o mais é do módulo**: `crates/ph2d-field*`, `crates/ph2d-panel-model3d`,
`shells/desktop/src/field3d_*`, `docs/3DModeling/`.

---

## §3 — Símbolos que podem COLIDIR (saída do `collision-surface.sh`, **não escrita de memória**)

```
SUPERFÍCIE DE COLISÃO — line/3DModeling contra main
  merge-base 330582deb   ·   45 commit(s)   ·   133 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                         99   (base: 99)
      └ tripla do gate               (99, 13, 14)   (base: (99, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS

▸ REGISTRO DE COMPONENTES — o contador é TRÊS
    ph2d-render (espelho)                  78   (base: 78)
    ph2d-script (espelho)                  78   (base: 78)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — último no disco: 0167   próximo livre: 0168
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — nenhum '+name' novo

▸ MARCADORES DE CONFLITO — nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou
  ✗   635 / 600   shells/desktop/src/field3d_input.rs
     6597 / 600   shells/desktop/src/input_dispatch.rs  (allowlist)
     1360 / 600   shells/desktop/src/main.rs            (allowlist)
    11322 / 600   shells/desktop/src/render_loop/mod.rs (allowlist)
```

### ⚠️ Os DOIS números desta linha que o `collision-surface.sh` NÃO mede

1. ⚠️⚠️ **`MODEL3D_SCROLLBAR_ID = NodeId(843)`** (`ph2d-editor-core/src/widget/scrollbar.rs`).
   No `main` do fork o comentário dizia *«Next free id is 843»* ⇒ esta linha tomou-o e escreveu
   *«Next free id is 844»*. ⛔ **Se outra linha também tomou 843, os dois lados escrevem o MESMO
   literal e o git funde MUDO** (a lei do `CLAUDE.md` §5.0). ⇒ **grepe `NodeId(843)` na árvore
   fundida** e confira o censo `("MODEL3D", …)` no fim daquele arquivo.
2. ⚠️ **`FIELD_DOC_VERSION` sobe de `4` para `10`** (`crates/ph2d-field/src/lib.rs`) — **seis**
   degraus (v5 verbo · v6 chanfro · v7 três formas · v8 prisma de duas pontas + cunha + arco ·
   v9 estrela/gaiola/elipsóide · v10 filete do arco). ⛔ **Não some com nada de outra linha**: só
   este módulo escreve aquele número, e nada persiste um `FieldDoc` (a escada está toda documentada
   no doc-comment da constante). Mas se alguém o tiver mexido, **conte, não escolha**.

### Símbolos NOVOS que outra linha pode ter escolhido

| símbolo | valor | onde |
|---|---|---|
| `MODEL3D_SCROLLBAR_ID` | `NodeId(843)` | `editor-core/widget/scrollbar.rs` ⚠️ ver acima |
| `model3d_verb_button` / `model3d_character_button` | hash de string | `editor-core/ids/chrome/model3d.rs` — **sem faixa numérica**, fora de disputa |
| `PrimitiveKind::{Cone, Capsule, Prism, Wedge, TorusArc, Star, BoxFrame, Ellipsoid}` | 8 variantes novas, `ALL: [_; 14]` | `ph2d-field` — módulo próprio |
| chaves i18n | 40+, prefixadas `panel.model3d.` · `field.dim.` · `viewport.model3d.` | `ph2d-i18n/src/model3d.rs` |
| cenas de smoke | `PH2D_FIELD_SMOKE=2..11` (a `=11` é nova) | `field3d_smoke_scenes.rs` — env própria do módulo |

---

## §4 — Contratos congelados (§6 do `CLAUDE.md`)

**Nenhum encostado.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool`/
`CanvasPaintTool`/`PanelEvent` estão **intocados** (a sonda confirma). Nenhum ADR novo.

---

## §5 — O que só o `ship.sh` apanha (o gate de integração NÃO roda)

- ✅ **`cargo fmt --all -- --check`** — limpo nesta worktree.
- ✅ **`clippy --all-targets`** — limpo em `ph2d-field`, `ph2d-field-eval`, `ph2d-field-ecs`,
  `ph2d-panel-model3d`, `ph2d-i18n`, `ph2d-editor-core`, `ph2d-host-desktop`.
- ✅ **LOC (`architecture_workspace_file_loc_cap`)** — ⚠️ **estava VERMELHO e foi curado no último
  commit.** Três arquivos que estavam **sob** o teto no `main` passaram-no nesta linha
  (`ph2d-field/src/lib.rs` 571→988, `dims.rs` 378→804, `ph2d-field-eval/src/lib.rs` 642→720). ⛔ A
  cura foi **partir para irmão** (`primitive.rs`, `dims_write.rs`, `stack.rs`), **nunca** allowlist.
  *Este gate vive na `ph2d-editor-core` e não corre nas suítes do módulo — foi a sonda de colisão que
  o apanhou, ao escrever este handoff.*
- ⚠️ **`typos`** — não corrido (binário ausente nesta máquina). Os textos novos são PT-BR/PT-PT
  densos; se o `ship.sh` reclamar, é aí.
- ✅ **`machete` / `deny` / `audit`** — **nenhuma dependência nova** (`Cargo.lock` sem `+name`).
- ⚠️ **`physics_ecs_c9`** — não corre na varredura impactada; esta linha não toca física.

---

## §6 — Ordem, dependências e o que smokar

**Sem dependências entre commits além da ordem cronológica.** Rebase linear sobre `main`, `--ff-only`
depois.

### Já smokado pelo Enio (aprovado)

| smoke | veredito |
|---|---|
| W100 (paleta de formas) | *«modal ok»* — depois de duas curas (§98) |
| W98 (juntas ao rodar) | *«smoke OK. Siga»* — depois da cura da marcha (§99) |
| W102 (pirâmide · cunha · arco) | *«smoke ok. Siga»* |
| W103 (estrela · gaiola · elipsóide) + W104 (toda aresta arredonda) | *«muito bom. apenas a estrela tem resultado ruim»* |
| W104-bis (meias-luas nos vales) | *«quase perfeito»* |

### ⚠️ O que NÃO foi smokado

- **W104-ter** (a compensação do ângulo nas quinas agudas — o último commit de produto). Ela muda a
  ponta da estrela **e** a quina de um **prisma triangular** (`Sides = 3`); num hexágono não faz
  nada, por construção.
- O **corte de LOC** (o último commit): é refactor puro, com as suítes verdes, mas **nenhum olho
  humano** o viu correr no app.
- As waves **W81–W96** (perf do traçado, quatro vistas, divisórias) foram smokadas ao longo da
  jornada anterior; não há veredito escrito para cada uma neste documento.

### Smokes do módulo

```
cd <árvore> && env PH2D_FIELD_SMOKE=<n> cargo run -p ph2d-host-desktop --release
```

`n = 2..11` (a **`=11`** é a da W103: estrela · gaiola · elipsóide). O pill **MODEL** abre o módulo
sem env nenhuma. ⚠️ **Preferência fora do repo:** `~/.ph2d/prefs.txt` — um `reduced_motion=1`
esquecido reprova smokes sobre produto correto.

---

## §7 — Riscos que o integrador deve olhar

1. ⚠️⚠️ **`NodeId(843)`** — §3, item 1. É o único número desta linha que soma com outra.
2. ⚠️ **`ph2d-editor-core` é foundational e três arquivos dele mudaram.** Todos **aditivos**; o
   `widget/mod.rs` é só re-export. Se outra linha mexeu no mesmo `scrollbar.rs`, o conflito é
   textual e resolve-se mantendo **as duas** entradas — mas **confira o literal do id**.
3. ⚠️ **`shells/desktop/src/input_dispatch.rs`** ganhou uma guarda de teclado. Ela tem de ficar
   **ANTES do primeiro tratador de tecla** — há gate a afirmá-lo
   (`the_field3d_keys_stand_down_while_the_palette_is_open`), e um merge que a reordene passa a
   compilar e a falhar em silêncio no app.
4. ⚠️ **O gate de LOC não corre nas suítes do módulo.** Depois de fundir, corra
   `cargo test -p ph2d-editor-core --test architecture_workspace_file_loc_cap` — é barato e foi
   exactamente o que esta linha quase deixou passar.
5. ⚠️ **Duas suítes desta linha são LENTAS de propósito** (`measure_sharp_edges`: ~5 s em debug).
   Elas defendem a promessa central do módulo (o filete alcança toda aresta); não as marque
   `#[ignore]` para acelerar o gate.

---

## §8 — A linha para o `CLAUDE.md §5` — ⚠️ **por escrever, e o texto está aqui**

⛔ **A linha `Aberto` do módulo NÃO está editada nesta worktree**: ela descreve o estado da **W80**
(o handoff de 26/08), e a DIRETRIZ é explícita em que o `CLAUDE.md §5` se edita **na integração, no
primário, uma linha de trabalho por vez**. ⇒ o que segue é o **texto literal** para o integrador
colar, e nada além disto entra lá — a narrativa das 24 waves é do doc 06.

### O que ACRESCENTAR à linha de estado (o módulo ganhou capacidade)

> ⭐⭐ **O catálogo de formas fechou** (W100–W103): **16 entradas** numa **paleta com busca** (`A` ou
> *+ Add shape…*), agrupadas por família — a fileira de chips cortava em `MAX_MODES = 8` e já tinha 8.
> O `Primitive` tem **14** famílias, e cada linha do catálogo carrega o **próprio construtor**
> (⛔ as quatro constantes `SHAPES.len() − N` morreram: acrescentar no fim fazia o botão *Extrude*
> abrir o diálogo de escultura, **sem erro nenhum**). ⭐⭐⭐ **E o filete alcança TODA aresta de toda
> forma** (W104): `0,0 %` da superfície sobre um vinco com o filete a metade do limite, nas dez
> formas que o têm — medido por uma sonda que **acha** as arestas pela variação da normal, e não por
> uma lista escrita à mão. ⚠️ Antes disso o `round` do **cone** e do **prisma** era **inerte** (`+0,0 %`
> de volume, campo bit a bit igual) e o da **cunha** fazia a peça **crescer 41 %**; o **arco de toro**
> não tinha controle de filete nenhum. `FIELD_DOC_VERSION` **4 → 10**. Cena **`=11`**.

### O que ACRESCENTAR à linha `Aberto`

> ⏳ **O filete só é um ARCO a 90°** — o operador recua o vértice `(1 − 1/√2)·r/sin α` e um arco
> verdadeiro recua `r·(1/sin α − 1)`; numa ponta de estrela (19°) isso é **`2,29×` menos** filete do
> que o número diz. Hoje isso é compensado **só nas quinas AGUDAS** (`max(1, factor)`), e as duas
> curas gerais estão **medidas e rejeitadas** (doc 06 §102.5 e §104.3) · ⏳ o teto de `round` da
> **estrela** é `12,3 %` do bordo, contra `43–60 %` de todas as outras formas — ela é a única em que
> a mistura é uma faixa estreita a atravessar uma face grande.

⛔ **Recusas medidas que NÃO se reconstroem** (mecanismo no doc 06 §101–§104): o **canto exato**
(`min(max(f1,f2,corda), disco)` no referencial `(u,w)` do par de planos — dá o arco certo e crava no
vértice de 3 vias) · a compensação aplicada às quinas **obtusas** (o prisma vai a `5,4 %` de aresta
viva) · o **recuo do disco** interior da estrela (inerte) · a **fórmula publicada do elipsóide**
(`‖∇f‖ = 1,86` e `f(centro) = −1` para qualquer tamanho) · o carácter **orgânico** no vale, na ponta
e no aro.

## §9 — Quatro coisas que uma leitura rápida do diff entende ao contrário

1. **`slab_and_walls` mudou de assinatura e de LEI.** Ela já não faz `offset(max(...), r)`: essa
   receita **não arredondava nada** (`{max−r<0}` é a interseção das duas peças dilatadas
   separadamente, e dilatar um semiespaço não tem canto). O `round` do cone e do prisma era
   **inerte** desde a W101, e o da cunha fazia a peça **crescer 41 %**. Doc 06 §101.1.
2. **`fillet_inflates` não é um enfeite.** Ela existe porque o filete que passou a existir **infla o
   gradiente** (`1,1943` medido), e o `inflation_depth` valia `0` para toda folha. Sem ela a marcha
   atravessa a superfície — o mesmo defeito do report de 29/08, um nível abaixo.
3. **A folga do sector da estrela (`round`) parece um épsilon e é geométrica.** O tecto é
   `inner·sin β`, e a varredura parte a forma a `2·round` **exactamente onde a conta diz**.
4. **`sharp_corner_radius` tem um `max(1, ·)` que é a metade que a faz funcionar.** Compensar as
   quinas **obtusas** estreita a mistura e cria o vinco que ela existe para curar — o prisma vai de
   `0,15` para `2,70` de quebra de curvatura. Há gate sobre os dois lados.

---

## §10 — Duas premissas minhas que a implementação REFUTOU

1. *«A escala do módulo é uniforme de propósito ⇒ não há como fazer um elipsóide»* — a nota
   respondia a **outra** pergunta (o `Xform::scale`, onde continua certa). Uma **primitiva** com três
   raios é uma folha, e a folha responde por si.
2. *«O `Blended::Exact` é um arco de raio `r`»* — só a **90°**. Fora dali o vértice recua
   `(1 − 1/√2)·r/sin α` em vez de `r·(1/sin α − 1)`, e numa ponta de estrela isso é `2,29×` menos
   filete do que o número diz. Medido por um gate que reprovou: `0,405396` contra `0,347714`.

---

## §11 — Estado da worktree

- `git status` **limpo** (nenhum `M`/`??`).
- `incremental/` **reclamado** (item 7 da §1.5.9) — ver o comando corrido no fecho.
- Suítes verdes nesta worktree: `ph2d-field` 46 · `ph2d-field-eval` 108 · `ph2d-field-ecs` 25 ·
  `ph2d-field-mesh` 9 · `ph2d-field-render` 39 · `ph2d-panel-model3d` 29 · `ph2d-editor-core` 1 269 ·
  `ph2d-host-desktop` 3 868.
- ⚠️ **Uma flake conhecida e PRÉ-EXISTENTE** apareceu numa corrida do `ph2d-host-desktop` e passou na
  seguinte: `only_the_lower_row_breathes_and_it_moves_with_the_playhead` (demos de áudio) — está
  nomeada no `CLAUDE.md` §5.0 como membro da família de flakes de recurso sob fan-out. **Não é desta
  linha** (o diff não toca áudio).

**Linha `3DModeling` pronta (HEAD `62c68e6c2`, 45 commits). Aguardo ordem de integração.**

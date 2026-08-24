# HANDOFF DE INTEGRAÇÃO — `line/3DModeling` (2026-08-23)

> DIRETRIZ §1.5.9. A linha está **fechada e parada**. ⛔ **Não integrei nem pushei** — nem farei sem
> ordem explícita do Enio (`CLAUDE.md` §0.7).
>
> ⚠️ **Este é o SEGUNDO handoff desta linha.** O [de 2026-08-22](HANDOFF_INTEGRACAO_line_3DModeling_2026-08-22.md)
> cobre as waves **1–34** (74 commits, integradas). Este cobre **só o que veio depois** — as waves
> **35–55**. Não releia o primeiro para integrar este: o `main` já o absorveu.

---

## 1. Identidade

| | |
|---|---|
| branch | `line/3DModeling` |
| worktree | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-3DModeling` |
| HEAD | o **tip** de `line/3DModeling` — ⚠️ **este handoff É o último commit da linha**, então não pode citar o próprio sha sem mentir a cada `amend`. A âncora estável é o commit anterior, **`af4029f40`**; leia o tip com `git rev-parse --short line/3DModeling` |
| merge-base com `main` | **`35f937cb2`** |
| commits | **25** (waves 35–55 + dois `fix`/`docs` + este handoff) |
| arquivos tocados | **79** |
| `main` à frente do fork | **ZERO** — `main` está exactamente no merge-base |

⭐ **O `--ff-only` é possível hoje, sem rebase nenhum**: `main` não andou desde a integração de
22/08. ⚠️ Isto vale para o `main` de **hoje** — se outra linha integrar antes desta, o integrador
**re-roda `collision-surface.sh`** (é a regra do §1.5.9 item 3, e a tabela do item 3 abaixo passa a
ser referência, não evidência).

**O que estas 21 waves acrescentam ao módulo:** o estado de **vista** que sobrevive a fechar o
painel · o **isolamento** como estado que se anuncia · a peça que **nasce enquadrada** · as **seis
vistas nomeadas** + a câmera alcançável por botão · o **gizmo de navegação** (bolas de eixo) com
posicionamento que foge da moldura · a **viagem** animada entre vistas (papel `Viewpoint`) · o
**perfil desenhado vira peça** (`+ Extrude` / `+ Revolve`) · a **régua da suavidade** corrigida para
a normal · e o **contorno vivo** com o knob de **Resolution**. Todas smokadas pelo Enio, uma a uma.

---

## 2. Foundational / compartilhado tocado, e porquê

⭐ **Tudo é ADITIVO** (`+N/-0`), com **duas** excepções marcadas ⚠️ — e as duas são cortes de LOC,
não reescritas de lógica.

| arquivo | churn | o quê, e porquê |
|---|---:|---|
| `crates/ph2d-editor-core/src/motion.rs` | **+26/-0** | ⭐ **variant novo `Role::Viewpoint`** + um braço em `law()`. Ver §2.1 — é o toque foundational com mais superfície |
| `crates/ph2d-editor-core/src/ids/chrome/model3d.rs` | +22/-0 | três famílias de `NodeId` novas (`model3d_view_button`, `model3d_camera_button`, `model3d_view_travel`). **Arquivo do módulo**, isolado de propósito |
| `crates/ph2d-editor-core/src/interaction/state/panel_ops.rs` | +13/-0 | `pub fn panel_rects()` — um iterador sobre o que já existia. Ver §2.2 |
| `crates/ph2d-i18n/src/model3d.rs` | +34/-0 | 14 chaves novas. **Arquivo do módulo** (o corte por painel foi feito exactamente para isto) |
| `shells/desktop/src/modal.rs` | **+104/-0** | ⚠️ **arquivo NOVO com nome genérico** — ver §2.3 |
| `shells/desktop/src/modal_tests.rs` | +178/-0 | os gates do acima |
| `shells/desktop/src/render_loop/mod.rs` | +156/-5 | despacho do módulo + **uma linha partilhada**: `ui_dt = modal::chrome_dt(...)` (§2.3) |
| `shells/desktop/src/project_load.rs` | +47/-0 | três «esquecimentos» de documento novo + abrir/enquadrar a peça no load |
| ⚠️ `shells/desktop/src/input_dispatch/keyboard.rs` | **+3/-35** | as seis teclas do módulo **saíram** para o irmão novo (§5.2) |
| `shells/desktop/src/input_dispatch/keyboard_field3d.rs` | +77/-0 | **arquivo NOVO** — o destino delas |
| `shells/desktop/src/input_dispatch.rs` · `main.rs` | +2 · +10 | declaram os módulos novos |
| `shells/desktop/src/project_tests.rs` · `project_field_tests.rs` | +5 · +219 | gates da peça no arquivo |
| `CLAUDE.md` | +20/-4 | a **quinta flake de relógio** (§5.3) + o §5 do módulo |
| `project-memory/` (4 arquivos) | +37/-2 | duas memórias novas sobre o arnês de mutação |

⚠️ **`Cargo.lock` NÃO foi tocado** e nenhum pacote externo novo entrou (conferido:
`collision-surface.sh` → *"nenhum '+name' novo"*). Nada para o `machete`/`deny`/`audit` nesta linha.

### 2.1 ⭐ `Role::Viewpoint` — o único variant novo num enum foundational

`crates/ph2d-editor-core/src/motion.rs` ganha `Role::Viewpoint` e um braço em `law()`, **antes** do
braço do `reduced`. É o toque com mais superfície partilhada da linha, e por isso:

- ⚠️ **A ORDEM dos braços é lei.** `Role::Viewpoint => Some(DISCRETE)` tem de ficar **acima** do
  arm que devolve `None` sob `reduced_motion` — é isso que faz a viagem sobreviver à preferência.
  Um merge que reordene os braços passa no compilador e **apaga a feature** em silêncio.
- ⚠️ **Um `match` sobre `Role` noutra linha vira erro de compilação** ao fundir (variant novo).
  Isso é o comportamento desejado — é onde o integrador vê que há uma lei nova a considerar.
- **Decisão do Enio, 2026-08-23**, com a alternativa na mão: *"o lerp não deve estar vinculado ao
  Reduced Motion. Mas deve ser o único modo."* O critério está escrito no doc do variant e é
  **estreito de propósito** (*o que substitui esta animação é um corte que desorienta mais do que
  ela*), com gate a exigir que um papel comum continue a morrer sob `reduced_motion`.

### 2.2 `panel_rects()` — porque não uma segunda lista

O gizmo de navegação precisa de saber onde a moldura está para não se esconder atrás dela. A
alternativa era copiar a lista *"que ids são painéis"* que o `cursor_over_hero_panel` já carrega — e
uma lista que se tem de lembrar é uma lista que se esquece (a lição da W48, mesmo módulo, mesmo dia).
O método é `+13/-0` e devolve o que o mapa **já** publicava.

### 2.3 ⚠️ `shells/desktop/src/modal.rs` — nome genérico, e uma linha partilhada

**Arquivo novo** com um nome que **não** diz «3D»: ele resolve um defeito do **shell inteiro**
(Enio, 22/08: *"não vejo em nenhum lugar a mensagem"*) — um diálogo modal congela o laço, e o
`wall_dt` do quadro seguinte mata todo toast criado antes dele. A cura é `chrome_dt(wall_dt, stall)`,
e ela entra numa **linha partilhada** do `render_loop`:

```rust
let ui_dt = crate::modal::chrome_dt(wall_dt, crate::modal::take_stall());
```

⚠️ **É o único ponto desta linha que muda comportamento fora do módulo** — todo consumidor de
`ui_dt` (toasts, motion) passa a ver o relógio com a paragem descontada. Está gateado
(`modal_tests.rs`, +178). ⚠️ **Risco de nome:** se outra linha criar um `shells/desktop/src/modal.rs`,
é colisão de mesmo-símbolo — não há hoje, mas o nome é apetecível.

---

## 3. Superfície de colisão (saída de `collision-surface.sh`, **colada**, não escrita de memória)

```
SUPERFÍCIE DE COLISÃO — line/3DModeling contra main
  merge-base 35f937cb2   ·   23 commit(s)   ·   79 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
    PROJECT_SCHEMA                         89   (base: 89)
      └ tripla do gate               (89, 13, 14)   (base: (89, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
    ph2d-ecs                               65   (base: 65)
    ph2d-render (espelho)                  66   (base: 66)
    ph2d-script (espelho)                  66   (base: 66)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0162   próximo livre: 0163
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
    nenhum '+name' novo

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou
    nenhum arquivo da linha passa do teto
───────────────────────────────────────────────────────────────────────────────
```

⚠️ **A tabela mede o commit `59b381a5e`** (o `af4029f40` é o `fix` das setas, que só troca nove
caracteres em quatro arquivos de teste da própria linha). ⚠️ **Prazo de validade:** ela descreve o
`main` de hoje; se outra linha integrar antes, **re-rode** (§1.5.9 item 3).

### 3.1 Símbolos NOVOS que uma outra linha pode ter escolhido também

| símbolo | valor literal | onde |
|---|---|---|
| variant de enum | `Role::Viewpoint` | `ph2d-editor-core/src/motion.rs` |
| variant de enum | `Param::Resolution` | `ph2d-field/src/dims.rs` |
| componente ECS | `"ph2d::field::FieldProfileSource"` | `ph2d-field-ecs/src/lib.rs` — ⚠️ **o id vem do NOME** (`stable_type_id`); duas linhas só colidem se escolherem a **mesma string**, e o registo entra em pânico ao ver isso |
| const pública | `MAX_PROFILE_RESOLUTION = 16` · `DEFAULT_PROFILE_RESOLUTION = 1` | `ph2d-field/src/profile.rs` |
| chaves i18n (14) | `panel.model3d.{view.*, camera.*, isolated, add.extrude, add.revolve, add.sculpt_scene, act.isolate}` · `field.dim.resolution` | `ph2d-i18n/src/model3d.rs` — **arquivo do módulo**, sem risco |
| famílias de `NodeId` | `model3d.view.{slot}` · `model3d.camera.{slot}` · `model3d.view.travel.{generation}` | `ids/chrome/model3d.rs` — hasheadas do nome, **arquivo do módulo** |
| módulos novos no shell | `modal`, `field3d_{view,views,navball,navball_paint,flight,mode,profile,profile_live,scene_gizmo,smoke_state}`, `input_dispatch::keyboard_field3d` | ⚠️ só o **`modal`** tem nome disputável (§2.3) |

⛔ **Nenhum número que SOMA entre linhas foi mexido** — nenhum schema, nenhum contador de registo,
nenhum número de ADR. A linha não cria ADR.

---

## 4. Contratos congelados encostados

**NENHUM.** `ph2d-nodegraph/src/node.rs` e `ph2d-editor-core/src/tool.rs` estão **intocados**
(conferido pelo `collision-surface.sh`). O módulo continua a ser um **drop-crate** que não implementa
`Tool` — a navegação e as teclas moram no **shell**, que é o que mantém `Tool=12` fora do caminho.

---

## 5. O que só o `ship.sh` pega — e ⛔ DUAS CERCAS QUE ESTAVAM VERMELHAS

⚠️ **Este é o item mais importante deste handoff**, e ele é uma confissão de processo: o fecho desta
jornada correu a suíte **inteira** das crates tocadas pela primeira vez em várias waves, e encontrou
**duas cercas vermelhas desde as W38–W51** — as duas curadas aqui, as duas pelo **mesmo mecanismo**.

> ⭐ **A lei, dita uma vez:** *um gate de ÁRVORE não é alcançado por um filtro de nome* — e o alvo do
> fecho deriva do **diff**, inclusive as crates que apenas **VARREM** o que a linha tocou, e não só
> as que ela editou. É a irmã da lição do clippy da W44, um nível acima.

### 5.1 `shell_files_respect_hr18_loc_cap` (vive em `shells/desktop/tests/`)

Quatro arquivos acima de 600 LOC, **três antes da última wave**:

| arquivo | em `main` | ao abrir a W55 | hoje |
|---|---:|---:|---:|
| `field3d_smoke.rs` | 506 | **790** | 580 |
| `field3d_scene.rs` | 555 | **659** | 492 |
| `input_dispatch/keyboard.rs` | 585 | **606** | 553 |
| `field3d_isolate_tests.rs` | — | **618** | 491 |

⚠️ O último tem um mecanismo próprio e conhecido: um argumento acrescentado a 32 chamadas não muda o
número de linhas — **o `cargo fmt` é que parte as chamadas longas e cria linhas**. *Medir LOC antes
do `fmt` mede outra coisa.*

Curado por **quatro cortes para o irmão**, cada um numa fronteira que já existia por dentro:

| novo arquivo | o que levou | a fronteira |
|---|---|---|
| `field3d_scene_gizmo.rs` | arrasto, pick, âncora, duplicar | *o que o gesto AGARRA* ≠ *o que a peça É* |
| `field3d_smoke_state.rs` | `Smoke`/`Grip`/`Drag`/`Ready`/`InFlight` + a célula | *o que existe* ≠ *o que se faz* |
| `input_dispatch/keyboard_field3d.rs` | as seis teclas do módulo, numa porta | o módulo irmão de escultura já tinha a dele |
| `field3d_profile_reach_tests.rs` | os gates de alcance do perfil (W53) | *o painel oferece?* ≠ *o isolamento diz-se?* |

⚠️ **Para o integrador:** a **ORDEM** das seis teclas viajou inteira e está dita no doc do módulo
novo — a entrada **numérica** vem antes da tecla de verbo, senão um `5` digitado no meio de um gesto
do gizmo vira um pedido de lente. *Reordenar ali é mudar comportamento, não estilo.*

### 5.2 `no_tofu_glyphs_in_ui_strings` (vive em `crates/ph2d-editor-core/tests/`)

Nove `→` (U+2192) em mensagens de `assert!` de quatro arquivos de teste da linha (W48/W49/W51). O
gate mora **noutra crate** e varre `shells/desktop/src/` inteiro ⇒ nenhuma corrida `-p
ph2d-host-desktop` com filtro de nome chegava a ele. Curadas para `->` no commit `af4029f40`.

### 5.3 O que o gate de integração **não** roda, e continua por conferir

- **`typos`** — corrido à mão sobre os paths da linha: **limpo**. ⚠️ `.typos.toml` **não** foi tocado.
- **`machete` / `deny` / `audit`** — nenhuma dependência nova (nenhum `Cargo.toml` da linha ganhou
  entrada, `Cargo.lock` intocado).
- **`fmt`** — limpo nas 7 crates (`cargo fmt -- --check`).
- **`clippy --all-targets`** — limpo nas **7 crates que a linha tocou**, derivadas do `git diff`:
  `ph2d-editor-core` · `ph2d-field` · `ph2d-field-ecs` · `ph2d-field-profile` · `ph2d-i18n` ·
  `ph2d-panel-model3d` · `ph2d-host-desktop`.
- ⚠️ **`nextest` na workspace inteira NÃO foi corrido** — só as 7 crates acima. Se outra crate tiver
  um gate que varre `shells/desktop/src/` (como os dois acima tinham), ele só aparece no `ship.sh`.
- ⚠️ **A QUINTA família de flake de relógio foi confirmada nesta linha** e está registada no
  `CLAUDE.md` §5: `the_fit_rebuilds_the_neighbourhood_not_the_whole_stroke` e
  `a_long_stroke_is_bounded_by_the_redundancy_floor_not_by_a_budget`
  ([`flip_fit_budget_tests.rs`](../../../shells/desktop/src/flip_fit_budget_tests.rs)) reprovaram
  sobre um diff que **não toca uma linha do Flip** e passaram 5 de 5 sozinhas. ⭐ A assinatura: **o
  conjunto de reprovadas muda entre corridas do mesmo binário**. *Re-rode sozinho antes de suspeitar
  do merge.*

---

## 6. Ordem, dependências e o que smoke-testar

**Ordem entre commits:** nenhuma dependência especial — os 24 são sequenciais e cada um compila.
O `af4029f40` (setas) é independente e pode ir em qualquer sítio depois do `59b381a5e`.

**Smokado pelo Enio, wave a wave:** todas as 21 waves (35–55). O `smoke OK` da última é de hoje.

**O que NÃO foi smokado, e vale um olhar depois de integrar:**

1. ⚠️ **O `modal.rs` toca o relógio de UI de TODO o app** (§2.3). Foi smokado *no caminho do
   modelador* (o toast da exportação aparece); **não** foi smokado num diálogo de outro módulo
   (abrir/salvar do Painter, importar sprite). Um toast que apareça e fique preso, ou que morra cedo
   depois de um diálogo, aponta para aqui.
2. ⚠️ **A `Role::Viewpoint` sobrevive ao `reduced_motion`** por decisão de produto. Com
   `~/.ph2d/prefs.txt` a dizer `reduced_motion=1`, **a viagem entre vistas continua a animar** — é o
   pedido literal do Enio, não um defeito. Todo o resto do app continua a morrer ali.
3. **O contorno vivo** (W55): editar a curva no Vector remodela a peça; a linha `Resolution` vai de
   1 a 16. Não foi smokado com **duas** peças do mesmo contorno, nem com o projeto salvo e reaberto
   depois de largar o desenho.

**Comando de smoke (a partir da árvore integrada):**

```
cd /home/enio/Documentos/Projetos/PH2D && cargo run -p ph2d-host-desktop --release
```

O módulo abre pelo pill **MODEL**; as cenas dirigidas são `PH2D_FIELD_SMOKE=<n>` (roteador:
[`field3d_smoke_scenes.rs`](../../../shells/desktop/src/field3d_smoke_scenes.rs)).

---

## 7. ⏸️ O que fica ABERTO no módulo (para a linha do §5, não para o integrador)

- ⏸️ A tabela que escolheu o **teto do nível de resolução** foi medida a `load ≈ 4,7`; a sonda
  `field3d_profile::tests::the_table_that_chose_the_resolution_ceiling` (`#[ignore]`) pede uma
  corrida com a máquina parada. O que ela pode mover é o **teto**, não a lei.
- ⏸️ **O traçado ficou ~2,4× mais caro desde a W3** e ninguém o reconferiu — achado da W54, suspeito
  nomeado (o anti-serrilhado adaptativo re-amostra a borda 4×). **Não** é regressão desta jornada.
- ⏸️ Nada na **Hierarquia** mostra que uma forma está ligada a um desenho; não há gesto para
  **largar** nem para **religar** o vínculo; um contorno de cada vez.
- ⏸️ O `Mirror` não se consegue demonstrar (adiado pelo Enio); a exportação não diz **onde** a peça
  está; religar uma escultura que mudou de sítio pede UI.

O mecanismo de cada wave está em [`06_resultados_cena_e_gizmo.md`](../06_resultados_cena_e_gizmo.md)
§36–§56, uma seção por wave, com a tabela medida e as provas de mutação ao lado. A lista viva do que
está aberto é o **§13** daquele arquivo.

---

## 8. Estado da worktree

- `git status` **limpo**; nada por comitar.
- ✅ `target/*/incremental` **reclamado** (DIRETRIZ §1.5.9 item 7).
- A linha **para aqui** e aguarda ordem de integração.

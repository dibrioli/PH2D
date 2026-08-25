# Handoff de integração — `line/3DModeling` (2026-08-24)

> DIRETRIZ §1.5.9. ⚠️ **A tabela de colisão abaixo é REFERÊNCIA, nunca evidência** — ela mede esta
> linha contra o `main` de **hoje**. O integrador **re-roda** `collision-surface.sh` em cada
> worktree imediatamente antes de fundir; a divergência entre as duas leituras é ela própria um
> achado.

## 1 — Identidade

| | |
|---|---|
| branch | `line/3DModeling` |
| HEAD | `57a853a30` |
| merge-base com `main` | `5d791f6b0` (rebase feito hoje, **depois** de outra linha integrar) |
| commits | **20** |
| arquivos | 48 (34 de código/config, 14 de docs+memória) |

⚠️ **O rebase de hoje teve UM conflito, e ele era de lista ordenada:** `project-memory/MEMORY.md`
ganhou uma entrada nesta linha e outra na linha que integrou no meio. Resolvido **mantendo as
duas** — é a lei do §5.0 (*número/entrada que soma entre linhas se conta, nunca se escolhe*).

## 2 — Foundational / compartilhado tocado, e por quê

| arquivo | o que | aditivo? |
|---|---|---|
| `crates/ph2d-panel-hierarchy/src/row.rs` | um braço em `badge_tone`: `"LNK" => TagTone::Success` | ✅ **puramente aditivo** — um braço novo num `match` de `&str`, nenhum tom existente mexido |
| `crates/ph2d-i18n/src/model3d.rs` | duas chaves: `panel.model3d.act.unlink` / `.link` | ✅ aditivo |
| `shells/desktop/src/render_loop/mod.rs` | 3 pontos: o selo do vínculo fundido no mapa de badges · `note_profile` passa o **id** em vez de um `bool` · o consumidor de `SelectRequest` vira **uma chamada** a `field3d_scene::apply` | ⚠️ **o terceiro ENCOLHE o arquivo** (−14 linhas): o `match` de 5 braços saiu para `field3d_scene::apply` |
| `Cargo.toml` do `ph2d-field-profile` | `[dev-dependencies]` → `ph2d-field-eval` + `fidget` | ✅ aditivo, **só de teste**; a direcção da dependência não inverte (o `-eval` depende do `ph2d-field`, nunca desta crate) ⇒ sem ciclo |
| `Cargo.lock` | as duas arestas **internas** acima | ✅ nenhum pacote externo novo |

⛔ **Nada mais fora de `crates/ph2d-field-*` e `shells/desktop/src/field3d_*`.**

## 3 — Superfície de colisão (`collision-surface.sh`, hoje)

```
SUPERFÍCIE DE COLISÃO — line/3DModeling contra main
  merge-base 5d791f6b0   ·   20 commit(s)   ·   48 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                         96   (base: 96)
      └ tripla do gate               (96, 13, 14)   (base: (96, 13, 14))
    VEC_SCENE_SCHEMA                       14   (base: 14)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-ecs                              —   (base: —)
    ph2d-render (espelho)                  71   (base: 71)
    ph2d-script (espelho)                  71   (base: 71)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR — último no disco: 0166   próximo livre: 0167
    esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — nenhum '+name' novo
▸ MARCADORES DE CONFLITO — nenhum nos arquivos da linha
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⭐ **Nenhum número que soma entre linhas foi mexido.** Sem schema, sem registro, sem ADR.

## 4 — Símbolos novos que podem colidir por MESMO-NOME

| símbolo | valor / forma | onde |
|---|---|---|
| `"LNK"` | literal de selo de linha | `ph2d-panel-hierarchy/src/row.rs` (`badge_tone`) |
| `panel.model3d.act.unlink` · `panel.model3d.act.link` | chaves i18n | `ph2d-i18n/src/model3d.rs` |
| `SelectRequest::Toggle` · `SelectRequest::AddMany` | variants novos de um enum **`pub(crate)` do shell** | `shells/desktop/src/field3d_scene.rs` |
| `Drag::Lasso` | variant novo de um enum **privado do módulo** | `field3d_smoke_state.rs` |
| `safe_march_step` | `pub fn` nova em `ph2d-field-eval` | `crates/ph2d-field-eval/src/lib.rs` |
| `surfaces_under` | `pub fn` nova em `ph2d-field-render` | `crates/ph2d-field-render/src/lib.rs` |
| `SLABS = 2` · `TILE = 64` | consts `pub(crate)` do renderer | `ph2d-field-render/src/tiles.rs` |

⚠️ **O `"LNK"` é o único que vive numa tabela que outra linha também pode estender** — a tabela de
tons de selo da Hierarquia. Se outra linha acrescentar um selo, os dois braços coexistem; o único
conflito possível é **duas linhas escolherem as mesmas três letras**, e aí o valor certo não é
nenhum dos dois lados (a lei do §5.0).

## 5 — Contratos congelados (§4)

**Nenhum encostado.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool`/
`CanvasPaintTool`/`PanelEvent` **intocados** (confirmado pelo `collision-surface.sh`). Nenhum ADR
criado.

## 6 — ⚠️ O que só o `ship.sh` pega (o gate de integração NÃO roda)

**Três avisos de clippy `--all-features`, TODOS pré-fork** (confirmado: os arquivos vêm de
`f94cb31cc` / `3d4af68a1`, ambos ancestrais do `main`):

| aviso | arquivo | origem |
|---|---|---|
| `unused import: stable_name_id` | `shells/desktop/src/joint_draw_tests.rs:10` | `f94cb31cc` (física, F1 passo 5a) |
| `unused import: stable_name_id` | `shells/desktop/src/render_loop/inspector_joint_tests.rs:12` | idem |
| `#[must_use]` sem mensagem sobre tipo já `#[must_use]` | `crates/ph2d-component-desc` | `3d4af68a1` |

⛔ **Não são desta linha e eu não os toquei** — mas o `ship.sh` roda clippy com features e vai
pará-los. *Quem shipar tem de os curar ou saber que estão lá.* Deps novas para o `machete`: nenhuma
externa (a única é `dev-dependency` **interna**). `typos`/`fmt`: limpos.

## 7 — Ordem e dependências entre commits

Linear, e a ordem importa em três sítios:

1. **W56 → W56e → W56f** (o traçado): o `march_slabs` da W56e substitui o `march` da W56d, e a
   W56f substitui a **constante** `SAFE_STEP` do renderer pela função do `-eval`. Fundir fora de
   ordem deixa o renderer a chamar uma constante que já não existe.
2. **W57 → W58**: o `profile_pick: Option<u64>` (que a W57 introduz no lugar de `has_profile: bool`)
   é lido pelo `acts_for`; a W58 não depende dele, mas as duas tocam `field3d_smoke_state.rs`.
3. **W58 → W58b → W58c → W58d**: as três correcções são sobre o mesmo gesto, e cada uma reescreve
   uma parte da anterior (`ToggleMany` → `AddMany` na última).

⇒ **Fundir a linha inteira, na ordem, é o caminho seguro.** Não há commit isolável no meio.

## 8 — O que smokar (e o que NÃO foi smokado)

✅ **Smokado pelo Enio, com report e correcção:** peça desenhada rápida (W56e/f) · vínculo ao
desenho visível e solto (W57) · laço de selecção (W58 → três reportes → W58b/c/d).

⏸️ **NÃO smokado — vale a pena olhar na integração:**

- **A peça com FURO** (W57): o gate prova que um contorno interior vira furo, mas o Enio nunca
  desenhou um. Smoke: desenhe um anel (círculo com círculo dentro, um contorno composto) e
  `+ Extrude`.
- **O `Link Drawing` a RELIGAR a outro desenho** (W57): o gate prova o botão; o Enio smokou o
  `Unlink`. Smoke: solte o vínculo, escolha **outro** contorno, e ligue.
- **O passo de marcha nas peças ARREDONDADAS** (W56f): a lei mantém-nas no passo curto de sempre e
  há gate de imagem, mas o Enio só olhou peças lisas. ⛔ Se alguma coisa correr mal aqui, o sintoma
  é **pixel de fundo no meio da peça**.

## 9 — ⚠️ Duas coisas que o integrador tem de saber

1. **A família de flake sob carga mordeu duas vezes hoje.** O `nextest-impacted` cancelou em
   `3325/11000` com dois vermelhos de
   `flip_smooth::resample_measurement::precisao::orcamento` — **membros nomeados** da lista do
   `CLAUDE.md` §5.0, e o diff desta linha **não tem um único arquivo `flip*`**. Sozinhos: verdes.
   Com `--no-fail-fast`: **11 000/11 000 passaram**. ⇒ *use `--no-fail-fast`, senão a suíte inteira
   fica por correr.*
2. **Uma corrida de `cargo test -p ph2d-host-desktop` deu 1 vermelho que não reproduziu** em duas
   seguintes, e cujo nome não chegou a imprimir; ela levou **92,8 s** contra 70,6 s das verdes.
   Mesma família.

## 10 — Estado do gate de fechamento

| | |
|---|---|
| `nextest-impacted --no-fail-fast` | ✅ **11 000 / 11 000**, 1 238 skipped |
| clippy `--all-targets --all-features` nas 6 crates tocadas | ✅ 0 desta linha (3 pré-fork, §6) |
| `cargo fmt --all --check` | ✅ |
| `cargo check --workspace --all-targets` | ✅ |
| `file_loc_caps` (shell, 600) | ✅ |
| `architecture_workspace_file_loc_cap` (700) | ✅ |
| `no_tofu_glyphs` | ✅ |
| `doc-index.sh --check` | ✅ 14 índices em dia |
| provas de mutação | **65 mutações, 65 vermelhas** com os três controles (22 W56 · 10 W56e · 7 W56f · 8 W57 · 20 W58) |

## 11 — As waves, em uma linha cada

| wave | o que |
|---|---|
| **W56e** | a marcha fatia em **profundidade** (`SLABS = 2`), e a fatia acordou um defeito latente: os quatro raios de canto **não bastam** na lente convergente (cura: a flecha do cone, com prova) |
| **W56f** | o passo da marcha é do **DOCUMENTO** — auditado construtor a construtor, só o arredondamento exacto infla (`√2`); o `Taper` **desce** a `0,844` |
| **W57** | o vínculo desenho→peça **vê-se** (selo `LNK`) e **solta-se** (`Unlink` / `Link Drawing`); ⭐ e o item «furos» **já estava construído** — a composição do `VecPath` o exprimia |
| **W58** | a selecção múltipla nasce no **canvas** (clique aditivo + laço), pela tecla que já existia |
| **W58b** | o laço apanha **o que está tapado** (as formas nascem empilhadas no alvo da câmera) |
| **W58c** | a moldura do laço pinta-se **sem nada selecionado** (estava do lado errado de uma guarda) |
| **W58d** | o laço **SOMA**, o clique alterna — e a assimetria é a lei |

Mecanismo, tabelas e recusas medidas: [`06_resultados_cena_e_gizmo.md`](../06_resultados_cena_e_gizmo.md)
§58–§64.

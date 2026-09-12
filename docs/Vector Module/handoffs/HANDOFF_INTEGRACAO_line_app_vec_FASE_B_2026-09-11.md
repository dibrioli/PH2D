# HANDOFF DE INTEGRAÇÃO — `line/app-vec` (W2/L4, **FASE B**) — 2026-09-11

> **Estado: FASE B fechada, e o FIM DA LINHA gateado NÃO foi alcançado — por um bloqueador NOMEADO
> que não é desta linha.** A linha não integra e não faz ship (CLAUDE.md §0.7).

## §1 — Identidade

| | |
|---|---|
| branch | `line/app-vec` |
| HEAD | `b0775b193e4dc49629047bf10adabeaeaab83254` |
| merge-base | `2dedac80a92f2ec62f9e7412c83192ed6315cb56` |
| commits | **4** |
| diffstat | 47 ficheiros, +219 / −147 (mais 71 renomeações/movimentos) |

## §2 — O que saiu, medido

| | antes | depois |
|---|---:|---:|
| `shells/desktop/src` — ficheiros | 1 518 | **1 498** |
| `shells/desktop/src` — LOC | 417 195 | **412 208** |
| `crates/ph2d-app-vec/src` — ficheiros | 12 | **33** |
| família ainda na shell | 127 (121 + 6 `vector_bridge*`) | **107** (101 + 6) |

**−4 987 LOC da shell.** O prefixo `vec_` saiu de dentro da crate (HOWTO §1.3) e isso desfez uma
**colisão de basename real**: `vec_bindings.rs` existia em `shells/desktop/src/` **e** em
`crates/ph2d-ecs/src/`.

## §3 — ⛔⛔ O ACHADO: o bloqueador desta família NÃO é a `App` — é `name_unique`

O grafo da família tem **uma raiz**, e ela não passa pela `App`:

```
vec_entities  ─ bloqueado por ─▶  name_unique::unique_name          (folha partilhada)
     │                            morph_set::is_set_member          (3.ª rodada)
     │                            render_loop::off_canvas::is_off_canvas
     ├─ #[path] ─▶ vec_zorder ─▶ vec_zorder_fixpoint_tests ─▶ undo · hero_intents ·
     │                                                        project_library · preview_drive
     └─ referido por 42 ficheiros da família
```

**Medido, com os cenários corridos:**

| cenário | move | fica |
|---|---:|---:|
| hoje | **0 mais** | 107 / 30 611 |
| **se todo o acoplamento a `App` for curado** | +4 / 615 LOC | 103 / 29 996 |
| + as 3 folhas curadas | +4 / 615 | 103 / 29 996 |
| + os 14 testes de integração-da-shell re-parentados | +14 / 3 194 | 93 / 27 417 |

⭐ **Curar a `App` compra 4 ficheiros.** Os 9 que ela prende batem imediatamente no `vec_entities`.
⇒ *a `App` nunca foi o bloqueador desta família* — exactamente a conclusão que a `line/app-physics`
tirou, agora **confirmada de forma independente a partir de outra família**, e com o cascata
quantificado (42 ficheiros atrás de um módulo de 268 LOC).

⚠️ **`name_unique` e `preview_drive` são DOIS dos TRÊS** que a batedora nomeou como folhas
partilhadas de linha própria (o terceiro é `inspector_ordering`). Esta linha acrescenta **dois
candidatos com a mesma forma** — predicados puros sobre o ECS partilhados entre famílias:
`morph_set::is_set_member` e `render_loop::off_canvas::is_off_canvas`.

⛔ **E a cura barata é pior que a doença, com número:** dar a `vec_entities::sync` um parâmetro para
o nome único custa **168 sítios de chamada**, espalhados por `bool_live`, `blend_live`,
`connector_live`, `envelope_live`, `instance_*`, `label_live` — as famílias da 3.ª rodada, que não
estão abertas. *Um parâmetro que atravessa 168 sítios de cinco famílias não é uma assinatura: é uma
wave.* Não o fiz.

## §4 — ⚠️ O CENSO CORRIGIU-SE QUATRO VEZES, e as quatro no mesmo sentido

Todas a favor de **mover demasiado** — o mesmo padrão da Fase A, que também se corrigiu quatro vezes:

| # | a régua dizia | a verdade | o que ela esquecia |
|---|---|---|---|
| 1 | «3 `impl App`» | **13** | `^impl App` perde os 10 `impl crate::App` (HOWTO §2.1 — pago por esta linha **duas** vezes) |
| 2 | 69 ficheiros livres | **85** | `\bApp\b` no ficheiro CRU: **7** mencionam-no só em doc-comment (§2.12) |
| 3 | `render_loop` bloqueia 10 | bloqueia 6 | `render_loop::vector_bridge` é ficheiro **desta** família na pasta do laço — *um censo por PASTA não vê posse* |
| 4 | 50 movíveis | **21** | um movido não pode referir um da família que **FICOU**; `vec_entities` prendia **30 de 50** |

⛔⛔ **E a 4.ª mordeu DEPOIS de eu já ter movido 50 ficheiros** — como na Fase A, onde mordeu depois
de 15. ⚠️ O `#[path]` é a aresta que domina tudo: há **801** arestas `#[path]` no `src/` da shell.

⛔⛔ **E encontrei o defeito na MINHA PRÓPRIA SONDA, a terceira mordida do §2.12:** a função que
tirava comentários **e strings** antes de contar apagava também os `#[path = "…"]` — *o caminho É uma
string*. A sonda correu sem a aresta que mais importa e leu **29 movíveis** onde a verdade era **0**.
⇒ hoje há duas funções (`sem_comentario` para o grafo, `cod` para o censo) e um **controlo positivo**
que exige que a aresta `vec_entities → vec_entities_group` exista no grafo.

⭐ **A cura de `vec_entities` que funcionou é cirúrgica e vale uma linha:** dos 30 bloqueados, **20
usavam só o `VecEntityMap`** — um alias de UMA linha sobre tipos de crates. Ele mudou-se para
`ph2d_app_vec::entity_map`, a shell re-exporta-o, e os ~190 sítios que o nomeiam ficam intactos.

## §5 — Foundational / partilhado tocado

| ficheiro | o quê |
|---|---|
| `shells/desktop/src/main.rs` | 32 `mod vec_x;` → alias `pub(crate) use ph2d_app_vec::x as vec_x;` (14 revertidos para `mod` quando os ficheiros voltaram) |
| `shells/desktop/src/render_loop/mod.rs` | 1 alias |
| `shells/desktop/src/vec_entities.rs` | o `VecEntityMap`/`MAX_DEPTH` passam a re-exportação da crate |
| `shells/desktop/tests/it/the_stroke_checkbox_is_wired.rs` | helper `familia()` + a visibilidade na fronteira (§2.9) |
| 15 ficheiros da shell | `#[path]` re-apontados |
| 16 ficheiros da shell | 85 referências re-ancoradas em `crate::vec_*` |
| `crates/ph2d-app-vec/Cargo.toml` | +4 deps medidas; `ph2d-panel-vector` deixa de ser opcional |

⭐ **ZERO contador partilhado se moveu** (`collision-surface.sh` corrido agora): `PROJECT_SCHEMA` 128
= base, `FLIP_SCHEMA` 13 = base, `DOC_VERSION` 18 = base, os dois espelhos de registo 86 = base,
contrato congelado **intocado**, **nenhum** ADR, **nenhum** pacote externo novo, zero marcadores de
conflito, nenhum teto de LOC estourado.

⚠️ **O `TETO_LOC` do `the_shell_only_shrinks` NÃO foi tocado** (regra 1) — e ele **passa**: os
−4 987 LOC não bastaram para a catraca declarar que já não descreve a árvore. O número medido é
**412 208** (de 417 195); quem o conta é o integrador, com a árvore junta.

## §6 — ⛔ O FIM DA LINHA gateado NÃO foi alcançado, e o motivo é aritmético

O bloco pedia: os quatro roteadores para dentro da crate · `max_level` contado · `"vec"` fora da
catraca `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL`. **Não foi feito, e não por falta de trabalho:**

| cena | o que ela precisa e não é da família |
|---|---|
| `vec_appearance_smoke` | `build_smoke::shape` |
| `vec_stack_smoke` | `build_smoke::shape` |
| `vec_fade_smoke` | `build_smoke::shape` · `fx_live::set_filter` |
| `vec_bone_smoke` | `build_smoke::shape` · `bone_gesture::create` · `image_import::{PackedSource,spawn_sprite}` · `skeleton_goal::add` · `skeleton_live::{bind,bind_image}` |

⚠️ **A catraca é ALL-OR-NOTHING por família, por construção** — o gate
`every_registered_family_declares_a_reachable_router` reprova nos dois sentidos: uma família na
catraca que declare **um** roteador tem a entrada dada como obsoleta. ⇒ não há como declarar 2 de 4.

⭐ **E o `build_smoke::shape` é uma FOLHA de 4 linhas**: `cook(kind,a,b,v)` + pôr a tinta, sendo
`cook`/`ShapeKind` de `ph2d_vec_scene` (uma crate). **22 ficheiros** de muitas famílias o chamam
(`fx_*`, `contour`, `falloff`, `twist`, `envelope`, `morph_fade`, `bone_gesture`, `skeleton_live`,
`build_smoke_*`, e as minhas 4). ⇒ pela mesma lei do §1.2 ele é **folha partilhada**, e o sítio certo
é ao lado do `cook` — em `ph2d-vec-scene`, onde a lei já vive. Custo: ~6 LOC + uma delegação de uma
linha no `build_smoke`. ⛔ **Não o fiz** porque sozinho ele destrava **2 de 4** cenas, e a catraca não
aceita metade — o `vec_bone_smoke` continuaria a precisar de 4 módulos do Esqueleto/importador.

⇒ **o fim da linha da `vec` abre quando (a) o `shape` for folha e (b) o Esqueleto/`image_import`
saírem.** É a 3.ª rodada, não esta linha.

## §7 — ⚠️ As armadilhas do HOWTO §2 que esta linha pagou, com o número

| § | armadilha | como apareceu aqui |
|---|---|---|
| 2.1 | o censo conta a FORMA | 13 `impl App`, não 3 — **2.ª vez** nesta família |
| 2.4 | as features não viajam | `paint_stack` usa `ph2d_panel_vector` em 9 sítios **sem `#[cfg]`**, o `mod` nunca foi gateado e o `render_loop` chama-o em 10 sítios ⇒ a shell já a exigia sempre. A dep passa a **não-opcional** e a feature gateia só a **presença** do `font_preview`, que **ERA** gateado. *A condição de compilação de um módulo não está escrita nele.* |
| 2.5 | `#[cfg(test)]` é invisível da outra crate | `bucket_repro.rs` abre com `#![cfg(test)]` ⇒ **da shell ele não existe**. A cura é a **ausência**: o alias sai, e ele corre com os testes da crate |
| 2.6 | `include_str!` **e o gémeo em runtime** | o `include_str!` falha alto; o `CARGO_MANIFEST_DIR/tests/fixtures/*.svg` do `bucket_repro` **só falhou quando o teste correu** — as 5 fixturas SVG mudaram-se com a sonda |
| 2.9 | a agulha nomeia um endereço | `the_stroke_checkbox_is_wired` lia `src/vec_stroke_present.rs`; ⭐ **quem falou primeiro foi o CONTROLO POSITIVO do próprio gate** |
| 2.12 | um censo lê prosa como código | 7 ficheiros acusados por doc-comment — **e depois a minha própria sonda**, que apagou o grafo de `#[path]` ao branquear strings |

## §8 — A PROVA (§3 do HOWTO)

| item | resultado |
|---|---|
| **(a) nenhum teste se perde** | **`ONLY-A = 0` e `ONLY-B = 0`, nas DUAS profundidades.** `22 659 → 22 659`; **81 `MOVED`**. Exacto nos dois sentidos: nada perdido, nada inventado |
| **(b) roteadores iguais** | **106 antes, 106 depois**, `diff` vazio |
| **(c) a shell encolheu** | **417 195 → 412 208 LOC** · 1 518 → 1 498 ficheiros |
| **(d) gate de fecho** | `fmt --check` ✓ · `clippy --all-targets --all-features` nas 2 crates ✓ · `typos` ✓ · `doc-index --check` ✓ (19) · **10** gates de teto de LOC ✓ · `ph2d-app-registry-init` **5/5** ✓ · **`nextest-impacted.sh`: 15 006 testes, 0 falhas** |
| **(e) smoke do dono** | binário **compilado e quente** — 1.ª corrida `27,55 s`, **2.ª corrida: `Finished` em `0,21 s`, ZERO linhas `Compiling`** (`target/smoke/ph2d-host-desktop`, 77,9 MB). `target/*/incremental` reclamado antes (10,4 GB) |

⚠️ **Sobre o relógio da alínea (c):** não entrego um `--timings` a frio. A razão é aritmética, não
carga: **−1,2 %** da shell, sobre os `16,8 s` de front-end que a auditoria mediu, é **~0,2 s** —
abaixo da variância entre corridas. E a máquina esteve a `load 5–33` com as linhas irmãs a construir,
o que pela regra do `CLAUDE.md §5.0` **anula** qualquer leitura. *A LOC é exacta e independente de
carga; o relógio só terá sinal quando os 107 que ficam saírem.*

## §9 — O que a FASE C herda, em ordem de valor

1. ⭐⭐ **A linha das FOLHAS PARTILHADAS** — `name_unique`, `preview_drive`, `inspector_ordering`
   (nomeadas pela batedora) **+ `morph_set::is_set_member`, `render_loop::off_canvas::is_off_canvas`,
   `build_smoke::shape`** (nomeadas aqui). São todas predicados/construtores **puros**, e as seis
   juntas destravam a raiz desta família (42 ficheiros) e o roteador. **É a maior alavanca da W2 que
   ainda não tem linha.**
2. `vec_entities` + `vec_transform` saem logo atrás delas (o `vec_transform` é bloqueado **só** pelo
   `vec_entities`).
3. Os **14** testes de integração-da-shell da família re-parentados para a raiz da shell — é onde o
   sujeito deles vive (§2.6), e valem +3 194 LOC no cenário medido.
4. ⏸️ O `VecCtx` (curar a `App`): **+4 ficheiros / 615 LOC**. Medido, e é o item de **menor** valor
   da lista — ao contrário do que o bloco supunha.

## §10 — O smoke

⚠️ **A extracção não muda produto** — o smoke confirma a INÉRCIA.

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-vec && env PH2D_VEC_BONE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

As outras da família, o mesmo comando trocando a variável: `PH2D_VEC_STACK_SMOKE=1` ·
`PH2D_VEC_APPEARANCE_SMOKE=1` · `PH2D_VEC_FADE_SMOKE=1` · `PH2D_VEC_SVG_SMOKE=1` (⚠️ esta vive em
`svg_import_smoke.rs`, não num ficheiro `vec_*` — ver doc 48 §7).

⏳ **NÃO smokado à mão:** o **encaixe** (snap) no canvas e o **lápis** — as duas partes cuja lei
atravessou a fronteira. Têm gates verdes e nenhuma passou por mão humana nesta jornada.

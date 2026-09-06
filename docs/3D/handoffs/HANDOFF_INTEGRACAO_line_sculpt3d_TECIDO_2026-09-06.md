# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d` · **o PINCEL DE TECIDO (W10)** · 2026-09-06

> ⛔ **A linha NÃO integra e NÃO pusha** (CLAUDE.md §0.7). Este documento é o que um **agente
> integrador** precisa, e só ele integra, por ordem explícita do Enio.
>
> O que se ganhou: [`05_a_vitoria_medida.md`](../cloth/05_a_vitoria_medida.md).
> O que falta: [`06_o_plano_do_que_falta.md`](../cloth/06_o_plano_do_que_falta.md).
> O mecanismo do dia: [`HANDOFF_line_sculpt3d_O_PINCEL_DE_TECIDO_2026-09-06.md`](HANDOFF_line_sculpt3d_O_PINCEL_DE_TECIDO_2026-09-06.md).

## §1 — Identidade

| | |
|---|---|
| ramo | `line/sculpt3d` · worktree `Worktrees/line-sculpt3d` |
| HEAD | `9fde71611` (o handoff acrescenta commits de doc por cima) |
| merge-base com `main` | `53832c884` |
| commits | **81** · **153** ficheiros |
| backup verificado | `/home/enio/Backups/PH2D/line-sculpt3d_tecido_2026-09-06.bundle` (+ `.txt`), etiqueta `backup/tecido-2026-09-06`, ramo `backup/line-sculpt3d-tecido-2026-09-06` |
| aprovado em smoke pelo dono | **três vezes** em 06/09 |

## §2 — Foundational / partilhado tocado, e por quê

⭐ **Tudo o que a linha tocou fora dela é APPEND-ONLY num ficheiro que já é do sculpt3d.** Não há
ponto de extensão novo, nem contrato mexido, nem número que se conte entre linhas.

| ficheiro | o que a linha fez | risco de colisão |
|---|---|---|
| `crates/ph2d-editor-core/src/ids/chrome/sculpt3d.rs` | **dois arrays de ids novos** (`SCULPT3D_CLOTH_MODE`, `SCULPT3D_CLOTH_AREA`), acrescentados no meio do bloco do pincel | ⚠️ **baixo**: o ficheiro é por-módulo. Outra linha que acrescente ids de sculpt3d no mesmo sítio funde textualmente; o gate `node_id_collisions` da editor-core apanha um hash repetido |
| `crates/ph2d-i18n/src/sculpt3d.rs` | **duas chaves** (`panel.sculpt3d.cloth_mode`, `.cloth_area`) | ⚠️ **baixo**, mesma razão |
| `Cargo.lock` | **um membro interno novo**, `ph2d-cloth`, sem dependência externa | ⚠️ o `collision-surface.sh` marca-o; **não** é pacote externo |
| `scripts/doc-index.sh` | **uma entrada** na tabela `DIRS`, para `docs/3D/cloth` | ⚠️ **baixo**, append numa lista |
| `shells/desktop/src/sculpt3d_*.rs` | o braço do gesto e o gate de undo | ⚠️ **médio se outra linha mexer no sculpt3d da shell** — ver §3 |

⛔ **Nada em `ph2d-ecs`, `ph2d-render`, `ph2d-script`, `project_schema.rs`, nem em contrato
congelado.**

## §3 — Superfície de colisão (saída do `collision-surface.sh`, colada)

```
SUPERFÍCIE DE COLISÃO — line/sculpt3d contra main
  merge-base 53832c884   ·   81 commit(s)   ·   153 arquivo(s)
▸ SCHEMAS
    PROJECT_SCHEMA                        114   (base: 114)
      └ tripla do gate               (114, 13, 18)   (base: (114, 13, 18))
    VEC_SCENE_SCHEMA                       18   (base: 18)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 18   (base: 18)
▸ REGISTRO DE COMPONENTES
    ph2d-render (espelho)                  80   (base: 80)
    ph2d-script (espelho)                  80   (base: 80)
▸ CONTRATO CONGELADO (§6)
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado
▸ ADR — esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock — 1 pacote '+name' novo: "ph2d-cloth"  (INTERNO)
▸ MARCADORES DE CONFLITO — nenhum nos arquivos da linha
▸ TETOS DE LOC — nenhum arquivo da linha passa do teto
```

⇒ **Zero números disputados.** A integração é, na prática, um `--ff-only` mais o resíduo textual dos
cinco ficheiros do §2.

⚠️ **Os ficheiros de código MODIFICADOS** (onde uma linha paralela pode encostar): 22, dos quais 16
são `ph2d-sculpt3d`, 4 são `ph2d-panel-sculpt3d`/`editor-core`/`i18n` e **4 são da shell**
(`sculpt3d_input.rs`, `sculpt3d_pull.rs`, `sculpt3d_undo_tests.rs`,
`tests/the_sculpt_gesture_is_wired.rs`). Os outros **26 ficheiros de código são NOVOS** — a crate
`ph2d-cloth` inteira (15) e onze irmãos por corte de responsabilidade.

## §4 — Contratos congelados encostados

**Nenhum.** `Tool=12` intocado (a navegação e o gesto do sculpt vivem na shell, ADR-0150);
`NodeOp`/`OpResolver`/`NodeManifest` intocados; a superfície do vector intocada.

## §5 — O que só o `ship.sh` pega (o gate de integração NÃO roda)

Corridos por mim nesta árvore: `cargo fmt --all -- --check` **limpo** · `clippy --all-targets` limpo
nas cinco crates tocadas · `typos` limpo nos caminhos da linha · `doc-index.sh --check` **em dia** ·
`architecture_workspace_file_loc_cap` verde.

⏳ **Por correr, e só o `ship.sh` os corre:** `cargo machete` · `cargo deny` · `cargo audit` ·
`nextest --cargo-profile ci-test` sobre a **workspace inteira** · o `physics_ecs_c9` na matriz de
três OS. ⚠️ **A crate nova (`ph2d-cloth`) não tem dependência externa**, então `deny`/`audit` não têm
superfície nova para reprovar — mas o `machete` pode acusar uma dependência interna não usada.

## §6 — Ordem, dependências, e o que smokar

⭐ **Esta linha não depende de nenhuma outra e nenhuma depende dela** — a crate é nova e o resto é
append-only. Pode entrar em qualquer posição da fila.

**O smoke, para o integrador confirmar depois de fundir:**

```
cd <árvore integrada> && cargo run -p ph2d-host-desktop --release
```
1. Pill **SCULPT** no topo · pincel **Cloth**.
2. As duas fileiras **Deformation** (8 chips) e **Simulation Area** (3) aparecem **só** com ele na mão.
3. Arrastar deforma o pano com a vizinhança a acompanhar; **Ctrl+Z** desfaz o traço inteiro.
4. ⛔ **Deu errado** se as fileiras aparecerem com outro pincel, se algum dos oito modos não mudar
   nada em relação ao anterior, ou se aparecer uma agulha fina saindo da superfície.
5. `PH2D_CLOTH_LAW=vbd` volta à lei anterior, para bissecar.

## §7 — ⚠️ SEIS coisas que uma leitura rápida do diff entende ao contrário

1. **A lei da referência ser a omissão não é «ligar o que estava pronto»** — ela expôs três leis
   transversais que o adaptador não honrava (o alpha, a simetria, a declaração de inversão).
2. **Duas das três barras do gate de artefactos foram RETIRADAS, não afrouxadas** — elas reprovavam
   a saída do próprio alvo (espinho `0,900` e estica `3,72×` nele, contra `0,690` e `2,98×` do
   defeito reproduzido).
3. **`PH2D_CLOTH_LAW=vbd` não é «o antigo», é o REPROVADO** — recusado pelo dono três vezes com foto
   e sem paridade medida em modo nenhum. Fica porque a bissecção é caminho suportado.
4. **O gate 20 da espec foi corrigido pela medição, não implementado à letra** — «a nossa assimetria
   nunca passa a do oráculo» não é propriedade de nenhum dos dois lados num regime caótico.
5. **Três tectos de LOC curados por CORTE, e dois estavam vermelhos ANTES desta jornada**,
   invisíveis a todos os portões da linha (o `--bins` não alcança os gates que vivem em `tests/`).
6. **O `plano_apertar_ponto_radial_local` a `1,380` não é dívida de lei** — é o regime em que o alvo
   deixa de ser determinista, e a fixture de força reduzida prova-o com erro `0,000`.

## §8 — ABERTO, com o número de cada um

Está tudo em [`06_o_plano_do_que_falta.md`](../cloth/06_o_plano_do_que_falta.md), com o dossiê por
item. O resumo: `27` dos `56` traços fora da barra, em **cinco** famílias que não são uma só —
Push (`0,944`), a esfera fora do Q12, o Snake Hook a partir do passo 3, o Inflate, o Expand — mais o
aperto com força alta, que é **decisão do dono** (a espec §5.2-ter põe as duas frases).

⭐ **O item mais barato da fila tem a lei já escrita e atestada na espec e ainda não implementada:**
o Push (§4.2-bis).

## §9 — Portão de fecho corrido nesta árvore

| suíte | resultado |
|---|---|
| `ph2d-cloth` | **33** passaram · 5 ignorados |
| `ph2d-sculpt3d` | **391** passaram · 148 ignorados |
| `ph2d-panel-sculpt3d` | **77** passaram |
| `ph2d-host-desktop` | **5192** passaram · 297 ignorados |
| `a_cloth_stroke_undoes` (GPU, `#[ignore]`) | passou, pelo braço real do gesto |
| `architecture_workspace_file_loc_cap` | verde |
| `cargo fmt --all --check` · `clippy --all-targets` · `typos` · `doc-index --check` | limpos |

## §10 — Resumo colável

```
line/sculpt3d — o PINCEL DE TECIDO (W10). 81 commits, 153 ficheiros, base 53832c884.
Crate NOVA ph2d-cloth (solver + gesto, clean-room do Blender sob a SPEC atestada).
O pincel Cloth passa a correr a lei da referência por omissão: 29 dos 56 traços do
oráculo dentro da barra de paridade (7 ao bit) contra 11 de 51 antes, e os oito modos
de deformação e as três áreas passam a ter chip no painel. 12 gates, todos provados
por mutação. ZERO schema mexido, ZERO contrato congelado, ZERO ADR, ZERO teto de LOC.
Foundational: dois arrays de ids e duas chaves de i18n, os dois em ficheiros que já
são do sculpt3d, append-only. Aprovado em smoke pelo dono três vezes.
```

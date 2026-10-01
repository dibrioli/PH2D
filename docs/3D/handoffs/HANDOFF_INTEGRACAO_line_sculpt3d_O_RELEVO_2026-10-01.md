# HANDOFF de INTEGRAÇÃO — `line/sculpt3d` · a água que roda e o RELEVO do impasto na peça (2026-10-01)

> **Para quem é:** o agente INTEGRADOR (só por ordem do dono — CLAUDE.md §0.7). Este documento é
> a superfície de colisão MEDIDA, o que mudou fora da família, os números e os smokes. A narrativa
> de cada assunto vive no doc dele — [`28`](../28_a_marca_da_agua_depois_de_rodar.md) (a água) e
> [`29`](../29_plano_o_relevo_do_impasto_na_peca.md) §5–§8 (o relevo) — e o post-mortem do impasto
> está no [`BUGS_painter.md`](../../Painter/BUGS_painter.md) `## Bug #29`, por ordem do dono.
>
> ⚠️ Ele **SUPERSEDE** o [handoff de 25/09](HANDOFF_INTEGRACAO_line_sculpt3d_A_LINHA_2026-09-25.md)
> como documento de integração desta linha: aquele já está no `main` (é o merge-base).

## §0. Estado em uma tabela

| | |
|---|---|
| ramo | `line/sculpt3d` · worktree `Worktrees/line-sculpt3d` |
| merge-base | `912a9652e` (= o `main` de agora: **0 commits atrás**, rebase **desnecessário**) |
| commits | **16** (14 de produto + 2 deste handoff e do doc de bugs), `93` ficheiros, `+7 878 −283` antes deste doc |
| smoke do dono | ✅ **aprovado em cada assunto** (a água 29–30/09; o relevo W1–W4 30/09; §6, §7 e §8 em 01/10) |
| contrato congelado (§6) | **intocado** (`node.rs` · `tool.rs`) |
| `PROJECT_SCHEMA` · `VEC_SCENE` · `FLIP` · `DOC_VERSION` · `FIELD_DOC_VERSION` | **0** (176 · 22 · 13 · 18 · 23, iguais à base) |
| registos de componentes (`ph2d-ecs` · `-render` · `-script`) | **0** (108 · 109 · 109) |
| `SCULPT_DOC_VERSION` | **3 → 5**, DENTRO do blob da escultura, com degrau para os dois (ver §3) |
| ADR | nenhum |
| `Cargo.lock` | nenhum pacote externo novo |
| tectos de LOC nos ficheiros tocados | nenhum passa (o `collision-surface.sh` confirma) |

## §1. O que a linha traz — dois assuntos

### 1.1 A tinta MOLHADA sobrevive a rodar a vista (29–30/09, sete commits)

O Painter na peça (meio *Wet Paint*): rodar/afastar a vista **não** fecha a pincelada que escorre,
não tira o Painter da mão, e o traço seguinte depois de rodar **não** deixa marca clara.

- `899403367` — a pincelada fecha só no braço do ESQUERDO; as teclas de VISTA ficam de fora
  (`keys_view::so_mexe_na_vista`, que lê a mesma tabela do `camera_key`).
- `06c1426e9` — **só um gesto que ESCREVE na peça toma o canvas** (`Drag::escreve_na_peca`,
  `match` exaustivo; `Sculpt3dScene::o_gesto_escreve_na_peca`).
- `431d9f1e6` — o PBR passa a ler a cor da tinta fina (dois ramos do `fs_core` liam o `in.vcolor`
  do vértice); **as abas da escultura e do Painter trocam a ferramenta** (§2); o registo da água.
- `82b0f9ee9` — a água **muda de vista** com a peça: `ph2d_wet_paint::grid::reproject_grid` +
  `reproject_screen_canvas` (Painter) + `tela_origem::origem` (escultura).
- `f08ccfbdd` · `ca47a273e` — a marca clara: a pincelada molhada e as que a continuam são uma
  **CADEIA** (`BaseDaCadeia`), e a semente é a peça SEM a cadeia. Doc 28 + `BUGS_sculpt3d #5`.

### 1.2 O RELEVO do impasto na peça (30/09–01/10, sete commits)

A etapa 3b do Painter na peça, por decisão do dono de 24/09: **relevo de LUZ** — o impasto ganha
espessura que pega luz e sombra, a forma não muda, `Ctrl+Z` desfaz-a, e ela não entra na silhueta
nem no ficheiro exportado.

| commit | o quê |
|---|---|
| `c36fe91f8` W1 | a tinta fina ganha um canal de relevo (`ph2d-mesh-colors::relevo`), a janela de desfazer, o documento `v4` |
| `c5fbff7ec` W2 | o canal sobe à placa (`@binding(7)`), a leitura `vec4`, o *bump* num `fs_core_n` que recebe a normal |
| `82a057060` W3 | o Painter pousa a ESPESSURA na peça; a luz 2D do impasto desliga-se NA TELA DA VISTA |
| `09b947ace` W4 | a voz do pen-down: relevo sem tinta fina diz que só a cor fica (chave i18n nova) |
| `27d8e9b85` §6 | o **CORPO** (o halo da borda) e a **SEMENTE** (as ferramentas sobre o relevo); documento `v5` |
| `f41733e31` §7 | a normal deixa de depender da VISTA: gradiente EXACTO, zero derivadas de ecrã |
| `3a5f70423` §8 | a normal inclinada não passa o **HORIZONTE** (a meia-lua na ponta vista de lado) |

## §2. O que mudou FORA da família — e porque é aditivo

| ficheiro | o quê | aditivo porque |
|---|---|---|
| `ph2d-editor-core/src/screens/hero/slot_tabs_ferramenta.rs` (**novo**, 90 L) | `intencao_da_aba(painel, ferramenta, modo_img) -> Option<EditorAction>` — clicar na aba da escultura com uma ferramenta de imagem pede `CancelActiveTool`; na do Painter com o IMG ligado pede `ActivateTool` | módulo irmão novo; **zero variantes novas no `action_bus`** |
| `ph2d-editor-core/src/screens/hero.rs` (+2) · `hero/slot_tabs.rs` (+8) | declara o módulo · `apply_event` empurra o pedido depois do `bump_panel_z` | append; o ramo só existe quando a função devolve `Some` |
| `ph2d-i18n/src/app_sculpt3d.rs` (+4) | `app.sculpt3d.recusa.o_relevo_pede_a_tinta_fina` | uma chave nova no ficheiro da família |
| `ph2d-wet-paint/src/grid/reproject.rs` (**novo**) + `grid.rs` (+4) | `reproject_grid` (copiar a grade por um mapa célula→célula, sem interpolar) | função nova; o solver não é tocado |
| `ph2d-mesh-colors` (`relevo.rs` **novo**, `amostragem.rs`, `lib.rs`, `uniformiza.rs`) | o canal `[altura, corpo]` opcional, lido pelos MESMOS pesos da cor (`pesos_tri`/`pesos_quad`) | `Option`: uma peça sem relevo não paga um byte; a ordem de acumulação da cor é a de sempre (ao bit) |
| `ph2d-mesh-render` (`tinta.wgsl`, `fonte.rs`, `relevo_normal.rs` **novo**, pipeline/upload) | `@binding(7)` + `TintaLida::g` + `tinta_inclina`/`tinta_horizonte` | o relevo é um **bit** do `armado` (`TINTA_RELEVO`); sem ele o `fs_main_tinta` toma o ramo de sempre e a normal **nem é tocada** |
| `ph2d-app-field3d/src/mode.rs` (+4) | só doc-comment | — |
| `shells/desktop/src/sculpt3d_host.rs` (+7 −1) | `if tomou && escreve` | ver o ⚠️ da catraca no §4 |
| `shells/desktop/src/render_loop/fase_painter_dispatch.rs` (+14) | lê `clay_on_screen` antes de desmontar o `gfx`; chama `abas::abas_seguem_a_ferramenta` **depois** da ponte do Painter | ⚠️ a ORDEM é load-bearing: a ponte reescreve a visibilidade da aba do Painter em todo quadro |

## §3. O formato do documento da escultura — `SCULPT_DOC_VERSION` 3 → 5

Mora **dentro do blob** da escultura, não no `PROJECT_SCHEMA` (que não se mexe). Dois degraus,
os dois com migração e gate:

- **`v4`** — o plano de tinta fina ganha `relevo: Option<…>` (a altura). Um `v3` abre **sem**
  relevo e com a tinta ao bit (`um_documento_v3_abre_sem_relevo_e_com_a_tinta_ao_bit`).
- **`v5`** — o relevo passa a ser o PAR `[altura, corpo]`. Um `v4` abre com o corpo **derivado**
  (`1` onde há altura, `0` onde não há — antes do corpo uma altura não nula só nascia debaixo de
  tinta): `um_documento_v4_abre_com_o_corpo_derivado_da_altura`.
- Um plano **sem** relevo custa **um byte** a mais do que no `v3` (o `None`), com gate.

⚠️ **Se outra linha desta rodada mexer no `doc.rs` da escultura, o número CONTA-SE** — o valor
certo raramente está num dos lados de um conflito (§5.0).

## §4. O que uma leitura rápida do diff entende ao contrário

1. **O relevo NÃO muda a geometria.** É uma normal de sombreado; a malha, a silhueta e o ficheiro
   exportado ficam iguais (decisão do dono). Quem procurar a altura no `MeshData` não a acha — ela
   vive no plano de tinta fina.
2. **Sem `Paint Detail` (tinta fina) não há relevo** — de propósito, e o pen-down DIZ-o (W4).
3. **O CORPO não é a cor.** É a cobertura da tinta (`0..1`), e é ele que impede o halo de
   acender; no `Ctrl+Z` um traço que só muda o corpo é um traço que mudou o relevo (gate).
4. **A semente guardada é a que a tela DEVOLVE**, nunca a enviada (um ULP de ida-e-volta).
5. **O `tinta_inclina` só comprime perto do HORIZONTE** e só abaixo da base: com corpo `0` devolve
   a base **ao bit**, e acima do limiar é o gradiente de superfície intacto.
6. **As abas trocam a ferramenta pelo CLIQUE**, nunca por *«quem está à frente»* — que muda sozinho
   quando um painel abre ou fecha.
7. ⚠️ **A shell CRESCEU `+20` linhas** (dois sítios, §2). A catraca `the_shell_only_shrinks` é
   corrida pela varredura impactada e o veredito dela está no §5; se a árvore COMBINADA da rodada a
   reprovar, a cura é MOVER para a crate da família, **nunca subir o número** (CLAUDE.md §2).

## §5. O portão desta linha (01/10)

Corrido de uma vez sobre o diff acumulado contra o merge-base (`load` `24 → 10` durante a corrida):

| passo | resultado |
|---|---|
| `BASE=$(git merge-base main HEAD) bash scripts/nextest-impacted.sh` | **18 853 / 18 853** verdes (11 511 fora do alcance) — inclui os gates GLOBAIS (tectos de ficheiro, **`the_shell_only_shrinks`** com os `+20`, censo de órfãos) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` | verdes |
| `cargo clippy --all-targets -- -D warnings` nas 10 crates tocadas (+ a shell) | zero avisos |
| `censos-da-arvore-combinada.sh` (HR-15 e tectos de LOC da SOMA) | **127 / 127**, controlo do filtro `12 de 12` — ⚠️ sobre esta árvore, que É o `main` + a linha (0 atrás) |
| `cargo fmt --all --check` | limpo |
| pré-voo dos **17** arneses de mutação | todas as âncoras casam uma vez |
| GPU com adaptador (`ph2d-mesh-render --test it -- --ignored relevo tinta`) | **7 / 7**; paridade do horizonte placa/CPU pior `9,2e-7` em `4 096` entradas |

`cargo machete` não corrido: a linha **não moveu código** entre crates nem mudou um `Cargo.toml`.

## §6. Mutação

| arnês (`docs/3D/ferramentas/`) | placar |
|---|---|
| `muta_o_horizonte_do_relevo.sh` (§8, sem placa) | **6 de 6** sangram + CONTROLO sobrevive; a 7.ª (o `K` do WGSL ≠ o da CPU) só a placa vê e **sangra** (`0,0507` contra `0,0265`) |
| `muta_a_normal_do_relevo.sh` (§7, placa) | **7 de 8** — a 8.ª é o CONTROLO |
| `muta_o_gemeo_em_wgsl.sh` (placa) | **5 de 6** — a W5 sobrevive de propósito (nomeada no arnês) |
| `muta_o_relevo_na_peca.sh` · `muta_o_relevo.sh` · `muta_o_plano_no_ficheiro.sh` | `16/17` · `10/11` · `17/18` (o último de cada é o CONTROLO), medidos no §6 |

⚠️ **O pré-voo de âncoras (`MUTA_SO_ANCORAS=1`) apanhou TRÊS âncoras mortas** nesta jornada: as
`N6`/`N7` (a lei mudou do `fonte.rs` para o `tinta.wgsl`) e a `W2` — morta pelo **meu** `f41733e31`,
que partiu um `if` numa linha em quatro. *Uma âncora que casa zero lê-se num placar como uma
mutação que sobreviveu.* As três foram re-ancoradas e o pré-voo dos dezassete arneses fecha
inteiro.

## §7. Smokes

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d && env PH2D_SCULPT3D_SMOKE=52 cargo run -p ph2d-host-desktop --profile smoke
```

- **Relevo:** `Paint Detail` → `8x`; `IMG` → `PNTR`; meio `Impasto`; um traço grosso. De frente
  liso; a borda sem anel cinzento; inclinado sem estilhaços; com a ponta junto ao contorno da bola
  sem meia-lua. `Smooth` e a faca mexem no relevo.
- **Água:** meio `Wet Paint`; pinte, rode a vista com o botão direito enquanto escorre — continua
  a escorrer, o Painter fica na mão, e o traço seguinte não deixa marca clara.

Prova de que o binário do comando acima está COMPILADO nesta worktree (a 2.ª corrida, depois do
`rm -rf target/*/incremental` e de uma 1.ª de `28,65 s`):

```
$ bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke
    Finished `smoke` profile [optimized] target(s) in 0.23s
```

## §8. A UMA linha que proponho para o `CLAUDE.md` §5 (3D / Sculpt)

> ⭐⭐⭐ **E A ETAPA 3b FECHOU — o IMPASTO pinta RELEVO na peça 3D, e a tinta molhada sobrevive a rodar a vista** (01/10, smoke do dono aprovado em cada assunto): relevo de LUZ no plano de tinta fina (`[altura, corpo]`, `SCULPT_DOC_VERSION` 3 → 5 com degraus), gradiente EXACTO sem derivadas de ecrã e normal comprimida contra o horizonte; post-mortem no [`BUGS_painter #29`](docs/Painter/BUGS_painter.md); ⏳ a faceta da retícula numa encosta muito inclinada e a etapa 4 (camadas e efeitos). [Handoff](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_sculpt3d_O_RELEVO_2026-10-01.md).

## §9. ABERTO

- ⏳ A altura é contínua e o gradiente **não** (por célula): numa encosta muito inclinada vê-se a
  faceta da retícula em degraus. Cura nomeada: interpolar gradientes por amostra; custo por medir.
- ⏳ A etapa 4 do Painter na peça (camadas e efeitos) espera a decisão do dono.
- ⏳ O relógio do upload incremental com relevo não foi medido numa máquina calma (o critério de
  desistência do doc 29 §4 é `> 1 ms` por quadro).

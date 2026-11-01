# HANDOFF de continuação — line/UIUX, F41 (01/11) → F42

Worktree `Worktrees/line-UIUX`, branch `line/UIUX`, HEAD `39c4b31b8d`, merge-base com o main `422fec2027`
(36 commits). **Nada integrado, nada enviado.** A F41 resolveu todo o §2 do
[handoff F40](HANDOFF_CONTINUACAO_line_UIUX_2026-10-30_F40.md) e passou por **três rodadas de smoke do
dono, todas OK em 01/11** (§3). Antes de ler: `cd` + `pwd` + `git branch --show-current`.

## 1 — Commits da F41 (de `5d843292ca` em diante; `git log --oneline main..HEAD`)

**Itens do §2 da F40**

| sha | o quê |
|---|---|
| `5d843292ca` | Follow Value sobre uma CAIXA lê o mesmo estado depois da v25; `PROJECT_SCHEMA` 209→210 |
| `3d27d2cec0` | regras do mundo e da mesa voltam com o Desfazer (F30 §6 item 8) |
| `3f7bfbb40d` | matriz de camadas da física animável: um losango, 36 canais (F30 §6 item 6c) |
| `65b4c71733` | testes: seleção/guias/laço do vetor no acento do tema; o losango de Segments grava |
| `031c692b6c` | o Length do osso empurra o filho e estica o desenho (decisão do dono 30/10); `PROJECT_SCHEMA` 210→211 |
| `b3b8a5d53d` `ddb1570602` | 3 provas para os sobreviventes da mutação do comprimento; o load garante o degrau da pele sem depender da janela |
| `5a77cac9e3` | `ph2d-app-skeleton` de volta sob o tecto: 11424→11085 (cap 11263) |
| `bc37b01e1a` | clippy do fecho (type `Pintor` no teste do acento) |
| `a546a6009a` | fotos do smoke F40+F41 |

**Correções do smoke 1**

| sha | o quê |
|---|---|
| `8437a846f6` | a pele mistura o esticão `Sᵢ=R(θᵢ)ᵀ·Lᵢ` (`centro.rs`, `blend_com`); osso esticado vai pela CPU; `COMPRIMENTO_MINIMO=1e-3` (filhos voltam depois de passar por 0) |
| `855f18928d` | o painel da Física pinta e vira o `vivo`: `Toque{visto,pedido}`, `ConducaoMundo::tocar` |
| `a70e80742a` | um Ctrl+Z por gesto: `history.rs` `commit_da_cena`/`liga_gesto_da_cena`, `undo.rs` `linha_do_topo` |
| `489af2ca9f` | cena `PH2D_CONSTRAINT_SMOKE=13` «Switch and Lamp»; `ph2d-app-constraints` 10693→10476 (helpers disco/bloco) |
| `052864cdcf` | `AutokeyState::esquece_o_quadro` após o undo (o quadro não recria a chave que o Ctrl+Z tirou) |

**Smoke 2**

| sha | o quê |
|---|---|
| `fedaa036d1` | fotos (osso 8, física 5) |
| `c9174f0df6` | `ph2d-panel-skeleton`: `set_current_bone_subject`/`abandona_a_edicao_do_osso` descartam texto/arrasto pendente dos 15 `VECTOR_BONE_FIELDS` ao trocar de osso; `bone_bridge::on_focus` arma o Transform |
| `28d17458e4` | `ph2d-editor-core` `interaction/toggle_paint.rs`: premir vira, arrastar pinta o estado. `PHYSICS_LAYER_PAINT_GROUP` (matriz Collision Layers) e `HIER_TOGGLE_PAINT_GROUP` (olho, cadeado, cadeado-do-grupo na Hierarchy, vertical e horizontal); um passo de desfazer por arraste; excluídos o chevron e os ícones de componente; 13/13 mutações vermelhas |

**Smoke 3**

| sha | o quê |
|---|---|
| `6355acf02d` | foto (física 5) |
| `39c4b31b8d` | `on_focus` arma o Transform também FORA da mão: em Object o 1.º clique escolhia o osso e o gizmo de objecto arrancava-o do pai. Efeito: escolher osso ⇒ Pose; `Tab` volta a Object. Mutação vista vermelha |

Fotos em `img_F41/` (34 ficheiros; `.log` + `.png` por cena). Há `.log`/`.png` NÃO rastreados em
`img_F30/F33/F34/F40/F41` (`git status`): não são da F41-código; comite-os só se quiser a foto.

## 2 — ABERTO

Nada de defeito.

- **(a) Integração da linha**: só por ordem do dono (36 commits sobre o main `422fec2027`; o main pode ter
  andado — leia `collision-surface.sh` pelo caminho absoluto). Degrau do schema: `python3
  scripts/schema-recount.py <valor no main>` (exige argumento).
- **(b) Fora de escopo por ordem do dono**: recuo da Luz 2D sob blocos pequenos
  ([BUGS_luz2d.md](../../Luz2D/BUGS_luz2d.md) §8).
- **(c) PRÓXIMO ITEM, F42** (pedido do dono, 01/11, verbatim): «Agora quero que cada tipo de objeto na
  hierarchy tenha um ícone único, bem representativo de sua classe e com cores diferentes dos outros
  tipos». Notas:
  - HR-15: zero hex; as cores entram como tokens do tema.
  - Mapear TODOS os tipos de objecto que a Hierarchy lista — **contar no código, não escolher**.
  - Maquete/foto ANTES de mandar ao dono (ele não é engenheiro: mostre, não descreva).
  - Respeitar a memória `project_hierarchy_order_rules_drawing_and_click_2026_11_01` (no índice do
    carregador; ainda não está em `project-memory/` desta linha — sem link) e o ícone de COMPONENTE que já existe (componente = ícone + cartão; não colidir com ele).
  - Os ícones de interruptor (olho/cadeado) agora pintam ao arrastar (`28d17458e4`): o ícone de tipo
    NÃO é interruptor e fica fora do grupo `HIER_TOGGLE_PAINT_GROUP`.

## 3 — Smoke (o dono já fez as 3 rodadas, 01/11: OK)

Passos para repetir (dono). Todos a partir de `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX`
com o binário `smoke` compilado do HEAD (`cargo build --profile smoke` via `bash scripts/ph2d-run.sh`).

1. `PH2D_VEC_BONE_SMOKE=8` — Length por osso; escolher um osso abre o Transform; o filho acompanha, o
   desenho estica. Errado se: o filho fica parado ou o traço rasga.
2. `PH2D_PHYSICS_SMOKE=5` — arrastar na matriz Collision Layers e nos ícones da Hierarchy pinta vários de
   uma vez; um Ctrl+Z desfaz o arraste inteiro. Errado se: precisa de vários Ctrl+Z.
3. `PH2D_CONSTRAINT_SMOKE=13` — «Switch and Lamp»: o interruptor acende/apaga a lâmpada.

Nota: fotos em tela virtual não clicam; os cliques estão provados por testes + o smoke do dono.

## 4 — Números contados

- `PROJECT_SCHEMA` = **211** (verificado em `shells/desktop/src/project_schema.rs:556`). Recontar com
  `schema-recount.py <n do main>` antes de citar o degrau.
- `ph2d-app-skeleton` 11085 (cap 11263) e `ph2d-app-constraints` 10476: valores dos commits
  `5a77cac9e3`/`489af2ca9f`, **NÃO reverificados** aqui (a catraca de LOC não foi achada por `git grep`;
  `wc -l` bruto da crate dá outra medida — a regra é a do gate `the_shell_only_shrinks`/tecto de LOC).
- Cenas: `PH2D_CONSTRAINT_SMOKE` 1..13; `PH2D_VEC_BONE_SMOKE` até 8; commits da linha sobre o main: 36.

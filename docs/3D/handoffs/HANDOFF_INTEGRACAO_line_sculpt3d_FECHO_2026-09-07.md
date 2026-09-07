# HANDOFF DE INTEGRAÇÃO — `line/sculpt3d`, fecho de 2026-09-07

> ⚠️ **Este doc descreve o mundo no dia em que foi escrito.** O estado vivo é o `CLAUDE.md` §5.
> ⛔ **A linha NÃO integra e NÃO pusha** (`CLAUDE.md` §0.7). Ela fecha, entrega isto, e PARA.

## §1 — Identidade

| | |
|---|---|
| **Ramo** | `line/sculpt3d` |
| **Worktree** | `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-sculpt3d` |
| **merge-base** | `815555aed` · **74 commits** · **124 ficheiros** |
| **Smoke** | ✅ **aprovado pelo dono em 2026-09-07** |
| **Assunto** | fecho da **W10 (pincel de tecido)** + a cura de um defeito de teclado que **não é do tecido** + o método `_ComoInvestigarApps` |

## §2 — Foundational / partilhado tocado, e por quê

| ficheiro | o quê | por quê aqui |
|---|---|---|
| `crates/ph2d-editor-core/src/interaction/dispatch/blur.rs` | **NOVO** — a porta `blur_focus` + `depart_focus` | ponto de extensão **append-only**: nenhuma assinatura existente mexida |
| `.../dispatch/mod.rs` · `.../interaction/mod.rs` | `mod blur;` + dois re-exports | duas linhas, aditivas |
| `.../dispatch/pointer_down.rs` | o bloco de partida de foco passa a **CHAMAR** a porta | −10/+6 linhas; comportamento idêntico, com gate a prová-lo |
| `crates/ph2d-editor-core/src/screens/hero.rs` | `HeroScreen::blur_focus` | método novo, aditivo |
| `shells/desktop/src/forwarding.rs` | `forward_blur_to_hero` | irmão do `forward_to_hero`, aditivo |
| `shells/desktop/src/input_dispatch.rs` | 1 bloco de 10 linhas no `on_mouse_input` | ⚠️ **posicional** — tem de ficar ANTES dos três consumidores que devolvem cedo |
| `scripts/doc-index.sh` | 1 entrada de directório | a lista é append-only por construção |
| `CLAUDE.md` | **§0.9 novo** · 1 linha no roteador §1 · a linha da W10 no §5 | ⚠️ o §0 passou de «memorize os 9» para **os 10** |

⚠️⚠️ **A colisão mais provável desta linha é o `CLAUDE.md`** — §0, §1 e §5 num commit só. As três
edições são **inserções em pontos distintos**; se houver conflito, ele é textual e o Mergiraf resolve.
⛔ **A numeração do §0 é o único sítio que exige olho:** se outra linha também acrescentou um
inegociável, o número **conta-se**, não se escolhe (`CLAUDE.md` §5.0).

## §3 — Superfície de colisão (`collision-surface.sh`, colada)

```
PROJECT_SCHEMA 121 (base 121) · tripla (121,13,22) igual à base
VEC_SCENE_SCHEMA — · FLIP_SCHEMA 13 · DOC_VERSION 18   — todos IGUAIS à base
registo de componentes: 82 / 82 (espelhos), igual à base
contrato congelado (§6): node.rs INTOCADO · tool.rs INTOCADO
ADR: esta linha não cria nenhum ⇒ fora de toda disputa de número
Cargo.lock: nenhum pacote externo novo
marcadores de conflito: nenhum
tetos de LOC: nenhum ficheiro da linha passa do teto
```

⭐ **Nenhum número partilhado se mexeu.** Esta linha não disputa schema, registo, ADR nem contrato.

## §4 — Contratos congelados encostados

**Nenhum.** `Tool=12`, `NodeOp=2`, `OpResolver=1`, `NodeManifest=8` intocados — e o
`collision-surface.sh` confirma-o por ficheiro.

## §5 — O que só o `ship.sh` pega (o gate desta árvore NÃO roda)

- `cargo machete` · `cargo deny` · `cargo audit` — não corridos aqui.
- `typos` **project-wide**: ⚠️ existe **um** erro **pré-existente no `main`** — a palavra portuguesa
  para *contexts*, em maiúsculas, na linha do Input Map no §5 — que **não é desta linha**, e que o
  `ship.sh` corre como `run_optional`. ⛔ **Não a cite literalmente num doc novo**: citá-la cria um
  segundo erro, e foi o que a 1.ª redacção deste handoff fez.
  ⭐ Os ficheiros novos desta linha estão **limpos** no `typos` (verificado).
- `physics_ecs_c9` na matriz 3-OS: esta linha não toca física.

## §6 — Ordem, dependências e o que smokar

**Sem dependência de outra linha.** Integra sozinha, em qualquer ordem.

Smoke de regressão depois de integrar (o defeito curado é do SHELL, não do 3D — ele vale para
qualquer módulo que tome o canvas):

```
cd <árvore integrada> && env PH2D_SCULPT3D_SMOKE=1 cargo run -p ph2d-host-desktop --release
```
1. Pincel **Cloth** → clique no **número** ao lado de um cursor (não no cursor: no número).
2. Volte a arrastar na esfera. **`Ctrl+Z`** tem de desfazer; **`Del`** tem de apagar a peça.
3. Digite um valor no número e clique **direto no barro sem Enter**: o valor tem de **valer**.

## §7 — ⚠️ SETE coisas que uma leitura rápida do diff entende ao contrário

1. **O «undo do Cloth» nunca esteve partido.** O contrato dele já era gateado
   (`todo_vertice_movido_foi_capturado`); o que faltava era o `Ctrl+Z` **chegar**. ⛔ Não procure
   defeito no `ph2d-cloth`: não há.
2. **O bloco novo no `input_dispatch.rs` é POSICIONAL.** Movê-lo para depois de qualquer um dos três
   consumidores reintroduz o defeito inteiro — há gate, e ele nomeia o report.
3. **`blur_focus` não é `set_focus(None)`.** Ela **compromete** o buffer numérico e o hex antes de
   largar. Simplificá-la faz o número que o artista digitou **evaporar em silêncio**.
4. **A idempotência é load-bearing, não elegância.** É ela que deixa a shell chamar a porta à frente
   de um consumidor que talvez não consuma — sem ela, todo clique de canvas não consumido emitiria
   `ValueChanged` **a dobrar**.
5. **O gate do shell lê o FONTE de propósito**, e o doc dele diz porquê: a propriedade é *posicional
   dentro de uma função*, e o caminho vivo exige uma `AppGfx` com surface de janela real. ⚠️ Ele
   **recusa ficar vácuo** — cada âncora é exigida por si.
6. **As 4 anotações `LITERAL-PX-OK` no `rows_cloth.rs` não mudam comportamento.** Elas curam um
   vermelho **desta linha** que o fecho anterior não alcançou: o `no_magic_numeric` vive em
   `ph2d-editor-core/tests/` e **varre as crates de painel irmãs** — nem a suíte do painel nem a do
   shell lá chegam.
7. **O `_ComoInvestigarApps` NÃO é um doc de clean-room.** Ele é o método **geral** (vale para alvo
   permissivo, onde não há parede nenhuma); a SKILL continua a ser a parede, e agora aponta para ele.

## §8 — ABERTO, com o número de cada um

| item | número |
|---|---|
| **Decisão do DONO** — o aperto com força alta (§5.2-ter): reproduzir ou limitar | `plano_apertar_ponto_plano_local`, **`2,04×` a barra**, o único dos `86` fora pela régua **p95** |
| Push · esfera fora da projecção · Snake Hook do passo 3 · Inflate · Expand | os `7` de `ABERTO_N`, cada um com o erro na bancada |
| Espessura do raio de colisão (`0,3` no alvo, fino no nosso) | ⛔ **sem lado aprovado** — não existe amostra do alvo com obstáculo |
| Portas de consola de **Audacity** e **Ardour** | ⏳ **não medidas**, e declaradas como tal no `01_o_arsenal.md` |

## §9 — Portão de fecho corrido NESTA árvore

| gate | resultado |
|---|---|
| `cargo test -p ph2d-host-desktop --no-fail-fast` | ✅ **220 suítes · 5 360 testes · 0 falhas · EXIT=0** |
| `cargo test -p ph2d-editor-core -p ph2d-panel-sculpt3d` | ✅ verde (inclui os 3 gates novos da porta) |
| `cargo test -p ph2d-sculpt3d --lib` | ✅ 358 testes |
| `cargo clippy --all-targets` (3 crates tocadas) | ✅ **0 avisos** |
| tetos de LOC + **censo de obsolescência** | ✅ `file_loc_caps` e `architecture_workspace_file_loc_cap`, as duas metades |
| `no_magic_numeric` | ✅ (era **vermelho**; curado neste fecho) |
| `bash scripts/doc-index.sh --check` | ✅ 17 índices em dia |
| **prova de mutação** | ✅ **4 mutações, 4 mortas** (M1 apagar a soltura · M2 movê-la para depois do consumidor 3D · M3 tirar o `commit_number_buffer` · M4 desligar a chamada no `dispatch_down`) |

⚠️ **Uma leitura minha foi corrigida no próprio fecho:** a 1.ª corrida da suíte do shell passou por
`| tail -30` e leu-se como verde. *Um `tail` é uma JANELA, não um veredito* — a corrida honesta
(`--no-fail-fast`, sem filtro, exit code preservado) é a da tabela.

## §10 — Resumo colável

```
line/sculpt3d @ 815555aed +74 commits · 124 ficheiros
Schemas: NENHUM mexido. Contratos congelados: INTOCADOS. ADR: nenhum. Cargo.lock: nada externo.
Foundational: ph2d-editor-core (porta blur_focus, append-only) + shells/desktop (1 bloco posicional)
Colisão provável: CLAUDE.md (§0 novo inegociável nº 9 — o NÚMERO conta-se, não se escolhe)
Fecho: 5 360 testes do shell verdes (EXIT=0), clippy 0, 4/4 mutações mortas, 1 vermelho pré-existente CURADO
Smoke: aprovado pelo dono em 2026-09-07
```

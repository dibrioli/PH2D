# Handoff de INTEGRAÇÃO — `line/GameEngineFuncional` (28/10)

Leitor: o agente integrador (DIRETRIZ §1.5.3 / §1.5.9). Smoke do dono: **aprovado 28/10** (onda 17b + o ícone no
centro do quadro). Integração e ship só por ordem do Enio.

## 1. Identidade

| | |
|---|---|
| Branch | `line/GameEngineFuncional` |
| HEAD | `022dd3dacd` |
| merge-base com o main | `c3662cd8e4` — **o main ainda está nesta base ⇒ `--ff-only` possível hoje** |
| Commits / ficheiros | 113 · 543 (+40 445 / −2 539) |
| Foundational/partilhado | 160 ficheiros, +8 153 / −837 (crates abaixo) + `shells/desktop` |
| Crates NOVAS | `ph2d-app-game` (o jogo: moldura, câmera no canvas, smoke) · `ph2d-app-sprite-anim` (editor de animação por intervalos) |

## 2. O vermelho herdado — CURADO no fecho

`todo_campo_sem_coluna_propria_ganha_a_do_losango` (onda 08: `sections/anim_rules.rs · fn rule_rows` pintava
`paint_dropdown_chip` sem a coluna do losango). Cura: a linha da regra reserva a coluna pela porta
`form_row_columns` e pinta `paint_decorator_dot` (o mesmo `x` das outras linhas; o «×» encosta à coluna); no gate,
`POR_FUNCAO` ganha `anim_rules.rs::rule_rows` = `NaoAnimavel` (a lista ESCOLHE o clip a cada quadro). O módulo
`architecture_every_editable_field_is_a_channel` inteiro: 6/6 verdes.

⚠️ Os handoffs 17/17b falam em «13 vermelhos herdados da suíte `it` da shell» sem os listar — **a suíte inteira
NÃO foi corrida na linha** (ordem do dono 23/10); o integrador corre-a e confronta com o `main` em `c3662cd8e4`.

## 3. Superfície de colisão (`scripts/collision-surface.sh`, VERBATIM)

```
SUPERFÍCIE DE COLISÃO — line/GameEngineFuncional contra main
  merge-base c3662cd8e4   ·   113 commit(s)   ·   543 arquivo(s)
───────────────────────────────────────────────────────────────────────────────
▸ SCHEMAS — ⚠️ o valor se CONTA contra o main do dia; confira nos TRÊS sítios
  ⚠ PROJECT_SCHEMA                        198   (base: 193)
  ⚠   └ tripla do gate               (198, 13, 23)   (base: (193, 13, 23))
    VEC_SCENE_SCHEMA                       23   (base: 23)
    FLIP_SCHEMA                            13   (base: 13)
    DOC_VERSION (timeline)                 23   (base: 23)
  ⚠️  esta linha TOCA project*.rs — a escada e a tripla moram em arquivos IRMÃOS;
      um degrau escrito no arquivo errado funde LIMPO e evapora.

▸ REGISTRO DE COMPONENTES — o contador é TRÊS, cada um roda só na suíte da própria crate
  ⚠ ph2d-ecs                              113   (base: 108)
  ⚠ ph2d-render (espelho)                 111   (base: 109)
  ⚠ ph2d-script (espelho)                 111   (base: 109)

▸ CONTRATO CONGELADO (§6) — deve ser INTOCADO; se não, exige ADR
    crates/ph2d-nodegraph/src/node.rs              intocado
    crates/ph2d-editor-core/src/tool.rs            intocado

▸ ADR — número escolhido numa linha paralela é PROVISÓRIO
    último no disco: 0191   próximo livre: 0192
    esta linha não cria ADR ⇒ fora de toda disputa de número

▸ Cargo.lock — pacote EXTERNO novo é o que importa; aresta interna não
  ⚠ 2 pacote(s) '+name' novo(s):
      "ph2d-app-game"
      "ph2d-app-sprite-anim"

▸ MARCADORES DE CONFLITO — inclui '|||||||' (diff3), que uma varredura de 3 marcadores NÃO vê
    nenhum nos arquivos da linha

▸ TETOS DE LOC nos arquivos que a linha tocou (700 workspace · 600 painel/shell · 500 widget · 650 tool-runtime)
  ✗  1033 / 600   shells/desktop/src/main.rs
───────────────────────────────────────────────────────────────────────────────
  ⚠️ Isto é o MAPA, não o gate. O gate mecânico é scripts/foundational-integrate.sh;
     o que exige julgamento (mesmo-símbolo, decisão de produto) continua leitura humana.
```

Leitura (DELTA, não valor): `main.rs` 1033/600 é isenção **pré-existente** e **ENCOLHEU** (base 1061). Os dois
pacotes novos do `Cargo.lock` são só arestas internas — nenhum pacote externo novo. Contratos §6 intocados; nenhum
ADR. Tabela medida contra o main de hoje = o merge-base; se o main andar, o integrador lê o valor no ficheiro
(`git show main:<arq>`) e conta o degrau da linha como delta.

### 3.1 Degraus do `PROJECT_SCHEMA` (193 → 198, +5)

Escada em `shells/desktop/src/project_schema.rs` (consts `PROJECT_SCHEMA_SEM_*` + braços do load); o campo novo do
`ProjectFile` em `shells/desktop/src/project.rs` (v198).

| Degrau | Onda | O quê |
|---|---|---|
| 193→194 | N1 | `CharacterAnimations` (as animações do personagem) |
| 194→195 | 14 | `TiledDraw` (Draw Mode Tiled) — ecs `+1`, espelhos `0` |
| 195→196 | 15 | `ImageUseMemory` (cada uso guarda a sua configuração) — ecs `+1` |
| 196→197 | 16 | `ImageRegion` (uso Region) — ecs `+1` |
| 197→198 | 17 | o **formato do jogo** (`jogo`: blob com versão própria, último campo do `ProjectFile`; os braços `197`…`189` acrescentam o `0` do `Vec` vazio) |

Todos COM leitura dos anteriores. **Blob `jogo`: `JOGO_VERSAO = 2`** (`project_settings.rs`; a 17b juntou o tamanho
do ícone, **sem** degrau de `PROJECT_SCHEMA`); v1 continua a abrir (`um_jogo_v1_abre_com_o_icone_de_fabrica`; v197
sem o campo: `um_v197_abre_com_o_formato_de_fabrica`). Registo ecs `+5` e espelhos `+2` vêm dos degraus acima e de
registos das ondas 07/08/11 — **não atribuo cada um à mão**: o integrador reconta (§6).

## 4. Foundational tocado — o que mudou e porquê é aditivo

(`git diff c3662cd8e4..HEAD --stat -- crates/<c>`)

| Crate | Δ | O quê |
|---|---|---|
| `ph2d-ecs` | 12 f, +866/−35 | `image_use_memory`, `sampling` (Tiled/Region), `lifetime`, `sim`, `sprite_anim` (reduz), registo `+5`. Componentes/ficheiros novos; registo append-only |
| `ph2d-editor-core` | 73 f, +2 827/−508 | `game_format` (formato do jogo, ícone 16–128), `game_stage`, `hier_rows::type_icon`, ids de menu Run, `action_bus`, dispatch do ponteiro. Variants/ids novos; os −508 são movimentos para as crates `app-*` |
| `ph2d-render` | 19 f, +1 716/−11 | atlas de fábrica (`atlas/fabrica*.svg`: boneco, moeda, chão, bandeira, montes, nuvens) + espelho do registo `+2`. Só adições |
| `ph2d-script` | 13 f, +1 291/−211 | ciclo de vida do script (ready/physics/process/exit, `ph2d.connect`), `api_help_life`, `api_table`, `lane`; −211 é `scene.rs` simplificado. Espelho do registo `+2` |
| `ph2d-component-desc` | 4 f, +64 | linhas de catálogo (core, image, physics) dos componentes novos — aditivo |
| `ph2d-i18n` | 11 f, +271/−28 | chaves novas (`anim_card`, `anim_editor`, `inspector_game`, asset browser, menus); as tabelas PT/EN andam juntas |
| `ph2d-physics` | 4 f, +251/−2 | `world/reshape.rs` (+ testes): refaz o collider do corpo agachado; `collider_build` ganha um parâmetro derivado |
| `ph2d-physics-ecs` | 17 f, +661/−37 | `components/anim_rules` (regras «quando → animação»), sondas do player (`player_probes`), `mortes`/`diagnostics` |
| `ph2d-sprite-sheet` | 2 f, +130 | `grid_guess` (adivinha a grelha de uma folha) — função nova |
| `ph2d-platformer` | 5 f, +76/−5 | `crouch` (corpo baixo ao abaixar) + testes |

Crates `app-*`/painéis/tools tocados (consumo, não foundational): `ph2d-app-code`, `-components`, `-macros`,
`-motion`, `-painter`, `-physics`, `-registry-init`, `-vec`; `ph2d-panel-asset-browser`, `-hierarchy`, `-inspector`,
`-registry-init`, `-timeline`; `ph2d-tool-flip`, `ph2d-tool-registry-init`; `shells/desktop`.

## 5. Ondas (plano `docs/GameEngineFuncional/NN_*.md` · handoff `handoffs/HANDOFF_CONTINUACAO_*`)

- **W1** (01) modos/seleção/inspector: Draw nos modos de imagem/vetor, o clique escolhe o de cima, `ObjectMode::Paint` saiu.
- **R0/N0** (04): o Play corre sobre cópia (o Stop devolve o documento ao bit); ciclo de vida do script; menu Run, janela do jogo.
- **N1 arte** (05/06): menu Add ▸ Game, Auto Animate = `CharacterAnimations`, atlas de fábrica vetorial.
- **07** a animação num cartão + editor por intervalos (`ph2d-app-sprite-anim`). **08** lista de regras «quando → animação», teclado no palco.
- **09** abaixar com corpo baixo (`ph2d-platformer`, `ph2d-physics::reshape`). **10** arte de agachar (aprovada 25/10).
- **11** câmara presa e fundo em parallax (Background Layer). **12** a imagem é asset do projecto (seletor Image).
- **13** isolamento por uso (medido contra a Unity). **14** Draw Mode Tiled com Largura/Altura. **15** o uso da imagem à vista (Parada|Repetida|Animada).
- **16** o 4.º uso, Region. **17** a câmera no canvas + o formato do jogo no projecto (moldura Always/When Selected).
- **17b** cura da câmera (auditoria de 2 lentes; 5 gates vermelhos antes da cura; mutação 15/16) + o **ícone no centro do quadro** (`022dd3dacd`, `moldura_tinta::icones_em` lê `PickWorld.camera_frames`).

## 6. Para o integrador

**Link novo para o CLAUDE.md §5.1** (o `grep -n "Game" CLAUDE.md` não acha entrada de Game Engine ⇒ criar a linha):
`docs/GameEngineFuncional/handoffs/HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-10-28.md`

Frase sugerida (≤ 300 car.): «**Game Engine** — o jogo de plataforma no editor: Play em janela/ecrã cheio, Player, regras de animação, uso da imagem (Parada/Repetida/Animada/Region), câmera no canvas com formato do jogo. Crates `ph2d-app-game` + `ph2d-app-sprite-anim`.» (+ smoke `PH2D_GAME_SMOKE=1..4`, este handoff e `[docs](docs/GameEngineFuncional/)`, ≤ 700 bytes.)

**Reconferir:**
1. `PROJECT_SCHEMA` nos **três sítios** (escada e tripla em ficheiros irmãos) — `python3 scripts/schema-recount.py`; contar o degrau como **+5 sobre o main do dia**, nunca copiar `198`.
2. Contadores do registo: `ph2d-ecs` (+5), espelhos `ph2d-render` e `ph2d-script` (+2) — cada um roda só na suíte da própria crate.
3. Censos, tectos de LOC e machete já verdes na linha (ver Fecho) — re-correr só se o main andar. Suíte inteira: 1×, do integrador.

## 7. Smokes (binário `smoke` já compilado)

`PH2D_GAME_SMOKE` aceita **1..4** (`CENAS = 4` em `crates/ph2d-app-game/src/smoke.rs`):

| n | Cena |
|---|---|
| 1 | o nível de plataforma: chão com vão, 2 plataformas, meta, 5 moedas; PARADA — Run ▸ Play in a Window (F5) |
| 2 | nível 2: só se monta pelo menu Add ▸ Game; Player, tecto baixo à esquerda |
| 3 | nível 2 + condutor do agachar (↓ e ← anda agachado sob o tecto baixo; solta ↓ e continua baixo; sai e levanta) |
| 4 | nível 2 com «heroi_azul» e «montes_por_do_sol» na biblioteca: imagem como asset, usos, Tiled, Region, **câmera no canvas** (a cena das ondas 12–17b) |

Comando (passo 1):
`cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=<n> ./target/smoke/ph2d-host-desktop`

Modificadores: `PH2D_GAME_SMOKE_DRIVE=window|fullscreen|editor` (condutor do Play) · `PH2D_GAME_SMOKE_FOTO=<nome>`
(instante pedido; nomes em `crates/ph2d-app-game/src/smoke_foto.rs`) · `PH2D_GAME_SMOKE_USOS=…` (sequência do seletor Use).
Fotos: `docs/GameEngineFuncional/handoffs/fotos_09` … `fotos_17`, `fotos_17b` (a 17b prova pelo ponteiro no app real,
tabela no handoff `HANDOFF_CONTINUACAO_2026-10-28_17b_CAMERA.md`).

Smoke do dono (cena 4, câmera): abrir o comando com `n=4` · selecionar «Camera» na Hierarchy · arrastar o miolo da
moldura (move o offset do Follow; a cruz da mira segue) · clicar sem mexer no miolo escolhe o objecto de baixo ·
arrastar uma borda muda só aquele limite · errado = ícone fora do centro do quadro, ou o miolo não arrasta.

## 8. ABERTO (além da §2)

- O ícone da câmera vai no CENTRO do quadro (dono 28/10, `022dd3dacd`); clique curto no próprio ícone fica com a câmera.
- O arrasto da câmera não grava chave com Auto Key; mudar o formato/ícone não tem desfazer (`ProjectSettings` fora do `ProjectState`).
- Câmera filha de outro objecto: o miolo soma o delta à translação LOCAL (pai rodado/escalado erra).
- Onda 18 (decidida, não feita): zona morta pintada, miniatura, escurecer fora; «a vista do editor segue a câmera do jogo».
- Recusado e medido (não reconstruir): selo «segue <alvo>» na Hierarchy (o selo de 3 letras é também a chave da cor).

## 9. Perfil do loop (`bash scripts/agent-loop-profile.sh`, VERBATIM)

```
PERFIL DO LOOP DO AGENTE — 20 sessao(oes) mais recentes
──────────────────────────────────────────────────────────────────────────────
  ✗ paralelismo de ferramenta              1.16/passo   alvo: >= 1,5  (12% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                128   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                265 : 117   alvo: <= 1,0  razao 2.3x (baseline: 4,3x)
  ✓ edicoes SEM conferir que casaram             1.4%   alvo: <= 2%  (10 de 706; o guarda R6 recusa-as desde 14/10)
  ✓ contexto relido por passo (media)         246 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               66 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
──────────────────────────────────────────────────────────────────────────────
  (rodapé de 2 linhas do script omitido)
```

## Fecho (portões)

Sobre o HEAD do commit de fecho (a seguir a `022dd3dacd`). Suíte inteira: NÃO corrida (é do integrador).

| Portão | Resultado |
|---|---|
| `cargo fmt --all -- --check` | ✓ |
| `PH2D_SUITE=fecho cargo clippy --workspace --all-targets -- -D warnings` | ✓ (2 m 37 s); re-corrido nas 5 crates do conserto ✓ |
| `check-standalone-optional.sh` · `check-workflow-packages.sh` | ✓ · ✓ (32 nomes / 411 membros) |
| `cargo machete` | ✗→✓: `ph2d-app-sprite-anim` precisa de `bevy_ecs` (o `#[derive(Resource)]` vem por `ph2d_ecs::Resource` e expande para `bevy_ecs::`) — `[package.metadata.cargo-machete] ignored`, com o porquê; tirar a dep NÃO compila (medido) |
| `file_loc_caps` 4/4 · `fn_loc_caps` 2/2 · `the_app_only_sheds_fields` 3/3 · `the_shell_only_shrinks` 1/1 · `arch_safe_clamp_only` 2/2 | ✓ |
| `censos-da-arvore-combinada.sh` | ✗ 119/121 → ✓ **121/121** (12/12 censos correram). Curas: `"Player"` na shell → `ph2d_app_game::smoke::player()`; `"SheetImage"` (o `impl Debug` à mão) → `debug_struct` + `NOT_LANGUAGE_TEXT` com o mecanismo |
| `every_editable_field_is_a_channel` (módulo) | ✓ 6/6 (§2) |
| `#[cfg(target_os` acrescentado pela linha | 0 |
| Smoke `cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida | 1.ª 41,19 s (incremental limpo); 2.ª `Finished smoke profile in 0.26s`, **0** `Compiling` |

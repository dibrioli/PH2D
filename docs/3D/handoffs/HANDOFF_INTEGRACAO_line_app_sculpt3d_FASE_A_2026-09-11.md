# Handoff de integração — `line/app-sculpt3d`, **FASE A** (2026-09-11)

> W2/L3 do [briefing de partir a shell](../../archive/integracao-jornadas/BRIEFINGS_W2_PARTIR_A_SHELL_2026-09-11.md) §5-A.
> ⚠️ **Isto fecha a FASE A, não a linha.** A Fase B (o corte para `crates/ph2d-app-sculpt3d`)
> espera a `line/app-host` integrar e o `HOWTO_partir_uma_familia_da_shell.md` existir.

## §1 — Identidade

| | |
|---|---|
| branch | `line/app-sculpt3d` |
| HEAD | `32c7c629b` |
| merge-base com `main` | `8fa4f115b` |
| commits | **6** |
| diff | `217 ficheiros · +1 321 / −999` |

```
32c7c629b  o gate batched apanhou QUATRO que o filtro `~sculpt` não via
b7c0268dd  o tecto de LOC do mod.rs estourou por ACUMULAÇÃO — curado por CORTE
fda87e63d  A2: as TRÊS portas que só precisavam da cena viram funções LIVRES
3f7909136  A2+A4: a crate NASCE, e os 9 campos soltos de `App` viram 2
c1a2615f3  A3: a família inteira vira UMA pasta — src/sculpt3d_*.rs → src/sculpt3d/
b6e1566c3  A1: a cena =17 sai — e o censo de UM canal teria apagado 17 cenas VIVAS
```

## §2 — Foundational / partilhado tocado, e porquê

| ficheiro | o quê | porquê |
|---|---|---|
| `shells/desktop/src/main.rs` | `mod sculpt3d;` fica, `mod sculpt3d_keys_view;` **sai**; o construtor de `App` perde 9 inicializadores e ganha 2 | a família passou a UMA pasta e a 2 campos |
| `shells/desktop/src/app_state.rs` | −9 campos, +2 | A2 (ADR-0075) |
| `shells/desktop/src/input_dispatch.rs` | 3 sítios de chamada | as portas viraram funções livres |
| `shells/desktop/src/render_loop/mod.rs` | 1 sítio de chamada | idem |
| `shells/desktop/Cargo.toml` | +1 dep **não-opcional** (`ph2d-app-sculpt3d`) | §4 abaixo |
| `shells/desktop/src/field3d_input.rs` | **2 linhas de doc** | um intra-doc link `[crate::sculpt3d_input]` que a mudança de módulo partiria ⚠️ é ficheiro da **família da `line/app-host`** |
| `crates/` (16 ficheiros) | **só prosa de doc-comment**: caminhos `sculpt3d_X.rs` → `sculpt3d/X.rs` | 1 deles é um `include_str!` real (`ph2d-quadfill/src/untangle_tests.rs`) |
| `CLAUDE.md` | **4 caminhos, ZERO narrativa** | links que quebrariam; prova por `--word-diff` no §7 |
| `docs/**` (51 ficheiros) | 133 linhas de caminho | precedente da W1 (588 citações) |

⛔ **Nenhum contador partilhado se move** — `collision-surface.sh` colado no §7.

## §3 — Símbolos novos que podem COLIDIR

| símbolo | valor | onde |
|---|---|---|
| crate `ph2d-app-sculpt3d` | — | `crates/` (membro por **glob**, zero edição central) |
| `ph2d_app_sculpt3d::Sculpt3dRequests` | struct | a crate |
| `ph2d_app_sculpt3d::rulers::*` | 9 `const` (eram `pub(super)` na shell) | a crate |
| `crate::sculpt3d::Sculpt3dShellState` | struct | `shells/desktop/src/sculpt3d/shell_state.rs` |
| `App::sculpt3d_req` · `App::sculpt3d` | 2 campos | `app_state.rs` |
| `crate::sculpt3d::{flush_grab, pointer_up, pointer_move}` | 3 fn livres | `sculpt3d/input.rs` |

⚠️ **`App::sculpt3d` é um NOME novo num `App` que já tinha `AppGfx::sculpt3d`** — são campos de
structs diferentes e não colidem, mas quem ler `self.sculpt3d` tem de saber qual. O gateado é o da
`App` (estado), o do `AppGfx` é a **cena** (`Option<Sculpt3dScene>`, intocada por esta linha).

## §4 — Contratos congelados encostados

**Nenhum.** `NodeOp`/`OpResolver`/`NodeManifest` e `Tool`/`RasterEditTool`/`CanvasPaintTool`/
`PanelEvent` intocados (o `collision-surface.sh` confirma). Zero ADR novo.

⛔ **A navegação orbital continua na shell, de propósito** (CLAUDE.md §5: *«nunca numa `Tool`»*) —
é isso que mantém `Tool=12` fora do caminho, e nada nesta linha lhe toca.

## §5 — O que a FASE A entregou, em números

| | antes (`main`) | depois |
|---|---:|---:|
| ficheiros da família | 111 soltos em `src/` | **111 em `src/sculpt3d/`** |
| `mod` da família no `main.rs` | 2 | **1** |
| campos de `App` da família | **9** | **2** |
| blocos `impl App` | 6 | 6 |
| métodos em `impl App` | **12** | **9** (−246 LOC) |
| `self.X` distintos dentro de `impl App` | **11** | **10** |
| níveis de smoke | 38 (1..39, sem lacuna) | **37** (`=17` apagada) |
| LOC de `shells/desktop/src` | 493 252 | 493 211 |
| `crates/ph2d-app-sculpt3d` | — | 238 LOC, **zero deps** |

⚠️ **A shell NÃO encolheu, e é por desenho.** A família inteira continua lá dentro; a Fase A
prepara o corte (uma pasta, dois campos, a crate a existir). Quem espera o ganho de build nesta
fase está a ler a tabela errada — ele é da Fase B.

## §6 — ⛔ O que precisa da SHELL e NÃO é da família (a lista que a L0 tem de cobrir)

Medido **método a método** dentro dos blocos `impl App`, não por `grep` de ficheiro:

| o que a família pede | quem o pede | o que é |
|---|---|---|
| `gfx` | 8 dos 9 métodos | o `device` e o **tamanho da superfície** — é de onde a cena se **cria** |
| `last_pointer` | 4 | posição do ponteiro (estado de janela) |
| `modifiers` | 2 | modificadores (estado de janela) |
| `donated_form` | 1 | a saída da doação, **consumida pelo renderer 2D** e pelo `painter_bridge` |
| `vec_pen` · `text_entry_focused` | 1 | **arbitragem com OUTRAS famílias** — quem tem o teclado |
| `sculpt3d_clay_on_screen` · `sculpt3d_keys_live` | 1 | arbitragem: quem tem o canvas |
| `sculpt3d_scene_mut` | 5 | o acessor da cena — **hoje `pub(crate)`**, e é isto (e não um método por campo) o que a shell empresta |

⇒ **o substrato precisa de: device + tamanho da superfície · posição do ponteiro · modificadores ·
cursor · um canal para a forma doada · e três predicados de arbitragem.** Nenhum é um campo de
`App` por-família: são serviços de janela e perguntas sobre quem detém o foco.

## §7 — ⚠️ O que uma leitura rápida do diff entende ao contrário

1. **«O `=17` foi apagado porque não era usado» — não: porque nenhum doc a CITA.** ⛔⛔ E o censo
   de **um canal só** (`PH2D_SCULPT3D_SMOKE=<n>` nos docs) teria apagado **17 cenas VIVAS**: os
   docs desta família citam-nas como `` `=36` `` em prosa. Entre as 17 está a **orelha** (`=36`),
   sujeito de `the_ear_does_not_ship_an_edge_across_the_piece` — *um gate cujo sujeito muda não
   afirma nada*. **As outras cinco linhas correm o mesmo censo hoje: avisem-nas.**
2. **«A crate é opcional como as outras três do módulo» — NÃO, e a excepção é load-bearing.** O
   `Sculpt3dRequests::doc` é um **passa-adiante**: um binário sem a feature `sculpt3d` carrega os
   bytes de uma escultura gravada do load ao save **sem os ler**. `optional = true` apagaria o
   campo nesse binário e o save seguinte descartaria a escultura. É também por isso que a crate
   **não pode ganhar dependências** — quem nunca esculpe paga-as.
3. **«São dois campos porque eu quis» — não: a `cfg` impôs.** Os 4 campos gateados carregam tipos
   do módulo (`LoadedPiece`, `SculptRowsSeen`) e uma struct gateada não pode guardar o `doc`,
   cujo valor inteiro é sobreviver a quem não tem a feature. Fundem-se na Fase B.
4. **«A promessa de inércia enfraqueceu» — ficou MAIS forte.** As três portas livres **recebem**
   `&mut Sculpt3dScene` ⇒ são inconstruíveis sem cena; nenhum `if` as pode esquecer. O gate
   `every_3d_port_is_inert_without_a_scene` passou a ter três populações (runtime · tipo · **o
   sítio de chamada procura a cena**), com prova de mutação na terceira.
5. **«O `mod.rs` perdeu 31 `#[path]` por limpeza» — foi a cura de um TECTO.** Ele estava em
   **exactamente 600** no `main` e as minhas linhas levaram-no a 644. Os `#[path]` ficaram
   *redundantes* quando a família virou pasta; **três ficam** porque o nome do módulo ≠ o do
   ficheiro (`announce_mod`, `drag_kinds`, `sculpt3d_preview`).
6. **«O `FILTER_DRAG_PER_PX` mudou-se por arrumação» — a razão dele DISSOLVEU.** A nota dizia, por
   escrito, *«mora aqui porque é a mesma espécie dos VIZINHOS»* — e os vizinhos (as 9 réguas) foram
   para a crate. Ele não pôde ir (é um re-export da `ph2d-sculpt3d`) ⇒ foi para o único consumidor.
7. **Os `MOVED` do `nextest-list-diff` são ZERO, e isso é o esperado nesta fase** — nenhum teste
   mudou de pacote porque nada saiu para a crate senão as réguas (que não têm testes).

## §8 — ⚠️ Premissas MINHAS que a medição derrubou

1. **«180 membros de `self.`»** (o censo do briefing) — o número mistura `self.` de **todos** os
   tipos da família. Dentro dos blocos `impl App` são **11**. *O acoplamento real era 16× menor
   do que a tabela de abertura dizia.*
2. **«`rulers.rs` é lei pura, zero imports»** — eu greppei `^use ` e a última linha era
   `pub(super) use ph2d_sculpt3d::FILTER_DRAG_PER_PX`. *Um grep ancorado no início da linha não vê
   um `use` com visibilidade à frente* — e a conclusão errada teria dado à crate uma dependência.
3. **«A feature `sculpt3d` é removível»** — **não é, e é PRÉ-EXISTENTE** (provado contra `main`,
   em ficheiros que não toquei): `field3d_export.rs` e `field3d_import.rs` alcançam
   `crate::sculpt3d::` sem `cfg` — exactamente **dois** símbolos, `import::read_pieces` e
   `lost_by` — mais `use ph2d_mesh`. ⇒ a cerca do §7.2 está certa e **hoje não é exercitável**.
   ⚠️ **Isto é da `line/app-host`**: a família piloto dela depende da minha.
4. **«O script de reapontamento é seguro porque é conduzido pelos ficheiros que movi»** — ele
   danificou **3** ficheiros do `ph2d-i18n`, onde `sculpt3d.rs` é o irmão da **própria crate**.
   Revertidos. *Uma renomeação conduzida por NOME de ficheiro atinge todo homónimo do repo*; a
   varredura de controlo que os achou pergunta *«a crate tem um irmão com o nome antigo?»*.
5. **«266/266 em `-E test(~sculpt)` é o fecho»** — o `nextest-impacted` correu **14 974** e achou
   **quatro** vermelhos, nenhum com «sculpt» no nome. *Um fecho filtrado pelo nome da família é
   cego aos gates que a medem de fora.*

## §9 — A PROVA (§3 do briefing)

**(a) Nenhum teste se perde** — `nextest-list-diff.py antes.txt final.txt`:
```
antes: 22635 testes (22011 chaves) | depois: 22638 (22014)
MOVED: 0   ·   ONLY-A (perdidos): 0   ·   ONLY-B (novos): 3
   + nothing_is_requested_before_anyone_asks          (ph2d-app-sculpt3d)
   + a_request_is_honoured_once_and_then_it_is_gone   (ph2d-app-sculpt3d)
   + the_document_bytes_survive_a_build_that_never_reads_them (ph2d-app-sculpt3d)
```
⭐ O terceiro corre **numa crate que não conhece o módulo 3D** — é essa a prova: ele não mede uma
função nossa, mede que o *lugar* onde o documento espera não depende da feature.

**(b) Roteadores e níveis** — conjunto do `main` contra o de agora:
`só no main: ['17']` · `só agora: []`. A `=17` é a poda deliberada da A1, e nada mais mudou.

**(c) A shell, a frio, `load 4,19`** (`CARGO_TARGET_DIR` novo, `-j 32`):

| unidade | duração |
|---|---:|
| `ph2d-host-desktop` (bin, check-test) | **16,35 s** |
| a 2.ª mais cara de **todo** o build (`ash`) | 9,41 s |
| `ph2d-app-sculpt3d` | **0,09 s** |

⚠️ **Isto é a BASELINE da Fase B, não um delta.** O `16,35 s` bate a auditoria de 10/09
(`16,8 s`, sem contenção) — e tem de bater: a Fase A move **−41 LOC de 493 252** (−0,008 %), que
está abaixo da variância da máquina. *Um par antes/depois aqui mediria ruído, não a mudança.* O
número que importa é o outro: **a shell continua a unidade mais cara do build inteiro, com 1,7× a
seguinte.**

**(d) Gate batched de fecho** — sobre o diff acumulado, 1×:
`nextest-impacted.sh` **14 974/14 974** · `clippy --all-targets -D warnings` verde ·
tectos de LOC verde · `fmt --check` verde · `typos` verde · `doc-index.sh --check` 19 índices em dia.

**(e) Smoke — o binário fica COMPILADO** (§1.5.9 item 9), `cd` e perfil byte a byte iguais aos do §11:
```
$ cargo build -p ph2d-host-desktop --profile smoke     # 1.ª: Finished `smoke` in 1m 41s
$ cargo build -p ph2d-host-desktop --profile smoke     # 2.ª — A PROVA:
    Finished `smoke` profile [optimized] target(s) in 0.34s      ← zero linhas "Compiling"
```
⭐ **E uma sanidade de ARRANQUE** (`PH2D_SCULPT3D_SMOKE=1`, 25 s), que nenhum teste alcança — o
`AppGfx` segura uma surface de janela real: o app abre, arma a cena (`malha com 98306 vértices /
98304 faces`), imprime o roteiro inteiro do módulo e fica vivo. *Um pânico de runtime vindo de
uma extracção não seria apanhado por gate nenhum.*

### `collision-surface.sh` (item 3 do §1.5.9)

```
▸ SCHEMAS      PROJECT_SCHEMA 128 (base: 128) · tripla (128,13,22) = base
               FLIP_SCHEMA 13 (base: 13) · DOC_VERSION 18 (base: 18)
▸ REGISTRO     ph2d-render 86 (base: 86) · ph2d-script 86 (base: 86)
▸ CONTRATO     node.rs intocado · tool.rs intocado
▸ ADR          esta linha não cria ADR ⇒ fora de toda disputa de número
▸ Cargo.lock   nenhum '+name' externo novo
▸ MARCADORES   nenhum
▸ TETOS        nenhum ficheiro da linha passa do teto
```
⚠️ **Prazo de validade (§1.5.9):** tirada em 2026-09-11 contra `main = 8fa4f115b`. **Re-corra-a na
worktree imediatamente antes de fundir** — cinco linhas irmãs fecham no mesmo dia.

### Prova de que o `CLAUDE.md` leva zero narrativa
```
git diff --word-diff main..HEAD -- CLAUDE.md
[-sculpt3d_keys.rs-] {+sculpt3d/keys.rs+}
[-sculpt3d.rs-] {+sculpt3d/mod.rs+}
[-shells/desktop/src/sculpt3d_entities.rs-] {+shells/desktop/src/sculpt3d/entities.rs+}
[-shells/desktop/src/sculpt3d_retopo_rulers.rs-] {+shells/desktop/src/sculpt3d/retopo_rulers.rs+}
```

## §10 — O que só o `ship.sh` apanha

`machete` (a dep nova é interna, mas o `ph2d-app-sculpt3d` tem **zero** deps ⇒ nada a podar) ·
`deny`/`audit` (nenhum pacote externo novo) · a matriz 3-OS. Nada de `fmt`/`typos`/`clippy`
latente: os três correram verdes aqui.

## §11 — O que smoke-testar (e o que NÃO foi smokado)

⛔ **A extracção não muda produto** — a prova é o `ONLY-A = 0` e os 14 974 verdes. O smoke é a
confirmação de que a cena ainda abre e o gesto ainda responde.

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-sculpt3d && env PH2D_SCULPT3D_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```
Cenas representativas das três metades que esta linha tocou: **`=1`** (a esfera e o gesto —
`pointer_down`/`move`/`up`, as duas formas de guarda) · **`=34`** (o filtro de malha, que lê o
`FILTER_DRAG_PER_PX` que mudou de ficheiro) · **`=39`** (`Connected Only`, a mais recente).
⚠️ **Rode uma vez SEM a env var** — é a metade que prova a inércia.

⛔ **NÃO smokado, e nomeado:** a `=17` deixou de existir (era o AO assado num toro; a malha era
`ph2d_mesh::shapes::torus(96, 48, 1.0, 0.42)` e a razão da forma está escrita no lugar dela).

## §12 — Para o INTEGRADOR

1. **Ordem:** esta linha não toca contrato, não move schema e não cria ADR. O que ela toca em
   comum com as irmãs é **textual e por região distinta**: a lista de `mod` do `main.rs`, campos
   de `app_state.rs`, deps do `Cargo.toml` da shell. Mergiraf funde; confira o olho.
2. ⚠️ **A `line/app-host` tem de saber do §8.3** — o `field3d` dela alcança `crate::sculpt3d::`
   sem `cfg`, em dois símbolos.
3. ⚠️ **As outras quatro famílias estão a correr o censo de cenas de UM canal** (§7.1). Se alguma
   fechar com «apaguei N cenas», vale a pena pedir-lhe o segundo canal antes de fundir.
4. **A linha do `CLAUDE.md` §5** (item 8 do §1.5.9) — proponho, para a bullet *3D / Sculpt*:
   > ⚠️ **A família saiu de `src/sculpt3d_*.rs` para `src/sculpt3d/`** (W2/L3 Fase A, 11/09) e a
   > `App` guarda-a em **dois** campos (`sculpt3d_req` não-gateado · `sculpt3d` gateado); a crate
   > [`ph2d-app-sculpt3d`](crates/ph2d-app-sculpt3d/) nasceu com as réguas do gesto e é dep
   > **não-opcional** de propósito — [handoff](docs/3D/handoffs/HANDOFF_INTEGRACAO_line_app_sculpt3d_FASE_A_2026-09-11.md).
5. ⛔ **A linha NÃO está fechada** — a Fase B espera a `line/app-host` integrar.

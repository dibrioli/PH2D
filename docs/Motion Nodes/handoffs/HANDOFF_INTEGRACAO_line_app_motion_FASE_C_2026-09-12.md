# `line/app-motion` — FASE C: **a família saiu, e a catraca dos roteadores MORREU** (2026-09-12)

> **Veredito:** a maior família do repo está fora da shell. `shells/desktop` desce de
> **373 937 para 262 540** linhas (**−111 397, −29,8 %**) e de **1 498 para 1 040** ficheiros.
> A `ph2d-app-motion` passa de **3** ficheiros para **454** (108 995 LOC).
>
> ⭐⭐⭐ **E `"motion"` era a ÚLTIMA da catraca `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL`.** O
> doc-comment dela prometia: *«a lista chega a vazia e então este bloco e a metade `if` do gate
> abaixo desaparecem com ela»*. Desapareceram. **As seis famílias declaram roteador.**
>
> ⛔ Zero sextos métodos no `AppHost` · zero contadores partilhados mexidos · `TETO_LOC` **não
> tocado** (regra 1 — ele está vermelho de propósito, ver §8).

---

## §1 — O que saiu, em quatro crates

| crate | ficheiros | LOC | o que é |
|---|---:|---:|---|
| **`ph2d-app-motion`** | 454 | 108 995 | a família (era 3 ficheiros) |
| **`ph2d-vec-art-live`** (nova) | 7 | 2 679 | a arte que uma forma revela: o ladrilho do padrão + a arte do pincel |
| **`ph2d-thumbnail`** (nova) | 1 | 115 | a redução a miniatura — **zero dependências** |
| **`ph2d-pan-diag`** (nova) | 1 | 187 | a sonda do drift de pan |

### O que FICOU na shell, e porquê

*A lei é «o prólogo fica, os corpos saem».*

| ficheiro | porquê |
|---|---|
| `motion_host.rs` *(novo)* | constrói o `MotionSceneCtx`. Composição — e o que o faz compilar é o **empréstimo disjunto de campos** |
| `field_gizmo_host.rs` *(novo)* | o `impl App` do gizmo: toca `self.field_gizmo_drag`, `self.modifiers`, `self.gfx`. **Um arrasto é estado de shell** (nasce num pen-down, sobrevive entre quadros) |
| `warp_gizmo_drag.rs` · `warp_smoke.rs` | os únicos dois `warp_*` que tocam a `App` |
| `ui_motion_smoke.rs` | ⚠️ **não é do Motion apesar do nome** — ver §7.4 |
| `render_loop/mod.rs` | o LAÇO. ⛔ Não foi abstraído |

---

## §2 — O FECHO, re-medido três vezes (o passo 1 do bloco)

`python3 scripts/fecho-da-familia.py motion --extra 'warp_'`

| momento | âncoras | quem as possui |
|---|---:|---|
| 11/09 (Fase B) | **26** | 13 módulos, 10 donos |
| 12/09 (Fase B, 2.ª volta) | **7** | 4 minhas, 5 ficheiros da `flip`, 2 da `vec` |
| **12/09 (Fase C, hoje)** | **5** | ⭐ **as cinco MINHAS** — a `flip` entregou |
| depois do passo 0 | **2** | `field_gizmo`, `picker_smoke` — as duas código da própria motion |

⭐ **Contrafactual medido antes de mover um ficheiro:** curar as 5 ⇒ `419 / 419` ficheiros,
**zero** ficam. Foi esse número que autorizou o corte.

⚠️ **E uma âncora dissolveu-se sozinha:** o `pan_diag.rs` tinha uma aresta para
`flip_pass::camera::camera_scene` e hoje tem **`0` para fora** — a `line/app-flip` levou o alvo.
*Uma âncora pode desaparecer sem que ninguém lhe toque; o fecho tem de ser re-medido, não herdado.*

---

## §3 — ⛔⛔ Os 38 roteadores, e porque eram 38 e não 9

O censo por **prefixo de ficheiro** via nove. Os outros **29** vivem em ficheiros **sem** o
prefixo `motion_`, soltos em `src/`: os 18 `value_*_smoke`, mais `splice`, `gradient`, `lens`,
`emitter`, `echo_family`, `transform_family`, `units`, `osc_ruler`, `driven_row`, `glow_dirt`,
`adapter`, `attribute_demo`.

⛔ **A catraca é all-or-nothing por família ⇒ os 29 invisíveis bloqueavam os 9 visíveis.** É
exactamente o que a `vec` pagou em 12/09 (*«cinco, não os quatro que o briefing listava»*), e o
doc dela já o dizia — eu li-o **depois** de o pagar.

### ⚠️ Cada `max_level` foi CONTADO, e três não eram o que pareciam

| roteador | lê-se | é | porquê |
|---|---:|---:|---|
| `PH2D_AUTOFIX_SMOKE` | 6 | **8** | o `_ =>` delega aos IRMÃOS, e os modos 7 e 8 vivem lá |
| `PH2D_MOTION_NODE_PATH_SMOKE` | 6 | **4** | os `3 =>`/`6 =>` são números de **FRAME**; o `mode()` mapeia `{0,2,3,4}` e o resto → 1 |
| `PH2D_SHAPE_SMOKE` | 2 | **3** | o irmão `_knobs` responde ao 3 |

*Um censo que conta braços de `match` sem saber sobre O QUÊ se casa erra nos dois sentidos.*
O `PH2D_GPU_COOK_DEMO` é **derivado** de `MAX_DEMO_LEVEL` — ⚠️ que **deixou de ser
`#[cfg(test)]`**, porque ganhou um consumidor de produto. Escrever `114` no `FAMILY` seria pôr o
número em dois sítios, que é como ele envelhece.

### ⭐ A forma `PH2D_*_SMOKE` ganhou uma excepção NOMEADA, não um afrouxamento

O roteador **principal** da motion — as 114 cenas que o dono smoka — chama-se
**`PH2D_GPU_COOK_DEMO`** e acaba em `_DEMO`. As três saídas:

1. *não o declarar* ⇒ o registo passaria a **mentir** sobre a maior família do repo;
2. *renomear a env* ⇒ parte **todo** passo de smoke já escrito. O nome **é** a superfície;
3. **declarar a excepção, com censo** ⇒ `ROTEADORES_FORA_DA_FORMA`, com as duas metades de
   obsolescência (*a env ainda existe? já segue a forma?*).

⭐ *O censo de obsolescência não morre com a catraca: ele muda de sujeito — media FAMÍLIAS,
agora mede ENVS.* ⛔ E ela **não** é `PH2D_GPU_COOK` (sem `_DEMO`), que é diagnóstico: duas
envs, um prefixo, papéis opostos.

---

## §4 — ⛔⛔ O defeito nº1 da minha própria régua mordeu pela TERCEIRA vez

O `scripts/fecho-da-familia.py` tem **dois** strippers, e o cabeçalho dele diz porquê: *quem mede
ARESTAS lê sem comentários mas COM as strings; quem conta CITAÇÕES lê sem as duas.*

⚠️ **O censo de nomes de ENV devolveu ZERO na 1.ª corrida** — porque eu usei o stripper que
branqueia strings, e um `env::var("PH2D_X")` tem o nome **dentro de uma string**. É o mesmo
defeito que apagou o grafo de `#[path]` (21× de erro), num sujeito novo.

⇒ **Escrever a lição num doc-comment não impede de a repetir; só um controlo positivo o faz.**
Os seis do `--autoteste` cobrem os dois strippers e apanharam-nos quando lhes toquei — e **não**
cobriam o caso «censo de nomes», que é o que falhou.

---

## §5 — As armadilhas do HOWTO que este corte pagou

| § | o que | o número |
|---|---|---|
| **1.3** | dependências **invisíveis** dentro da shell | **70** (56 + 9 + 5). O piloto teve **4**. Resolveram-se em CAMADAS: cada ronda de `check` revelava a seguinte |
| **2.4** | ⛔⛔ a FEATURE, e ela quase passou **muda** | `109` sítios sob `panel-motion-graph`/`panel-motion-params`. Sem as declarar, o `cfg` é falso **por construção** e o `motion_bridge` perdia o `mod surfaces` — *o que a ferramenta ABRE e FECHA* — compilando verde com o painel lateral morto. ⭐ Falhou alto **por acidente**: o `use surfaces::painel_lateral` ao lado do `mod` **não** está sob `cfg` |
| **2.5** | `cfg(test)` invisível do outro lado | **1** item atravessa (`driven_value_for_probe`) ⇒ feature `test-support`, do tamanho do que atravessa |
| **2.6** | `include_str!` (alto) + `read_to_string` (**mudo**) | 5 + 3. ⚠️ E eu **escrevi o segundo errado**: o cwd de um teste é a raiz da CRATE, não a do repo ⇒ `../../crates/`, não `../crates/`. *Duas formas de citar um ficheiro, duas âncoras, e só uma falha a compilar* |
| **2.3** | o nome NU que o `use` liga | **26** (`super::motion_glow_layer`, `motion_bridge::`) |
| **2.13** | visibilidade | 944 `pub(crate)` + 9 `pub(super)` abertos. ⚠️ Conferido **antes**: zero gates ancoram na visibilidade destes símbolos |
| **2.7** | o filtro por prefixo | ⭐ **não aconteceu** — ver §6 |

⚠️ **E `pub(in crate::render_loop)` (7 sítios) teve de virar `pub(crate)`** — o que fecha um
ciclo: a minha régua estava **certa** em não o contar como dependência (é um âmbito, não uma
aresta), e ele precisava de mudar na mesma. *Não ser uma aresta não é o mesmo que não ser afectado.*

---

## §6 — ⛔ Desvio deliberado do HOWTO §1.3: o prefixo NÃO saiu dos nomes

O HOWTO manda tirar o `<fam>_` dos ficheiros. **Não tirei**, e a razão está escrita no
`motion.rs` desde 11/09: *«os `#[path]` internos NÃO precisaram de UMA edição — são nomes de
ficheiro IRMÃO, e mover o conjunto INTEIRO preserva-os por construção»*. São **371** hoje.

⭐ **E o desvio pagou-se no mesmo dia:** o filtro do gate `every_demo_scene_ends_in_an_output_node`
(`name.starts_with("motion_state_conferencia_demos")`) **sobreviveu à mudança sem uma edição**.
É a §2.7 a **não** acontecer — o modo de falha mudo que o HOWTO diz ser o pior.

---

## §7 — O que uma leitura rápida deste diff entende ao contrário

1. ⚠️ **«os 632 do bloco»** — o cluster são **2 638** linhas. Os dois ficheiros de produto levam
   cinco satélites que não tinham escolha: quatro são filhos `#[path]` deles e o quinto é a sonda
   de custo, `#[cfg(test)]` no `main.rs`, que mede o `resolve` **pela porta do produto**. Os cinco
   têm **zero** arestas para fora.
2. ⚠️ **«`ph2d-panel-vector` numa folha é erro»** — é um **cheiro pré-existente e NOMEADO** no
   `Cargo.toml`: o `art_state` devolve `PatternArt`, que é a escolha AUTORADA no painel. Mudar o
   tipo de casa é wave da `line/app-vec`, não desta.
3. ⚠️ **«a `pan_diag` podia ir para a `ph2d-app-motion`»** — ⛔ **não podia**: ela lê
   `ph2d_app_flip`. Pô-la lá faria uma família depender de **outra** (HOWTO §1.2). *O critério não
   é «quem a usa», é «o que ELA usa».*
4. ⚠️ **«o `ui_motion_smoke` é do Motion»** — **não é**, e eu movi-o antes de o medir. Ele toca
   `self.ui_motion_smoke_done` e abre o painel de **física**; o `CLAUDE.md` lista-o sob o
   **Vector**. Foi devolvido. *Um censo por nome acha o que se CHAMA assim, não o que É assim.*
5. ⚠️ **«o `warp_*` é do `render_loop`»** — é do **Motion**: o cabeçalho do `warp_gizmo.rs` diz
   *«as alças que o `motion.four_point_warp` e o `motion.bezier_warp` passam a ter na tela»*. O
   prefixo `motion_` não o via — a §2.7 **ao contrário** (ali varre de menos na crate nova; aqui
   varria de menos na shell).
6. ⚠️ **«944 `pub` é afrouxar»** — dentro de uma crate `pub(crate)` e `pub` são a mesma coisa; o
   que muda é o que a **shell** alcança, que é o ponto. E os `pub(super)` foram abertos **um a um,
   dirigidos pelo compilador**, não em varredura.
7. ⭐ **«um gate foi afrouxado para passar»** — o contrário: o censo das membranas passou a
   **tirar comentários**, e ficou a medir melhor do que media antes de eu lhe tocar (§8.2).

---

## §8 — Premissas minhas que a medição derrubou

1. ⛔ **«o `playhead` é só LEITURA»** (escrito por mim no próprio ficheiro que o declarava): a
   cena `=7` do `motion_object_smoke` chama `playhead.play()` **de propósito** — *«um offset de
   tempo só é visível numa animação que CORRE»*. **Uma cena de smoke não observa, ela ENCENA.**
2. ⛔ **«são dez campos»**: são **onze**. O `hero_screen` ficou de fora e a régua tinha-o listado
   (`.hero_screen 1 f`) — eu li a linha como *«algo que o AppGfx segura»* em vez de *«algo que
   uma CENA toca»*. ⚠️ *Uma tabela ordenada por frequência põe o caso raro no fim, e o fim é onde
   se deixa de ler.*
3. ⛔ **«o censo das membranas só precisa do caminho novo»**: ele contou **7** onde há 6, porque o
   sétimo **cita** a porta num doc-comment. §2.12, **latente** — dentro do `render_loop` nenhum
   ficheiro a citava em prosa. *Foi a MUDANÇA que revelou o defeito.*
4. ⛔ **«`ToolRegistry` vem de `ph2d_editor_core`»** — vem de `ph2d_editor`. A minha régua dá o
   **nome** do tipo, não o **caminho** dele.

---

## §9 — A prova

| | |
|---|---|
| baseline `cargo nextest list --workspace --cargo-profile ci-test` | **22 668** (capturada antes de mover) |
| `cargo check -p ph2d-app-motion --all-targets` | ✅ **0** erros, **0** avisos |
| `cargo check -p ph2d-host-desktop --all-targets` | ✅ **0** erros, **0** avisos |
| ⚠️ **`cargo test -p ph2d-host-desktop --test it`** (regra 2, À PARTE) | ✅ **812 passed, 0 failed**, 6 ignored — e ele apanhou **12** gates com o `check` verde |
| `cargo test -p ph2d-app-registry-init` | ✅ **5 passed, 0 failed** |
| `python3 scripts/fecho-da-familia.py --autoteste` | ✅ 6/6 |
| contadores partilhados | **inalterados** (`PROJECT_SCHEMA`, `FLIP_SCHEMA`, registos do `ph2d-ecs`) |
| 6.º método no `AppHost` | **nenhum pedido** |

### ⛔ O `TETO_LOC` está VERMELHO, e é o marcador de progresso (regra 1)

```
a catraca está OBSOLETA: a shell tem 262 447 linhas e o tecto ainda diz 377 937
— uma folga de 115 490 linhas.
```

⛔ **Não lhe toquei.** É um número que soma entre linhas e é do **integrador**, sobre a árvore
combinada, **depois** do `cargo fmt --all`. O medido nesta worktree ao fim do `fmt` é
**262 540**; o gate sugere `266 447` (medido + a folga de composição de 4 000) — ⚠️ e esse valor
é sobre **esta** árvore, não sobre a combinada.

---

## §10 — Aberto, e o que fica para quem vier

1. ⏳ **O smoke é do dono.** 38 roteadores mudaram de casa e nenhum mudou de nome nem de número —
   mas isso é uma afirmação sobre o código, não sobre o ecrã.
2. ⚠️ **A `ph2d-app-motion` tem 454 ficheiros e 109 k LOC.** Ela é agora a maior unidade de
   compilação depois da shell. *O tecto do relógio mudou de sítio; ninguém o mediu ainda.*
3. ⚠️ **`ph2d-panel-vector` é dependência da `ph2d-vec-art-live`** (§7.2) — dívida nomeada da
   `line/app-vec`, não desta.
4. ⏳ **`ui_motion_smoke` ficou na shell** e é do **Vector** (§7.4): quando aquela linha reabrir,
   é dela.
5. ⚠️ **O `scripts/fecho-da-familia.py` ainda vive só nesta worktree** — ele entra no `main` com
   esta integração, e o HOWTO §1.1-bis já o chama pelo nome no passo 1.

---

## §11 — O portão batched, e o que ele devolveu

`bash scripts/nextest-impacted.sh` — **16 245 testes, 16 242 passaram, 3 falharam**:

| ✗ | veredito |
|---|---|
| `architecture_the_shell_only_shrinks` | ⛔ **ESPERADO** (regra 1) — o `TETO_LOC` é do integrador. Ver §9 |
| `instructional_docs_only_cite_paths_that_exist` | ✅ **CURADO** — o `CLAUDE.md` citava o roteador em `shells/desktop/src/motion/…` (2 sítios) |
| `no_expression_allocates_no_link_frame` (`ph2d-timeline`) | ⚠️ **FLAKE DE FAN-OUT — proponho promoção** |

### ⚠️ Proposta: `no_expression_allocates_no_link_frame` entra na lista de flakes (CLAUDE.md §5.0)

As **quatro** assinaturas, medidas:

1. é um **contador de alocações** — a espécie própria que a lista já nomeia (*«sob fan-out o
   alocador global reutiliza arenas de outra maneira»*);
2. **zero linhas** do diff desta linha na `ph2d-timeline` (`git diff --stat main..HEAD` naquela
   crate: vazio);
3. **3 de 3 verde sozinho**, e ⭐ **a `load 38–48`** — ou seja, passa sob carga alta e reprova
   sob o fan-out de 16 245: *o discriminador é o FAN-OUT, não o relógio*, que é exactamente o que
   a nota do `flip_fit_cache` já regista;
4. reprovou no meio de uma corrida de 16 mil.

⚠️ Ela é **irmã** da `apply_from_doc_is_zero_alloc_steady_state`, que já está na lista e vive na
**mesma crate** — e que **passou** nesta mesma corrida. *Duas contagens de alocação no mesmo
ficheiro-vizinho, uma na lista e outra não, é como a lista envelhece.*

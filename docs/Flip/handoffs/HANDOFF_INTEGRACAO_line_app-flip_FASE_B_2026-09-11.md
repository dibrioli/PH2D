# Handoff de INTEGRAÇÃO — `line/app-flip`, W2 **FASE B** (2026-09-11)

> **A linha fecha aqui, e fecha com UM BLOQUEIO NOMEADO que é decisão do integrador.**
> O *fim da linha* gateado **foi alcançado** (o `const FAMILY` declara os 18 roteadores que a
> família lê, e a `flip` nunca esteve na catraca `FAMILIAS_COM_O_ROTEADOR_AINDA_NA_SHELL`).
> ⛔ **Mas 84 % da família continua na shell**, presa por **duas funções de outra família** — §3.
> Fase A: [`HANDOFF_INTERMEDIO_line_app-flip_FASE_A_2026-09-11.md`](HANDOFF_INTERMEDIO_line_app-flip_FASE_A_2026-09-11.md) ·
> molde: [`HOWTO_partir_uma_familia_da_shell.md`](../../IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md).

## 1 — Identidade

| | |
|---|---|
| branch | `line/app-flip` |
| merge-base | `2dedac80a` (main de 11/09, pós-integração da Fase A) |
| commits | 5 |
| **contadores partilhados** | **NENHUM se move** — `PROJECT_SCHEMA` 128, `FLIP_SCHEMA` 13, `DOC_VERSION` 18, registos 86/86, todos **iguais à base** |
| contratos congelados (§6) | **intocados** (`node.rs`, `tool.rs`) |
| ADR novo | nenhum ⇒ fora de toda disputa de número |
| pacote externo novo no `Cargo.lock` | **nenhum** (as 3 deps novas da crate são internas + `bytemuck`, que a shell já tinha) |
| `TETO_LOC` do `the_shell_only_shrinks` | ⛔ **NÃO tocado** (§1 regra 1) — o número medido está no §5 |

## 2 — ⭐⭐⭐ O FIM DA LINHA, e o que o destravou (não foi um refactor grande)

O `const FAMILY` passou de **15 para 18** roteadores. ⭐ **O que os prendia era UMA função.**

Os três (`PH2D_FLIP_HARDNESS_SMOKE` — o mestre —, `PH2D_FLIP_PRESSURE_SMOKE`,
`PH2D_FLIP_RESAMPLE_SMOKE`) precisavam de **uma só** coisa do `draw`: `stroke_from_samples`. E o
`draw.rs` inteiro estava preso porque **uma** das suas funções — `bake_stroke` — pergunta ao
`autokey` qual é o desenho-alvo e escreve na `strip`, e esses dois caem na cascata do §3.

⇒ `bake_stroke` saiu para [`shells/desktop/src/flip/bake.rs`](../../../shells/desktop/src/flip/bake.rs)
(**46 linhas**) e a **lei inteira do traço** (378 linhas: `FlipDraw`, `stroke_from_samples`,
`stroke_from_samples_cached`, `build_stroke`, `simplify_tolerance`, `resample_step`,
`srgb8_to_linear` + os testes) foi para a crate. Com ela, as três cenas saíram **inteiras**.

| | |
|---|---|
| `max_level` dos três | **1**, e foi **CONTADO no roteador** (os três são `var_os(..).is_some()` ⇒ interruptores) |
| `cargo run -p ph2d-app-sync` | 4 blocos regenerados; gate de *staleness* verde |
| os 5 gates do registo | **5 passaram** (`no_two_families_claim_the_same_router` incluído) |
| a shell ainda lê os três? | **não** (grep de código, sem comentários) |

⛔ **`PH2D_FLIP_FILL_DEBUG` e `PH2D_FLIP_SELECT_DEBUG` NÃO entraram, e a ausência é a decisão:**
são **diagnóstico** (ligam um `eprintln!`), não roteadores de cena. Um registo que os aceitasse
prometeria ao dono uma cena que não existe.

## 3 — ⛔⛔⛔ O BLOQUEIO: duas funções de outra família prendem 84 % da `flip`

**É o item nº 1 deste handoff e é decisão do integrador** — a §1 regra 2 do bloco de reabertura
manda exactamente isto: *«se a sua família tropeçar numa destas, não é porta e não é sua: reporte, é
linha própria.»*

```
crate::vec_transform::{world_transform, xform_of_transform}   4 ficheiros, 5 sítios
crate::name_unique::unique_name                               1 ficheiro,  1 sítio
```

**A cascata, medida** (com os blocos `impl crate::App` já resolvidos pelo trait/assinatura):

```
vec_transform ─→ transform ─→ tween_correct → strip → autokey → draw(bake) ─┐
                    │                       ↘ edit_gesture → select → layers, select_points
                    │                       ↘ entities  (← name_unique)
                    ├─→ pose_gizmo → trace
                    ├─→ gizmo_view
                    └─→ selection_gizmo
```

| | módulos | LOC |
|---|---:|---:|
| a família `flip` (pasta `src/flip/`) | 44 | 17 862 |
| **presos pela cascata** | **28** | **16 012 (90 % da pasta)** |
| movidos nesta Fase B | — | 2 838 |

**O que as duas funções SÃO, medido:** matemática de ECS **genérica**. `world_transform` são 6 linhas
sobre `ph2d_ecs`; `xform_of_transform` são 10 sobre `ph2d_ecs` + `ph2d_vec_scene::Xform`;
`unique_name` é `ph2d_ecs::SimWorld`. **Nenhuma toca a `App`** — escrito em tipos, pedem `&SimWorld`
e `Entity`. ⭐ O doc do nosso próprio `transform.rs` já o dizia: *«Reusa os helpers GENÉRICOS de
`vec_transform` — eles não tocam `VecScene`, só o `SimWorld`/`Transform`»*, e o `flip/transform.rs`
**já chama `ph2d_ecs::parent_world_transform` directamente**.

**As quatro saídas, cada uma fechada por uma regra:**

| saída | porquê está fechada |
|---|---|
| mover `vec_transform.rs` | é a árvore da `line/app-vec` (reaberta, parada no `main`), **74 consumidores** — 19 na pasta `vec/` + ~50 soltos dela. ⛔ §1 regra 5 |
| duplicar os dois helpers na crate | segunda resposta à mesma pergunta; o `ph2d-ecs` tem um aviso escrito: *«anything that computes in world space must ask THIS»* |
| usar o gémeo `ph2d_ecs::world_transform` | ⛔ **não é substituição directa** — ele devolve `Option<Transform>`, o da shell cai para `Transform::IDENTITY` (medido na Fase A) |
| 6.º método no `AppHost` | **não é porta**: as funções não pedem nada da `App`. A §1 regra 2 respondida pela batedora aplica-se à letra |

⇒ **A cura que o HOWTO §1.2 prescreve é uma FOLHA partilhada** (*«duas famílias que partilham código
partilham uma FOLHA, nunca uma delas à outra»*). ⚠️ Medido: **nem `ph2d-vec-scene`** (dona do
`Xform`) **nem `ph2d-ecs`** (dono do `Transform`) têm hoje a conversão nem a variante com queda para
identidade — logo a folha é trabalho novo, e é a **mesma espécie** das três folhas que a §1 regra 2
já nomeia (`inspector_ordering`, `preview_drive`, `name_unique`). ⭐ **`name_unique` é literalmente
uma delas**, e a `flip` toca-a num sítio.

## 4 — O que saiu, em três cortes

| # | o que | LOC |
|---|---|---:|
| 1 | **os CORPOS puros do passe de render** (HOWTO §4: o laço fica, os corpos saem) — `cursor`, `pass_cache`, `pass_ghosts`, `pass_stage` + testes | 933 |
| 2 | **6 cenas de smoke viram funções livres** — `arm(flip, tools, playhead)` | 460 |
| 3 | **3 cenas com o barramento de painéis** — `arm(flip, tools, hero, playhead) -> bool` | 183 |
| 4 | **o `draw` parte-se** e as 3 cenas do fim da linha saem inteiras | 1 262 |
| | **total** | **2 838** |

⭐ **O `HeroScreen` entra por PARÂMETRO, e há precedente no próprio substrato:**
`ph2d_app_host::canvas_area::visible(hero: &HeroScreen, …)`. O que o HOWTO §1.5 proíbe é um **método
do trait DEVOLVER** um handle — isso deixaria a família alcançar tudo, a qualquer hora; um parâmetro
é a shell a escolher o que entrega, num sítio que ela controla.
⭐ E o `any_input_this_frame` volta como **valor de retorno** (`-> bool`), nunca por `&mut bool`: a
fronteira atravessa-se com um valor, não com um campo alheio.

## 5 — A prova (§3 do HOWTO · regra H)

**(a) Nenhum teste se perde** — `cargo nextest list --workspace --cargo-profile ci-test`, antes
(na base) e depois, por `scripts/nextest-list-diff.py`:

```
antes: 22659 testes (22035 chaves) | depois: 22659 (22035)
MOVED (mesma chave, outro pacote/binário): 22     → ph2d-app-flip
ONLY-A (perdidos): 0          ✅
ONLY-B (novos):    0
```

**(b) Nenhum roteador se perde** — censo **com os comentários retirados** nos dois lados:
**18 envs lidas por CÓDIGO, idênticas**, zero perdidas, zero inventadas.
⚠️⚠️ **A armadilha §2.12 apanhada ao vivo:** a 1.ª corrida do censo acusou `PH2D_FIELD_SMOKE` como
«roteador perdido». Ele é do **field3d**, aparece em **dois comentários** do `render_loop/mod.rs`, e
o ficheiro nem é meu. *Um censo que lê prosa como código mente nos DOIS sentidos* — e mentiu do lado
caro: leu-se como uma cena apagada.

**(c) A shell encolheu** — e ⚠️ **as DUAS réguas desta grandeza discordam, o que É o achado:**

| régua | antes (main) | depois | Δ |
|---|---:|---:|---:|
| **a do gate** (`shells/desktop` inteira, com `tests/`) | **451 084** | **448 246** | **−2 838** |
| a minha 1.ª (só `src/`) | 417 195 | 414 357 | −2 838 |

⚠️ O `the_shell_only_shrinks` conta a **pasta da crate**, não `src/` — e o `TETO_LOC = 455_084` é
exactamente `451 084 + FOLGA_DE_COMPOSICAO(4 000)`. Eu li a minha régua contra o tecto dele e concluí
que a metade de obsolescência ia reprovar; **ela passa**, porque a base era outra. *Duas leituras da
mesma grandeza a discordar é o achado, e a que decide é a do gate.* ⛔ **`TETO_LOC` não foi tocado** —
quem o reconta é o integrador, sobre a árvore junta.

⛔ **A medição de RELÓGIO não foi feita, e a razão é honesta:** o delta é **0,63 %** da shell
(2 838 de 451 084). Um `--timings` a frio mediria ruído, não a cura — e produzir um número que não
descreve nada é pior que dizer que não se mediu. **Fica para quando a folha do §3 destravar os 84 %.**

**(d) Gate de fecho** — `nextest-impacted.sh`: **15 006 testes, 15 006 passaram, 0 falharam**
(`load 2,50` impresso ao lado — máquina calma, logo as réguas de razão são honestas).
`clippy --all-targets` nas duas crates: **0 avisos**. `cargo fmt --all --check`: limpo.
`the_shell_only_shrinks`: **passa**. Os 5 gates do `ph2d-app-registry-init`: **passam**.

**(e) Smoke** — §8.

## 6 — ⚠️ As armadilhas que esta Fase B pagou (para as outras quatro linhas)

1. ⛔ **`pub(super)` não é `pub(crate)`.** 22 itens dos ficheiros do `render_loop` eram `pub(super)`
   — ali queria dizer *«visível ao `render_loop`»*, e na crate quer dizer *«visível à raiz»*. O
   `draw_flip_cursor` ficou **invisível ao único consumidor dele** e o aviso foi `never used`, não um
   erro. *Uma varredura que só converte `pub(crate)` deixa a porta fechada.*
2. ⚠️ **Três dependências eram INVISÍVEIS até a crate existir** (HOWTO §1.3): `ph2d-flip-render`,
   `ph2d-vector`, `bytemuck` — e depois `ph2d-vec-scene`. Dentro da shell as quatro eram dependência
   do binário e ninguém as declarava.
3. ⚠️ **Os `use` não viajam com o corpo.** Mover um corpo de função para outro ficheiro perde os
   imports dele: 9 ficheiros perderam `Ordering`, e vários `Hold`/`KeyKind`/`DupMode`/`Pose`. Falha
   **ALTA** (E0433) — a metade boa.
4. ⛔ **A mesma regex falha por INDENTAÇÃO.** O `let gfx = self.gfx.as_mut()…` de três ficheiros vive
   dentro de um braço de `match` (12 espaços) e a minha limpeza cobria 8. Falhou alto (E0424
   *«expected value, found module `self`»*), mas **cada correcção foi por `assert` de contagem** —
   uma delas casou **zero** e o script **abortou** em vez de imprimir sucesso (CLAUDE.md §2).
5. ⛔⛔ **E o defeito mais caro foi MEU, num script de reescrita:** os meus rewrites do `lib.rs`
   faziam *head + `pub mod`s + tail* e **apagaram em silêncio o doc-comment inteiro do `const
   FAMILY`** — 14 linhas, incluindo a nota da dívida que esta Fase B existia para fechar. Só apareceu
   quando fui editá-la e o `index()` não a achou. Recuperado do `main`.
   ⇒ *Um script que reescreve por FATIAS perde o que está entre elas, e não avisa.* **Um `assert` de
   que o tamanho não encolheu mais do que o previsto teria apanhado isto.**
6. ⚠️ **Verificada a armadilha §2.8 (vacuidade):** os 9 `*_app.rs` foram de ~148 para 12–14 linhas.
   **Nenhum gate os lê por caminho** — se lesse, ficaria **VERDE a medir um invólucro**. Confirmado
   por grep nos dois sentidos.
7. ⚠️ **`#[path]` é aresta dura nos dois sentidos** (a lição da `line/app-vec`): `flip_pass_camera` +
   os testes dele são filhos de `flip_pass.rs`, que está preso ⇒ **ficam**. Dos 15 ficheiros do
   `render_loop`, só **5** eram movíveis; os outros 10 (2 815 LOC) são filhos de pais presos.

## 7 — Foundational tocado, e símbolos novos

**Foundational:** `shells/desktop/src/render_loop/mod.rs` (−4 declarações de `mod`),
`shells/desktop/src/flip/mod.rs`, `shells/desktop/Cargo.toml` (nada novo — a dep já existia da Fase A),
`crates/ph2d-app-registry-init/src/lib.rs` (**gerado** por `ph2d-app-sync`, não editado à mão).

| símbolo novo | valor | onde |
|---|---|---|
| módulos da crate | `cursor`, `pass_cache`, `pass_ghosts`, `pass_stage`, `draw` + 9 `*_smoke::arm` | `crates/ph2d-app-flip/src/lib.rs` |
| ficheiro novo na shell | `flip/bake.rs` + 9 `flip/*_smoke_app.rs` | `shells/desktop/src/flip/` |
| roteadores no `FAMILY` | **15 → 18** | `crates/ph2d-app-flip/src/lib.rs` |
| deps novas da crate | `ph2d-flip-render`, `ph2d-vector`, `ph2d-vec-scene`, `bytemuck` | internas + `bytemuck` (já no lock) |

⛔ **Nenhum id, const, variant, token ou schema novo.** Nada a colidir.

## 8 — O que smoke-testar (nada mudou de produto — é isso que se confirma)

Os três do fim da linha, que são os que mais se mexeram:

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-flip && env PH2D_FLIP_HARDNESS_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-flip && env PH2D_FLIP_STRIP_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-app-flip && env PH2D_FLIP_TIP_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
```

## 8-bis — Smoke COMPILADO (DIRETRIZ §1.5.9 item 9 · regra I)

`cargo build -p ph2d-host-desktop --profile smoke` na **worktree desta linha**, 2ª corrida:

```
    Finished `smoke` profile [optimized] target(s) in 0.22s
```

**Zero linhas `Compiling`.** Binário em `target/smoke/ph2d-host-desktop` (77,9 MB).
O `target/*/incremental` foi reclamado antes (**10,6 GB**: 8,1 de `debug` + 2,5 de `smoke`), e as
duas coisas não se anulam — o perfil `smoke` recria o dele na 1.ª corrida, que é a que acabou de
correr.

## 9 — O que só o `ship.sh` apanha

`fmt` e `clippy --all-targets` correm aqui (§5); **não** correram: `machete` (a crate ganhou 4 deps —
é o candidato mais provável a um `✗`), `deny`, `audit`, `typos`, e o `doc-index` sobre
`docs/Flip/handoffs/README.md`, que ganha esta entrada.

## 10 — Para o integrador

1. ⛔ **A decisão do §3 é a única coisa que esta linha pede**, e ela serve ≥2 famílias: uma folha com
   `world_transform`/`xform_of_transform` (e provavelmente `unique_name`). Sem ela a `flip` fica
   com **16 012 LOC** na shell e o mesmo vale, em parte, para a `vec`.
2. **Reconte o `TETO_LOC`** sobre a árvore junta — a minha ponta é `448 246` pela régua do gate.
3. ⚠️ **O `render_loop/mod.rs` tem 13 995 linhas** e está na allowlist; eu tirei-lhe 4 declarações de
   `mod` e nada mais.

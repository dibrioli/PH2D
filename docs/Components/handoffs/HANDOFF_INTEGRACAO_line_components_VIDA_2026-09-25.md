# HANDOFF DE INTEGRAÇÃO — `line/components`: os ABERTOS + a PARALAXE + a VIDA E DANO (W0–W5) — 2026-09-25

> ⛔⛔ **Este documento SUPERSEDE o [`…_PARALAXE_2026-09-23.md`](HANDOFF_INTEGRACAO_line_components_PARALAXE_2026-09-23.md)
> (e, através dele, o [`…_OS_ABERTOS_2026-09-20.md`](HANDOFF_INTEGRACAO_line_components_OS_ABERTOS_2026-09-20.md))
> como documento de integração.** Nenhum dos três blocos foi integrado: a linha entra no `main` de
> uma vez. **O detalhe dos dois primeiros blocos continua nos handoffs deles** (o §3 da PARALAXE — o
> que um merge textual pode partir — vale inteiro e não é repetido aqui); este documento re-mede a
> superfície da linha INTEIRA e acrescenta o bloco da vida.

## §0 — ⛔ LEIA PRIMEIRO: o `--ff-only` VAI RECUSAR, e a causa não é código

A árvore do **primário** (`/home/enio/Documentos/Projetos/PH2D`, ramo `main`) tem **memória por
gravar de várias sessões** — o symlink `~/.claude/projects/<key>/memory` aponta para o
`project-memory/` do primário, logo qualquer sessão escreve lá. Medido em 25/09:

| ficheiro no primário | estado | a linha também o traz? |
|---|---|---|
| `project-memory/MEMORY.md` | modificado | **sim, com conteúdo DIFERENTE** (`+10/−4` entre os dois) |
| `project-memory/reference_topic_gate_discipline.md` | modificado | **sim, DIFERENTE** |
| `project-memory/reference_topic_measurement_discipline.md` | modificado | **sim, DIFERENTE** |
| `project-memory/reference_topic_mutation_proofs.md` | modificado | **sim, DIFERENTE** |
| `project-memory/feedback_a_picker_swatch_emits_no_event_and_a_click_arm_for_it_is_dead_code.md` | por rastrear | sim, **idêntico** (sha1 igual) |
| mais 4 modificados e 5 por rastrear | — | não |

⇒ `git merge --ff-only line/components` no primário **recusa** («your local changes would be
overwritten» / «untracked working tree files would be overwritten»). ⛔ **Não descarte nada** — são
lições de outras frentes (o albedo por texel, as guardas cegas a `NaN`, o corpus de ossos…).
⭐ **A forma segura:** gravar primeiro a memória do primário **como commit próprio no `main`**
(`git add -- project-memory/…` só desses ficheiros), depois `git -C Worktrees/line-components rebase
main` — os quatro ficheiros conflituam e a cura é a **UNIÃO** (memória é append: cada lado
acrescentou entradas; nenhuma apaga a do outro) —, e só então o `--ff-only`. ⚠️ O `MEMORY.md` tem
**tecto medido de `22 000` bytes** (há gate); a linha deixa-o em `21 989`, logo a união **vai
passar** do tecto — a cura que o próprio índice prescreve é descer entradas para o tópico da
família, nunca subir o número. O ficheiro idêntico (`feedback_a_picker_swatch…`) resolve-se sozinho
depois de o primário o gravar.

---

## §1 — Identidade e superfície de colisão, MEDIDA

- ramo `line/components` · HEAD **`02a4be606`** · **53 commits** · `382` ficheiros
  (`+32 458` / `−797`)
- ⭐ **rebased em 25/09 sobre o `main` de então** (`20a630f1b`) — o `main` tinha andado **UM**
  commit desde o merge-base antigo `395da6a55`, só memória (`project_teste_cascadeur_…`), com
  **zero** ficheiros em comum com a linha ⇒ rebase sem conflito. **A árvore combinada É esta**, e os
  censos da soma correram sobre ela (§5).

`bash /home/enio/Documentos/Projetos/PH2D/scripts/collision-surface.sh`, colado:

```
  merge-base 20a630f1b   ·   53 commit(s)   ·   382 arquivo(s)
▸ SCHEMAS
  ⚠ PROJECT_SCHEMA                        171   (base: 160)
  ⚠   └ tripla do gate               (171, 13, 22)   (base: (160, 13, 22))
    VEC_SCENE_SCHEMA 22 · FLIP_SCHEMA 13 · DOC_VERSION 18 · FIELD_DOC_VERSION 23   (intactos)
▸ REGISTRO DE COMPONENTES
  ⚠ ph2d-ecs                              107   (base: 103)
  ⚠ ph2d-render (espelho)                 108   (base: 104)
  ⚠ ph2d-script (espelho)                 108   (base: 104)
▸ CONTRATO CONGELADO (§6)       node.rs intocado · tool.rs intocado
▸ ADR                           esta linha não cria ADR
▸ Cargo.lock                    1 '+name' novo: "ph2d-health"  (crate INTERNA nova — zero pacote externo)
▸ MARCADORES DE CONFLITO        nenhum
▸ TETOS DE LOC                  nenhum arquivo da linha passa do teto
```

| grandeza | aqui | `main` | delta | de onde |
|---|---|---|---|---|
| `PROJECT_SCHEMA` | `171` | `160` | **+11** | `+3` abertos (`161`–`163`) · `+5` paralaxe (`164`–`168`) · `+3` vida (`169` Health+Damage · `170` HealthBar · `171` o campo `knockback_recovery` do `TopDownPlayer`) |
| registo `ph2d-ecs` / `-render` / `-script` | `107`/`108`/`108` | `103`/`104`/`104` | **+4** nos três | os quatro `Scroll*` da paralaxe |
| ⚠️ **registo da FÍSICA** (`register_physics_components`, gate `reg.len()`) | **`40`** | `37` | **+3** | `Health` · `Damage` · `HealthBar` — ⛔ **o `collision-surface.sh` NÃO o mostra**; conte-o no `ph2d-physics-ecs/src/lib.rs` |
| `LIVE_SECTIONS` | `42` | `38` | **+4** | Parallax (paralaxe) · Health · Damage · Health Bar |
| `any_live_section([bool; N])` | `34` | `32` | **+2** | |
| `SignalVerb::ALL` e o array de ids `INSP_ACTION_VERB` | `12` | `10` | **+2** | `Damage` · `Heal` (**APENDADOS** — a posição é a tag do postcard) |

⚠️⚠️ **CONTE O DELTA, nunca o literal** — as quatro últimas linhas são contagens que SOMAM entre
linhas, e *a colisão passa MUDA quando duas linhas escrevem o mesmo literal*. Os onze degraus moram
na escada do [`project_schema.rs`](../../../shells/desktop/src/project_schema.rs) e a tripla no
[`project_schema_tests.rs`](../../../shells/desktop/src/project_schema_tests.rs); todos **sem degrau
de migração** (decisão do dono de 26/08: um blob anterior é recusado em voz alta), com a EXCEPÇÃO
declarada da paralaxe (`GameCameraV128` + `migrate_v128_to_v129`, handoff dela §8).
⛔ **O `project_schema.rs` está em `577` de `600` linhas** — uma rodada que some mais de ~2 degraus
ao desta linha estoura o tecto, e a cura é a **quinta faixa arquivada** da escada (o padrão
`project_schema_history_v…` já tem quatro irmãos — `grep 'mod project_schema_history' main.rs`
antes de cortar), nunca uma isenção.

---

## §2 — O bloco da VIDA E DANO (plano 28, W0–W5)

Pesquisa · plano · resultados por wave: [`27_pesquisa_vida_e_dano.md`](../27_pesquisa_vida_e_dano.md)
e [`28_plano_vida_e_dano.md`](../28_plano_vida_e_dano.md) (§7 = W1 · §8 = W2 · §9 = W3 · §10 = W2b ·
§11 = W4 · §12 = W5). O oráculo é a extensão *Health* do **GDevelop 5.6.282 (MIT)**, corrida sem
interface (W0, 19 cenários com cabeçalho).

| wave | o que traz |
|---|---|
| **W1** | crate-folha nova **`ph2d-health`** — a lei pura (vida, escudo, armadura, invencibilidade, regeneração), **ao bit** contra o GDevelop nos 19 cenários, com as divergências declaradas em `Regras::CASA` |
| **W2** | `Health`/`Damage` (`ph2d-physics-ecs`) e a ponte **no passo da física**, em live **e** replay, com o estado no anel (`ControllerMemory`); três fontes de golpe (contacto sólido · sensor · o canal do mover nas três pontes); a **porta das mortes** (`bridge::mortes::mortes_anunciadas`) filtrada por `is_transient`; cena `PH2D_VIDA_SMOKE=1` |
| **W3** | as secções **Health** e **Damage** do Inspector; o registo na física (a cópia profunda da fábrica só leva componentes REGISTADOS — era o *«ninguém sumiu»* do smoke da W2) |
| **W2b** | verbos **`Damage`/`Heal`** na tabela de acções (o verbo ANUNCIA, `ActionReport::pedidos_de_vida` → `PhysicsBridge::pede_vida`, aplicado no PRÓXIMO tique e gravado na fita por tique); `ArgKind` novo para a dica do argumento |
| **W4** | a **barra de vida** (`HealthBar`, registado): lei do rasto em `ph2d_hud::barra`, ponte `health_bar_bridge` (o 5.º produtor do slot `extra` do passe de sprites, `z_order = u32::MAX`), secção **Health Bar**, e o PLACAR (barra que mostra a vida de outro objecto pelo nome) |
| **W5** | o **impacto**: pausa no golpe (relógio de parede retido antes do acumulador do passo fixo), piscar (`ph2d_ecs::BlinkOff`, **derivado e NÃO registado**, lido pela porta única `draws_this_frame`), empurrão pela normal do toque (dinâmico por `PhysicsWorld::push_velocity`; mover de vista de cima por canal próprio `TopDownState::knockback`), números de dano; cena `PH2D_VIDA_SMOKE=2` |

⭐ **Achados pré-existentes curados de passagem** (cada um com gate): a secção **Top-Down Player**
nunca era semeada do objecto (faltava o `sync_topdown` desde o TOP-20 #13) · a secção **Signal
Actions** do Inspector desceu da shell para a família (catraca `the_shell_only_shrinks`).

---

## §3 — ⚠️ O que um merge textual pode partir — o bloco da VIDA (o da paralaxe está no §3 dela)

1. ⛔⛔ **Dois ficheiros MUDARAM-SE da shell** (`git mv`): `shells/desktop/src/render_loop/inspector_action.rs`
   (+ `_tests`) → [`ph2d-app-components/src/signal_actions_inspector.rs`](../../../crates/ph2d-app-components/src/signal_actions_inspector.rs).
   Outra linha que edite o endereço antigo conflitua por *rename/modify* — re-aplicar no novo.
   (Somado ao `inspector_camera.rs` da paralaxe, são **dois** movimentos da shell nesta linha.)
2. **Variantes novas em enums partilhados** — um `match` exaustivo noutra linha deixa de compilar
   (o compilador diz onde): `ComponentEdit::Vida(VidaFieldEdit)` · `SignalVerb::{Damage, Heal}`
   (**apendadas**; e o array de ids do seletor tem de seguir, senão o verbo existe e o artista não
   lhe chega) · `TopDownFieldEdit::KnockbackRecovery`.
3. **Campos novos em structs com literais noutras linhas** — a cura é o neutro de cada um:
   - `TopDownLaw::knockback_recovery` (`24.0`, a lei) · `TopDownPlayer::knockback_recovery` ·
     `InspectorTopDownInfo::knockback_recovery` · `TopDownState::knockback` (`[0,0]`);
   - `Health`/`Damage` nasceram nesta linha, mas os campos da W5 (`death_hitstop_s`, `blink_s`,
     `knockback_taken` = **`1.0`**, `numbers`, `numbers_color`, `numbers_size`; `hitstop_s`,
     `knockback`, `knockback_lift`) entram em todo *struct literal* com `..Default::default()`.
4. **O `ActionReport` perdeu o `Eq`** (carrega um `f64`) — uma comparação `==` dele noutra linha
   deixa de compilar.
5. **Fases do quadro tocadas** (a ORDEM é o atrito real, não os ficheiros):
   - `fase_fixed_step_clocks.rs` — o `ImpactoState::retem` corre **ANTES** do
     `fixed_step.advance(wall_dt)`; uma linha que mexa no `wall_dt` entre os dois muda a pausa;
   - `fase_physics_step.rs` — o `impacto.ouve` corre **DEPOIS** do dispatch (lê os factos do tique);
   - `fase_physics_overlay.rs` — os números pintam-se no fim, na banda da cena;
   - `motores_do_quadro.rs` / `present.rs` / `fase_fabrica_e_morte.rs` — as barras correm, entram
     no atalho de apresentação e **renascem** com a corrida.
6. **Contagens-catraca do painel** que outra linha que acrescente secção também escreve: ver §1
   (`LIVE_SECTIONS`, `any_live_section`), mais as fixturas de `InspectorVidaInfo` em
   `as_quatro_seccoes_que_estreavam_a_catraca.rs` e `o_inspector_armado.rs`.
7. **`ph2d-physics/src/world/blast.rs` ganhou `push_velocity`** (aditivo, só `Dynamic`, finito,
   acorda o corpo) e **`ph2d-entity-visibility/src/off_canvas.rs`** ganhou um termo no
   `draws_this_frame` (`BlinkOff`) — porta ÚNICA de desenho: uma linha que acrescente outro termo
   funde ao lado.

---

## §4 — ⏳ O que fica ABERTO

- ⛔ **SMOKE DA W5: *«Não vejo a bala»*** (dono, 25/09 — o resto do smoke aprovado). **NÃO
  investigado**, por ordem do dono (*«seguiremos depois»*): a bala da cena `=2` dispara (os
  inimigos levam golpe, voam e perdem vida) e **não se vê**. ⚠️ Pode ser o mesmo que o **defeito G**
  (a bala só-sensor, aberto desde a W2) ou outro — não medido. É a primeira coisa da próxima sessão
  da linha; **não bloqueia a integração** (é apresentação de uma cena de smoke, e a lei está
  gateada).
- **W6** (tipos de dano e resistências + dano contínuo) e **W7** (tutorial PDF + cena final) — por
  fazer.
- As fronteiras declaradas de cada wave: plano 28 §8.4, §10.7, §11.6, §12.7 (a pausa não congela a
  interface · números sem acumulação · projéctil e cinemático sem mover não são empurrados).
- Os abertos da paralaxe: handoff dela §4.

---

## §5 — A prova de fecho

| portão | resultado |
|---|---|
| `bash scripts/nextest-impacted.sh` (antes do rebase; o rebase só trouxe memória) | **`18 273 / 18 275`** — as duas reprovadas são **membros já nomeados** da família de flakes de fan-out (`the_cost_of_depth_is_linear_not_explosive` · `the_cost_of_a_player_is_linear_in_their_number`), **3 de 3 verdes sozinhas até a `load 117`**, zero linhas do diff nas crates delas |
| `cargo clippy -D warnings --all-targets` nas crates tocadas | **zero** |
| `cargo fmt --all --check` | limpo |
| `bash scripts/censos-da-arvore-combinada.sh` (**depois do rebase**, §1.5.9 5-bis) | **verde** — `12 de 12` censos correram, com o controlo do filtro |
| `cargo machete` | **zero** dependências por usar |
| `bash scripts/doc-index.sh --check` | em dia |
| catraca `the_shell_only_shrinks` | verde |
| `#[cfg(target_os` escrito/movido | **nenhum** nesta linha ⇒ o cruzamento para macOS não se aplica |

**Mutação:** W1..W5 cada uma com arnês versionado em [`ferramentas/`](../ferramentas/)
(`mutacao_vida_*.sh`); a da W5 dá **`51 / 51`** depois de três gates nascerem de sobreviventes
(plano §12.5). Paralaxe: `86 / 86` (handoff dela §5).

**Fotos** pelo [`fotografa_cena.sh`](../ferramentas/fotografa_cena.sh) (ecrã virtual, nunca o do
dono). ⛔ A da W5 apanhou herói e espinho cortados pela borda esquerda com os gates verdes — curado,
e a cerca do `x` é erro de compilação.

⚠️ **O que só o `ship.sh` corre e esta linha não correu:** `typos`, `cargo deny`, `cargo audit`,
`check-standalone-optional.sh`, `check-workflow-packages.sh` e o `cargo check --workspace
--all-targets` com `CARGO_BUILD_WARNINGS=deny` — a linha não acrescentou dependência externa nem
feature opcional, mas nada disto foi medido aqui.

---

## §6 — Os smokes (o comando inteiro)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_VIDA_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-components && env PH2D_VIDA_SMOKE=2 cargo run -p ph2d-host-desktop --profile smoke
```

A `=1` é a coluna de alvos com vidas diferentes, o veneno (tecla **J**), a cura do roxo e as barras
com o placar; a `=2` é o golpe que PESA (tecla **Q**) — com a bala **invisível** (§4). Os smokes da
paralaxe e dos abertos estão nos handoffs deles. ✅ Aprovados pelo dono: vida W2–W4, W5 (menos a
bala), paralaxe (depois da cura) e os abertos.

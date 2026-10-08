# HANDOFF DE INTEGRAÇÃO — `line/motion-value`, CICLO 13: os cinco abertos do módulo (07/10)

> Leitor: o integrador (e a próxima LLM da linha). Ordem do dono: *«resolver tudo que está em aberto em
> um único ciclo»* — a lista do [handoff de continuação](HANDOFF_CONTINUACAO_line_motion_value_2026-10-07_ABERTOS_DO_MODULO.md).
> O porquê inteiro (medições, desenhos, recusas) está no [doc 122](../122_ciclo_13_os_abertos_do_modulo.md).
> ⛔ A linha NÃO integra nem faz push: integrar é só por ordem do dono.

## §1 — Base e commits

- Base: `main` em `2a925859d` (a linha avançada por `--ff-only` depois da rodada de seis linhas). O
  `main` não andou durante o ciclo (`git rebase main` = no-op à abertura).
- Commits (`git log --oneline 2a925859d..HEAD`): o handoff de continuação · `ciclo 13 (1/5)` Bug #14 ·
  `(2/5)` + o doc do kernel do `pulse.beat` · `(3+5/5)` as ligações guardadas e a leitura no submit do
  cozimento · `(4/5 + docs)` a tabela do tecto, doc 122, doc 103, BUGS, o roteiro da escada, o gate das
  ligações · o censo do Bug #14 para um irmão (tecto de LOC) · este handoff.

## §2 — O que fechou (os cinco)

| # | item | veredito | gate |
|---|---|---|---|
| 1 | Bug #14 | **não era perda**: a estrela tinha `~3 px` (`0,07` em mundo, `48 px`/unidade) e a sonda `PH2D_PAN_DIAG` era cega à rota do dispositivo. `ESTRELA = 0,22`; a sonda diz a rota da placa | `the_sparks_draw_legible_stars_on_both_routes` (vermelho a `0,07`); censo `census_stamps_downstream_of_a_zone` (`1` cena) |
| 2 | empate batida-tique do `pulse.beat` | o compasso (`k₀`, `frac`, `sc`) decide-se no hospedeiro em `f64` (`DerivedUniform`); as duas rotas fazem a mesma aritmética; `sc` com `7` bits ⇒ `i·sc` exacto (imune a FMA) | `parity_pulse_beat` com `91` tiques · `parity_pulse_beat_on_the_tick` · a paridade do nascimento por pulso deixa de saltar tiques |
| 3 | preparo de `value.attribute` + `value.reduce` | `+0,070 → +0,040 ms` (RTX) / `+0,039` (iGPU) sobre o constante — kill `≤ 0,05` ✅ — por ligações, uniforms e rascunhos GUARDADOS (`ph2d-gpu-cook/src/ligacoes.rs`) em todo o cozimento | `as_ligacoes_guardadas_nao_crescem_e_as_velhas_morrem`; sonda `sonda_as_ligacoes_guardadas` |
| 4 | tecto de `32 768` | **remedido** (§4 do doc 122): desktop imagem `≥ 1 048 576`, estrela `524 288` a 60 fps; proxy de telemóvel estrela `32 768` / `~49 152`, imagem `~131 072` / `~262 144`. **O número é do dono** — recomendação `65 536`; nada mudou | — (medição) |
| 5 | submissão da leitura de volta | o `0,34–0,42 ms` não reproduz (`0,045–0,058`; o `submit` próprio `0,019`/`0,016`); a leitura vai no MESMO `submit` do cozimento, destino/leitura/ligações guardados | `o_cozimento_leva_a_leitura_no_seu_submit`; o controlo de `um_quadro_da_cpu_descarta_a_leitura_da_placa` (que apanhou o `Buffer is already mapped` da leitura guardada — curado por `solta_a_leitura`) |

## §3 — Superfície de colisão (para o integrador)

`bash scripts/collision-surface.sh` (caminho absoluto, nesta worktree): `PROJECT_SCHEMA 187` (base
`187`), tripla `(187, 13, 22)`, `VEC_SCENE 22`, `FLIP 13`, timeline `18`; registos de componentes
`105/106/106` iguais à base; **contratos congelados INTOCADOS** (`node.rs`, `tool.rs`); nenhuma ADR;
nenhum pacote externo novo no `Cargo.lock`; zero marcadores; **nenhum ficheiro acima do tecto de LOC**
(o `motion_state_demo_router_census.rs` passava — `763/700` — e o censo novo saiu para
`motion_state_demo_router_census_zona.rs`).

Ficheiros que outra linha pode ter tocado (foundational da placa): `ph2d-gpu-cook/src/{encode,
reduce_stage,reduce,scan,stream_op,tap,tap_voo,lib,estado,grid}.rs` (a memória de ligações entra em
cada criação de `BindGroup`/uniform — um conflito aqui resolve-se mantendo a chamada a
`self.ligacoes.grupo(...)`/`uniforme(...)` no lugar do `create_bind_group`); `ph2d-pan-diag/src/lib.rs`
(`note_device`); `ph2d-node-pulse-beat/src/lib.rs` (o `Compasso`, e um `const assert` que prende
`MAX_INSTANCIAS_POR_NO ≤ 131 072` — quem subir o tecto acima disso baixa o `SC_BITS`).

## §4 — Gate batched (sobre `2a925859d..HEAD`)

| passo | resultado | carga |
|---|---|---|
| `nextest-impacted.sh` (`BASE=2a925859d`, inclui `file_loc_caps`/órfãos) | ✅ `4887/4887` | `23,6` |
| `RUSTFLAGS=-D warnings cargo check --all-targets` (gpu-cook, app-motion, pulse-beat, pan-diag) | ✅ | `78` |
| `cargo clippy --all-targets -D warnings` (as quatro) | ✅ depois da cura (`HashMap` → `BTreeMap` na memória de ligações, HR-5; três empréstimos no `encode`) | — |
| `cargo fmt --check` (as quatro) | ✅ depois da cura (ordem dos `mod` no router) | — |
| `censos-da-arvore-combinada.sh` | ✅ `12/12` censos, `114/114` | `56` |
| `doc-index.sh --check` | ✅ `17` índices | — |
| placa `ph2d-gpu-cook --release --include-ignored` | ✅ `57/57` + `337/338` (o flake de carga, `3/3` sozinho a `7,8`–`11,8`); o SIGSEGV ao sair (§6) | `35` |
| placa `ph2d-app-motion --lib --release --include-ignored` | ✅ para o diff: a corrida inteira bate o prazo de `1800 s` (sondas lentas); dos `14` vermelhos sob contenção, `8` passam sozinhos e `6` são sondas `#[ignore]` de ficheiros FORA do diff (ver §7) | — |
| depois das curas do clippy: `ligacoes_guardadas`, `tap_sem_espera`, `readout`, `parity_pulse` na placa | ✅ `1`/`3`/`15`/`15` | — |

## §5 — Provas de mutação (pré-voo das âncoras `1×`, alvo visto VERDE antes, corrida limpa no fim, árvore restaurada com `touch`)

| # | mutação | gate | resultado |
|---|---|---|---|
| 1 | `ESTRELA` `0,22 → 0,07` | `the_sparks_draw_legible_stars_on_both_routes` | sangrou (`:297`) |
| 2 | `bt_k` pela lei `f32` antiga do `playhead` | `parity_pulse_beat` + `_on_the_tick` | sangrou `2/2` (`:166`) |
| 3 | `quantiza` = identidade (`sc` sem os `7` bits) | os mesmos | ⚠️ **SOBREVIVEU** — nesta placa o produto não fundido dá o mesmo lado; a imunidade a FMA é uma lei SEM régua nesta máquina (nomeada; `SC_BITS = 24` nem compila — o `const assert`) |
| 4 | não guardar a ligação (`grupos.insert` apagado) | `as_ligacoes_guardadas_nao_crescem_e_as_velhas_morrem` | sangrou (`:71`) |
| 5 | não envelhecer (`grupos.retain` apagado) | o mesmo | sangrou no 2.º assert (`13` contra `32`) |
| 6 | `let tap = None;` no cozimento | `tap_sem_espera` | sangrou (`o_cozimento_leva…` `:254`, `sem_nada…` `:180`) |
| 7 | `solta_a_leitura` vazia | `um_quadro_da_cpu_descarta_a_leitura_da_placa` | sangrou (`Buffer is already mapped`) |

## §6 — Conhecidos que NÃO são deste ciclo

- ⛔ O binário de testes `it` do `ph2d-gpu-cook` morre com `SIGSEGV` dentro de `libnvidia-glcore`
  DEPOIS dos testes, na RTX: **ablação** — o binário da base `2a925859d` (worktree temporária, target
  próprio) morre igual, `3/3` contra `3/3`, intercalado. Família do doc 121 §9.18 (F).
- `crossing_the_reach_boundary_does_not_step_the_cost` — flake de carga registado; `2/2` verde sozinho.
- Fora do módulo (para o integrador registar): a sonda `FIELD_DOC_VERSION` do `collision-surface.sh`
  estava cega desde a saída do 3D (o handoff de continuação nomeou-a; nesta corrida não imprimiu linha).

## §7 — O que fica ABERTO

- **O número do tecto** — decisão do dono (doc 122 §4.3); a cura a sério nomeada é um tecto POR
  DISPOSITIVO, não feito.
- A sonda `PH2D_PAN_DIAG` diz a rota da placa mas não lê o MUNDO das cópias do dispositivo (seria uma
  descarga por quadro) — nomeado, não pedido.
- A imunidade do `pulse.beat` a FMA não tem régua nesta máquina (mutação 3, §5).
- ANTERIORES a este ciclo, apanhados pelo gate (ficheiros fora do diff): o gerador
  `write_the_value_figures` (tutorial 6) coze a `=117` sem publicar as formas e lê `0` valores desde
  que a `=117` ganhou estrelas; as figuras `tutoriais/fig/params*.html` estão velhas contra o código
  (o gerador escreve «digitável até 181» onde o ficheiro diz `1000000`; o `motion.grid` perdeu a linha
  `Shape`) — o PDF do tutorial também; as sondas `audit_the_corner_radius_cost` /
  `audit_the_stamp_frame_split` pedem as `102 400` linhas recusadas no ciclo 6; `audit_what_the_camera_can_see`
  (`23,4` contra `~39`); `the_ceiling_is_honoured_on_every_tick_including_the_turn` (ABERTO declarado no
  próprio `#[ignore]`); `measure_the_source_group` pede `PH2D_MOTION_SO_COM_FORMA=0`.

## §8 — Smoke e binário

- Smoke ao dono: a `=27` (`PH2D_GPU_COOK_DEMO=27`, `--release`) com as estrelas a estourar — fotografada
  (`fotografa_cena.sh`, `1600 × 900`): a `~7 s` estrelas de `~10 px` legíveis; a `8 s` a cascata já saiu
  do quadro (a cena dura `~4 s`; rever pela Timeline). E a tabela do tecto (doc 122 §4.3) para ele decidir.
- O binário (2.ª corrida, zero `Compiling`):

```
$ bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --release
▸ linha line_motion_value · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `release` profile [optimized] target(s) in 0.23s
```

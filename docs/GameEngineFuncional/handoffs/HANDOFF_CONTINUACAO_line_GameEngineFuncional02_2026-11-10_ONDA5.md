# HANDOFF de continuação — `line/GameEngineFuncional02`, 10/11, onda 5 («Set Scale to 1» em cadeia · sai «Create this action»)

Leitor: o próximo agente da linha (ou o integrador). Handoff de integração anterior (já no `main`): [OUTLINE_E_F3](HANDOFF_INTEGRACAO_line_GameEngineFuncional02_2026-11-09_OUTLINE_E_F3.md). Memória das decisões do dono: [respostas à crítica da física](../../../project-memory/project_physics_critique_answers_2026_11_09.md) (acrescentada em `a018ff806b`).

## 1. Identidade

Branch `line/GameEngineFuncional02`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02`. Reaberta do `main` de 10/11 por `git reset --keep main`: os 8 commits antigos (HEAD antigo `859b28d667`) já estavam no `main` pela rodada de integração de 10/11. `PROJECT_SCHEMA` no `main` = 231; **esta onda NÃO mexe no schema** nem em registos, ADR ou contratos congelados. Nada integrado, nenhum push.

`git log --oneline main..HEAD` (antes deste handoff), do mais antigo ao mais novo:

| SHA | O quê |
|---|---|
| `a018ff806b` | memória do dono (2 linhas) |
| `cab51710a9` | sai «Create this action» do cartão do Trigger |
| `988aefcf72` | foto do cartão do Trigger sem o botão velho |
| `3bb9f4e91c` | «Set Scale to 1» conserta a cadeia inteira |
| `271a4af715` | a foto «esticada» mostra a cadeia |
| `2d1af2732c` | foto correta (`fisica_esticada`) |

## 2. Decisões do dono (10/11)

1. «Set Scale to 1» conserta TUDO num clique só = **SIM** (fecha o ABERTO 1 do handoff anterior).
2. Retirar o botão antigo «Create this action» = delegado («o que achar melhor») → **retirado** (fecha o ABERTO 4).

## 3. O que mudou

**Item 1 — `3bb9f4e91c` (+164 −70, 11 ficheiros).**
- `InspectorPhysicsInfo::estica` passou de `Option<u64>` a `Vec<u64>`: a cadeia de quem estica (a forma e cada pai esticado). Literais fora desta linha que escrevam `estica: None` partem (agora `Vec::new()`/`Default`).
- `PhysicsFieldEdit::ScaleToOne` é variante nova (match exaustivo parte).
- `crates/ph2d-app-physics/src/inspector/estica.rs`: `quem_estica(world, forma) -> Vec<Entity>` devolve a lista; `escala_a_um(...)` põe ±1 mantendo o SINAL (o espelho, p. ex. Scale X −2, vira −1).
- Aplicação: `physics_apply.rs`, `recibo_fisica.rs`, `physics_cartao_forma.rs`, `body.rs`, `inspector_model_physics.rs`. A cadeia inteira é UM passo de undo (um Ctrl+Z devolve forma e pais).
- Caso de foto `fisica_esticada` em `crates/ph2d-physics-scene/src/fotos_dos_cartoes.rs`: corpo Crate (1,3) com forma redonda (2,1) (`271a4af715`, `2d1af2732c`).

**Item 2 — `cab51710a9` (+71 −185, 13 ficheiros).** Removidos: `botao_criar_a_accao`, `INSP_TRIGGER_CREATE_ACTION`, `EditorAction::CreateInputAction`, `PedidoDoMapa::Criar` e a chave i18n. O aviso passa a «…yet — press Create and set key.». O «Create and set key» da linha Key (`action_trigger_tecla.rs`) é o ÚNICO que cria a acção. Tocados: `ph2d-app-components` (`trigger_smoke*.rs`), `ph2d-editor-core` (`action_bus.rs`, `input_map/pedidos.rs`), `ph2d-i18n` (`inspector_game.rs`), `ph2d-panel-inspector` (`event_action_trigger`, `ids`, `populate`, `sections/action_trigger.rs`, testes), `shells/desktop` (`fase_bus_inspector.rs`, `a_accao_pedida_pela_fileira_nasce_no_mapa.rs`).

## 4. O que um merge pode partir

- Quem constrói `InspectorPhysicsInfo { estica: … }` com `Option` (agora `Vec<u64>`): `body.rs`, `seam_physics.rs`, `o_inspector_armado.rs`, `o_cartao_da_fisica.rs`, `o_detail_trava_e_a_solta_sem_remove.rs` já ajustados.
- `match` exaustivo sobre `PhysicsFieldEdit` (variante `ScaleToOne`).
- Quem ainda use `EditorAction::CreateInputAction`, `PedidoDoMapa::Criar`, `INSP_TRIGGER_CREATE_ACTION` ou a chave i18n do botão velho: removidos.
- Pontos quentes: `ph2d-app-components` (PixelArt) tocado em 2 ficheiros (`trigger_smoke.rs`, `trigger_smoke_tests.rs`); `action_trigger_inspector.rs` NÃO tocado.
- **Catraca:** `physics_apply.rs` está em **599 linhas, teto 600** — qualquer linha nova (de qualquer linha viva) reprova; cura = MOVER, nunca subir o teto.
- Contadores como DELTA: schema 0 · registos 0 · cenas de smoke 0 (a foto `fisica_esticada` é um caso de foto, não uma cena) · ADR 0.

## 5. Prova de fecho (10/11, na linha; suíte completa e `nextest-impacted` NÃO corridos: são do integrador)

- Testes por nome, verdes: `set_scale_to_1_conserta_a_cadeia_num_clique` (`ph2d-app-physics/src/physics_tests_f1.rs`) · `sobre_a_accao_que_falta_so_a_linha_key_cria` (`ph2d-panel-inspector/tests/it/a_seccao_gatilho_esta_viva.rs`) · `criar_e_esc_deixa_a_accao_criada_sem_tecla` (`ph2d-editor-core/tests/it/o_cartao_do_trigger_escolhe_a_tecla.rs`). Os da linha Key em `crates/ph2d-panel-inspector/tests/it/a_linha_key_do_gatilho.rs` continuam verdes.
- **Mutação: 5 mutações, todas vermelhas** (cada uma sangra no teste do item acima; a lista nominal M1..M5 não foi copiada para este ficheiro — se precisar, refaça por `git show 3bb9f4e91c`/`cab51710a9`).
- `cargo fmt --all` sem diff. `clippy -D warnings` limpo nas crates tocadas: ph2d-app-components, ph2d-app-physics, ph2d-editor-core, ph2d-i18n, ph2d-panel-inspector, ph2d-panel-registry-init, ph2d-physics-scene, ph2d-host-desktop.

## 6. Smokes (o dono já fez: OK 10/11)

Prefixo: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && … cargo run --profile smoke -p ph2d-host-desktop`.

1. **Bola esticada.** `PH2D_GAME_SMOKE=4 PH2D_GAME_SMOKE_FOTO=fisica_esticada`. Aviso de forma esticada visível; clicar «Set Scale to 1» → aviso some, forma e Crate em 1×1; Ctrl+Z UMA vez devolve as duas; Scale X −2 vira −1 (espelho mantido). Errado: precisa de dois cliques, ou o espelho perde o sinal. Foto: [`fotos_fisica/onda5/01_esticada.png`](fotos_fisica/onda5/01_esticada.png).
2. **Trigger sem o botão velho.** `PH2D_TRIGGER_SMOKE=1`. Action = «fier» (por criar) → só «Create and set key»; clicar + Esc → «has no key yet» e botão «Set key». Errado: aparece «Create this action». Foto: [`fotos_fisica/onda5/02_trigger_sem_botao_velho.png`](fotos_fisica/onda5/02_trigger_sem_botao_velho.png).

Armadilha das fotos: com `docs/Components/ferramentas/fotografa_cena.sh`, as env vars vão TODAS no 1.º argumento, e os casos de física levam o prefixo `fisica_`.

## 7. ABERTO

Próximo item: a escolher com o dono. Candidatos, por ordem de risco:

1. As três suspeitas NÃO confirmadas do handoff anterior (§11.2): (a) altura da cápsula convertida com raio velho no mesmo quadro quando os dois têm chave (`canais_corpo.rs` `edicao` → `event_physics.rs:198`); (b) «Delete animation» do modal da regra B pode custar 2 passos de Ctrl+Z (`fase_bus_physics_sections.rs:45-49`); (c) janela Input Map aberta + cartão armado na mesma acção: a linha «Bind…» mostra escuta mas a tecla troca. (A suspeita (d), «N listen» contar a própria caixa, segue aberta também.) Meça antes de curar.
2. Crescimento do `OutlineCache` e da tabela de peças (`ph2d-physics/src/world/outline.rs`, `ph2d-physics-ecs/src/outline.rs`): só crescem numa sessão longa a arrastar Detail.
3. Mover os testes GEF02 para fora da shell (catraca `the_shell_only_shrinks`; cura = mover).
4. Canais da família JUNTA ainda em radianos.
5. Herdados do handoff anterior §11.5 (censo de sinais, O(pontos²) do traço, ponte de navegação, macro que muda o nº de pontos sem modal, painel `PH2D_CARDS=0`).

**Riscos:** `physics_apply.rs` a 1 linha do teto (599/600). Ficheiros `.log` de fotos por rastrear em `docs/GameEngineFuncional/handoffs/fotos_fisica/` (lixo do `fotografa_cena.sh`): NÃO comitar com `-A`; comite só por path. Suíte completa + `nextest-impacted`: do integrador. Integração só por ordem do Enio; smoke do dono não é aprovação de integração.

## 8. Prompt de continuação (colar numa janela nova)

```
/pd-linha-assumir line/GameEngineFuncional02

Worktree: /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02
Antes de ler: cd lá, `pwd`, `git branch --show-current` (tem de dar line/GameEngineFuncional02).
HEAD = o commit do handoff docs/GameEngineFuncional/handoffs/HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA5.md
(confira com `git log --oneline -3`). Leia esse handoff (§7 ABERTO) e CLAUDE.md.

Estado: onda 5 feita e com smoke do dono OK em 10/11 («Set Scale to 1» conserta a cadeia
num clique; sai «Create this action»). Nada integrado, nada com push. PROJECT_SCHEMA do main = 231.

Missão: o Enio escolhe o próximo item da §7 (suspeitas a/b/c/d da auditoria — MEÇA antes de curar;
OutlineCache; mover testes GEF02 para fora da shell; canais da JUNTA em radianos). Pergunte-lhe qual, em linguagem simples.

Regras de sempre: UI só em inglês, por tokens/i18n (zero hex, zero string solta); teste por NOME,
nascido vermelho, com prova de mutação; inner loop = cargo-check-narrow; comando pesado por
scripts/ph2d-run.sh; commit só por path (`git commit --no-verify -m … -- <paths>`), nunca -A,
não comite os .log de fotos; physics_apply.rs está em 599/600 — cure por MOVER; suíte completa é do
integrador; SEM push, integração ou fecho sem ordem do Enio; smoke ao dono em passos numerados com
foto tirada antes via docs/Components/ferramentas/fotografa_cena.sh (env vars todas no 1º argumento,
casos de física com prefixo fisica_) e imagem como link markdown relativo à raiz do repo principal
(Worktrees/line-GameEngineFuncional02/docs/…png). Respostas ao Enio: curtas, sem jargão.
```

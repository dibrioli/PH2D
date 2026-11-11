# HANDOFF de continuação — `line/GameEngineFuncional02`, 10/11, item 5 inteiro + JUNTA e MUNDO por tique + livro da junta

Leitor: o próximo agente da linha. Anterior: [ONDA8_LOOK_TRACO](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA8_LOOK_TRACO.md). O item 5 vinha do [ONDA8_ITENS_2_A_4 §7](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA8_ITENS_2_A_4.md) e fecha aqui.

## 1. Identidade

Branch `line/GameEngineFuncional02`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02`. Commits desta janela sobre `2e86f3d5e4` (do mais antigo ao mais novo):

- `bb5aa69d2c` — 5.5: a macro que toca não troca o contorno de uma forma com pontos animados.
- `d509c7100a` — 5.6: painel clássico (`PH2D_CARDS=0`) apaga os três travados do Outline.
- `f1fc68f464` — três imports sem uso (8.6b, 8.1a); o CI nega avisos.
- `b0c883aa53` — 5.4: nav e impressão do chão contam todas as formas do corpo.
- `5b8fc8ca46` — 5.1: censo de sinais ouve botões e listas do painel do artista.
- `8e150966ef` — 5.3: contorno acima do tecto desiste após o 1.º corte.
- `200f104a9a` — corte de LOC: comentários de `ph2d-app-physics` (−352 linhas).
- `fe7ed896ec` — item 2: JUNTA e MUNDO chegam ao solver por tique.
- `e55a3a90bd` — livro da junta: a outra mão leva só os campos que mudou.
- `9e7b28957c` — corte de LOC: comentários de `ph2d-app-macros` (−230 linhas).
- este handoff.

**`PROJECT_SCHEMA` inalterado: 233.** Merge-base = `bc4a6ce930`. Nada integrado, nenhum push. Binário `--profile smoke`: `target/smoke/ph2d-host-desktop` é de 22:54, ANTERIOR ao HEAD (23:56); outro agente está a compilá-lo agora — ver a resposta do smoke.

## 2. O que mudou

- **5.5 (`bb5aa69d2c`)** — `op::aplicar` mantém a topologia, logo não precisa de guarda. O buraco era a REPRODUÇÃO da macro: `palco::conduz` conduzia o blob do `ColliderOutline` sobre uma forma com trilha de pontos. Agora salta se `travados::numero_travado`. `ph2d-app-macros` passa a depender de `ph2d-shape-handles`. Mutação 2/2.
- **5.6 (`d509c7100a`)** — `num_row_unit` do painel clássico lê `card_field::parado` (agora `pub`): sem hit e `TextInputState::Disabled`. Mutação 2/2.
- **5.4 (`b0c883aa53`)** — `nav_custo::formas_da_zona` conta a principal e as partes via `nav_malha::pose_da_peca`; «estreito» = o corpo cabe em nenhuma forma (raio máximo). `palco_fisica::impressao` passa por todas as formas ativas. Mutação 3/3.
- **5.1 (`5b8fc8ca46`)** — `intent_de` e `intents_que_emite` do painel do artista; `signal-census` ganha `Fora.painel`/`com_painel` e FORA `AuthoredPanel.rows.signal Emit` (5 origens); a shell tem `authored_intents::{nomes_do_painel, publica_censo}`. Limite: só as linhas que o painel pinta agora (ao vivo se visível, senão compilado). Mutação 3/3.
- **5.3 (`8e150966ef`)** — a sonda mostrou que as peças acima de `MAX_POINTS` já eram saltadas, o marching custa 8–12 %, e o custo era o `simplify` refeito a cada tentativa. `simplify_capped` devolve `None` quando os vértices guardados passam o tecto; mesmos bytes abaixo dele. `trace_fitted` de ruído 256²: 1,18 s → 40 ms; 512²: 10,8 s → 156 ms; pentes ~inalterados. Tabela em `crates/ph2d-outline-trace/README.md` §Ceilings. Mutação 2/2.
- **Item 2 (`fe7ed896ec`)** — medido: com a curva da gravidade o desvio entre 1 e 3 tiques por quadro era 44,3 cm; com a rigidez da mola, 1,9 cm; scrub = play já dava 0. Cura: `SceneAtTick::mundo` + `PhysicsBridge::poe_a_cena` (porta única dos 4 laços + bake); `TimelineScene.livro = Livros{drive,fisica,mundo}`; `put` aplica `canais_junta` no livro do quadro; `mundo()` = `canais_mundo::alvo_do_tique` respeitando `ConducaoMundo.maos`. O despacho tem 3 livros; a shell passa `&mut self.physics.mundo`. Mutação 3/3. Curou também 2 avisos de clippy herdados.
- **Livro da junta (`e55a3a90bd`, subagente)** — a semente de âncora da ponte (`anchored=true`) fazia o livro ver «outra mão» e adotar o valor da curva como autorado. Cura: `Driven::com_outra_mao`, fusão de 3 vias campo a campo para `JointParams` (desestruturação exaustiva) em `ph2d-preview-drive`. Testes `o_ledger_do_quadro_guarda_a_junta_do_documento` e `a_outra_mao_so_leva_os_campos_que_mudou`. Mutação 2/2. **Lacuna conhecida:** se a curva fica parada depois do quadro 1, não há chamada `driven`, logo não há fusão até ela mudar (inofensivo: o autorado continua o documento).
- **Cortes de LOC** — `200f104a9a` (`joint_anchor_drag`, `joint_glyphs`) e `9e7b28957c` (20 ficheiros de `ph2d-app-macros`, 14 598 / 14 630), zero código.

## 3. ⛔ Catracas

Medido agora com `find crates/<c> -name '*.rs' | xargs cat | wc -l`:

- `ph2d-app-physics` **37 763 / 37 798**.
- `ph2d-app-components` **80 869 / 80 869** (no tecto, intocado).
- `ph2d-app-macros` **14 598 / 14 630**.
- shell (`shells/desktop`) **188 913 / 189 041**.

Gate `the_shell_only_shrinks_and_every_app_crate_too`.

## 4. ABERTO, por ordem

1. **Smoke do dono desta janela** — passos na resposta do smoke; fotos em `fotos_fisica/onda9/`.
2. **[OUTLINE_E_F3 §11.1–11.4](HANDOFF_INTEGRACAO_line_GameEngineFuncional02_2026-11-09_OUTLINE_E_F3.md)** — inalterados.
3. **Fecho da linha** (só por ordem do Enio): clippy/fmt 1× (`PH2D_SUITE=fecho`) das crates tocadas — `ph2d-app-macros`, `ph2d-shape-handles`, `ph2d-editor-core`, `ph2d-panel-inspector`, `ph2d-physics-ecs`, `ph2d-panel-authored`, `ph2d-signal-census`, `ph2d-outline-trace`, `ph2d-app-physics`, `ph2d-preview-drive`, shell; depois o handoff de integração e o §5 do CLAUDE.md. A suíte inteira é do integrador.

# HANDOFF DE INTEGRAÇÃO — `line/Macros-Automação`, W19 (a família «mundo»; fecha também W17 + W18) → `main` (2026-10-24)

> A linha fecha e PARA: integrar e shipar só por ordem do Enio, pelo integrador (DIRETRIZ §1.5.3).
> Pedido: [PROMPT_W19](PROMPT_W19_familia_mundo_2026-10-24.md). Plano (medições e números da sonda): [`17_W19_plano.md`](../17_W19_plano.md).
> ADR: [0190](../../architecture/decisions/0190-a-macro-e-um-objeto-e-leva-tudo-o-que-usa.md) (**número PROVISÓRIO**, §4).
> Anterior (W17 + W18, **ainda não integradas — entram nesta integração**): [handoff W18](HANDOFF_INTEGRACAO_line_Macros_2026-10-23_W18.md)
> (que por sua vez aponta o [handoff W17](HANDOFF_INTEGRACAO_line_Macros_2026-10-23_W17.md)).

## §0 — Superfície de colisão e base

- Base: merge-base `b48f8b9512` (main de 24/10; rebase limpo no início da W19). HEAD: `340b0c766d`.
  W19 = **19 commits** `be59b50b3c..340b0c766d`; por baixo vêm os commits de W17/W18 já rebaseados (ver handoff W18).
  Intervalo total: `git log --oneline b48f8b9512..HEAD`.
- Toca **fundacional** (Modo L, append-only):
  - `ph2d-ecs`: `scene::diff::linhas_que_mudam` (diferença por linha de dois `WorldSnapshot`).
  - `ph2d-macro`: formato **1.4** — família `"mundo"` (`exige "mundo"`), `Track::continua` (etiqueta 9, `exige "objeto.continua"`);
    o MINOR só sobe quando a característica é usada (gravações sem mundo continuam ao byte).
  - `ph2d-preview-drive`: `BlobDrive` (componentes inteiros que um motor põe e tira, pelo blob do registo).
  - `ph2d-physics` / `ph2d-physics-ecs`: `set_policy` do anel de pontos de retoma, `set_checkpoint_policy` (append-only).
  - `ph2d-editor-core`: `FamiliaDaMacro::Mundo`, `TipoDaEtapaView::{Componentes, Nasce, Apagado, Simula}`,
    `EtapasDaFamilia::pode_apagar` (por omissão `Ok`), `TranspoeDaFamilia::continua`, chaves novas.
  - `ph2d-i18n`: `macros_mundo.rs`, `macros_smoke8.rs` (novos), chaves em `macros.rs`/`macros_biblioteca.rs`, registo em `lib.rs`.
  - `ph2d-app-components`: `remap_object_refs` passa a `pub`.
- Famílias: `ph2d-app-macros` (módulo `mundo/`: `formato`, `grava`, `estado`, `portas`, `aplica`, `embute`, `palco`,
  `palco_fisica`; `etapas_mundo`, `cena_do_mundo`, `aplicar_continua`), `ph2d-app-painter`, `ph2d-app-motion`
  (`transpor_continua`), `ph2d-tool-painter` (`MacroEmCena` por trilha; portas `deactivate`/`forget_pending_layer` Ambiente),
  `ph2d-panel-macros`.
- Shell (ganchos): `undo_app_macros.rs` (`empurra` + `aplica_e_conta_as_macros`), `undo_app.rs`, `fase_physics_step.rs`,
  `fase_extract_inputs.rs`, `fase_macros.rs`, `macro_smoke.rs`.
- Contadores como DELTA: `CENAS_DO_SMOKE` 7 → **8**; `PROJECT_SCHEMA` **0 degraus** (a macro viaja no blob do componente
  `Macro`); ADR **0 novos** (o 0190 provisório da W18 continua a recontar); **165** portas em `mundo::portas::PORTAS`,
  com censo de dois lados na shell (`todo_componente_registado_tem_porta_no_mundo`); shell ≈ **188 967** linhas,
  `the_shell_only_shrinks` verde; `undo_app.rs` 600/600 (HR-18).
- Nenhum contrato §6 tocado.

## §1 — O que é a W19

Decisão 7 do dono (23/10, verbatim no plano §0): «a macro não será só imagem. Tudo na game engine pode criar macros
[…] a macro é um objeto completo e independente». A W19 fecha esse pedido: o artista pinta um player, põe um collider
e simula a física **dentro do objecto Macro**; a macro leva isso tudo e, aplicada noutro projecto, o boneco nasce, pinta-se
e cai no chão **desse** projecto. Também: várias imagens e vários Motion no mesmo objecto Macro (sai a recusa `one_per_family`).

Decisões do dono (24/10, delegou: «tome as melhores decisões sem olhar custos e siga»): **9** — a queda cai no chão do
projecto de DESTINO e no que vem na macro; **10** — as teclas do jogador durante «Simular» vão na macro (troço da fita).
Já em `project-memory/project_macro_library_owner_decisions_2026_10_23.md`.

Desenho (plano §2): o mundo grava-se pelo MESMO passo do undo (`linhas_que_mudam` filtrada à sub-árvore do objecto Macro,
ids locais, tabela de portas com censo, recursos embutidos); «Simular» grava-se como evento `Simula { tiques, conferência,
teclas, obstáculos }` lido da ponte, **nunca poses por quadro**; tocar = palco que conduz o mundo pelo preview (nada entra
no undo nem no save) e recoze o `Simula` num rapier privado com pontos de retoma.

## §2 — O que foi construído (por peça, com commits)

| Peça | Commit | Onde / lei |
|---|---|---|
| Plano (medição antes do código, kill-criterion escrito antes) | `be59b50b3c` | `17_W19_plano.md` |
| `linhas_que_mudam` (Nasce/Morre/Muda, junção linear, `Arc` iguais saltam por ponteiro) + formato 1.4 (`"mundo"`, `FAMILIA_MUNDO`) + decisões 9–10 | `75b14a4387` | `ph2d-ecs`, `ph2d-macro`; gates de bytes e `exige` |
| `Track::continua` (etiqueta 9, `exige "objeto.continua"`) — voltar a uma imagem já gravada continua a trilha, não faz nascer nada | `c08175afc1` | `ph2d-macro` |
| Várias imagens/Motion no mesmo objecto Macro: a sessão guarda `Trilhas` (abertas + a viva); trocar de objecto fecha/abre; sai `MACRO_RECUSA_UM_DE_CADA` | `e9ec7b752a` | gate `duas_imagens_no_objeto_macro_gravam_duas_trilhas_e_voltar_continua` |
| A reprodução toca TODAS as trilhas do Painter e do Motion de um objecto | `17594f88ee` | `ph2d-app-painter`, `ph2d-app-motion` (`transpor_continua`) |
| Etapas do mundo no vocabulário (`Componentes`, `Nasce`, `Apagado`, `Simula`; `pode_apagar`; chaves `world_step_*`; nomes) | `d08663434e` | `ph2d-editor-core`, `ph2d-i18n/macros_mundo.rs` |
| `BlobDrive` no preview | `2e36268e45` | `ph2d-preview-drive` |
| O palco do mundo: `S(t)` conduzido por blobs; `Simula` recozido num rapier privado | `955bad7698` | `mundo/palco*.rs` |
| Aplicar com a família «mundo»: o FIM nasce no destino num passo só (`deep_copy_subtree` + `remap_object_refs`) | `cc8a42c048` | `mundo/aplica.rs`; `aplicar::familia` exaustivo, família desconhecida recusada pelo nome |
| Gravar o mundo: sub-árvore a cada passo do undo e depois de cada desfazer, corridas da física pela ponte com a fita; `mundo::fecha` ao Parar (ids locais, quem faz nascer, portas, embutidos); estado num instante (`Lida`); 165 portas + censo; etapas no painel; aviso `panel.macros.refusal.world_not_recorded` | `47200101a4` | `mundo/{grava,estado,portas,embute}.rs`, ganchos da shell |
| Arestas do aplicar: apagados a gravar, continuações, pré-multiplicado | `8ac696bf60` | gates por aresta |
| Palco retoma de pontos (a cada 60 tiques, tecto 64 MiB), confere o bit, ouve as etapas | `200f30884a` | `palco_fisica::RETOMA`; `retomar_de_um_ponto_da_o_mesmo_bit_que_do_zero`, `o_scrub_para_tras_recoze_e_da_o_mesmo` |
| A sprite do atlas grava-se com os seus píxeis (nunca a célula crua noutro projecto) | `5c7089c039` | `mundo/embute.rs` |
| Cena `PH2D_MACRO_SMOKE=8` + sonda `a_cena_8_percorre_o_caminho_inteiro` + fotos | `b6cc572312` | `ph2d-i18n/macros_smoke8.rs`, `macro_smoke.rs`; fotos em [`smoke_2026-10-24_W19/`](smoke_2026-10-24_W19/) |
| Os 4 defeitos que a cena 8 achou no app (gates sem placa não viam): (a) o `Simula` partia-se com render > 60 Hz; (b) a pose do solver gravada como autoria; (c) o FIM no destino recozido ao aplicar; (d) a conferência só no mesmo chão (blake3 dos `Static` de fora); sair do Paint e voltar a gravar já não recusa | `00c7840e18` | gates `tres_quadros_de_render_por_tique_dao_um_simula`, `o_pause_nao_grava_a_pose_do_solver_como_autoria`, `aplicado_noutro_chao_o_fim_e_o_chao_novo`, `a_conferencia_so_se_faz_sobre_o_mesmo_chao`, `sair_do_paint_e_voltar_a_gravar_nao_recusa` |
| A pose do solver tira-se por TRAJECTÓRIA (a captura do undo chega um tique atrasada); a cena mostra o Inspector no passo 5; fotos novas | `467150c007` | — |
| Fecho: fmt · clippy (`Remapeia` type alias, `enumerate`, `then_some`, `if let`) · tecto HR-18 | `f165875791` · `63670625da` · `340b0c766d` | `undo_app.rs` 601 → 600 (o gancho passa a `undo_app_macros::aplica_e_conta_as_macros`); frase da cena 8 legível pelo censo dos avisos |

**Medição da reprodução (plano §3)**: 1.ª rodada (loadavg 150–245) recozer do zero = **58,9 ms** a 600 tiques/100 corpos ⇒
REPROVOU o kill-criterion (≤ 4 ms). Cura: pontos de retoma a cada 60 tiques ⇒ pior scrub com 100 corpos **1,75 ms** (2.ª
rodada, loadavg 4–6); conduzir o estado **0,110 ms/quadro** a 100 objectos (limite 1 ms). «Palco renderizado» recusado por LEI
(segunda fonte do que se pinta).

**Números da sonda da cena 8** (banca sem placa, 100 px/m): trilha mundo com 2 eventos (collider a `9,48 s`; um `Simula`
a `13,00 s`, tiques `0..100`); 1.º chão: assenta em `y = −1,7400 m`, recozido = conferência ao bit
`[1.6256938e-4, -1.7400224, 1.3265806e-7]`; 2.º chão (projecto limpo, mais abaixo): fim `−3,2400 m`; texels pintados
`4 871 / 16 384`. Painel no fim: 6 etapas (3 pinceladas, «Canvas appears», «RigidBody, Collider added», «Simulate · 2.48 s»).

## §3 — Resultados do portão de fecho

- `cargo fmt --check` ✓ (restam 4 ficheiros do `ph2d-gpu` que vêm do main — zero diff contra o merge-base).
- clippy `--all-targets -D warnings` nas crates da linha + host ✓.
- `nextest-impacted` (fecho): **19 757** corridos / 19 751 passaram / **3** falharam / **3** timeouts.
  - As **2 falhas reais**, curadas em `340b0c766d`: `file_loc_caps` (`undo_app.rs` 601 → 600) e o censo
    `toda_mensagem_de_aviso_da_tabela_cabe_inteira` (frase em `cena_da_biblioteca.rs`); verdes por nome.
  - Flakes de carga ([FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)), verdes sozinhos: `ph2d-host-code-scenes
    a_cena_3_conduzida_passo_a_passo` 3/3 (loadavg 12); `ph2d-app-components as_duas_barras_do_igual…` e `ph2d-app-flip` ×2
    (timeouts de 180 s, loadavg 6–12).
- Censos **121**: 120 + o `file_loc_caps` curado.

## §3b — Provas de mutação

| # | Mutação | Resultado |
|---|---|---|
| 1 | `linhas_que_mudam` compara só por `ptr_eq` | SANGRA |
| 2 | os componentes «tirados» saem do resultado | SANGRA |
| 3 | porta do `Collider` apagada | SANGRA — o censo + 2 gates de grava |
| 4 | `sub_arvore` aceita tudo | SANGRA |
| 5 | sessão sem `Continua` | SANGRA |
| 6 | codec sem `exige "mundo"` | SANGRA |
| 7 | família desconhecida → Motion | SANGRA |

7/7 sangram. Os agentes de onda provaram ainda: palco (8/8: Kinematic, `release_to_authored`, pose da física, recozer para
trás, obstáculos, componente ausente, `substitute`, não-vivo escondido); retoma/ordem/conferência/espera (4/4); aplicar do
mundo (7 mutações, cada gate vermelho em ≥ 1); arestas (vermelhos vistos); etapas (3/3).

## §4 — ABERTO / para o integrador

Recontagens (a fonte é o código):
- Integrar com **`PH2D_SUITE=fecho`**: `ph2d-ecs` e `ph2d-physics` tocados ⇒ suíte inteira.
- **ADR 0190 é PROVISÓRIO** (herdado da W18): reconte contra o `main` e renomeie ficheiro e links (plano, ADR, handoffs, `decisions/README.md`).
- `CENAS_DO_SMOKE` = 8 (`ph2d-app-macros/src/lib.rs`); se outra linha somou cenas, reconte.
- `PROJECT_SCHEMA`: 0 degraus — confirme com `python3 scripts/schema-recount.py`.
- i18n, chaves novas deste diff: `panel.macros.step.world.{added,appears,changed,deleted,more,on,parent,removed,simulate,unnamed}`,
  `panel.macros.error.world_{physics_differs,step_is_fixed,step_keeps_its_order}`, `panel.macros.refusal.world_{not_recorded,deleted_objects,two_notices}`, `app.macros.smoke8.{floor,step.1..11}`; saem
  `MACRO_RECUSA_UM_DE_CADA`/`one_per_family`. Lista exacta: `git diff b48f8b9512..HEAD -- crates/ph2d-i18n`.
  **Conflito provável** em `ph2d-i18n/src/lib.rs` (registo partilhado) e em `CENAS_DO_SMOKE`.

Limites conhecidos (por desenho ou por medir):
- 6 componentes com referência a objecto fora do censo `RefKind::Object` são recusados em voz alta: `Bone`, `IkGoal`,
  `Skin`, `NavLink`, `ProjectileMotion`, `Factory` — defeito PRÉ-EXISTENTE também das receitas/prefabs (uma cópia aponta o original).
- Recusados por serem dado do projecto fora do objecto: `Tags`, refs de vector (`VecPathRef`…), `LuauScript`,
  `AudioSource2D`, `SpriteSheetRef`, `FlipObjectRef`, prefab links.
- Imagens de 16 bits voltam a 8; pré-multiplicada volta em alfa recto (mesma imagem, não os bytes).
- Embutido = os píxeis do FIM (uma sprite que trocou de imagem a meio mostra a final nos passos do meio).
- Um objecto apagado a gravar não se mostra na reprodução (nomeado no aviso).
- Detalhe da conferência em m/graus (sem ppm no palco).
- Mais de 64 MiB de pontos (~8 min a 100 corpos) recoze do início.
- A cena 8 não tem «Ficheiro ▸ Novo» que se conduza: apaga e põe um chão novo (a frase do ecrã diz-o).
- O nome do objecto Macro na etapa diz «object» quando a trilha não leva `Name`.
- Herdados da W18 e ainda válidos: `Encaixe` escala só o raio do pincel; grupo/backdrop pré-existente no Motion é recusado;
  aviso de 16 bits sem teste de comportamento (GPU).

Um merge pode partir: `the_shell_only_shrinks` e `file_loc_caps` (`undo_app.rs` está em 600/600) se outra linha crescer a
shell ou o `undo_app.rs`; `todo_componente_registado_tem_porta_no_mundo` se outra linha registar um componente sem decisão
de porta (a cura é uma entrada em `mundo::portas::PORTAS`); o censo das portas do Painter; os índices de i18n.

## §5 — Smoke para o Enio (cena 8, ~1 min, sozinha)

1. No terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && PH2D_MACRO_SMOKE=8 ./target/smoke/ph2d-host-desktop`
   (se o binário não existir: `PH2D_MACRO_SMOKE=8 cargo run --profile smoke -p ph2d-host-desktop`).
2. Não mexa no rato por cerca de 1 minuto. A cena conduz sozinha e escreve no ecrã o que cada um dos onze passos é.
3. Tem de acontecer: o boneco pintado cai no chão e guarda-se; depois, num projecto limpo com o chão mais abaixo,
   «Criar novo» faz nascer o boneco, pinta-o, e ele cai no chão NOVO. No fim, o painel Macros lista 6 passos, incluindo «Simulate».
4. Deu errado se: o boneco pára no ar; cai no sítio do 1.º chão em vez do novo; aparece um aviso vermelho; ou o painel
   diz que algo não ficou gravado.
5. A cena 7 (a biblioteca) continua igual: `PH2D_MACRO_SMOKE=7` no mesmo comando.

## §6 — Smoke do dono

- **Por fazer** (cena `=8`).

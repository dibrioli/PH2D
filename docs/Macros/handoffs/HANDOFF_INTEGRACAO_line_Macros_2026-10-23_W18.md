# HANDOFF DE INTEGRAÇÃO — `line/Macros-Automação`, W18 (a biblioteca de macros; fecha também a W17) → `main` (2026-10-23)

> A linha fecha e PARA: integrar e shipar só por ordem do Enio, pelo integrador (DIRETRIZ §1.5.3).
> Pedido: [PROMPT_W18](PROMPT_W18_biblioteca_de_macros_2026-10-23.md). Plano: [`16_W18_plano.md`](../16_W18_plano.md).
> ADR: [0190](../../architecture/decisions/0190-a-macro-e-um-objeto-e-leva-tudo-o-que-usa.md) (**número PROVISÓRIO**, §4).
> Anterior (W17, fecha nesta): [handoff W17](HANDOFF_INTEGRACAO_line_Macros_2026-10-23_W17.md).

## §0 — Superfície de colisão e base

- Base: `972f411a6f` (main de 23/10). HEAD de código: `434f76db45` + `0033539aa6`·`8e1fabdf3d` (só comentário, §3b #1b).
  Handoff: `27278fa03a` (1.ª escrita); o fecho do §3/§3b vem no commit por cima.
  Intervalo: `git log --oneline 972f411a6f..HEAD` (≈ 33 commits, W17 + W18).
- Toca **fundacional**: `ph2d-ecs` (`ReataPorIdentidade`, `654dad577f`), `ph2d-macro` (formato, MINOR 3),
  `ph2d-editor-core` (`TranspoeDaFamilia`, `MacroRequest`, `MACRO_*`), `ph2d-i18n` (`macros.rs`,
  `macros_biblioteca.rs`), `ph2d-assets`/painel de Assets (`AssetKind::Macro`), `ph2d-sprite-screen`
  (`gravacao_no_alvo`), `ph2d-tool-painter`, `ph2d-app-painter`, `ph2d-app-motion`, `ph2d-app-macros`,
  `ph2d-panel-macros`, e a shell (`fase_macros.rs`, `project`/undo: chamadas ao `Reata`).
- Contadores como DELTA: shell **+43 linhas líquidas** (teto `the_shell_only_shrinks` 189 041, medido no fecho);
  `CENAS_DO_SMOKE` 6 → **7**; `AssetKind::ALL` → **3** (+1 chip, id `asset_browser.kind.3`);
  `PROJECT_SCHEMA` **0 degraus**; ADR **+1** (0190 provisório); formato da macro 1.2 → **1.3** (MINOR 3).
- Nenhum contrato §6 tocado (`grep ph2d_macro` em nodegraph/tool-registry/vector ⇒ vazio no fecho do plano).

## §1 — O que é a W18

O dono (23/10, decisões 1–8, verbatim no plano §0): a macro serve receita pessoal **e** ficheiro a enviar; leva o
objecto e a **sua criação** (autossuficiente); onde já há desenho escolhe-se na hora (novo / por cima); vive numa lista
no painel MACROS com «guardar como asset»; por cima de imagem de outro tamanho fica no **tamanho original, onde o
gizmo está** (a macro é um **objecto próprio** com pos/rot/escala, criado pelo «+» da Hierarchy, gizmo do vazio); as
pinceladas **entram na imagem de baixo**; «tudo pode ser macro» (o mundo: collider, física — **W19**); o Motion
«igual». Desenho em [ADR-0190](../../architecture/decisions/0190-a-macro-e-um-objeto-e-leva-tudo-o-que-usa.md) e
plano §2; recusas medidas no plano (fim).

## §2 — O que foi construído (por peça, com commits)

| Peça | Commit | Onde / lei |
|---|---|---|
| Plano, prompt, ADR | `ffacb3a18e` · `28a2310f06` · `02d8dcdccd` | `16_W18_plano.md`, ADR-0190 |
| Formato: ficha de nascimento `Track::nasce` (etiqueta 8), `exige "objeto"`, MINOR 3; ficheiro `.ph2dmacro` (modo + nome no topo) | `b0c5138d59` | `ph2d-macro`; gates de bytes congelados e ida-e-volta; uma gravação sem ficha continua 1.2 ao bit |
| Biblioteca em disco (guardar, listar, renomear, apagar, importar, exportar) | `9bc2dd94fc` | `ph2d_app_macros::biblioteca`; pasta `PH2D_MACROS_DIR` › XDG › `~/.local/share/ph2d/macros` |
| Objecto Macro: Add ▸ Macro (vazio + `Macro`, gizmo do vazio); Gravar arma o objecto e abre a trilha DENTRO dele; o que nasce a gravar fica filho; recusas nomeadas (fora, fundo demais, uma por família `one_per_family`, nada gravado) | `9bc2dd94fc` | `object_add`, `Familias::quem_grava` |
| `Macros` por omissão não tem disco (só `Macros::do_utilizador` resolve a pasta do dono, no 1.º gesto) | `d60a4e1c8b` | gate `o_macros_por_omissao_nao_le_a_pasta_do_utilizador` |
| Motion: `transpor::so_do_dono` (o ficheiro leva só o grafo do dono, ids remapeados), `aplica` (novo ou por cima; nós do dono intocados), `ErroAoTranspor` nomeado; Mapa da trilha em `transpor_mapa.rs` | `0cfac4aece` · `9d1ad04ffa` | 4 gates vermelhos contra stub |
| Painter por cima: `inicial::sobre_documento` (etiqueta 6), `Encaixe` afim px-gravado→px-alvo numa porta só, raio do pincel pela porta `authored_spec` (`raio_do_pincel`, também as 8 janelas sujas), `fim::documento_no_fim` partilha o laço de `imagens_da_gravacao` | `df480b776c` | 5 gates (2 mutações sangram) |
| Vocabulário no painel (`MacroDaBibliotecaView`, `OndeAplicar`, pedidos Guarda/Aplica/Renomeia/Apaga/Exporta/Importa/GuardaComoAsset), `por_cima_alvo` | `11f06a07b3` · `7592028b70` | `ph2d-editor-core` |
| Secção **Biblioteca** do painel MACROS (verbos numa tabela só; aviso Danger por convenção `.error.`/`.refusal.`) | `a4dddc0900` | 7 gates de costura (4 sangram) |
| `AssetKind::Macro` derivado de `receita_de_macro` (endereço continua `Component{stable_id}`), rótulo `asset.kind.macro`, 4.º chip | `b38a747dcd` | 2 gates (sangram sem a marca) |
| Encaixe vivo: `PalcosDoPainter::segue` lê o gizmo a cada quadro (`poe_encaixe`); lei na folha `ph2d-sprite-screen::gravacao_no_alvo`; `AtlasDaCena`, `Relogio` | `4aa9a1abdf` | gate `o_encaixe_vivo_segue_o_gizmo` |
| Porta de aplicar: `aplicar::aplica` + `Portas` (Novo e Por cima; misto/incompatível recusam pelo nome), `gestos.rs` (Guarda, Aplica, Renomeia, Apaga, Exporta/Importa por `ph2d_app_host::modal`, `GuardaComoAsset` = Name + Macro + MasterRoot); exportar prova cada trilha até ao fim e não escreve o que não toca; `TranspoeDaFamilia` na folha (editor-core); receita dos Assets aplica-se (duplo-clique, arrasto `PlacePrefab.over`) | `0d2b77d059` | 5 gates vermelhos sem o despacho |
| Aplicar escolhe o objecto nascido; «Por cima» sem escolha pede (`panel.macros.lib.apply_over_needs_target`); relógio das macros sai da shell | `4654dcb49f` | 2 gates |
| Aplicar = **UM passo de undo**: Motion `atados` (o undo global que tira o objecto Macro desfaz o do Motion no mesmo quadro; só no topo), `instala_nascido` (clipe sem passo próprio da Timeline) | `03c230207b` | gates `aplicar_novo_…` e `aplicar_por_cima_… e_desfaze_lo_num_passo` |
| A imagem de uma macro aplicada volta do ficheiro (carimbo `PaintedDoc` + target reescrito) | `fff114a797` | gate `a_imagem_de_uma_macro_aplicada_volta_do_ficheiro_…`; `documentos_no_projecto::{carimba_e_colhe, devolve}` saem da shell |
| **Undo/redo re-liga documentos pintados por `StableId`** — `ph2d_ecs::ReataPorIdentidade`, `documentos_no_projecto::Reata` (antes/depois/em_volta), `PainterTool::reata_documentos`; a shell chama `em_volta` à volta do `ProjectState::restore` | `5b7883ec13` · `654dad577f` | gate `desfazer_e_refazer_o_aplicar_devolve_a_pintura`. ⚠️ **Defeito PRÉ-EXISTENTE de toda imagem pintada** (o restauro respawna o mundo com bits novos e os documentos, chaveados por bits, ficavam órfãos), corrigido aqui, não só para macros |
| Objecto Macro vazio não se lê como estragado (`etapas::vistas` de bytes vazios = zero etapas) | `db6f880deb` | gate `o_objeto_macro_vazio_nao_se_le_como_estragado` |
| Aviso de 16 bits: aplicar Painter por cima de imagem a 16 bits diz que pinta a 8 (`warn_precision_loss`, `app.macros.apply.eight_bit_copy`) | `992a4b0d08` | `every_pixel_consumer_declares_what_it_costs`; **sem teste de comportamento** (GPU) |
| Cena `PH2D_MACRO_SMOKE=7` (`ph2d_app_macros::cena_da_biblioteca`) + sonda `a_cena_7_percorre_o_caminho_inteiro` | `aa5f84b58c` | §5 e plano §6 |
| Fecho: fmt/clippy/tectos | `eeb58eebd0` · `16d769aa07` · `9d1ad04ffa` · `434f76db45` | `project_tests.rs`/`main.rs` no HR-18; transpor 719→305(+`transpor_mapa` 427), caminho 707→516(+`caminho_diagnostico` 193, `Espia` re-exportado) |

**Números medidos da cena =7** (Digital, 100 px/m; sonda = o mesmo roteiro): passo 9 — imagem `256×256`, **5 918**
texels, eixo `0,01°`, comprimento `168,0 px`; passo 10 — imagem `512×256`, objecto Macro rodado 30°, **5 892** texels,
eixo `−29,8°` (y para baixo), `168,6 px`. Fotos: [`smoke_2026-10-23_W18/`](smoke_2026-10-23_W18/) (passos 1, 4, 5, 9, 10).

W17 (fecho): os 2 defeitos do relato do dono e o 3.º smoke estão no [handoff W17](HANDOFF_INTEGRACAO_line_Macros_2026-10-23_W17.md)
(`d25c4262b1`, `9c1c53896f`, `fb45c93aec`, `d0ac6a70d5`, `951afc09fe`, `70eae59ede`).

## §3 — Resultados do portão de fecho

- `cargo fmt --check` ✓ · clippy `--all-targets -D warnings` nas crates da linha ✓ · censos **121/121** ✓.
- Suíte impactada: **19 094** corridos / **19 059** passaram / **4** falharam (**todos curados**) / **31** timeouts.
  - Os 4: censo das portas do Painter `toda_porta_do_painter_tem_decisao` → `197a693b9d` (`reata_documentos` declarado
    Ambiente); escada de camadas (dev família→família) → `654dad577f`; `every_stack_of_rows_asks_the_rhythm` →
    `03c7c20976`; `every_pixel_consumer_declares_what_it_costs` → `992a4b0d08`.
  - Os 31 timeouts = **flakes de carga** (loadavg 145–305, ver [FLAKES_DE_CARGA](../../DevOps/FLAKES_DE_CARGA.md)); todos
    passaram sozinhos (loadavg 19–305). Por crate: `ph2d-app-components` 1 · Flip 2 · `ph2d-panel-painter-layers` 4 ·
    `ph2d-tool-painter` 8 · Luz 2D 16.
- Tectos de LOC: `transpor.rs` 719→305, `caminho.rs` 707→516 (cura por corte, nenhuma isenção). Shell +43 vs teto 189 041.
- **Provas de mutação: ver §3b.**
- **Suíte final após `654dad577f` (toca `ph2d-ecs`)** (`BASE=654dad577f~1`, loadavg ~145–210): **19 945** corridos /
  **19 938** passaram / **1** falhou / **6** timeouts — **todos flakes de carga, verdes sozinhos** (loadavg 26–44):
  `ph2d-host-code-scenes a_cena_3_conduzida_passo_a_passo` (`Wait(ListHas("set"))` não chegou em 30 s; sozinho 15 s ✓);
  timeouts `ph2d-app-components` 1, Flip 2, Luz 2D 2, `ph2d-panel-painter-layers` 1 (os mesmos da lista acima).
  Depois: `cargo fmt --all --check` ✓ · clippy `--all-targets -D warnings` (`ph2d-ecs`, `ph2d-sprite-screen`,
  `ph2d-app-macros`, `ph2d-app-motion`, `ph2d-app-painter`, `ph2d-panel-macros`, `ph2d-panel-timeline`,
  `ph2d-host-desktop`) ✓.

## §3b — Provas de mutação

| # | Onda | Mutação | Resultado |
|---|---|---|---|
| 1 | W17 | `largada_de` → sempre `None` | SANGRA |
| 1b | W17 | tirar `.filter(\|l\| e.fim == l.antes)` de `largada_de` | SOBREVIVE — **2.ª guarda, sem quadro que a veja**: o `paint` chama `envelhece_a_largada` (via `interact::process`) ANTES de ler, com o MESMO snapshot, e ele limpa a largada no quadro em que a edição chega. Filtro mantido (a função vale sozinha); o comentário do teste deixou de afirmar que esta mutação sangra (`0033539aa6`) |
| 2a / 2b | W17 | `conduz_o_ponteiro` (as duas) | SANGRAM |
| 3 | W17 | `troca_campos` | SANGRA |
| 3b | W17 | tirar `stroke_mask` de `campos_do_traco!` | SANGRA — `cada_campo_do_paint_state_e_do_traco_ou_partilhado` vermelho (`tracos_tests.rs:26`, `t ^ p`); reposto, verde |
| 4 | W17 | `ColorDropAberto` removido | SANGRA |
| 5 | W18 | `exige "objeto"` | SANGRA |
| 6a / 6b | W18 | `no_cru` / `e_do_dono` | SANGRAM (cada uma vista por UM teste só) |
| 7 | W18 | exportar salta o «tocar até ao fim» | SANGRA |
| 8a / 8b / 8c | W18 | encaixe: escala do pincel, matriz transposta | SANGRAM |
| 9 | W18 | `dentro_de` | SANGRA |
| 10 | W18 | `escreve_atomico` (escrita directa) | SOBREVIVE por desenho — só um crash a meio da escrita o mostra |

## §4 — ABERTO / para o integrador

Recontagens (a fonte é o código):
- **ADR 0190 é PROVISÓRIO**: contado contra o máximo `0189` do `main` e das worktrees; reconte, renomeie o ficheiro e
  os links (plano, ADR, este handoff, `decisions/README.md`).
- `CENAS_DO_SMOKE` = 7 (`ph2d-app-macros/src/lib.rs`); se outra linha somou cenas, reconte.
- i18n chaves novas: `panel.macros.*`, `panel.macros.lib.*`, `app.macros.*` (inclui `apply.eight_bit_copy`),
  `asset.kind.macro`, `object_add.macro`; conflito provável em `ph2d-i18n` (ficheiros `macros.rs`/`macros_biblioteca.rs`
  são nossos; o registo em `lib.rs` é partilhado).
- `AssetKind::ALL` agora 3 (+ chip `asset_browser.kind.3`); outra linha que some tipos de asset reconta.
- `PROJECT_SCHEMA`: 0 degraus (`python3 scripts/schema-recount.py` deve confirmar).
- `ph2d-ecs` tocado (`ReataPorIdentidade`) ⇒ **suíte inteira na integração** (`PH2D_SUITE=fecho`).

Limites conhecidos (por desenho ou por medir):
- Uma imagem + um Motion por objecto Macro (`panel.macros.refusal.one_per_family`).
- A família **«mundo»** (collider, física — decisão 7 do dono) é a **W19**: a porta declara-se recusada, o painel diz o
  que não ficou gravado; precisa de plano próprio (palco de um objecto que ainda não nasceu).
- O `Encaixe` escala só o **raio** do pincel; ângulo do dab, inclinação e ajustes em px não seguem rotação/escala.
- Um grupo/backdrop pré-existente numa gravação do Motion é recusado (`ErroAoTranspor::VistaDeAntes`).
- O aviso de 16 bits não tem teste de comportamento (precisa de GPU).
- Sem `README.md` de índice em `docs/Macros/handoffs/` (a pasta não está no `DIRS` de `scripts/doc-index.sh`); este
  handoff é citado pelo `CLAUDE.md §5`.
- A memória `project_macro_library_owner_decisions_2026_10_23.md` entra por este commit com as decisões 5–8.

Um merge pode partir: gates de LOC da shell (`the_shell_only_shrinks`) se outra linha crescer a shell; o censo de portas
do Painter (`toda_porta_do_painter_tem_decisao`) se alguém acrescentar uma porta sem decisão; os índices de i18n.

## §5 — Smoke para o Enio (cena 7, ~1 min, sozinha)

1. No terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && PH2D_MACRO_SMOKE=7 cargo run --profile smoke -p ph2d-host-desktop`
2. Não mexa no rato: a cena conduz sozinha e escreve no ecrã o que cada passo é. Passos que vai ver: **+ ▸ Macro** na
   Hierarchy → **Gravar** → uma imagem nova dentro do objecto Macro → duas pinceladas (vermelha, azul) → **Parar** →
   **Guardar na biblioteca** → **Exportar…** → apagar o objecto → **Importar…** → **Aplicar ▸ Criar novo** (a imagem
   nasce e as pinceladas tocam) → uma imagem maior e **Aplicar ▸ Por cima**, com o objecto Macro rodado 30°.
3. Tem de acontecer: no passo «Criar novo» as duas pinceladas aparecem numa imagem nova; no «Por cima» elas
   aparecem na imagem maior **no mesmo tamanho**, inclinadas, à esquerda do centro (onde o objecto Macro está), sem
   esticar.
4. Deu errado se: a lista da biblioteca não mostra a macro guardada/importada; uma mensagem vermelha «This recording
   cannot be read.» (era o defeito do objecto vazio, curado); as pinceladas esticadas à imagem maior; ou Ctrl+Z após
   aplicar deixar a imagem sem pintura.
5. A biblioteca de teste fica em `$TMPDIR/ph2d-macro-smoke-7/` (a sua nunca é tocada, salvo `PH2D_MACROS_DIR`).

## §6 — Smoke do dono

- **24/10: APROVADO** (cena `=7`). Próxima onda: [PROMPT_W19](PROMPT_W19_familia_mundo_2026-10-24.md) — a família «mundo».

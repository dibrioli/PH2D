# Prompt de continuação — linha PainterWatercolor · 12/11: resolver o desempenho do módulo (ordem do dono)

`/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-12_desempenho_total.md`

Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor`, branch `line/PainterWatercolor`.
`cd` + `pwd` + `git branch --show-current` ANTES de ler. ⛔ NÃO faça `git rebase main` a meio: há agentes a comitar.

## Ordem do dono (11–12/11, verbatim no essencial)

«Se esforce para resolver todos os problemas de performance do módulo a todo custo. Se necessário, auditoria
exaustiva. Não esqueça do handoff de integração ao final de tudo. Não integre. O integrador será estabelecido por mim.
Vc está só. Não pare até terminar.» Auditoria com esforço `xhigh`.

## Estado (ler o `git log main..HEAD` — cada commit diz o quê, a medição e os testes)

- Feito nesta onda: `a0148c712e` (toque Impasto 4K 19,9 → 12,0, `bytemuck::zeroed_vec`) · `a95be6612d` (Ctrl+Z não
  confinado só re-sobe as camadas cujo Arc mudou). Medição: `docs/Painter/missao_4k/medicoes/REGRESSOES_TOQUE_E_UNDO_2026-11-11.md`
  (⛔ as duas armadilhas da régua: A×B só no MESMO target; Filbert 4K bimodal no tempo — matriz n≥4 intercalada).
- Agentes lançados em 11–12/11 (procure os commits deles; o que não estiver comitado, refaça):
  A — envio sem cópia dos planos da luz (fold direto num `Buffer` mapeado + `copy_buffer_to_texture`; ~14 ms de
  `memcpy` no fio da UI em cada dobra da tela inteira: toque e Ctrl+Z); B — toque/Ctrl+Z do Wet (FotoDoMotor por tile
  em COW, `bring_home`, preparo, grade inteira do `ph2d-wet-gpu`; bissectar o +5,4 do toque Wet); C — B23.2 (`Poll` →
  `Wait`, CPU parada 48–70 % → ≤ 5 %). B mediu e não curou (12/11): 1.º toque Wet 4K = o motor nasce NA HORA
  (20–38 ms) porque o `wetpaint_tick`/`prepara_o_toque` nunca corre antes do 1.º gesto (a ferramenta não recebe
  `on_tick`, `shells/desktop/src/render_loop/fase_fixed_step_clocks.rs:58`); Ctrl+Z Wet = ~10 ms de memset de
  `flow_x`/`flow_y` (`ph2d-wet-paint/src/grid/snapshot.rs` `Registo::restaura`) que `FotoDoMotor::repoe` reescreve,
  + `poe` dos 126 tiles 8,8 ms; o +5,4 do toque Wet é `47ec5ad159` (o plano do papel = o agente A). B2 implementa as
  duas curas no branch `tmp/painter-wet-1112` da worktree `/home/enio/Documentos/Projetos/.ph2d-bis-4k` → cherry-pick.
- **Estado às ~01:30 de 12/11 (nenhum agente em voo):**
  1. **Envio sem cópia (A): POR COMITAR na árvore da linha** — `crates/ph2d-render/src/{impasto_light.rs,lib.rs}`,
     `crates/ph2d-render/tests/it/impasto_light_gpu.rs`, `crates/ph2d-tool-painter/src/{lib.rs,tool/mod.rs}`,
     `.../tool/paint/{impasto_gpu.rs,impasto_gpu_reuso_tests.rs}`, `crates/ph2d-app-painter/src/painter_gpu_preview.rs`,
     `crates/ph2d-app-painter/tests/the_impasto_fold_walks_the_dirty_rect.rs`. Desenho: reserva de 2 buffers
     `MAP_WRITE|COPY_SRC` por plano (o wgpu 30 zera um `mapped_at_creation` novo), dobra paralela por faixas de 16
     linhas direto na vista mapeada (`PlanosEmEnvio::faixa`, `dobrar_faixas`), `copy_buffer_to_texture` no encoder da
     luz; `impasto_gpu_planes_reusing` saiu; `planos_de_tamanho` saiu. Paridade ao bit provada (9 testes tool, 6
     render lib, 5 GPU na Radeon incl. `o_envio_sem_copia_desenha_o_que_o_write_texture_desenhava` + mutação).
     Medição SUJA (carga 6–41): Filbert toque 74,2→65,2 · Ctrl+Z 62,9→54,2; Wet toque 62,7→48,5. Binários
     `bin_envA`/`bin_envB` no scratch `/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/fb53f5ce-2f42-4773-b6f1-219f6399f510/scratchpad/`.
     `the_shell_folds_the_window_the_tool_reports_dirty` já reprovava no HEAD (procura `let plane_win`): curar.
  2. **Curas do Wet (B2): no branch `tmp/painter-wet-1112`** da worktree `.ph2d-bis-4k` — `dd4d9207b6` (Ctrl+Z sem o
     memset do fluxo), `ca6bf04bf1` (o preparo arranca ao ligar a tela/mudar modo ou meio), `04ca713600` (testes;
     todos os de undo verdes). Medição SUJA (carga 9–47) — binários `bin_a95be6612d`/`bin_ca6bf04bf1` no scratch.
     Falta: controlo do teste do preparo; a causa de o Painter não receber `on_tick` antes do 1.º toque (shell);
     `poe` dos 126 tiles (8,8 ms).
  3. **Laço a pedido: feito** — `d16298c35f`, `50872fe70b`, `709c9ee59d` (B23.2 no BACKLOG). Aberto: o nome do cartão
     solto que pisca (`ph2d-editor-core/src/panel/cards_solto.rs:93`), a câmara da apresentação do Quadro
     (`screens/hero/board_team_show.rs:73`), a rolagem de borda (`panel/cards_rolagem.rs:18`), queda/fade dos cartões
     (`cards_lista.rs:165`, `cards_solto_some.rs:30`) e o fmt de `card_loose_ops.rs` + `a_frame_request_has_a_listener.rs`.
  4. **PRÓXIMO PASSO (sozinho na máquina, nada a compilar em paralelo):** UMA matriz calma intercalada n≥5, binários
     do MESMO target construídos em série: base `a95be6612d` · +envio · +envio+Wet (cherry-pick dos 3 commits),
     cenas Wet/Filbert/Impasto 4K (+ Filbert 1080p); alarmes de pincel antes×depois (mesma lista de vermelhos);
     comitar o envio se ganhar ≥ 3 ms e o Wet se ≥ 1 ms; gravar a tabela em `REGRESSOES_TOQUE_E_UNDO_2026-11-11.md`.
- Censo dos abertos (auditoria xhigh 12/11): os instantes que o artista sente a 4K, por ordem de ataque:
  B14.3+W5/W9 (cerdas grandes: soltar 1,34 s, traço p95 190 ms) · B28.3/B28.1 (Color mixing largo: traço p95 400 ms;
  falta o carimbo, `Ladrilhos::le`/`Sombra::le`) · B11.3 + B13.5–13.7 (água Wet com Óleo Real 34 → 3,6–5 img/s) ·
  B19.10 (custo por camada no compositor, 16 camadas UHD ~126 ms) · B18.1–18.3 (salvar/duplicar/trocar camada) ·
  B20.6/B20.11 (máscara, formigas) · B21.1 (Mailbox, medir) · B27.1 (fila da caneta) · B42.3/B17.9 (memória do undo).
  Do DONO (não decidir): B14.2 (luz cheia no gesto: soltar 34 → 19), B13.3 (sono da água), B43.6 (cgu1).

## Regras

- RTX proibida; pesado por `bash scripts/ph2d-run.sh`; testes por NOME; curas sem mudar pixel (alarmes de pincel
  vermelhos por desenho — nunca regravar; compare a lista de vermelhos antes × depois).
- Medir em janela calma (load < 5) e intercalado; dois agentes a compilar enquanto um mede invalidam a régua.
- Fecho: fmt/clippy das crates tocadas 1× (`PH2D_SUITE=fecho`), `the_shell_only_shrinks`/tectos de LOC, handoff de
  integração `docs/Painter/handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-12_<TEMA>.md` + link no §5.1
  do Painter no CLAUDE.md, `rm -rf target/*/incremental`, binário smoke compilado 2× (2.ª saída no handoff),
  `git worktree remove /home/enio/Documentos/Projetos/.ph2d-bis-4k` e `rm -rf /home/enio/Documentos/Projetos/.ph2d-bis-4k-target`.
- Resposta ao dono: curta, PT-BR, sem jargão, smoke em passos numerados.

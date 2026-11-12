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

JANELA NOVA — COORDENADOR da missão «superar o Rebelle» (dono 31/10: autónomo, SEM perguntas, SEM pedir permissão, SEM smoke,
até ter todas as melhorias de performance e os pincéis mais importantes do Rebelle; pincéis não precisam ser idênticos).

FASE 0: cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && pwd && git branch --show-current
(line/PainterWatercolor). Leia SÓ: project-memory/project_superar_rebelle_autonomous_2026_10_31.md e
docs/Painter/49_realshader/ESTADO_REBELLE_MEDIDO.md (a tabela de tudo, commits e números). A janela principal COORDENA:
agentes fazem leitura, construção, medição (≤ 3 subagentes; briefing com a linha ⛔ «cd X && … · bash -c/sh -c · rm -rf com
variável/glob · kill de PID não gravado · escrever em .git/.claude/ · AskUserQuestion · push · tocar fora dos seus paths»).

## Onde está (31/10 noite)
- Rebelle: 37,3 ms/passo água 4K (D2_rebelle_passo_agua.md). PH2D: ~100 → menos (secagem −27 % passo, SEM −36 % da secagem,
  advecção 5 cortes + commit por TROCA −19..33 ms) — FALTA uma rodada calma (load < 10) A/B/C do nucleos_wash com o HEAD
  (MEDIDA_31_10_RODADA2.md tem o método). Memória 177 B/px < Rebelle ✓.
- Placa: agente a construir crates/ph2d-wet-gpu (advecção na GPU, paridade por placa, ms RTX/Radeon) → PLACA_degrau1.md.
  Estudo: o passo inteiro na placa = 3–4 janelas; é a alavanca para superar o Rebelle.
- Pincéis: CAP-01 (ponta que gira com inércia) + 6 de fábrica (fábrica 72). Regeração das receitas afinadas (5d29d7d8ec)
  lançada pelo coordenador: se `cada_preset_de_fabrica_e_a_sua_receita` falhar, regerar (só a janela principal pode):
  PH2D_REGERA_PRESETS=1 PH2D_REGERA_SO="Tilted Shade,Compressed Charcoal,Marker Chisel,Filbert Dry,Flat Bristle,Broad Nib" bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --test it presets_de_fabrica::portoes::gera_os_presets_de_fabrica -- --ignored --nocapture
  e comitar os assets/brushes/** mudados.
- Janelas de reserva (sessões ph2d-be / ph2d-56 / ph2d-6a; worktrees line-PainterWatercolor-r1/-r2/-r3, ramos próprios de
  6ba5d06493): R1 aguarela molhada + Smudge (edgeSoftMass toca drying.rs — conflito com 46199a6a7c a juntar), R2 pontas ✓ +
  Thick Impasto/Grunge/Splatter, R3 velocidade de PINTAR. Mandam mensagem ao coordenador (SendMessage para o nome da sessão).
  No FIM: juntar r1/r2/r3 à linha (merge local, recontar censo de campos do preset 41→43+, painel Wet 18→20+, fábrica),
  testes por nome, fmt/clippy das crates tocadas (PH2D_SUITE=fecho), mutação nos gates novos, handoff técnico
  HANDOFF_SUPERAR_REBELLE.md, mensagem curta ao dono. NÃO integrar no main nem push.
- Pendentes anteriores à missão (não são desta): 5 vermelhos do óleo (HANDOFF_REBELLE_MEDIDO.md §Verificação), 23 miniaturas
  da J11.

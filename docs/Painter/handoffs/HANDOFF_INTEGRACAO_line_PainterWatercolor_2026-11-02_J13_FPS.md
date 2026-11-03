# HANDOFF DE INTEGRAÇÃO — line/PainterWatercolor, janela 13 (02/11): a óptica do Real Oil sai do fio da tela

Ramo `line/PainterWatercolor`, HEAD `126943905d`, merge-base com o main `3df5894489` (= o main de agora; 0 commits do
main por trazer). A linha leva **347 commits** sobre o main: os da rodada de 02/11 (handoff anterior,
[VERMELHOS](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-02_VERMELHOS.md), que continua a valer para tudo até
`8a3c88cbc6`) + **19 desta janela** (`8a3c88cbc6..126943905d`). Nada integrado nem empurrado.

## 1. O que esta janela muda (detalhe e números: [J13](../Arquitetura/J13_fps_do_real_oil.md), plano [J13b](../Arquitetura/J13b_plano_otica_fora.md))

Medido no app (1024², Radeon, a cena nova `PH2D_WETPAINT_SMOKE=3`): com Real Oil na tela e a água a correr sobre o
óleo, o quadro parado ia a 72–82 ms (12 fps) porque a composição da óptica do óleo (40–60 ms) corria no fio da tela
a cada passo da água. Cura estrutural, ao byte:

| commit | passo |
|---|---|
| `6f3dac38c7` | a cena do FPS `PH2D_WETPAINT_SMOKE=3` (`ph2d-app-painter/src/wetpaint_fps.rs`; `PH2D_FPS_OLEO`, `PH2D_FPS_TELA`) |
| `7718d06194` | sonda `diag_j13_fps_real_oil_o_que_muda` (ignorada) |
| `f89483646a` | P0 — a óptica é a função pura `compoe_a_otica` (`wetpaint/composite_otica.rs`) |
| `2f0d2a8ef2` | P1 — `worker_loop`/`spawn` em `offthread_laco.rs` |
| `7135589087` | P2 — `Engine::espelho_sujo` + o `Espelho` das grades (`ph2d-wet-paint/src/painter/espelho.rs`) |
| `637e68a9ad` | P3 — `FactosDaOtica` (Copy, Send) |
| `000a57a923` | P4 — porta síncrona `cfg(test)` + portão `a_otica_pelo_espelho_e_a_tela_de_casa_ao_byte` |
| `44138ff9fa` | P5 — a thread do compositor (`offthread_otica.rs`) |
| `6b27b67d4e` | P6 — o tique pede e aplica a imagem (`otica_no_tique.rs`, `veu.rs`) |
| `f4f1959c29` | P7 — **ligado por omissão**; `PH2D_WET_OTICA_FORA=0` volta ao caminho de casa |
| `cbff1ee20a` | P8 — no pen-down a água herdada vai pela dívida do compositor |
| `124ea2bcd5` | o controlo de `the_frame_does_not_run_a_sim_stage` procurava a palavra num comentário (mudo) |
| `6502042a03`, `126943905d` | `cargo fmt` (a cura `a4b69b428c` do integrador veio sem formatar) e clippy `-D warnings` zerado |

Resultado no app: quadro parado **16,5–17,0 ms em todas as rodadas** (contra 72–82); a água a 40 → 12 imagens/s conforme
a carga, a simulação nos 40 passos/s. Recusada e medida (ramo `exp/j13-luz-guardada`, **não apague**): a cache da luz por
célula com limiar — mais lenta em todo ε.

## 2. Fecho (1× sobre o diff da janela, por nome; máquina com load 13–17)

fmt ✓ e clippy `-p ph2d-tool-painter -p ph2d-wet-paint -p ph2d-app-painter --all-targets --profile smoke -D warnings` ✓ ·
`architecture_workspace_file_loc_cap` ✓ · `the_shell_only_shrinks` ✓ · `architecture_claude_md_cabe_no_orcamento` ✓ ·
`family_tests` ✓ · wet-paint `espelho` 1 · `oleo` 69 · `snapshot` 1 ✓ · tool-painter `otica` 9 · `offthread` 13 ·
`por_quadro` 16 · `showwet` 2 · `leak` 4 · `ritmo` 1 · `undo_tinta_viva` 9 · `smooth_edges` 15 · `oleo_cache_tests` 2 ·
`pen_down` 3 · `the_frame_does_not_run_a_sim_stage` 1 ✓. Portões novos com mutação vermelha: o espelho ao bit (3),
a óptica pelo espelho ao byte (3), o compositor ao byte (2), o tique (2), o pen-down (2).

## 3. ⛔ Vermelhos CONHECIDOS — BLOQUEIAM a integração (o CI do main reprova; os mesmos 4 do handoff anterior, nenhum novo — não afrouxe)

Dono 02/11: «os vermelhos não impedem a integração?» — impedem. **Ordem do dono: o INTEGRADOR cura os 4 e integra** (a
linha não os cura). Nenhum se afrouxa nem se ignora; cada cura vai com A/B no commit pai e, onde há lei, mutação.

| # | teste | cura pedida |
|---|---|---|
| 1 | `macro_track::roteiro::tests::o_splash_tocado_pinta_a_tela_e_o_inicio_e_branco` (`ph2d-tool-painter/src/macro_track/roteiro.rs:~691`) | o dono decidiu (02/11) que a cena macro 1 (Splash) FICA como está: o teste cobra o comportamento de antes. Leia o assert que reprova, confirme que o que a cena faz hoje é o que o dono aprovou no smoke, e mude o assert para a lei de hoje com tecto MEDIDO (não «qualquer coisa passa»). |
| 2 | `a_largura_e_a_prometida` (lento, ~20–25 min; só Smear Wet r7: 17,0 para 14) | `sonda_monotonia_da_largura` com `PH2D_SONDA_PRESET="Smear Wet"`; alinhar `raio_do_motor_da_tabela` (`tests/it/presets_de_fabrica.rs`) com `de_tabela`; a régua do gerador `largura_do_rasto` tem defeito a r < 1; regerar a tabela do Smear Wet com `PH2D_MEM_MAX=64G`, um preset de cada vez. |
| 3 | `transparencia::o_modo_blender_casa_com_o_cycles_e_o_clear_glass_e_o_de_antes` | nosso L÷papel 0,152 × Cycles 0,124 (a 0,5) e 0,115 × 0,076 (a 1) na malha real; nas rampas sintéticas casa. Próximo passo: malha 2D sintética no Cycles (oráculo que se CORRE, CLAUDE §0.9) para separar a estrutura 2D fechada (TIR / luz que escapa na borda); curar a LEI, nunca a régua. |
| 4 | `transparencia::a_transmissao_e_o_relevo_do_papel_mostram_o_grao` | Paper Relief 1 lê 0,490 contra a âncora 0,67 (faixa 0,55..0,8), calibração velha irreprodutível (antes de `ca26a7738a`). Medir de novo a âncora pela mesma régua sobre a tela de hoje e decidir pela medição (o piso do espalhar tira 0,026: sem ele 0,516); não escolher o número. |

Detalhe e história de cada um: §5 do [handoff anterior](HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-11-02_VERMELHOS.md).

## 4. Smoke do dono (02/11)

«FPS parece ter melhorado mas chega a 35 FPS, ainda longe dos 60» — no traço À MÃO. A cena automática mede 60 parado
e no traço a 250 eventos/s; o rato do dono manda mais. **Aberto, próxima janela:** medir a cena a 1000 eventos/s
(`HZ` em `wetpaint_fps.rs`) — o J11 mediu o ponteiro do Dabber com Real Oil a 4,5 ms por EVENTO — e o quadro do
pen-down (30–64 ms uma vez por traço). Outros abertos: J13 §6.

## Para o integrador

- **Curar os 4 vermelhos do §3 é seu (ordem do dono 02/11), antes de integrar.**
- A suíte completa e o `nextest-impacted` são seus (a linha só correu por nome).
- **Não apague** `exp/j13-luz-guardada`, `integ/2026-11-02-pintura`, `-pintura2`, `-com-painter`,
  `backup/line-PainterWatercolor-pre-rebase-2026-11-02`.
- O compositor sobe uma thread por sessão de Wet Paint ao relógio (nunca em relógio fixo/macros): os portões de bit
  com `set_wet_relogio_fixo` ficam no caminho de casa. Se algum teste de app/shell com Wet Paint ao relógio divergir,
  A/B com `PH2D_WET_OTICA_FORA=0`.
- Gerador das tabelas de largura: `PH2D_MEM_MAX=64G`.
- O smoke é do Enio; integrar e shipar só por ordem dele.

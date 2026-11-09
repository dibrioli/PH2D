/pd-linha-assumir PainterWatercolor docs/Painter/handoffs/PROMPT_CONTINUACAO_2026-11-09_pastel_K1.md

Estado (09/11, noite): linha `line/PainterWatercolor`, base `main` `f254e42bf1`, por integrar (12 commits). O anterior
(selo da prévia, smoke OK) está em `PROMPT_CONTINUACAO_2026-11-09_pastel_motor.md`.

FEITO nesta janela (`8d9aca8d7a`): o Height ganhou `softness` (grão de borda macia) e `tip_depth` (borda quebrada pelo
papel) em `ph2d-painter-brush/src/texture/lei.rs`, gravados em `wetpaint_preset/json.rs` só quando ligados (neutros ao
bit; nenhum pincel de fábrica usa o Height). Rodadas 3–5 contra o Krita Chalk Soft: LEIA a «FASE 3» de
`docs/Painter/handoffs/material_2026-11-09_pastel/README.md` (tabela, receita **K1**, recusas medidas) e veja
`folha_motor.png`. Dados crus: scratch da sessão (cena/cena_volta + krita_* em
`/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/4295e5e0-a183-452a-b0e5-03297d83bace/scratchpad`).

ESPERA O DONO: a triagem da folha (K1 serve / quase / não) e se quer o enchimento do dente além do Krita.

PRÓXIMO, num bloco, depois do «serve» do dono:
1. Receita K1 no Pastel de `ph2d-tool-painter/tests/it/familias/csp_secos.rs` (ver README FASE 3; o Crayon fica com o
   `PAPEL_ASPERO` dele). Regerar o preset (`PH2D_REGERA_PRESETS=1`, `presets_de_fabrica.rs:9-14`).
2. Painel: duas linhas novas no FIM de `LINHAS_DA_PONTA` (`tool/paint/ponta.rs`; índices 4/5/14/15 estão escritos à mão
   noutros sítios — não inserir no meio): `Softness` («Grain softness», num 0..1 passo 0,01) e `TipContact` («Tip
   contact»: Flat · Rounded). Sítios: enum, array 26→28, `tabela`, `opcoes`, `escreve` (tem `_ => {}` — sem o braço é
   no-op MUDO), `le` (o `dual =>` apanha tudo: o braço vem antes), `ids/painter_ponta.rs` (28), `ph2d-panel-painter-layers/
   src/ponta_linhas.rs` (`ROTULOS_DA_PONTA`, `decode_tip_contact_option`, `visivel` só no modo Height = `ponta[1] == 4`),
   `event/option_route.rs`, `ph2d-i18n/src/painter_ponta.rs`; teste de costura em `tests/it/seam_ponta.rs`.
3. Alarmes de pincel por nome (`a_largura_de_fabrica_e_a_aprovada`, `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`):
   vão reprovar no Pastel — relatório ao smoke, regravar SÓ com `PH2D_PINCEIS_APROVA=1` depois do sim do dono.
4. Mostrar ao dono se o Chalk/Crayon herdam a lei (decisão dele).
Aberto: curva da pressão do K1 a p 0,5–0,65 abaixo da do Krita; o pálido no branco/blocos é a `mistura_opaca`.

Fila depois (do prompt anterior): memória do 1.º traço · marca do Wet por camada · Wet Wash Mix · Splash ·
`SPRAY_FIXO_MAX` com Wet blend · painel a 245 px · dicas longas · papel de imagem sem dente com Grain.
Inegociáveis e fecho: os do prompt anterior (bateria por nome inclui agora `ph2d-painter-brush` `texture::lei::tests`).

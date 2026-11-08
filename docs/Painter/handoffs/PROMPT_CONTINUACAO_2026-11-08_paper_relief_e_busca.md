/pd-linha-assumir PainterWatercolor

Estado: linha `line/PainterWatercolor`, último commit = o do fecho da onda «fila pós-smoke» (08/11, depois de
`ac3afbdad5`). Composite em px, Paper que morde por cima do Grain, #78 revogado, busca no dropdown e prévia dos
esbatedores feitos; smoke do dono OK nisso. Nada integrado nem enviado.

LEIA ANTES DE TOCAR EM CÓDIGO:
- `docs/Painter/handoffs/HANDOFF_ONDA_line_PainterWatercolor_2026-11-08_FILA_POS_SMOKE.md` — o que entrou, o veredito
  do dono verbatim, a FILA 1–7 e os avisos (cena do Composite sobreposta pelas definições gravadas; método de foto).
- `project-memory/project_queue_smoke_verdicts_2026_11_08b.md` (a decisão do Paper) e
  `project_color_mixing_tuned_for_our_engine_not_csp_2026_11_08.md`.
- `crates/ph2d-painter-brush/src/dente.rs` e `crates/ph2d-tool-painter/src/tool/paint/dab_spec.rs::poe_o_papel_como_grain`.

Próximo item: a fila do handoff, em bloco (CLAUDE.md §0.10). Ordem proposta: (1) Paper — o efeito do papel na tinta
tem de crescer com o Relief e ganhar um multiplicador NOVO no Paper («ajustar o efeito no pincel»), nos dois caminhos
(papel-como-grain e o slot `dente`); medir com a sonda `sonda_auditoria_papel_grao` antes/depois e mostrar ao dono ·
(2) campo de busca do dropdown bonito, no idioma dos nossos painéis, fixo no topo · (3) perguntar de novo: aprovar as 4
miniaturas dos esbatedores e se Smear/Blur Basic ficam com as faixas · depois (4) Pastel, (5) abertos herdados,
(6) papel de imagem, (7) memória do 1.º traço só depois do CI da integração (confira o estado).

Inegociáveis: Watercolor Basic idêntico ao byte (`o_watercolor_basic_nao_mudou_com_o_pigment_mix`); Paper desligado e
pincel sem Grain = mesmo byte; os 2 alarmes de pincel por nome (`a_largura_de_fabrica_e_a_aprovada`,
`a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`) — as 4 miniaturas vermelhas dos esbatedores NÃO se regravam sem
o dono (`PH2D_PINCEIS_APROVA=1`); o default do multiplicador novo não muda documentos existentes sem o dono ver;
teste no app em Xvfb próprio, perfil smoke, `import -window`, nunca `spectacle`, foto antes do smoke; UI nova:
maquete/foto lado a lado antes do smoke, textos em inglês; agentes ajudantes 45 min cada; nada de
`fila-pesada.sh correr`; não integre nem faça push sem ordem do dono.

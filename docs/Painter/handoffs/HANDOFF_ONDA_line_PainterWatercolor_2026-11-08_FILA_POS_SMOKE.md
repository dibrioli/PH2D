# Handoff de ONDA — fila pós-smoke (08/11) — a linha CONTINUA

Linha `line/PainterWatercolor`. Nada integrado nem enviado. Commit da onda: `ac3afbdad5` (sobre `fb9e141f68`).
Fotos e montagens: [`material_2026-11-08_fila/`](material_2026-11-08_fila/). Bugs: [`BUGS_painter.md`](../BUGS_painter.md) (#78 revogado).

## O que entrou (`ac3afbdad5`)

- **Composite em px** — o Size de cada camada mostra/edita px de DIÂMETRO = `round(2·size_px·tamanho_efectivo)` (com o
  piso do Spacing; `composite.rs::tamanho_efectivo`, `BrushSettings::composite_layer_size_effective/_floor`); o modelo
  continua multiplicador (formato e porta de macro intactos). Lógica em `populate_composite_chips.rs` (`liga_tamanho`,
  re-ligado a cada quadro nos 2 painéis). Rótulo «Size (px)». O Size do pincel nos cartões arredonda a px inteiro
  (`paint_wetpaint_presets.rs::numeros_dos_volumes`). Painel antigo: o que não deixa o rótulo de caixa caber inteiro
  (cor, Erases, Strength na fileira A) desce de fileira (`paint_composite.rs`, `fileiras_da_camada`); o título quebrado
  «Composite / Brush» empurra as camadas (o «Bru» solto era a 2.ª linha dele, desde 03/11). Teste
  `no_cartao_do_composite_cada_nome_de_caixa_cabe_e_o_titulo_nao_invade` (tecto `CORTADOS_POR_LARGURA`, só desce;
  245 px ainda corta 6 Strength + 12 — largura estreita demais), `seam_composite_size_px` (2).
- **O Paper morde por cima do Grain** — slot `Dente` no `BrushSpec` (`ph2d-painter-brush/src/dente.rs`, `dente_at`),
  enchido em `dab_spec.rs::poe_o_papel_como_grain` quando há Grain próprio (`papel_pode_morder`); multiplica a cobertura
  em `dab/bands.rs`, `stamp.rs` (banda da tela), `stamp_color/accumulate*.rs`; desliga a cache de máscara
  (`dab_mask_cacheable`). Sem Grain = caminho antigo ao byte. Custo +4–8 % (`sonda_custo_do_dente`, ignorado). Testes
  `dente_do_papel_por_cima_do_grain` (3, vermelhos com a cura revertida). Sonda `sonda_auditoria_papel_grao` ganhou
  as variantes `papel_*` e o filtro `PH2D_AUD_VAR`.
- **#78 REVOGADO** (dono: «voltar ao antes, nos três») — saíram `Deposito`, `sorteia_a_pegada`, `carimbou_em`;
  Pointillism/Pastel/Chalk = miniatura aprovada (alarme verde). `os_pinceis_que_espalham_enchem_com_a_mistura` (invertido).
- **Busca no dropdown de presets** — `paint_wetpaint_presets_busca.rs` (campo = o `TextInput` do «Search…» da
  Hierarchy; ↑↓ = `arm_list_nav` do Board; sem acentos; «No brushes found»; arrastar desligado ao filtrar). Testes
  `paint_wetpaint_presets_busca_tests` (5), `seam_wet_presets_busca` (2).
- **Prévia dos que só mexem tinta** — `wetpaint_preset/preview.rs::so_mexe_tinta` (por medição: 0 px no vazio + Wet
  blend abaixo do que cobre, ou Smear/Blur) ⇒ Finger Tip, Gouache Blender, Smear Basic, Blur Basic recebem três faixas
  (`tintas_por_baixo`) na miniatura E no preview vivo. Testes `preview_esbatedores_tests` (3). Alarme de miniatura
  VERMELHO só para esses 4 — **NÃO regravado** (o dono aprova). Smoke novo `PH2D_COMPOSITE_PRESET=<nome>`.

## Veredito do dono no smoke (08/11)

OK: Composite em px · Pointillism/Pastel/Chalk como antes · prévia dos esbatedores · busca funciona.
Ditos: «o aspecto do campo de busca ficou feio. Mas funciona.» · «Independente da intensidade do relevo de paper, o
efeito sobre a tinta é o mesmo. isso não está correto. além de corrigir isso precisamos de um novo parâmetro
multiplicador em paper para poder ajustar o efeito no pincel de forma que não seja determinado apenas pelo valor do
relevo.» Foto do campo feio: o contorno azul grosso encosta nas bordas do popover, o campo rola com a lista.
Não respondeu: aprovar as 4 miniaturas dos esbatedores; Smear/Blur Basic ficam com as faixas? (perguntar de novo no smoke).

## FILA de implementação (a janela nova reordena medindo; famílias num bloco, CLAUDE.md §0.10)

1. **Paper: o efeito na tinta segue o Relief + multiplicador novo no Paper.** Mecanismo: a profundidade do dente é
   SÓ `paper_depth / PAPER_TOOTH_MAX` (o Tooth do pincel) — em `dente.rs` e no ramo antigo papel-como-grain de
   `poe_o_papel_como_grain`; o Relief (`substrate_depth`) só liga (> 0). Cura: profundidade = f(Relief) × Tooth ×
   «multiplicador do efeito no pincel» (controlo NOVO no cartão/painel do Paper, inglês, rótulo que se explica, gravado
   no documento com default que não muda o que existe — medir e decidir o default; os 2 caminhos). Medir antes/depois
   com a sonda (Relief 0,2/0,5/0,8/1 × multiplicador) e mostrar ao dono. UI nova ⇒ maquete + foto lado a lado.
2. **Campo de busca bonito** — no idioma dos nossos painéis: margem interna, contorno fino de foco, fixo no topo (não
   rola com a lista: precisa de faixa com o fundo do popover — o agente disse que o painel não a tem em cartões;
   resolver), altura e tipo como os campos do cartão. Maquete + foto lado a lado antes do smoke.
3. **Aprovação pendente**: as 4 miniaturas dos esbatedores (`target/relatorio-pinceis/index.html`) e se Smear/Blur
   Basic ficam com as faixas.
4. **(d) Pastel parecer pastel, não spray** — oráculo do produto do dono ANTES de construir; simular antes; o dono faz a triagem.
5. **(g) Abertos herdados:** Wet Wash Mix (grão não aparece + ~4× lento) · Splash (o dono olha) · filme do Óleo Real
   (SÓ por ordem) · `SPRAY_FIXO_MAX` não re-medido com Wet blend · painel antigo a 245 px ainda corta Strength/Size.
6. **Papel de IMAGEM sem dente** quando há Grain próprio (só nasce pela Aquarela «Use as Paper»; pede 2.ª imagem nos
   núcleos) · `uma_passagem.rs` lê o grão cru (o dente não é dividido no nível de uma passagem — hoje inalcançável).
7. **Memória do 1.º traço (dono 07/11, «depois do CI»)** — NÃO feita; `TECTO_DA_MEMORIA_MIB` em
   `tests/it/presets_de_fabrica/memoria.rs`; ver `project-memory/project_first_stroke_undo_memory_cut_next_round_2026_11_07.md`.

## Avisos para a janela nova

- A cena `PH2D_COMPOSITE_SMOKE=1` é sobreposta pelas definições gravadas do utilizador (abre com Composite desligado);
  com `XDG_CONFIG_HOME`/`XDG_DATA_HOME` vazios ela aparece certa. Fotos: Xvfb próprio 1920×1080, `xdotool windowsize`
  + `windowfocus`, `import -window root`; uma app de cada vez na placa.
- No fecho da LINHA (não agora): clippy/fmt das crates tocadas 1×, `rm -rf target/*/incremental`, handoff de integração.

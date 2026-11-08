# Handoff de ONDA — curas da mistura (08/11) — a linha CONTINUA

Linha `line/PainterWatercolor`. Nada integrado nem enviado. Commits: `0604fd6f5c` · `6b0522894b` · `24c82ab7c3`.
Auditoria: [`AUDITORIA_MISTURA_2026-11-08.md`](../AUDITORIA_MISTURA_2026-11-08.md) · tabela cura/commit/teste: [`BUGS_painter.md`](../BUGS_painter.md) (§ Bugs #73–#80).

## O que entrou

- `0604fd6f5c` — #73–#78 (motor) + pincel guardado; 12 testes em `crates/ph2d-tool-painter/tests/it/curas_da_mistura.rs`.
  #73 porta da mistura (`stamp_route.rs`) só exclui `EraseAlpha` · #74 `mistura_camada[pos]` (`composite_acumulado.rs::acumula_camada`) ·
  #75 Strength volta a agir (consequência de #73+#74) · #76 `peso_do_pincel(amount,carga)` (`mistura_opaca.rs`) ·
  #77 rampas/Per-Layer/ponta de pixel (`Pegada::Mascara.com_cor`) · #78 `Deposito::SoAmostra` entre passos (spacing > `PASSO_DA_MISTURA`).
  Brinde: `o_pincel_guardado_pinta_o_mesmo_traco` (`dab_spec.rs::authored_spec`, mistura Off no ramo não-Digital; diferia 848 px).
- `6b0522894b` — #79 `AmostraDaMistura` (json `"sample"` só fora do default; Gouache Blender e Finger Tip de fábrica «All visible layers») ·
  #80 regras `ProfundidadeDoGraoSemPapel` e `ProfundidadeNaAguada` (censo `TODOS` = 54).
- `24c82ab7c3` — o seletor Sample esmaece inteiro com a mistura Off nos 2 painéis; a dica cabe. Fotos em
  [`material_2026-11-08_curas_da_mistura/`](material_2026-11-08_curas_da_mistura/).

## Provas

- Mutações: todas sangram (script do scratchpad da janela, saída `target/curas/mutacoes.txt`, não versionados). A da Strength só sangra com porta + slot partilhado juntos.
- Sondas antes → depois: C[E,B] α 0 · C[B,B] 60×120 eventos Δ0 · Strength 0,1×1 Δ3–4 (era 0) · Finger Tip no vazio (248,241,241) (era verde) · Per-Layer Δ9–12 · Pointillism cobertura 0,40 (Off 0,38) · Pastel 0,16 (Off 0,16) · Dry Gouache 0,74 vs Off 0,81 (fora do mecanismo do #78).
- Alarmes de pincel: largura VERDE; miniatura VERMELHA para Pastel, Pointillism e Chalk (esperado do #78) — **NÃO regravada**; relatório `target/relatorio-pinceis/index.html`. Watercolor Basic idêntica ao byte: verde.
- App em Xvfb próprio, fotos olhadas.

## Veredito do dono no smoke (08/11)

OK: Gouache Blender arrasta a cor de outra camada + Sample · Finger Tip no vazio · Composite Erase/Strength · Grain Colors com mistura · mistura Off esmaece.
Ditos: «Dentro de composite os sizes devem ser em px.» · «precisa conferir se com grain/grain colors não mata Paper.» ·
«Pointillism melhor como era antes. Pastel está parecendo em Spray ruim. Não se parece pastel.» (interpretação a CONFERIR: «antes» = o denso de antes do #78 com a mistura ligada; mostrar a montagem antes/depois/Off e só então decidir; o alarme de miniatura continua sem aprovação.)
Pedidos: campo de busca no dropdown de pincéis · prévia especial para pincéis que só mexem tinta (Finger Tip, Gouache Blender).

## FILA de implementação (ordem proposta; a janela nova pode reordenar medindo)

Famílias equivalentes num bloco (CLAUDE.md §0.10): (a)+(b) são a mesma superfície (Composite/Grain × mistura); (e)+(f) são o mesmo dropdown/miniatura; (c)+(d) são a mesma decisão de look do spray denso.

1. **(a) Composite: sizes em px nas camadas** (hoje relativos; o dono quer px, como o Size do meio).
2. **(b) Grain/Grain Colors com mistura não matam o Paper** — MEDIR (cena 600×300 da sonda `sonda_auditoria_papel_grao.rs`, Paper ligado, mistura Wet × Off); curar só se medir perda.
3. **(c) Pointillism** — montar antes (pré-`0604fd6f5c`)/depois/Off lado a lado, mostrar ao dono, devolver o look aprovado.
4. **(d) Pastel parecer pastel, não spray** — oráculo do produto do dono ANTES de construir; simular antes; o dono faz a triagem.
5. **(e) Campo de busca no dropdown de presets de pincel.** UI nova ⇒ maquete/foto lado a lado antes do smoke; texto em inglês.
6. **(f) Prévia especial dos esbatedores** (Finger Tip, Gouache Blender e qualquer pincel que só mexe tinta): o efeito sobre alguns traços de tinta pré-pintados, na miniatura e onde mais a prévia aparece.
7. **(g) Abertos herdados:** Wet Wash Mix (grão não aparece + ~4× lento, `dab::grain_at`) · Pastel fraco (cruza com d) · Splash (o dono olha) · filme do Óleo Real (SÓ por ordem) · `SPRAY_FIXO_MAX` não re-medido com Wet blend · painel antigo `PH2D_CARDS=0`: a dica corta as últimas letras de TODAS (~5 %, pré-existente) e os 2 seletores sem rótulo · rótulo «Composite Brush» sobrepõe «Accumulate».
8. **Memória do 1.º traço (dono 07/11, «depois do CI»):** NÃO feita — `TECTO_DA_MEMORIA_MIB` ainda `[(300,515),(1000,2175)]` em `tests/it/presets_de_fabrica/memoria.rs`; o handoff de integração 07/11 deixou-a «para a próxima rodada». Só entra depois do CI da integração; ver `project-memory/project_first_stroke_undo_memory_cut_next_round_2026_11_07.md` (marcar só a caixa do traço; medir `undo_auditoria undo_tinta foto_em_tiles`).

Regra do dono: alarmes de pincel por nome (`a_largura_de_fabrica_e_a_aprovada`, `a_miniatura_de_fabrica_e_a_que_o_motor_pinta_hoje`); só o dono aprova regravar (`PH2D_PINCEIS_APROVA=1`).

## ABERTO ao fechar a onda

Tudo da fila acima; as 3 miniaturas vermelhas (Pastel, Pointillism, Chalk) aguardam o dono. No fecho da LINHA (não agora): clippy/fmt das crates tocadas 1×, `rm -rf target/*/incremental`, handoff de integração (DIRETRIZ §1.5.9).

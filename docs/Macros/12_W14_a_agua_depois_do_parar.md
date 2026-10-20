# Macros — W14: a água depois do Parar (2026-10-20)

> Pedido: [PROMPT_W14](handoffs/PROMPT_W14_a_agua_depois_do_parar_2026-10-20.md) — os itens 1–2 do §8 da
> [W8](handoffs/HANDOFF_INTEGRACAO_line_Macros_2026-10-17_W8.md). Antes: [W13](11_W13_o_tutorial_ensina_o_fill.md).

## §1 — Medições antes do desenho

Sonda `macro_track::sonda_w14_tests::sonda_o_salto_no_fim_do_clipe` (ignorada; 1024², Painter ligado, Wet Paint,
relógio fixo), `(texels ≠, > 2, > 8, > 32 níveis, o maior)`:

| | 2 pinceladas (199 772 pintados) | 4 pinceladas (304 819) |
|---|---|---|
| a água depois do Parar | 3 430 quadros (57,2 s) | 2 846 (47,4 s) |
| último quadro do clipe ≠ editor a +5,2 s (o fim do ▷ do caminho base) | 173 598 · 159 120 · 133 194 · 32 700 · 223 | 269 257 · … · 45 833 · 218 |
| último quadro ≠ água assentada | **185 270 (93 %)** · 171 678 · 147 470 · 42 889 · 121 | 285 190 · 261 707 · 214 291 · 60 083 · 134 |
| a água num quadro (média · o maior) | 6 896 · (14 498, 118) | 14 813 · (24 250, 193) |
| a imagem do fim nova de uma etapa editada (a espera do item 2) | **27,2 s** (`load` 291) | **39,8 s** (`load` 112) |

Onde o salto se via (explorador, lido no código): a ranhura mostra o palco enquanto um clipe escreve o `MacroTempo`;
a Keys arma o laço do clipe por omissão (`instala.rs:93`) e a faixa `MacroTime` segura o último valor
(`Extrap::Hold`) — logo o clipe parado no fim, ou o Autoplay depois do fim, mostrava a tela do Parar; ao fechar a
Keys / parar o Play da cena a sprite passava ao documento de uma vez.

O item 2: a espera (a imagem nova — sem tecto — e a água do editor parada, tecto 120 s; ou a conferência da água
gravada) suprimia o desfazer global (`macro_a_reescrever`, `macro_a_conferir`): **todas** as acções globais desse
intervalo (mover, renomear, componentes, vetor, Flip, guias, estados de UI, biblioteca, tags) entravam no passo da
edição.

## §2 — Decisão do dono (20/10, perguntada com a tabela)

**A água continua depois do fim do clipe** — ~1 min na tela, até à imagem final, sem salto; o clipe não fica mais
comprido. Recusadas: o salto no último quadro; deixar. (17/10: o clipe mais longo / o Parar esperar a água.)
Memória `project_the_water_keeps_flowing_after_the_clip_ends_2026_10_20`.

## §3 — A cura do item 1 (a cauda)

- `MacroEmCena::corre_a_cauda(dt, ate)` → `TocadorDoPainter::corre_a_cauda` → `PainterTool::wet_cauda`
  (`tool/paint/wetpaint/cauda.rs`): só com todos os passos tocados; relógio fixo. Sem `ate`, a MESMA porta da
  imagem do fim (`drena_sem_compor` + `on_tick`) ⇒ assenta nela ao byte. Com `ate`, estágio a estágio até ao
  estado guardado, e lá o composite normal + `wetpaint_end_session` (como o editor). Devolve «ainda corre»; a tela
  só se recompõe no quadro que deu um passo.
- **Onde a água da gravação parou**: o composite guarda o `MarcaDaAgua` do estado que a tela mostra
  (`WetPaintState::composta`); a sessão nascida SEM a marca (a da gravação) que acaba com a marca posta (o 1.º toque
  do artista a assa, W12; uma troca de meio) grava-o na marca (`MarcaDoArtista::agua_da_gravacao_parou`, a 1.ª
  vez fica). `FORMATO` 3 da marca (o 2 abre sem ela). Gravar o projeto com a água viva grava esse estado na cópia
  (`PainterTool::marca_a_gravar`); uma marca vazia com ele grava-se.
- O palco: `PalcosDoPainter::dt_do_quadro` (a shell dá o `dt` do quadro) e `cauda::anda` na subida, com o `ate`
  da marca do Painter ligado ou da guardada.

⛔ Recusas medidas: recompor a tela inteira na paragem (`mark_dirty_full`): 3 056 de 9 216 texels diferiam do
documento (o editor só compõe o que a água sujou). «Não andou neste quadro» ≠ «acabou»: um quadro de 16,7 ms não dá
um passo da água (25 ms) — tratar o primeiro como fim cortava a cauda no 1.º quadro.

## §4 — A cura do item 2 (a edição que flutua)

`ph2d_app_macros::espera::EdicaoQueEspera`: os objectos que esperam (`a_reescrever_em` do palco, `a_conferir_em`
das conferências) ficam com o `Macro` de antes no baseline e em cada passo registado na espera
(`a_comparar`); o que o artista faz regista-se; quando a espera acaba o passo regista-se já (com a transação da
imagem, se aterrou), mesmo sem entrada; um Ctrl+Z na espera desfaz o que veio por cima e a edição fica
(`restaurado`); sem nada por cima, o Ctrl+Z regista a edição e desfá-la (`antes_de_desfazer`). Na pilha: o
movimento, depois a edição com a imagem — o 1.º Ctrl+Z desfaz a edição (o que se viu por último). A shell:
`EstadoComMacros` no `ProjectState` (`WorldSnapshot::component_blob` / `set_component_blob`) e três chamadas.

⛔ Recusado (desenho): emendar os passos já registados com a imagem que chega — o Painter tem a pilha dele e o passo
da imagem fica por baixo de traços do artista, que não se desfaz fora de ordem.

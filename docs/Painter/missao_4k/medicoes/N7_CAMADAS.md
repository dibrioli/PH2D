# N.º 7 — os gestos de camada a 4K (trocar · voltar · duplicar · apagar · grupo)

Matriz de partida: `N7_MATRIZ_resumo.txt` / `N7_MATRIZ_NOVA_resumo.txt` (Textured Paper, 4K, load < 1):
trocar/voltar 33–36 ms em todo pincel, duplicar ~38, apagar ~34, grupo ~41; Watercolor Basic 65–75; Wet trocar 58–66.

## Onde estava o custo (trocar / voltar)

O log da matriz (`target/mede-pen-up/matriz/omissão_digital_…log`) põe UM quadro lento na troca (32,5 ms), com o
despacho do pintor a 8,4 ms, dos quais 7,5 ms são a **dobra do relevo em TELA inteira** (`note_gpu_fold`, `janela=TELA`);
o resto é a cadeia da placa a 4K: compor a pilha, a luz, a premultiplicação e a cópia ao slot
(`painter_gpu_preview::compose_light_premul`). A troca não muda a figura, mas a cura B18.3 (`a_troca_mantem_a_figura`)
**nunca pegava no app**, por duas condições a cair juntas:

1. `!impasto_visible()`: com o papel (substrato) ligado o relevo está sempre aceso, então toda troca era «recompõe».
   Os campos do relevo só leem a activa pelo traço aberto (`stroke_height`/`stroke_film`, limpos no fecho,
   `impasto.rs:611`); a tinta do papel lê os mesmos bytes na tela ou em `images`.
2. `composited.is_some()`: a pista da placa drena com `composited = None` (`take_preview_dirty`), então na pista do
   app (GPU) a pergunta respondia «não» mesmo sem relevo.

## Cura (ao bit)

`tool/layers/cache.rs::a_troca_mantem_a_figura`:
- o relevo aceso passa quando `o_relevo_nao_le_a_activa()` (planos vivos vazios);
- a pista da placa em dia (`composited None && !gpu_lane_stale && !preview_dirty`) conta como figura guardada:
  a troca não marca o sujo, o `try_drive` não corre, o slot fica.
- Chave A×B: `PH2D_TROCA_RECOMPOE=1` (no arranque) ou `cache::TROCA_RECOMPOE_COMO_ANTES` em execução = o caminho de antes.

Gates (`troca_figura_tests.rs`): `a_troca_sem_recompor_mostra_a_figura_da_recomposicao_inteira` ganhou «papel aceso» e
«relevo de impasto» (cache fica, figura = recomposição inteira ao bit, undo/redo ao byte);
`a_troca_na_pista_da_placa_nao_pede_drenagem_e_a_figura_fica` (sem papel, com papel, e o controlo que recompõe).
Mutação: M1 (volta `!impasto_visible()`) → 2 vermelhos; M2 (sem a pista da placa) → 1 vermelho. Vizinhos verdes:
`troca` 34 · `fundo` 6 · `duplicate` 2 · `camadas_integridade` 9 · `o_fundo_do_toque` 3 · `adormec` 32.

## Números (sonda temporária no crate, 4K, papel, 2 camadas de ruído, `--profile smoke`, A×B ABBA, mín/mediana de 8, load ~8)

| variante | pista | trocar (chamada) ms | drenagem seguinte ms |
|---|---|---|---|
| antes | CPU | 1,27 / 1,36 | **60,92 / 70,21** (recompor + luz) |
| antes | placa | 1,44 / 1,57 | **5,14 / 6,75** (só a dobra da CPU; no app + compor/luz/cópia na placa ≈ 24 ms) |
| agora | CPU | 1,28 / 1,32 | 0,00 / 0,00 |
| agora | placa | 1,25 / 1,32 | 0,00 / 0,00 |

Esperado no app: trocar/voltar do piso de ~33 ms para ~o quadro base (~11 ms) em todos os pincéis sem tinta viva.
A pista CPU do Watercolor Basic (vidro sob papel de cor não passa: `papel_atravessa_vidro`) e o Wet com tinta viva
(`traco_ou_tinta_viva`) continuam a recompor — ver «o que falta».

## O que falta (não feito nesta onda)

- **Duplicar / apagar / grupo** mudam a figura de verdade: hoje `invalidate_composite` (pilha inteira + dobra inteira).
  Cura proposta: confinar o sujo à janela da tinta da camada (`undo_delta::confine::janela_da_tinta`, como o «+» do
  B18.4) — duplicar e apagar mudam a figura só onde a camada tem α > 0; grupo de uma camada em pass-through precisa
  de prova de igualdade ao bit antes de manter.
- **Wet trocar** (58–66 ms, até 235): a sessão viva adormece na troca; cura análoga à do «+» (`a_tinta_que_adormeceu_muda`
  só na janela da tinta) em vez de `invalidate_composite`.
- **Watercolor Basic** (~70 ms em todo gesto): conferir se é a pista CPU (vidro sob papel) — o resumo não o diz.

## Re-correr a matriz (coordenador)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor && cargo build -p ph2d-host-desktop --profile smoke \
  && bash scripts/ph2d-run.sh bash docs/Painter/ferramentas/mede_pen_up_4k.sh matriz 3840x2160 papel
# A×B: a mesma linha com PH2D_TROCA_RECOMPOE=1 no ambiente = o caminho de antes
```

## Onda 2 (13/11) — Watercolor na troca; apagar · duplicar · grupo na janela da tinta

- **Watercolor Basic ~70 ms em todo gesto — mecanismo:** o *Textured Paper* põe `papel = cor do papel` (≠ branco), e com
  Watercolor isso põe o documento na CPU (`papel_pede_a_cpu`); a 1.ª aguada cria vidro, então a troca caía em
  `!papel_atravessa_vidro()` e recompunha 4K inteiro na CPU com vidro e relevo. O gate novo viu-o vermelho antes
  da cura (figura certa, cache caído). A composição pelo vidro lê os bytes e o vidro de CADA camada, nunca a activa
  (o traço aberto que o segue já está fora): a exclusão só vale com `PH2D_TROCA_RECOMPOE=1`.
  Gate `a_troca_com_a_aguada_no_papel_de_cor_nao_recompoe_e_a_figura_fica` (cache fica, ao bit, undo/redo; controlo recompõe).
- **Apagar / duplicar / grupo:** a figura muda só onde a camada tem α > 0 (fora dela não compõe nada, em qualquer modo,
  opacidade, ordem ou grupo). `cache.rs::alcance_da_tinta_de` mede `janela_da_tinta` ANTES do gesto e
  `a_estrutura_muda_so_em` mantém o cache e marca só essa janela. Não provado (recompõe tudo como antes): ajuste ou
  recorte na pilha, relevo/cobertura/material da camada, tinta viva do Wet, vidro sob papel de cor (as guardas do «+»).
  Gate `apagar_duplicar_e_agrupar_recompoem_so_a_janela_da_tinta` (3 gestos × normal · modos e opacidade · aguada · ajuste
  × controlo; ao bit contra a recomposição inteira, undo/redo). Mutação M3 (sem o `mark_dirty` da janela) → vermelho.
- Vizinhos verdes: `troca` 36 · `fundo` 6 · `duplicate` 2 · `camadas_integridade` 9 · `o_fundo_do_toque` 3 · `adormec` 32 · `mais_sem_diff` 3.
- **Não medido a 4K nesta onda** (prazo do agente): o ganho esperado é a troca do Watercolor cair ao piso das outras e
  apagar/duplicar/grupo pagarem só a caixa da tinta; confirmar na matriz.
- **Aberto:** Wet a trocar (58–66 ms: a tinta viva força a recomposição inteira — confinar à janela da adormecida como
  o «+»); apagar/duplicar/grupo com aguada no papel de cor (provar que tirar o último vidro não muda a figura fora da tinta).

## Onda 3 (13/11) — o Wet a trocar de camada só na janela da tinta

- **Mecanismo:** `a_troca_mantem_a_figura` recusava toda troca com a sessão viva (`traco_ou_tinta_viva`), e a do alvo
  que acorda também caía na 2.ª pergunta → `invalidate_composite` (pilha 4K inteira + dobra). A tinta viva do Wet vive
  NA tela da camada (o motor escreve no `canvas_rgba`; `wet_ultima_imagem` pousa a última imagem e marca o próprio
  sujo), então a figura só muda onde a tinta mexe.
- **Cura** (`mutate.rs::set_active_layer`, `cache.rs::a_troca_mantem_salvo_a_tinta` / `a_tinta_que_acordou_muda`): se o
  único impeditivo é a tinta viva (traço por fechar continua a recusar), a sessão adormeceu de facto (`dormiu`) e a
  pergunta salvo-a-tinta vale depois do acordar, o cache fica e marca-se a janela da tinta da que adormeceu
  (`a_tinta_que_adormeceu_muda`, como o «+») e da que acordou. Chave em execução: `PH2D_TROCA_RECOMPOE=1` /
  `TROCA_RECOMPOE_COMO_ANTES` = o caminho de antes.
- **Gate** `a_troca_com_a_tinta_viva_do_wet_marca_so_a_janela_da_tinta` (sem papel, papel aceso, controlo
  `TROCA_RECOMPOE_SEMPRE`): cache fica, janela < tela, figura = recomposição inteira ao bit, undo/redo.
  Mutação M4 (ramo novo desligado) → vermelho. M5/M6 (sem a marca da adormecida / da que acorda) → verdes:
  EQUIVALENTES na pista CPU, porque a sessão marca o próprio sujo; ficam como no «+» pela pista da placa (versão da
  camada), sem gate da placa que as decida — custo ≈ 2 varreduras `janela_da_tinta` (~1–3 ms a 4K).
- **Números** (teste temporário não comitado, 4K, Textured Paper, `--profile smoke`, ABBA 6 blocos × 3 trocas, load ~4–6):

  | pincel | variante | troca (chamada) ms | drenagem seguinte ms |
  |---|---|---|---|
  | Wet Wash | antes | 2,0–8,5 | **44,9–59,7** |
  | Wet Wash | agora | 4,4–10,4 | 3,6–7,0 |
  | Express Oil | antes | 2,6–5,5 | **42,5–46,9** |
  | Express Oil | agora | 5,0–8,4 | 3,1–5,9 |
  | Filbert | antes | 1,8–5,3 | **43,9–51,2** |
  | Filbert | agora | 4,6–7,8 | 3,6–6,5 |

  Troca + quadro: ~45–64 → ~8–17 ms. O `flush.bring_home` (espera do motor) é 0,5–6,6 ms: sobrepô-lo não paga o risco
  de ordem, ficou.
- **Os 235 ms da camada nova (Express Oil/Filbert): NÃO reproduzidos** nesta cena (pior troca 10,4 ms, «+» 2,2–6,1 ms,
  2 camadas, 3 trocas após «+» e traço). Hipótese a conferir na matriz: o fio da compressão da adormecida (Continuação 3
  do item 3: um traço 300–760 ms após o «+» ainda o encontra) ou o 1.º aquecimento do fundo; a matriz traz a marca.
- **Apagar/duplicar/grupo com aguada no papel de cor: NÃO provado, continua a recompor tudo** (guarda
  `papel_atravessa_vidro` de `a_nova_transparente_mantem_a_figura`). Sem prova de que a composição pelo vidro não lê
  o vidro da vizinha fora da janela da tinta+vidro, não entra (o prazo do agente acabou antes da prova).
- `cargo build -p ph2d-tool-painter --profile smoke --tests` compila. Vizinhos verdes por nome: `troca` 37 · `fundo` 6 ·
  `duplicate` 2 · `camadas_integridade` 9 · `o_fundo_do_toque` 3 · `adormec` 32 · `mais_sem_diff` 3 · `vidro` 32.

## Onda 4 (13/11) — o que o APP paga além dos testes

Matriz do app (camada nova, 4K, Textured Paper, load < 1), ms (trocar / voltar / duplicar / apagar / grupo):
antes das ondas `N7_MATRIZ_NOVA_resumo.txt`, depois das ondas 1–3 `N7_MATRIZ_FIM_NOVA_resumo.txt` (log `N7_MATRIZ_FIM_NOVA.log`):

| pincel | antes das ondas | depois das ondas 1–3 |
|---|---|---|
| Digital (Chalk etc.) | 34/34/40/33/41 | 13/13/34/27/36 |
| Watercolor Basic | 80/73/76/74/80 | 14/13/85/66/72 |
| Express Oil | 235/82/41/77/41 | 45/43/41/49/41 |
| Filbert | 239/96/39/93/41 | 64/63/46/60/40 |
| Wet Liner | 88/91/53/73/42 | **100**/41/61/44/47 |
| omissão wet | 75/78/55/66/46 | 73/81/55/37/41 |

Leitura dos logs por pincel (`target/mede-pen-up/matriz/*.log`, o quadro lento e o `[paint-perf] WORST split` na janela
de tempo de cada marca; as linhas da coluna 3 das `.marcas` não batem com o log, use o relógio `parede=`):

1. **Wet a trocar (40–100 ms) — mecanismo e cura (feito).** Wet Liner: quadro 99 ms, despacho 45 ms com a dobra em
   **TELA** (27 ms), ou seja, `invalidate_composite`. No app a simulação viva marca o próprio sujo a ~40 imagens/s
   (`wet_ultima_imagem`), então no clique há quase sempre um sujo **pendente**, e `a_troca_mantem_com` exigia
   `!preview_dirty`. Os testes da onda 3 andam com o relógio fixo, sem sujo pendente. Cura (`cache.rs::a_troca_mantem_com`):
   um sujo pendente **confinado** (`dirty_rect` presente) não impede manter a figura. A drenagem seguinte recompõe-o de
   qualquer modo, e fora dele o cache (ou o slot da placa, sem `gpu_lane_stale`) é a figura. Com
   `PH2D_TROCA_RECOMPOE=1` vale o `!preview_dirty` de antes.
   - Gate `a_troca_com_um_sujo_pendente_nao_recompoe_e_a_figura_fica`: píxeis escritos e marcados na activa antes da
     troca; pista CPU com e sem a tinta viva do Wet; o cache fica, o sujo contém o pendente (sem a tinta viva, não é a
     tela), a figura é a da recomposição inteira ao bit, com undo/redo; na pista da placa a pista não fica velha e a
     reivindicação é a do sujo; o controlo `TROCA_RECOMPOE_SEMPRE` recompõe. Mutação M7 (volta `!preview_dirty`) dá vermelho.
   - Sonda `sonda_n7_app_tests.rs` (`#[ignore]`: a cena da matriz a 4K com Textured Paper, `--profile smoke`), drenagem
     depois da troca com o sujo pendente: digital 45,8/43,9 → **2,0/1,4 ms**, Wet 41,7/40,6 → **6,5/6,3 ms**.
2. **O piso de ~25–35 ms do duplicar/apagar/grupo (mecanismo achado, NÃO curado).** Chalk: duplicar dá um quadro de
   32,6 ms com despacho de 5,0 ms e `fold 0.3 janela=rect`; apagar 25,6/1,2; grupo 34,7/0,9. A ferramenta já confina (a
   sonda mede gesto ~1,3 ms + drenagem CPU 4,6–5,8 ms). Quem paga é a **placa**: `ph2d-app-painter/src/painter_gpu_preview.rs::regiao_suja`
   só confina a composição quando `s.entradas.0 == ops`. Duplicar, apagar e agrupar mudam a lista achatada, então a
   placa compõe a pilha 4K inteira (≈24 ms, a mesma conta da onda 1), mesmo com a dobra já confinada.
   Cura proposta: a ferramenta expõe «a estrutura mudou só no sujo» (posto por `a_estrutura_muda_so_em` no ramo
   `Window`, consumido no `take_preview_dirty`), e o `regiao_suja` aceita `ops` diferentes quando esse sinal vem com
   luts e papel iguais. Prova necessária: `composite_region_into_canvas` com a lista nova recompõe a região só das
   fatias, sem cache intermédio por lista. Gate na placa ao bit contra `SEMPRE_INTEIRA`, com mutação. Não entrou nesta
   onda porque o prazo do agente não cobria o gate na placa.
3. **Watercolor Basic duplicar/apagar/grupo (66–85 ms): não provado, continua a recompor.** O log diz pista CPU,
   `branch=FULL-composite`, preview 49–68 ms + upload 5,3: `alcance_da_tinta_de` → `None` pela guarda
   `papel_atravessa_vidro` (a do «+»). O caso vidro + papel de cor continua sem prova (ver a onda 3). Também ficam de
   fora **Wet duplicar/grupo**: `alcance_da_tinta_de` recusa a `tinta_viva()` e a sonda mostra a drenagem inteira
   (44–50 ms). A cura seria a da troca da onda 3 (marcar a janela da que adormece/acorda).

Vizinhos verdes por nome (`--profile smoke`): `troca` 36 · `fundo` 6 · `duplicate` 2 · `camadas_integridade` 9 ·
`o_fundo_do_toque` 3 · `adormec` 32 · `mais_sem_diff` 3 · `vidro` 32. `cargo build -p ph2d-tool-painter --profile smoke --tests` compila.

Re-correr (coordenador): a mesma linha do «Re-correr a matriz» acima. Esperado: Wet trocar/voltar caem para ~13–20 ms;
duplicar/apagar/grupo ficam iguais até ao item 2.

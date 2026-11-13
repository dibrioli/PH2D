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

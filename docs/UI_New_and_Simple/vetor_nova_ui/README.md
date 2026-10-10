# vetor_nova_ui — a nova UI do Vector

> Aberta em **2026-10-07** a pedido do Enio: *«toda UI/UX do vector está encarcerada no painel
> lateral. Precisamos de uma investigação e um plano para tornar a UI/UX mais próxima dos mais
> intuitivos e divertidos apps vetoriais, distribuindo seções em várias barras (lateral esquerda,
> superior e inferior, além do painel lateral direito). (…) Eu particularmente gosto mais do estilo
> Affinity que do Inkscape.»* — e depois: *«apps para iPad costumam ser bons nisso.»*
>
> **Estado: plano DECIDIDO (07/10)** — ferramentas flutuando à esquerda · opções em cima · teclas do
> **Illustrator** (o padrão mais aceito, medido) · barra de ações junto da seleção **ligada** no desktop e
> no iPad · Hierarquia onde está ([`03_plano.md`](03_plano.md) §6, [`../00_DECISOES_DO_ENIO.md`](../00_DECISOES_DO_ENIO.md) D10).
> **F0–F3 ENTREGUES na `line/UIUX` (07/10)** — handoff
> [`HANDOFF_INTEGRACAO_line_UIUX_2026-10-07_NOVA_UI_DO_VETOR.md`](../handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-07_NOVA_UI_DO_VETOR.md),
> foto do «depois» em [`fotos/depois_01`](fotos/depois_01_cena9_caneta_na_mao_com_duas_formas.png),
> smoke `PH2D_OBJECT_MODE_SMOKE=9`. **F4–F5 (08/10)**: a roda saiu (D10.2), a paleta que ensina e o
> toque ficaram — [handoff](../handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-08_F4_F5_ALCANCE_E_TOQUE.md).
> **F6 (08/10)**: dois dedos arrastam a vista e a pinça dá zoom à volta deles, cena `=11`, foto
> [`fotos/f6_01`](fotos/f6_01_cena11_dois_dedos_arrastam_e_aproximam.png) —
> [handoff](../handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-08_F6_NAVEGAR_COM_OS_DEDOS.md).
> **F7 QuickShape (08/10)**: desenhar e SEGURAR = forma perfeita com nós (linha, polilinha,
> triângulo, rectângulo, polígono, elipse/círculo), arrastar escala e roda, `Shift` regulariza;
> interruptor na fila do Lápis. A lei bate as **65** medições do Xournal++
> ([`oraculo_quickshape/00_resultados.md`](oraculo_quickshape/00_resultados.md)); cena `=12`, foto
> [`fotos/f7_01`](fotos/f7_01_cena12_tres_tracos_seguros_viram_formas.png) —
> [handoff](../handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-08_F7_QUICKSHAPE.md). **F7b (09/10)**:
> formas pequenas, mais sensibilidade (o parado com folga, a elipse no meio da faixa) —
> [handoff](../handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-09_F7b_QUICKSHAPE_SENSIBILIDADE.md).
> **F7c (10/10)**: o ARCO — um arco de mão seguro vira um arco de círculo perfeito (§3g do oráculo;
> foto [`fotos/f7c_01`](fotos/f7c_01_cena12_um_arco_seguro_vira_arco_perfeito.png)) —
> [handoff](../handoffs/HANDOFF_INTEGRACAO_line_UIUX_2026-10-10_F7c_QUICKSHAPE_ARCOS.md).
> **F8 (10/10)**: *Ver ▸ Enquadrar tudo / seleção* e o beliscão rápido — a vista voa até tudo caber na
> área visível; o «Frame All» que já existia (só imagens, na janela, a saltar) foi curado. Lei e
> oráculos em [`11_enquadrar_tudo.md`](11_enquadrar_tudo.md); cena `=14`, foto
> [`fotos/f8_01`](fotos/f8_01_cena14_tres_formas_fora_da_tela_e_ver_enquadrar_tudo.png).

## O roteador

| você quer | leia |
|---|---|
| **O que existe hoje, medido no código e fotografado** — 41 secções numa coluna, 16 ferramentas em botões de texto, nenhuma tecla de ferramenta, e a classificação GESTO/SELEÇÃO/DOCUMENTO que já é o mapa das barras | [`01_o_estado_atual.md`](01_o_estado_atual.md) |
| ⭐ **O estudo** — os cinco lugares em que todo app amado converge, por que o Affinity é o modelo (e onde ele tropeçou), o que os apps de iPad sabem, as armadilhas que todos pagaram, e o teclado partilhado | [`02_estudo_os_apps_amados.md`](02_estudo_os_apps_amados.md) |
| ⭐⭐ **O plano** — os desenhos alvo (desktop e iPad), a tabela de destino das 41 secções, as peças e onde vivem, as fases F0–F5 com gates, o que fica de fora, e **as decisões do Enio** (respondidas) | [`03_plano.md`](03_plano.md) |
| ⭐ **O prompt da próxima janela** — F4 (radial, `Ctrl+K` que ensina) e F5 (iPad), com o estado da linha e as lições de F0–F3 | [`04_prompt_F4_F5.md`](04_prompt_F4_F5.md) |
| O prompt de F6 — navegar com os dedos, o QuickShape à espera do oráculo, a ponte do toque | [`05_prompt_F6_toque_completo.md`](05_prompt_F6_toque_completo.md) |
| ⭐ **O oráculo do QuickShape** — o Xournal++ corrido sobre 43 traços nossos: triagem de licença, o roteiro, as leis medidas, o que o Procreate manda a mais | [`oraculo_quickshape/00_resultados.md`](oraculo_quickshape/00_resultados.md) |
| O prompt de F7 — o QuickShape no PH2D (a lei pura, o gesto de segurar, a porta visível, a cena 12) | [`06_prompt_F7_quickshape.md`](06_prompt_F7_quickshape.md) |
| ⭐ **O prompt da próxima janela** — F7c: o QuickShape com ARCOS (pedido do dono, 10/10) | [`07_prompt_F7c_arcos.md`](07_prompt_F7c_arcos.md) |
| ⭐ **Enquadrar tudo (F8)** — o tldraw e o Excalidraw corridos (a mesma lei com parâmetros diferentes), a margem derivada das barras, o voo pelo ponto fixo, o beliscão rápido, as recusas | [`11_enquadrar_tudo.md`](11_enquadrar_tudo.md) · [`oraculo_enquadrar/`](oraculo_enquadrar/) |
| As fontes, com URL por afirmação | [`referencias/R1_affinity.md`](referencias/R1_affinity.md) · [`R2_figma_sketch_penpot_graphite.md`](referencias/R2_figma_sketch_penpot_graphite.md) · [`R3_illustrator_curve_rive_inkscape.md`](referencias/R3_illustrator_curve_rive_inkscape.md) · [`R4_apps_de_ipad.md`](referencias/R4_apps_de_ipad.md) |
| As fotos do «antes» e o precedente interno (a barra curta e a barra de estilo do Quadro) | [`fotos/`](fotos/) |

## As leis que esta pasta herda (não re-litigar)

[`../00_DECISOES_DO_ENIO.md`](../00_DECISOES_DO_ENIO.md): painéis ancorados com flutuação declarada (D1)
· um sítio por comando, cortado por âmbito (D2) · ⛔ **nenhuma faixa nova de altura** (recusa de
31/08, tablets) · encaixes fixos (D4) · régua e trilho são regiões da área (D5) · o modo é do tipo do
objecto, e o Vetor é um layout próprio (D3/D6/D7).

## Como as fontes foram obtidas

Quatro pesquisas web em paralelo (2026-10-07), lendo manuais oficiais, notas de versão, fóruns e
reviews. ⚠️ É **leitura**, não oráculo: basta para decidir **onde** vai um controlo, nunca para a LEI
de um gesto (`CLAUDE.md` §0.9). O Graphite (Apache-2.0) está clonado em
[`../referencias/graphite`](../referencias/graphite) pelo `fetch-referencias.sh`. O Inkscape 1.4.4 está
instalado nesta máquina (`pacman -Q inkscape`), se for preciso corrê-lo. O Affinity não tem build para
Linux.

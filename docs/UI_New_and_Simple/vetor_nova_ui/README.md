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
> smoke `PH2D_OBJECT_MODE_SMOKE=9`. Abertos: F4 e F5 (depois do smoke do dono).

## O roteador

| você quer | leia |
|---|---|
| **O que existe hoje, medido no código e fotografado** — 41 secções numa coluna, 16 ferramentas em botões de texto, nenhuma tecla de ferramenta, e a classificação GESTO/SELEÇÃO/DOCUMENTO que já é o mapa das barras | [`01_o_estado_atual.md`](01_o_estado_atual.md) |
| ⭐ **O estudo** — os cinco lugares em que todo app amado converge, por que o Affinity é o modelo (e onde ele tropeçou), o que os apps de iPad sabem, as armadilhas que todos pagaram, e o teclado partilhado | [`02_estudo_os_apps_amados.md`](02_estudo_os_apps_amados.md) |
| ⭐⭐ **O plano** — os desenhos alvo (desktop e iPad), a tabela de destino das 41 secções, as peças e onde vivem, as fases F0–F5 com gates, o que fica de fora, e **as decisões do Enio** (respondidas) | [`03_plano.md`](03_plano.md) |
| ⭐ **O prompt da próxima janela** — F4 (radial, `Ctrl+K` que ensina) e F5 (iPad), com o estado da linha e as lições de F0–F3 | [`04_prompt_F4_F5.md`](04_prompt_F4_F5.md) |
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

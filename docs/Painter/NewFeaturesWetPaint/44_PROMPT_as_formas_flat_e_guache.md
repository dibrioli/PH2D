# 44 — PROMPT: as FORMAS — a ponta reta do Flat e o corpo que cobre da guache (doc 18 §5 bloco C) (17/10)

TROCA DE AGENTE — `line/PainterWatercolor`. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
&& pwd && git branch --show-current`; `git status --short` limpo; `bash scripts/btrfs-health.sh`. Leia SÓ: este prompt ·
[doc 18](18_mapa_das_midias.md) §2 (a régua e a barra), §3 (as linhas **aquarela** e **guache**), §4.5 e §5 · §2–§3 e
§6 do [handoff P43](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-17_P43_PO_FINO.md) ·
[43_medicao](43_medicao_o_po_fino.md) §1 e §3 (o MÉTODO e as duas armadilhas) · o censo Flat/Fan do
[prompt 39](39_PROMPT_w5_flat_fan_saem_doc_do_shape.md) (o que saiu do motor e porquê) · lista [26](26_ABERTOS.md).
Regras de 13–14/10 (teste POR NOME, suíte a cada 10 commits, clippy só no fecho, edição provada). Oráculo: o Rebelle
corre-se (CLAUDE.md §0.9); nada dele se lê nem se copia. O teste do Rebelle acaba por volta de 06/11.

## De onde vimos

A linha integrou no `main` a 17/10 (prompts 35–43, [handoff de integração](../handoffs/HANDOFF_INTEGRACAO_line_PainterWatercolor_2026-10-17.md))
e foi reaberta no `main` `ad5f8fe39c`. O bloco A (a lei da tinta: o Glaze por pincel) e o bloco B (o PÓ: `dust`, o
Airbrush) do 18 §5 estão feitos — fábrica 18. **O smoke do dono do P43 ainda não voltou**: se chegar com defeitos,
eles vêm ANTES deste prompt (a lição do P42: os três defeitos estavam nos traços sobrepostos e na tinta a secar).

Fica o bloco **C — as formas** (18 §3):
- **aquarela Flat**: o Rebelle tem a FORMA chata de pontas RETAS; o nosso `dab_flatten` só achata (um Round mais
  claro com outro nome). Fotos `fixtures/rebelle/wcflat.png`, e para a ponta seca `ponta_textura_flatdry_{20,40}.png`.
- **guache**: corpo chapado, sem orla escura. O Gouache Round do Rebelle nos volumes de fábrica dá cob 0,22 · borda
  0,99 — **igual ao nosso Wet Round** (0,19 · 1,00): um preset «Guache» seria o Round renomeado. Pede «o corpo que
  cobre». Fotos `gouache.png`, `gouacheflat.png`, `ponta_textura_gouachedry_{20,40}.png`,
  `ponta_textura_opacidade_gouachedry_fan_round_40.png`.

## Pedido

1. **Medir primeiro (um lote só, §0.10)** — a régua é a do 18 §2 (as métricas que DISTINGUEM a mídia do controlo
   acima de 3× o ruído do Rebelle + a largura ± 2 px) e, para o Flat, a **extensão nas pontas** (a métrica nova: o
   comprimento do traço além do último carimbo, reto × redondo — defina-a no `midias_regua.py` e no `regua_midia.rs`
   juntas, e prove que o Round e a foto do Flat a distinguem).
   - A IDENTIDADE antes de qualquer lei: as **18** da fábrica (`a_lei_da_tinta::sem_glaze_cada_preset_de_fabrica_pinta_a_cena_de_antes`
     — hoje 17 + 2 cenas; junte o Airbrush), pinadas no HEAD de partida.
   - Pergunte ao CÓDIGO antes de declarar uma ausência (18 §4.5): o que o P39 tirou do motor (`BrushShape::{Flat,
     Fan}`) e o que ficou no hospedeiro (`dab_flatten`, `dab_curve`, a ponta de imagem do Shape, `taper`) — a ponta
     reta pode já ser uma ponta de IMAGEM retangular no Shape, ou o `taper`; e a guache pode ser o Glaze negativo /
     a Opacity / o `pigment` alto com a água baixa (o 18 §4.5 errou duas vezes por não varrer o eixo inteiro).
   - O oráculo: se as fotos não bastam (as duas guaches a 2 tamanhos? o Flat deitado e de pé?), corra o Rebelle com
     `ferramentas/gera_eventos.py`; feche SÓ com `ferramentas/gotas_fecha.sh`.
   - Os candidatos medidos com `ferramentas/midias_lote.py` (o medidor do P43: ordena e dá a distância ao Rebelle) e
     no CUSTO por traço (mínimo de 9 intercalados, `docs/DevOps/MEDIR_VELOCIDADE.md`).
   - ⚠️ **As duas armadilhas do P43** (43 §3–§4): (a) uma receita com `felt`/`bristleCount`/`bristleSize`/
     `bristleStrength` no mapa `engine` RECRIA as cerdas por cima do Shape «None»; (b) a sonda pinta à lei de largura
     do Wet Round — o preset de fábrica traz a SUA lei: confirme o raio do portão com `{"base": "<preset>"}`. Meça SECO
     (`PH2D_SONDA_SECA=1320`; o controlo seco também).
2. **A lei**, se a composição não chegar: propriedade do PINCEL (campo do preset por NOME, omissão = o de hoje ao byte;
   conte se `SCHEMA` sobe e porquê). Nenhum pincel existente muda (a identidade das 18).
3. **Os pincéis de fábrica**: Watercolor › Flat e Gouache (Round e Flat, se forem mídias diferentes — 18 §3: receitas
   DIFERENTES, senão é um preset com dois nomes), cada um no `midias_de_fabrica` com o controlo FORA; a contagem
   conta-se no código (`the_factory_presets_are_counted`, e as duas contagens que o P43 esqueceu: `rows::every_visible_knob_has_a_row`
   se nascer um knob, `shape_color_tests::a_cor_sobrevive_a_agua_nos_pinceis_de_fabrica`); rótulos novos em
   `rotulo_da_pasta` + i18n.
4. **Painel**: se a forma ganhar controlo, onde se mostra (o Shape ou o Tuning), i18n, tokens; censo — nenhum morto.
5. Portões vermelhos primeiro; mutação por lei (os sobreviventes ganham portão, como no P42 e no P43); prancha
   Rebelle × nosso.
6. Fecho: suíte afetada + clippy (`PH2D_SUITE=fecho`); handoff ≤ 8 KB; smoke ao dono em passos numerados, com VÁRIOS
   traços sobrepostos e a tinta a secar.

## Do P43, abertos (meça junto se forem da mesma família; senão fiquem no handoff)

- O Airbrush a **20 px** lê textura 28,7 contra 22,5 (40 e 80 batem) — a régua 5×5 num traço de 13 px?
- A **pressão**: no Spray 2 do Rebelle a rampa ESMAECE (pressão → opacidade); a nossa ESTREITA (pressão → tamanho).
  Se o Wet Paint não tiver a pressão → opacidade por pincel, é a mesma família do «corpo» da guache (o que a pressão
  e a Opacity fazem ao traço) — junte ao lote.

## Perguntas do dono ainda abertas (faça-as no smoke)

- **B13**: a Shape Color no pincel salvo do Wet Paint?
- **B14**: a mistura de pigmentos (K–M) continua só da obra (o Glaze é do pincel)?

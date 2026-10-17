# 43 — PROMPT: o PÓ FINO — o aerógrafo pinta névoa de pontos, não manchas (doc 18 §5 bloco B) (17/10)

TROCA DE AGENTE — `line/PainterWatercolor`. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
&& pwd && git branch --show-current`; `git status --short` limpo. Leia SÓ: este prompt · [doc 18](18_mapa_das_midias.md)
§2 (a régua e a barra), §3 (a linha **aerógrafo**) e §5 · §1–§3 e §6 do [handoff P42](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-16_P42_LEI_DA_TINTA.md)
· [42_medicao](42_medicao_a_lei_da_tinta.md) §1 e §5 (o MÉTODO: a sonda da célula, o oráculo corrido, a identidade
antes da lei) · lista [26](26_ABERTOS.md). Regras de 13–14/10 (teste POR NOME, suíte a cada 10 commits, clippy só no
fecho, edição provada). Oráculo: o Rebelle corre-se (CLAUDE.md §0.9); nada dele se lê nem se copia (texturas, código).

## De onde vimos

Prompt 42 fechado e aprovado no smoke do dono (16–17/10): o Glaze POR PINCEL (plano `Grid::vela`), os 5 pincéis de
tinta e marcador na fábrica (17), e três defeitos do smoke curados (a 3.ª camada molhada, as bordas que esbranquiçavam).
O bloco A do 18 §5 está feito. Ficam B (o pó fino, este prompt) e C (as formas: a ponta reta do Flat; o corpo que
cobre da guache).

O defeito que o B cura (18 §3): o melhor candidato de aerógrafo de hoje (grão Noise + dispersão) acerta a orla (13 ·
Rebelle 12), a cobertura (0,33 · 0,38) e a borda (0,38 · 0,54), mas a **textura é 7,2 % contra 22,7 %**: pinta
MANCHAS, não PÓ — o grão do motor não desce abaixo da CÉLULA da grelha (a razão `grid_map`), e o lado do grão 0,02 =
0,05 dá o mesmo. Fotos: `fixtures/rebelle/airbrush.png` (Airbrush · Default/Spray 2); os 5 Spray estão em
`fixtures/rebelle_presets_281.json` (parâmetros em texto).

## Pedido

1. **Medir primeiro (um lote só, §0.10):**
   - O portão de IDENTIDADE antes de qualquer lei: estenda `a_lei_da_tinta::sem_glaze_cada_preset_de_fabrica_pinta_a_cena_de_antes`
     (hoje 12 + 2 cenas) às 17 da fábrica, pinadas no HEAD de partida.
   - Onde o grão vive e a que escala: a célula do motor (`grid_map`, a razão da sessão), o grão do pincel (slot Grain,
     `BrushSpec::texture`), o render por píxel (`render.rs`), a dispersão (`jitter`). Pergunte ao CÓDIGO antes de
     declarar uma ausência (18 §4.5: «"não é possível" sem varrer o eixo inteiro é um palpite»).
   - O oráculo: se a foto do Spray 2 não basta para a barra, corra o Rebelle (os 5 Spray, ou o mesmo Spray a 2 tamanhos)
     com `ferramentas/gera_eventos.py`; feche SÓ com `ferramentas/gotas_fecha.sh` (há outras telas virtuais). O teste do
     Rebelle acaba por volta de 06/11.
   - Candidatos, cada um medido na barra do 18 §2 (as 7 métricas + a largura ± 2 px) e no CUSTO por traço (mínimo de 9
     intercalados, `docs/DevOps/MEDIR_VELOCIDADE.md`): (a) pontos por carimbo à resolução da TELA (uma máscara de pó
     por píxel), (b) o grão abaixo da célula no RENDER, (c) a razão da grelha 1 só para o pincel de pó — e o que mais o
     código mostrar.
2. **A lei:** o pó como propriedade do PINCEL (campo do preset por NOME, omissão = o de hoje ao byte; conte se `SCHEMA`
   sobe e porquê). Nenhum pincel existente muda (a identidade das 17).
3. **O pincel de fábrica:** Airbrush (Spray) medido na barra contra `airbrush.png`; a contagem da fábrica conta-se no código
   (`the_factory_presets_are_counted`); o rótulo da pasta nova em `rotulo_da_pasta` (ph2d-app-painter) + i18n.
4. **Painel:** se o pó ganhar controlo, onde se mostra (Tuning ou o cartão do pincel), i18n, tokens; censo — nenhum morto.
5. Portões vermelhos primeiro; mutação por lei (os sobreviventes ganham portão, como no P42); prancha Rebelle × nosso.
6. Fecho: suíte afetada + clippy (`PH2D_SUITE=fecho`); handoff ≤ 8 KB; smoke ao dono em passos numerados. ⚠️ Corra o
   smoke com VÁRIOS traços sobrepostos e com a tinta a secar — foi aí que o dono achou os três defeitos do P42.

## Perguntas do dono ainda abertas (faça-as no smoke)

- **B13**: a Shape Color no pincel salvo do Wet Paint?
- **B14**: a mistura de pigmentos (K–M) continua só da obra (o Glaze é do pincel)? Medido no P42: por pincel não mudava
  nenhum dos 5.

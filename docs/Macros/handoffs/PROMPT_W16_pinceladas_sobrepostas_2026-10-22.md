# PROMPT W16 — duas pinceladas que se sobrepõem no tempo tocam ao mesmo tempo (ordem do dono 22/10)

> Rota «linha reaberta» de `docs/IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md`:
> `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Macros-Automação && pwd && git branch --show-current`
> antes de ler; **sem reset nem rebase** (W8–W15 integram juntas). Leia antes o
> [handoff W15](HANDOFF_INTEGRACAO_line_Macros_2026-10-22_W15.md) e o [doc W15](../13_W15_o_passo_e_as_chaves.md).

## O pedido do dono (verbatim, 22/10, depois da W15)

«como o keyframe é criado no mouse up, arrastar keyframes de modo que as animações dos traços se sobreponham, apaga
os keyframes.. Isso não pode acontecer. deve permitir sobreposições»

⇒ Arrastar o losango de uma etapa («Macro steps», o losango está no FIM = o mouse up) para trás até a pincelada dela
COMEÇAR antes de a anterior ACABAR é um gesto válido: no ▷ as duas pincelam ao mesmo tempo, nos 4 meios (Digital,
Watercolor, Impasto, Wet Paint). A W15 já impede que o arrasto apague as etapas (a edição é recusada antes de se
gravar: `gate7_sobrepor_duas_pinceladas_nunca_apaga_as_etapas`); a W16 troca a recusa pela sobreposição.

## O que já se sabe (medido/lido na W15, confira o código antes de acreditar)

- A recusa: `ph2d-macro/src/etapa.rs` `efetiva` — espera negativa maior que o intervalo até ao fim da etapa anterior
  ⇒ `ErroDeEdicao::EsperaMaiorQueOIntervalo`. A doc de `LeitorDaFamilia` assume etapas sem sobreposição.
- O tocador é sequencial: `ph2d-tool-painter/src/macro_track/tocar.rs` chama `on_canvas_pointer` / `on_tick` no
  ÚNICO `PainterTool`; `Passo::Ponteiro{pos, pressao, tilt, fase}` (`macro_track/passo.rs`) não leva identidade de
  traço. `PaintState.stroke: Option<Stroke>` é um só (`tool/paint/state.rs`); estado por traço também em
  `solid_path`, `solid_fill_owed`, `dabs`, `hover_*` e em `ph2d-painter-brush/src/stroke.rs` (`last_pos`, `accum`,
  `last_pressure`, `rng`, …). Wet Paint: o motor de água é da sessão do tool (`wetpaint/session.rs`), o traço só
  deposita (`dab_route.rs`); a fita grava as respostas do trabalhador pela ordem dos pedidos.
- Na sobreposição os DOIS fluxos trazem relógio (`Preview`/`Tique`) na janela comum: fica UM relógio por quadro.

## O que a W16 faz

1. `/pd-feature`: plano primeiro, com as alternativas e a escolha (padrão-ouro). Candidatas: (a) uma tabela de
   traços abertos por id no `PainterTool` (o mesmo motor de água recebe os dois); (b) troca de contexto do estado
   por traço no tocador (um `Stroke` guardado por faixa e trocado antes de cada passo dela); (c) um `PainterTool`
   por faixa (provavelmente recusada: duas águas). Meça antes de escolher; superfície de colisão com a
   `line/PainterWatercolor` (motor Wet Paint) no handoff.
2. O formato: a faixa/id do traço no `Passo::Ponteiro` (campo etiquetado novo; ausente = 0) — confirme a regra de
   campo desconhecido em `ph2d_macro::rec` e a política de versão; suíte com `motivo: formato gravado`.
3. `efetiva`: a espera negativa para lá do intervalo passa a ENTRELAÇAR a etapa com a anterior por tempo (um relógio
   por quadro), e o Motion (a outra família) diz se aceita ou recusa — sem silêncio.
4. Gates vermelhos primeiro (as duas pincelam no mesmo quadro; a imagem final = as duas inteiras; ▷ repetível ao
   byte), mutação, cena `=6` nova variante conduzida e fotografada, handoff.

## Perguntas ao dono que seguem abertas (W15 §8)

- O veredito do smoke da W14 (a água depois do fim do clipe).
- O Dur(s) 6,208 da captura de 22/10 (a sequência de gestos).

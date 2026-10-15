# 36 — PROMPT: a cor do Shape muda o Mapping e estraga o Grain (Digital) + a avaliação das rampas de cor (14/10)

TROCA DE AGENTE — `line/PainterWatercolor`. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
&& pwd && git branch --show-current` → `line/PainterWatercolor`; `git status --short` limpo; `bash scripts/btrfs-health.sh`.
Leia SÓ: este prompt · o [handoff P35](../handoffs/HANDOFF_CONTINUACAO_line_PainterWatercolor_2026-10-14_P35_PASTEL_REBELLE.md)
§1 · a [lista 26](26_ABERTOS.md) · DIRETRIZ §3.D (modificar feature existente). Regras de 13–14/10 (teste POR NOME,
suíte a cada 10 commits, clippy só no fecho, edição provada). ⚠️ «faça como achar melhor e pare de me fazer perguntas»
(dono, 11/10). ⚠️ A ordem de 07/10 («só o Wet Paint evolui») tem uma EXCEÇÃO aqui: o dono pediu (14/10) para corrigir
estes dois defeitos no **Digital**.

## O pedido do dono (14/10), verbatim

> *«Antes de seguir com wet paint vamos avaliar e corrigir algumas coisas: Em Digital: primeiro: Por que ao usar Shape
> color parece haver uma mudança no Mapping de Shape para o tipo Tiled. Essa mudança é indesejada. Precisamos corrigir.
> segundo: Por que grain que funciona corretamente se não temos color no color ramp de Shape Colors, muda de
> comportamento e funciona de forma inadequada se colocamos cores (desmarcamos o B&W) de Shape colors? Avalie a
> possibilidade e performance de termos color ramp em Shape Color, Grain Color e Paper Color (ainda não existe) para
> todos os modos de pintura (inclusive wet paint e water color). Avalie o custo de acrescentar novas seções Grain e
> Grain colors conforme o desejo do usuário, além das já existentes.»*

## Parte 1 — os dois defeitos (corrigir nesta janela)

**Método (o do prompt 35): a régua ANTES da cura, com controlo, e a prancha olhada por si antes do smoke.**

- **D1 — a cor do Shape muda o Mapping para Tiled.** A régua: o MESMO traço desenhado em duas posições (deslocado
  `k` px); com o Mapping **View** o padrão do Shape anda COM o traço (correlação do recorte deslocado ≈ 1), com
  **Tiled** fica preso à tela. Meça B&W ligado (controlo) × desligado (cores na rampa), com o Mapping em View, Random
  e Tiled — uma tabela por rota.
- **D2 — o Grain «funciona mal» com cores na rampa do Shape.** Primeiro DEFINA o defeito por medida: o mesmo traço
  com Grain e (a) Shape sem cor, (b) rampa B&W, (c) rampa com cores — compare a pegada do grão (cobertura, contraste
  2–8 px, posição do padrão). O dono não descreveu o sintoma: fotografe as três e descreva-o na prancha.
- **Hipótese (NÃO medida):** as duas são a MESMA família — com a rampa de cor ligada o carimbo vai por outra rota
  (`stamp_color_*`, `stamp_cache.rs`, `stamp_color_cache.rs`, `stamp_color_dynamic.rs`, `stamp_route.rs`;
  `accumulate.rs` pergunta `mapping.is_canvas_fixed()`), que amostra o Shape e o Grain com uma lei SUA (a
  cache da cor em coordenadas da tela? o grão aplicado sobre a cor e não sobre a cobertura?). Se for, é uma lei
  duplicada: cure UMA porta para todas as rotas e varra por CONSUMIDOR (todas as rotas de carimbo, os cinco meios),
  §0.10 — um bloco só.
- Portões: um por defeito, pelo produto (o traço pintado), com controlo e prova de mutação. ⛔ C15: `stamp_cache.rs`
  está a 700/700 de LOC — corte antes de crescer, nunca isenção.

## Parte 2 — a avaliação (um doc, sem código de produto): `37_rampas_de_cor_shape_grain_papel.md`

1. **Rampa de cor no Shape, no Grain e no Papel (novo), em CADA meio** (Digital, Impasto, Watercolor, Wet Paint,
   Composite): de onde vem a cor em cada motor hoje (o Wet Paint deposita a cor do pincel pela silhueta; a aguada
   reconstrói-se da cor do pincel — `inercia.rs::CorDaTexturaSemLeitor`), o que teria de mudar no motor para que cada
   rampa AJA (e não seja um controlo esmaecido), e o que um oráculo faz (Rebelle, Krita, Procreate — corra-os, não leia).
2. **Desempenho, MEDIDO** ([MEDIR_VELOCIDADE.md](../../DevOps/MEDIR_VELOCIDADE.md): variantes no mesmo processo,
   intercaladas, o mínimo): o custo por carimbo e por traço com cada rampa desligada · B&W · cor, por rota e por meio.
3. **Secções de Grain e Grain Color acrescentáveis pelo artista** (N grãos): o modelo de dados (`BrushSpec` é `Copy`),
   o formato do preset (versão, migração), o painel (registo, censos, esmaecido), e como o custo por carimbo cresce
   com N (meça N = 1, 2, 4). O preço em LOC e em portões.
4. Uma tabela de decisão por item (faz / não faz / quanto custa / o que se perde) e a recomendação — **a decisão é do
   dono** (B da lista 26).

## Fecho

Handoff ≤ 8 KB e a lista 26 atualizada (D1, D2 e os itens da avaliação). O smoke vai ao dono em passos numerados, com a
PRANCHA antes/depois e os números de cada régua.

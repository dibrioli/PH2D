# HANDOFF — Janela 8 (30/10), linha `line/PainterWatercolor`

## Feito (commits)
- `1a7c1aa31c` dívida da J7: 2 portões ao byte que o #53 mudou e a J7 não correu (`shape_color_tests` cena 6, `opcoes_tests` S
  inclinado); sobre o branco ≤ 2 níveis ⇒ regravados. Lição: o #53 mexe na COR da camada, todo portão que lê `canvas_rgba`
  (não a composição) é população dele — corra `tool::paint::wetpaint` inteiro por nome ao mexer na lei do ecrã.
- `3c283ce666` OKLCH por omissão nas rampas NOVAS (`layers::rampa_nova()`); `ColorRamp::default()` fica RGB.
- `1e9a32d0af` **Over Oil** (dono: «A, B e C viram um dropdown»): `Ecra::com_oleo(OleoCelula)`; Tint = Glaze 1 · Mix (fábrica)
  = o tapar extra pinta a mistura K-M · Blend = K-M + Glaze 0,5. A escolha mora em `OleoCelula::aguada` (12 → 16 B, só nas
  grades com óleo), gravada só pela aguada que pousa SOBRE óleo. Qualquer óleo (`oleo > 0`) vale a escolha INTEIRA.
  Gates em `wetpaint/sobre_oleo_tests.rs` e `grid/oleo_celula_tests.rs`; 2 mutações vermelhas.

## Smoke do dono (30/10, depois do `1e9a32d0af`) — REPROVADO como «mescla realista»
Foto: [`pranchas/J8_smoke_dono_over_oil.png`](pranchas/J8_smoke_dono_over_oil.png) (Dabber azul, Wet Flat amarelo, papel bege).
Palavras dele: «não é uma mescla realista, ainda temos bordas pixeladas e pretas que não se justificam. Melhor investigar o
Rebelle e tentar aprender com ele a atingir um resultado realista.»
O que a foto mostra: (a) o óleo sob a aguada fica um verde-petróleo CHAPADO (sem veios, sem variação de diluição);
(b) a borda do óleo em DEGRAUS de célula, verde vivo; (c) píxeis PRETOS/escuros no contorno; (d) pontinhos amarelos soltos.
Hipóteses (NÃO medidas): (b) o degrau `oleo > 0 ⇒ escolha inteira` (`assenta::Ecra::com_oleo`) liga a lei por célula sem
transição; (c) a K-M de `uc` com `sc` onde `a_s` é pequeno ou o `resolve` (Newton) do assentar a sair da caixa; ou a
sombra do Real Oil (`oleo_sombra`) na orla. Confira na foto/sonda antes de acreditar (METODOLOGIA 2).

## Próxima janela — o Rebelle como ORÁCULO (CLAUDE.md §0.9)
1. Rebelle 8 sob Wine: `project-memory/reference_rebelle_trial_installed_under_wine.md` (sessão `kwin --virtual`, nunca
   `spectacle`). Cena NOSSA: papel bege, um óleo azul grosso (o pincel Oil do Rebelle mais parecido com o Dabber), e por
   cima uma aguada amarela (Watercolor Wet Flat), a meio do traço e assentada; a mesma cena no PH2D
   (`diag_aguada_por_cima_do_oleo`, já grava `p_meio_v*`/`p_fim_v*`). Grave as fotos do Rebelle como fixture com cabeçalho.
2. Meça POR PASSO na faixa da aguada sobre o óleo: cor média, variação (desvio) — os veios, a diluição —, a largura da
   transição na borda do óleo, e se o Rebelle tem píxeis mais escuros que as duas tintas.
3. SIMULE a cura (≤ 4 variantes, num processo; o Over Oil é o seletor) e mande ao dono «nosso | Rebelle» antes de construir.
4. O #54 (faixa creme fina onde a tinta escorreu sem óleo; o Marker Bullet) entra na mesma medição: o Rebelle diz se a
   aguada sobre QUALQUER tinta mistura — a decisão continua do dono.

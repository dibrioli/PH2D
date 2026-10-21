# Exame da família `efeitos` (J4, catálogo M — especiais, efeitos, o Marker/Effect do Rebelle)

Receitas e portões: `crates/ph2d-tool-painter/tests/it/familias/efeitos.rs` (`MEIOS` = a fábrica, `EM_ESTUDO` = os
reprovados com receita). Pranchas: `pranchas/efeitos/` (+ `cegas/`, `chaves/`). Régua: `exames/<slug>_regua.txt`.
Identidade por foto: `exames/efeitos_foto.py` (a do portão; `--neon` a do Glow). Pasta `Digital/Effects/Effects`.

**Aprovados: Dodge (M-2) · Burn (M-2) · Color Glaze (M-3).** **Reprovados (em estudo):** Glow (M-1) · Sketchy Web
(M-7) · Metallic (M-4, sem receita). **Fundidos:** Hue e Overlay → Color Glaze. **Não são pincéis:** Luminosity,
Saturation. **Não tentados:** Glitter (M-5), Particles (M-6) — ver a fila.

## Motor (aditivo, neutro ao bit; portão + mutação)

1. `Dynamics::speed_lag_s` — linha «Speed lag» (`speedLag`, `7c3ce795f3`): o `speed1_slowness` do libmypaint (ISC;
   0,04 s em 164/196 pincéis): a velocidade da L12 por um filtro de 1.ª ordem, o traço nasce a v = 0 (largo) e afina.
   0 = a lei da J3 (velocidade desde o 1.º dab). Portão `mao::velocidade::a_velocidade_com_atraso_nasce_larga`;
   mutações M1 (filtro ignorado) e M2 (a espera da J3 armada) SANGRAM.
2. `BrushSpec::wire_sorteio` (`3c9397442c`) e `BrushSpec::wire_cordas` (`8bba066751`): as cordas do Wire num ponto
   sorteado da fatia, e quantas por dab dentro do ORÇAMENTO medido (`contagem × history ≤ 4 × 24`, o pior evento de
   3,9 ms da tabela do `WIRE_HISTORY_MAX`). Portões `sketchy_tests::as_cordas_sorteadas_cobrem_a_janela`,
   `…::as_cordas_por_dab_respeitam_o_orcamento`; as duas mutações SANGRAM. Só no preset (sem UI).
3. Bancada (`8c3cf34e35` + este commit): `PH2D_EXAME_CAMADA=1` / `pinta_cena_com(…, camada)` — o candidato numa
   CAMADA transparente sobre o papel (o Effect do Rebelle pousa no papel em Normal com o mesmo α; no papel opaco o
   Add e o Color sobre o branco são branco). A biblioteca pinta-se sempre no papel opaco.
4. Miniatura (`wetpaint_preset/preview.rs`): um modo de cor que não é o Mix nem o Multiply é uma ferramenta — a
   miniatura mostra-o sobre a faixa vermelha (saía em branco); a faixa a fluxo 1 sem Accumulate (a velatura de um
   efeito pintava-a pálida). As miniaturas das borrachas e do Blender Soft ficaram ao byte (o portão verde).

## M-2 Dodge · M-2 Burn · M-3 Color Glaze — APROVADOS

- **Receita** (Digital, macro `efeito!`): o modo W3C em sRGB (Add · LinearBurn · Color — o do ficheiro do Rebelle,
  06 §2), `blend_sobre_vazio` (Normal no papel vazio), Accumulate a fluxo baixo NUM PLANO do traço (`traco.tecto 1`:
  nos 8 bits da tela os incrementos de < ½ nível por dab perdiam-se — o G do Color ficava 0, o R do Dodge 191),
  passo 0,1, sem estabilizador (atrasava o traço 3–4 px). Queda: o SINO gaussiano σ 0,45 no Dodge e no Burn (sobre o
  vermelho o Rebelle concentra o escurecimento: 50/25/10 % 20·26·32 px; a Linear dava 18·28·34 — «faixa larga», r4),
  a Linear no Color (o cone somado = o perfil do Rebelle no papel: 0,57 a ½ raio, 0,07 a 0,9). Fluxos 0,019 · 0,021
  · 0,012. A pressão leva a força; o tamanho é o do artista.
- **Oráculo** Rebelle `MARKER/effect_{dodge,burn,color}` (só a cena «sobre»; A = B), base Hard Round, escala 1.
  Medido (`efeitos_foto.py`, cruzamento sobre o vermelho · véu no papel): Dodge (193,9,34)·0,095 × Rebelle
  (193,9,34)·0,091 · Burn (172,0,16)·0,108 × (170,0,11)·0,091 · Color (180,4,26)·0,060 × (181,4,25)·0,060.
  **Divergência decidida** (decisão 7): o Linear Burn do Rebelle tira 39 % do azul e 10 % do vermelho, que nenhuma
  fórmula a um α dá; com um α só, o véu do Burn no papel fica 0,017 acima.
- **Régua** 168/192 · 172/192 · 78/80. **Vizinho** (fora da família): Burn → Glaze Round 2,0; Color → Glaze Round
  2,15; Dodge → Burn 1,43 (irmão: um clareia, o outro escurece — a prancha mostra-o sobre o vermelho).
- **Razão de uso**: luz (Dodge) e sombra (Burn) por cima do que está pintado, e trocar a cor sem mexer no valor (Color
  Glaze), em VELATURAS que se constroem passagem a passagem — nenhum pincel da biblioteca mexe só no que já está.
- **Revisão cega** (6 rondas, revisores novos a cada cura). r1–r2 listaram: as pontas e a orla curtas (cura: o sino /
  a Linear e o passo), o R do Dodge e o G do Color (cura: o plano), o escurecimento do Burn fraco (fluxo). Finais:
  Dodge r6 *«ao olho, os três quase não se distinguem; a separação saiu de medir os pixels»*; Burn r6 *«nos traços azuis
  sobre o papel não vejo diferença»* (só a base); Color r6 confiança BAIXA, *«quase indistinguíveis; só ampliando»*.
  Todos acharam o original pela BASE (as faixas do Rebelle são do Marker dele — rectângulo, orla escura; é a bancada,
  não o pincel). Fica, repetido por medição de píxel: os traços do Rebelle acabam ~3–4 px antes (no fim de cada
  traço), e o leve grão de papel dele.
- **Custo** p95 a 5 · 40 · 200 px: Dodge 0,20 · 0,61 · 14,65 ms (9/475 > 16,7; load 39–42) · Burn 0,17 · 0,59 · 8,88
  (3/475; load 36–46) · Color 0,19 · 0,63 · 10,13 (6/475; load 62–68). RSS 31–32 MiB. O plano + Accumulate a 200 px é
  o que pesa; o relógio sob esta carga é indicativo (J5 re-mede calmo).
- **Portão** `familias::efeitos::cada_efeito_se_reconhece_pela_foto` (o cruzamento e o véu; a lei dos irmãos) e
  `…::o_portao_dos_efeitos_sangra_com_cada_mutacao` (permanente): Dodge→Mix, Dodge sem «On empty», Burn fluxo ×3,
  Burn→Screen, Color→Luminosity — as 5 SANGRAM.
- **Fusões**: Hue ≡ Color sobre a tinta saturada (06 §2; as fotos (181,4,26) × (181,4,25)); Overlay = um Color mais
  escuro na bateria (179,0,20) com o véu 4× (`pranchas/efeitos/overlay_glaze.png`) — o modo é uma linha do cartão.
  Luminosity e Saturation: ΔE ≤ 1,5 na bateria, e o Saturation do Rebelle não segue fórmula (decisão 7).

## M-1 Glow — REPROVADO (em estudo)

- Receita (Digital): a queda do libmypaint do `experimental/glow` (CC0; h 0,12 em 8 pontos rectos — o tecto da curva
  é 8: com 11 o preset caía na curva de omissão) somada por dabs densos (passo 0,09, fluxo 0,45); pressão no tamanho
  0,37; velocidade 0,3 com o ATRASO 0,04 s (o motor 1). Néon medido (pico · núcleo · halo · patamar): MyPaint 0,82 ·
  18 · 52 · 0,26; nosso 0,79 · 18 · 50 · 0,26; o recto rápido 14 × 16 px, o vertical rápido 22 × 22.
- Revisores r1 e r2 (média, acharam o MyPaint): (1) os cruzamentos sobre o vermelho a quase PRETO — a mistura
  ESPECTRAL do MyPaint 2 (o `pigment` do Digital estraga o perfil: núcleo 18 → 40); (2) o CAROÇO escuro onde a caneta
  pousa (o `slow_tracking` dele empilha dabs no arranque). A cabeça do traço rápido (r1) curou-se com o atraso.
- Vizinho Mapping Pen 1,67. Custo p95 0,20 · 0,30 · 1,90 ms (load 14–18).

## M-7 Sketchy Web — REPROVADO (em estudo)

- Oráculo Krita `v) Experimental Webs` (o motor Curve; os `Sketching Chrome` do Krita não mostram a teia nesta bateria).
  Receita: `LineKind::Wire` sem a linha de dabs, 24 cordas (motor 2) em 2,5 diâmetros, cada uma 1 px a 0,06; escala 1.
  A fita em degradê casa (a 4 cordas eram quatro fios). Vizinho Mapping Pen 1,64.
- Revisor r1 (média): os traços RÁPIDOS do Krita param a meio (o Curve não pinta o fim de um traço rápido), o fio da
  recta dele é mais fino e claro, um pingo escuro no arranque, a teia dele mais irregular (o centro da espiral cheio).

## M-4 Metallic — REPROVADO (sem receita)

Rebelle `OIL_AND_ACRYLIC/metallic_round`: papel de alumínio AMARROTADO (vincos finos e nítidos, relevo baixo, brilho
fraco) sobre tinta que cobre. O Impasto com `impasto_metallic` 1 dá um tubo liso brilhante (`pranchas/efeitos/
metallic.png`); com a fonte `Grain` nenhum grão o faz: Clouds/Stucci a 4–20 px saem bolhas lisas, Voronoi blocos,
Musgrave quadriculado — e o grão FURA o pigmento (o Rebelle cobre). Falta uma textura de vincos (ruído «ridged») e
o relevo pelo grão sem tirar tinta.

## Para a fila (coordenador)

1. **Mistura espectral do Digital** (Glow; já na fila desde a Ballpoint da J3) — dono: `texturas` (modos do Digital).
2. **Os dabs que empilham onde a caneta pousa** (o `slow_tracking` do MyPaint; Glow).
3. **Metallic**: textura de vincos + relevo pelo grão que não fura o pigmento (Impasto) — motor médio.
4. **Particles (M-6)**: partículas com física (CAP-24: estado por partícula que vive além do dab — gravidade, fluxo;
   o motor só emite dabs no ponto do traço) — grande. Os oráculos que correm não o mostram: o `Random Particles` do
   Krita é o spray (dono `texturas`), o `Experimental Webs` é o Curve (M-7), os do MyPaint (DNA, particules) são outros.
5. **Glitter (M-5)**: sem foto (o Glitter Tube do ArtRage não está no `lista_artrage.tsv`: falta o id da ferramenta).
6. Sem UI ainda (só pelo preset): `wire_sorteio`, `wire_cordas`.
7. A base da cena «sobre» do Rebelle para os efeitos é o Marker dele (rectângulo, orla escura): todos os revisores
   acharam o original por ela. Uma base que a imite tiraria o sinal da bancada (o nosso Marker Bullet não o faz).

# Exame da família `texturas` (J4, catálogo K + o reexame de J-3 Splatter e J-5 Stipple)

Receitas e portões: `tests/it/familias/texturas.rs` (MEIOS = aprovados, `EM_ESTUDO` = reprovados) e, para J-3/J-5,
`familias/aerografo.rs` (`EM_ESTUDO`). Imagens dos carimbos: desenhadas no próprio ficheiro, procedurais, só polinómios
e raízes (nenhuma copiada de app nenhum). Pranchas: `pranchas/texturas/` (+ `cegas/`, `chaves/`). Régua do portão por
foto: `exames/texturas_foto.py`. Custo: `exame.sh --custo … 5,40,200`, perfil smoke (load 9–33: indicativo).

**Aprovados: Halftone (K-6) · Stars (K-5).** **Curados, NÃO aprovados (fora da fábrica, em estudo):** Foliage (K-2),
Grass (K-3), Bokeh (K-4), Noise Grit (K-8), Splatter (J-3), Stipple (J-5). Não tentados: K-1 Clouds (o MyPaint `Clouds`
enche a tela inteira à escala da bateria: sem oráculo que se compare), K-7 Hatching (o `Screentone_Moire` do Krita não
é hachura), K-9 Grunge (Wet: dual + fibras, outro dono), J-2 Spray Can (pede duas populações e a emissão parada).

## Motor (aditivo, desligado por omissão, os outros pincéis ao bit)

1. **`DabJitter::particle_px`** (`68b6abaf4c`) — o diâmetro de cada marca do spray em PÍXEIS de tela: o espalhamento
   continua o do pincel; a 5 px a gota é maior que o pincel (o cordão do Krita). Portão
   `marca::tests::a_particula_em_px_e_a_mesma_em_todo_pincel` (vermelho antes de ligar a lei). Mutações: a lei
   ignorada → SANGRA; o preset que não a grava → SANGRA (`wetpaint_preset::tests`, 2 testes).
2. **`CorDaMao::vies`** — `[matiz, sat, valor]` somados à cor de cada dab (o sorteio ENVIESADO do Pointillism do Krita:
   média (48,110,210) sobre (23,101,179)). Portão `cor_mao::tests::o_vies_desloca_a_cor_de_cada_dab` (vermelho antes).
   Mutações V1 (a cor ignora o viés) e V2 (o preset não o grava): SANGRAM. Sem UI (só pelo preset).
3. **`ReceitaMeio::texturas`** (fábrica, `68b6abaf4c`) — as imagens que a receita desenha (Shape e mangueira L19), a
   fila da J3 «N imagens na receita de fábrica». Mutação M2 do portão (o gerador ignora-as): SANGRA.

## K-6 Halftone — APROVADO

- **Receita** (Digital): o grão `Dots` preso à TELA (`Tiled`, 45°, passo 256/33 ≈ 7,8 px, raio 0,3, macieza 0,7) num
  redondo de dureza 0,8 com a orla de 3 px (`borda_px`: a 5 px a trama empalidece); tamanho pela pressão (curva 3,0) e
  opacidade (0,6, curva 1,5). 40 px = o diâmetro do preset do Krita.
- **Oráculo** Krita `y) Screentones Regular`; régua **352/452** (escala 1,04). Vizinho **Soft Round d 2,44**.
- **Razão de uso**: a retícula do mangá/retro — pontos que não crescem com o pincel, presos ao papel.
- **Revisões** (7): r1 · r2 · r3 · r5 · **r7 (final)** escolheram o NOSSO como original em 6/6, confiança baixa
  (*«a decisão é quase uma moeda ao ar»*, r2). r4 e r6 acharam o Krita nas variantes de orla 4 px/dureza 0,95 (o traço
  saía estreito e ralo) — voltou-se ao estado de r5/r7. **Fica (descrito como do Krita, todas as vezes de confiança
  baixa):** o limiar de pressão do Krita (nada até ~35–40 % da rampa), o 5 px dele mais pálido, a borda dura a 200 px.
- **Custo** p95 0,17 · 0,37 · 0,96 ms; RSS 26 MiB (load 25–28).
- **Portão** `cada_carimbo_se_reconhece_pela_foto` (pontos/100 px, cv das áreas, contraste a 200 e a 40 px; os irmãos
  são as 5 outras fotos da família). Mutação M1 (`Dots` → `None`): SANGRA.

## K-5 Stars — APROVADO

- **Receita** (Digital): a estrela de cinco pontas NOSSA (`estrela`: vértices de 72°/36° em constantes, a orla 0,7 →
  1), 2 por passo a 0,85·D, espalhada ao longo (0,5) e através (0,9·D), tamanho sorteado ±0,45, rotação livre,
  opacidade 0,5, a pressão no tamanho (1,0) e na opacidade (0,6). 108 px e a pressão = o preset do Rebelle.
- **Oráculo** Rebelle `Watercolor/Spatter/Star`, à escala nominal 1 (a largura ¼-do-máximo de uma nuvem de marcas
  soltas mede UMA marca, não o pincel: a escala medida pintava as nossas estrelas a 0,6). Vizinho Charcoal Powder 2,95.
- **Razão de uso**: céus estrelados, brilhos, confeitos — estrelas inteiras espalhadas, de tamanhos vários.
- **Revisões**: r1 · r2 · r3 · **r4 (o estado final)** escolheram o NOSSO como original em 6/6, confiança média a alta.
  r5, numa variante mais espalhada e graúda (cura do que r2–r4 listaram), achou o Rebelle em 4/6 → voltou-se ao estado
  de r4. **Divergência decidida:** a 200 px o Rebelle carimba QUADRADOS (a imagem dele cortada pela caixa; Max Size 300,
  não é tecto) — não se imita um defeito. Fica, de confiança média: o Rebelle espalha um pouco mais e deixa cacos
  pálidos na pressão baixa.
- **Custo** p95 0,19 · 0,27 · 0,39 ms; RSS 24–32 MiB (load 27–30). Régua 326/454. **Portão** idem (gota, contraste e
  cv das áreas a 40 px). Mutações: M2 (o gerador sem as imagens da receita) e M3 (partícula 0,7 → 0,25, gota 28 × 216): SANGRAM.

## Curados, NÃO aprovados (a receita da última ronda em `EM_ESTUDO`; o que falta)

- **K-2 Foliage** (Krita `z) Stamp Leaves`; raminho nosso, 7 rondas): r2, r3, r5, r7 tomaram o nosso pelo original;
  r1, r4, r6 o Krita. **Repetido por 6 revisores:** a 5 px o Krita é uma linha contínua, o nosso pontilhado — o carimbo
  de imagem amostrado sem filtro a 2–3 px parte-se (⇒ fila: o *mipmap* da imagem da ponta). E a pressão: no Krita só a
  opacidade, largura constante (r4, r5, r7).
- **K-3 Grass** (Krita `z) Stamp Grass`; 3 tufos nossos pela mangueira): r1, r2 o nosso; r3–r6 o Krita (média). Os
  revisores contradizem-se na orientação (r1: o Krita roda; r3–r4: de pé; olhado: de pé, inclinado à direita). Falta
  o tufo do Krita — lâminas finas, curvas, translúcidas, variadas, em moitas com vãos; o nosso lê-se «hachura».
- **K-4 Bokeh** (Krita `z) Stamp Bokeh`; o aro por `custom_falloff`): r1, r2 o nosso; r3–r6 o Krita (baixa). Repetido:
  na pressão baixa o Krita deixa um rasto DENSO de bolinhas (o passo segue o tamanho do dab) e mistura gigantes com
  micro-pontos ⇒ fila: o espaçamento pelo diâmetro do dab À PRESSÃO.
- **K-8 Noise Grit** (Krita `y) Texture Noise`): o DistortedNoise lia-se «anéis num tubo» (r1); os riscos nossos lêem-se
  «palitos» (r2–r4, média): falta o caracol borrado do carimbo do Krita e, de novo, a linha contínua a 5 px (mipmap).
- **J-3 Splatter** (Rebelle `Watercolor/Spatter/Splats`, Size 84): r1–r3 acharam o Rebelle (média). O Splats é AGUARELA:
  a orla escura, o miolo claro e a mistura onde se sobrepõe ⇒ recurso de outro dono (`aguarela`: orla/backrun) ou um
  Splatter no Wet. A forma (roseta, raios curtos) aproximou-se; a 200 px ainda falta a gota gigante isolada.
- **J-5 Stipple** (Krita `v) Texture Pointillism`, 114 px): com `particle_px` 18–20 px e o viés da cor, r1, r3, r4 o
  nosso; r2, r5 o Krita. Repetido (r2, r3, r5): o Krita agrupa os pontos em TUFOS regulares a 200 px (o carimbo de
  vários pontos) e os pontos dele são redondos, de borda mole.

## O portão da família (`familias::texturas::`)

`cada_carimbo_se_reconhece_pela_foto` (Halftone M1, Stars M2 e M3 SANGRAM; controlo verde; as contas de `texturas_foto.py` nas faixas de 40 e 200 px da cena `tamanhos`;
as 6 fotos do Krita/Rebelle medidas; só os aprovados se pintam, os outros são irmãos) · `os_em_estudo_nao_estao_na_fabrica`
(nenhum reprovado tem ficheiro, e cada um ainda gera o preset, com as suas imagens).

## Para a fila (coordenador)

1. **Mipmap/filtro da imagem da ponta a tamanhos pequenos** (Foliage 6 revisões, Noise Grit 3): a 5 px o carimbo de
   imagem parte-se em pontos; o Krita dá uma linha contínua. Motor médio (todas as rotas de Shape `Image`).
2. **Passo pelo diâmetro do dab à pressão** (Bokeh: o rasto denso de bolinhas na pressão baixa).
3. **Spray Can (J-2)**: duas populações (véu + gotas) e a emissão com o traço parado — por fazer; `particle_px` já serve.
4. **Splats/Spatter de aguarela** (J-3) — pede a orla escura/mistura da aguarela: dono `aguarela`.
5. As leis novas sem UI: `particle_px` (cartão Jitter) e `vies` (cartão Randomize Color).
6. A escala da bancada para carimbos espalhados (a largura ¼-do-máximo mede uma marca): exame aqui a escala 1.

# Oráculo janela 9 — o 2.º traço que COMEÇA dentro da tinta molhada e CORRE ao longo dela (Rebelle 8)

- **Data:** 2026-10-25, 14:25–14:32 (sessão sem ecrã: tela virtual 1920×1080, socket `ph2d-oraculo-j9`, DISPLAY `:2`;
  fechada no fim, só ela e o `wineserver` do prefixo).
- **App:** Rebelle 8.3.4 Pro (teste, Wine 11, `~/Apps/rebelle`), porta WebSocket (Motion IO) — eventos JSON, um por
  mensagem, nenhum clique real. Corre-se, não se lê fonte (CLAUDE.md §0.9).
- **Pincel/tinta:** `WATERCOLOR` · `Watercolor/Liner` (Max Size 300) — o mesmo da linha «Liner» da cena `sobre` em
  `docs/Painter/Pinceis/exames/aguarela.md` (Motor J5). Faixa 60 px (`size` 34,2 %), 2.º traço 40 px (`size` 26,1 %),
  pressão 1,0, passo 3 px, papel `Default/EX00 White Simple`, semente 7, tela 1000×640, foto a zoom 100 %.
  Cores: vermelho (191,0,18) · castanho (60,36,20) · azul (23,101,179).
- **Tempo:** os dois traços chegam em < 1 s (a faixa está MOLHADA quando o 2.º entra). Fotos: `_t0` logo após o
  2.º traço, `_t1s`, `_t5s`, `_sim240` (depois de `SIMULATION` 240), `_seco` (depois de `DRY`).
- **Ferramentas:** `ferramentas/` (`gera.py` = `dentro` e `sobre`; `gera_variantes.py` = as corrigidas; `corre.sh`
  = envio + fotos; `inicia.sh`/`sessao.sh` = a tela virtual com a guarda dos ecrãs do dono).

## As cenas (coordenadas da tela)

| cena | faixa vermelha (molhada) | 2.º traço |
|---|---|---|
| `dentro` (1.ª corrida) | (300,320)→(700,320) | castanho (433,330)→(850,330) |
| `dentro2` | igual, SET_BRUSH repetido | castanho (433,330)→(850,330) |
| `dentroazul` | igual, SET_BRUSH repetido | azul (433,330)→(850,330) |
| `largocast` | DOIS traços (300,305)→(700,305) e (300,335)→(700,335) = faixa de 84 px | castanho (433,325)→(850,325) |
| `largoazul` | idem | azul (433,325)→(850,325) |
| `sobre` (controle) | a receita da bateria: faixa seca y=470 (+SIM 240 +DRY), faixa molhada y=150 (60→940) | azuis verticais x=250, 500, 750 (o 3.º com passo 25) e um horizontal no papel y=310 |

⚠️ Na 1.ª `dentro` a faixa saiu com **37 px** em vez de 52 (o 1.º traço depois de uma troca pinta ao tamanho
anterior — lei já conhecida, `PH2D_REBELLE_REPETE_TROCA`): o castanho ficou metade no papel. As `dentro2`/`dentroazul`
repetem o SET_BRUSH (faixa 294–345 = 52 px), mas o traço de 40 px em y=330 ainda põe a borda de baixo dele na borda
da faixa; as `largo*` põem o traço INTEIRO dentro do vermelho — são estas que respondem.

## Medições (colunas; azul = B−R > −30 e fora do branco; castanho = R < 150; largura em px)

**`largoazul` / `largocast` — traço todo dentro (faixa 278–362):**

| foto | papel (x=800) | dentro do vermelho x=450…700 | entrada x=425 | x=415 (antes do início) | saída x=720 |
|---|---|---|---|---|---|
| t0 | 36 (307–342) | 34 (308–341) | — | — | 48 |
| t1s | 38 | 57 (306–362)* | 53 | — | 52 |
| t5s | 38 (306–343) | **61** (294–354) | 51 | 20 | 48 |
| sim240 | 38 | **56** (296–351), castanho 56–58 | 52 | 34 | 48 |

(*t1s do azul: a orla roxa conta até a borda da faixa por um instante.) Razão dentro ÷ papel: **1,47–1,53** (sim240),
1,6 (t5s). Alarga **para os DOIS lados por igual** (topo 308→296, base 341→351/352: +12 / +10 px), e a largura é
**constante** de x≈435 até x≈700 — ao longo de todo o caminho dentro do vermelho.

**`dentroazul` / `dentro2` — borda de baixo do traço na borda da faixa:** papel 38; dentro 45–46 (303/304–348),
razão 1,18–1,21, e o alargamento é **só para cima** (topo 311→303, +8 px; a base fica em 348, no papel) — só o lado
que está sobre tinta molhada abre.

**`sobre` (controle, mesma métrica, y=150):** t0 34 · t5s 50 · sim240 **55** contra 38 no papel = **1,45**
(a medição antiga dava 1,42 — reproduz). Com a métrica «distância ao vermelho > 45» a mancha mede 76 (t5s) / 65 (sim).

**O vermelho:** a borda da faixa NÃO se move (278–362 antes e depois, igual à zona sem 2.º traço em x=350); não há
vermelho empurrado para fora ao longo do caminho nem na entrada. Ele é **substituído** dentro do tubo: sobra
vermelho só entre o tubo e a borda (278–295 em cima, 352–362 em baixo).

## A resposta: **(b)** — um tubo espalhado ao longo de TODO o caminho dentro do vermelho, de largura constante

- **Largura:** ≈ 1,5 × a largura do traço no papel (56–58 contra 38), a MESMA razão da mancha do `sobre` (1,45).
  ⇒ A «mancha redonda» do `sobre` não é um evento de entrada: é este mesmo alargamento local, recortado pela faixa
  de 52 px que o traço atravessa (o comprimento dentro do molhado é ~1,4 × a largura, por isso parece um disco).
- **Entrada:** nenhuma mancha extra — a tampa inicial fica redonda e difusa, do tamanho do tubo (x=425: 52; x=435: 55),
  sem bolha maior que o tubo.
- **Bordas:** retas e paralelas ao caminho, mas **macias** (degradê de ~8–10 px, sem aresta dura nem orla escura);
  o miolo é uniforme.
- **Saída:** onde o traço sai do vermelho estreita em ~20–40 px até à largura do papel (x=720: 48), e leva um
  pouco de vermelho na cor por ~100 px (castanho no eixo: (107,63,54) dentro → (106,76,63) a x=720 → (122,106,95) a
  x=840; azul (85,113,170) → (97,150,203)).
- **Tempo:** o alargamento NÃO está no t0 (34–36, igual ao papel); cresce até ~5 s e o `SIMULATION` assenta-o um
  pouco mais estreito (61 → 56). O nosso motor põe tudo no instante do traço.
- **Lado:** só abre o lado do traço que está sobre tinta molhada (`dentro2`: +8 px em cima, 0 em baixo).

⇒ Para o nosso «Wet bloom»: a lei não é «um disco no meio da faixa à 1.ª entrada»; é **alargar o traço por onde
ele passa sobre tinta molhada de outra cor, píxel a píxel do caminho, ~1,5× (cada lado só se aquele lado estiver
molhado), com borda macia, substituindo o pigmento de baixo sem empurrar a borda da faixa**.

## Ficheiros

- `dentro_{t0,t1s,t5s,sim240,seco}.png`, `dentro2_*`, `dentroazul_*`, `largocast_*`, `largoazul_*`, `sobre_*` — tela 1000×640.
- `resumo_faixa_larga.png` (largocast t0/sim240; largoazul t0/t5s/sim240), `resumo_faixa_borda.png` (dentro2 e
  dentroazul t0/t5s/sim240), `resumo_dentro_1a_corrida.png` (a 1.ª `dentro`, faixa estreita).

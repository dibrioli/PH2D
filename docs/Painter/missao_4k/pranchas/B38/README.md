# B38 — Clone e Heal numa camada transparente

Em cada imagem: esquerda = antes, meio = depois, direita = o que mudou (quanto mais vermelho, mais mudou). Recorte central `h/2`.
A cena: camada nova transparente com uma faixa amarela MACIA (borda de alfa parcial) na metade esquerda, sobre branco, blocos de cor e papel Textured com relevo; 1080p e 4K.

- `*_clone_*`: Clone com Grain Checker em Tiled (o caminho por píxel do grão), fonte na borda de cima da faixa, traço vertical no vazio da direita. Antes: a tinta levada ao vazio ganhava uma orla CINZA-ESCURA (mistura recta com o `0,0,0,0` do vazio). Depois: a orla sai na cor da tinta, meio transparente. 1 327–1 704 píxeis mudam, todos mais claros, pior 59–64 níveis.
- `*_heal_*`: Heal (Inpaint) que atravessa a ponta da faixa. Antes: o buraco enchia-se de PRETO OPACO no lado vazio (o alfa era ignorado e o buraco escrito opaco). Depois: o vazio fica vazio e a tinta continua a tinta. 5 246–11 116 píxeis mudam, pior 240–255.

- `*_heal_opaco_*` (B38.3): o Heal da mesma faixa pintada DIRETO no fundo opaco. Antes: cada nível em f32 (SSD e voto em vírgula flutuante; a placa e a CPU podiam escolher outro remendo). Depois: SSD inteiro sobre RGBA8 pré-multiplicado, empate pelo menor índice de fonte, voto Q16 — a placa dá o mesmo byte que a CPU. O preenchimento é o mesmo à vista; mudam 982–2 897 píxeis, quase todos ±1–2, pior 6–77 nos cantos dos blocos (outro remendo igualmente bom num empate). `PRANCHA_B38_3=1` gera só estas.

Gerador: `prancha_b38_clone_e_heal_na_camada_transparente` (`ph2d-tool-painter`, `--lib -- --ignored`); o «antes» é o caminho velho posto de volta no mesmo processo (`prancha_b38_mistura_recta`, `prancha_b38_alfa_de_antes`, `prancha_b38_3_nivel_f32_de_antes` — o nível f32 guardado verbatim em `ph2d-inpaint/src/antigo/`).

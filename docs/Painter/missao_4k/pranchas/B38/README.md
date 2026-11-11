# B38 — Clone e Heal numa camada transparente

Em cada imagem: esquerda = antes, meio = depois, direita = o que mudou (quanto mais vermelho, mais mudou). Recorte central `h/2`.
A cena: camada nova transparente com uma faixa amarela MACIA (borda de alfa parcial) na metade esquerda, sobre branco, blocos de cor e papel Textured com relevo; 1080p e 4K.

- `*_clone_*`: Clone com Grain Checker em Tiled (o caminho por píxel do grão), fonte na borda de cima da faixa, traço vertical no vazio da direita. Antes: a tinta levada ao vazio ganhava uma orla CINZA-ESCURA (mistura recta com o `0,0,0,0` do vazio). Depois: a orla sai na cor da tinta, meio transparente. 1 327–1 704 píxeis mudam, todos mais claros, pior 59–64 níveis.
- `*_heal_*`: Heal (Inpaint) que atravessa a ponta da faixa. Antes: o buraco enchia-se de PRETO OPACO no lado vazio (o alfa era ignorado e o buraco escrito opaco). Depois: o vazio fica vazio e a tinta continua a tinta. 5 246–11 116 píxeis mudam, pior 240–255.

Gerador: `prancha_b38_clone_e_heal_na_camada_transparente` (`ph2d-tool-painter`, `--lib -- --ignored`); o «antes» é o caminho velho posto de volta no mesmo processo (`prancha_b38_mistura_recta`, `prancha_b38_alfa_de_antes`).

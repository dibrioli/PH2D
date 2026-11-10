# B16.2 — a borda dos traços antigos não muda mais quando você troca o tamanho do pincel

Em cada imagem: esquerda = antes, meio = depois, direita = o que mudou (branco = nada mudou).
A cena: seis pinceladas de aquarela (Watercolor Basic) lado a lado, na mesma sessão molhada, trocando o tamanho do pincel a cada uma, sobre branco, blocos de cor e papel Textured com relevo, em 1080p e 4K.
Resultado: nenhum pixel mudou nestas seis telas. A diferença só aparece com pincéis muito finos (3 a 5 px), em cerca de 90 pixels da borda de um traço antigo, que agora ficam com a cara do traço dele em vez da do pincel novo.
O ganho é de velocidade: ao soltar a caneta, a aquarela agora refaz só o traço novo também quando o tamanho muda (4K: ~119 ms → ~3 ms).

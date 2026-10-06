# Oráculo: o mundo do rapier quente × frio, a taça, as estabilizações, o salto e o rolamento

Doc 121 §9.20. Fora do workspace (`[workspace]` próprio). Corre com um `CARGO_TARGET_DIR` próprio:

    cargo build --release --offline
    target/release/oraculo_rapier <k> <passos> <iteracoes> <modo 0 quente|1 frio|2 quente+copia> <segundos> <taca fora 0|1> <estabilizacoes>
    target/release/sobrepostas   # duas peças que nascem sobrepostas: a velocidade que sobra
    target/release/salto         # o salto efectivo (0,88 numa caixa de face)
    target/release/rampa         # o contacto achatado numa rampa de 12° (recusado: acelera a bola)

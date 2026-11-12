for i in 1 2 3 4 5; do
  for P in "Filbert:wet" "omissão:wet" "omissão:impasto"; do
    [ "$P" = "omissão:impasto" ] && B="57b4ddbac7 cura12" || B="e81e56bb1f 57b4ddbac7 curaB cura12"
    [ $((i%2)) = 0 ] && B=$(echo $B | tr ' ' '\n' | tac | tr '\n' ' ')
    for b in $B; do N=1 OUT=/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/fb53f5ce-2f42-4773-b6f1-219f6399f510/scratchpad/r11/b$i bash /tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/fb53f5ce-2f42-4773-b6f1-219f6399f510/scratchpad/mede_tela.sh "$b|$P"; done
  done
done

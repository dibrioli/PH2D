for i in 1 2 3 4; do
  for c in "3840x2160|Filbert:wet" "3840x2160|omissão:wet" "3840x2160|omissão:impasto" "1920x1080|Filbert:wet"; do
    T=${c%%|*}; P=${c#*|}; [ $((i%2)) = 1 ] && O="e81e56bb1f 57b4ddbac7 curaB" || O="curaB 57b4ddbac7 e81e56bb1f"
    for b in $O; do TELA=$T N=1 OUT=/tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/fb53f5ce-2f42-4773-b6f1-219f6399f510/scratchpad/r10/b$i bash /tmp/claude-1000/-home-enio-Documentos-Projetos-PH2D/fb53f5ce-2f42-4773-b6f1-219f6399f510/scratchpad/mede_tela.sh "$b|$P"; done
  done
done

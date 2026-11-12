"""Põe o conteúdo de um ficheiro na área de transferência da Xvfb (sem xclip) e segura-a N s.

Uso: python3 -I clip.py <ficheiro> <segundos>  (o DISPLAY da Xvfb no ambiente).
"""
import sys
import tkinter

r = tkinter.Tk()
r.withdraw()
r.clipboard_clear()
r.clipboard_append(open(sys.argv[1]).read())
r.update()
r.after(int(sys.argv[2]) * 1000, r.destroy)
r.mainloop()

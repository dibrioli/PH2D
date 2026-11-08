import sys,numpy as np
from PIL import Image
f,out,k=sys.argv[1],sys.argv[2],int(sys.argv[3])
p=np.frombuffer(open(f,'rb').read(),np.uint8).reshape(880,2400,4)[:,k*800:(k+1)*800]
a=p[...,3:]/255; Image.fromarray((p[...,:3]*a+255*(1-a)).astype(np.uint8)).save(out)

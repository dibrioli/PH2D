import sys,numpy as np
from PIL import Image
def load(p):
    a=np.fromfile(p,np.uint8).reshape(880,2400,4).astype(np.float32)/255
    # composta sobre xadrez claro p/ ver o vazio
    yy,xx=np.mgrid[0:880,0:2400]; ch=np.where(((yy//20+xx//20)%2)==0,0.85,0.7)[...,None]
    rgb=a[...,:3]*a[...,3:]+ch*(1-a[...,3:])
    return (rgb*255).astype(np.uint8)
if __name__=='__main__':
    Image.fromarray(load(sys.argv[1])).resize((1200,440)).save(sys.argv[2])

import json
import os
import sys
from PIL import Image

def resize_op(im, tsize, opt):
    return im.resize(tsize);

def crop_op(im, tsize, opt):
    xo = -opt['xoff'];
    yo = -opt['yoff']
    return im.crop((xo, yo, tsize[0]+xo, tsize[1]+yo));

class SpritePacker:
    def __init__(self, file):
        self.fitopr = {'resize': resize_op, 'crop': crop_op, 'expand': None };
        with open(file) as fp:
            tobj = json.load(fp);
            self.ofile = tobj['output'];
            self.fext = tobj['file_extension'];
            self.anim_dirs = tobj['dirs'];
            self.frame_width = tobj['frame_width'];
            self.frame_height = tobj['frame_height'];
            self.fitop = tobj['force_frame'];
            if self.fitop['name'] not in self.fitopr:
                print("Invalid force frame operation!");
                exit();
        self.anim_files = [];
        for idr in self.anim_dirs:
            self.anim_files.append([f for f in os.listdir(idr) if f.endswith(self.fext)]);
        self.maxframes = max((len(s) for s in self.anim_files));
        self.img = Image.new('RGBA', (self.maxframes*self.frame_width, len(self.anim_dirs)*self.frame_height), (0,0,0,0));

    def __repr__(self):
        toprint = [];
        for attr, val in self.__dict__.items():
            if not attr.startswith("_"):
                toprint.append(attr + ": " + str(val));
        return "SpritePacker{\n\t" + ',\n\t'.join(toprint) + "\n}";

    def pack(self):
        tsize = (self.frame_width, self.frame_height);
        for i in range(len(self.anim_files)):
            cur = self.anim_files[i];
            cdr = self.anim_dirs[i];
            for j in range(len(cur)):
                im = Image.open(cdr+"/"+cur[j]);
                if im.size != tsize:
                    im = self.fitopr[self.fitop['name']](im, tsize,self.fitop);
                self.img.paste(im, (j*self.frame_width, i*self.frame_height));
                print("Packed", cdr+"/"+cur[j], "successfully");
            print("Dir", cdr, "packed completely");    

    def finish(self):
        self.img.show();
        self.img.save(self.ofile);

if __name__ == '__main__':
    sp = SpritePacker(sys.argv[1]);
    print(sp);
    sp.pack();
    sp.finish();
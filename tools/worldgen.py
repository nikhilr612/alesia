## World binary file format:
## 0. Magic: 0xFADE00FF
## 1. World size tuple(u8,u8);
## 2. Main tile data (single layer)
## ---- Padding --- [6 bytes, 0x00]
## 3. Static Game Objects encoded as [0xfe, 0xed, o_type ,id, worldx, worldy]
import sys

magic = bytes([0xfa, 0xde, 0x00, 0xff]);
contread = bytes([0xfe, 0xed]);

class WorldFile:
	def __enter__(self):
		return self

	def __init__(self, fname, width, height):
		self.file = open(fname, 'wb');
		self.file.write(magic);
		self.width = width;
		self.height = height;
		self.file.write(bytes([self.width, self.height]));

	def put_tilelist(self, ls):
		if ls == None:
			self.file.write(bytes([0,0]));
		else:
			self.file.write(bytes([0xda,0xd7, len(ls)]));
			self.file.write(bytes(ls));

	def put_bytes(self, bs):
		self.file.write(bytes([len(bs)]));
		self.file.write(bytes(bs));

	def put_string(self, s):
		l = len(s);
		self.file.write(bytes([(l & 0xff00) >> 8, l & 0xff]));
		self.file.write(bytes(s.encode('ascii')));

	def dump_tdata(self, tdata):
		self.file.write(bytearray(tdata));

	def pad(self, n):
		self.file.write(bytes(n));

	def put_object(self, tid, t, tx, ty):
		self.file.write(contread);
		self.file.write(bytes([t, tid, tx, ty]));

	def __exit__(self, t, v, trac):
		self.file.close();

class IdempotentIdxMap:
	def __init__(self, d):
		self.d = d;

	def get(self, key, df=None):
		return self.d.get(key, df) if self.d is not None else None;

	def __repr__(self):
		return self.d.__repr__();

	def __getitem__(self, key):
		if self.d is None or key not in self.d:
			return key
		else:
			return self.d[key]

def main(name, w, h, m=None):
	m = IdempotentIdxMap(m);
	with WorldFile(name + ".alw", w, h) as w, open(name+"_map.csv",'r') as ter, open(name+"_bldg.csv") as bu:
		w.put_tilelist(m.get('blocked'));
		w.put_bytes(m.get('heal', []));
		w.put_bytes(m.get('damage', []));
		# Terrain data;
		for l in ter:
			i = [m[int(s.strip())] for s in l.split(",")];
			w.dump_tdata(i);
		w.put_string(m.get('title', name));
		w.put_string(m.get('intro', ''));
		w.put_string(m.get('victory', 'You won'));
		w.put_string(m.get('defeat', 'You lost'));
		# Objects
		ty = 0;
		for l in bu:
			tx = 0;
			for s in l.split(","):
				s = int(s.strip());
				if s != -1:
					s, t = m[(s,0)];
					w.put_object(s, t, tx, ty);
				tx += 1;
			ty += 1;

if __name__ == "__main__":
	if len(sys.argv) < 4:
		print("Insufficient arguments");
		exit();
	w,h = int(sys.argv[2]), int(sys.argv[3]);
	if len(sys.argv) == 4:
		main(sys.argv[1], w, h);
	elif len(sys.argv) == 5:
		from ast import literal_eval
		from pathlib import Path
		m = literal_eval(Path(sys.argv[4]).read_text());
		main(sys.argv[1], w, h, m);
	else:
		print("Unrecognized option(s): ", sys.argv[5:]);

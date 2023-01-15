//! Creates a Window, initializes Raylib.
//! # Example
//! ```
//! use alesia::display::Display;
//! use alesia::utils::ResourceSet;
//! use alesia::world::World;
//! let d = Display::new_s(940, 640, "Display test");
//! d.begin_s(ResourceSet::new(), World::blank()); // Blank for example's sake
//! ```

use std::cmp::Ordering;

const BOX_STATICS: bool = false;
const RENDER_FILTER_GAP: i32 = 2;
const GRAYCOL: Color = Color {
        r: 200,
        g: 150,
        b: 200,
        a: 150,
    };
const XOFF: f32 = 20.0;
const HPREC: Rectangle = Rectangle {
	x: XOFF,
	y: 40.0,
	width: 120.0,
	height: 20.0
};

const TITLE_OFF: f32 = 75.0;

const INTRO_OFF: f32 = 165.0;

const PROHIBITED_TCOL: Color = Color {
	r: 190,
	g: 116,
	b: 34,
	a: 127,
};

const ALLOWED_TCOL: Color = Color {
	r: 244,
	g: 180,
	b: 112,
	a: 127,
};

const HEAL_TCOL: Color = Color {
	r: 185, g: 252, b: 61, 
	a: 127
};

const DAMAGE_TCOL: Color = Color {
	r: 121, g: 14, b: 171,
	a: 127
};

const ENEMY_TCOL: Color = Color {
	r: 200, g: 36, b: 36, a: 127
};

const PLAYER_TCOL: Color = Color {
	r: 44, g: 72, b: 224, a: 127
};
use raylib::text::Font;
use raylib::math::Rectangle;
use raylib::texture::Texture2D;
use raylib::prelude::RaylibTexture2D;
use raylib::drawing::RaylibDrawHandle;
use raylib::drawing::RaylibMode2D;
use crate::utils::StateListener;
use crate::input::InputHandler;
use raylib::RaylibHandle;
use raylib::ffi::KeyboardKey;
use raylib::math::Vector2;
use raylib::camera::Camera2D;
use crate::world::World;
use crate::utils::ResourceSet;
use raylib::prelude::Color;
use raylib::prelude::RaylibDraw;
use raylib::prelude::RaylibMode2DExt;
use raylib::prelude::RaylibAudio;

/// A plain struct with fields for width, height and title of window.
pub struct Display {
	/// The width of the window in pixels
	width: i32,
	/// The height of the window in pixels
	height: i32,
	/// The target fps
	fps: u32,
	/// Flag to enable or disable vsync
	vsync: bool,
	/// Title of the window
	title: String,
	/// Master Volume
	mvolume: f32,
	/// Clear colour
	col: Color
}

struct Renderable<'a> {
	wpos: Vector2,
	spos: Vector2,
	tex: &'a Texture2D,
	reg: Option<Rectangle>,
	tint: Color,
	is_static: bool
}

impl Renderable<'_> {
	fn new_static(tex: &Texture2D, x: i32, y: i32, wx: i32, wy: i32) -> Renderable {
		Renderable {
			wpos: Vector2::new(wx as f32, wy as f32),
			spos: Vector2::new(x as f32, y as f32),
			tex: tex,
			reg: None,
			tint: Color::WHITE,
			is_static: true
		}
	}

	fn new_static_texreg(tex: &Texture2D, spos: Vector2, wx: i32, wy: i32, rec: Rectangle) -> Renderable {
		Renderable {
			wpos: Vector2::new(wx as f32, wy as f32),
			spos: spos,
			tex: tex,
			reg: Some(rec),
			tint: Color::WHITE,
			is_static: true	
		}
	}

	fn new_unit(tex: &Texture2D, wpos: Vector2, spos: Vector2, rec: Rectangle, tint: Color) -> Renderable {
		Renderable {
			tex: tex,
			wpos: wpos,
			spos: spos,
			reg: Some(rec),
			tint: tint,
			is_static: false
		}
	}

	fn cmp(r1: &Renderable, r2: &Renderable) -> Ordering{
		let diff = r1.wpos - r2.wpos;
		if diff.x < 0.0 {
			Ordering::Less
		} else if diff.x > 0.0 {
			Ordering::Greater
		} else {
			if diff.y > 0.0 {
				Ordering::Greater
			} else if diff.y < 0.0 {
				Ordering::Less
			} else {
				Ordering::Equal
			}
		}
	}
}

impl Display {
	/// Constructor method. Returns Display struct with specified width, height, title, target fps, vsync and clear colour.
	pub fn new(width: i32, height: i32, fps: u32, vsync: bool, title: &str, col: Color, mvolume: f32) -> Display{
		Display {
			width: width,
			height: height,
			title: title.to_string(),
			fps: fps,
			vsync: vsync,
			mvolume: mvolume,
			col: col
		}
	}

	/// Constructor method. Returns display struct with specified width, height, title, 60fps, vsync-enabled and black background.
	pub fn new_s(width: i32, height: i32, title: &str) -> Display {
		Display::new(width, height, 60, true, title, Color::BLACK, 1.0)
	}

	/// Overload for `Display.begin`, uses default state listener, which ignores all notifications.
	pub fn begin_s(self, rs: ResourceSet, w: World) {
		self.begin(rs, w, StateListener::new());
	}

	/// Begin the draw-update loop.
	pub fn begin(self, mut rs: ResourceSet, mut w: World, mut sl: StateListener) {
		// Initialization
		let mut rb = raylib::init();
		let mut cam = Camera2D {
			target: Vector2::new(0.0,0.0),
			offset: Vector2::new(0.0,0.0),
			rotation: 0.0,
			zoom: 1.0
		};
		rb.size(self.width, self.height).title(&self.title);
		if self.vsync {
			rb.vsync();
		}
		let (mut rl, thread) = rb.build();
		rl.set_exit_key(Some(KeyboardKey::KEY_NULL));
		let mut rlau = RaylibAudio::init_audio_device();
		// Load resources
		println!("info [alesia/display.rs] : Loading resources from resource set.");
		crate::utils::load_all(&mut rs, &mut rl, &thread);
		rl.set_target_fps(self.fps);
		rlau.set_master_volume(self.mvolume);
		sl.notify_init();

		if let Some(a) = rs.get_music(w.bgm_id) {
			rlau.play_music_stream(a);
		}

		let mut is = InputHandler::new();

		// Main loop
		while !rl.window_should_close() {
			let r = rl.get_mouse_position();
			// Draw scope. All rendering occurs here.
			{	
				let mut d = rl.begin_drawing(&thread);
				d.clear_background(self.col);
				// Camera scope.
				{
					_man_cam(&mut cam, &w);
					let mut d = d.begin_mode2D(cam);
					self._draw_world(&mut d, &w, &rs, &is, &r, &mut rlau);
				}
				// HUD Goes here.
				d.draw_fps(0,0);

				if is.show {
					let rtex = rs.get_texture(0xf2);
					d.draw_texture(rtex, 0,0, Color::WHITE);
					d.draw_rectangle_lines_ex(HPREC, 3, Color::BLACK);
					let (h, mh) = crate::world::_unit_health(&w, is.cur_id);
					let hfrac = (h as f32) / (mh as f32);
					let width = (116.0*(hfrac)) as i32;
					let col = Color {
						r: (255.0*(1.0-hfrac)) as u8,
						g: (255.0*hfrac) as u8,
						b: 0,
						a: 255
					};
					d.draw_rectangle((XOFF as i32)+2, 42, width, 15, col);
					d.draw_text_ex(rs.get_default_font(), &format!("HP: {} / {}", h, mh), Vector2::new(XOFF,64.0), 22.0, 1.0, Color::BLACK);
					d.draw_text_ex(rs.get_default_font(), &format!("{}", is), Vector2::new(XOFF,20.0), 22.0, 1.2, Color::BLACK);
					if is.show_info {
						let tex = rs.get_texture(0xf8);
						d.draw_texture(tex, 0, rtex.height(), Color::WHITE);
						match crate::world::_unit_info(&w, is.cur_id) {
							Some(text) => {
								d.draw_text_ex(rs.get_default_font(), text, Vector2::new(XOFF, rtex.height() as f32 + 20.0), 22.0, 1.0, Color::BLACK);
							},
							None => {}
						}
						self._draw_minimap(&mut d, &w);
					}
				}
				if is.get_state() == 7 {
					self._draw_window(0xf5, w.map_title(), w.intro_text(), &rs, &mut d);
				} else if is.get_state() == 5 {
					self._draw_window(0xf6, "Victory is thine", w.victory_text(), &rs, &mut d);
				} else if is.get_state() == 6 {
					self._draw_window(0xf6, "'Tis defeat", w.defeat_text(), &rs, &mut d);
				}
			}
			// Save screenshot
			if rl.is_key_pressed(KeyboardKey::KEY_S) && rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
				rl.take_screenshot(&thread,"screen.png");
			}
			// Camera controls are always active.
			_cam_control(&mut w, &rl);
			is.handle(&mut w, &rl, &mut sl, &mut rlau, &mut rs);
			if let Some(a) = rs.get_music(w.bgm_id) {
				rlau.update_music_stream(a);
			}
		}
	}

	fn is_pos_offscreen(&self, v: &Vector2, w: &World, gap: i32) -> bool {
		let (cx, cy) = w.get_cpos();
		let gap = gap as f32;
		let (xmin, xmax) = (cx - gap, gap + cx + self.width as f32);
		let (ymin, ymax) = (cy - gap, gap +cy + self.height as f32);
		if v.x < xmin || v.x > xmax || v.y < ymin || v.y > ymax {
			true
		} else {
			false
		}
	}
	
	fn is_ipos_offscreen(&self, x: i32, y: i32, w: &World, gap: i32) -> bool {
		let (cx, cy) = w.get_cpos();
		let (cx, cy) = (cx as i32, cy as i32);
		let (xmin, xmax) = (cx-gap, cx + self.width + gap);
		let (ymin, ymax) = (cy-gap, cy + self.height + gap);
		if x < xmin || x > xmax || y < ymin || y > ymax {
			true
		} else {
			false
		}
	}

	fn _draw_minimap(&self, d: &mut RaylibDrawHandle<'_>, world: &World) {
		let total_side = self.width / 4;
		let xoff = self.width - total_side;
		let (w, h) = world.map_size();
		let side: i32 = total_side / w as i32;
		for i in 0..w as i32 {
			for j in 0..h as i32 {
				let rx: i32 = xoff + i*side;
				let ry: i32 = j*side;
				d.draw_rectangle(rx, ry, side, side, _tile_colour(i, j, world))
			}
		}
		for (_id, u) in &world.units {
			let (i,j) = (u.wpos.x as i32, u.wpos.y as i32);
			let (cx, cy) = (xoff + i*side + side/2, j * side + side/2);
			if u.player {
				d.draw_ellipse(cx, cy, (side/3) as f32, (side/3) as f32, PLAYER_TCOL);
			} else {
				d.draw_ellipse(cx, cy, (side/3) as f32, (side/3) as f32, ENEMY_TCOL);
			}
		}
	}

	fn _draw_window(&self, id: u8, title: &str, body: &str, rs: &ResourceSet, d: &mut RaylibDrawHandle<'_>) {
		let tex = rs.get_texture(id);
		let corner = Vector2::new(0.5*(self.width - tex.width()) as f32, 0.5*(self.height - tex.height()) as f32);
		d.draw_texture_v(tex, corner, Color::WHITE);
		let fnt = rs.get_default_font();
		self._draw_text_centered(d, fnt, body, 23.0, 1.0, INTRO_OFF + corner.y);
		self._draw_text_centered(d, fnt, title, 32.0, 1.0, TITLE_OFF + corner.y);
	}

	fn _draw_tile(&self, w: &World, mut rec: Rectangle, tset: &Texture2D, tx: i32, ty: i32, d: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>, n: i32) {
		let (wi, hi) = w.map_size();
		if tx >= 0 && tx < wi as i32 && ty >= 0 && ty < hi as i32 {
			let (rpos, pos) = crate::world::prep_tiledraw(w, tx, ty, n);
			rec.x = rpos.x;
			rec.y = rpos.y;
			d.draw_texture_rec(tset, rec, pos, Color::WHITE);
		}
	}

	fn _draw_text_centered(&self, d: &mut RaylibDrawHandle<'_>, fnt: &Font, text: &str, fntsize: f32, spacing: f32, yoff: f32) {
		let s = raylib::core::text::measure_text_ex(fnt, text, fntsize, spacing);
		let pos = Vector2::new(0.5*((self.width as f32) - s.x), yoff);
		d.draw_text_ex(fnt, text, pos, fntsize, spacing, Color::BLACK);
	}

	#[inline]
	fn _is_rec_offscreen(&self, w: &World, pos: Vector2, width: f32, height: f32) -> bool {
		if !self.is_pos_offscreen(&pos, w, RENDER_FILTER_GAP) {
			return false;
		}
		if !self.is_pos_offscreen(&(pos + Vector2::new(width, 0.0)), w, RENDER_FILTER_GAP) {
			return false;
		}
		if !self.is_pos_offscreen(&(pos + Vector2::new(0.0, height)), w, RENDER_FILTER_GAP) {
			return false;
		}
		if !self.is_pos_offscreen(&(pos + Vector2::new(width, height)), w, RENDER_FILTER_GAP) {
			return false;
		}
		return true;
	}

	#[inline]
	fn _draw_world(&self, d: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>, w: &World, rs: &ResourceSet, is: &InputHandler, r: &Vector2, rlau: &mut RaylibAudio) {
		if w.show_map() {
			let tset = rs.get_texture(0xf0);
			let tsize = w.get_tile_size();
			let rec = Rectangle::new(0.0, 0.0, tsize.0 as f32, tsize.1 as f32);
			let n = tset.width() / tsize.0;
			let xl = self.width/tsize.0 + 1;
			let yl = self.height/tsize.1 + 1;
			for gx in -1..xl {
				for gy in  -1..yl {
					let wp = (((gx as f32) + 0.5) * tsize.0 as f32, ((gy as f32) + 0.5) * tsize.1 as f32);
					let (tx, ty) = crate::world::tile_at(w, wp.0, wp.1);
					if gx == 0 {
						self._draw_tile(w, rec, tset, tx-1, ty, d, n);
					}
					if gy == yl {
						self._draw_tile(w, rec, tset, tx, ty+1, d, n);
					}
					self._draw_tile(w, rec, tset, tx, ty-1, d, n);
					self._draw_tile(w, rec, tset, tx, ty, d, n);
				}
			}
		}
		let mut renderables = vec![];
		for (_id, sp) in &w.units {
			let (tid, rec, pos, sif) = sp.prep_draw(w);
			if self._is_rec_offscreen(w, pos, rec.width, rec.height) {
				continue
			}
			if let Some((id, lp)) = sif {
				let s = rs.get_sound(id);
				if sp.nascent_state() || lp{
					if !rlau.is_sound_playing(s) {rlau.play_sound(s)};
				}
			}
			let rcol = if is.show && *_id == is.cur_id {Color::YELLOW} 
						else if is.get_state() == 0 && is.is_frozen(&*_id) {Color::GRAY}
						else {sp.get_tint()};
			renderables.push(Renderable::new_unit(rs.get_texture(tid), sp.wpos, pos, rec, rcol));
		}
		for st in &w.statics {
			let (tid, x, y) = st.prep_draw(w);
			// if self.is_ipos_offscreen(x, y, w, RENDER_FILTER_GAP) {
			// 	continue;
			// }
			//let (mut bw, mut bh) = (0,0);
			if rs.is_texture_region(tid) {
				let (tex, rec) = rs.get_texture_region(tid);
				let pos = Vector2::new(x as f32, y as f32 - rec.height + w.get_tile_size().1 as f32);
				//d.draw_texture_rec(tex, rec, pos, Color::WHITE);
				if self._is_rec_offscreen(w, pos, rec.width, rec.height) {
					continue;
				}
				renderables.push(Renderable::new_static_texreg(tex, pos, st.wx, st.wy, *rec));
			} else {
				let tex = rs.get_texture(tid);
				let (x,y) = (x, y -tex.height() + w.get_tile_size().1);
				// d.draw_texture(tex, x, y, Color::WHITE);
				if self._is_rec_offscreen(w, Vector2::new(x as f32,y as f32), tex.width() as f32, tex.height() as f32) {
					continue;
				}
				renderables.push(Renderable::new_static(tex, x, y, st.wx, st.wy));
			}
			//if BOX_STATICS {
			//	d.draw_rectangle_lines(x, y, bw, bh, Color::WHITE);
			//}
		}
		renderables.sort_by(Renderable::cmp);	// Sort draw by world position, render farthest first.
		//println!("Number of renderables is {}", renderables.len());
		let (mut bw, mut bh) = (0, 0);
		for rd in renderables {
			if let Some(rec) = rd.reg {
				bw = rec.width as i32;
				bh = rec.height as i32;
				d.draw_texture_rec(rd.tex, rec, rd.spos, rd.tint)
			} else {
				bw = rd.tex.width();
				bh = rd. tex.height();
				d.draw_texture_v(rd.tex, rd.spos, rd.tint)
			}
			if BOX_STATICS && rd.is_static {
				d.draw_rectangle_lines(rd.spos.x as i32, rd.spos.y as i32, bw, bh, Color::WHITE);
			}
		}
		for proj in &w.projectiles {
			let (st, en) = proj._prep_draw(w);
			//println!("Drawing line from: {:?}, to: {:?}", st, en);
			d.draw_line_ex(st, en, 1.5, Color::BLACK);
		}
		// Select Tile.
		if is.show {
			let t = crate::world::tile_at(w,r.x, r.y);
			let u = crate::world::wots(w,t.0, t.1);
			d.draw_texture(rs.get_texture(0xf1), u.0, u.1, Color::WHITE);
			if is.get_state() == 1 {
				let (sx, ex, sy, ey) = is._boxrange();
				for y in sy..=ey {
					for x in sx..=ex {
						let t = is.tile_shade(x, y);
						if !crate::world::tile_type_at(w, x, y).allowed() {
							continue;
						}
						if t != 0 {
							let u = crate::world::wots(&w, x, y);
							d.draw_texture(rs.get_texture(0xf2 + t), u.0, u.1, Color::WHITE);
						}
					}	
				}
			} else if is.get_state() == 4{
				let (sx, ex, sy, ey) = is._atkrange();
				for y in sy..=ey {
					for x in sx..=ex {
						let v = is._inrange(x, y);
						if v == 1 {
							let u = crate::world::wots(w, x, y);
							d.draw_texture(rs.get_texture(0xf4), u.0, u.1, Color::WHITE);
						} else if v == -1 {
							let (tid, rec, pos) = w.units.get(&is.cur_id).unwrap()._stand_frame(w, x, y);
							d.draw_texture_rec(rs.get_texture(tid), rec, pos, GRAYCOL);
						}
					}	
				} 
			}
		}
	}
}

#[inline]
fn _man_cam(cam: &mut Camera2D, w: &World) {
	let (cx, cy) = w.get_cpos();
	cam.target.x = cx;
	cam.target.y = cy;
	cam.offset.x = w.coff.0;
	cam.offset.y = w.coff.1;
}

#[inline]
fn _cam_control(w: &mut World, rl: &RaylibHandle) {
	if rl.is_key_down(KeyboardKey::KEY_LEFT) {
		w.cam_wx -= rl.get_frame_time() * 4.0;
	}
	if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
		w.cam_wx += rl.get_frame_time() * 4.0;
	}
	if rl.is_key_down(KeyboardKey::KEY_UP) {
		w.cam_wy -= 4.0 * rl.get_frame_time();
	}
	if rl.is_key_down(KeyboardKey::KEY_DOWN) {
		w.cam_wy += 4.0 * rl.get_frame_time();
	}
}

fn _tile_colour(x: i32, y: i32, w: &World) -> &Color {
	let tty = crate::world::tile_type_at(w, x, y);
	match tty {
		crate::world::TileType::Prohibited => &PROHIBITED_TCOL,
		crate::world::TileType::Allowed => &ALLOWED_TCOL,
		crate::world::TileType::Heal => &HEAL_TCOL,
		crate::world::TileType::Damage => &DAMAGE_TCOL
	}
}
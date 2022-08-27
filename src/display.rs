//! Creates a Window, initializes Raylib.
//! # Example
//! ```
//! use alesia::display::Display;
//! use alesia::utils::ResourceSet;
//! use alesia::world::World;
//! let d = Display::new_s(940, 640, "Display test");
//! d.begin_s(ResourceSet::new(), World::blank()); // Blank for example's sake
//! ```

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
	/// Clear colour
	col: Color
}

impl Display {
	/// Constructor method. Returns Display struct with specified width, height, title, target fps, vsync and clear colour.
	pub fn new(width: i32, height: i32, fps: u32, vsync: bool, title: &str, col: Color) -> Display{
		Display {
			width: width,
			height: height,
			title: title.to_string(),
			fps: fps,
			vsync: vsync,
			col: col
		}
	}

	/// Constructor method. Returns display struct with specified width, height, title, 60fps, vsync-enabled and black background.
	pub fn new_s(width: i32, height: i32, title: &str) -> Display {
		Display::new(width, height, 60, true, title, Color::BLACK)
	}

	/// Overload for `Display.begin`, uses default state listener, which ignores all notifications.
	pub fn begin_s(self, rs: ResourceSet, w: World) {
		self.begin(rs, w, StateListener::new());
	}

	/// Begin the draw-update loop.
	pub fn begin(self, mut rs: ResourceSet, mut w: World, sl: StateListener) {
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
		let mut rlau = RaylibAudio::init_audio_device();
		// Load resources
		println!("info [alesia/display.rs] : Loading resources from resource set.");
		crate::utils::load_all(&mut rs, &mut rl, &thread);
		rl.set_target_fps(self.fps);
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
					d.draw_texture(rs.get_texture(0xf2), 0,0, Color::WHITE);
					d.draw_text_ex(rs.get_default_font(), &format!("{}", is), Vector2::new(5.0,20.0), 22.0, 1.2, Color::BLACK);
				}
			}
			// Save screenshot
			if rl.is_key_pressed(KeyboardKey::KEY_S) && rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
				rl.take_screenshot(&thread,"screen.png");
			}
			// Camera controls are always active.
			_cam_control(&mut w, &rl);
			is.handle(&mut w, &rl, &sl, &mut rlau, &mut rs);
			if let Some(a) = rs.get_music(w.bgm_id) {
				rlau.update_music_stream(a);
			}
		}
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
		for st in &w.statics {
			let (tid, x, y) = st.prep_draw(&w);
			d.draw_texture(rs.get_texture(tid), x, y, Color::WHITE)
		}
		for (_id, sp) in &w.units {
			let (tid, rec, pos, sif) = sp.prep_draw(&w);
			if let Some((id, lp)) = sif {
				let s = rs.get_sound(id);
				if sp.nascent_state() || lp{
					if !rlau.is_sound_playing(s) {rlau.play_sound(s)};
				}
			}
			d.draw_texture_rec(rs.get_texture(tid), rec, pos, if is.show && *_id == is.cur_id {Color::YELLOW} else {sp.get_tint()});
		}
		// Select Tile.
		if is.show {
			let t = crate::world::tile_at(&w,r.x, r.y);
			let u = crate::world::wots(&w,t.0, t.1);
			d.draw_texture(rs.get_texture(0xf1), u.0, u.1, Color::WHITE);
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
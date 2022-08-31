//! Handles User Input.

use raylib::audio::RaylibAudio;
use crate::utils::ResourceSet;
use std::collections::HashSet;
use crate::world::World;
use raylib::RaylibHandle;
use crate::utils::StateListener;
use raylib::math::Vector2;
use raylib::ffi::MouseButton;
use raylib::ffi::KeyboardKey;
use std::fmt;

#[derive(Debug)]
/// An enum containing all possible orders followed by units.
pub enum Order {
	/// Order to move unit with id, to tile position.
	MOVE(u8, i32, i32),
	/// Order to make unit with id, attack from tile position.
	ATTACK(u8, u8, i32, i32)
}

/// Plain struct to store state variables related to user input.
pub struct InputHandler {
	/// The unsigned byte identifier of the unit currently selected.
	pub cur_id: u8,
	movn: u8,
	range: u8,
	last_tile: (i32, i32),
	uname: String,
	state: u8,	// 0 - select unit, 2 - select movement tiles and attack tile, 1 - play
	ovec: Vec<Order>,
	frozen: HashSet<u8>,
	isplrsel: bool,
	/// Flag to show or hide UI.
	pub show: bool
}

impl InputHandler {
	/// Constructor method.
	pub fn new() -> InputHandler {
		InputHandler {
			cur_id: 0,
			movn: 0,
			range: 0,
			uname: "".to_string(),
			last_tile: (0, 0),
			state: 0,
			ovec: vec![],
			frozen: HashSet::new(),
			isplrsel: false,
			show: false,
		}
	}

	/// Method invoked during game loop to handle key and mouse inputs.
	pub fn handle(&mut self, w: &mut World, rl: &RaylibHandle, sl: &StateListener, rlau: &mut RaylibAudio, rs: &mut ResourceSet) {
		if self.state == 2 || self.state == 3 {
			self.ovec.retain(|o| {
				crate::world::order_pending(o,w)
			});
			let delta = rl.get_frame_time();
			let mut torem = Vec::new();
			for (_id, u) in &mut w.units {
				u.update(&w.unit_types, delta);
				if u.health <= 0.0 {
					torem.push(*_id);
				}
			}
			for e in torem {let _ = &mut w.units.remove(&e);}
			if self.ovec.len() == 0 {
				self.state = 0;
			}
			return;
		}
		if rl.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
			if self.state == 0 {
				self.select_unit(w, rl.get_mouse_position());
				if self.state == 1{
					rlau.play_sound(rs.get_sound(0xff));
				}
 			} else if self.state == 1 {
 				self.select_move_tile(w, rl.get_mouse_position());
 			}
		}
		if rl.is_key_pressed(KeyboardKey::KEY_D) {
			self.confirm_move();
		}
		if rl.is_key_pressed(KeyboardKey::KEY_E) {
			if self.ovec.len() != 0 {
				self.frozen.remove(&self.cur_id);
				self.ovec.clear();
			}
			self.reset();
		}
		if rl.is_key_pressed(KeyboardKey::KEY_ENTER) && self.state == 0 {
			self.state = 3;
			self.frozen.clear();
			self.ovec.clear();
			let m = w.bgm_id;
			sl.notify_turn(w, &mut self.ovec);
			if w.bgm_id != m {
				if let Some(a) = rs.get_music(m) {
					rlau.stop_music_stream(a);
				}
				if let Some(a) = rs.get_music(w.bgm_id){
					rlau.play_music_stream(a);
				}
			}
		}
	}

	fn reset(&mut self) {
		self.state = 0;
		self.show = false;
		self.cur_id = 0;
		self.movn = 0;
		self.range = 0;
		self.uname = String::from("");
		self.isplrsel = false;
		println!("Following orders were issued {:?}", self.ovec);
	}

	fn select_unit(&mut self, w: &World, mpos: Vector2) {
		let (tx, ty) = crate::world::tile_at(w, mpos.x, mpos.y);
		for (id, u) in &w.units {
			if (u.wpos.x as i32) == tx && (u.wpos.y as i32) == ty {
				self.last_tile = (tx, ty);
				self.cur_id = *id;
				let t = crate::world::_guinfo(w, u);
				self.movn = t.0;
				self.range = t.1;
				self.uname = t.2;
				self.isplrsel = t.3;
				self.show = true;
				self.state = 1;
				break;
			}
		}
	}

	#[inline]
	fn confirm_move(&mut self){
		self.frozen.insert(self.cur_id);
		self.reset();
		self.state = 2;
	}

	fn select_move_tile(&mut self, w: &World, mpos: Vector2) {
		if self.isplrsel && !self.frozen.contains(&self.cur_id) {
			let (tx, ty) = crate::world::tile_at(w, mpos.x, mpos.y);
			if !crate::world::tile_perm_at(w, tx, ty) {
				return;
			}
			for (i, u) in &w.units {
				if (u.wpos.x as i32) == tx && (u.wpos.y as i32) == ty {
					if *i == self.cur_id {
						self.confirm_move();
					} else if crate::world::is_tile_atrange((tx,ty),self.last_tile,self.range){
						self.ovec.push(Order::ATTACK(self.cur_id, *i, self.last_tile.0, self.last_tile.1));
						self.confirm_move();
					}
					return;
				}
			}
			if self.movn >= 1 && crate::world::is_tile_atrange((tx,ty), self.last_tile, 1){
				self.ovec.push(Order::MOVE(self.cur_id, tx, ty));
				self.last_tile = (tx,ty);
				self.movn -= 1;
			}
			if self.movn == 0 {
				self.confirm_move();
			}
		} else {
			self.state = 0;
		}
	}
}

impl fmt::Display for InputHandler {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{} id:{}\nT:{}\nLT:{:?}\nF:{:?}\ns:{}", self.uname, self.cur_id, self.movn, self.last_tile, self.frozen, self.state)
	}
}
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
	ATTACK(u8, u8, i32, i32),
	/// Order to declare player victory.
	VICTORY,
	/// Order to modify a unit's health relative to maximum health.
	MutHealthR(u8, f32),
	/// Order to modify a unit's health by incrementing a non-relative value.
	MutHealthA(u8, f32),
	/// Order to declare player defeat.
	DEFEAT
}

/// Plain struct to store state variables related to user input.
#[derive(Debug)]
pub(crate) struct InputHandler {
	/// The unsigned byte identifier of the unit currently selected.
	pub cur_id: u8,
	movn: u8,
	movn_i: u8,
	range: u8,
	last_tile: (i32, i32),
	cur_upos: (i32, i32),
	uname: String,
	/// 0 - player turn.
	/// 1 - player - unit selected, select movement tile.
	/// 4 - player - movement selected, select attack tile.
	/// 2 - player turn ends; player units move
	/// 3 - enemy turn.
	/// 5 - player victory.
	/// 6 - player defeat.
	/// 7 - intro.
	state: u8,
	ovec: Vec<Order>,
	frozen: HashSet<u8>,
	isplrsel: bool,
	/// Flag to show or hide UI.
	pub show: bool,
	/// Flag to show or unit type information.
	pub show_info: bool
}

impl InputHandler {
	/// Constructor method.
	pub fn new() -> InputHandler {
		InputHandler {
			cur_id: 0,
			movn: 0,
			movn_i: 0,
			range: 0,
			uname: "".to_string(),
			last_tile: (0, 0),
			cur_upos: (0,0),
			state: 7,
			ovec: vec![],
			frozen: HashSet::new(),
			isplrsel: false,
			show: false,
			show_info: false
		}
	}

	/// Method invoked during game loop to handle key and mouse inputs.
	pub fn handle(&mut self, w: &mut World, rl: &RaylibHandle, sl: &StateListener, rlau: &mut RaylibAudio, rs: &mut ResourceSet) {
		if self.state == 2 || self.state == 3 {
			let mut next_state = None;
			self.ovec.retain(|o| {
				crate::world::order_pending(o,w, &mut next_state)
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

			let mut torem = Vec::new();
			for (i, p) in (&mut w.projectiles).iter_mut().enumerate() {
				p.update(delta);
				if p.reached {
					torem.push(i)
				}
			}
			for e in torem {w.projectiles.remove(e);};

			if self.ovec.len() == 0 && w.projectiles.len() == 0 {
				self.state = 0;
			}
			if let Some(i) = next_state {
				self.state = i;
			}
			return;
		}
		if rl.is_mouse_button_pressed(MouseButton::MOUSE_LEFT_BUTTON) {
			eprintln!("Click at {:?}", rl.get_mouse_position());
			if self.state == 0 {
				self.select_unit(w, rl.get_mouse_position());
				if self.state == 1{
					rlau.play_sound(rs.get_sound(0xff));
				}
 			} else if self.state == 1 {
 				self.select_move_tile(w, rl.get_mouse_position());
 			} else if self.state == 4 {
 				self.select_attack_tile(w, rl.get_mouse_position());
 			} else if self.state == 7 {
 				self.state = 0;
 				return;
 			}
		}
		if rl.is_mouse_button_pressed(MouseButton::MOUSE_RIGHT_BUTTON) {
			self.show_info = !self.show_info;
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
			// To check.
			let m = w.bgm_id;
			sl.notify_turn(w, &mut self.ovec);
			// Switch music.
			if w.bgm_id != m {
				if let Some(a) = rs.get_music(m) {
					rlau.stop_music_stream(a);
				}
				if let Some(a) = rs.get_music(w.bgm_id){
					rlau.play_music_stream(a);
				}
			}
			for (i, u) in &w.units {
				if let crate::world::TileType::Heal = crate::world::tile_type_at(&w, u.wpos.x as i32, u.wpos.y as i32)  {
					self.ovec.push(Order::MutHealthR(*i, 0.25));
				}
				if let crate::world::TileType::Damage = crate::world::tile_type_at(&w, u.wpos.x as i32, u.wpos.y as i32)  {
					self.ovec.push(Order::MutHealthR(*i, -0.35));
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
				eprintln!("Selected unit id={}, at {:?}", self.cur_id, self.last_tile);
				let t = crate::world::_guinfo(w, u);
				self.movn = t.0;
				self.movn_i = t.0;
				self.range = t.1;
				self.uname = t.2;
				self.isplrsel = t.3;
				self.show = true;
				self.cur_upos = (tx, ty);
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
			if !crate::world::tile_type_at(w, tx, ty).allowed() {
				return;
			}
			for (i, u) in &w.units {
				if (u.wpos.x as i32) == tx && (u.wpos.y as i32) == ty {
					if *i == self.cur_id {
						self.state = 4;
					} /*else if crate::world::is_tile_atrange((tx,ty),self.last_tile,self.range){
						self.ovec.push(Order::ATTACK(self.cur_id, *i, self.last_tile.0, self.last_tile.1));
						self.confirm_move();
					}*/ // Changed controls. 
					return;
				}
			}
			if self.movn >= 1 && crate::world::is_tile_atrange((tx,ty), self.last_tile, 1){
				self.ovec.push(Order::MOVE(self.cur_id, tx, ty));
				self.last_tile = (tx,ty);
				self.movn -= 1;
			}
			if self.movn == 0 {
				self.state = 4;
			}
		} else {
			self.state = 0;
		}
	}

	fn select_attack_tile(&mut self, w: &World, mpos: Vector2) {
		let (tx, ty) = crate::world::tile_at(w, mpos.x, mpos.y);
		if !crate::world::is_tile_atrange((tx, ty), self.last_tile, self.range) {
			self.confirm_move();
			return;
		}
		for (i, u) in &w.units {
			if (u.wpos.x as i32) == tx && (u.wpos.y as i32) == ty {
				if *i != self.cur_id {
					self.ovec.push(Order::ATTACK(self.cur_id, *i, self.last_tile.0, self.last_tile.1));
				}
				break;
			}
		}
		self.confirm_move();
	}

	pub fn tile_shade(&self, tx: i32, ty: i32) -> u8 {
		let dst = i32::abs(tx - self.cur_upos.0) + i32::abs(ty - self.cur_upos.1);
		if dst == 0 {
			0
		} else if dst <= self.movn_i.into() {
			1
		} else if dst <= (self.movn_i + self.range).into() {
			2
		} else {
			0
		}
	}

	pub fn get_state(&self) -> u8 {
		self.state
	}

	pub fn _boxrange(&self) -> (i32, i32, i32, i32) {
		let s = (self.movn_i + self.range) as i32;
		(self.cur_upos.0 - s, self.cur_upos.0 + s, self.cur_upos.1 - s, self.cur_upos.1 + s)
	}

	pub fn _atkrange(&self) -> (i32, i32, i32, i32) {
		let r = self.range as i32;
		(self.last_tile.0 - r, self.last_tile.0 + r, self.last_tile.1 - r, self.last_tile.1 + r)
	}

	pub fn _inrange(&self, x: i32, y: i32) -> i32 {
		let dst = i32::abs(x - self.last_tile.0) + i32::abs(y - self.last_tile.1);
		if dst == self.range.into() {
			1
		} else if dst == 0 {
			-1
		} else {
			0
		}
	}

	pub(crate) fn is_frozen(&self, u: &u8) -> bool {
		return self.frozen.contains(u);
	}
}

impl fmt::Display for InputHandler {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{}[id:{}]\n\nMov:{}\nRng:{}", self.uname, self.cur_id, self.movn_i, self.range)
	}
}
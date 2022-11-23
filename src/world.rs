//! Manages Renderables, Game Objects, Isometric Tilemaps, and Co-ordinate conversions.

use std::fmt::Error;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::io::Read;
use std::fs::File;
use std::collections::HashMap;
use raylib::math::Vector2;
use raylib::math::Rectangle;
use raylib::prelude::Color;
use crate::input::Order;

const EPS: f32 = 0.1;
const CONTROL_PT: Vector2 = Vector2 {
	x: -1.0,
	y: -1.0
};

#[derive(Debug, Clone)]
pub enum TileType {
	/// Tile type for tiles which units cannot move onto.
	Prohibited,
	/// Tile type for tiles which heal units.
	Heal,
	/// Tile type for tiles which damage units.
	Damage,
	/// Tile type for tiles which units can move onto.
	Allowed,
}

impl TileType {
	pub fn allowed(&self) -> bool {
		match self {
			TileType::Prohibited => false,
			_ => true
		}
	}
}

/// Pointer to damage calculating function;
/// *`atk` - the unit type id of the attacking unit.
/// *`def` - the unit type id of the defending unit.
pub enum DamageFunc {
	Handle(fn (atk: u8, def: u8) -> f32),
	CHandle(extern "C" fn (atk: u8, def: u8) -> f32)
}

impl DamageFunc {
	fn invoke(&self, atk: u8, def: u8) -> f32 {
		match self {
			DamageFunc::Handle(r) => r(atk, def),
			DamageFunc::CHandle(r) => r(atk, def)
		}
	}
}

pub(crate) struct Projectile {
	target: Vector2,
	ctrlpt: Vector2,
	velocity: Vector2,
	position: Vector2,
	source: Vector2,
	length: f32,
	lifetime: f32,
	expected: f32,
	pub(crate) reached: bool
}

impl Projectile {
	/// Create a projectile from *source* to *target* with given speed and length.
	fn new(target: Vector2, source: Vector2, speed: f32, len: f32) -> Projectile {
		let diff = target - source;
		let velocity = diff.normalized().scale_by(speed);
		Projectile {
			target: target,
			velocity: velocity,
			position: source,
			source: source,
			ctrlpt: source + CONTROL_PT,
			length: len,
			lifetime: 0.0,
			expected: diff.length() / speed,
			reached: false,
		}
	}

	pub(crate) fn update(&mut self, delta: f32) {
		let diff = self.position - self.target;
		if f32::abs(diff.x) < EPS && f32::abs(diff.y) < EPS {
			self.reached = true;
			return;
		}
		self.lifetime += delta;
		let t = self.lifetime / self.expected;
		self.velocity = self.bezier_vel(t);
		self.position = self.bezier_pos(t);
	}

	pub fn _prep_draw(&self, w: &World) -> (Vector2, Vector2) {
		let vp = Vector2::new(w.tile_size.0 as f32 /2.0, w.tile_size.1 as f32 / 2.0);
		let off = self.velocity.normalized().scale_by(self.length);
		(wots_v(w, self.position) + vp , wots_v(w, self.position + off) + vp)
	}

	fn bezier_vel(&self, t: f32) -> Vector2 {
		let p_10 = self.ctrlpt - self.source;
		let p_21 = self.target - self.ctrlpt;
		p_10.scale_by(2.0*(1.0-t)) + p_21.scale_by(2.0*t)
	}

	fn bezier_pos(&self, t: f32) -> Vector2 {
		let p_10 = self.ctrlpt - self.source;
		let p_21 = self.target - self.ctrlpt;
		let p1 = self.source + p_10.scale_by(t);
		let p2 = self.ctrlpt + p_21.scale_by(t);
		let p = p1 + (p2-p1).scale_by(t);
		p
	}
}

#[derive(Debug)]
/// Plain struct to store map data
struct TileMap {
	// Tile data
	/// The width of the map
	map_width: usize,
	/// THe height of the map
	map_height: usize,
	/// Flattened array consisting of the tiles of the map
 	map_tiles: Vec<u8>,
 	/// HashMap to map tile id to corresponding movement permissions.
 	tile_perm: HashMap<u8, TileType>,

 	// Text data
 	/// Map title
 	title: String,
 	/// Text shown before map begins.
 	intro_text: String,
 	/// Text shown when player wins.
 	victory_text: String,
 	/// Text shown when player loses.
 	defeat_text: String,

	/// Flag to show or hide map.	
	show: bool
}

impl TileMap {
	fn empty() -> TileMap {
		TileMap {
			map_width: 0,
			map_height: 0,
			map_tiles: vec![],
			tile_perm: HashMap::new(),
			title: String::new(),
			intro_text: String::new(),
			victory_text: String::new(),
			defeat_text: String::new(),
			show: false
		}
	}
}

fn no_dmg(_: u8, _: u8) -> f32 {
	0.0
}

/// Plain struct to contain sprites, tilemap, gameobjects etc.
pub struct World {
	/// Vector containing all StaticTex structs to be rendered.
	pub(crate) statics: Vec<StaticTex>,
	/// Map containing all registered unit types.
	pub(crate) unit_types: HashMap<u8,UnitType>,
	/// Map containing all alive units.
	pub units: HashMap<u8, Unit>,
	/// Vector containing all projectiles yet to reach their target.
	pub(crate) projectiles: Vec<Projectile>,
	/// The origin of the world
	origin: (i32, i32),
	/// Tile Map of the world.
	tilemap: TileMap,
	/// Size of isometric tiles in pixels
	tile_size: (i32, i32),
	/// Camera abscissa in world-co-ordinates.
	pub cam_wx: f32,
	/// Camera ordinate in world-co-ordinates.
	pub cam_wy: f32,
	/// Camera offset
	pub coff: (f32, f32),
	/// The internal identifier of the music currently playing in the background.
	pub bgm_id: u8,
	/// The function pointer for damage function
	pub(crate) dmg_func: DamageFunc,
}

///#TODO: Remove in Release
impl Drop for World {
	fn drop(&mut self) {
		println!("INFO: Dropping World!")
	}
}

impl Debug for World {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
		write!(f,"World {{\n\tstatics: {:?}\n\tunits: {:?}\n\ttilemap: {:?}\n}}", self.statics, self.units, self.tilemap)
	}
}

impl World {
	/// Create an empty world struct.
	/// Tile size is 96x48.
	pub fn blank() -> World {
		World {
			statics: vec![],
			projectiles: vec![],
			unit_types: HashMap::new(),
			units: HashMap::new(),
			origin: (0,0),
			tile_size: (96, 48),
			tilemap: TileMap::empty(),
			cam_wx: 0.0,
			cam_wy: 0.0,
			coff: (0.0, 0.0),
			bgm_id: 0,
			dmg_func: DamageFunc::Handle(no_dmg),
		}
	}

	/// Create an empty world with specified origin and tile size
	/// *`ox` - abscissa of origin.
	/// *'oy' - ordinate of origin.
	/// *'tx' - width of tile.
	/// *'ty' - height of tile.
	pub fn blank_o(ox: i32, oy: i32, tx: i32, ty: i32) -> World{
		World {
			statics: vec![],
			projectiles: vec![],
			unit_types: HashMap::new(),
			units: HashMap::new(),
			origin: (ox,oy),
			tile_size: (tx,ty),
			tilemap: TileMap::empty(),
			cam_wx: 0.0,
			cam_wy: 0.0,
			coff: (0.0, 0.0),
			bgm_id: 0,
			dmg_func: DamageFunc::Handle(no_dmg),
		}	
	}

	/// Get camera position in screen co-ordinates
	pub fn get_cpos(&self) -> (f32, f32) {
		return wots_f(self, self.cam_wx, self.cam_wy);
	}

	/// Get size of tiles in tileset as a tuple of (width, height)
	pub fn get_tile_size(&self) -> (i32, i32) {
		return self.tile_size;
	}

	/// Set camera position in world co-ordinates
	pub fn set_cpos(&mut self, x: f32, y: f32) {
		self.cam_wy = x;
		self.cam_wy = y;
	}

	/// Returns the size of the tilemap as a tuple (width, height)
	pub fn map_size(&self) -> (usize, usize) {
		return (self.tilemap.map_width, self.tilemap.map_height)
	}

	/// Returns true if tilemap has been loaded with tile data.
	pub fn show_map(&self) -> bool {
		return self.tilemap.show;
	}

	/// Set the music id for the background music.
	pub fn set_bgm(&mut self, id: u8) {
		self.bgm_id = id;
	}

	/// Set the damage calculation function.
	pub fn bind_damage_func(&mut self, f: fn(u8, u8) -> f32) {
		self.dmg_func = DamageFunc::Handle(f);
	}

	/// Get the text to be displayed before starting gameplay.
	pub fn intro_text(&self) -> &str {
		&self.tilemap.intro_text
	}

	/// Get the title of the map loaded.
	pub fn map_title(&self) -> &str {
		&self.tilemap.title
	}

	/// Get the text to be displayed when player wins.
	pub fn victory_text(&self) -> &str {
		&self.tilemap.victory_text
	}

	/// Get the text to be displayed when player loses.
	pub fn defeat_text(&self) -> &str {
		&self.tilemap.defeat_text
	}
}

/// Plain struct to specify the texture, world co-ordinates, and size of a static image
#[derive(Debug)]
pub struct StaticTex {
	/// The id of texture from resource set.
	tex_id: u8,
	/// The world x-co-ordinate of the image.
	wx: i32,
	/// The world y-co-ordinate of the image.
	wy: i32,
}

impl StaticTex {
	/// Return the texture id, and on-screen position of the static.
	pub fn prep_draw(&self, w: &World) -> (u8, i32, i32) {
		let (x,y) = wots(w, self.wx, self.wy);
		(self.tex_id, x, y)
	}
}

struct AnimInfo {
	frame_width: u32,
	frame_height: u32,
	frame_rate: f32,
	sfr_x: u32,
	sfr_y: u32,
	nframes: u8,
	snd_info: Option<(u8, bool)>,
	flip: bool
}

/// A struct to specify a Unit-Type. Contains details common to all units of a given type.
pub struct UnitType {
	tex_id: u8,
	max_health: f32,
	attack_dur: f32,
	/// The rate at which the sprite moves on-screen in tiles per second.
	mov_rate: f32,
	/// The display name of units belonging to this type.
	pub name: String,
	/// Info-string to be displayed. for this unit type.
	pub info: Option<String>,
	/// The maximum number of tiles units of this type can move in a turn.
	movement: u8,
	/// The range of the unit.
	range: u8,
	/// Animation related info
	anim: Vec<AnimInfo>,
}

///#TODO: Remove in release
impl Drop for UnitType {
	fn drop(&mut self) {
		eprintln!("Dropping struct UnitType:{} !", self.name);
	}
}

impl UnitType {
	/// Constructor method.
	/// Create a new unit-type with the specified name, stats, and texture.
	/// * `tex_id` - The internal identifier of the unit's texture.
	/// * `name` - The display name of the unit.
	/// * `max_health` - The max health of the unit.
	/// * `mov_rate` - The rate at which the unit moves in tiles per second.
	/// * `movement` - The number of tiles a unit of this type can move.
	/// * `range` - The range of the unit's attack.
	/// * `attack_dur` - The duration of attack state in seconds
	pub fn new(tex_id: u8, name: String, max_health: f32, mov_rate: f32, movement: u8, range: u8, attack_dur: f32) -> UnitType {
		UnitType {
			tex_id: tex_id,
			name: name,
			info: None,
			anim: vec![],
			max_health: max_health,
			mov_rate: mov_rate,
			movement: movement,
			range: range,
			attack_dur: attack_dur
		}
	}

	/// Define an animation for this unit without an accompanying sound.
	/// All unit animations must be defined in the exact order as enum variants of [`Unit State`]
	///
	/// The corresponding texture must also have the animations in the same order.
	/// All frames of a given animation must be laid out horizontally, with the same ordinate.
	/// * `fs` - size of animation frame.
	/// * `fn` - number of frames in the animation.
	/// * `cf` - the starting frame location, in pixels.
	/// * `r` - frame rate for the animation.
	/// * `flip` - Flag to render mirror image of frame.
	/// The duration of Standing Animation is also the duration of the attack animation.
	pub fn def_anim_muted(&mut self, fs: (u32, u32), frn: u8, cf: (u32, u32), fr: f32, flip: bool) {
		self.anim.push(AnimInfo {
			frame_width: fs.0,
			frame_height: fs.1,
			frame_rate: fr,
			sfr_x: cf.0,
			sfr_y: cf.1,
			nframes: frn,
			flip: flip,
			snd_info: None
		});
	}

	/// Define an animation for this unit with an accompanying sound.
	/// All unit animations must be defined in the exact order as enum variants of [`Unit State`]
	///
	/// The corresponding texture must also have the animations in the same order.
	/// All frames of a given animation must be laid out horizontally, with the same ordinate.
	/// * `fs` - size of animation frame.
	/// * `fn` - number of frames in the animation.
	/// * `cf` - the starting frame location, in pixels.
	/// * `r` - frame rate for the animation.
	/// * `flip` - Flag to render mirror image of frame.
	///	* `snd` - The internal identifier of the animation sound.
	///	* `lp` - Flag to loop sound until animation is complete.
	/// The duration of Standing Animation is also the duration of the attack animation.
	pub fn def_anim(&mut self, fs: (u32, u32), frn: u8, cf: (u32, u32), fr: f32, flip: bool, snd: u8, lp: bool) {
		self.anim.push(AnimInfo {
			frame_width: fs.0,
			frame_height: fs.1,
			frame_rate: fr,
			sfr_x: cf.0,
			sfr_y: cf.1,
			nframes: frn,
			flip: flip,
			snd_info: Some((snd, lp))
		});
	}

	/// Set the info string of this unit type.
	pub fn set_info(&mut self, text: String) {
		self.info = Some(text);
	}
}

/// An enum of all possible states of a unit. Every UnitState has a corresponding animation.
#[derive(Debug)]
pub enum UnitState {
	/// Unit increments y-co-ordinate steadily. 
	WalkDown,
	/// Unit increments x-co-ordinate steadily.
	WalkLeft,
	/// Unit decrements y-co-ordinate steadily.
	WalkUp,
	/// Unit decrements x-co-ordinate steadily.
	WalkRight,
	/// Unit attacks tile below it
	AttackDown,
	/// Unit attacks tile west of it
	AttackLeft,
	/// Unit attacks tile above it.
	AttackUp,
	/// Unit attacks tile east of it.
	AttackRight,
	/// No change in unit position.
	Stand
}

impl UnitState {
	fn is_idle(&self) -> bool {
		match self {
			UnitState::Stand => true,
			_ => false
		}
	}
}

fn state_as_usize(u: &UnitState) -> usize {
	match u {
		UnitState::WalkDown => 0,
		UnitState::WalkLeft => 1,
		UnitState::WalkUp => 2,
		UnitState::WalkRight => 3,
		UnitState::AttackDown => 4,
		UnitState::AttackLeft => 5,
		UnitState::AttackUp => 6,
		UnitState::AttackRight => 7,
		UnitState::Stand => 8
	}
}

/// Plain struct to represent a unit in the world.
#[derive(Debug)]
pub struct Unit {
	type_id: u8,
	/// The health (HP) of the unit.
	pub health: f32,
	/// Field to store a user-defined 'state' value for sprite.
	state: UnitState,
	/// The tint to be applied to the sprite (hex colour).
	pub tint: i32,
	/// The position of the sprite in the world.
	pub wpos: Vector2,
	/// Counter for animation time. 
	ftime: f32,
	/// Counter for time elapsed in a non-idle state.
	stime: f32,
	frame: u8,
	busy: bool,
	/// Flag to mark whether the unit belongs to player or enemy.
	pub player: bool
}

impl Unit {
	fn new(tid: u8, tint: i32, wpos: Vector2, plr: bool, health: f32) -> Unit {
		Unit {
			type_id: tid,
			tint: tint,
			wpos: wpos,
			player: plr,
			health: health,
			state: UnitState::Stand,
			frame: 0,
			ftime: 0.0,
			stime: 0.0,
			busy: false,
		}
	}

	#[inline]
	fn get_anim_info<'a>(&self, w: &'a World) -> (&'a UnitType, &'a AnimInfo) {
		let ut = w.unit_types.get(&self.type_id).expect(&format!("fatal [draw]: Unit type id {} does not exist", self.type_id));
		let idx = state_as_usize(&self.state);
		let aif = &ut.anim[idx];
		return (ut, aif)
	}

	/// Prepare the unit for rendering.
	pub fn prep_draw(&self, w: &World) -> (u8, Rectangle, Vector2, Option<(u8, bool)>){
		let (ut, aif) = self.get_anim_info(w);
		let rec = Rectangle {
			height: (aif.frame_height as f32),
			width: if aif.flip {-1.0} else {1.0} * (aif.frame_width) as f32,
			x: (aif.sfr_x + (self.frame as u32)*aif.frame_width) as f32,
			y: (aif.sfr_y as f32)
		};
		let v2 = Vector2::new(0.5*(w.tile_size.0 - aif.frame_width as i32) as f32, 0.5*(w.tile_size.1 - aif.frame_height as i32) as f32);
		(ut.tex_id, rec, wots_v(w,self.wpos) + v2, aif.snd_info)
	}

	pub fn _stand_frame(&self, w: &World, tx: i32, ty: i32) -> (u8, Rectangle, Vector2) {
		let ut = w.unit_types.get(&self.type_id).unwrap();
		let aif = &ut.anim[8];
		let rec = Rectangle {
			height: (aif.frame_height as f32),
			width: if aif.flip {-1.0} else {1.0} * (aif.frame_width) as f32,
			x: (aif.sfr_x as f32),
			y: (aif.sfr_y as f32)
		};
		let v2 = Vector2::new(0.5*(w.tile_size.0 - aif.frame_width as i32) as f32, 0.5*(w.tile_size.1 - aif.frame_height as i32) as f32);
		let (sx, sy) = wots(w, tx, ty);
		let pos = Vector2::new(sx as f32, sy as f32) + v2;
		(ut.tex_id, rec, pos)
	}

	/// Get the tint colour for the unit.
	pub fn get_tint(&self) -> Color {
		Color::get_color(self.tint)
	}

	/// Update function of the Unit.
	pub fn update(&mut self, uh: &HashMap<u8,UnitType>, delta: f32) {
		self.ftime += delta;
		let ut = uh.get(&self.type_id).expect(&format!("fatal [draw]: Unit type id {} does not exist", self.type_id));
		if !self.state.is_idle() {
			self.stime += delta;
		}
		let aif = &ut.anim[state_as_usize(&self.state)];
		self.frame = f32::floor(self.ftime * aif.frame_rate) as u8;
		if self.frame >= aif.nframes.into() {
			self.frame = 0;
			self.ftime = 0.0;
		}
		let ds = delta * ut.mov_rate;
		match self.state {
			UnitState::WalkDown => {self.wpos.y += ds},
			UnitState::WalkLeft => {self.wpos.x -= ds},
			UnitState::WalkUp => {self.wpos.y -= ds},
			UnitState::WalkRight => {self.wpos.x += ds},
			_ => ()
		};
	}

	/// Returns true if the unit has changed state within the current frame.
	pub fn nascent_state(&self) -> bool {
		return self.stime == 0.0;
	}
}

/// Convert world co-ordinates into screen co-ordinates for rendering purposes.
pub fn wots(w: &World,xw: i32, yw: i32) -> (i32, i32) {
	return (w.origin.0 + (xw-yw)*w.tile_size.0/2, w.origin.1 + (xw+yw)*w.tile_size.1/2)
}

fn wots_v(w: &World, v: Vector2) -> Vector2 {
	let v2 = wots_f(w, v.x, v.y);
	return Vector2::new(v2.0, v2.1);
}

/// Convert world co-ordinates to screen co-ordinates.
fn wots_f(w: &World, xw: f32, yw: f32) -> (f32, f32) {
	return ((w.origin.0 as f32) + (xw-yw)*(0.5*w.tile_size.0 as f32), (w.origin.1 as f32) + (xw+yw)*(0.5*w.tile_size.1 as f32))
}

/// Get the world position of the virtual tile at given screen position.
pub fn tile_at(w: &World, x: f32, y: f32) -> (i32, i32) {
	let cpos = w.get_cpos();
	let x = x + cpos.0 - w.origin.0 as f32;
	let y = y + cpos.1 - w.origin.1 as f32;
	let tx = f32::floor(x/w.tile_size.0 as f32);
	let ty = f32::floor(y/w.tile_size.1 as f32);
	let tix = x - (tx + 0.5)*w.tile_size.0 as f32;
	let tiy = y - (ty + 0.5)*w.tile_size.1 as f32;
	let v = (2.0*f32::abs(tix) / w.tile_size.0 as f32) + (2.0*f32::abs(tiy) / w.tile_size.1 as f32);
	let rx = (tx + ty) as i32;
	let ry = (ty - tx) as i32;
	//println!("Tile at: v: {}, rx: {}, ry: {}, tix: {}, tiy: {}, tx: {}, ty: {}", v, rx, ry, tix, tiy, tx, ty); // Debug
	if v <= 1.0 {
		return (rx, ry);
	} else if tix > 0.0 && tiy > 0.0 {
		return (rx + 1, ry);
	} else if tix < 0.0 && tiy > 0.0 {
		return (rx, ry + 1);
	} else if tix > 0.0 && tiy < 0.0 {
		return (rx, ry - 1);
	} else {
		return (rx-1, ry);
	}
}

/// Add a static image/texture of given size to the world at the specified location.
/// Statics are rendered in insertion/creation order.
pub fn create_static(w: &mut World, tex_id: u8, co_ords: (i32,i32)) {
	let s = StaticTex {
		tex_id: tex_id,
		wx: co_ords.0,
		wy: co_ords.1
	};
	w.statics.push(s);
}

/// Register a unit type into the world.
/// Only registered types can have units.
pub fn register_unit_type(w: &mut World, u: UnitType, id: u8) {
	w.unit_types.insert(id, u);
}

/// Spawn a unit of the given type with specified tint, and position.
/// * `plr` - Flag to mark this unit as player-controllable.
pub fn spawn_unit(w: &mut World, type_id: u8, co_ords: (i32, i32), tint: i32, plr: bool) -> u8 {
	let ut = w.unit_types.get(&type_id).expect("Invalid unit type!");
	let u = Unit::new(type_id, tint,Vector2::new(co_ords.0 as f32, co_ords.1 as f32) ,plr, ut.max_health);
	
	// Generate id.
	let mut id = w.units.len(); let mut f: u8 = 0;
	while w.units.contains_key(&f) {
		id ^= id << 13;
		id ^= id >> 17;
		id ^= id << 5;
		f = (id & 0xff) as u8
	}
	w.units.insert(f, u);
	return f;
}

/// Returns true if given order has not yet been completed, else false.
pub fn order_pending(o: &Order, w: &mut World, next_state: &mut Option<u8>) -> bool {
	match o {
		Order::MOVE(id, tx, ty) => crate::world::has_unit_moved(w, *id, (*tx, *ty)),
		Order::ATTACK(id, target, tx, ty) => crate::world::has_unit_attacked(w, *id, *target, (*tx, *ty)),
		Order::VICTORY => {
			*next_state = Some(5);
			false
		},
		Order::DEFEAT => {
			*next_state = Some(6);
			false
		},
		Order::MutHealthA(id, delta) => {
			let u = w.units.get_mut(id).unwrap();
			u.health += delta;
			false
		}
		Order::MutHealthR(id, delta) => {
			let u = w.units.get_mut(id).unwrap();
			let absdel = delta * w.unit_types.get(&u.type_id).unwrap().max_health * delta;
			u.health += absdel;
			false
		}
	}
}

fn has_unit_moved(w: &mut World, uid: u8, co_ords: (i32, i32)) -> bool {
	let u: &mut Unit = w.units.get_mut(&uid).expect("Invalid unit ID");
	if u.busy {
		let ux = f32::abs(u.wpos.x - co_ords.0 as f32);
		let uy = f32::abs(u.wpos.y - co_ords.1 as f32);
		if ux <= 0.05 && uy <= 0.05{
			_chust(u, UnitState::Stand);
			u.wpos.x = co_ords.0 as f32;
			u.wpos.y = co_ords.1 as f32;
			u.busy = false;
			return false;
		} else {
			return true;
		}
	} else {
		_chust(u,_gdir(u, co_ords.0, co_ords.1, uid));
		u.busy = true;
		return true;
	}
}

fn has_unit_attacked(w: &mut World, uid: u8, trg: u8, co_ords: (i32,i32)) -> bool {
	let tp = w.units.get(&trg).expect("Invalid unit ID").wpos;
	let u: &mut Unit = w.units.get_mut(&uid).expect("Invalid unit ID");
	let ut = w.unit_types.get(&u.type_id).expect("Invalid unit type ID");
	if u.busy {
		let ux = f32::abs(u.wpos.x - co_ords.0 as f32);
		let uy = f32::abs(u.wpos.y - co_ords.1 as f32);
		if ux == 0.0 && uy == 0.0 && u.stime >= ut.attack_dur{
			_chust(u,UnitState::Stand);
			u.busy = false;
			let atk_id = u.type_id;
			let t = w.units.get_mut(&trg).expect("Invalid unit ID");
			let dmg = w.dmg_func.invoke(atk_id, t.type_id);
			t.health -= dmg; //ut.max_health*ut.base_attack;
			return false;
		} else {
			return true;
		}
	} else {
		_chust(u,_gadir(u, tp, uid));
		let dst = i32::abs(tp.x as i32 - co_ords.0) + i32::abs(tp.y as i32 - co_ords.1);
		if dst > 1 {
			let vec = Vector2::new(co_ords.0 as f32, co_ords.1 as f32);
			w.projectiles.push(Projectile::new(tp, vec, 4.0, 0.5));
		}
		u.busy = true;
		return true;
	}
}

/// Returns true if unit of specified id can be controlled by player.
/// Returns false if the unit does not exist.
pub fn is_unit_player_controlled(w: &World, uid: u8) -> bool {
	match w.units.get(&uid) {
		Some(u) => u.player,
		_ => false
	}
}

/// Mutator for unit state.
/// Does nothing if unit id does not exist.  
pub fn set_unit_state(w: &mut World, uid: u8, us: UnitState) {
	println!("Changed unit {} state to {:?}", uid, us);
	let u = w.units.get_mut(&uid);
	if let Some(u) = u {
		_chust(u, us);
	}
}

fn _chust(u: &mut Unit, us: UnitState) {
	u.state = us;
	u.stime = 0.0;
	u.ftime = 0.0;
	u.frame = 0;
}

#[allow(missing_docs)]
pub fn _guinfo(w: &World, u: &Unit) -> (u8, u8, String, bool) {
	let mut s = String::new();
	let ut = w.unit_types.get(&u.type_id).expect("Invalid unit type ID");
	s.push_str(&ut.name);
	(ut.movement, ut.range, s, u.player)
}

fn _gdir(v: &Unit, tx: i32, ty: i32, uid: u8) -> UnitState{
	let v = v.wpos;
	let (wx, wy) = (v.x as i32, v.y as i32);
	let tx = tx - wx;
	let ty = ty - wy;
	println!("uid:{}, wx: {}, wy: {}, tx: {}, ty: {}", uid, wx, wy, tx, ty);
	if tx == 0 && ty == 1 {
		return UnitState::WalkDown;
	} else if tx == 1 && ty == 0 {
		return UnitState::WalkRight;
	} else if tx == 0 && ty == -1 {
		return UnitState::WalkUp;
	} else if tx == -1 && ty == 0 {
		return UnitState::WalkLeft;
	} else {
		UnitState::Stand
	}
}

fn _gadir(v: &Unit, w: Vector2, uid: u8) -> UnitState{
	let v = v.wpos - w;
	let (tx, ty) = (v.x as i32, v.y as i32);
	println!("uid:{}, tx: {}, ty: {}", uid, tx, ty);
	if i32::abs(ty) > i32::abs(tx) {
		if ty <= 0 {UnitState::AttackDown} else {UnitState::AttackUp}
	} else {
		if tx <= 0 {UnitState::AttackRight} else {UnitState::AttackLeft}
	}
}

/// Returns true if two tile positions are within a given range of each other.
pub fn is_tile_atrange(t1: (i32, i32), t2: (i32, i32), r: u8) -> bool{
	let x = i32::abs(t1.0 - t2.0);
	let y = i32::abs(t1.1 - t2.1);
	println!("t1: {:?}, t2: {:?}, diff: {:?}, r: {}", t1, t2, (x,y), r);
	return x + y == (r as i32); 
}

const MAGIC: [u8; 4] = [0xfa, 0xde, 0x00, 0xff];
const CONT_READ: [u8; 2] = [0xfe,0xed];
const MPSIG: [u8; 2] = [0xda, 0xd7];
macro_rules! bferr {
	($f:ident, $emsg:literal) => {
		{
			eprintln!("fatal [load_world]: Malformed world file {}, cause: {}", $f, $emsg);
			return false;
		}
	};
}
fn read_tilelist(f: &mut File, tperm: &mut HashMap<u8, TileType>, fpath: &str, perm: TileType) -> bool {
	let mut buf1 = [0];
	let n = f.read(&mut buf1).expect("Failed to read tile list.");
	if n < 1 {
		bferr!(fpath, "Failed to read tile list.");
	}
	let mut b = vec![0; buf1[0] as usize];
	let n = f.read(&mut b).expect("Failed to read tile list.");
	if n < b.len() {
		bferr!(fpath, "Failed to read tile list.");
	}
	for v in b {
		tperm.insert(v, perm.clone());
	}
	true
}

fn read_string(f: &mut File, fpath: &str, st: &mut String, mut buf2: [u8; 2]) -> bool {
	let n = f.read(&mut buf2).expect("Failed to read ascii string");
	if n < 2 {
		bferr!(fpath, "Failed to read ascii string");
	}
	let s = (buf2[0] as u16) << 8| buf2[1] as u16;
	let mut b = vec![0; s.into()];
	let n = f.read(&mut b).expect("IOError while reading string");
	if n < s.into() {
		bferr!(fpath, "Failed to read ascii string");
	}
	st.push_str(std::str::from_utf8(&b).expect("Failed to decode utf-8 string."));
	true
}

/// Load tile map data from the specified file into the world
/// * `_w` - The world to load [TileMap] into
/// * `fpath` - The path to the file containing map data.  
/// Returns `true` if map data could successfully be loaded, otherwise false.  
/// ## Binary Format
/// The file specified by `fpath` must conform to the following binary format:
///
/// > First four bytes of the file are exactly `[250, 222, 0, 255]`  
/// > The next byte specifies the width of the map.  
/// > The following byte specified the height of the map.  
/// > The next `w*h` bytes, where `w` and `h` are map width and height repsectively, comprise map data for each tile.  
/// > The next 6 bytes form a mandatory padding (thus must be identically zero).  
/// > The remaining section of the file defines game objects, and their position in the world.  
/// > Game Objects are encoded as 6 byte sequences that begin with `[254, 237]`. The third byte defines the game object type.  
/// > The fourth byte is the `type-parameter` for a given game object. The fifth and sixth bytes define the x and y co-ordinates of the game object.  
///
/// If any of the mandatory components of the format are missing in the file specified, the file is termed as *a malformed world file*.
/// The term Game Object is *merely an abstract construct* (with no direct counterpart in the engine) used to allow for a common format of specification for statics, and units.
/// The following table summarizes `type` and `type-parameter` relations:
///
/// | Type (`u8`) |      Parameter        |
/// | ----------- | --------------------- |
/// | Static (0) | The texture id of the static |
/// | Player Unit (1) | The type id of the unit |
/// | Enemy Unit (2) | The type id of the unit |
/// ## Panics
/// The function panics with appropriate error messages if:  
/// 1. The file could not be found or opened (does not block until file is available)
/// 2. An I/O Error occurs, and read fails.
/// 3. Memory allocation of map data fails.
pub fn load_world(_w: &mut World, fpath: &str) -> bool {
	let mut f = match File::open(fpath) {
		Err(e) => panic!("Failed to load world file: {}, due to an error. Cause: {}", fpath, e),
		Ok(a) => a
	};

	// Data Buffers.
	let mut buf4:[u8; 4] = [0,0,0,0];
	let mut buf2:[u8; 2] = [0,0];
	
	// Read MAGIC
	let _n = f.read(&mut buf4).expect("Failed to read MAGIC bytes from world file.");
	if buf4 != MAGIC {
		bferr!(fpath, "World file does not begin with MAGIC.")
	}

	// Read file size
	let n = f.read(&mut buf2).expect("Failed to read world size from world file.");
	if n < 2 {
		bferr!(fpath, "Could not infer world size; not specified.");
	}
	let (w, h) = (buf2[0] as usize, buf2[1] as usize);

	// Check if movement permission data is present
	let n = f.read(&mut buf2).expect("Failed to read movement permissions notifier (u16).");
	if n < 2 {
		bferr!(fpath, "Failed to read tile movement permissions notifier.");
	}
	let mut tperm = HashMap::new();
	if buf2 != [0,0] {
		if buf2 != MPSIG {
			bferr!(fpath, "Invalid data at end of section. Allowed: 0xDAD7 or 0x0000");
		}
		if !read_tilelist(&mut f, &mut tperm, fpath, TileType::Prohibited) {return false;}
		if !read_tilelist(&mut f, &mut tperm, fpath, TileType::Heal) {return false;}
		if !read_tilelist(&mut f, &mut tperm, fpath, TileType::Damage) {return false;}
	}
	// Read tile data. TODO: Switch to a more efficient version using mid-size buffers.
	let mut tdata = Vec::new();
	{
		let s = w * h;
		tdata.try_reserve(s).expect(&format!("Failed to allocate {} bytes of memory for map data", s));
		tdata.resize(s, 0);
	}
	let n = f.read(&mut tdata).expect("Failed to read tile data from world file.");
	if n < tdata.len() {
		bferr!(fpath, "World file does not specify all tiles (premature termination of file).");
	}

	let mut title = String::from("");
	if !read_string(&mut f, fpath, &mut title, buf2) {return false;}
	let mut intro_text = String::from("");
	if !read_string(&mut f, fpath, &mut intro_text, buf2) {return false;}
	let mut victory_text = String::from("");
	if !read_string(&mut f, fpath, &mut victory_text, buf2) {return false;}
	let mut defeat_text = String::from("");
	if !read_string(&mut f, fpath, &mut defeat_text, buf2) {return false;}

	_w.tilemap = TileMap {
		map_width: w,
		map_height: h,
		map_tiles: tdata,
		tile_perm: tperm,
		title: title,
		intro_text: intro_text,
		defeat_text: defeat_text,
		victory_text: victory_text,
		show: true
	};

	/*match f.seek(SeekFrom::Current(6)) {
		Err(_e) => {
			eprintln!("error: {}", _e);
			bferr!(fpath, "6 byte padding after tile data absent.");
		},
		Ok(_) => ()
	};*/ // Skip 6 bytes for padding.

	let mut n = f.read(&mut buf2).expect("Failed to read continue notifier.");
	if n < 2{
		eprintln!("debug [load_world]: Reached EOF");
		return true;
	}
	while n == 2 && buf2 == CONT_READ {
		n = f.read(&mut buf4).expect("Failed to read game object data.");
		if n < 4 {
			bferr!(fpath, "Game Object data must be specified as a raw 4-byte sequence comprising type, id, x, and y.");
		}
		eprintln!("Game Object Data: {:?}", buf4);
		match buf4[0] {
			0 => create_static(_w, buf4[1], (buf4[2] as i32, buf4[3] as i32)),
			1 => {spawn_unit(_w, buf4[1], (buf4[2] as i32, buf4[3] as i32), -1, true);},
			2 => {spawn_unit(_w, buf4[1], (buf4[2] as i32, buf4[3] as i32), -0x38ffc328, false);},
			a => {eprintln!("warning: Unrecognized game object TYPE={}", a);}
		};
		n = f.read(&mut buf2).expect("Failed to read continue notifier.")
	}
	return true;
}

/// Return position of tile texture in tileset and tile position on-screen.
pub fn prep_tiledraw(w: &World, x: i32, y: i32, n: i32) -> (Vector2, Vector2) {
	let idx = ((y as usize)*w.tilemap.map_width+(x as usize)) % w.tilemap.map_tiles.len();
	let t = w.tilemap.map_tiles[idx];
	let ty = t as i32 / n;
	let tx = t as i32 % n;
	let u = crate::world::wots(w, x, y);
	return (Vector2::new((tx*w.tile_size.0) as f32, (ty*w.tile_size.1) as f32), Vector2::new(u.0 as f32, u.1 as f32))
}

/// Check if the there exists a unit with the specified id.
pub fn is_uid_valid(w: &World, uid: u8) -> bool {
	return w.units.contains_key(&uid);
}

/// Returns a vector containing units of all alive units.
pub fn id_list(w: &World) -> Vec<u8> {
	w.units.keys().cloned().collect()
}

/// Returns true if the tile specified allows movement. 
pub fn tile_type_at(w: &World, x: i32, y: i32) -> TileType {
	if x < 0 || y < 0 {
		return TileType::Prohibited;
	}
	let idx = ((y as usize)*w.tilemap.map_width+(x as usize)) % w.tilemap.map_tiles.len();
	let t = w.tilemap.map_tiles[idx];
	if w.tilemap.tile_perm.contains_key(&t) {
		return w.tilemap.tile_perm.get(&t).unwrap().clone()
	} else {
		TileType::Allowed
	}
}

pub(crate) fn _unit_health(w: &World, uid: u8) -> (f32, f32) {
	let u = w.units.get(&uid).unwrap();
	let h = u.health;
	let mh = w.unit_types.get(&u.type_id).unwrap().max_health;
	(h, mh)
}

pub(crate) fn  _unit_info(w: &World, uid: u8) -> Option<&String> {
	let u = w.units.get(&uid).unwrap();
	let ut = w.unit_types.get(&u.type_id).unwrap();
	ut.info.as_ref()
}

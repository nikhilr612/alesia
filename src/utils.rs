//! Provides basic utilities like resource management

use raylib::audio::Music;
use crate::input::Order;
use raylib::text::Font;
use raylib::RaylibThread;
use std::collections::HashMap;
use raylib::RaylibHandle;
use raylib::prelude::Texture2D;
use raylib::prelude::Sound;

enum ResType {
	Tex,
	Fnt,
	Snd,
	Mus
}

/// Struct for storing, and managing resources such as textures and cues.
/// # Example
/// ```
/// use alesia::utils::ResourceSet;
/// let mut rs = ResourceSet::new();
///	rs.map_texture(0, "mypic.png");
/// ```
pub struct ResourceSet {
	to_load: Vec<(u8, ResType, String)>,
	texs: HashMap<u8, Texture2D>,
	fonts: HashMap<u8, Font>,
	sounds: HashMap<u8, Sound>, 
	tracks: HashMap<u8, Music>,
	deftex: u8,
	deffont: u8
}

///#TODO: Remove in Release
impl Drop for ResourceSet {
	fn drop(&mut self){
		println!("INFO: Dropping ResourceSet");
	}
}

impl ResourceSet {
	/// Default constructor.
	pub fn new() -> ResourceSet {
		ResourceSet {
			to_load: vec![],
			texs: HashMap::new(),
			fonts: HashMap::new(),
			sounds: HashMap::new(),
			tracks: HashMap::new(),
			deftex: 0,
			deffont: 0
		}
	}

	/// Map a texture to an internal unsigned byte identifier.
	/// Certain byte identifiers enumerated below are reserved:
	/// * 240 (`0xf0`)- the tileset. 
	/// * 241(`0xf1`) - the 'select tile' image that follows the cursor when a unit is selected to be moved.
	/// * 242(`0xf2`) - the background image for the unit information text.
	/// The method does not load textures, but stores id-path mappings so that they may later be loaded once an OpenGL context is available.
	pub fn map_texture(&mut self, id: u8, path: &str) {
		self.deftex = id;
		self.to_load.push((id, ResType::Tex, path.to_string()));
	}

	/// Map a font to an internal unsigned byte identifier.
	/// The method does not load fonts, but stores id-path mappings so that they may later be loaded once an OpenGL context is available.
	pub fn map_font(&mut self, id: u8, path: &str) {
		self.deffont = id;
		self.to_load.push((id, ResType::Fnt, path.to_string()));
	}

	/// Return the texture (if it exists) with the specified id.
	pub fn get_texture(&self, id: u8) -> &Texture2D {
		match self.texs.get(&id) {
			Some(tex) => tex,
			_ => self.get_default_texture()
		}
	}

	/// Return the font (if it exists) with the specified id.
	pub fn get_font(&self, id: u8) -> &Font {
		match self.fonts.get(&id) {
			Some(f) => f,
			_ => self.get_default_font()
		}
	}

	/// Set the default texture of the resource set.
	/// The default texture is returned whenever a texture id is not registered or loaded.
	/// If not set, the last texture mapped is considered as the default texture.
	/// If no textures have been mapped, then default texture id is 0.
	pub fn set_default_texture(&mut self, id: u8) {
		self.deftex = id;
	}

	/// Get the default texture.
	/// # Panics
	/// If the texture has failed to load, or not been loaded.
	pub fn get_default_texture(&self) -> &Texture2D {
		match self.texs.get(&self.deftex) {
			Some(tex) => tex,
			_ => panic!("Default texture [ID={}] has not been loaded", self.deftex),
		}
	}

	/// Get the default font. Used for UI.
	/// # Panics
	/// If the font has failed to laod, or not been loaded.
	pub fn get_default_font(&self) -> &Font {
		match self.fonts.get(&self.deffont) {
			Some(f) => f,
			_ => panic!("Default font [ID={}] has not been loaded", self.deffont)
		}
	}

	/// Map a sound to an internal unsigned byte identifier
	/// Certain byte identifiers enumerated below are reserved:
	/// * 255 ('0xff') - The sound played when any unit is selected by the player.
	/// The method does not load sounds, but stores id-path mappings so that requisite files may be loaded once initialization is complete.
	pub fn map_sound(&mut self, id: u8, path: &str) {
		self.to_load.push((id, ResType::Snd, path.to_string()));
	}

	/// Return the sound (if it exists) with the specified id.
	pub fn get_sound(&self, id: u8) -> &Sound {
		match self.sounds.get(&id) {
			Some(snd) => snd,
			_ => panic!("Sound [ID={}] has not been loaded", id)
		}
	}

	/// Map a music track to an internal unsigned byte identifier
	/// The method does not load music, but stores id-path mappings so that requisite files may be loaded once initialization is complete.
	pub fn map_music(&mut self, id: u8, path: &str) {
		self.to_load.push((id, ResType::Mus, path.to_string()));
	}

	/// Return the sound (if it exists) with the specified id.
	pub fn get_music(&mut self, id: u8) -> Option<&mut Music> {
		self.tracks.get_mut(&id)
	}
}

/// Load all resources from the set.
/// ## Panics
/// If any mapped resource fails to load, then this function panics.
pub fn load_all(rs: &mut ResourceSet, rl: &mut RaylibHandle, rthread: &RaylibThread) {
	for (id, rtyp, path) in rs.to_load.iter() {
		match rtyp{
			ResType::Tex => {
				let ermsg = format!("warning [resources]: failed to load texture id={}, from {}", *id, path);
				let tex = rl.load_texture(rthread, path).expect(&ermsg);
				rs.texs.insert(*id, tex);
			},
			ResType::Fnt => {
				let ermsg = format!("warning [resources]: failed to load font id={}, from {}", *id, path);
				let f = rl.load_font(rthread, path).expect(&ermsg);
				rs.fonts.insert(*id, f);
			},
			ResType::Snd => {
				let ermsg = format!("warning [resources]: failed to load sound id={}, from {}", *id, path);
				let snd = Sound::load_sound(path).expect(&ermsg);
				rs.sounds.insert(*id, snd);
			},
			ResType::Mus => {
				let ermsg = format!("warning [resources]: failed to load music track id={}, from {}", *id, path);
				let snd = Music::load_music_stream(rthread, path).expect(&ermsg);
				rs.tracks.insert(*id, snd);
			}
		}
	}
}

type InitHandle = fn();
/// Type alias for nullable C ABI function pointer for `on_init` [callback](StateListener). 
pub type CInitHandle = Option<extern "C" fn()>; 
type TurnHandle = fn(&mut crate::world::World, &mut Vec<Order>);
/// Type alias for nullable C ABI function pointer for `on_turn` [callback](StateListener).
/// # Safety
/// The call site retains ownership of all non-primitive parameters. 
/// **Under no circumstances must the references be released within this callback**
pub type CTurnHandle = Option<extern "C" fn(*mut crate::world::World, *mut Vec<Order>)>;


/// Plain struct to store callbacks for the following events:
/// 1. Display initialization.
/// 2. Player turn end.
pub struct StateListener {
	raw: bool,
	on_init: Option<InitHandle>,
	on_init_raw: CInitHandle,
	on_turn: Option<TurnHandle>,
	on_turn_raw: CTurnHandle
}

impl StateListener {
	/// Constructor method.
	pub fn new() -> StateListener {
		StateListener {
			raw: false,
			on_init: None,
			on_init_raw: None,
			on_turn: None,
			on_turn_raw: None
		}
	}

	/// Constructor method for 'raw' listeners (i.e, C ABI function pointers are bound)
	/// Used by [the native API](crate::napi)
	pub fn _new_raw() -> StateListener {
		StateListener {
			raw: true,
			on_init: None,
			on_init_raw: None,
			on_turn: None,
			on_turn_raw: None
		}
	}

	/// Bind a function for callback when display is initialized.
	pub fn bind_init(&mut self, f: InitHandle) {
		if self.raw {
			eprintln!("warning [state_listener]: rust fp bound to raw listener!");
		}
		self.on_init = Some(f);
	}

	/// Bind a function for callback when player turn ends.
	pub fn bind_turn(&mut self, f: TurnHandle) {
		if self.raw {
			eprintln!("warning [state_listener]: rust fp bound to raw listener!");
		}
		self.on_turn = Some(f);
	}

	/// FFI Internal
	pub fn _bind_rawinit(&mut self, f: CInitHandle) {
		if !self.raw {
			eprintln!("warning [state_listener]: C fp bound to state listener!");
		}
		self.on_init_raw = f;
	}

	/// FFI Internal
	pub fn _bind_rawturn(&mut self, f: CTurnHandle) {
		if !self.raw {
			eprintln!("warning [state_listener]: C fp bound to state_listener!");
		}
		self.on_turn_raw = f;
	}

	/// Notify this listener that display initialization has been completed.
	pub fn notify_init(&self) {
		if self.raw {
			if let Some(f) = self.on_init_raw {
				f();
			}
		} else {
			if let Some(f) = self.on_init {
				f();
			}
		}
	}

	/// Notify this listener that turn has ended, i.e, transition from WAITING -> ALIVE 
	pub fn notify_turn(&self, w: &mut crate::world::World, ih: &mut Vec<Order>) {
		if self.raw {
			if let Some(f) = self.on_turn_raw {
				f(w,ih);
			}
		} else {
			if let Some(f) = self.on_turn {
				f(w,ih);
			}
		}
	}
}


/*struct EventInfo {
	etext: String,
	optlabels: Vec<String>,
	fontsize: f32,
	text_rec_size: (i32, i32),
	win_size: (i32, i32),
	colour: i32,
}

impl EventInfo {
	fn new(t: String, w_width: i32, w_height: i32, tb_width: i32, tb_height: i32, fsize: f32, col: i32) -> EventInfo {
		EventInfo {
			etext: t,
			optlabels: vec![],
			fontsize: fsize,
			text_rec_size: (tb_width, tb_height),
			win_size: (w_width, w_height),
			colour: col
		}
	}
}*/
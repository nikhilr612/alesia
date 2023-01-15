//! Exposes FFI-functions that are exported into dynamic libraries or shared objects.
//!
//! The functions follow the following general name scheme:
//! 1. If the function name `name_of_the_function` is unique across the crate, then `alsNameOfTheFunction` is the chosen name
//! 	1.1 For Clarity's sake, the function name may be followed by `_Struct` if function is a struct method.
//! 2. If the function is a constructor for a struct, i.e, `Struct::new` then `alsNewStruct` is the chosen name
//! 3. Destructors have names of the form `alsFreeStruct` where `Struct::drop` is invoked.
//! 4. Functions exclusive to the Native API begin with `alsn` and are named in `CamelCase`.
//! 
//! Wrapper functions will not panic unless underlying engine functions panic, i.e, in case of errors most wrapper functions will simply return (in case of void return type) or return a default value after printing appropriate error messages.
//! *Exception*. Wrapper functions that require a const c_char* pointer panic if the CStr is not a valid Utf8 String.
//! If the function returns a mutable pointer, then in case of an error a [NULL Pointer](std::ptr::null_mut) is returned.

use crate::input::Order;
use raylib::prelude::Color;
use crate::utils::CTurnHandle;
use crate::utils::CInitHandle;
use crate::utils::StateListener;
use crate::display::Display;
use crate::world::UnitType;
use crate::world;
use crate::world::World;
use std::ffi::CStr;
use std::os::raw::c_char;
use crate::utils::ResourceSet;
use std::ptr;

macro_rules! create_release {
	($fname:ident, $stype:ident) => {
		#[no_mangle]
		/// Free a leaked reference
		pub extern "C" fn $fname(a: *mut $stype) {
			if a.is_null() {
				return;
			}
			unsafe {
				Box::from_raw(a);
			}
		}
	};
}

macro_rules! check_nonnull {
	($pname:ident, $msg: expr) => {
		if $pname.is_null() {
			eprintln!($msg);
			return;
		}
	};
	($pname:ident, $msg: expr, $ret: expr) => {
		if $pname.is_null() {
			eprintln!($msg);
			return $ret;
		}
	};
}

#[allow(missing_docs)]
#[no_mangle]
pub extern "C" fn alsNewResourceSet() -> *mut ResourceSet {
	let rset = ResourceSet::new();
	Box::into_raw(Box::new(rset))
}

#[allow(missing_docs)]
#[no_mangle]
pub extern "C" fn  alsBlank_World() -> *mut World {
	let w = World::blank();
	Box::into_raw(Box::new(w))
}

#[allow(missing_docs)]
#[no_mangle]
pub extern "C" fn alsNewUnitType(tid: u8, name: *const c_char, health: f32, mov_rate: f32, movt: u8, ran: u8, adur: f32) -> *mut world::UnitType {
	check_nonnull!(name, "fatal [napi]: Pointer to UnitType display name String is NULL", ptr::null_mut());
	//Convert String
	let p = unsafe { CStr::from_ptr(name) };
    let p = p.to_str().map(|s| s.to_owned()).expect("UnitType display name is not UtfString");
	
	let ut = world::UnitType::new(tid, p, health, mov_rate, movt, ran, adur);
	Box::into_raw(Box::new(ut))
}

#[allow(missing_docs)]
#[no_mangle]
pub extern "C" fn alsNewStateListener() -> *mut StateListener {
	let s = StateListener::_new_raw();
	Box::into_raw(Box::new(s))
}

pub type DfuncType = extern "C" fn(u8, u8) -> f32;

create_release!(alsFreeResourceSet, ResourceSet);
create_release!(alsFreeWorld, World);
create_release!(alsFreeUnitType, UnitType);
create_release!(alsFreeStateListener, StateListener);

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsnFreeVec(v: *mut Vec<u8>) {
	check_nonnull!(v, "warning [napi]: Vec_u8 pointer is NULL");
	unsafe {
		Box::from_raw(v);
	}
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsMapTexture(rs: *mut ResourceSet, id: u8, path: *const c_char) {
	check_nonnull!(rs, "fatal [napi]: Pointer to ResourceSet is NULL");
	check_nonnull!(path, "fatal [napi]: Pointer to ResourceSet Path String is NULL");
	//Copy String
	let p = unsafe { CStr::from_ptr(path) };
    let p = p.to_str().map(|s| s.to_owned()).expect("ResourceSet path is not UtfString");
    
    unsafe {
    	let r = &mut *rs;
    	r.map_texture(id, &p);
    }
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsMapTextureRegion(rs: *mut ResourceSet, tid: u8, parent_id: u8, x: f32, y: f32, w: f32, h: f32) {
	check_nonnull!(rs, "fatal [napi]: Pointer to ResourceSet is NULL");
	unsafe {
		let r = &mut *rs;
		r.map_texture_region(tid, parent_id, x, y, w, h);
	}	
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsCreateStatic(w: *mut World, tex_id: u8, cx: i32, cy: i32) {
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL");
	unsafe {
		let w = &mut *w;
		world::create_static(w, tex_id, (cx, cy));
	}
}

#[no_mangle]
/// FFI for `UnitType.def_anim_muted`. Tuples have been expanded into individual arguments.
pub extern "C" fn alsDefAnim(u: *mut UnitType, fw: u32, fh: u32, frn: u8, cfx: u32, cfy: u32, fr: f32, flip: bool) {
	check_nonnull!(u, "fatal [napi]: Pointer to UnitType is NULL");
	unsafe {
		(&mut *u).def_anim_muted((fw, fh), frn, (cfx,cfy), fr, flip);
	}
}

#[no_mangle]
/// FFI for `UnitType.def_anim`. Tuples have been expanded into indivifual arguments.
pub extern "C" fn alsDefAnimUnmuted(u: *mut UnitType, fw: u32, fh: u32, frn: u8, cfx: u32, cfy: u32, fr: f32, flip: bool, snd: u8, lp: bool) {
	check_nonnull!(u, "fatal [napi]: Pointer to UnitType is NULL");
	unsafe {
		(&mut *u).def_anim((fw, fh), frn, (cfx,cfy), fr, flip, snd, lp);
	}	
}

#[no_mangle]
/// Creates the display and begins the game.
/// ResourceSet and World are deallocated when this method returns.
pub extern "C" fn alsBeginS_Display(sw: i32, sh: i32, t: *const c_char, rs: *mut ResourceSet, w: *mut World) {
	check_nonnull!(rs, "fatal [napi]: Pointer to ResourceSet is NULL");
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL");
	check_nonnull!(t, "fatal [napi]: Pointer to Display Title String is NULL");

	// Allocate and Copy
	let p = unsafe { CStr::from_ptr(t) };
    let p = p.to_str().map(|s| s).expect("ResourceSet path is not UtfString");

	unsafe {
		let wb = Box::from_raw(w);
		let rsb = Box::from_raw(rs);
		let d = Display::new_s(sw, sh, p);
		d.begin_s(*rsb, *wb);
	}
}

#[no_mangle]
/// Creates the display and begins the game.
/// ResourceSet, World, and StateListener are deallocated when this method returns.
pub extern "C" fn alsBegin_Display(sw: i32, sh: i32, vsync: bool, fps: u32, t: *const c_char, rs: *mut ResourceSet, w: *mut World, sl: *mut StateListener, mvl: f32) {
	check_nonnull!(rs, "fatal [napi]: Pointer to ResourceSet is NULL");
	check_nonnull!(w, "fatal [napi]: Pointer to world is NULL");
	check_nonnull!(t, "fatal [napi]: Pointer to Display Title String is NULL");
	check_nonnull!(sl, "fatal [napi]: Pointer to StateListener is NULL");

	// Allocate and Copy
	let p = unsafe { CStr::from_ptr(t) };
    let p = p.to_str().map(|s| s).expect("ResourceSet path is not UtfString");

    unsafe {
		let wb = Box::from_raw(w);
		let rsb = Box::from_raw(rs);
		let d = Display::new(sw, sh, fps, vsync, p, Color::BLACK, mvl);
		let s = Box::from_raw(sl);
		d.begin(*rsb, *wb, *s);
	}   
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsRegisterUnitType(w: *mut World,u: *mut UnitType, id: u8) {
	check_nonnull!(u, "fatal [napi]: Pointer to UnitType is NULL");
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL");
	unsafe {
		let w = &mut *w;
		let ub = Box::from_raw(u);
		world::register_unit_type(w, *ub, id);
	}
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsSetUnitInfo(u: *mut UnitType, s: *const c_char) {
	check_nonnull!(u, "fatal [napi]: Pointer to UnitType is NULL");
	check_nonnull!(s, "fatal [napi]: Pointer to information string is NULL");
	//Copy String
	let p = unsafe { CStr::from_ptr(s) };
    let p = p.to_str().map(|s| s.to_owned()).expect("ResourceSet path is not UtfString");
    
    unsafe {
    	let r = &mut *u;
    	r.set_info(p);
    }
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsBindDamageFunc(w: *mut World, f: DfuncType) {
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL");
	unsafe {
		(*w).dmg_func = world::DamageFunc::CHandle(f);
	}
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsMapFont(rs: *mut ResourceSet, id: u8, path: *const c_char) {
	check_nonnull!(rs, "fatal [napi]: Pointer to ResourceSet is NULL");
	check_nonnull!(path, "fatal [napi]: Pointer to ResourceSet Path String is NULL");
	//Copy String
	let p = unsafe { CStr::from_ptr(path) };
    let p = p.to_str().map(|s| s.to_owned()).expect("ResourceSet path is not UtfString");
    
    unsafe {
    	let r = &mut *rs;
    	r.map_font(id, &p);
    }
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsBindInit(sl: *mut StateListener, f: CInitHandle) {
	check_nonnull!(sl, "fatal [napi]: Pointer to StateListener is NULL");
	unsafe {
		let sl = &mut *sl;
		sl._bind_rawinit(f)
	}
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsBindTurn(sl: *mut StateListener, f: CTurnHandle) {
	check_nonnull!(sl, "fatal [napi]: Pointer to StateListener is NULL");
	unsafe {
		let sl = &mut *sl;
		sl._bind_rawturn(f)
	}
}

#[no_mangle]
/// Getter for the health of the unit with specified ID.
/// Returns -1.0 on NULL pointer or invalid ID.
pub extern "C" fn alsnGetUnitHealth(w: *mut World, uid: u8) -> f32 {
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL", -1.0);
	unsafe {
		let w = &*w;
		if let Some(h) = w.units.get(&uid){
			h.health
		} else {
			-1.0
		}
	}
}


#[no_mangle]
/// Getter for the x position of the unit with specified ID.
pub extern "C" fn alsnGetUnitX(w: *const world::Unit) -> f32 {
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL", -1.0);
	unsafe {
		let w = &*w;
		w.wpos.x
	}
}

#[no_mangle]
/// Getter for the x position of the unit with specified ID.
pub extern "C" fn alsnGetUnitY(w: *const world::Unit) -> f32 {
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL", -1.0);
	unsafe {
		let w = &*w;
		w.wpos.y
	}
}

#[no_mangle]
/// Get an immutable (i.e, `readonly`) reference to the unit with the specified id.
/// # Paincs
/// If the specified unit ID is invalid.
/// # Safety
/// The function exposes an immutable reference to the [Unit](crate::world::Unit) instance corresponding to the id.
/// **Under no circumstances must the reference returned be released by the callsite**.
/// **Under no circumstances must the reference be cast to a mutable type and modified**.
pub extern "C" fn alsnUnitRef(w: *mut World, uid: u8) -> *const world::Unit {
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL", ptr::null());
	unsafe {
		let w = &*w;
		let u = w.units.get(&uid).expect("Invalid unit ID");
		u
	}
}

#[no_mangle]
/// Returns true if the unit with given id is an enemy unit.
pub extern "C" fn alsnIsUnitFoe(uref: *const world::Unit) -> bool {
	check_nonnull!(uref, "fatal [napi]: Pointer to UnitRef is NULL", false);
	unsafe {
		let u = &*uref;
		u.player
	}
}

#[no_mangle]
/// Push a move order. See enum [Order].
pub extern "C" fn alsnPushMoveOrder(i: *mut Vec<Order>, uid: u8, tx: i32, ty: i32) {
	check_nonnull!(i, "fatal [napi]: Pointer to Order Vector is NULL");
	unsafe {
		let i = &mut *i;
		i.push(Order::MOVE(uid, tx, ty));
	}
}

#[no_mangle]
/// Push an attack Order. See enum [Order]. 
pub extern "C" fn alsnPushAttackOrder(i: *mut Vec<Order>, uid: u8, target: u8, tx: i32, ty: i32) {
	check_nonnull!(i, "fatal [napi]: Pointer to Order Vector is NULL");
	unsafe {
		let i = &mut *i;
		i.push(Order::ATTACK(uid, target, tx, ty));
	}	
}

#[no_mangle]
/// Push a victory order
pub extern "C" fn alsnPushVictoryOrder(i: *mut Vec<Order>) {
	check_nonnull!(i, "fatal [napi]: Pointer to Order Vector is NULL");
	unsafe {
		let i = &mut *i;
		i.push(Order::VICTORY);
	}
}

#[no_mangle]
/// Push a victory order
pub extern "C" fn alsnPushDefeatOrder(i: *mut Vec<Order>) {
	check_nonnull!(i, "fatal [napi]: Pointer to Order Vector is NULL");
	unsafe {
		let i = &mut *i;
		i.push(Order::DEFEAT);
	}
}

#[no_mangle]
/// Push a victory order
pub extern "C" fn alsnPushMutHealthOrder(i: *mut Vec<Order>, uid: u8, val: f32, is_rel: bool) {
	check_nonnull!(i, "fatal [napi]: Pointer to Order Vector is NULL");
	unsafe {
		let i = &mut *i;
		if is_rel {
			i.push(Order::MutHealthR(uid, val))
		} else {
			i.push(Order::MutHealthA(uid, val))
		}
	}
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsSpawnUnit(w: *mut World, tid: u8, tx: i32, ty: i32, tint: i32, plr: bool) -> u8 {
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL", 0x00);
	unsafe {
		let w = &mut *w;
		crate::world::spawn_unit(w, tid, (tx, ty), tint, plr)
	}
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsIdList(w: *const World) -> *mut Vec<u8> {
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL", ptr::null_mut());
	unsafe{Box::into_raw(Box::new(world::id_list(&*w)))}
}

#[no_mangle]
/// Return length of byte vector.
pub extern "C" fn alsnVecLen(u: *const Vec<u8>) -> usize {
	unsafe{(&*u).len()}
}

#[no_mangle]
/// Return byte value at index `elm` in `Vec<u8>`
pub extern "C" fn alsnVecAt(u: *const Vec<u8>, elm: usize) -> u8 {
	unsafe{(&*u)[elm]}
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsMapSound(rs: *mut ResourceSet, id: u8, path: *const c_char) {
	check_nonnull!(rs, "fatal [napi]: Pointer to ResourceSet is NULL");
	check_nonnull!(path, "fatal [napi]: Pointer to ResourceSet Path String is NULL");
	//Copy String
	let p = unsafe { CStr::from_ptr(path) };
    let p = p.to_str().map(|s| s.to_owned()).expect("ResourceSet path is not UtfString");
    
    unsafe {
    	let r = &mut *rs;
    	r.map_sound(id, &p);
    }
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsLoadMap(w: *mut World, fpath: *const c_char) -> bool {
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL", false);
	check_nonnull!(fpath, "fatal [napi]: Level file path string is NULL", false);
	let p = unsafe { CStr::from_ptr(fpath) };
    let p = p.to_str().map(|s| s.to_owned()).expect("ResourceSet path is not UtfString");
    unsafe {
    	let w = &mut *w;
    	world::load_world(w, &p)
    }
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsMapMusic(rs: *mut ResourceSet, id: u8, path: *const c_char) {
	check_nonnull!(rs, "fatal [napi]: Pointer to ResourceSet is NULL");
	check_nonnull!(path, "fatal [napi]: Pointer to ResourceSet Path String is NULL");
	//Copy String
	let p = unsafe { CStr::from_ptr(path) };
    let p = p.to_str().map(|s| s.to_owned()).expect("ResourceSet path is not UtfString");
    
    unsafe {
    	let r = &mut *rs;
    	r.map_music(id, &p);
    }
}

#[no_mangle]
/// Checks if the specified unit ID is valid.
pub extern "C" fn alsVerifyUID(w: *const World, uid: u8) -> bool {
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL", false);
	unsafe {
		let w = &*w;
		world::is_uid_valid(w, uid)
	}
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsTilePermAt(w: *const World, x: i32, y: i32) -> bool {
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL", false);
	unsafe {
		let w = &*w;
		world::tile_type_at(w, x, y).allowed()
	}
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsGetTypeID(w: *const World, uid: u8) -> u8 {
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL", 0x00);
	unsafe {
		let w = &*w;
		world::get_type_id(w, uid)
	}
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsnGetWorldWidth(w: *const World) -> usize {
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL", 0x00);
	unsafe {
		let w = &*w;
		w.map_size().0
	}
}

#[no_mangle]
#[allow(missing_docs)]
pub extern "C" fn alsnGetWorldHeight(w: *const World) -> usize {
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL", 0x00);
	unsafe {
		let w = &*w;
		w.map_size().1
	}
}
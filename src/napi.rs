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
pub extern "C" fn alsNewUnitType(tid: u8, name: *const c_char, health: f32, mov_rate: f32, movt: u8, ran: u8, atk: f32, adur: f32) -> *mut world::UnitType {
	check_nonnull!(name, "fatal [napi]: Pointer to UnitType display name String is NULL", ptr::null_mut());
	//Convert String
	let p = unsafe { CStr::from_ptr(name) };
    let p = p.to_str().map(|s| s.to_owned()).expect("UnitType display name is not UtfString");
	
	let ut = world::UnitType::new(tid, p, health, mov_rate, movt, ran, atk, adur);
	Box::into_raw(Box::new(ut))
}

#[allow(missing_docs)]
#[no_mangle]
pub extern "C" fn alsNewStateListener() -> *mut StateListener {
	let s = StateListener::_new_raw();
	Box::into_raw(Box::new(s))
}

create_release!(alsFreeResourceSet, ResourceSet);
create_release!(alsFreeWorld, World);
create_release!(alsFreeUnitType, UnitType);
create_release!(alsFreeStateListener, StateListener);

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
pub extern "C" fn alsCreateStatic(w: *mut World, tex_id: u8, cx: i32, cy: i32) {
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL");
	unsafe {
		let w = &mut *w;
		world::create_static(w, tex_id, (cx, cy));
	}
}

#[no_mangle]
/// FFI for `UnitType.def_anim`. Tuples have been expanded into individual arguments.
pub extern "C" fn alsDefAnim(u: *mut UnitType, fw: u32, fh: u32, frn: u8, cfx: u32, cfy: u32, fr: f32, flip: bool) {
	check_nonnull!(u, "fatal [napi]: Pointer to UnitType is NULL");
	unsafe {
		(&mut *u).def_anim_muted((fw, fh), frn, (cfx,cfy), fr, flip);
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
pub extern "C" fn alsBegin_Display(sw: i32, sh: i32, t: *const c_char, rs: *mut ResourceSet, w: *mut World, sl: *mut StateListener) {
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
		let d = Display::new_s(sw, sh, p);
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
#[allow(missing_docs)]
pub extern "C" fn alsSpawnUnit(w: *mut World, tid: u8, tx: i32, ty: i32, tint: i32, plr: bool) -> u8 {
	check_nonnull!(w, "fatal [napi]: Pointer to World is NULL", 0x00);
	unsafe {
		let w = &mut *w;
		crate::world::spawn_unit(w, tid, (tx, ty), tint, plr)
	}
}
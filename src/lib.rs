//! Alesia, a straight-forward game engine in rust
#![warn(missing_docs)]

pub mod utils;
pub mod display;
pub mod world;
pub mod input;
pub mod napi;

#[test]
fn it_works() {
    let mut rs = utils::ResourceSet::new();
    let mut w = world::World::blank_o(0,0, 96, 48);
    rs.map_texture(0, "res/tile.png");
    rs.map_texture(1, "res/castle.png");
    rs.map_texture(2, "res/newswords.png");
    rs.map_texture(241, "res/select_tile.png");
    rs.map_texture(242, "res/uitex.png");
    rs.map_font(0, "res/def.ttf");
    rs.map_sound(1, "res/sword.wav");
    rs.map_sound(255, "res/select.wav");
    //rs.map_music(0, "res/bgm.mp3");
    for i in 0..5 {
        for j in 0..5 {
            world::create_static(&mut w, 0, (i, j));
        }
    }
    world::create_static(&mut w, 1, (2,2));
    let mut ut = world::UnitType::new(2, "Swordsperson".to_string(), 10.0, 0.2,  2, 1, 0.1, 3.0);
    ut.def_anim_muted((32,48), 10, (0,0), 5.0, false);
    ut.def_anim_muted((32,48), 10, (0,48), 5.0, false);
    ut.def_anim_muted((32,48), 10, (0,48), 5.0, true);
    ut.def_anim_muted((32,48), 10, (0,0), 5.0, true);
    ut.def_anim((48,64), 5, (0,96), 5.0, false, 1, true);
    ut.def_anim((48,64), 5, (0,160), 5.0, false, 1, true);
    ut.def_anim((48,64), 5, (0,160), 5.0, true, 1, true);
    ut.def_anim((48,64), 5, (0,96), 5.0, true, 1, true);
    ut.def_anim_muted((32,48), 1, (0,0), 2.0, false);
    world::register_unit_type(&mut w,ut, 0);
    world::spawn_unit(&mut w, 0, (3,1), -0x38ffc328, false);
    world::spawn_unit(&mut w, 0, (1,1), -1, true);
    world::spawn_unit(&mut w, 0, (0,2), -1, true);
    let d = display::Display::new_s(1296, 800, "Display test");
    let mut sl = utils::StateListener::new();
    sl.bind_init(|| {
        println!("Finished initialization of display!");
    });
    sl.bind_turn(|w, o| {
        println!("Test turn! {:?}, {:?}", w, o);
        w.set_bgm(0);
    });
    d.begin(rs, w, sl);
}

#[test]
fn load_map() {
    let mut rs = utils::ResourceSet::new();
    let mut w = world::World::blank();
    rs.map_texture(1, "res/castle.png");
    rs.map_texture(0, "res/house.png");
    rs.map_texture(3, "res/newswords.png");
    rs.map_texture(240, "res/tiles.png");
    rs.map_texture(241, "res/select_tile.png");
    rs.map_texture(242, "res/uitex.png");
    rs.map_font(0, "res/def.ttf");
    rs.map_sound(1, "res/sword.wav");
    rs.map_sound(255, "res/select.wav");
    let mut ut = world::UnitType::new(3, "Swordsperson".to_string(), 10.0, 0.2,  2, 1, 0.1, 7.0);
    ut.def_anim_muted((32,48), 10, (0,0), 5.0, false);
    ut.def_anim_muted((32,48), 10, (0,48), 5.0, false);
    ut.def_anim_muted((32,48), 10, (0,48), 5.0, true);
    ut.def_anim_muted((32,48), 10, (0,0), 5.0, true);
    ut.def_anim_muted((48,64), 5, (0,96), 5.0, false);
    ut.def_anim_muted((48,64), 5, (0,160), 5.0, false);
    ut.def_anim_muted((48,64), 5, (0,160), 5.0, true);
    ut.def_anim_muted((48,64), 5, (0,96), 5.0, true);
    ut.def_anim_muted((32,48), 1, (0,0), 5.0, false);
    world::register_unit_type(&mut w, ut, 0);
    world::load_world(&mut w, "res/testmap.alw");
    let d = display::Display::new_s(960, 816, "Display test");
    d.begin_s(rs, w);
}

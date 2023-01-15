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
    rs.map_texture(243, "res/move_tile.png");
    rs.map_texture(244, "res/attack_tile.png");
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
    let mut ut = world::UnitType::new(2, "Swordsperson".to_string(), 10.0, 0.2,  2, 1, 3.0);
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
    rs.map_texture(4, "res/walls.png");
    rs.map_texture(5, "res/archer.png");
    rs.map_texture(6, "res/Cavalry.png");
    rs.map_texture_region(5, 4, 0.0, 0.0, 111.0, 192.0);
    rs.map_texture_region(6, 4, 192.0, 48.0, 96.0, 116.0);
    rs.map_texture_region(7, 4, 384.0, 48.0, 96.0, 116.0);
    rs.map_texture(240, "res/isotileset.png");
    rs.map_texture(241, "res/select_tile.png");
    rs.map_texture(242, "res/uitex.png");
    rs.map_texture(243, "res/move_tile.png");
    rs.map_texture(244, "res/attack_tile.png");
    rs.map_texture(248, "res/uinfobox.png");
    rs.map_texture(245, "res/textbox.png");
    rs.map_font(0, "res/def.ttf");
    rs.map_sound(1, "res/sword.wav");
    rs.map_sound(255, "res/select.wav");

    let mut ut = world::UnitType::new(3, "Swordsman".to_string(), 10.0, 0.5,  2, 1, 1.5);
    ut.def_anim_muted((32,48), 10, (0,0), 8.0, false);
    ut.def_anim_muted((32,48), 10, (0,48), 8.0, false);
    ut.def_anim_muted((32,48), 10, (0,48), 8.0, true);
    ut.def_anim_muted((32,48), 10, (0,0), 8.0, true);
    ut.def_anim_muted((48,64), 5, (0,96), 8.0, false);
    ut.def_anim_muted((48,64), 5, (0,160), 8.0, false);
    ut.def_anim_muted((48,64), 5, (0,160), 8.0, true);
    ut.def_anim_muted((48,64), 5, (0,96), 8.0, true);
    ut.def_anim_muted((32,48), 1, (0,0), 8.0, false);
    ut.set_info("Wields long swords.\nAtk: 10\tDef:5".to_string());
    world::register_unit_type(&mut w, ut, 0);

    let mut ut = world::UnitType::new(5, "Archer".to_string(), 10.0, 0.5,  2, 2, 1.5);
    ut.def_anim_muted((32,48), 10, (0,0), 6.0, false);
    ut.def_anim_muted((32,48), 10, (0,48), 6.0, false);
    ut.def_anim_muted((32,48), 10, (0,48), 6.0, true);
    ut.def_anim_muted((32,48), 10, (0,0), 6.0, true);
    ut.def_anim_muted((32,64), 10, (0,96), 6.0, false);
    ut.def_anim_muted((32,64), 10, (0,160), 6.0, false);
    ut.def_anim_muted((32,64), 10, (0,160), 6.0, true);
    ut.def_anim_muted((32,64), 10, (0,96), 6.0, true);
    ut.def_anim_muted((32,48), 1, (0,0), 6.0, false);
    ut.set_info("Wields bows.\nAtk: 10\tDef:5".to_string());
    world::register_unit_type(&mut w, ut, 1);

    /*let mut ut = world::UnitType::new(6, "Cavalry".to_string(), 10.0, 0.5,  5, 1, 1.5);
    ut.def_anim_muted((80,64), 10, (0,0), 6.0, false);
    ut.def_anim_muted((80,64), 10, (0,64), 6.0, false);
    ut.def_anim_muted((80,64), 10, (0,64), 6.0, true);
    ut.def_anim_muted((80,64), 10, (0,0), 6.0, true);

    ut.def_anim_muted((80,112), 10, (0,128), 6.0, false);
    ut.def_anim_muted((80,112), 10, (0,240), 6.0, false);
    ut.def_anim_muted((80,112), 10, (0,240), 6.0, true);
    ut.def_anim_muted((80,112), 10, (0,128), 6.0, true);
    ut.def_anim_muted((80,68), 1, (0,0), 6.0, false);
    ut.set_info("Mounted unit, wields swords.\nAtk: 10\tDef:5".to_string());
    world::register_unit_type(&mut w, ut, 1);*/

    println!("World load success: {}", world::load_world(&mut w, "res/testmap2.alw"));
    let d = display::Display::new_s(1296, 816, "Display test");
    let mut sl = utils::StateListener::new();
    sl.bind_init(|| {
        println!("Finished initialization of display!");
    });
    sl.bind_turn(|_w, _o| {
        //o.push(input::Order::DEFEAT);
    });
    w.bind_damage_func(|_id1, _id2| {
        4.0
    });
    d.begin(rs, w, sl);
}

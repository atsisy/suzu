use torifune::device as tdev;
use tdev::VirtualKey;
use torifune::core::Clock;
use torifune::graphics as tgraphics;
use torifune::graphics::object::*;
use tgraphics::object as tobj;
use ggez::input as ginput;
use ginput::mouse::MouseButton;
use torifune::numeric;
use crate::core::{TextureID, GameData};
use torifune::core::Updatable;
use super::*;
use crate::object;
use crate::core::map_parser as mp;

pub struct DreamScene<'a> {
    player: object::Character<'a>,
    key_listener: tdev::KeyboardListener,
    clock: Clock,
    tile_map: mp::StageObjectMap,
    camera: numeric::Rect,
}

impl<'a> DreamScene<'a> {
    
    pub fn new(ctx: &mut ggez::Context, game_data: &'a GameData) -> DreamScene<'a>  {

        let key_listener = tdev::KeyboardListener::new_masked(vec![tdev::KeyInputDevice::GenericKeyboard],
                                                                  vec![]);

        let player = object::Character::new(tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                game_data.ref_texture(TextureID::Ghost1),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(0.1, 0.1),
                0.0, 0, object::move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                0), vec![]),
        object::TextureSpeedInfo::new(0.08, numeric::Vector2f::new(0.0, 0.0), numeric::Rect::new(0.0, 0.0, 1366.0, 600.0)));

        let camera = numeric::Rect::new(0.0, 0.0, 1366.0, 768.0);
        
        DreamScene {
            player: player,
            key_listener: key_listener,
            clock: 0,
            tile_map: mp::StageObjectMap::new(ctx, "./resources/sample.tmx", camera),
            camera: camera,
        }
    }

    fn check_key_event(&mut self, ctx: &ggez::Context) {
        if self.key_listener.current_key_status(ctx, &VirtualKey::Right) == tdev::KeyStatus::Pressed {
            self.right_key_handler(ctx);
        }

        if self.key_listener.current_key_status(ctx, &VirtualKey::Left) == tdev::KeyStatus::Pressed {
            self.left_key_handler(ctx);
        }

        if self.key_listener.current_key_status(ctx, &VirtualKey::Up) == tdev::KeyStatus::Pressed {
            self.up_key_handler(ctx);
        }
    }

    fn right_key_handler(&mut self, _ctx: &ggez::Context) {
        let offset = numeric::Vector2f::new(3.0, 0.0);
        //self.player.obj_mut().move_diff(offset);
        self.tile_map.move_camera(offset);
    }

    fn left_key_handler(&mut self, _ctx: &ggez::Context) {
        let offset = numeric::Vector2f::new(-3.0, 0.0);
        //self.player.obj_mut().move_diff(offset);
        self.tile_map.move_camera(offset);
    }

    fn up_key_handler(&mut self, _ctx: &ggez::Context) {
        let t = self.get_current_clock();

        self.player
            .speed_info_mut()
            .set_speed(numeric::Vector2f::new(0.0, -12.0));
        self.player
            .speed_info_mut()
            .fall_start(t);
        self.player
            .obj_mut()
            .override_move_func(object::move_fn::gravity_move(-5.0, 24.0, 600.0, 0.2), t)
    }

    fn check_collision(&mut self, ctx: &mut ggez::Context) {
        let collision_info = self.tile_map.check_collision(self.player.obj().get_drawing_area(ctx));
        if collision_info.collision  {
            //println!("{:?}", collision_info.boundly);
            self.player.fix_collision(ctx, &collision_info, self.get_current_clock());
            return ();
        }
    }
}

impl<'a> SceneManager for DreamScene<'a> {
    
    fn key_down_event(&mut self, _ctx: &mut ggez::Context, _vkey: tdev::VirtualKey) {
    }
    
    fn key_up_event(&mut self,
                    _ctx: &mut ggez::Context,
                    vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 up!"),
            _ => (),
        }
    }

    fn mouse_motion_event(&mut self,
                          _ctx: &mut ggez::Context,
                          _point: numeric::Point2f,
                          _offset: numeric::Vector2f) {

    }

    fn mouse_button_down_event(&mut self,
                               _ctx: &mut ggez::Context,
                               _button: MouseButton,
                               _point: numeric::Point2f) {
    }
    
    fn mouse_button_up_event(&mut self,
                             _ctx: &mut ggez::Context,
                             _button: MouseButton,
                             _point: numeric::Point2f) {
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context) {
        let t = self.get_current_clock();
        
        self.check_key_event(ctx);

        self.player.update(ctx, t);

        self.check_collision(ctx);
        
        self.tile_map.update(ctx, t);
        /*
        self.player
            .obj_mut()
            .move_with_func(t);
        */
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.tile_map.draw(ctx).unwrap();
        self.player
            .obj()
            .draw(ctx).unwrap();
    }
    
    fn post_process(&mut self, _ctx: &mut ggez::Context) -> SceneTransition {
        self.update_current_clock();
        SceneTransition::Keep
    }

    fn transition(&self) -> SceneID {
        SceneID::Dream
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }
    
    fn update_current_clock(&mut self) {
        self.clock += 1;
    }
    
}

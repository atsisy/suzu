pub mod task_scene;
pub mod dream_scene;
pub mod scenario_scene;

use torifune::device as tdev;
use torifune::core::Clock;
use ggez::input as ginput;
use torifune::numeric;
use torifune::graphics as tgraphics;
use tgraphics::object as tobj;
use tgraphics::{DrawableComponent, DrawableObject};

use crate::core::GameData;

#[derive(Debug, Eq, PartialEq)]
pub enum SceneTransition {
    Keep,
    Reset,
    Transition,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SceneID {
    Null,
    MainDesk,
    Scenario,
    Dream,
}

pub trait SceneManager {
    
    fn key_down_event(&mut self,
                      _ctx: &mut ggez::Context,
                      _game_data: &GameData,
                      _vkey: tdev::VirtualKey) {
    }
     
    fn key_up_event(&mut self,
                    _ctx: &mut ggez::Context,
                    _game_data: &GameData,
                    _vkey: tdev::VirtualKey){
    }

    fn mouse_motion_event(&mut self,
                          _ctx: &mut ggez::Context,
                          _game_data: &GameData,
                          _point: numeric::Point2f,
                          _offset: numeric::Vector2f){
    }

    fn mouse_button_down_event(&mut self,
                               _ctx: &mut ggez::Context,
                               _game_data: &GameData,
                               _button: ginput::mouse::MouseButton,
                               _point: numeric::Point2f){
    }
    
    fn mouse_button_up_event(&mut self,
                             _ctx: &mut ggez::Context,
                             _game_data: &GameData,
                             _button: ginput::mouse::MouseButton,
                             _point: numeric::Point2f){
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context);
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context);
    fn post_process(&mut self, ctx: &mut ggez::Context) -> SceneTransition;
    fn transition(&self) -> SceneID;
    

    fn get_current_clock(&self) -> Clock;
    
    fn update_current_clock(&mut self);
}

pub struct SimpleObjectContainer {
    container: Vec<tobj::SimpleObject>,
}

impl SimpleObjectContainer {
    fn new() -> SimpleObjectContainer {
        SimpleObjectContainer {
            container: Vec::new(),
        }
    }

    fn add(&mut self, obj: tobj::SimpleObject) {
        self.container.push(obj);
    }

    fn sort_with_depth(&mut self) {
        self.container.sort_by(tgraphics::drawable_object_sort_with_depth);
    }

    fn get_raw_container(&self) -> &Vec<tobj::SimpleObject> {
        &self.container
    }

    fn get_raw_container_mut(&mut self) -> &mut Vec<tobj::SimpleObject> {
        &mut self.container
    }

    fn get_minimum_depth(&mut self) -> i8 {
        self.sort_with_depth();
        if let Some(depth) = self.container.last() {
            depth.get_drawing_depth()
        } else {
            127
        }
    }

    fn len(&self) -> usize {
        self.container.len()
    }

    fn change_depth_equally(&mut self, offset: i8)  {
        for obj in &mut self.container {
            let current_depth = obj.get_drawing_depth();
            let next_depth: i16 = (current_depth as i16) + (offset as i16);

            if next_depth <= 127 && next_depth >= -128 {
                // 範囲内
                obj.set_drawing_depth(next_depth as i8);
            } else if next_depth > 0 {
                // 範囲外（上限）
                obj.set_drawing_depth(127);
            } else {
                // 範囲外（下限）
                obj.set_drawing_depth(-128);
            }
        }
        
    }
}

pub struct NullScene {
}

impl NullScene {

    pub fn new() -> Self {
        NullScene {}
    }
}

impl SceneManager for NullScene {

    fn pre_process(&mut self, _ctx: &mut ggez::Context) {
        
    }
    
    fn drawing_process(&mut self, _ctx: &mut ggez::Context) {
        
    }
    fn post_process(&mut self, _ctx: &mut ggez::Context) -> SceneTransition {
        SceneTransition::Keep
    }

    fn transition(&self) -> SceneID {
        SceneID::Null
    }

    fn get_current_clock(&self) -> Clock {
        0
    }
    
    fn update_current_clock(&mut self) {
        
    }
}

use std::collections::HashMap;
use torifune::device as tdev;
use torifune::core::Clock;
use torifune::graphics as tgraphics;
use torifune::graphics::object::*;
use tgraphics::object as tobj;
use ggez::input as ginput;
use ginput::mouse::MouseButton;
use ggez::graphics as ggraphics;
use torifune::numeric;
use torifune::hash;
use crate::core::GameData;
use super::*;

struct MouseInformation {
    last_clicked: HashMap<MouseButton, numeric::Point2f>,
    last_dragged: HashMap<MouseButton, numeric::Point2f>,
    dragging: HashMap<MouseButton, bool>,
}

impl MouseInformation {

    fn new() -> MouseInformation {
        MouseInformation {
            last_clicked: hash![(MouseButton::Left, numeric::Point2f { x: 0.0, y: 0.0 }),
                                (MouseButton::Right, numeric::Point2f { x: 0.0, y: 0.0 }),
                                (MouseButton::Middle, numeric::Point2f { x: 0.0, y: 0.0 })],
            last_dragged: hash![(MouseButton::Left, numeric::Point2f { x: 0.0, y: 0.0 }),
                                (MouseButton::Right, numeric::Point2f { x: 0.0, y: 0.0 }),
                                (MouseButton::Middle, numeric::Point2f { x: 0.0, y: 0.0 })],
            dragging: hash![(MouseButton::Left, false),
                            (MouseButton::Right, false),
                            (MouseButton::Middle, false)]
        }
    }

    fn get_last_clicked(&self, button: MouseButton) -> numeric::Point2f {
        match self.last_clicked.get(&button) {
            Some(x) => *x,
            None => panic!("No such a mouse button"),
        }
    }

    fn set_last_clicked(&mut self, button: MouseButton, point: numeric::Point2f) {
        if self.last_clicked.insert(button, point) == None {
            panic!("No such a mouse button")
        }
    }

    fn get_last_dragged(&self, button: MouseButton) -> numeric::Point2f {
        match self.last_dragged.get(&button) {
            Some(x) => *x,
            None => panic!("No such a mouse button"),
        }
    }

    fn set_last_dragged(&mut self, button: MouseButton, point: numeric::Point2f) {
        if self.last_dragged.insert(button, point) == None {
            panic!("No such a mouse button")
        }
    }

    fn is_dragging(&self, button: ginput::mouse::MouseButton) -> bool {
        match self.dragging.get(&button) {
            Some(x) => *x,
            None => panic!("No such a mouse button"),
        }
    }

    fn update_dragging(&mut self, button: MouseButton, drag: bool) {
        if self.dragging.insert(button, drag) == None {
            panic!("No such a mouse button")
        }
    }
    
}

pub struct TaskScene<'a> {
    desk_objects: Vec<tobj::SimpleObject<'a>>,
    clock: Clock,
    mouse_info: MouseInformation,
}

impl<'a> TaskScene<'a> {
    pub fn new(_ctx: &mut ggez::Context, game_data: &'a GameData) -> TaskScene<'a>  {
        
        let obj = tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                game_data.ref_texture(0),
                numeric::Point2f { x: 0.0, y: 0.0 },
                numeric::Vector2f { x: 0.5, y: 0.5 },
                0.0, 0,  Box::new(move |p: & dyn tobj::MovableObject, t: Clock| {
                    torifune::numeric::Point2f{x: p.get_position().x, y: p.get_position().y}
                }),
                0), vec![]);
        
        TaskScene {
            desk_objects: vec![obj],
            clock: 0,
            mouse_info: MouseInformation::new()
        }
    }

    fn dragging_handler(&mut self,
                        ctx: &mut ggez::Context,
                        point: numeric::Point2f,
                        offset: numeric::Vector2f) {
        for p in &mut self.desk_objects {
            if p.get_drawing_area(ctx).contains(point) {
                let last = self.mouse_info.get_last_dragged(MouseButton::Left);
                p.move_diff(numeric::Vector2f {x: point.x - last.x, y: point.y - last.y});
            }
        }
    }
}

impl<'a> SceneManager for TaskScene<'a> {
    
    fn key_down_event(&mut self, ctx: &mut ggez::Context, vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => println!("Action1 down!"),
            _ => (),
        }
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
                          ctx: &mut ggez::Context,
                          point: numeric::Point2f,
                          offset: numeric::Vector2f) {
        if self.mouse_info.is_dragging(MouseButton::Left) {
            println!("x: {}, y: {} ::: offset_x: {}, offset_y: {}", point.x, point.y, offset.x, offset.y);
            let d = numeric::Vector2f { x: offset.x / 2.0,  y: offset.y / 2.0 };
            self.dragging_handler(ctx, point, d);
            self.mouse_info.set_last_dragged(MouseButton::Left, point);
        }

    }

    fn mouse_button_down_event(&mut self,
                               ctx: &mut ggez::Context,
                               button: MouseButton,
                               point: numeric::Point2f) {
        self.mouse_info.set_last_clicked(button, point);
        self.mouse_info.set_last_dragged(button, point);
        self.mouse_info.update_dragging(button, true);
    }
    
    fn mouse_button_up_event(&mut self,
                             ctx: &mut ggez::Context,
                             button: MouseButton,
                             point: numeric::Point2f) {
        self.mouse_info.update_dragging(button, false);
        
    }

    fn pre_process(&mut self, ctx: &mut ggez::Context) {
        for p in &mut self.desk_objects {
            p.move_with_func(self.clock);
        }
    }
    
    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        for p in &self.desk_objects {
            p.draw(ctx).unwrap();
        }
    }
    
    fn post_process(&mut self, ctx: &mut ggez::Context) -> SceneTransition {
        SceneTransition::Keep
    }

    fn transition(&self) -> SceneID {
        SceneID::MainDesk
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }
    
    fn update_current_clock(&mut self) {
        self.clock += 1;
    }
    
}

use torifune::device as tdev;
use torifune::core::Clock;
use torifune::numeric;
use torifune::graphics as tg;

pub fn halt(pos: numeric::Point2f)
            -> Box<dyn Fn(& dyn tg::object::MovableObject, Clock) -> numeric::Point2f> {
    Box::new(move |p: & dyn tg::object::MovableObject, _t: Clock| {
        pos
    })
}

pub fn gravity_move(init_speed: f32, max_speed: f32, border_y: f32, a: f32)
                -> Box<dyn Fn(& dyn tg::object::MovableObject, Clock) -> numeric::Point2f> {
    Box::new(move |p: & dyn tg::object::MovableObject, t: Clock| {
        let p = p.get_position();
        let next_spped = ((t as f32) * a) + init_speed;
        
        let speed = if next_spped < max_speed {
            next_spped
        } else {
            max_speed
        };
        
        let mut next = numeric::Point2f::new(p.x, p.y + (speed));
        if next.y > border_y {
            next.y = border_y;
        }

        next
    })
}

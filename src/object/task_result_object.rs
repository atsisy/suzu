use std::rc::Rc;

use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::numeric;
use torifune::graphics::object::*;
use torifune::graphics::object::sub_screen::*;
use torifune::graphics::*;
use torifune::impl_texture_object_for_wrapped;
use torifune::impl_drawable_object_for_wrapped;

use crate::scene::work_scene::TaskResult;
use crate::core::{GameData, FontID};
use crate::object::effect;

pub struct DrawableTaskResult {
    title_text: EffectableWrap<MovableWrap<VerticalText>>,
    background: SimpleObject,
    canvas: SubScreen,
}

impl DrawableTaskResult {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData,
	       rect_pos: numeric::Rect, task_result: TaskResult,
	       background: SimpleObject, t: Clock) -> Self {
	let font_info = FontInformation::new(
	    game_data.get_font(FontID::JpFude1),
	    numeric::Vector2f::new(30.0, 30.0),
	    ggraphics::Color::from_rgba_u32(0x000000ff));
	DrawableTaskResult {
	    title_text: EffectableWrap::new(
		MovableWrap::new(
		    Box::new(VerticalText::new("お仕事結果".to_string(),
					       numeric::Point2f::new(500.0, 100.0),
					       numeric::Vector2f::new(1.0, 1.0),
					       0.0,
					       0,
					       font_info)), None, t),
		vec![effect::appear_bale_down_from_top(100, t)]),
	    background: background,
	    canvas: SubScreen::new(ctx, rect_pos, 0, ggraphics::Color::from_rgba_u32(0)),
	}
    }
}

impl DrawableComponent for DrawableTaskResult {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
	    sub_screen::stack_screen(ctx, &self.canvas);

	    self.background.draw(ctx)?;
	    self.title_text.draw(ctx)?;

	    sub_screen::pop_screen(ctx);
            self.canvas.draw(ctx).unwrap();
        }
        Ok(())
    }

    fn hide(&mut self) {
        self.canvas.hide()
    }

    fn appear(&mut self) {
        self.canvas.appear()
    }

    fn is_visible(&self) -> bool {
        self.canvas.is_visible()
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth)
    }

    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}

impl DrawableObject for DrawableTaskResult {
    impl_drawable_object_for_wrapped!{canvas}
}

impl TextureObject for DrawableTaskResult {
    impl_texture_object_for_wrapped!{canvas}
}

impl Effectable for DrawableTaskResult {
    fn effect(&mut self, ctx: &ggez::Context, t: Clock) {
	self.title_text.effect(ctx, t);
    }
}

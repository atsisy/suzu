use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::graphics::object::sub_screen;
use sub_screen::SubScreen;
use torifune::numeric;
use torifune::graphics::object::*;
use torifune::graphics::drawable::{DrawableComponent, DrawableObjectEssential};

use crate::object::effect;
use crate::flush_delay_event;
use crate::scene::*;
use crate::object::util_object::*;
use crate::core::{GameData, FontID, TextureID, TileBatchTextureID};

pub trait NotificationContents: DrawableComponent {
    fn required_size(&self) -> numeric::Vector2f;
}

pub struct NotificationContentsData {
    pub header_text: String,
    pub main_text: String,
}

impl NotificationContentsData {
    pub fn new(header_text: String, main_text: String) -> Self {
	NotificationContentsData {
	    header_text: header_text,
	    main_text: main_text,
	}
    }
}

pub struct GeneralNotificationContents {
    main_text: VerticalText,
    header_text: UniText,
    required_size: numeric::Vector2f,
    drwob_essential: DrawableObjectEssential,
}

impl GeneralNotificationContents {
    pub fn new(ctx: &mut ggez::Context, game_data: &GameData, data: NotificationContentsData, depth: i8) -> Self {
	let font_info = FontInformation::new(
	    game_data.get_font(FontID::JpFude1),
	    numeric::Vector2f::new(28.0, 28.0),
	    ggraphics::Color::from_rgba_u32(0xff)
	);
	
	let mut main_text = VerticalText::new(
	    data.main_text.to_string(),
	    numeric::Point2f::new(0.0, 0.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0,
	    font_info,
	);

	let mut header_text = UniText::new(
	    data.header_text.to_string(),
	    numeric::Point2f::new(0.0, 0.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0,
	    font_info,
	);

	let size = numeric::Vector2f::new(
	    header_text.get_drawing_size(ctx).x + 60.0,
	    main_text.get_drawing_size(ctx).y + 120.0
	);

	header_text.make_center(ctx, numeric::Point2f::new(size.x / 2.0, 40.0));
	main_text.make_center(ctx, numeric::Point2f::new(size.x / 2.0, (size.y / 2.0) + 10.0));
	
	GeneralNotificationContents {
	    main_text: main_text,
	    header_text: header_text,
	    required_size: size,
	    drwob_essential: DrawableObjectEssential::new(true, depth),
	}
    }
}

impl DrawableComponent for GeneralNotificationContents {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	if self.is_visible() {
	    self.main_text.draw(ctx)?;
	    self.header_text.draw(ctx)?;
	}

	Ok(())
    }

    
    fn hide(&mut self) {
        self.drwob_essential.visible = false;
    }

    fn appear(&mut self) {
        self.drwob_essential.visible = true;
    }

    fn is_visible(&self) -> bool {
        self.drwob_essential.visible
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        self.drwob_essential.drawing_depth = depth;
    }

    fn get_drawing_depth(&self) -> i8 {
        self.drwob_essential.drawing_depth
    }
}

impl NotificationContents for GeneralNotificationContents {
    fn required_size(&self) -> numeric::Vector2f {
	self.required_size
    }
}

pub struct NotificationArea {
    default_animation_time: Clock,
    background: UniTexture,
    appearance_frame: Option<TileBatchFrame>,
    event_list: DelayEventList<Self>,
    right_top_position: numeric::Point2f,
    contents: Option<Box<dyn NotificationContents>>,
    area: Option<EffectableWrap<MovableWrap<SubScreen>>>,
    drwob_essential: DrawableObjectEssential,
}

impl NotificationArea {
    pub fn new(game_data: &GameData, right_top_position: numeric::Point2f, depth: i8) -> Self {
	let texture = UniTexture::new(
	    game_data.ref_texture(TextureID::Paper1),
	    numeric::Point2f::new(0.0, 0.0),
	    numeric::Vector2f::new(1.0, 1.0),
	    0.0,
	    0
	);

	NotificationArea {
	    default_animation_time: 40,
	    appearance_frame: None,
	    background: texture,
	    event_list: DelayEventList::new(),
	    right_top_position: right_top_position,
	    contents: None,
	    area: None,
	    drwob_essential: DrawableObjectEssential::new(true, depth)
	}
    }

    fn new_appearance_frame(&mut self, ctx: &mut ggez::Context, game_data: &GameData) {
	let size = self.area.as_ref().unwrap().get_drawing_size(ctx);
	self.appearance_frame = Some(
	    TileBatchFrame::new(
		game_data,
		TileBatchTextureID::TaishoStyle1,
		numeric::Rect::new(0.0, 0.0, size.x, size.y),
		numeric::Vector2f::new(0.5, 0.5),
		0
	));
    }
    
    pub fn insert_new_contents(
	&mut self,
	ctx: &mut ggez::Context,
	game_data: &GameData,
	contents: Box<dyn NotificationContents>,
	t: Clock
    ) {
	self.contents = Some(contents);
	self.update_area_canvas(ctx, t);
	self.new_appearance_frame(ctx, game_data);
	self.set_appear_animation(t);
    }

    pub fn insert_new_contents_generic(
	&mut self,
	ctx: &mut ggez::Context,
	game_data: &GameData,
	generic_data: NotificationContentsData,
	t: Clock,
    ) {
	let contents = Box::new(GeneralNotificationContents::new(ctx, game_data, generic_data, 0));
	self.insert_new_contents(ctx, game_data, contents, t);
    }
    
    fn set_hide_animation(&mut self, t: Clock) {
	if let Some(area) = self.area.as_mut() {
	    area.clear_effect();
	    area.add_effect(vec![
		effect::hide_bale_down_from_top(self.default_animation_time, t),
	    ]);

	    let scheduled = t + self.default_animation_time;
	    self.event_list.add_event(
		Box::new(move |slf: &mut NotificationArea, _, _| {
		    slf.area = None;
		    slf.contents = None;
		}), scheduled);
	}
    }
    
    fn set_appear_animation(&mut self, t: Clock) {
	if let Some(area) = self.area.as_mut() {
	    area.clear_effect();
	    area.add_effect(vec![
		effect::appear_bale_down_from_top(self.default_animation_time, t),
	    ]);

	    let scheduled = t + 120;
	    self.event_list.add_event(
		Box::new(move |slf: &mut NotificationArea, _, _| {
		    slf.set_hide_animation(scheduled);
		}), scheduled);
	}
    }

    fn update_area_canvas(&mut self, ctx: &mut ggez::Context, t: Clock) {
	let area_size = self.contents.as_ref().unwrap().required_size();
	self.area = Some(
	    EffectableWrap::new(
		MovableWrap::new(
		    Box::new(SubScreen::new(
			ctx,
			numeric::Rect::new(self.right_top_position.x - area_size.x , 10.0, area_size.x, area_size.y),
			0,
			ggraphics::Color::from_rgba_u32(0),
		    )),
		    None,
		    t
		),
		Vec::new(),
	    )
	);
    }
    
    pub fn update(&mut self, ctx: &mut ggez::Context, game_data: &GameData, t: Clock) {
	flush_delay_event!(self, self.event_list, ctx, game_data, t);
	
	if let Some(area) = self.area.as_mut() {
	    area.move_with_func(t);
	    area.effect(ctx, t);
	}
    }
}

impl DrawableComponent for NotificationArea {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	if self.is_visible() {
	    if let Some(canvas) = self.area.as_mut() {
		sub_screen::stack_screen(ctx, canvas);
		
		self.background.draw(ctx)?;
		
		if let Some(frame) = self.appearance_frame.as_mut() {
		    frame.draw(ctx)?;
		}
		
		if let Some(contents) = self.contents.as_mut() {
		    contents.draw(ctx)?;
		}

		sub_screen::pop_screen(ctx);
		canvas.draw(ctx).unwrap();
	    }
	}

	Ok(())
    }

    fn hide(&mut self) {
        self.drwob_essential.visible = false;
    }

    fn appear(&mut self) {
        self.drwob_essential.visible = true;
    }

    fn is_visible(&self) -> bool {
        self.drwob_essential.visible
    }

    fn set_drawing_depth(&mut self, depth: i8) {
        self.drwob_essential.drawing_depth = depth;
    }

    fn get_drawing_depth(&self) -> i8 {
        self.drwob_essential.drawing_depth
    }
}

use torifune::core::Clock;
use torifune::graphics::drawable::*;
use torifune::graphics::object::*;

use crate::core::{SavableData, SuzuContext, TextureID, TileBatchTextureID};
use crate::scene::*;
use crate::object::save_scene_object::*;
use crate::object::effect_object;
use crate::object::util_object::*;

use crate::flush_delay_event;

pub struct SaveScene {
    background: UniTexture,
    exit_button: SelectButton,
    event_list: DelayEventList<Self>,
    scene_transition_effect: Option<effect_object::ScreenTileEffect>,
    save_entry_table: SaveEntryTable,
    scene_transition: SceneID,
    scene_transition_type: SceneTransition,
    clock: Clock,
}

impl SaveScene {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>) -> Self {
	let save_data_list = (1..=4)
	    .map(|slot_index|
		 match SavableData::new_load(slot_index) {
		     Ok(savable_data) => Some(savable_data) ,
		     Err(_) => None,
		 })
	    .collect();
	    
	let save_entry_table = SaveEntryTable::new(
	    ctx,
	    numeric::Rect::new(50.0, 50.0, 1248.0, 672.0),
	    save_data_list,
	    0
	);

	let background = UniTexture::new(
	    ctx.resource.ref_texture(TextureID::JpHouseTexture),
	    numeric::Point2f::new(0.0, 0.0),
	    numeric::Vector2f::new(0.7, 0.7),
	    0.0,
	    0
	);

	let scene_transition_effect = Some(effect_object::ScreenTileEffect::new(
            ctx,
            TileBatchTextureID::Shoji,
            numeric::Rect::new(
                0.0,
                0.0,
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
            30,
            effect_object::SceneTransitionEffectType::Open,
            effect_object::TilingEffectType::WholeTile,
            -128,
            0,
        ));

	let exit_button = SelectButton::new(
	    ctx,
	    numeric::Rect::new(1000.0, (crate::core::WINDOW_SIZE_Y as f32) - 80.0, 100.0, 50.0),
	    Box::new(UniTexture::new(
		ctx.resource.ref_texture(TextureID::ChoicePanel1),
		numeric::Point2f::new(0.0, 0.0),
		numeric::Vector2f::new(1.0, 1.0),
		0.0,
		0
	    )),
	);
	
        let mut event_list = DelayEventList::new();
        event_list.add_event(
            Box::new(move |slf: &mut Self, _, _| {
                slf.scene_transition_effect = None;
            }),
	    31,
        );
	
        SaveScene {
	    background: background,
	    event_list: event_list,
	    exit_button: exit_button,
	    scene_transition_effect: scene_transition_effect,
	    save_entry_table: save_entry_table,
            scene_transition: SceneID::Save,
	    scene_transition_type: SceneTransition::Keep,
            clock: 0,
        }
    }

    fn exit_scene_poping<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
	self.scene_transition_effect = Some(effect_object::ScreenTileEffect::new(
            ctx,
            TileBatchTextureID::Shoji,
            numeric::Rect::new(
                0.0,
                0.0,
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
            30,
            effect_object::SceneTransitionEffectType::Close,
            effect_object::TilingEffectType::WholeTile,
            -128,
            t,
        ));

	self.event_list.add_event(
            Box::new(move |slf: &mut Self, _, _| {
		slf.scene_transition = SceneID::Scenario;
		slf.scene_transition_type = SceneTransition::PoppingTransition;
            }),
	    31,
        );
    }

    fn load_and_scene_swap<'a>(&mut self, ctx: &mut SuzuContext<'a>, slot: u8, t: Clock) {
	ctx.savable_data.replace(SavableData::new_load(slot).unwrap());

        self.scene_transition_effect = Some(effect_object::ScreenTileEffect::new(
            ctx,
            TileBatchTextureID::Shoji,
            numeric::Rect::new(
                0.0,
                0.0,
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
            30,
            effect_object::SceneTransitionEffectType::Close,
            effect_object::TilingEffectType::WholeTile,
            -128,
            t,
        ));

	self.event_list.add_event(
            Box::new(move |slf: &mut Self, _, _| {
		slf.scene_transition = SceneID::Scenario;
		slf.scene_transition_type = SceneTransition::SwapTransition;
            }),
	    31,
        );
    }
}

impl SceneManager for SaveScene {
    fn mouse_button_up_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        _button: ginput::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
	let t = self.get_current_clock();
	
	match self.save_entry_table.click_handler(ctx, point) {
	    SaveDataOperation::Loading(slot) => {
		self.load_and_scene_swap(ctx, slot, t);
	    },
	    _ => (),
	}

	if self.exit_button.contains(ctx.context, point) {
	    self.exit_scene_poping(ctx, t);
	}
    }

    fn pre_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
	let t = self.get_current_clock();
	
	if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.effect(ctx.context, t);
        }

        flush_delay_event!(self, self.event_list, ctx, self.get_current_clock());
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
	self.background.draw(ctx).unwrap();
	self.save_entry_table.draw(ctx).unwrap();

	self.exit_button.draw(ctx).unwrap();
	
	if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.draw(ctx).unwrap();
        }
    }

    fn post_process<'a>(&mut self, _ctx: &mut SuzuContext<'a>) -> SceneTransition {
        self.update_current_clock();

	self.scene_transition_type
    }

    fn transition(&self) -> SceneID {
        self.scene_transition
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }

    fn update_current_clock(&mut self) {
        self.clock += 1;
    }
}
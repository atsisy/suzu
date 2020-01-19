use super::*;
use crate::core::*;
use crate::core::util;

pub fn create_dobj_random(ctx: &mut ggez::Context, game_data: &GameData,
			  obj_type: DeskObjectType, t: Clock) -> DeskObject {
    let paper_info = CopyingRequestInformation::new_random(game_data,
                                                           GensoDate::new(128, 12, 8),
                                                           GensoDate::new(128, 12, 8));

    DeskObject::new(
        Box::new(OnDeskTexture::new(
            UniTexture::new(
                game_data.ref_texture(TextureID::select_random()),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(0.1, 0.1),
                0.0, 0))),
        Box::new(CopyingRequestPaper::new(ctx, ggraphics::Rect::new(0.0, 0.0, 420.0, 350.0), TextureID::Paper1,
                                          &paper_info,
                                          game_data, t)), 0, obj_type, t)
}

pub fn create_dobj_book_random(_ctx: &mut ggez::Context,
			       game_data: &GameData, obj_type: DeskObjectType, t: Clock) -> DeskObject {
    let texture = *util::random_select(LARGE_BOOK_TEXTURE.iter()).unwrap();
    DeskObject::new(
        Box::new(OnDeskTexture::new(
	    UniTexture::new(
            game_data.ref_texture(texture),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(0.1, 0.1),
            0.0, 0))),
        Box::new(OnDeskTexture::new(
	    UniTexture::new(
            game_data.ref_texture(texture),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(0.3, 0.3),
            0.0, 0))), 0, obj_type, t)
}
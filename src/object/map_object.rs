use std::collections::HashMap;
use std::collections::VecDeque;
use std::rc::Rc;
use std::str::FromStr;

use ggez::graphics as ggraphics;

use torifune::core::Clock;
use torifune::distance;
use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::numeric;
use torifune::debug;

use crate::core::map_parser as mp;
use crate::core::{BookInformation, BookShelfInformation, GameData};
use crate::object::collision::*;
use crate::scene::{SceneID, DelayEventList};
use crate::object::task_object::tt_main_component::CustomerRequest;
use crate::object::task_object::tt_sub_component::{BorrowingInformation, CopyingRequestInformation, ReturnBookInformation, GensoDate};

///
/// ある範囲内に速さを収めたい時に使用する構造体
///
pub struct SpeedBorder {
    pub positive_x: f32,
    pub negative_x: f32,
    pub positive_y: f32,
    pub negative_y: f32,
}

impl SpeedBorder {
    ///
    /// あるx方向の速さを範囲内に丸め込む
    ///
    pub fn round_speed_x(&self, speed: f32) -> f32 {
        if speed > self.positive_x {
            self.positive_x
        } else if speed < self.negative_x {
            self.negative_x
        } else {
            speed
        }
    }

    ///
    /// あるy方向の速さを範囲内に丸め込む
    ///
    pub fn round_speed_y(&self, speed: f32) -> f32 {
        if speed > self.positive_y {
            self.positive_y
        } else if speed < self.negative_y {
            self.negative_y
        } else {
            speed
        }
    }
}

pub struct TextureSpeedInfo {
    speed: numeric::Vector2f,
    speed_border: SpeedBorder,
}

impl TextureSpeedInfo {
    pub fn new(speed: numeric::Vector2f, border: SpeedBorder) -> TextureSpeedInfo {
        TextureSpeedInfo {
            speed: speed,
            speed_border: border,
        }
    }

    pub fn add_speed(&mut self, speed: numeric::Vector2f) {
        self.speed += speed;
    }

    pub fn set_speed(&mut self, speed: numeric::Vector2f) {
        self.speed = speed;
    }

    pub fn set_speed_x(&mut self, speed: f32) {
        self.speed.x = self.speed_border.round_speed_x(speed);
    }

    pub fn set_speed_y(&mut self, speed: f32) {
        self.speed.y = self.speed_border.round_speed_y(speed);
    }

    pub fn get_speed(&self) -> numeric::Vector2f {
        self.speed
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum AnimationType {
    OneShot,
    Loop,
    Times(usize, usize),
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum AnimationStatus {
    Playing,
    OneLoopFinish,
}

struct SeqTexture {
    textures: Vec<Rc<ggraphics::Image>>,
    index: usize,
}

impl SeqTexture {
    pub fn new(textures: Vec<Rc<ggraphics::Image>>) -> Self {
        SeqTexture {
            textures: textures,
            index: 0,
        }
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn current_frame(&self) -> Rc<ggraphics::Image> {
        self.textures[self.index % self.textures.len()].clone()
    }

    pub fn next_frame(
        &mut self,
        t: AnimationType,
    ) -> Result<Rc<ggraphics::Image>, AnimationStatus> {
        self.index += 1;

        match t {
            AnimationType::OneShot | AnimationType::Times(_, _) => {
                if self.index == self.textures.len() {
                    return Err(AnimationStatus::OneLoopFinish);
                }
            }
            _ => (),
        }

        return Ok(self.current_frame());
    }
}

pub struct TextureAnimation {
    textures: Vec<SeqTexture>,
    current_mode: usize,
    object: SimpleObject,
    animation_type: AnimationType,
    next_mode: usize,
    frame_speed: Clock,
}

impl TextureAnimation {
    pub fn new(
        obj: SimpleObject,
        textures: Vec<Vec<Rc<ggraphics::Image>>>,
        mode: usize,
        frame_speed: Clock,
    ) -> Self {
        TextureAnimation {
            textures: textures
                .iter()
                .map(|vec| SeqTexture::new(vec.to_vec()))
                .collect(),
            current_mode: mode,
            object: obj,
            animation_type: AnimationType::Loop,
            next_mode: mode,
            frame_speed: frame_speed,
        }
    }

    pub fn get_object(&self) -> &SimpleObject {
        &self.object
    }

    pub fn get_mut_object(&mut self) -> &mut SimpleObject {
        &mut self.object
    }

    pub fn change_mode(&mut self, mode: usize, animation_type: AnimationType, next_mode: usize) {
        self.current_mode = mode;
        self.next_mode = next_mode;
        self.animation_type = animation_type;
        self.textures[self.current_mode].reset();
    }

    fn next_frame(&mut self) {
        match self.textures[self.current_mode].next_frame(self.animation_type) {
            // アニメーションは再生中. 特に操作は行わず、ただテクスチャを切り替える
            Ok(texture) => self.get_mut_object().replace_texture(texture),

            // アニメーションが終点に到達なんらかの処理を施す必要がある
            Err(status) => {
                // アニメーションに関してイベントが発生. イベントの種類ごとに何ら可の処理を施す
                match status {
                    // 一回のループが終了したらしい. これは、AnimationType::{OneShot, Times}で発生する
                    AnimationStatus::OneLoopFinish => {
                        // 現在のアニメーションのタイプごとに処理を行う
                        let t = &self.animation_type;
                        match t {
                            &AnimationType::OneShot => {
                                // OneShotの場合
                                // デフォルトのループに切り替える
                                self.animation_type = AnimationType::Loop;
                                self.current_mode = self.next_mode;
                            }
                            &AnimationType::Times(mut cur, lim) => {
                                // Timesの場合
                                // ループカウンタをインクリメントする
                                cur += 1;

                                // まだループする予定
                                if cur < lim {
                                    // 最初のテクスチャに戻し、アニメーションを再開
                                    self.textures[self.current_mode].reset();
                                    let texture = self.textures[self.current_mode].current_frame();
                                    self.get_mut_object().replace_texture(texture);
                                } else {
                                    // OneShotの場合と同じく、デフォルトのループに切り替える
                                    self.animation_type = AnimationType::Loop;
                                    self.current_mode = self.next_mode;
                                }
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    pub fn try_next_frame(&mut self, t: Clock) {
        if t % self.frame_speed == 0 {
            self.next_frame();
        }
    }
}

pub struct TwoStepPoint {
    pub previous: numeric::Point2f,
    pub current: numeric::Point2f,
}

impl TwoStepPoint {
    pub fn diff(&self) -> numeric::Vector2f {
        self.current - self.previous
    }

    pub fn update(&mut self, pos: numeric::Point2f) {
        self.previous = self.current;
        self.current = pos;
    }

    pub fn move_diff(&mut self, pos: &numeric::Vector2f) {
        self.previous = self.current;
        self.current += *pos;
    }
}

pub trait OnMap : DrawableComponent {
    // マップ上のテクスチャ描画開始地点を返す
    fn get_map_position(&self) -> numeric::Point2f;

    // マップ上のテクスチャ描画領域の右下の位置を返す
    fn get_map_position_bottom_right(&self, ctx: &mut ggez::Context) -> numeric::Point2f;
    
    // マップ上のテクスチャ描画開始地点を設定する
    fn set_map_position(&mut self, position: numeric::Point2f);
}

pub struct MapObject {
    last_position: numeric::Point2f,
    object: TextureAnimation,
    speed_info: TextureSpeedInfo,
    map_position: TwoStepPoint,
    collision_crop: numeric::Rect,
}

impl MapObject {
    pub fn new(
        obj: SimpleObject,
        textures: Vec<Vec<Rc<ggraphics::Image>>>,
        mode: usize,
        speed_info: TextureSpeedInfo,
        map_position: numeric::Point2f,
        collision_crop: numeric::Rect,
        frame_speed: Clock,
    ) -> MapObject {
        MapObject {
            last_position: obj.get_position(),
            map_position: TwoStepPoint {
                previous: map_position,
                current: map_position,
            },
            speed_info: speed_info,
            object: TextureAnimation::new(obj, textures, mode, frame_speed),
            collision_crop: collision_crop,
        }
    }

    pub fn get_collision_area(&self, ctx: &mut ggez::Context) -> numeric::Rect {
        let croppped_size = self.get_collision_size(ctx);
	let collision_top_offset = self.get_collision_top_offset(ctx);
        let position = self.obj().get_position();

        numeric::Rect::new(
            position.x + collision_top_offset.x,
            position.y + collision_top_offset.y,
            croppped_size.x,
            croppped_size.y,
        )
    }

    pub fn get_collision_size(&self, ctx: &mut ggez::Context) -> numeric::Vector2f {
        let drawing_size = self.obj().get_drawing_size(ctx);

        numeric::Vector2f::new(
            drawing_size.x * (self.collision_crop.w - self.collision_crop.x),
            drawing_size.y * (self.collision_crop.h - self.collision_crop.y),
        )
    }

    fn get_collision_top_offset(&self, ctx: &mut ggez::Context) -> numeric::Vector2f {
	let drawing_size = self.obj().get_drawing_size(ctx);
	
	numeric::Vector2f::new(
            drawing_size.x * self.collision_crop.x,
            drawing_size.y * self.collision_crop.y)
    }

    pub fn speed_info(&self) -> &TextureSpeedInfo {
        &self.speed_info
    }

    pub fn speed_info_mut(&mut self) -> &mut TextureSpeedInfo {
        &mut self.speed_info
    }

    pub fn change_animation_mode(&mut self, mode: usize) {
        self.object.change_mode(mode, AnimationType::Loop, mode);
    }

    pub fn obj(&self) -> &SimpleObject {
        self.object.get_object()
    }

    pub fn obj_mut(&mut self) -> &mut SimpleObject {
        self.object.get_mut_object()
    }

    pub fn get_last_position(&self) -> numeric::Point2f {
        self.last_position
    }

    pub fn undo_move(&mut self) {
        let last = self.get_last_position();
        self.object.get_mut_object().set_position(last);
    }

    fn get_last_move_distance(&self) -> numeric::Vector2f {
        let current = self.object.get_object().get_position();
        numeric::Vector2f::new(
            current.x - self.last_position.x,
            current.y - self.last_position.y,
        )
    }

    pub fn get_last_map_move_distance(&self) -> numeric::Vector2f {
        self.map_position.diff()
    }

    pub fn set_map_position_with_collision_top_offset(&mut self, ctx: &mut ggez::Context, position: numeric::Point2f) {
	let offset = self.get_collision_top_offset(ctx);
        self.map_position.update(position - offset);
    }

    pub fn get_map_position_with_collision_top_offset(&self, ctx: &mut ggez::Context) -> numeric::Point2f {
	let offset = self.get_collision_top_offset(ctx);
	self.map_position.current + offset
    }

    ///
    /// キャラクタテクスチャの上側が衝突した場合
    /// どれだけ、テクスチャを移動させれば良いのかを返す
    ///
    fn fix_collision_above(
        &mut self,
        _ctx: &mut ggez::Context,
        info: &CollisionInformation,
        _: Clock,
    ) -> f32 {
        (info.object1_position.unwrap().y + info.object1_position.unwrap().h + 0.1)
            - info.object2_position.unwrap().y
    }

    ///
    /// キャラクタテクスチャの下側が衝突した場合
    /// どれだけ、テクスチャを移動させれば良いのかを返す
    ///
    fn fix_collision_bottom(
        &mut self,
        ctx: &mut ggez::Context,
        info: &CollisionInformation,
        _: Clock,
    ) -> f32 {
        let area = self.get_collision_size(ctx);
        info.object1_position.unwrap().y - (info.object2_position.unwrap().y + area.y) - 1.0
    }

    ///
    /// キャラクタテクスチャの右側が衝突した場合
    /// どれだけ、テクスチャを移動させれば良いのかを返す
    ///
    fn fix_collision_right(
        &mut self,
        ctx: &mut ggez::Context,
        info: &CollisionInformation,
        _: Clock,
    ) -> f32 {
        let area = self.get_collision_size(ctx);
        (info.object1_position.unwrap().x - 2.0) - (info.object2_position.unwrap().x + area.x)
    }

    ///
    /// キャラクタテクスチャの左側が衝突した場合
    /// どれだけ、テクスチャを移動させれば良いのかを返す
    ///
    fn fix_collision_left(
        &mut self,
        _ctx: &mut ggez::Context,
        info: &CollisionInformation,
        _t: Clock,
    ) -> f32 {
        (info.object1_position.unwrap().x + info.object1_position.unwrap().w + 0.5)
            - info.object2_position.unwrap().x
    }

    ///
    /// 垂直方向の衝突（めり込み）を修正するメソッド
    ///
    pub fn fix_collision_vertical(
        &mut self,
        ctx: &mut ggez::Context,
        info: &CollisionInformation,
        t: Clock,
    ) -> f32 {
        if info.center_diff.unwrap().y < 0.0 {
            return self.fix_collision_bottom(ctx, &info, t);
        } else if info.center_diff.unwrap().y > 0.0 {
            return self.fix_collision_above(ctx, &info, t);
        }

        0.0
    }

    ///
    /// 水平方向の衝突（めり込み）を修正するメソッド
    ///
    pub fn fix_collision_horizon(
        &mut self,
        ctx: &mut ggez::Context,
        info: &CollisionInformation,
        t: Clock,
    ) -> f32 {
        if info.center_diff.unwrap().x < 0.0 {
            return self.fix_collision_right(ctx, &info, t);
        } else if info.center_diff.unwrap().x > 0.0 {
            return self.fix_collision_left(ctx, &info, t);
        }

        0.0
    }

    pub fn update_texture(&mut self, t: Clock) {
        self.object.try_next_frame(t);
    }

    pub fn move_map(&mut self, offset: numeric::Vector2f) {
        self.map_position.move_diff(&offset);
    }

    pub fn update_display_position(&mut self, camera: &numeric::Rect) {
        let dp = mp::map_to_display(&self.map_position.current, camera);
        self.object.get_mut_object().set_position(dp);
    }

    pub fn check_collision_with_character(
        &self,
        ctx: &mut ggez::Context,
        chara: &MapObject,
    ) -> CollisionInformation {
        let a1 = self.get_collision_area(ctx);
        let a2 = chara.get_collision_area(ctx);

        if a1.overlaps(&a2) {
            CollisionInformation::new_collision(
                a1,
                a2,
                numeric::Vector2f::new(a2.x - a1.x, a2.y - a1.y),
            )
        } else {
            CollisionInformation::new_not_collision()
        }
    }
}

impl DrawableComponent for MapObject {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
	    self.obj_mut()
		.draw(ctx)
    		.unwrap();
        }
        Ok(())
    }
    
    fn hide(&mut self) {
	self.obj_mut()
	    .hide()
    }

    fn appear(&mut self) {
	self.obj_mut()
	    .appear()
    }

    fn is_visible(&self) -> bool {
	self.obj()
	    .is_visible()
    }

    fn set_drawing_depth(&mut self, depth: i8) {
	self.obj_mut()
	    .set_drawing_depth(depth)
    }

    fn get_drawing_depth(&self) -> i8 {
	self.obj()
	    .get_drawing_depth()
    }
}

impl OnMap for MapObject {
    // マップ上のテクスチャ描画開始地点を返す
    fn get_map_position(&self) -> numeric::Point2f {
        self.map_position.current
    }
    
    // マップ上のテクスチャ描画領域の右下の位置を返す
    fn get_map_position_bottom_right(&self, ctx: &mut ggez::Context) -> numeric::Point2f {
	self.get_map_position() + self.obj().get_drawing_size(ctx)
    }
    
    // マップ上のテクスチャ描画開始地点を設定する
    fn set_map_position(&mut self, position: numeric::Point2f) {
        self.map_position.update(position);
    }
}

pub struct DamageEffect {
    pub hp_damage: i16,
    pub mp_damage: f32,
}

pub struct AttackCore {
    center_position: numeric::Point2f,
    radius: f32,
}

impl AttackCore {
    pub fn new(center: numeric::Point2f, radius: f32) -> Self {
        AttackCore {
            center_position: center,
            radius: radius,
        }
    }

    pub fn distance(&self, obj: &AttackCore) -> f32 {
        distance!(self.center_position, obj.center_position)
    }

    pub fn check_collision(&self, obj: &AttackCore) -> bool {
        let d = self.distance(obj);
        d < (self.radius + obj.radius)
    }

    pub fn move_diff(&mut self, offset: numeric::Vector2f) {
        self.center_position += offset;
    }
}

pub struct PlayerStatus {
    pub hp: i16,
    pub mp: f32,
}

pub struct PlayableCharacter {
    character: MapObject,
    status: PlayerStatus,
    shelving_book: Vec<BookInformation>,
}

impl PlayableCharacter {
    pub fn new(character: MapObject, status: PlayerStatus) -> Self {
        PlayableCharacter {
            character: character,
            status: status,
            shelving_book: Vec::new(),
        }
    }

    pub fn get_center_map_position(&self, ctx: &mut ggez::Context) -> numeric::Point2f {
        let drawing_size = self.character.obj().get_drawing_size(ctx);
        self.get_map_position() + numeric::Vector2f::new(drawing_size.x / 2.0, drawing_size.y / 2.0)
    }

    pub fn get_character_object(&self) -> &MapObject {
        &self.character
    }

    pub fn get_mut_character_object(&mut self) -> &mut MapObject {
        &mut self.character
    }

    pub fn get_shelving_book(&self) -> &Vec<BookInformation> {
        &self.shelving_book
    }

    pub fn get_shelving_book_mut(&mut self) -> &mut Vec<BookInformation> {
        &mut self.shelving_book
    }

    pub fn update_shelving_book(&mut self, shelving_book: Vec<BookInformation>) {
        self.shelving_book = shelving_book;
    }

    pub fn fix_collision_horizon(
        &mut self,
        ctx: &mut ggez::Context,
        info: &CollisionInformation,
        t: Clock,
    ) -> f32 {
        self.character.fix_collision_horizon(ctx, info, t)
    }

    pub fn fix_collision_vertical(
        &mut self,
        ctx: &mut ggez::Context,
        info: &CollisionInformation,
        t: Clock,
    ) -> f32 {
        self.character.fix_collision_vertical(ctx, info, t)
    }

    pub fn move_map(&mut self, offset: numeric::Vector2f) {
        self.character.move_map(offset);
    }

    pub fn move_map_current_speed_x(&mut self, border: f32) {
        let x_speed = self.get_character_object().speed_info().get_speed().x;
        let overflow = (self.get_map_position().x + x_speed) - border;
        if overflow > 0.0 {
            self.move_map(numeric::Vector2f::new(x_speed - overflow, 0.0))
        } else {
            self.move_map(numeric::Vector2f::new(x_speed, 0.0))
        }
    }

    pub fn move_map_current_speed_y(&mut self, border: f32) {
        let y_speed = self.get_character_object().speed_info().get_speed().y;
        let overflow = (self.get_map_position().y + y_speed) - border;
        if overflow > 0.0 {
            self.move_map(numeric::Vector2f::new(0.0, y_speed - overflow))
        } else {
            self.move_map(numeric::Vector2f::new(0.0, y_speed))
        }
    }

    pub fn attack_damage_check(
        &mut self,
        ctx: &mut ggez::Context,
        attack_core: &AttackCore,
        damage: &DamageEffect,
    ) {
        let center = self.get_map_position() + self.character.obj().get_center_offset(ctx);
        if distance!(center, attack_core.center_position) < attack_core.radius {
            self.status.hp -= damage.hp_damage;
            self.status.mp -= damage.mp_damage;
        }
    }

    pub fn get_speed(&self) -> numeric::Vector2f {
        self.character.speed_info().get_speed()
    }

    pub fn set_speed(&mut self, speed: numeric::Vector2f) {
        self.character.speed_info_mut().set_speed(speed);
    }

    pub fn set_speed_x(&mut self, speed: f32) {
        self.character.speed_info_mut().set_speed_x(speed);
    }

    pub fn set_speed_y(&mut self, speed: f32) {
        self.character.speed_info_mut().set_speed_y(speed);
    }

    pub fn reset_speed(&mut self) {
        self.character
            .speed_info_mut()
            .set_speed(numeric::Vector2f::new(0.0, 0.0));
    }
}

impl DrawableComponent for PlayableCharacter {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
	    self.get_mut_character_object()
		.obj_mut()
		.draw(ctx)
    		.unwrap();
        }
        Ok(())
    }
    
    fn hide(&mut self) {
	self.get_mut_character_object()
	    .hide()
    }

    fn appear(&mut self) {
	self.get_mut_character_object()
	    .appear()
    }

    fn is_visible(&self) -> bool {
	self.get_character_object()
	    .is_visible()
    }

    fn set_drawing_depth(&mut self, depth: i8) {
	self.get_mut_character_object()
	    .set_drawing_depth(depth)
    }

    fn get_drawing_depth(&self) -> i8 {
	self.get_character_object()
	    .get_drawing_depth()
    }
}

impl OnMap for PlayableCharacter {
    // マップ上のテクスチャ描画開始地点を返す
    fn get_map_position(&self) -> numeric::Point2f {
	self.character.get_map_position()
    }
    
    // マップ上のテクスチャ描画領域の右下の位置を返す
    fn get_map_position_bottom_right(&self, ctx: &mut ggez::Context) -> numeric::Point2f {
	self.character.get_map_position_bottom_right(ctx)
    }
    
    // マップ上のテクスチャ描画開始地点を設定する
    fn set_map_position(&mut self, position: numeric::Point2f) {
	self.character.set_map_position(position);
    }
}

pub struct CustomerDestPoint {
    candidates: Vec<numeric::Vector2u>,
}

impl CustomerDestPoint {
    pub fn new(candidates: Vec<numeric::Vector2u>) -> Self {
	CustomerDestPoint {
	    candidates: candidates,
	}
    }
    
    pub fn random_select(&self) -> numeric::Vector2u {
	let random_index = rand::random::<usize>() % self.candidates.len();
        *self.candidates.get(random_index).unwrap()
    }
}

#[derive(Debug)]
pub struct CustomerMoveQueue {
    queue: VecDeque<numeric::Point2f>,
}

impl CustomerMoveQueue {
    pub fn new() -> Self {
	CustomerMoveQueue {
	    queue: VecDeque::new(),
	}
    }
    
    pub fn enqueue(&mut self, points: Vec<numeric::Point2f>) {
	self.queue.extend(points);
    } 

    pub fn dequeue(&mut self) -> Option<numeric::Point2f> {
	self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
	self.queue.len()
    }

    pub fn empty(&self) -> bool {
	self.len() == 0
    }

    pub fn clear(&mut self) {
	self.queue.clear();
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CustomerCharacterStatus {
    Ready = 0,
    Moving,
    WaitOnClerk,
    WaitOnBookShelf,
}

pub struct CustomerCharacter {
    event_list: DelayEventList<Self>,
    character: MapObject,
    move_data: CustomerDestPoint,
    move_queue: CustomerMoveQueue,
    customer_status: CustomerCharacterStatus,
    shopping_is_done: bool,
    current_goal: numeric::Point2f,
}

impl CustomerCharacter {
    pub fn new(character: MapObject, move_data: CustomerDestPoint) -> Self {
        CustomerCharacter {
	    event_list: DelayEventList::new(),
            character: character,
	    move_data: move_data,
	    move_queue: CustomerMoveQueue::new(),
	    customer_status: CustomerCharacterStatus::Ready,
	    shopping_is_done: false,
	    current_goal: numeric::Point2f::new(0.0, 0.0),
        }
    }

    pub fn update_current_destination(&mut self, ctx: &mut ggez::Context,
				      map_data: &mp::StageObjectMap, dest: numeric::Vector2u) -> Option<Vec<numeric::Point2f>> {
	if self.move_queue.empty() {
	    let maybe_start = map_data
		.map_position_to_tile_position(self.character.get_map_position_with_collision_top_offset(ctx));
	    if let Some(map_start_pos) = maybe_start {
		let maybe_route = map_data.find_shortest_route(map_start_pos, dest);
		if let Some(route) = maybe_route {
		    return Some(
			route
    			    .iter()
			    .map(|tp| map_data.tile_position_to_map_position(*tp))
    			    .collect())
		}
	    }
	}

	None
    }

    fn update_animation_mode_with_rad(&mut self, rad: f32) {

	if rad >= 45.0_f32.to_radians() && rad < 135.0_f32.to_radians() {
	    // 上向き
            self.get_mut_character_object()
                .change_animation_mode(0);
	}

	if rad >= 135.0_f32.to_radians() && rad < 225.0_f32.to_radians() {
	    // 左向き
            self.get_mut_character_object()
                .change_animation_mode(3);
	}

	if rad >= 225.0_f32.to_radians() && rad < 315.0_f32.to_radians() {
	    // 下向き
            self.get_mut_character_object()
                .change_animation_mode(1);
	}

	if (rad >= 315.0_f32.to_radians() && rad <= 360.0_f32.to_radians()) ||
	    (rad >= 0.0_f32.to_radians() && rad < 45.0_f32.to_radians()){
		// 右向き
                self.get_mut_character_object()
                    .change_animation_mode(2);
	}
    }

    fn override_move_effect(&mut self, ctx: &mut ggez::Context, goal_point: numeric::Point2f) {
	let current = self.get_character_object().get_map_position_with_collision_top_offset(ctx);
        let offset = numeric::Point2f::new(goal_point.x - current.x, goal_point.y - current.y);

	let speed = if offset.x == 0.0 && offset.y == 0.0 {
	    numeric::Vector2f::new(0.0, 0.0)
	} else {
	    let rad =
		if offset.x >= 0.0 {
		    if offset.y >= 0.0 {
			(offset.y / offset.x).atan()	
		    } else {
			(offset.y / offset.x).atan() + 360.0_f32.to_radians()
		    }
		} else {
		    (offset.y / offset.x).atan() + 180.0_f32.to_radians()
		};
	    let speed = numeric::Vector2f::new(rad.cos() * 1.4, rad.sin() * 1.4);
	    
	    debug::debug_screen_push_text(&format!("rad: {}", rad.to_degrees()));

	    self.update_animation_mode_with_rad(rad);

	    speed
	};

	debug::debug_screen_push_text(&format!("goal: {}:{}, speed: {}:{}", goal_point.x, goal_point.y, speed.x, speed.y));
	
	self.character.speed_info_mut().set_speed(speed);
    }
    
    fn update_move_effect(&mut self, ctx: &mut ggez::Context, map_data: &mp::StageObjectMap, t: Clock) {
	if self.move_queue.empty() {
	    let maybe_next_route = self.update_current_destination(ctx, map_data, self.move_data.random_select());

	    debug::debug_screen_push_text(&format!("{:?}", maybe_next_route));
	    
	    self.event_list.add_event(
		Box::new(move |customer, _, _| {
		    if let Some(next_route) = maybe_next_route {
			customer.move_queue.enqueue(next_route);
			customer.customer_status = CustomerCharacterStatus::Ready;
		    }
		}), t + 100);
	    
	    self.customer_status = CustomerCharacterStatus::WaitOnBookShelf;
	    return ();
	}

	let maybe_next_position = self.move_queue.dequeue();
	if let Some(next_position) = maybe_next_position {
	    debug::debug_screen_push_text(&format!("next: {:?}", next_position));
	    self.override_move_effect(ctx, next_position);
	    self.current_goal = next_position;
	    self.customer_status = CustomerCharacterStatus::Moving;
	}
    }

    pub fn set_destination_forced(&mut self, ctx: &mut ggez::Context,
				  map_data: &mp::StageObjectMap, dest: numeric::Vector2u) {
	self.move_queue.clear();
	let maybe_next_route = self.update_current_destination(ctx, map_data, dest);
	
	debug::debug_screen_push_text(&format!("{:?}", maybe_next_route));

	if let Some(next_route) = maybe_next_route {
	    self.move_queue.enqueue(next_route);
	    self.customer_status = CustomerCharacterStatus::Ready;
	}
    }

    pub fn reset_speed(&mut self) {
        self.character
            .speed_info_mut()
            .set_speed(numeric::Vector2f::new(0.0, 0.0));
    }

    fn is_goal_now(&mut self, ctx: &mut ggez::Context) -> bool {
	let current = self.get_character_object().get_map_position_with_collision_top_offset(ctx);
	distance!(current, self.current_goal) < 1.5
    }

    fn flush_delay_event(&mut self, ctx: &mut ggez::Context, game_data: &GameData, t: Clock) {
	// 最後の要素の所有権を移動
        while let Some(event) = self.event_list.move_top() {
            // 時間が来ていない場合は、取り出した要素をリストに戻して処理ループを抜ける
            if event.run_time > t {
                self.event_list.add(event);
                break;
            }
	    
            // 所有権を移動しているため、selfを渡してもエラーにならない
            (event.func)(self, ctx, game_data);
        }
    }

    fn generate_hold_request(&mut self, game_data: &GameData) -> CustomerRequest {
	let random_select = rand::random::<usize>() % 3;
        match random_select {
            0 => CustomerRequest::Borrowing(BorrowingInformation::new_random(
                game_data,
                GensoDate::new(128, 12, 20),
                GensoDate::new(128, 12, 20),
            )),
            1 => CustomerRequest::Returning(ReturnBookInformation::new_random(
                game_data,
                GensoDate::new(128, 12, 20),
                GensoDate::new(128, 12, 20),
            )),
            _ => CustomerRequest::Copying(CopyingRequestInformation::new_random(
                game_data,
                GensoDate::new(12, 12, 12),
                GensoDate::new(12, 12, 12),
            )),
        }
    }
    
    pub fn try_update_move_effect(&mut self, ctx: &mut ggez::Context, game_data: &GameData,
				  map_data: &mp::StageObjectMap, counter: numeric::Vector2u, t: Clock) {
	self.flush_delay_event(ctx, game_data, t);
	
	match self.customer_status {
	    CustomerCharacterStatus::Ready => {
		self.update_move_effect(ctx, map_data, t);
	    },
	    CustomerCharacterStatus::Moving => {
		//println!("queue: {:?}", self.move_queue);
		// debug::debug_screen_push_text(&format!("goal: {}:{}, current: {}:{}",
		//  				       self.current_goal.x, self.current_goal.y,
		//  				       self.get_map_position().x, self.get_map_position().y));
		if self.is_goal_now(ctx) {
		    let goal = self.current_goal;
		    
		    debug::debug_screen_push_text(&format!("goal: {:?}", goal));
		    self.get_mut_character_object().set_map_position_with_collision_top_offset(ctx, goal);
		    self.customer_status = CustomerCharacterStatus::Ready;
		    self.reset_speed();

		    if !self.shopping_is_done &&
			map_data.map_position_to_tile_position(goal).unwrap() == counter {
			self.customer_status = CustomerCharacterStatus::WaitOnClerk;
			self.shopping_is_done = true;
		    }
		}
	    },
	    CustomerCharacterStatus::WaitOnClerk => {
		
	    }
	    _ => (),
	}
    }

    pub fn is_wait_on_clerk(&self) -> bool {
	self.customer_status == CustomerCharacterStatus::WaitOnClerk
    }
    
    pub fn check_rise_hand(&mut self, game_data: &GameData) -> Option<CustomerRequest> {
	if self.customer_status == CustomerCharacterStatus::WaitOnClerk {
	    Some(self.generate_hold_request(game_data))
	} else {
	    None
	}
    }

    pub fn get_map_position(&self) -> numeric::Point2f {
        self.character.get_map_position()
    }

    pub fn get_character_object(&self) -> &MapObject {
        &self.character
    }

    pub fn get_mut_character_object(&mut self) -> &mut MapObject {
        &mut self.character
    }

    pub fn fix_collision_horizon(
        &mut self,
        ctx: &mut ggez::Context,
        info: &CollisionInformation,
        t: Clock,
    ) -> f32 {
        self.character.fix_collision_horizon(ctx, info, t)
    }

    pub fn fix_collision_vertical(
        &mut self,
        ctx: &mut ggez::Context,
        info: &CollisionInformation,
        t: Clock,
    ) -> f32 {
        self.character.fix_collision_vertical(ctx, info, t)
    }

    pub fn move_map(&mut self, offset: numeric::Vector2f) {
        self.character.move_map(offset);
    }

    pub fn move_map_current_speed_x(&mut self) {
        self.move_map(numeric::Vector2f::new(
            self.get_character_object().speed_info().get_speed().x,
            0.0,
        ))
    }

    pub fn move_map_current_speed_y(&mut self) {
        self.move_map(numeric::Vector2f::new(
            0.0,
            self.get_character_object().speed_info().get_speed().y,
        ))
    }

    pub fn get_attack_core(&self, ctx: &mut ggez::Context) -> AttackCore {
        AttackCore::new(
            self.character.get_map_position() + self.character.obj().get_center_offset(ctx),
            10.0,
        )
    }
}

impl DrawableComponent for CustomerCharacter {
    fn draw(&mut self, ctx:
	    &mut ggez::Context) -> ggez::GameResult<()> {
	if self.is_visible() {
	    self.get_mut_character_object()
		.draw(ctx)
    		.unwrap();
	}
	Ok(())
    }
    
    fn hide(&mut self) {
	self.get_mut_character_object()
	    .hide()
    }

    fn appear(&mut self) {
	self.get_mut_character_object()
	    .appear()
    }

    fn is_visible(&self) -> bool {
	self.get_character_object()
	    .is_visible()
    }

    fn set_drawing_depth(&mut self, depth: i8) {
	self.get_mut_character_object()
	    .set_drawing_depth(depth)
    }

    fn get_drawing_depth(&self) -> i8 {
	self.get_character_object()
	    .get_drawing_depth()
    }
}

impl OnMap for CustomerCharacter {
    // マップ上のテクスチャ描画開始地点を返す
    fn get_map_position(&self) -> numeric::Point2f {
	self.character.get_map_position()
    }
    
    // マップ上のテクスチャ描画領域の右下の位置を返す
    fn get_map_position_bottom_right(&self, ctx: &mut ggez::Context) -> numeric::Point2f {
	self.character.get_map_position_bottom_right(ctx)
    }
    
    // マップ上のテクスチャ描画開始地点を設定する
    fn set_map_position(&mut self, position: numeric::Point2f) {
	self.character.set_map_position(position);
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum EventTrigger {
    Action,
    Touch,
}

impl FromStr for EventTrigger {
    type Err = ();

    fn from_str(trigger_str: &str) -> Result<Self, Self::Err> {
        match trigger_str {
            "action" => Ok(Self::Action),
            "touch" => Ok(Self::Touch),
            _ => panic!("Error: EventTrigger::from_str"),
        }
    }
}

pub trait MapEvent {
    fn get_trigger_method(&self) -> EventTrigger;
}

pub struct MapTextEvent {
    trigger: EventTrigger,
    text: String,
}

impl MapTextEvent {
    pub fn from_toml_object(toml_script: &toml::value::Value) -> Self {
        MapTextEvent {
            trigger: EventTrigger::from_str(toml_script.get("trigger").unwrap().as_str().unwrap())
                .unwrap(),
            text: toml_script
                .get("text")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string(),
        }
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }
}

impl MapEvent for MapTextEvent {
    fn get_trigger_method(&self) -> EventTrigger {
        self.trigger
    }
}

pub struct MapEventSceneSwitch {
    trigger: EventTrigger,
    switch_scene: SceneID,
}

impl MapEventSceneSwitch {
    pub fn new(trigger: EventTrigger, switch_scene: SceneID) -> Self {
        MapEventSceneSwitch {
            trigger: trigger,
            switch_scene: switch_scene,
        }
    }

    pub fn from_toml_object(toml_script: &toml::value::Value) -> Self {
        MapEventSceneSwitch {
            trigger: EventTrigger::from_str(toml_script.get("trigger").unwrap().as_str().unwrap())
                .unwrap(),
            switch_scene: SceneID::from_str(
                toml_script
                    .get("switch-scene-id")
                    .unwrap()
                    .as_str()
                    .unwrap(),
            )
            .unwrap(),
        }
    }

    pub fn get_switch_scene_id(&self) -> SceneID {
        self.switch_scene
    }
}

impl MapEvent for MapEventSceneSwitch {
    fn get_trigger_method(&self) -> EventTrigger {
        self.trigger
    }
}

pub struct BookStoreEvent {
    trigger: EventTrigger,
    book_shelf_info: BookShelfInformation,
}

impl BookStoreEvent {
    pub fn from_toml_object(toml_script: &toml::value::Value) -> Self {
        let shelf_info = toml_script.get("shelf-info").unwrap().as_table().unwrap();
        let book_shelf_info = BookShelfInformation::new(
            shelf_info
                .get("begin-number")
                .unwrap()
                .as_integer()
                .unwrap() as u16,
            shelf_info.get("end-number").unwrap().as_integer().unwrap() as u16,
        );

        BookStoreEvent {
            trigger: EventTrigger::from_str(toml_script.get("trigger").unwrap().as_str().unwrap())
                .unwrap(),
            book_shelf_info: book_shelf_info,
        }
    }

    pub fn get_book_shelf_info(&self) -> &BookShelfInformation {
        &self.book_shelf_info
    }
}

impl MapEvent for BookStoreEvent {
    fn get_trigger_method(&self) -> EventTrigger {
        self.trigger
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum BuiltinEventSymbol {
    SelectShelvingBook = 0,
}

impl FromStr for BuiltinEventSymbol {
    type Err = ();

    fn from_str(builtin_event_symbol: &str) -> Result<Self, Self::Err> {
        match builtin_event_symbol {
            "select-shelving-book" => Ok(Self::SelectShelvingBook),
            _ => panic!("Error: BuiltinEventSymbol::from_str"),
        }
    }
}

#[derive(Clone, Copy)]
pub struct BuiltinEvent {
    trigger: EventTrigger,
    event_symbol: BuiltinEventSymbol,
}

impl BuiltinEvent {
    pub fn from_toml_object(toml_script: &toml::value::Value) -> Self {
        let builtin_event_info = toml_script
            .get("builtin-event-info")
            .unwrap()
            .as_table()
            .unwrap();
        BuiltinEvent {
            trigger: EventTrigger::from_str(toml_script.get("trigger").unwrap().as_str().unwrap())
                .unwrap(),
            event_symbol: BuiltinEventSymbol::from_str(
                builtin_event_info.get("symbol").unwrap().as_str().unwrap(),
            )
            .unwrap(),
        }
    }

    pub fn get_event_symbol(&self) -> BuiltinEventSymbol {
        self.event_symbol
    }
}

impl MapEvent for BuiltinEvent {
    fn get_trigger_method(&self) -> EventTrigger {
        self.trigger
    }
}

pub enum MapEventElement {
    TextEvent(MapTextEvent),
    SwitchScene(MapEventSceneSwitch),
    BookStoreEvent(BookStoreEvent),
    BuiltinEvent(BuiltinEvent),
}

impl MapEvent for MapEventElement {
    fn get_trigger_method(&self) -> EventTrigger {
        match self {
            Self::TextEvent(text) => text.get_trigger_method(),
            Self::SwitchScene(switch_scene) => switch_scene.get_trigger_method(),
            Self::BookStoreEvent(book_store_event) => book_store_event.get_trigger_method(),
            Self::BuiltinEvent(builtin_event) => builtin_event.get_trigger_method(),
        }
    }
}

pub struct MapEventList {
    event_table: HashMap<numeric::Point2i, MapEventElement>,
}

impl MapEventList {
    pub fn from_file(file_path: &str) -> Self {
        let mut table = HashMap::new();

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => panic!("Failed to read: {}", file_path),
        };

        let root = content.parse::<toml::Value>().unwrap();
        let array = root["event-panel"].as_array().unwrap();

        for elem in array {
            let position_data = elem.get("position").unwrap().as_table().unwrap();
            let position = numeric::Point2i::new(
                position_data.get("x").unwrap().as_integer().unwrap() as i32,
                position_data.get("y").unwrap().as_integer().unwrap() as i32,
            );
            if let Some(type_info) = elem.get("type") {
                match type_info.as_str().unwrap() {
                    "text" => {
                        table.insert(
                            position,
                            MapEventElement::TextEvent(MapTextEvent::from_toml_object(elem)),
                        );
                    }
                    "switch-scene" => {
                        table.insert(
                            position,
                            MapEventElement::SwitchScene(MapEventSceneSwitch::from_toml_object(
                                elem,
                            )),
                        );
                    }
                    "book-shelf" => {
                        table.insert(
                            position,
                            MapEventElement::BookStoreEvent(BookStoreEvent::from_toml_object(elem)),
                        );
                    }
                    "builtin-event" => {
                        table.insert(
                            position,
                            MapEventElement::BuiltinEvent(BuiltinEvent::from_toml_object(elem)),
                        );
                    }
                    _ => eprintln!("Error"),
                }
            } else {
                eprintln!("Error");
            }
        }

        MapEventList { event_table: table }
    }

    pub fn register_event(&mut self, point: numeric::Point2i, event: MapEventElement) -> &mut Self {
        self.event_table.insert(point, event);
        self
    }

    pub fn check_event(
        &self,
        trigger: EventTrigger,
        point: numeric::Point2i,
    ) -> Option<&MapEventElement> {
        if let Some(event_element) = self.event_table.get(&point) {
            if event_element.get_trigger_method() == trigger {
                return Some(&event_element);
            }
        }

        None
    }
}

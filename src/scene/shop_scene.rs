use std::cell::RefCell;
use std::cmp::PartialOrd;
use std::collections::VecDeque;
use std::rc::Rc;

use torifune::device as tdev;
use torifune::graphics::object::*;

use ggez::graphics as ggraphics;
use tdev::VirtualKey;
use torifune::core::Clock;
use torifune::core::Updatable;
use torifune::graphics::drawable::*;

use ggez::input::mouse::MouseButton;
use torifune::debug;
use torifune::numeric;

use super::*;
use crate::core::map_parser as mp;
use crate::core::{
    FontID, GameResource, SuzuContext, TileBatchTextureID,
};
use crate::flush_delay_event;
use crate::object::effect_object;
use crate::object::map_object::*;
use crate::object::notify;
use crate::object::scenario::*;
use crate::object::shop_object::*;
use crate::object::task_object::tt_main_component::CustomerRequest;
use crate::object::*;
use effect_object::{SceneTransitionEffectType, TilingEffectType};
use notify::*;

struct CharacterGroup {
    group: Vec<CustomerCharacter>,
    drwob_essential: DrawableObjectEssential,
}

impl CharacterGroup {
    pub fn new() -> Self {
        CharacterGroup {
            group: Vec::new(),
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    #[inline(always)]
    pub fn add(&mut self, character: CustomerCharacter) {
        self.group.push(character);
    }

    #[inline(always)]
    pub fn drain_remove_if<F>(&mut self, f: F) -> Vec<CustomerCharacter>
    where
        F: Fn(&CustomerCharacter) -> bool,
    {
        let mut removed = Vec::new();

        for index in (0..self.group.len()).rev() {
            if f(self.group.get(index).as_ref().unwrap()) {
                let removed_character = self.group.swap_remove(index);
                removed.push(removed_character);
            }
        }

        removed
    }

    #[inline(always)]
    pub fn remove_if<F>(&mut self, f: F)
    where
        F: Fn(&CustomerCharacter) -> bool,
    {
        self.group.retain(|c| !f(c));
    }

    pub fn sort_by_y_position(&mut self) {
        self.group.sort_by(|a, b| {
            a.get_map_position()
                .y
                .partial_cmp(&b.get_map_position().y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn len(&self) -> usize {
        self.group.len()
    }

    pub fn move_and_collision_check(
        &mut self,
        ctx: &mut ggez::Context,
        camera: &numeric::Rect,
        tile_map: &mp::StageObjectMap,
        t: Clock,
    ) {
        self.group.iter_mut().for_each(|customer| {
            ShopScene::customer_move_and_collision_check(ctx, customer, camera, tile_map, t)
        });
    }

    pub fn iter(&self) -> std::slice::Iter<CustomerCharacter> {
        self.group.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<CustomerCharacter> {
        self.group.iter_mut()
    }
}

impl DrawableComponent for CharacterGroup {
    #[inline(always)]
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        self.group.iter_mut().for_each(|e| {
            e.get_mut_character_object().obj_mut().draw(ctx).unwrap();
        });
        Ok(())
    }

    #[inline(always)]
    fn hide(&mut self) {
        self.drwob_essential.visible = false;
    }

    #[inline(always)]
    fn appear(&mut self) {
        self.drwob_essential.visible = true;
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
        self.drwob_essential.visible
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
        self.drwob_essential.drawing_depth = depth;
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
        self.drwob_essential.drawing_depth
    }
}

///
/// ## マップ上のデータをまとめる構造体
///
/// ### tile_map
/// tilesetで構成された描画可能なマップ
///
/// ### event_map
/// マップ上のイベントをまとめておく構造体
///
/// ### scenario_box
/// マップ上に表示されるテキストボックス
///
struct MapData {
    pub tile_map: mp::StageObjectMap,
    pub event_map: MapEventList,
    pub scenario_box: Option<ScenarioBox>,
}

impl MapData {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        map_id: u32,
        camera: Rc<RefCell<numeric::Rect>>,
    ) -> Self {
        let map_constract_data = ctx.resource.get_map_data(map_id).unwrap();

        MapData {
            tile_map: mp::StageObjectMap::new(
                ctx.context,
                &map_constract_data.map_file_path,
                camera.clone(),
                numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
                numeric::Vector2f::new(6.0, 6.0),
            ),
            event_map: MapEventList::from_file(&map_constract_data.event_map_file_path),
            scenario_box: None,
        }
    }

    pub fn check_event_panel(
        &self,
        trigger: EventTrigger,
        point: numeric::Point2f,
        _t: Clock,
    ) -> Option<&MapEventElement> {
        let tile_size = self.tile_map.get_tile_drawing_size();
        self.event_map.check_event(
            trigger,
            numeric::Point2i::new(
                (point.x as f32 / tile_size.x) as i32,
                (point.y as f32 / tile_size.y) as i32,
            ),
        )
    }
}

struct MapObjectDrawer<'a> {
    ref_list: Vec<Box<&'a mut dyn OnMap>>,
}

impl<'a> MapObjectDrawer<'a> {
    pub fn new() -> MapObjectDrawer<'a> {
        MapObjectDrawer {
            ref_list: Vec::new(),
        }
    }

    pub fn add(&mut self, onmap: &'a mut dyn OnMap) {
        self.ref_list.push(Box::new(onmap));
    }

    pub fn sort(&mut self, ctx: &mut ggez::Context) {
        self.ref_list.sort_by(|a, b| {
            a.get_map_position_bottom_right(ctx)
                .y
                .partial_cmp(&b.get_map_position_bottom_right(ctx).y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    pub fn draw(&mut self, ctx: &mut ggez::Context) {
        for obj in &mut self.ref_list {
            obj.draw(ctx).unwrap();
        }

        self.ref_list.clear();
    }
}

struct ShopClock {
    hour: u8,
    minute: u8,
}

impl ShopClock {
    pub fn new(hour: u8, minute: u8) -> Self {
        ShopClock {
            hour: hour,
            minute: minute,
        }
    }

    pub fn add_minute(&mut self, minute: u8) {
        self.minute += minute;

        self.add_hour(self.minute / 60);

        self.minute = self.minute % 60;
    }

    pub fn add_hour(&mut self, hour: u8) {
        self.hour += hour;
        self.hour = self.hour % 24;
    }

    pub fn is_past(&self, hour: u8, minute: u8) -> bool {
        match self.hour.cmp(&hour) {
            std::cmp::Ordering::Greater => true,
            std::cmp::Ordering::Equal => match self.minute.cmp(&minute) {
                std::cmp::Ordering::Greater => true,
                _ => false,
            },
            std::cmp::Ordering::Less => false,
        }
    }

    pub fn equals(&self, hour: u8, minute: u8) -> bool {
        self.hour == hour && self.minute == minute
    }
}

impl std::fmt::Display for ShopClock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}:{})", self.hour, self.minute)
    }
}

///
/// # 夢の中のステージ
///
/// ## フィールド
/// ### player
/// プレイキャラ
///
/// ### key_listener
/// キー監視用
///
/// ### map_event_lsit
/// マップ上のイベント
///
/// ### clock
/// 基準クロック
///
/// ### tile_map
/// マップ情報
///
/// ### camera
/// マップを覗くカメラ
///
pub struct ShopScene {
    player: PlayableCharacter,
    character_group: CharacterGroup,
    shop_special_object: ShopSpecialObject,
    key_listener: tdev::KeyboardListener,
    clock: Clock,
    shop_clock: ShopClock,
    map: MapData,
    event_list: DelayEventList<Self>,
    shop_menu: ShopMenuMaster,
    customer_request_queue: VecDeque<CustomerRequest>,
    customer_queue: VecDeque<CustomerCharacter>,
    camera: Rc<RefCell<numeric::Rect>>,
    dark_effect_panel: DarkEffectPanel,
    transition_status: SceneTransition,
    transition_scene: SceneID,
    scene_transition_effect: Option<effect_object::ScreenTileEffect>,
    notification_area: NotificationArea,
}

impl ShopScene {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>, map_id: u32) -> ShopScene {
        let key_listener =
            tdev::KeyboardListener::new_masked(vec![tdev::KeyInputDevice::GenericKeyboard], vec![]);

        let camera = Rc::new(RefCell::new(numeric::Rect::new(0.0, 0.0, 1366.0, 768.0)));

        let map_position = numeric::Point2f::new(800.0, 830.0);

        let player = PlayableCharacter::new(character_factory::create_character(
            character_factory::CharacterFactoryOrder::PlayableDoremy1,
            ctx.resource,
            &camera.borrow(),
            map_position,
        ));

        let mut character_group = CharacterGroup::new();
        character_group.add(CustomerCharacter::new(
            ctx.resource,
            character_factory::create_character(
                character_factory::CharacterFactoryOrder::CustomerSample,
                ctx.resource,
                &camera.borrow(),
                numeric::Point2f::new(1170.0, 870.0),
            ),
            CustomerDestPoint::new(vec![
                numeric::Vector2u::new(10, 3),
                numeric::Vector2u::new(6, 3),
                numeric::Vector2u::new(4, 10),
            ]),
        ));

        let mut map = MapData::new(ctx, map_id, camera.clone());
        map.tile_map.build_collision_map();

        ShopScene {
            player: player,
            character_group: character_group,
            shop_special_object: ShopSpecialObject::new(),
            key_listener: key_listener,
            clock: 0,
            shop_clock: ShopClock::new(8, 0),
            map: map,
            event_list: DelayEventList::new(),
            shop_menu: ShopMenuMaster::new(ctx, numeric::Vector2f::new(450.0, 768.0), 0),
            customer_request_queue: VecDeque::new(),
            customer_queue: VecDeque::new(),
            dark_effect_panel: DarkEffectPanel::new(
                ctx.context,
                numeric::Rect::new(0.0, 0.0, 1366.0, 768.0),
                0,
            ),
            camera: camera,
            transition_scene: SceneID::SuzunaShop,
            transition_status: SceneTransition::Keep,
            scene_transition_effect: None,
            notification_area: NotificationArea::new(
                ctx.resource,
                numeric::Point2f::new((crate::core::WINDOW_SIZE_X - 20) as f32, 20.0),
                0,
            ),
        }
    }

    ///
    /// キー入力のイベントハンドラ
    ///
    fn check_key_event(&mut self, ctx: &ggez::Context) {
        if self.map.scenario_box.is_none() {
            //self.player.reset_speed();
            if self
                .key_listener
                .current_key_status(ctx, &VirtualKey::RightSub)
                == tdev::KeyStatus::Pressed
            {
                self.right_key_handler();
            }

            if self
                .key_listener
                .current_key_status(ctx, &VirtualKey::LeftSub)
                == tdev::KeyStatus::Pressed
            {
                self.left_key_handler();
            }

            if self
                .key_listener
                .current_key_status(ctx, &VirtualKey::UpSub)
                == tdev::KeyStatus::Pressed
            {
                self.up_key_handler();
            }

            if self
                .key_listener
                .current_key_status(ctx, &VirtualKey::DownSub)
                == tdev::KeyStatus::Pressed
            {
                self.down_key_handler();
            }
        }
    }

    ///
    /// カメラを動かすメソッド
    ///
    pub fn move_camera(&mut self, offset: numeric::Vector2f) {
        self.camera.borrow_mut().x += offset.x;
        self.camera.borrow_mut().y += offset.y;
    }

    pub fn set_camera_x(&mut self, offset: f32) {
        self.camera.borrow_mut().x = offset;
    }

    pub fn set_camera_y(&mut self, offset: f32) {
        self.camera.borrow_mut().y = offset;
    }

    pub fn set_camera(&mut self, offset: numeric::Vector2f) {
        self.camera.borrow_mut().x = offset.x;
        self.camera.borrow_mut().y = offset.y;
    }
    
    fn right_key_handler(&mut self) {
        self.player
            .get_mut_character_object()
            .change_animation_mode(2);
        self.player.set_speed_x(4.0);
    }

    fn left_key_handler(&mut self) {
        self.player
            .get_mut_character_object()
            .change_animation_mode(3);
        self.player.set_speed_x(-4.0);
    }

    fn up_key_handler(&mut self) {
        self.player
            .get_mut_character_object()
            .change_animation_mode(1);
        self.player.set_speed_y(-4.0);
    }

    fn down_key_handler(&mut self) {
        self.player
            .get_mut_character_object()
            .change_animation_mode(0);
        self.player.set_speed_y(4.0);
    }

    pub fn customer_move_and_collision_check(
        ctx: &mut ggez::Context,
        customer: &mut CustomerCharacter,
        camera: &numeric::Rect,
        tile_map: &mp::StageObjectMap,
        t: Clock,
    ) {
        customer.move_map_current_speed_y();

        // 当たり判定の前に描画位置を決定しないとバグる。この仕様も直すべき
        customer
            .get_mut_character_object()
            .update_display_position(camera);

        ShopScene::check_collision_vertical(ctx, customer.get_mut_character_object(), tile_map, t);
        customer
            .get_mut_character_object()
            .update_display_position(camera);

        customer.move_map_current_speed_x();
        customer
            .get_mut_character_object()
            .update_display_position(camera);
        ShopScene::check_collision_horizon(ctx, customer.get_mut_character_object(), tile_map, t);
        customer
            .get_mut_character_object()
            .update_display_position(camera);
    }

    fn fix_camera_position(&self) -> numeric::Point2f {
        numeric::Point2f::new(
            if self.player.get_map_position().x >= 800.0 {
                self.player.get_character_object().obj().get_position().x
            } else if self.player.get_map_position().x >= 650.0 {
                650.0
            } else {
                self.player.get_map_position().x
            },
            if self.player.get_map_position().y >= 1130.0 {
                self.player.get_character_object().obj().get_position().y
            } else if self.player.get_map_position().y >= 400.0 {
                400.0
            } else {
                self.player.get_map_position().y
            },
        )
    }

    ///
    /// マップオブジェクトとの衝突を調べるためのメソッド
    ///
    fn check_collision_horizon(
        ctx: &mut ggez::Context,
        character: &mut MapObject,
        tile_map: &mp::StageObjectMap,
        t: Clock,
    ) {
        let collision_info = tile_map.check_character_collision(ctx, character);

        // 衝突していたか？
        if collision_info.collision {
            // 修正動作
            let diff = character.fix_collision_horizon(ctx, &collision_info, t);
            character.move_map(numeric::Vector2f::new(diff, 0.0));
        }
    }

    ///
    /// マップオブジェクトとの衝突を調べるためのメソッド
    ///
    fn check_collision_vertical(
        ctx: &mut ggez::Context,
        character: &mut MapObject,
        tile_map: &mp::StageObjectMap,
        t: Clock,
    ) {
        let collision_info = tile_map.check_character_collision(ctx, character);

        // 衝突していたか？
        if collision_info.collision {
            // 修正動作
            let diff = character.fix_collision_vertical(ctx, &collision_info, t);
            character.move_map(numeric::Vector2f::new(0.0, diff));
        }
    }

    ///
    /// PlayerCharacterのマップチップとのX方向の衝突を修正する
    ///
    fn playable_check_collision_horizon(&mut self, ctx: &mut ggez::Context) {
        let t = self.get_current_clock();

        // プレイヤーのマップチップに対する衝突を修正
        Self::check_collision_horizon(
            ctx,
            self.player.get_mut_character_object(),
            &self.map.tile_map,
            t,
        );

        // プレイヤーのマップ座標を更新
        self.player
            .get_mut_character_object()
            .update_display_position(&self.camera.borrow());

        // カメラをプレイヤーにフォーカス
        self.focus_camera_playable_character_x();
    }

    ///
    /// PlayerCharacterのマップチップとのY方向の衝突を修正する
    ///
    fn playable_check_collision_vertical(&mut self, ctx: &mut ggez::Context) {
        let t = self.get_current_clock();

        // プレイヤーのマップチップに対する衝突を修正
        Self::check_collision_vertical(
            ctx,
            self.player.get_mut_character_object(),
            &self.map.tile_map,
            t,
        );

        // プレイヤーのマップ座標を更新
        self.player
            .get_mut_character_object()
            .update_display_position(&self.camera.borrow());

        // カメラをプレイヤーにフォーカス
        self.focus_camera_playable_character_y();
    }

    fn focus_camera_playable_character_x(&mut self) {
        // カメラとプレイヤーキャラクターの差分を計算し、プレイヤーが中心に来るようにカメラを移動
        let player_diff = self.player.get_mut_character_object().obj().get_position()
            - self.fix_camera_position();
        self.move_camera(numeric::Vector2f::new(player_diff.x, 0.0));
    }

    fn focus_camera_playable_character_y(&mut self) {
        // カメラとプレイヤーキャラクターの差分を計算し、プレイヤーが中心に来るようにカメラを移動
        let player_diff = self.player.get_mut_character_object().obj().get_position()
            - self.fix_camera_position();
        self.move_camera(numeric::Vector2f::new(0.0, player_diff.y));
    }

    ///
    /// PlayerCharacterの他キャラクターとのX方向の衝突を修正する
    ///
    fn check_character_collision_x(&mut self, ctx: &mut ggez::Context, t: Clock) {
        // マップ座標を更新
        self.player
            .get_mut_character_object()
            .update_display_position(&self.camera.borrow());

        // 他キャラクターすべてとの衝突判定を行う
        for e in self.character_group.iter_mut() {
            // 他キャラクターのマップ座標を更新
            e.get_mut_character_object()
                .update_display_position(&self.camera.borrow());

            // 衝突情報を取得
            let collision_info = self
                .player
                .get_character_object()
                .check_collision_with_character(ctx, e.get_character_object());

            // collisionフィールドがtrueなら、衝突している
            if collision_info.collision {
                // プレイヤーと他キャラクターの衝突状況から、プレイヤーがどれだけ、突き放されればいいのか計算
                let diff = self
                    .player
                    .get_mut_character_object()
                    .fix_collision_horizon(ctx, &collision_info, t);

                // プレイヤーの突き放し距離分動かす
                self.player
                    .get_mut_character_object()
                    .move_map(numeric::Vector2f::new(-diff, 0.0));

                // プレイヤーのマップ座標を更新
                self.player
                    .get_mut_character_object()
                    .update_display_position(&self.camera.borrow());
            }
        }

        // カメラをプレイヤーに合わせる
        self.focus_camera_playable_character_x();
    }

    ///
    /// PlayerCharacterの他キャラクターとのY方向の衝突を修正する
    ///
    fn check_character_collision_y(&mut self, ctx: &mut ggez::Context, t: Clock) {
        // マップ座標を更新
        self.player
            .get_mut_character_object()
            .update_display_position(&self.camera.borrow());

        // 他キャラクターすべてとの衝突判定を行う
        for e in self.character_group.iter_mut() {
            // 他キャラクターのマップ座標を更新
            e.get_mut_character_object()
                .update_display_position(&self.camera.borrow());

            // 衝突情報を取得
            let collision_info = self
                .player
                .get_character_object()
                .check_collision_with_character(ctx, e.get_character_object());

            // collisionフィールドがtrueなら、衝突している
            if collision_info.collision {
                // プレイヤーと他キャラクターの衝突状況から、プレイヤーがどれだけ、突き放されればいいのか計算
                let diff = self
                    .player
                    .get_mut_character_object()
                    .fix_collision_vertical(ctx, &collision_info, t);

                // プレイヤーの突き放し距離分動かす
                self.player
                    .get_mut_character_object()
                    .move_map(numeric::Vector2f::new(0.0, -diff));

                // プレイヤーのマップ座標を更新
                self.player
                    .get_mut_character_object()
                    .update_display_position(&self.camera.borrow());
            }
        }

        // カメラをプレイヤーに合わせる
        self.focus_camera_playable_character_y();
    }

    fn move_playable_character_x(&mut self, ctx: &mut ggez::Context, t: Clock) {
        // プレイヤーのX方向の移動
        self.player.move_map_current_speed_x(1500.0);

        // マップ座標を更新, これで、衝突判定を行えるようになる
        self.player
            .get_mut_character_object()
            .update_display_position(&self.camera.borrow());

        // マップチップとの衝突判定（横）
        self.playable_check_collision_horizon(ctx);

        // 他キャラクターとの当たり判定
        self.check_character_collision_x(ctx, t);
    }

    fn move_playable_character_y(&mut self, ctx: &mut ggez::Context, t: Clock) {
        // プレイヤーのY方向の移動
        self.player.move_map_current_speed_y(1500.0);
        // マップ座標を更新, これで、衝突判定を行えるようになる
        self.player
            .get_mut_character_object()
            .update_display_position(&self.camera.borrow());

        // マップチップとの衝突判定（縦）
        self.playable_check_collision_vertical(ctx);

        // 他キャラクターとの当たり判定
        self.check_character_collision_y(ctx, t);
    }

    fn move_playable_character(&mut self, ctx: &mut ggez::Context, t: Clock) {
        // キーのチェック
        self.check_key_event(ctx);

        self.player.get_mut_character_object().update_texture(t);

        self.move_playable_character_x(ctx, t);
        self.move_playable_character_y(ctx, t);
    }

    pub fn run_builtin_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        builtin_event: BuiltinEvent,
    ) {
        match builtin_event.get_event_symbol() {
            BuiltinEventSymbol::SelectShelvingBook => {
                self.dark_effect_panel
                    .new_effect(8, self.get_current_clock(), 0, 200);
                self.shop_special_object.show_shelving_select_ui(
                    ctx,
                    self.player.get_shelving_book().clone(),
                    self.get_current_clock(),
                );
            }
        }
    }

    fn scene_transition_close_effect<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        self.scene_transition_effect = Some(effect_object::ScreenTileEffect::new(
            ctx,
            TileBatchTextureID::Shoji,
            numeric::Rect::new(
                0.0,
                0.0,
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
            60,
            SceneTransitionEffectType::Close,
            TilingEffectType::WholeTile,
            -128,
            t,
        ));
    }

    fn check_event_panel_onmap<'a>(&mut self, ctx: &mut SuzuContext<'a>, trigger: EventTrigger) {
        let t = self.get_current_clock();
        let target_event = self.map.check_event_panel(
            trigger,
            self.player.get_center_map_position(ctx.context),
            self.get_current_clock(),
        );

        if let Some(event_element) = target_event {
            match event_element {
                MapEventElement::TextEvent(text) => {
                    println!("{}", text.get_text());

                    let mut scenario_box =
                        ScenarioBox::new(ctx, numeric::Rect::new(33.0, 470.0, 1300.0, 270.0), t);
                    scenario_box.text_box.set_fixed_text(
                        text.get_text(),
                        FontInformation::new(
                            ctx.resource.get_font(FontID::JpFude1),
                            numeric::Vector2f::new(32.0, 32.0),
                            ggraphics::Color::from_rgba_u32(0x000000ff),
                        ),
                    );
                    self.map.scenario_box = Some(scenario_box);
                }
                MapEventElement::SwitchScene(switch_scene) => {
                    if !self.customer_request_queue.is_empty() && !self.customer_queue.is_empty() {
                        let switch_scene_id = switch_scene.get_switch_scene_id();

                        self.event_list.add_event(
                            Box::new(move |slf: &mut ShopScene, ctx, _| {
                                slf.transition_status = SceneTransition::StackingTransition;
                                slf.transition_scene = switch_scene_id;

                                if slf.transition_scene == SceneID::MainDesk {
                                    slf.shop_clock.add_minute(10);
                                }

                                let mut customer = slf.customer_queue.pop_front().unwrap();

                                customer.set_destination_forced(
                                    ctx.context,
                                    &slf.map.tile_map,
                                    numeric::Vector2u::new(15, 10),
                                );
                                slf.character_group.add(customer);
                            }),
                            t + 31,
                        );

                        self.scene_transition_close_effect(ctx, t);
                    }
                }
                MapEventElement::BookStoreEvent(book_store_event) => {
                    debug::debug_screen_push_text(&format!(
                        "book store event: {:?}",
                        book_store_event.get_book_shelf_info()
                    ));
                    self.dark_effect_panel
                        .new_effect(8, self.get_current_clock(), 0, 200);
                    self.shop_special_object.show_storing_select_ui(
                        ctx,
                        book_store_event.get_book_shelf_info().clone(),
                        self.player.get_shelving_book().clone(),
                        t,
                    );
                }
                MapEventElement::BuiltinEvent(builtin_event) => {
                    let builtin_event = builtin_event.clone();
                    self.run_builtin_event(ctx, builtin_event);
                }
            }
        }
    }

    fn update_playable_character_texture(&mut self, rad: f32) {
        if rad >= 45.0_f32.to_radians() && rad < 135.0_f32.to_radians() {
            // 上向き
            self.player
                .get_mut_character_object()
                .change_animation_mode(0);
        }

        if rad >= 135.0_f32.to_radians() && rad < 225.0_f32.to_radians() {
            // 左向き
            self.player
                .get_mut_character_object()
                .change_animation_mode(3);
        }

        if rad >= 225.0_f32.to_radians() && rad < 315.0_f32.to_radians() {
            // 下向き
            self.player
                .get_mut_character_object()
                .change_animation_mode(1);
        }

        if (rad >= 315.0_f32.to_radians() && rad <= 360.0_f32.to_radians())
            || (rad >= 0.0_f32.to_radians() && rad < 45.0_f32.to_radians())
        {
            // 右向き
            self.player
                .get_mut_character_object()
                .change_animation_mode(2);
        }
    }

    pub fn start_mouse_move(&mut self, ctx: &mut ggez::Context, point: numeric::Point2f) {
        let current = self.player.get_character_object().obj().get_center(ctx);
        let offset = numeric::Point2f::new(point.x - current.x, point.y - current.y);
        let rad = if offset.x >= 0.0 {
            if offset.y >= 0.0 {
                (offset.y / offset.x).atan()
            } else {
                (offset.y / offset.x).atan() + 360.0_f32.to_radians()
            }
        } else {
            (offset.y / offset.x).atan() + 180.0_f32.to_radians()
        };
        let speed = numeric::Vector2f::new(rad.cos() * 4.0, rad.sin() * 4.0);

        self.player.set_speed(speed);
        self.update_playable_character_texture(rad);
    }

    pub fn switched_and_restart<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        let t = self.get_current_clock();
        let animation_time = 30;

        self.transition_scene = SceneID::SuzunaShop;

        self.scene_transition_effect = Some(effect_object::ScreenTileEffect::new(
            ctx,
            TileBatchTextureID::Shoji,
            numeric::Rect::new(
                0.0,
                0.0,
                crate::core::WINDOW_SIZE_X as f32,
                crate::core::WINDOW_SIZE_Y as f32,
            ),
            animation_time,
            SceneTransitionEffectType::Open,
            TilingEffectType::WholeTile,
            -128,
            t,
        ));

        // self.event_list.add_event(
        //     Box::new(move |slf: &mut ShopScene, _, _| { slf.scene_transition_effect = None; }),
        //     animation_time + 1
        // );
    }

    pub fn update_task_result<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        self.shop_menu
            .update_contents(ctx, self.player.get_shelving_book());
    }

    fn try_hide_shelving_select_ui<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        let select_result = self
            .shop_special_object
            .hide_shelving_select_ui(self.get_current_clock());
        if let Some((boxed, shelving)) = select_result {
            ctx.savable_data.task_result.not_shelved_books = boxed;
            self.player.update_shelving_book(shelving);
            self.shop_menu.update_contents(
                ctx,
                self.player.get_shelving_book(),
            );
            self.dark_effect_panel
                .new_effect(8, self.get_current_clock(), 200, 0);
        }
    }

    fn try_hide_storing_select_ui<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        let store_result = self
            .shop_special_object
            .hide_storing_select_ui(self.get_current_clock());
        if let Some((_stored, shelving)) = store_result {
            self.player.update_shelving_book(shelving);
            self.shop_menu.update_contents(
                ctx,
                self.player.get_shelving_book(),
            );
            self.dark_effect_panel
                .new_effect(8, self.get_current_clock(), 200, 0);
        }
    }

    pub fn pop_customer_request(&mut self) -> Option<CustomerRequest> {
        self.customer_request_queue.pop_front()
    }

    pub fn update_shop_clock_regular<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        if self.get_current_clock() % 40 == 0 {
            debug::debug_screen_push_text(&format!("{}", self.shop_clock));
            self.shop_clock.add_minute(5);

            if self.shop_clock.equals(12, 0) {
                self.notification_area.insert_new_contents_generic(
                    ctx,
                    NotificationContentsData::new(
                        "セラ知オ".to_string(),
                        "十二時ヲ過ギマシタ".to_string(),
                        NotificationType::Time,
                    ),
                    t,
                );
            }
        }
    }

    pub fn check_shop_clock_regular<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        if self.shop_clock.equals(18, 0) {
            self.event_list.add_event(
                Box::new(move |slf: &mut Self, _, _| {
                    slf.transition_status = SceneTransition::SwapTransition;
                    slf.transition_scene = SceneID::DayResult;
                }),
                t + 120,
            );

            self.scene_transition_close_effect(ctx, t);
        }
    }

    fn random_add_customer(&mut self, game_data: &GameResource) {
        if
        /* self.shop_clock.minute % 1 == 0 && */
        rand::random::<usize>() % 150 == 0 {
            self.character_group.add(CustomerCharacter::new(
                game_data,
                character_factory::create_character(
                    character_factory::CharacterFactoryOrder::CustomerSample,
                    game_data,
                    &self.camera.borrow(),
                    numeric::Point2f::new(1200.0, 870.0),
                ),
                CustomerDestPoint::new(vec![
                    numeric::Vector2u::new(10, 3),
                    numeric::Vector2u::new(6, 3),
                    numeric::Vector2u::new(4, 10),
                ]),
            ));
        }
    }

    fn notify_customer_calling<'a>(&mut self, ctx: &mut SuzuContext<'a>, t: Clock) {
        let notification = Box::new(notify::GeneralNotificationContents::new(
            ctx,
            NotificationContentsData::new(
                "セラ知オ".to_string(),
                "御客ガ呼ンデイマス".to_string(),
                NotificationType::CustomerCalling,
            ),
            0,
        ));
        self.notification_area
            .insert_new_contents(ctx, notification, t);
    }

    fn transition_to_copy_scene(&mut self) {
        self.transition_status = SceneTransition::StackingTransition;
        self.transition_scene = SceneID::Copying;
    }
}

impl SceneManager for ShopScene {
    fn key_down_event<'a>(&mut self, ctx: &mut SuzuContext<'a>, vkey: tdev::VirtualKey) {
        match vkey {
            tdev::VirtualKey::Action1 => {
                if let Some(scenario_box) = self.map.scenario_box.as_mut() {
                    if scenario_box.get_text_box_status() == TextBoxStatus::FixedText {
                        self.map.scenario_box = None;
                    }
                } else {
                    debug::debug_screen_push_text("OK");
                    self.check_event_panel_onmap(ctx, EventTrigger::Action);
                }
            }
            tdev::VirtualKey::Action2 => {
                self.shop_menu.toggle_first_menu(self.get_current_clock());
                if self.shop_menu.first_menu_is_open() {
                    self.dark_effect_panel
                        .new_effect(8, self.get_current_clock(), 0, 200);
                } else {
                    self.dark_effect_panel
                        .new_effect(8, self.get_current_clock(), 200, 0);
                }
            }
            tdev::VirtualKey::Action3 => {
                self.try_hide_shelving_select_ui(ctx);
                self.try_hide_storing_select_ui(ctx);
            }
            tdev::VirtualKey::Action4 => {
                self.transition_to_copy_scene();
            }
            tdev::VirtualKey::Action5 => {
                self.transition_status = SceneTransition::StackingTransition;
                self.transition_scene = SceneID::MainDesk;
            }
            _ => (),
        }

        self.shop_menu
            .menu_key_action(vkey, self.get_current_clock());
    }

    fn mouse_motion_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        _offset: numeric::Vector2f,
    ) {
        if ggez::input::mouse::button_pressed(ctx.context, MouseButton::Left) {
            self.start_mouse_move(ctx.context, point);
        }
    }

    fn mouse_button_down_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        point: numeric::Point2f,
    ) {
        match button {
            MouseButton::Left => {
                self.start_mouse_move(ctx.context, point);
            }
            MouseButton::Right => {
                self.player.reset_speed();
            }
            _ => (),
        }

        self.shop_special_object
            .mouse_down_action(ctx, button, point, self.get_current_clock());
    }

    fn mouse_button_up_event<'a>(
        &mut self,
        _ctx: &mut SuzuContext<'a>,
        button: MouseButton,
        _point: numeric::Point2f,
    ) {
        match button {
            MouseButton::Left => {
                self.player.reset_speed();
            }
            _ => (),
        }
    }

    fn mouse_wheel_event<'a>(
        &mut self,
        ctx: &mut SuzuContext<'a>,
        point: numeric::Point2f,
        x: f32,
        y: f32,
    ) {
        self.shop_special_object
            .mouse_wheel_scroll_action(ctx, point, x, y);
    }

    fn pre_process<'a>(&mut self, ctx: &mut SuzuContext<'a>) {
        let t = self.get_current_clock();

        flush_delay_event!(self, self.event_list, ctx, t);

        if !self.shop_menu.first_menu_is_open() && !self.shop_special_object.is_enable_now() {
            self.random_add_customer(ctx.resource);
            self.move_playable_character(ctx.context, t);
            self.check_event_panel_onmap(ctx, EventTrigger::Touch);

            self.character_group.move_and_collision_check(
                ctx.context,
                &self.camera.borrow(),
                &self.map.tile_map,
                t,
            );

            let mut rising_customers = self
                .character_group
                .drain_remove_if(|customer: &CustomerCharacter| customer.is_wait_on_clerk());

            // 新しく客が列に並んだら、通知をする
            if !rising_customers.is_empty() {
                self.notify_customer_calling(ctx, t);
            }

            for customer in &mut rising_customers {
                if let Some(request) =
                    customer.check_rise_hand(ctx)
                {
                    self.customer_request_queue.push_back(request);
                }
            }

            self.customer_queue.extend(rising_customers);

            for customer in self.character_group.iter_mut() {
                customer.try_update_move_effect(
                    ctx,
                    &self.map.tile_map,
                    numeric::Vector2u::new(4, 10),
                    numeric::Vector2u::new(15, 10),
                    t,
                );
                customer.get_mut_character_object().update_texture(t);
            }

            for customer in &mut self.customer_queue {
                Self::customer_move_and_collision_check(
                    ctx.context,
                    customer,
                    &self.camera.borrow(),
                    &self.map.tile_map,
                    t,
                );
                customer.try_update_move_effect(
                    ctx,
                    &self.map.tile_map,
                    numeric::Vector2u::new(4, 10),
                    numeric::Vector2u::new(15, 10),
                    t,
                );
                customer.get_mut_character_object().update_texture(t);
            }

            self.character_group.remove_if(|c| c.is_got_out());

            self.character_group.sort_by_y_position();

            // マップ描画の準備
            self.map.tile_map.update(ctx.context, t);
        }

        // 時刻の更新
        self.update_shop_clock_regular(ctx, t);
        self.check_shop_clock_regular(ctx, t);

        // 暗転の描画
        self.dark_effect_panel.run_effect(ctx.context, t);

        self.notification_area.update(ctx, t);

        if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.effect(ctx.context, t);
        }

        // select_uiなどの更新
        flush_delay_event!(self, self.event_list, ctx, t);
        self.shop_special_object.update(ctx, t);

        // メニューの更新
        self.shop_menu.update(ctx.context, t);
    }

    fn drawing_process(&mut self, ctx: &mut ggez::Context) {
        self.map.tile_map.draw(ctx).unwrap();

        let mut map_obj_drawer = MapObjectDrawer::new();

        map_obj_drawer.add(&mut self.player);

        self.character_group.draw(ctx).unwrap();

        for customer in self.character_group.iter_mut() {
            map_obj_drawer.add(customer);
        }

        for queued_customer in &mut self.customer_queue {
            map_obj_drawer.add(queued_customer);
        }

        map_obj_drawer.sort(ctx);
        map_obj_drawer.draw(ctx);

        if let Some(scenario_box) = self.map.scenario_box.as_mut() {
            scenario_box.draw(ctx).unwrap();
        }

        if let Some(transition_effect) = self.scene_transition_effect.as_mut() {
            transition_effect.draw(ctx).unwrap();
        }

        self.dark_effect_panel.draw(ctx).unwrap();

        self.shop_special_object.draw(ctx).unwrap();
        self.shop_menu.draw(ctx).unwrap();

        self.notification_area.draw(ctx).unwrap();
    }

    fn post_process<'a>(&mut self, _ctx: &mut SuzuContext<'a>) -> SceneTransition {
        self.update_current_clock();
        self.transition_status
    }

    fn transition(&self) -> SceneID {
        self.transition_scene
    }

    fn get_current_clock(&self) -> Clock {
        self.clock
    }

    fn update_current_clock(&mut self) {
        self.clock += 1;
    }
}

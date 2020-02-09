use std::collections::VecDeque;

use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::sub_screen;
use torifune::numeric;

use torifune::impl_texture_object_for_wrapped;
use torifune::impl_drawable_object_for_wrapped;

use std::str::FromStr;
use crate::core::{TextureID, FontID, GameData};
use super::*;

pub enum ScenarioID {
    Test1,
}

impl FromStr for ScenarioID {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "Test1" => Ok(Self::Test1),
            _ => Err(())
        }
    }
}

pub struct ScenarioTextAttribute {
    pub fpc: f32,
    pub font_info: FontInformation,
}

pub struct ScenarioTextSegment {
    text: String,
    attribute: ScenarioTextAttribute,
}

impl ScenarioTextSegment {
    pub fn new(text: &str, fpc: f32, font_info: FontInformation) -> Self {
        ScenarioTextSegment {
            text: text.to_string(),
            attribute: ScenarioTextAttribute { fpc: fpc, font_info: font_info },
        }
    }

    pub fn from_toml_using_default(obj: &toml::value::Table, game_data: &GameData, default: &ScenarioTextAttribute) -> Self {
        let text = obj.get("text");
        
        let fpc = if let Some(fpc) = obj.get("fpc") {
            fpc.as_float().unwrap() as f32
        } else {
            default.fpc
        };
        
        let font_scale = if let Some(font_scale) = obj.get("font_scale") {
            font_scale.as_float().unwrap() as f32
        } else {
            default.font_info.scale.x
        };
        
        let color = if let Some(color) = obj.get("color") {
            ggraphics::Color::from_rgba_u32(color.as_integer().unwrap() as u32)
        } else {
            default.font_info.color
        };
        
        ScenarioTextSegment {
            text: text.unwrap().as_str().unwrap().to_string(),
            attribute: ScenarioTextAttribute { fpc: fpc,
                                               font_info: FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                                                               numeric::Vector2f::new(font_scale, font_scale),
                                                                               color) },
        }
    }
    
    fn slice_text_bytes(&self, begin: usize, end: usize) -> &str {
        unsafe {
            self.text.as_str().get_unchecked(begin..end)
        }
    }

    fn count_indent(&self) -> usize {
        self.text.chars().fold(0, |sum, ch| sum + if ch == '\n' { 1 } else { 0 })
    }

    fn end_with_indent(&self) -> bool {
        self.text.chars().last().unwrap() == '\n'
    }

    fn last_line_length(&self) -> usize {
        let mut length: usize = 0;
        let it = self.text.chars().rev();

        for ch in it {
            if ch == '\n' {
                break;
            }
            length += 1
        }

        length
    }

    pub fn slice(&self, length: usize) -> &str {
        let mut reached_flag = false;

        let str_bytes = self.text.as_str().as_bytes().len();
        
        for (count, (bytes, _)) in self.text.as_str().char_indices().enumerate() {
            if reached_flag {
                return self.slice_text_bytes(0, bytes as usize);
            }

            if count == length {
                reached_flag = true;
            }
        }
        
        return self.slice_text_bytes(0, str_bytes as usize);
    }
    
    pub fn get_fpc(&self) -> f32 {
        self.attribute.fpc
    }

    pub fn str_len(&self) -> usize {
        self.text.chars().count()
    }

    pub fn get_font_info(&self) -> &FontInformation {
        &self.attribute.font_info
    }

    pub fn get_font_scale(&self) -> numeric::Vector2f {
        self.attribute.font_info.scale
    }

    pub fn get_attribute(&self) -> &ScenarioTextAttribute {
        &self.attribute
    }
}

pub struct ScenarioText {
    seq_text: Vec<ScenarioTextSegment>,
    iterator: f32,
    current_segment_index: usize,
    total_length: usize,
}

impl ScenarioText {
    pub fn new(toml_scripts: &toml::value::Array, game_data: &GameData, default: &ScenarioTextAttribute) -> Self {
        let mut seq_text = Vec::<ScenarioTextSegment>::new();

        for elem in toml_scripts {
            if let toml::Value::Table(scenario) = elem {
                seq_text.push(ScenarioTextSegment::from_toml_using_default(scenario, game_data, default));
            }
        }

        let total_length: usize = seq_text.iter().fold(0, |sum, s| sum + s.str_len());
        
        ScenarioText {
            seq_text: seq_text,
            iterator: 0.0,
            current_segment_index: 0,
            total_length: total_length,
        }
    }

    fn current_iterator(&self) -> usize {
        self.iterator as usize
    }

    pub fn update_iterator(&mut self) {
        let current_segment = self.seq_text.get(self.current_segment_index).unwrap();
        self.iterator += current_segment.get_fpc();

        if self.iterator as usize >= self.total_length {
            self.iterator = self.total_length as f32;
        }
    }

    pub fn reset_segment(&mut self) {
        self.current_segment_index = 0;
    }

    pub fn set_current_segment(&mut self, index: usize) {
        self.current_segment_index = index;
    }

    pub fn iterator_finish(&self) -> bool {
        self.iterator as usize == self.total_length
    }

    pub fn seq_text_iter(&self) -> std::slice::Iter<ScenarioTextSegment> {
        self.seq_text.iter()
    }
}

pub struct Scenario {
    tachie: Vec<SimpleObject>,
    scenario: Vec<ScenarioText>,
    page: usize,
}

impl Scenario {
    pub fn new(file_path: &str, game_data: &GameData) -> Self {
        let mut tachie = Vec::new();
        let mut scenario = Vec::new();

        let content = match std::fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => panic!("Failed to read: {}", file_path),
        };
        
        let root = content.parse::<toml::Value>().unwrap();

        let toml_default_attribute = root["default-text-attribute"].as_table().unwrap();
        let default_font_scale = toml_default_attribute["font_scale"].as_float().unwrap() as f32;
        let default_attribute = ScenarioTextAttribute {
            fpc: toml_default_attribute["fpc"].as_float().unwrap() as f32,
            font_info: FontInformation::new(game_data.get_font(FontID::DEFAULT),
                                            numeric::Vector2f::new(default_font_scale, default_font_scale),
                                            ggraphics::Color::from_rgba_u32(toml_default_attribute["color"].as_integer().unwrap() as u32)),
        };
        
        let array = root["scenario-group"].as_array().unwrap();
        scenario.push(ScenarioText::new(array, game_data, &default_attribute));
        // for elem in array {
        //     let table = elem.as_table().unwrap();
        //     scenario.push(ScenarioText::new(elem.as_array().unwrap(), game_data));
        // }

        let tachie_array = root["using-tachie"].as_array().unwrap();
        for elem in tachie_array {
            let tid = TextureID::from_str(elem.as_str().unwrap());
            if let Ok(tid) = tid {
            tachie.push(SimpleObject::new(
                MovableUniTexture::new(
                    game_data.ref_texture(tid),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(0.1, 0.1),
                    0.0, 0, move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                    0), vec![]));
            }
        }

        Scenario {
            tachie: tachie,
            scenario: scenario,
            page: 0,
        }
    }

    pub fn next_page(&mut self) {
        if !self.final_page() {
            self.page += 1;
        }
    }

    pub fn final_page(&self) -> bool {
        self.scenario.len() - 1 == self.page
    }

    pub fn update_current_page(&mut self) {
        self.current_page_mut().update_iterator();
    }
    
    pub fn current_page(&self) -> &ScenarioText {
        &self.scenario.get(self.page).unwrap()
    }

    pub fn current_page_mut(&mut self) -> &mut ScenarioText {
        self.scenario.get_mut(self.page).unwrap()
    }
}

pub struct TextBox {
    box_lines: usize,
    buffered_text: VecDeque<SimpleText>,
    text: VecDeque<SimpleText>,
    background: SimpleObject,
    canvas: SubScreen,
}

impl TextBox {
    pub fn new(ctx: &mut ggez::Context, rect: numeric::Rect,
               mut background: SimpleObject, box_lines: usize, _t: Clock) -> Self {
        background.fit_scale(ctx, numeric::Vector2f::new(rect.w, rect.h));
        background.set_position(numeric::Point2f::new(0.0, 0.0));
        TextBox {
	    box_lines: box_lines,
	    buffered_text: VecDeque::new(),
            text: VecDeque::new(),
            background: background,
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::Color::from_rgba_u32(0xff00ffff)),
        }
    }

    // ScenarioTextSegmentを改行で分割しVec<SimpleText>に変換する
    pub fn text_from_segment(segment: &ScenarioTextSegment, length: usize) -> Vec<SimpleText> {
	let mut text_lines = Vec::new();
	
	for line in segment.slice(length).lines() {
	    text_lines.push(SimpleText::new(tobj::MovableText::new(line.to_string(),
								   numeric::Point2f::new(0.0, 0.0),
								   numeric::Vector2f::new(1.0, 1.0),
								   0.0,
								   0,
								   None,
								   segment.attribute.font_info, 0),
					    Vec::new()));
	}

	text_lines
    }

    pub fn update_text(&mut self, scenario: &ScenarioText) -> usize {
	// 表示するテキストバッファをクリア。これで、新しくテキストを詰めていく
        self.text.clear();
	self.buffered_text.clear();

	// 表示する文字数を取得。この値を減算していき文字数チェックを行う
        let mut remain = scenario.current_iterator();

	// この関数の返り値は、テキスト化したセグメントの数なので、カウンタを初期化する
        let mut seg_count: usize = 0;

        for seg in scenario.seq_text_iter() {
	    // このテキストセグメントの文字数を取得
            let seg_len = seg.str_len();

	    // セグメントの切り出す文字数を計算
            let slice_len = if seg_len < remain {
                seg_len
            } else {
                remain
            };

	    // 行ごとにSimleTextに変換していく
	    let mut index = 0;
	    for mut line in Self::text_from_segment(seg, slice_len) {
		line.move_diff(numeric::Vector2f::new(0.0, seg.get_attribute().font_info.scale.y * index as f32));
		self.text.push_back(line);
		index += 1;
	    }

	    // テキストボックスの行数制限
	    if self.text.len() > self.box_lines {
		// オーバーしたら、buffered_textに追加
		self.buffered_text.push_back(self.text.pop_front().unwrap());
	    }

	    // 残りの文字数が0になったら、break
            remain -= slice_len;
            if remain as usize <= 0 {
                break;
            }

            seg_count += 1;
        }

	// ボックスに入ったSimpleTextの位置を設定
	let mut pos = numeric::Point2f::new(50.0, 50.0);
	for line in &mut self.text {
	    line.set_position(pos);
	    pos.y += line.ref_wrapped_object_mut().get_font_scale().y;
	}

	// 処理したセグメントの数を返す
        seg_count
    }

}

impl DrawableComponent for TextBox {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
	    sub_screen::stack_screen(ctx, &self.canvas);
            
            self.background.draw(ctx)?;

            for d in &mut self.text {
                d.draw(ctx)?;
            }

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

pub struct ScenarioEvent {
    scenario: Scenario,
    text_box: TextBox,
    canvas: SubScreen,
}

impl ScenarioEvent {
    pub fn new(ctx: &mut ggez::Context, rect: numeric::Rect, file_path: &str, game_data: &GameData, t: Clock) -> Self {
        let background = tobj::SimpleObject::new(
            tobj::MovableUniTexture::new(
                game_data.ref_texture(TextureID::TextBackground),
                numeric::Point2f::new(20.0, 20.0),
                numeric::Vector2f::new(0.8, 0.8),
                0.0,
                0,
                move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                0), Vec::new());
        ScenarioEvent {
            scenario: Scenario::new(file_path, game_data),
            text_box: TextBox::new(
                ctx,
                numeric::Rect::new(10.0, 10.0, rect.w - 20.0, rect.h - 20.0),
                background, 3, t),
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::Color::from_rgba_u32(0x00)),
        }
    }
    
    pub fn update_text(&mut self) {
        self.scenario.current_page_mut().update_iterator();
        let current_segment = self.text_box.update_text(&self.scenario.current_page());
        self.scenario.current_page_mut().set_current_segment(current_segment);
    }

    pub fn next_page(&mut self) {
        self.scenario.next_page();
    }
}

impl DrawableComponent for ScenarioEvent {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
	    sub_screen::stack_screen(ctx, &self.canvas);

            self.text_box.draw(ctx)?;
            
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

struct ChoicePanel {
    panel: UniTexture,
}

impl ChoicePanel {
    pub fn new(panel: UniTexture) -> Self {
	ChoicePanel {
	    panel: panel,
	}
    }
}

impl DrawableComponent for ChoicePanel {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	self.panel.draw(ctx)
    }

    fn hide(&mut self) {
	self.panel.hide();
    }

    fn appear(&mut self) {
	self.panel.appear();
    }

    fn is_visible(&self) -> bool {
	self.panel.is_visible()
    }

    fn set_drawing_depth(&mut self, depth: i8) {
	self.panel.set_drawing_depth(depth)
    }

    fn get_drawing_depth(&self) -> i8 {
	self.panel.get_drawing_depth()
    }
}

impl DrawableObject for ChoicePanel {
    impl_drawable_object_for_wrapped!{panel}
}

impl TextureObject for ChoicePanel {
    impl_texture_object_for_wrapped!{panel}
}

pub struct ChoiceBox {
    choice_text: Vec<String>,
    panels: Vec<ChoicePanel>,
    selecting: usize,
    canvas: SubScreen,
}

impl ChoiceBox {
    fn generate_choice_panel(ctx: &mut ggez::Context, game_data: &GameData,
			     size: usize, left_top: numeric::Vector2f, align: f32) -> Vec<ChoicePanel> {
	let mut choice_panels = Vec::new();
	let mut panel = TextureID::ChoicePanel1 as u32;
	let mut pos: numeric::Point2f = left_top.into();

	for _ in 0..size {
	    choice_panels.push(ChoicePanel::new(
		UniTexture::new(
		    game_data.ref_texture(TextureID::from_u32(panel).unwrap()),
		    pos,
		    numeric::Vector2f::new(1.0, 1.0),
		    0.0,
		    0)));
	    pos.x += choice_panels.last().unwrap().get_drawing_size(ctx).x;
	    pos.x += align;
	    panel += 1;
	}

	choice_panels
    }
    
    pub fn new(ctx: &mut ggez::Context, pos_rect: numeric::Rect,
	       game_data: &GameData, choice_text: Vec<String>) -> Self {
	ChoiceBox {
	    panels: Self::generate_choice_panel(ctx, game_data, choice_text.len(),
						numeric::Vector2f::new(10.0, 10.0), 10.0),
	    choice_text: choice_text,
	    selecting: 0,
	    canvas: SubScreen::new(ctx, pos_rect, 0, ggraphics::Color::from_rgba_u32(0)),
	}
    }

    pub fn get_selecting_index(&self) -> usize {
	self.selecting
    }

    pub fn get_selecting_str(&self) -> &str {
	self.choice_text.get(self.get_selecting_index()).as_ref().unwrap()
    }

    pub fn move_right(&mut self) {
	if self.choice_text.len() > (self.selecting + 1) {
	    self.selecting += 1;
	}
    }

    pub fn move_left(&mut self) {
	if self.selecting > 0 {
	    self.selecting -= 1;
	}
    }
}

impl DrawableComponent for ChoiceBox {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
	if self.is_visible() {
	    sub_screen::stack_screen(ctx, &self.canvas);

	    for panel in &mut self.panels {
		panel.draw(ctx)?;
	    }

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

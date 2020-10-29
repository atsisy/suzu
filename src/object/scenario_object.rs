use std::collections::HashMap;
use std::rc::Rc;

use ggez::graphics as ggraphics;

use torifune::graphics::drawable::*;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::*;
use torifune::numeric;
use torifune::roundup2f;

use torifune::graphics::object::sub_screen::SubScreen;

use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;

use crate::core::*;
use crate::object::util_object::*;
use crate::set_table_frame_cell_center;

use number_to_jk::number_to_jk;

use serde::{Deserialize, Serialize};

pub struct SuzunaStatusMainPage {
    table_frame: TableFrame,
    desc_text: Vec<VerticalText>,
    reputation_text: VerticalText,
    money_text: VerticalText,
    day_text: VerticalText,
    kosuzu_level_text: VerticalText,
    drwob_essential: DrawableObjectEssential,
}

impl SuzunaStatusMainPage {
    pub fn new<'a>(ctx: &mut SuzuContext<'a>) -> Self {
        let normal_scale_font = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(24.0, 24.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let large_scale_font = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(36.0, 36.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );

        let table_frame = TableFrame::new(
            ctx.resource,
            numeric::Point2f::new(150.0, 30.0),
            TileBatchTextureID::OldStyleFrame,
            FrameData::new(vec![120.0, 220.0], vec![40.0; 3]),
            numeric::Vector2f::new(0.25, 0.25),
            0,
        );

        let mut desc_text = Vec::new();

        for (index, s) in vec!["評判", "習熟度", "所持金"].iter().enumerate() {
            let mut vtext = VerticalText::new(
                s.to_string(),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                normal_scale_font,
            );

            set_table_frame_cell_center!(
                ctx.context,
                table_frame,
                vtext,
                numeric::Vector2u::new(index as u32, 0)
            );

            desc_text.push(vtext);
        }

        let mut reputation_text = VerticalText::new(
            number_to_jk(ctx.savable_data.suzunaan_status.reputation as u64),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            normal_scale_font,
        );

        set_table_frame_cell_center!(
            ctx.context,
            table_frame,
            reputation_text,
            numeric::Vector2u::new(0, 1)
        );

        let mut money_text = VerticalText::new(
            format!(
                "{}円",
                number_to_jk(ctx.savable_data.task_result.total_money as u64)
            ),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            normal_scale_font,
        );

        set_table_frame_cell_center!(
            ctx.context,
            table_frame,
            money_text,
            numeric::Vector2u::new(2, 1)
        );

        let mut kosuzu_level_text = VerticalText::new(
            format!("{}", number_to_jk(0)),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            normal_scale_font,
        );

        set_table_frame_cell_center!(
            ctx.context,
            table_frame,
            kosuzu_level_text,
            numeric::Vector2u::new(1, 1)
        );

        SuzunaStatusMainPage {
            table_frame: table_frame,
            reputation_text: reputation_text,
            desc_text: desc_text,
            day_text: VerticalText::new(
                format!(
                    "{}月{}日",
                    number_to_jk::number_to_jk(ctx.savable_data.date.month as u64),
                    number_to_jk::number_to_jk(ctx.savable_data.date.day as u64),
                ),
                numeric::Point2f::new(600.0, 50.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                large_scale_font,
            ),
            money_text: money_text,
            kosuzu_level_text: kosuzu_level_text,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }
}

impl DrawableComponent for SuzunaStatusMainPage {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.table_frame.draw(ctx).unwrap();
            self.day_text.draw(ctx).unwrap();

            for vtext in self.desc_text.iter_mut() {
                vtext.draw(ctx).unwrap();
            }

            self.reputation_text.draw(ctx).unwrap();
            self.money_text.draw(ctx).unwrap();

            self.kosuzu_level_text.draw(ctx).unwrap();
        }

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

#[derive(Copy, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub enum SuzunaAdType {
    ShopNobori,
    TownNobori,
    Chindon,
    NewsPaper,
    BunBunMaruPaper,
    AdPaper,
}

impl SuzunaAdType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "ShopNobori" => Self::ShopNobori,
            "TownNobori" => Self::TownNobori,
            "Chindon" => Self::Chindon,
            "NewsPaper" => Self::NewsPaper,
            "BunBunMaruPaper" => Self::BunBunMaruPaper,
            "AdPaper" => Self::AdPaper,
            _ => panic!("Unknown SuzunaAdType => {:?}", s),
        }
    }
}

pub struct AdEntry {
    check_box: CheckBox,
    desc_text: UniText,
    drwob_essential: DrawableObjectEssential,
}

impl AdEntry {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        pos: numeric::Point2f,
        check_box_size: numeric::Vector2f,
        default_check: bool,
        desc_text: String,
        depth: i8,
    ) -> Self {
        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(18.0, 18.0),
            ggraphics::Color::from_rgba_u32(0xff),
        );

        let choice_box_texture = Box::new(UniTexture::new(
            ctx.ref_texture(TextureID::ChoicePanel1),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            depth,
        ));

        AdEntry {
            check_box: CheckBox::new(
                ctx,
                numeric::Rect::new(pos.x, pos.y, check_box_size.x, check_box_size.y),
                choice_box_texture,
                default_check,
                0,
            ),
            desc_text: UniText::new(
                desc_text,
                numeric::Point2f::new(pos.x + check_box_size.x + 20.0, pos.y),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                font_info,
            ),
            drwob_essential: DrawableObjectEssential::new(true, depth),
        }
    }

    pub fn is_checked(&self) -> bool {
        self.check_box.checked_now()
    }
}

impl DrawableComponent for AdEntry {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.check_box.draw(ctx)?;
            self.desc_text.draw(ctx)?;
        }

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

pub struct ScenarioAdPage {
    header_text: UniText,
    ad_table: HashMap<SuzunaAdType, AdEntry>,
    drwob_essential: DrawableObjectEssential,
}

impl ScenarioAdPage {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        pos: numeric::Point2f,
        area_size: numeric::Vector2f,
        depth: i8,
    ) -> Self {
        let mut ad_table = HashMap::new();

        let mut entry_pos = numeric::Point2f::new(pos.x + 70.0, pos.y + 100.0);

        for (index, (ty_str, ad_type)) in vec![
            ("チラシ", SuzunaAdType::AdPaper),
            ("ちんどん屋", SuzunaAdType::Chindon),
            ("のぼり（店前）", SuzunaAdType::ShopNobori),
            ("のぼり（里）", SuzunaAdType::TownNobori),
            ("新聞", SuzunaAdType::NewsPaper),
            ("文々。新聞", SuzunaAdType::BunBunMaruPaper),
        ]
        .iter()
        .enumerate()
        {
            let entry = AdEntry::new(
                ctx,
                entry_pos,
                numeric::Vector2f::new(32.0, 32.0),
                ctx.savable_data.get_ad_status(*ad_type),
                format!(
                    "{:　<7}{:　>4}円/日",
                    ty_str,
                    ctx.resource.get_default_ad_cost(*ad_type)
                ),
                depth,
            );

            ad_table.insert(*ad_type, entry);

            if index % 2 == 0 {
                entry_pos.x = 400.0;
            } else {
                entry_pos.x = pos.x + 70.0;
                entry_pos.y += 64.0;
            }
        }

        let font_info = FontInformation::new(
            ctx.resource.get_font(FontID::Cinema),
            numeric::Vector2f::new(30.0, 30.0),
            ggraphics::BLACK,
        );
        let mut header_text = UniText::new(
            "鈴奈庵の宣伝広告".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            font_info,
        );

        header_text.make_center(ctx.context, numeric::Point2f::new(area_size.x / 2.0, 50.0));

        ScenarioAdPage {
            header_text: header_text,
            ad_table: ad_table,
            drwob_essential: DrawableObjectEssential::new(true, depth),
        }
    }

    pub fn click_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, click_point: numeric::Point2f) {
        for (ad_type, entry) in self.ad_table.iter_mut() {
            entry.check_box.click_handler(click_point);
            ctx.savable_data
                .change_ad_status(*ad_type, entry.is_checked());
        }
    }
}

impl DrawableComponent for ScenarioAdPage {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.header_text.draw(ctx)?;
            for (_, entry) in self.ad_table.iter_mut() {
                entry.draw(ctx)?;
            }
        }

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

pub struct SuzunaStatusPages {
    main_page: SuzunaStatusMainPage,
    ad_page: ScenarioAdPage,
    current_page: usize,
}

impl SuzunaStatusPages {
    pub fn new(main_page: SuzunaStatusMainPage, ad_page: ScenarioAdPage) -> Self {
        SuzunaStatusPages {
            main_page: main_page,
            ad_page: ad_page,
            current_page: 0,
        }
    }

    pub fn draw_page(&mut self, ctx: &mut ggez::Context) {
        match self.current_page {
            0 => self.main_page.draw(ctx).unwrap(),
            1 => self.ad_page.draw(ctx).unwrap(),
            _ => (),
        }
    }

    fn next_page(&mut self) {
        if self.current_page >= 1 {
            return;
        }

        self.current_page += 1;
    }

    fn prev_page(&mut self) {
        if self.current_page <= 0 {
            return;
        }

        self.current_page -= 1;
    }

    pub fn get_current_page_num(&self) -> usize {
        self.current_page
    }

    pub fn page_len(&self) -> usize {
        2
    }

    pub fn click_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, click_point: numeric::Point2f) {
        match self.current_page {
            0 => (),
            1 => self.ad_page.click_handler(ctx, click_point),
            _ => (),
        }
    }
}

pub struct SuzunaStatusScreen {
    canvas: SubScreen,
    background: UniTexture,
    pages: SuzunaStatusPages,
    go_left_texture: UniTexture,
    go_right_texture: UniTexture,
}

impl SuzunaStatusScreen {
    pub fn new<'a>(
        ctx: &mut SuzuContext<'a>,
        rect: numeric::Rect,
        depth: i8,
    ) -> SuzunaStatusScreen {
        let background_texture = UniTexture::new(
            ctx.ref_texture(TextureID::TextBackground),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
        );

        let mut left = UniTexture::new(
            ctx.ref_texture(TextureID::GoNextPageLeft),
            numeric::Point2f::new(0.0, rect.h - 32.0),
            numeric::Vector2f::new(0.5, 0.5),
            0.0,
            0,
        );
        left.hide();

        let right = UniTexture::new(
            ctx.ref_texture(TextureID::GoNextPageRight),
            numeric::Point2f::new(rect.w - 32.0, rect.h - 32.0),
            numeric::Vector2f::new(0.5, 0.5),
            0.0,
            0,
        );

        SuzunaStatusScreen {
            canvas: SubScreen::new(
                ctx.context,
                rect,
                depth,
                ggraphics::Color::from_rgba_u32(0xffffffff),
            ),
            background: background_texture,
            pages: SuzunaStatusPages::new(
                SuzunaStatusMainPage::new(ctx),
                ScenarioAdPage::new(
                    ctx,
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(rect.w, rect.h),
                    0,
                ),
            ),
            go_left_texture: left,
            go_right_texture: right,
        }
    }

    fn check_move_page_icon_visibility(&mut self) {
        self.go_right_texture.appear();
        self.go_left_texture.appear();

        if self.pages.get_current_page_num() == 0 {
            self.go_left_texture.hide();
        } else if self.pages.get_current_page_num() == self.pages.page_len() - 1 {
            self.go_right_texture.hide();
        }
    }

    pub fn click_handler<'a>(&mut self, ctx: &mut SuzuContext<'a>, click_point: numeric::Point2f) {
        if !self.canvas.contains(click_point) {
            return;
        }

        let rpoint = self.canvas.relative_point(click_point);

        if self.go_right_texture.contains(ctx.context, rpoint) {
            self.pages.next_page();
            self.check_move_page_icon_visibility();
            ctx.process_utility.redraw();
        } else if self.go_left_texture.contains(ctx.context, rpoint) {
            self.pages.prev_page();
            self.check_move_page_icon_visibility();
            ctx.process_utility.redraw();
        }

        self.pages.click_handler(ctx, rpoint);
    }
}

impl DrawableComponent for SuzunaStatusScreen {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.background.draw(ctx)?;
            self.pages.draw_page(ctx);

            self.go_right_texture.draw(ctx)?;
            self.go_left_texture.draw(ctx)?;

            sub_screen::pop_screen(ctx);
            self.canvas.draw(ctx).unwrap();
        }

        Ok(())
    }

    #[inline(always)]
    fn hide(&mut self) {
        self.canvas.hide()
    }

    #[inline(always)]
    fn appear(&mut self) {
        self.canvas.appear()
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
        self.canvas.is_visible()
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
        self.canvas.set_drawing_depth(depth)
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
        self.canvas.get_drawing_depth()
    }
}

impl DrawableObject for SuzunaStatusScreen {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for SuzunaStatusScreen {
    impl_texture_object_for_wrapped! {canvas}
}

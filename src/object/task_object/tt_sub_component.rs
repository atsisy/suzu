use std::cmp::Ordering;
use std::rc::Rc;

use ggez::graphics as ggraphics;
use ggez::input as ginput;
use ginput::mouse::MouseCursor;

use torifune::core::Clock;
use torifune::graphics::object::sub_screen;
use torifune::graphics::object::sub_screen::SubScreen;
use torifune::graphics::object::*;
use torifune::graphics::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::impl_texture_object_for_wrapped;
use torifune::numeric;
use torifune::roundup2f;

use crate::core::BookInformation;
use crate::object::move_fn;
use crate::object::util_object::*;

use super::Clickable;
use crate::core::{FontID, GameData, TextureID};

use number_to_jk::number_to_jk;

#[derive(Debug, Clone, Copy)]
pub struct GensoDate {
    pub season: u32,
    pub month: u8,
    pub day: u8,
}

impl GensoDate {
    pub fn new(season: u32, month: u8, day: u8) -> Self {
        GensoDate {
            season: season,
            month: month,
            day: day,
        }
    }

    pub fn new_empty() -> Self {
        GensoDate {
            season: 0,
            month: 0,
            day: 0,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "第{}季 {}月 {}日",
            number_to_jk(self.season as u64),
            number_to_jk(self.month as u64),
            number_to_jk(self.day as u64)
        )
    }
}

#[derive(Clone)]
pub struct BorrowingInformation {
    pub borrowing: Vec<BookInformation>,
    pub borrower: String,
    pub borrow_date: GensoDate,
    pub return_date: GensoDate,
}

impl BorrowingInformation {
    pub fn new(
        borrowing: Vec<BookInformation>,
        borrower: &str,
        borrow_date: GensoDate,
        return_date: GensoDate,
    ) -> Self {
        BorrowingInformation {
            borrowing: borrowing,
            borrower: borrower.to_string(),
            borrow_date,
            return_date,
        }
    }

    pub fn new_random(
        game_data: &GameData,
        borrow_date: GensoDate,
        return_date: GensoDate,
    ) -> Self {
        let borrowing_num = rand::random::<u32>() % 5;
        let mut borrow_books = Vec::new();

        for _ in 0..borrowing_num {
            borrow_books.push(game_data.book_random_select().clone());
        }

        Self::new(
            borrow_books,
            game_data.customer_random_select(),
            borrow_date,
            return_date,
        )
    }
}

#[derive(Clone)]
pub struct ReturnBookInformation {
    pub returning: Vec<BookInformation>,
    pub borrower: String,
    pub borrow_date: GensoDate,
    pub return_date: GensoDate,
}

impl ReturnBookInformation {
    pub fn new(
        returning: Vec<BookInformation>,
        borrower: &str,
        borrow_date: GensoDate,
        return_date: GensoDate,
    ) -> Self {
        ReturnBookInformation {
            returning: returning,
            borrower: borrower.to_string(),
            borrow_date,
            return_date,
        }
    }

    pub fn new_random(
        game_data: &GameData,
        borrow_date: GensoDate,
        return_date: GensoDate,
    ) -> Self {
        let borrowing_num = rand::random::<u32>() % 5;
        let mut borrow_books = Vec::new();

        for _ in 0..borrowing_num {
            borrow_books.push(game_data.book_random_select().clone());
        }

        Self::new(
            borrow_books,
            game_data.customer_random_select(),
            borrow_date,
            return_date,
        )
    }
}

///
/// TaskSceneでクリックしたときに取得できるデータ
///
#[derive(Clone)]
pub enum HoldData {
    BookName(String),
    CustomerName(String),
    Date(GensoDate),
    None,
}

impl ToString for HoldData {
    fn to_string(&self) -> String {
        match self {
            HoldData::BookName(name) => name.to_string(),
            HoldData::CustomerName(name) => name.to_string(),
            HoldData::Date(date) => date.to_string(),
            HoldData::None => "".to_string(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum OnDeskType {
    Book = 0,
    BorrowingRecordBook,
    CopyingPaper,
    Silhouette,
    Goods,
    Texture,
}

pub trait OnDesk: TextureObject + Clickable {
    fn ondesk_whose(&self) -> i32;

    fn click_data(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> HoldData;

    fn insert_data(&mut self, _: &mut ggez::Context, _: numeric::Point2f, _: &HoldData) -> bool {
        false
    }

    fn get_type(&self) -> OnDeskType;
}

pub struct DrawableCalendar {
    date_data: GensoDate,
    paper: UniTexture,
    season_text: VerticalText,
    month_text: VerticalText,
    day_text: VerticalText,
    canvas: SubScreen,
}

impl DrawableCalendar {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        rect: numeric::Rect,
        date: GensoDate,
        paper_tid: TextureID,
    ) -> Self {
        let font_info = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(14.0, 14.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );
        let default_scale = numeric::Vector2f::new(1.0, 1.0);

        DrawableCalendar {
            paper: UniTexture::new(
                game_data.ref_texture(paper_tid),
                numeric::Point2f::new(0.0, 0.0),
                default_scale,
                0.0,
                0,
            ),
            season_text: VerticalText::new(
                format!("{}季", number_to_jk(date.season as u64)),
                numeric::Point2f::new(50.0, 4.0),
                default_scale,
                0.0,
                0,
                font_info,
            ),
            month_text: VerticalText::new(
                format!("{}月", number_to_jk(date.month as u64)),
                numeric::Point2f::new(32.0, 4.0),
                default_scale,
                0.0,
                0,
                font_info,
            ),
            day_text: VerticalText::new(
                format!("{}日", number_to_jk(date.day as u64)),
                numeric::Point2f::new(16.0, 4.0),
                default_scale,
                0.0,
                0,
                font_info,
            ),
            date_data: date,
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::Color::from_rgba_u32(0x00000000)),
        }
    }
}

impl DrawableComponent for DrawableCalendar {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.paper.draw(ctx)?;
            self.season_text.draw(ctx)?;
            self.month_text.draw(ctx)?;
            self.day_text.draw(ctx)?;

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

impl DrawableObject for DrawableCalendar {
    #[inline(always)]
    fn set_position(&mut self, pos: numeric::Point2f) {
        self.canvas.set_position(pos);
    }

    #[inline(always)]
    fn get_position(&self) -> numeric::Point2f {
        self.canvas.get_position()
    }

    #[inline(always)]
    fn move_diff(&mut self, offset: numeric::Vector2f) {
        self.canvas.move_diff(offset);
    }
}

impl TextureObject for DrawableCalendar {
    impl_texture_object_for_wrapped! {canvas}
}

impl Clickable for DrawableCalendar {
    fn clickable_status(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
    ) -> MouseCursor {
        if self.canvas.get_drawing_area(ctx).contains(point) {
            MouseCursor::Grab
        } else {
            MouseCursor::Default
        }
    }
}

impl OnDesk for DrawableCalendar {
    fn ondesk_whose(&self) -> i32 {
        0
    }

    fn click_data(&self, _: &mut ggez::Context, _: numeric::Point2f) -> HoldData {
        HoldData::Date(self.date_data)
    }

    fn get_type(&self) -> OnDeskType {
        OnDeskType::Goods
    }
}

pub struct OnDeskTexture {
    texture: UniTexture,
    on_desk_type: OnDeskType,
}

impl OnDeskTexture {
    pub fn new(obj: UniTexture, on_desk_type: OnDeskType) -> Self {
        OnDeskTexture {
            texture: obj,
            on_desk_type: on_desk_type,
        }
    }
}

impl DrawableComponent for OnDeskTexture {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        self.texture.draw(ctx)
    }

    #[inline(always)]
    fn hide(&mut self) {
        self.texture.hide();
    }

    #[inline(always)]
    fn appear(&mut self) {
        self.texture.appear();
    }

    #[inline(always)]
    fn is_visible(&self) -> bool {
        self.texture.is_visible()
    }

    #[inline(always)]
    fn set_drawing_depth(&mut self, depth: i8) {
        self.texture.set_drawing_depth(depth);
    }

    #[inline(always)]
    fn get_drawing_depth(&self) -> i8 {
        self.texture.get_drawing_depth()
    }
}

impl DrawableObject for OnDeskTexture {
    #[inline(always)]
    fn set_position(&mut self, pos: numeric::Point2f) {
        self.texture.set_position(pos);
    }

    #[inline(always)]
    fn get_position(&self) -> numeric::Point2f {
        self.texture.get_position()
    }

    #[inline(always)]
    fn move_diff(&mut self, offset: numeric::Vector2f) {
        self.texture.move_diff(offset);
    }
}

impl TextureObject for OnDeskTexture {
    impl_texture_object_for_wrapped! {texture}
}

impl Clickable for OnDeskTexture {}

impl OnDesk for OnDeskTexture {
    fn ondesk_whose(&self) -> i32 {
        0
    }

    fn click_data(&self, _: &mut ggez::Context, _: numeric::Point2f) -> HoldData {
        HoldData::None
    }

    fn get_type(&self) -> OnDeskType {
        self.on_desk_type
    }
}

pub struct OnDeskBook {
    info: BookInformation,
    book_texture: UniTexture,
    title: VerticalText,
    canvas: SubScreen,
}

impl OnDeskBook {
    pub fn new(
        ctx: &mut ggez::Context,
        game_data: &GameData,
        texture_id: TextureID,
        info: BookInformation,
    ) -> Self {
        let texture = game_data.ref_texture(texture_id);
        let book_texture = UniTexture::new(
            texture,
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(0.2, 0.2),
            0.0,
            0,
        );
        let book_area = book_texture.get_drawing_area(ctx);
        let book_title = info.get_name().to_string();

        OnDeskBook {
            info: info,
            book_texture: book_texture,
            title: VerticalText::new(
                book_title,
                numeric::Point2f::new(40.0, 30.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                FontInformation::new(
                    game_data.get_font(FontID::JpFude1),
                    numeric::Vector2f::new(18.0, 18.0),
                    ggraphics::Color::from_rgba_u32(0x000000ff),
                ),
            ),
            canvas: SubScreen::new(
                ctx,
                book_area,
                0,
                ggraphics::Color::from_rgba_u32(0x00000000),
            ),
        }
    }

    pub fn get_book_info(&self) -> &BookInformation {
        &self.info
    }
}

impl DrawableComponent for OnDeskBook {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.book_texture.draw(ctx)?;
            self.title.draw(ctx)?;

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

impl DrawableObject for OnDeskBook {
    fn set_position(&mut self, pos: numeric::Point2f) {
        self.canvas.set_position(pos);
    }

    fn get_position(&self) -> numeric::Point2f {
        self.canvas.get_position()
    }

    fn move_diff(&mut self, offset: numeric::Vector2f) {
        self.canvas.move_diff(offset);
    }
}

impl TextureObject for OnDeskBook {
    impl_texture_object_for_wrapped! {canvas}
}

impl Clickable for OnDeskBook {}

impl OnDesk for OnDeskBook {
    fn ondesk_whose(&self) -> i32 {
        0
    }

    fn click_data(&self, _: &mut ggez::Context, _: numeric::Point2f) -> HoldData {
        return HoldData::BookName(self.info.get_name().to_string());
    }

    fn get_type(&self) -> OnDeskType {
        OnDeskType::Book
    }
}

pub struct BorrowingPaper {
    title: SimpleText,
    table_frame: TableFrame,
    borrowing: Vec<SimpleText>,
    book_head: SimpleText,
    borrower: SimpleText,
    borrow_date: SimpleText,
    return_date: SimpleText,
    paper_texture: SimpleObject,
    canvas: SubScreen,
}

impl BorrowingPaper {
    pub fn new(
        ctx: &mut ggez::Context,
        rect: ggraphics::Rect,
        paper_tid: TextureID,
        info: &BorrowingInformation,
        game_data: &GameData,
        t: Clock,
    ) -> Self {
        let mut pos = numeric::Point2f::new(210.0, 370.0);
        let table_frame = TableFrame::new(
            game_data,
            numeric::Point2f::new(200.0, 50.0),
            FrameData::new(vec![500.0, 64.0], vec![64.0; 2]),
            numeric::Vector2f::new(0.1, 0.1),
            0,
        );
        let borrowing = info
            .borrowing
            .iter()
            .map(|book_info| {
                pos += numeric::Vector2f::new(0.0, 30.0);
                SimpleText::new(
                    MovableText::new(
                        book_info.name.to_string(),
                        pos,
                        numeric::Vector2f::new(1.0, 1.0),
                        0.0,
                        0,
                        move_fn::halt(pos),
                        FontInformation::new(
                            game_data.get_font(FontID::DEFAULT),
                            numeric::Vector2f::new(24.0, 24.0),
                            ggraphics::Color::from_rgba_u32(0x000000ff),
                        ),
                        t,
                    ),
                    Vec::new(),
                )
            })
            .collect();

        let paper_texture = SimpleObject::new(
            MovableUniTexture::new(
                game_data.ref_texture(paper_tid),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                t,
            ),
            Vec::new(),
        );

        let book_head = SimpleText::new(
            MovableText::new(
                "貸出本".to_string(),
                numeric::Point2f::new(50.0, 400.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                move_fn::halt(numeric::Point2f::new(50.0, 350.0)),
                FontInformation::new(
                    game_data.get_font(FontID::DEFAULT),
                    numeric::Vector2f::new(28.0, 28.0),
                    ggraphics::Color::from_rgba_u32(0x000000ff),
                ),
                t,
            ),
            Vec::new(),
        );

        let title_text = SimpleText::new(
            MovableText::new(
                "鈴奈庵 貸出票".to_string(),
                numeric::Point2f::new(270.0, 100.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                move_fn::halt(numeric::Point2f::new(250.0, 100.0)),
                FontInformation::new(
                    game_data.get_font(FontID::DEFAULT),
                    numeric::Vector2f::new(28.0, 28.0),
                    ggraphics::Color::from_rgba_u32(0x000000ff),
                ),
                t,
            ),
            Vec::new(),
        );

        let borrower = SimpleText::new(
            MovableText::new(
                format!("借りた人   {}", info.borrower),
                numeric::Point2f::new(50.0, 200.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                move_fn::halt(numeric::Point2f::new(250.0, 100.0)),
                FontInformation::new(
                    game_data.get_font(FontID::DEFAULT),
                    numeric::Vector2f::new(28.0, 28.0),
                    ggraphics::Color::from_rgba_u32(0x000000ff),
                ),
                t,
            ),
            Vec::new(),
        );

        let borrow_date = SimpleText::new(
            MovableText::new(
                format!("貸出日     {}", info.borrow_date.to_string()),
                numeric::Point2f::new(50.0, 250.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                move_fn::halt(numeric::Point2f::new(250.0, 100.0)),
                FontInformation::new(
                    game_data.get_font(FontID::DEFAULT),
                    numeric::Vector2f::new(28.0, 28.0),
                    ggraphics::Color::from_rgba_u32(0x000000ff),
                ),
                t,
            ),
            Vec::new(),
        );

        let return_date = SimpleText::new(
            MovableText::new(
                format!("返却期限   {}", info.return_date.to_string()),
                numeric::Point2f::new(50.0, 300.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                move_fn::halt(numeric::Point2f::new(50.0, 300.0)),
                FontInformation::new(
                    game_data.get_font(FontID::DEFAULT),
                    numeric::Vector2f::new(28.0, 28.0),
                    ggraphics::Color::from_rgba_u32(0x000000ff),
                ),
                t,
            ),
            Vec::new(),
        );

        BorrowingPaper {
            title: title_text,
            table_frame: table_frame,
            borrower: borrower,
            book_head: book_head,
            paper_texture: paper_texture,
            borrowing: borrowing,
            borrow_date: borrow_date,
            return_date: return_date,
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::BLACK),
        }
    }

    pub fn new_random(
        ctx: &mut ggez::Context,
        rect: ggraphics::Rect,
        paper_tid: TextureID,
        borrow_date: GensoDate,
        return_date: GensoDate,
        game_data: &GameData,
        _t: Clock,
    ) -> Self {
        let mut borrowing = Vec::new();

        for _ in 0..(rand::random::<u32>() % 7) {
            borrowing.push(game_data.book_random_select().clone());
        }

        let borrow_info = &BorrowingInformation::new(
            borrowing,
            game_data.customer_random_select(),
            borrow_date,
            return_date,
        );

        Self::new(ctx, rect, paper_tid, &borrow_info, game_data, 0)
    }
}

impl DrawableComponent for BorrowingPaper {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.table_frame.draw(ctx)?;
            self.paper_texture.draw(ctx)?;
            self.title.draw(ctx)?;
            self.book_head.draw(ctx)?;
            self.borrower.draw(ctx)?;
            self.borrow_date.draw(ctx)?;
            self.return_date.draw(ctx)?;

            for d in &mut self.borrowing {
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

impl DrawableObject for BorrowingPaper {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for BorrowingPaper {
    impl_texture_object_for_wrapped! {canvas}
}

impl Clickable for BorrowingPaper {
    fn button_up(
        &mut self,
        ctx: &mut ggez::Context,
        _: &GameData,
        _: Clock,
        _button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let rp = self.canvas.relative_point(point);

        if self.title.get_drawing_area(ctx).contains(rp) {
            println!("aaaaaaaaa");
        }
    }
}

#[derive(Debug, Clone)]
pub struct CopyingRequestInformation {
    pub book_info: BookInformation,
    pub customer: String,
    pub request_date: GensoDate,
    pub return_date: GensoDate,
}

impl CopyingRequestInformation {
    pub fn new(
        book_info: BookInformation,
        customer: String,
        request_date: GensoDate,
        return_date: GensoDate,
    ) -> Self {
        CopyingRequestInformation {
            book_info: book_info,
            customer: customer,
            request_date: request_date,
            return_date: return_date,
        }
    }

    pub fn new_random(
        game_data: &GameData,
        request_date: GensoDate,
        return_date: GensoDate,
    ) -> Self {
        let book_info = game_data.book_random_select();
        CopyingRequestInformation {
            book_info: book_info.clone(),
            customer: game_data.customer_random_select().to_string(),
            request_date: request_date,
            return_date: return_date,
        }
    }
}

pub struct CopyingRequestPaper {
    title: VerticalText,
    request_book: VerticalText,
    customer: VerticalText,
    request_date: VerticalText,
    return_date: VerticalText,
    book_type: VerticalText,
    pages: VerticalText,
    canvas: SubScreen,
    paper_texture: SimpleObject,
    raw_info: CopyingRequestInformation,
}

impl CopyingRequestPaper {
    pub fn new(
        ctx: &mut ggez::Context,
        rect: ggraphics::Rect,
        paper_tid: TextureID,
        info: CopyingRequestInformation,
        game_data: &GameData,
        t: Clock,
    ) -> Self {
        let default_scale = numeric::Vector2f::new(1.0, 1.0);
        let mut font_info = FontInformation::new(
            game_data.get_font(FontID::JpFude1),
            numeric::Vector2f::new(16.0, 16.0),
            ggraphics::Color::from_rgba_u32(0x000000ff),
        );
        let mut pos = numeric::Point2f::new(rect.w - 50.0, 0.0);

        let paper_texture = SimpleObject::new(
            MovableUniTexture::new(
                game_data.ref_texture(paper_tid),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                t,
            ),
            Vec::new(),
        );

        let title_text = VerticalText::new(
            "鈴奈庵 転写依頼票".to_string(),
            numeric::Point2f::new(pos.x, 50.0),
            default_scale,
            0.0,
            0,
            font_info,
        );
        pos.x -= 60.0;

        font_info.scale = numeric::Vector2f::new(20.0, 20.0);

        let request_book = VerticalText::new(
            format!("転写本    {}", info.book_info.name),
            numeric::Point2f::new(pos.x, 50.0),
            default_scale,
            0.0,
            0,
            font_info,
        );
        pos.x -= 24.0;

        let pages = VerticalText::new(
            format!("頁数   {}", number_to_jk(info.book_info.pages as u64)),
            numeric::Point2f::new(pos.x, 50.0),
            default_scale,
            0.0,
            0,
            font_info,
        );
        pos.x -= 24.0;

        let book_type = VerticalText::new(
            format!("寸法   {}", info.book_info.size),
            numeric::Point2f::new(pos.x, 50.0),
            default_scale,
            0.0,
            0,
            font_info,
        );
        pos.x -= 180.0;

        font_info.scale = numeric::Vector2f::new(16.0, 16.0);

        let customer = VerticalText::new(
            format!("依頼者 {}", info.customer),
            numeric::Point2f::new(pos.x, 50.0),
            default_scale,
            0.0,
            0,
            font_info,
        );
        pos.x -= 24.0;

        let request_date = VerticalText::new(
            format!("依頼日 {}", info.request_date.to_string()),
            numeric::Point2f::new(pos.x, 50.0),
            default_scale,
            0.0,
            0,
            font_info,
        );
        pos.x -= 24.0;

        let return_date = VerticalText::new(
            format!("完了予定 {}", info.return_date.to_string()),
            numeric::Point2f::new(pos.x, 50.0),
            default_scale,
            0.0,
            0,
            font_info,
        );
        pos.x -= 24.0;

        let mut canvas = SubScreen::new(ctx, rect, 0, ggraphics::BLACK);
        canvas.set_filter(ggraphics::FilterMode::Nearest);

        CopyingRequestPaper {
            title: title_text,
            request_book: request_book,
            customer: customer,
            paper_texture: paper_texture,
            request_date: request_date,
            return_date: return_date,
            pages: pages,
            canvas: canvas,
            book_type: book_type,
            raw_info: info,
        }
    }
}

impl DrawableComponent for CopyingRequestPaper {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.paper_texture.draw(ctx)?;
            self.title.draw(ctx)?;
            self.customer.draw(ctx)?;
            self.request_date.draw(ctx)?;
            self.return_date.draw(ctx)?;
            self.pages.draw(ctx)?;
            self.book_type.draw(ctx)?;
            self.request_book.draw(ctx)?;

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

impl DrawableObject for CopyingRequestPaper {
    impl_drawable_object_for_wrapped! {canvas}
}

impl TextureObject for CopyingRequestPaper {
    impl_texture_object_for_wrapped! {canvas}
}

impl Clickable for CopyingRequestPaper {}

impl OnDesk for CopyingRequestPaper {
    fn ondesk_whose(&self) -> i32 {
        0
    }

    fn click_data(&self, ctx: &mut ggez::Context, point: numeric::Point2f) -> HoldData {
        let rpoint = self.canvas.relative_point(point);

        if self.request_book.get_drawing_area(ctx).contains(rpoint) {
            return HoldData::BookName(self.raw_info.book_info.get_name().to_string());
        }

        if self.customer.get_drawing_area(ctx).contains(rpoint) {
            return HoldData::CustomerName(self.raw_info.customer.to_string());
        }

        HoldData::None
    }

    fn get_type(&self) -> OnDeskType {
        OnDeskType::CopyingPaper
    }
}

pub struct BorrowingRecordBookPage {
    raw_info: BorrowingInformation,
    info_table: TableFrame,
    books_table: TableFrame,
    borrow_book: Vec<VerticalText>,
    book_head: VerticalText,
    book_status: VerticalText,
    borrower: VerticalText,
    borrow_date: VerticalText,
    return_date: VerticalText,
    paper_texture: SimpleObject,
    canvas: SubScreen,
}

impl BorrowingRecordBookPage {
    pub fn new(
        ctx: &mut ggez::Context,
        rect: ggraphics::Rect,
        paper_tid: TextureID,
        info: &BorrowingInformation,
        game_data: &GameData,
        t: Clock,
    ) -> Self {
        let mut page = Self::new_empty(ctx, rect, paper_tid, game_data, t);

        page.borrow_book = info
            .borrowing
            .iter()
            .enumerate()
            .map(|iter_data| {
                let (i, book_info) = iter_data;

                let mut vtext = VerticalText::new(
                    book_info.name.to_string(),
                    numeric::Point2f::new(0.0, 0.0),
                    numeric::Vector2f::new(1.0, 1.0),
                    0.0,
                    0,
                    FontInformation::new(
                        game_data.get_font(FontID::JpFude1),
                        numeric::Vector2f::new(18.0, 18.0),
                        ggraphics::Color::from_rgba_u32(0x000000ff),
                    ),
                );
                vtext.make_center(
                    ctx,
                    roundup2f!(page.books_table.get_center_of(
                        numeric::Vector2u::new(5 - i as u32, 0),
                        page.books_table.get_position()
                    )),
                );

                vtext
            })
            .collect();

        page
    }

    pub fn new_empty(
        ctx: &mut ggez::Context,
        rect: ggraphics::Rect,
        paper_tid: TextureID,
        game_data: &GameData,
        t: Clock,
    ) -> Self {
        let table_frame = TableFrame::new(
            game_data,
            numeric::Point2f::new(rect.w - 200.0, 40.0),
            FrameData::new(vec![150.0, 300.0], vec![40.0; 3]),
            numeric::Vector2f::new(0.2, 0.2),
            0,
        );

        let mut borrower = VerticalText::new(
            "借りた人".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );
        borrower.make_center(
            ctx,
            roundup2f!(
                table_frame.get_center_of(numeric::Vector2u::new(2, 0), table_frame.get_position())
            ),
        );

        let mut borrow_date = VerticalText::new(
            "貸出日".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );
        borrow_date.make_center(
            ctx,
            roundup2f!(
                table_frame.get_center_of(numeric::Vector2u::new(1, 0), table_frame.get_position())
            ),
        );

        let mut return_date = VerticalText::new(
            "返却期限".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );
        return_date.make_center(
            ctx,
            roundup2f!(
                table_frame.get_center_of(numeric::Vector2u::new(0, 0), table_frame.get_position())
            ),
        );

        let books_table = TableFrame::new(
            game_data,
            numeric::Point2f::new(rect.w - 550.0, 30.0),
            FrameData::new(vec![380.0, 70.0], vec![40.0; 6]),
            numeric::Vector2f::new(0.2, 0.2),
            0,
        );

        let mut book_head = VerticalText::new(
            "貸出本名称".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );
        book_head.make_center(
            ctx,
            roundup2f!(
                books_table.get_center_of(numeric::Vector2u::new(5, 0), books_table.get_position())
            ),
        );

        let mut book_status = VerticalText::new(
            "状態".to_string(),
            numeric::Point2f::new(0.0, 0.0),
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(24.0, 24.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );
        book_status.make_center(
            ctx,
            roundup2f!(
                books_table.get_center_of(numeric::Vector2u::new(5, 1), books_table.get_position())
            ),
        );

        let paper_texture = SimpleObject::new(
            MovableUniTexture::new(
                game_data.ref_texture(paper_tid),
                numeric::Point2f::new(0.0, 0.0),
                numeric::Vector2f::new(1.0, 1.0),
                0.0,
                0,
                move_fn::halt(numeric::Point2f::new(0.0, 0.0)),
                t,
            ),
            Vec::new(),
        );

        BorrowingRecordBookPage {
            raw_info: BorrowingInformation::new(
                Vec::new(),
                "",
                GensoDate::new_empty(),
                GensoDate::new_empty(),
            ),
            info_table: table_frame,
            books_table: books_table,
            borrow_book: Vec::new(),
            borrower: borrower,
            book_head: book_head,
            book_status: book_status,
            paper_texture: paper_texture,
            borrow_date: borrow_date,
            return_date: return_date,
            canvas: SubScreen::new(ctx, rect, 0, ggraphics::BLACK),
        }
    }

    pub fn get_borrowing_info(&self) -> &BorrowingInformation {
        &self.raw_info
    }

    pub fn get_borrowing_info_mut(&mut self) -> &mut BorrowingInformation {
        &mut self.raw_info
    }

    pub fn relative_point(&self, point: numeric::Point2f) -> numeric::Point2f {
        self.canvas.relative_point(point)
    }

    pub fn replace_borrower_name(&mut self, game_data: &GameData, name: &str) -> &mut Self {
        let pos = self.borrower.get_position();
        self.borrower = VerticalText::new(
            format!("借りた人   {}", name),
            pos,
            numeric::Vector2f::new(1.0, 1.0),
            0.0,
            0,
            FontInformation::new(
                game_data.get_font(FontID::JpFude1),
                numeric::Vector2f::new(20.0, 20.0),
                ggraphics::Color::from_rgba_u32(0x000000ff),
            ),
        );
        self
    }
}

impl DrawableComponent for BorrowingRecordBookPage {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            sub_screen::stack_screen(ctx, &self.canvas);

            self.paper_texture.draw(ctx)?;
            self.info_table.draw(ctx)?;
            self.books_table.draw(ctx)?;
            self.book_head.draw(ctx)?;
            self.book_status.draw(ctx)?;
            self.borrower.draw(ctx)?;
            self.borrow_date.draw(ctx)?;
            self.return_date.draw(ctx)?;

            for d in &mut self.borrow_book {
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

impl DrawableObject for BorrowingRecordBookPage {
    #[inline(always)]
    fn set_position(&mut self, pos: numeric::Point2f) {
        self.canvas.set_position(pos);
    }

    #[inline(always)]
    fn get_position(&self) -> numeric::Point2f {
        self.canvas.get_position()
    }

    #[inline(always)]
    fn move_diff(&mut self, offset: numeric::Vector2f) {
        self.canvas.move_diff(offset)
    }
}

impl Clickable for BorrowingRecordBookPage {
    fn button_down(
        &mut self,
        _ctx: &mut ggez::Context,
        _: &GameData,
        _: Clock,
        _button: ggez::input::mouse::MouseButton,
        _point: numeric::Point2f,
    ) {
    }

    fn button_up(
        &mut self,
        _ctx: &mut ggez::Context,
        _: &GameData,
        _: Clock,
        _button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        let rpoint = self.canvas.relative_point(point);
        println!("{:?}", rpoint);
    }
}

impl TextureObject for BorrowingRecordBookPage {
    impl_texture_object_for_wrapped! {canvas}
}

pub struct BorrowingRecordBook {
    pages: Vec<BorrowingRecordBookPage>,
    rect: numeric::Rect,
    current_page: usize,
    drwob_essential: DrawableObjectEssential,
}

impl BorrowingRecordBook {
    pub fn new(rect: ggraphics::Rect) -> Self {
        BorrowingRecordBook {
            pages: Vec::new(),
            rect: rect,
            current_page: 0,
            drwob_essential: DrawableObjectEssential::new(true, 0),
        }
    }

    pub fn add_page(
        &mut self,
        ctx: &mut ggez::Context,
        info: &BorrowingInformation,
        game_data: &GameData,
        t: Clock,
    ) -> &Self {
        let page_rect = if let Some(page) = self.get_current_page() {
            page.get_drawing_area(ctx)
        } else {
            self.rect
        };
        self.pages.push(BorrowingRecordBookPage::new(
            ctx,
            page_rect,
            TextureID::Paper1,
            info,
            game_data,
            t,
        ));
        self
    }

    pub fn add_empty_page(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        t: Clock,
    ) -> &Self {
        let page_rect = if let Some(page) = self.get_current_page() {
            page.get_drawing_area(ctx)
        } else {
            self.rect
        };
        self.pages.push(BorrowingRecordBookPage::new_empty(
            ctx,
            page_rect,
            TextureID::Paper1,
            game_data,
            t,
        ));
        self
    }

    fn get_current_page(&self) -> Option<&BorrowingRecordBookPage> {
        self.pages.get(self.current_page)
    }

    fn get_current_page_mut(&mut self) -> Option<&mut BorrowingRecordBookPage> {
        self.pages.get_mut(self.current_page)
    }

    fn next_page(&mut self) {
        if self.current_page < self.pages.len() {
            self.current_page += 1;
        }
    }

    fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
        }
    }

    fn borrow_date_insert_check(
        ctx: &mut ggez::Context,
        rpoint: numeric::Point2f,
        page: &mut BorrowingRecordBookPage,
        data: &HoldData,
    ) -> bool {
        if page.borrow_date.get_drawing_area(ctx).contains(rpoint) {
            match data {
                HoldData::Date(date_data) => {
                    page.borrow_date
                        .replace_text(format!("貸出日 {}", date_data.to_string()));
                    return true;
                }
                _ => (),
            }
        }

        return false;
    }

    fn borrower_customer_insert_check(
        ctx: &mut ggez::Context,
        rpoint: numeric::Point2f,
        page: &mut BorrowingRecordBookPage,
        data: &HoldData,
    ) -> bool {
        if page.borrower.get_drawing_area(ctx).contains(rpoint) {
            match data {
                HoldData::CustomerName(customer_name) => {
                    page.borrower
                        .replace_text(format!("借りる人 {}", customer_name.to_string()));
                    return true;
                }
                _ => (),
            }
        }

        return false;
    }
}

impl DrawableComponent for BorrowingRecordBook {
    #[inline(always)]
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            if self.pages.len() > 0 {
                self.pages.get_mut(self.current_page).unwrap().draw(ctx)?;
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

impl DrawableObject for BorrowingRecordBook {
    #[inline(always)]
    fn set_position(&mut self, pos: numeric::Point2f) {
        self.rect.x = pos.x;
        self.rect.y = pos.y;

        for page in &mut self.pages {
            page.set_position(pos);
        }
    }

    #[inline(always)]
    fn get_position(&self) -> numeric::Point2f {
        self.rect.point().into()
    }

    #[inline(always)]
    fn move_diff(&mut self, offset: numeric::Vector2f) {
        self.rect.x += offset.x;
        self.rect.y += offset.y;

        for page in &mut self.pages {
            page.move_diff(offset);
        }
    }
}

impl TextureObject for BorrowingRecordBook {
    #[inline(always)]
    fn set_scale(&mut self, scale: numeric::Vector2f) {
        self.get_current_page_mut().unwrap().set_scale(scale);
    }

    #[inline(always)]
    fn get_scale(&self) -> numeric::Vector2f {
        self.get_current_page().unwrap().get_scale()
    }

    #[inline(always)]
    fn set_rotation(&mut self, rad: f32) {
        self.get_current_page_mut().unwrap().set_rotation(rad);
    }

    #[inline(always)]
    fn get_rotation(&self) -> f32 {
        self.get_current_page().unwrap().get_rotation()
    }

    #[inline(always)]
    fn set_crop(&mut self, crop: ggraphics::Rect) {
        self.get_current_page_mut().unwrap().set_crop(crop)
    }

    #[inline(always)]
    fn get_crop(&self) -> ggraphics::Rect {
        self.get_current_page().unwrap().get_crop()
    }

    #[inline(always)]
    fn set_drawing_color(&mut self, color: ggraphics::Color) {
        self.get_current_page_mut()
            .unwrap()
            .set_drawing_color(color)
    }

    #[inline(always)]
    fn get_drawing_color(&self) -> ggraphics::Color {
        self.get_current_page().unwrap().get_drawing_color()
    }

    #[inline(always)]
    fn set_alpha(&mut self, alpha: f32) {
        self.get_current_page_mut().unwrap().set_alpha(alpha)
    }

    #[inline(always)]
    fn get_alpha(&self) -> f32 {
        self.get_current_page().unwrap().get_alpha()
    }

    #[inline(always)]
    fn set_transform_offset(&mut self, offset: numeric::Point2f) {
        self.get_current_page_mut()
            .unwrap()
            .set_transform_offset(offset)
    }

    #[inline(always)]
    fn get_transform_offset(&self) -> numeric::Point2f {
        self.get_current_page().unwrap().get_transform_offset()
    }

    #[inline(always)]
    fn get_texture_size(&self, ctx: &mut ggez::Context) -> numeric::Vector2f {
        self.get_current_page().unwrap().get_texture_size(ctx)
    }

    #[inline(always)]
    fn replace_texture(&mut self, _: Rc<ggraphics::Image>) {}

    #[inline(always)]
    fn set_color(&mut self, color: ggraphics::Color) {
        self.get_current_page_mut().unwrap().set_color(color)
    }

    #[inline(always)]
    fn get_color(&mut self) -> ggraphics::Color {
        self.get_current_page_mut().unwrap().get_color()
    }
}

impl Clickable for BorrowingRecordBook {
    fn button_down(
        &mut self,
        _ctx: &mut ggez::Context,
        _: &GameData,
        _: Clock,
        _button: ggez::input::mouse::MouseButton,
        _point: numeric::Point2f,
    ) {
    }

    fn button_up(
        &mut self,
        ctx: &mut ggez::Context,
        game_data: &GameData,
        t: Clock,
        button: ggez::input::mouse::MouseButton,
        point: numeric::Point2f,
    ) {
        if let Some(page) = self.get_current_page_mut() {
            let rpoint = page.relative_point(point);

            if rpoint.x < 20.0 {
                println!("next page!!");
                self.add_empty_page(ctx, game_data, t);
                self.next_page();
            } else if rpoint.x > page.get_drawing_size(ctx).x - 20.0 {
                println!("prev page!!");
                self.prev_page();
            } else {
                page.button_up(ctx, game_data, t, button, point);
            }
        }
    }
}

impl OnDesk for BorrowingRecordBook {
    fn ondesk_whose(&self) -> i32 {
        0
    }

    fn click_data(&self, _: &mut ggez::Context, _: numeric::Point2f) -> HoldData {
        HoldData::None
    }

    fn insert_data(
        &mut self,
        ctx: &mut ggez::Context,
        point: numeric::Point2f,
        data: &HoldData,
    ) -> bool {
        let mut insert_done_flag = false;

        if let Some(page) = self.get_current_page_mut() {
            let rpoint = page.relative_point(point);
            let mut hit_book_index = None;

            for (index, book) in page.borrow_book.iter().enumerate() {
                if book.get_drawing_area(ctx).contains(rpoint) {
                    hit_book_index = Some(index);
                }
            }

            if let Some(hit_book_index) = hit_book_index {
                match data {
                    HoldData::BookName(name) => {
                        page.get_borrowing_info_mut()
                            .borrowing
                            .push(BookInformation {
                                name: name.to_string(),
                                pages: 0,
                                size: "".to_string(),
                                billing_number: 0,
                            });
                        page.borrow_book
                            .get_mut(hit_book_index)
                            .unwrap()
                            .replace_text(name.to_string());
                        insert_done_flag = true;
                    }
                    _ => (),
                }
            }

            if Self::borrow_date_insert_check(ctx, rpoint, page, data)
                || Self::borrower_customer_insert_check(ctx, rpoint, page, data)
            {
                return true;
            }
        }

        insert_done_flag
    }

    fn get_type(&self) -> OnDeskType {
        OnDeskType::BorrowingRecordBook
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeskObjectType {
    CustomerObject = 0,
    BorrowRecordBook,
    SuzunaObject,
}

pub struct DeskObject {
    small: Box<EffectableWrap<MovableWrap<dyn OnDesk>>>,
    large: Box<EffectableWrap<MovableWrap<dyn OnDesk>>>,
    switch: u8,
    object_type: DeskObjectType,
}

impl DeskObject {
    pub fn new(
        small: Box<dyn OnDesk>,
        large: Box<dyn OnDesk>,
        switch: u8,
        obj_type: DeskObjectType,
        t: Clock,
    ) -> Self {
        DeskObject {
            small: Box::new(EffectableWrap::new(
                MovableWrap::new(small, None, t),
                Vec::new(),
            )),
            large: Box::new(EffectableWrap::new(
                MovableWrap::new(large, None, t),
                Vec::new(),
            )),
            switch: switch,
            object_type: obj_type,
        }
    }

    pub fn enable_small(&mut self) {
        self.switch = 0;
    }

    pub fn enable_large(&mut self) {
        self.switch = 1;
    }

    pub fn get_object(&self) -> &Box<EffectableWrap<MovableWrap<dyn OnDesk>>> {
        match self.switch {
            0 => &self.small,
            1 => &self.large,
            _ => panic!("Failed to object selecting. select = {}", self.switch),
        }
    }

    pub fn get_object_mut(&mut self) -> &mut Box<EffectableWrap<MovableWrap<dyn OnDesk>>> {
        match self.switch {
            0 => &mut self.small,
            1 => &mut self.large,
            _ => panic!("Failed to object selecting. select = {}", self.switch),
        }
    }

    pub fn get_object_type(&self) -> DeskObjectType {
        self.object_type
    }
}

pub struct DeskObjectContainer {
    container: Vec<DeskObject>,
}

impl DeskObjectContainer {
    pub fn new() -> Self {
        DeskObjectContainer {
            container: Vec::new(),
        }
    }

    pub fn add(&mut self, obj: DeskObject) {
        self.container.push(obj);
    }

    pub fn sort_with_depth(&mut self) {
        self.container.sort_by(|a: &DeskObject, b: &DeskObject| {
            let (ad, bd) = (
                a.get_object().get_drawing_depth(),
                b.get_object().get_drawing_depth(),
            );
            if ad > bd {
                Ordering::Less
            } else if ad < bd {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
    }

    pub fn get_raw_container(&self) -> &Vec<DeskObject> {
        &self.container
    }

    pub fn get_raw_container_mut(&mut self) -> &mut Vec<DeskObject> {
        &mut self.container
    }

    pub fn get_minimum_depth(&mut self) -> i8 {
        self.sort_with_depth();
        if let Some(depth) = self.container.last() {
            depth.get_object().get_drawing_depth()
        } else {
            127
        }
    }

    pub fn len(&self) -> usize {
        self.container.len()
    }

    pub fn change_depth_equally(&mut self, offset: i8) {
        for obj in &mut self.container {
            let current_depth = obj.get_object().get_drawing_depth();
            let next_depth: i16 = (current_depth as i16) + (offset as i16);

            if next_depth <= 127 && next_depth >= -128 {
                // 範囲内
                obj.get_object_mut().set_drawing_depth(next_depth as i8);
            } else if next_depth > 0 {
                // 範囲外（上限）
                obj.get_object_mut().set_drawing_depth(127);
            } else {
                // 範囲外（下限）
                obj.get_object_mut().set_drawing_depth(-128);
            }
        }
    }
}

pub struct ObjectContainer<T> {
    container: Vec<T>,
}

impl<T> ObjectContainer<T> {
    pub fn new() -> Self {
        ObjectContainer {
            container: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn add(&mut self, obj: T) {
        self.container.push(obj);
    }

    #[inline(always)]
    pub fn remove_if<F>(&mut self, f: F)
    where
        F: Fn(&T) -> bool,
    {
        self.container.retain(|e| !f(e));
    }

    pub fn len(&self) -> usize {
        self.container.len()
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.container.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        self.container.iter_mut()
    }
}

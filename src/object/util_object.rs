use ggez::graphics as ggraphics;

use torifune::graphics::*;
use torifune::impl_drawable_object_for_wrapped;
use torifune::numeric;

use crate::core::{GameData, TileBatchTextureID};

pub struct FrameData {
    each_cols_size: Vec<f32>,
    each_rows_size: Vec<f32>,
}

impl FrameData {
    pub fn new(each_cols_size: Vec<f32>, each_rows_size: Vec<f32>) -> Self {
        FrameData {
            each_cols_size: each_cols_size,
            each_rows_size: each_rows_size,
        }
    }

    pub fn width(&self) -> f32 {
        self.each_rows_size.iter().fold(0.0, |sum, size| sum + size)
    }

    pub fn height(&self) -> f32 {
        self.each_cols_size.iter().fold(0.0, |sum, size| sum + size)
    }

    pub fn get_row_size_at(&self, index: usize) -> f32 {
        *self.each_rows_size.get(index).unwrap()
    }

    pub fn get_col_size_at(&self, index: usize) -> f32 {
        *self.each_cols_size.get(index).unwrap()
    }
}

pub struct TableFrame {
    tile_batch: TileBatch,
    frame_data: FrameData,
    drwob_essential: DrawableObjectEssential,
    frame_scale: numeric::Vector2f,
}

impl TableFrame {
    pub fn new(
        game_data: &GameData,
        position: numeric::Point2f,
        frame_data: FrameData,
	frame_scale: numeric::Vector2f,
        draw_depth: i8,
    ) -> Self {

	let mut tile_batch = game_data.ref_tile_batch(TileBatchTextureID::OldStyleFrame);
	tile_batch.set_position(position);
	
        let mut table_frame = TableFrame {
            tile_batch: tile_batch,
            frame_data: frame_data,
            drwob_essential: DrawableObjectEssential::new(true, draw_depth),
	    frame_scale: frame_scale,
        };

        table_frame.update_tile_batch();

        table_frame
    }

    fn get_scaled_tile_size(&self) -> numeric::Vector2f {
	let tile_size = self.tile_batch.get_tile_size();
	numeric::Vector2f::new(tile_size.x as f32 * self.frame_scale.x,
			      tile_size.y as f32 * self.frame_scale.y)
    }

    ///
    /// 線の幅を含めてTableFrameの高さを返す
    ///
    fn real_height(&self) -> f32 {
	let tile_size = self.get_scaled_tile_size();
	self.frame_data.height() + (self.frame_data.each_cols_size.len() as f32 * tile_size.y)
    }

    ///
    /// 線の幅を含めてTableFrameの幅を返す
    ///
    fn real_width(&self) -> f32 {
	let tile_size = self.get_scaled_tile_size();
	self.frame_data.width() + (self.frame_data.each_rows_size.len() as f32 * tile_size.x)
    }

    fn tile_per_vline(&self, length: f32) -> usize {
	let tile_size = self.get_scaled_tile_size();
	(length / tile_size.y) as usize
    }

    fn tile_per_hline(&self, length: f32) -> usize {
	let tile_size = self.get_scaled_tile_size();
	(length / tile_size.x) as usize
    }

    ///
    /// あるPointが含まれているグリッドの位置を返す
    ///
    pub fn get_grid_position(&self, point: numeric::Point2f) -> numeric::Vector2u {
	let frame_position = self.get_position();
	let rpoint = numeric::Point2f::new(point.x - frame_position.x, point.y - frame_position.y);
	let mut remain = rpoint;
	let mut grid_position = numeric::Vector2u::new(0, 0);
	let tile_size = self.get_scaled_tile_size();

	for size in &self.frame_data.each_rows_size {
	    remain.x -= size + (tile_size.x as f32 * 1.5);
	    if remain.x < 0.0 {
		break;
	    }
	    grid_position.x += 1;
	}

	for size in &self.frame_data.each_cols_size {
	    remain.y -= size + (tile_size.y as f32 * 1.5);
	    if remain.y < 0.0 {
		break;
	    }
	    grid_position.y += 1;
	}

	grid_position
    }

    ///
    /// あるPointがどの枠に含まれていて、その枠はどこを始点として描画されているかを返す
    ///
    pub fn get_grid_topleft(&self, grid_position: numeric::Vector2u, offset: numeric::Vector2f) -> numeric::Point2f {
	let mut remain_grid_position = grid_position;
	let mut top_left = numeric::Point2f::new(0.0, 0.0);
	let tile_size = self.get_scaled_tile_size();

	for size in &self.frame_data.each_rows_size {
	    top_left.x += tile_size.x as f32;
	    if remain_grid_position.x == 0 {
		break;
	    }
	    top_left.x += size;
	    remain_grid_position.x -= 1;
	}

	for size in &self.frame_data.each_cols_size {
	    top_left.y += tile_size.y as f32;
	    if remain_grid_position.y == 0 {
		break;
	    }
	    top_left.y += size;
	    remain_grid_position.y -= 1;
	}

	top_left + offset
    }

    ///
    /// 垂直方向の線を引くメソッド
    ///
    fn stroke_vline_batch(&mut self, begin: numeric::Point2f) {
	let tile_size = self.get_scaled_tile_size();
	let height = self.real_height();

	let begin = numeric::Point2f::new(begin.x.round(), begin.y.round());
	let mut position = begin;
	println!("{}", self.tile_per_vline(height));
	
	for _ in 2..self.tile_per_vline(height) {
	    position.y += tile_size.y;
            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(3, 0),
                position,
		self.frame_scale,
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );
        }

	self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(3, 1),
	    begin,
	    self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

	self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(3, 2),
	    numeric::Point2f::new(begin.x, begin.y + height - tile_size.y),
	    self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );
    }

    ///
    /// 水平方向の線を引くメソッド
    ///
    fn stroke_hline_batch(&mut self, begin: numeric::Point2f) {
	let tile_size = self.get_scaled_tile_size();
	let width = self.real_width();

	let begin = numeric::Point2f::new(begin.x.round(), begin.y.round());
	let mut position = begin;
	for _ in 2..self.tile_per_hline(width) {
	    position.x += tile_size.x;
            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(4, 0),
                position,
		self.frame_scale,
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );
        }

	self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(4, 1),
	    begin,
	    self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

	self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(4, 2),
	    numeric::Point2f::new(begin.x + width - tile_size.y, begin.y),
	    self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );
    }

    ///
    /// Tile Batchの情報を更新する
    ///
    pub fn update_tile_batch(&mut self) {
        self.tile_batch.clear_batch();

        let tile_size = self.get_scaled_tile_size();
	let width = self.real_width();
	let height = self.real_height();

	println!("width:height = {}:{}", width, height);

	//
	// 水平方向の枠だけ描画
	//
        let mut top_dest_pos = numeric::Point2f::new(tile_size.x.round(),
						     0.0);
        let mut bottom_dest_pos = numeric::Point2f::new(tile_size.x.round(),
							(height - tile_size.x).round());
        for _ in 2..self.tile_per_hline(width) {
            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(1, 0),
                top_dest_pos,
		self.frame_scale,
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );

            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(1, 2),
                bottom_dest_pos,
		self.frame_scale,
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );
            top_dest_pos.x += tile_size.x;
            bottom_dest_pos.x += tile_size.x;
        }

	//
	// 垂直方向の枠だけ描画
	//
        let mut left_dest_pos = numeric::Point2f::new(0.0, tile_size.y);
        let mut right_dest_pos = numeric::Point2f::new(width - tile_size.x,
						       tile_size.y);
        for _ in 2..self.tile_per_vline(height) {
            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(0, 1),
                left_dest_pos,
		self.frame_scale,
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );

            self.tile_batch.add_batch_tile_position(
                numeric::Vector2u::new(2, 1),
                right_dest_pos,
		self.frame_scale,
                ggraphics::Color::from_rgb_u32(0xffffffff),
            );
            left_dest_pos.y += tile_size.y;
            right_dest_pos.y += tile_size.y;
        }

	//
	// 枠の角を描画
	//
        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(0, 0),
            numeric::Point2f::new(0.0, 0.0),
	    self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(2, 0),
            numeric::Point2f::new(width - tile_size.x, 0.0),
	    self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(0, 2),
            numeric::Point2f::new(0.0, height - tile_size.y), 
	    self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

        self.tile_batch.add_batch_tile_position(
            numeric::Vector2u::new(2, 2),
            numeric::Point2f::new(width - tile_size.x as f32,
				  height - tile_size.y as f32),
	    self.frame_scale,
            ggraphics::Color::from_rgb_u32(0xffffffff),
        );

	//
	// 中身のグリッドを描画
	//
	let mut position = numeric::Point2f::new(0.0, 0.0);
	for i in 0..self.frame_data.each_rows_size.len() - 1 {
	    position.x += self.frame_data.each_rows_size.get(i).unwrap() + tile_size.x;
	    self.stroke_vline_batch(position);
	}

	let mut position = numeric::Point2f::new(0.0, 0.0);
	for i in 0..self.frame_data.each_cols_size.len() - 1 {
	    position.y += self.frame_data.each_cols_size.get(i).unwrap() + tile_size.y;
	    self.stroke_hline_batch(position);
	}
    
    }
}

impl DrawableComponent for TableFrame {
    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        if self.is_visible() {
            self.tile_batch.draw(ctx).unwrap()
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

impl DrawableObject for TableFrame {
    impl_drawable_object_for_wrapped! {tile_batch}
}
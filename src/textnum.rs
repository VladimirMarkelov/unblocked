use tetra::graphics::{Color, DrawParams, Rectangle, Texture};
use tetra::math::Vec2;
use tetra::Context;

// A struct that can draw a number digit by digit using a texture with 10 digits
pub struct TextNumber {
    digits: Texture, //texture with 10 digits (0..9)
    digit_w: f32,    // width of a digit (all digits have the same width)
    digit_h: f32,    // height of a digit
}

// Parameters to display a number
#[derive(Clone)]
pub struct TextParams {
    // how many numbers to show (used when displaying right-aligned numbers or numbers with
    // leading zeroes
    width: u8,
    // Show zeroes or spaces when width is bigger than the number of digits in the number
    leading_zeroes: bool,
    // used only if width is set and leading_zeroes is off
    right_align: bool,
    // optional color tint
    color: Option<Color>,
}

impl TextParams {
    pub fn new() -> Self {
        TextParams { width: 0, leading_zeroes: false, right_align: false, color: None }
    }
    pub fn with_width(self, w: u8) -> Self {
        TextParams { width: w, ..self }
    }
    pub fn with_right_align(self) -> Self {
        TextParams { right_align: true, ..self }
    }
    pub fn with_leading_zeroes(self) -> Self {
        TextParams { leading_zeroes: true, ..self }
    }
    pub fn with_color(self, c: Color) -> Self {
        TextParams { color: Some(c), ..self }
    }
}

impl TextNumber {
    pub fn new(ctx: &mut Context, bytes: &[u8]) -> tetra::Result<TextNumber> {
        let mut tx = TextNumber { digits: Texture::from_encoded(ctx, bytes)?, digit_w: 0.0, digit_h: 0.0 };
        tx.digit_w = (tx.digits.width() / 10) as f32;
        tx.digit_h = tx.digits.height() as f32;
        Ok(tx)
    }

    pub fn digit_size(&self) -> Vec2<f32> {
        Vec2::new(self.digit_w, self.digit_h)
    }

    pub fn draw(&mut self, ctx: &mut Context, start_pos: Vec2<f32>, n: u32, param: TextParams) {
        let mut d: Vec<u32> = Vec::new();

        // split a number into its digits
        if n == 0 {
            d.push(0);
        } else {
            let mut n = n;
            while n > 0 {
                let m = n % 10;
                n /= 10;
                d.insert(0, m);
            }
        }

        // add extra zeroes if required
        if param.width != 0 && param.leading_zeroes {
            while d.len() < param.width as usize {
                d.insert(0, 0);
            }
        }

        // fix starting position if the number is right aligned
        let mut p: Vec2<f32> = start_pos;
        if param.width != 0 && param.right_align && d.len() < param.width as usize {
            p = Vec2::new(p.x + ((param.width as usize) - d.len()) as f32 * self.digit_w, p.y);
        }

        // show digits one by one
        for digit in d {
            let clip = Rectangle::new(digit as f32 * self.digit_w, 0.0, self.digit_w, self.digit_h);
            let mut dp = DrawParams::new().position(p);
            if let Some(c) = param.color {
                dp = dp.color(c);
            }
            self.digits.draw_region(ctx, clip, dp);
            p = Vec2::new(p.x + self.digit_w, p.y);
        }
    }
}

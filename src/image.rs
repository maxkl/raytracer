
#[derive(Clone)]
pub struct RgbImage {
    width: usize,
    height: usize,
    data: Vec<u8>,
}

impl RgbImage {
    pub fn new(w: usize, h: usize) -> RgbImage {
        RgbImage {
            width: w,
            height: h,
            data: vec![0; w * h * 3],
        }
    }

    pub fn from_raw(w: usize, h: usize, mut data: Vec<u8>) -> RgbImage {
        data.resize(w * h * 3, 0);
        RgbImage {
            width: w,
            height: h,
            data,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn into_raw(self) -> Vec<u8> {
        self.data
    }

    fn pixel_index(&self, x: usize, y: usize) -> usize {
        (y * self.width + x) * 3
    }

    pub fn put_pixel(&mut self, x: usize, y: usize, color: &(u8, u8, u8)) {
        let index = self.pixel_index(x, y);
        self.data[index] = color.0;
        self.data[index + 1] = color.1;
        self.data[index + 2] = color.2;
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> (u8, u8, u8) {
        let index = self.pixel_index(x, y);
        (
            self.data[index],
            self.data[index + 1],
            self.data[index + 2],
        )
    }
}

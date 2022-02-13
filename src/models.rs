use image::{Pixel, Rgb, RgbImage};

pub fn get_model(m_idx: u16) -> Box<dyn Model> {
    return match m_idx {
        0 => { Box::new(Left {}) }
        1 => { Box::new(Average {}) }
        _ => {
            panic!("unknown model")
        }
    };
}

pub trait Model {
    fn encode(&self, img: &RgbImage) -> Vec<u8>;
    fn decode(&self, w: u32, h: u32, bytes: Vec<u8>) -> RgbImage;
}

pub struct Left {}

impl Model for Left {
    fn encode(&self, img: &RgbImage) -> Vec<u8> {
        let mut prev = Rgb([0u8, 0u8, 0u8]);
        return img.pixels().flat_map(|p| {
            let delta = p.map2(&prev, |it, ot| { it.wrapping_sub(ot) });
            prev = *p;
            return delta.0;
        }).collect();
    }

    fn decode(&self, w: u32, h: u32, bytes: Vec<u8>) -> RgbImage {
        let mut imgbuf = image::ImageBuffer::new(w, h);
        let mut prev = Rgb([0u8, 0u8, 0u8]);

        for (pixel, b) in imgbuf.pixels_mut().zip(bytes.chunks(3)) {
            *pixel = Rgb([
                prev.0[0].wrapping_add(b[0]),
                prev.0[1].wrapping_add(b[1]),
                prev.0[2].wrapping_add(b[2])
            ]);
            prev = *pixel;
        }
        return imgbuf;
    }
}

pub struct Average {}

impl Average {
    fn get_predicted(img: &RgbImage, x: u32, y: u32) -> Rgb<u8> {
        match (x, y) {
            (0, 0) => { //top left corner
                Rgb([0u8, 0u8, 0u8])
            }
            (0, _) => { //left border
                *img.get_pixel(x, y - 1) //above
            }
            (_, 0) => { //top row
                *img.get_pixel(x - 1, y) //left
            }
            (_, _) => {
                let left = img.get_pixel(x - 1, y);
                let up = img.get_pixel(x, y - 1);
                let up_left = img.get_pixel(x - 1, y - 1);
                Rgb([
                    ((left[0] as i16 + up[0] as i16 + up_left[0] as i16) / 3) as u8,
                    ((left[1] as i16 + up[1] as i16 + up_left[1] as i16) / 3) as u8,
                    ((left[2] as i16 + up[2] as i16 + up_left[2] as i16) / 3) as u8,
                ])
            }
        }
    }
}

impl Model for Average {
    fn encode(&self, img: &RgbImage) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        for (x, y, p) in img.enumerate_pixels() {
            let prev = Self::get_predicted(img, x, y);
            let d = p.map2(&prev, |it, ot| { it.wrapping_sub(ot) });
            for b in d.0 {
                bytes.push(b);
            }
        }
        return bytes;
    }

    fn decode(&self, w: u32, h: u32, bytes: Vec<u8>) -> RgbImage {
        let mut img = image::ImageBuffer::new(w, h);
        for ((x, y), delta) in (0..w).zip(0..h).zip(bytes.chunks(3)) {
            let pred = Self::get_predicted(&img, x, y);
            let p = Rgb([
                pred.0[0].wrapping_add(delta[0]),
                pred.0[1].wrapping_add(delta[1]),
                pred.0[2].wrapping_add(delta[2])
            ]);
            img.put_pixel(x, y, p);
        }
        return img;
    }
}




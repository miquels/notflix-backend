use poem::{
    error::NotFoundError, web::StaticFileRequest, Body, FromRequest, IntoResponse, Request,
    ResponseParts, Result,
};
use poem_openapi::payload::{Binary, Response};

use super::Api;
use crate::models;
use crate::util::Id;

#[derive(serde::Deserialize, Debug)]
pub struct ImageOpts {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub quality: Option<u32>,
}

impl ImageOpts {
    fn is_some(&self) -> bool {
        self.width.is_some() || self.height.is_some() || self.quality.is_some()
    }
}

impl Api {
    /// Retrieve image.
    pub async fn get_image(
        &self,
        collection_id: u32,
        mediaitem_id: Id,
        image_id: i64,
        whq: ImageOpts,
        req: &Request,
    ) -> Result<Response<Binary<Body>>> {
        let coll = self.state.config.get_collection(collection_id).ok_or(NotFoundError)?;
        let mi = models::MediaInfo::get(&self.state.db.handle, mediaitem_id)
            .await?
            .ok_or(NotFoundError)?;
        let img = mi.thumbs.iter().find(|i| i.image_id == image_id).ok_or(NotFoundError)?;
        let mut file = format!("{}/{}/{}", coll.directory, mi.directory.path, img.fileinfo.path);

        if whq.is_some() {
            let outfile = "/tmp/test.jpg".to_string();
            resize_image(&file, &outfile, whq).await?;
            file = outfile;
        }

        // Create static file responder.
        let sfr = StaticFileRequest::from_request_without_body(req).await?;
        let poem_resp = sfr.create_response(&file, true)?.into_response();

        Ok(poem_response_to_binary(poem_resp))
    }
}

fn poem_response_to_binary(resp: poem::Response) -> Response<Binary<poem::Body>> {
    let (ResponseParts { status, version, headers, extensions }, body) = resp.into_parts();
    let _ = (version, extensions);
    let mut headers = headers;

    // Transfer body and status.
    let mut oai_resp = Response::new(Binary(body));
    oai_resp = oai_resp.status(status);

    // Transfer headers.
    let mut prev = None;
    for (name, val) in headers.drain() {
        let hname = match name {
            Some(hn) => {
                prev = Some(hn.clone());
                hn
            },
            None => prev.clone().unwrap(),
        };
        oai_resp = oai_resp.header(hname, val);
    }
    oai_resp
}

fn decode_whq(whq: ImageOpts, src_width: u32, src_height: u32) -> (u32, u32, u32) {
    let mut w = src_width;
    let mut h = src_height;
    let q = whq.quality.unwrap_or(100);

    if let Some(width) = whq.width {
        w = width;
        if whq.height.is_none() {
            h = ((src_height as f64 / src_width as f64) * width as f64) as u32;
        }
    }
    if let Some(height) = whq.height {
        h = height;
        if whq.width.is_none() {
            w = ((src_width as f64 / src_height as f64) * height as f64) as u32;
        }
    }

    (w, h, q)
}

#[cfg(not(feature = "with-magick_rust"))]
mod resize {
    use std::fs::File as StdFile;
    use std::io::{BufReader, BufWriter};

    use super::*;
    use ::image::{codecs::jpeg::JpegEncoder, io::Reader as ImageReader};
    use tokio::task;

    pub(super) async fn resize_image(
        infile: &str,
        outfile: &str,
        whq: ImageOpts,
    ) -> anyhow::Result<()> {
        task::block_in_place(move || resize(infile, outfile, whq))
    }

    fn resize(infile: &str, outfile: &str, whq: ImageOpts) -> anyhow::Result<()> {
        let begin = std::time::Instant::now();

        let mut img = {
            #[cfg(feature = "with-zune-jpeg")]
            {
                match zune_decode::decode(&infile) {
                    Some(img) => img,
                    None => {
                        let rdr = BufReader::with_capacity(32768, StdFile::open(&infile)?);
                        ImageReader::new(rdr).with_guessed_format()?.decode()?
                    },
                };
            }
            #[cfg(not(feature = "with-zune-jpeg"))]
            {
                let rdr = BufReader::with_capacity(32768, StdFile::open(&infile)?);
                ImageReader::new(rdr).with_guessed_format()?.decode()?
            }
        };

        let (w, h, q) = decode_whq(whq, img.width(), img.height());
        log::trace!("get_image: new dimensions: w={}, h={}, q={}", w, h, q);
        log::trace!("get_image: time_to_decode: {:?}", begin.elapsed());

        let begin = std::time::Instant::now();

        #[cfg(feature = "with-fast_image_resize")]
        {
            img = fir_resize::resize(img, w, h);
        }
        #[cfg(not(feature = "with-fast_image_resize"))]
        {
            img = img.thumbnail(w, h);
            // This doesn't really do very much.
            // img = img.unsharpen(0.4, 10);
        }
        log::trace!("get_image: time_to_resize: {:?}", begin.elapsed());

        let begin = std::time::Instant::now();
        let output = StdFile::create(&outfile)?;
        let output = BufWriter::with_capacity(32768, output);
        let mut jpeg = JpegEncoder::new_with_quality(output, q as u8);
        jpeg.encode(img.as_bytes(), img.width(), img.height(), img.color())?;
        log::trace!("get_image:: time_to_encode: {:?}", begin.elapsed());

        Ok(())
    }
}
#[cfg(not(feature = "with-magick_rust"))]
use resize::resize_image;

#[cfg(feature = "with-zune-jpeg")]
mod zune_decode {
    use ::image::{DynamicImage, RgbImage};

    // This doesn't quite work yet, the resized image ends up distorted.
    // Happens with both Image::thumbnail and fast_image_resize::Resizer.
    pub(super) fn decode(filename: &str) -> Option<DynamicImage> {
        if !filename.ends_with(".jpg") && !filename.ends_with(".jpeg") {
            return None;
        }
        let opts = zune_jpeg::ZuneJpegOptions::new().set_out_colorspace(zune_jpeg::ColorSpace::RGB);
        let mut zd = zune_jpeg::Decoder::new_with_options(opts);
        let zbuffer = zd.decode_file(&filename).ok()?;
        let rgb = RgbImage::from_raw(zd.width() as u32, zd.height() as u32, zbuffer)?;
        Some(DynamicImage::ImageRgb8(rgb))
    }
}

#[cfg(feature = "with-fast_image_resize")]
mod fir_resize {
    use ::image::{DynamicImage, RgbImage, RgbaImage};
    use fast_image_resize as fr;
    use std::num::NonZeroU32;

    pub(super) fn resize(img: DynamicImage, width: u32, height: u32) -> DynamicImage {
        let (pixel_type, raw) = if img.color().has_alpha() {
            (fr::PixelType::U8x4, img.to_rgba8().into_raw())
        } else {
            (fr::PixelType::U8x3, img.to_rgb8().into_raw())
        };
        let src_width = NonZeroU32::new(img.width()).unwrap();
        let src_height = NonZeroU32::new(img.height()).unwrap();
        let mut src_image = fr::Image::from_vec_u8(src_width, src_height, raw, pixel_type).unwrap();

        // Multiple RGB channels of source image by alpha channel
        // (not required for the Nearest algorithm)
        let alpha_mul_div = fr::MulDiv::default();
        if pixel_type == fr::PixelType::U8x4 {
            alpha_mul_div.multiply_alpha_inplace(&mut src_image.view_mut()).unwrap();
        }

        // Create container for data of destination image
        let dst_width = NonZeroU32::new(width).unwrap();
        let dst_height = NonZeroU32::new(height).unwrap();
        let mut dst_image = fr::Image::new(dst_width, dst_height, src_image.pixel_type());

        // Get mutable view of destination image data
        let mut dst_view = dst_image.view_mut();

        // Create Resizer instance and resize source image
        // into buffer of destination image
        let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Hamming));
        log::trace!("fast_image_resize: CPU: {:?}", resizer.cpu_extensions());
        resizer.resize(&src_image.view(), &mut dst_view).unwrap();

        if pixel_type == fr::PixelType::U8x4 {
            // Divide RGB channels of destination image by alpha
            alpha_mul_div.divide_alpha_inplace(&mut dst_view).unwrap();
            let img = RgbaImage::from_vec(width, height, dst_image.into_vec()).unwrap();
            DynamicImage::ImageRgba8(img)
        } else {
            let img = RgbImage::from_vec(width, height, dst_image.into_vec()).unwrap();
            DynamicImage::ImageRgb8(img)
        }
    }
}

#[cfg(feature = "with-magick_rust")]
mod magick_resize {
    use super::*;
    use magick_rust::{magick_wand_genesis, MagickWand};
    use std::sync::Once;

    static START: Once = Once::new();

    pub(super) async fn resize_image(
        infile: &str,
        outfile: &str,
        whq: ImageOpts,
    ) -> anyhow::Result<()> {
        tokio::task::block_in_place(move || {
            sync_resize(infile, outfile, whq)?;
            Ok::<_, anyhow::Error>(())
        })
    }

    fn sync_resize(infile: &str, outfile: &str, whq: ImageOpts) -> anyhow::Result<()> {
        START.call_once(|| {
            magick_wand_genesis();
        });
        let begin = std::time::Instant::now();
        let mut wand = MagickWand::new();
        wand.read_image(infile)?;
        let src_width = wand.get_image_width() as u32;
        let src_height = wand.get_image_height() as u32;
        let (dst_width, dst_height, q) = decode_whq(whq, src_width, src_height);
        wand.thumbnail_image(dst_width as usize, dst_height as usize);
        if q != 100 {
            let _ = wand.set_image_compression_quality(q as usize);
            let _ = wand.strip_image();
        }
        wand.write_image(outfile)?;
        log::trace!("get_image: new dimensions: w={}, h={}", dst_width, dst_height);
        log::trace!("get_image: magick resize took: {:?}", begin.elapsed());
        Ok(())
    }
}
#[cfg(feature = "with-magick_rust")]
use magick_resize::resize_image;

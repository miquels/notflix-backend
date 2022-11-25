use std::fs::File as StdFile;
use std::io::{BufReader, BufWriter};

use ::image::codecs::jpeg::JpegEncoder;
use ::image::imageops::FilterType;
use ::image::io::Reader as ImageReader;

use poem::{
    error::NotFoundError, web::StaticFileRequest, Body, FromRequest, IntoResponse, Request,
    ResponseParts, Result,
};
use poem_openapi::payload::{Binary, Response};
use tokio::task;

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
            file = task::block_in_place(move || {
                let now = std::time::Instant::now();

                let filename = "/tmp/test.jpg".to_string();
                let rdr = BufReader::with_capacity(32768, StdFile::open(&file)?);

                /* // this doesn't work for now, alas. maybe later?
                let zune = true;
                let mut img = if zune {
                    use ::image::{DynamicImage, RgbaImage};
                    let opts = zune_jpeg::ZuneJpegOptions::new();
                        .set_out_colorspace(zune_jpeg::ColorSpace::RGBA);
                    let mut zd = zune_jpeg::Decoder::new_with_options(opts);
                    let zbuffer = zd.decode_file(&filename)?;
                    let rgba = RgbaImage::from_raw(zd.width() as u32, zd.height() as u32, zbuffer);
                    DynamicImage::ImageRgba8(rgba.unwrap())
                } else {
                    ImageReader::new(rdr).with_guessed_format()?.decode()?
                };
                */
                let mut img = ImageReader::new(rdr).with_guessed_format()?.decode()?;

                log::trace!("get_image: start_to_decode: {:?}", now.elapsed());

                let ImageOpts { width, height, quality } = whq;
                let mut w = 0;
                let mut h = 0;
                let q = quality.unwrap_or(100);

                if let Some(width) = width {
                    w = width;
                    if height.is_none() {
                        h = ((img.height() as f64 / img.width() as f64) * width as f64) as u32;
                    }
                }
                if let Some(height) = height {
                    h = height;
                    if width.is_none() {
                        w = ((img.width() as f64 / img.height() as f64) * height as f64) as u32;
                    }
                }

                log::trace!("get_image: new dimensions: w={}, h={}, q={}", w, h, q);

                if w > 0 && h > 0 {
                    // img = img.resize(w, h, FilterType::Nearest);
                    img = img.resize(w, h, FilterType::Triangle);
                    log::trace!("get_image: time_to_resize: {:?}", now.elapsed());
                }

                let output = StdFile::create(&filename)?;
                let output = BufWriter::with_capacity(32768, output);
                let mut jpeg = JpegEncoder::new_with_quality(output, q as u8);
                jpeg.encode(img.as_bytes(), img.width(), img.height(), img.color())?;
                log::trace!("get_image:: time_to_encode: {:?}", now.elapsed());

                Ok::<_, anyhow::Error>(filename)
            })?;
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

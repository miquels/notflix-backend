use poem_openapi::{
    payload::Binary,
    payload::Response,
};
use poem::{Body, FromRequest, IntoResponse, Request, ResponseParts, Result, error::NotFoundError, web::StaticFileRequest};
use super::Api;
use crate::models;
use crate::util::Id;

impl Api {
    /// Retrieve image.
    pub async fn get_image(&self, req: &Request, collection_id: u32, mediaitem_id: Id, image_id: i64) -> Result<Response<Binary<Body>>> {
        let coll = self.state.config.get_collection(collection_id).ok_or(NotFoundError)?;
        let mi = models::MediaInfo::get(&self.state.db.handle, mediaitem_id).await?.ok_or(NotFoundError)?;
        let img = mi.thumbs.iter().find(|i| i.image_id == image_id).ok_or(NotFoundError)?;
        let file = format!("{}/{}/{}", coll.directory, mi.directory.path, img.fileinfo.path);

        // Create static file responder.
        let sfr = StaticFileRequest::from_request_without_body(req).await?;
        log::trace!("get_image 6");
        let poem_resp = sfr.create_response(&file, true)?.into_response();
        log::trace!("get_image 7");

        Ok(poem_response_to_binary(poem_resp))
    }
}

fn poem_response_to_binary(resp: poem::Response) -> Response<Binary<poem::Body>> {
    let (ResponseParts{ status, version, headers, extensions }, body) = resp.into_parts();
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

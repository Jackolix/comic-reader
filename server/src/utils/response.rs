use rocket::http::{ContentType, Header};
use rocket::response::{self, Response, Responder};
use rocket::Request;
use std::io::Cursor;

pub struct BinaryResponse {
    pub data: Vec<u8>,
    pub content_type: ContentType,
    pub filename: Option<String>,
}

impl<'r> Responder<'r, 'static> for BinaryResponse {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        let mut response = Response::build();
        response.header(self.content_type);

        if let Some(filename) = self.filename {
            response.header(Header::new(
                "Content-Disposition",
                format!("attachment; filename=\"{}\"", filename)
            ));
        }

        response.sized_body(self.data.len(), Cursor::new(self.data));
        Ok(response.finalize())
    }
}
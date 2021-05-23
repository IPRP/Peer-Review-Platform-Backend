use crate::utils::attachment_path;
use rocket::http::ContentType;
use rocket::Data;
use rocket_multipart_form_data::{
    MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use std::fs;

const FILE_LIMIT: u64 = 50 * 1024 * 1024;

#[post("/submission/upload", data = "<data>")]
pub fn upload(content_type: &ContentType, data: Data) -> &'static str {
    // See: https://docs.rs/rocket-multipart-form-data/0.9.6/rocket_multipart_form_data/
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::text("title"),
        MultipartFormDataField::file("file").size_limit(FILE_LIMIT),
    ]);

    let mut multipart_form_data = MultipartFormData::parse(content_type, data, options).unwrap();
    let file = multipart_form_data.files.get("file");
    let title = multipart_form_data.texts.remove("title");

    let backup_title = if let Some(mut text_fields) = title {
        let text_field = text_fields.remove(0);
        text_field.text
    } else {
        String::from("attachment.data")
    };

    if let Some(unwrapped_file) = file {
        let file = &unwrapped_file[0];
        let file_name = if file.file_name.is_some() {
            file.file_name.as_ref().unwrap().clone()
        } else {
            backup_title
        };

        println!("{}", &file_name);
        println!("{}", &file.path.to_str().unwrap());
        let content = fs::read_to_string(&file.path).unwrap();
        println!("{}", content);

        // Copy file to attachments folder
        fs::copy(&file.path, &attachment_path().join(file_name));

        return "Ok";
    }
    "Not Ok"
}

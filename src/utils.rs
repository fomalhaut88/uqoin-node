use actix_web::{HttpResponse, Result as ActixResult, Error as ActixError};

// use crate::appdata::AppData;


pub type APIResult = ActixResult<HttpResponse, ActixError>;
// pub type WebAppData = web::Data<AppData>;

use tokio::io::Result as TkResult;
use actix_web::{web, HttpResponse, Result as ActixResult, Error as ActixError};

use crate::appdata::AppData;


pub type TokioResult<T> = TkResult<T>;
pub type APIResult = ActixResult<HttpResponse, ActixError>;
pub type WebAppData = web::Data<AppData>;

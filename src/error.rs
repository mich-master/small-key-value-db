use actix_web::{ HttpResponse };

pub enum DbError {
    DatabaseError,
    InsertKeyExist,
    UpdateKeyNotExist,
    UpdateValueMatch,
    DeleteKeyNotExist,
    GetKeyNotExist,
}

impl From<DbError> for HttpResponse {
    fn from(err: DbError) -> HttpResponse {
        match err {
            DbError::DatabaseError => HttpResponse::InternalServerError().finish(),
            DbError::InsertKeyExist => HttpResponse::BadRequest().body("Error. INSERT operation: Key already exists.\n"),
            DbError::UpdateKeyNotExist => HttpResponse::BadRequest().body("Error. UPDATE operation: Key does not exist.\n"),
            DbError::UpdateValueMatch => HttpResponse::BadRequest().body("Error. UPDATE operation: Value matches content.\n"),
            DbError::DeleteKeyNotExist => HttpResponse::BadRequest().body("Error. DELETE operation: Key does not exist.\n"),
            DbError::GetKeyNotExist => HttpResponse::BadRequest().body("Error. GET operation: Key does not exist.\n"),
        }
    }
}

use serde::{ Deserialize };
use std::sync::atomic::{ AtomicBool, Ordering };
use std::sync::Arc;
use std::time::Duration;
use tokio::time::delay_for;
use actix_web::{web, App, HttpServer, HttpResponse };

mod error;
mod dbdata;

use dbdata::DbData;

const DB_FILE_NAME: &'static str = "db.json";
const MAX_VALUE_SIZE: usize = 1024*1024;
const TICKER_DURATION: u64 = 1000;
const DUMP_PERIODICITY: u64 = 5000;
const STATISTICS_PERIODICITY: u64 = 60000;

#[derive(Deserialize)]
struct PathInfo {
    key: String,
}

fn db_insert(data: web::Data<DbData>, path: web::Path<PathInfo>, value: String) -> HttpResponse {

    println!("INSERT {} : {}",path.key,value);
    data.insert(path.into_inner().key,value)
        .map_or_else(|db_error| {
            db_error.into()
        },|_| {
            HttpResponse::Ok().finish()
        })
}

fn db_update(data: web::Data<DbData>, path: web::Path<PathInfo>, value: String) -> HttpResponse {

    println!("UPDATE {} : {}",path.key,value);
    data.update(path.into_inner().key,value)
        .map_or_else(|db_error| {
            db_error.into()
        },|_| {
            HttpResponse::Ok().finish()
        })
}

fn db_delete(data: web::Data<DbData>, path: web::Path<PathInfo>) -> HttpResponse {

    println!("DELETE {}",path.key);
    data.delete(path.into_inner().key)
        .map_or_else(|db_error| {
            db_error.into()
        },|_| {
            HttpResponse::Ok().finish()
        })
}

fn db_get(data: web::Data<DbData>, path: web::Path<PathInfo>) -> HttpResponse {

    println!("GET {}",path.key);
    data.get(path.into_inner().key)
        .map_or_else(|db_error| {
            db_error.into()
        },|value| {
            HttpResponse::Ok().body(value)
        })
}

async fn ctrl_c_awaiter(run: Arc<AtomicBool>) {

    tokio::signal::ctrl_c().await.expect("failed to listen for event");
    run.store(false, Ordering::SeqCst);
    // println!("received ctrl-c event");
}

async fn dumper_process(data: web::Data<DbData>) {
    
    let run = Arc::new(AtomicBool::new(true));

    tokio::task::spawn( ctrl_c_awaiter(run.clone()) );

    let mut t_dump: u64 = 0;
    let mut t_statistics: u64 = 0;
    while run.load(Ordering::SeqCst) {
        if t_dump >= DUMP_PERIODICITY {
            data.dump(DB_FILE_NAME).unwrap();
            t_dump = 0;
        } else {
            t_dump += TICKER_DURATION;
        }
        if t_statistics >= STATISTICS_PERIODICITY {
            println!("{}",data.statistics().unwrap());
            t_statistics = 0;
        } else {
            t_statistics += TICKER_DURATION;
        }
        delay_for(Duration::from_millis(TICKER_DURATION)).await;
    };
    data.dump(DB_FILE_NAME).unwrap();
}

#[actix_web::main]
async fn main() {

    let data: web::Data<DbData> = web::Data::new(DbData::create(DB_FILE_NAME));
    let data_servicing = data.clone();

    let dumper =
        tokio::task::spawn( 
            dumper_process(data_servicing)
        );
    
    let server =
        HttpServer::new(move || {
            App::new()
                .app_data(web::PayloadConfig::new(MAX_VALUE_SIZE))
                .app_data(data.clone())
                .route("/{key}", web::post().to(db_insert))
                .route("/{key}", web::put().to(db_update))
                .route("/{key}", web::delete().to(db_delete))
                .route("/{key}", web::get().to(db_get))
        })
        .keep_alive(5)
        .bind("127.0.0.1:1080").unwrap()
        .run();

    let (_,_) = tokio::join!(dumper,server);
}


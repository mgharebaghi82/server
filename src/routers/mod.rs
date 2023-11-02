// use std::fs::{self, File, OpenOptions};
// use std::io::{BufWriter, Write};

use axum::extract::{Multipart, Query};
use axum::http::Method;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{http::StatusCode, Json, Router};
use futures_util::StreamExt;
use mongodb::bson::{doc, from_document, to_document, Document};
use mongodb::{Client, Collection, Database};
use serde::{Deserialize, Serialize};
use tower_http::cors::{AllowHeaders, Any, CorsLayer};

pub fn create_routes() -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers(AllowHeaders::any());
    let app: Router = Router::new()
        .route("/", get(main_page_data))
        .route("/post_data", post(insert_datas))
        .route("/query_param", get(query_params))
        .route("/whitepaper", get(whitepaper))
        // .route("/relays", post(relays))
        .layer(cors);

    app
}

#[derive(Serialize, Deserialize, Debug)]
struct Cards {
    title: String,
    desc: String,
    body: String,
    img: String,
    category: String,
}

async fn website_db() -> Database {
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap();
    let db = client.database("centipedeweb");
    db
}

async fn main_page_data() -> Json<Vec<Cards>> {
    let mut all_data = Vec::new();

    let main_collection: Collection<Document> = website_db().await.collection("main_collection");

    let mut cursor = main_collection.find(None, None).await.unwrap();
    while let Some(result) = cursor.next().await {
        let doc = result.unwrap();
        let data: Cards = from_document(doc).unwrap();
        all_data.push(data);
    }

    Json(all_data)
}

async fn insert_datas(mut multipart: Multipart) -> impl IntoResponse {
    let mut card = Cards {
        title: String::new(),
        desc: String::new(),
        body: String::new(),
        img: String::new(),
        category: String::new(),
    };

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();

        match name.as_str() {
            "title" => {
                card.title = String::from_utf8(field.bytes().await.unwrap().clone().into()).unwrap()
            }
            "desc" => {
                card.desc = String::from_utf8(field.bytes().await.unwrap().clone().into()).unwrap()
            }
            "body" => {
                card.body = String::from_utf8(field.bytes().await.unwrap().clone().into()).unwrap()
            }
            "img" => {
                let file_name = field.file_name().unwrap().to_string();
                let path =
                    std::path::PathBuf::from("/home/client/build/static/media").join(file_name);
                card.img = format!("/static/media/{}", field.file_name().unwrap());
                std::fs::write(path, field.bytes().await.unwrap().clone()).unwrap();
            }
            "category" => {
                card.category =
                    String::from_utf8(field.bytes().await.unwrap().clone().into()).unwrap()
            }
            _ => {}
        }
    }

    let card_todoc = to_document(&card).unwrap();
    let coll: Collection<Document> = website_db().await.collection("main_collection");
    coll.insert_one(card_todoc, None).await.unwrap();

    (StatusCode::OK, "recieved".to_string())
}

#[derive(Serialize, Deserialize)]
struct QueryParams {
    message: String,
}

async fn query_params(Query(query): Query<QueryParams>) -> Json<Cards> {
    let main_coll: Collection<Document> = website_db().await.collection("main_collection");
    let filter = doc! {"title": query.message};
    let find_doc = main_coll.find_one(filter, None).await.unwrap();
    let card: Cards = from_document(find_doc.unwrap()).unwrap();
    Json(card)
}

async fn whitepaper() -> Json<Vec<Cards>> {
    let mut papers: Vec<Cards> = Vec::new();
    let main_coll: Collection<Document> = website_db().await.collection("main_collection");
    let filter = doc! {"category": "whitepaper".to_string()};
    let mut cursor = main_coll.find(filter, None).await.unwrap();
    while let Some(doc) = cursor.next().await {
        let paper: Cards = from_document(doc.unwrap()).unwrap();
        papers.push(paper);
    }

    Json(papers)
}

// async fn relays(body: String) -> String {
//     let relays_exist = fs::metadata("relays.dat").is_ok();
//     if relays_exist {
//         let file = OpenOptions::new()
//             .write(true)
//             .append(true)
//             .open("relays.dat")
//             .unwrap();
//         let mut writer = BufWriter::new(file);
//         writeln!(writer, "{}", body).unwrap();
//     } else {
//         File::create("relays.dat").unwrap();
//         let file = OpenOptions::new()
//             .write(true)
//             .append(true)
//             .open("relays.dat")
//             .unwrap();
//         let mut writer = BufWriter::new(file);
//         writeln!(writer, "{}", body).unwrap();
//     }
//     body
// }

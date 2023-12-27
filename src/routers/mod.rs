use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};

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
        .route("/relays", post(post_relays))
        .route("/relays", get(get_relays))
        .route("/rpc", post(rpc_address))
        .route("/rpc", get(get_rpc_addresses))
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

#[derive(Debug, Serialize, Deserialize)]
struct Addresses {
    addr: Vec<String>,
}

async fn post_relays(body: String) -> String {
    let mut response = Addresses { addr: Vec::new() };
    let relays_exist = fs::metadata("/home/Downloads/relays.dat").is_ok();

    if relays_exist {
        let relays_file = File::open("/home/Downloads/relays.dat").unwrap();
        let reader = BufReader::new(relays_file);
        let mut exist_adrr = Vec::new();
        for line in reader.lines() {
            let addr = line.unwrap();
            response.addr.push(addr.clone());
            exist_adrr.push(addr.clone());
        }

        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .open("/home/Downloads/relays.dat")
            .unwrap();
        let mut writer = BufWriter::new(file);
        if !exist_adrr.contains(&body) {
            writeln!(writer, "{}", body).unwrap();
        }
    } else {
        File::create("/home/Downloads/relays.dat").unwrap();
        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .open("/home/Downloads/relays.dat")
            .unwrap();
        let mut writer = BufWriter::new(file);
        writeln!(writer, "{}", body).unwrap();

        response.addr.push(body);
    }

    let str_res = serde_json::to_string(&response).unwrap();
    str_res
}

async fn get_relays() -> String {
    let mut response = Addresses { addr: Vec::new() };
    let relays_exist = fs::metadata("/home/Downloads/relays.dat").is_ok();

    if relays_exist {
        let relays_file = File::open("/home/Downloads/relays.dat").unwrap();
        let reader = BufReader::new(relays_file);
        for line in reader.lines() {
            let addr = line.unwrap();
            response.addr.push(addr);
        }
    }

    let str_addresses = serde_json::to_string(&response).unwrap();
    str_addresses
}

async fn rpc_address(address: String) -> String {
    let path = "/home/Downloads/rpsees.dat";
    let rpc_file_exist = fs::metadata(path).is_ok();
    if rpc_file_exist {
        let rpc_file = File::open(path).unwrap();
        let reader = BufReader::new(rpc_file);
        let mut addresses = Vec::new();
        for addr in reader.lines() {
            let exist_addr = addr.unwrap();
            addresses.push(exist_addr);
        }

        if !addresses.contains(&address) {
            let w_rpc_file = OpenOptions::new().append(true).write(true).open(path).unwrap();
            let mut writer = BufWriter::new(w_rpc_file);
            writeln!(writer, "{}", address).unwrap();
        }
    } else {
        let rpc_file = OpenOptions::new().append(true).write(true).open(path).unwrap();
        let mut writer = BufWriter::new(rpc_file);
        writeln!(writer, "{}", address).unwrap();
    }
    "".to_string()
}

async fn get_rpc_addresses() -> Json<Vec<String>> {
    let path = "/home/Downloads/rpsees.dat";
    let addresses_file = File::open(path).unwrap();
    let mut addresses = Vec::new();
    let reader = BufReader::new(addresses_file);

    for i in reader.lines() {
        let addr = i.unwrap();
        addresses.push(addr);
    }

    Json(addresses)
}

// use tokio_tungstenite::tungstenite::Message;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::str::FromStr;
use std::time::Duration;

use axum::extract::{
    self,
    ws::{WebSocket, WebSocketUpgrade},
    Query,
};
use axum::http::Method;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{http::StatusCode, Json, Router};
use futures_util::StreamExt;
use mongodb::bson::{doc, from_document, to_document, Document};
use mongodb::options::FindOptions;
use mongodb::{Client, Collection, Database};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tower_http::cors::{AllowHeaders, Any, CorsLayer};
mod structures;
use structures::Block;

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
        .route("/rmaddr", post(rm_address))
        .route("/rmrpc", post(rm_rpc))
        .route("/apis", get(handle_apis))
        .route("/blockchain", get(handle_blockchain))
        .route("/blockchain_pages", post(handle_blockchain_pages))
        .route("/remaining_coins", get(remaining_centis))
        .route("/wsutxo", get(|ws| ws_handler(ws)))
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

async fn blockchain_db() -> Database {
    let client = Client::with_uri_str("mongodb://localhost:27017")
        .await
        .unwrap();
    let db = client.database("Blockchain");
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

async fn insert_datas(extract::Json(data): extract::Json<Cards>) -> impl IntoResponse {
    let card_todoc = to_document(&data).unwrap();
    let coll: Collection<Document> = website_db().await.collection("main_collection");
    match coll.insert_one(card_todoc, None).await {
        Ok(_) => (StatusCode::OK, "recieved".to_string()),
        Err(_) => (StatusCode::NOT_ACCEPTABLE, "Error".to_string()),
    }
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

async fn get_relays() -> Json<Addresses> {
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

    Json(response)
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
            let w_rpc_file = OpenOptions::new()
                .append(true)
                .write(true)
                .open(path)
                .unwrap();
            let mut writer = BufWriter::new(w_rpc_file);
            writeln!(writer, "{}", address).unwrap();
        }
    } else {
        let rpc_file = OpenOptions::new()
            .append(true)
            .write(true)
            .open(path)
            .unwrap();
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

async fn rm_address(addr: String) -> String {
    let relays_file = File::open("/home/Downloads/relays.dat").unwrap();
    let reader = BufReader::new(relays_file);
    let mut relays = Vec::new();
    for i in reader.lines() {
        let addr = i.unwrap();
        relays.push(addr);
    }

    let index = relays.iter().position(|address| address.clone() == addr);
    match index {
        Some(i) => {
            relays.remove(i);
            let mut writer_file = File::create("/home/Downloads/relays.dat").unwrap();
            for addr in relays {
                writeln!(writer_file, "{}", addr).unwrap();
            }
        }
        None => {}
    }

    "removed".to_string()
}

async fn rm_rpc(addr: String) -> String {
    let relays_file = File::open("/home/Downloads/rpsees.dat").unwrap();
    let reader = BufReader::new(relays_file);
    let mut all_rpcs = Vec::new();
    for i in reader.lines() {
        let addr = i.unwrap();
        all_rpcs.push(addr);
    }

    let index = all_rpcs.iter().position(|address| address.contains(&addr));
    match index {
        Some(i) => {
            all_rpcs.remove(i);
            let mut writer_file = File::create("/home/Downloads/rpsees.dat").unwrap();
            for addr in all_rpcs {
                writeln!(writer_file, "{}", addr).unwrap();
            }
        }
        None => {}
    }

    "removed".to_string()
}

async fn handle_apis() -> Json<Vec<Cards>> {
    let mut all_apis_doc = Vec::new();
    let apis_coll: Collection<Document> = website_db().await.collection("main_collection");
    let filter = doc! {"category": "api".to_string()};
    let mut cursor = apis_coll.find(filter, None).await.unwrap();
    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(data) => {
                let api: Cards = from_document(data).unwrap();
                all_apis_doc.push(api)
            }
            Err(_) => break,
        }
    }

    Json(all_apis_doc)
}

#[derive(Debug, Serialize, Deserialize)]
struct Pagination {
    current_page: i64,
    page_size: i64,
}

//get all blokchain from blockchain database in mongodb and sent it to client website as json response
async fn handle_blockchain_pages(extract::Json(data): extract::Json<Pagination>) -> Json<Vec<Block>> {
    let mut blocks = Vec::new();
    let blocks_coll: Collection<Document> = blockchain_db().await.collection("Blocks");
    let filter = doc! {"header.number": {"$gte": (data.current_page * data.page_size) - 49, "$lte": data.current_page * data.page_size}};
    let mut cursor = blocks_coll.find(filter, None).await.unwrap();
    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(data) => {
                let block: Block = from_document(data).unwrap();
                blocks.push(block)
            }
            Err(_) => break,
        }
    }

    Json(blocks)
}

async fn handle_blockchain() -> Json<Vec<Block>> {
    let mut blocks = Vec::new();
    let blocks_coll:Collection<Document> = blockchain_db().await.collection("Blocks");
    let sort = doc! {"header.number": -1};
    let option = FindOptions::builder().sort(sort).limit(50).build();
    let mut cursor = blocks_coll.find(None, option).await.unwrap();
    while let Some(result) = cursor.next().await  {
        if let Ok(doc) = result {
            let block:Block = from_document(doc).unwrap();
            blocks.push(block);
        }
    } 

    Json(blocks)
}

async fn remaining_centis() -> String {
    let blocks_coll: Collection<Document> = blockchain_db().await.collection("Blocks");
    let mut cursor = blocks_coll.find(None, None).await.unwrap();
    let mut generated_centis = Decimal::from_str("0.0").unwrap();
    let all_centies = Decimal::from_str("21000000.0").unwrap();
    while let Some(doc) = cursor.next().await {
        match doc {
            Ok(document) => {
                let block: Block = from_document(document).unwrap();
                generated_centis += block.body.coinbase.coinbase_data.reward.round_dp(12);
            }
            Err(_) => break,
        }
    }
    (all_centies - generated_centis.round_dp(12)).to_string()
}

#[derive(Debug, Serialize, Deserialize)]
struct Centies {
    _id: String,
}

async fn ws_utxo(mut socket: WebSocket) {
    // let stream = UnboundedReceiverStream::new(rx);
    let blockchain_coll: Collection<Document> = blockchain_db().await.collection("Blocks");
    let mut watching = blockchain_coll.watch(None, None).await.unwrap();

    tokio::spawn(async move {
        loop {
            if let Some(change_stream) = watching.next().await {
                let mut base_reward = Decimal::from_str("50.0").unwrap().round_dp(12);
                let mut reamining_centies = Decimal::from_str("21000000.0").unwrap();
                match change_stream {
                    Ok(change) => {
                        let document = change.full_document.unwrap();
                        let block: Block = from_document(document).unwrap();
                        let block_number = block.header.number;
                        for i in 1..block_number {
                            if i.abs() % 150000 == 0 {
                                base_reward = base_reward / Decimal::from_str("2.0").unwrap();
                                reamining_centies -= base_reward;
                            } else {
                                reamining_centies -= base_reward;
                            }
                        }
                        match socket
                            .send(extract::ws::Message::Text(reamining_centies.to_string()))
                            .await
                        {
                            Ok(_) => tokio::time::sleep(Duration::from_secs(2)).await,
                            Err(_) => {
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        println!("error from change stream");
                        break; // Receiver has closed, exit the loop
                    }
                }
            }
        }
    });
}

async fn ws_handler(webs: WebSocketUpgrade) -> impl IntoResponse {
    webs.on_upgrade(|socket| ws_utxo(socket))
}

use axum::Router;
use routers::create_routes;

mod routers;

pub async fn run() {
    let app: Router = create_routes();

    axum::Server::bind(&"127.0.0.1:3002".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
    println!("start serevr...");
}

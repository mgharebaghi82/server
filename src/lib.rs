use axum::Router;
use routers::create_routes;

mod routers;

pub async fn run() {
    let app: Router = create_routes();

    axum::Server::bind(&"0.0.0.0:3002".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

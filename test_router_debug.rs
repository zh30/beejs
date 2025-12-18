use beejs::edge::global_router::{GlobalRouter, RouteResult};

#[tokio::main]
async fn main() {
    let router = GlobalRouter::new();
    
    match router.route_request(37.7749, -122.4194).await {
        Ok(route) => {
            println!("Success! Route: {:?}", route);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

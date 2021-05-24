use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::RwLock;
use std::task::Poll;

use hyper::server::conn::Http;
use hyper::service::Service;
use hyper::Method;
use hyper::{Body, Request, Response};

use tokio::net::TcpListener;
use handlebars::Handlebars;
use serde::{Serialize};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {



    let addr: SocketAddr = ([127, 0, 0, 1], 8080).into();
    let tcp_listener = TcpListener::bind(addr).await?;

    let mut reg = Handlebars::<'static>::new();
    reg.register_template_file("index.html", "./pages/index.html")?;
    
    let render = Render {
        reg: Arc::new(RwLock::new(reg)),
        //rhai: Arc::new(RwLock::new(Engine::new_raw()))
    };

    loop {
        let (tcp_stream, _) = tcp_listener.accept().await?;
        let rnd = render.clone();
        tokio::spawn(async move {
            if let Err(http_err) = Http::new()
                .http1_only(true)
                .http1_keep_alive(true)
                .serve_connection(tcp_stream, rnd)
                .await
            {
                eprintln!("Error while serving HTTP connection: {}", http_err);
            }
        });
    }
}

#[derive(Serialize)]
struct User {
    name: String
}

#[derive(Clone)]
struct Render {
    reg: Arc<RwLock<Handlebars<'static>>>,
    //rhai: Arc<RwLock<Engine>>
}

impl Service<Request<Body>> for Render {
    type Response = Response<Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let res = Ok::<Self::Response, Self::Error>(match (req.method(), req.uri().path()) {
            (&Method::GET, "/") => {
                let reg = self.reg.read().unwrap();
                let user = User{name: "Matvei".into() };
                let html = reg.render("index.html", &user).unwrap();
                Response::builder().body(Body::from(html)).unwrap()
            }

            _ => Response::builder().body(Body::from("sdasdsd")).unwrap(),
        });
        Box::pin(async { res })
    }
}

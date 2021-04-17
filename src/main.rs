use anyhow::Result;
use async_std::{
    fs,
    net::{TcpListener, TcpStream},
    path::PathBuf,
    prelude::*,
};
use futures::StreamExt;

#[async_std::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("localhost:8080").await?;
    listener
        .incoming()
        .for_each_concurrent(None, |stream| async {
            let stream = stream.unwrap();
            handle_connection(stream).await.unwrap();
        })
        .await;
    Ok(())
}

async fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).await?;

    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    let res = req.parse(&buffer)?;
    if res.is_complete() && req.method.unwrap() == "GET" {
        let mut path = req.path.unwrap();
        eprintln!("GET {}", path);

        if path == "/" {
            path = "/index.html";
        }
        let path = PathBuf::from(
            std::env::current_dir()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                + path,
        );

        let response = if path.is_file().await {
            let contents = fs::read_to_string(path).await?;
            format!("HTTP/1.1 200 OK\r\n\r\n{}", contents)
        } else {
            "HTTP/1.1 404 NOT FOUND\r\n\r\nNot found".to_string()
        };

        stream.write(response.as_bytes()).await?;
        stream.flush().await?;
    }

    Ok(())
}
